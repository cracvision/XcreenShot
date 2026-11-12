//! Document effects (EDT-053 to EDT-068): a stack of effects applied to the final
//! image when it leaves the editor (export, copy, share, print) and shown in the
//! editor. The `.xshot` keeps the stack and the image without them. Each entry is
//! `{"type": …, "enabled": bool, …its settings}`; types this build does not know are
//! kept and skipped.

use image::{imageops, Rgba as Px, RgbaImage};
use serde_json::Value;

use crate::color::Rgba;

/// Effect types, in the order of the panel (EDT-053).
pub const TYPES: &[&str] = &[
    "border",
    "edges",
    "shadow",
    "perspective",
    "pageCurl",
    "reflection",
    "fade",
    "filters",
    "color",
    "replace",
    "watermark",
    "details",
    "trim",
    "corners",
    "scale",
    "rotate",
];

/// Types that change the size of the result (EDT-053): they grow the export, not the
/// editable canvas.
pub const GROWING: &[&str] = &[
    "border",
    "edges",
    "shadow",
    "perspective",
    "pageCurl",
    "reflection",
    "details",
    "trim",
    "scale",
    "rotate",
];

/// What effects need besides the pixels.
pub struct Ctx<'a> {
    pub canvas_color: Option<Rgba>,
    pub dpi: f64,
    /// The capture's metadata (CAP-056), for Capture details.
    pub meta: &'a Value,
    /// Pictures of the document (`images/`), for image watermarks.
    pub picture: &'a dyn Fn(&str) -> Option<RgbaImage>,
}

/// The result and where the canvas's top-left corner landed in it.
pub struct Output {
    pub image: RgbaImage,
    pub origin: [i64; 2],
}

fn f(v: &Value, k: &str, d: f64) -> f64 {
    v.get(k).and_then(Value::as_f64).unwrap_or(d)
}
fn b(v: &Value, k: &str, d: bool) -> bool {
    v.get(k).and_then(Value::as_bool).unwrap_or(d)
}
fn s<'a>(v: &'a Value, k: &str, d: &'a str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or(d)
}
fn color(v: &Value, k: &str, d: &str) -> Option<Rgba> {
    match v.get(k) {
        Some(Value::Null) => None,
        Some(Value::String(c)) => Rgba::parse(c),
        _ => Rgba::parse(d),
    }
}

/// Whether an entry is on.
pub fn enabled(e: &Value) -> bool {
    b(e, "enabled", true)
}

/// Effects applied to the image only, before the objects are drawn: Filters with
/// "Base image only" and watermarks Below (EDT-062, EDT-066).
pub fn on_base(e: &Value) -> bool {
    match s(e, "type", "") {
        "filters" => b(e, "baseOnly", false),
        "watermark" => s(e, "layer", "above") == "below",
        _ => false,
    }
}

/// Checks an entry's settings (EDT-053): its type and the ranges of its values.
pub fn check(e: &Value) -> Result<(), String> {
    let t = s(e, "type", "");
    if !TYPES.contains(&t) && t != "spotlight" {
        return Err(format!("there is no effect \"{t}\""));
    }
    let range = |k: &str, lo: f64, hi: f64| -> Result<(), String> {
        match e.get(k).and_then(Value::as_f64) {
            Some(x) if !(lo..=hi).contains(&x) => Err(format!("{t}: {k} must be between {lo} and {hi}")),
            _ => Ok(()),
        }
    };
    match t {
        "border" => range("width", 1.0, 100.0)?,
        "edges" => {
            range("size", 2.0, 80.0)?;
            range("randomness", 0.0, 100.0)?;
        }
        "shadow" => {
            range("opacity", 0.0, 100.0)?;
            range("distance", 0.0, 100.0)?;
            range("blur", 0.0, 100.0)?;
            range("angle", 0.0, 360.0)?;
        }
        "perspective" => {
            range("horizontal", -45.0, 45.0)?;
            range("vertical", -45.0, 45.0)?;
            range("depth", 0.0, 100.0)?;
        }
        "pageCurl" => range("size", 5.0, 50.0)?,
        "reflection" => {
            range("size", 10.0, 100.0)?;
            range("opacity", 0.0, 100.0)?;
            range("gap", 0.0, 50.0)?;
        }
        "fade" => {
            range("size", 2.0, 200.0)?;
            range("intensity", 0.0, 100.0)?;
        }
        "color" => {
            range("brightness", -100.0, 100.0)?;
            range("contrast", -100.0, 100.0)?;
            range("hue", -180.0, 180.0)?;
            range("saturation", -100.0, 100.0)?;
            range("gamma", 0.1, 5.0)?;
        }
        "replace" => range("tolerance", 0.0, 100.0)?,
        "watermark" => {
            range("scale", 5.0, 200.0)?;
            range("opacity", 0.0, 100.0)?;
            range("rotation", -90.0, 90.0)?;
            range("margin", 0.0, 200.0)?;
        }
        "scale" => range("value", 1.0, 100_000.0)?,
        _ => {}
    }
    Ok(())
}

/// Applies the enabled effects of `stack` that work on the whole image, in order.
/// `image` is the canvas with the objects drawn (or, for a floating image, its
/// picture).
pub fn apply(stack: &[Value], image: RgbaImage, ctx: &Ctx) -> Output {
    let mut out = Output {
        image,
        origin: [0, 0],
    };
    for e in stack.iter().filter(|e| enabled(e) && !on_base(e)) {
        out = apply_one(e, out, ctx);
    }
    out
}

/// Applies the enabled base-only effects (EDT-062, EDT-066) to the image under the
/// objects.
pub fn apply_base(stack: &[Value], image: RgbaImage, ctx: &Ctx) -> RgbaImage {
    let mut img = image;
    for e in stack.iter().filter(|e| enabled(e) && on_base(e)) {
        img = apply_one(
            e,
            Output {
                image: img,
                origin: [0, 0],
            },
            ctx,
        )
        .image;
    }
    img
}

fn apply_one(e: &Value, out: Output, ctx: &Ctx) -> Output {
    let Output { image, origin } = out;
    let (w, h) = (image.width() as i64, image.height() as i64);
    let shift = |o: Output, dx: i64, dy: i64| Output {
        image: o.image,
        origin: [origin[0] + dx, origin[1] + dy],
    };
    match s(e, "type", "") {
        "border" => {
            let (img, d) = border(&image, e);
            shift(Output { image: img, origin }, d, d)
        }
        "shadow" => {
            let (img, dx, dy) = shadow(&image, e, ctx.canvas_color);
            shift(Output { image: img, origin }, dx, dy)
        }
        "reflection" => Output {
            image: reflection(&image, e),
            origin,
        },
        "fade" => Output {
            image: fade(image, e),
            origin,
        },
        "filters" => Output {
            image: filters(image, e),
            origin,
        },
        "color" => Output {
            image: color_adjust(image, e),
            origin,
        },
        "replace" => Output {
            image: replace(image, e),
            origin,
        },
        "corners" => Output {
            image: corners(image, e),
            origin,
        },
        "trim" => {
            let (img, x0, y0) = trim(&image, e);
            shift(Output { image: img, origin }, -x0, -y0)
        }
        "scale" => {
            let img = scale(&image, e);
            let k = img.width() as f64 / w.max(1) as f64;
            Output {
                image: img,
                origin: [
                    (origin[0] as f64 * k).round() as i64,
                    (origin[1] as f64 * k).round() as i64,
                ],
            }
        }
        "rotate" => {
            let img = match f(e, "angle", 90.0).round() as i64 {
                180 => imageops::rotate180(&image),
                270 => imageops::rotate270(&image),
                _ => imageops::rotate90(&image),
            };
            Output { image: img, origin }
        }
        "edges" => {
            let (img, dx, dy) = crate::fx_shapes::edges(&image, e);
            shift(Output { image: img, origin }, dx, dy)
        }
        "perspective" => {
            let (img, dx, dy) = crate::fx_shapes::perspective(&image, e);
            shift(Output { image: img, origin }, dx, dy)
        }
        "pageCurl" => Output {
            image: crate::fx_shapes::page_curl(image, e),
            origin,
        },
        "watermark" => Output {
            image: crate::fx_marks::watermark(image, e, ctx),
            origin,
        },
        "details" => {
            let (img, dy) = crate::fx_marks::details(image, e, ctx);
            shift(Output { image: img, origin }, 0, dy)
        }
        _ => {
            let _ = h;
            Output { image, origin }
        }
    }
}

// ---------------------------------------------------------------------------
// Border, shadow, reflection, fade (EDT-055, EDT-057, EDT-060, EDT-061)

/// A frame around the image: outside (it grows by the width) or inside. It follows
/// transparent corners (EDT-055, EDT-075). Returns the image and how much it grew on
/// each side.
fn border(img: &RgbaImage, e: &Value) -> (RgbaImage, i64) {
    let c = color(e, "color", "#1F2430").unwrap_or(Rgba {
        r: 31,
        g: 36,
        b: 48,
        a: 255,
    });
    let width = f(e, "width", 2.0).clamp(1.0, 100.0);
    let inside = s(e, "position", "outside") == "inside";
    let pad = if inside { 0 } else { width.ceil() as i64 };
    let (w, h) = (img.width() as i64, img.height() as i64);
    let (ow, oh) = ((w + 2 * pad) as usize, (h + 2 * pad) as usize);
    // Opaque pixels of the image, in the result's coordinates.
    let mut solid = vec![false; ow * oh];
    for (x, y, p) in img.enumerate_pixels() {
        solid[(y as i64 + pad) as usize * ow + (x as i64 + pad) as usize] = p.0[3] >= 128;
    }
    let mut out = RgbaImage::new(ow as u32, oh as u32);
    imageops::overlay(&mut out, img, pad, pad);
    let frame = Px([c.r, c.g, c.b, c.a]);
    if inside {
        // Within `width` of a transparent pixel or of the image's edge.
        let empty: Vec<bool> = solid.iter().map(|s| !s).collect();
        let d = distance_to(&empty, ow, oh, true);
        for (i, p) in out.pixels_mut().enumerate() {
            if solid[i] && d[i] <= width as f32 {
                *p = over(*p, frame);
            }
        }
    } else {
        // A rectangular image gets square corners; one with transparent corners
        // (rounded, EDT-075) a frame that follows them.
        let corners_solid = [(0, 0), (w - 1, 0), (0, h - 1), (w - 1, h - 1)]
            .iter()
            .all(|&(x, y)| img.get_pixel(x as u32, y as u32).0[3] >= 128);
        let d = if corners_solid {
            chessboard_to(&solid, ow, oh)
        } else {
            distance_to(&solid, ow, oh, false)
        };
        for (i, p) in out.pixels_mut().enumerate() {
            if !solid[i] && d[i] <= width as f32 {
                // Under what the image has there (its soft edges), over nothing.
                *p = over(frame, *p);
            }
        }
    }
    (out, pad)
}

/// The largest of the horizontal and vertical distances to the nearest `target`
/// cell: frames with square corners.
fn chessboard_to(target: &[bool], w: usize, h: usize) -> Vec<f32> {
    const BIG: f32 = 1e9;
    let mut d: Vec<f32> = target.iter().map(|&t| if t { 0.0 } else { BIG }).collect();
    for pass in 0..2 {
        let ys: Vec<usize> = if pass == 0 {
            (0..h).collect()
        } else {
            (0..h).rev().collect()
        };
        let xs: Vec<usize> = if pass == 0 {
            (0..w).collect()
        } else {
            (0..w).rev().collect()
        };
        let step: i64 = if pass == 0 { -1 } else { 1 };
        for &y in &ys {
            for &x in &xs {
                let i = y * w + x;
                let mut v = d[i];
                for (dx, dy) in [(step, 0), (0, step), (step, step), (-step, step)] {
                    let (nx, ny) = (x as i64 + dx, y as i64 + dy);
                    if nx >= 0 && ny >= 0 && nx < w as i64 && ny < h as i64 {
                        v = v.min(d[ny as usize * w + nx as usize] + 1.0);
                    }
                }
                d[i] = v;
            }
        }
    }
    d
}

/// For each cell, about how far (in cells) the nearest `target` cell is: a two-pass
/// chamfer distance. With `edge`, outside the grid counts as a target.
pub(crate) fn distance_to(target: &[bool], w: usize, h: usize, edge: bool) -> Vec<f32> {
    const BIG: f32 = 1e9;
    let mut d: Vec<f32> = target.iter().map(|&t| if t { 0.0 } else { BIG }).collect();
    let get = |d: &Vec<f32>, x: i64, y: i64| -> f32 {
        if x < 0 || y < 0 || x >= w as i64 || y >= h as i64 {
            if edge {
                0.0
            } else {
                BIG
            }
        } else {
            d[y as usize * w + x as usize]
        }
    };
    let (a, b) = (1.0f32, std::f32::consts::SQRT_2);
    for y in 0..h as i64 {
        for x in 0..w as i64 {
            let i = y as usize * w + x as usize;
            let v = d[i]
                .min(get(&d, x - 1, y) + a)
                .min(get(&d, x, y - 1) + a)
                .min(get(&d, x - 1, y - 1) + b)
                .min(get(&d, x + 1, y - 1) + b);
            d[i] = v;
        }
    }
    for y in (0..h as i64).rev() {
        for x in (0..w as i64).rev() {
            let i = y as usize * w + x as usize;
            let v = d[i]
                .min(get(&d, x + 1, y) + a)
                .min(get(&d, x, y + 1) + a)
                .min(get(&d, x + 1, y + 1) + b)
                .min(get(&d, x - 1, y + 1) + b);
            d[i] = v;
        }
    }
    d
}

/// A shadow behind the image (EDT-057): direction (angle clockwise from the right;
/// distance 0 = all around), color, opacity, blur; on a transparent background or
/// the canvas color. Returns the image and where the original's corner went.
fn shadow(img: &RgbaImage, e: &Value, canvas: Option<Rgba>) -> (RgbaImage, i64, i64) {
    let c = color(e, "color", "#000000").unwrap_or(Rgba {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    });
    let a = f(e, "angle", 45.0).to_radians();
    let dist = f(e, "distance", 10.0).clamp(0.0, 100.0);
    let blur = f(e, "blur", 20.0).clamp(0.0, 100.0);
    let opacity = (f(e, "opacity", 50.0) / 100.0).clamp(0.0, 1.0) as f32 * c.a as f32 / 255.0;
    let (dx, dy) = ((a.cos() * dist).round() as i64, (a.sin() * dist).round() as i64);
    // Room for the blur; none without it.
    let m = if blur > 0.0 {
        (blur * 1.5).ceil() as i64 + 1
    } else {
        0
    };
    let (w, h) = (img.width() as i64, img.height() as i64);
    // The result covers the image and its shadow.
    let x0 = 0.min(dx - m);
    let y0 = 0.min(dy - m);
    let x1 = w.max(w + dx + m);
    let y1 = h.max(h + dy + m);
    let (ow, oh) = ((x1 - x0) as usize, (y1 - y0) as usize);
    let mut alpha = vec![0f32; ow * oh];
    for (i, p) in img.pixels().enumerate() {
        let (x, y) = (i as i64 % w + dx - x0, i as i64 / w + dy - y0);
        alpha[y as usize * ow + x as usize] = p.0[3] as f32;
    }
    blur_plane(&mut alpha, ow, oh, blur / 2.0);
    let background = (s(e, "background", "transparent") == "canvas")
        .then_some(canvas)
        .flatten();
    let mut out = match background {
        Some(bg) => RgbaImage::from_pixel(ow as u32, oh as u32, Px([bg.r, bg.g, bg.b, bg.a])),
        None => RgbaImage::new(ow as u32, oh as u32),
    };
    for (i, p) in out.pixels_mut().enumerate() {
        let sa = (alpha[i] * opacity / 255.0).clamp(0.0, 1.0);
        *p = over(*p, Px([c.r, c.g, c.b, (sa * 255.0).round() as u8]));
    }
    imageops::overlay(&mut out, img, -x0, -y0);
    (out, -x0, -y0)
}

/// `top` over `bottom`, straight alpha.
pub(crate) fn over(bottom: Px<u8>, top: Px<u8>) -> Px<u8> {
    let ta = top.0[3] as f32 / 255.0;
    let ba = bottom.0[3] as f32 / 255.0;
    let a = ta + ba * (1.0 - ta);
    if a <= 0.0 {
        return Px([0, 0, 0, 0]);
    }
    let mix = |t: u8, b: u8| ((t as f32 * ta + b as f32 * ba * (1.0 - ta)) / a).round() as u8;
    Px([
        mix(top.0[0], bottom.0[0]),
        mix(top.0[1], bottom.0[1]),
        mix(top.0[2], bottom.0[2]),
        (a * 255.0).round() as u8,
    ])
}

/// A Gaussian of standard deviation `sigma` on one plane, as three box blurs.
pub(crate) fn blur_plane(v: &mut [f32], w: usize, h: usize, sigma: f64) {
    if sigma < 0.3 || w == 0 || h == 0 {
        return;
    }
    let r = (((4.0 * sigma * sigma + 1.0).sqrt().round() as i64) / 2).max(1);
    let mut tmp = vec![0f32; v.len()];
    for _ in 0..3 {
        for (horizontal, len, lines) in [(true, w, h), (false, h, w)] {
            let len = len as i64;
            for line in 0..lines {
                let idx = |i: i64| -> usize {
                    if horizontal {
                        line * w + i as usize
                    } else {
                        i as usize * w + line
                    }
                };
                let get = |v: &[f32], i: i64| if i < 0 || i >= len { 0.0 } else { v[idx(i)] };
                let mut sum: f32 = (-r..=r).map(|i| get(v, i)).sum();
                for i in 0..len {
                    tmp[idx(i)] = sum / (2 * r + 1) as f32;
                    sum += get(v, i + r + 1) - get(v, i - r);
                }
            }
            v.copy_from_slice(&tmp);
        }
    }
}

/// A mirrored copy under the image that fades out (EDT-060).
fn reflection(img: &RgbaImage, e: &Value) -> RgbaImage {
    let (w, h) = img.dimensions();
    let size = ((f(e, "size", 30.0).clamp(10.0, 100.0) / 100.0) * h as f64).round() as u32;
    let gap = f(e, "gap", 0.0).clamp(0.0, 50.0).round() as u32;
    let start = (f(e, "opacity", 40.0).clamp(0.0, 100.0) / 100.0) as f32;
    let mut out = RgbaImage::new(w, h + gap + size);
    imageops::overlay(&mut out, img, 0, 0);
    for y in 0..size {
        // The reflection's first row is the image's last.
        let src_y = h - 1 - y;
        let k = start * (1.0 - (y + 1) as f32 / size.max(1) as f32);
        for x in 0..w {
            let mut p = *img.get_pixel(x, src_y);
            p.0[3] = (p.0[3] as f32 * k).round() as u8;
            out.put_pixel(x, h + gap + y, p);
        }
    }
    out
}

/// The edges turn transparent gradually (EDT-061).
fn fade(mut img: RgbaImage, e: &Value) -> RgbaImage {
    let (w, h) = (img.width() as f64, img.height() as f64);
    let size = f(e, "size", 40.0).clamp(2.0, 200.0);
    let k = (f(e, "intensity", 100.0) / 100.0).clamp(0.0, 1.0);
    let sides = e.get("sides");
    let side = |n: &str| {
        sides
            .and_then(|v| v.get(n))
            .and_then(Value::as_bool)
            .unwrap_or(true)
    };
    let (top, bottom, left, right) = (side("top"), side("bottom"), side("left"), side("right"));
    for (x, y, p) in img.enumerate_pixels_mut() {
        // Pixels on the edge are 0 away from it: they turn fully transparent.
        let (x, y) = (x as f64, y as f64);
        let mut d = f64::INFINITY;
        if top {
            d = d.min(y);
        }
        if bottom {
            d = d.min(h - 1.0 - y);
        }
        if left {
            d = d.min(x);
        }
        if right {
            d = d.min(w - 1.0 - x);
        }
        let t = (d / size).clamp(0.0, 1.0);
        // Smooth, transparent at the very edge.
        let keep = 1.0 - k * (1.0 - t * t * (3.0 - 2.0 * t));
        p.0[3] = (p.0[3] as f64 * keep).round() as u8;
    }
    img
}

// ---------------------------------------------------------------------------
// Filters, color adjustment, color replacement (EDT-062 to EDT-064)

/// One filter's switch and value: `{"on": bool, "value": n}`, or a bare switch.
fn filter_on(e: &Value, name: &str) -> Option<f64> {
    let v = e.get(name)?;
    match v {
        Value::Bool(true) => Some(0.0),
        Value::Object(_) if b(v, "on", false) => Some(f(v, "value", 0.0)),
        _ => None,
    }
}

fn filters(mut img: RgbaImage, e: &Value) -> RgbaImage {
    if let Some(n) = filter_on(e, "soften") {
        img = box_blur_image(&img, n.clamp(1.0, 10.0) as u32);
    }
    if let Some(n) = filter_on(e, "sharpen") {
        let soft = box_blur_image(&img, 1);
        let k = n.clamp(1.0, 10.0) as f32 / 4.0;
        for (p, q) in img.pixels_mut().zip(soft.pixels()) {
            for c in 0..3 {
                let v = p.0[c] as f32 + k * (p.0[c] as f32 - q.0[c] as f32);
                p.0[c] = v.round().clamp(0.0, 255.0) as u8;
            }
        }
    }
    let gray = filter_on(e, "grayscale").is_some();
    let sepia = filter_on(e, "sepia").map(|v| (v / 100.0).clamp(0.0, 1.0));
    let mono = filter_on(e, "mono").map(|v| v.clamp(0.0, 255.0));
    let invert = filter_on(e, "invert").is_some();
    let opacity = filter_on(e, "opacity").map(|v| (v / 100.0).clamp(0.0, 1.0));
    for p in img.pixels_mut() {
        let [r, g, bl, a] = p.0.map(|c| c as f64);
        let mut c = [r, g, bl];
        if gray {
            let l = 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2];
            c = [l, l, l];
        }
        if let Some(k) = sepia {
            let sp = [
                0.393 * c[0] + 0.769 * c[1] + 0.189 * c[2],
                0.349 * c[0] + 0.686 * c[1] + 0.168 * c[2],
                0.272 * c[0] + 0.534 * c[1] + 0.131 * c[2],
            ];
            c = [0, 1, 2].map(|i| c[i] + (sp[i] - c[i]) * k);
        }
        if let Some(t) = mono {
            let l = 0.299 * c[0] + 0.587 * c[1] + 0.114 * c[2];
            let v = if l >= t { 255.0 } else { 0.0 };
            c = [v, v, v];
        }
        if invert {
            c = c.map(|v| 255.0 - v);
        }
        let a = opacity.map(|k| a * k).unwrap_or(a);
        p.0 = [c[0], c[1], c[2], a].map(|v| v.round().clamp(0.0, 255.0) as u8);
    }
    img
}

fn box_blur_image(img: &RgbaImage, r: u32) -> RgbaImage {
    let (w, h) = img.dimensions();
    let mut data: Vec<u8> = img.as_raw().clone();
    // Premultiplied, as the shared blur expects.
    for p in data.as_chunks_mut::<4>().0 {
        let a = p[3] as u32;
        for v in &mut p[..3] {
            *v = ((*v as u32 * a + 127) / 255) as u8;
        }
    }
    crate::effects::blur(&mut data, w as usize, h as usize, r as usize);
    for p in data.as_chunks_mut::<4>().0 {
        let a = p[3] as u32;
        if let Some(half) = (a / 2).checked_add(0).filter(|_| a > 0) {
            for v in &mut p[..3] {
                *v = ((*v as u32 * 255 + half) / a).min(255) as u8;
            }
        }
    }
    RgbaImage::from_raw(w, h, data).expect("same size")
}

fn color_adjust(mut img: RgbaImage, e: &Value) -> RgbaImage {
    let brightness = f(e, "brightness", 0.0) / 100.0 * 255.0;
    let contrast = (f(e, "contrast", 0.0) / 100.0 + 1.0).max(0.0);
    let hue = f(e, "hue", 0.0);
    let saturation = (f(e, "saturation", 0.0) / 100.0 + 1.0).max(0.0);
    let gamma = f(e, "gamma", 1.0).clamp(0.1, 5.0);
    for p in img.pixels_mut() {
        let mut c = [p.0[0] as f64, p.0[1] as f64, p.0[2] as f64];
        if hue != 0.0 || saturation != 1.0 {
            let (h, s, l) = to_hsl(c);
            c = from_hsl(
                ((h + hue) % 360.0 + 360.0) % 360.0,
                (s * saturation).clamp(0.0, 1.0),
                l,
            );
        }
        c = c.map(|v| {
            let v = (v - 127.5) * contrast + 127.5 + brightness;
            255.0 * (v.clamp(0.0, 255.0) / 255.0).powf(1.0 / gamma)
        });
        for (v, x) in p.0[..3].iter_mut().zip(c) {
            *v = x.round().clamp(0.0, 255.0) as u8;
        }
    }
    img
}

fn to_hsl(c: [f64; 3]) -> (f64, f64, f64) {
    let [r, g, b] = c.map(|v| v / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let l = (max + min) / 2.0;
    if max == min {
        return (0.0, 0.0, l);
    }
    let d = max - min;
    let s = if l > 0.5 {
        d / (2.0 - max - min)
    } else {
        d / (max + min)
    };
    let h = if max == r {
        (g - b) / d + if g < b { 6.0 } else { 0.0 }
    } else if max == g {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    (h * 60.0, s, l)
}

fn from_hsl(h: f64, s: f64, l: f64) -> [f64; 3] {
    if s == 0.0 {
        return [l * 255.0; 3];
    }
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    let hk = h / 360.0;
    let t = |mut t: f64| {
        if t < 0.0 {
            t += 1.0;
        }
        if t > 1.0 {
            t -= 1.0;
        }
        if t < 1.0 / 6.0 {
            p + (q - p) * 6.0 * t
        } else if t < 0.5 {
            q
        } else if t < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - t) * 6.0
        } else {
            p
        }
    };
    [t(hk + 1.0 / 3.0), t(hk), t(hk - 1.0 / 3.0)].map(|v| v * 255.0)
}

/// Color A becomes B (or they swap), within a tolerance (EDT-064). `null` is
/// Transparent.
fn replace(mut img: RgbaImage, e: &Value) -> RgbaImage {
    let a = color(e, "a", "#FFFFFF");
    let bc = color(e, "b", "#000000");
    let tol = (f(e, "tolerance", 5.0).clamp(0.0, 100.0) / 100.0 * 255.0).round() as i32;
    let swap = s(e, "mode", "replace") == "swap";
    let like = |p: &[u8; 4], c: Option<Rgba>| match c {
        Some(c) => p[3] > 0 && (0..3).all(|i| (p[i] as i32 - [c.r, c.g, c.b][i] as i32).abs() <= tol),
        None => p[3] as i32 <= tol,
    };
    let paint = |c: Option<Rgba>| match c {
        Some(c) => [c.r, c.g, c.b, c.a],
        None => [0, 0, 0, 0],
    };
    for p in img.pixels_mut() {
        if like(&p.0, a) {
            p.0 = paint(bc);
        } else if swap && like(&p.0, bc) {
            p.0 = paint(a);
        }
    }
    img
}

// ---------------------------------------------------------------------------
// Transform effects (EDT-068)

/// The canvas's own rounded corners (EDT-075): transparent outside a radius.
pub fn round_corners(img: RgbaImage, radius: f64) -> RgbaImage {
    corners(img, &serde_json::json!({"shape": "radius", "radius": radius}))
}

/// Rounded corners: a radius in px, or a circle (an ellipse fitted to the image).
fn corners(mut img: RgbaImage, e: &Value) -> RgbaImage {
    let (w, h) = (img.width() as f64, img.height() as f64);
    let circle = s(e, "shape", "radius") == "circle";
    let r = f(e, "radius", 20.0).clamp(0.0, w.min(h) / 2.0);
    for (x, y, p) in img.enumerate_pixels_mut() {
        let (px, py) = (x as f64 + 0.5, y as f64 + 0.5);
        let inside = if circle {
            let (dx, dy) = ((px - w / 2.0) / (w / 2.0), (py - h / 2.0) / (h / 2.0));
            // Antialiased by the distance to the edge, about a pixel wide.
            let d = (dx * dx + dy * dy).sqrt();
            ((1.0 - d) * w.min(h) / 2.0 + 0.5).clamp(0.0, 1.0)
        } else {
            let cx = px.clamp(r, w - r);
            let cy = py.clamp(r, h - r);
            let d = ((px - cx).powi(2) + (py - cy).powi(2)).sqrt();
            (r - d + 0.5).clamp(0.0, 1.0)
        };
        p.0[3] = (p.0[3] as f64 * inside).round() as u8;
    }
    img
}

/// Removes uniform borders (like EDT-051): the color of the top-left pixel, within a
/// tolerance. Returns the image and where it was cut.
fn trim(img: &RgbaImage, e: &Value) -> (RgbaImage, i64, i64) {
    let (w, h) = img.dimensions();
    let tol = (f(e, "tolerance", 5.0).clamp(0.0, 100.0) / 100.0 * 255.0).round() as i32;
    let edge = img.get_pixel(0, 0).0;
    let same = |p: &[u8; 4]| (0..4).all(|i| (p[i] as i32 - edge[i] as i32).abs() <= tol);
    let row = |y: u32| (0..w).all(|x| same(&img.get_pixel(x, y).0));
    let col = |x: u32, y0: u32, y1: u32| (y0..y1).all(|y| same(&img.get_pixel(x, y).0));
    let mut y0 = 0;
    while y0 < h && row(y0) {
        y0 += 1;
    }
    if y0 == h {
        return (img.clone(), 0, 0);
    }
    let mut y1 = h;
    while y1 > y0 && row(y1 - 1) {
        y1 -= 1;
    }
    let mut x0 = 0;
    while x0 < w && col(x0, y0, y1) {
        x0 += 1;
    }
    let mut x1 = w;
    while x1 > x0 && col(x1 - 1, y0, y1) {
        x1 -= 1;
    }
    (
        imageops::crop_imm(img, x0, y0, x1 - x0, y1 - y0).to_image(),
        x0 as i64,
        y0 as i64,
    )
}

/// Scales by a percentage, by the longest side, or to a width or a height (EDT-068).
fn scale(img: &RgbaImage, e: &Value) -> RgbaImage {
    let (w, h) = (img.width() as f64, img.height() as f64);
    let v = f(e, "value", 100.0).max(1.0);
    let k = match s(e, "mode", "percent") {
        "longest" => v / w.max(h),
        "width" => v / w,
        "height" => v / h,
        _ => v / 100.0,
    };
    let (nw, nh) = ((w * k).round().max(1.0) as u32, (h * k).round().max(1.0) as u32);
    if (nw, nh) == img.dimensions() {
        return img.clone();
    }
    imageops::resize(img, nw, nh, imageops::FilterType::Lanczos3)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ctx() -> (Value, impl Fn(&str) -> Option<RgbaImage>) {
        (json!({}), |_: &str| None)
    }

    fn run(stack: Value, img: RgbaImage) -> Output {
        let (meta, pic) = ctx();
        let c = Ctx {
            canvas_color: None,
            dpi: 96.0,
            meta: &meta,
            picture: &pic,
        };
        apply(stack.as_array().unwrap(), img, &c)
    }

    #[test]
    fn an_outside_border_grows_the_result() {
        let img = RgbaImage::from_pixel(400, 300, Px([255, 255, 255, 255]));
        let out = run(json!([{"type": "border", "width": 5, "color": "#FF0000"}]), img);
        assert_eq!(out.image.dimensions(), (410, 310));
        assert_eq!(out.origin, [5, 5]);
        assert_eq!(out.image.get_pixel(1, 1).0, [255, 0, 0, 255]);
        assert_eq!(out.image.get_pixel(200, 150).0, [255, 255, 255, 255]);
    }

    #[test]
    fn a_shadow_is_soft_and_disabled_effects_do_nothing() {
        let img = RgbaImage::from_pixel(100, 80, Px([255, 255, 255, 255]));
        let out = run(
            json!([{"type": "shadow", "angle": 45, "distance": 10, "blur": 20}]),
            img.clone(),
        );
        let (w, h) = out.image.dimensions();
        assert!(w > 100 && h > 80);
        let corner = out.image.get_pixel(w - 25, h - 25).0;
        assert!(corner[3] > 0 && corner[3] < 255, "semi-transparent: {corner:?}");
        let off = run(json!([{"type": "shadow", "enabled": false}]), img);
        assert_eq!(off.image.dimensions(), (100, 80));
    }

    #[test]
    fn filters_and_color_work_on_pixels() {
        let img = RgbaImage::from_pixel(10, 10, Px([200, 30, 30, 255]));
        let gray = run(
            json!([{"type": "filters", "grayscale": {"on": true}}]),
            img.clone(),
        )
        .image;
        let p = gray.get_pixel(0, 0).0;
        assert!(p[0] == p[1] && p[1] == p[2]);
        let half = run(
            json!([{"type": "filters", "opacity": {"on": true, "value": 50}}]),
            img.clone(),
        )
        .image;
        assert_eq!(half.get_pixel(0, 0).0[3], 128);
        let mono = run(
            json!([{"type": "filters", "mono": {"on": true, "value": 128}}]),
            img.clone(),
        )
        .image;
        assert!(mono.pixels().all(|p| p.0[0] == 0 || p.0[0] == 255));
        let flat = run(json!([{"type": "color", "saturation": -100}]), img).image;
        let q = flat.get_pixel(0, 0).0;
        assert!((q[0] as i32 - q[1] as i32).abs() <= 1);
    }

    #[test]
    fn reflection_fade_replace_and_transforms() {
        let img = RgbaImage::from_pixel(50, 400, Px([255, 255, 255, 255]));
        let r = run(json!([{"type": "reflection", "size": 30}]), img.clone()).image;
        assert_eq!(r.height(), 400 + 120);
        assert_eq!(r.get_pixel(10, 519).0[3], 0, "fades out");
        let f = run(json!([{"type": "fade", "size": 10}]), img.clone()).image;
        assert_eq!(f.get_pixel(0, 200).0[3], 0);
        assert_eq!(f.get_pixel(25, 200).0[3], 255);
        let t = run(
            json!([{"type": "replace", "a": "#FFFFFF", "b": null, "tolerance": 0}]),
            img.clone(),
        )
        .image;
        assert_eq!(t.get_pixel(10, 10).0[3], 0);
        let s = run(
            json!([{"type": "scale", "mode": "percent", "value": 50}]),
            img.clone(),
        )
        .image;
        assert_eq!(s.dimensions(), (25, 200));
        let rot = run(json!([{"type": "rotate", "angle": 90}]), img.clone()).image;
        assert_eq!(rot.dimensions(), (400, 50));
        let c = run(json!([{"type": "corners", "radius": 20}]), img).image;
        assert_eq!(c.get_pixel(0, 0).0[3], 0);
    }
}
