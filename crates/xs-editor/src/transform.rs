//! Several objects changed as one (EDT-080, EDT-083): stretched to a new box, turned
//! around a point, or given the same width or height. Pure geometry on stored objects;
//! each function returns the `set` that `objects::update` applies, so the result is
//! validated and stored in canonical form like any other change.
//!
//! `app/src/editor/transform.ts` is its copy for the canvas preview; the tests of both
//! check the same cases.

use serde_json::{json, Map, Value};
use xs_core::document::Object;

/// An unturned rectangle in canvas pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

impl Rect {
    pub fn from_value(v: &Value) -> Option<Rect> {
        if let Some(a) = v.as_array() {
            let n = |i: usize| a.get(i).and_then(Value::as_f64);
            return Some(Rect {
                x: n(0)?,
                y: n(1)?,
                w: n(2)?,
                h: n(3)?,
            });
        }
        let n = |k: &str| v.get(k).and_then(Value::as_f64);
        Some(Rect {
            x: n("x")?,
            y: n("y")?,
            w: n("w")?,
            h: n("h")?,
        })
    }

    pub fn center(&self) -> [f64; 2] {
        [self.x + self.w / 2.0, self.y + self.h / 2.0]
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    /// What is in `from` stretched to fill `to`; with `strokes`, line widths and text
    /// sizes grow and shrink too.
    Scale { from: Rect, to: Rect, strokes: bool },
    /// Turned `degrees` clockwise around `center`.
    Rotate { center: [f64; 2], degrees: f64 },
    /// The width (or the height) set to `size`, keeping the left (or top) side. A
    /// picture keeps its proportions; an arrow does not change.
    Equal { width: bool, size: f64 },
}

/// Width and height of a text laid out with these properties.
pub type Measure<'a> = &'a dyn Fn(&Map<String, Value>) -> Option<(f64, f64)>;

/// A box turned `rotation` degrees clockwise around its center.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Boxed {
    pub rect: Rect,
    pub rotation: f64,
}

fn xy(v: Option<&Value>) -> Option<[f64; 2]> {
    let a = v?.as_array()?;
    Some([a.first()?.as_f64()?, a.get(1)?.as_f64()?])
}

fn rotation_of(p: &Map<String, Value>) -> f64 {
    p.get("rotation").and_then(Value::as_f64).unwrap_or(0.0)
}

/// The box of an object drawn in a rectangle; arrows have none.
pub fn box_of(o: &Object, measure: Measure) -> Option<Boxed> {
    let p = &o.props;
    match o.kind.as_str() {
        "shape" | "blur" | "image" | "step" | "callout" | "highlight" => Some(Boxed {
            rect: Rect::from_value(p.get("rect")?)?,
            rotation: rotation_of(p),
        }),
        "text" => {
            let at = xy(p.get("at"))?;
            let (w, h) = measure(p)?;
            let w = p.get("width").and_then(Value::as_f64).unwrap_or(w);
            Some(Boxed {
                rect: Rect {
                    x: at[0],
                    y: at[1],
                    w,
                    h,
                },
                rotation: rotation_of(p),
            })
        }
        _ => None,
    }
}

fn turn(p: [f64; 2], c: [f64; 2], degrees: f64) -> [f64; 2] {
    let (s, k) = degrees.to_radians().sin_cos();
    let (dx, dy) = (p[0] - c[0], p[1] - c[1]);
    [c[0] + dx * k - dy * s, c[1] + dx * s + dy * k]
}

fn corners(b: &Boxed) -> [[f64; 2]; 4] {
    let r = b.rect;
    let c = r.center();
    [
        [r.x, r.y],
        [r.x + r.w, r.y],
        [r.x + r.w, r.y + r.h],
        [r.x, r.y + r.h],
    ]
    .map(|p| turn(p, c, b.rotation))
}

/// Canvas points an object reaches: an arrow's ends or its box's corners.
pub fn extent(o: &Object, measure: Measure) -> Vec<[f64; 2]> {
    if o.kind == "pen" {
        return o
            .props
            .get("points")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(|p| xy(Some(p))).collect())
            .unwrap_or_default();
    }
    if o.kind == "arrow" || o.kind == "line" {
        return [xy(o.props.get("from")), xy(o.props.get("to"))]
            .into_iter()
            .flatten()
            .collect();
    }
    box_of(o, measure)
        .map(|b| corners(&b).to_vec())
        .unwrap_or_default()
}

/// The unturned box around all of `objects` (EDT-079).
pub fn bounds<'a>(objects: impl IntoIterator<Item = &'a Object>, measure: Measure) -> Option<Rect> {
    let pts: Vec<[f64; 2]> = objects.into_iter().flat_map(|o| extent(o, measure)).collect();
    if pts.is_empty() {
        return None;
    }
    let x0 = pts.iter().map(|p| p[0]).fold(f64::INFINITY, f64::min);
    let y0 = pts.iter().map(|p| p[1]).fold(f64::INFINITY, f64::min);
    let x1 = pts.iter().map(|p| p[0]).fold(f64::NEG_INFINITY, f64::max);
    let y1 = pts.iter().map(|p| p[1]).fold(f64::NEG_INFINITY, f64::max);
    Some(Rect {
        x: x0,
        y: y0,
        w: x1 - x0,
        h: y1 - y0,
    })
}

/// Degrees in (-180, 180].
pub fn normalized(degrees: f64) -> f64 {
    let mut r = degrees % 360.0;
    if r > 180.0 {
        r -= 360.0;
    } else if r <= -180.0 {
        r += 360.0;
    }
    r
}

/// How much each axis grows: a side of zero (a level arrow) does not grow.
fn factors(from: &Rect, to: &Rect) -> (f64, f64) {
    let f = |a: f64, b: f64| if a.abs() < 1e-9 { 1.0 } else { (b / a).abs() };
    (f(from.w, to.w), f(from.h, to.h))
}

fn mapped(p: [f64; 2], from: &Rect, to: &Rect) -> [f64; 2] {
    let (sx, sy) = factors(from, to);
    [to.x + (p[0] - from.x) * sx, to.y + (p[1] - from.y) * sy]
}

/// A thickness or size grown by `k`, to a tenth, kept in its range.
fn grown(v: Option<f64>, k: f64, min: f64, max: f64) -> Option<f64> {
    v.map(|v| ((v * k * 10.0).round() / 10.0).clamp(min, max))
}

fn style_num(o: &Object, key: &str) -> Option<f64> {
    o.props.get("style")?.get(key)?.as_f64()
}

fn rect_value(r: Rect, whole: bool) -> Value {
    if whole {
        // Whole pixels keep a picture sharp (EDT-078).
        json!({"x": r.x.round(), "y": r.y.round(), "w": r.w.round().max(1.0), "h": r.h.round().max(1.0)})
    } else {
        json!({"x": r.x, "y": r.y, "w": r.w.max(1.0), "h": r.h.max(1.0)})
    }
}

/// A rectangle of `w` × `h` centered on `c`.
fn around(c: [f64; 2], w: f64, h: f64) -> Rect {
    Rect {
        x: c[0] - w / 2.0,
        y: c[1] - h / 2.0,
        w,
        h,
    }
}

/// The `set` that applies `op` to `o`; empty when it does not change.
pub fn transformed(o: &Object, op: &Op, measure: Measure) -> Map<String, Value> {
    let mut set = Map::new();
    // A stroke's points move like a polyline's (EDT-037).
    if o.kind == "pen" {
        let points: Vec<[f64; 2]> = o
            .props
            .get("points")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(|p| xy(Some(p))).collect())
            .unwrap_or_default();
        match *op {
            Op::Scale {
                from: a,
                to: b,
                strokes,
            } => {
                let moved: Vec<[f64; 2]> = points.iter().map(|p| mapped(*p, &a, &b)).collect();
                set.insert("points".into(), json!(moved));
                if strokes {
                    let (sx, sy) = factors(&a, &b);
                    if let Some(w) = grown(style_num(o, "width"), (sx * sy).sqrt(), 1.0, 100.0) {
                        set.insert("style".into(), json!({ "width": w }));
                    }
                }
            }
            Op::Rotate { center, degrees } => {
                let moved: Vec<[f64; 2]> = points.iter().map(|p| turn(*p, center, degrees)).collect();
                set.insert("points".into(), json!(moved));
            }
            Op::Equal { .. } => {}
        }
        return set;
    }
    if o.kind == "arrow" || o.kind == "line" {
        let (Some(from), Some(to)) = (xy(o.props.get("from")), xy(o.props.get("to"))) else {
            return set;
        };
        match *op {
            Op::Scale {
                from: a,
                to: b,
                strokes,
            } => {
                set.insert("from".into(), json!(mapped(from, &a, &b)));
                set.insert("to".into(), json!(mapped(to, &a, &b)));
                if strokes {
                    let (sx, sy) = factors(&a, &b);
                    if let Some(w) = grown(style_num(o, "width"), (sx * sy).sqrt(), 1.0, 100.0) {
                        set.insert("style".into(), json!({ "width": w }));
                    }
                }
            }
            Op::Rotate { center, degrees } => {
                set.insert("from".into(), json!(turn(from, center, degrees)));
                set.insert("to".into(), json!(turn(to, center, degrees)));
            }
            Op::Equal { .. } => {}
        }
        return set;
    }
    let Some(b) = box_of(o, measure) else {
        return set;
    };
    let (r, rot) = (b.rect, b.rotation);
    let (sin, cos) = rot.to_radians().sin_cos();
    let text = o.kind == "text";
    let image = o.kind == "image";
    // The new size and where the center goes; a text's height follows its lines.
    let (c, w, h, rotation) = match *op {
        Op::Scale {
            from: a,
            to: bx,
            strokes,
        } => {
            let (sx, sy) = factors(&a, &bx);
            // Each side stretched along the object's own turned axes.
            let w = r.w * (sx * cos).hypot(sy * sin);
            let h = r.h * (sx * sin).hypot(sy * cos);
            if strokes {
                let k = (sx * sy).sqrt();
                match o.kind.as_str() {
                    "shape" => {
                        if let Some(v) = grown(style_num(o, "outlineWidth"), k, 0.0, 100.0) {
                            set.insert("style".into(), json!({ "outlineWidth": v }));
                        }
                    }
                    "text" => {
                        if let Some(v) = grown(style_num(o, "size"), k, 1.0, 1000.0) {
                            set.insert("style".into(), json!({ "size": v }));
                        }
                    }
                    "callout" => {
                        let mut st = Map::new();
                        for (key, lo, hi) in [
                            ("outlineWidth", 0.0, 20.0),
                            ("size", 1.0, 1000.0),
                            ("tailWidth", 2.0, 200.0),
                        ] {
                            if let Some(v) = grown(style_num(o, key), k, lo, hi) {
                                st.insert(key.into(), json!(v));
                            }
                        }
                        set.insert("style".into(), Value::Object(st));
                    }
                    _ => {}
                }
            }
            (mapped(r.center(), &a, &bx), w, h, rot)
        }
        Op::Rotate { center, degrees } => (
            turn(r.center(), center, degrees),
            r.w,
            r.h,
            normalized(rot + degrees),
        ),
        Op::Equal { width, size } => {
            let size = size.max(1.0);
            let (w, h) = match (width, image) {
                (true, true) => (size, r.h * size / r.w.max(1e-9)),
                (true, false) => (size, r.h),
                (false, true) => (r.w * size / r.h.max(1e-9), size),
                (false, false) if text => return set,
                (false, false) => (r.w, size),
            };
            // The left and top sides stay where they are, in the object's own frame.
            let (lx, ly) = ((w - r.w) / 2.0, (h - r.h) / 2.0);
            let c0 = r.center();
            (
                [c0[0] + lx * cos - ly * sin, c0[1] + lx * sin + ly * cos],
                w,
                h,
                rot,
            )
        }
    };
    if text {
        let mut probe = o.props.clone();
        probe.insert("width".into(), json!(w));
        if let Some(Value::Object(changes)) = set.get("style") {
            let mut style = probe
                .get("style")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default();
            style.extend(changes.clone());
            probe.insert("style".into(), Value::Object(style));
        }
        let h = measure(&probe).map(|m| m.1).unwrap_or(h);
        let at = around(c, w, h);
        set.insert("at".into(), json!([at.x, at.y]));
        set.insert("width".into(), json!(w.max(1.0)));
    } else {
        set.insert("rect".into(), rect_value(around(c, w, h), image));
    }
    if rotation != rot {
        set.insert("rotation".into(), json!(rotation));
    }
    // A callout's tails point at the same things (EDT-028).
    if let Some(tails) = o.props.get("tails").and_then(Value::as_array) {
        let moved: Vec<Value> = tails
            .iter()
            .filter_map(|t| xy(Some(t)))
            .map(|t| match *op {
                Op::Scale { from, to, .. } => json!(mapped(t, &from, &to)),
                Op::Rotate { center, degrees } => json!(turn(t, center, degrees)),
                Op::Equal { .. } => json!(t),
            })
            .collect();
        set.insert("tails".into(), Value::Array(moved));
    }
    set
}

/// The width or height the largest of `objects` has: what "Equal" gives the others.
pub fn largest(objects: &[&Object], width: bool, measure: Measure) -> Option<f64> {
    objects
        .iter()
        .filter(|o| width || o.kind != "text")
        .filter_map(|o| box_of(o, measure))
        .map(|b| if width { b.rect.w } else { b.rect.h })
        .fold(None, |m: Option<f64>, v| Some(m.map_or(v, |m| m.max(v))))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj(kind: &str, props: Value) -> Object {
        Object {
            id: "o".into(),
            kind: kind.into(),
            props: props.as_object().cloned().unwrap(),
        }
    }
    fn no_text(_: &Map<String, Value>) -> Option<(f64, f64)> {
        None
    }
    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-6
    }

    #[test]
    fn growing_the_box_half_again_grows_each_object_and_keeps_their_places() {
        let arrow = obj(
            "arrow",
            json!({"from": [0, 0], "to": [100, 0], "style": {"width": 4}}),
        );
        let image = obj(
            "image",
            json!({"rect": {"x": 100, "y": 50, "w": 100, "h": 50}, "rotation": 0}),
        );
        let from = bounds([&arrow, &image], &no_text).unwrap();
        assert_eq!(
            from,
            Rect {
                x: 0.0,
                y: 0.0,
                w: 200.0,
                h: 100.0
            }
        );
        let to = Rect {
            x: 0.0,
            y: 0.0,
            w: 300.0,
            h: 150.0,
        };
        let op = Op::Scale {
            from,
            to,
            strokes: true,
        };
        let a = transformed(&arrow, &op, &no_text);
        assert_eq!(a["to"], json!([150.0, 0.0]));
        assert_eq!(a["style"]["width"], json!(6.0));
        let i = transformed(&image, &op, &no_text);
        assert_eq!(i["rect"], json!({"x": 150.0, "y": 75.0, "w": 150.0, "h": 75.0}));
        // Without "Scale line widths and text" the arrow keeps its width.
        let op = Op::Scale {
            from,
            to,
            strokes: false,
        };
        assert!(transformed(&arrow, &op, &no_text).get("style").is_none());
    }

    #[test]
    fn a_quarter_turn_goes_around_the_common_center() {
        let shapes: Vec<Object> = [0.0, 100.0, 200.0]
            .iter()
            .map(|x| {
                obj(
                    "shape",
                    json!({"rect": {"x": x, "y": 0, "w": 20, "h": 10}, "rotation": 0}),
                )
            })
            .collect();
        let all = bounds(shapes.iter(), &no_text).unwrap();
        let center = all.center();
        assert_eq!(center, [110.0, 5.0]);
        let op = Op::Rotate {
            center,
            degrees: 90.0,
        };
        let first = transformed(&shapes[0], &op, &no_text);
        // Its center (10, 5) turns to (110, -95), around (110, 5).
        let r = Rect::from_value(&first["rect"]).unwrap();
        assert!(
            close(r.center()[0], 110.0) && close(r.center()[1], -95.0),
            "{r:?}"
        );
        assert_eq!(first["rotation"], json!(90.0));
        let middle = transformed(&shapes[1], &op, &no_text);
        let r = Rect::from_value(&middle["rect"]).unwrap();
        assert!(close(r.center()[0], 110.0) && close(r.center()[1], 5.0));
    }

    #[test]
    fn equal_width_keeps_the_left_side_and_a_picture_its_proportions() {
        let shape = obj(
            "shape",
            json!({"rect": {"x": 10, "y": 10, "w": 20, "h": 10}, "rotation": 0}),
        );
        let image = obj(
            "image",
            json!({"rect": {"x": 50, "y": 0, "w": 40, "h": 20}, "rotation": 0}),
        );
        let arrow = obj("arrow", json!({"from": [0, 0], "to": [500, 0]}));
        let size = largest(&[&shape, &image, &arrow], true, &no_text).unwrap();
        assert_eq!(size, 40.0, "arrows have no width to match");
        let op = Op::Equal { width: true, size };
        assert_eq!(
            transformed(&shape, &op, &no_text)["rect"],
            json!({"x": 10.0, "y": 10.0, "w": 40.0, "h": 10.0})
        );
        assert!(transformed(&arrow, &op, &no_text).is_empty());
        let op = Op::Equal {
            width: false,
            size: 40.0,
        };
        assert_eq!(
            transformed(&image, &op, &no_text)["rect"],
            json!({"x": 50.0, "y": 0.0, "w": 80.0, "h": 40.0})
        );
    }

    #[test]
    fn angles_stay_within_a_half_turn() {
        assert_eq!(normalized(270.0), -90.0);
        assert_eq!(normalized(-190.0), 170.0);
        assert_eq!(normalized(180.0), 180.0);
    }
}
