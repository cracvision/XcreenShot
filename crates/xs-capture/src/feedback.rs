//! Capture confirmation (CAP-055): a sound and a flash over the captured area when the
//! capture is taken. Both sounds are made here from plain waves; nothing is taken from
//! another app.

use crate::Rect;

const RATE: u32 = 44_100;

/// A 16-bit mono WAV file of `samples` (-1.0 to 1.0).
fn wav(samples: &[f32]) -> Vec<u8> {
    let data = (samples.len() * 2) as u32;
    let mut out = Vec::with_capacity(44 + data as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes()); // PCM
    out.extend_from_slice(&1u16.to_le_bytes()); // mono
    out.extend_from_slice(&RATE.to_le_bytes());
    out.extend_from_slice(&(RATE * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data.to_le_bytes());
    for s in samples {
        out.extend_from_slice(&((s.clamp(-1.0, 1.0) * i16::MAX as f32) as i16).to_le_bytes());
    }
    out
}

/// The sound of a capture: `shutter` (two short bursts, like a camera) or `soft` (a
/// quiet, quickly fading tone).
pub fn sound(kind: &str) -> Option<Vec<u8>> {
    let n = |ms: u32| (RATE * ms / 1000) as usize;
    let samples = match kind {
        "shutter" => {
            // Deterministic noise, so the sound is always the same.
            let mut seed = 0x2545_f491_u32;
            let mut noise = move || {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                (seed as f32 / u32::MAX as f32) * 2.0 - 1.0
            };
            let mut out = vec![0.0f32; n(150)];
            for (start, gain) in [(0, 0.55), (n(70), 0.4)] {
                let mut low = 0.0;
                for i in 0..n(45) {
                    let t = i as f32 / RATE as f32;
                    // A little low-pass takes the hiss off the noise.
                    low += (noise() - low) * 0.35;
                    out[start + i] += low * gain * (-t * 90.0).exp();
                }
            }
            out
        }
        "soft" => (0..n(70))
            .map(|i| {
                let t = i as f32 / RATE as f32;
                let attack = (t * 2000.0).min(1.0);
                (t * 1_150.0 * std::f32::consts::TAU).sin() * 0.22 * attack * (-t * 60.0).exp()
            })
            .collect(),
        _ => return None,
    };
    Some(wav(&samples))
}

/// Plays the capture sound once, at the system volume, unless the system is in quiet
/// time or presenting (CAP-055).
#[cfg(windows)]
pub fn play(kind: &str) {
    use std::sync::OnceLock;
    use windows_sys::Win32::Media::Audio::{PlaySoundW, SND_ASYNC, SND_MEMORY, SND_NODEFAULT};
    use windows_sys::Win32::UI::Shell::{
        SHQueryUserNotificationState, QUNS_PRESENTATION_MODE, QUNS_QUIET_TIME,
    };
    static SHUTTER: OnceLock<Option<Vec<u8>>> = OnceLock::new();
    static SOFT: OnceLock<Option<Vec<u8>>> = OnceLock::new();
    let cell = match kind {
        "shutter" => &SHUTTER,
        "soft" => &SOFT,
        _ => return,
    };
    let mut state = 0;
    // SAFETY: out-parameter call.
    if unsafe { SHQueryUserNotificationState(&mut state) } == 0
        && (state == QUNS_QUIET_TIME || state == QUNS_PRESENTATION_MODE)
    {
        return;
    }
    // Kept for the life of the app: the system reads it while it plays.
    if let Some(bytes) = cell.get_or_init(|| sound(kind)) {
        // SAFETY: with SND_MEMORY the first argument points to a whole WAV file that
        // outlives the asynchronous playback (a static).
        unsafe {
            PlaySoundW(
                bytes.as_ptr() as *const u16,
                std::ptr::null_mut(),
                SND_MEMORY | SND_ASYNC | SND_NODEFAULT,
            )
        };
    }
}

#[cfg(not(windows))]
pub fn play(_kind: &str) {}

/// The system asks for less motion: the flash becomes a still border (APP-045).
#[cfg(windows)]
fn reduced_motion() -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, SPI_GETCLIENTAREAANIMATION};
    let mut on = 1i32;
    // SAFETY: out-parameter call with a BOOL.
    unsafe { SystemParametersInfoW(SPI_GETCLIENTAREAANIMATION, 0, &mut on as *mut i32 as *mut _, 0) };
    on == 0
}

/// Lights the captured area up for an instant (at most 200 ms), in a window that never
/// appears in a capture. Returns at once; the flash runs on its own thread.
#[cfg(windows)]
pub fn flash(area: Rect) {
    use windows_sys::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows_sys::Win32::Graphics::Gdi::{
        CombineRgn, CreateRectRgn, CreateSolidBrush, DeleteObject, SetWindowRgn, RGN_DIFF,
    };
    use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, PeekMessageW, RegisterClassExW,
        SetLayeredWindowAttributes, SetWindowDisplayAffinity, SetWindowPos, TranslateMessage, HWND_TOPMOST,
        LWA_ALPHA, MSG, PM_REMOVE, SWP_NOACTIVATE, SWP_SHOWWINDOW, WDA_EXCLUDEFROMCAPTURE, WNDCLASSEXW,
        WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST, WS_EX_TRANSPARENT, WS_POPUP,
    };

    unsafe extern "system" fn proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        // SAFETY: default handling of a message of our own window.
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }

    if area.width == 0 || area.height == 0 {
        return;
    }
    let still = reduced_motion();
    std::thread::spawn(move || {
        let class: Vec<u16> = "XcreenShotCaptureFlash\0".encode_utf16().collect();
        // SAFETY: a window class and one window of this thread, destroyed before it ends;
        // the class name outlives the calls.
        unsafe {
            let module = GetModuleHandleW(std::ptr::null());
            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                style: 0,
                lpfnWndProc: Some(proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: module,
                hIcon: std::ptr::null_mut(),
                hCursor: std::ptr::null_mut(),
                hbrBackground: CreateSolidBrush(0x00FF_FFFF),
                lpszMenuName: std::ptr::null(),
                lpszClassName: class.as_ptr(),
                hIconSm: std::ptr::null_mut(),
            };
            // Registered again on later flashes: that fails harmlessly.
            RegisterClassExW(&wc);
            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE,
                class.as_ptr(),
                std::ptr::null(),
                WS_POPUP,
                area.x,
                area.y,
                area.width as i32,
                area.height as i32,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                module,
                std::ptr::null(),
            );
            if hwnd.is_null() {
                return;
            }
            SetWindowDisplayAffinity(hwnd, WDA_EXCLUDEFROMCAPTURE);
            let (w, h) = (area.width as i32, area.height as i32);
            if still {
                // A border that appears and goes, without animation.
                let t = 4.min(w / 2).min(h / 2).max(1);
                let outer = CreateRectRgn(0, 0, w, h);
                let inner = CreateRectRgn(t, t, w - t, h - t);
                CombineRgn(outer, outer, inner, RGN_DIFF);
                DeleteObject(inner);
                // The window owns the region from here.
                SetWindowRgn(hwnd, outer, 0);
            }
            let pump = || {
                let mut msg: MSG = std::mem::zeroed();
                while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }
            };
            let steps: &[(u8, u64)] = if still {
                &[(200, 200)]
            } else {
                &[
                    (110, 16),
                    (95, 16),
                    (80, 16),
                    (65, 16),
                    (50, 16),
                    (38, 16),
                    (26, 16),
                    (15, 16),
                    (6, 16),
                ]
            };
            let mut shown = false;
            for (alpha, ms) in steps {
                SetLayeredWindowAttributes(hwnd, 0, *alpha, LWA_ALPHA);
                if !shown {
                    SetWindowPos(
                        hwnd,
                        HWND_TOPMOST,
                        area.x,
                        area.y,
                        w,
                        h,
                        SWP_NOACTIVATE | SWP_SHOWWINDOW,
                    );
                    shown = true;
                }
                pump();
                std::thread::sleep(std::time::Duration::from_millis(*ms));
            }
            DestroyWindow(hwnd);
            pump();
        }
    });
}

#[cfg(not(windows))]
pub fn flash(_area: Rect) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_sounds_are_short_valid_wavs() {
        for kind in ["shutter", "soft"] {
            let w = sound(kind).unwrap();
            assert_eq!(&w[0..4], b"RIFF");
            assert_eq!(&w[8..12], b"WAVE");
            let samples = (w.len() - 44) / 2;
            assert!(samples < RATE as usize / 4, "{kind} lasts under 250 ms");
            assert!(w[44..].iter().any(|b| *b != 0), "{kind} is not silence");
        }
        assert!(sound("none").is_none());
    }
}
