//! Final image of a document (EDT-110): canvas color, the cropped base pixels and
//! every object, flattened, without any interface element (handles, guides).

use std::collections::BTreeMap;
use std::sync::Arc;

use image::RgbaImage;
use tiny_skia::{
    FillRule, LineCap, LineJoin, Mask, Paint, PathBuilder, Pixmap, PixmapPaint, Stroke, StrokeDash, Transform,
};
use xs_core::document::{Canvas, Document, Flattened, Object, PixelRect};

use crate::color::Rgba;
use crate::effects;
use crate::geometry::{dash, Arrow, Geometry, Shape};
use crate::shapes::{
    inner_rect, pixel_bounds, BlurObject, BlurType, ImageObject, Rect, ShapeKind, ShapeObject,
};
use crate::text::{self, Text};

/// Original pixels of a document, straight RGBA, shared between renders, with the
/// pictures of its floating images (`images/`, EDT-078), decoded.
#[derive(Clone)]
pub struct Base {
    pub width: u32,
    pub height: u32,
    pub rgba: Arc<Vec<u8>>,
    pub images: Arc<BTreeMap<String, Arc<Pixmap>>>,
}

impl Base {
    pub fn from_image(image: RgbaImage) -> Base {
        Base {
            width: image.width(),
            height: image.height(),
            rgba: Arc::new(image.into_raw()),
            images: Arc::default(),
        }
    }

    pub fn to_image(&self) -> RgbaImage {
        RgbaImage::from_raw(self.width, self.height, self.rgba.as_ref().clone()).expect("buffer size")
    }

    /// Adds the picture of a floating image, from its encoded file (PNG or JPG).
    pub fn add_image(&mut self, name: &str, bytes: &[u8]) -> Result<(), String> {
        let picture = premultiplied(&crate::encode::decode(bytes)?).ok_or("empty image")?;
        Arc::make_mut(&mut self.images).insert(name.to_string(), Arc::new(picture));
        Ok(())
    }
}

/// `image` as a premultiplied pixmap, ready to draw.
fn premultiplied(image: &RgbaImage) -> Option<Pixmap> {
    let mut pm = Pixmap::new(image.width(), image.height())?;
    for (d, s) in pm
        .data_mut()
        .as_chunks_mut::<4>()
        .0
        .iter_mut()
        .zip(image.as_raw().as_chunks::<4>().0)
    {
        let a = s[3] as u32;
        let pm = |v: u8| ((v as u32 * a + 127) / 255) as u8;
        *d = if a == 255 {
            *s
        } else {
            [pm(s[0]), pm(s[1]), pm(s[2]), s[3]]
        };
    }
    Some(pm)
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

/// Draws `layer` over `pm` at `opacity` %: translucent objects are drawn whole first,
/// so their own parts (head and shaft, fill and outline) do not darken each other.
fn blend_layer(pm: &mut Pixmap, opacity: f64, draw: impl FnOnce(&mut Pixmap)) {
    if opacity >= 100.0 {
        draw(pm);
        return;
    }
    let Some(mut layer) = Pixmap::new(pm.width(), pm.height()) else {
        return;
    };
    draw(&mut layer);
    let blend = PixmapPaint {
        opacity: (opacity / 100.0).clamp(0.0, 1.0) as f32,
        ..PixmapPaint::default()
    };
    pm.draw_pixmap(0, 0, layer.as_ref(), &blend, Transform::identity(), None);
}

/// Rotation around the center of `r`, in the canvas.
fn turned(r: Rect, rotation: f64) -> Transform {
    Transform::from_rotate_at(
        rotation as f32,
        (r.x + r.w / 2.0) as f32,
        (r.y + r.h / 2.0) as f32,
    )
}

/// Path of a rectangle or ellipse filling `r` (before rotation).
fn outline_path(kind: ShapeKind, r: Rect) -> Option<tiny_skia::Path> {
    let rect =
        tiny_skia::Rect::from_xywh(r.x as f32, r.y as f32, r.w.max(0.01) as f32, r.h.max(0.01) as f32)?;
    match kind {
        ShapeKind::Rectangle => Some(PathBuilder::from_rect(rect)),
        ShapeKind::Ellipse => PathBuilder::from_oval(rect),
    }
}

fn draw_shape(pm: &mut Pixmap, s: &ShapeObject) {
    let t = turned(s.rect, s.rotation);
    blend_layer(pm, s.style.opacity, |pm| {
        if let Some(fill) = s.style.fill.as_deref().and_then(Rgba::parse) {
            if let Some(path) = outline_path(s.style.shape_type, s.rect) {
                pm.fill_path(&path, &paint(fill), FillRule::Winding, t, None);
            }
        }
        let w = s.style.outline_width;
        if w > 0.0 {
            let inner = inner_rect(s.rect, w);
            if let (Some(path), Some(color)) = (
                outline_path(s.style.shape_type, inner),
                Rgba::parse(&s.style.outline),
            ) {
                let dash = dash(s.style.line_style, w);
                let mut st = stroke(w, dash.as_deref());
                st.line_join = LineJoin::Miter;
                pm.stroke_path(&path, &paint(color), &st, t, None);
            }
        }
    });
}

/// The blur, pixelation or solid cover of `b` over what `below` shows, transparent
/// outside its shape, with where it goes: `(patch, x, y)`.
fn blur_patch(below: &Pixmap, b: &BlurObject) -> Option<(Pixmap, i32, i32)> {
    let [x, y, w, h] = pixel_bounds(b.rect, b.rotation, below.width(), below.height())?;
    let (x, y, w, h) = (x as i32, y as i32, w as u32, h as u32);
    let mut mask = Mask::new(w, h)?;
    let path = outline_path(b.style.shape_type, b.rect)?;
    let t = turned(b.rect, b.rotation).post_translate(-x as f32, -y as f32);
    mask.fill_path(&path, FillRule::Winding, true, t);
    let mut out = Pixmap::new(w, h)?;
    if b.style.blur_type == BlurType::Solid {
        let c = Rgba::parse(&b.style.color).unwrap_or(Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        });
        out.fill(tiny_skia::Color::from_rgba8(c.r, c.g, c.b, c.a));
    } else {
        // Blur reads around the area too; pixelation only inside it.
        let pad = if b.style.blur_type == BlurType::Blur {
            3 * b.style.radius() as i32
        } else {
            0
        };
        let (ex0, ey0) = ((x - pad).max(0), (y - pad).max(0));
        let ex1 = (x + w as i32 + pad).min(below.width() as i32);
        let ey1 = (y + h as i32 + pad).min(below.height() as i32);
        let (ew, eh) = ((ex1 - ex0) as usize, (ey1 - ey0) as usize);
        let src = below.data();
        let stride = below.width() as usize * 4;
        let mut area = Vec::with_capacity(ew * eh * 4);
        for row in ey0 as usize..ey1 as usize {
            let start = row * stride + ex0 as usize * 4;
            area.extend_from_slice(&src[start..start + ew * 4]);
        }
        match b.style.blur_type {
            BlurType::Blur => effects::blur(&mut area, ew, eh, b.style.radius() as usize),
            _ => effects::pixelate(
                &mut area,
                ew,
                eh,
                b.style.block() as usize,
                b.rect.x.floor() as i64 - ex0 as i64,
                b.rect.y.floor() as i64 - ey0 as i64,
            ),
        }
        let dst = out.data_mut();
        for row in 0..h as usize {
            let from = ((row + (y - ey0) as usize) * ew + (x - ex0) as usize) * 4;
            dst[row * w as usize * 4..(row + 1) * w as usize * 4]
                .copy_from_slice(&area[from..from + w as usize * 4]);
        }
    }
    out.apply_mask(&mask);
    Some((out, x, y))
}

/// Draws one object. Kinds this build does not know are skipped (a newer file).
fn draw_object(pm: &mut Pixmap, object: &Object, dpi: f64, images: &BTreeMap<String, Arc<Pixmap>>) {
    match object.kind.as_str() {
        "arrow" => {
            let Ok(a) = Arrow::from_props(&object.props) else {
                return;
            };
            let color = Rgba::parse(&a.style.color).unwrap_or(Rgba::WHITE);
            let g = a.geometry();
            blend_layer(pm, a.style.opacity, |pm| draw_geometry(pm, &g, color));
        }
        "text" => {
            let Ok(t) = Text::from_props(&object.props) else {
                return;
            };
            let Some((raster, laid)) = text::rasterize(&t, dpi) else {
                return;
            };
            let r = Rect {
                x: t.at[0],
                y: t.at[1],
                w: laid.width as f64,
                h: laid.height as f64,
            };
            let paint = PixmapPaint {
                opacity: (t.style.opacity / 100.0).clamp(0.0, 1.0) as f32,
                quality: tiny_skia::FilterQuality::Bilinear,
                ..PixmapPaint::default()
            };
            let at = turned(r, t.rotation).pre_translate(r.x as f32, r.y as f32);
            pm.draw_pixmap(0, 0, raster.as_ref(), &paint, at, None);
        }
        "shape" => {
            if let Ok(s) = ShapeObject::from_props(&object.props) {
                draw_shape(pm, &s);
            }
        }
        "image" => {
            let Ok(i) = ImageObject::from_props(&object.props) else {
                return;
            };
            let Some(picture) = images.get(&i.image) else {
                return;
            };
            let r = i.rect;
            let paint = PixmapPaint {
                opacity: (i.style.opacity / 100.0).clamp(0.0, 1.0) as f32,
                quality: tiny_skia::FilterQuality::Bicubic,
                ..PixmapPaint::default()
            };
            let at = turned(r, i.rotation)
                .pre_translate(r.x as f32, r.y as f32)
                .pre_scale(
                    (r.w / picture.width() as f64) as f32,
                    (r.h / picture.height() as f64) as f32,
                );
            pm.draw_pixmap(0, 0, picture.as_ref().as_ref(), &paint, at, None);
        }
        "blur" => {
            let Ok(b) = BlurObject::from_props(&object.props) else {
                return;
            };
            if let Some((patch, x, y)) = blur_patch(pm, &b) {
                pm.draw_pixmap(
                    x,
                    y,
                    patch.as_ref(),
                    &PixmapPaint::default(),
                    Transform::identity(),
                    None,
                );
            }
        }
        _ => {}
    }
}

fn canvas(doc: &Document, base: &Base) -> Pixmap {
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
    pm
}

fn straight(pm: &Pixmap) -> RgbaImage {
    let mut out = Vec::with_capacity(pm.data().len());
    for px in pm.pixels() {
        let c = px.demultiply();
        out.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    RgbaImage::from_raw(pm.width(), pm.height(), out).expect("buffer size")
}

/// Corners of `r` turned `rotation` degrees clockwise around its center.
fn turned_corners(r: Rect, rotation: f64) -> [[f64; 2]; 4] {
    let (cx, cy) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
    let (s, c) = rotation.to_radians().sin_cos();
    [
        (r.x, r.y),
        (r.x + r.w, r.y),
        (r.x + r.w, r.y + r.h),
        (r.x, r.y + r.h),
    ]
    .map(|(x, y)| {
        let (dx, dy) = (x - cx, y - cy);
        [cx + dx * c - dy * s, cy + dx * s + dy * c]
    })
}

/// What an object covers, `[x0, y0, x1, y1]` in canvas pixels, with its arrowheads,
/// its rotation and the stroke of its ends: how far the canvas grows to fit it
/// (EDT-071). An arrow's shaft counts by its two points, so one drawn from an edge
/// does not widen the canvas by its thickness.
pub fn object_bounds(object: &Object, dpi: f64) -> Option<[f64; 4]> {
    let mut b: Option<[f64; 4]> = None;
    let mut add = |x: f64, y: f64| {
        let e = b.get_or_insert([x, y, x, y]);
        *e = [e[0].min(x), e[1].min(y), e[2].max(x), e[3].max(y)];
    };
    let rect = match object.kind.as_str() {
        "arrow" => {
            let g = Arrow::from_props(&object.props).ok()?.geometry();
            if let Some([x1, y1, x2, y2]) = g.shaft {
                add(x1, y1);
                add(x2, y2);
            }
            for cap in &g.caps {
                match cap {
                    Shape::Path {
                        points,
                        fill,
                        stroke_width,
                        ..
                    } => {
                        let r = if *fill { 0.0 } else { stroke_width / 2.0 };
                        for [x, y] in points.as_chunks::<2>().0 {
                            add(x - r, y - r);
                            add(x + r, y + r);
                        }
                    }
                    Shape::Circle {
                        cx,
                        cy,
                        r,
                        fill,
                        stroke_width,
                    } => {
                        let r = r + if *fill { 0.0 } else { stroke_width / 2.0 };
                        add(cx - r, cy - r);
                        add(cx + r, cy + r);
                    }
                }
            }
            None
        }
        "text" => {
            let t = Text::from_props(&object.props).ok()?;
            let laid = text::layout(&t, dpi);
            Some((
                Rect {
                    x: t.at[0],
                    y: t.at[1],
                    w: laid.width as f64,
                    h: laid.height as f64,
                },
                t.rotation,
            ))
        }
        "shape" => ShapeObject::from_props(&object.props)
            .ok()
            .map(|s| (s.rect, s.rotation)),
        "blur" => BlurObject::from_props(&object.props)
            .ok()
            .map(|b| (b.rect, b.rotation)),
        "image" => ImageObject::from_props(&object.props)
            .ok()
            .map(|i| (i.rect, i.rotation)),
        _ => None,
    };
    if let Some((r, rotation)) = rect {
        for [x, y] in turned_corners(r, rotation) {
            add(x, y);
        }
    }
    b
}

/// The final image of `doc` over `base`, straight RGBA: the flattened objects, then the
/// objects (EDT-091).
pub fn render(doc: &Document, base: &Base) -> RgbaImage {
    let mut pm = canvas(doc, base);
    for object in flattened_objects(doc).iter().chain(&doc.objects) {
        draw_object(&mut pm, object, doc.dpi, &base.images);
    }
    straight(&pm)
}

/// The flattened objects of `doc` in canvas pixels, oldest first (EDT-091).
pub fn flattened_objects(doc: &Document) -> Vec<Object> {
    let (dx, dy) = (-doc.crop.x as f64, -doc.crop.y as f64);
    doc.flattened
        .iter()
        .flat_map(|f| f.objects.iter())
        .map(|o| {
            let mut o = o.clone();
            o.translate(dx, dy);
            o
        })
        .collect()
}

fn union(a: Option<[f64; 4]>, b: [f64; 4]) -> Option<[f64; 4]> {
    Some(match a {
        None => b,
        Some(a) => [a[0].min(b[0]), a[1].min(b[1]), a[2].max(b[2]), a[3].max(b[3])],
    })
}

/// What a flattened layer covers, `[x0, y0, x1, y1]` in canvas pixels (EDT-091).
pub fn layer_bounds(doc: &Document, layer: &Flattened) -> Option<[f64; 4]> {
    let (dx, dy) = (-doc.crop.x as f64, -doc.crop.y as f64);
    layer
        .objects
        .iter()
        .filter_map(|o| object_bounds(o, doc.dpi))
        .fold(None, union)
        .map(|b| [b[0] + dx, b[1] + dy, b[2] + dx, b[3] + dy])
}

/// What the editor shows under the objects when some are flattened (EDT-091): the
/// original pixels with the flattened layers drawn over them, and where its top left
/// corner is, in base pixels. It reaches past the original pixels where a flattened
/// object does, as far as the canvas. `None` without flattened layers.
pub fn flattened_view(doc: &Document, base: &Base) -> Option<(RgbaImage, i64, i64)> {
    let r = flattened_view_rect(doc, base.width, base.height)?;
    let view = Document {
        canvas: Canvas {
            width: r.width,
            height: r.height,
            color: None,
            corner_radius: 0.0,
            extra: Default::default(),
        },
        crop: r,
        objects: Vec::new(),
        ..doc.clone()
    };
    Some((render(&view, base), r.x, r.y))
}

/// Where [`flattened_view`] is, in base pixels, for original pixels of `width` ×
/// `height`. `None` without flattened layers.
pub fn flattened_view_rect(doc: &Document, width: u32, height: u32) -> Option<PixelRect> {
    if doc.flattened.is_empty() {
        return None;
    }
    let image = [0.0, 0.0, width as f64, height as f64];
    let c = doc.crop;
    let reach = union(
        Some(image),
        [
            c.x as f64,
            c.y as f64,
            (c.x + c.width as i64) as f64,
            (c.y + c.height as i64) as f64,
        ],
    )?;
    let layers = doc
        .flattened
        .iter()
        .flat_map(|f| &f.objects)
        .filter_map(|o| object_bounds(o, doc.dpi))
        .fold(Some(image), union)?;
    let x0 = layers[0].max(reach[0]).floor() as i64;
    let y0 = layers[1].max(reach[1]).floor() as i64;
    let x1 = layers[2].min(reach[2]).ceil() as i64;
    let y1 = layers[3].min(reach[3]).ceil() as i64;
    Some(PixelRect {
        x: x0,
        y: y0,
        width: (x1 - x0).max(1) as u32,
        height: (y1 - y0).max(1) as u32,
    })
}

/// The original pixels with the permanent flattened layers merged in: what `base.png`
/// holds once the file is written, so it no longer has what a flattened blur hid
/// (EDT-091). `None` without permanent layers.
pub fn baked_base(doc: &Document, base: &Base) -> Option<RgbaImage> {
    let permanent: Vec<Flattened> = doc
        .flattened
        .iter()
        .filter(|f| f.permanent.is_some())
        .cloned()
        .collect();
    if permanent.is_empty() {
        return None;
    }
    let view = Document {
        canvas: Canvas {
            width: base.width,
            height: base.height,
            color: None,
            corner_radius: 0.0,
            extra: Default::default(),
        },
        crop: PixelRect {
            x: 0,
            y: 0,
            width: base.width,
            height: base.height,
        },
        objects: Vec::new(),
        flattened: permanent,
        ..doc.clone()
    };
    Some(render(&view, base))
}

/// What the editor canvas shows for objects Rust draws (ADR 0004), with its top-left
/// corner in canvas pixels: text (unrotated; the canvas turns it) and blur or
/// pixelation (as it looks over what is below it). `None` for other objects.
pub fn object_image(doc: &Document, base: &Base, id: &str) -> Option<(RgbaImage, i32, i32)> {
    let index = doc.objects.iter().position(|o| o.id == id)?;
    let object = &doc.objects[index];
    match object.kind.as_str() {
        "text" => {
            let t = Text::from_props(&object.props).ok()?;
            let (raster, _) = text::rasterize(&t, doc.dpi)?;
            Some((straight(&raster), t.at[0].floor() as i32, t.at[1].floor() as i32))
        }
        "blur" => {
            let b = BlurObject::from_props(&object.props).ok()?;
            if b.style.blur_type == BlurType::Solid {
                return None;
            }
            let mut pm = canvas(doc, base);
            for below in flattened_objects(doc).iter().chain(&doc.objects[..index]) {
                draw_object(&mut pm, below, doc.dpi, &base.images);
            }
            let (patch, x, y) = blur_patch(&pm, &b)?;
            Some((straight(&patch), x, y))
        }
        _ => None,
    }
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

    fn object(kind: &str, props: serde_json::Value) -> Object {
        Object {
            id: format!("{kind}1"),
            kind: kind.into(),
            props: props.as_object().unwrap().clone(),
        }
    }

    /// White image with black text-like stripes in 100..200 × 50..90.
    fn striped() -> Base {
        let mut img = RgbaImage::from_pixel(300, 150, image::Rgba([255, 255, 255, 255]));
        for y in 50..90 {
            for x in (100..200).step_by(4) {
                img.put_pixel(x, y, image::Rgba([0, 0, 0, 255]));
            }
        }
        Base::from_image(img)
    }

    #[test]
    fn solid_blur_leaves_no_trace_of_what_it_covers() {
        let base = striped();
        let mut doc = Document::new(300, 150, 96.0);
        doc.objects.push(object(
            "blur",
            json!({"rect": {"x": 90, "y": 40, "w": 120, "h": 60}, "style": {"blurType": "solid"}}),
        ));
        let img = render(&doc, &base);
        for y in 41..99 {
            for x in 91..209 {
                assert_eq!(img.get_pixel(x, y).0, [0, 0, 0, 255], "at {x},{y}");
            }
        }
        assert_eq!(img.get_pixel(50, 20).0, [255, 255, 255, 255]);
    }

    #[test]
    fn blur_and_pixelate_hide_detail_and_bigger_intensity_makes_bigger_blocks() {
        let base = striped();
        let blurred = |kind: &str, intensity: f64| {
            let mut doc = Document::new(300, 150, 96.0);
            doc.objects.push(object(
                "blur",
                json!({"rect": {"x": 100, "y": 50, "w": 100, "h": 40}, "style": {"blurType": kind, "intensity": intensity}}),
            ));
            render(&doc, &base)
        };
        let b = blurred("blur", 60.0);
        // A stripe and its gap become close grays.
        let (on, off) = (b.get_pixel(148, 70).0[0] as i32, b.get_pixel(150, 70).0[0] as i32);
        assert!((on - off).abs() < 30, "{on} vs {off}");
        // Over a gradient, each pixelation block is one flat step.
        let mut ramp = RgbaImage::new(300, 150);
        for (x, _, p) in ramp.enumerate_pixels_mut() {
            *p = image::Rgba([x as u8, x as u8, x as u8, 255]);
        }
        let ramp = Base::from_image(ramp);
        let pixelated = |intensity: f64| {
            let mut doc = Document::new(300, 150, 96.0);
            doc.objects.push(object(
                "blur",
                json!({"rect": {"x": 100, "y": 50, "w": 100, "h": 40}, "style": {"blurType": "pixelate", "intensity": intensity}}),
            ));
            render(&doc, &ramp)
        };
        let run = |img: &RgbaImage| {
            (101..200)
                .take_while(|x| img.get_pixel(*x, 60) == img.get_pixel(100, 60))
                .count()
                + 1
        };
        let (small, big) = (run(&pixelated(20.0)), run(&pixelated(100.0)));
        assert!(big > small && small > 1, "{big} vs {small}");
    }

    #[test]
    fn shape_outline_stays_inside_its_size() {
        let base = white(300, 150);
        let mut doc = Document::new(300, 150, 96.0);
        doc.objects.push(object(
            "shape",
            json!({"rect": {"x": 50, "y": 20, "w": 200, "h": 100}, "style": {"outline": "#FF0000", "outlineWidth": 10}}),
        ));
        let img = render(&doc, &base);
        assert_eq!(img.get_pixel(50, 70).0, [255, 0, 0, 255], "outer edge is outline");
        assert_eq!(img.get_pixel(249, 70).0, [255, 0, 0, 255]);
        assert_eq!(img.get_pixel(49, 70).0, [255, 255, 255, 255], "nothing outside");
        assert_eq!(img.get_pixel(250, 70).0, [255, 255, 255, 255]);
        assert_eq!(img.get_pixel(150, 70).0, [255, 255, 255, 255], "transparent fill");
    }

    #[test]
    fn text_is_drawn_where_it_is_and_has_its_own_image() {
        let base = white(300, 150);
        let mut doc = Document::new(300, 150, 96.0);
        doc.objects.push(object(
            "text",
            json!({"at": [20, 30], "text": "XXXX", "style": {"color": "#0000FF", "size": 30}}),
        ));
        let img = render(&doc, &base);
        let blue = |x0: u32, x1: u32| {
            (x0..x1)
                .flat_map(|x| (0..150).map(move |y| (x, y)))
                .filter(|(x, y)| img.get_pixel(*x, *y).0 == [0, 0, 255, 255])
                .count()
        };
        assert!(blue(20, 200) > 20);
        assert_eq!(blue(0, 19), 0);
        let (own, x, y) = object_image(&doc, &base, "text1").unwrap();
        assert_eq!((x, y), (20, 30));
        assert!(own.width() > 40);
    }

    /// EDT-078: a floating image is drawn scaled into its rectangle, its bounds turn
    /// with it, and a missing picture draws nothing.
    #[test]
    fn floating_images_are_scaled_into_their_rectangle() {
        let mut base = white(100, 60);
        let red = RgbaImage::from_pixel(10, 5, image::Rgba([255, 0, 0, 255]));
        let png = crate::encode::png(&red, 96.0, true).unwrap();
        base.add_image("img-1.png", &png).unwrap();
        let mut doc = Document::new(100, 60, 96.0);
        doc.objects.push(object(
            "image",
            json!({"image": "img-1.png", "rect": {"x": 10, "y": 10, "w": 40, "h": 20}}),
        ));
        doc.objects.push(object(
            "image",
            json!({"image": "img-2.png", "rect": {"x": 60, "y": 10, "w": 20, "h": 20}}),
        ));
        let img = render(&doc, &base);
        assert_eq!(img.get_pixel(12, 12).0, [255, 0, 0, 255]);
        assert_eq!(img.get_pixel(48, 28).0, [255, 0, 0, 255], "scaled up to 40 x 20");
        assert_eq!(img.get_pixel(52, 12).0, [255, 255, 255, 255]);
        assert_eq!(
            img.get_pixel(70, 20).0,
            [255, 255, 255, 255],
            "no picture, nothing drawn"
        );
        let mut turned = doc.objects[0].clone();
        turned.props.insert("rotation".into(), json!(90));
        let b = object_bounds(&turned, 96.0).unwrap();
        assert!((b[0] - 20.0).abs() < 1e-9 && (b[1] - 0.0).abs() < 1e-9, "{b:?}");
        assert!((b[2] - 40.0).abs() < 1e-9 && (b[3] - 40.0).abs() < 1e-9, "{b:?}");
    }

    #[test]
    fn transparent_base_pixels_stay_transparent() {
        let base = Base::from_image(RgbaImage::from_pixel(4, 4, image::Rgba([10, 20, 30, 0])));
        let out = render(&Document::new(4, 4, 96.0), &base);
        assert_eq!(out.get_pixel(0, 0).0[3], 0);
    }
}
