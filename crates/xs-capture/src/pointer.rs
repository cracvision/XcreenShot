//! The mouse pointer as a picture (CAP-047): the shape it has at that moment (arrow,
//! hand, text bar, an app's own cursor), with its transparency, and where its
//! top-left corner is on the desktop, in physical pixels. Screen captures never
//! contain the pointer; this is how it gets into a document when asked for.

use image::RgbaImage;

use crate::Rect;

#[derive(Debug, Clone)]
pub struct ScreenPointer {
    pub image: RgbaImage,
    /// Top-left corner of `image` on the desktop (the pointer position minus its
    /// hot spot).
    pub x: i32,
    pub y: i32,
}

impl ScreenPointer {
    /// The part of the pointer inside `area`, placed in `area`'s pixels; `None` when it
    /// was outside (CAP-047: no object then).
    pub fn within(&self, area: Rect) -> Option<xs_editor::Pointer> {
        let own = Rect {
            x: self.x,
            y: self.y,
            width: self.image.width(),
            height: self.image.height(),
        };
        let part = own.intersect(&area)?;
        let image = image::imageops::crop_imm(
            &self.image,
            (part.x - self.x) as u32,
            (part.y - self.y) as u32,
            part.width,
            part.height,
        )
        .to_image();
        // A fully transparent corner of the bitmap is not the pointer.
        if image.pixels().all(|p| p.0[3] == 0) {
            return None;
        }
        Some(xs_editor::Pointer {
            image,
            x: part.x - area.x,
            y: part.y - area.y,
        })
    }
}

/// Lays `pointer` over `image`, for what leaves without the editor: the clipboard and
/// the `--out` file (CAP-046, CAP-047).
pub fn flatten(image: &RgbaImage, pointer: &xs_editor::Pointer) -> RgbaImage {
    let mut out = image.clone();
    image::imageops::overlay(&mut out, &pointer.image, pointer.x as i64, pointer.y as i64);
    out
}

/// The pointer from two pictures of it, drawn on black and on white: where they
/// differ by the background, the pointer is see-through. Pixels that invert what is
/// under them (the text bar) come out black, which reads on the usual light fields.
pub fn from_black_and_white(width: u32, height: u32, black: &[u8], white: &[u8]) -> RgbaImage {
    let mut out = RgbaImage::new(width, height);
    for (i, px) in out.pixels_mut().enumerate() {
        let b = &black[i * 4..i * 4 + 3];
        let w = &white[i * 4..i * 4 + 3];
        let diff: i32 = (0..3).map(|c| w[c] as i32 - b[c] as i32).sum::<i32>() / 3;
        if diff < 0 {
            // Inverts the background: show it black.
            px.0 = [0, 0, 0, 255];
            continue;
        }
        let alpha = (255 - diff).clamp(0, 255) as u8;
        if alpha == 0 {
            px.0 = [0, 0, 0, 0];
            continue;
        }
        let un = |v: u8| ((v as u32 * 255 + alpha as u32 / 2) / alpha as u32).min(255) as u8;
        // The pictures are BGRA, as GDI paints them.
        px.0 = [un(b[2]), un(b[1]), un(b[0]), alpha];
    }
    out
}

#[cfg(windows)]
pub fn grab() -> Option<ScreenPointer> {
    use std::ffi::c_void;
    use std::mem::{size_of, zeroed};
    use windows_sys::Win32::Graphics::Gdi::{
        CreateCompatibleDC, CreateDIBSection, DeleteDC, DeleteObject, GetDC, GetObjectW, PatBlt, ReleaseDC,
        SelectObject, BITMAP, BITMAPINFO, BITMAPINFOHEADER, BI_RGB, BLACKNESS, DIB_RGB_COLORS, WHITENESS,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DrawIconEx, GetCursorInfo, GetIconInfo, CURSORINFO, CURSOR_SHOWING, DI_NORMAL, HCURSOR, ICONINFO,
    };

    /// The pointer drawn over a black or white square, as BGRA rows from the top.
    unsafe fn draw(cursor: HCURSOR, w: i32, h: i32, fill: u32) -> Option<Vec<u8>> {
        // SAFETY: a memory DC and DIB made and released here; `bits` holds w * h
        // BGRA pixels while the bitmap lives.
        unsafe {
            let screen = GetDC(std::ptr::null_mut());
            let dc = CreateCompatibleDC(screen);
            ReleaseDC(std::ptr::null_mut(), screen);
            let mut info: BITMAPINFO = zeroed();
            info.bmiHeader = BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                ..zeroed()
            };
            let mut bits: *mut c_void = std::ptr::null_mut();
            let bitmap = CreateDIBSection(dc, &info, DIB_RGB_COLORS, &mut bits, std::ptr::null_mut(), 0);
            if bitmap.is_null() || bits.is_null() {
                DeleteDC(dc);
                return None;
            }
            let old = SelectObject(dc, bitmap);
            PatBlt(dc, 0, 0, w, h, fill);
            let drawn = DrawIconEx(dc, 0, 0, cursor, w, h, 0, std::ptr::null_mut(), DI_NORMAL) != 0;
            let pixels = std::slice::from_raw_parts(bits as *const u8, (w * h * 4) as usize).to_vec();
            SelectObject(dc, old);
            DeleteObject(bitmap);
            DeleteDC(dc);
            drawn.then_some(pixels)
        }
    }

    // SAFETY: plain Win32 calls with out-parameters; every GDI object made here is
    // released before returning.
    unsafe {
        let mut info: CURSORINFO = zeroed();
        info.cbSize = size_of::<CURSORINFO>() as u32;
        if GetCursorInfo(&mut info) == 0 || info.flags & CURSOR_SHOWING == 0 || info.hCursor.is_null() {
            return None;
        }
        let mut icon: ICONINFO = zeroed();
        if GetIconInfo(info.hCursor, &mut icon) == 0 {
            return None;
        }
        let mut mask: BITMAP = zeroed();
        GetObjectW(
            icon.hbmMask,
            size_of::<BITMAP>() as i32,
            &mut mask as *mut BITMAP as *mut c_void,
        );
        let monochrome = icon.hbmColor.is_null();
        DeleteObject(icon.hbmMask);
        if !monochrome {
            DeleteObject(icon.hbmColor);
        }
        let w = mask.bmWidth;
        // A black-and-white cursor stacks its two masks in one bitmap.
        let h = if monochrome {
            mask.bmHeight / 2
        } else {
            mask.bmHeight
        };
        if w <= 0 || h <= 0 {
            return None;
        }
        let black = draw(info.hCursor, w, h, BLACKNESS)?;
        let white = draw(info.hCursor, w, h, WHITENESS)?;
        Some(ScreenPointer {
            image: from_black_and_white(w as u32, h as u32, &black, &white),
            x: info.ptScreenPos.x - icon.xHotspot as i32,
            y: info.ptScreenPos.y - icon.yHotspot as i32,
        })
    }
}

/// macOS arrives with the Mac tests: the capture says so in its warnings.
#[cfg(not(windows))]
pub fn grab() -> Option<ScreenPointer> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transparency_comes_from_the_two_backgrounds() {
        // Four pixels, BGRA: opaque red, see-through, half-transparent white, inverting.
        let black = [0, 0, 255, 0, 0, 0, 0, 0, 128, 128, 128, 0, 255, 255, 255, 0];
        let white = [0, 0, 255, 0, 255, 255, 255, 0, 255, 255, 255, 0, 0, 0, 0, 0];
        let img = from_black_and_white(4, 1, &black, &white);
        assert_eq!(img.get_pixel(0, 0).0, [255, 0, 0, 255]);
        assert_eq!(img.get_pixel(1, 0).0[3], 0);
        let half = img.get_pixel(2, 0).0;
        assert!((126..=130).contains(&half[3]) && half[0] > 250, "{half:?}");
        assert_eq!(img.get_pixel(3, 0).0, [0, 0, 0, 255]);
    }

    #[test]
    fn only_the_part_inside_the_capture_counts() {
        let pointer = ScreenPointer {
            image: RgbaImage::from_pixel(10, 10, image::Rgba([0, 0, 0, 255])),
            x: 95,
            y: 45,
        };
        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 50,
        };
        let inside = pointer.within(area).unwrap();
        assert_eq!((inside.x, inside.y), (95, 45));
        assert_eq!(inside.image.dimensions(), (5, 5));
        let elsewhere = Rect {
            x: 200,
            y: 0,
            width: 100,
            height: 50,
        };
        assert!(pointer.within(elsewhere).is_none());
        let flat = flatten(&RgbaImage::new(100, 50), &inside);
        assert_eq!(flat.get_pixel(97, 47).0, [0, 0, 0, 255]);
        assert_eq!(flat.get_pixel(90, 40).0[3], 0);
    }

    #[cfg(windows)]
    #[test]
    fn the_pointer_has_a_shape() {
        // Without a desktop (a service), there may be no pointer: nothing to check then.
        if let Some(p) = grab() {
            assert!(p.image.width() >= 8 && p.image.height() >= 8);
            assert!(p.image.pixels().any(|px| px.0[3] == 255));
        }
    }
}
