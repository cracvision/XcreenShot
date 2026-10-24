//! Objects that come from commands: validation, the tool's current style (EDT-014)
//! and the name of the undo step (EDT-092).

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use xs_core::document::Object;
use xs_image::geometry::{point, Arrow, ArrowStyle};

/// Current style of each tool: what the next object will look like (EDT-014).
/// Kept between sessions.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Styles {
    pub arrow: ArrowStyle,
}

/// Object types `document.annotate` knows (EDT-119). Only arrows exist in this build.
const LATER_TYPES: &[&str] = &[
    "line",
    "text",
    "callout",
    "shape",
    "step",
    "stamp",
    "blur",
    "highlight",
    "magnify",
    "spotlight",
    "image",
];

fn only_keys(input: &Map<String, Value>, allowed: &[&str]) -> Result<(), String> {
    match input.keys().find(|k| !allowed.contains(&k.as_str())) {
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

/// Validates one object of `document.annotate` and fills what is missing from the
/// current style. Returns its type and stored properties.
pub fn new_object(input: &Value, styles: &Styles) -> Result<(String, Map<String, Value>), String> {
    let obj = input.as_object().ok_or("must be an object")?;
    let kind = obj
        .get("type")
        .and_then(Value::as_str)
        .ok_or("type is required")?;
    match kind {
        "arrow" => {
            only_keys(obj, &["type", "from", "to", "style"])?;
            let from = point(obj.get("from"), "from")?;
            let to = point(obj.get("to"), "to")?;
            let style = ArrowStyle::merged(&styles.arrow, &style_changes(obj.get("style"))?)?;
            let arrow = Arrow { from, to, style };
            if arrow.length() < 1.0 {
                return Err("from and to must be at least 1 pixel apart".to_string());
            }
            Ok((kind.to_string(), arrow_props(&arrow)))
        }
        k if LATER_TYPES.contains(&k) => Err(format!("type \"{k}\" arrives in a later build")),
        k => Err(format!("unknown type \"{k}\"")),
    }
}

fn arrow_props(a: &Arrow) -> Map<String, Value> {
    let mut m = Map::new();
    m.insert("from".into(), json!(a.from));
    m.insert("to".into(), json!(a.to));
    m.insert("style".into(), serde_json::to_value(&a.style).unwrap_or_default());
    m
}

/// What an update changed, for the undo step name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Geometry,
    Style,
}

/// Applies `set` (geometry to replace, style properties to change) to `object`.
/// Fields of the stored object this build does not know are kept.
pub fn update(object: &mut Object, set: &Map<String, Value>) -> Result<Change, String> {
    match object.kind.as_str() {
        "arrow" => {
            only_keys(set, &["from", "to", "style"])?;
            let mut arrow = Arrow::from_props(&object.props)?;
            if set.contains_key("from") {
                arrow.from = point(set.get("from"), "from")?;
            }
            if set.contains_key("to") {
                arrow.to = point(set.get("to"), "to")?;
            }
            if let Some(s) = set.get("style") {
                arrow.style = ArrowStyle::merged(&arrow.style, &style_changes(Some(s))?)?;
            }
            if arrow.length() < 1.0 {
                return Err("from and to must be at least 1 pixel apart".to_string());
            }
            for (k, v) in arrow_props(&arrow) {
                object.props.insert(k, v);
            }
            Ok(if set.contains_key("style") {
                Change::Style
            } else {
                Change::Geometry
            })
        }
        k => Err(format!("objects of type \"{k}\" cannot be changed in this build")),
    }
}

/// i18n key of the undo step for objects of one `kind` (`history.*`).
pub fn label(action: &str, kinds: &[&str]) -> String {
    let what = match kinds {
        [k] => k,
        _ => "objects",
    };
    format!("history.{action}_{what}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_style_comes_from_the_tool() {
        let mut styles = Styles::default();
        styles.arrow.color = "#00FF00".into();
        let (kind, props) =
            new_object(&json!({"type": "arrow", "from": [0, 0], "to": [10, 0]}), &styles).unwrap();
        assert_eq!(kind, "arrow");
        assert_eq!(props["style"]["color"], "#00FF00");
        assert_eq!(props["style"]["endCap"], "filled");
    }

    #[test]
    fn explains_what_is_wrong() {
        let s = Styles::default();
        let err = |v: Value| new_object(&v, &s).unwrap_err();
        assert!(err(json!({"type": "spiral"})).contains("unknown type"));
        assert!(err(json!({"type": "text", "at": [0, 0]})).contains("later build"));
        assert!(err(json!({"type": "arrow", "from": [0, 0]})).contains("to"));
        assert!(err(json!({"type": "arrow", "from": [0, 0], "to": [0, 0]})).contains("1 pixel"));
        assert!(err(json!({"type": "arrow", "from": [0, 0], "to": [9, 9], "colour": 1})).contains("colour"));
        assert!(err(json!({"type": "arrow", "from": [0, 0], "to": [9, 9], "style": "Bold"})).contains("M2"));
    }

    #[test]
    fn updates_geometry_or_style_and_keeps_unknown_fields() {
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
            json!({"from": [5, 5], "to": [20, 5]}).as_object().unwrap(),
        )
        .unwrap();
        assert_eq!(moved, Change::Geometry);
        assert_eq!(o.props["from"], json!([5.0, 5.0]));
        let styled = update(&mut o, json!({"style": {"width": 12}}).as_object().unwrap()).unwrap();
        assert_eq!(styled, Change::Style);
        assert_eq!(o.props["style"]["width"], json!(12.0));
        assert_eq!(o.props["future"], json!(1));
        assert!(update(&mut o, json!({"style": {"width": 0}}).as_object().unwrap()).is_err());
    }
}
