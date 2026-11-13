//! Files opened with XcreenShot (APP-001): double click, "Open with" or the command
//! line, also when XcreenShot is already running. Captures and images open in the
//! editor; `.xtheme` and `.xtemplate` start their import in the editor (EDT-018,
//! EDT-101); `.xpreset` starts the presets import in the capture panel (CAP-063).
//! Imports need their window's dialog, so their paths wait here until the window
//! takes them (`app.take_opening`).

use std::path::Path;
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};

use crate::i18n::tr;
use crate::{CommandError, CommandResult, Context, Core, ErrorCode};

/// What a file opened with XcreenShot does.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Kind {
    /// Opens in the editor.
    Document,
    /// Its import starts in the editor.
    EditorImport,
    /// Its import starts in the capture panel.
    Presets,
}

/// The kind of `path`, by its extension; `None`: XcreenShot does not open it.
pub fn kind(path: &str) -> Option<Kind> {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase)?;
    match ext.as_str() {
        "xshot" | "png" | "jpg" | "jpeg" => Some(Kind::Document),
        "xtheme" | "xtemplate" => Some(Kind::EditorImport),
        "xpreset" => Some(Kind::Presets),
        _ => None,
    }
}

#[derive(Default)]
struct Opening {
    editor: Mutex<Vec<String>>,
    presets: Mutex<Vec<String>>,
}

fn service(core: &Core) -> Arc<Opening> {
    core.service(Opening::default)
}

/// `app.open_files`.
pub fn open_files(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let paths: Vec<String> = p
        .get("paths")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).map(String::from).collect())
        .unwrap_or_default();
    if paths.is_empty() {
        return Err(CommandError::new(ErrorCode::InvalidParams, "paths"));
    }
    let mut opened = Vec::new();
    let mut skipped = Vec::new();
    let (mut editor, mut presets) = (Vec::new(), Vec::new());
    for path in paths {
        if !Path::new(&path).is_file() {
            skipped.push(json!({
                "path": path,
                "reason": tr(ctx.lang, "errors.file_not_found", &[]),
            }));
            continue;
        }
        match kind(&path) {
            Some(Kind::Document) => {
                match core.call(
                    ctx.source,
                    "document.open",
                    json!({ "path": path, "show": true }),
                    Some(ctx.lang),
                ) {
                    Ok(r) => opened.push(r["documentId"].clone()),
                    Err(e) => skipped.push(json!({ "path": path, "reason": e.message })),
                }
            }
            Some(Kind::EditorImport) => editor.push(path),
            Some(Kind::Presets) => presets.push(path),
            None => skipped.push(json!({
                "path": path,
                "reason": tr(ctx.lang, "errors.cannot_open_file", &[]),
            })),
        }
    }
    let imports = editor.len() + presets.len();
    let host = core.host();
    if !editor.is_empty() {
        service(core).editor.lock().unwrap().extend(editor);
        host.show_main_window()?;
        host.editor_event("opening", json!({ "kind": "editor" }));
    }
    if !presets.is_empty() {
        service(core).presets.lock().unwrap().extend(presets);
        core.call(
            ctx.source,
            "capture.panel",
            json!({ "show": true }),
            Some(ctx.lang),
        )?;
        host.editor_event("opening", json!({ "kind": "presets" }));
    }
    Ok(json!({ "opened": opened, "imports": imports, "skipped": skipped }))
}

/// `app.take_opening`: the files waiting for `kind`'s window (`editor` or `presets`),
/// once.
pub fn take(core: &Core, p: &Value) -> CommandResult {
    let svc = service(core);
    let list = match p.get("kind").and_then(Value::as_str) {
        Some("editor") => &svc.editor,
        Some("presets") => &svc.presets,
        _ => return Err(CommandError::new(ErrorCode::InvalidParams, "kind")),
    };
    let paths = std::mem::take(&mut *list.lock().unwrap());
    Ok(json!({ "paths": paths }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_file_goes_where_it_belongs() {
        assert_eq!(kind(r"C:\a\b.xshot"), Some(Kind::Document));
        assert_eq!(kind("shot.PNG"), Some(Kind::Document));
        assert_eq!(kind("dark.xtheme"), Some(Kind::EditorImport));
        assert_eq!(kind("steps.xtemplate"), Some(Kind::EditorImport));
        assert_eq!(kind("mine.XPRESET"), Some(Kind::Presets));
        assert_eq!(kind("notes.txt"), None);
        assert_eq!(kind("no_extension"), None);
    }
}
