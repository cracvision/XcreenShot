//! Preferences (APP-006, APP-007) with stable setting keys (`area.name`).
//!
//! Only settings whose feature exists are defined; owners add theirs as they land.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::RwLock;

use serde::Serialize;
use serde_json::{json, Map, Value};

use crate::command::Source;
use crate::error::{CommandError, ErrorCode};
use crate::i18n::{tr, Lang};
use crate::paths::write_atomic;
use crate::platform::Platform;
use crate::schema;

/// Preferences sections, in sidebar order (APP-006).
pub const SECTIONS: [&str; 9] = [
    "general",
    "capture",
    "video",
    "editor",
    "library",
    "share",
    "shortcuts",
    "advanced",
    "experimental",
];

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingDef {
    pub key: &'static str,
    pub section: &'static str,
    pub schema: Value,
    pub default: Value,
    /// `None` = both platforms.
    pub platform: Option<Platform>,
    /// CLI/MCP may change it with `settings.set` (APP-032).
    pub automation_writable: bool,
    pub spec_id: &'static str,
}

fn def(
    key: &'static str,
    section: &'static str,
    schema: Value,
    default: Value,
    spec_id: &'static str,
) -> SettingDef {
    SettingDef {
        key,
        section,
        schema,
        default,
        platform: None,
        automation_writable: true,
        spec_id,
    }
}

fn boolean() -> Value {
    json!({"type": "boolean"})
}

fn one_of(values: &[&str]) -> Value {
    json!({"type": "string", "enum": values})
}

/// All settings that exist in this build, in display order.
pub fn definitions() -> Vec<SettingDef> {
    let mut icon_click = def(
        "general.iconClickAction",
        "general",
        one_of(&["capturePanel", "universalCapture", "editor"]),
        json!("capturePanel"),
        "APP-002",
    );
    icon_click.platform = Some(Platform::Windows);
    let locked_down = |mut d: SettingDef| {
        d.automation_writable = false;
        d
    };
    vec![
        def(
            "general.launchAtLogin",
            "general",
            boolean(),
            json!(true),
            "APP-003",
        ),
        def("general.showIcon", "general", boolean(), json!(true), "APP-002"),
        def(
            "general.monochromeIcon",
            "general",
            boolean(),
            json!(false),
            "APP-002",
        ),
        icon_click,
        def(
            "general.keepRunning",
            "general",
            boolean(),
            json!(true),
            "APP-004",
        ),
        def(
            "general.theme",
            "general",
            one_of(&["system", "light", "dark"]),
            json!("system"),
            "APP-012",
        ),
        def(
            "general.language",
            "general",
            one_of(&["system", "en", "es"]),
            json!("system"),
            "APP-013",
        ),
        def(
            "shortcuts.profile",
            "shortcuts",
            one_of(&["coexistence", "compatibility"]),
            json!("coexistence"),
            "APP-021",
        ),
        def(
            "shortcuts.takeOver",
            "shortcuts",
            boolean(),
            json!(false),
            "APP-020",
        ),
        def(
            "advanced.keepEditorLoaded",
            "advanced",
            boolean(),
            json!(true),
            "APP-004",
        ),
        locked_down(def(
            "automation.enabled",
            "advanced",
            boolean(),
            json!(true),
            "APP-032",
        )),
        locked_down(def(
            "automation.notifyOnCapture",
            "advanced",
            boolean(),
            json!(true),
            "APP-032",
        )),
        locked_down(def(
            "automation.allowOnlineShare",
            "advanced",
            boolean(),
            json!(false),
            "APP-032",
        )),
        def(
            "diagnostics.verbose",
            "advanced",
            boolean(),
            json!(false),
            "APP-046",
        ),
        def(
            "capture.hideOnCapture",
            "capture",
            boolean(),
            json!(true),
            "CAP-023",
        ),
        def(
            "capture.metadata.windowTitle",
            "capture",
            boolean(),
            json!(true),
            "CAP-056",
        ),
        def(
            "capture.metadata.url",
            "capture",
            boolean(),
            json!(true),
            "CAP-056",
        ),
        def(
            "capture.metadata.userAndMachine",
            "capture",
            boolean(),
            json!(true),
            "CAP-056",
        ),
    ]
}

/// Persistent settings store backed by `settings.json`. Only non-default values are
/// written, so changing a default in a new version reaches users who never touched it.
pub struct SettingsStore {
    path: Option<PathBuf>,
    defs: Vec<SettingDef>,
    values: RwLock<BTreeMap<String, Value>>,
    platform: Platform,
}

impl SettingsStore {
    pub fn load(path: PathBuf) -> SettingsStore {
        let values = std::fs::read(&path)
            .ok()
            .and_then(|b| serde_json::from_slice::<BTreeMap<String, Value>>(&b).ok())
            .unwrap_or_default();
        let store = SettingsStore {
            path: Some(path),
            defs: definitions(),
            values: RwLock::new(BTreeMap::new()),
            platform: Platform::current(),
        };
        // Drop unknown or invalid values instead of failing to start.
        let mut clean = BTreeMap::new();
        for (k, v) in values {
            if let Some(d) = store.def(&k) {
                if schema::validate(&d.schema, &v).is_ok() {
                    clean.insert(k, v);
                }
            }
        }
        *store.values.write().unwrap() = clean;
        store
    }

    pub fn in_memory() -> SettingsStore {
        SettingsStore {
            path: None,
            defs: definitions(),
            values: RwLock::new(BTreeMap::new()),
            platform: Platform::current(),
        }
    }

    pub fn defs(&self) -> &[SettingDef] {
        &self.defs
    }

    pub fn def(&self, key: &str) -> Option<&SettingDef> {
        self.defs.iter().find(|d| d.key == key)
    }

    fn available(&self, d: &SettingDef) -> bool {
        d.platform.is_none_or(|p| p == self.platform)
    }

    fn not_found(&self, key: &str, lang: Lang) -> CommandError {
        CommandError::new(
            ErrorCode::NotFound,
            tr(lang, "errors.unknown_setting", &[("key", key)]),
        )
    }

    pub fn get(&self, key: &str) -> Option<Value> {
        let d = self.def(key)?;
        Some(
            self.values
                .read()
                .unwrap()
                .get(key)
                .cloned()
                .unwrap_or_else(|| d.default.clone()),
        )
    }

    pub fn get_bool(&self, key: &str) -> bool {
        self.get(key).and_then(|v| v.as_bool()).unwrap_or(false)
    }

    pub fn get_str(&self, key: &str) -> String {
        self.get(key)
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default()
    }

    /// UI language resolved from `general.language`.
    pub fn lang(&self) -> Lang {
        Lang::resolve(&self.get_str("general.language"))
    }

    /// Validates and stores a value. Returns `true` if it changed.
    pub fn set(&self, key: &str, value: Value, source: Source, lang: Lang) -> Result<bool, CommandError> {
        let d = self.def(key).ok_or_else(|| self.not_found(key, lang))?;
        if !self.available(d) {
            return Err(CommandError::new(
                ErrorCode::NotSupportedOnPlatform,
                tr(lang, "errors.setting_other_platform", &[("key", key)]),
            ));
        }
        if source.is_automation() && !d.automation_writable {
            return Err(CommandError::new(
                ErrorCode::PermissionDenied,
                tr(lang, "errors.setting_preferences_only", &[("key", key)]),
            ));
        }
        schema::validate(&d.schema, &value).map_err(|e| {
            CommandError::new(
                ErrorCode::InvalidParams,
                tr(
                    lang,
                    "errors.invalid_value",
                    &[("key", key), ("reason", &e.reason)],
                ),
            )
        })?;
        let changed = {
            let mut values = self.values.write().unwrap();
            let old = values.get(key).cloned().unwrap_or_else(|| d.default.clone());
            if value == d.default {
                values.remove(key);
            } else {
                values.insert(key.to_string(), value.clone());
            }
            old != value
        };
        if changed {
            self.save()
                .map_err(|e| CommandError::internal(format!("cannot save settings: {e}")))?;
        }
        Ok(changed)
    }

    /// Replaces a value from an external source of truth (e.g. the OS login items list)
    /// without the automation checks.
    pub fn sync_external(&self, key: &str, value: Value) -> bool {
        let Some(d) = self.def(key) else { return false };
        let changed = {
            let mut values = self.values.write().unwrap();
            let old = values.get(key).cloned().unwrap_or_else(|| d.default.clone());
            if value == d.default {
                values.remove(key);
            } else {
                values.insert(key.to_string(), value.clone());
            }
            old != value
        };
        if changed {
            let _ = self.save();
        }
        changed
    }

    fn save(&self) -> std::io::Result<()> {
        let Some(path) = &self.path else { return Ok(()) };
        let values = self.values.read().unwrap();
        write_atomic(path, &serde_json::to_vec_pretty(&*values)?)
    }

    /// Entries for `settings.list` and the Preferences window.
    pub fn list(&self, lang: Lang) -> Vec<Value> {
        self.defs
            .iter()
            .filter(|d| self.available(d))
            .map(|d| {
                let base = format!("settings.{}", d.key.replace('.', "_"));
                json!({
                    "key": d.key,
                    "section": d.section,
                    "label": tr(lang, &format!("{base}.label"), &[]),
                    "description": tr(lang, &format!("{base}.description"), &[]),
                    "schema": d.schema,
                    "default": d.default,
                    "value": self.get(d.key),
                    "locked": false,
                    "automationWritable": d.automation_writable,
                    "specId": d.spec_id,
                })
            })
            .collect()
    }

    pub fn snapshot(&self) -> Map<String, Value> {
        self.defs
            .iter()
            .filter(|d| self.available(d))
            .map(|d| (d.key.to_string(), self.get(d.key).unwrap_or(Value::Null)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_spec_tables() {
        let s = SettingsStore::in_memory();
        assert_eq!(s.get("general.theme"), Some(json!("system")));
        assert_eq!(s.get("general.language"), Some(json!("system")));
        assert_eq!(s.get("general.keepRunning"), Some(json!(true)));
        assert_eq!(s.get("general.monochromeIcon"), Some(json!(false)));
        assert_eq!(s.get("automation.enabled"), Some(json!(true)));
        assert_eq!(s.get("automation.allowOnlineShare"), Some(json!(false)));
        assert_eq!(s.get("shortcuts.profile"), Some(json!("coexistence")));
    }

    #[test]
    fn rejects_invalid_and_unknown_values() {
        let s = SettingsStore::in_memory();
        let e = s
            .set("general.theme", json!("purple"), Source::Ui, Lang::En)
            .unwrap_err();
        assert_eq!(e.code, ErrorCode::InvalidParams);
        let e = s.set("nope.nope", json!(1), Source::Ui, Lang::En).unwrap_err();
        assert_eq!(e.code, ErrorCode::NotFound);
    }

    /// APP-032: automation cannot change `automation.*`.
    #[test]
    fn automation_cannot_change_sensitive_settings() {
        let s = SettingsStore::in_memory();
        let e = s
            .set("automation.enabled", json!(false), Source::Cli, Lang::En)
            .unwrap_err();
        assert_eq!(e.code.exit_code(), 6);
        assert!(e.message.contains("Preferences"), "{}", e.message);
        assert!(s
            .set("automation.enabled", json!(false), Source::Ui, Lang::En)
            .unwrap());
    }

    #[test]
    fn persists_only_non_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("settings.json");
        let s = SettingsStore::load(path.clone());
        s.set("general.theme", json!("dark"), Source::Ui, Lang::En)
            .unwrap();
        s.set("general.language", json!("system"), Source::Ui, Lang::En)
            .unwrap();
        let saved: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        assert_eq!(saved, json!({"general.theme": "dark"}));
        let reloaded = SettingsStore::load(path);
        assert_eq!(reloaded.get("general.theme"), Some(json!("dark")));
    }

    #[test]
    fn every_setting_section_exists() {
        for d in definitions() {
            assert!(SECTIONS.contains(&d.section), "{}", d.key);
            assert!(schema::validate(&d.schema, &d.default).is_ok(), "{}", d.key);
        }
    }
}
