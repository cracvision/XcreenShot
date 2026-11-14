//! The redactions auto redact manages (SMT-010 to SMT-013): normal Blur objects (EDT-040)
//! that also carry `autoRedact: {category, source}`, where `source` is the rectangle of
//! the data they cover, in canvas pixels. Never the text (SMT-017). An analysis replaces
//! the managed redactions in one undo step, keeping those that still cover something
//! wanted (moved or restyled by hand, they stay as they are) so that nothing is doubled.

use image::RgbaImage;
use serde_json::{json, Map, Value};
use xs_core::document::Object;
use xs_core::{Context, Core};

use crate::{changed, editor, objects};

/// The property that marks a managed redaction.
pub const MARK: &str = "autoRedact";

/// A redaction an analysis wants: its category, the data's rectangle and the object's
/// rectangle (the data plus the margin), in canvas pixels.
#[derive(Debug, Clone, PartialEq)]
pub struct Wanted {
    pub category: String,
    pub source: [f64; 4],
    pub rect: [f64; 4],
}

/// A managed redaction in a document.
#[derive(Debug, Clone, PartialEq)]
pub struct Managed {
    pub object_id: String,
    pub category: String,
    pub source: [f64; 4],
    /// The object's own rectangle, as it is now.
    pub rect: [f64; 4],
}

fn four(v: Option<&Value>) -> Option<[f64; 4]> {
    let a = v?.as_array()?;
    Some([
        a.first()?.as_f64()?,
        a.get(1)?.as_f64()?,
        a.get(2)?.as_f64()?,
        a.get(3)?.as_f64()?,
    ])
}

fn rect_of(o: &Object) -> [f64; 4] {
    let r = o.props.get("rect");
    let n = |k: &str| r.and_then(|r| r.get(k)).and_then(Value::as_f64).unwrap_or(0.0);
    [n("x"), n("y"), n("w"), n("h")]
}

fn managed_of(o: &Object) -> Option<Managed> {
    let m = o.props.get(MARK)?;
    Some(Managed {
        object_id: o.id.clone(),
        category: m.get("category")?.as_str()?.to_string(),
        source: four(m.get("source")).unwrap_or_else(|| rect_of(o)),
        rect: rect_of(o),
    })
}

fn same(a: [f64; 4], b: [f64; 4]) -> bool {
    a.iter().zip(b).all(|(x, y)| (x - y).abs() < 0.5)
}

/// The final image of a document without its managed redactions, and where the
/// canvas's corner is in it: what an analysis reads.
pub fn image_without(core: &Core, id: &str) -> Result<(RgbaImage, [i64; 2]), String> {
    let mut snap = editor(core).store.snapshot(id)?;
    snap.document.objects.retain(|o| !o.props.contains_key(MARK));
    Ok(snap.render_with_origin())
}

/// The managed redactions of a document, bottom first.
pub fn managed(core: &Core, id: &str) -> Result<Vec<Managed>, String> {
    editor(core)
        .store
        .with(id, |d| d.document.objects.iter().filter_map(managed_of).collect())
        .ok_or_else(|| "document closed".to_string())
}

/// The blur style of a managed redaction of `category`: faces as ellipses (SMT-013).
fn style_for(category: &str, style: &Map<String, Value>) -> Map<String, Value> {
    let mut s = style.clone();
    s.insert(
        "shapeType".into(),
        json!(if category == "face" {
            "ellipse"
        } else {
            "rectangle"
        }),
    );
    s
}

fn new_object(id: String, w: &Wanted, style: &Map<String, Value>) -> Result<Object, String> {
    let [x, y, width, height] = w.rect;
    let mut props = objects::checked(
        "blur",
        json!({
            "rect": {"x": x, "y": y, "w": width.max(1.0), "h": height.max(1.0)},
            "rotation": 0,
            "style": style_for(&w.category, style),
        })
        .as_object()
        .expect("object"),
    )?;
    props.insert(MARK.into(), json!({"category": w.category, "source": w.source}));
    Ok(Object {
        id,
        kind: "blur".into(),
        props,
    })
}

/// What changed in one reconciliation.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Outcome {
    pub added: usize,
    pub removed: usize,
    pub restyled: usize,
}

/// Leaves the managed redactions of a document as `wanted` says, in one undo step
/// (`history.auto_redact`), and none when nothing changes. Those that already cover a
/// wanted thing stay (with their own changes); with `restyle`, they all take that
/// style. New ones take `style` and go on top.
pub fn reconcile(
    core: &Core,
    ctx: &Context,
    id: &str,
    wanted: &[Wanted],
    style: &Map<String, Value>,
    restyle: bool,
) -> Result<Outcome, String> {
    let out = editor(core)
        .store
        .mutate(id, "history.auto_redact", |doc| {
            let mut outcome = Outcome::default();
            let mut matched = vec![false; wanted.len()];
            let mut kept = Vec::with_capacity(doc.objects.len());
            for mut o in std::mem::take(&mut doc.objects) {
                let Some(m) = managed_of(&o) else {
                    kept.push(o);
                    continue;
                };
                let hit = wanted
                    .iter()
                    .enumerate()
                    .find(|(i, w)| !matched[*i] && w.category == m.category && same(w.source, m.source));
                let Some((i, _)) = hit else {
                    outcome.removed += 1;
                    continue;
                };
                matched[i] = true;
                if restyle {
                    let next = Value::Object(style_for(&m.category, style));
                    if o.props.get("style") != Some(&next) {
                        let mut props = o.props.clone();
                        props.insert("style".into(), next);
                        let mark = props.remove(MARK);
                        let mut checked = objects::checked("blur", &props)?;
                        if let Some(mark) = mark {
                            checked.insert(MARK.into(), mark);
                        }
                        o.props = checked;
                        outcome.restyled += 1;
                    }
                }
                kept.push(o);
            }
            doc.objects = kept;
            for (w, _) in wanted.iter().zip(&matched).filter(|(_, m)| !**m) {
                let oid = doc.next_object_id();
                doc.objects.push(new_object(oid, w, style)?);
                outcome.added += 1;
            }
            if outcome == Outcome::default() {
                // Nothing to undo: the store keeps the document as it was.
                return Err(String::new());
            }
            Ok(outcome)
        })
        .ok_or_else(|| "document closed".to_string())?;
    match out {
        Ok(outcome) => {
            changed(core, ctx, id);
            Ok(outcome)
        }
        Err(e) if e.is_empty() => Ok(Outcome::default()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faces_are_ellipses() {
        let s = json!({"blurType": "solid", "color": "#000000", "intensity": 60});
        let s = s.as_object().unwrap();
        assert_eq!(style_for("face", s)["shapeType"], "ellipse");
        assert_eq!(style_for("email", s)["shapeType"], "rectangle");
        let w = Wanted {
            category: "email".into(),
            source: [10.0, 10.0, 50.0, 12.0],
            rect: [8.0, 8.0, 54.0, 16.0],
        };
        let o = new_object("o1".into(), &w, s).unwrap();
        let m = managed_of(&o).unwrap();
        assert_eq!(
            (m.category.as_str(), m.source, m.rect),
            ("email", w.source, w.rect)
        );
        // The text is never kept (SMT-017).
        assert!(!serde_json::to_string(&o.props).unwrap().contains("text"));
    }
}
