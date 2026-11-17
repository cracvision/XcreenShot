//! The magnifier of live drawing (VID-049): a round ×2 lens that follows the pointer,
//! made with Windows' magnification API. It shows what is under it, without itself,
//! and it is a normal window, so the recording has it too. It lives on the thread of
//! the input hook, which runs a message loop.

use windows_sys::Win32::Foundation::{HWND, RECT};
use windows_sys::Win32::Graphics::Gdi::{CreateEllipticRgn, InvalidateRect, SetWindowRgn};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::HiDpi::{
    SetThreadDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
};
use windows_sys::Win32::UI::Magnification::{
    MagInitialize, MagSetWindowFilterList, MagSetWindowSource, MagSetWindowTransform, MAGTRANSFORM,
    MW_FILTERMODE_EXCLUDE, WC_MAGNIFIERW,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, RegisterClassW, SetLayeredWindowAttributes, SetWindowPos,
    ShowWindow, HWND_TOPMOST, LWA_ALPHA, SWP_NOACTIVATE, SWP_SHOWWINDOW, SW_SHOWNOACTIVATE, WNDCLASSW,
    WS_CHILD, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
    WS_VISIBLE,
};

/// How many times bigger.
const ZOOM: f32 = 2.0;

pub struct Lens {
    host: HWND,
    view: HWND,
    size: i32,
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain([0]).collect()
}

impl Lens {
    /// A lens of `size` physical pixels across, hidden until it moves.
    pub fn new(size: i32) -> Option<Lens> {
        let class = wide("XcreenShotLens");
        // SAFETY: Win32 window creation on this thread; the windows are destroyed by
        // `Drop` on the same thread.
        unsafe {
            // Physical pixels, as the input hook gives them.
            SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
            if MagInitialize() == 0 {
                return None;
            }
            let module = GetModuleHandleW(std::ptr::null());
            let wc = WNDCLASSW {
                lpfnWndProc: Some(DefWindowProcW),
                hInstance: module,
                lpszClassName: class.as_ptr(),
                ..std::mem::zeroed()
            };
            // Registering twice fails harmlessly.
            RegisterClassW(&wc);
            let host = CreateWindowExW(
                WS_EX_TOPMOST | WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                class.as_ptr(),
                std::ptr::null(),
                WS_POPUP,
                0,
                0,
                size,
                size,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                module,
                std::ptr::null(),
            );
            if host.is_null() {
                return None;
            }
            SetLayeredWindowAttributes(host, 0, 255, LWA_ALPHA);
            let view = CreateWindowExW(
                0,
                WC_MAGNIFIERW,
                std::ptr::null(),
                WS_CHILD | WS_VISIBLE,
                0,
                0,
                size,
                size,
                host,
                std::ptr::null_mut(),
                module,
                std::ptr::null(),
            );
            if view.is_null() {
                DestroyWindow(host);
                return None;
            }
            let mut t = MAGTRANSFORM {
                v: [ZOOM, 0.0, 0.0, 0.0, ZOOM, 0.0, 0.0, 0.0, 1.0],
            };
            MagSetWindowTransform(view, &mut t);
            // It never shows itself.
            let mut me = host;
            MagSetWindowFilterList(view, MW_FILTERMODE_EXCLUDE, 1, &mut me);
            // Round.
            SetWindowRgn(host, CreateEllipticRgn(0, 0, size + 1, size + 1), 1);
            Some(Lens { host, view, size })
        }
    }

    /// Centers the lens on (`x`, `y`) (virtual screen) and shows what is there.
    pub fn move_to(&self, x: i32, y: i32) {
        let half = self.size / 2;
        let reach = (self.size as f32 / ZOOM / 2.0) as i32;
        // SAFETY: calls on this thread's own windows.
        unsafe {
            SetWindowPos(
                self.host,
                HWND_TOPMOST,
                x - half,
                y - half,
                self.size,
                self.size,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );
            MagSetWindowSource(
                self.view,
                RECT {
                    left: x - reach,
                    top: y - reach,
                    right: x + reach,
                    bottom: y + reach,
                },
            );
            InvalidateRect(self.view, std::ptr::null(), 1);
            ShowWindow(self.host, SW_SHOWNOACTIVATE);
        }
    }
}

impl Drop for Lens {
    fn drop(&mut self) {
        // SAFETY: the windows this thread made, destroyed once.
        unsafe {
            DestroyWindow(self.host);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows_sys::Win32::Graphics::Gdi::{GetDC, GetPixel, ReleaseDC};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
    };

    /// Shows a lens at (300, 300) for half a second (by hand: it is on the screen)
    /// and reads a few of its pixels: it shows the screen, not black.
    /// `cargo test -p xs-capture -- --ignored --nocapture a_lens_shows_the_screen`
    #[test]
    #[ignore]
    fn a_lens_shows_the_screen() {
        let read = |d: i32| -> Vec<u32> {
            // SAFETY: reading screen pixels.
            unsafe {
                let dc = GetDC(std::ptr::null_mut());
                let v = (0..9)
                    .map(|i| GetPixel(dc, 300 + (i % 3 - 1) * d, 300 + (i / 3 - 1) * d))
                    .collect();
                ReleaseDC(std::ptr::null_mut(), dc);
                v
            }
        };
        // SAFETY: this thread in physical pixels, like the hook's.
        unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
        let before = read(20);
        let lens = Lens::new(220).expect("a lens");
        lens.move_to(300, 300);
        let until = std::time::Instant::now() + std::time::Duration::from_millis(500);
        // SAFETY: a message pump for this thread's windows, then pixel reads.
        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            while std::time::Instant::now() < until {
                while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
                std::thread::sleep(std::time::Duration::from_millis(10));
            }
            let samples = read(40);
            drop(lens);
            println!(
                "{before:06x?}
{samples:06x?}"
            );
            assert!(samples.iter().any(|p| *p != 0), "all black");
            // Twice as big: 40 px out in the lens is 20 px out on the screen (edges of
            // letters come out a little softer: the lens smooths).
            let near =
                |a: u32, b: u32| (0..3).all(|k| ((a >> (8 * k)) & 255).abs_diff((b >> (8 * k)) & 255) <= 64);
            let same = before
                .iter()
                .zip(&samples)
                .filter(|(a, b)| near(**a, **b))
                .count();
            assert!(same >= 7, "{same} of 9");
        }
    }
}
