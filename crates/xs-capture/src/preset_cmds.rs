//! The `preset.*` commands (CAP-058 to CAP-064, APP-030).

use std::path::PathBuf;
use std::sync::{Arc, Weak};

use serde_json::{json, Value};
use xs_core::command::Registry;
use xs_core::i18n::tr;
use xs_core::platform::Platform;
use xs_core::settings_io::convert;
use xs_core::shortcuts::Accelerator;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode, Lang};

use crate::panel;
use crate::presets::{self, Preset, Stored};

/// `.xpreset` format (CAP-063).
pub const FORMAT: &str = "xcreenshot-presets";
pub const FORMAT_VERSION: u64 = 1;

fn bad(lang: Lang, param: &str, reason: &str) -> CommandError {
    CommandError::new(
        ErrorCode::InvalidParams,
        tr(
            lang,
            "errors.invalid_param",
            &[("param", param), ("reason", reason)],
        ),
    )
}

fn not_found(lang: Lang, what: &str) -> CommandError {
    CommandError::new(
        ErrorCode::NotFound,
        tr(lang, "errors.preset_not_found", &[("name", what)]),
    )
}

fn upgrade(weak: &Weak<Core>) -> Result<Arc<Core>, CommandError> {
    weak.upgrade()
        .ok_or_else(|| CommandError::internal("shutting down"))
}

/// The preset named by `id`, or by its name (`name`, any case).
fn find(stored: &Stored, p: &Value, lang: Lang) -> Result<usize, CommandError> {
    if let Some(id) = p.get("id").and_then(Value::as_str) {
        return stored
            .presets
            .iter()
            .position(|x| x.id == id)
            .ok_or_else(|| not_found(lang, id));
    }
    if let Some(name) = p.get("name").and_then(Value::as_str) {
        let wanted = name.trim().to_lowercase();
        return stored
            .presets
            .iter()
            .position(|x| {
                presets::display_name(x, lang).to_lowercase() == wanted || x.name.to_lowercase() == wanted
            })
            .ok_or_else(|| not_found(lang, name));
    }
    Err(bad(lang, "id", "give the preset's id or name"))
}

fn listed(core: &Core, lang: Lang) -> Value {
    let stored = presets::load(core);
    let list: Vec<Value> = stored
        .presets
        .iter()
        .map(|p| presets::to_json(core, p, lang))
        .collect();
    json!({ "presets": list })
}

/// The settings a new or updated preset takes: the panel's active tab, or given.
fn settings_from(
    core: &Core,
    p: &Value,
    kind: &str,
    base: &Value,
    lang: Lang,
) -> Result<Value, CommandError> {
    if p.get("fromCurrent").and_then(Value::as_bool) == Some(true) {
        let panel = panel::load(core);
        if panel.tab != kind {
            return Err(bad(
                lang,
                "fromCurrent",
                "the panel's active tab is of another kind",
            ));
        }
        let tab = if kind == "universal" {
            serde_json::to_value(&panel.universal)
        } else {
            serde_json::to_value(&panel.image)
        }
        .unwrap_or_default();
        return presets::checked_settings(kind, &tab).map_err(|e| bad(lang, "settings", &e));
    }
    let mut merged = base.clone();
    if let (Some(m), Some(changes)) = (
        merged.as_object_mut(),
        p.get("settings").and_then(Value::as_object),
    ) {
        for (k, v) in changes {
            m.insert(k.clone(), v.clone());
        }
    }
    presets::checked_settings(kind, &merged).map_err(|e| bad(lang, "settings", &e))
}

fn create(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let from_current = p.get("fromCurrent").and_then(Value::as_bool) == Some(true);
    let kind = if from_current {
        panel::load(core).tab
    } else {
        p.get("kind")
            .and_then(Value::as_str)
            .unwrap_or("image")
            .to_string()
    };
    if !presets::KINDS.contains(&kind.as_str()) {
        return Err(bad(lang, "kind", "universal or image"));
    }
    let settings = settings_from(core, p, &kind, &json!({}), lang)?;
    let mut stored = presets::load(core);
    let wanted = match p.get("name").and_then(Value::as_str) {
        Some(n) => presets::clean_name(n).map_err(|e| bad(lang, "name", &e))?,
        None => presets::proposed_name(&kind, &settings, lang),
    };
    let name = presets::unique_name(&stored.presets, &wanted, None, lang);
    let preset = Preset {
        id: presets::new_id(),
        name,
        kind,
        settings,
        builtin: None,
        renamed: false,
        extra: Default::default(),
    };
    stored.presets.push(preset.clone());
    presets::save(core, &stored);
    presets::changed(core);
    Ok(presets::to_json(core, &preset, lang))
}

fn update(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let mut stored = presets::load(core);
    let i = find(&stored, p, lang)?;
    let current = stored.presets[i].clone();
    let settings = settings_from(core, p, &current.kind, &current.settings, lang)?;
    stored.presets[i].settings = settings;
    let preset = stored.presets[i].clone();
    presets::save(core, &stored);
    presets::changed(core);
    Ok(presets::to_json(core, &preset, lang))
}

fn rename(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let mut stored = presets::load(core);
    let i = find(&stored, &json!({ "id": p["id"] }), lang)?;
    let wanted = presets::clean_name(p["to"].as_str().or(p["newName"].as_str()).unwrap_or_default())
        .map_err(|e| bad(lang, "to", &e))?;
    let id = stored.presets[i].id.clone();
    let name = presets::unique_name(&stored.presets, &wanted, Some(&id), lang);
    stored.presets[i].name = name;
    stored.presets[i].renamed = true;
    let preset = stored.presets[i].clone();
    presets::save(core, &stored);
    presets::changed(core);
    Ok(presets::to_json(core, &preset, lang))
}

fn duplicate(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let mut stored = presets::load(core);
    let i = find(&stored, p, lang)?;
    let original = stored.presets[i].clone();
    let wanted = tr(
        lang,
        "presets.copy_name",
        &[("name", &presets::display_name(&original, lang))],
    );
    let name = presets::unique_name(&stored.presets, &wanted, None, lang);
    // The copy has no shortcut, so the two never collide (CAP-060).
    let copy = Preset {
        id: presets::new_id(),
        name,
        builtin: None,
        renamed: true,
        ..original
    };
    stored.presets.insert(i + 1, copy.clone());
    presets::save(core, &stored);
    presets::changed(core);
    Ok(presets::to_json(core, &copy, lang))
}

fn delete(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let ids: Vec<String> = match p.get("ids").and_then(Value::as_array) {
        Some(a) => a.iter().filter_map(Value::as_str).map(str::to_string).collect(),
        None => p
            .get("id")
            .and_then(Value::as_str)
            .map(|s| vec![s.to_string()])
            .unwrap_or_default(),
    };
    if ids.is_empty() {
        return Err(bad(lang, "ids", "give the presets to delete"));
    }
    let mut stored = presets::load(core);
    if let Some(missing) = ids.iter().find(|id| !stored.presets.iter().any(|x| &x.id == *id)) {
        return Err(not_found(lang, missing));
    }
    let mut deleted = Vec::new();
    for (index, preset) in stored.presets.iter().enumerate() {
        if ids.contains(&preset.id) {
            let key = presets::slot_of(&preset.id).key();
            let shortcut = core.shortcuts.forget(&key);
            deleted.push((index, preset.clone(), shortcut));
        }
    }
    stored.presets.retain(|x| !ids.contains(&x.id));
    presets::save(core, &stored);
    presets::changed(core);
    let count = deleted.len();
    let undo = presets::keep_for_undo(deleted);
    Ok(json!({ "deleted": count, "undo": undo }))
}

fn restore(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let undo = p["undo"].as_str().unwrap_or_default();
    let Some(deleted) = presets::take_undo(undo) else {
        return Err(not_found(lang, undo));
    };
    let mut stored = presets::load(core);
    for (index, preset, shortcut) in &deleted {
        let at = (*index).min(stored.presets.len());
        stored.presets.insert(at, preset.clone());
        if let Some(acc) = shortcut {
            core.shortcuts
                .put_back(&presets::slot_of(&preset.id).key(), acc.clone());
        }
    }
    presets::save(core, &stored);
    presets::changed(core);
    Ok(json!({ "restored": deleted.len() }))
}

fn move_to(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let mut stored = presets::load(core);
    let i = find(&stored, p, lang)?;
    let index = p["index"]
        .as_u64()
        .ok_or_else(|| bad(lang, "index", "give the new place, 0 for the first"))? as usize;
    let preset = stored.presets.remove(i);
    let at = index.min(stored.presets.len());
    stored.presets.insert(at, preset);
    presets::save(core, &stored);
    presets::changed(core);
    Ok(json!({ "index": at }))
}

fn restore_builtins(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let reset = p.get("reset").and_then(Value::as_bool).unwrap_or(false);
    let mut stored = presets::load(core);
    let mut restored = Vec::new();
    let mut modified = Vec::new();
    for (id, kind, settings) in presets::builtins() {
        let original = presets::checked_settings(kind, &settings).unwrap_or(settings);
        match stored
            .presets
            .iter()
            .position(|x| x.builtin.as_deref() == Some(id))
        {
            None => {
                stored.presets.push(Preset {
                    id: presets::new_id(),
                    name: id.to_string(),
                    kind: kind.to_string(),
                    settings: original,
                    builtin: Some(id.to_string()),
                    renamed: false,
                    extra: Default::default(),
                });
                restored.push(tr(lang, &format!("presets.builtin.{id}"), &[]));
            }
            Some(i) => {
                let x = &mut stored.presets[i];
                if !presets::same_settings(kind, &x.settings, &original) || x.renamed {
                    modified.push(presets::display_name(x, lang));
                    if reset {
                        x.settings = original;
                        x.renamed = false;
                    }
                }
            }
        }
    }
    presets::save(core, &stored);
    presets::changed(core);
    Ok(json!({ "restored": restored, "modified": modified, "reset": reset }))
}

fn run(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let stored = presets::load(core);
    let i = find(&stored, p, lang)?;
    let preset = &stored.presets[i];
    let (command, params, extra) = presets::plan(preset).map_err(|e| {
        CommandError::new(
            ErrorCode::InvalidParams,
            tr(
                lang,
                "errors.preset_unusable",
                &[("name", &presets::display_name(preset, lang)), ("reason", &e)],
            ),
        )
        .with_details(json!({ "id": preset.id, "reason": "settings" }))
    })?;
    crate::dispatch(core, ctx, command, &params, extra)
}

/// The `.xpreset` file of some presets (CAP-063): no destination's secrets ever.
pub fn export_file(core: &Core, ids: Option<&[String]>, lang: Lang) -> Value {
    let stored = presets::load(core);
    let list: Vec<Value> = stored
        .presets
        .iter()
        .filter(|p| ids.is_none_or(|ids| ids.contains(&p.id)))
        .enumerate()
        .map(|(order, p)| {
            json!({
                "name": presets::display_name(p, lang),
                "kind": p.kind,
                "settings": p.settings,
                "order": order,
                "builtin": p.builtin,
                // By physical key, as Preferences keeps it (APP-017).
                "shortcut": presets::shortcut_of(core, &p.id).map(|a| a.to_string()),
            })
        })
        .collect();
    json!({
        "format": FORMAT,
        "formatVersion": FORMAT_VERSION,
        "app": env!("CARGO_PKG_VERSION"),
        "platform": Platform::current().as_str(),
        "exported": chrono::Local::now().to_rfc3339(),
        "presets": list,
    })
}

fn export(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let ids: Option<Vec<String>> = p
        .get("ids")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect());
    let file = export_file(core, ids.as_deref(), lang);
    let count = file["presets"].as_array().map_or(0, Vec::len);
    match p.get("out").and_then(Value::as_str) {
        Some(out) => {
            let path = PathBuf::from(out);
            let bytes = serde_json::to_vec_pretty(&file).unwrap_or_default();
            xs_core::paths::write_atomic(&path, &bytes).map_err(|e| CommandError::internal(e.to_string()))?;
            Ok(json!({ "path": path, "count": count }))
        }
        None => Ok(json!({ "file": file, "count": count })),
    }
}

/// One preset of a file, as read (CAP-063).
struct Incoming {
    name: String,
    kind: String,
    settings: Value,
    shortcut: Option<Accelerator>,
    /// Why it cannot be imported, if it cannot.
    problem: Option<String>,
}

/// Reads an `.xpreset`: nothing at all if it is damaged, newer or not one.
pub fn read_file(path: &std::path::Path, lang: Lang) -> Result<(Platform, Vec<Value>), CommandError> {
    let unreadable = |reason: &str| {
        CommandError::new(
            ErrorCode::InvalidParams,
            tr(
                lang,
                "errors.preset_file",
                &[("name", &path.to_string_lossy()), ("reason", reason)],
            ),
        )
    };
    let bytes = std::fs::read(path).map_err(|e| unreadable(&e.to_string()))?;
    let file: Value = serde_json::from_slice(&bytes).map_err(|_| unreadable("not a presets file"))?;
    if file["format"] != FORMAT {
        return Err(unreadable("not a presets file"));
    }
    let version = file["formatVersion"].as_u64().unwrap_or(0);
    if version > FORMAT_VERSION {
        return Err(unreadable("made by a newer XcreenShot"));
    }
    let platform = match file["platform"].as_str() {
        Some("macos") => Platform::Macos,
        _ => Platform::Windows,
    };
    let list = file["presets"]
        .as_array()
        .cloned()
        .ok_or_else(|| unreadable("it has no presets"))?;
    Ok((platform, list))
}

/// The presets group of `.xsettings` (APP-011, CAP-059): the same content as an
/// `.xpreset` file.
fn settings_export(core: &Core, lang: Lang) -> Value {
    export_file(core, None, lang)
}

/// Reads the presets group back: names that exist follow the file's choice (keep both,
/// replace or skip); `changes` lists the presets that would be added or replaced.
fn settings_import(
    core: &Core,
    group: &Value,
    from: Platform,
    opts: &xs_core::settings_io::ImportOptions,
    lang: Lang,
) -> Result<Value, String> {
    use xs_core::settings_io::OnConflict;
    let list = group["presets"].as_array().cloned().unwrap_or_default();
    let on_conflict = match opts.on_conflict {
        OnConflict::Replace => "replace",
        OnConflict::Skip => "skip",
        OnConflict::KeepBoth => "keepBoth",
    };
    let incoming = incoming_of(
        vec![(from, list)],
        opts.convert_shortcuts,
        core.shortcuts.platform(),
    );
    let stored = presets::load(core);
    let same = |x: &Incoming| {
        stored.presets.iter().any(|p| {
            presets::display_name(p, lang).to_lowercase() == x.name.to_lowercase()
                && p.kind == x.kind
                && presets::same_settings(&p.kind, &p.settings, &x.settings)
        })
    };
    // What is here as the file has it is not imported again.
    let fresh: Vec<Incoming> = incoming.into_iter().filter(|x| !same(x)).collect();
    let changes: Vec<String> = fresh
        .iter()
        .filter(|x| x.problem.is_none())
        .map(|x| x.name.clone())
        .collect();
    if opts.dry_run {
        return Ok(json!({ "changes": changes }));
    }
    let r = apply(core, fresh, on_conflict, None, lang);
    Ok(json!({ "changes": changes, "imported": r["imported"], "withoutShortcut": r["withoutShortcut"] }))
}

/// The presets of the files' lists, checked, with their shortcuts for this platform.
fn incoming_of(lists: Vec<(Platform, Vec<Value>)>, convert_keys: bool, here: Platform) -> Vec<Incoming> {
    let mut incoming = Vec::new();
    for (from, list) in lists {
        for v in list {
            let name = v["name"].as_str().map(str::trim).unwrap_or_default().to_string();
            let kind = v["kind"].as_str().unwrap_or_default().to_string();
            let settings = presets::checked_settings(&kind, &v["settings"]);
            let shortcut = v["shortcut"]
                .as_str()
                .and_then(|s| s.parse::<Accelerator>().ok())
                .and_then(|a| {
                    if convert_keys {
                        convert(&a, from, here)
                    } else {
                        Some(a)
                    }
                });
            let problem = match (&settings, presets::clean_name(&name)) {
                (Err(e), _) => Some(e.clone()),
                (_, Err(e)) => Some(e),
                _ => None,
            };
            incoming.push(Incoming {
                name,
                kind,
                settings: settings.unwrap_or_default(),
                shortcut,
                problem,
            });
        }
    }
    incoming
}

fn import(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let paths: Vec<PathBuf> = p
        .get("paths")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(PathBuf::from)
        .collect();
    if paths.is_empty() {
        return Err(bad(lang, "paths", "give one or more .xpreset files"));
    }
    let dry_run = p.get("dryRun").and_then(Value::as_bool).unwrap_or(false);
    let convert_keys = p.get("convertShortcuts").and_then(Value::as_bool).unwrap_or(true);
    let on_conflict = p.get("onConflict").and_then(Value::as_str).unwrap_or("keepBoth");
    if !["keepBoth", "replace", "skip"].contains(&on_conflict) {
        return Err(bad(lang, "onConflict", "keepBoth, replace or skip"));
    }
    let only: Option<Vec<String>> = p
        .get("only")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect());
    let here = core.shortcuts.platform();
    // Every file first: a damaged one imports nothing at all.
    let mut lists = Vec::new();
    for path in &paths {
        lists.push(read_file(path, lang)?);
    }
    let incoming = incoming_of(lists, convert_keys, here);
    let stored = presets::load(core);
    let profile = core.profile();
    let taken = |acc: &Accelerator| core.shortcuts.bindings(profile).iter().any(|(_, a)| a == acc);
    if dry_run {
        let rows: Vec<Value> = incoming
            .iter()
            .map(|x| {
                json!({
                    "name": x.name,
                    "kind": x.kind,
                    "exists": stored.presets.iter().any(|p| presets::display_name(p, lang).to_lowercase() == x.name.to_lowercase()),
                    "shortcut": x.shortcut.as_ref().map(|a| a.display(here)),
                    "shortcutTaken": x.shortcut.as_ref().is_some_and(taken),
                    "problem": x.problem,
                })
            })
            .collect();
        return Ok(json!({ "dryRun": true, "presets": rows }));
    }
    Ok(apply(core, incoming, on_conflict, only.as_deref(), lang))
}

/// Adds (or replaces) the incoming presets and gives them their shortcuts when free.
fn apply(
    core: &Core,
    incoming: Vec<Incoming>,
    on_conflict: &str,
    only: Option<&[String]>,
    lang: Lang,
) -> Value {
    let mut stored = presets::load(core);
    let profile = core.profile();
    let taken = |acc: &Accelerator| core.shortcuts.bindings(profile).iter().any(|(_, a)| a == acc);
    let mut imported = Vec::new();
    let mut skipped = Vec::new();
    let mut without_shortcut = Vec::new();
    let mut shortcuts: Vec<(String, Accelerator)> = Vec::new();
    for x in incoming {
        if only.is_some_and(|o| !o.iter().any(|n| n.to_lowercase() == x.name.to_lowercase())) {
            continue;
        }
        if let Some(problem) = x.problem {
            skipped.push(json!({ "name": x.name, "reason": problem }));
            continue;
        }
        let existing = stored
            .presets
            .iter()
            .position(|p| presets::display_name(p, lang).to_lowercase() == x.name.to_lowercase());
        let id = match (existing, on_conflict) {
            (Some(_), "skip") => {
                skipped.push(json!({ "name": x.name, "reason": "exists" }));
                continue;
            }
            (Some(i), "replace") => {
                let p = &mut stored.presets[i];
                p.kind = x.kind;
                p.settings = x.settings;
                p.id.clone()
            }
            _ => {
                let name = presets::unique_name(&stored.presets, &x.name, None, lang);
                let preset = Preset {
                    id: presets::new_id(),
                    name,
                    kind: x.kind,
                    settings: x.settings,
                    builtin: None,
                    renamed: true,
                    extra: Default::default(),
                };
                let id = preset.id.clone();
                stored.presets.push(preset);
                id
            }
        };
        imported.push(id.clone());
        if let Some(acc) = x.shortcut {
            if taken(&acc) || shortcuts.iter().any(|(_, a)| *a == acc) {
                without_shortcut.push(x.name.clone());
            } else {
                shortcuts.push((id, acc));
            }
        }
    }
    presets::save(core, &stored);
    presets::changed(core);
    for (id, acc) in shortcuts {
        let _ = core
            .shortcuts
            .set(&presets::slot_of(&id), Some(acc), false, profile, lang);
    }
    core.shortcuts_changed();
    json!({
        "imported": imported.len(),
        "ids": imported,
        "skipped": skipped,
        "withoutShortcut": without_shortcut,
    })
}

/// Handlers of the `preset` area, inside the capture extension.
pub fn register(reg: &mut Registry, weak: &Weak<Core>) {
    // Presets travel in Export settings too (APP-011, CAP-059).
    xs_core::settings_io::add_group("presets", settings_export, settings_import);
    macro_rules! on {
        ($id:literal, $f:expr) => {{
            let w = weak.clone();
            reg.handle($id, move |ctx, p| {
                let core = upgrade(&w)?;
                $f(&core, ctx, &p)
            });
        }};
    }
    on!("preset.list", |core: &Core, ctx: &Context, _p: &Value| Ok(
        listed(core, ctx.lang)
    ));
    on!("preset.run", run);
    on!("preset.create", create);
    on!("preset.update", update);
    on!("preset.rename", rename);
    on!("preset.duplicate", duplicate);
    on!("preset.delete", delete);
    on!("preset.restore", restore);
    on!("preset.move", move_to);
    on!("preset.restore_builtins", restore_builtins);
    on!("preset.export", export);
    on!("preset.import", import);
}

#[cfg(test)]
mod tests {
    use super::*;
    use xs_core::Source;

    fn core() -> Arc<Core> {
        Core::for_tests_with(Platform::Windows, vec![crate::extension()])
    }

    fn call(core: &Core, id: &str, p: Value) -> Value {
        core.call(Source::Cli, id, p, Some(Lang::En))
            .unwrap_or_else(|e| panic!("{id}: {e:?}"))
    }

    fn by_name(core: &Core, name: &str) -> Preset {
        presets::load(core)
            .presets
            .into_iter()
            .find(|p| presets::display_name(p, Lang::En) == name)
            .unwrap_or_else(|| panic!("no preset {name}"))
    }

    fn names(core: &Core) -> Vec<String> {
        call(core, "preset.list", json!({}))["presets"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["name"].as_str().unwrap().to_string())
            .collect()
    }

    /// CAP-064: a new install has Region to clipboard, without a shortcut, named in the
    /// app's language; once deleted, it does not come back by itself, but Restore does.
    #[test]
    fn built_in_presets() {
        let c = core();
        let list = call(&c, "preset.list", json!({}));
        let first = &list["presets"][0];
        assert_eq!(first["name"], "Region to clipboard");
        assert_eq!(first["builtin"], "region_clipboard");
        assert_eq!(first["shortcut"], Value::Null);
        assert_eq!(first["settings"]["open"], "none");
        assert_eq!(first["settings"]["copy"], true);
        let es = c
            .call(Source::Cli, "preset.list", json!({}), Some(Lang::Es))
            .unwrap();
        assert_eq!(es["presets"][0]["name"], "Región al portapapeles");
        // Extract text comes with the app too (SMT-005).
        assert_eq!(list["presets"][1]["builtin"], "extract_text");
        assert_eq!(list["presets"][1]["settings"]["selection"], "text");
        // And Scrolling, automatic and vertical from the beginning (CAP-064, CAP-035).
        assert_eq!(list["presets"][2]["builtin"], "scrolling");
        assert_eq!(list["presets"][2]["settings"]["scroll"]["type"], "auto");
        assert_eq!(list["presets"][2]["settings"]["scroll"]["direction"], "vertical");
        let id = first["id"].as_str().unwrap();
        call(&c, "preset.delete", json!({"ids": [id]}));
        assert_eq!(
            names(&c),
            ["Extract text", "Scrolling"],
            "it does not come back at the next load"
        );
        let r = call(&c, "preset.restore_builtins", json!({}));
        assert_eq!(r["restored"], json!(["Region to clipboard"]));
        // Changed, it is reported, and put back only when asked.
        let id = by_name(&c, "Region to clipboard").id;
        call(
            &c,
            "preset.update",
            json!({"id": id, "settings": {"open": "editor"}}),
        );
        let r = call(&c, "preset.restore_builtins", json!({}));
        assert_eq!(r["modified"], json!(["Region to clipboard"]));
        call(&c, "preset.restore_builtins", json!({"reset": true}));
        assert_eq!(by_name(&c, "Region to clipboard").settings["open"], "none");
    }

    /// CAP-059: from the panel's active tab, or from defaults; taken names get " (2)";
    /// creating does not touch the panel; a preset plans with its own settings only.
    #[test]
    fn creating_presets() {
        let c = core();
        c.call(
            Source::Ui,
            "capture.configure",
            json!({"tab": "image", "image": {"selection": "window", "open": "none", "timing": "delay", "delay": 3}}),
            None,
        )
        .unwrap();
        let panel_before = call(&c, "capture.settings", json!({}));
        let p = call(&c, "preset.create", json!({"fromCurrent": true}));
        assert_eq!(p["kind"], "image");
        assert_eq!(p["settings"]["selection"], "window");
        assert_eq!(p["settings"]["delay"], 3);
        assert_eq!(p["name"], "Window", "a name that says what it does");
        let fresh = call(&c, "preset.create", json!({"kind": "image", "name": "Window"}));
        assert_eq!(fresh["name"], "Window (2)");
        assert_eq!(
            fresh["settings"]["selection"], "region",
            "defaults, not the panel's"
        );
        let u = call(
            &c,
            "preset.create",
            json!({"kind": "universal", "settings": {"selection": "fullscreen"}}),
        );
        assert_eq!(u["name"], "Universal: Full screen");
        assert_eq!(call(&c, "capture.settings", json!({})), panel_before);
        let (command, params, _) = presets::plan(&by_name(&c, "Window")).unwrap();
        assert_eq!(command, "capture.window");
        assert_eq!(params["open"], "none");
        assert_eq!(params["delay"], 3);
        assert_eq!(call(&c, "capture.settings", json!({})), panel_before);
        let bad = c
            .call(Source::Cli, "preset.create", json!({"name": "a\nb"}), None)
            .unwrap_err();
        assert_eq!(bad.code, ErrorCode::InvalidParams);
        let unknown = c
            .call(Source::Cli, "preset.run", json!({"name": "nope"}), None)
            .unwrap_err();
        assert_eq!(unknown.code, ErrorCode::NotFound);
    }

    /// CAP-060, CAP-061, CAP-062: a shortcut follows a rename, not a duplicate; a delete
    /// frees it and Undo brings the preset back in its place with it; moving reorders.
    #[test]
    fn shortcuts_rename_duplicate_delete_undo_and_order() {
        let c = core();
        let a = call(&c, "preset.create", json!({"name": "A"}));
        let id = a["id"].as_str().unwrap().to_string();
        call(
            &c,
            "shortcuts.set",
            json!({"command": a["slot"], "shortcut": "Ctrl+Alt+Shift+1"}),
        );
        let rows = call(&c, "shortcuts.list", json!({}));
        let row = rows["rows"]
            .as_array()
            .unwrap()
            .iter()
            .find(|r| r["command"] == a["slot"])
            .cloned()
            .unwrap();
        assert_eq!(row["title"], "A");
        assert_eq!(row["group"], "presets");
        assert_eq!(row["runs"], "preset.run");
        // Another command's shortcut is not taken without Reassign (CAP-062, APP-018).
        let taken = c
            .call(
                Source::Cli,
                "shortcuts.set",
                json!({"command": a["slot"], "shortcut": "Ctrl+Alt+Shift+K"}),
                None,
            )
            .unwrap_err();
        assert_eq!(taken.code, ErrorCode::Busy);
        assert_eq!(taken.details.unwrap()["conflict"], "duplicate");
        call(&c, "preset.rename", json!({"id": id, "to": "Ventana"}));
        let after = call(&c, "preset.list", json!({}));
        let renamed = after["presets"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == json!(id))
            .cloned()
            .unwrap();
        assert_eq!(renamed["name"], "Ventana");
        assert_eq!(renamed["shortcut"], "Ctrl+Alt+Shift+Digit1");
        let copy = call(&c, "preset.duplicate", json!({"id": id}));
        assert_eq!(copy["name"], "Ventana (copy)");
        assert_eq!(copy["shortcut"], Value::Null);
        assert_eq!(
            names(&c),
            [
                "Region to clipboard",
                "Extract text",
                "Scrolling",
                "Ventana",
                "Ventana (copy)"
            ]
        );
        // Delete and Undo.
        let r = call(&c, "preset.delete", json!({"ids": [id]}));
        assert_eq!(r["deleted"], 1);
        assert!(presets::shortcut_of(&c, &id).is_none(), "the keys are free");
        assert_eq!(
            names(&c),
            [
                "Region to clipboard",
                "Extract text",
                "Scrolling",
                "Ventana (copy)"
            ]
        );
        call(&c, "preset.restore", json!({"undo": r["undo"]}));
        assert_eq!(
            names(&c),
            [
                "Region to clipboard",
                "Extract text",
                "Scrolling",
                "Ventana",
                "Ventana (copy)"
            ]
        );
        assert_eq!(
            presets::shortcut_of(&c, &id).unwrap().to_string(),
            "Ctrl+Alt+Shift+Digit1"
        );
        // Reorder; the icon menu has the same order.
        call(&c, "preset.move", json!({"id": copy["id"], "index": 0}));
        assert_eq!(
            names(&c),
            [
                "Ventana (copy)",
                "Region to clipboard",
                "Extract text",
                "Scrolling",
                "Ventana"
            ]
        );
        let menu: Vec<String> = presets::menu_items(&c).into_iter().map(|(_, n)| n).collect();
        assert_eq!(menu, names(&c));
    }

    /// APP-011, CAP-059: presets travel in the settings file with their shortcuts; what is
    /// there already as the file has it is not imported again.
    #[test]
    fn presets_travel_in_the_settings_file() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("a.xsettings");
        let a = core();
        let one = call(
            &a,
            "preset.create",
            json!({"name": "Uno", "settings": {"selection": "window"}}),
        );
        call(
            &a,
            "shortcuts.set",
            json!({"command": one["slot"], "shortcut": "Ctrl+Alt+Shift+2"}),
        );
        call(&a, "settings.export", json!({"out": file}));
        let text = std::fs::read_to_string(&file).unwrap();
        let exported: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(exported["groups"]["presets"]["presets"][3]["name"], "Uno");
        assert!(
            !exported["groups"]["shortcuts"]["overrides"]
                .as_object()
                .unwrap()
                .keys()
                .any(|k| k.starts_with("preset.")),
            "a preset's shortcut goes with the preset"
        );

        let b = core();
        let dry = call(
            &b,
            "settings.import",
            json!({"path": file, "dryRun": true, "groups": ["presets"]}),
        );
        assert_eq!(dry["groups"]["presets"]["changes"], json!(["Uno"]), "{dry}");
        call(
            &b,
            "settings.import",
            json!({"path": file, "groups": ["presets"]}),
        );
        assert_eq!(
            names(&b),
            ["Region to clipboard", "Extract text", "Scrolling", "Uno"]
        );
        let uno = by_name(&b, "Uno");
        assert_eq!(
            presets::shortcut_of(&b, &uno.id).unwrap().to_string(),
            "Ctrl+Alt+Shift+Digit2"
        );
        let again = call(
            &b,
            "settings.import",
            json!({"path": file, "dryRun": true, "groups": ["presets"]}),
        );
        assert_eq!(again["groups"]["presets"]["changes"], json!([]));
    }

    /// CAP-063: export and import keep settings, order and shortcuts; from the other
    /// platform the shortcuts are converted; a name that exists is kept, replaced or
    /// skipped; a damaged or newer file imports nothing.
    #[test]
    fn export_and_import_xpreset() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("mios.xpreset");
        let a = core();
        let one = call(
            &a,
            "preset.create",
            json!({"name": "Uno", "settings": {"selection": "window"}}),
        );
        call(&a, "preset.create", json!({"name": "Dos", "kind": "universal"}));
        call(
            &a,
            "shortcuts.set",
            json!({"command": one["slot"], "shortcut": "Ctrl+Alt+Shift+I"}),
        );
        let r = call(&a, "preset.export", json!({"out": file}));
        assert_eq!(r["count"], 5);
        let text = std::fs::read_to_string(&file).unwrap();
        assert!(text.contains("\"format\": \"xcreenshot-presets\""));

        let b = core();
        let dry = call(&b, "preset.import", json!({"paths": [file], "dryRun": true}));
        assert_eq!(dry["presets"].as_array().unwrap().len(), 5);
        assert_eq!(
            dry["presets"][0]["exists"], true,
            "Region to clipboard is there already"
        );
        call(
            &b,
            "preset.import",
            json!({"paths": [file], "onConflict": "skip"}),
        );
        assert_eq!(
            names(&b),
            ["Region to clipboard", "Extract text", "Scrolling", "Uno", "Dos"]
        );
        let uno = by_name(&b, "Uno");
        assert_eq!(uno.settings["selection"], "window");
        assert_eq!(
            presets::shortcut_of(&b, &uno.id).unwrap().to_string(),
            "Ctrl+Alt+Shift+KeyI"
        );
        // Again, keeping both: a copy with " (2)" and no shortcut (it is taken).
        let r = call(&b, "preset.import", json!({"paths": [file], "only": ["Uno"]}));
        assert_eq!(r["withoutShortcut"], json!(["Uno"]));
        assert_eq!(
            names(&b),
            [
                "Region to clipboard",
                "Extract text",
                "Scrolling",
                "Uno",
                "Dos",
                "Uno (2)"
            ]
        );
        // Replace changes the one there.
        let mut changed: Value = serde_json::from_str(&text).unwrap();
        changed["presets"][3]["settings"]["selection"] = json!("fullscreen");
        std::fs::write(&file, serde_json::to_vec(&changed).unwrap()).unwrap();
        call(
            &b,
            "preset.import",
            json!({"paths": [file], "only": ["Uno"], "onConflict": "replace"}),
        );
        assert_eq!(by_name(&b, "Uno").settings["selection"], "fullscreen");

        // From a Mac: Cmd becomes Ctrl.
        let mac = dir.path().join("mac.xpreset");
        let from_mac = json!({"format": "xcreenshot-presets", "formatVersion": 1, "platform": "macos",
            "presets": [{"name": "Del Mac", "kind": "image", "settings": {}, "shortcut": "Super+Alt+Shift+KeyJ"}]});
        std::fs::write(&mac, from_mac.to_string()).unwrap();
        let c = core();
        call(&c, "preset.import", json!({"paths": [mac]}));
        let del_mac = presets::load(&c).presets.last().cloned().unwrap();
        assert_eq!(
            presets::shortcut_of(&c, &del_mac.id).unwrap().to_string(),
            "Ctrl+Alt+Shift+KeyJ"
        );

        // Damaged or newer: nothing at all.
        let bad = dir.path().join("bad.xpreset");
        std::fs::write(&bad, "{ not json").unwrap();
        let before = names(&c);
        assert!(c
            .call(Source::Cli, "preset.import", json!({"paths": [mac, bad]}), None)
            .is_err());
        let newer = dir.path().join("newer.xpreset");
        let newer_file = json!({"format": "xcreenshot-presets", "formatVersion": 9, "presets": []});
        std::fs::write(&newer, newer_file.to_string()).unwrap();
        assert!(c
            .call(Source::Cli, "preset.import", json!({"paths": [newer]}), None)
            .is_err());
        assert_eq!(names(&c), before);
    }
}
