//! Windows: the session ending (signing out, shutting down, restarting). XcreenShot
//! never holds it up: as a forced quit, it leaves the unsaved documents recoverable
//! and stops and saves a recording (APP-004, VID-013), then lets Windows go on. A
//! hidden window on a thread of its own hears it, so nothing waits on the UI thread.

use std::sync::{mpsc, Arc, OnceLock};
use std::time::Duration;

use serde_json::json;
use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetMessageW, RegisterClassW, TranslateMessage, MSG,
    WM_ENDSESSION, WM_QUERYENDSESSION, WNDCLASSW, WS_EX_TOOLWINDOW, WS_POPUP,
};
use xs_core::{Core, Source};

static CORE: OnceLock<Arc<Core>> = OnceLock::new();

/// How long the work may take: Windows waits a few seconds before it says an app is
/// holding up the shutdown; a recording's file takes up to 2 s.
const GRACE: Duration = Duration::from_secs(4);

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Starts listening for the end of the session. Once.
pub fn watch(core: Arc<Core>) {
    if CORE.set(core).is_err() {
        return;
    }
    let started = std::thread::Builder::new()
        .name("session-end".into())
        // SAFETY: a window class and a hidden top-level window of this thread, and its
        // message loop; the strings outlive the calls.
        .spawn(|| unsafe {
            let class = wide("XcreenShotSessionEnd");
            let instance = GetModuleHandleW(std::ptr::null());
            let wc = WNDCLASSW {
                lpfnWndProc: Some(proc),
                hInstance: instance,
                lpszClassName: class.as_ptr(),
                ..std::mem::zeroed()
            };
            RegisterClassW(&wc);
            // Top-level (not message-only): only those hear the session end. Never shown.
            let hwnd = CreateWindowExW(
                WS_EX_TOOLWINDOW,
                class.as_ptr(),
                wide("XcreenShot").as_ptr(),
                WS_POPUP,
                0,
                0,
                0,
                0,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                instance,
                std::ptr::null(),
            );
            if hwnd.is_null() {
                tracing::warn!("the session end cannot be heard: no window");
                return;
            }
            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        });
    if let Err(e) = started {
        tracing::warn!(error = %e, "the session end cannot be heard");
    }
}

unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        // Never in the way of signing out or shutting down.
        WM_QUERYENDSESSION => 1,
        WM_ENDSESSION if wparam != 0 => {
            session_ends();
            0
        }
        // SAFETY: the default handling of this window's other messages.
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

/// The session ends: the forced quit's work, for at most [`GRACE`].
fn session_ends() {
    let Some(core) = CORE.get().cloned() else { return };
    tracing::info!("the session is ending: quitting");
    let (done, wait) = mpsc::channel();
    std::thread::spawn(move || {
        if let Err(e) = core.call(Source::Ui, "app.quit", json!({ "force": true }), None) {
            tracing::warn!(error = %e.message, "quitting at the session end failed");
        }
        let _ = done.send(());
    });
    if wait.recv_timeout(GRACE).is_err() {
        tracing::warn!("quitting at the session end took too long");
    }
}
