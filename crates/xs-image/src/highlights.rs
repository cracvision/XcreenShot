//! Highlighter (EDT-036): a translucent area that mixes with what is below like a
//! fluorescent marker, so dark text stays dark (multiply).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tiny_skia::{BlendMode, FillRule, PathBuilder, Pixmap, PixmapPaint, Transform};

use crate::color::Rgba;
use crate::render::{paint, turned};
use crate::shapes::{rect, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HighlightShape {
    Rectangle,
    RoundedRectangle,
    Ellipse,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct HighlightStyle {
    pub color: String,
    /// 0–100 %.
    pub opacity: f64,
    pub shape: HighlightShape,
    /// Only the text inside the area, one band per line (SMT-021).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub text_only: bool,
}

impl Default for HighlightStyle {
    /// The theme's yellow at half strength (EDT-017).
    fn default() -> Self {
        HighlightStyle {
            color: "#FFE14D".to_string(),
            opacity: 50.0,
            shape: HighlightShape::Rectangle,
            text_only: false,
        }
    }
}

impl HighlightStyle {
    pub fn merged(base: &HighlightStyle, changes: &Map<String, Value>) -> Result<HighlightStyle, String> {
        let mut v = serde_json::to_value(base).map_err(|e| e.to_string())?;
        if let Some(m) = v.as_object_mut() {
            for (k, val) in changes {
                m.insert(k.clone(), val.clone());
            }
        }
        let s: HighlightStyle = serde_json::from_value(v).map_err(|e| e.to_string())?;
        Rgba::parse(&s.color).ok_or("color must be a color like #RRGGBB")?;
        if !(0.0..=100.0).contains(&s.opacity) {
            return Err("opacity must be between 0 and 100".to_string());
        }
        Ok(s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct HighlightObject {
    pub rect: Rect,
    pub rotation: f64,
    pub style: HighlightStyle,
}

impl HighlightObject {
    pub fn from_props(props: &Map<String, Value>) -> Result<HighlightObject, String> {
        let style = match props.get("style") {
            Some(Value::Object(s)) => HighlightStyle::merged(&HighlightStyle::default(), s)?,
            Some(_) => return Err("style must be an object".to_string()),
            None => HighlightStyle::default(),
        };
        Ok(HighlightObject {
            rect: rect(props.get("rect"))?,
            rotation: props.get("rotation").and_then(Value::as_f64).unwrap_or(0.0),
            style,
        })
    }
}

fn path(shape: HighlightShape, r: Rect) -> Option<tiny_skia::Path> {
    let rect =
        tiny_skia::Rect::from_xywh(r.x as f32, r.y as f32, r.w.max(0.01) as f32, r.h.max(0.01) as f32)?;
    match shape {
        HighlightShape::Rectangle => Some(PathBuilder::from_rect(rect)),
        HighlightShape::Ellipse => PathBuilder::from_oval(rect),
        HighlightShape::RoundedRectangle => {
            // Corners a quarter of the shorter side, like a marker's round tip.
            let (x, y, w, h) = (r.x as f32, r.y as f32, r.w as f32, r.h as f32);
            let q = 0.25 * w.min(h);
            let k = 0.552_284_8 * q;
            let mut pb = PathBuilder::new();
            pb.move_to(x + q, y);
            pb.line_to(x + w - q, y);
            pb.cubic_to(x + w - q + k, y, x + w, y + q - k, x + w, y + q);
            pb.line_to(x + w, y + h - q);
            pb.cubic_to(x + w, y + h - q + k, x + w - q + k, y + h, x + w - q, y + h);
            pb.line_to(x + q, y + h);
            pb.cubic_to(x + q - k, y + h, x, y + h - q + k, x, y + h - q);
            pb.line_to(x, y + q);
            pb.cubic_to(x, y + q - k, x + q - k, y, x + q, y);
            pb.close();
            pb.finish()
        }
    }
}

/// Draws a highlight over `pm`, multiplying its color into what is below.
pub fn draw(pm: &mut Pixmap, h: &HighlightObject) {
    let Some(color) = Rgba::parse(&h.style.color) else {
        return;
    };
    let Some(p) = path(h.style.shape, h.rect) else {
        return;
    };
    let Some(mut layer) = Pixmap::new(pm.width(), pm.height()) else {
        return;
    };
    layer.fill_path(
        &p,
        &paint(color),
        FillRule::Winding,
        turned(h.rect, h.rotation),
        None,
    );
    let blend = PixmapPaint {
        opacity: (h.style.opacity / 100.0).clamp(0.0, 1.0) as f32,
        blend_mode: BlendMode::Multiply,
        ..PixmapPaint::default()
    };
    pm.draw_pixmap(0, 0, layer.as_ref(), &blend, Transform::identity(), None);
}
