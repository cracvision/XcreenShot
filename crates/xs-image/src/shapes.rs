//! Objects drawn in a rectangle: Shape (EDT-029) and Blur (EDT-040).
//!
//! The rectangle is in canvas pixels; `rotation` turns it clockwise around its center.
//! `inner_rect` and `pixel_bounds` are repeated in `app/src/editor/geometry.ts` and
//! checked against `tests/fixtures/shape-geometry.json`.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::color::Rgba;
use crate::geometry::LineStyle;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// `{"x", "y", "w", "h"}` with a size of at least one pixel.
pub fn rect(v: Option<&Value>) -> Result<Rect, String> {
    let r: Rect = v
        .cloned()
        .ok_or("rect is required")
        .and_then(|v| serde_json::from_value(v).map_err(|_| "rect must be {x, y, w, h}"))?;
    if !(r.w >= 1.0 && r.h >= 1.0) {
        return Err("rect must be at least 1 pixel wide and high".to_string());
    }
    Ok(r)
}

fn rotation(props: &Map<String, Value>) -> f64 {
    props.get("rotation").and_then(Value::as_f64).unwrap_or(0.0)
}

/// Where the outline is drawn: inside the shape's size, so the size shown is the
/// outer one (EDT-029).
pub fn inner_rect(r: Rect, outline: f64) -> Rect {
    let o = outline.min(r.w).min(r.h).max(0.0);
    Rect {
        x: r.x + o / 2.0,
        y: r.y + o / 2.0,
        w: r.w - o,
        h: r.h - o,
    }
}

/// Whole pixels covered by `r` turned by `rotation` degrees, inside a canvas of
/// `width` × `height`: `[x, y, w, h]`, or `None` when it is all outside.
pub fn pixel_bounds(r: Rect, rotation: f64, width: u32, height: u32) -> Option<[i64; 4]> {
    let (cx, cy) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
    let (s, c) = rotation.to_radians().sin_cos();
    let corners = [(-r.w, -r.h), (r.w, -r.h), (r.w, r.h), (-r.w, r.h)].map(|(dx, dy)| {
        let (dx, dy) = (dx / 2.0, dy / 2.0);
        (cx + dx * c - dy * s, cy + dx * s + dy * c)
    });
    let min_x = corners.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
    let max_x = corners.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
    let min_y = corners.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
    let max_y = corners.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);
    let x0 = (min_x.floor() as i64).max(0);
    let y0 = (min_y.floor() as i64).max(0);
    let x1 = (max_x.ceil() as i64).min(width as i64);
    let y1 = (max_y.ceil() as i64).min(height as i64);
    (x1 > x0 && y1 > y0).then_some([x0, y0, x1 - x0, y1 - y0])
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ShapeKind {
    Rectangle,
    Ellipse,
}

fn merged<T: Serialize + for<'de> Deserialize<'de>>(
    base: &T,
    changes: &Map<String, Value>,
) -> Result<T, String> {
    let mut v = serde_json::to_value(base).map_err(|e| e.to_string())?;
    if let Some(obj) = v.as_object_mut() {
        for (k, val) in changes {
            obj.insert(k.clone(), val.clone());
        }
    }
    serde_json::from_value(v).map_err(|e| format!("style: {e}"))
}

fn check_color(name: &str, c: &str) -> Result<(), String> {
    Rgba::parse(c)
        .map(|_| ())
        .ok_or_else(|| format!("{name} \"{c}\" is not #RRGGBB or #RRGGBBAA"))
}

/// Shape properties of M1 (EDT-029). Gradients and shadow arrive in M2.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ShapeStyle {
    pub shape_type: ShapeKind,
    /// `None` is Transparent: only the outline (EDT-029).
    pub fill: Option<String>,
    pub outline: String,
    /// 0–100 px; 0 = no outline.
    pub outline_width: f64,
    pub line_style: LineStyle,
    pub opacity: f64,
}

impl Default for ShapeStyle {
    /// A hollow coral rectangle, from the default theme (EDT-017).
    fn default() -> Self {
        ShapeStyle {
            shape_type: ShapeKind::Rectangle,
            fill: None,
            outline: "#F2545B".to_string(),
            outline_width: 4.0,
            line_style: LineStyle::Solid,
            opacity: 100.0,
        }
    }
}

impl ShapeStyle {
    pub fn merged(base: &ShapeStyle, changes: &Map<String, Value>) -> Result<ShapeStyle, String> {
        let s: ShapeStyle = merged(base, changes)?;
        if let Some(f) = &s.fill {
            check_color("fill", f)?;
        }
        check_color("outline", &s.outline)?;
        if !(0.0..=100.0).contains(&s.outline_width) {
            return Err("outlineWidth must be between 0 and 100".to_string());
        }
        if !(0.0..=100.0).contains(&s.opacity) {
            return Err("opacity must be between 0 and 100".to_string());
        }
        Ok(s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ShapeObject {
    pub rect: Rect,
    pub rotation: f64,
    pub style: ShapeStyle,
}

impl ShapeObject {
    pub fn from_props(props: &Map<String, Value>) -> Result<ShapeObject, String> {
        let style = match props.get("style") {
            Some(Value::Object(s)) => ShapeStyle::merged(&ShapeStyle::default(), s)?,
            Some(_) => return Err("style must be an object".to_string()),
            None => ShapeStyle::default(),
        };
        Ok(ShapeObject {
            rect: rect(props.get("rect"))?,
            rotation: rotation(props),
            style,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum BlurType {
    Blur,
    Pixelate,
    /// Covers the area with an opaque color: the one to use for confidential data.
    Solid,
}

/// Blur properties of M1 (EDT-040). Polygon and rounded rectangle arrive in M2.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct BlurStyle {
    pub blur_type: BlurType,
    /// Only with Solid.
    pub color: String,
    /// 1–100.
    pub intensity: f64,
    pub shape_type: ShapeKind,
}

impl Default for BlurStyle {
    fn default() -> Self {
        BlurStyle {
            blur_type: BlurType::Blur,
            color: "#000000".to_string(),
            intensity: 60.0,
            shape_type: ShapeKind::Rectangle,
        }
    }
}

impl BlurStyle {
    pub fn merged(base: &BlurStyle, changes: &Map<String, Value>) -> Result<BlurStyle, String> {
        let s: BlurStyle = merged(base, changes)?;
        check_color("color", &s.color)?;
        if !(1.0..=100.0).contains(&s.intensity) {
            return Err("intensity must be between 1 and 100".to_string());
        }
        Ok(s)
    }

    /// Radius of the blur, in pixels.
    pub fn radius(&self) -> u32 {
        (1.0 + self.intensity * 0.25).round() as u32
    }

    /// Side of a pixelation block, in pixels.
    pub fn block(&self) -> u32 {
        (2.0 + self.intensity * 0.3).round() as u32
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlurObject {
    pub rect: Rect,
    pub rotation: f64,
    pub style: BlurStyle,
}

impl BlurObject {
    pub fn from_props(props: &Map<String, Value>) -> Result<BlurObject, String> {
        let style = match props.get("style") {
            Some(Value::Object(s)) => BlurStyle::merged(&BlurStyle::default(), s)?,
            Some(_) => return Err("style must be an object".to_string()),
            None => BlurStyle::default(),
        };
        Ok(BlurObject {
            rect: rect(props.get("rect"))?,
            rotation: rotation(props),
            style,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn cases() -> Vec<Value> {
        let mut out = Vec::new();
        for (r, outline, rot) in [
            (
                Rect {
                    x: 10.0,
                    y: 20.0,
                    w: 200.0,
                    h: 100.0,
                },
                10.0,
                0.0,
            ),
            (
                Rect {
                    x: 10.5,
                    y: 20.25,
                    w: 33.3,
                    h: 7.0,
                },
                20.0,
                30.0,
            ),
            (
                Rect {
                    x: -50.0,
                    y: -20.0,
                    w: 120.0,
                    h: 80.0,
                },
                0.0,
                45.0,
            ),
            (
                Rect {
                    x: 380.0,
                    y: 190.0,
                    w: 60.0,
                    h: 40.0,
                },
                3.0,
                -15.0,
            ),
        ] {
            out.push(json!({
                "rect": r, "outline": outline, "rotation": rot,
                "inner": inner_rect(r, outline),
                "bounds": pixel_bounds(r, rot, 400, 200),
            }));
        }
        out
    }

    fn fixture_path() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/shape-geometry.json")
    }

    #[test]
    fn matches_the_shared_fixture() {
        let text = serde_json::to_string_pretty(&cases()).unwrap() + "\n";
        if std::env::var_os("XS_REGEN_FIXTURES").is_some() {
            std::fs::write(fixture_path(), &text).unwrap();
        }
        let saved: Value = serde_json::from_str(&std::fs::read_to_string(fixture_path()).unwrap()).unwrap();
        let now: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(saved, now, "update geometry.ts and regenerate the fixture");
    }

    #[test]
    fn outline_stays_inside_the_size() {
        let r = inner_rect(
            Rect {
                x: 0.0,
                y: 0.0,
                w: 200.0,
                h: 100.0,
            },
            10.0,
        );
        assert_eq!((r.x, r.y, r.w, r.h), (5.0, 5.0, 190.0, 90.0));
    }

    #[test]
    fn bounds_cover_rotation_and_clip_to_the_canvas() {
        let r = Rect {
            x: 0.0,
            y: 0.0,
            w: 100.0,
            h: 100.0,
        };
        assert_eq!(pixel_bounds(r, 0.0, 400, 400), Some([0, 0, 100, 100]));
        let [_, _, w, _] = pixel_bounds(Rect { x: 100.0, ..r }, 45.0, 400, 400).unwrap();
        assert!(w > 140, "a turned square is wider: {w}");
        assert_eq!(pixel_bounds(Rect { x: 500.0, ..r }, 0.0, 400, 400), None);
    }

    #[test]
    fn styles_validate() {
        let bad_shape =
            |v: Value| ShapeStyle::merged(&ShapeStyle::default(), v.as_object().unwrap()).is_err();
        assert!(bad_shape(json!({"outlineWidth": 101})));
        assert!(bad_shape(json!({"fill": "blue"})));
        assert!(bad_shape(json!({"shapeType": "star"})));
        assert!(!bad_shape(json!({"fill": null, "shapeType": "ellipse"})));
        assert!(!bad_shape(json!({"fill": "#FFFFFF80"})));
        let bad_blur = |v: Value| BlurStyle::merged(&BlurStyle::default(), v.as_object().unwrap()).is_err();
        assert!(bad_blur(json!({"intensity": 0})));
        assert!(bad_blur(json!({"blurType": "smudge"})));
        assert!(!bad_blur(json!({"blurType": "pixelate", "intensity": 100})));
        let small = BlurStyle {
            intensity: 20.0,
            ..BlurStyle::default()
        };
        let big = BlurStyle {
            intensity: 100.0,
            ..BlurStyle::default()
        };
        assert!(big.block() > small.block());
    }
}
