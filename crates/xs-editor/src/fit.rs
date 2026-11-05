//! The canvas grows to fit objects that stick out of it and shrinks back when they
//! return, never below the crop the user chose (EDT-071; Carlos, 2026-10-27, brings it
//! to M1 for drawing, moving, resizing and pasting).
//!
//! Objects a crop left outside stay as the crop left them: they do not make the canvas
//! grow until they are wholly inside the crop again. Blurs never do: they only work on
//! the image.

use xs_core::document::{Document, Object, PixelRect};
use xs_image::object_bounds;

use crate::objects;

/// Less than half a pixel out is inside: the canvas grows by whole pixels.
const HALF: f64 = 0.5;

fn within(b: [f64; 4], area: [f64; 4]) -> bool {
    b[0] > area[0] - HALF && b[1] > area[1] - HALF && b[2] < area[2] + HALF && b[3] < area[3] + HALF
}

fn grows(o: &Object) -> bool {
    o.kind != "blur"
}

/// The objects of `doc` that are not wholly inside its canvas: what a crop leaves outside.
pub fn outside_canvas(doc: &Document) -> Vec<String> {
    let area = [0.0, 0.0, doc.canvas.width as f64, doc.canvas.height as f64];
    doc.objects
        .iter()
        .filter(|o| object_bounds(o, doc.dpi).is_some_and(|b| !within(b, area)))
        .map(|o| o.id.clone())
        .collect()
}

/// Makes the canvas the user's crop plus whatever the objects need around it. Moves the
/// objects when the canvas grows or shrinks to the left or the top, so they stay on the
/// same place of the image.
pub fn fit_canvas(doc: &mut Document) {
    let crop = doc.crop;
    let frame = doc.frame.unwrap_or(crop);
    let fx = (frame.x - crop.x) as f64;
    let fy = (frame.y - crop.y) as f64;
    let area = [fx, fy, fx + frame.width as f64, fy + frame.height as f64];
    let dpi = doc.dpi;

    // Left outside by a crop: ordinary again once wholly inside the crop.
    let objects = &doc.objects;
    doc.outside.retain(|id| {
        objects
            .iter()
            .find(|o| &o.id == id)
            .is_some_and(|o| !object_bounds(o, dpi).is_some_and(|b| within(b, area)))
    });

    let mut e = area;
    for o in doc
        .objects
        .iter()
        .filter(|o| grows(o) && !doc.outside.contains(&o.id))
    {
        if let Some(b) = object_bounds(o, dpi) {
            e = [e[0].min(b[0]), e[1].min(b[1]), e[2].max(b[2]), e[3].max(b[3])];
        }
    }
    let x0 = (e[0] + HALF).floor() as i64;
    let y0 = (e[1] + HALF).floor() as i64;
    let x1 = (e[2] - HALF).ceil() as i64;
    let y1 = (e[3] - HALF).ceil() as i64;
    let window = PixelRect {
        x: crop.x + x0,
        y: crop.y + y0,
        width: (x1 - x0).max(1) as u32,
        height: (y1 - y0).max(1) as u32,
    };
    if window == crop {
        return;
    }
    for o in &mut doc.objects {
        objects::translate(o, -x0 as f64, -y0 as f64);
    }
    doc.crop = window;
    doc.canvas.width = window.width;
    doc.canvas.height = window.height;
    doc.frame = (window != frame).then_some(frame);
}
