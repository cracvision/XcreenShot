//! Windows: native windows that paint the frozen screens straight from memory
//! (CAP-013, APP-048). The transparent selection webview sits on top. Moving tens of
//! megabytes into a webview per monitor takes hundreds of milliseconds; a GDI blit
//! takes a few, whatever the number or size of monitors.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, RECT, WPARAM};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, InvalidateRect, SetDIBitsToDevice, UpdateWindow, BITMAPINFO, BITMAPINFOHEADER,
    BI_RGB, DIB_RGB_COLORS, PAINTSTRUCT,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, RegisterClassW, SetWindowDisplayAffinity, SetWindowPos, ShowWindow,
    HTNOWHERE, HWND_TOPMOST, MA_NOACTIVATE, SWP_NOACTIVATE, SWP_SHOWWINDOW, SW_HIDE, WDA_EXCLUDEFROMCAPTURE,
    WM_ERASEBKGND, WM_MOUSEACTIVATE, WM_NCHITTEST, WM_PAINT, WNDCLASSW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TOPMOST, WS_POPUP,
};

struct Bitmap {
    width: i32,
    height: i32,
    bgra: Vec<u8>,
}

/// Window handle (as usize, so it can live in a static) -> what it paints.
fn bitmaps() -> &'static Mutex<HashMap<usize, Bitmap>> {
    static B: OnceLock<Mutex<HashMap<usize, Bitmap>>> = OnceLock::new();
    B.get_or_init(Default::default)
}

/// One native window per monitor, created on the UI thread and reused.
fn windows() -> &'static Mutex<Vec<usize>> {
    static W: OnceLock<Mutex<Vec<usize>>> = OnceLock::new();
    W.get_or_init(Default::default)
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(Some(0)).collect()
}

unsafe extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    // SAFETY: called by the system on the UI thread with valid handles; painting uses
    // the bitmap stored for this window.
    unsafe {
        match msg {
            WM_PAINT => {
                let mut ps: PAINTSTRUCT = std::mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);
                if let Some(b) = bitmaps().lock().unwrap().get(&(hwnd as usize)) {
                    let mut info: BITMAPINFO = std::mem::zeroed();
                    info.bmiHeader = BITMAPINFOHEADER {
                        biSize: std::mem::size_of::<BITMAPINFOHEADER>() as u32,
                        biWidth: b.width,
                        biHeight: -b.height, // top-down rows
                        biPlanes: 1,
                        biBitCount: 32,
                        biCompression: BI_RGB,
                        ..std::mem::zeroed()
                    };
                    SetDIBitsToDevice(
                        hdc,
                        0,
                        0,
                        b.width as u32,
                        b.height as u32,
                        0,
                        0,
                        0,
                        b.height as u32,
                        b.bgra.as_ptr().cast(),
                        &info,
                        DIB_RGB_COLORS,
                    );
                }
                EndPaint(hwnd, &ps);
                0
            }
            WM_ERASEBKGND => 1,
            // Input belongs to the selection window on top; this one never takes focus.
            WM_MOUSEACTIVATE => MA_NOACTIVATE as LRESULT,
            WM_NCHITTEST => HTNOWHERE as LRESULT,
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

/// Creates the native window for `index` if needed. UI thread only.
fn ensure(index: usize) -> HWND {
    let mut list = windows().lock().unwrap();
    while list.len() <= index {
        // SAFETY: standard window class registration and creation on the UI thread;
        // the class name and window title outlive the calls.
        unsafe {
            let class = wide("XcreenShotFrozenScreen");
            let instance = GetModuleHandleW(std::ptr::null());
            let wc = WNDCLASSW {
                lpfnWndProc: Some(wnd_proc),
                hInstance: instance,
                lpszClassName: class.as_ptr(),
                ..std::mem::zeroed()
            };
            RegisterClassW(&wc); // fails harmlessly once registered
            let hwnd = CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE,
                class.as_ptr(),
                wide("XcreenShot").as_ptr(),
                WS_POPUP,
                0,
                0,
                1,
                1,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                instance,
                std::ptr::null(),
            );
            // Never part of a capture or a screen share (CAP-023).
            SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE);
            list.push(hwnd as usize);
        }
    }
    list[index] as HWND
}

/// Shows the frozen image of monitor `index` over `rect` (physical pixels). UI thread.
/// `bgra` is the frame already converted (off the UI thread, `xs_capture::interactive::to_bgra`).
pub fn show(index: usize, rect: RECT, width: u32, height: u32, bgra: Vec<u8>) {
    let hwnd = ensure(index);
    bitmaps().lock().unwrap().insert(
        hwnd as usize,
        Bitmap {
            width: width as i32,
            height: height as i32,
            bgra,
        },
    );
    // SAFETY: hwnd was created above on this thread.
    unsafe {
        SetWindowPos(
            hwnd,
            HWND_TOPMOST,
            rect.left,
            rect.top,
            rect.right - rect.left,
            rect.bottom - rect.top,
            SWP_NOACTIVATE | SWP_SHOWWINDOW,
        );
        InvalidateRect(hwnd, std::ptr::null(), 0);
        UpdateWindow(hwnd);
    }
}

/// Hides every frozen screen and frees the images. UI thread.
pub fn hide_all() {
    let list = windows().lock().unwrap().clone();
    for hwnd in list {
        // SAFETY: handles created by `ensure` on the UI thread.
        unsafe { ShowWindow(hwnd as HWND, SW_HIDE) };
    }
    bitmaps().lock().unwrap().clear();
}
