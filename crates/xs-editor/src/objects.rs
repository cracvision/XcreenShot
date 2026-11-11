//! Objects that come from commands: validation, the tool's current style (EDT-014)
//! and the name of the undo step (EDT-092).
//!
//! Every object is stored in a canonical form: its geometry and its whole style, so a
//! `.xshot` looks the same on any machine whatever the tools' styles there.

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use xs_core::document::{translate_props, Object};
use xs_image::geometry::{Arrow, ArrowStyle};
use xs_image::shapes::{BlurObject, BlurStyle, ImageObject, ShapeObject, ShapeStyle};
use xs_image::text::{Text, TextStyle};

/// Current style of each tool: what the next object will look like (EDT-014).
/// Kept between sessions.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Styles {
    pub arrow: ArrowStyle,
    pub text: TextStyle,
    pub shape: ShapeStyle,
    pub blur: BlurStyle,
}

impl Styles {
    /// The current style of the tool that draws `kind`.
    pub fn of(&self, kind: &str) -> Option<Value> {
        match kind {
            "arrow" => serde_json::to_value(&self.arrow).ok(),
            "text" => serde_json::to_value(&self.text).ok(),
            "shape" => serde_json::to_value(&self.shape).ok(),
            "blur" => serde_json::to_value(&self.blur).ok(),
            _ => None,
        }
    }

    /// Changes the current style of the tool that draws `kind`.
    pub fn set(&mut self, kind: &str, changes: &Map<String, Value>) -> Result<Value, String> {
        match kind {
            "arrow" => self.arrow = ArrowStyle::merged(&self.arrow, changes)?,
            "text" => self.text = TextStyle::merged(&self.text, changes)?,
            "shape" => self.shape = ShapeStyle::merged(&self.shape, changes)?,
            "blur" => self.blur = BlurStyle::merged(&self.blur, changes)?,
            k => return Err(format!("the {k} tool has no style")),
        }
        self.of(kind).ok_or_else(|| "style".to_string())
    }
}

/// Object types `document.annotate` knows (EDT-119) that arrive in later builds.
const LATER_TYPES: &[&str] = &[
    "line",
    "callout",
    "step",
    "stamp",
    "highlight",
    "magnify",
    "spotlight",
];

/// Geometry properties of each type (everything but `style`).
fn geometry_keys(kind: &str) -> Option<&'static [&'static str]> {
    match kind {
        "arrow" => Some(&["from", "to"]),
        "text" => Some(&["at", "width", "text", "rotation"]),
        "shape" | "blur" | "image" => Some(&["rect", "rotation"]),
        _ => None,
    }
}

fn only_keys(input: &Map<String, Value>, allowed: &[&str]) -> Result<(), String> {
    match input
        .keys()
        .find(|k| k.as_str() != "style" && !allowed.contains(&k.as_str()))
    {
        Some(k) => Err(format!("unknown property \"{k}\"")),
        None => Ok(()),
    }
}

fn style_changes(v: Option<&Value>) -> Result<Map<String, Value>, String> {
    match v {
        None => Ok(Map::new()),
        Some(Value::Object(m)) => Ok(m.clone()),
        // A saved style by name (EDT-015) arrives in M2.
        Some(Value::String(_)) => Err("saved styles by name arrive in M2; pass the properties".to_string()),
        Some(_) => Err("style must be an object".to_string()),
    }
}

fn merge_style(current: Option<&Value>, changes: &Map<String, Value>) -> Value {
    let mut style = current.and_then(Value::as_object).cloned().unwrap_or_default();
    for (k, v) in changes {
        style.insert(k.clone(), v.clone());
    }
    Value::Object(style)
}

/// Validates `props` for `kind` and returns them in canonical form.
fn canonical(kind: &str, props: &Map<String, Value>) -> Result<Map<String, Value>, String> {
    let value = match kind {
        "arrow" => {
            let a = Arrow::from_props(props)?;
            if a.length() < 1.0 {
                return Err("from and to must be at least 1 pixel apart".to_string());
            }
            json!({"from": a.from, "to": a.to, "style": a.style})
        }
        "text" => {
            let t = Text::from_props(props)?;
            if t.text.trim().is_empty() {
                return Err("text must not be empty".to_string());
            }
            json!({"at": t.at, "width": t.width, "text": t.text, "rotation": t.rotation, "style": t.style})
        }
        "shape" => {
            let s = ShapeObject::from_props(props)?;
            json!({"rect": s.rect, "rotation": s.rotation, "style": s.style})
        }
        "blur" => {
            let b = BlurObject::from_props(props)?;
            json!({"rect": b.rect, "rotation": b.rotation, "style": b.style})
        }
        "image" => {
            let i = ImageObject::from_props(props)?;
            json!({"image": i.image, "rect": i.rect, "rotation": i.rotation, "style": i.style})
        }
        k => return Err(format!("objects of type \"{k}\" cannot be changed in this build")),
    };
    Ok(value.as_object().cloned().unwrap_or_default())
}

/// Validates one object of `document.annotate` and fills what is missing from the
/// current style. Returns its type and stored properties.
pub fn new_object(input: &Value, styles: &Styles) -> Result<(String, Map<String, Value>), String> {
    let obj = input.as_object().ok_or("must be an object")?;
    let kind = obj
        .get("type")
        .and_then(Value::as_str)
        .ok_or("type is required")?;
    if kind == "image" {
        // Its picture has to travel with it: document.insert_images (EDT-078).
        return Err("images are added with document.insert_images".to_string());
    }
    let Some(keys) = geometry_keys(kind) else {
        return Err(if LATER_TYPES.contains(&kind) {
            format!("type \"{kind}\" arrives in a later build")
        } else {
            format!("unknown type \"{kind}\"")
        });
    };
    let mut props = obj.clone();
    props.remove("type");
    only_keys(&props, keys)?;
    let style = merge_style(styles.of(kind).as_ref(), &style_changes(obj.get("style"))?);
    props.insert("style".into(), style);
    Ok((kind.to_string(), canonical(kind, &props)?))
}

/// A floating image showing `image` (a file of `images/`) in `rect` = `[x, y, w, h]`
/// (EDT-078).
pub fn image_object(image: &str, rect: [f64; 4]) -> Result<Map<String, Value>, String> {
    let [x, y, w, h] = rect;
    let props = json!({"image": image, "rect": {"x": x, "y": y, "w": w, "h": h}, "rotation": 0.0});
    canonical("image", props.as_object().expect("object"))
}

/// What an update changed, for the undo step name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    /// Only its position.
    Move,
    /// Anything else: size, style, text…
    Change,
}

/// Where an object is, minus where it would be after `translate`: equal when the
/// object only moved.
fn shape_of(kind: &str, props: &Map<String, Value>) -> Value {
    let mut p = props.clone();
    if let Some(origin) = origin(kind, &p) {
        translate_props(&mut p, -origin[0], -origin[1]);
    }
    Value::Object(p)
}

fn origin(kind: &str, p: &Map<String, Value>) -> Option<[f64; 2]> {
    let xy = |v: &Value| Some([v.get(0)?.as_f64()?, v.get(1)?.as_f64()?]);
    match kind {
        "arrow" => xy(p.get("from")?),
        "text" => xy(p.get("at")?),
        _ => {
            let r = p.get("rect")?;
            Some([r.get("x")?.as_f64()?, r.get("y")?.as_f64()?])
        }
    }
}

/// Applies `set` (geometry to replace, style properties to change) to `object`.
/// Fields of the stored object this build does not know are kept.
pub fn update(object: &mut Object, set: &Map<String, Value>) -> Result<Change, String> {
    // Locked (EDT-085): kept apart from the geometry and the style, saved in the file.
    let mut set = set.clone();
    if let Some(lock) = set.remove("locked") {
        match lock.as_bool() {
            Some(true) => {
                object.props.insert("locked".into(), Value::Bool(true));
            }
            Some(false) => {
                object.props.remove("locked");
            }
            None => return Err("locked must be true or false".to_string()),
        }
        if set.is_empty() {
            return Ok(Change::Change);
        }
    }
    let set = &set;
    let keys = geometry_keys(&object.kind).ok_or_else(|| {
        format!(
            "objects of type \"{}\" cannot be changed in this build",
            object.kind
        )
    })?;
    only_keys(set, keys)?;
    let mut props = object.props.clone();
    for (k, v) in set {
        if k == "style" {
            let style = merge_style(props.get("style"), &style_changes(Some(v))?);
            props.insert("style".into(), style);
        } else {
            props.insert(k.clone(), v.clone());
        }
    }
    let before = shape_of(
        &object.kind,
        &canonical(&object.kind, &object.props).unwrap_or_default(),
    );
    let next = canonical(&object.kind, &props)?;
    let change = if shape_of(&object.kind, &next) == before {
        Change::Move
    } else {
        Change::Change
    };
    for (k, v) in next {
        object.props.insert(k, v);
    }
    Ok(change)
}

/// Moves an object by (`dx`, `dy`) canvas pixels, whatever its type: its points
/// (`from`, `to`, `at`, `points`) and its `rect`. Used when cropping (EDT-049).
pub fn translate(object: &mut Object, dx: f64, dy: f64) {
    object.translate(dx, dy);
}

/// i18n key of the undo step for objects of one `kind` (`history.*`).
/// The stacking order after moving `ids` (EDT-081): to the front or the back of all,
/// or one place up or down past the next object that is not moving. Moved objects keep
/// their order among themselves.
pub fn arranged(objects: &mut Vec<Object>, ids: &[String], to: &str) {
    let moving = |o: &Object| ids.contains(&o.id);
    match to {
        "front" | "back" => {
            let (picked, rest): (Vec<Object>, Vec<Object>) = objects.drain(..).partition(moving);
            if to == "front" {
                objects.extend(rest);
                objects.extend(picked);
            } else {
                objects.extend(picked);
                objects.extend(rest);
            }
        }
        "forward" => {
            for i in (0..objects.len().saturating_sub(1)).rev() {
                if moving(&objects[i]) && !moving(&objects[i + 1]) {
                    objects.swap(i, i + 1);
                }
            }
        }
        "backward" => {
            for i in 1..objects.len() {
                if moving(&objects[i]) && !moving(&objects[i - 1]) {
                    objects.swap(i - 1, i);
                }
            }
        }
        _ => {}
    }
}

/// The groups an object belongs to, outermost first (EDT-082).
pub fn groups_of(o: &Object) -> Vec<String> {
    o.props
        .get("groups")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).map(String::from).collect())
        .unwrap_or_default()
}

fn set_groups(o: &mut Object, groups: Vec<String>) {
    if groups.is_empty() {
        o.props.remove("groups");
    } else {
        o.props.insert("groups".into(), json!(groups));
    }
}

/// A group id no object uses yet: `g1`, `g2`…
pub fn new_group_id(objects: &[Object]) -> String {
    let used: Vec<String> = objects.iter().flat_map(groups_of).collect();
    (1..)
        .map(|n| format!("g{n}"))
        .find(|g| !used.contains(g))
        .unwrap_or_default()
}

/// Puts `ids` in a new group around whatever groups they were in (EDT-082). Returns it.
pub fn group(objects: &mut [Object], ids: &[String]) -> String {
    let g = new_group_id(objects);
    for o in objects.iter_mut().filter(|o| ids.contains(&o.id)) {
        let mut groups = groups_of(o);
        groups.insert(0, g.clone());
        set_groups(o, groups);
    }
    g
}

/// Takes `ids` out of their outermost group; the groups inside it stay.
pub fn ungroup(objects: &mut [Object], ids: &[String]) {
    for o in objects.iter_mut().filter(|o| ids.contains(&o.id)) {
        let mut groups = groups_of(o);
        if !groups.is_empty() {
            groups.remove(0);
            set_groups(o, groups);
        }
    }
}

/// Copies keep being a group among themselves, not with the originals (EDT-088).
pub fn regroup_copies(objects: &[Object], copies: &mut [Object]) {
    let mut taken: Vec<String> = objects.iter().flat_map(groups_of).collect();
    let mut renamed: std::collections::BTreeMap<String, String> = Default::default();
    for c in copies.iter_mut() {
        let groups: Vec<String> = groups_of(c)
            .into_iter()
            .map(|g| {
                renamed
                    .entry(g)
                    .or_insert_with(|| {
                        let fresh = (1..)
                            .map(|n| format!("g{n}"))
                            .find(|x| !taken.contains(x))
                            .unwrap_or_default();
                        taken.push(fresh.clone());
                        fresh
                    })
                    .clone()
            })
            .collect();
        set_groups(c, groups);
    }
}

pub fn label(action: &str, kinds: &[&str]) -> String {
    // One kind (even several objects of it) names it; a mix is "objects".
    let what = match kinds.first() {
        Some(k) if kinds.iter().all(|x| x == k) => *k,
        _ => "objects",
    };
    format!("history.{action}_{what}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn named(ids: &[&str]) -> Vec<Object> {
        ids.iter()
            .map(|id| Object {
                id: id.to_string(),
                kind: "shape".into(),
                props: Map::new(),
            })
            .collect()
    }

    fn order(objects: &[Object]) -> Vec<&str> {
        objects.iter().map(|o| o.id.as_str()).collect()
    }

    #[test]
    fn groups_nest_and_copies_get_their_own() {
        let mut o = named(&["a", "b", "c"]);
        let g1 = group(&mut o, &["a".into(), "b".into()]);
        let g2 = group(&mut o, &["a".into(), "b".into(), "c".into()]);
        assert_eq!(groups_of(&o[0]), [g2.clone(), g1.clone()]);
        assert_eq!(groups_of(&o[2]), std::slice::from_ref(&g2));
        ungroup(&mut o, &["a".into(), "b".into(), "c".into()]);
        assert_eq!(
            groups_of(&o[0]),
            std::slice::from_ref(&g1),
            "the inner group stays"
        );
        assert!(o[2].props.get("groups").is_none());
        let mut copies = vec![o[0].clone(), o[1].clone()];
        regroup_copies(&o, &mut copies);
        let new = groups_of(&copies[0]);
        assert_eq!(new, groups_of(&copies[1]), "the copies are one group");
        assert_ne!(new, [g1]);
    }

    #[test]
    fn stacking_order() {
        // Bottom first: a, b, c.
        let mut o = named(&["a", "b", "c"]);
        arranged(&mut o, &["c".into()], "back");
        assert_eq!(order(&o), ["c", "a", "b"], "the top one goes under the other two");
        arranged(&mut o, &["c".into()], "forward");
        assert_eq!(order(&o), ["a", "c", "b"], "one place up, not to the top");
        arranged(&mut o, &["a".into(), "c".into()], "front");
        assert_eq!(order(&o), ["b", "a", "c"], "several keep their order");
        arranged(&mut o, &["c".into()], "backward");
        assert_eq!(order(&o), ["b", "c", "a"]);
    }

    #[test]
    fn missing_style_comes_from_the_tool() {
        let mut styles = Styles::default();
        styles.arrow.color = "#00FF00".into();
        let (kind, props) =
            new_object(&json!({"type": "arrow", "from": [0, 0], "to": [10, 0]}), &styles).unwrap();
        assert_eq!(kind, "arrow");
        assert_eq!(props["style"]["color"], "#00FF00");
        assert_eq!(props["style"]["endCap"], "filled");
        let (_, shape) = new_object(
            &json!({"type": "shape", "rect": {"x": 1, "y": 2, "w": 30, "h": 40}, "style": {"shapeType": "ellipse"}}),
            &styles,
        )
        .unwrap();
        assert_eq!(shape["style"]["shapeType"], "ellipse");
        assert_eq!(shape["style"]["outlineWidth"], json!(4.0));
        assert_eq!(shape["rotation"], json!(0.0));
    }

    #[test]
    fn explains_what_is_wrong() {
        let s = Styles::default();
        let err = |v: Value| new_object(&v, &s).unwrap_err();
        assert!(err(json!({"type": "spiral"})).contains("unknown type"));
        assert!(err(json!({"type": "callout", "rect": {}})).contains("later build"));
        assert!(err(json!({"type": "arrow", "from": [0, 0]})).contains("to"));
        assert!(err(json!({"type": "arrow", "from": [0, 0], "to": [0, 0]})).contains("1 pixel"));
        assert!(err(json!({"type": "arrow", "from": [0, 0], "to": [9, 9], "colour": 1})).contains("colour"));
        assert!(err(json!({"type": "arrow", "from": [0, 0], "to": [9, 9], "style": "Bold"})).contains("M2"));
        assert!(err(json!({"type": "text", "at": [0, 0], "text": "  "})).contains("empty"));
        assert!(err(json!({"type": "shape", "rect": {"x": 0, "y": 0, "w": 0, "h": 5}})).contains("1 pixel"));
        assert!(err(
            json!({"type": "blur", "rect": {"x": 0, "y": 0, "w": 9, "h": 5}, "style": {"blurType": "x"}})
        )
        .contains("style"));
    }

    #[test]
    fn updates_tell_moves_from_changes_and_keep_unknown_fields() {
        let (_, mut props) = new_object(
            &json!({"type": "arrow", "from": [0, 0], "to": [10, 0]}),
            &Styles::default(),
        )
        .unwrap();
        props.insert("future".into(), json!(1));
        let mut o = Object {
            id: "o1".into(),
            kind: "arrow".into(),
            props,
        };
        let moved = update(
            &mut o,
            json!({"from": [5, 5], "to": [15, 5]}).as_object().unwrap(),
        )
        .unwrap();
        assert_eq!(moved, Change::Move);
        let bent = update(&mut o, json!({"to": [20, 20]}).as_object().unwrap()).unwrap();
        assert_eq!(bent, Change::Change);
        let styled = update(&mut o, json!({"style": {"width": 12}}).as_object().unwrap()).unwrap();
        assert_eq!(styled, Change::Change);
        assert_eq!(o.props["style"]["width"], json!(12.0));
        assert_eq!(o.props["future"], json!(1));
        assert!(update(&mut o, json!({"style": {"width": 0}}).as_object().unwrap()).is_err());
        assert!(update(&mut o, json!({"rect": {}}).as_object().unwrap()).is_err());
    }

    #[test]
    fn translate_moves_every_kind() {
        let s = Styles::default();
        for input in [
            json!({"type": "arrow", "from": [0, 0], "to": [10, 0]}),
            json!({"type": "text", "at": [3, 4], "text": "Hi"}),
            json!({"type": "blur", "rect": {"x": 3, "y": 4, "w": 5, "h": 6}}),
        ] {
            let (kind, props) = new_object(&input, &s).unwrap();
            let mut o = Object {
                id: "o1".into(),
                kind: kind.clone(),
                props,
            };
            let before = origin(&kind, &o.props).unwrap();
            translate(&mut o, -10.0, 20.0);
            assert_eq!(
                origin(&kind, &o.props).unwrap(),
                [before[0] - 10.0, before[1] + 20.0]
            );
        }
    }

    #[test]
    fn tool_styles_change_and_validate() {
        let mut s = Styles::default();
        let v = s
            .set("text", json!({"size": 20, "bold": true}).as_object().unwrap())
            .unwrap();
        assert_eq!(v["size"], json!(20.0));
        assert!(s
            .set("blur", json!({"intensity": 500}).as_object().unwrap())
            .is_err());
        assert!(s.set("crop", &Map::new()).is_err());
    }
}
