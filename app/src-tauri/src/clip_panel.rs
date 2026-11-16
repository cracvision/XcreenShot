//! XcreenShot's clipboard (TRY-018): the panel with the recent tray's captures, and the
//! paste into the app that was in front when it opened. Pasting is one simulated
//! Ctrl+V: no hook, nothing that keeps the mouse or the keyboard.

use std::sync::Mutex;

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

use crate::windows::theme;

/// The panel's window label.
pub const CLIPBOARD: &str = "clipboard";

/// The window that was in front when the panel opened: where an insert pastes.
static TARGET: Mutex<isize> = Mutex::new(0);

/// Opens the panel (`Some(true)`), closes it (`Some(false)`) or the other way (`None`);
/// whether it is open now.
pub fn panel(app: &AppHandle, open: Option<bool>) -> tauri::Result<bool> {
    let shown = app.get_webview_window(CLIPBOARD);
    let want = open.unwrap_or(shown.is_none());
    if !want {
        if let Some(w) = shown {
            w.destroy()?;
        }
        return Ok(false);
    }
    if let Some(w) = shown {
        let _ = w.set_focus();
        return Ok(true);
    }
    remember_target();
    let lang = app.state::<crate::AppState>().core.lang();
    let w = WebviewWindowBuilder::new(app, CLIPBOARD, WebviewUrl::App("index.html".into()))
        .title(xs_core::i18n::tr(lang, "ui.clipboard.title", &[]))
        .decorations(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        // Shows in screenshots, like the system's own clipboard: a tester can keep it
        // as evidence (TRY-018).
        .theme(theme(app))
        .build()?;
    // Centered on the work area of the monitor under the pointer.
    let pointer = app.cursor_position().ok();
    let monitor = pointer
        .and_then(|p| app.monitor_from_point(p.x, p.y).ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten());
    if let Some(m) = monitor {
        let scale = m.scale_factor();
        let area = m.work_area();
        let (w_px, h_px) = ((760.0 * scale) as u32, (500.0 * scale) as u32);
        let (w_px, h_px) = (w_px.min(area.size.width), h_px.min(area.size.height));
        w.set_size(tauri::PhysicalSize::new(w_px, h_px))?;
        w.set_position(tauri::PhysicalPosition::new(
            area.position.x + (area.size.width - w_px) as i32 / 2,
            area.position.y + (area.size.height - h_px) as i32 / 2,
        ))?;
    }
    w.show()?;
    w.set_focus()?;
    Ok(true)
}

#[cfg(windows)]
fn remember_target() {
    use windows_sys::Win32::System::Threading::GetCurrentProcessId;
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
    // SAFETY: reading the foreground window and its process.
    let (hwnd, pid) = unsafe {
        let hwnd = GetForegroundWindow();
        let mut pid = 0u32;
        GetWindowThreadProcessId(hwnd, &mut pid);
        (hwnd, pid)
    };
    // XcreenShot's own windows are not a place to paste into.
    // SAFETY: reading this process's id.
    let ours = unsafe { GetCurrentProcessId() };
    *TARGET.lock().unwrap() = if hwnd.is_null() || pid == ours {
        0
    } else {
        hwnd as isize
    };
}

#[cfg(not(windows))]
fn remember_target() {}

/// The windows a paste can go to, the one that was in front when the panel opened
/// first and marked `previous`.
pub fn targets() -> Vec<serde_json::Value> {
    let previous = *TARGET.lock().unwrap() as u32;
    let mut list: Vec<serde_json::Value> = xs_capture::picker::app_windows()
        .into_iter()
        .map(|w| {
            serde_json::json!({
                "id": w.id,
                "app": w.app,
                "title": w.title,
                "minimized": w.minimized,
                "previous": previous != 0 && w.id == previous,
            })
        })
        .collect();
    list.sort_by_key(|w| !w["previous"].as_bool().unwrap_or(false));
    list
}

/// Brings a window forward (`target`, else the one that was in front when the panel
/// opened), restoring it if minimized, and presses Ctrl+V once. Its title.
#[cfg(windows)]
pub fn paste(target: Option<u32>) -> Result<String, String> {
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
        SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_CONTROL, VK_V,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetWindowTextW, IsIconic, IsWindow, SetForegroundWindow, ShowWindow, SW_RESTORE,
    };
    let hwnd = target
        .map(|t| t as isize)
        .unwrap_or_else(|| *TARGET.lock().unwrap());
    if hwnd == 0 {
        return Err("no window to paste into".into());
    }
    // SAFETY: plain calls on a window handle, checked first; the title buffer outlives
    // its call; SendInput reads the array for the length of the call.
    unsafe {
        let hwnd = hwnd as windows_sys::Win32::Foundation::HWND;
        if IsWindow(hwnd) == 0 {
            return Err("the window to paste into is gone".into());
        }
        if IsIconic(hwnd) != 0 {
            ShowWindow(hwnd, SW_RESTORE);
        }
        SetForegroundWindow(hwnd);
        std::thread::sleep(std::time::Duration::from_millis(150));
        let key = |vk: u16, up: bool| INPUT {
            r#type: INPUT_KEYBOARD,
            Anonymous: INPUT_0 {
                ki: KEYBDINPUT {
                    wVk: vk,
                    wScan: 0,
                    dwFlags: if up { KEYEVENTF_KEYUP } else { 0 },
                    time: 0,
                    dwExtraInfo: 0,
                },
            },
        };
        let keys = [
            key(VK_CONTROL, false),
            key(VK_V, false),
            key(VK_V, true),
            key(VK_CONTROL, true),
        ];
        let sent = SendInput(
            keys.len() as u32,
            keys.as_ptr(),
            std::mem::size_of::<INPUT>() as i32,
        );
        if sent as usize != keys.len() {
            return Err("the paste keys could not be sent".into());
        }
        let mut buf = [0u16; 256];
        let n = GetWindowTextW(hwnd, buf.as_mut_ptr(), buf.len() as i32);
        Ok(String::from_utf16_lossy(&buf[..n.max(0) as usize]))
    }
}

#[cfg(not(windows))]
pub fn paste(_target: Option<u32>) -> Result<String, String> {
    Err("not yet on this system".into())
}
