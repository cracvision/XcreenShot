//! Final image of a document (EDT-110): canvas color, the cropped base pixels and
//! every object, flattened, without any interface element (handles, guides).

use std::sync::Arc;

use image::RgbaImage;
use tiny_skia::{
    FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, PixmapPaint, Stroke, StrokeDash, Transform,
};
use xs_core::document::{Document, Object};

use crate::color::Rgba;
use crate::geometry::{Arrow, Geometry, Shape};

/// Original pixels of a document, straight RGBA, shared between renders.
#[derive(Clone)]
pub struct Base {
    pub width: u32,
    pub height: u32,
    pub rgba: Arc<Vec<u8>>,
}

impl Base {
    pub fn from_image(image: RgbaImage) -> Base {
        Base {
            width: image.width(),
            height: image.height(),
            rgba: Arc::new(image.into_raw()),
        }
    }

    pub fn to_image(&self) -> RgbaImage {
        RgbaImage::from_raw(self.width, self.height, self.rgba.as_ref().clone()).expect("buffer size")
    }
}

/// Premultiplied copy of the part of `base` under the canvas, and where it goes.
fn base_window(doc: &Document, base: &Base) -> Option<(Pixmap, i32, i32)> {
    let c = doc.crop;
    let x0 = c.x.max(0);
    let y0 = c.y.max(0);
    let x1 = (c.x + c.width as i64).min(base.width as i64);
    let y1 = (c.y + c.height as i64).min(base.height as i64);
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    let (w, h) = ((x1 - x0) as u32, (y1 - y0) as u32);
    let mut pm = Pixmap::new(w, h)?;
    let stride = base.width as usize * 4;
    let out = pm.data_mut();
    for row in 0..h as usize {
        let src = (y0 as usize + row) * stride + x0 as usize * 4;
        let src = &base.rgba[src..src + w as usize * 4];
        let dst = &mut out[row * w as usize * 4..(row + 1) * w as usize * 4];
        for (d, s) in dst.as_chunks_mut::<4>().0.iter_mut().zip(src.as_chunks::<4>().0) {
            let a = s[3] as u32;
            if a == 255 {
                *d = *s;
            } else {
                let pm = |v: u8| ((v as u32 * a + 127) / 255) as u8;
                *d = [pm(s[0]), pm(s[1]), pm(s[2]), s[3]];
            }
        }
    }
    Some((pm, (x0 - c.x) as i32, (y0 - c.y) as i32))
}

fn paint(color: Rgba) -> Paint<'static> {
    let mut p = Paint::default();
    p.set_color_rgba8(color.r, color.g, color.b, color.a);
    p.anti_alias = true;
    p
}

fn stroke(width: f64, dash: Option<&[f64]>) -> Stroke {
    Stroke {
        width: width as f32,
        line_cap: LineCap::Butt,
        line_join: LineJoin::Round,
        dash: dash.and_then(|d| StrokeDash::new(d.iter().map(|v| *v as f32).collect(), 0.0)),
        ..Stroke::default()
    }
}

fn draw_geometry(pm: &mut Pixmap, g: &Geometry, color: Rgba) {
    let p = paint(color);
    if let Some([x1, y1, x2, y2]) = g.shaft {
        let mut pb = PathBuilder::new();
        pb.move_to(x1 as f32, y1 as f32);
        pb.line_to(x2 as f32, y2 as f32);
        if let Some(path) = pb.finish() {
            pm.stroke_path(
                &path,
                &p,
                &stroke(g.width, g.dash.as_deref()),
                Transform::identity(),
                None,
            );
        }
    }
    for shape in &g.caps {
        match shape {
            Shape::Path {
                points,
                closed,
                fill,
                stroke_width,
            } => {
                let mut pb = PathBuilder::new();
                for (i, xy) in points.as_chunks::<2>().0.iter().enumerate() {
                    if i == 0 {
                        pb.move_to(xy[0] as f32, xy[1] as f32);
                    } else {
                        pb.line_to(xy[0] as f32, xy[1] as f32);
                    }
                }
                if *closed {
                    pb.close();
                }
                let Some(path) = pb.finish() else { continue };
                if *fill {
                    pm.fill_path(&path, &p, FillRule::Winding, Transform::identity(), None);
                } else {
                    pm.stroke_path(
                        &path,
                        &p,
                        &stroke(*stroke_width, None),
                        Transform::identity(),
                        None,
                    );
                }
            }
            Shape::Circle {
                cx,
                cy,
                r,
                fill,
                stroke_width,
            } => {
                let Some(path) = PathBuilder::from_circle(*cx as f32, *cy as f32, *r as f32) else {
                    continue;
                };
                if *fill {
                    pm.fill_path(&path, &p, FillRule::Winding, Transform::identity(), None);
                } else {
                    pm.stroke_path(
                        &path,
                        &p,
                        &stroke(*stroke_width, None),
                        Transform::identity(),
                        None,
                    );
                }
            }
        }
    }
}

/// Draws one object. Kinds this build does not know are skipped (a newer file).
fn draw_object(pm: &mut Pixmap, object: &Object) {
    let (geometry, color, opacity) = match object.kind.as_str() {
        "arrow" => match Arrow::from_props(&object.props) {
            Ok(a) => (
                a.geometry(),
                Rgba::parse(&a.style.color).unwrap_or(Rgba::WHITE),
                a.style.opacity,
            ),
            Err(_) => return,
        },
        _ => return,
    };
    if opacity >= 100.0 {
        draw_geometry(pm, &geometry, color);
        return;
    }
    // Translucent objects are drawn whole and then blended, so the head and the shaft
    // do not darken where they overlap.
    let Some(mut layer) = Pixmap::new(pm.width(), pm.height()) else {
        return;
    };
    draw_geometry(&mut layer, &geometry, color);
    let blend = PixmapPaint {
        opacity: (opacity / 100.0).clamp(0.0, 1.0) as f32,
        ..PixmapPaint::default()
    };
    pm.draw_pixmap(0, 0, layer.as_ref(), &blend, Transform::identity(), None);
}

/// The final image of `doc` over `base`, straight RGBA.
pub fn render(doc: &Document, base: &Base) -> RgbaImage {
    let (w, h) = (doc.canvas.width.max(1), doc.canvas.height.max(1));
    let mut pm = Pixmap::new(w, h).expect("canvas size");
    if let Some(color) = doc.canvas.color.as_deref().and_then(Rgba::parse) {
        pm.fill(tiny_skia::Color::from_rgba8(color.r, color.g, color.b, color.a));
    }
    if let Some((window, x, y)) = base_window(doc, base) {
        pm.draw_pixmap(
            x,
            y,
            window.as_ref(),
            &PixmapPaint::default(),
            Transform::identity(),
            None,
        );
    }
    for object in &doc.objects {
        draw_object(&mut pm, object);
    }
    let mut out = Vec::with_capacity((w * h * 4) as usize);
    for px in pm.pixels() {
        let c = px.demultiply();
        out.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    RgbaImage::from_raw(w, h, out).expect("buffer size")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn white(w: u32, h: u32) -> Base {
        Base::from_image(RgbaImage::from_pixel(w, h, image::Rgba([255, 255, 255, 255])))
    }

    fn arrow(props: serde_json::Value) -> Object {
        Object {
            id: "o1".into(),
            kind: "arrow".into(),
            props: props.as_object().unwrap().clone(),
        }
    }

    #[test]
    fn base_only_is_the_base() {
        let base = white(20, 10);
        let doc = Document::new(20, 10, 96.0);
        assert_eq!(render(&doc, &base), base.to_image());
    }

    #[test]
    fn draws_an_arrow_in_its_color_and_nothing_elsewhere() {
        let base = white(400, 200);
        let mut doc = Document::new(400, 200, 96.0);
        doc.objects.push(arrow(json!({
            "from": [100, 100], "to": [300, 100],
            "style": {"color": "#0000FF", "width": 10}
        })));
        let img = render(&doc, &base);
        assert_eq!(img.get_pixel(200, 100).0, [0, 0, 255, 255], "on the shaft");
        assert_eq!(img.get_pixel(296, 100).0, [0, 0, 255, 255], "inside the head");
        assert_eq!(img.get_pixel(200, 130).0, [255, 255, 255, 255], "off the arrow");
        assert_eq!(img.get_pixel(310, 100).0, [255, 255, 255, 255], "past the point");
    }

    #[test]
    fn translucent_arrows_blend_once() {
        let base = white(400, 200);
        let mut doc = Document::new(400, 200, 96.0);
        doc.objects.push(arrow(json!({
            "from": [100, 100], "to": [300, 100],
            "style": {"color": "#000000", "width": 10, "opacity": 50}
        })));
        let img = render(&doc, &base);
        let shaft = img.get_pixel(200, 100).0;
        // Where the shaft runs into the head, it is not darker than elsewhere.
        let overlap = img.get_pixel(270, 100).0;
        assert!((shaft[0] as i32 - 128).abs() <= 2, "{shaft:?}");
        assert_eq!(shaft, overlap);
    }

    #[test]
    fn crop_shows_only_its_window() {
        let mut img = RgbaImage::from_pixel(10, 10, image::Rgba([255, 255, 255, 255]));
        img.put_pixel(6, 7, image::Rgba([255, 0, 0, 255]));
        let base = Base::from_image(img);
        let mut doc = Document::new(4, 4, 96.0);
        doc.crop = xs_core::document::PixelRect {
            x: 5,
            y: 5,
            width: 4,
            height: 4,
        };
        let out = render(&doc, &base);
        assert_eq!(out.dimensions(), (4, 4));
        assert_eq!(out.get_pixel(1, 2).0, [255, 0, 0, 255]);
    }

    #[test]
    fn transparent_base_pixels_stay_transparent() {
        let base = Base::from_image(RgbaImage::from_pixel(4, 4, image::Rgba([10, 20, 30, 0])));
        let out = render(&Document::new(4, 4, 96.0), &base);
        assert_eq!(out.get_pixel(0, 0).0[3], 0);
    }
}
