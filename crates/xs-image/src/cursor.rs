//! The pointer as an object (EDT-093): a floating image marked as a cursor, drawn as
//! it was captured or as a standard shape (arrow, hand, text, cross), with an optional
//! highlight circle under it. The shapes are polygons in the unit square, scaled to
//! the object's box; `app/src/editor/cursorShapes.ts` holds the same ones.

use serde::{Deserialize, Serialize};
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Stroke, Transform};

use crate::shapes::Rect;

/// How a cursor looks (EDT-093).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CursorLook {
    /// "captured", "arrow", "hand", "text" or "cross".
    #[serde(default = "captured")]
    pub shape: String,
    /// "none", "yellow" or "accent".
    #[serde(default = "none")]
    pub highlight: String,
    /// The captured pointer's size in pixels: 100 % (EDT-093, 50 to 400 %).
    #[serde(default)]
    pub natural: [f64; 2],
}

fn captured() -> String {
    "captured".to_string()
}

fn none() -> String {
    "none".to_string()
}

pub const SHAPES: [&str; 5] = ["captured", "arrow", "hand", "text", "cross"];
pub const HIGHLIGHTS: [&str; 3] = ["none", "yellow", "accent"];

impl CursorLook {
    pub fn check(&self) -> Result<(), String> {
        if !SHAPES.contains(&self.shape.as_str()) {
            return Err(format!("cursor shape must be one of {}", SHAPES.join(", ")));
        }
        if !HIGHLIGHTS.contains(&self.highlight.as_str()) {
            return Err(format!(
                "cursor highlight must be one of {}",
                HIGHLIGHTS.join(", ")
            ));
        }
        Ok(())
    }
}

/// The standard shapes: outlines in the unit square (x right, y down).
pub fn polygon(shape: &str) -> &'static [[f64; 2]] {
    match shape {
        "hand" => &[
            [0.36, 0.0],
            [0.48, 0.0],
            [0.5, 0.42],
            [0.62, 0.36],
            [0.74, 0.4],
            [0.86, 0.44],
            [0.96, 0.52],
            [0.94, 0.8],
            [0.8, 1.0],
            [0.36, 1.0],
            [0.12, 0.66],
            [0.06, 0.5],
            [0.16, 0.46],
            [0.34, 0.62],
        ],
        "text" => &[
            [0.2, 0.0],
            [0.8, 0.0],
            [0.8, 0.1],
            [0.56, 0.1],
            [0.56, 0.9],
            [0.8, 0.9],
            [0.8, 1.0],
            [0.2, 1.0],
            [0.2, 0.9],
            [0.44, 0.9],
            [0.44, 0.1],
            [0.2, 0.1],
        ],
        "cross" => &[
            [0.44, 0.0],
            [0.56, 0.0],
            [0.56, 0.44],
            [1.0, 0.44],
            [1.0, 0.56],
            [0.56, 0.56],
            [0.56, 1.0],
            [0.44, 1.0],
            [0.44, 0.56],
            [0.0, 0.56],
            [0.0, 0.44],
            [0.44, 0.44],
        ],
        // The classic pointer.
        _ => &[
            [0.0, 0.0],
            [0.0, 0.82],
            [0.22, 0.64],
            [0.36, 0.98],
            [0.5, 0.92],
            [0.36, 0.6],
            [0.64, 0.6],
        ],
    }
}

/// Where the pointer points, in the unit square: the tip of the arrow and the hand,
/// the middle of the others. The highlight is centred there.
pub fn hotspot(shape: &str) -> [f64; 2] {
    match shape {
        "hand" => [0.42, 0.02],
        "text" | "cross" => [0.5, 0.5],
        _ => [0.04, 0.04],
    }
}

/// The highlight's radius for a box of this size.
pub fn highlight_radius(rect: &Rect) -> f64 {
    rect.w.max(rect.h) * 0.75
}

/// The highlight circle: its centre and radius in canvas pixels, if it has one.
pub fn highlight_circle(look: &CursorLook, rect: &Rect) -> Option<([f64; 2], f64)> {
    if look.highlight == "none" {
        return None;
    }
    let [hx, hy] = hotspot(&look.shape);
    Some((
        [rect.x + hx * rect.w, rect.y + hy * rect.h],
        highlight_radius(rect),
    ))
}

/// Draws the highlight (under) of a cursor turned `rotation` degrees about its box's
/// centre.
pub fn draw_highlight(pm: &mut Pixmap, look: &CursorLook, rect: &Rect, rotation: f64, transform: Transform) {
    let Some(([cx, cy], r)) = highlight_circle(look, rect) else {
        return;
    };
    let (red, green, blue) = if look.highlight == "yellow" {
        (255, 214, 10)
    } else {
        (45, 127, 249)
    };
    let mut paint = Paint::default();
    paint.set_color_rgba8(red, green, blue, 110);
    paint.anti_alias = true;
    let Some(circle) = PathBuilder::from_circle(cx as f32, cy as f32, r as f32) else {
        return;
    };
    let turn = Transform::from_rotate_at(
        rotation as f32,
        (rect.x + rect.w / 2.0) as f32,
        (rect.y + rect.h / 2.0) as f32,
    );
    pm.fill_path(
        &circle,
        &paint,
        FillRule::Winding,
        transform.pre_concat(turn),
        None,
    );
}

/// Draws a standard shape in `rect`: white with a black outline, like the system's.
pub fn draw_shape(
    pm: &mut Pixmap,
    shape: &str,
    rect: &Rect,
    rotation: f64,
    opacity: f64,
    transform: Transform,
) {
    let pts = polygon(shape);
    let mut pb = PathBuilder::new();
    for (i, [x, y]) in pts.iter().enumerate() {
        let (px, py) = ((rect.x + x * rect.w) as f32, (rect.y + y * rect.h) as f32);
        if i == 0 {
            pb.move_to(px, py);
        } else {
            pb.line_to(px, py);
        }
    }
    pb.close();
    let Some(path) = pb.finish() else { return };
    let turn = Transform::from_rotate_at(
        rotation as f32,
        (rect.x + rect.w / 2.0) as f32,
        (rect.y + rect.h / 2.0) as f32,
    );
    let t = transform.pre_concat(turn);
    let alpha = (opacity / 100.0 * 255.0).round().clamp(0.0, 255.0) as u8;
    let mut fill = Paint::default();
    fill.set_color_rgba8(255, 255, 255, alpha);
    fill.anti_alias = true;
    pm.fill_path(&path, &fill, FillRule::Winding, t, None);
    let mut line = Paint::default();
    line.set_color_rgba8(0, 0, 0, alpha);
    line.anti_alias = true;
    let width = (rect.w.max(rect.h) / 18.0).max(1.0) as f32;
    let stroke = Stroke {
        width,
        line_join: tiny_skia::LineJoin::Round,
        ..Stroke::default()
    };
    pm.stroke_path(&path, &line, &stroke, t, None);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn looks_are_checked_and_shapes_draw() {
        let ok = CursorLook {
            shape: "hand".into(),
            highlight: "yellow".into(),
            natural: [32.0, 32.0],
        };
        assert!(ok.check().is_ok());
        let bad = CursorLook {
            shape: "star".into(),
            ..ok.clone()
        };
        assert!(bad.check().is_err());
        let mut pm = Pixmap::new(100, 100).unwrap();
        let rect = Rect {
            x: 20.0,
            y: 20.0,
            w: 40.0,
            h: 40.0,
        };
        draw_highlight(&mut pm, &ok, &rect, 0.0, Transform::identity());
        draw_shape(&mut pm, "hand", &rect, 0.0, 100.0, Transform::identity());
        // The highlight is under the hand's tip; the hand is drawn in its box.
        let tip = pm.pixel(37, 21).unwrap();
        assert!(tip.alpha() > 0);
        let middle = pm.pixel(40, 60).unwrap();
        assert!(middle.alpha() > 200, "the hand's palm");
        let far = pm.pixel(99, 99).unwrap();
        assert_eq!(far.alpha(), 0);
    }
}
