//! Fills that are a color or a gradient (EDT-020). In a document a fill is written as
//! before, `"#RRGGBB"`, or as `{"type": "linear" | "radial", "angle", "stops"}`.

use serde::{Deserialize, Serialize};
use tiny_skia::{GradientStop, LinearGradient, Paint, Point, RadialGradient, SpreadMode, Transform};

use crate::color::Rgba;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Fill {
    Color(String),
    Gradient(Gradient),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GradientKind {
    Linear,
    Radial,
}

/// A gradient over an object's box (EDT-020). `angle` turns clockwise from the right:
/// at 90° the first stop is at the top.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Gradient {
    #[serde(rename = "type")]
    pub kind: GradientKind,
    #[serde(default)]
    pub angle: f64,
    pub stops: Vec<Stop>,
}

/// A color stop: its color (with its opacity) and where it is, 0–100 %.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Stop {
    pub color: String,
    pub pos: f64,
}

impl From<&str> for Fill {
    fn from(c: &str) -> Fill {
        Fill::Color(c.to_string())
    }
}

impl Fill {
    /// Checks the values: a color, or 2 to 8 stops at 0–100 %.
    pub fn check(&self, name: &str) -> Result<(), String> {
        match self {
            Fill::Color(c) => {
                Rgba::parse(c).ok_or_else(|| format!("{name} must be a color like #RRGGBB or a gradient"))?;
            }
            Fill::Gradient(g) => {
                if !(2..=8).contains(&g.stops.len()) {
                    return Err(format!("{name}: a gradient has 2 to 8 stops"));
                }
                if !(0.0..=360.0).contains(&g.angle) {
                    return Err(format!("{name}: angle must be between 0 and 359"));
                }
                for s in &g.stops {
                    Rgba::parse(&s.color).ok_or_else(|| format!("{name}: {} is not a color", s.color))?;
                    if !(0.0..=100.0).contains(&s.pos) {
                        return Err(format!("{name}: stop positions go from 0 to 100"));
                    }
                }
            }
        }
        Ok(())
    }

    /// The color, or a gradient's first stop: what stands for it where only one fits.
    pub fn main_color(&self) -> Option<Rgba> {
        match self {
            Fill::Color(c) => Rgba::parse(c),
            Fill::Gradient(g) => g.sorted().first().and_then(|s| Rgba::parse(&s.color)),
        }
    }

    /// A paint for this fill over the box `[x, y, w, h]` (in the coordinates the path
    /// is drawn in, before the object's turn).
    pub fn paint(&self, rect: [f64; 4]) -> Option<Paint<'static>> {
        let mut p = Paint {
            anti_alias: true,
            ..Paint::default()
        };
        match self {
            Fill::Color(c) => {
                let c = Rgba::parse(c)?;
                p.set_color_rgba8(c.r, c.g, c.b, c.a);
            }
            Fill::Gradient(g) => {
                let stops: Vec<GradientStop> = g
                    .sorted()
                    .iter()
                    .filter_map(|s| {
                        let c = Rgba::parse(&s.color)?;
                        Some(GradientStop::new(
                            (s.pos / 100.0) as f32,
                            tiny_skia::Color::from_rgba8(c.r, c.g, c.b, c.a),
                        ))
                    })
                    .collect();
                let (start, end, radius) = g.geometry(rect);
                let shader = match g.kind {
                    GradientKind::Linear => LinearGradient::new(
                        Point::from_xy(start[0] as f32, start[1] as f32),
                        Point::from_xy(end[0] as f32, end[1] as f32),
                        stops,
                        SpreadMode::Pad,
                        Transform::identity(),
                    ),
                    GradientKind::Radial => RadialGradient::new(
                        Point::from_xy(start[0] as f32, start[1] as f32),
                        0.0,
                        Point::from_xy(start[0] as f32, start[1] as f32),
                        radius.max(0.5) as f32,
                        stops,
                        SpreadMode::Pad,
                        Transform::identity(),
                    ),
                }?;
                p.shader = shader;
            }
        }
        Some(p)
    }

    /// The fill's color at `(x, y)` of the box `rect`: for pixels (the Fill tool).
    pub fn color_at(&self, x: f64, y: f64, rect: [f64; 4]) -> Option<Rgba> {
        match self {
            Fill::Color(c) => Rgba::parse(c),
            Fill::Gradient(g) => {
                let (start, end, radius) = g.geometry(rect);
                let t = match g.kind {
                    GradientKind::Linear => {
                        let (dx, dy) = (end[0] - start[0], end[1] - start[1]);
                        let len2 = (dx * dx + dy * dy).max(1e-9);
                        ((x - start[0]) * dx + (y - start[1]) * dy) / len2
                    }
                    GradientKind::Radial => ((x - start[0]).hypot(y - start[1])) / radius.max(0.5),
                };
                Some(g.at(t.clamp(0.0, 1.0) * 100.0))
            }
        }
    }
}

impl Gradient {
    fn sorted(&self) -> Vec<Stop> {
        let mut s = self.stops.clone();
        s.sort_by(|a, b| a.pos.total_cmp(&b.pos));
        s
    }

    /// Start and end of the gradient's line over `rect` (linear), or its center and
    /// radius (radial): it covers the whole box.
    fn geometry(&self, r: [f64; 4]) -> ([f64; 2], [f64; 2], f64) {
        let c = [r[0] + r[2] / 2.0, r[1] + r[3] / 2.0];
        match self.kind {
            GradientKind::Linear => {
                let a = self.angle.to_radians();
                let (dx, dy) = (a.cos(), a.sin());
                let half = (r[2] * dx.abs() + r[3] * dy.abs()) / 2.0;
                (
                    [c[0] - dx * half, c[1] - dy * half],
                    [c[0] + dx * half, c[1] + dy * half],
                    0.0,
                )
            }
            GradientKind::Radial => (c, c, r[2].max(r[3]) / 2.0),
        }
    }

    /// The color at `pos` (0–100), between the stops around it.
    fn at(&self, pos: f64) -> Rgba {
        let stops = self.sorted();
        let color = |s: &Stop| Rgba::parse(&s.color).unwrap_or(Rgba::WHITE);
        let first = stops.first().map(color).unwrap_or(Rgba::WHITE);
        let mut prev: Option<&Stop> = None;
        for s in &stops {
            if pos <= s.pos {
                let Some(p) = prev else {
                    return color(s);
                };
                let k = if s.pos > p.pos {
                    (pos - p.pos) / (s.pos - p.pos)
                } else {
                    1.0
                };
                let (a, b) = (color(p), color(s));
                let mix = |x: u8, y: u8| (x as f64 + (y as f64 - x as f64) * k).round() as u8;
                return Rgba {
                    r: mix(a.r, b.r),
                    g: mix(a.g, b.g),
                    b: mix(a.b, b.b),
                    a: mix(a.a, b.a),
                };
            }
            prev = Some(s);
        }
        prev.map(color).unwrap_or(first)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn red_to_blue(angle: f64) -> Fill {
        Fill::Gradient(Gradient {
            kind: GradientKind::Linear,
            angle,
            stops: vec![
                Stop {
                    color: "#FF0000".into(),
                    pos: 0.0,
                },
                Stop {
                    color: "#0000FF".into(),
                    pos: 100.0,
                },
            ],
        })
    }

    #[test]
    fn a_90_degree_gradient_goes_from_top_to_bottom() {
        let g = red_to_blue(90.0);
        let r = [0.0, 0.0, 100.0, 60.0];
        let top = g.color_at(50.0, 0.0, r).unwrap();
        let bottom = g.color_at(50.0, 60.0, r).unwrap();
        assert_eq!((top.r, top.b), (255, 0));
        assert_eq!((bottom.r, bottom.b), (0, 255));
        // Written as before when it is a color; a gradient reads back.
        assert_eq!(serde_json::to_value(Fill::from("#00FF00")).unwrap(), "#00FF00");
        let v = serde_json::to_value(&g).unwrap();
        assert_eq!(serde_json::from_value::<Fill>(v).unwrap(), g);
    }

    #[test]
    fn radial_gradients_start_at_the_center_and_values_are_checked() {
        let g = Fill::Gradient(Gradient {
            kind: GradientKind::Radial,
            angle: 0.0,
            stops: vec![
                Stop {
                    color: "#FFFFFF".into(),
                    pos: 0.0,
                },
                Stop {
                    color: "#00FF00".into(),
                    pos: 50.0,
                },
                Stop {
                    color: "#000000".into(),
                    pos: 100.0,
                },
            ],
        });
        let r = [0.0, 0.0, 100.0, 100.0];
        assert_eq!(g.color_at(50.0, 50.0, r).unwrap(), Rgba::WHITE);
        assert_eq!(g.color_at(50.0, 25.0, r).unwrap().g, 255);
        assert!(g.check("fill").is_ok());
        let one = Fill::Gradient(Gradient {
            kind: GradientKind::Linear,
            angle: 0.0,
            stops: vec![Stop {
                color: "#FFFFFF".into(),
                pos: 0.0,
            }],
        });
        assert!(one.check("fill").is_err());
    }
}
