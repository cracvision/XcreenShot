//! "Only the text" (SMT-021): a highlight, a blur or a spotlight whose style asks for
//! the text only covers the text inside its area, one band per line. The bands are
//! kept in the object (`bands`, relative to its rect's top-left corner, so they follow
//! a drag); the core finds them again when the object moves or changes size
//! (`bandsFor` is the rect they were found for). Without bands (no text found yet, or
//! none), the object covers its whole area.

use serde_json::{json, Map, Value};

use crate::shapes::{rect, Rect};

/// The style asks for the text only.
pub fn wanted(props: &Map<String, Value>) -> bool {
    props
        .get("style")
        .and_then(|s| s.get("textOnly"))
        .and_then(Value::as_bool)
        == Some(true)
}

/// The bands to draw, in canvas pixels; `None` to draw the whole area.
pub fn bands(props: &Map<String, Value>) -> Option<Vec<Rect>> {
    if !wanted(props) {
        return None;
    }
    let r = rect(props.get("rect")).ok()?;
    let out: Vec<Rect> = props
        .get("bands")?
        .as_array()?
        .iter()
        .filter_map(|b| {
            let a = b.as_array()?;
            let n = |i: usize| a.get(i).and_then(Value::as_f64);
            Some(Rect {
                x: r.x + n(0)?,
                y: r.y + n(1)?,
                w: n(2)?,
                h: n(3)?,
            })
        })
        .filter(|b| b.w > 0.0 && b.h > 0.0)
        .collect();
    (!out.is_empty()).then_some(out)
}

/// The bands need finding again: the text only is asked and the object moved,
/// changed size or never had them.
pub fn stale(props: &Map<String, Value>) -> bool {
    if !wanted(props) {
        return false;
    }
    let Ok(r) = rect(props.get("rect")) else {
        return false;
    };
    let found_for: Option<[f64; 4]> = props
        .get("bandsFor")
        .and_then(|v| serde_json::from_value(v.clone()).ok());
    match found_for {
        Some(f) => [r.x, r.y, r.w, r.h]
            .iter()
            .zip(f)
            .any(|(a, b)| (a - b).abs() > 0.25),
        None => true,
    }
}

/// The text lines of a reading, each as the rectangles of its words in canvas
/// pixels, give the bands of an object at `r`: per line, the words whose middle is
/// inside, joined, with a little room around them (`pad` pixels).
pub fn find(r: Rect, lines: &[Vec<[f64; 4]>], pad: f64) -> Vec<[f64; 4]> {
    let inside = |w: &[f64; 4]| {
        let (cx, cy) = (w[0] + w[2] / 2.0, w[1] + w[3] / 2.0);
        cx >= r.x && cx <= r.x + r.w && cy >= r.y && cy <= r.y + r.h
    };
    lines
        .iter()
        .filter_map(|words| {
            let picked: Vec<&[f64; 4]> = words.iter().filter(|w| inside(w)).collect();
            if picked.is_empty() {
                return None;
            }
            let x0 = picked.iter().map(|w| w[0]).fold(f64::MAX, f64::min) - pad;
            let y0 = picked.iter().map(|w| w[1]).fold(f64::MAX, f64::min) - pad;
            let x1 = picked.iter().map(|w| w[0] + w[2]).fold(f64::MIN, f64::max) + pad;
            let y1 = picked.iter().map(|w| w[1] + w[3]).fold(f64::MIN, f64::max) + pad;
            // Inside the area, relative to its corner.
            let (x0, y0) = (x0.max(r.x), y0.max(r.y));
            let (x1, y1) = (x1.min(r.x + r.w), y1.min(r.y + r.h));
            let round = |v: f64| (v * 10.0).round() / 10.0;
            (x1 > x0 && y1 > y0).then(|| [round(x0 - r.x), round(y0 - r.y), round(x1 - x0), round(y1 - y0)])
        })
        .collect()
}

/// Keeps the bands found for the object's rect.
pub fn set(props: &mut Map<String, Value>, bands: &[[f64; 4]]) {
    if let Ok(r) = rect(props.get("rect")) {
        props.insert("bands".into(), json!(bands));
        props.insert("bandsFor".into(), json!([r.x, r.y, r.w, r.h]));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn props(v: Value) -> Map<String, Value> {
        v.as_object().unwrap().clone()
    }

    #[test]
    fn one_band_per_line_inside_the_area() {
        let lines = vec![
            vec![[10.0, 10.0, 40.0, 12.0], [55.0, 10.0, 30.0, 12.0]],
            vec![[10.0, 30.0, 90.0, 12.0]],
            vec![[10.0, 46.0, 20.0, 12.0]],
            // Outside.
            vec![[300.0, 10.0, 20.0, 12.0]],
        ];
        let r = Rect {
            x: 5.0,
            y: 5.0,
            w: 120.0,
            h: 50.0,
        };
        let b = find(r, &lines, 1.0);
        // Three lines of different length: three bands, each as wide as its line; the
        // third, half inside, is cut at the area's edge.
        assert_eq!(
            b,
            vec![
                [4.0, 4.0, 77.0, 14.0],
                [4.0, 24.0, 92.0, 14.0],
                [4.0, 40.0, 22.0, 10.0]
            ]
        );
    }

    #[test]
    fn bands_follow_the_object_and_go_stale() {
        let mut p =
            props(json!({"rect": {"x": 100, "y": 50, "w": 80, "h": 40}, "style": {"textOnly": true}}));
        assert!(stale(&p));
        assert_eq!(bands(&p), None, "none found yet: the whole area");
        set(&mut p, &[[2.0, 3.0, 50.0, 10.0]]);
        assert!(!stale(&p));
        let b = bands(&p).unwrap();
        assert_eq!((b[0].x, b[0].y, b[0].w), (102.0, 53.0, 50.0));
        // Moved: the bands go with it until found again.
        p.insert("rect".into(), json!({"x": 110, "y": 50, "w": 80, "h": 40}));
        assert!(stale(&p));
        assert_eq!(bands(&p).unwrap()[0].x, 112.0);
        // Not asked: the whole area, and nothing to find.
        p.insert("style".into(), json!({"textOnly": false}));
        assert!(!stale(&p));
        assert_eq!(bands(&p), None);
    }
}
