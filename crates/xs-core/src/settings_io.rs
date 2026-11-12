//! Export and import of settings (APP-011): a readable `.xsettings` JSON with a format
//! version, the app's version and the platform it came from. Groups: preferences,
//! shortcuts (with their profile), themes and saved effects, and the toolbar.
//! Capture presets (CAP-058, M3) and share destinations (SHR-, M6) join when they
//! exist. Never in the file: passwords, tokens, signed-in sessions, the library or the
//! logs (credentials live in the system keychain, not in any of these stores).

use std::collections::BTreeMap;

use serde_json::{json, Map, Value};

use crate::platform::Platform;
use crate::shortcuts::{validate, Accelerator, Profile, ShortcutScope};
use crate::{Core, Lang, Source};

pub const FORMAT: &str = "xcreenshot-settings";
pub const FORMAT_VERSION: u64 = 1;

/// The groups, in the order the dialogs show them.
pub const GROUPS: &[&str] = &["preferences", "shortcuts", "themes", "toolbar", "presets"];

/// A group whose owner lives outside the core (capture presets, CAP-059): what it
/// writes, and how it reads it back (with the file's platform) and says what changes.
pub type GroupExport = fn(&Core, Lang) -> Value;
pub type GroupImport = fn(&Core, &Value, Platform, &ImportOptions, Lang) -> Result<Value, String>;

static EXTRA_GROUPS: std::sync::Mutex<Vec<(&'static str, GroupExport, GroupImport)>> =
    std::sync::Mutex::new(Vec::new());

/// Adds a group of another crate to the file (once per name).
pub fn add_group(name: &'static str, export: GroupExport, import: GroupImport) {
    let mut groups = EXTRA_GROUPS.lock().unwrap();
    if !groups.iter().any(|(n, _, _)| *n == name) {
        groups.push((name, export, import));
    }
}

fn extra_groups() -> Vec<(&'static str, GroupExport, GroupImport)> {
    EXTRA_GROUPS.lock().unwrap().clone()
}

/// What the core keeps for the editor (written by `xs-editor`).
const THEMES_KEY: &str = "editor.themes";
const EFFECTS_KEY: &str = "effects.saved";
const TOOLBAR_KEY: &str = "editor.toolbar";
/// Settings that are not preferences of the file: the shortcut profile goes with the
/// shortcuts.
const PROFILE_KEY: &str = "shortcuts.profile";

fn wanted(groups: &[String], g: &str) -> bool {
    groups.is_empty() || groups.iter().any(|x| x == g)
}

/// The file's content for `groups` (all when empty).
pub fn export(core: &Core, groups: &[String]) -> Value {
    let mut out = Map::new();
    if wanted(groups, "preferences") {
        let mut prefs = core.settings.snapshot();
        prefs.remove(PROFILE_KEY);
        out.insert("preferences".into(), Value::Object(prefs));
    }
    if wanted(groups, "shortcuts") {
        // A preset's shortcut travels with its preset, in the presets group.
        let overrides: BTreeMap<String, Value> = core
            .shortcuts
            .overrides()
            .into_iter()
            .filter(|(k, _)| !crate::shortcuts::is_preset_key(k))
            .map(|(k, v)| (k, v.map(|a| json!(a.to_string())).unwrap_or(Value::Null)))
            .collect();
        out.insert(
            "shortcuts".into(),
            json!({
                "profile": core.settings.get(PROFILE_KEY),
                "overrides": overrides,
            }),
        );
    }
    if wanted(groups, "themes") {
        // The user's own themes and favorites; the built-in themes come with the app.
        let themes = core.recall(THEMES_KEY).unwrap_or(Value::Null);
        let own: Vec<Value> = themes["themes"]
            .as_array()
            .into_iter()
            .flatten()
            .filter(|t| t["builtIn"] != json!(true))
            .cloned()
            .collect();
        out.insert(
            "themes".into(),
            json!({
                "themes": own,
                "favorites": themes["favorites"],
                "effects": core.recall(EFFECTS_KEY),
            }),
        );
    }
    if wanted(groups, "toolbar") {
        out.insert("toolbar".into(), json!({ "items": core.recall(TOOLBAR_KEY) }));
    }
    for (name, export, _) in extra_groups() {
        if wanted(groups, name) {
            out.insert(name.into(), export(core, core.lang()));
        }
    }
    json!({
        "format": FORMAT,
        "formatVersion": FORMAT_VERSION,
        "app": crate::platform::VERSION_FULL,
        "platform": core.shortcuts.platform().as_str(),
        "exported": chrono::Local::now().to_rfc3339(),
        "groups": out,
    })
}

/// How an imported item with the name of an existing one is handled (APP-011).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OnConflict {
    Replace,
    KeepBoth,
    Skip,
}

impl OnConflict {
    pub fn parse(s: &str) -> Option<OnConflict> {
        match s {
            "replace" => Some(OnConflict::Replace),
            "keepBoth" => Some(OnConflict::KeepBoth),
            "skip" => Some(OnConflict::Skip),
            _ => None,
        }
    }
}

pub struct ImportOptions {
    pub groups: Vec<String>,
    /// Shortcuts from the other platform: converted (`Ctrl` ↔ `Cmd`…) or left as they
    /// are on this computer.
    pub convert_shortcuts: bool,
    pub on_conflict: OnConflict,
    /// Only say what would change.
    pub dry_run: bool,
}

/// A shortcut of the other platform on this one: `Ctrl` ↔ `Cmd`, `Alt` ↔ `Option`,
/// `Win` ↔ `Control`. Keys this platform does not have give `None`.
pub fn convert(acc: &Accelerator, from: Platform, to: Platform) -> Option<Accelerator> {
    if from == to {
        return Some(acc.clone());
    }
    const PC_ONLY: &[&str] = &[
        "PrintScreen",
        "Insert",
        "ScrollLock",
        "Pause",
        "NumLock",
        "ContextMenu",
    ];
    if to == Platform::Macos && PC_ONLY.contains(&acc.code.as_str()) {
        return None;
    }
    // On both sides `meta` is the Windows key / Command and `ctrl` is Control.
    Some(Accelerator {
        ctrl: acc.meta,
        meta: acc.ctrl,
        alt: acc.alt,
        shift: acc.shift,
        code: acc.code.clone(),
    })
}

/// A list kept by name: what `incoming` adds, replaces or skips, by `on_conflict`.
/// Returns the new list and how many were added, replaced and skipped.
fn merge_named(
    current: Vec<Value>,
    incoming: &[Value],
    on_conflict: OnConflict,
    new_id: &mut dyn FnMut() -> String,
) -> (Vec<Value>, [usize; 3]) {
    let mut list = current;
    let mut counts = [0, 0, 0];
    for item in incoming {
        let name = item["name"].as_str().unwrap_or_default().to_string();
        let same = list
            .iter()
            .position(|x| x["name"].as_str().is_some_and(|n| n.eq_ignore_ascii_case(&name)));
        // The very same item (an export imported back): nothing to do.
        let without_id = |v: &Value| {
            let mut v = v.clone();
            if let Some(o) = v.as_object_mut() {
                o.remove("id");
            }
            v
        };
        if same.is_some_and(|i| without_id(&list[i]) == without_id(item)) {
            continue;
        }
        match (same, on_conflict) {
            (Some(i), OnConflict::Replace) => {
                let mut v = item.clone();
                v["id"] = list[i]["id"].clone();
                list[i] = v;
                counts[1] += 1;
            }
            (Some(_), OnConflict::Skip) => counts[2] += 1,
            (Some(_), OnConflict::KeepBoth) => {
                let mut v = item.clone();
                let mut n = 2;
                let free = loop {
                    let candidate = format!("{name} ({n})");
                    if !list.iter().any(|x| x["name"] == json!(candidate)) {
                        break candidate;
                    }
                    n += 1;
                };
                v["name"] = json!(free);
                v["id"] = json!(new_id());
                list.push(v);
                counts[0] += 1;
            }
            (None, _) => {
                let mut v = item.clone();
                if list.iter().any(|x| x["id"] == v["id"]) {
                    v["id"] = json!(new_id());
                }
                list.push(v);
                counts[0] += 1;
            }
        }
    }
    (list, counts)
}

/// Reads `file`, applies the chosen groups (unless `dry_run`) and says, by group, what
/// changes and what was left out.
pub fn import(core: &Core, file: &Value, opts: &ImportOptions, lang: Lang) -> Result<Value, String> {
    if file["format"] != json!(FORMAT) {
        return Err("not an XcreenShot settings file".into());
    }
    if file["formatVersion"].as_u64().unwrap_or(0) > FORMAT_VERSION {
        return Err("made by a newer XcreenShot".into());
    }
    let here = core.shortcuts.platform();
    let from = match file["platform"].as_str() {
        Some("macos") => Platform::Macos,
        _ => Platform::Windows,
    };
    let groups = &file["groups"];
    let mut summary = Map::new();

    if wanted(&opts.groups, "preferences") {
        if let Some(prefs) = groups["preferences"].as_object() {
            let (mut changes, mut other, mut invalid) = (Vec::new(), Vec::new(), Vec::new());
            for (key, value) in prefs {
                if key == PROFILE_KEY {
                    continue;
                }
                match core.settings.def(key) {
                    None => other.push(key.clone()),
                    Some(_) if core.settings.get(key).as_ref() == Some(value) => {}
                    Some(_) => {
                        if opts.dry_run {
                            changes.push(key.clone());
                            continue;
                        }
                        match core.settings.set(key, value.clone(), Source::Ui, lang) {
                            Ok(true) => {
                                changes.push(key.clone());
                                core.setting_changed(key, value);
                            }
                            Ok(false) => {}
                            // A setting of the other platform, or a value this build
                            // does not take: kept in the file, not applied.
                            Err(_) => invalid.push(key.clone()),
                        }
                    }
                }
            }
            summary.insert(
                "preferences".into(),
                json!({ "changes": changes, "notApplied": other.into_iter().chain(invalid).collect::<Vec<_>>() }),
            );
        }
    }

    if wanted(&opts.groups, "shortcuts") {
        if let Some(sc) = groups["shortcuts"].as_object() {
            let cross = from != here;
            let mut changes = Vec::new();
            let mut unconvertible = Vec::new();
            let mut conflicts = Vec::new();
            if cross && !opts.convert_shortcuts {
                summary.insert(
                    "shortcuts".into(),
                    json!({ "changes": [], "kept": true, "unconvertible": [], "conflicts": [] }),
                );
            } else {
                let profile = sc.get("profile").filter(|p| p.is_string());
                if let Some(p) = profile {
                    if core.settings.get(PROFILE_KEY).as_ref() != Some(p) {
                        changes.push(PROFILE_KEY.to_string());
                        if !opts.dry_run
                            && core
                                .settings
                                .set(PROFILE_KEY, p.clone(), Source::Ui, lang)
                                .is_ok()
                        {
                            core.setting_changed(PROFILE_KEY, p);
                        }
                    }
                }
                let profile = core
                    .settings
                    .get(PROFILE_KEY)
                    .and_then(|v| v.as_str().and_then(Profile::parse))
                    .unwrap_or(Profile::Coexistence);
                let mut wanted_map: BTreeMap<String, Option<Accelerator>> = BTreeMap::new();
                for (key, v) in sc
                    .get("overrides")
                    .and_then(Value::as_object)
                    .into_iter()
                    .flatten()
                {
                    let Some(slot) = core.shortcuts.slot_by_key(key) else {
                        continue;
                    };
                    let acc = match v.as_str() {
                        None => None,
                        Some(text) => match text
                            .parse::<Accelerator>()
                            .ok()
                            .and_then(|a| convert(&a, from, here))
                        {
                            Some(a) if validate(&a, slot.scope, here).is_ok() => Some(a),
                            _ => {
                                unconvertible.push(json!({ "slot": key, "shortcut": text }));
                                wanted_map.insert(key.clone(), None);
                                continue;
                            }
                        },
                    };
                    if let Some(a) = &acc {
                        // Two imported shortcuts, or one and an existing one, on the
                        // same keys: the later one stays unassigned (APP-019).
                        let clash = wanted_map.iter().any(|(k, other)| {
                            other.as_ref() == Some(a)
                                && core.shortcuts.slot_by_key(k).is_some_and(|o| {
                                    o.scope == slot.scope
                                        || o.scope == ShortcutScope::Global
                                        || slot.scope == ShortcutScope::Global
                                })
                        });
                        if clash {
                            conflicts.push(json!({ "slot": key, "shortcut": a.to_string() }));
                            wanted_map.insert(key.clone(), None);
                            continue;
                        }
                    }
                    wanted_map.insert(key.clone(), acc);
                }
                for (key, acc) in &wanted_map {
                    if let Some(slot) = core.shortcuts.slot_by_key(key) {
                        if core.shortcuts.binding(&slot, profile) != *acc {
                            changes.push(key.clone());
                        }
                    }
                }
                if !opts.dry_run {
                    core.shortcuts.replace_overrides(&wanted_map, profile);
                    core.shortcuts_changed();
                }
                summary.insert(
                    "shortcuts".into(),
                    json!({ "changes": changes, "kept": false, "unconvertible": unconvertible, "conflicts": conflicts }),
                );
            }
        }
    }

    if wanted(&opts.groups, "themes") {
        let t = &groups["themes"];
        if t.is_object() {
            let mut stored = core.recall(THEMES_KEY).unwrap_or(Value::Null);
            let current: Vec<Value> = stored["themes"].as_array().cloned().unwrap_or_default();
            let mut next = stored["nextId"].as_u64().unwrap_or(1);
            let mut new_id = || {
                next += 1;
                format!("t{next}")
            };
            let incoming: Vec<Value> = t["themes"].as_array().cloned().unwrap_or_default();
            let (themes, theme_counts) = merge_named(current, &incoming, opts.on_conflict, &mut new_id);
            let effects_now: Vec<Value> = core
                .recall(EFFECTS_KEY)
                .and_then(|v| v.as_array().cloned())
                .unwrap_or_default();
            let mut effect_n = effects_now.len() as u64;
            let mut effect_id = || {
                effect_n += 1;
                format!("imported-{}-{effect_n}", chrono::Local::now().timestamp_millis())
            };
            let effects_in: Vec<Value> = t["effects"].as_array().cloned().unwrap_or_default();
            let (effects, effect_counts) =
                merge_named(effects_now, &effects_in, opts.on_conflict, &mut effect_id);
            let favorites_now: Vec<Value> = stored["favorites"].as_array().cloned().unwrap_or_default();
            let favorites_in: Vec<Value> = t["favorites"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter(|f| {
                    !favorites_now
                        .iter()
                        .any(|x| x["tool"] == f["tool"] && x["style"] == f["style"])
                })
                .collect();
            if !opts.dry_run {
                if theme_counts[0] + theme_counts[1] > 0 || !favorites_in.is_empty() {
                    if stored.is_null() {
                        stored = json!({ "themes": [], "active": "base", "favorites": [], "nextId": 1 });
                    }
                    stored["themes"] = json!(themes);
                    stored["nextId"] = json!(next);
                    let mut favs = favorites_now.clone();
                    favs.extend(favorites_in.iter().cloned());
                    stored["favorites"] = json!(favs);
                    core.remember(THEMES_KEY, stored);
                    core.memory_changed(THEMES_KEY);
                }
                if effect_counts[0] + effect_counts[1] > 0 {
                    core.remember(EFFECTS_KEY, json!(effects));
                    core.memory_changed(EFFECTS_KEY);
                }
            }
            summary.insert(
                "themes".into(),
                json!({
                    "themes": { "added": theme_counts[0], "replaced": theme_counts[1], "skipped": theme_counts[2] },
                    "effects": { "added": effect_counts[0], "replaced": effect_counts[1], "skipped": effect_counts[2] },
                    "favorites": favorites_in.len(),
                }),
            );
        }
    }

    if wanted(&opts.groups, "toolbar") {
        let items = &groups["toolbar"]["items"];
        if groups["toolbar"].is_object() {
            let changes = core.recall(TOOLBAR_KEY).as_ref() != Some(items) && !items.is_null();
            if !opts.dry_run && changes {
                core.remember(TOOLBAR_KEY, items.clone());
                core.memory_changed(TOOLBAR_KEY);
            }
            summary.insert("toolbar".into(), json!({ "changes": changes }));
        }
    }

    for (name, _, import) in extra_groups() {
        if wanted(&opts.groups, name) && groups[name].is_object() {
            summary.insert(name.into(), import(core, &groups[name], from, opts, lang)?);
        }
    }

    Ok(json!({
        "platform": from.as_str(),
        "samePlatform": from == here,
        "app": file["app"],
        "dryRun": opts.dry_run,
        "groups": summary,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Source;

    fn call(core: &Core, id: &str, p: Value) -> Value {
        core.call(Source::Cli, id, p, Some(Lang::En))
            .unwrap_or_else(|e| panic!("{id}: {e:?}"))
    }

    fn opts(groups: &[&str]) -> ImportOptions {
        ImportOptions {
            groups: groups.iter().map(|g| g.to_string()).collect(),
            convert_shortcuts: true,
            on_conflict: OnConflict::KeepBoth,
            dry_run: false,
        }
    }

    /// APP-011: what one computer exports, a fresh one takes whole: preferences,
    /// shortcuts, themes and the toolbar; no secret goes in the file.
    #[test]
    fn settings_travel_whole_to_a_fresh_computer() {
        let a = Core::for_tests(Platform::Windows);
        call(
            &a,
            "settings.set",
            json!({"key": "general.theme", "value": "dark"}),
        );
        call(&a, "settings.set", json!({"key": "tray.capacity", "value": 20}));
        call(
            &a,
            "shortcuts.set",
            json!({"command": "capture.image", "scope": "global", "shortcut": "Ctrl+Alt+Shift+KeyJ"}),
        );
        a.remember(TOOLBAR_KEY, json!(["move", "arrow", "sep", "text"]));
        a.remember(
            THEMES_KEY,
            json!({"themes": [{"id": "t5", "name": "Cliente", "colors": ["#112233", "#445566"], "shadow": false}], "active": "base", "favorites": [], "nextId": 5}),
        );
        a.remember(
            EFFECTS_KEY,
            json!([{"id": "e1", "name": "Mi marco", "stack": []}]),
        );
        let file = export(&a, &[]);
        let text = serde_json::to_string(&file).unwrap();
        for secret in ["apiKey", "token", "password", "library.location"] {
            assert!(!text.contains(secret), "no {secret} in the file");
        }
        assert_eq!(file["format"], FORMAT);
        assert_eq!(file["platform"], "windows");

        let b = Core::for_tests(Platform::Windows);
        let r = import(&b, &file, &opts(&[]), Lang::En).unwrap();
        assert!(
            r["groups"]["preferences"]["changes"].as_array().unwrap().len() >= 2,
            "{r}"
        );
        assert_eq!(b.settings.get("general.theme"), Some(json!("dark")));
        assert_eq!(b.settings.get("tray.capacity"), Some(json!(20)));
        let slot = b.shortcuts.slot_by_key("capture.image@global").unwrap();
        assert_eq!(
            b.shortcuts
                .binding(&slot, Profile::Coexistence)
                .map(|a| a.to_string()),
            Some("Ctrl+Alt+Shift+KeyJ".to_string())
        );
        assert_eq!(
            b.recall(TOOLBAR_KEY),
            Some(json!(["move", "arrow", "sep", "text"]))
        );
        assert_eq!(b.recall(THEMES_KEY).unwrap()["themes"][0]["name"], "Cliente");
        assert_eq!(b.recall(EFFECTS_KEY).unwrap()[0]["name"], "Mi marco");
    }

    /// APP-011: only the shortcuts group changes nothing else; a dry run changes
    /// nothing at all.
    #[test]
    fn importing_one_group_leaves_the_rest() {
        let a = Core::for_tests(Platform::Windows);
        call(
            &a,
            "settings.set",
            json!({"key": "general.theme", "value": "dark"}),
        );
        call(
            &a,
            "shortcuts.set",
            json!({"command": "capture.image", "scope": "global", "shortcut": "Ctrl+Alt+Shift+KeyJ"}),
        );
        let file = export(&a, &[]);
        let b = Core::for_tests(Platform::Windows);
        let dry = ImportOptions {
            dry_run: true,
            ..opts(&[])
        };
        let r = import(&b, &file, &dry, Lang::En).unwrap();
        assert!(r["groups"]["shortcuts"]["changes"]
            .as_array()
            .unwrap()
            .contains(&json!("capture.image@global")));
        assert_eq!(
            b.settings.get("general.theme"),
            Some(json!("system")),
            "a dry run changes nothing"
        );
        import(&b, &file, &opts(&["shortcuts"]), Lang::En).unwrap();
        assert_eq!(
            b.settings.get("general.theme"),
            Some(json!("system")),
            "only the shortcuts"
        );
        let slot = b.shortcuts.slot_by_key("capture.image@global").unwrap();
        assert!(b
            .shortcuts
            .binding(&slot, Profile::Coexistence)
            .is_some_and(|a| a.to_string() == "Ctrl+Alt+Shift+KeyJ"));
    }

    /// APP-011: from Windows to macOS, Ctrl becomes Cmd; PrintScreen has no key there
    /// and stays unassigned, listed; keeping this computer's shortcuts is an option.
    #[test]
    fn shortcuts_convert_between_platforms() {
        let win = Core::for_tests(Platform::Windows);
        call(
            &win,
            "shortcuts.set",
            json!({"command": "capture.image", "scope": "global", "shortcut": "Ctrl+Alt+PrintScreen"}),
        );
        call(
            &win,
            "shortcuts.set",
            json!({"command": "capture.universal", "scope": "global", "shortcut": "Ctrl+Alt+Shift+KeyJ"}),
        );
        let file = export(&win, &["shortcuts".to_string()]);
        let mac = Core::for_tests(Platform::Macos);
        let r = import(&mac, &file, &opts(&[]), Lang::En).unwrap();
        let un = r["groups"]["shortcuts"]["unconvertible"].as_array().unwrap();
        assert!(un.iter().any(|u| u["slot"] == "capture.image@global"), "{r}");
        let image = mac.shortcuts.slot_by_key("capture.image@global").unwrap();
        assert_eq!(
            mac.shortcuts.binding(&image, Profile::Coexistence),
            None,
            "left unassigned"
        );
        let universal = mac.shortcuts.slot_by_key("capture.universal@global").unwrap();
        assert_eq!(
            mac.shortcuts
                .binding(&universal, Profile::Coexistence)
                .map(|a| a.to_string()),
            Some("Alt+Shift+Super+KeyJ".to_string())
        );
        let other = Core::for_tests(Platform::Macos);
        let keep = ImportOptions {
            convert_shortcuts: false,
            ..opts(&[])
        };
        let r = import(&other, &file, &keep, Lang::En).unwrap();
        assert_eq!(r["groups"]["shortcuts"]["kept"], json!(true));
        assert!(other.shortcuts.overrides().is_empty());
    }

    /// APP-011: a theme with the name of one already here: keep both, skip or replace.
    #[test]
    fn themes_with_the_same_name() {
        let theme = |colors: &str| json!({"id": "t2", "name": "Cliente", "colors": [colors, "#FFFFFF"], "shadow": false});
        let file = json!({"format": FORMAT, "formatVersion": 1, "platform": "windows", "groups": {"themes": {"themes": [theme("#000000")], "favorites": [], "effects": []}}});
        let here = |core: &Core| {
            core.remember(
                THEMES_KEY,
                json!({"themes": [theme("#FF0000")], "active": "t2", "favorites": [], "nextId": 2}),
            );
        };
        let names = |core: &Core| -> Vec<String> {
            core.recall(THEMES_KEY).unwrap()["themes"]
                .as_array()
                .unwrap()
                .iter()
                .map(|t| t["name"].as_str().unwrap().to_string())
                .collect()
        };
        let c = Core::for_tests(Platform::Windows);
        here(&c);
        import(&c, &file, &opts(&[]), Lang::En).unwrap();
        assert_eq!(names(&c), ["Cliente", "Cliente (2)"]);
        let c = Core::for_tests(Platform::Windows);
        here(&c);
        import(
            &c,
            &file,
            &ImportOptions {
                on_conflict: OnConflict::Skip,
                ..opts(&[])
            },
            Lang::En,
        )
        .unwrap();
        assert_eq!(names(&c), ["Cliente"]);
        assert_eq!(c.recall(THEMES_KEY).unwrap()["themes"][0]["colors"][0], "#FF0000");
        let c = Core::for_tests(Platform::Windows);
        here(&c);
        import(
            &c,
            &file,
            &ImportOptions {
                on_conflict: OnConflict::Replace,
                ..opts(&[])
            },
            Lang::En,
        )
        .unwrap();
        assert_eq!(c.recall(THEMES_KEY).unwrap()["themes"][0]["colors"][0], "#000000");
        assert!(import(&c, &json!({"format": "other"}), &opts(&[]), Lang::En).is_err());
        // An export imported back on the same computer adds nothing.
        let mine = export(&c, &["themes".to_string()]);
        let r = import(&c, &mine, &opts(&[]), Lang::En).unwrap();
        assert_eq!(r["groups"]["themes"]["themes"]["added"], json!(0), "{r}");
        assert_eq!(names(&c), ["Cliente"]);
    }
}
