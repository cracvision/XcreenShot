//! Pixel tools (EDT-047 Fill, EDT-048 Eraser; ADR 0005): they work on the image as it
//! shows without objects (`image_in_base`), or on a floating image's picture, and give
//! back the changed rectangle as new pixels.

use crate::color::Rgba;
use image::RgbaImage;
use tiny_skia::{LineCap, LineJoin, PathBuilder, Pixmap, Stroke, Transform};

/// A rectangle of pixels: `[x, y, w, h]`.
pub type Area = [u32; 4];

/// The pixels a fill reaches from `seed`: those whose channels all differ from the
/// seed's by at most `tolerance` % of 255; touching it (4 ways), or anywhere with
/// `global`. Returns the mask and its bounds, or `None` if `seed` is outside.
pub fn reach(img: &RgbaImage, seed: (u32, u32), tolerance: f64, global: bool) -> Option<(Vec<bool>, Area)> {
    let (w, h) = img.dimensions();
    if seed.0 >= w || seed.1 >= h {
        return None;
    }
    let target = img.get_pixel(seed.0, seed.1).0;
    let tol = (tolerance.clamp(0.0, 100.0) / 100.0 * 255.0).round() as i32;
    let like = |p: [u8; 4]| (0..4).all(|c| (p[c] as i32 - target[c] as i32).abs() <= tol);
    let mut mask = vec![false; (w * h) as usize];
    if global {
        for (i, p) in img.pixels().enumerate() {
            mask[i] = like(p.0);
        }
    } else {
        let mut stack = vec![seed];
        mask[(seed.1 * w + seed.0) as usize] = true;
        while let Some((x, y)) = stack.pop() {
            let near = [
                (x.wrapping_sub(1), y),
                (x + 1, y),
                (x, y.wrapping_sub(1)),
                (x, y + 1),
            ];
            for (nx, ny) in near {
                if nx >= w || ny >= h {
                    continue;
                }
                let i = (ny * w + nx) as usize;
                if !mask[i] && like(img.get_pixel(nx, ny).0) {
                    mask[i] = true;
                    stack.push((nx, ny));
                }
            }
        }
    }
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
    for y in 0..h {
        for x in 0..w {
            if mask[(y * w + x) as usize] {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    (x1 >= x0 && y1 >= y0).then_some((mask, [x0, y0, x1 - x0 + 1, y1 - y0 + 1]))
}

/// Paints `color` over the masked pixels at `opacity` %; `None` makes them transparent.
pub fn fill(img: &mut RgbaImage, mask: &[bool], color: Option<Rgba>, opacity: f64) {
    let k = (opacity / 100.0).clamp(0.0, 1.0);
    let w = img.width();
    for (i, on) in mask.iter().enumerate() {
        if !on {
            continue;
        }
        let (x, y) = (i as u32 % w, i as u32 / w);
        let p = img.get_pixel_mut(x, y);
        match color {
            Some(c) => {
                let a = k * c.a as f64 / 255.0;
                let mix = |old: u8, new: u8| (old as f64 * (1.0 - a) + new as f64 * a).round() as u8;
                let alpha = (p.0[3] as f64 + (255.0 - p.0[3] as f64) * a).round() as u8;
                p.0 = [mix(p.0[0], c.r), mix(p.0[1], c.g), mix(p.0[2], c.b), alpha];
            }
            None => p.0[3] = (p.0[3] as f64 * (1.0 - k)).round() as u8,
        }
    }
}

/// The part `area` of `img`.
pub fn cut(img: &RgbaImage, area: Area) -> RgbaImage {
    image::imageops::crop_imm(img, area[0], area[1], area[2], area[3]).to_image()
}

/// Erases along `points` with a tip of `width` px (round or square) at `opacity` %:
/// the pixels lose their alpha, so the canvas color shows (EDT-048). Returns the
/// rectangle it touched, or `None` if it is outside.
pub fn erase(
    img: &mut RgbaImage,
    points: &[[f64; 2]],
    width: f64,
    square: bool,
    opacity: f64,
    antialias: bool,
) -> Option<Area> {
    let (w, h) = img.dimensions();
    let r = width / 2.0 + 1.0;
    let xs = points.iter().map(|p| p[0]);
    let ys = points.iter().map(|p| p[1]);
    let x0 = (xs.clone().fold(f64::INFINITY, f64::min) - r).floor().max(0.0) as u32;
    let y0 = (ys.clone().fold(f64::INFINITY, f64::min) - r).floor().max(0.0) as u32;
    let x1 = ((xs.fold(f64::NEG_INFINITY, f64::max) + r).ceil() as u32).min(w);
    let y1 = ((ys.fold(f64::NEG_INFINITY, f64::max) + r).ceil() as u32).min(h);
    if points.is_empty() || x1 <= x0 || y1 <= y0 {
        return None;
    }
    let (aw, ah) = (x1 - x0, y1 - y0);
    let mut mask = Pixmap::new(aw, ah)?;
    let mut pb = PathBuilder::new();
    let first = points[0];
    pb.move_to((first[0] - x0 as f64) as f32, (first[1] - y0 as f64) as f32);
    for p in points.iter().skip(1) {
        pb.line_to((p[0] - x0 as f64) as f32, (p[1] - y0 as f64) as f32);
    }
    if points.len() == 1 {
        // A single press still erases a dot.
        pb.line_to(
            (first[0] - x0 as f64 + 0.01) as f32,
            (first[1] - y0 as f64) as f32,
        );
    }
    let path = pb.finish()?;
    let mut ink = tiny_skia::Paint::default();
    ink.set_color_rgba8(0, 0, 0, 255);
    ink.anti_alias = antialias;
    let st = Stroke {
        width: width.max(1.0) as f32,
        line_cap: if square { LineCap::Square } else { LineCap::Round },
        line_join: if square { LineJoin::Miter } else { LineJoin::Round },
        ..Stroke::default()
    };
    mask.stroke_path(&path, &ink, &st, Transform::identity(), None);
    let k = (opacity / 100.0).clamp(0.0, 1.0);
    for y in 0..ah {
        for x in 0..aw {
            let m = mask.pixel(x, y).map(|p| p.alpha()).unwrap_or(0) as f64 / 255.0;
            if m > 0.0 {
                let p = img.get_pixel_mut(x0 + x, y0 + y);
                p.0[3] = (p.0[3] as f64 * (1.0 - m * k)).round() as u8;
            }
        }
    }
    Some([x0, y0, aw, ah])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn white_with_blue_square() -> RgbaImage {
        RgbaImage::from_fn(40, 20, |x, y| {
            if (5..10).contains(&x) && (5..10).contains(&y) || (30..35).contains(&x) && (5..10).contains(&y) {
                image::Rgba([0, 0, 255, 255])
            } else {
                image::Rgba([255, 255, 255, 255])
            }
        })
    }

    #[test]
    fn fills_touching_pixels_or_all_of_that_color() {
        let img = white_with_blue_square();
        let (mask, area) = reach(&img, (6, 6), 0.0, false).unwrap();
        assert_eq!(area, [5, 5, 5, 5], "only the touching blue square");
        assert!(!mask[(6 * 40 + 31) as usize]);
        let (mask, _) = reach(&img, (6, 6), 0.0, true).unwrap();
        assert!(mask[(6 * 40 + 31) as usize], "global reaches the other one");
        let mut out = img.clone();
        fill(&mut out, &mask, Rgba::parse("#00FF00"), 100.0);
        assert_eq!(out.get_pixel(31, 6).0, [0, 255, 0, 255]);
        assert_eq!(out.get_pixel(20, 15).0, [255, 255, 255, 255]);
        // Transparent: the white background goes.
        let (bg, _) = reach(&img, (0, 0), 0.0, false).unwrap();
        fill(&mut out, &bg, None, 100.0);
        assert_eq!(out.get_pixel(0, 0).0[3], 0);
    }

    #[test]
    fn erasing_makes_pixels_transparent_along_the_stroke() {
        let mut img = RgbaImage::from_pixel(50, 20, image::Rgba([255, 0, 0, 255]));
        let area = erase(&mut img, &[[5.0, 10.0], [45.0, 10.0]], 6.0, false, 100.0, true).unwrap();
        assert_eq!(img.get_pixel(25, 10).0[3], 0);
        assert_eq!(img.get_pixel(25, 1).0[3], 255, "away from the stroke");
        assert!(area[0] <= 2 && area[2] >= 40);
    }
}
