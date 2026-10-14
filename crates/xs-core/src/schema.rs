//! Minimal JSON Schema subset used for command parameters and setting values.
//!
//! Supported keywords: `type` (string, boolean, integer, number, object, array, null,
//! or a list of them), `enum`, `properties`, `required`, `additionalProperties: false`,
//! `items`, `minimum`, `maximum`. That is all the registry needs; the schemas stay
//! valid standard JSON Schema for MCP clients.

use serde_json::Value;

/// Validation failure: `path` names the offending parameter (`""` for the root).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaError {
    pub path: String,
    pub reason: String,
}

impl std::fmt::Display for SchemaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.path.is_empty() {
            write!(f, "{}", self.reason)
        } else {
            write!(f, "{}: {}", self.path, self.reason)
        }
    }
}

fn type_matches(ty: &str, v: &Value) -> bool {
    match ty {
        "string" => v.is_string(),
        "boolean" => v.is_boolean(),
        "integer" => v.is_i64() || v.is_u64(),
        "number" => v.is_number(),
        "object" => v.is_object(),
        "array" => v.is_array(),
        "null" => v.is_null(),
        _ => true,
    }
}

pub fn validate(schema: &Value, value: &Value) -> Result<(), SchemaError> {
    validate_at(schema, value, "")
}

fn err(path: &str, reason: impl Into<String>) -> SchemaError {
    SchemaError {
        path: path.to_string(),
        reason: reason.into(),
    }
}

fn validate_at(schema: &Value, value: &Value, path: &str) -> Result<(), SchemaError> {
    match schema.get("type") {
        Some(Value::String(ty)) if !type_matches(ty, value) => {
            return Err(err(path, format!("expected {ty}")));
        }
        Some(Value::Array(types))
            if !types
                .iter()
                .filter_map(Value::as_str)
                .any(|t| type_matches(t, value)) =>
        {
            return Err(err(
                path,
                format!("expected one of {}", Value::Array(types.clone())),
            ));
        }
        _ => {}
    }
    if let Some(Value::Array(options)) = schema.get("enum") {
        if !options.contains(value) {
            return Err(err(
                path,
                format!("expected one of {}", Value::Array(options.clone())),
            ));
        }
    }
    if let Some(n) = value.as_f64() {
        if let Some(min) = schema.get("minimum").and_then(Value::as_f64) {
            if n < min {
                return Err(err(path, format!("must be >= {min}")));
            }
        }
        if let Some(max) = schema.get("maximum").and_then(Value::as_f64) {
            if n > max {
                return Err(err(path, format!("must be <= {max}")));
            }
        }
    }
    if let Value::Object(map) = value {
        let props = schema.get("properties").and_then(Value::as_object);
        if let Some(Value::Array(required)) = schema.get("required") {
            for name in required.iter().filter_map(Value::as_str) {
                if !map.contains_key(name) {
                    return Err(err(&join(path, name), "required parameter is missing"));
                }
            }
        }
        let closed = schema.get("additionalProperties") == Some(&Value::Bool(false));
        for (k, v) in map {
            match props.and_then(|p| p.get(k)) {
                Some(sub) => validate_at(sub, v, &join(path, k))?,
                None if closed => return Err(err(&join(path, k), "unknown parameter")),
                None => {}
            }
        }
    }
    if let (Value::Array(items), Some(item_schema)) = (value, schema.get("items")) {
        for (i, item) in items.iter().enumerate() {
            validate_at(item_schema, item, &format!("{path}[{i}]"))?;
        }
    }
    Ok(())
}

fn join(path: &str, name: &str) -> String {
    if path.is_empty() {
        name.to_string()
    } else {
        format!("{path}.{name}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn schema() -> Value {
        json!({
            "type": "object",
            "properties": {
                "section": {"type": "string", "enum": ["general", "shortcuts"]},
                "count": {"type": "integer", "minimum": 1, "maximum": 200}
            },
            "required": ["section"],
            "additionalProperties": false
        })
    }

    #[test]
    fn accepts_valid() {
        validate(&schema(), &json!({"section": "general", "count": 5})).unwrap();
    }

    #[test]
    fn names_unknown_parameter() {
        let e = validate(&schema(), &json!({"section": "general", "bogus": 1})).unwrap_err();
        assert_eq!(e.path, "bogus");
    }

    #[test]
    fn rejects_missing_required_enum_and_range() {
        assert_eq!(validate(&schema(), &json!({})).unwrap_err().path, "section");
        assert!(validate(&schema(), &json!({"section": "x"})).is_err());
        assert!(validate(&schema(), &json!({"section": "general", "count": 0})).is_err());
        assert!(validate(&schema(), &json!({"section": "general", "count": "5"})).is_err());
    }
}
