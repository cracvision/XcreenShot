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
pub const SECTIONS: [&str; 10] = [
    "general",
    "capture",
    "video",
    "editor",
    "library",
    "share",
    "ai",
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

/// Models the cloud AI can use (SMT-039), most capable first.
pub const AI_MODELS: [&str; 3] = ["claude-opus-5-5", "claude-sonnet-5-5", "claude-haiku-5-5"];

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
        // Tooltips and hints (APP-014).
        def(
            "general.tooltips",
            "general",
            one_of(&["off", "brief", "detailed"]),
            json!("detailed"),
            "APP-014",
        ),
        def("general.hints", "general", boolean(), json!(true), "APP-014"),
        // Notification categories (APP-016); Share and Updates arrive with their features.
        def(
            "notifications.captures",
            "general",
            boolean(),
            json!(true),
            "APP-016",
        ),
        def(
            "notifications.problems",
            "general",
            boolean(),
            json!(true),
            "APP-016",
        ),
        def(
            "notifications.automation",
            "general",
            boolean(),
            json!(true),
            "APP-016",
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
        // Preferences > Library (LIB-, TRY-), M1 part.
        def(
            "library.autosaveImages",
            "library",
            boolean(),
            json!(true),
            "LIB-020",
        ),
        def(
            "library.keepEditableCopy",
            "library",
            boolean(),
            json!(true),
            "LIB-021",
        ),
        def(
            "library.confirmDelete",
            "library",
            boolean(),
            json!(true),
            "LIB-015",
        ),
        def(
            "tray.clearUnpinnedOnExit",
            "library",
            boolean(),
            json!(false),
            "TRY-008",
        ),
        // Preferences > Editor (EDT-118), in its order; the rest arrive with their features.
        def("editor.shrinkToFit", "editor", boolean(), json!(true), "EDT-007"),
        def("editor.rememberView", "editor", boolean(), json!(true), "EDT-007"),
        def(
            "editor.reopenDocuments",
            "editor",
            boolean(),
            json!(true),
            "EDT-120",
        ),
        def(
            "editor.escClosesEditor",
            "editor",
            boolean(),
            json!(false),
            "EDT-012",
        ),
        def(
            "editor.toolbarLabels",
            "editor",
            boolean(),
            json!(true),
            "EDT-003",
        ),
        def(
            "editor.autoSelectAfterDraw",
            "editor",
            boolean(),
            json!(true),
            "EDT-022",
        ),
        def(
            "editor.simplifyLockOnExit",
            "editor",
            boolean(),
            json!(false),
            "EDT-041",
        ),
        def("editor.scaleStrokes", "editor", boolean(), json!(true), "EDT-080"),
        def(
            "editor.cursorStylePreview",
            "editor",
            boolean(),
            json!(false),
            "EDT-015",
        ),
        def(
            "editor.autoExpandCanvas",
            "editor",
            boolean(),
            json!(true),
            "EDT-071",
        ),
        def("editor.snap", "editor", boolean(), json!(true), "EDT-084"),
        def("editor.quickEditBar", "editor", boolean(), json!(true), "EDT-006"),
        // The pixel grid from a zoom on (EDT-008).
        def("editor.pixelGrid", "editor", boolean(), json!(true), "EDT-008"),
        def(
            "editor.pixelGridMinZoom",
            "editor",
            json!({"type": "integer", "minimum": 400, "maximum": 1600}),
            json!(600),
            "EDT-008",
        ),
        // Around the canvas: "auto" follows the theme, or a color (EDT-010).
        def(
            "editor.workspaceColor",
            "editor",
            json!({"type": "string"}),
            json!("auto"),
            "EDT-010",
        ),
        // What fills transparency in JPG, BMP, GIF without it and PDF (EDT-112).
        def(
            "editor.flattenBackground",
            "editor",
            json!({"type": "string"}),
            json!("#FFFFFF"),
            "EDT-112",
        ),
        // A new image (EDT-103): its size and canvas color (null: transparent).
        def(
            "editor.newImage.width",
            "editor",
            json!({"type": "integer", "minimum": 1, "maximum": 32000}),
            json!(1280),
            "EDT-103",
        ),
        def(
            "editor.newImage.height",
            "editor",
            json!({"type": "integer", "minimum": 1, "maximum": 32000}),
            json!(720),
            "EDT-103",
        ),
        def(
            "editor.newImage.color",
            "editor",
            json!({"type": ["string", "null"]}),
            json!("#FFFFFF"),
            "EDT-103",
        ),
        def(
            "editor.deleteCroppedContent",
            "editor",
            boolean(),
            json!(false),
            "EDT-050",
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
        // Cloud AI (SMT-039): only the Preferences window may turn it on.
        locked_down(def("ai.enabled", "ai", boolean(), json!(false), "SMT-039")),
        def(
            "ai.model",
            "ai",
            one_of(&AI_MODELS),
            json!("claude-opus-5-5"),
            "SMT-039",
        ),
        def(
            "ai.namingModel",
            "ai",
            one_of(&AI_MODELS),
            json!("claude-haiku-5-5"),
            "SMT-039",
        ),
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
            "capture.showMagnifier",
            "capture",
            boolean(),
            json!(true),
            "CAP-015",
        ),
        def("capture.autoDetect", "capture", boolean(), json!(true), "CAP-016"),
        def(
            "capture.simpleCrosshair",
            "capture",
            boolean(),
            json!(false),
            "CAP-014",
        ),
        // Capture confirmation (CAP-055); its notification is the Captures category.
        def(
            "capture.sound",
            "capture",
            one_of(&["none", "shutter", "soft"]),
            json!("none"),
            "CAP-055",
        ),
        def("capture.flash", "capture", boolean(), json!(true), "CAP-055"),
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

    /// Puts the settings of `section` (all, with `None`) back to their defaults
    /// (APP-010). Returns the keys that changed, with their default.
    pub fn reset(&self, section: Option<&str>, keep: &[&str]) -> std::io::Result<Vec<(String, Value)>> {
        let mut changed = Vec::new();
        {
            let mut values = self.values.write().unwrap();
            for d in &self.defs {
                if section.is_some_and(|s| s != d.section) || !self.available(d) || keep.contains(&d.key) {
                    continue;
                }
                if values.remove(d.key).is_some_and(|old| old != d.default) {
                    changed.push((d.key.to_string(), d.default.clone()));
                }
            }
        }
        if !changed.is_empty() {
            self.save()?;
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
