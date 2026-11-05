//! Geometry of annotation objects, in canvas pixels.
//!
//! These formulas are the reference: `app/src/editor/geometry.ts` repeats them for the
//! canvas, and `tests/fixtures/arrow-geometry.json` (written by this module's tests
//! with `XS_REGEN_FIXTURES=1`) is checked by both sides (ADR 0004).

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::color::Rgba;

/// End of an arrow (EDT-024).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Cap {
    None,
    Filled,
    Open,
    Thin,
    Concave,
    Circle,
    CircleHollow,
    Square,
    Diamond,
    Bar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LineStyle {
    Solid,
    Dash,
    Dot,
    DashDot,
}

/// Arrow properties of M1 (EDT-024). Curves, cap sizes and shadow arrive in M2.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct ArrowStyle {
    pub color: String,
    /// 1–100 px.
    pub width: f64,
    pub start_cap: Cap,
    pub end_cap: Cap,
    pub line_style: LineStyle,
    /// 0–100 %.
    pub opacity: f64,
}

impl Default for ArrowStyle {
    /// The arrow of the default theme (EDT-017: coral #F2545B).
    fn default() -> Self {
        ArrowStyle {
            color: "#F2545B".to_string(),
            width: 6.0,
            start_cap: Cap::None,
            end_cap: Cap::Filled,
            line_style: LineStyle::Solid,
            opacity: 100.0,
        }
    }
}

impl ArrowStyle {
    pub fn validate(&self) -> Result<(), String> {
        if Rgba::parse(&self.color).is_none() {
            return Err(format!("color \"{}\" is not #RRGGBB or #RRGGBBAA", self.color));
        }
        if !(1.0..=100.0).contains(&self.width) {
            return Err("width must be between 1 and 100".to_string());
        }
        if !(0.0..=100.0).contains(&self.opacity) {
            return Err("opacity must be between 0 and 100".to_string());
        }
        Ok(())
    }

    /// `base` with the properties in `changes`; unknown or invalid ones are an error.
    pub fn merged(base: &ArrowStyle, changes: &Map<String, Value>) -> Result<ArrowStyle, String> {
        let mut v = serde_json::to_value(base).map_err(|e| e.to_string())?;
        if let Some(obj) = v.as_object_mut() {
            for (k, val) in changes {
                obj.insert(k.clone(), val.clone());
            }
        }
        let style: ArrowStyle = serde_json::from_value(v).map_err(|e| format!("style: {e}"))?;
        style.validate()?;
        Ok(style)
    }
}

/// An arrow object: from its tail (`from`) to its point (`to`).
#[derive(Debug, Clone, PartialEq)]
pub struct Arrow {
    pub from: [f64; 2],
    pub to: [f64; 2],
    pub style: ArrowStyle,
}

pub fn point(v: Option<&Value>, name: &str) -> Result<[f64; 2], String> {
    let arr = v
        .and_then(Value::as_array)
        .filter(|a| a.len() == 2)
        .ok_or_else(|| format!("{name} must be [x, y]"))?;
    let x = arr[0].as_f64().ok_or_else(|| format!("{name} must be [x, y]"))?;
    let y = arr[1].as_f64().ok_or_else(|| format!("{name} must be [x, y]"))?;
    Ok([x, y])
}

impl Arrow {
    /// Reads an arrow stored in a document. Missing style properties take the defaults.
    pub fn from_props(props: &Map<String, Value>) -> Result<Arrow, String> {
        let style = match props.get("style") {
            Some(Value::Object(s)) => ArrowStyle::merged(&ArrowStyle::default(), s)?,
            Some(_) => return Err("style must be an object".to_string()),
            None => ArrowStyle::default(),
        };
        Ok(Arrow {
            from: point(props.get("from"), "from")?,
            to: point(props.get("to"), "to")?,
            style,
        })
    }

    pub fn length(&self) -> f64 {
        (self.to[0] - self.from[0]).hypot(self.to[1] - self.from[1])
    }

    pub fn geometry(&self) -> Geometry {
        arrow_geometry(self.from, self.to, &self.style)
    }
}

/// A piece of an object to draw: a polygon or polyline, or a circle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Shape {
    #[serde(rename_all = "camelCase")]
    Path {
        /// x0, y0, x1, y1…
        points: Vec<f64>,
        closed: bool,
        fill: bool,
        /// Stroked with round joins and butt ends when not filled.
        stroke_width: f64,
    },
    #[serde(rename_all = "camelCase")]
    Circle {
        cx: f64,
        cy: f64,
        r: f64,
        fill: bool,
        stroke_width: f64,
    },
}

/// What to draw for an arrow: the shaft (butt ends, optional dashes) and its ends.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Geometry {
    /// x1, y1, x2, y2. `None` when the arrow has no length.
    pub shaft: Option<[f64; 4]>,
    pub width: f64,
    /// On/off lengths, in pixels.
    pub dash: Option<Vec<f64>>,
    pub caps: Vec<Shape>,
}

/// Dash pattern of a line style, proportional to the width.
pub fn dash(style: LineStyle, width: f64) -> Option<Vec<f64>> {
    match style {
        LineStyle::Solid => None,
        LineStyle::Dash => Some(vec![3.0 * width, 2.0 * width]),
        LineStyle::Dot => Some(vec![width, width]),
        LineStyle::DashDot => Some(vec![3.0 * width, 1.5 * width, width, 1.5 * width]),
    }
}

/// One arrow end at `tip`, pointing along the unit vector `u`. Returns the shapes
/// and how far the shaft stops short of the tip.
fn cap(kind: Cap, tip: [f64; 2], u: [f64; 2], w: f64, scale: f64) -> (Vec<Shape>, f64) {
    let n = [-u[1], u[0]];
    let l = (3.0 * w + 6.0) * scale;
    let h = 0.45 * (3.0 * w + 6.0) * scale;
    let r = (1.5 * w + 2.0) * scale;
    // tip + a·u + b·n
    let at = |a: f64, b: f64| [tip[0] + a * u[0] + b * n[0], tip[1] + a * u[1] + b * n[1]];
    let path = |pts: &[[f64; 2]], closed: bool, fill: bool, stroke_width: f64| Shape::Path {
        points: pts.iter().flat_map(|p| [p[0], p[1]]).collect(),
        closed,
        fill,
        stroke_width,
    };
    match kind {
        Cap::None => (vec![], 0.0),
        Cap::Filled => (
            vec![path(&[tip, at(-l, h), at(-l, -h)], true, true, 0.0)],
            l / 2.0,
        ),
        Cap::Open => (
            vec![path(&[at(-l, h), tip, at(-l, -h)], false, false, w)],
            w / 2.0,
        ),
        Cap::Thin => {
            let h = 0.3 * l;
            (
                vec![path(&[at(-l, h), tip, at(-l, -h)], false, false, w)],
                w / 2.0,
            )
        }
        Cap::Concave => (
            vec![path(
                &[tip, at(-l, h), at(-0.65 * l, 0.0), at(-l, -h)],
                true,
                true,
                0.0,
            )],
            0.45 * l,
        ),
        Cap::Circle => (
            vec![Shape::Circle {
                cx: tip[0],
                cy: tip[1],
                r,
                fill: true,
                stroke_width: 0.0,
            }],
            0.0,
        ),
        Cap::CircleHollow => (
            vec![Shape::Circle {
                cx: tip[0],
                cy: tip[1],
                r,
                fill: false,
                stroke_width: (w * 0.5).max(1.0),
            }],
            r,
        ),
        Cap::Square => (
            vec![path(
                &[at(r, r), at(r, -r), at(-r, -r), at(-r, r)],
                true,
                true,
                0.0,
            )],
            0.0,
        ),
        Cap::Diamond => (
            vec![path(
                &[at(1.3 * r, 0.0), at(0.0, r), at(-1.3 * r, 0.0), at(0.0, -r)],
                true,
                true,
                0.0,
            )],
            0.0,
        ),
        Cap::Bar => (vec![path(&[at(0.0, h), at(0.0, -h)], false, false, w)], 0.0),
    }
}

pub fn arrow_geometry(from: [f64; 2], to: [f64; 2], style: &ArrowStyle) -> Geometry {
    let w = style.width;
    let dx = to[0] - from[0];
    let dy = to[1] - from[1];
    let len = dx.hypot(dy);
    if len < 1e-6 {
        return Geometry {
            shaft: None,
            width: w,
            dash: None,
            caps: Vec::new(),
        };
    }
    let d = [dx / len, dy / len];
    // Ends never take more than 40 % of the arrow each.
    let scale = (0.4 * len / (3.0 * w + 6.0)).min(1.0);
    let (mut caps, start_inset) = cap(style.start_cap, from, [-d[0], -d[1]], w, scale);
    let (end_caps, end_inset) = cap(style.end_cap, to, d, w, scale);
    caps.extend(end_caps);
    Geometry {
        shaft: Some([
            from[0] + start_inset * d[0],
            from[1] + start_inset * d[1],
            to[0] - end_inset * d[0],
            to[1] - end_inset * d[1],
        ]),
        width: w,
        dash: dash(style.line_style, w),
        caps,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Cases shared with the canvas (app/src/editor/geometry.test.ts).
    fn cases() -> Vec<(Value, Geometry)> {
        let caps = [
            Cap::None,
            Cap::Filled,
            Cap::Open,
            Cap::Thin,
            Cap::Concave,
            Cap::Circle,
            Cap::CircleHollow,
            Cap::Square,
            Cap::Diamond,
            Cap::Bar,
        ];
        let lines = [
            LineStyle::Solid,
            LineStyle::Dash,
            LineStyle::Dot,
            LineStyle::DashDot,
        ];
        let mut out = Vec::new();
        for (i, c) in caps.iter().enumerate() {
            let style = ArrowStyle {
                width: 2.5 + i as f64,
                start_cap: caps[(i + 3) % caps.len()],
                end_cap: *c,
                line_style: lines[i % lines.len()],
                ..ArrowStyle::default()
            };
            for (from, to) in [([100.0, 100.0], [300.0, 100.0]), ([10.0, 50.0], [25.0, 20.0])] {
                let input = json!({"from": from, "to": to, "style": style});
                out.push((input, arrow_geometry(from, to, &style)));
            }
        }
        out.push((
            json!({"from": [5, 5], "to": [5, 5], "style": ArrowStyle::default()}),
            arrow_geometry([5.0, 5.0], [5.0, 5.0], &ArrowStyle::default()),
        ));
        out
    }

    fn fixture_path() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/arrow-geometry.json")
    }

    #[test]
    fn matches_the_shared_fixture() {
        let cases: Vec<Value> = cases()
            .into_iter()
            .map(|(input, g)| json!({"input": input, "geometry": g}))
            .collect();
        let text = serde_json::to_string_pretty(&cases).unwrap() + "\n";
        if std::env::var_os("XS_REGEN_FIXTURES").is_some() {
            std::fs::create_dir_all(fixture_path().parent().unwrap()).unwrap();
            std::fs::write(fixture_path(), &text).unwrap();
        }
        let saved = std::fs::read_to_string(fixture_path())
            .expect("run once with XS_REGEN_FIXTURES=1 to write the fixture");
        let saved: Value = serde_json::from_str(&saved).unwrap();
        assert!(
            close(&saved, &Value::Array(cases)),
            "geometry changed: update geometry.ts and regenerate the fixture"
        );
    }

    /// Equal JSON, with numbers equal to 1e-9 (text round trips may move the last digit).
    fn close(a: &Value, b: &Value) -> bool {
        match (a, b) {
            (Value::Number(x), Value::Number(y)) => (x.as_f64().unwrap() - y.as_f64().unwrap()).abs() < 1e-9,
            (Value::Array(x), Value::Array(y)) => {
                x.len() == y.len() && x.iter().zip(y).all(|(p, q)| close(p, q))
            }
            (Value::Object(x), Value::Object(y)) => {
                x.len() == y.len() && x.iter().all(|(k, v)| y.get(k).is_some_and(|w| close(v, w)))
            }
            _ => a == b,
        }
    }

    #[test]
    fn horizontal_arrow_ends_at_its_point() {
        let a =
            Arrow::from_props(json!({"from": [100, 100], "to": [300, 100]}).as_object().unwrap()).unwrap();
        let g = a.geometry();
        let Shape::Path { points, .. } = &g.caps[0] else {
            panic!("filled head is a path")
        };
        assert_eq!((points[0], points[1]), (300.0, 100.0));
        // Shaft stops inside the head, never past the point.
        assert!(g.shaft.unwrap()[2] < 300.0);
    }

    #[test]
    fn rejects_unknown_or_out_of_range_style() {
        let base = ArrowStyle::default();
        let bad = |v: Value| ArrowStyle::merged(&base, v.as_object().unwrap()).is_err();
        assert!(bad(json!({"colour": "#000000"})));
        assert!(bad(json!({"width": 0})));
        assert!(bad(json!({"opacity": 101})));
        assert!(bad(json!({"endCap": "spiral"})));
        assert!(bad(json!({"color": "red"})));
        assert!(!bad(
            json!({"color": "#000000", "width": 2.5, "endCap": "circle"})
        ));
    }
}
