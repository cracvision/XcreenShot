//! The desktop's background, without windows (CAP-029): each monitor's wallpaper
//! placed as the system places it (fill, fit, stretch, center, tile or span), over the
//! background color. Windows as objects uses it as the base where app windows were.

use image::{imageops, Rgba, RgbaImage};

use crate::compose::Rect;

/// How the system places the picture on a monitor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Position {
    Center,
    Tile,
    Stretch,
    Fit,
    Fill,
    /// One picture across every monitor.
    Span,
}

/// One monitor's part of the background.
pub struct Monitor {
    /// In physical pixels of the virtual screen.
    pub rect: Rect,
    pub picture: Option<RgbaImage>,
}

/// The background of `area` (physical pixels of the virtual screen).
pub fn compose(area: Rect, monitors: &[Monitor], position: Position, color: [u8; 3]) -> RgbaImage {
    let mut out = RgbaImage::from_pixel(area.width, area.height, Rgba([color[0], color[1], color[2], 255]));
    if position == Position::Span {
        // The bounding box of every monitor, filled by the one picture.
        let Some(picture) = monitors.iter().find_map(|m| m.picture.as_ref()) else {
            return out;
        };
        let Some(all) = monitors.iter().map(|m| m.rect).reduce(union) else {
            return out;
        };
        let layer = place(picture, all.width, all.height, Position::Fill, color);
        for m in monitors {
            if let Some(part) = m.rect.intersect(&area) {
                copy(&layer, part.x - all.x, part.y - all.y, &mut out, part, area);
            }
        }
        return out;
    }
    for m in monitors {
        let (Some(picture), Some(part)) = (m.picture.as_ref(), m.rect.intersect(&area)) else {
            continue;
        };
        let layer = place(picture, m.rect.width, m.rect.height, position, color);
        copy(&layer, part.x - m.rect.x, part.y - m.rect.y, &mut out, part, area);
    }
    out
}

fn union(a: Rect, b: Rect) -> Rect {
    let x0 = a.x.min(b.x);
    let y0 = a.y.min(b.y);
    let x1 = (a.x + a.width as i32).max(b.x + b.width as i32);
    let y1 = (a.y + a.height as i32).max(b.y + b.height as i32);
    Rect {
        x: x0,
        y: y0,
        width: (x1 - x0) as u32,
        height: (y1 - y0) as u32,
    }
}

/// Copies `part` (virtual-screen pixels) of `layer`, which starts at (`lx`, `ly`) of
/// it, into `out`, which covers `area`.
fn copy(layer: &RgbaImage, lx: i32, ly: i32, out: &mut RgbaImage, part: Rect, area: Rect) {
    let piece = imageops::crop_imm(layer, lx.max(0) as u32, ly.max(0) as u32, part.width, part.height);
    imageops::replace(
        out,
        &piece.to_image(),
        (part.x - area.x) as i64,
        (part.y - area.y) as i64,
    );
}

/// `picture` laid on a `w` × `h` monitor as `position` says.
fn place(picture: &RgbaImage, w: u32, h: u32, position: Position, color: [u8; 3]) -> RgbaImage {
    let mut out = RgbaImage::from_pixel(w, h, Rgba([color[0], color[1], color[2], 255]));
    let (pw, ph) = (picture.width().max(1) as f64, picture.height().max(1) as f64);
    let scaled = |s: f64| {
        let sw = ((pw * s).round() as u32).max(1);
        let sh = ((ph * s).round() as u32).max(1);
        imageops::resize(picture, sw, sh, imageops::FilterType::Triangle)
    };
    let centered = |out: &mut RgbaImage, img: &RgbaImage| {
        let x = (w as i64 - img.width() as i64) / 2;
        let y = (h as i64 - img.height() as i64) / 2;
        imageops::overlay(out, img, x, y);
    };
    match position {
        Position::Stretch => {
            out = imageops::resize(picture, w, h, imageops::FilterType::Triangle);
        }
        Position::Fit => centered(&mut out, &scaled((w as f64 / pw).min(h as f64 / ph))),
        Position::Fill | Position::Span => centered(&mut out, &scaled((w as f64 / pw).max(h as f64 / ph))),
        Position::Center => centered(&mut out, picture),
        Position::Tile => {
            let mut y = 0;
            while y < h {
                let mut x = 0;
                while x < w {
                    imageops::overlay(&mut out, picture, x as i64, y as i64);
                    x += picture.width().max(1);
                }
                y += picture.height().max(1);
            }
        }
    }
    out
}

/// The desktop's background over `area`, as this system has it; `None` if it cannot
/// be known.
#[cfg(windows)]
pub fn desktop(area: Rect) -> Option<RgbaImage> {
    use windows::core::PCWSTR;
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoTaskMemFree, CLSCTX_ALL, COINIT_MULTITHREADED,
    };
    use windows::Win32::UI::Shell::{
        DesktopWallpaper, IDesktopWallpaper, DWPOS_CENTER, DWPOS_FILL, DWPOS_FIT, DWPOS_SPAN, DWPOS_STRETCH,
        DWPOS_TILE,
    };

    // SAFETY: COM calls on this thread; every string the API allocates is freed with
    // CoTaskMemFree after it is copied.
    unsafe {
        // Already initialized in another mode is fine: COM works either way.
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
        let wall: IDesktopWallpaper = CoCreateInstance(&DesktopWallpaper, None, CLSCTX_ALL).ok()?;
        let take = |p: windows::core::PWSTR| -> String {
            if p.is_null() {
                return String::new();
            }
            let s = p.to_string().unwrap_or_default();
            CoTaskMemFree(Some(p.0 as *const _));
            s
        };
        let position = match wall.GetPosition().ok()? {
            DWPOS_CENTER => Position::Center,
            DWPOS_TILE => Position::Tile,
            DWPOS_STRETCH => Position::Stretch,
            DWPOS_FIT => Position::Fit,
            DWPOS_SPAN => Position::Span,
            DWPOS_FILL => Position::Fill,
            _ => Position::Fill,
        };
        let color = wall.GetBackgroundColor().map(|c| c.0).unwrap_or(0);
        let color = [
            (color & 0xff) as u8,
            ((color >> 8) & 0xff) as u8,
            ((color >> 16) & 0xff) as u8,
        ];
        let count = wall.GetMonitorDevicePathCount().ok()?;
        let mut monitors = Vec::new();
        let mut pictures: Vec<(String, RgbaImage)> = Vec::new();
        for i in 0..count {
            let Ok(raw) = wall.GetMonitorDevicePathAt(i) else {
                continue;
            };
            let id = take(raw);
            let wide: Vec<u16> = id.encode_utf16().chain(Some(0)).collect();
            // Monitors that are not on have no rectangle.
            let Ok(r) = wall.GetMonitorRECT(PCWSTR(wide.as_ptr())) else {
                continue;
            };
            let file = wall
                .GetWallpaper(PCWSTR(wide.as_ptr()))
                .map(take)
                .unwrap_or_default();
            let picture = if file.is_empty() {
                None
            } else if let Some((_, p)) = pictures.iter().find(|(f, _)| *f == file) {
                Some(p.clone())
            } else {
                match crate::filesource::decode_file(std::path::Path::new(&file)) {
                    Ok(p) => {
                        pictures.push((file, p.clone()));
                        Some(p)
                    }
                    Err(e) => {
                        tracing::warn!(error = %e, "desktop background not read");
                        return None;
                    }
                }
            };
            monitors.push(Monitor {
                rect: Rect {
                    x: r.left,
                    y: r.top,
                    width: (r.right - r.left).max(0) as u32,
                    height: (r.bottom - r.top).max(0) as u32,
                },
                picture,
            });
        }
        if monitors.is_empty() {
            return None;
        }
        Some(compose(area, &monitors, position, color))
    }
}

#[cfg(not(windows))]
pub fn desktop(_area: Rect) -> Option<RgbaImage> {
    // macOS: arrives with the Mac capture work; the base stays the screen as seen.
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rect(x: i32, y: i32, width: u32, height: u32) -> Rect {
        Rect { x, y, width, height }
    }

    fn picture(w: u32, h: u32) -> RgbaImage {
        // Left half red, right half blue: placement shows in the colors.
        RgbaImage::from_fn(w, h, |x, _| {
            if x < w / 2 {
                Rgba([255, 0, 0, 255])
            } else {
                Rgba([0, 0, 255, 255])
            }
        })
    }

    #[test]
    fn fit_leaves_the_color_and_fill_covers() {
        let m = [Monitor {
            rect: rect(0, 0, 200, 100),
            picture: Some(picture(100, 100)),
        }];
        let fit = compose(rect(0, 0, 200, 100), &m, Position::Fit, [0, 255, 0]);
        assert_eq!(fit.get_pixel(10, 50).0, [0, 255, 0, 255], "color at the sides");
        assert_eq!(fit.get_pixel(60, 50).0, [255, 0, 0, 255]);
        assert_eq!(fit.get_pixel(140, 50).0, [0, 0, 255, 255]);
        let fill = compose(rect(0, 0, 200, 100), &m, Position::Fill, [0, 255, 0]);
        assert_eq!(fill.get_pixel(5, 50).0, [255, 0, 0, 255], "no color left");
        assert_eq!(fill.get_pixel(195, 50).0, [0, 0, 255, 255]);
    }

    #[test]
    fn each_monitor_gets_its_own_and_the_area_cuts() {
        let m = [
            Monitor {
                rect: rect(0, 0, 100, 100),
                picture: Some(RgbaImage::from_pixel(10, 10, Rgba([255, 0, 0, 255]))),
            },
            Monitor {
                rect: rect(100, 0, 100, 100),
                picture: None,
            },
        ];
        // Only the second monitor and a bit of the first.
        let out = compose(rect(90, 0, 110, 100), &m, Position::Stretch, [9, 9, 9]);
        assert_eq!((out.width(), out.height()), (110, 100));
        assert_eq!(out.get_pixel(5, 5).0, [255, 0, 0, 255]);
        assert_eq!(
            out.get_pixel(50, 5).0,
            [9, 9, 9, 255],
            "a monitor with no picture"
        );
    }

    #[test]
    fn span_crosses_monitors() {
        let m = [
            Monitor {
                rect: rect(0, 0, 100, 100),
                picture: Some(picture(200, 100)),
            },
            Monitor {
                rect: rect(100, 0, 100, 100),
                picture: Some(picture(200, 100)),
            },
        ];
        let out = compose(rect(0, 0, 200, 100), &m, Position::Span, [0, 0, 0]);
        assert_eq!(out.get_pixel(20, 50).0, [255, 0, 0, 255]);
        assert_eq!(out.get_pixel(180, 50).0, [0, 0, 255, 255]);
    }
}
