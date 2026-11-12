//! Effects that change the image's outline (EDT-056, EDT-058, EDT-059): decorative
//! edges, perspective and skew, and a page curl.

use image::{imageops, Rgba as Px, RgbaImage};
use serde_json::{json, Value};

use crate::color::Rgba;

fn f(v: &Value, k: &str, d: f64) -> f64 {
    v.get(k).and_then(Value::as_f64).unwrap_or(d)
}
fn s<'a>(v: &'a Value, k: &str, d: &'a str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or(d)
}

/// A repeatable random sequence: the same seed draws the same torn edge everywhere
/// (EDT-056: the variant is kept in the file).
struct Rng(u64);
impl Rng {
    fn next(&mut self) -> f64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// How deep the edge cuts at `t` px along a side, 0..=size.
fn profile(style: &str, size: f64, len: usize, randomness: f64, seed: u64) -> Vec<f64> {
    let mut out = vec![0.0; len];
    match style {
        "wavy" => {
            for (t, d) in out.iter_mut().enumerate() {
                *d = size * (0.5 + 0.5 * (t as f64 * std::f64::consts::TAU / (size * 4.0)).sin());
            }
        }
        "sawtooth" => {
            for (t, d) in out.iter_mut().enumerate() {
                let u = (t as f64 / size) % 2.0;
                *d = size * (u - 1.0).abs();
            }
        }
        "scallop" => {
            let r = size / 2.0;
            for (t, d) in out.iter_mut().enumerate() {
                let u = t as f64 % size - r;
                *d = r - (r * r - u * u).max(0.0).sqrt() + r * 0.0;
            }
        }
        _ => {
            // Torn: a random walk every few pixels, smoothed between.
            let mut rng = Rng(seed.max(1));
            let step = 4usize;
            let k = (randomness / 100.0).clamp(0.0, 1.0);
            let mut v = size * 0.5;
            let mut knots = Vec::with_capacity(len / step + 2);
            for _ in 0..=len / step + 1 {
                v = (v + (rng.next() * 2.0 - 1.0) * size * 0.45 * (0.2 + k)).clamp(size * 0.1, size);
                knots.push(v);
            }
            for (t, d) in out.iter_mut().enumerate() {
                let (i, frac) = (t / step, (t % step) as f64 / step as f64);
                *d = knots[i] * (1.0 - frac) + knots[i + 1] * frac;
            }
        }
    }
    out
}

/// Decorative edges (EDT-056): torn, wavy, sawtooth, bevel, faded or scalloped, on
/// the chosen sides, with an optional shadow. Returns the image and where the
/// original's corner went.
pub fn edges(img: &RgbaImage, e: &Value) -> (RgbaImage, i64, i64) {
    let mut out = img.clone();
    let (w, h) = (img.width() as usize, img.height() as usize);
    let size = f(e, "size", 16.0).clamp(2.0, 80.0);
    let style = s(e, "style", "torn");
    let sides = e.get("sides");
    let side = |n: &str, d: bool| sides.and_then(|v| v.get(n)).and_then(Value::as_bool).unwrap_or(d);
    let seed = e.get("seed").and_then(Value::as_u64).unwrap_or(7);
    let randomness = f(e, "randomness", 50.0);
    // For each chosen side: its length, and how far a pixel is from it.
    let list: Vec<(&str, usize)> = [("top", w), ("bottom", w), ("left", h), ("right", h)]
        .into_iter()
        .filter(|(n, _)| side(n, *n == "bottom"))
        .collect();
    for (i, (name, len)) in list.iter().enumerate() {
        let p = profile(style, size, *len, randomness, seed.wrapping_add(i as u64 * 7919));
        for y in 0..h {
            for x in 0..w {
                let (t, d) = match *name {
                    "top" => (x, y as f64),
                    "bottom" => (x, (h - 1 - y) as f64),
                    "left" => (y, x as f64),
                    _ => (y, (w - 1 - x) as f64),
                };
                if d > size + 1.0 {
                    continue;
                }
                let px = out.get_pixel_mut(x as u32, y as u32);
                match style {
                    "fade" => {
                        let k = (d / size).clamp(0.0, 1.0);
                        px.0[3] = (px.0[3] as f64 * k).round() as u8;
                    }
                    "bevel" => {
                        // Lit from the top left: those sides lighter, the others darker.
                        let k = (1.0 - d / size).clamp(0.0, 1.0) * 0.35;
                        let lit = *name == "top" || *name == "left";
                        for c in 0..3 {
                            let v = px.0[c] as f64;
                            px.0[c] = if lit { v + (255.0 - v) * k } else { v * (1.0 - k) }.round() as u8;
                        }
                    }
                    _ => {
                        let keep = (d - p[t] + 0.5).clamp(0.0, 1.0);
                        px.0[3] = (px.0[3] as f64 * keep).round() as u8;
                    }
                }
            }
        }
    }
    // The edge's shadow: a small soft one in the chosen direction.
    if e.get("shadow").and_then(Value::as_bool).unwrap_or(false) {
        let sh = json!({
            "type": "shadow",
            "color": e.get("shadowColor").cloned().unwrap_or(json!("#000000")),
            "angle": f(e, "shadowAngle", 45.0),
            "distance": 4,
            "blur": 8,
            "opacity": 45,
        });
        let ctx_meta = json!({});
        let none = |_: &str| None;
        let ctx = crate::fx::Ctx {
            canvas_color: None,
            dpi: 96.0,
            meta: &ctx_meta,
            picture: &none,
        };
        let o = crate::fx::apply(&[sh], out, &ctx);
        return (o.image, o.origin[0], o.origin[1]);
    }
    (out, 0, 0)
}

/// The 3 × 3 projective transform taking the four `from` points to `to`.
// Row operations on a small matrix read best with indices.
#[allow(clippy::needless_range_loop)]
fn homography(from: [[f64; 2]; 4], to: [[f64; 2]; 4]) -> Option<[f64; 9]> {
    let mut a = [[0f64; 9]; 8];
    for i in 0..4 {
        let ([x, y], [u, v]) = (from[i], to[i]);
        a[2 * i] = [x, y, 1.0, 0.0, 0.0, 0.0, -u * x, -u * y, u];
        a[2 * i + 1] = [0.0, 0.0, 0.0, x, y, 1.0, -v * x, -v * y, v];
    }
    // Gauss-Jordan on the 8 × 9 augmented matrix.
    for col in 0..8 {
        let pivot = (col..8).max_by(|&i, &j| a[i][col].abs().total_cmp(&a[j][col].abs()))?;
        if a[pivot][col].abs() < 1e-12 {
            return None;
        }
        a.swap(col, pivot);
        let p = a[col][col];
        for k in col..9 {
            a[col][k] /= p;
        }
        for row in 0..8 {
            if row != col {
                let m = a[row][col];
                for k in col..9 {
                    a[row][k] -= m * a[col][k];
                }
            }
        }
    }
    let h: Vec<f64> = (0..8).map(|i| a[i][8]).collect();
    Some([h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7], 1.0])
}

fn map(h: &[f64; 9], x: f64, y: f64) -> [f64; 2] {
    let w = h[6] * x + h[7] * y + h[8];
    [(h[0] * x + h[1] * y + h[2]) / w, (h[3] * x + h[4] * y + h[5]) / w]
}

/// The image's color at `(x, y)`, between its pixels; transparent outside.
fn sample(img: &RgbaImage, x: f64, y: f64) -> Px<u8> {
    let (w, h) = (img.width() as i64, img.height() as i64);
    let (x, y) = (x - 0.5, y - 0.5);
    let (x0, y0) = (x.floor() as i64, y.floor() as i64);
    let (fx, fy) = (x - x0 as f64, y - y0 as f64);
    let at = |x: i64, y: i64| -> [f64; 4] {
        if x < 0 || y < 0 || x >= w || y >= h {
            [0.0; 4]
        } else {
            let p = img.get_pixel(x as u32, y as u32).0;
            let a = p[3] as f64;
            [p[0] as f64 * a, p[1] as f64 * a, p[2] as f64 * a, a]
        }
    };
    let (a, b, c, d) = (at(x0, y0), at(x0 + 1, y0), at(x0, y0 + 1), at(x0 + 1, y0 + 1));
    let mut o = [0f64; 4];
    for k in 0..4 {
        o[k] =
            a[k] * (1.0 - fx) * (1.0 - fy) + b[k] * fx * (1.0 - fy) + c[k] * (1.0 - fx) * fy + d[k] * fx * fy;
    }
    if o[3] <= 0.0 {
        return Px([0, 0, 0, 0]);
    }
    Px([o[0] / o[3], o[1] / o[3], o[2] / o[3], o[3]].map(|v| v.round().clamp(0.0, 255.0) as u8))
}

/// Perspective (a turn in 3D) or skew (EDT-058), on a transparent background. Returns
/// the image and where the original's top-left corner went.
pub fn perspective(img: &RgbaImage, e: &Value) -> (RgbaImage, i64, i64) {
    let (w, h) = (img.width() as f64, img.height() as f64);
    let hz = f(e, "horizontal", 0.0).clamp(-45.0, 45.0).to_radians();
    let vt = f(e, "vertical", 0.0).clamp(-45.0, 45.0).to_radians();
    let src = [[0.0, 0.0], [w, 0.0], [w, h], [0.0, h]];
    let dst: [[f64; 2]; 4] = if s(e, "style", "perspective") == "skew" {
        // A flat slant: rows slide sideways, columns up or down.
        src.map(|[x, y]| [x + (y - h / 2.0) * hz.tan(), y + (x - w / 2.0) * vt.tan()])
    } else {
        let depth = (f(e, "depth", 50.0) / 100.0).clamp(0.0, 1.0);
        let focal = w.max(h) * (1.2 + (1.0 - depth) * 8.0);
        src.map(|[x, y]| {
            let (cx, cy) = (x - w / 2.0, y - h / 2.0);
            // Turn around the vertical axis, then the horizontal one.
            let (x1, z1) = (cx * hz.cos(), -cx * hz.sin());
            let (y2, z2) = (cy * vt.cos() - z1 * vt.sin(), cy * vt.sin() + z1 * vt.cos());
            let k = focal / (focal + z2);
            [x1 * k + w / 2.0, y2 * k + h / 2.0]
        })
    };
    let min_x = dst.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min).floor();
    let min_y = dst.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min).floor();
    let max_x = dst.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max).ceil();
    let max_y = dst.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max).ceil();
    let dst = dst.map(|[x, y]| [x - min_x, y - min_y]);
    let (ow, oh) = ((max_x - min_x).max(1.0) as u32, (max_y - min_y).max(1.0) as u32);
    let Some(back) = homography(dst, src) else {
        return (img.clone(), 0, 0);
    };
    let mut out = RgbaImage::new(ow, oh);
    for (x, y, p) in out.enumerate_pixels_mut() {
        let [u, v] = map(&back, x as f64 + 0.5, y as f64 + 0.5);
        if u >= -1.0 && v >= -1.0 && u <= w + 1.0 && v <= h + 1.0 {
            *p = sample(img, u, v);
        }
    }
    (out, (dst[0][0]).round() as i64, (dst[0][1]).round() as i64)
}

/// A page curl (EDT-059): a corner folds over, showing its back; what was behind it
/// is transparent.
pub fn page_curl(img: RgbaImage, e: &Value) -> RgbaImage {
    let corner = s(e, "corner", "bottomRight").to_string();
    // Work on the bottom-right corner: flip there and back.
    let flip_x = corner.ends_with("Left");
    let flip_y = corner.starts_with("top");
    let mut work = img;
    if flip_x {
        imageops::flip_horizontal_in_place(&mut work);
    }
    if flip_y {
        imageops::flip_vertical_in_place(&mut work);
    }
    let (w, h) = (work.width() as f64, work.height() as f64);
    let size = (f(e, "size", 20.0).clamp(5.0, 50.0) / 100.0) * w.min(h);
    let back = e
        .get("color")
        .and_then(Value::as_str)
        .and_then(Rgba::parse)
        .unwrap_or(Rgba {
            r: 217,
            g: 217,
            b: 217,
            a: 255,
        });
    // The fold: the line x + y = w + h - size. Beyond it, nothing; the flap is the
    // cut triangle mirrored over the line.
    let line = w + h - size;
    let mut out = work.clone();
    for (x, y, p) in out.enumerate_pixels_mut() {
        let (px, py) = (x as f64 + 0.5, y as f64 + 0.5);
        let d = px + py - line;
        if d > 0.0 {
            p.0[3] = (p.0[3] as f64 * (0.5 - d).clamp(0.0, 1.0)).round() as u8;
        }
    }
    // A soft shadow under the flap, then the flap.
    for (x, y, p) in out.enumerate_pixels_mut() {
        let (px, py) = (x as f64 + 0.5, y as f64 + 0.5);
        let inside = px >= w - size && py >= h - size && px + py <= line;
        let near = px >= w - size - 6.0 && py >= h - size - 6.0 && px + py <= line + 0.5;
        if inside {
            // Lighter at the fold, darker at its tip.
            let t = ((line - (px + py)) / size).clamp(0.0, 1.0);
            let k = 1.0 - 0.25 * t;
            let flap = Px([
                (back.r as f64 * k) as u8,
                (back.g as f64 * k) as u8,
                (back.b as f64 * k) as u8,
                255,
            ]);
            *p = flap;
        } else if near && p.0[3] > 0 {
            let dist = ((w - size - px).max(0.0)).max((h - size - py).max(0.0));
            let k = 0.25 * (1.0 - dist / 6.0).clamp(0.0, 1.0);
            for c in 0..3 {
                p.0[c] = (p.0[c] as f64 * (1.0 - k)).round() as u8;
            }
        }
    }
    if flip_y {
        imageops::flip_vertical_in_place(&mut out);
    }
    if flip_x {
        imageops::flip_horizontal_in_place(&mut out);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn torn_bottom_edge_is_irregular_and_the_rest_straight() {
        let img = RgbaImage::from_pixel(200, 100, Px([255, 255, 255, 255]));
        let (out, _, _) = edges(&img, &json!({"style": "torn", "size": 16, "seed": 42}));
        // Bottom row: some transparent, the top row whole.
        let bottom: Vec<u8> = (0..200).map(|x| out.get_pixel(x, 99).0[3]).collect();
        assert!(bottom.contains(&0));
        assert!((0..200).all(|x| out.get_pixel(x, 0).0[3] == 255));
        // The same seed, the same edge.
        let (again, _, _) = edges(&img, &json!({"style": "torn", "size": 16, "seed": 42}));
        assert_eq!(out, again);
    }

    #[test]
    fn perspective_makes_a_trapezoid_and_skew_slants() {
        let img = RgbaImage::from_pixel(200, 100, Px([255, 0, 0, 255]));
        let (p, _, _) = perspective(
            &img,
            &json!({"style": "perspective", "horizontal": 20, "depth": 80}),
        );
        // One side shorter than the other: the corners of the result are empty.
        assert_eq!(p.get_pixel(0, 0).0[3].min(p.get_pixel(p.width() - 1, 0).0[3]), 0);
        let (k, _, _) = perspective(&img, &json!({"style": "skew", "vertical": -10}));
        assert!(k.height() > 100, "slanted: {:?}", k.dimensions());
    }

    #[test]
    fn a_page_curl_leaves_the_corner_transparent() {
        let img = RgbaImage::from_pixel(100, 100, Px([0, 0, 255, 255]));
        let out = page_curl(img.clone(), &json!({"corner": "bottomRight", "size": 20}));
        assert_eq!(out.get_pixel(99, 99).0[3], 0);
        assert_eq!(out.get_pixel(2, 2).0, [0, 0, 255, 255]);
        let tl = page_curl(img, &json!({"corner": "topLeft", "size": 20}));
        assert_eq!(tl.get_pixel(0, 0).0[3], 0);
        assert_eq!(tl.get_pixel(99, 99).0, [0, 0, 255, 255]);
    }
}
