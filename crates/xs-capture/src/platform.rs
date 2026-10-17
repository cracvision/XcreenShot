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
        let name = description.unwrap_or_else(|| {
            if fallback_name.is_empty() {
                path.as_deref()
                    .and_then(|p| std::path::Path::new(p).file_stem())
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default()
            } else {
                fallback_name.to_string()
            }
        });
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

    pub fn foreground_pid() -> Option<u32> {
        // Frontmost-app lookup arrives with the macOS capture work (Windows first).
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

pub use imp::{app_info, cursor_position, foreground_pid, is_regular_window, os_info, user_and_machine};
