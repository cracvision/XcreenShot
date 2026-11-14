//! Capture presets (CAP-058 to CAP-064): named capture settings of one panel tab,
//! run from the panel, the icon menu, their own global shortcut or `preset.run`,
//! without touching the panel's own settings.
//!
//! They live in the core's memory (`capture.presets`), in the user's order. Each one
//! has a row in Preferences > Shortcuts (`preset.<id>`, group Presets) whose shortcut
//! belongs to the preset: it survives a rename and a profile change, and goes when the
//! preset goes.

use std::collections::HashMap;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use xs_core::i18n::{tr, Lang};
use xs_core::shortcuts::{Accelerator, ShortcutScope, Slot};
use xs_core::Core;

use crate::panel::{ImageTab, PanelSettings, UniversalTab};

pub const MEMORY_KEY: &str = "capture.presets";

/// The kinds of preset that exist in this build (video arrives with M4).
pub const KINDS: [&str; 3] = ["universal", "image", "video"];

/// Longest name (CAP-059).
pub const NAME_MAX: usize = 64;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preset {
    pub id: String,
    /// As the user wrote it; a built-in one not renamed shows its translated name.
    pub name: String,
    /// `universal` or `image`.
    pub kind: String,
    /// The tab's settings. Fields this build does not know (a mode of the other
    /// platform) are kept as they came (CAP-063).
    pub settings: Value,
    /// A preset that comes with the app: which one (CAP-064).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub builtin: Option<String>,
    /// The user named it: a built-in name is no longer translated (CAP-064).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub renamed: bool,
    /// Fields of a newer build, kept as they are.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Stored {
    pub presets: Vec<Preset>,
    /// Built-in presets already created once: deleting one does not bring it back
    /// at the next start (CAP-064).
    #[serde(default)]
    pub builtins_seen: Vec<String>,
}

/// The presets that come with the app, in order (CAP-064).
pub fn builtins() -> Vec<(&'static str, &'static str, Value)> {
    vec![
        (
            "region_clipboard",
            "image",
            json!({"selection": "region", "open": "none", "copy": true, "cursor": false, "timing": "none", "delay": 5}),
        ),
        (
            "extract_text",
            "image",
            json!({"selection": "text", "open": "none", "copy": true, "cursor": false, "timing": "none", "delay": 5}),
        ),
        (
            "scrolling",
            "image",
            json!({"selection": "scrolling", "open": "editor", "copy": true, "cursor": false, "timing": "none", "delay": 5,
                "scroll": {"type": "auto", "direction": "vertical", "startFrom": "start", "wait": 0.3, "restore": true}}),
        ),
        // The step recorder (SMT-022).
        (
            "steps",
            "image",
            json!({"selection": "steps", "open": "editor", "copy": false, "cursor": true, "timing": "none", "delay": 5,
                "steps": {"area": "all", "title": ""}}),
        ),
    ]
}

fn builtin_preset(id: &str) -> Option<Preset> {
    builtins()
        .into_iter()
        .find(|(b, _, _)| *b == id)
        .map(|(b, kind, settings)| Preset {
            id: new_id(),
            name: b.to_string(),
            kind: kind.to_string(),
            settings,
            builtin: Some(b.to_string()),
            renamed: false,
            extra: Map::new(),
        })
}

pub fn new_id() -> String {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(0);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    format!(
        "p{:x}{:x}",
        nanos as u64 & 0xffff_ffff_ffff,
        N.fetch_add(1, Ordering::Relaxed)
    )
}

/// The presets, with the built-in ones a new install (or a new version) brings.
pub fn load(core: &Core) -> Stored {
    let mut stored = core
        .recall(MEMORY_KEY)
        .and_then(|v| serde_json::from_value::<Stored>(v).ok())
        .unwrap_or_default();
    let mut added = false;
    for (id, _, _) in builtins() {
        if !stored.builtins_seen.iter().any(|s| s == id) {
            if let Some(p) = builtin_preset(id) {
                stored.presets.push(p);
            }
            stored.builtins_seen.push(id.to_string());
            added = true;
        }
    }
    if added {
        core.remember(MEMORY_KEY, serde_json::to_value(&stored).unwrap_or_default());
    }
    stored
}

pub fn save(core: &Core, stored: &Stored) {
    core.remember(MEMORY_KEY, serde_json::to_value(stored).unwrap_or_default());
}

/// The name shown: a built-in preset not renamed is called in the app's language.
pub fn display_name(p: &Preset, lang: Lang) -> String {
    match (&p.builtin, p.renamed) {
        (Some(b), false) => tr(lang, &format!("presets.builtin.{b}"), &[]),
        _ => p.name.clone(),
    }
}

/// A name as kept: 1 to 64 characters, no line breaks (CAP-059).
pub fn clean_name(name: &str) -> Result<String, String> {
    let n = name.trim();
    if n.is_empty() {
        return Err("give a name".into());
    }
    if n.chars().count() > NAME_MAX {
        return Err(format!("at most {NAME_MAX} characters"));
    }
    if n.contains(['\n', '\r']) {
        return Err("no line breaks".into());
    }
    Ok(n.to_string())
}

/// `wanted`, or "wanted (2)"… when another preset has it, whatever the case (CAP-059).
pub fn unique_name(presets: &[Preset], wanted: &str, except: Option<&str>, lang: Lang) -> String {
    let taken = |n: &str| {
        presets
            .iter()
            .filter(|p| Some(p.id.as_str()) != except)
            .any(|p| display_name(p, lang).to_lowercase() == n.to_lowercase())
    };
    if !taken(wanted) {
        return wanted.to_string();
    }
    let mut n = 2;
    loop {
        let candidate = format!("{wanted} ({n})");
        if !taken(&candidate) {
            return candidate;
        }
        n += 1;
    }
}

/// The tab settings of a kind, checked; unknown fields stay as they came.
pub fn checked_settings(kind: &str, settings: &Value) -> Result<Value, String> {
    if !KINDS.contains(&kind) {
        return Err(format!("kind must be one of {}", KINDS.join(", ")));
    }
    let panel = panel_with(kind, settings)?;
    let mut normalized = match kind {
        "universal" => serde_json::to_value(&panel.universal),
        // Video presets (VID-009): what to record and the timer.
        "video" => serde_json::to_value(&panel.video),
        _ => serde_json::to_value(&panel.image),
    }
    .map_err(|e| e.to_string())?;
    // What this build does not know is kept (CAP-063).
    if let (Some(n), Some(s)) = (normalized.as_object_mut(), settings.as_object()) {
        for (k, v) in s {
            n.entry(k.clone()).or_insert_with(|| v.clone());
        }
    }
    Ok(normalized)
}

/// Whether two settings of a `kind` preset mean the same: fields a newer build added
/// with their default values count as the same as missing ones.
pub fn same_settings(kind: &str, a: &Value, b: &Value) -> bool {
    let norm = |v: &Value| checked_settings(kind, v).unwrap_or_else(|_| v.clone());
    norm(a) == norm(b)
}

/// Panel settings whose `kind` tab is the preset's, for planning a capture.
fn panel_with(kind: &str, settings: &Value) -> Result<PanelSettings, String> {
    let mut panel = PanelSettings {
        tab: kind.to_string(),
        ..PanelSettings::default()
    };
    match kind {
        "universal" => {
            panel.universal =
                serde_json::from_value::<UniversalTab>(settings.clone()).map_err(|e| e.to_string())?
        }
        "video" => {
            panel.video = serde_json::from_value::<crate::panel::VideoTab>(settings.clone())
                .map_err(|e| e.to_string())?
        }
        _ => panel.image = serde_json::from_value::<ImageTab>(settings.clone()).map_err(|e| e.to_string())?,
    }
    panel.validate()?;
    Ok(panel)
}

/// The command and parameters that run a preset (CAP-058): its own settings only.
pub fn plan(p: &Preset) -> Result<(&'static str, Value, Value), String> {
    let panel = panel_with(&p.kind, &p.settings)?;
    let (command, params, extra) = panel.plan(&p.kind);
    Ok((command, params, extra))
}

fn selection_word(selection: &str, lang: Lang) -> String {
    tr(lang, &format!("presets.selection.{selection}"), &[])
}

/// A name that says what a preset does ("Región", "Ventana con retardo"…, CAP-059).
pub fn proposed_name(kind: &str, settings: &Value, lang: Lang) -> String {
    let selection = settings["selection"].as_str().unwrap_or("region");
    let base = selection_word(selection, lang);
    if kind == "universal" {
        return tr(lang, "presets.universal_name", &[("selection", &base)]);
    }
    if kind == "video" {
        return tr(lang, "presets.video_name", &[("selection", &base)]);
    }
    base
}

/// The short summary of a row (CAP-058): "Región · Editor · Portapapeles · 5 s".
pub fn summary(p: &Preset, lang: Lang) -> String {
    let s = &p.settings;
    let mut parts = vec![selection_word(s["selection"].as_str().unwrap_or("region"), lang)];
    // Video (VID-009): Video · Region · 5 min.
    if p.kind == "video" {
        parts.insert(0, tr(lang, "presets.kind.video", &[]));
        if let Some(limit) = s["limit"].as_u64().filter(|l| *l > 0) {
            parts.push(tr(
                lang,
                "presets.summary.limit",
                &[("minutes", &(limit / 60).max(1).to_string())],
            ));
        }
        return parts.join(" · ");
    }
    if p.kind == "universal" {
        parts.insert(0, tr(lang, "presets.kind.universal", &[]));
    }
    parts.push(tr(
        lang,
        if s["open"] == "none" {
            "presets.summary.no_editor"
        } else {
            "presets.summary.editor"
        },
        &[],
    ));
    if s["copy"] == true {
        parts.push(tr(lang, "presets.summary.clipboard", &[]));
    }
    if s["cursor"] == true {
        parts.push(tr(lang, "presets.summary.cursor", &[]));
    }
    let delay = if p.kind == "universal" {
        if s["delay5"] == true {
            5
        } else {
            0
        }
    } else if s["timing"] == "delay" {
        s["delay"].as_u64().unwrap_or(5)
    } else {
        0
    };
    if delay > 0 {
        parts.push(tr(
            lang,
            "presets.summary.delay",
            &[("seconds", &delay.to_string())],
        ));
    }
    parts.join(" · ")
}

/// The row of a preset in Preferences > Shortcuts. Its strings live as long as the
/// app: a few bytes per preset ever made.
pub fn slot_of(id: &str) -> Slot {
    static MADE: Mutex<Option<HashMap<String, Slot>>> = Mutex::new(None);
    let mut made = MADE.lock().unwrap();
    let map = made.get_or_insert_with(HashMap::new);
    if let Some(s) = map.get(id) {
        return *s;
    }
    let slot = Slot {
        id: Box::leak(format!("preset.{id}").into_boxed_str()),
        command: "preset.run",
        params: Box::leak(json!({ "id": id }).to_string().into_boxed_str()),
        scope: ShortcutScope::Global,
        group: "presets",
    };
    map.insert(id.to_string(), slot);
    slot
}

/// Tells the rest of the app that the presets changed: their shortcut rows, the
/// global shortcuts, the icon menu and the open windows.
pub fn changed(core: &Core) {
    let stored = load(core);
    let lang = core.lang();
    let rows = stored
        .presets
        .iter()
        .map(|p| (slot_of(&p.id), display_name(p, lang)))
        .collect();
    core.shortcuts.set_extra(rows);
    core.shortcuts_changed();
    core.host().editor_event("presets", json!({}));
}

/// A preset's shortcut, if it has one.
pub fn shortcut_of(core: &Core, id: &str) -> Option<Accelerator> {
    core.shortcuts.binding(&slot_of(id), core.profile())
}

/// The icon menu's Presets submenu (CAP-005): id and name of each, in order.
pub fn menu_items(core: &Core) -> Vec<(String, String)> {
    let lang = core.lang();
    load(core)
        .presets
        .iter()
        .map(|p| (p.id.clone(), display_name(p, lang)))
        .collect()
}

/// Paths a preset uses that do not exist on this computer (CAP-063): the File
/// destination's folder (it is created when capturing, but may be another computer's)
/// and the fixed file of Selection = File.
pub fn missing_paths(p: &Preset) -> Vec<String> {
    let s = &p.settings;
    let text = |v: &Value| v.as_str().map(str::trim).unwrap_or_default().to_string();
    let mut out = Vec::new();
    let sends_file = s["share"]
        .as_array()
        .is_some_and(|a| a.iter().any(|d| d == "file"));
    let folder = text(&s["file"]["folder"]);
    if sends_file && !folder.is_empty() && !std::path::Path::new(&folder).is_dir() {
        out.push(folder);
    }
    let fixed = text(&s["sourceFile"]);
    if s["selection"] == "file" && !fixed.is_empty() && !std::path::Path::new(&fixed).is_file() {
        out.push(fixed);
    }
    out
}

/// A preset as `preset.list` shows it.
pub fn to_json(core: &Core, p: &Preset, lang: Lang) -> Value {
    let platform = core.shortcuts.platform();
    let shortcut = shortcut_of(core, &p.id);
    let mut warnings = Vec::new();
    if plan(p).is_err() {
        warnings.push(json!({ "code": "settings", "message": tr(lang, "presets.warning_settings", &[]) }));
    }
    for path in missing_paths(p) {
        warnings.push(json!({
            "code": "path",
            "path": path,
            "message": tr(lang, "presets.warning_path", &[("path", &path)]),
        }));
    }
    json!({
        "id": p.id,
        "name": display_name(p, lang),
        "kind": p.kind,
        "settings": p.settings,
        "summary": summary(p, lang),
        "builtin": p.builtin,
        "shortcut": shortcut.as_ref().map(|a| a.to_string()),
        "shortcutDisplay": shortcut.as_ref().map(|a| a.display(platform)),
        "slot": slot_of(&p.id).id,
        // Its interval series runs or its schedule waits (CAP-049, CAP-050, CAP-058).
        "running": crate::interval::of_preset(core, &p.id).is_some()
            || crate::schedule::of_preset(core, &p.id).is_some(),
        "warnings": warnings,
    })
}

/// Deleted presets kept for Undo (CAP-060): where each was, it, and its shortcut.
type Deleted = Vec<(usize, Preset, Option<Option<Accelerator>>)>;
static UNDO: Mutex<Option<HashMap<String, Deleted>>> = Mutex::new(None);

pub fn keep_for_undo(deleted: Deleted) -> String {
    let id = new_id();
    UNDO.lock()
        .unwrap()
        .get_or_insert_with(HashMap::new)
        .insert(id.clone(), deleted);
    id
}

pub fn take_undo(id: &str) -> Option<Deleted> {
    UNDO.lock().unwrap().as_mut().and_then(|m| m.remove(id))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn preset(name: &str) -> Preset {
        Preset {
            id: new_id(),
            name: name.into(),
            kind: "image".into(),
            settings: json!({}),
            builtin: None,
            renamed: false,
            extra: Map::new(),
        }
    }

    /// CAP-063: paths of another computer are marked to be checked.
    #[test]
    fn paths_that_do_not_exist_are_marked() {
        let dir = tempfile::tempdir().unwrap();
        let here = dir.path().to_string_lossy().into_owned();
        let mut p = preset("Archivo");
        p.settings = json!({"share": ["file"], "file": {"folder": here}});
        assert!(missing_paths(&p).is_empty());
        let gone = dir.path().join("no-such").to_string_lossy().into_owned();
        p.settings = json!({"share": ["file"], "file": {"folder": gone}});
        assert_eq!(missing_paths(&p), vec![gone.clone()]);
        // Not sent to File: the folder does not matter.
        p.settings = json!({"share": [], "file": {"folder": gone}});
        assert!(missing_paths(&p).is_empty());
        // The default folder (empty) is never marked.
        p.settings = json!({"share": ["file"], "file": {"folder": ""}});
        assert!(missing_paths(&p).is_empty());
        let file = dir.path().join("gone.png").to_string_lossy().into_owned();
        p.settings = json!({"selection": "file", "sourceFile": file});
        assert_eq!(missing_paths(&p), vec![file]);
    }

    #[test]
    fn names_are_checked_and_made_unique() {
        assert!(clean_name("  ").is_err());
        assert!(clean_name(&"x".repeat(65)).is_err());
        assert!(clean_name("a\nb").is_err());
        assert_eq!(clean_name("  Ventana ").unwrap(), "Ventana");
        let list = vec![preset("Ventana"), preset("ventana (2)")];
        assert_eq!(unique_name(&list, "VENTANA", None, Lang::Es), "VENTANA (3)");
        assert_eq!(unique_name(&list, "Región", None, Lang::Es), "Región");
        let id = list[0].id.clone();
        assert_eq!(unique_name(&list, "Ventana", Some(&id), Lang::Es), "Ventana");
    }

    #[test]
    fn settings_are_checked_and_unknown_fields_kept() {
        let ok = checked_settings("image", &json!({"selection": "window", "iphone": {"x": 1}})).unwrap();
        assert_eq!(ok["selection"], "window");
        assert_eq!(ok["open"], "editor", "the defaults fill what is missing");
        assert_eq!(ok["iphone"], json!({"x": 1}));
        assert!(checked_settings("image", &json!({"selection": "nope"})).is_err());
        assert!(checked_settings("camera", &json!({})).is_err());
        // Video presets (VID-009): the Video tab's settings, defaults filled in.
        let v = checked_settings("video", &json!({"selection": "window", "limit": 300})).unwrap();
        assert_eq!(v, json!({"selection": "window", "limit": 300, "open": "editor"}));
        assert!(checked_settings("video", &json!({"selection": "camera"})).is_err());
        // Open in (VID-052): editor or none.
        assert!(checked_settings("video", &json!({"open": "quick"})).is_err());
        let mut p = preset("Grabar ventana");
        p.kind = "video".into();
        p.settings = v;
        let (command, params, _) = plan(&p).unwrap();
        assert_eq!(command, "record.video");
        assert_eq!(params["selection"], "window");
        assert_eq!(params["open"], "editor");
        assert_eq!(summary(&p, Lang::Es), "Video · Ventana · 5 min");
    }

    #[test]
    fn a_preset_plans_with_its_own_settings() {
        let mut p = preset("x");
        p.settings = checked_settings(
            "image",
            &json!({"selection": "fullscreen", "open": "none", "copy": true}),
        )
        .unwrap();
        let (command, params, _) = plan(&p).unwrap();
        assert_eq!(command, "capture.fullscreen");
        assert_eq!(params["open"], "none");
        assert_eq!(params["copy"], true);
        assert_eq!(
            summary(&p, Lang::Es),
            "Pantalla completa · Sin editor · Portapapeles"
        );
    }
}
