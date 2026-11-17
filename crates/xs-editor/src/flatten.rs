//! Flatten and Unflatten (EDT-091). Flattened objects leave the list of objects and
//! become a layer of the image: drawn over the original pixels and under the objects,
//! kept in the `.xshot` as they were (in base pixels, so a crop does not move them), and
//! made objects again on request.
//!
//! Flattening a blur cannot be undone that way: its purpose is to hide what is below.
//! Its layer, and the earlier layers under it, are merged into `base.png` when the file
//! is written (`xs_image::render::baked_base`).

use serde_json::Map;
use xs_core::document::{Document, Flattened, Object};
use xs_image::object_bounds;

/// What flattening made: its layers, oldest first, and whether one is permanent.
#[derive(Debug, Clone, PartialEq)]
pub struct Made {
    pub layers: Vec<String>,
    pub permanent: bool,
}

fn overlap(a: [f64; 4], b: [f64; 4]) -> bool {
    a[0] < b[2] && b[0] < a[2] && a[1] < b[3] && b[1] < a[3]
}

fn push(doc: &mut Document, objects: Vec<Object>, permanent: Option<&str>, made: &mut Made) {
    let id = doc.next_flattened_id();
    doc.flattened.push(Flattened {
        id: id.clone(),
        objects,
        permanent: permanent.map(str::to_string),
        extra: Map::new(),
    });
    made.layers.push(id);
}

/// Flattens the objects `ids` of `doc`, keeping their order: each blur becomes a
/// permanent layer of its own, the objects between blurs layers that can come back.
/// `None` when none of them is there.
pub fn flatten(doc: &mut Document, ids: &[String]) -> Option<Made> {
    let (mut picked, rest): (Vec<_>, Vec<_>) = std::mem::take(&mut doc.objects)
        .into_iter()
        .partition(|o| ids.contains(&o.id));
    doc.objects = rest;
    if picked.is_empty() {
        return None;
    }
    let (dx, dy) = (doc.crop.x as f64, doc.crop.y as f64);
    for o in &mut picked {
        o.translate(dx, dy);
    }
    doc.outside.retain(|id| !ids.contains(id));
    let dpi = doc.dpi;
    let mut made = Made {
        layers: Vec::new(),
        permanent: false,
    };
    let mut run = Vec::new();
    for o in picked {
        if o.kind != "blur" {
            run.push(o);
            continue;
        }
        if !run.is_empty() {
            push(doc, std::mem::take(&mut run), None, &mut made);
        }
        // A blur merged into the image over earlier layers merges them too.
        if let Some(z) = object_bounds(&o, dpi) {
            for layer in doc.flattened.iter_mut().filter(|l| l.permanent.is_none()) {
                let under = layer
                    .objects
                    .iter()
                    .filter_map(|x| object_bounds(x, dpi))
                    .any(|b| overlap(b, z));
                if under {
                    layer.permanent = Some("covered".into());
                }
            }
        }
        push(doc, vec![o], Some("blur"), &mut made);
        made.permanent = true;
    }
    if !run.is_empty() {
        push(doc, run, None, &mut made);
    }
    Some(made)
}

/// Why a layer cannot become objects again, or `None` when it can.
pub fn blocked(doc: &Document, layer: &str) -> Option<String> {
    match doc.flattened.iter().find(|l| l.id == layer) {
        None => Some("not_found".into()),
        Some(l) => l.permanent.clone(),
    }
}

/// Makes the layer `layer` objects again, under the other objects, where it showed.
/// Returns their ids.
pub fn unflatten(doc: &mut Document, layer: &str) -> Result<Vec<String>, String> {
    if let Some(reason) = blocked(doc, layer) {
        return Err(reason);
    }
    let index = doc
        .flattened
        .iter()
        .position(|l| l.id == layer)
        .ok_or("not_found")?;
    let l = doc.flattened.remove(index);
    let (dx, dy) = (-doc.crop.x as f64, -doc.crop.y as f64);
    let mut back = Vec::new();
    for mut o in l.objects {
        o.translate(dx, dy);
        // Ids stay, unless one was taken meanwhile (a file from elsewhere).
        if doc.objects.iter().chain(&back).any(|x| x.id == o.id) {
            o.id = doc.next_object_id();
        }
        back.push(o);
    }
    let ids = back.iter().map(|o| o.id.clone()).collect();
    doc.objects.splice(0..0, back);
    Ok(ids)
}

/// Makes every layer that can objects again, oldest at the bottom. Returns their ids.
pub fn unflatten_all(doc: &mut Document) -> Vec<String> {
    let layers: Vec<String> = doc
        .flattened
        .iter()
        .rev()
        .filter(|l| l.permanent.is_none())
        .map(|l| l.id.clone())
        .collect();
    let mut ids = Vec::new();
    for l in layers {
        if let Ok(mut back) = unflatten(doc, &l) {
            back.append(&mut ids);
            ids = back;
        }
    }
    ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn obj(id: &str, kind: &str, props: serde_json::Value) -> Object {
        Object {
            id: id.into(),
            kind: kind.into(),
            props: props.as_object().cloned().unwrap(),
        }
    }

    fn doc() -> Document {
        let mut d = Document::new(200, 100, 96.0);
        // A crop: layers keep base pixels, objects canvas pixels.
        d.crop.x = 10;
        d.crop.y = 5;
        d.objects = vec![
            obj(
                "o1",
                "arrow",
                json!({"from": [0, 0], "to": [50, 0], "style": {"width": 4}}),
            ),
            obj(
                "o2",
                "shape",
                json!({"rect": {"x": 20, "y": 20, "w": 30, "h": 30}, "rotation": 0}),
            ),
            obj(
                "o3",
                "blur",
                json!({"rect": {"x": 0, "y": 0, "w": 60, "h": 10}, "rotation": 0}),
            ),
        ];
        d
    }

    #[test]
    fn a_flattened_arrow_comes_back_where_it_was() {
        let mut d = doc();
        let made = flatten(&mut d, &["o1".into()]).unwrap();
        assert_eq!(
            made,
            Made {
                layers: vec!["f1".into()],
                permanent: false
            }
        );
        assert_eq!(d.objects.len(), 2);
        assert_eq!(d.flattened[0].objects[0].props["from"], json!([10.0, 5.0]));
        // A new object does not take its id.
        assert_eq!(d.next_object_id(), "o4");
        assert_eq!(unflatten(&mut d, "f1").unwrap(), ["o1"]);
        assert_eq!(d.objects[0].id, "o1", "under the others");
        assert_eq!(d.objects[0].props["from"], json!([0.0, 0.0]));
        assert!(d.flattened.is_empty());
    }

    #[test]
    fn a_flattened_blur_stays_and_takes_the_layers_under_it() {
        let mut d = doc();
        flatten(&mut d, &["o1".into()]).unwrap();
        flatten(&mut d, &["o2".into()]).unwrap();
        let made = flatten(&mut d, &["o3".into()]).unwrap();
        assert!(made.permanent);
        assert_eq!(blocked(&d, "f3").as_deref(), Some("blur"));
        // The arrow runs under the blur; the shape does not.
        assert_eq!(blocked(&d, "f1").as_deref(), Some("covered"));
        assert_eq!(blocked(&d, "f2"), None);
        assert!(unflatten(&mut d, "f1").is_err());
        assert_eq!(unflatten_all(&mut d), ["o2"]);
        assert_eq!(d.flattened.len(), 2);
    }

    #[test]
    fn only_the_blur_and_what_it_covers_stay_flat() {
        let mut d = doc();
        // Far from the blur: it can come back after flattening everything.
        d.objects.push(obj(
            "o4",
            "shape",
            json!({"rect": {"x": 150, "y": 60, "w": 20, "h": 20}, "rotation": 0}),
        ));
        let ids: Vec<String> = d.objects.iter().map(|o| o.id.clone()).collect();
        let made = flatten(&mut d, &ids).unwrap();
        assert_eq!(made.layers, ["f1", "f2", "f3"]);
        assert_eq!(blocked(&d, "f1").as_deref(), Some("covered"));
        assert_eq!(blocked(&d, "f2").as_deref(), Some("blur"));
        assert_eq!(blocked(&d, "f3"), None);
        assert_eq!(unflatten_all(&mut d), ["o4"]);
    }

    #[test]
    fn unflattening_all_keeps_the_oldest_at_the_bottom() {
        let mut d = doc();
        flatten(&mut d, &["o2".into()]).unwrap();
        flatten(&mut d, &["o1".into()]).unwrap();
        assert_eq!(unflatten_all(&mut d), ["o2", "o1"]);
        let order: Vec<&str> = d.objects.iter().map(|o| o.id.as_str()).collect();
        assert_eq!(order, ["o2", "o1", "o3"]);
    }
}
