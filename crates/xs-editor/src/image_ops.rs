//! Image and canvas operations (EDT-069, EDT-070, EDT-072, EDT-073): they make new
//! original pixels (ADR 0006) and carry the crop, the objects and the flattened layers
//! along.

use image::{imageops, RgbaImage};
use serde_json::{Map, Value};
use xs_core::document::{Document, PixelRect};

use crate::objects;
use crate::transform::{self, Measure, Op, Rect};

/// How pixels are resampled (EDT-069).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resample {
    /// Lanczos to shrink, bicubic to enlarge.
    Auto,
    Bicubic,
    Bilinear,
    Lanczos,
    Nearest,
}

impl Resample {
    pub fn parse(s: &str) -> Option<Resample> {
        Some(match s {
            "auto" => Resample::Auto,
            "bicubic" => Resample::Bicubic,
            "bilinear" => Resample::Bilinear,
            "lanczos" => Resample::Lanczos,
            "nearest" => Resample::Nearest,
            _ => return None,
        })
    }

    fn filter(self, enlarging: bool) -> imageops::FilterType {
        match self {
            Resample::Auto if enlarging => imageops::FilterType::CatmullRom,
            Resample::Auto | Resample::Lanczos => imageops::FilterType::Lanczos3,
            Resample::Bicubic => imageops::FilterType::CatmullRom,
            Resample::Bilinear => imageops::FilterType::Triangle,
            Resample::Nearest => imageops::FilterType::Nearest,
        }
    }
}

/// Applies `op` to every object (canvas pixels) and `base_op` to every flattened
/// object (base pixels).
fn transform_all(doc: &mut Document, op: &Op, base_op: &Op, measure: Measure) {
    let dpi = doc.dpi;
    for o in doc.objects.iter_mut() {
        let set: Map<String, Value> = transform::transformed(o, op, measure);
        if !set.is_empty() {
            let _ = objects::update(o, &set);
            objects::fit(o, dpi);
        }
    }
    for layer in doc.flattened.iter_mut() {
        for o in layer.objects.iter_mut() {
            let set = transform::transformed(o, base_op, measure);
            if !set.is_empty() {
                let _ = objects::update(o, &set);
            }
        }
    }
}

/// Resizes the document by `kx`, `ky` (EDT-069): the original pixels are resampled,
/// and the crop, objects (with line widths and text sizes) and flattened layers scale
/// with them. Returns the new original pixels.
pub fn resize(
    doc: &mut Document,
    pixels: &RgbaImage,
    kx: f64,
    ky: f64,
    how: Resample,
    measure: Measure,
) -> RgbaImage {
    let (bw, bh) = pixels.dimensions();
    let (nw, nh) = (
        ((bw as f64 * kx).round() as u32).max(1),
        ((bh as f64 * ky).round() as u32).max(1),
    );
    let out = if (nw, nh) == (bw, bh) {
        pixels.clone()
    } else {
        imageops::resize(pixels, nw, nh, how.filter(kx > 1.0 || ky > 1.0))
    };
    let (cw, ch) = (doc.canvas.width as f64, doc.canvas.height as f64);
    let op = Op::Scale {
        from: Rect {
            x: 0.0,
            y: 0.0,
            w: cw,
            h: ch,
        },
        to: Rect {
            x: 0.0,
            y: 0.0,
            w: cw * kx,
            h: ch * ky,
        },
        strokes: true,
    };
    let base_op = Op::Scale {
        from: Rect {
            x: 0.0,
            y: 0.0,
            w: bw as f64,
            h: bh as f64,
        },
        to: Rect {
            x: 0.0,
            y: 0.0,
            w: bw as f64 * kx,
            h: bh as f64 * ky,
        },
        strokes: true,
    };
    transform_all(doc, &op, &base_op, measure);
    let c = doc.crop;
    doc.crop = PixelRect {
        x: (c.x as f64 * kx).round() as i64,
        y: (c.y as f64 * ky).round() as i64,
        width: ((c.width as f64 * kx).round() as u32).max(1),
        height: ((c.height as f64 * ky).round() as u32).max(1),
    };
    doc.canvas.width = doc.crop.width;
    doc.canvas.height = doc.crop.height;
    doc.crop_history.clear();
    doc.frame = None;
    doc.patches.clear();
    out
}

/// Turns the document `degrees` clockwise (EDT-072): the original pixels turn (the new
/// corners transparent or `background`), the canvas grows to hold the turned one, and
/// the objects turn with it. Returns the new original pixels.
pub fn rotate(
    doc: &mut Document,
    pixels: &RgbaImage,
    degrees: f64,
    background: Option<[u8; 4]>,
    measure: Measure,
) -> RgbaImage {
    let d = degrees.rem_euclid(360.0);
    let (bw, bh) = (pixels.width() as f64, pixels.height() as f64);
    let quarter = (d / 90.0).round();
    let exact = (d - quarter * 90.0).abs() < 1e-9;
    let out = if exact {
        match quarter as i64 % 4 {
            1 => imageops::rotate90(pixels),
            2 => imageops::rotate180(pixels),
            3 => imageops::rotate270(pixels),
            _ => pixels.clone(),
        }
    } else {
        xs_image::fx_marks::turn(pixels, d, background)
    };
    let (nw, nh) = (out.width() as f64, out.height() as f64);
    let (s, c) = d.to_radians().sin_cos();
    // Base pixels: p' = R (p - cb) + cb'.
    let (cb, cb2) = ([bw / 2.0, bh / 2.0], [nw / 2.0, nh / 2.0]);
    let map = |p: [f64; 2]| {
        let (x, y) = (p[0] - cb[0], p[1] - cb[1]);
        [x * c - y * s + cb2[0], x * s + y * c + cb2[1]]
    };
    // The canvas's corners, turned: the new canvas holds them.
    let cr = doc.crop;
    let corners = [
        [cr.x as f64, cr.y as f64],
        [(cr.x + cr.width as i64) as f64, cr.y as f64],
        [(cr.x + cr.width as i64) as f64, (cr.y + cr.height as i64) as f64],
        [cr.x as f64, (cr.y + cr.height as i64) as f64],
    ]
    .map(map);
    let x0 = corners.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min).round();
    let y0 = corners.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min).round();
    let x1 = corners
        .iter()
        .map(|p| p[0])
        .fold(f64::NEG_INFINITY, f64::max)
        .round();
    let y1 = corners
        .iter()
        .map(|p| p[1])
        .fold(f64::NEG_INFINITY, f64::max)
        .round();
    // Objects: turn around the base center (in canvas pixels), then move to the new
    // canvas.
    let center = [cb[0] - cr.x as f64, cb[1] - cr.y as f64];
    let op = Op::Rotate { center, degrees: d };
    let base_op = Op::Rotate {
        center: cb,
        degrees: d,
    };
    transform_all(doc, &op, &base_op, measure);
    let (dx, dy) = (cb2[0] - x0 - center[0], cb2[1] - y0 - center[1]);
    for o in doc.objects.iter_mut() {
        objects::translate(o, dx, dy);
    }
    let (fdx, fdy) = (cb2[0] - cb[0], cb2[1] - cb[1]);
    for layer in doc.flattened.iter_mut() {
        for o in layer.objects.iter_mut() {
            objects::translate(o, fdx, fdy);
        }
    }
    doc.crop = PixelRect {
        x: x0 as i64,
        y: y0 as i64,
        width: ((x1 - x0) as u32).max(1),
        height: ((y1 - y0) as u32).max(1),
    };
    doc.canvas.width = doc.crop.width;
    doc.canvas.height = doc.crop.height;
    doc.crop_history.clear();
    doc.frame = None;
    doc.patches.clear();
    out
}

/// The canvas set to `width` × `height`, growing or shrinking on the sides the anchor
/// does not hold (EDT-070). Like a crop: it can always come back.
pub fn canvas_size(doc: &mut Document, width: u32, height: u32, anchor: &str) {
    let c = doc.crop;
    let (dw, dh) = (width as i64 - c.width as i64, height as i64 - c.height as i64);
    // How much of the change goes to the left and the top.
    let fx = if anchor.ends_with("Left") || anchor == "left" {
        0
    } else if anchor.ends_with("Right") || anchor == "right" {
        dw
    } else {
        dw / 2
    };
    let fy = if anchor.starts_with("top") {
        0
    } else if anchor.starts_with("bottom") {
        dh
    } else {
        dh / 2
    };
    doc.crop_history.push(c);
    doc.crop = PixelRect {
        x: c.x - fx,
        y: c.y - fy,
        width: width.max(1),
        height: height.max(1),
    };
    for o in doc.objects.iter_mut() {
        objects::translate(o, fx as f64, fy as f64);
    }
    doc.canvas.width = doc.crop.width;
    doc.canvas.height = doc.crop.height;
    doc.frame = Some(doc.crop);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use xs_core::document::Object;

    fn no_text(_: &Map<String, Value>) -> Option<(f64, f64)> {
        None
    }

    fn arrow(doc: &mut Document) {
        doc.objects.push(Object {
            id: "o1".into(),
            kind: "arrow".into(),
            props: json!({"from": [100, 100], "to": [300, 100], "style": {"color": "#FF0000", "width": 8}})
                .as_object()
                .cloned()
                .unwrap(),
        });
    }

    #[test]
    fn resizing_scales_pixels_and_objects() {
        let mut doc = Document::new(2000, 1000, 96.0);
        arrow(&mut doc);
        let px = RgbaImage::new(2000, 1000);
        let out = resize(&mut doc, &px, 0.5, 0.5, Resample::Auto, &no_text);
        assert_eq!(out.dimensions(), (1000, 500));
        assert_eq!((doc.canvas.width, doc.canvas.height), (1000, 500));
        assert_eq!(doc.objects[0].props["to"], json!([150.0, 50.0]));
        assert_eq!(doc.objects[0].props["style"]["width"], json!(4.0));
        // Nearest: each pixel doubled, no mixing.
        let two = RgbaImage::from_fn(2, 1, |x, _| {
            if x == 0 {
                image::Rgba([255, 0, 0, 255])
            } else {
                image::Rgba([0, 0, 255, 255])
            }
        });
        let mut d2 = Document::new(2, 1, 96.0);
        let big = resize(&mut d2, &two, 2.0, 2.0, Resample::Nearest, &no_text);
        assert_eq!(big.get_pixel(1, 1).0, [255, 0, 0, 255]);
        assert_eq!(big.get_pixel(2, 0).0, [0, 0, 255, 255]);
    }

    #[test]
    fn turning_a_quarter_swaps_the_sides_and_comes_back() {
        let mut doc = Document::new(800, 600, 96.0);
        arrow(&mut doc);
        let px = RgbaImage::from_fn(800, 600, |x, y| {
            image::Rgba([(x % 256) as u8, (y % 256) as u8, 0, 255])
        });
        let r = rotate(&mut doc, &px, 90.0, None, &no_text);
        assert_eq!(r.dimensions(), (600, 800));
        assert_eq!((doc.canvas.width, doc.canvas.height), (600, 800));
        // The arrow from (100,100) now starts at (600-100, 100) = (500, 100).
        let from = doc.objects[0].props["from"].clone();
        assert!((from[0].as_f64().unwrap() - 500.0).abs() < 0.01, "{from}");
        assert!((from[1].as_f64().unwrap() - 100.0).abs() < 0.01, "{from}");
        let back = rotate(&mut doc, &r, -90.0, None, &no_text);
        assert_eq!(back, px, "exactly the original");
        let f = doc.objects[0].props["from"].clone();
        assert!(
            (f[0].as_f64().unwrap() - 100.0).abs() < 0.01 && (f[1].as_f64().unwrap() - 100.0).abs() < 0.01
        );
        // 15°: transparent new corners.
        let mut d3 = Document::new(100, 100, 96.0);
        let t = rotate(
            &mut d3,
            &RgbaImage::from_pixel(100, 100, image::Rgba([0, 0, 0, 255])),
            15.0,
            None,
            &no_text,
        );
        assert!(t.width() > 100);
        assert_eq!(t.get_pixel(0, 0).0[3], 0);
    }

    #[test]
    fn the_canvas_grows_around_its_anchor() {
        let mut doc = Document::new(600, 400, 96.0);
        arrow(&mut doc);
        canvas_size(&mut doc, 1000, 1000, "center");
        assert_eq!((doc.crop.x, doc.crop.y), (-200, -300));
        assert_eq!(
            doc.objects[0].props["from"],
            json!([300.0, 400.0]),
            "it stays on the image"
        );
    }
}
