//! Sending later (CAP-052): with Open in = Editor, a capture with destinations sends
//! nothing at once. The editor shows "Waiting to send to: <destinations>" with Send
//! and Don't send; Send writes the image as it is then (with what was drawn on it).
//! Closing the document without sending sends nothing; the capture stays in the
//! library.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde_json::{json, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode, Lang};

use crate::filedest::{self, FileDest};
use crate::filename::Values;

/// What a document waits to send.
#[derive(Clone)]
pub struct Waiting {
    pub file: FileDest,
    pub values: Values,
    pub dpi: f64,
    pub meta: Value,
}

#[derive(Default)]
pub struct PendingState {
    map: Mutex<HashMap<String, Waiting>>,
}

fn service(core: &Core) -> Arc<PendingState> {
    core.service(PendingState::default)
}

fn destinations(lang: Lang) -> Vec<String> {
    vec![tr(lang, "ui.editor.dest_file", &[])]
}

fn tell(core: &Core, document_id: &str, waiting: bool) {
    let lang = core.lang();
    core.host().editor_event(
        "pending_share",
        json!({
            "documentId": document_id,
            "waiting": waiting,
            "destinations": if waiting { destinations(lang) } else { Vec::new() },
        }),
    );
}

/// Keeps what `document_id` will send when the user says so.
pub fn wait(core: &Core, document_id: &str, waiting: Waiting) {
    service(core)
        .map
        .lock()
        .unwrap()
        .insert(document_id.to_string(), waiting);
    tell(core, document_id, true);
}

/// `capture.pending_shares`: the documents waiting to send.
pub fn list(core: &Core, lang: Lang) -> Value {
    let svc = service(core);
    let map = svc.map.lock().unwrap();
    let items: Vec<Value> = map
        .keys()
        .map(|id| json!({ "documentId": id, "destinations": destinations(lang) }))
        .collect();
    json!({ "items": items })
}

/// Forgets what documents that are no longer open were waiting to send.
pub fn forget_closed(core: &Core) {
    let open = xs_editor::editor(core).store.ids();
    service(core)
        .map
        .lock()
        .unwrap()
        .retain(|id, _| open.contains(id));
}

/// `capture.send_pending`: Send (the image as it is now) or Don't send.
pub fn send(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let id = p
        .get("documentId")
        .and_then(Value::as_str)
        .ok_or_else(|| CommandError::new(ErrorCode::InvalidParams, "documentId"))?
        .to_string();
    let Some(waiting) = service(core).map.lock().unwrap().remove(&id) else {
        return Err(CommandError::new(
            ErrorCode::NotFound,
            tr(lang, "errors.nothing_pending", &[]),
        ));
    };
    tell(core, &id, false);
    if p.get("send").and_then(Value::as_bool) == Some(false) {
        return Ok(json!({ "sent": false }));
    }
    let snapshot = xs_editor::editor(core)
        .store
        .snapshot(&id)
        .map_err(|e| CommandError::internal(e.to_string()))?;
    let image = snapshot.render();
    let xshot = xs_editor::editor(core)
        .store
        .with(&id, |d| d.path.clone())
        .flatten()
        .unwrap_or_default();
    let host = core.host();
    let choose = move |path: &std::path::Path| host.pick_save(path);
    let mut values = waiting.values.clone();
    values.width = image.width();
    values.height = image.height();
    match filedest::write(
        core,
        &waiting.file,
        &image,
        &xshot,
        &values,
        waiting.dpi,
        Some(&waiting.meta),
        &choose,
    ) {
        Ok(path) => Ok(json!({ "sent": true, "files": [path] })),
        Err(e) if e == "cancelled" => Ok(json!({ "sent": false })),
        Err(e) => Err(CommandError::new(
            ErrorCode::Internal,
            tr(
                lang,
                "notify.file_destination_failed",
                &[("folder", &waiting.file.folder().to_string_lossy())],
            ),
        )
        .with_details(json!({ "reason": e }))),
    }
}
