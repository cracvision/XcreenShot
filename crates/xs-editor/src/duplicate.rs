//! `library.duplicate` (TRY-010): a full, independent copy of each item beside it,
//! named "<name> (copy)", "(copy 2)"… It keeps the editable objects, the tags and the
//! capture metadata, and gets a new ID; it is not a favorite. Unsaved changes of an
//! open document go into the copy; a PNG or JPG with unsaved changes is copied as an
//! editable `.xshot`. With `open`, the copies join the tray next to their originals,
//! and a single copy is shown.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::{json, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode, Lang};

use crate::store::Format;
use crate::{activate, editor, emit_list, internal, open, write_file};

/// "<stem> (copy)", then "(copy 2)", "(copy 3)"… free in `dir`.
pub fn copy_name(dir: &Path, stem: &str, ext: &str, word: &str) -> PathBuf {
    let taken = |p: &Path| p.exists() || xs_core::document::partial_path(p).exists();
    let first = dir.join(format!("{stem} ({word}).{ext}"));
    if !taken(&first) {
        return first;
    }
    (2..)
        .map(|n| dir.join(format!("{stem} ({word} {n}).{ext}")))
        .find(|p| !taken(p))
        .unwrap_or(first)
}

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

/// One item to copy: its file, and the open document if there is one.
struct Source {
    path: PathBuf,
    doc: Option<String>,
}

fn sources(core: &Core, lang: Lang, p: &Value) -> Result<Vec<Source>, CommandError> {
    let ed = editor(core);
    let lib = xs_library::library(core);
    let texts = |key: &str| -> Vec<String> {
        p.get(key)
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect()
    };
    let mut out = Vec::new();
    for id in texts("documentIds") {
        let path = ed
            .store
            .with(&id, |d| d.path.clone())
            .ok_or_else(|| {
                CommandError::new(
                    ErrorCode::NotFound,
                    tr(lang, "errors.document_not_found", &[("id", &id)]),
                )
            })?
            .ok_or_else(|| bad(lang, "documentIds", "a document without a file"))?;
        out.push(Source { path, doc: Some(id) });
    }
    for id in texts("ids") {
        let item = lib.get(&id).ok_or_else(|| {
            CommandError::new(
                ErrorCode::NotFound,
                tr(lang, "errors.library_not_found", &[("id", &id)]),
            )
        })?;
        let path = lib.abs(&item.path);
        let doc = ed.store.find_by_path(&path);
        out.push(Source { path, doc });
    }
    for path in texts("paths").into_iter().map(PathBuf::from) {
        let doc = ed.store.find_by_path(&path);
        out.push(Source { path, doc });
    }
    if out.is_empty() {
        return Err(bad(lang, "ids", "give ids, documentIds or paths"));
    }
    Ok(out)
}

pub fn duplicate(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let ed = editor(core);
    let lib = xs_library::library(core);
    let word = tr(lang, "ui.library.copy_word", &[]);
    let show = p.get("open").and_then(Value::as_bool).unwrap_or(false);
    let mut copies: Vec<(Source, PathBuf, Vec<String>)> = Vec::new();
    for source in sources(core, lang, p)? {
        let name = source
            .path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let ext = source
            .path
            .extension()
            .map(|e| e.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_default();
        // A video: its file, with its cuts when they are not saved yet (VID-066).
        let video = source
            .doc
            .as_ref()
            .filter(|id| ed.store.with(id, |d| d.is_video()).unwrap_or(false));
        if let Some(id) = video {
            let dir = source.path.parent().map(Path::to_path_buf).unwrap_or_default();
            let target = copy_name(&dir, &name, &ext, &word);
            let tmp = core.paths.local_dir.join("tmp").join("duplicate");
            let file = crate::video::current_file(core, id, &tmp).map_err(|e| internal(lang, &name, e))?;
            std::fs::copy(&file, &target).map_err(|e| internal(lang, &name, e))?;
            if file != source.path {
                let _ = std::fs::remove_file(&file);
            }
            let tags = lib.find(&source.path).map(|i| i.tags).unwrap_or_default();
            copies.push((source, target, tags));
            continue;
        }
        // The state to copy: the open document's (with its unsaved changes) or the file.
        let open_state = source.doc.as_ref().and_then(|id| {
            ed.store
                .with(id, |d| d.dirty() || d.format == Format::Xshot)
                .filter(|needed| *needed)
                .and_then(|_| ed.store.snapshot(id).ok())
        });
        let (bytes, ext) = match open_state {
            Some(snap) => (
                crate::encode(&snap, Format::Xshot, 90).map_err(|e| internal(lang, &name, e))?,
                "xshot".to_string(),
            ),
            None => (
                std::fs::read(&source.path).map_err(|e| internal(lang, &name, e))?,
                ext,
            ),
        };
        let dir = source.path.parent().map(Path::to_path_buf).unwrap_or_default();
        let target = copy_name(&dir, &name, &ext, &word);
        write_file(&target, &bytes).map_err(|e| internal(lang, &name, e))?;
        let tags = lib.find(&source.path).map(|i| i.tags).unwrap_or_default();
        copies.push((source, target, tags));
    }
    lib.scan();
    let mut out = Vec::new();
    let single = copies.len() == 1;
    for (source, target, tags) in &copies {
        let id = lib.find(target).map(|i| i.id);
        if let (Some(id), false) = (&id, tags.is_empty()) {
            let _ = lib.tag(std::slice::from_ref(id), tags, &[]);
        }
        let mut document = None;
        if show {
            let doc = open(core, lang, target, false)?;
            ed.store.show_in_tray(&doc);
            // Beside its original in the tray.
            let others: Vec<String> = ed.store.tray().into_iter().filter(|t| *t != doc).collect();
            if let Some(at) = source
                .doc
                .as_ref()
                .and_then(|o| others.iter().position(|t| t == o))
            {
                ed.store.move_to(&doc, at + 1);
            }
            if single {
                activate(core, &doc);
            }
            document = Some(doc);
        }
        out.push(json!({
            "source": source.path,
            "path": target,
            "id": id,
            "documentId": document,
        }));
    }
    if show && !single {
        emit_list(core);
    }
    Ok(json!({ "copies": out }))
}
