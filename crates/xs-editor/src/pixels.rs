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
