//! Spotlight (EDT-039): everything outside one or more zones is darkened, blurred or
//! both. Several spotlights share the outside: the union of their zones stays lit and
//! the rest changes once, with the settings they all share.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tiny_skia::{FillRule, PathBuilder, Pixmap};

use crate::color::Rgba;
use crate::effects;
use crate::render::{paint, turned};
use crate::shapes::{rect, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SpotShape {
    Rectangle,
    RoundedRectangle,
    Ellipse,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct SpotlightStyle {
    pub shape: SpotShape,
    /// The outside's color.
    pub color: String,
    /// 0–100 % of that color over the outside.
    pub opacity: f64,
    /// 0–100: how much the outside is blurred.
    pub blur: f64,
    /// 0–50 px of soft edge.
    pub feather: f64,
    /// Only the text inside the area, one band per line (SMT-021).
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub text_only: bool,
}

impl Default for SpotlightStyle {
    fn default() -> Self {
        SpotlightStyle {
            shape: SpotShape::Rectangle,
            color: "#000000".to_string(),
            opacity: 60.0,
            blur: 0.0,
            feather: 0.0,
            text_only: false,
        }
    }
}

/// The settings every spotlight of a document shares (EDT-039): all but the shape.
pub const SHARED: [&str; 4] = ["color", "opacity", "blur", "feather"];

impl SpotlightStyle {
    pub fn merged(base: &SpotlightStyle, changes: &Map<String, Value>) -> Result<SpotlightStyle, String> {
        let mut v = serde_json::to_value(base).map_err(|e| e.to_string())?;
        if let Some(m) = v.as_object_mut() {
            for (k, val) in changes {
                m.insert(k.clone(), val.clone());
            }
        }
        let s: SpotlightStyle = serde_json::from_value(v).map_err(|e| e.to_string())?;
        Rgba::parse(&s.color).ok_or("color must be a color like #RRGGBB")?;
        for (name, v, hi) in [
            ("opacity", s.opacity, 100.0),
            ("blur", s.blur, 100.0),
            ("feather", s.feather, 50.0),
        ] {
            if !(0.0..=hi).contains(&v) {
                return Err(format!("{name} must be between 0 and {hi}"));
            }
        }
        Ok(s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SpotlightObject {
    pub rect: Rect,
    pub rotation: f64,
    pub style: SpotlightStyle,
}

impl SpotlightObject {
    pub fn from_props(props: &Map<String, Value>) -> Result<SpotlightObject, String> {
        let style = match props.get("style") {
            Some(Value::Object(s)) => SpotlightStyle::merged(&SpotlightStyle::default(), s)?,
            Some(_) => return Err("style must be an object".to_string()),
            None => SpotlightStyle::default(),
        };
        Ok(SpotlightObject {
            rect: rect(props.get("rect"))?,
            rotation: props.get("rotation").and_then(Value::as_f64).unwrap_or(0.0),
            style,
        })
    }
}

fn zone_path(shape: SpotShape, r: Rect) -> Option<tiny_skia::Path> {
    let rect =
        tiny_skia::Rect::from_xywh(r.x as f32, r.y as f32, r.w.max(0.01) as f32, r.h.max(0.01) as f32)?;
    match shape {
        SpotShape::Rectangle => Some(PathBuilder::from_rect(rect)),
        SpotShape::Ellipse => PathBuilder::from_oval(rect),
        SpotShape::RoundedRectangle => {
            let (x, y, w, h) = (r.x as f32, r.y as f32, r.w as f32, r.h as f32);
            let q = 0.2 * w.min(h);
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

/// Darkens and blurs `pm` outside the zones of `spots`, once, with `style`.
pub fn draw(pm: &mut Pixmap, spots: &[SpotlightObject], style: &SpotlightStyle) {
    let (w, h) = (pm.width() as usize, pm.height() as usize);
    let Some(mut lit) = Pixmap::new(pm.width(), pm.height()) else {
        return;
    };
    for s in spots {
        if let Some(p) = zone_path(s.shape(), s.rect) {
            lit.fill_path(
                &p,
                &paint(Rgba::WHITE),
                FillRule::Winding,
                turned(s.rect, s.rotation),
                None,
            );
        }
    }
    if style.feather > 0.0 {
        effects::blur(lit.data_mut(), w, h, (style.feather / 2.0).round() as usize);
    }
    let mut outside = pm.clone();
    if style.blur > 0.0 {
        effects::blur(
            outside.data_mut(),
            w,
            h,
            (style.blur / 5.0).round().max(1.0) as usize,
        );
    }
    let c = Rgba::parse(&style.color).unwrap_or(Rgba {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    });
    let a = (style.opacity / 100.0).clamp(0.0, 1.0) * c.a as f64 / 255.0;
    let tint = [c.r as f64, c.g as f64, c.b as f64];
    let mask = lit.data();
    let out = outside.data();
    for (i, px) in pm.data_mut().as_chunks_mut::<4>().0.iter_mut().enumerate() {
        let m = mask[i * 4 + 3] as f64 / 255.0;
        if m >= 1.0 {
            continue;
        }
        let o = &out[i * 4..i * 4 + 4];
        let alpha = o[3] as f64 / 255.0;
        for ch in 0..3 {
            // Premultiplied: the tint covers the outside where it has pixels.
            let dark = o[ch] as f64 * (1.0 - a) + tint[ch] * alpha * a;
            px[ch] = (px[ch] as f64 * m + dark * (1.0 - m)).round().clamp(0.0, 255.0) as u8;
        }
        px[3] = (px[3] as f64 * m + o[3] as f64 * (1.0 - m)).round() as u8;
    }
}

impl SpotlightObject {
    fn shape(&self) -> SpotShape {
        self.style.shape
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spot(x: f64) -> SpotlightObject {
        SpotlightObject {
            rect: Rect {
                x,
                y: 0.0,
                w: 10.0,
                h: 10.0,
            },
            rotation: 0.0,
            style: SpotlightStyle::default(),
        }
    }

    fn white(w: u32, h: u32) -> Pixmap {
        let mut pm = Pixmap::new(w, h).unwrap();
        pm.fill(tiny_skia::Color::WHITE);
        pm
    }

    #[test]
    fn the_outside_darkens_once_and_the_zones_stay() {
        let mut pm = white(40, 10);
        let spots = [spot(0.0), spot(20.0)];
        draw(&mut pm, &spots, &SpotlightStyle::default());
        let px = |x: u32| pm.pixel(x, 5).unwrap().demultiply();
        assert_eq!(px(5).red(), 255, "lit");
        assert_eq!(px(25).red(), 255, "lit too");
        // 60 % black over white, once.
        assert_eq!(px(15).red(), 102);
        assert_eq!(px(35).red(), 102);
    }

    #[test]
    fn blur_without_opacity_does_not_darken() {
        let mut pm = Pixmap::new(40, 10).unwrap();
        for x in 0..40 {
            let c = if x % 2 == 0 { 0 } else { 255 };
            for y in 0..10 {
                let i = ((y * 40 + x) * 4) as usize;
                pm.data_mut()[i..i + 4].copy_from_slice(&[c, c, c, 255]);
            }
        }
        let style = SpotlightStyle {
            opacity: 0.0,
            blur: 50.0,
            ..SpotlightStyle::default()
        };
        draw(&mut pm, &[spot(0.0)], &style);
        let p = pm.pixel(30, 5).unwrap().demultiply();
        assert!(
            (100..=155).contains(&p.red()),
            "blurred to grey, not darkened: {}",
            p.red()
        );
        assert_eq!(pm.pixel(4, 5).unwrap().demultiply().red(), 0, "inside unchanged");
    }
}
