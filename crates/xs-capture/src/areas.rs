//! Multiple areas (CAP-033): several areas picked in one selection become one image,
//! each area where the composition puts it.
//!
//! - **Original position** (default): the canvas is the rectangle around all the areas,
//!   each one at its real place; the rest is transparent and overlapping areas join.
//! - **Stacked**: one below the other in the order they were picked, left aligned, `gap`
//!   pixels apart.
//! - **In a row**: one beside the other, top aligned, `gap` pixels apart.

use image::RgbaImage;

use crate::Rect;

/// How the areas are put together (CAP-033).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Composition {
    Original,
    Stacked,
    Row,
}

impl Composition {
    pub fn parse(s: &str) -> Composition {
        match s {
            "stacked" => Composition::Stacked,
            "row" => Composition::Row,
            _ => Composition::Original,
        }
    }
}

/// The joined image, and where each area sits on it (canvas pixels, in the order given).
pub fn compose(parts: &[(Rect, RgbaImage)], composition: Composition, gap: u32) -> (RgbaImage, Vec<Rect>) {
    if parts.is_empty() {
        return (RgbaImage::new(1, 1), Vec::new());
    }
    let places: Vec<Rect> = match composition {
        Composition::Original => {
            let x0 = parts.iter().map(|(r, _)| r.x).min().unwrap_or(0);
            let y0 = parts.iter().map(|(r, _)| r.y).min().unwrap_or(0);
            parts
                .iter()
                .map(|(r, img)| Rect {
                    x: r.x - x0,
                    y: r.y - y0,
                    width: img.width(),
                    height: img.height(),
                })
                .collect()
        }
        Composition::Stacked => {
            let mut y = 0i32;
            parts
                .iter()
                .map(|(_, img)| {
                    let place = Rect {
                        x: 0,
                        y,
                        width: img.width(),
                        height: img.height(),
                    };
                    y += img.height() as i32 + gap as i32;
                    place
                })
                .collect()
        }
        Composition::Row => {
            let mut x = 0i32;
            parts
                .iter()
                .map(|(_, img)| {
                    let place = Rect {
                        x,
                        y: 0,
                        width: img.width(),
                        height: img.height(),
                    };
                    x += img.width() as i32 + gap as i32;
                    place
                })
                .collect()
        }
    };
    let width = places.iter().map(|p| p.right()).max().unwrap_or(1).max(1) as u32;
    let height = places.iter().map(|p| p.bottom()).max().unwrap_or(1).max(1) as u32;
    let mut out = RgbaImage::new(width, height);
    for ((_, img), place) in parts.iter().zip(&places) {
        image::imageops::replace(&mut out, img, place.x as i64, place.y as i64);
    }
    (out, places)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn area(x: i32, y: i32, w: u32, h: u32, v: u8) -> (Rect, RgbaImage) {
        (
            Rect {
                x,
                y,
                width: w,
                height: h,
            },
            RgbaImage::from_pixel(w, h, Rgba([v, v, v, 255])),
        )
    }

    /// CAP-033: two 200 × 100 areas in opposite corners, at their original place: the
    /// rectangle around them, transparent between them.
    #[test]
    fn original_position_keeps_the_places_with_transparency_between() {
        let parts = [area(0, 0, 200, 100, 10), area(1000, 600, 200, 100, 20)];
        let (img, places) = compose(&parts, Composition::Original, 10);
        assert_eq!(img.dimensions(), (1200, 700));
        assert_eq!(img.get_pixel(5, 5).0, [10, 10, 10, 255]);
        assert_eq!(img.get_pixel(1100, 650).0, [20, 20, 20, 255]);
        assert_eq!(img.get_pixel(600, 350).0[3], 0, "transparent between them");
        assert_eq!(places[1].x, 1000);
    }

    /// CAP-033: the same areas stacked: 200 × 210 (10 px apart).
    #[test]
    fn stacked_areas_go_one_below_the_other() {
        let parts = [area(0, 0, 200, 100, 10), area(1000, 600, 200, 100, 20)];
        let (img, places) = compose(&parts, Composition::Stacked, 10);
        assert_eq!(img.dimensions(), (200, 210));
        assert_eq!(places[1].y, 110);
        assert_eq!(img.get_pixel(5, 150).0, [20, 20, 20, 255]);
    }

    #[test]
    fn a_row_puts_them_side_by_side() {
        let parts = [area(0, 0, 200, 100, 10), area(1000, 600, 150, 80, 20)];
        let (img, places) = compose(&parts, Composition::Row, 10);
        assert_eq!(img.dimensions(), (360, 100));
        assert_eq!(places[1].x, 210);
        assert_eq!(img.get_pixel(300, 90).0[3], 0, "below the shorter one");
    }

    #[test]
    fn overlapping_areas_join() {
        let parts = [area(0, 0, 100, 100, 10), area(50, 50, 100, 100, 20)];
        let (img, _) = compose(&parts, Composition::Original, 10);
        assert_eq!(img.dimensions(), (150, 150));
        assert_eq!(img.get_pixel(75, 75).0, [20, 20, 20, 255]);
    }
}
