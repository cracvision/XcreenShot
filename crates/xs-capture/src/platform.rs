//! Small OS queries used around a capture: pointer position, foreground app and
//! process details for the capture metadata (CAP-047, CAP-056).

use serde::Serialize;

/// Application that owns a window (CAP-056 "Aplicación").
#[derive(Debug, Clone, Default, Serialize)]
pub struct AppInfo {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Executable path on Windows; bundle identifier on macOS.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OsInfo {
    pub name: String,
    pub version: String,
    pub build: String,
}

/// The name to show for an app: its description, unless it is a file name or a path
/// (packaged apps such as Notepad say "Notepad.exe" or give their full path), then the
/// name without folder or ".exe".
pub fn app_name(description: Option<&str>, fallback: &str, path: Option<&str>) -> String {
    let file_like = |s: &str| s.contains(['\\', '/']) || s.to_ascii_lowercase().ends_with(".exe");
    let stem = |s: &str| {
        let base = s.rsplit(['\\', '/']).next().unwrap_or(s);
        match base.len().checked_sub(4) {
            Some(cut) if base.to_ascii_lowercase().ends_with(".exe") => base[..cut].to_string(),
            _ => base.to_string(),
        }
    };
    for candidate in [description.unwrap_or(""), fallback] {
        let candidate = candidate.trim();
        if candidate.is_empty() {
            continue;
        }
        return if file_like(candidate) {
            stem(candidate)
        } else {
            candidate.to_string()
        };
    }
    path.map(stem).unwrap_or_default()
}

#[cfg(test)]
mod name_tests {
    use super::app_name;

    #[test]
    fn packaged_apps_get_a_clean_name() {
        let path = r"C:\Program Files\WindowsApps\Microsoft.WindowsNotepad_11.2609.11.0_x64__8wekyb3d8bbwe\Notepad\Notepad.exe";
        // Notepad describes itself as its file name.
        assert_eq!(app_name(Some("Notepad.exe"), "", Some(path)), "Notepad");
        // Or gives its full path as the window's app name.
        assert_eq!(app_name(None, path, Some(path)), "Notepad");
        assert_eq!(app_name(Some("Google Chrome"), "chrome", None), "Google Chrome");
        assert_eq!(app_name(Some("  "), "Paint", None), "Paint");
        assert_eq!(app_name(None, "", Some(r"C:\Tools\thing.EXE")), "thing");
        assert_eq!(app_name(None, "", None), "");
    }
}

#[cfg(windows)]
mod imp {
    use super::*;
    use windows_sys::Win32::Foundation::{CloseHandle, POINT};
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW, VS_FIXEDFILEINFO,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetCursorPos, GetForegroundWindow, GetWindowLongPtrW, GetWindowThreadProcessId, GWL_EXSTYLE,
        WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
    };

    /// A real application window, not a click-through overlay (game and GPU overlays,
    /// screen-sharing frames) or a tool window (CAP-016, CAP-056).
    pub fn is_regular_window(id: u32) -> bool {
        // SAFETY: reading the style of a window handle; an invalid handle returns 0.
        let ex = unsafe { GetWindowLongPtrW(id as usize as _, GWL_EXSTYLE) } as u32;
        ex & (WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW) == 0
    }

    pub fn cursor_position() -> Option<(i32, i32)> {
        let mut p = POINT { x: 0, y: 0 };
        // SAFETY: out-parameter call.
        (unsafe { GetCursorPos(&mut p) } != 0).then_some((p.x, p.y))
    }

    /// A window hidden by the system: on another virtual desktop, or a suspended app
    /// (CAP-030).
    pub fn is_cloaked(id: u32) -> bool {
        use windows_sys::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
        let mut cloaked: u32 = 0;
        // SAFETY: out-parameter of the right size; an invalid handle fails.
        let ok = unsafe {
            DwmGetWindowAttribute(
                id as usize as _,
                DWMWA_CLOAKED as u32,
                &mut cloaked as *mut u32 as *mut _,
                std::mem::size_of::<u32>() as u32,
            )
        } == 0;
        ok && cloaked != 0
    }

    /// A window the task switcher would show: not owned by another one (a popup or a
    /// helper of an app), unless it asks to be (CAP-030).
    pub fn is_app_window(id: u32) -> bool {
        use windows_sys::Win32::UI::WindowsAndMessaging::{GetWindow, GW_OWNER, WS_EX_APPWINDOW};
        let hwnd = id as usize as _;
        // SAFETY: reading a window's owner and style; an invalid handle gives null and 0.
        let (owner, ex) = unsafe {
            (
                GetWindow(hwnd, GW_OWNER),
                GetWindowLongPtrW(hwnd, GWL_EXSTYLE) as u32,
            )
        };
        owner.is_null() || ex & WS_EX_APPWINDOW != 0
    }

    /// Restores a minimized window without activating it, or minimizes it again
    /// (CAP-030).
    /// The session is locked (or on the secure desktop): nothing to capture
    /// (CAP-049, CAP-050).
    pub fn session_locked() -> bool {
        use windows_sys::Win32::System::StationsAndDesktops::{
            CloseDesktop, OpenInputDesktop, DESKTOP_READOBJECTS,
        };
        // SAFETY: opens and closes a handle to the input desktop; it fails while the
        // lock screen has it.
        unsafe {
            let desk = OpenInputDesktop(0, 0, DESKTOP_READOBJECTS);
            if desk.is_null() {
                return true;
            }
            CloseDesktop(desk);
        }
        false
    }

    pub fn show_window(id: u32, show: bool) {
        use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_MINIMIZE, SW_SHOWNOACTIVATE};
        // SAFETY: a window handle; an invalid one does nothing.
        unsafe {
            ShowWindow(
                id as usize as _,
                if show { SW_SHOWNOACTIVATE } else { SW_MINIMIZE },
            )
        };
    }

    /// The process that put the clipboard's content there (CAP-039), if the system
    /// knows it.
    pub fn clipboard_owner_pid() -> Option<u32> {
        use windows_sys::Win32::System::DataExchange::GetClipboardOwner;
        // SAFETY: plain calls; no owner or a closed window yields nothing.
        unsafe {
            let hwnd = GetClipboardOwner();
            if hwnd.is_null() {
                return None;
            }
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, &mut pid);
            (pid != 0).then_some(pid)
        }
    }

    pub fn foreground_pid() -> Option<u32> {
        // SAFETY: plain calls; a null window yields pid 0.
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                return None;
            }
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, &mut pid);
            (pid != 0).then_some(pid)
        }
    }

    fn exe_path(pid: u32) -> Option<String> {
        // SAFETY: the handle is checked and closed; the buffer outlives the call.
        unsafe {
            let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if h.is_null() {
                return None;
            }
            let mut buf = vec![0u16; 1024];
            let mut len = buf.len() as u32;
            let ok = QueryFullProcessImageNameW(h, PROCESS_NAME_WIN32, buf.as_mut_ptr(), &mut len);
            CloseHandle(h);
            (ok != 0).then(|| String::from_utf16_lossy(&buf[..len as usize]))
        }
    }

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(Some(0)).collect()
    }

    /// File version and description from the executable's version resource.
    fn version_info(path: &str) -> (Option<String>, Option<String>) {
        let wpath = wide(path);
        // SAFETY: buffers are sized by the API and outlive every pointer derived from them.
        unsafe {
            let size = GetFileVersionInfoSizeW(wpath.as_ptr(), std::ptr::null_mut());
            if size == 0 {
                return (None, None);
            }
            let mut data = vec![0u8; size as usize];
            if GetFileVersionInfoW(wpath.as_ptr(), 0, size, data.as_mut_ptr().cast()) == 0 {
                return (None, None);
            }
            let mut ptr: *mut core::ffi::c_void = std::ptr::null_mut();
            let mut len = 0u32;
            let version = (VerQueryValueW(data.as_ptr().cast(), wide("\\").as_ptr(), &mut ptr, &mut len)
                != 0
                && !ptr.is_null())
            .then(|| {
                let info = &*(ptr as *const VS_FIXEDFILEINFO);
                format!(
                    "{}.{}.{}.{}",
                    info.dwFileVersionMS >> 16,
                    info.dwFileVersionMS & 0xffff,
                    info.dwFileVersionLS >> 16,
                    info.dwFileVersionLS & 0xffff
                )
            });
            // First translation, then its FileDescription.
            let mut description = None;
            if VerQueryValueW(
                data.as_ptr().cast(),
                wide("\\VarFileInfo\\Translation").as_ptr(),
                &mut ptr,
                &mut len,
            ) != 0
                && len >= 4
            {
                let lang = *(ptr as *const u16);
                let cp = *(ptr as *const u16).add(1);
                let key = format!("\\StringFileInfo\\{lang:04x}{cp:04x}\\FileDescription");
                if VerQueryValueW(data.as_ptr().cast(), wide(&key).as_ptr(), &mut ptr, &mut len) != 0
                    && len > 1
                {
                    let s = std::slice::from_raw_parts(ptr as *const u16, len as usize - 1);
                    // Some files report the length in bytes: stop at the first NUL.
                    let end = s.iter().position(|&c| c == 0).unwrap_or(s.len());
                    let text = String::from_utf16_lossy(&s[..end]).trim().to_string();
                    if !text.is_empty() {
                        description = Some(text);
                    }
                }
            }
            (version, description)
        }
    }

    pub fn app_info(pid: u32, fallback_name: &str) -> AppInfo {
        let path = exe_path(pid);
        let (version, description) = path.as_deref().map(version_info).unwrap_or((None, None));
        let name = super::app_name(description.as_deref(), fallback_name, path.as_deref());
        AppInfo {
            name,
            version,
            id: path,
        }
    }

    pub fn os_info() -> OsInfo {
        let build = xs_core::core::windows_build().unwrap_or(0);
        let display = xs_core::winreg::read_string(
            xs_core::winreg::HKLM,
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
            "DisplayVersion",
        )
        .unwrap_or_default();
        OsInfo {
            name: if build >= 22000 {
                "Windows 11"
            } else {
                "Windows 10"
            }
            .to_string(),
            version: display,
            build: build.to_string(),
        }
    }

    pub fn user_and_machine() -> (String, String) {
        (
            std::env::var("USERNAME").unwrap_or_default(),
            std::env::var("COMPUTERNAME").unwrap_or_default(),
        )
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use super::*;

    #[repr(C)]
    struct CGPoint {
        x: f64,
        y: f64,
    }

    #[link(name = "ApplicationServices", kind = "framework")]
    extern "C" {
        fn CGEventCreate(source: *const core::ffi::c_void) -> *mut core::ffi::c_void;
        fn CGEventGetLocation(event: *mut core::ffi::c_void) -> CGPoint;
        fn CFRelease(cf: *const core::ffi::c_void);
    }

    pub fn cursor_position() -> Option<(i32, i32)> {
        // SAFETY: the event is created, read and released here.
        unsafe {
            let event = CGEventCreate(std::ptr::null());
            if event.is_null() {
                return None;
            }
            let p = CGEventGetLocation(event);
            CFRelease(event);
            Some((p.x as i32, p.y as i32))
        }
    }

    pub fn is_regular_window(_id: u32) -> bool {
        true
    }

    pub fn is_cloaked(_id: u32) -> bool {
        false
    }

    pub fn is_app_window(_id: u32) -> bool {
        true
    }

    pub fn show_window(_id: u32, _show: bool) {}

    pub fn session_locked() -> bool {
        false
    }

    pub fn foreground_pid() -> Option<u32> {
        // Frontmost-app lookup arrives with the macOS capture work (Windows first).
        None
    }

    pub fn clipboard_owner_pid() -> Option<u32> {
        // The pasteboard does not say who wrote it.
        None
    }

    pub fn app_info(_pid: u32, fallback_name: &str) -> AppInfo {
        AppInfo {
            name: fallback_name.to_string(),
            version: None,
            id: None,
        }
    }

    fn sw_vers(arg: &str) -> String {
        std::process::Command::new("sw_vers")
            .arg(arg)
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default()
    }

    pub fn os_info() -> OsInfo {
        OsInfo {
            name: "macOS".to_string(),
            version: sw_vers("-productVersion"),
            build: sw_vers("-buildVersion"),
        }
    }

    pub fn user_and_machine() -> (String, String) {
        let mut buf = [0u8; 256];
        // SAFETY: buffer is valid for its length.
        let host = if unsafe { libc::gethostname(buf.as_mut_ptr().cast(), buf.len()) } == 0 {
            let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
            String::from_utf8_lossy(&buf[..end]).into_owned()
        } else {
            String::new()
        };
        (std::env::var("USER").unwrap_or_default(), host)
    }
}

pub use imp::{
    app_info, clipboard_owner_pid, cursor_position, foreground_pid, is_app_window, is_cloaked,
    is_regular_window, os_info, session_locked, show_window, user_and_machine,
};

/// The app that copied what is on the clipboard (CAP-039), if the system says.
pub fn clipboard_app() -> Option<AppInfo> {
    clipboard_owner_pid()
        .map(|pid| app_info(pid, ""))
        .filter(|a| !a.name.is_empty())
}
