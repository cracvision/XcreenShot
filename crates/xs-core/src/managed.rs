//! Managed configuration (APP-035): an optional `managed.json` an administrator places
//! for every user of the computer. `defaults` gives a setting its starting value,
//! `locked` fixes it, and `policy` holds rules that are not settings. A broken file
//! never stops the app: what is valid applies, the rest is listed as errors.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde_json::{json, Map, Value};

use crate::schema;
use crate::settings::SettingDef;

/// The policies this version knows (APP-035's table).
const POLICIES: &[&str] = &[
    "disabledDestinations",
    "shareLinkProvider",
    "disableVideoCapture",
    "disableUpdates",
    "updateSource",
    "hideExperimental",
    "experimental",
    "automation",
    "skipFirstRun",
    "feedback",
    "presets",
    "themes",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// No file.
    Inactive,
    Active,
    /// A file with errors: what is valid applies.
    Errors,
}

#[derive(Debug, Clone)]
pub struct Managed {
    pub path: PathBuf,
    pub status: Status,
    pub modified: Option<String>,
    pub errors: Vec<String>,
    pub defaults: BTreeMap<String, Value>,
    pub locked: BTreeMap<String, Value>,
    pub policy: Map<String, Value>,
}

impl Managed {
    pub fn inactive(path: PathBuf) -> Managed {
        Managed {
            path,
            status: Status::Inactive,
            modified: None,
            errors: Vec::new(),
            defaults: BTreeMap::new(),
            locked: BTreeMap::new(),
            policy: Map::new(),
        }
    }

    pub fn status_str(&self) -> &'static str {
        match self.status {
            Status::Inactive => "inactive",
            Status::Active => "active",
            Status::Errors => "errors",
        }
    }

    pub fn flag(&self, policy: &str) -> bool {
        self.policy.get(policy).and_then(Value::as_bool) == Some(true)
    }

    /// What `app.managed` says.
    pub fn to_json(&self) -> Value {
        json!({
            "status": self.status_str(),
            "path": self.path,
            "modified": self.modified,
            "errors": self.errors,
            "defaults": self.defaults,
            "locked": self.locked,
            "policy": self.policy,
        })
    }
}

/// Where the file is: next to the app in portable mode (APP-038); otherwise the
/// system's folder for every user. An isolated instance (tests) may name its own with
/// `XCREENSHOT_MANAGED_FILE`, so tests never read the real one.
pub fn location(portable: Option<&Path>) -> PathBuf {
    if crate::paths::isolated() {
        if let Some(p) = std::env::var_os("XCREENSHOT_MANAGED_FILE") {
            return PathBuf::from(p);
        }
    }
    if let Some(dir) = portable {
        return dir.join("managed.json");
    }
    #[cfg(windows)]
    {
        let base = std::env::var_os("ProgramData")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"C:\ProgramData"));
        base.join("XcreenShot").join("managed.json")
    }
    #[cfg(target_os = "macos")]
    {
        PathBuf::from("/Library/Application Support/XcreenShot/managed.json")
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        PathBuf::from("/etc/xcreenshot/managed.json")
    }
}

/// Reads and checks the file at `path`.
pub fn load(path: &Path, defs: &[SettingDef]) -> Managed {
    let Ok(bytes) = std::fs::read(path) else {
        return Managed::inactive(path.to_path_buf());
    };
    let modified = std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .map(|t| chrono::DateTime::<chrono::Local>::from(t).to_rfc3339());
    let mut m = parse(&bytes, defs);
    m.path = path.to_path_buf();
    m.modified = modified;
    if m.status == Status::Errors {
        tracing::warn!(path = %path.display(), errors = ?m.errors, "managed configuration has errors");
    } else {
        tracing::info!(path = %path.display(), "managed configuration applied");
    }
    m
}

/// The file's content checked against the settings and the known policies.
pub fn parse(bytes: &[u8], defs: &[SettingDef]) -> Managed {
    let mut m = Managed::inactive(PathBuf::new());
    m.status = Status::Active;
    let root: Value = match serde_json::from_slice(bytes) {
        Ok(v) => v,
        Err(e) => {
            m.status = Status::Errors;
            m.errors
                .push(format!("line {}, column {}: {e}", e.line(), e.column()));
            return m;
        }
    };
    let Some(root) = root.as_object() else {
        m.status = Status::Errors;
        m.errors.push("the file must be a JSON object".into());
        return m;
    };
    match root.get("version") {
        Some(v) if v.as_u64() == Some(1) => {}
        Some(v) => m
            .errors
            .push(format!("version {v} is not known; read as version 1")),
        None => m.errors.push("version is missing; read as version 1".into()),
    }
    for key in root.keys() {
        if !["version", "defaults", "locked", "policy"].contains(&key.as_str()) {
            m.errors.push(format!("\"{key}\" is not known"));
        }
    }
    let settings = |section: &str, into: &mut BTreeMap<String, Value>, errors: &mut Vec<String>| {
        let Some(v) = root.get(section) else { return };
        let Some(obj) = v.as_object() else {
            errors.push(format!("{section} must be an object"));
            return;
        };
        for (k, v) in obj {
            match defs.iter().find(|d| d.key == k) {
                None => errors.push(format!("{section}.{k}: there is no such setting")),
                Some(d) => match schema::validate(&d.schema, v) {
                    Ok(()) => {
                        into.insert(k.clone(), v.clone());
                    }
                    Err(e) => errors.push(format!("{section}.{k}: {}", e.reason)),
                },
            }
        }
    };
    let mut errors = std::mem::take(&mut m.errors);
    settings("defaults", &mut m.defaults, &mut errors);
    settings("locked", &mut m.locked, &mut errors);
    if let Some(v) = root.get("policy") {
        match v.as_object() {
            None => errors.push("policy must be an object".into()),
            Some(obj) => {
                for (k, v) in obj {
                    if !POLICIES.contains(&k.as_str()) {
                        errors.push(format!("policy.{k} is not known"));
                        continue;
                    }
                    let ok = match k.as_str() {
                        "disableVideoCapture" | "disableUpdates" | "hideExperimental" | "skipFirstRun" => {
                            v.is_boolean()
                        }
                        "disabledDestinations" | "presets" | "themes" => {
                            v.as_array().is_some_and(|a| a.iter().all(Value::is_string))
                        }
                        "updateSource" | "feedback" => v.is_string(),
                        "automation" | "experimental" | "shareLinkProvider" => v.is_object(),
                        _ => true,
                    };
                    if ok {
                        m.policy.insert(k.clone(), v.clone());
                    } else {
                        errors.push(format!("policy.{k} has the wrong type"));
                    }
                }
            }
        }
    }
    // Automation off by policy is a lock on its setting (APP-032).
    if m.policy.get("automation").and_then(|a| a.get("enabled")) == Some(&json!(false)) {
        m.locked.insert("automation.enabled".into(), json!(false));
    }
    m.errors = errors;
    if !m.errors.is_empty() {
        m.status = Status::Errors;
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_parts_apply_and_errors_are_listed() {
        let defs = crate::settings::definitions();
        let m = parse(
            br#"{"version": 1,
                "defaults": {"general.language": "es", "nope.key": 1},
                "locked": {"capture.autoDetect": false, "general.theme": "purple"},
                "policy": {"disableVideoCapture": true, "automation": {"enabled": false}, "wat": 1}}"#,
            &defs,
        );
        assert_eq!(m.status, Status::Errors);
        assert_eq!(m.defaults.get("general.language"), Some(&json!("es")));
        assert_eq!(m.locked.get("capture.autoDetect"), Some(&json!(false)));
        assert!(!m.locked.contains_key("general.theme"), "invalid value ignored");
        assert_eq!(m.locked.get("automation.enabled"), Some(&json!(false)));
        assert!(m.flag("disableVideoCapture"));
        assert_eq!(m.errors.len(), 3, "{:?}", m.errors);
    }

    #[test]
    fn a_broken_file_says_where() {
        let m = parse(
            b"{\n  \"version\": 1,\n  oops\n}",
            &crate::settings::definitions(),
        );
        assert_eq!(m.status, Status::Errors);
        assert!(m.errors[0].starts_with("line 3"), "{:?}", m.errors);
        assert!(m.locked.is_empty() && m.defaults.is_empty());
    }
}
