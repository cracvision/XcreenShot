//! The recent tray (TRY-003, TRY-004, TRY-005, TRY-017): its order, pinned items,
//! capacity and the `tray.*` commands. The tray is the documents shown in the editor;
//! closing one never deletes its file.

use std::path::PathBuf;
use std::sync::Arc;

use serde_json::{json, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode, Lang};

use crate::{activate, editor, emit_list, open, recovery};

/// How many items the tray keeps (TRY-003).
pub fn capacity(core: &Core) -> usize {
    core.settings
        .get("tray.capacity")
        .and_then(|v| v.as_u64())
        .unwrap_or(50)
        .clamp(1, 200) as usize
}

/// Over the capacity, the oldest items that are not pinned, have no unsaved changes
/// and are not shown leave the tray (their files stay). Returns whether any left.
pub fn trim(core: &Core) -> bool {
    let ed = editor(core);
    let order = ed.store.tray();
    let cap = capacity(core);
    if order.len() <= cap {
        return false;
    }
    let active = ed.store.active();
    let mut free: Vec<(u64, String)> = order
        .iter()
        .filter(|id| active.as_deref() != Some(id.as_str()))
        .filter_map(|id| {
            ed.store
                .with(id, |d| (!d.pinned && !d.dirty()).then(|| (d.arrival, id.clone())))
                .flatten()
        })
        .collect();
    free.sort();
    let leaving: Vec<String> = free
        .into_iter()
        .take(order.len() - cap)
        .map(|(_, id)| id)
        .collect();
    if leaving.is_empty() {
        return false;
    }
    for doc in ed.store.remove(&leaving) {
        if let Some(rid) = doc.recovery_id {
            recovery::remove(&ed.recovery_dir(), &rid);
        }
    }
    true
}

fn not_found(lang: Lang, id: &str) -> CommandError {
    CommandError::new(
        ErrorCode::NotFound,
        tr(lang, "errors.document_not_found", &[("id", id)]),
    )
}

/// The documents a command names: `documentIds`, or `documentId`, or the one shown.
fn targets(core: &Core, lang: Lang, p: &Value) -> Result<Vec<String>, CommandError> {
    let ed = editor(core);
    let ids: Vec<String> = match p.get("documentIds").and_then(Value::as_array) {
        Some(list) => list
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect(),
        None => match p.get("documentId").and_then(Value::as_str) {
            Some(id) => vec![id.to_string()],
            None => ed.store.active().into_iter().collect(),
        },
    };
    if ids.is_empty() {
        return Err(CommandError::new(
            ErrorCode::InvalidParams,
            tr(lang, "errors.no_document", &[]),
        ));
    }
    if let Some(missing) = ids.iter().find(|id| !ed.store.exists(id)) {
        return Err(not_found(lang, missing));
    }
    Ok(ids)
}

/// `tray.list`: the items in the tray's order.
pub fn list(core: &Core) -> Value {
    let ed = editor(core);
    let lib = xs_library::library(core);
    let active = ed.store.active();
    let items: Vec<Value> = ed
        .store
        .tray()
        .iter()
        .filter_map(|id| {
            ed.store.with(id, |d| {
                let library_id = d.path.as_deref().and_then(|p| lib.find(p)).map(|i| i.id);
                json!({
                    "documentId": d.id,
                    "name": d.name(),
                    "path": d.path,
                    "libraryId": library_id,
                    "pinned": d.pinned,
                    "dirty": d.dirty(),
                    "active": active.as_deref() == Some(d.id.as_str()),
                    "missing": d.path.as_ref().is_some_and(|p| !p.exists()),
                })
            })
        })
        .collect();
    json!({ "items": items, "capacity": capacity(core) })
}

pub fn command(core: &Arc<Core>, ctx: &Context, id: &str, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let ed = editor(core);
    match id {
        "tray.list" => Ok(list(core)),
        "tray.open" => {
            let path = match (
                p.get("id").and_then(Value::as_str),
                p.get("path").and_then(Value::as_str),
            ) {
                (Some(item), _) => {
                    let lib = xs_library::library(core);
                    let found = lib.get(item).ok_or_else(|| {
                        CommandError::new(
                            ErrorCode::NotFound,
                            tr(lang, "errors.library_not_found", &[("id", item)]),
                        )
                    })?;
                    lib.abs(&found.path)
                }
                (None, Some(path)) => PathBuf::from(path),
                (None, None) => {
                    return Err(CommandError::new(
                        ErrorCode::InvalidParams,
                        tr(
                            lang,
                            "errors.invalid_param",
                            &[("param", "id"), ("reason", "give id or path")],
                        ),
                    ))
                }
            };
            let show = p.get("activate").and_then(Value::as_bool).unwrap_or(true);
            let doc = open(core, lang, &path, show)?;
            if !show {
                // In the tray without showing it in the editor.
                ed.store.show_in_tray(&doc);
                emit_list(core);
            }
            Ok(json!({ "documentId": doc }))
        }
        "tray.activate" => {
            let doc = p.get("documentId").and_then(Value::as_str).unwrap_or_default();
            if !ed.store.exists(doc) {
                return Err(not_found(lang, doc));
            }
            activate(core, doc);
            Ok(json!({ "documentId": doc }))
        }
        "tray.pin" | "tray.unpin" => {
            let ids = targets(core, lang, p)?;
            let changed = ed.store.set_pinned(&ids, id == "tray.pin");
            if changed {
                emit_list(core);
            }
            Ok(json!({ "changed": changed, "items": list(core)["items"] }))
        }
        "tray.move" => {
            let doc = p.get("documentId").and_then(Value::as_str).unwrap_or_default();
            let index = p.get("index").and_then(Value::as_u64).unwrap_or(0) as usize;
            let at = ed.store.move_to(doc, index).ok_or_else(|| not_found(lang, doc))?;
            emit_list(core);
            let pinned = ed.store.with(doc, |d| d.pinned).unwrap_or(false);
            Ok(json!({ "index": at, "pinned": pinned }))
        }
        "tray.close" => {
            let order = ed.store.tray();
            let which = p.get("which").and_then(Value::as_str).unwrap_or("this");
            let ids: Vec<String> = match which {
                "all" => order,
                "unpinned" => order
                    .into_iter()
                    .filter(|i| ed.store.with(i, |d| !d.pinned).unwrap_or(false))
                    .collect(),
                "others" => {
                    let keep = targets(core, lang, p)?;
                    order.into_iter().filter(|i| !keep.contains(i)).collect()
                }
                "this" => targets(core, lang, p)?,
                other => {
                    return Err(CommandError::new(
                        ErrorCode::InvalidParams,
                        tr(
                            lang,
                            "errors.invalid_param",
                            &[("param", "which"), ("reason", other)],
                        ),
                    ))
                }
            };
            if ids.is_empty() {
                return Ok(json!({ "status": "closed", "closed": [], "pending": [] }));
            }
            let mut params = json!({ "documentIds": ids });
            for key in ["save", "discard"] {
                if let Some(v) = p.get(key) {
                    params[key] = v.clone();
                }
            }
            crate::close(core, ctx, &params)
        }
        _ => Err(CommandError::internal(format!("unknown tray command {id}"))),
    }
}
