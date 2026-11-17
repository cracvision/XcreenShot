//! Menu capture (CAP-031): the menus open on screen, found when the screens are frozen,
//! cut out with their shape: transparent between levels and outside rounded corners.
//!
//! On Windows the system's menus are windows of class `#32768`; menus an app draws
//! itself are not found (the capture then goes on as a region, CAP-031). The menu bar
//! item a menu hangs from comes from the owner window's menu bar.

use image::RgbaImage;
use serde::Serialize;

use crate::Rect;

/// The menus open now, outermost level first, and the menu bar item they hang from.
#[derive(Debug, Clone, Default, Serialize)]
pub struct OpenMenus {
    pub levels: Vec<Rect>,
    #[serde(rename = "barItem")]
    pub bar_item: Option<Rect>,
}

/// What a menu capture keeps (CAP-031): every open level or only the first, and the
/// menu bar item or not. Returns the rectangles to cut, in desktop pixels.
pub fn parts(menus: &OpenMenus, submenus: bool, bar: bool) -> Vec<Rect> {
    let mut out: Vec<Rect> = if submenus {
        menus.levels.clone()
    } else {
        menus.levels.iter().take(1).copied().collect()
    };
    if bar {
        if let Some(b) = menus.bar_item {
            out.insert(0, b);
        }
    }
    out
}

/// The rectangle around `parts`.
pub fn bounds(parts: &[Rect]) -> Option<Rect> {
    let first = *parts.first()?;
    Some(parts.iter().skip(1).fold(first, |a, r| {
        let (x0, y0) = (a.x.min(r.x), a.y.min(r.y));
        let (x1, y1) = (a.right().max(r.right()), a.bottom().max(r.bottom()));
        Rect {
            x: x0,
            y: y0,
            width: (x1 - x0) as u32,
            height: (y1 - y0) as u32,
        }
    }))
}

/// Makes transparent what lies outside `parts` (desktop pixels) on `image`, which shows
/// `area`; corners of the menu levels are rounded by `radius` pixels.
pub fn cut_out(image: &mut RgbaImage, area: Rect, parts: &[Rect], radius: u32, rounded: &[bool]) {
    let (w, h) = image.dimensions();
    for y in 0..h {
        for x in 0..w {
            let (dx, dy) = (area.x + x as i32, area.y + y as i32);
            let inside = parts
                .iter()
                .zip(rounded.iter().chain(std::iter::repeat(&false)))
                .any(|(r, round)| r.contains(dx, dy) && (!round || inside_rounded(r, dx, dy, radius as i32)));
            if !inside {
                image.get_pixel_mut(x, y).0[3] = 0;
            }
        }
    }
}

/// Whether a point of `r` is inside its corners rounded by `radius`.
fn inside_rounded(r: &Rect, x: i32, y: i32, radius: i32) -> bool {
    if radius <= 0 {
        return true;
    }
    let cx = if x < r.x + radius {
        r.x + radius
    } else if x >= r.right() - radius {
        r.right() - radius - 1
    } else {
        return true;
    };
    let cy = if y < r.y + radius {
        r.y + radius
    } else if y >= r.bottom() - radius {
        r.bottom() - radius - 1
    } else {
        return true;
    };
    let (ddx, ddy) = ((x - cx) as f32, (y - cy) as f32);
    ddx * ddx + ddy * ddy <= (radius as f32 + 0.5) * (radius as f32 + 0.5)
}

#[cfg(windows)]
pub fn open_menus() -> OpenMenus {
    win::open_menus()
}

#[cfg(not(windows))]
pub fn open_menus() -> OpenMenus {
    OpenMenus::default()
}

#[cfg(windows)]
mod win {
    use super::*;
    use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM, RECT};
    use windows_sys::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_EXTENDED_FRAME_BOUNDS};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetClassNameW, GetGUIThreadInfo, GetMenuBarInfo, GetWindowRect,
        GetWindowThreadProcessId, IsWindowVisible, GUITHREADINFO, MENUBARINFO, OBJID_MENU,
    };

    unsafe extern "system" fn collect(hwnd: HWND, data: LPARAM) -> BOOL {
        // SAFETY: data is the Vec passed by open_menus, alive for the enumeration.
        let found = unsafe { &mut *(data as *mut Vec<HWND>) };
        let mut class = [0u16; 16];
        // SAFETY: the buffer is valid for its length.
        let n = unsafe { GetClassNameW(hwnd, class.as_mut_ptr(), class.len() as i32) };
        if n > 0
            && String::from_utf16_lossy(&class[..n as usize]) == "#32768"
            && unsafe { IsWindowVisible(hwnd) } != 0
        {
            found.push(hwnd);
        }
        1
    }

    fn visible_rect(hwnd: HWND) -> Option<Rect> {
        let mut r = RECT {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        // SAFETY: out-parameters of the right size.
        let ok = unsafe {
            DwmGetWindowAttribute(
                hwnd,
                DWMWA_EXTENDED_FRAME_BOUNDS as u32,
                &mut r as *mut RECT as *mut _,
                std::mem::size_of::<RECT>() as u32,
            ) == 0
                || GetWindowRect(hwnd, &mut r) != 0
        };
        (ok && r.right > r.left && r.bottom > r.top).then(|| Rect {
            x: r.left,
            y: r.top,
            width: (r.right - r.left) as u32,
            height: (r.bottom - r.top) as u32,
        })
    }

    /// The menu bar item right above the first level, in the window that owns the menu.
    fn bar_item(menu: HWND, first: Rect) -> Option<Rect> {
        // SAFETY: plain calls with out-parameters of the right size.
        unsafe {
            let thread = GetWindowThreadProcessId(menu, std::ptr::null_mut());
            let mut info: GUITHREADINFO = std::mem::zeroed();
            info.cbSize = std::mem::size_of::<GUITHREADINFO>() as u32;
            if GetGUIThreadInfo(thread, &mut info) == 0 || info.hwndMenuOwner.is_null() {
                return None;
            }
            for item in 1..=64 {
                let mut bar: MENUBARINFO = std::mem::zeroed();
                bar.cbSize = std::mem::size_of::<MENUBARINFO>() as u32;
                if GetMenuBarInfo(info.hwndMenuOwner, OBJID_MENU, item, &mut bar) == 0 {
                    break;
                }
                let r = bar.rcBar;
                let touches = (r.bottom - first.y).abs() <= 6;
                let over = r.left < first.right() && r.right > first.x;
                if touches && over && r.right > r.left {
                    return Some(Rect {
                        x: r.left,
                        y: r.top,
                        width: (r.right - r.left) as u32,
                        height: (r.bottom - r.top) as u32,
                    });
                }
            }
            None
        }
    }

    pub fn open_menus() -> OpenMenus {
        let mut found: Vec<HWND> = Vec::new();
        // SAFETY: the callback only reads window data and pushes into `found`.
        unsafe { EnumWindows(Some(collect), &mut found as *mut Vec<HWND> as LPARAM) };
        // Enumeration goes from the top: the deepest submenu first.
        found.reverse();
        let levels: Vec<Rect> = found.iter().filter_map(|h| visible_rect(*h)).collect();
        let bar_item = match (found.first(), levels.first()) {
            (Some(h), Some(r)) => bar_item(*h, *r),
            _ => None,
        };
        OpenMenus { levels, bar_item }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn rect(x: i32, y: i32, w: u32, h: u32) -> Rect {
        Rect {
            x,
            y,
            width: w,
            height: h,
        }
    }

    fn two_levels() -> OpenMenus {
        OpenMenus {
            levels: vec![rect(100, 100, 200, 300), rect(300, 150, 180, 120)],
            bar_item: Some(rect(100, 70, 60, 30)),
        }
    }

    /// CAP-031: with submenus, both levels; with the menu bar, its item too.
    #[test]
    fn the_parts_follow_the_properties() {
        let m = two_levels();
        assert_eq!(parts(&m, true, false).len(), 2);
        assert_eq!(parts(&m, false, false), vec![rect(100, 100, 200, 300)]);
        assert_eq!(parts(&m, true, true)[0], rect(100, 70, 60, 30));
        assert_eq!(bounds(&parts(&m, true, true)), Some(rect(100, 70, 380, 330)));
    }

    /// CAP-031: transparent between the levels and outside the rounded corners.
    #[test]
    fn the_shape_is_kept() {
        let m = two_levels();
        let p = parts(&m, true, false);
        let area = bounds(&p).unwrap();
        let mut img = RgbaImage::from_pixel(area.width, area.height, Rgba([200, 200, 200, 255]));
        cut_out(&mut img, area, &p, 8, &[true, true]);
        // Inside each level.
        assert_eq!(img.get_pixel(50, 50).0[3], 255);
        assert_eq!(img.get_pixel(250, 100).0[3], 255);
        // Between them, below the submenu.
        assert_eq!(img.get_pixel(300, 250).0[3], 0);
        // A rounded corner.
        assert_eq!(img.get_pixel(0, 0).0[3], 0);
        assert_eq!(img.get_pixel(8, 8).0[3], 255);
    }
}
