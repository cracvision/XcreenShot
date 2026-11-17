//! The spotlight and lens effect on a selection (EDT-065): the outside dims and blurs,
//! the inside can grow in place, and a border can go around it. It is fixed into the
//! pixels.

use image::RgbaImage;
use serde_json::Value;

use crate::selection::Mask;

fn f(v: &Value, k: &str, d: f64) -> f64 {
    v.get(k).and_then(Value::as_f64).unwrap_or(d)
}

// Color planes indexed by channel.
#[allow(clippy::needless_range_loop)]
pub fn spotlight(img: &RgbaImage, mask: &Mask, e: &Value) -> RgbaImage {
    let (w, h) = img.dimensions();
    let dim = (f(e, "dim", 50.0) / 100.0).clamp(0.0, 1.0) as f32;
    let blur = f(e, "blur", 0.0).clamp(0.0, 100.0);
    let zoom = (f(e, "zoom", 100.0) / 100.0).clamp(1.0, 4.0);
    // Outside: blurred, then dimmed.
    let mut outside = img.clone();
    if blur > 0.0 {
        let mut planes: Vec<Vec<f32>> = (0..4)
            .map(|c| {
                outside
                    .pixels()
                    .map(|p| p.0[c] as f32 * if c < 3 { p.0[3] as f32 / 255.0 } else { 1.0 })
                    .collect()
            })
            .collect();
        for plane in planes.iter_mut() {
            crate::fx::blur_plane(plane, w as usize, h as usize, blur / 10.0);
        }
        for (i, p) in outside.pixels_mut().enumerate() {
            let a = planes[3][i].max(0.0);
            for c in 0..3 {
                p.0[c] = if a > 0.0 {
                    (planes[c][i] * 255.0 / a).round().clamp(0.0, 255.0) as u8
                } else {
                    0
                };
            }
            p.0[3] = a.round().clamp(0.0, 255.0) as u8;
        }
    }
    for p in outside.pixels_mut() {
        for c in 0..3 {
            p.0[c] = (p.0[c] as f32 * (1.0 - dim)).round() as u8;
        }
    }
    // Inside: the zone, grown around its center.
    let mut inside = img.clone();
    if zoom > 1.0 {
        if let Some(b) = mask.bounds() {
            let (cx, cy) = (b[0] as f64 + b[2] as f64 / 2.0, b[1] as f64 + b[3] as f64 / 2.0);
            for y in b[1]..b[1] + b[3] {
                for x in b[0]..b[0] + b[2] {
                    let sx = (cx + (x as f64 + 0.5 - cx) / zoom)
                        .floor()
                        .clamp(0.0, w as f64 - 1.0) as u32;
                    let sy = (cy + (y as f64 + 0.5 - cy) / zoom)
                        .floor()
                        .clamp(0.0, h as f64 - 1.0) as u32;
                    inside.put_pixel(x, y, *img.get_pixel(sx, sy));
                }
            }
        }
    }
    let mut out = RgbaImage::new(w, h);
    for (i, p) in out.pixels_mut().enumerate() {
        let k = mask.alpha[i] as f32 / 255.0;
        let (a, b) = (outside.as_raw(), inside.as_raw());
        for c in 0..4 {
            p.0[c] = (a[i * 4 + c] as f32 * (1.0 - k) + b[i * 4 + c] as f32 * k).round() as u8;
        }
    }
    // A border around the zone, outside it.
    let width = f(e, "borderWidth", 0.0).clamp(0.0, 50.0);
    if width > 0.0 {
        let c = e
            .get("borderColor")
            .and_then(Value::as_str)
            .and_then(crate::color::Rgba::parse)
            .unwrap_or(crate::color::Rgba::WHITE);
        let solid: Vec<bool> = mask.alpha.iter().map(|&a| a >= 128).collect();
        let d = crate::fx::distance_to(&solid, w as usize, h as usize, false);
        for (i, p) in out.pixels_mut().enumerate() {
            if !solid[i] && d[i] <= width as f32 {
                p.0 = [c.r, c.g, c.b, 255];
            }
        }
    }
    out
}
