//! Magnify (EDT-038): a lens that shows a zone of what is below it, enlarged. The lens
//! can sit elsewhere: then the zone keeps a thin outline and two connectors join it to
//! the lens.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tiny_skia::{FillRule, FilterQuality, Paint, PathBuilder, Pattern, Pixmap, SpreadMode, Transform};

use crate::color::Rgba;
use crate::geometry::point;
use crate::render::{blend_layer, paint, stroke};
use crate::shapes::{rect, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LensShape {
    Circle,
    Square,
    Rectangle,
    RoundedRectangle,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct MagnifyStyle {
    pub shape: LensShape,
    /// 110–800 %.
    pub zoom: f64,
    pub outline: String,
    /// 0–20 px.
    pub outline_width: f64,
    /// Lines from the zone to a lens that sits elsewhere.
    pub connectors: bool,
    pub opacity: f64,
}

impl Default for MagnifyStyle {
    fn default() -> Self {
        MagnifyStyle {
            shape: LensShape::Circle,
            zoom: 200.0,
            outline: "#F2545B".to_string(),
            outline_width: 3.0,
            connectors: true,
            opacity: 100.0,
        }
    }
}

impl MagnifyStyle {
    pub fn merged(base: &MagnifyStyle, changes: &Map<String, Value>) -> Result<MagnifyStyle, String> {
        let mut v = serde_json::to_value(base).map_err(|e| e.to_string())?;
        if let Some(m) = v.as_object_mut() {
            for (k, val) in changes {
                m.insert(k.clone(), val.clone());
            }
        }
        let s: MagnifyStyle = serde_json::from_value(v).map_err(|e| e.to_string())?;
        Rgba::parse(&s.outline).ok_or("outline must be a color like #RRGGBB")?;
        if !(110.0..=800.0).contains(&s.zoom) {
            return Err("zoom must be between 110 and 800".to_string());
        }
        if !(0.0..=20.0).contains(&s.outline_width) {
            return Err("outlineWidth must be between 0 and 20".to_string());
        }
        if !(0.0..=100.0).contains(&s.opacity) {
            return Err("opacity must be between 0 and 100".to_string());
        }
        Ok(s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MagnifyObject {
    /// The zone shown, in canvas pixels.
    pub source: Rect,
    /// Where the lens's center is.
    pub at: [f64; 2],
    pub style: MagnifyStyle,
}

impl MagnifyObject {
    pub fn from_props(props: &Map<String, Value>) -> Result<MagnifyObject, String> {
        let style = match props.get("style") {
            Some(Value::Object(s)) => MagnifyStyle::merged(&MagnifyStyle::default(), s)?,
            Some(_) => return Err("style must be an object".to_string()),
            None => MagnifyStyle::default(),
        };
        let source = rect(props.get("source"))?;
        let at = match props.get("at") {
            Some(v) => point(Some(v), "at")?,
            None => [source.x + source.w / 2.0, source.y + source.h / 2.0],
        };
        Ok(MagnifyObject { source, at, style })
    }

    /// The lens: the zone's size times the zoom, around `at`.
    pub fn lens(&self) -> Rect {
        let k = self.style.zoom / 100.0;
        let (w, h) = (self.source.w * k, self.source.h * k);
        Rect {
            x: self.at[0] - w / 2.0,
            y: self.at[1] - h / 2.0,
            w,
            h,
        }
    }

    /// Whether the lens sits away from its zone (then the zone and connectors show).
    pub fn moved(&self) -> bool {
        let c = [
            self.source.x + self.source.w / 2.0,
            self.source.y + self.source.h / 2.0,
        ];
        (c[0] - self.at[0]).hypot(c[1] - self.at[1]) > 1.0
    }

    /// The box around zone and lens, `[x0, y0, x1, y1]`, without the outline.
    pub fn span(&self) -> [f64; 4] {
        let (s, l) = (self.source, self.lens());
        [
            s.x.min(l.x),
            s.y.min(l.y),
            (s.x + s.w).max(l.x + l.w),
            (s.y + s.h).max(l.y + l.h),
        ]
    }
}

fn shape_path(shape: LensShape, r: Rect) -> Option<tiny_skia::Path> {
    let rect =
        tiny_skia::Rect::from_xywh(r.x as f32, r.y as f32, r.w.max(0.01) as f32, r.h.max(0.01) as f32)?;
    match shape {
        LensShape::Circle => PathBuilder::from_oval(rect),
        LensShape::Square | LensShape::Rectangle => Some(PathBuilder::from_rect(rect)),
        LensShape::RoundedRectangle => {
            let (x, y, w, h) = (r.x as f32, r.y as f32, r.w as f32, r.h as f32);
            let q = 0.15 * w.min(h);
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

/// The point of the shape in `r` farthest along the unit direction `n`, from its
/// center: where a connector parallel to the line between zone and lens touches it.
fn support(shape: LensShape, r: Rect, n: (f64, f64)) -> (f64, f64) {
    let (a, b) = (r.w / 2.0, r.h / 2.0);
    match shape {
        LensShape::Circle => {
            let d = (a * a * n.0 * n.0 + b * b * n.1 * n.1).sqrt().max(1e-9);
            (a * a * n.0 / d, b * b * n.1 / d)
        }
        _ => {
            let side = |v: f64, half: f64| if v.abs() < 1e-9 { 0.0 } else { half * v.signum() };
            (side(n.0, a), side(n.1, b))
        }
    }
}

/// Draws the lens over `pm`, taking what it shows from `below` (canvas pixels). `shift`
/// places canvas pixels on `pm` (identity when `pm` is the canvas).
pub fn draw(pm: &mut Pixmap, below: &Pixmap, m: &MagnifyObject, shift: Transform) {
    let s = &m.style;
    let lens = m.lens();
    let src = m.source;
    let outline = Rgba::parse(&s.outline).unwrap_or(Rgba::WHITE);
    let ow = s.outline_width;
    blend_layer(pm, s.opacity, |pm| {
        // The zone and the connectors, when the lens sits elsewhere.
        if m.moved() {
            let thin = stroke(ow.clamp(1.0, 2.0), None);
            if let Some(p) = shape_path(s.shape, src) {
                pm.stroke_path(&p, &paint(outline), &thin, shift, None);
            }
            if s.connectors {
                let sc = (src.x + src.w / 2.0, src.y + src.h / 2.0);
                let (dx, dy) = (m.at[0] - sc.0, m.at[1] - sc.1);
                let len = dx.hypot(dy).max(1e-9);
                let n = (-dy / len, dx / len);
                for sign in [1.0, -1.0] {
                    let dir = (n.0 * sign, n.1 * sign);
                    let (zs, ls) = (support(s.shape, src, dir), support(s.shape, lens, dir));
                    let mut pb = PathBuilder::new();
                    pb.move_to((sc.0 + zs.0) as f32, (sc.1 + zs.1) as f32);
                    pb.line_to((m.at[0] + ls.0) as f32, (m.at[1] + ls.1) as f32);
                    if let Some(p) = pb.finish() {
                        pm.stroke_path(&p, &paint(outline), &thin, shift, None);
                    }
                }
            }
        }
        // The lens: the zone of `below`, enlarged.
        let Some(path) = shape_path(s.shape, lens) else {
            return;
        };
        let k = (s.zoom / 100.0) as f32;
        let to_lens = Transform::from_translate(lens.x as f32, lens.y as f32)
            .pre_scale(k, k)
            .pre_translate(-src.x as f32, -src.y as f32);
        let paint = Paint {
            shader: Pattern::new(
                below.as_ref(),
                SpreadMode::Pad,
                FilterQuality::Bicubic,
                1.0,
                // fill_path moves the shader with the path (`shift`).
                to_lens,
            ),
            anti_alias: true,
            ..Paint::default()
        };
        pm.fill_path(&path, &paint, FillRule::Winding, shift, None);
        if ow > 0.0 {
            if let Some(p) = shape_path(s.shape, crate::shapes::inner_rect(lens, ow)) {
                pm.stroke_path(&p, &crate::render::paint(outline), &stroke(ow, None), shift, None);
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn a_lens_is_the_zone_times_the_zoom() {
        let props = json!({"source": {"x": 10, "y": 10, "w": 50, "h": 50}, "style": {"zoom": 300}});
        let m = MagnifyObject::from_props(props.as_object().unwrap()).unwrap();
        let l = m.lens();
        assert_eq!((l.w, l.h), (150.0, 150.0));
        assert_eq!(m.at, [35.0, 35.0], "on its zone at first");
        assert!(!m.moved());
    }
}
