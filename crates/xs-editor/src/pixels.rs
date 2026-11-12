//! Pixel tools on a document (EDT-047 Fill, EDT-048 Eraser; ADR 0005): they change
//! the image, as a patch of new pixels, or a floating image, as a new picture.

use image::RgbaImage;
use xs_core::document::{Document, Object, Patch, PixelRect};
use xs_image::color::Rgba;
use xs_image::pixels;
use xs_image::render::{image_in_base, layer_bounds};
use xs_image::shapes::ImageObject;
use xs_image::Base;

/// What a pixel tool does, in canvas pixels.
#[derive(Debug, Clone)]
pub enum Tool {
    /// EDT-047: `color` `None` is Transparent.
    Fill {
        at: [f64; 2],
        color: Option<Rgba>,
        tolerance: f64,
        opacity: f64,
        global: bool,
    },
    /// EDT-048.
    Erase {
        points: Vec<[f64; 2]>,
        width: f64,
        square: bool,
        opacity: f64,
        antialias: bool,
    },
}

/// The image's new pixels over `rect`, in base pixels: what to store as a patch.
pub struct Edit {
    pub rect: PixelRect,
    pub pixels: RgbaImage,
}

/// What `tool` does to the image under the objects (with what is flattened and the
/// earlier edits). `None` if it changes nothing.
pub fn edit_image(doc: &Document, base: &Base, tool: &Tool) -> Option<Edit> {
    let mut img = image_in_base(doc, base);
    // Canvas pixels to base pixels.
    let (dx, dy) = (doc.crop.x as f64, doc.crop.y as f64);
    let area = apply(&mut img, tool, |p| [p[0] + dx, p[1] + dy], 1.0)?;
    Some(Edit {
        rect: PixelRect {
            x: area[0] as i64,
            y: area[1] as i64,
            width: area[2],
            height: area[3],
        },
        pixels: pixels::cut(&img, area),
    })
}

/// What `tool` does to the picture of the floating image `object` (EDT-047, EDT-048):
/// the whole new picture, or `None` if it changes nothing.
pub fn edit_picture(object: &Object, mut picture: RgbaImage, tool: &Tool) -> Option<RgbaImage> {
    let io = ImageObject::from_props(&object.props).ok()?;
    let r = io.rect;
    if r.w <= 0.0 || r.h <= 0.0 {
        return None;
    }
    let (pw, ph) = (picture.width() as f64, picture.height() as f64);
    let (cx, cy) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
    let (s, c) = (-io.rotation).to_radians().sin_cos();
    // Undo the turn around the center, then the stretch.
    let to_picture = |p: [f64; 2]| {
        let (x, y) = (p[0] - cx, p[1] - cy);
        let (x, y) = (cx + x * c - y * s, cy + x * s + y * c);
        [(x - r.x) * pw / r.w, (y - r.y) * ph / r.h]
    };
    let scale = (pw / r.w + ph / r.h) / 2.0;
    apply(&mut picture, tool, to_picture, scale)?;
    Some(picture)
}

/// Runs `tool` on `img`, with `map` taking its points to `img`'s pixels and `scale`
/// its sizes. The changed rectangle, or `None` if nothing changed.
fn apply(
    img: &mut RgbaImage,
    tool: &Tool,
    map: impl Fn([f64; 2]) -> [f64; 2],
    scale: f64,
) -> Option<pixels::Area> {
    match tool {
        Tool::Fill {
            at,
            color,
            tolerance,
            opacity,
            global,
        } => {
            let [x, y] = map(*at);
            if x < 0.0 || y < 0.0 {
                return None;
            }
            let (mask, area) = pixels::reach(img, (x as u32, y as u32), *tolerance, *global)?;
            pixels::fill(img, &mask, *color, *opacity);
            Some(area)
        }
        Tool::Erase {
            points,
            width,
            square,
            opacity,
            antialias,
        } => {
            let points: Vec<[f64; 2]> = points.iter().map(|p| map(*p)).collect();
            pixels::erase(img, &points, width * scale, *square, *opacity, *antialias)
        }
    }
}

/// Adds the patch `image` over `rect` (base pixels). The flattened layers it touches
/// that could still become objects stay flat: the patch has their pixels (EDT-091).
pub fn add_patch(doc: &mut Document, rect: PixelRect, image: String) {
    let (x0, y0) = ((rect.x - doc.crop.x) as f64, (rect.y - doc.crop.y) as f64);
    let (x1, y1) = (x0 + rect.width as f64, y0 + rect.height as f64);
    let touched: Vec<usize> = doc
        .flattened
        .iter()
        .enumerate()
        .filter(|(_, l)| l.permanent.is_none())
        .filter(|(_, l)| {
            layer_bounds(doc, l).is_some_and(|b| b[0] < x1 && b[2] > x0 && b[1] < y1 && b[3] > y0)
        })
        .map(|(i, _)| i)
        .collect();
    for i in touched {
        doc.flattened[i].permanent = Some("edited".into());
    }
    doc.patches.push(Patch {
        rect,
        image,
        extra: Default::default(),
    });
}

// ---------------------------------------------------------------------------
// Pixel selections (EDT-043 to EDT-046)

use serde_json::{json, Value};
use std::sync::Arc;
use xs_image::selection::{self, Gap, Mask, Shape};
use xs_image::Transform;

/// The pixel selection of an open document: on the image (`target` `None`, in base
/// pixels) or on a floating image (its picture's pixels). Not part of the document
/// and not an undo step.
#[derive(Debug, Clone)]
pub struct PixelSelection {
    pub target: Option<String>,
    pub mask: Arc<Mask>,
    outline: Arc<Vec<Vec<[u32; 2]>>>,
    /// The rectangle or ellipse drawn alone, in canvas pixels, for its numeric
    /// position and size (EDT-086).
    pub shape: Option<(String, [f64; 4])>,
}

impl PixelSelection {
    pub fn new(target: Option<String>, mask: Mask, shape: Option<(String, [f64; 4])>) -> PixelSelection {
        let outline = mask.outline();
        PixelSelection {
            target,
            mask: Arc::new(mask),
            outline: Arc::new(outline),
            shape,
        }
    }

    /// What the editor draws: the outline and bounds in canvas pixels.
    pub fn info(&self, doc: &Document) -> Option<Value> {
        let to_canvas = to_canvas(doc, self.target.as_deref(), self.mask.width, self.mask.height)?;
        let map = |x: f32, y: f32| {
            let mut p = [tiny_point(x, y)];
            to_canvas.map_points(&mut p);
            [round2(p[0].x), round2(p[0].y)]
        };
        let outline: Vec<Vec<[f64; 2]>> = self
            .outline
            .iter()
            .map(|l| l.iter().map(|p| map(p[0] as f32, p[1] as f32)).collect())
            .collect();
        let b = self.mask.bounds()?;
        let corners = [
            map(b[0] as f32, b[1] as f32),
            map((b[0] + b[2]) as f32, b[1] as f32),
            map((b[0] + b[2]) as f32, (b[1] + b[3]) as f32),
            map(b[0] as f32, (b[1] + b[3]) as f32),
        ];
        let x0 = corners.iter().map(|c| c[0]).fold(f64::INFINITY, f64::min);
        let y0 = corners.iter().map(|c| c[1]).fold(f64::INFINITY, f64::min);
        let x1 = corners.iter().map(|c| c[0]).fold(f64::NEG_INFINITY, f64::max);
        let y1 = corners.iter().map(|c| c[1]).fold(f64::NEG_INFINITY, f64::max);
        Some(json!({
            "target": self.target,
            "bounds": {"x": x0, "y": y0, "w": x1 - x0, "h": y1 - y0},
            "outline": outline,
            "shape": self.shape.as_ref().map(|(k, r)| json!({"type": k, "rect": {"x": r[0], "y": r[1], "w": r[2], "h": r[3]}})),
        }))
    }
}

fn tiny_point(x: f32, y: f32) -> xs_image::selection::Point {
    xs_image::selection::Point::from_xy(x, y)
}

fn round2(v: f32) -> f64 {
    (v as f64 * 100.0).round() / 100.0
}

/// From the target's pixels to canvas pixels: the image is offset by the crop; a
/// floating image is stretched and turned (EDT-078).
pub fn to_canvas(doc: &Document, target: Option<&str>, width: u32, height: u32) -> Option<Transform> {
    match target {
        None => Some(Transform::from_translate(-doc.crop.x as f32, -doc.crop.y as f32)),
        Some(id) => {
            let o = doc.objects.iter().find(|o| o.id == id)?;
            let io = ImageObject::from_props(&o.props).ok()?;
            let r = io.rect;
            if width == 0 || height == 0 || r.w <= 0.0 || r.h <= 0.0 {
                return None;
            }
            let (cx, cy) = ((r.x + r.w / 2.0) as f32, (r.y + r.h / 2.0) as f32);
            Some(
                Transform::from_rotate_at(io.rotation as f32, cx, cy)
                    .pre_translate(r.x as f32, r.y as f32)
                    .pre_scale((r.w / width as f64) as f32, (r.h / height as f64) as f32),
            )
        }
    }
}

/// A part to select, in canvas pixels (EDT-043, EDT-046).
#[derive(Debug, Clone)]
pub enum Part {
    Shape(Shape),
    /// The magic wand at a point.
    Color {
        at: [f64; 2],
        tolerance: f64,
        global: bool,
    },
    All,
}

/// The mask `part` selects on `pixels` (the target's), with `to_target` taking canvas
/// pixels there.
pub fn part_mask(part: &Part, pixels: &RgbaImage, to_target: Transform, antialias: bool) -> Mask {
    let (w, h) = pixels.dimensions();
    match part {
        Part::Shape(s) => Mask::of_shape(w, h, s, to_target, antialias),
        Part::Color {
            at,
            tolerance,
            global,
        } => {
            let mut p = [tiny_point(at[0] as f32, at[1] as f32)];
            to_target.map_points(&mut p);
            if p[0].x < 0.0 || p[0].y < 0.0 {
                return Mask::empty(w, h);
            }
            Mask::of_color(pixels, (p[0].x as u32, p[0].y as u32), *tolerance, *global)
        }
        Part::All => Mask::full(w, h),
    }
}

/// The pixels a selection works on: the image without objects (base pixels), or a
/// floating image's picture.
pub fn target_pixels(
    doc: &Document,
    base: &Base,
    target: Option<&str>,
    picture: Option<RgbaImage>,
) -> Option<RgbaImage> {
    match target {
        None => Some(image_in_base(doc, base)),
        Some(_) => picture,
    }
}

/// What an action on the selection does to the target's pixels (EDT-044, EDT-045):
/// the new pixels and, for the image, the rectangle that changed.
pub struct Changed {
    pub pixels: RgbaImage,
    pub area: [u32; 4],
}

/// Deletes the selected pixels, leaving `gap`.
pub fn delete(pixels: &RgbaImage, mask: &Mask, gap: Gap) -> Option<Changed> {
    let area = mask.bounds()?;
    let mut out = pixels.clone();
    selection::fill_gap(&mut out, mask, gap);
    Some(Changed { pixels: out, area })
}

/// Moves the selected pixels by whole pixels, leaving `gap` where they were; the
/// moved selection too.
pub fn shift(pixels: &RgbaImage, mask: &Mask, dx: i64, dy: i64, gap: Gap) -> Option<(Changed, Mask)> {
    let (cut, at) = selection::selected_pixels(pixels, mask)?;
    let mut out = pixels.clone();
    selection::fill_gap(&mut out, mask, gap);
    image::imageops::overlay(&mut out, &cut, at[0] as i64 + dx, at[1] as i64 + dy);
    let moved = mask.shifted(dx, dy);
    let (w, h) = (pixels.width() as i64, pixels.height() as i64);
    let x0 = (at[0] as i64).min(at[0] as i64 + dx).clamp(0, w);
    let y0 = (at[1] as i64).min(at[1] as i64 + dy).clamp(0, h);
    let x1 = (at[0] as i64 + at[2] as i64)
        .max(at[0] as i64 + at[2] as i64 + dx)
        .clamp(0, w);
    let y1 = (at[1] as i64 + at[3] as i64)
        .max(at[1] as i64 + at[3] as i64 + dy)
        .clamp(0, h);
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    let area = [x0 as u32, y0 as u32, (x1 - x0) as u32, (y1 - y0) as u32];
    Some((Changed { pixels: out, area }, moved))
}

/// The part `area` of `img`.
pub fn cut_area(img: &RgbaImage, area: [u32; 4]) -> RgbaImage {
    pixels::cut(img, area)
}

/// `[x, y, w, h]` as a rectangle of base pixels.
pub fn rect_of(area: [u32; 4]) -> PixelRect {
    PixelRect {
        x: area[0] as i64,
        y: area[1] as i64,
        width: area[2],
        height: area[3],
    }
}
