//! Pen (EDT-037): a freehand stroke through `points`, with a tip (round, square, or a
//! calligraphic bar), smoothed when it is made.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tiny_skia::{FillRule, LineCap, LineJoin, PathBuilder, Pixmap, Stroke, Transform};

use crate::color::Rgba;
use crate::geometry::point;
use crate::render::{blend_layer, paint};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PenTip {
    Circle,
    Square,
    /// A thin bar at 45°, like a calligraphy pen.
    Slash,
    /// A thin horizontal bar.
    Bar,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct PenStyle {
    pub color: String,
    /// 1–100 px.
    pub width: f64,
    pub opacity: f64,
    pub tip: PenTip,
    /// Smooth edges.
    pub antialias: bool,
    /// 0–100 %: how much the hand's tremble is smoothed when the stroke is made.
    pub smooth: f64,
    /// Make an editable stroke; otherwise it is painted into the image.
    pub editable: bool,
}

impl Default for PenStyle {
    fn default() -> Self {
        PenStyle {
            color: "#F2545B".to_string(),
            width: 4.0,
            opacity: 100.0,
            tip: PenTip::Circle,
            antialias: true,
            smooth: 30.0,
            editable: true,
        }
    }
}

impl PenStyle {
    pub fn merged(base: &PenStyle, changes: &Map<String, Value>) -> Result<PenStyle, String> {
        let mut v = serde_json::to_value(base).map_err(|e| e.to_string())?;
        if let Some(m) = v.as_object_mut() {
            for (k, val) in changes {
                m.insert(k.clone(), val.clone());
            }
        }
        let s: PenStyle = serde_json::from_value(v).map_err(|e| e.to_string())?;
        Rgba::parse(&s.color).ok_or("color must be a color like #RRGGBB")?;
        if !(1.0..=100.0).contains(&s.width) {
            return Err("width must be between 1 and 100".to_string());
        }
        if !(0.0..=100.0).contains(&s.opacity) {
            return Err("opacity must be between 0 and 100".to_string());
        }
        if !(0.0..=100.0).contains(&s.smooth) {
            return Err("smooth must be between 0 and 100".to_string());
        }
        Ok(s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PenObject {
    pub points: Vec<[f64; 2]>,
    pub style: PenStyle,
}

impl PenObject {
    pub fn from_props(props: &Map<String, Value>) -> Result<PenObject, String> {
        let style = match props.get("style") {
            Some(Value::Object(s)) => PenStyle::merged(&PenStyle::default(), s)?,
            Some(_) => return Err("style must be an object".to_string()),
            None => PenStyle::default(),
        };
        let points = props
            .get("points")
            .and_then(Value::as_array)
            .ok_or("points must be a list of [x, y]")?
            .iter()
            .map(|p| point(Some(p), "each point"))
            .collect::<Result<Vec<_>, _>>()?;
        if points.is_empty() {
            return Err("a stroke needs at least one point".to_string());
        }
        Ok(PenObject { points, style })
    }

    /// What it covers, `[x0, y0, x1, y1]`, with the tip's reach.
    pub fn bounds(&self) -> [f64; 4] {
        let r = self.style.width / 2.0 + 1.0;
        let mut b = [f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY];
        for p in &self.points {
            b = [
                b[0].min(p[0] - r),
                b[1].min(p[1] - r),
                b[2].max(p[0] + r),
                b[3].max(p[1] + r),
            ];
        }
        b
    }
}

/// The stroke as made by a hand: smoothed by `amount` % (moving averages, the ends
/// kept), then without points that add nothing.
pub fn smoothed(points: &[[f64; 2]], amount: f64) -> Vec<[f64; 2]> {
    let mut pts = points.to_vec();
    let passes = (amount.clamp(0.0, 100.0) / 100.0 * 6.0).round() as usize;
    for _ in 0..passes {
        if pts.len() < 3 {
            break;
        }
        let mut next = pts.clone();
        for i in 1..pts.len() - 1 {
            next[i] = [
                (pts[i - 1][0] + 2.0 * pts[i][0] + pts[i + 1][0]) / 4.0,
                (pts[i - 1][1] + 2.0 * pts[i][1] + pts[i + 1][1]) / 4.0,
            ];
        }
        pts = next;
    }
    simplified(&pts, 0.25)
}

/// Ramer–Douglas–Peucker: drops points closer than `eps` to the line between their
/// neighbours that stay.
fn simplified(pts: &[[f64; 2]], eps: f64) -> Vec<[f64; 2]> {
    if pts.len() < 3 {
        return pts.to_vec();
    }
    let (a, b) = (pts[0], pts[pts.len() - 1]);
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let len = dx.hypot(dy);
    let dist = |p: [f64; 2]| {
        if len < 1e-9 {
            (p[0] - a[0]).hypot(p[1] - a[1])
        } else {
            ((p[0] - a[0]) * dy - (p[1] - a[1]) * dx).abs() / len
        }
    };
    let (i, d) = pts[1..pts.len() - 1]
        .iter()
        .enumerate()
        .map(|(i, p)| (i + 1, dist(*p)))
        .fold((0, 0.0), |m, x| if x.1 > m.1 { x } else { m });
    if d <= eps {
        return vec![a, b];
    }
    let mut left = simplified(&pts[..=i], eps);
    let right = simplified(&pts[i..], eps);
    left.pop();
    left.extend(right);
    left
}

/// Draws a stroke (EDT-037).
pub fn draw(pm: &mut Pixmap, p: &PenObject) {
    let Some(color) = Rgba::parse(&p.style.color) else {
        return;
    };
    let w = p.style.width;
    let pts = &p.points;
    let mut ink = paint(color);
    ink.anti_alias = p.style.antialias;
    blend_layer(pm, p.style.opacity, |pm| match p.style.tip {
        PenTip::Circle | PenTip::Square => {
            let round = p.style.tip == PenTip::Circle;
            if pts.len() == 1 || pts.iter().all(|q| *q == pts[0]) {
                // A dot.
                let [x, y] = pts[0];
                let path = if round {
                    PathBuilder::from_circle(x as f32, y as f32, (w / 2.0) as f32)
                } else {
                    tiny_skia::Rect::from_xywh((x - w / 2.0) as f32, (y - w / 2.0) as f32, w as f32, w as f32)
                        .map(PathBuilder::from_rect)
                };
                if let Some(path) = path {
                    pm.fill_path(&path, &ink, FillRule::Winding, Transform::identity(), None);
                }
                return;
            }
            // Through the midpoints, curving at each point: smooth without wiggles.
            let mut pb = PathBuilder::new();
            pb.move_to(pts[0][0] as f32, pts[0][1] as f32);
            for i in 1..pts.len() - 1 {
                let mid = [
                    (pts[i][0] + pts[i + 1][0]) / 2.0,
                    (pts[i][1] + pts[i + 1][1]) / 2.0,
                ];
                pb.quad_to(pts[i][0] as f32, pts[i][1] as f32, mid[0] as f32, mid[1] as f32);
            }
            let last = pts[pts.len() - 1];
            pb.line_to(last[0] as f32, last[1] as f32);
            if let Some(path) = pb.finish() {
                let st = Stroke {
                    width: w as f32,
                    line_cap: if round { LineCap::Round } else { LineCap::Square },
                    line_join: if round { LineJoin::Round } else { LineJoin::Miter },
                    ..Stroke::default()
                };
                pm.stroke_path(&path, &ink, &st, Transform::identity(), None);
            }
        }
        PenTip::Slash | PenTip::Bar => {
            // A flat nib swept along the stroke: its width depends on the direction.
            let (nx, ny) = if p.style.tip == PenTip::Slash {
                (std::f64::consts::FRAC_1_SQRT_2, -std::f64::consts::FRAC_1_SQRT_2)
            } else {
                (1.0, 0.0)
            };
            let (hx, hy) = (nx * w / 2.0, ny * w / 2.0);
            let t = (w * 0.12).max(1.0) / 2.0;
            let (tx, ty) = (-ny * t, nx * t);
            let nib = |c: [f64; 2]| {
                [
                    (c[0] + hx + tx, c[1] + hy + ty),
                    (c[0] - hx + tx, c[1] - hy + ty),
                    (c[0] - hx - tx, c[1] - hy - ty),
                    (c[0] + hx - tx, c[1] + hy - ty),
                ]
            };
            let fill = |pm: &mut Pixmap, quad: [(f64, f64); 4]| {
                let mut pb = PathBuilder::new();
                pb.move_to(quad[0].0 as f32, quad[0].1 as f32);
                for q in &quad[1..] {
                    pb.line_to(q.0 as f32, q.1 as f32);
                }
                pb.close();
                if let Some(path) = pb.finish() {
                    pm.fill_path(&path, &ink, FillRule::Winding, Transform::identity(), None);
                }
            };
            for c in pts {
                fill(pm, nib(*c));
            }
            for pair in pts.windows(2) {
                let (a, b) = (pair[0], pair[1]);
                fill(
                    pm,
                    [
                        (a[0] + hx, a[1] + hy),
                        (a[0] - hx, a[1] - hy),
                        (b[0] - hx, b[1] - hy),
                        (b[0] + hx, b[1] + hy),
                    ],
                );
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wiggle(n: usize) -> Vec<[f64; 2]> {
        (0..n)
            .map(|i| [i as f64 * 2.0, if i % 2 == 0 { 0.0 } else { 3.0 }])
            .collect()
    }

    fn roughness(pts: &[[f64; 2]]) -> f64 {
        pts.windows(3)
            .map(|w| (w[0][1] - 2.0 * w[1][1] + w[2][1]).abs())
            .sum::<f64>()
            / pts.len().max(1) as f64
    }

    #[test]
    fn smoothing_takes_the_tremble_away_and_keeps_the_ends() {
        let raw = wiggle(40);
        let none = smoothed(&raw, 0.0);
        let full = smoothed(&raw, 100.0);
        assert_eq!(full.first(), raw.first());
        assert_eq!(full.last(), raw.last());
        assert!(
            roughness(&full) < roughness(&none) / 5.0,
            "{} {}",
            roughness(&full),
            roughness(&none)
        );
    }

    #[test]
    fn a_straight_stroke_keeps_two_points() {
        let line: Vec<[f64; 2]> = (0..20).map(|i| [i as f64, i as f64]).collect();
        assert_eq!(smoothed(&line, 30.0), [[0.0, 0.0], [19.0, 19.0]]);
    }
}
