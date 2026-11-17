//! Simplify (EDT-041): flat shapes that stand in for text and interface details, so an
//! image does not age when the interface changes. A document can show its original
//! image under them, dimmed, or not at all (`Document.simplify`).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tiny_skia::{FillRule, PathBuilder, Pixmap};

use crate::color::Rgba;
use crate::render::{blend_layer, paint, turned};
use crate::shapes::{rect, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SimpleKind {
    Block,
    RoundedBlock,
    /// A bar of one line's height.
    TextLine,
    /// Several bars, the last one shorter.
    Paragraph,
    Circle,
    Button,
    /// A generic icon: a rounded square with a dot.
    Icon,
}

/// A saved palette: the colors new simplified shapes take.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Palette {
    pub name: String,
    pub colors: Vec<String>,
}

/// The palettes that come with the app.
pub fn default_palettes() -> Vec<Palette> {
    let p = |name: &str, colors: &[&str]| Palette {
        name: name.to_string(),
        colors: colors.iter().map(|c| c.to_string()).collect(),
    };
    vec![
        p(
            "Neutral",
            &["#D9DDE3", "#B4BAC4", "#8A929E", "#5B6472", "#2D7FF9", "#FFFFFF"],
        ),
        p(
            "Ocean",
            &["#D6E8FF", "#9CC6FF", "#5A9DF2", "#2D6CC4", "#173E7A", "#FFFFFF"],
        ),
        p(
            "Warm",
            &["#FFE7D6", "#FFC9A3", "#F59E6C", "#E5484D", "#7A2E2E", "#FFFFFF"],
        ),
    ]
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct SimpleStyle {
    pub kind: SimpleKind,
    /// `#RRGGBB` or `#RRGGBBAA`.
    pub fill: String,
    /// Behind text lines and paragraphs; `None` for none.
    pub background: Option<String>,
    /// A text line's height, px.
    pub text_height: f64,
    /// Between a paragraph's lines, times the text height.
    pub line_spacing: f64,
    pub opacity: f64,
}

impl Default for SimpleStyle {
    fn default() -> Self {
        SimpleStyle {
            kind: SimpleKind::Block,
            fill: "#B4BAC4".to_string(),
            background: None,
            text_height: 10.0,
            line_spacing: 1.8,
            opacity: 100.0,
        }
    }
}

/// The Simplify tool's style: an object style plus the palettes (EDT-041).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct SimplifyToolStyle {
    #[serde(flatten)]
    pub shape: SimpleStyle,
    pub palettes: Vec<Palette>,
    /// The palette in use, by name.
    pub palette: String,
}

impl Default for SimplifyToolStyle {
    fn default() -> Self {
        SimplifyToolStyle {
            shape: SimpleStyle::default(),
            palettes: default_palettes(),
            palette: "Neutral".to_string(),
        }
    }
}

fn check(s: &SimpleStyle) -> Result<(), String> {
    Rgba::parse(&s.fill).ok_or("fill must be a color like #RRGGBB")?;
    if let Some(b) = &s.background {
        Rgba::parse(b).ok_or("background must be a color like #RRGGBB")?;
    }
    if !(2.0..=200.0).contains(&s.text_height) {
        return Err("textHeight must be between 2 and 200".to_string());
    }
    if !(1.0..=5.0).contains(&s.line_spacing) {
        return Err("lineSpacing must be between 1 and 5".to_string());
    }
    if !(0.0..=100.0).contains(&s.opacity) {
        return Err("opacity must be between 0 and 100".to_string());
    }
    Ok(())
}

impl SimpleStyle {
    pub fn merged(base: &SimpleStyle, changes: &Map<String, Value>) -> Result<SimpleStyle, String> {
        let mut v = serde_json::to_value(base).map_err(|e| e.to_string())?;
        if let Some(m) = v.as_object_mut() {
            for (k, val) in changes {
                m.insert(k.clone(), val.clone());
            }
        }
        let s: SimpleStyle = serde_json::from_value(v).map_err(|e| e.to_string())?;
        check(&s)?;
        Ok(s)
    }
}

impl SimplifyToolStyle {
    pub fn merged(
        base: &SimplifyToolStyle,
        changes: &Map<String, Value>,
    ) -> Result<SimplifyToolStyle, String> {
        let mut v = serde_json::to_value(base).map_err(|e| e.to_string())?;
        if let Some(m) = v.as_object_mut() {
            for (k, val) in changes {
                m.insert(k.clone(), val.clone());
            }
        }
        let s: SimplifyToolStyle = serde_json::from_value(v).map_err(|e| e.to_string())?;
        check(&s.shape)?;
        for p in &s.palettes {
            if p.name.trim().is_empty() || p.colors.is_empty() {
                return Err("each palette needs a name and colors".to_string());
            }
            for c in &p.colors {
                Rgba::parse(c).ok_or("palette colors must be like #RRGGBB")?;
            }
        }
        Ok(s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SimpleObject {
    pub rect: Rect,
    pub rotation: f64,
    pub style: SimpleStyle,
}

impl SimpleObject {
    pub fn from_props(props: &Map<String, Value>) -> Result<SimpleObject, String> {
        let style = match props.get("style") {
            Some(Value::Object(s)) => {
                // A tool style brings its palettes along: they are not the shape's.
                let mut s = s.clone();
                s.remove("palettes");
                s.remove("palette");
                SimpleStyle::merged(&SimpleStyle::default(), &s)?
            }
            Some(_) => return Err("style must be an object".to_string()),
            None => SimpleStyle::default(),
        };
        Ok(SimpleObject {
            rect: rect(props.get("rect"))?,
            rotation: props.get("rotation").and_then(Value::as_f64).unwrap_or(0.0),
            style,
        })
    }
}

fn rounded(x: f32, y: f32, w: f32, h: f32, r: f32) -> Option<tiny_skia::Path> {
    let q = r.clamp(0.0, w.min(h) / 2.0);
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

/// Draws a simplified shape (EDT-041).
pub fn draw(pm: &mut Pixmap, o: &SimpleObject) {
    let s = &o.style;
    let Some(fill) = Rgba::parse(&s.fill) else {
        return;
    };
    let r = o.rect;
    let t = turned(r, o.rotation);
    let (x, y, w, h) = (r.x as f32, r.y as f32, r.w.max(1.0) as f32, r.h.max(1.0) as f32);
    let back = s.background.as_deref().and_then(Rgba::parse);
    blend_layer(pm, s.opacity, |pm| {
        let mut put = |path: Option<tiny_skia::Path>, c: Rgba| {
            if let Some(p) = path {
                pm.fill_path(&p, &paint(c), FillRule::Winding, t, None);
            }
        };
        match s.kind {
            SimpleKind::Block => put(
                tiny_skia::Rect::from_xywh(x, y, w, h).map(PathBuilder::from_rect),
                fill,
            ),
            SimpleKind::RoundedBlock => put(rounded(x, y, w, h, 0.2 * w.min(h)), fill),
            SimpleKind::Circle => put(
                tiny_skia::Rect::from_xywh(x, y, w, h).and_then(PathBuilder::from_oval),
                fill,
            ),
            SimpleKind::TextLine => {
                if let Some(b) = back {
                    put(
                        tiny_skia::Rect::from_xywh(x, y, w, h).map(PathBuilder::from_rect),
                        b,
                    );
                }
                let bar = (s.text_height as f32).min(h);
                put(rounded(x, y + (h - bar) / 2.0, w, bar, bar / 2.0), fill);
            }
            SimpleKind::Paragraph => {
                if let Some(b) = back {
                    put(
                        tiny_skia::Rect::from_xywh(x, y, w, h).map(PathBuilder::from_rect),
                        b,
                    );
                }
                let bar = (s.text_height as f32).min(h);
                let step = (bar * s.line_spacing as f32).max(bar + 1.0);
                let lines = (((h - bar) / step).floor() as usize + 1).max(1);
                for i in 0..lines {
                    let last = i + 1 == lines && lines > 1;
                    let lw = if last { w * 0.62 } else { w };
                    put(rounded(x, y + i as f32 * step, lw, bar, bar / 2.0), fill);
                }
            }
            SimpleKind::Button => {
                put(rounded(x, y, w, h, 0.3 * h.min(w)), fill);
                let label = Rgba {
                    a: 200,
                    ..Rgba::WHITE
                };
                let (lw, lh) = (w * 0.5, (h * 0.22).max(2.0));
                put(
                    rounded(x + (w - lw) / 2.0, y + (h - lh) / 2.0, lw, lh, lh / 2.0),
                    label,
                );
            }
            SimpleKind::Icon => {
                let side = w.min(h);
                let (ix, iy) = (x + (w - side) / 2.0, y + (h - side) / 2.0);
                put(rounded(ix, iy, side, side, side * 0.25), fill);
                let dot = Rgba {
                    a: 200,
                    ..Rgba::WHITE
                };
                put(
                    PathBuilder::from_circle(ix + side / 2.0, iy + side / 2.0, side * 0.18),
                    dot,
                );
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_tool_style_leaves_its_palettes_out_of_a_shape() {
        let props = json!({"rect": {"x": 0, "y": 0, "w": 50, "h": 20}, "style": {"kind": "textLine", "palette": "Ocean", "palettes": []}});
        let o = SimpleObject::from_props(props.as_object().unwrap()).unwrap();
        assert_eq!(o.style.kind, SimpleKind::TextLine);
    }

    #[test]
    fn palettes_need_names_and_colors() {
        let bad = json!({"palettes": [{"name": "", "colors": ["#FFFFFF"]}]});
        assert!(SimplifyToolStyle::merged(&SimplifyToolStyle::default(), bad.as_object().unwrap()).is_err());
    }
}
