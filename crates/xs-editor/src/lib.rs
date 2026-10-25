//! The image editor's documents (EDT-): the documents open in the editor, their undo
//! history, saving and exporting, and the handlers of the `document.*` and `editor.*`
//! commands declared in `xs_core::catalog` (EDT-119).
//!
//! Build 4a of M1: Move and Arrow, undo and redo, `.xshot`, PNG and JPG, Copy all,
//! close, unsaved changes and recovery copies (ADR 0004). Each capture is saved in
//! the library as `.xshot` (LIB-020).

pub mod objects;
pub mod recovery;
pub mod store;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use serde_json::{json, Map, Value};
use xs_core::core::{Extension, QuitGuard};
use xs_core::document::{Document, Object};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode, Lang, Source};
use xs_image::Base;

use crate::objects::Styles;
use crate::store::{DocStore, Format, Kind, OpenError, Snapshot};

/// Tools that exist in this build (EDT-003); the rest of `catalog::TOOLS` come later.
pub const AVAILABLE_TOOLS: [&str; 6] = ["move", "arrow", "text", "shape", "blur", "crop"];
/// Zoom steps in percent (EDT-007).
pub const ZOOM_STEPS: [f64; 16] = [
    10.0, 25.0, 33.0, 50.0, 67.0, 100.0, 150.0, 200.0, 300.0, 400.0, 600.0, 800.0, 1200.0, 1600.0, 2400.0,
    3200.0,
];

const STYLES_KEY: &str = "editor.styles";
const TOOL_KEY: &str = "editor.tool";
const VIEWS_KEY: &str = "editor.views";
const FLATTEN_OK_KEY: &str = "editor.flattenWarningOff";
/// Remembered views (EDT-007), most recent first.
const VIEWS_LIMIT: usize = 200;

/// Editor state of one core: documents, tool and styles.
pub struct Editor {
    pub store: DocStore,
    styles: Mutex<Styles>,
    tool: Mutex<String>,
    library: Mutex<PathBuf>,
    recovery_dir: Mutex<PathBuf>,
    /// One file write at a time (autosave in the background, saves, recovery).
    writing: Mutex<()>,
    /// Objects copied last, to paste them editable (EDT-089).
    copied: Mutex<Option<Copied>>,
}

/// What `document.copy_objects` put on the clipboard.
struct Copied {
    objects: Vec<Object>,
    from_document: String,
    /// Of the image written to the system clipboard: if the clipboard still holds it,
    /// pasting gives back the editable objects.
    fingerprint: u64,
}

impl Editor {
    fn load(core: &Core) -> Editor {
        let styles = core
            .recall(STYLES_KEY)
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        let tool = core
            .recall(TOOL_KEY)
            .and_then(|v| v.as_str().map(str::to_string))
            .filter(|t| AVAILABLE_TOOLS.contains(&t.as_str()))
            .unwrap_or_else(|| "move".to_string());
        Editor {
            store: DocStore::default(),
            styles: Mutex::new(styles),
            tool: Mutex::new(tool),
            library: Mutex::new(xs_library::location()),
            recovery_dir: Mutex::new(recovery::dir(&core.paths.local_dir)),
            writing: Mutex::new(()),
            copied: Mutex::new(None),
        }
    }

    pub fn styles(&self) -> Styles {
        self.styles.lock().unwrap().clone()
    }

    pub fn library(&self) -> PathBuf {
        self.library.lock().unwrap().clone()
    }

    pub fn recovery_dir(&self) -> PathBuf {
        self.recovery_dir.lock().unwrap().clone()
    }

    /// Other library and recovery folders (isolated tests).
    pub fn set_folders(&self, library: PathBuf, recovery: PathBuf) {
        *self.library.lock().unwrap() = library;
        *self.recovery_dir.lock().unwrap() = recovery;
    }
}

/// The editor of `core`.
pub fn editor(core: &Core) -> Arc<Editor> {
    core.service(|| Editor::load(core))
}

// ---------------------------------------------------------------------------
// Errors and events

fn invalid(msg: String) -> CommandError {
    CommandError::new(ErrorCode::InvalidParams, msg)
}

fn internal(lang: Lang, name: &str, reason: impl std::fmt::Display) -> CommandError {
    CommandError::new(
        ErrorCode::Internal,
        tr(
            lang,
            "errors.save_failed",
            &[("name", name), ("reason", &reason.to_string())],
        ),
    )
}

/// The document a command acts on: `documentId`, or the one shown in the editor.
fn resolve(core: &Core, lang: Lang, p: &Value) -> Result<String, CommandError> {
    let ed = editor(core);
    match p.get("documentId").and_then(Value::as_str) {
        Some(id) if ed.store.exists(id) => Ok(id.to_string()),
        Some(id) => Err(CommandError::new(
            ErrorCode::NotFound,
            tr(lang, "errors.document_not_found", &[("id", id)]),
        )),
        None => ed
            .store
            .active()
            .ok_or_else(|| CommandError::new(ErrorCode::NotFound, tr(lang, "errors.no_document", &[]))),
    }
}

fn translate_labels(mut info: Value, lang: Lang) -> Value {
    for key in ["undoLabel", "redoLabel"] {
        if let Some(label) = info.get(key).and_then(Value::as_str).map(str::to_string) {
            info[key] = json!(tr(lang, &label, &[]));
        }
    }
    info
}

fn info(core: &Core, id: &str, lang: Lang) -> Option<Value> {
    editor(core).store.info(id).map(|i| translate_labels(i, lang))
}

/// Tells the editor view about a document that changed (whoever changed it).
fn emit_document(core: &Core, id: &str) {
    if let Some(i) = info(core, id, core.lang()) {
        core.host().editor_event("document", i);
    }
    emit_title(core);
}

fn list(core: &Core) -> Value {
    let ed = editor(core);
    json!({ "active": ed.store.active(), "documents": ed.store.summaries() })
}

fn emit_list(core: &Core) {
    core.host().editor_event("documents", list(core));
    emit_title(core);
}

/// Window title: the document shown, and "— Edited" while it has unsaved changes
/// (EDT-012).
fn emit_title(core: &Core) {
    let ed = editor(core);
    let lang = core.lang();
    let app = tr(lang, "app.name", &[]);
    let title = ed
        .store
        .active()
        .and_then(|id| ed.store.with(&id, |d| (d.name(), d.dirty())))
        .map(|(name, dirty)| {
            let name = if dirty {
                tr(lang, "editor.edited", &[("name", &name)])
            } else {
                name
            };
            format!("{name} — {app}")
        })
        .unwrap_or(app);
    core.host().editor_event("title", json!({ "title": title }));
}

// ---------------------------------------------------------------------------
// Saving

fn write_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let partial = xs_core::document::partial_path(path);
    let result = std::fs::write(&partial, bytes).and_then(|_| std::fs::rename(&partial, path));
    if result.is_err() {
        let _ = std::fs::remove_file(&partial);
    }
    result
}

fn encode(snap: &Snapshot, format: Format, quality: u8) -> Result<Vec<u8>, String> {
    match format {
        Format::Png => xs_image::encode::png(&snap.render(), snap.document.dpi, false),
        Format::Jpg => xs_image::encode::jpg(
            &snap.render(),
            snap.document.dpi,
            quality,
            xs_image::color::Rgba::WHITE,
        ),
        Format::Xshot => {
            let mut buf = std::io::Cursor::new(Vec::new());
            snap.to_xshot(false)?
                .to_writer(&mut buf)
                .map_err(|e| e.to_string())?;
            Ok(buf.into_inner())
        }
    }
}

/// Writes `snap` to `path` in `format`. `.xshot` files go through their safe writer,
/// leaving out cropped pixels with `drop_cropped` (EDT-050).
fn write_snapshot(
    ed: &Editor,
    snap: &Snapshot,
    path: &Path,
    format: Format,
    quality: u8,
    drop_cropped: bool,
) -> Result<u64, String> {
    let _guard = ed.writing.lock().unwrap();
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if format == Format::Xshot {
        snap.to_xshot(drop_cropped)?
            .write(path)
            .map_err(|e| e.to_string())?;
        return Ok(std::fs::metadata(path).map(|m| m.len()).unwrap_or(0));
    }
    let bytes = encode(snap, format, quality)?;
    write_file(path, &bytes).map_err(|e| e.to_string())?;
    Ok(bytes.len() as u64)
}

/// "Delete cropped content when saving" (EDT-050).
fn drop_cropped(core: &Core) -> bool {
    core.settings.get_bool("editor.deleteCroppedContent")
}

/// Saves a document in its own file and clears its unsaved state (EDT-107).
fn save_own(core: &Core, id: &str) -> Result<PathBuf, String> {
    let ed = editor(core);
    let snap = ed.store.snapshot(id)?;
    let path = snap.path.clone().ok_or("no file")?;
    write_snapshot(&ed, &snap, &path, snap.format, 90, drop_cropped(core))?;
    ed.store.mark_saved(id, snap.version);
    drop_recovery(core, id);
    Ok(path)
}

/// Saves library documents with unsaved changes (EDT-012): when switching document,
/// closing, closing the window and quitting.
fn autosave(core: &Core, id: &str) {
    let ed = editor(core);
    let due = ed
        .store
        .with(id, |d| d.kind == Kind::Library && d.dirty())
        .unwrap_or(false);
    if due {
        match save_own(core, id) {
            Ok(_) => emit_document(core, id),
            Err(e) => {
                tracing::error!(error = %e, "library autosave failed");
                let lang = core.lang();
                let name = ed.store.with(id, |d| d.name()).unwrap_or_default();
                core.host().notify(
                    "problems",
                    &tr(lang, "app.name", &[]),
                    &tr(lang, "errors.save_failed", &[("name", &name), ("reason", &e)]),
                );
            }
        }
    }
}

/// Saves every library document with unsaved changes (window closed, APP-004).
pub fn save_library_documents(core: &Core) {
    for id in editor(core).store.ids() {
        autosave(core, &id);
    }
}

fn drop_recovery(core: &Core, id: &str) {
    let ed = editor(core);
    if let Some(Some(rid)) = ed.store.with(id, |d| d.recovery_id.take()) {
        recovery::remove(&ed.recovery_dir(), &rid);
    }
}

/// Writes a recovery copy of every document changed since its last copy (EDT-012).
pub fn write_recovery_copies(core: &Core) {
    let ed = editor(core);
    for id in ed.store.ids() {
        let due = ed
            .store
            .with(&id, |d| d.dirty() && d.version != d.recovery_version)
            .unwrap_or(false);
        if !due {
            continue;
        }
        let Ok(snap) = ed.store.snapshot(&id) else {
            continue;
        };
        let rid = ed
            .store
            .with(&id, |d| {
                d.recovery_id.get_or_insert_with(recovery::new_id).clone()
            })
            .unwrap_or_else(recovery::new_id);
        let entry = recovery::Entry {
            id: rid.clone(),
            name: snap.name.clone(),
            original_path: snap.path.clone(),
            kind: snap.kind.as_str().to_string(),
            format: snap.format.ext().to_string(),
            saved_at: chrono::Utc::now(),
        };
        let dir = ed.recovery_dir();
        let written = write_snapshot(
            &ed,
            &snap,
            &recovery::document_path(&dir, &rid),
            Format::Xshot,
            90,
            // A recovery copy keeps everything: it is the document as it was.
            false,
        )
        .and_then(|_| recovery::write_entry(&dir, &entry).map_err(|e| e.to_string()));
        match written {
            Ok(()) => {
                ed.store.with(&id, |d| d.recovery_version = snap.version);
            }
            Err(e) => tracing::warn!(error = %e, "recovery copy failed"),
        }
    }
}

/// Recovery copies every 30 s (EDT-012). The shell starts it once.
pub fn start_background(core: &Arc<Core>) {
    let weak = Arc::downgrade(core);
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_secs(recovery::INTERVAL_SECS));
        let Some(core) = weak.upgrade() else { return };
        write_recovery_copies(&core);
    });
}

// ---------------------------------------------------------------------------
// Opening and showing

/// Shows `id` in the editor; the document shown before is saved if it is a library
/// document (EDT-012).
fn activate(core: &Arc<Core>, id: &str) {
    let ed = editor(core);
    let previous = ed.store.set_active(Some(id));
    if let Some(prev) = previous.filter(|p| p != id) {
        // Saving can take a moment with a large image: do not hold up the switch.
        let core = core.clone();
        std::thread::spawn(move || autosave(&core, &prev));
    }
    emit_list(core);
    emit_document(core, id);
}

fn open_error(lang: Lang, path: &Path, e: OpenError) -> CommandError {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    match e {
        OpenError::Unsupported => invalid(tr(lang, "errors.unsupported_format", &[])),
        OpenError::Newer => {
            invalid(tr(lang, "errors.newer_format", &[])).with_details(json!({"reason": "newer_format"}))
        }
        OpenError::Unreadable(reason) if !path.exists() => CommandError::new(
            ErrorCode::NotFound,
            tr(
                lang,
                "errors.cannot_open",
                &[("name", &name), ("reason", &reason)],
            ),
        ),
        OpenError::Unreadable(reason) => invalid(tr(
            lang,
            "errors.cannot_open",
            &[("name", &name), ("reason", &reason)],
        )),
    }
}

fn remembered_view(core: &Core, path: &Path) -> Option<Value> {
    core.recall(VIEWS_KEY)?
        .as_array()?
        .iter()
        .find(|v| v["path"].as_str() == Some(&path.to_string_lossy()))
        .map(|v| v["view"].clone())
}

fn remember_view(core: &Core, path: &Path, view: &Value) {
    let key = path.to_string_lossy().into_owned();
    let mut views: Vec<Value> = core
        .recall(VIEWS_KEY)
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_default();
    views.retain(|v| v["path"].as_str() != Some(&key));
    views.insert(0, json!({ "path": key, "view": view }));
    views.truncate(VIEWS_LIMIT);
    core.remember(VIEWS_KEY, Value::Array(views));
}

/// Opens a file as a document (EDT-105). With `show`, it becomes the document shown.
pub fn open(core: &Arc<Core>, lang: Lang, path: &Path, show: bool) -> Result<String, CommandError> {
    let ed = editor(core);
    let id = match ed.store.find_by_path(path) {
        Some(id) => id,
        None => {
            let mut doc = DocStore::load(path, &ed.library()).map_err(|e| open_error(lang, path, e))?;
            doc.view = remembered_view(core, path);
            ed.store.insert(doc, show).0
        }
    };
    if show {
        activate(core, &id);
    } else {
        emit_list(core);
    }
    Ok(id)
}

/// Saves a new capture in the library as `.xshot` (LIB-020) and keeps it open, not
/// yet shown. Returns its document id and file.
pub fn add_capture(
    core: &Core,
    image: &image::RgbaImage,
    meta: Value,
    scale: f64,
    app: Option<&str>,
) -> Result<(String, PathBuf), String> {
    add_image(core, image, meta, scale, app)
}

/// A new library document from pixels (a capture, a pasted image).
fn add_image(
    core: &Core,
    image: &image::RgbaImage,
    meta: Value,
    scale: f64,
    app: Option<&str>,
) -> Result<(String, PathBuf), String> {
    let ed = editor(core);
    let path = xs_library::new_item_path(&ed.library(), app, chrono::Local::now(), "xshot");
    let dpi = (96.0 * scale).round();
    let document = Document::new(image.width(), image.height(), dpi);
    let png = xs_image::encode::png(image, dpi, true)?;
    let thumb = xs_image::encode::thumbnail(image, xs_core::document::THUMB_SIZE);
    let file = xs_core::document::Xshot {
        manifest: xs_core::document::Manifest::current(),
        document: document.clone(),
        base_png: png.clone(),
        meta: meta.clone(),
        thumb_png: xs_image::encode::png(&thumb, 0.0, true)?,
        images: Default::default(),
    };
    {
        let _guard = ed.writing.lock().unwrap();
        file.write(&path).map_err(|e| e.to_string())?;
    }
    let doc = DocStore::from_image(
        path.clone(),
        Kind::Library,
        document,
        meta,
        png,
        Base::from_image(image.clone()),
    );
    let (id, _) = ed.store.insert(doc, false);
    emit_list(core);
    Ok((id, path))
}

// ---------------------------------------------------------------------------
// Closing and quitting

fn pending_item(core: &Core, id: &str) -> Option<Value> {
    editor(core).store.with(
        id,
        |d| json!({ "kind": "document", "documentId": d.id, "name": d.name(), "path": d.path }),
    )
}

/// External documents with unsaved changes among `ids` (they need a decision).
fn needing_decision(core: &Core, ids: &[String]) -> Vec<String> {
    let ed = editor(core);
    ids.iter()
        .filter(|id| {
            ed.store
                .with(id, |d| d.kind == Kind::External && d.dirty())
                .unwrap_or(false)
        })
        .cloned()
        .collect()
}

struct EditorQuitGuard(Weak<Core>);

impl QuitGuard for EditorQuitGuard {
    fn pending(&self, _lang: Lang) -> Vec<Value> {
        let Some(core) = self.0.upgrade() else {
            return Vec::new();
        };
        let ids = editor(&core).store.ids();
        needing_decision(&core, &ids)
            .iter()
            .filter_map(|id| pending_item(&core, id))
            .collect()
    }

    fn before_quit(&self) {
        let Some(core) = self.0.upgrade() else { return };
        save_library_documents(&core);
        // Whatever is still unsaved stays recoverable (APP-004 forced quit, APP-047).
        write_recovery_copies(&core);
    }
}

fn close(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    let ed = editor(core);
    let lang = ctx.lang;
    let targets: Vec<String> = if p.get("all").and_then(Value::as_bool).unwrap_or(false) {
        ed.store.ids()
    } else {
        vec![resolve(core, lang, p)?]
    };
    let listed = |key: &str| -> Vec<String> {
        p.get(key)
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
            .unwrap_or_default()
    };
    let (save, discard) = (listed("save"), listed("discard"));
    let undecided: Vec<String> = needing_decision(core, &targets)
        .into_iter()
        .filter(|id| !save.contains(id) && !discard.contains(id))
        .collect();
    if !undecided.is_empty() {
        let pending: Vec<Value> = undecided.iter().filter_map(|id| pending_item(core, id)).collect();
        return Ok(json!({ "status": "needs_decision", "closed": [], "pending": pending }));
    }
    for id in &targets {
        let (kind, dirty) = ed
            .store
            .with(id, |d| (d.kind, d.dirty()))
            .unwrap_or((Kind::Library, false));
        if dirty && (kind == Kind::Library || save.contains(id)) {
            save_own(core, id).map_err(|e| {
                let name = ed.store.with(id, |d| d.name()).unwrap_or_default();
                internal(lang, &name, e)
            })?;
        }
    }
    let order = ed.store.ids();
    let active = ed.store.active();
    for doc in ed.store.remove(&targets) {
        if let Some(rid) = doc.recovery_id {
            recovery::remove(&ed.recovery_dir(), &rid);
        }
    }
    // Show the next document of the strip, or the one before (EDT-108).
    if let Some(active) = active.filter(|a| targets.contains(a)) {
        let pos = order.iter().position(|i| *i == active).unwrap_or(0);
        let next = order[pos..]
            .iter()
            .chain(order[..pos].iter().rev())
            .find(|i| !targets.contains(i) && ed.store.with(i, |d| d.shown).unwrap_or(false))
            .cloned();
        match next {
            Some(next) => activate(core, &next),
            None => {
                ed.store.set_active(None);
                emit_list(core);
            }
        }
    } else {
        emit_list(core);
    }
    Ok(json!({ "status": "closed", "closed": targets, "pending": [] }))
}

// ---------------------------------------------------------------------------
// Commands

fn object_ids(p: &Value, lang: Lang) -> Result<Vec<String>, CommandError> {
    let mut ids: Vec<String> = p
        .get("objectIds")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    if let Some(id) = p.get("objectId").and_then(Value::as_str) {
        ids.push(id.to_string());
    }
    if ids.is_empty() {
        return Err(invalid(tr(
            lang,
            "errors.invalid_param",
            &[
                ("param", "objectId"),
                ("reason", "objectId or objectIds is required"),
            ],
        )));
    }
    Ok(ids)
}

/// Kinds of the given objects, or `not_found` naming the first missing one.
fn kinds_of(core: &Core, lang: Lang, doc: &str, ids: &[String]) -> Result<Vec<String>, CommandError> {
    let found = editor(core)
        .store
        .with(doc, |d| {
            ids.iter()
                .map(|i| d.document.object(i).map(|o| o.kind.clone()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    found
        .into_iter()
        .zip(ids)
        .map(|(k, id)| {
            k.ok_or_else(|| {
                CommandError::new(
                    ErrorCode::NotFound,
                    tr(lang, "errors.object_not_found", &[("id", id)]),
                )
            })
        })
        .collect()
}

/// After a change: tell the view, and save library documents that automation changed
/// while nobody is looking at them.
fn changed(core: &Core, ctx: &Context, id: &str) {
    emit_document(core, id);
    emit_list(core);
    let unseen = editor(core).store.active().as_deref() != Some(id);
    if ctx.source.is_automation() && unseen {
        autosave(core, id);
    }
}

fn set_tool_style(
    core: &Core,
    tool: &str,
    changes: &Map<String, Value>,
    lang: Lang,
) -> Result<Value, CommandError> {
    let ed = editor(core);
    let (style, styles) = {
        let mut styles = ed.styles.lock().unwrap();
        let style = styles.set(tool, changes).map_err(|r| {
            invalid(tr(
                lang,
                "errors.invalid_param",
                &[("param", "style"), ("reason", &r)],
            ))
        })?;
        (style, styles.clone())
    };
    let all = serde_json::to_value(&styles).unwrap_or_default();
    core.remember(STYLES_KEY, all.clone());
    core.host().editor_event("styles", all);
    Ok(style)
}

fn zoom_step(current: f64, up: bool) -> f64 {
    if up {
        ZOOM_STEPS
            .iter()
            .copied()
            .find(|s| *s > current + 0.01)
            .unwrap_or(3200.0)
    } else {
        ZOOM_STEPS
            .iter()
            .rev()
            .copied()
            .find(|s| *s < current - 0.01)
            .unwrap_or(10.0)
    }
}

fn upgrade(weak: &Weak<Core>) -> Result<Arc<Core>, CommandError> {
    weak.upgrade()
        .ok_or_else(|| CommandError::internal("shutting down"))
}

/// Handlers of the `document.*` and `editor.*` commands.
pub fn extension() -> Extension {
    Box::new(|reg, weak| {
        if let Some(core) = weak.upgrade() {
            core.add_quit_guard(Arc::new(EditorQuitGuard(weak.clone())));
        }
        macro_rules! on {
            ($id:literal, |$core:ident, $ctx:ident, $p:ident| $body:expr) => {{
                let weak = weak.clone();
                reg.handle($id, move |$ctx: &Context, $p: Value| {
                    let $core = upgrade(&weak)?;
                    #[allow(unused_variables)]
                    let $ctx = $ctx;
                    $body
                });
            }};
        }

        on!("document.open", |core, ctx, p| {
            let path = PathBuf::from(p["path"].as_str().unwrap_or_default());
            let show = p
                .get("show")
                .and_then(Value::as_bool)
                .unwrap_or(!ctx.source.is_automation());
            let id = open(&core, ctx.lang, &path, show)?;
            if show {
                // Opened either way; without a window there is just nothing to show.
                let _ = core.host().show_main_window();
            }
            let mut out = editor(&core).store.with(&id, |d| d.summary()).unwrap_or_default();
            out["documentId"] = json!(id);
            Ok(out)
        });
        on!("document.list", |core, ctx, _p| Ok(list(&core)));
        on!("document.info", |core, ctx, p| {
            let id = resolve(&core, ctx.lang, &p)?;
            info(&core, &id, ctx.lang).ok_or_else(|| CommandError::internal("document closed"))
        });
        on!("document.annotate", |core, ctx, p| {
            let id = resolve(&core, ctx.lang, &p)?;
            let styles = editor(&core).styles();
            let mut objects = Vec::new();
            for (i, input) in p["objects"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .iter()
                .enumerate()
            {
                let index = (i + 1).to_string();
                let o = objects::new_object(input, &styles).map_err(|reason| {
                    invalid(tr(
                        ctx.lang,
                        "errors.invalid_object",
                        &[("index", &index), ("reason", &reason)],
                    ))
                    .with_details(json!({ "index": i + 1, "reason": reason }))
                })?;
                objects.push(o);
            }
            let kinds: Vec<&str> = objects.iter().map(|(k, _)| k.as_str()).collect();
            let label = if ctx.source.is_automation() {
                "history.annotate".to_string()
            } else {
                objects::label("draw", &kinds)
            };
            let ids = editor(&core)
                .store
                .mutate(&id, &label, |doc| {
                    let mut ids = Vec::new();
                    for (kind, props) in objects {
                        let oid = doc.next_object_id();
                        doc.objects.push(Object {
                            id: oid.clone(),
                            kind,
                            props,
                        });
                        ids.push(oid);
                    }
                    Ok::<_, CommandError>(ids)
                })
                .ok_or_else(|| CommandError::internal("document closed"))??;
            changed(&core, ctx, &id);
            Ok(json!({ "ids": ids }))
        });
        on!("document.update_object", |core, ctx, p| {
            let id = resolve(&core, ctx.lang, &p)?;
            let oids = object_ids(&p, ctx.lang)?;
            let kinds = kinds_of(&core, ctx.lang, &id, &oids)?;
            let set = p["set"].as_object().cloned().unwrap_or_default();
            // Try the change on copies first: it names the undo step and finds errors.
            let mut moved_only = true;
            let copies = editor(&core)
                .store
                .with(&id, |d| {
                    oids.iter()
                        .filter_map(|o| d.document.object(o).cloned())
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            for mut o in copies {
                let change = objects::update(&mut o, &set).map_err(|r| {
                    invalid(tr(
                        ctx.lang,
                        "errors.invalid_param",
                        &[("param", "set"), ("reason", &r)],
                    ))
                })?;
                moved_only &= change == objects::Change::Move;
            }
            let action = if moved_only { "move" } else { "change" };
            let kind_refs: Vec<&str> = kinds.iter().map(String::as_str).collect();
            let updated = editor(&core)
                .store
                .mutate(&id, &objects::label(action, &kind_refs), |doc| {
                    let mut out = Vec::new();
                    for oid in &oids {
                        let o = doc
                            .object_mut(oid)
                            .ok_or_else(|| CommandError::internal("object vanished"))?;
                        objects::update(o, &set).map_err(|r| {
                            invalid(tr(
                                ctx.lang,
                                "errors.invalid_param",
                                &[("param", "set"), ("reason", &r)],
                            ))
                        })?;
                        out.push(serde_json::to_value(&*o).unwrap_or_default());
                    }
                    Ok::<_, CommandError>(out)
                })
                .ok_or_else(|| CommandError::internal("document closed"))??;
            // EDT-014: a property changed on selected objects also becomes the tool's
            // current style. Agents do not change the user's styles.
            if ctx.source == Source::Ui {
                if let Some(style) = set.get("style").and_then(Value::as_object) {
                    let mut seen: Vec<&String> = Vec::new();
                    for kind in &kinds {
                        if !seen.contains(&kind) {
                            seen.push(kind);
                            let _ = set_tool_style(&core, kind, style, ctx.lang);
                        }
                    }
                }
            }
            changed(&core, ctx, &id);
            Ok(json!({ "objects": updated }))
        });
        on!("document.delete_object", |core, ctx, p| {
            let id = resolve(&core, ctx.lang, &p)?;
            let oids = object_ids(&p, ctx.lang)?;
            let kinds = kinds_of(&core, ctx.lang, &id, &oids)?;
            let kind_refs: Vec<&str> = kinds.iter().map(String::as_str).collect();
            editor(&core)
                .store
                .mutate(&id, &objects::label("delete", &kind_refs), |doc| {
                    doc.objects.retain(|o| !oids.contains(&o.id));
                    Ok::<_, CommandError>(())
                })
                .ok_or_else(|| CommandError::internal("document closed"))??;
            changed(&core, ctx, &id);
            Ok(json!({ "deleted": oids }))
        });
        for (cmd, redo) in [("document.undo", false), ("document.redo", true)] {
            let weak = weak.clone();
            reg.handle(cmd, move |ctx: &Context, p: Value| {
                let core = upgrade(&weak)?;
                let id = resolve(&core, ctx.lang, &p)?;
                let label = editor(&core)
                    .store
                    .step(&id, redo)
                    .ok_or_else(|| CommandError::internal("document closed"))?;
                if label.is_some() {
                    changed(&core, ctx, &id);
                }
                let (can_undo, can_redo) = editor(&core)
                    .store
                    .info(&id)
                    .map(|i| (i["canUndo"].as_bool(), i["canRedo"].as_bool()))
                    .unwrap_or_default();
                Ok(json!({
                    "label": label.map(|l| tr(ctx.lang, &l, &[])),
                    "canUndo": can_undo.unwrap_or(false),
                    "canRedo": can_redo.unwrap_or(false),
                }))
            });
        }
        on!("document.save", |core, ctx, p| {
            let id = resolve(&core, ctx.lang, &p)?;
            let ed = editor(&core);
            let name = ed.store.with(&id, |d| d.name()).unwrap_or_default();
            let Some(target) = p.get("path").and_then(Value::as_str).map(PathBuf::from) else {
                // EDT-107: the first time a flat file gets its objects flattened, explain it.
                let (kind, format, objects) = ed
                    .store
                    .with(&id, |d| (d.kind, d.format, d.document.objects.len()))
                    .unwrap_or((Kind::Library, Format::Xshot, 0));
                let confirmed = p.get("confirmFlatten").and_then(Value::as_bool).unwrap_or(false);
                let warning_off = core
                    .recall(FLATTEN_OK_KEY)
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);
                if kind == Kind::External
                    && format != Format::Xshot
                    && objects > 0
                    && ctx.source == Source::Ui
                    && !confirmed
                    && !warning_off
                {
                    return Ok(json!({ "status": "needs_decision", "reason": "flatten", "name": name }));
                }
                if p.get("dontAskAgain").and_then(Value::as_bool).unwrap_or(false) {
                    core.remember(FLATTEN_OK_KEY, json!(true));
                }
                let path = save_own(&core, &id).map_err(|e| internal(ctx.lang, &name, e))?;
                emit_document(&core, &id);
                emit_list(&core);
                return Ok(json!({ "status": "saved", "path": path, "linked": true }));
            };
            let format = match p.get("format").and_then(Value::as_str) {
                Some(f) => Format::from_name(f),
                None => Format::from_path(&target),
            }
            .ok_or_else(|| {
                invalid(tr(
                    ctx.lang,
                    "errors.invalid_param",
                    &[("param", "format"), ("reason", "use .xshot, .png or .jpg")],
                ))
            })?;
            let target = if target.extension().is_none() {
                target.with_extension(format.ext())
            } else {
                target
            };
            let snap = ed.store.snapshot(&id).map_err(|e| internal(ctx.lang, &name, e))?;
            write_snapshot(&ed, &snap, &target, format, 90, drop_cropped(&core))
                .map_err(|e| internal(ctx.lang, &name, e))?;
            // LIB-021: a library document stays linked to its library file (Save as
            // writes a copy). A file from elsewhere follows the new file (EDT-107).
            let linked = snap.kind == Kind::External;
            if linked {
                let kind = if format == Format::Xshot && xs_library::contains(&ed.library(), &target) {
                    Kind::Library
                } else {
                    Kind::External
                };
                ed.store.relink(&id, target.clone(), kind, format, snap.version);
                drop_recovery(&core, &id);
                emit_document(&core, &id);
                emit_list(&core);
            }
            Ok(json!({ "status": "saved", "path": target, "linked": linked }))
        });
        on!("document.export", |core, ctx, p| {
            let id = resolve(&core, ctx.lang, &p)?;
            let ed = editor(&core);
            let format = Format::from_name(p["format"].as_str().unwrap_or_default()).unwrap_or(Format::Png);
            let mut target = PathBuf::from(p["path"].as_str().unwrap_or_default());
            if target.extension().is_none() {
                target.set_extension(format.ext());
            }
            let name = target
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            if target.exists() {
                match p.get("onConflict").and_then(Value::as_str).unwrap_or("keepBoth") {
                    "replace" => {}
                    "fail" => {
                        return Err(invalid(tr(ctx.lang, "errors.file_exists", &[("name", &name)])));
                    }
                    _ => {
                        let dir = target.parent().map(Path::to_path_buf).unwrap_or_default();
                        let stem = target
                            .file_stem()
                            .map(|s| s.to_string_lossy().into_owned())
                            .unwrap_or_default();
                        let ext = target
                            .extension()
                            .map(|s| s.to_string_lossy().into_owned())
                            .unwrap_or_default();
                        target = xs_library::unique(&dir, &stem, &ext);
                    }
                }
            }
            let quality = p.get("quality").and_then(Value::as_u64).unwrap_or(90) as u8;
            let snap = ed.store.snapshot(&id).map_err(|e| internal(ctx.lang, &name, e))?;
            let bytes = write_snapshot(&ed, &snap, &target, format, quality, false)
                .map_err(|e| internal(ctx.lang, &name, e))?;
            Ok(json!({
                "path": target,
                "width": snap.document.canvas.width,
                "height": snap.document.canvas.height,
                "bytes": bytes,
            }))
        });
        on!("document.crop", |core, ctx, p| {
            let id = resolve(&core, ctx.lang, &p)?;
            if p.get("trim").and_then(Value::as_bool).unwrap_or(false) {
                return Err(invalid(tr(
                    ctx.lang,
                    "errors.invalid_param",
                    &[("param", "trim"), ("reason", "trimming borders arrives in M2")],
                )));
            }
            let r = p.get("rect").ok_or_else(|| {
                invalid(tr(
                    ctx.lang,
                    "errors.invalid_param",
                    &[("param", "rect"), ("reason", "rect is required")],
                ))
            })?;
            let num = |k: &str| r[k].as_f64().unwrap_or(0.0);
            let (x0, y0) = (num("x").floor() as i64, num("y").floor() as i64);
            let (x1, y1) = (
                (num("x") + num("w")).ceil() as i64,
                (num("y") + num("h")).ceil() as i64,
            );
            let base = editor(&core).store.with(&id, |d| d.base_size).unwrap_or((1, 1));
            let result = editor(&core)
                .store
                .mutate(&id, "history.crop", |doc| {
                    // Non-destructive (EDT-050): the canvas is a window over the original
                    // pixels, and can grow back up to their edges.
                    let old = doc.crop;
                    let nx0 = (old.x + x0).clamp(0, base.0 as i64);
                    let ny0 = (old.y + y0).clamp(0, base.1 as i64);
                    let nx1 = (old.x + x1).clamp(0, base.0 as i64);
                    let ny1 = (old.y + y1).clamp(0, base.1 as i64);
                    if nx1 - nx0 < 1 || ny1 - ny0 < 1 {
                        return Err(invalid(tr(
                            ctx.lang,
                            "errors.invalid_param",
                            &[("param", "rect"), ("reason", "the crop leaves no image")],
                        )));
                    }
                    let (dx, dy) = ((nx0 - old.x) as f64, (ny0 - old.y) as f64);
                    for o in &mut doc.objects {
                        objects::translate(o, -dx, -dy);
                    }
                    doc.crop_history.push(old);
                    doc.crop = xs_core::document::PixelRect {
                        x: nx0,
                        y: ny0,
                        width: (nx1 - nx0) as u32,
                        height: (ny1 - ny0) as u32,
                    };
                    doc.canvas.width = doc.crop.width;
                    doc.canvas.height = doc.crop.height;
                    Ok::<_, CommandError>((doc.canvas.width, doc.canvas.height))
                })
                .ok_or_else(|| CommandError::internal("document closed"))??;
            changed(&core, ctx, &id);
            Ok(json!({ "width": result.0, "height": result.1 }))
        });
        on!("document.copy_all", |core, ctx, p| {
            let id = resolve(&core, ctx.lang, &p)?;
            let snap = editor(&core)
                .store
                .snapshot(&id)
                .map_err(|e| internal(ctx.lang, "", e))?;
            copy_image(&snap.render()).map_err(|_| {
                CommandError::new(ErrorCode::Internal, tr(ctx.lang, "errors.copy_failed", &[]))
            })?;
            Ok(json!({}))
        });
        on!("document.copy_objects", |core, ctx, p| {
            let id = resolve(&core, ctx.lang, &p)?;
            let oids = object_ids(&p, ctx.lang)?;
            let kinds = kinds_of(&core, ctx.lang, &id, &oids)?;
            let ed = editor(&core);
            let snap = ed.store.snapshot(&id).map_err(|e| internal(ctx.lang, "", e))?;
            let objects: Vec<Object> = snap
                .document
                .objects
                .iter()
                .filter(|o| oids.contains(&o.id))
                .cloned()
                .collect();
            // Editable objects for XcreenShot and, at the same time, a PNG with
            // transparency for other apps (EDT-089).
            let image = objects_image(&snap, &oids);
            if let Some(img) = &image {
                copy_image(img).map_err(|_| {
                    CommandError::new(ErrorCode::Internal, tr(ctx.lang, "errors.copy_failed", &[]))
                })?;
            }
            *ed.copied.lock().unwrap() = Some(Copied {
                objects,
                from_document: id.clone(),
                fingerprint: image.as_ref().map(fingerprint).unwrap_or(0),
            });
            if p.get("cut").and_then(Value::as_bool).unwrap_or(false) {
                let kind_refs: Vec<&str> = kinds.iter().map(String::as_str).collect();
                ed.store
                    .mutate(&id, &objects::label("cut", &kind_refs), |doc| {
                        doc.objects.retain(|o| !oids.contains(&o.id));
                        Ok::<_, CommandError>(())
                    })
                    .ok_or_else(|| CommandError::internal("document closed"))??;
                changed(&core, ctx, &id);
            }
            Ok(json!({ "count": oids.len() }))
        });
        on!("document.paste", |core, ctx, p| {
            let ed = editor(&core);
            let target = match p.get("documentId").and_then(Value::as_str) {
                Some(_) => Some(resolve(&core, ctx.lang, &p)?),
                None => ed.store.active(),
            };
            let center = p
                .get("center")
                .and_then(Value::as_array)
                .and_then(|a| Some([a.first()?.as_f64()?, a.get(1)?.as_f64()?]));
            let in_place = p.get("inPlace").and_then(Value::as_bool).unwrap_or(false);
            let (image, text) = read_clipboard();
            let ours = {
                let copied = ed.copied.lock().unwrap();
                copied
                    .as_ref()
                    .filter(|c| image.as_ref().map(fingerprint) == Some(c.fingerprint))
                    .map(|c| (c.objects.clone(), c.from_document.clone()))
            };
            // An image from elsewhere with nothing open: a new document (EDT-089).
            let Some(id) = target else {
                let img = image.ok_or_else(|| {
                    CommandError::new(ErrorCode::NotFound, tr(ctx.lang, "errors.nothing_to_paste", &[]))
                })?;
                let (doc_id, _) = add_image(
                    &core,
                    &img,
                    json!({ "source": "clipboard" }),
                    1.0,
                    Some("Clipboard"),
                )
                .map_err(|e| internal(ctx.lang, "", e))?;
                activate(&core, &doc_id);
                let _ = core.host().show_main_window();
                return Ok(json!({ "pasted": "document", "documentId": doc_id, "ids": [] }));
            };
            let (mut objects, kind) = if let Some((objects, from)) = ours {
                // Shifted 10 px when the originals are still there, centered in the view
                // otherwise; in place with Ctrl+Shift+V (EDT-089).
                let originals_here = from == id
                    && ed
                        .store
                        .with(&id, |d| {
                            objects.iter().all(|o| d.document.object(&o.id).is_some())
                        })
                        .unwrap_or(false);
                let (dx, dy) = if in_place {
                    (0.0, 0.0)
                } else if originals_here {
                    (10.0, 10.0)
                } else if let (Some(c), Some(e)) = (center, extent(&objects)) {
                    (c[0] - (e[0] + e[2]) / 2.0, c[1] - (e[1] + e[3]) / 2.0)
                } else {
                    (0.0, 0.0)
                };
                let objects = objects
                    .into_iter()
                    .map(|mut o| {
                        objects::translate(&mut o, dx, dy);
                        o
                    })
                    .collect::<Vec<_>>();
                (objects, "objects")
            } else if let Some(text) = text {
                // Plain text: a text object with the current style (EDT-089).
                let at = center.unwrap_or([20.0, 20.0]);
                let (k, props) =
                    objects::new_object(&json!({"type": "text", "at": at, "text": text}), &ed.styles())
                        .map_err(|r| {
                            invalid(tr(
                                ctx.lang,
                                "errors.invalid_object",
                                &[("index", "1"), ("reason", &r)],
                            ))
                        })?;
                (
                    vec![Object {
                        id: String::new(),
                        kind: k,
                        props,
                    }],
                    "text",
                )
            } else if image.is_some() {
                return Err(CommandError::new(
                    ErrorCode::NotFound,
                    tr(ctx.lang, "errors.paste_image_later", &[]),
                ));
            } else {
                return Err(CommandError::new(
                    ErrorCode::NotFound,
                    tr(ctx.lang, "errors.nothing_to_paste", &[]),
                ));
            };
            let kinds: Vec<String> = objects.iter().map(|o| o.kind.clone()).collect();
            let kind_refs: Vec<&str> = kinds.iter().map(String::as_str).collect();
            let ids = ed
                .store
                .mutate(&id, &objects::label("paste", &kind_refs), |doc| {
                    let mut ids = Vec::new();
                    for o in objects.iter_mut() {
                        o.id = doc.next_object_id();
                        ids.push(o.id.clone());
                        doc.objects.push(o.clone());
                    }
                    Ok::<_, CommandError>(ids)
                })
                .ok_or_else(|| CommandError::internal("document closed"))??;
            changed(&core, ctx, &id);
            Ok(json!({ "pasted": kind, "documentId": id, "ids": ids }))
        });
        on!("document.close", |core, ctx, p| close(&core, ctx, &p));
        on!("editor.show", |core, ctx, p| {
            if p.get("documentId").is_some() {
                let id = resolve(&core, ctx.lang, &p)?;
                activate(&core, &id);
            }
            core.host().show_main_window()?;
            Ok(json!({}))
        });
        on!("editor.fonts", |_core, ctx, _p| Ok(
            json!({ "fonts": xs_image::text::families() })
        ));
        on!("editor.state", |core, ctx, _p| {
            let ed = editor(&core);
            let recoverable = recoverable(&core).len();
            Ok(json!({
                "tool": *ed.tool.lock().unwrap(),
                "tools": AVAILABLE_TOOLS,
                "styles": ed.styles(),
                "active": ed.store.active(),
                "documents": ed.store.summaries(),
                "recoverable": recoverable,
                "zoomSteps": ZOOM_STEPS,
            }))
        });
        on!("editor.select_tool", |core, ctx, p| {
            let tool = p["tool"].as_str().unwrap_or_default().to_string();
            if !AVAILABLE_TOOLS.contains(&tool.as_str()) {
                return Err(CommandError::new(
                    ErrorCode::NotFound,
                    tr(ctx.lang, "errors.tool_later", &[]),
                ));
            }
            *editor(&core).tool.lock().unwrap() = tool.clone();
            core.remember(TOOL_KEY, json!(tool));
            core.host().editor_event("tool", json!({ "tool": tool }));
            Ok(json!({ "tool": tool }))
        });
        on!("editor.set_style", |core, ctx, p| {
            let tool = p["tool"].as_str().unwrap_or_default();
            let changes = p["style"].as_object().cloned().unwrap_or_default();
            let style = set_tool_style(&core, tool, &changes, ctx.lang)?;
            Ok(json!({ "style": style }))
        });
        on!("editor.zoom", |core, ctx, p| {
            let id = resolve(&core, ctx.lang, &p)?;
            let mode = p["mode"].as_str().unwrap_or("actual").to_string();
            let value = p.get("value").and_then(Value::as_f64);
            let view = editor(&core).store.with(&id, |d| d.view.clone()).flatten();
            let current = view.as_ref().and_then(|v| v["zoom"].as_f64()).unwrap_or(100.0);
            let expected = match mode.as_str() {
                "actual" => Some(100.0),
                "percent" => Some(value.ok_or_else(|| {
                    invalid(tr(
                        ctx.lang,
                        "errors.invalid_param",
                        &[("param", "value"), ("reason", "required with mode percent")],
                    ))
                })?),
                "in" => Some(zoom_step(current, true)),
                "out" => Some(zoom_step(current, false)),
                // The editor works out fit and original size from its window.
                _ => None,
            };
            core.host().editor_event(
                "zoom",
                json!({ "documentId": id, "mode": mode, "value": expected.or(value) }),
            );
            Ok(json!({ "zoom": expected }))
        });
        on!("editor.view", |core, ctx, p| {
            let id = p["documentId"].as_str().unwrap_or_default().to_string();
            let view = json!({
                "zoom": p["zoom"], "x": p["x"], "y": p["y"],
                "viewportWidth": p["viewportWidth"], "viewportHeight": p["viewportHeight"], "dpr": p["dpr"],
            });
            let path = editor(&core)
                .store
                .with(&id, |d| {
                    d.view = Some(view.clone());
                    d.path.clone()
                })
                .flatten();
            if let Some(path) = path {
                remember_view(
                    &core,
                    &path,
                    &json!({ "zoom": p["zoom"], "x": p["x"], "y": p["y"] }),
                );
            }
            Ok(json!({}))
        });
        on!("editor.recovery_list", |core, ctx, _p| Ok(
            json!({ "items": recoverable(&core) })
        ));
        on!("editor.recovery_restore", |core, ctx, p| {
            let ids = p["ids"].as_array().cloned().unwrap_or_default();
            let mut opened = Vec::new();
            for rid in ids.iter().filter_map(Value::as_str) {
                let id = restore(&core, rid).ok_or_else(|| {
                    CommandError::new(
                        ErrorCode::NotFound,
                        tr(ctx.lang, "errors.recovery_not_found", &[("id", rid)]),
                    )
                })?;
                opened.push(id);
            }
            if let Some(last) = opened.last() {
                activate(&core, last);
                let _ = core.host().show_main_window();
            }
            Ok(json!({ "documents": opened }))
        });
        on!("editor.recovery_discard", |core, ctx, p| {
            let ed = editor(&core);
            for rid in p["ids"]
                .as_array()
                .cloned()
                .unwrap_or_default()
                .iter()
                .filter_map(Value::as_str)
            {
                recovery::remove(&ed.recovery_dir(), rid);
            }
            Ok(json!({}))
        });
    })
}

/// Items of the recovery folder that no open document is using (APP-047).
fn recoverable(core: &Core) -> Vec<Value> {
    let ed = editor(core);
    let in_use: Vec<String> = ed
        .store
        .ids()
        .iter()
        .filter_map(|id| ed.store.with(id, |d| d.recovery_id.clone()).flatten())
        .collect();
    recovery::list(&ed.recovery_dir())
        .into_iter()
        .filter(|e| !in_use.contains(&e.id))
        .map(|e| serde_json::to_value(e).unwrap_or_default())
        .collect()
}

/// Opens a recovery copy as the document it was, with its unsaved changes.
fn restore(core: &Core, rid: &str) -> Option<String> {
    let ed = editor(core);
    let dir = ed.recovery_dir();
    let entry = recovery::list(&dir).into_iter().find(|e| e.id == rid)?;
    let copy = recovery::document_path(&dir, rid);
    let mut doc = DocStore::load(&copy, &ed.library()).ok()?;
    doc.path = entry.original_path.clone();
    doc.kind = if entry.kind == "library" {
        Kind::Library
    } else {
        Kind::External
    };
    doc.format = Format::from_name(&entry.format).unwrap_or(Format::Xshot);
    doc.recovery_id = Some(rid.to_string());
    let (id, added) = ed.store.insert(doc, true);
    if added {
        // The recovered state is not in its file yet.
        ed.store.mark_unsaved(&id);
    }
    Some(id)
}

fn fingerprint(image: &image::RgbaImage) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    image.dimensions().hash(&mut h);
    image.as_raw().hash(&mut h);
    h.finish()
}

/// The objects alone, on a transparent background, cut to what they cover: what other
/// apps get when objects are copied (EDT-089).
fn objects_image(snap: &store::Snapshot, ids: &[String]) -> Option<image::RgbaImage> {
    let mut doc = snap.document.clone();
    doc.objects.retain(|o| ids.contains(&o.id));
    doc.canvas.color = None;
    let (w, h) = (doc.canvas.width, doc.canvas.height);
    doc.crop = xs_core::document::PixelRect {
        x: 0,
        y: 0,
        width: w,
        height: h,
    };
    let clear = Base::from_image(image::RgbaImage::new(w, h));
    let img = xs_image::render(&doc, &clear);
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
    for (x, y, p) in img.enumerate_pixels() {
        if p.0[3] > 0 {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x + 1);
            y1 = y1.max(y + 1);
        }
    }
    (x1 > x0 && y1 > y0).then(|| image::imageops::crop_imm(&img, x0, y0, x1 - x0, y1 - y0).to_image())
}

fn read_clipboard() -> (Option<image::RgbaImage>, Option<String>) {
    let Ok(mut cb) = arboard::Clipboard::new() else {
        return (None, None);
    };
    let image = cb
        .get_image()
        .ok()
        .and_then(|i| image::RgbaImage::from_raw(i.width as u32, i.height as u32, i.bytes.into_owned()));
    let text = cb.get_text().ok().filter(|t| !t.trim().is_empty());
    (image, text)
}

/// Canvas extent of objects: `[x0, y0, x1, y1]` from their points and rectangles.
fn extent(objects: &[Object]) -> Option<[f64; 4]> {
    let mut e: Option<[f64; 4]> = None;
    let mut add = |x: f64, y: f64| {
        let b = e.get_or_insert([x, y, x, y]);
        *b = [b[0].min(x), b[1].min(y), b[2].max(x), b[3].max(y)];
    };
    for o in objects {
        for key in ["from", "to", "at"] {
            if let Some([x, y]) = o
                .props
                .get(key)
                .and_then(|v| Some([v.get(0)?.as_f64()?, v.get(1)?.as_f64()?]))
            {
                add(x, y);
            }
        }
        if let Some(r) = o.props.get("rect") {
            let n = |k: &str| r.get(k).and_then(Value::as_f64).unwrap_or(0.0);
            add(n("x"), n("y"));
            add(n("x") + n("w"), n("y") + n("h"));
        }
    }
    e
}

/// Copies an image to the clipboard, retrying while another app holds it (EDT-090).
fn copy_image(image: &image::RgbaImage) -> Result<(), arboard::Error> {
    let data = arboard::ImageData {
        width: image.width() as usize,
        height: image.height() as usize,
        bytes: std::borrow::Cow::Borrowed(image.as_raw()),
    };
    let mut last = None;
    for _ in 0..10 {
        match arboard::Clipboard::new().and_then(|mut c| c.set_image(data.clone())) {
            Ok(()) => return Ok(()),
            Err(e) => last = Some(e),
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(last.unwrap_or(arboard::Error::ClipboardOccupied))
}

#[cfg(test)]
mod tests;
