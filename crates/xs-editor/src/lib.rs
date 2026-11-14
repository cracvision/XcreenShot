//! The image editor's documents (EDT-): the documents open in the editor, their undo
//! history, saving and exporting, and the handlers of the `document.*` and `editor.*`
//! commands declared in `xs_core::catalog` (EDT-119).
//!
//! Build 4a of M1: Move and Arrow, undo and redo, `.xshot`, PNG and JPG, Copy all,
//! close, unsaved changes and recovery copies (ADR 0004). Each capture is saved in
//! the library as `.xshot` (LIB-020).

pub mod auto_redact;
pub mod duplicate;
pub mod fit;
pub mod flatten;
pub mod guide;
pub mod guide_export;
pub mod guide_out;
pub mod image_ops;
pub mod library_export;
pub mod objects;
pub mod pixels;
pub mod quick_view;
pub mod recovery;
pub mod samples;
pub mod sheet_pdf;
pub mod steps;
pub mod store;
mod template_cmds;
pub mod templates;
pub use template_cmds::template_thumbnail;
pub mod themes;
pub mod transform;
pub mod tray;
pub mod video;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex, PoisonError, Weak};
use std::time::{Duration, Instant};

use serde_json::{json, Map, Value};
use xs_core::core::{Extension, QuitGuard};
use xs_core::document::{Document, Object, PixelRect};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode, Lang, Source};
use xs_image::selection::Mask;
use xs_image::Base;

use crate::objects::Styles;
use crate::store::{DocStore, Format, Kind, OpenError, Snapshot};

/// Tools that exist in this build (EDT-003); the rest of `catalog::TOOLS` come later.
pub const AVAILABLE_TOOLS: [&str; 21] = [
    "favorites",
    "move",
    "arrow",
    "line",
    "text",
    "callout",
    "shape",
    "stamp",
    "step",
    "highlight",
    "pen",
    "magnify",
    "spotlight",
    "blur",
    "simplify",
    "select",
    "wand",
    "fill",
    "eraser",
    "crop",
    "cutout",
];
/// Zoom steps in percent (EDT-007).
pub const ZOOM_STEPS: [f64; 16] = [
    10.0, 25.0, 33.0, 50.0, 67.0, 100.0, 150.0, 200.0, 300.0, 400.0, 600.0, 800.0, 1200.0, 1600.0, 2400.0,
    3200.0,
];

const STYLES_KEY: &str = "editor.styles";
const TOOL_KEY: &str = "editor.tool";
const VIEWS_KEY: &str = "editor.views";
const FLATTEN_OK_KEY: &str = "editor.flattenWarningOff";
/// The documents of the strip when XcreenShot last ended (EDT-120).
const OPEN_KEY: &str = "editor.openDocuments";
const REOPEN_SETTING: &str = "editor.reopenDocuments";
/// Longest wait of a command for the documents reopened at startup (EDT-120).
const REOPEN_WAIT: Duration = Duration::from_secs(10);
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
    /// Documents being reopened at startup; commands wait for them (EDT-120).
    reopening: Mutex<bool>,
    reopened: Condvar,
    /// The open documents last remembered, to write them only when they change.
    remembered_open: Mutex<Option<Value>>,
    /// Quitting with "clear the recent tray" (TRY-008): the list is not written again.
    forget_open: std::sync::atomic::AtomicBool,
    /// The view `editor.show` asked for before the window loaded (LIB-001).
    requested_view: Mutex<Option<String>>,
    /// New captures while automatic saving is off (LIB-020).
    unsaved_dir: Mutex<PathBuf>,
    /// The Photo fill running, to stop it (EDT-045).
    filling: Mutex<Option<Arc<std::sync::atomic::AtomicBool>>>,
    /// Themes, saved styles and favorites (EDT-015 to EDT-018).
    themes: Mutex<themes::Library>,
}

/// What `document.copy_objects` put on the clipboard.
struct Copied {
    objects: Vec<Object>,
    from_document: String,
    /// Pictures of the floating images among them (EDT-078), so they paste anywhere.
    images: BTreeMap<String, Vec<u8>>,
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
            library: Mutex::new(xs_library::location_for(core)),
            recovery_dir: Mutex::new(recovery::dir(&core.paths.local_dir)),
            writing: Mutex::new(()),
            copied: Mutex::new(None),
            reopening: Mutex::new(false),
            reopened: Condvar::new(),
            remembered_open: Mutex::new(None),
            forget_open: std::sync::atomic::AtomicBool::new(false),
            requested_view: Mutex::new(None),
            unsaved_dir: Mutex::new(core.paths.local_dir.join("unsaved")),
            filling: Mutex::new(None),
            themes: Mutex::new(themes::Library::load(core.recall(themes::KEY))),
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

    /// Captures kept out of the library until they are saved (LIB-020 off).
    fn unsaved_dir(&self) -> PathBuf {
        self.unsaved_dir.lock().unwrap().clone()
    }

    /// Waits, briefly, for the documents reopened at startup (EDT-120).
    fn wait_reopened(&self) {
        let busy = self.reopening.lock().unwrap_or_else(PoisonError::into_inner);
        let _ = self.reopened.wait_timeout_while(busy, REOPEN_WAIT, |busy| *busy);
    }

    /// Other library and recovery folders (isolated tests). Unsaved captures then wait
    /// inside the recovery folder.
    pub fn set_folders(&self, library: PathBuf, recovery: PathBuf) {
        *self.library.lock().unwrap() = library;
        *self.unsaved_dir.lock().unwrap() = recovery.join("unsaved");
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

/// The document a command names with `documentId`, or the one shown in the editor.
/// For other crates' commands that act on a document (SMT-040).
pub fn resolve_document(core: &Core, lang: Lang, p: &Value) -> Result<String, CommandError> {
    resolve(core, lang, p)
}

/// The final image of an open document and where the canvas's top left is in it (its
/// effects can make it larger) (SMT-007).
pub fn final_image_with_origin(core: &Core, id: &str) -> Result<(image::RgbaImage, [i64; 2]), String> {
    Ok(editor(core).store.snapshot(id)?.render_with_origin())
}

/// The final image of an open document, with its objects (EDT-110, SMT-040).
pub fn final_image(core: &Core, id: &str) -> Result<image::RgbaImage, String> {
    Ok(editor(core).store.snapshot(id)?.render())
}

/// The files to hand to another app when items are dragged out of XcreenShot
/// (TRY-015): open documents by ID and files by path. A PNG or JPG without unsaved
/// changes goes as it is; a `.xshot`, or a document with unsaved changes, is
/// flattened to a PNG in `out`, named after the document. The originals never change.
pub fn files_for_drag(
    core: &Core,
    docs: &[String],
    paths: &[PathBuf],
    out: &Path,
) -> Result<Vec<PathBuf>, String> {
    let ed = editor(core);
    let mut files = Vec::new();
    let flat = |name: &str, image: &image::RgbaImage, dpi: f64| -> Result<PathBuf, String> {
        std::fs::create_dir_all(out).map_err(|e| e.to_string())?;
        let stem = xs_library::file_name(name).unwrap_or_else(|| "XcreenShot".into());
        let target = xs_library::unique(out, &stem, "png");
        let bytes = xs_image::encode::png(image, dpi, false)?;
        write_file(&target, &bytes).map_err(|e| e.to_string())?;
        Ok(target)
    };
    let from_doc = |id: &str, files: &mut Vec<PathBuf>| -> Result<(), String> {
        // A video: its file, cut when it has cuts not saved (VID-066).
        if ed.store.with(id, |d| d.is_video()).unwrap_or(false) {
            files.push(video::current_file(core, id, out)?);
            return Ok(());
        }
        let snap = ed.store.snapshot(id)?;
        let dirty = ed.store.with(id, |d| d.dirty()).unwrap_or(true);
        match &snap.path {
            Some(p) if !dirty && snap.format != Format::Xshot && p.is_file() => files.push(p.clone()),
            _ => files.push(flat(&snap.name, &snap.render(), snap.document.dpi)?),
        }
        Ok(())
    };
    for id in docs {
        from_doc(id, &mut files)?;
    }
    for path in paths {
        if let Some(id) = ed.store.find_by_path(path) {
            from_doc(&id, &mut files)?;
            continue;
        }
        match Format::from_path(path) {
            Some(Format::Xshot) => {
                let doc = DocStore::load(path, &ed.library()).map_err(|e| format!("{e:?}"))?;
                let name = path
                    .file_stem()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_default();
                files.push(flat(&name, &DocStore::render_doc(&doc)?, doc.document.dpi)?);
            }
            Some(_) if path.is_file() => files.push(path.clone()),
            _ => return Err(format!("not found: {}", path.display())),
        }
    }
    Ok(files)
}

/// Renames the file of an open document in its folder, keeping its extension
/// (SMT-041, TRY-006). Never replaces another file.
fn rename(core: &Core, lang: Lang, p: &Value) -> Result<Value, CommandError> {
    let id = resolve(core, lang, p)?;
    let ed = editor(core);
    let wanted = p["name"].as_str().unwrap_or_default();
    let name = xs_library::file_name(wanted).ok_or_else(|| {
        invalid(tr(
            lang,
            "errors.invalid_param",
            &[("param", "name"), ("reason", "empty")],
        ))
    })?;
    let path = ed
        .store
        .with(&id, |d| d.path.clone())
        .flatten()
        .ok_or_else(|| invalid(tr(lang, "errors.rename_unsaved", &[])))?;
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().into_owned())
        .unwrap_or_default();
    let target = path.with_file_name(if ext.is_empty() {
        name.clone()
    } else {
        format!("{name}.{ext}")
    });
    if target == path {
        return Ok(json!({ "path": path, "name": name }));
    }
    // A change of case only is the same file on Windows and macOS.
    let same_file = target.to_string_lossy().to_lowercase() == path.to_string_lossy().to_lowercase();
    if target.exists() && !same_file {
        let shown = target
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        return Err(invalid(tr(lang, "errors.file_exists", &[("name", &shown)])));
    }
    {
        let _guard = ed.writing.lock().unwrap();
        std::fs::rename(&path, &target).map_err(|e| {
            CommandError::new(
                ErrorCode::Internal,
                tr(
                    lang,
                    "errors.rename_failed",
                    &[("name", &name), ("reason", &e.to_string())],
                ),
            )
        })?;
        ed.store.with(&id, |d| d.path = Some(target.clone()));
    }
    // The library item keeps its ID (LIB-022).
    xs_library::xmeta::follow(&path, &target);
    tracing::info!(document = %id, "document renamed");
    emit_list(core);
    Ok(json!({ "path": target, "name": name }))
}

/// The final image of a document as a PNG an agent can look at (SMT-043, APP-029):
/// scaled down to `maxSize` on its longest side.
fn document_image(core: &Core, lang: Lang, p: &Value) -> Result<Value, CommandError> {
    let id = resolve(core, lang, p)?;
    let max = p
        .get("maxSize")
        .and_then(Value::as_u64)
        .unwrap_or(1568)
        .clamp(64, 2576) as u32;
    let (name, path) = editor(core)
        .store
        .with(&id, |d| (d.name(), d.path.clone()))
        .ok_or_else(|| CommandError::internal("document closed"))?;
    let full = final_image(core, &id).map_err(CommandError::internal)?;
    let image = if full.width().max(full.height()) > max {
        xs_image::encode::thumbnail(&full, max)
    } else {
        full
    };
    let png = xs_image::encode::png(&image, 0.0, true).map_err(CommandError::internal)?;
    use base64::Engine as _;
    Ok(json!({
        "image": {"mimeType": "image/png", "data": base64::engine::general_purpose::STANDARD.encode(png)},
        "width": image.width(),
        "height": image.height(),
        "name": name,
        "path": path,
    }))
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

/// Tells the editor view about a document another crate changed (SMT-021).
pub fn document_changed(core: &Core, id: &str) {
    emit_document(core, id);
}

/// Tells the editor view about a document that changed (whoever changed it).
fn emit_document(core: &Core, id: &str) {
    if let Some(i) = info(core, id, core.lang()) {
        core.host().editor_event("document", i);
    }
    emit_title(core);
    remember_open(core);
}

fn list(core: &Core) -> Value {
    let ed = editor(core);
    json!({
        "active": ed.store.active(),
        "documents": ed.store.summaries(),
        // The tray, left to right (TRY-004, TRY-005).
        "tray": ed.store.tray(),
    })
}

fn emit_list(core: &Core) {
    // Over the tray's capacity, the oldest free items leave it (TRY-003).
    tray::trim(core);
    core.host().editor_event("documents", list(core));
    emit_title(core);
    remember_open(core);
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
        Format::Video | Format::Gif => Err("a video is not an image".into()),
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

/// Where a new item of the library is written: the library, or the waiting folder
/// while the library is out of reach (LIB-030).
pub(crate) fn library_target(core: &Core) -> PathBuf {
    xs_library::waiting::target_root(&editor(core).library(), &core.paths.local_dir)
}

/// Saves a document in its own file and clears its unsaved state (EDT-107). A
/// capture kept unsaved (LIB-020) goes into the library.
fn save_own(core: &Core, id: &str) -> Result<PathBuf, String> {
    let ed = editor(core);
    let snap = ed.store.snapshot(id)?;
    let mut path = snap.path.clone().ok_or("no file")?;
    let unsaved = path.starts_with(ed.unsaved_dir());
    if unsaved {
        let app = snap.meta["app"].as_str();
        path = xs_library::new_item_path(&library_target(core), app, chrono::Local::now(), "xshot");
    }
    write_snapshot(&ed, &snap, &path, snap.format, 90, drop_cropped(core))?;
    if unsaved {
        if let Some(old) = &snap.path {
            let _ = std::fs::remove_file(old);
        }
        ed.store
            .relink(id, path.clone(), Kind::Library, Format::Xshot, snap.version);
    } else {
        ed.store.mark_saved(id, snap.version);
    }
    drop_recovery(core, id);
    Ok(path)
}

/// LIB-021: after a flat file from elsewhere is saved with its objects flattened, an
/// editable `.xshot` copy goes to the library (the same one at each save), linked to
/// the file.
fn keep_editable_copy(core: &Core, id: &str) {
    let ed = editor(core);
    let worth = ed
        .store
        .with(id, |d| {
            d.kind == Kind::External && d.format != Format::Xshot && !d.document.objects.is_empty()
        })
        .unwrap_or(false);
    if !worth || !core.settings.get_bool("library.keepEditableCopy") {
        return;
    }
    let Ok(mut snap) = ed.store.snapshot(id) else {
        return;
    };
    let existing = ed.store.with(id, |d| d.editable_copy.clone()).flatten();
    let target = existing.unwrap_or_else(|| {
        let dir = library_target(core).join(chrono::Local::now().format("%Y-%m").to_string());
        xs_library::unique(&dir, &snap.name, "xshot")
    });
    if let Some(original) = &snap.path {
        snap.meta["linkedFile"] = json!(original);
    }
    match write_snapshot(&ed, &snap, &target, Format::Xshot, 90, false) {
        Ok(_) => {
            ed.store.with(id, |d| d.editable_copy = Some(target.clone()));
            tracing::info!("editable copy kept in the library");
        }
        Err(e) => tracing::warn!(error = %e, "could not keep the editable copy"),
    }
}

/// Saves library documents with unsaved changes (EDT-012): when switching document,
/// closing, closing the window and quitting.
fn autosave(core: &Core, id: &str) {
    let ed = editor(core);
    let due = ed
        .store
        // A video's cuts are saved when the user says (ADR 0008): saving one encodes
        // it all again.
        .with(id, |d| d.kind == Kind::Library && d.dirty() && !d.is_video())
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
            // A video's cuts live only while it is open (ADR 0008).
            .with(&id, |d| {
                d.dirty() && d.version != d.recovery_version && !d.is_video()
            })
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
            ..Default::default()
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
        OpenError::TooLarge(w, h) => invalid(tr(
            lang,
            "errors.image_too_large",
            &[
                ("name", &name),
                ("width", &w.to_string()),
                ("height", &h.to_string()),
            ],
        ))
        .with_details(json!({"reason": "too_large", "width": w, "height": h})),
        // A video the system cannot decode says which codec (VID-055).
        OpenError::Codec(codec, reason) => invalid(tr(
            lang,
            "errors.video_codec",
            &[("name", &name), ("codec", &codec), ("reason", &reason)],
        ))
        .with_details(json!({"reason": "codec", "codec": codec})),
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
    let imported = import_video(core, path);
    let path = imported.as_deref().unwrap_or(path);
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

/// A video or GIF from outside the library goes into it, as it is (VID-055): copied to
/// the folder of this month, keeping its format. The same file opened again uses the
/// copy already there. `None`: open it where it is (in the library already, or it
/// does not read).
fn import_video(core: &Core, path: &Path) -> Option<PathBuf> {
    let video = match crate::store::Format::from_path(path)? {
        Format::Video => xs_video::facts(path).is_ok(),
        Format::Gif => std::fs::read(path)
            .ok()
            .and_then(|b| xs_image::gif_anim::info(&b))
            .is_some(),
        _ => false,
    };
    let ed = editor(core);
    if !video || xs_library::contains(&ed.library(), path) || !path.is_file() {
        return None;
    }
    let dir = library_target(core).join(chrono::Local::now().format("%Y-%m").to_string());
    let stem = path.file_stem()?.to_string_lossy().into_owned();
    let ext = path.extension()?.to_string_lossy().to_ascii_lowercase();
    let size = std::fs::metadata(path).ok()?.len();
    let same = dir.join(format!("{stem}.{ext}"));
    if std::fs::metadata(&same).is_ok_and(|m| m.len() == size) {
        return Some(same);
    }
    std::fs::create_dir_all(&dir).ok()?;
    let target = xs_library::unique(&dir, &stem, &ext);
    std::fs::copy(path, &target).ok()?;
    xs_library::library(core).scan();
    Some(target)
}

// ---------------------------------------------------------------------------
// Reopening at startup (EDT-120)

/// The documents of the editor's strip, in order, and the one shown; the pinned ones
/// and the tray's order (TRY-004, TRY-005). With `only_pinned`, just those (TRY-008).
fn open_documents_of(core: &Core, only_pinned: bool) -> Value {
    let ed = editor(core);
    let active = ed.store.active();
    let mut paths = Vec::new();
    let mut pinned = Vec::new();
    let mut shown = Value::Null;
    for id in ed.store.ids() {
        let Some((Some(path), true, pin)) = ed.store.with(&id, |d| (d.path.clone(), d.shown, d.pinned))
        else {
            continue;
        };
        if only_pinned && !pin {
            continue;
        }
        let path = Value::String(path.to_string_lossy().into_owned());
        if active.as_deref() == Some(id.as_str()) {
            shown = path.clone();
        }
        if pin {
            pinned.push(path.clone());
        }
        paths.push(path);
    }
    let order: Vec<Value> = ed
        .store
        .tray()
        .iter()
        .filter_map(|id| ed.store.with(id, |d| d.path.clone()).flatten())
        .map(|p| Value::String(p.to_string_lossy().into_owned()))
        .filter(|p| paths.contains(p))
        .collect();
    json!({ "documents": paths, "active": shown, "pinned": pinned, "order": order })
}

fn open_documents(core: &Core) -> Value {
    open_documents_of(core, false)
}

/// Keeps the list of open documents up to date at every change, so it survives any
/// way of ending: quitting, shutting down or a crash.
fn remember_open(core: &Core) {
    let ed = editor(core);
    if *ed.reopening.lock().unwrap_or_else(PoisonError::into_inner)
        || ed.forget_open.load(std::sync::atomic::Ordering::Relaxed)
    {
        return;
    }
    let value = if core.settings.get_bool(REOPEN_SETTING) {
        open_documents(core)
    } else {
        Value::Null
    };
    let mut last = ed.remembered_open.lock().unwrap();
    if last.as_ref() != Some(&value) {
        core.remember(OPEN_KEY, value.clone());
        *last = Some(value);
    }
}

/// Ends the reopening, even if it stopped halfway, and lets waiting commands go on.
struct Reopening(Arc<Core>);

impl Drop for Reopening {
    fn drop(&mut self) {
        let ed = editor(&self.0);
        *ed.reopening.lock().unwrap_or_else(PoisonError::into_inner) = false;
        ed.reopened.notify_all();
    }
}

/// Reopens, in the background and without windows, the documents that were in the
/// strip when XcreenShot last ended. Editor commands wait until it finishes. The shell
/// calls it once at startup, before it answers commands.
pub fn reopen_documents(core: &Arc<Core>) {
    // Captures left unsaved by a previous run (LIB-020): their changes, if any, are
    // in the recovery copies.
    let _ = std::fs::remove_dir_all(editor(core).unsaved_dir());
    let saved = core.recall(OPEN_KEY).filter(|v| !v.is_null());
    if !core.settings.get_bool(REOPEN_SETTING) {
        if saved.is_some() {
            core.remember(OPEN_KEY, Value::Null);
        }
        return;
    }
    let Some(saved) = saved.filter(|v| v["documents"].as_array().is_some_and(|a| !a.is_empty())) else {
        return;
    };
    *editor(core).reopening.lock().unwrap() = true;
    let guard = Reopening(core.clone());
    std::thread::spawn(move || {
        let started = Instant::now();
        let (opened, skipped) = reopen(&guard.0, &saved);
        let core = guard.0.clone();
        drop(guard);
        tracing::info!(
            opened,
            skipped,
            elapsed_ms = started.elapsed().as_millis() as u64,
            "documents reopened"
        );
        emit_list(&core);
    });
}

/// Opens the remembered documents; returns how many opened and how many were left out.
fn reopen(core: &Core, saved: &Value) -> (usize, usize) {
    let ed = editor(core);
    // A document with a recovery copy comes back through the recovery, with its changes.
    let recovering: Vec<PathBuf> = recovery::list(&ed.recovery_dir())
        .into_iter()
        .filter_map(|e| e.original_path)
        .collect();
    let library = ed.library();
    let (mut opened, mut skipped) = (0, 0);
    let mut shown = None;
    for p in saved["documents"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        let path = PathBuf::from(p);
        if recovering.iter().any(|r| store::same_file(r, &path)) {
            skipped += 1;
            continue;
        }
        match DocStore::load(&path, &library) {
            Ok(mut doc) => {
                doc.view = remembered_view(core, &path);
                let (id, _) = ed.store.insert(doc, true);
                if shown.is_none() || saved["active"].as_str() == Some(p) {
                    shown = Some(id);
                }
                opened += 1;
            }
            // Moved or deleted while XcreenShot was closed: left out without a word.
            Err(_) => skipped += 1,
        }
    }
    let paths = |key: &str| -> Vec<PathBuf> {
        saved[key]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .map(PathBuf::from)
            .collect()
    };
    ed.store.restore_tray(&paths("pinned"), &paths("order"));
    if let Some(id) = shown {
        ed.store.set_active(Some(&id));
    }
    (opened, skipped)
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
    add_capture_with(core, image, meta, scale, app, None)
}

/// The pointer taken with a capture (CAP-047): its picture and where its top-left
/// corner falls in the image, in image pixels.
pub struct Pointer {
    pub image: image::RgbaImage,
    pub x: i32,
    pub y: i32,
}

/// [`add_capture`] with the pointer, which arrives as a floating image the user can
/// move, resize or delete without a trace underneath (CAP-047; the Cursor object
/// itself, with its shapes, is EDT-093 in M2).
pub fn add_capture_with(
    core: &Core,
    image: &image::RgbaImage,
    meta: Value,
    scale: f64,
    app: Option<&str>,
    pointer: Option<&Pointer>,
) -> Result<(String, PathBuf), String> {
    let autosave = core.settings.get_bool("library.autosaveImages");
    add_capture_kept(core, image, meta, scale, app, pointer, autosave)
}

/// [`add_capture_with`], saved in the library or kept unsaved as `autosave` says
/// instead of the setting (quick annotation's "Save to the library", CAP-057).
pub fn add_capture_kept(
    core: &Core,
    image: &image::RgbaImage,
    meta: Value,
    scale: f64,
    app: Option<&str>,
    pointer: Option<&Pointer>,
    autosave: bool,
) -> Result<(String, PathBuf), String> {
    let ed = editor(core);
    let extra = pointer
        .map(|p| -> Result<(Picture, [f64; 2]), String> {
            let png = xs_image::encode::png(&p.image, 0.0, true)?;
            Ok((
                picture(png, "png", p.image.width(), p.image.height()),
                [p.x as f64, p.y as f64],
            ))
        })
        .transpose()?;
    let (path, document, png) = write_library_item_in(
        core,
        image,
        meta.clone(),
        scale,
        app,
        extra.as_ref(),
        None,
        autosave,
    )?;
    let mut base = Base::from_image(image.clone());
    let mut images = BTreeMap::new();
    if let Some((pic, _)) = &extra {
        base.add_image(&pic.name, &pic.bytes)?;
        images.insert(pic.name.clone(), pic.bytes.clone());
    }
    let doc = DocStore::from_image(
        path.clone(),
        if autosave { Kind::Library } else { Kind::External },
        document,
        meta,
        png,
        base,
    )
    .with_images(images);
    let (id, _) = ed.store.insert(doc, false);
    if !autosave {
        // Not in the library until saved: the strip marks it, closing asks (LIB-020).
        ed.store.mark_unsaved(&id);
    }
    emit_list(core);
    Ok((id, path))
}

/// Whether the canvas grows to fit objects that stick out of it (EDT-071).
fn auto_expand(core: &Core) -> bool {
    core.settings.get_bool("editor.autoExpandCanvas")
}

/// Saves a capture in the library without opening it (CAP-067).
pub fn save_capture(
    core: &Core,
    image: &image::RgbaImage,
    meta: Value,
    scale: f64,
    app: Option<&str>,
) -> Result<PathBuf, String> {
    write_library_item(core, image, meta, scale, app, None).map(|(path, _, _)| path)
}

/// A new library document from pixels (a capture, a pasted image).
fn add_image(
    core: &Core,
    image: &image::RgbaImage,
    meta: Value,
    scale: f64,
    app: Option<&str>,
) -> Result<(String, PathBuf), String> {
    add_image_with(core, image, meta, scale, app, None)
}

/// [`add_image`] with the canvas of a new image (EDT-103).
fn add_image_with(
    core: &Core,
    image: &image::RgbaImage,
    meta: Value,
    scale: f64,
    app: Option<&str>,
    fill: Option<&Value>,
) -> Result<(String, PathBuf), String> {
    let ed = editor(core);
    let autosave = core.settings.get_bool("library.autosaveImages");
    let (path, document, png) = write_library_item_with(core, image, meta.clone(), scale, app, None, fill)?;
    let doc = DocStore::from_image(
        path.clone(),
        if autosave { Kind::Library } else { Kind::External },
        document,
        meta,
        png,
        Base::from_image(image.clone()),
    );
    let (id, _) = ed.store.insert(doc, false);
    if !autosave {
        // Not in the library until saved: the strip marks it, closing asks (LIB-020).
        ed.store.mark_unsaved(&id);
    }
    emit_list(core);
    Ok((id, path))
}

/// Writes a new `.xshot` of `image` in the library: its path, document and PNG.
fn write_library_item(
    core: &Core,
    image: &image::RgbaImage,
    meta: Value,
    scale: f64,
    app: Option<&str>,
    picture: Option<&(Picture, [f64; 2])>,
) -> Result<(PathBuf, Document, Vec<u8>), String> {
    write_library_item_with(core, image, meta, scale, app, picture, None)
}

/// A canvas fill: `"#RRGGBB(AA)"`, a gradient or null (transparent), checked. The
/// canvas's color and gradient (EDT-074).
fn canvas_fill(v: &Value) -> Result<(Option<String>, Option<Value>), String> {
    match v {
        Value::Null => Ok((None, None)),
        Value::String(s) => {
            xs_image::color::Rgba::parse(s).ok_or("use #RRGGBB, #RRGGBBAA, a gradient or null")?;
            Ok((Some(s.clone()), None))
        }
        Value::Object(_) => {
            let fill: xs_image::paint::Fill = serde_json::from_value(v.clone()).map_err(|e| e.to_string())?;
            fill.check("color")?;
            let first = fill.main_color().ok_or("a gradient needs colors")?;
            Ok((Some(first.to_hex()), Some(v.clone())))
        }
        _ => Err("use #RRGGBB, #RRGGBBAA, a gradient or null".into()),
    }
}

/// [`write_library_item`] with the canvas of a new image (EDT-103) instead of the
/// default one.
fn write_library_item_with(
    core: &Core,
    image: &image::RgbaImage,
    meta: Value,
    scale: f64,
    app: Option<&str>,
    picture: Option<&(Picture, [f64; 2])>,
    fill: Option<&Value>,
) -> Result<(PathBuf, Document, Vec<u8>), String> {
    let autosave = core.settings.get_bool("library.autosaveImages");
    write_library_item_in(core, image, meta, scale, app, picture, fill, autosave)
}

/// [`write_library_item_with`], in the library or not as `autosave` says.
#[allow(clippy::too_many_arguments)]
fn write_library_item_in(
    core: &Core,
    image: &image::RgbaImage,
    meta: Value,
    scale: f64,
    app: Option<&str>,
    picture: Option<&(Picture, [f64; 2])>,
    fill: Option<&Value>,
    autosave: bool,
) -> Result<(PathBuf, Document, Vec<u8>), String> {
    let ed = editor(core);
    // Off (LIB-020): the capture waits in the app's data until it is saved.
    // Multi-capture saves to the library either way (CAP-067).
    let folder = if autosave || meta["mode"] == "multi" {
        library_target(core)
    } else {
        ed.unsaved_dir()
    };
    let path = xs_library::new_item_path(&folder, app, chrono::Local::now(), "xshot");
    let dpi = (96.0 * scale).round();
    let mut document = Document::new(image.width(), image.height(), dpi);
    // An opaque capture has a white canvas; one with transparency (a window with
    // rounded corners) keeps it transparent (EDT-074).
    if image.pixels().all(|p| p.0[3] == 255) {
        document.canvas.color = Some("#FFFFFF".to_string());
    }
    if let Some(fill) = fill {
        let (color, gradient) = canvas_fill(fill)?;
        document.canvas.color = color;
        document.canvas.gradient = gradient;
    }
    let png = xs_image::encode::png(image, dpi, true)?;
    let mut images = BTreeMap::new();
    let mut shown = std::borrow::Cow::Borrowed(image);
    if let Some((pic, [x, y])) = picture {
        let mut props = objects::image_object(&pic.name, [*x, *y, pic.width as f64, pic.height as f64])?;
        // The captured pointer is a Cursor object (EDT-093): as captured, no highlight.
        if let Some(style) = props.get_mut("style").and_then(Value::as_object_mut) {
            style.insert(
                "cursor".to_string(),
                json!({"shape": "captured", "highlight": "none", "natural": [pic.width, pic.height]}),
            );
        }
        document.objects.push(Object {
            id: document.next_object_id(),
            kind: "image".to_string(),
            props,
        });
        images.insert(pic.name.clone(), pic.bytes.clone());
        let mut base = Base::from_image(image.clone());
        base.add_image(&pic.name, &pic.bytes)?;
        shown = std::borrow::Cow::Owned(xs_image::render(&document, &base));
    } else if fill.is_some() {
        // A new image shows its canvas (EDT-103).
        shown = std::borrow::Cow::Owned(xs_image::render(&document, &Base::from_image(image.clone())));
    }
    let thumb = xs_image::encode::thumbnail(&shown, xs_core::document::THUMB_SIZE);
    let file = xs_core::document::Xshot {
        manifest: xs_core::document::Manifest::current(),
        document: document.clone(),
        base_png: png.clone(),
        meta: meta.clone(),
        thumb_png: xs_image::encode::png(&thumb, 0.0, true)?,
        images,
    };
    {
        let _guard = ed.writing.lock().unwrap();
        file.write(&path).map_err(|e| e.to_string())?;
    }
    Ok((path, document, png))
}

// ---------------------------------------------------------------------------
// Floating images (EDT-078)

/// Space between pictures stacked in a new document (as the "Stacked" layout of
/// CAP-033).
const STACK_GAP: u32 = 10;
/// Offset between pictures inserted together in an open document.
const CASCADE: f64 = 24.0;

/// A picture to insert: its file name in `images/`, its bytes and its size.
struct Picture {
    name: String,
    bytes: Vec<u8>,
    width: u32,
    height: u32,
}

/// FNV-1a: the same picture inserted twice is stored once.
fn content_hash(bytes: &[u8]) -> u64 {
    bytes.iter().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ *b as u64).wrapping_mul(0x0100_0000_01b3)
    })
}

fn picture(bytes: Vec<u8>, ext: &str, width: u32, height: u32) -> Picture {
    Picture {
        name: format!("{}{:016x}.{ext}", store::INSERTED, content_hash(&bytes)),
        bytes,
        width,
        height,
    }
}

/// What a file looks like as a floating image: the final image of a document (the
/// open state if it is open), or a PNG or JPG as it is.
fn picture_of(core: &Core, lang: Lang, path: &Path) -> Result<Picture, CommandError> {
    let ed = editor(core);
    let rendered = |image: image::RgbaImage| -> Result<Picture, CommandError> {
        let png = xs_image::encode::png(&image, 0.0, true).map_err(|e| internal(lang, "", e))?;
        Ok(picture(png, "png", image.width(), image.height()))
    };
    if let Some(id) = ed.store.find_by_path(path) {
        let snap = ed.store.snapshot(&id).map_err(|e| internal(lang, "", e))?;
        return rendered(snap.render());
    }
    match Format::from_path(path) {
        Some(Format::Xshot) => {
            let doc = DocStore::load(path, &ed.library()).map_err(|e| open_error(lang, path, e))?;
            rendered(DocStore::render_doc(&doc).map_err(|e| internal(lang, "", e))?)
        }
        Some(format) => {
            let bytes = std::fs::read(path)
                .map_err(|e| open_error(lang, path, OpenError::Unreadable(e.to_string())))?;
            // Too large is said before decoding it (APP-050).
            if let Some((w, h)) = image::ImageReader::new(std::io::Cursor::new(&bytes))
                .with_guessed_format()
                .ok()
                .and_then(|r| r.into_dimensions().ok())
                .filter(|(w, h)| !xs_image::encode::within_limits(*w, *h))
            {
                return Err(open_error(lang, path, OpenError::TooLarge(w, h)));
            }
            let image = xs_image::encode::decode(&bytes)
                .map_err(|e| open_error(lang, path, OpenError::Unreadable(e)))?;
            Ok(picture(bytes, format.ext(), image.width(), image.height()))
        }
        None => Err(open_error(lang, path, OpenError::Unsupported)),
    }
}

/// Adds `pictures` to document `id` as floating images, in one undo step, each at its
/// own size with its top-left corner at the matching point of `at`.
fn place_pictures(
    core: &Core,
    lang: Lang,
    id: &str,
    pictures: &[Picture],
    at: &[[f64; 2]],
    action: &str,
) -> Result<Vec<String>, CommandError> {
    let ed = editor(core);
    let files = pictures
        .iter()
        .map(|p| (p.name.clone(), p.bytes.clone()))
        .collect();
    ed.store
        .add_images(id, &files)
        .map_err(|e| internal(lang, "", e))?;
    let objects: Vec<Map<String, Value>> = pictures
        .iter()
        .zip(at)
        .map(|(p, [x, y])| objects::image_object(&p.name, [*x, *y, p.width as f64, p.height as f64]))
        .collect::<Result<_, _>>()
        .map_err(invalid)?;
    let auto = auto_expand(core);
    ed.store
        .mutate(id, &objects::label(action, &["image"]), |doc| {
            let mut ids = Vec::new();
            for props in objects {
                let oid = doc.next_object_id();
                doc.objects.push(Object {
                    id: oid.clone(),
                    kind: "image".to_string(),
                    props,
                });
                ids.push(oid);
            }
            if auto {
                fit::fit_canvas(doc);
            }
            Ok::<_, CommandError>(ids)
        })
        .ok_or_else(|| CommandError::internal("document closed"))?
}

/// `document.insert_images`: files as floating images of the document shown, in a
/// cascade from `center`. With nothing open, one library document opens as itself
/// and several make a new document with them stacked (CAP-067).
fn insert_images(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let ed = editor(core);
    let paths: Vec<PathBuf> = p["paths"]
        .as_array()
        .map(|a| a.iter().filter_map(Value::as_str).map(PathBuf::from).collect())
        .unwrap_or_default();
    if paths.is_empty() {
        return Err(invalid(tr(lang, "errors.nothing_to_insert", &[])));
    }
    let target = match p.get("documentId").and_then(Value::as_str) {
        Some(_) => Some(resolve(core, lang, p)?),
        None => ed.store.active(),
    };
    if target.is_none() && paths.len() == 1 && Format::from_path(&paths[0]) == Some(Format::Xshot) {
        let id = open(core, lang, &paths[0], true)?;
        return Ok(json!({ "documentId": id, "ids": [], "created": false }));
    }
    let pictures = paths
        .iter()
        .map(|path| picture_of(core, lang, path))
        .collect::<Result<Vec<_>, _>>()?;
    let (id, at, created) = match target {
        Some(id) => {
            let canvas = ed
                .store
                .with(&id, |d| (d.document.canvas.width, d.document.canvas.height))
                .ok_or_else(|| CommandError::internal("document closed"))?;
            let center = p
                .get("center")
                .and_then(Value::as_array)
                .and_then(|a| Some([a.first()?.as_f64()?, a.get(1)?.as_f64()?]))
                .unwrap_or([canvas.0 as f64 / 2.0, canvas.1 as f64 / 2.0]);
            let at = pictures
                .iter()
                .enumerate()
                .map(|(i, pic)| {
                    let step = CASCADE * i as f64;
                    [
                        (center[0] - pic.width as f64 / 2.0 + step).round(),
                        (center[1] - pic.height as f64 / 2.0 + step).round(),
                    ]
                })
                .collect::<Vec<_>>();
            (id, at, false)
        }
        None => {
            let width = pictures.iter().map(|p| p.width).max().unwrap_or(1);
            let height =
                pictures.iter().map(|p| p.height).sum::<u32>() + STACK_GAP * (pictures.len() as u32 - 1);
            let blank = image::RgbaImage::new(width, height);
            let (id, _) = add_image(core, &blank, json!({ "source": "insert" }), 1.0, None)
                .map_err(|e| internal(lang, "", e))?;
            let mut y = 0.0;
            let at = pictures
                .iter()
                .map(|pic| {
                    let here = [0.0, y];
                    y += (pic.height + STACK_GAP) as f64;
                    here
                })
                .collect::<Vec<_>>();
            (id, at, true)
        }
    };
    let ids = place_pictures(core, lang, &id, &pictures, &at, "insert")?;
    if created {
        activate(core, &id);
    }
    changed(core, ctx, &id);
    Ok(json!({ "documentId": id, "ids": ids, "created": created }))
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
                .with(id, |d| (d.kind == Kind::External || d.is_video()) && d.dirty())
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
        // TRY-008: the recent tray starts next time with only the pinned items (the
        // files stay).
        if core.settings.get_bool("tray.clearUnpinnedOnExit") {
            let ed = editor(&core);
            ed.forget_open.store(true, std::sync::atomic::Ordering::Relaxed);
            let pinned = open_documents_of(&core, true);
            let empty = pinned["documents"].as_array().is_none_or(|a| a.is_empty());
            core.remember(OPEN_KEY, if empty { Value::Null } else { pinned });
        }
        // Whatever is still unsaved stays recoverable (APP-004 forced quit, APP-047).
        write_recovery_copies(&core);
    }
}

fn close(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    let ed = editor(core);
    let lang = ctx.lang;
    let targets: Vec<String> = if p.get("all").and_then(Value::as_bool).unwrap_or(false) {
        ed.store.ids()
    } else if let Some(ids) = p.get("documentIds").and_then(Value::as_array) {
        let ids: Vec<String> = ids
            .iter()
            .filter_map(|v| v.as_str().map(str::to_string))
            .collect();
        if let Some(missing) = ids.iter().find(|id| !ed.store.exists(id)) {
            return Err(CommandError::new(
                ErrorCode::NotFound,
                tr(lang, "errors.document_not_found", &[("id", missing)]),
            ));
        }
        ids
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
        let (kind, dirty, video) = ed
            .store
            .with(id, |d| (d.kind, d.dirty(), d.is_video()))
            .unwrap_or((Kind::Library, false, false));
        if dirty && video && save.contains(id) {
            video::save(core, ctx, id, None)?;
        } else if dirty && ((kind == Kind::Library && !video) || save.contains(id)) {
            save_own(core, id).map_err(|e| {
                let name = ed.store.with(id, |d| d.name()).unwrap_or_default();
                internal(lang, &name, e)
            })?;
        }
    }
    let order = ed.store.tray();
    let active = ed.store.active();
    for doc in ed.store.remove(&targets) {
        if let Some(rid) = doc.recovery_id {
            recovery::remove(&ed.recovery_dir(), &rid);
        }
        // A capture never saved (LIB-020) leaves nothing behind.
        if let Some(path) = doc.path.filter(|p| p.starts_with(ed.unsaved_dir())) {
            let _ = std::fs::remove_file(path);
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

/// `document.effects` (EDT-053): the effect stack of the document, or of the floating
/// image `objectId`, in one undo step.
fn effects_command(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let id = resolve(core, lang, p)?;
    let bad = |param: &str, reason: &str| {
        invalid(tr(
            lang,
            "errors.invalid_param",
            &[("param", param), ("reason", reason)],
        ))
    };
    let action = p
        .get("action")
        .and_then(Value::as_str)
        .unwrap_or("set")
        .to_string();
    let target = p.get("objectId").and_then(Value::as_str).map(str::to_string);
    let index = p.get("index").and_then(Value::as_u64).map(|i| i as usize);
    // A watermark picture given as a file goes inside the document (EDT-066), so it
    // looks the same on another computer. Before the step: storing it takes the lock.
    let mut owned = p.clone();
    let keep_picture = |e: &mut Value| -> Result<(), CommandError> {
        let Some(path) = e.get("path").and_then(Value::as_str).map(str::to_string) else {
            return Ok(());
        };
        let img = std::fs::read(&path)
            .ok()
            .and_then(|b| xs_image::encode::decode(&b).ok())
            .ok_or_else(|| bad("path", "it is not a PNG or JPG picture"))?;
        let name = store_picture(core, &id, &img, lang)?;
        if let Some(m) = e.as_object_mut() {
            m.remove("path");
            m.insert("source".into(), json!("image"));
            m.insert("image".into(), json!(name));
        }
        Ok(())
    };
    if let Some(e) = owned.get_mut("effect") {
        keep_picture(e)?;
    }
    if let Some(list) = owned.get_mut("effects").and_then(Value::as_array_mut) {
        for e in list.iter_mut() {
            keep_picture(e)?;
        }
    }
    let p = &owned;
    let checked = |e: &Value| -> Result<Value, CommandError> {
        let mut e = e.clone();
        if !e.is_object() {
            return Err(bad("effect", "an effect is an object with its type"));
        }
        xs_image::fx::check(&e).map_err(|r| bad("effect", &r))?;
        // A torn edge keeps its variant in the file (EDT-056).
        if e["type"] == "edges" && e.get("seed").is_none() {
            e["seed"] = json!(
                chrono::Utc::now()
                    .timestamp_nanos_opt()
                    .unwrap_or(7)
                    .unsigned_abs()
                    % 1_000_000_007
            );
        }
        if e.get("enabled").is_none() {
            e["enabled"] = json!(true);
        }
        Ok(e)
    };
    let stack = editor(core)
        .store
        .mutate(&id, "history.effects", |doc| {
            let list: &mut Vec<Value> = match &target {
                None => &mut doc.effects,
                Some(oid) => {
                    let o = doc
                        .objects
                        .iter_mut()
                        .find(|o| &o.id == oid)
                        .ok_or_else(|| bad("objectId", "there is no such object"))?;
                    if o.kind != "image" {
                        return Err(bad(
                            "objectId",
                            "only the document and floating images have effects",
                        ));
                    }
                    let v = o.props.entry("effects").or_insert_with(|| json!([]));
                    if !v.is_array() {
                        *v = json!([]);
                    }
                    v.as_array_mut().expect("an array")
                }
            };
            let at = |i: Option<usize>, len: usize| {
                i.filter(|&i| i < len)
                    .ok_or_else(|| bad("index", "there is no effect there"))
            };
            match action.as_str() {
                "set" => {
                    let all = p
                        .get("effects")
                        .and_then(Value::as_array)
                        .ok_or_else(|| bad("effects", "give the list"))?;
                    *list = all.iter().map(checked).collect::<Result<_, _>>()?;
                }
                "add" => {
                    let e = checked(p.get("effect").ok_or_else(|| bad("effect", "give the effect"))?)?;
                    list.push(e);
                }
                "update" => {
                    let i = at(index, list.len())?;
                    let changes = p
                        .get("effect")
                        .and_then(Value::as_object)
                        .ok_or_else(|| bad("effect", "give the changes"))?;
                    let mut e = list[i].clone();
                    for (k, v) in changes {
                        e[k] = v.clone();
                    }
                    list[i] = checked(&e)?;
                }
                "remove" => {
                    let i = at(index, list.len())?;
                    list.remove(i);
                }
                "move" => {
                    let i = at(index, list.len())?;
                    let to = p
                        .get("to")
                        .and_then(Value::as_u64)
                        .ok_or_else(|| bad("to", "give the new position"))?
                        as usize;
                    let e = list.remove(i);
                    list.insert(to.min(list.len()), e);
                }
                // Remove all: every effect off (EDT-053).
                "clear" => {
                    for e in list.iter_mut() {
                        e["enabled"] = json!(false);
                    }
                }
                other => return Err(bad("action", other)),
            }
            // One decorative edge per target (EDT-056).
            if list
                .iter()
                .filter(|e| e["type"] == "edges" && xs_image::fx::enabled(e))
                .count()
                > 1
            {
                return Err(bad("effect", "only one decorative edge at a time"));
            }
            Ok::<_, CommandError>(list.clone())
        })
        .ok_or_else(|| CommandError::internal("document closed"))??;
    changed(core, ctx, &id);
    Ok(json!({ "effects": stack }))
}

/// The original pixels with the edits of pixels (ADR 0005), without the flattened
/// layers: what resizing and turning work on (ADR 0006).
fn pixels_with_patches(doc: &Document, base: &Base) -> image::RgbaImage {
    let view = Document {
        flattened: Vec::new(),
        ..doc.clone()
    };
    xs_image::render::image_in_base(&view, base)
}

/// `document.resize`, `document.rotate`, `document.revert`, `document.canvas` and
/// `document.canvas_size` (EDT-069, EDT-070, EDT-072, EDT-074 to EDT-077).
fn image_command(core: &Core, ctx: &Context, p: &Value, which: &str) -> CommandResult {
    let lang = ctx.lang;
    let id = resolve(core, lang, p)?;
    let bad = |param: &str, reason: &str| {
        invalid(tr(
            lang,
            "errors.invalid_param",
            &[("param", param), ("reason", reason)],
        ))
    };
    let ed = editor(core);
    let doc = ed
        .store
        .with(&id, |d| d.document.clone())
        .ok_or_else(|| CommandError::internal("document closed"))?;
    let measure = |props: &Map<String, Value>| {
        let t = xs_image::text::Text::from_props(props).ok()?;
        let l = xs_image::text::layout(&t, doc.dpi);
        Some((f64::from(l.width), f64::from(l.height)))
    };
    let num = |k: &str| p.get(k).and_then(Value::as_f64);
    // The new document, and the new original pixels when they change.
    let mut next = doc.clone();
    let mut pixels: Option<image::RgbaImage> = None;
    // Crop to canvas: annotations left sticking out (EDT-051).
    let mut sticking: Vec<String> = Vec::new();
    // Cut out: objects across the strip, which stay (EDT-052).
    let mut crossing: Vec<String> = Vec::new();
    let label = match which {
        "resize" => {
            let (cw, ch) = (doc.canvas.width as f64, doc.canvas.height as f64);
            let (kx, ky) = match (num("percent"), num("width"), num("height")) {
                (Some(pc), _, _) => (pc / 100.0, pc / 100.0),
                (None, Some(w), Some(h)) => (w / cw, h / ch),
                (None, Some(w), None) => (w / cw, w / cw),
                (None, None, Some(h)) => (h / ch, h / ch),
                _ => return Err(bad("width", "give width, height or percent")),
            };
            if !(kx > 0.0
                && ky > 0.0
                && xs_image::encode::within_limits((kx * cw).round() as u32, (ky * ch).round() as u32))
            {
                return Err(bad(
                    "width",
                    "up to 32000 px per side and 500 megapixels (APP-050)",
                ));
            }
            if let Some(dpi) = num("dpi") {
                next.dpi = dpi.clamp(1.0, 10_000.0);
            }
            // Without resampling only the resolution changes (EDT-069).
            if p.get("resampleImage").and_then(Value::as_bool) == Some(false) {
                if num("dpi").is_none() {
                    next.dpi = (doc.dpi / kx).clamp(1.0, 10_000.0);
                }
            } else {
                let how =
                    image_ops::Resample::parse(p.get("resample").and_then(Value::as_str).unwrap_or("auto"))
                        .ok_or_else(|| bad("resample", "auto, bicubic, bilinear, lanczos or nearest"))?;
                let base = ed.store.base(&id).map_err(|e| internal(lang, "", e))?;
                let src = pixels_with_patches(&doc, &base);
                pixels = Some(image_ops::resize(&mut next, &src, kx, ky, how, &measure));
            }
            "history.resize"
        }
        "rotate" => {
            let degrees = num("degrees").ok_or_else(|| bad("degrees", "give the angle"))?;
            if !(-360.0..=360.0).contains(&degrees) {
                return Err(bad("degrees", "between -359.9 and 359.9"));
            }
            let background = match p.get("background").and_then(Value::as_str) {
                Some("canvas") => doc
                    .canvas
                    .color
                    .as_deref()
                    .and_then(xs_image::color::Rgba::parse)
                    .map(|c| [c.r, c.g, c.b, c.a]),
                _ => None,
            };
            let base = ed.store.base(&id).map_err(|e| internal(lang, "", e))?;
            let src = pixels_with_patches(&doc, &base);
            pixels = Some(image_ops::rotate(&mut next, &src, degrees, background, &measure));
            "history.rotate"
        }
        "revert" => {
            // The image as captured or opened (EDT-076): base.png, no edits.
            let (bytes, size) = ed
                .store
                .with(&id, |d| (d.base_source.bytes.clone(), d.base_size))
                .ok_or_else(|| CommandError::internal("document closed"))?;
            let dpi = xs_image::encode::png_dpi(&bytes).unwrap_or(doc.dpi);
            let opaque = xs_image::encode::decode(&bytes)
                .map(|i| i.pixels().all(|p| p.0[3] == 255))
                .unwrap_or(true);
            next = Document::new(size.0, size.1, dpi);
            next.canvas.color = opaque.then(|| "#FFFFFF".to_string());
            next.original_changed = doc.original_changed;
            "history.revert"
        }
        "canvas" => {
            if let Some(c) = p.get("color") {
                next.canvas.gradient = None;
                next.canvas.color = match c {
                    Value::Null => None,
                    Value::String(s) => {
                        xs_image::color::Rgba::parse(s)
                            .ok_or_else(|| bad("color", "use #RRGGBB, #RRGGBBAA, a gradient or null"))?;
                        Some(s.clone())
                    }
                    // A gradient (EDT-020, EDT-074); the color keeps its first stop.
                    Value::Object(_) => {
                        let fill: xs_image::paint::Fill =
                            serde_json::from_value(c.clone()).map_err(|e| bad("color", &e.to_string()))?;
                        fill.check("color").map_err(|e| bad("color", &e))?;
                        let first = fill
                            .main_color()
                            .ok_or_else(|| bad("color", "a gradient needs colors"))?;
                        next.canvas.gradient = Some(c.clone());
                        Some(first.to_hex())
                    }
                    _ => return Err(bad("color", "use #RRGGBB, #RRGGBBAA, a gradient or null")),
                };
            }
            match p.get("cornerRadius") {
                Some(Value::String(s)) if s == "circle" => {
                    // A circle: first a centered square (EDT-075).
                    let (w, h) = (doc.canvas.width as i64, doc.canvas.height as i64);
                    let side = w.min(h);
                    let (x0, y0) = ((w - side) / 2, (h - side) / 2);
                    let base_size = ed.store.with(&id, |d| d.base_dims()).unwrap_or((1, 1));
                    crop_to(&mut next, base_size, [x0, y0, x0 + side, y0 + side])
                        .ok_or_else(|| bad("cornerRadius", "nothing would be left"))?;
                    next.canvas.corner_radius = side as f64 / 2.0;
                }
                Some(v) => {
                    let r = v
                        .as_f64()
                        .ok_or_else(|| bad("cornerRadius", "a number of px or \"circle\""))?;
                    let max = doc.canvas.width.min(doc.canvas.height) as f64 / 2.0;
                    next.canvas.corner_radius = r.clamp(0.0, max);
                }
                None => {}
            }
            if let Some(dpi) = num("dpi") {
                if !(1.0..=10_000.0).contains(&dpi) {
                    return Err(bad("dpi", "between 1 and 10000"));
                }
                next.dpi = dpi;
            }
            "history.canvas"
        }
        "flip" => {
            let horizontal = match p.get("direction").and_then(Value::as_str) {
                Some("vertical") => false,
                Some("horizontal") | None => true,
                Some(other) => return Err(bad("direction", other)),
            };
            let picked: Vec<String> = p
                .get("objectIds")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
                .unwrap_or_default();
            if picked.is_empty() {
                // The whole image and every object (EDT-073).
                let base = ed.store.base(&id).map_err(|e| internal(lang, "", e))?;
                let src = pixels_with_patches(&doc, &base);
                pixels = Some(image_ops::flip(&mut next, &src, horizontal, &measure));
            } else {
                // Object > Flip: only those, around the box that holds them.
                let objs: Vec<&Object> = doc.objects.iter().filter(|o| picked.contains(&o.id)).collect();
                if objs.len() != picked.len() {
                    return Err(bad("objectIds", "some of them do not exist"));
                }
                let b = transform::bounds(objs.iter().copied(), &measure)
                    .ok_or_else(|| bad("objectIds", "they have no size"))?;
                let axis = if horizontal {
                    b.x + b.w / 2.0
                } else {
                    b.y + b.h / 2.0
                };
                for o in next.objects.iter_mut().filter(|o| picked.contains(&o.id)) {
                    image_ops::mirror(o, axis, horizontal, &measure);
                }
            }
            // Floating images flip their pictures too.
            let flips: Vec<(String, String)> = next
                .objects
                .iter()
                .filter(|o| o.kind == "image" && (picked.is_empty() || picked.contains(&o.id)))
                .filter_map(|o| Some((o.id.clone(), o.props.get("image")?.as_str()?.to_string())))
                .collect();
            for (oid, name) in flips {
                let Some(img) = ed
                    .store
                    .image_file(&id, &name)
                    .and_then(|b| xs_image::encode::decode(&b).ok())
                else {
                    continue;
                };
                let flipped = if horizontal {
                    image::imageops::flip_horizontal(&img)
                } else {
                    image::imageops::flip_vertical(&img)
                };
                let new = store_picture(core, &id, &flipped, lang)?;
                if let Some(o) = next.object_mut(&oid) {
                    o.props.insert("image".into(), json!(new));
                }
            }
            "history.flip"
        }
        "trim" => {
            // Uniform borders around what the canvas shows (EDT-051).
            let tolerance = num("tolerance").unwrap_or(2.0);
            if !(0.0..=20.0).contains(&tolerance) {
                return Err(bad("tolerance", "between 0 and 20"));
            }
            let snap = ed.store.snapshot(&id).map_err(|e| internal(lang, "", e))?;
            let shown = xs_image::render(&snap.document, &snap.base);
            let whole = [0, 0, shown.width(), shown.height()];
            let found = xs_image::fx::trim_bounds(&shown, tolerance).filter(|b| *b != whole);
            let Some([x0, y0, x1, y1]) = found else {
                // Nothing to trim: said, not an error (EDT-051).
                return Ok(json!({
                    "changed": false,
                    "width": doc.canvas.width,
                    "height": doc.canvas.height,
                    "message": tr(lang, "editor.no_borders", &[]),
                }));
            };
            let base_size = ed.store.with(&id, |d| d.base_dims()).unwrap_or((1, 1));
            crop_to(&mut next, base_size, [x0 as i64, y0 as i64, x1 as i64, y1 as i64])
                .ok_or_else(|| bad("tolerance", "nothing would be left"))?;
            "history.trim"
        }
        "fit_canvas" => {
            // The original pixels and every object, nothing outside (EDT-051).
            let (bw, bh) = ed.store.with(&id, |d| d.base_dims()).unwrap_or((1, 1));
            let c = doc.crop;
            let mut e = [
                -c.x as f64,
                -c.y as f64,
                (bw as i64 - c.x) as f64,
                (bh as i64 - c.y) as f64,
            ];
            let flat = xs_image::render::flattened_objects(&doc);
            for o in doc.objects.iter().chain(flat.iter()) {
                if let Some(b) = xs_image::object_bounds(o, doc.dpi) {
                    e = [e[0].min(b[0]), e[1].min(b[1]), e[2].max(b[2]), e[3].max(b[3])];
                }
            }
            // Less than half a pixel out is inside, as when the canvas grows (EDT-071).
            let x0 = (e[0] + 0.5).floor() as i64;
            let y0 = (e[1] + 0.5).floor() as i64;
            let x1 = (e[2] - 0.5).ceil() as i64;
            let y1 = (e[3] - 0.5).ceil() as i64;
            let (w, h) = (doc.canvas.width as i64, doc.canvas.height as i64);
            if (x0, y0, x1, y1) == (0, 0, w, h) || x1 <= x0 || y1 <= y0 {
                return Ok(json!({"changed": false, "width": w, "height": h}));
            }
            next.crop_history.push(c);
            next.crop = xs_core::document::PixelRect {
                x: c.x + x0,
                y: c.y + y0,
                width: (x1 - x0) as u32,
                height: (y1 - y0) as u32,
            };
            for o in next.objects.iter_mut() {
                objects::translate(o, -x0 as f64, -y0 as f64);
            }
            next.canvas.width = next.crop.width;
            next.canvas.height = next.crop.height;
            next.frame = None;
            next.outside.clear();
            "history.fit_canvas"
        }
        "crop_to_canvas" => {
            // What sticks out of the canvas goes: floating images lose that part of
            // their picture; annotations stay as they are and are reported (EDT-051).
            let (w, h) = (doc.canvas.width as f64, doc.canvas.height as f64);
            let inside = |b: [f64; 4]| b[0] > -0.5 && b[1] > -0.5 && b[2] < w + 0.5 && b[3] < h + 0.5;
            let mut cut = 0;
            let mut gone: Vec<String> = Vec::new();
            for o in next.objects.iter_mut() {
                let Some(b) = xs_image::object_bounds(o, doc.dpi) else {
                    continue;
                };
                if inside(b) || o.kind == "blur" || o.kind == "spotlight" {
                    continue;
                }
                let turned = o.props.get("rotation").and_then(Value::as_f64).unwrap_or(0.0) != 0.0;
                if o.kind != "image" || turned {
                    sticking.push(o.id.clone());
                    continue;
                }
                let r = &o.props["rect"];
                let rect = [
                    r["x"].as_f64().unwrap_or(0.0),
                    r["y"].as_f64().unwrap_or(0.0),
                    r["w"].as_f64().unwrap_or(0.0),
                    r["h"].as_f64().unwrap_or(0.0),
                ];
                let (vx0, vy0) = (rect[0].max(0.0), rect[1].max(0.0));
                let (vx1, vy1) = ((rect[0] + rect[2]).min(w), (rect[1] + rect[3]).min(h));
                if vx1 - vx0 < 0.5 || vy1 - vy0 < 0.5 {
                    // Wholly outside: nothing of it is left.
                    gone.push(o.id.clone());
                    continue;
                }
                let Some(name) = o.props.get("image").and_then(Value::as_str) else {
                    continue;
                };
                let Some(pic) = ed
                    .store
                    .image_file(&id, name)
                    .and_then(|b| xs_image::encode::decode(&b).ok())
                else {
                    continue;
                };
                let (kx, ky) = (pic.width() as f64 / rect[2], pic.height() as f64 / rect[3]);
                let px0 = (((vx0 - rect[0]) * kx).round() as u32).min(pic.width() - 1);
                let py0 = (((vy0 - rect[1]) * ky).round() as u32).min(pic.height() - 1);
                let px1 = (((vx1 - rect[0]) * kx).round() as u32).clamp(px0 + 1, pic.width());
                let py1 = (((vy1 - rect[1]) * ky).round() as u32).clamp(py0 + 1, pic.height());
                let part = image::imageops::crop_imm(&pic, px0, py0, px1 - px0, py1 - py0).to_image();
                let new = store_picture(core, &id, &part, lang)?;
                o.props.insert("image".into(), json!(new));
                o.props.insert(
                    "rect".into(),
                    json!({"x": vx0, "y": vy0, "w": vx1 - vx0, "h": vy1 - vy0}),
                );
                cut += 1;
            }
            next.objects.retain(|o| !gone.contains(&o.id));
            next.outside.retain(|id| !gone.contains(id));
            cut += gone.len();
            if cut == 0 {
                return Ok(json!({
                    "changed": false,
                    "width": doc.canvas.width,
                    "height": doc.canvas.height,
                    "sticking": sticking,
                    "message": tr(lang, "editor.nothing_to_crop", &[("count", &sticking.len().to_string())]),
                }));
            }
            "history.crop_to_canvas"
        }
        "cut_out" => {
            // Cut out a strip, or add space (EDT-052): new original pixels (ADR 0006).
            use xs_image::cutout::{self, Join, Strip};
            let across = match p.get("direction").and_then(Value::as_str) {
                Some("horizontal") | None => true,
                Some("vertical") => false,
                Some(other) => return Err(bad("direction", other)),
            };
            let adding = match p.get("mode").and_then(Value::as_str) {
                Some("remove") | None => false,
                Some("add") => true,
                Some(other) => return Err(bad("mode", other)),
            };
            let join = Join::parse(p.get("join").and_then(Value::as_str).unwrap_or("straight"))
                .ok_or_else(|| bad("join", "straight, torn, wavy or zigzag"))?;
            let gap = num("gap").unwrap_or(16.0);
            if !(8.0..=60.0).contains(&gap) {
                return Err(bad("gap", "between 8 and 60"));
            }
            let (a, b) = match (num("from"), num("to")) {
                (Some(a), Some(b)) => (a.min(b), a.max(b)),
                _ => return Err(bad("from", "give from and to, in canvas pixels")),
            };
            let base = ed.store.base(&id).map_err(|e| internal(lang, "", e))?;
            let src = pixels_with_patches(&doc, &base);
            let c = doc.crop;
            let (off, len, shown) = if across {
                (c.y, src.height() as i64, c.height as i64)
            } else {
                (c.x, src.width() as i64, c.width as i64)
            };
            // In base pixels, inside the image and the canvas.
            let (lo, hi) = (off.max(0), (off + shown).min(len));
            let f = (a.round() as i64 + off).clamp(lo, hi);
            let t = (b.round() as i64 + off).clamp(lo, hi);
            if t - f < 1 {
                return Err(bad("to", "the strip is empty"));
            }
            let strip = if across { Strip::Across } else { Strip::Down };
            let new = if adding {
                cutout::add(&src, strip, f as u32, (t - f) as u32)
            } else {
                let seed = ((f as u64) << 32) ^ (t as u64);
                cutout::cut(&src, strip, f as u32, t as u32, join, gap as u32, seed)
            };
            let new_len = if across { new.height() } else { new.width() } as i64;
            // How far what comes after the strip moves along the axis.
            let shift = (new_len - len) as f64;
            // Objects wholly after the strip move with their part; those across it stay
            // and are reported (EDT-052). `start` and `end` in the same pixels as `o`.
            let place = |o: &mut Object, start: f64, end: f64, crossing: &mut Vec<String>| {
                let Some(bx) = xs_image::object_bounds(o, doc.dpi) else {
                    return;
                };
                let (o0, o1) = if across { (bx[1], bx[3]) } else { (bx[0], bx[2]) };
                let after = if adding { start } else { end };
                if o0 >= after - 0.5 {
                    let (dx, dy) = if across { (0.0, shift) } else { (shift, 0.0) };
                    objects::translate(o, dx, dy);
                } else if o1 > start + 0.5 {
                    crossing.push(o.id.clone());
                }
            };
            let (cf, ct) = ((f - off) as f64, (t - off) as f64);
            for o in next.objects.iter_mut() {
                place(o, cf, ct, &mut crossing);
            }
            let mut flat_crossing = Vec::new();
            for o in next.flattened.iter_mut().flat_map(|l| l.objects.iter_mut()) {
                place(o, f as f64, t as f64, &mut flat_crossing);
            }
            if across {
                next.crop.height = (next.crop.height as i64 + shift as i64).max(1) as u32;
            } else {
                next.crop.width = (next.crop.width as i64 + shift as i64).max(1) as u32;
            }
            next.canvas.width = next.crop.width;
            next.canvas.height = next.crop.height;
            next.crop_history.clear();
            next.frame = None;
            next.patches.clear();
            next.outside = fit::outside_canvas(&next);
            pixels = Some(new);
            if adding {
                "history.add_space"
            } else {
                "history.cut_out"
            }
        }
        "canvas_size" => {
            let (w, h) = (
                num("width").unwrap_or(doc.canvas.width as f64),
                num("height").unwrap_or(doc.canvas.height as f64),
            );
            if w < 1.0 || h < 1.0 || !xs_image::encode::within_limits(w.round() as u32, h.round() as u32) {
                return Err(bad(
                    "width",
                    "1 px to 32000 px per side and 500 megapixels (APP-050)",
                ));
            }
            let anchor = p.get("anchor").and_then(Value::as_str).unwrap_or("center");
            image_ops::canvas_size(&mut next, w.round() as u32, h.round() as u32, anchor);
            "history.canvas_size"
        }
        other => return Err(bad("command", other)),
    };
    let stored = match &pixels {
        Some(img) => Some(store_picture(core, &id, img, lang)?),
        None => None,
    };
    ed.store
        .mutate(&id, label, |d| {
            *d = next.clone();
            if let Some(name) = &stored {
                d.base_image = Some(name.clone());
            }
            Ok::<_, CommandError>(())
        })
        .ok_or_else(|| CommandError::internal("document closed"))??;
    // A selection of pixels no longer fits new pixels.
    if pixels.is_some() || which == "revert" {
        ed.store.with(&id, |d| d.pixel_selection = None);
    }
    changed(core, ctx, &id);
    let mut info = ed
        .store
        .with(&id, |d| json!({"width": d.document.canvas.width, "height": d.document.canvas.height, "dpi": d.document.dpi}))
        .unwrap_or_default();
    info["changed"] = json!(true);
    if !crossing.is_empty() {
        info["crossing"] = json!(crossing);
        info["warnings"] = json!([tr(
            lang,
            "editor.crossing_strip",
            &[("count", &crossing.len().to_string())]
        )]);
    }
    if !sticking.is_empty() {
        info["sticking"] = json!(sticking);
        info["warnings"] = json!([tr(
            lang,
            "editor.still_sticking",
            &[("count", &sticking.len().to_string())]
        )]);
    }
    Ok(info)
}

/// Crops `doc` to `[x0, y0, x1, y1]` of its canvas (EDT-049), keeping the objects on
/// the image. `None` if nothing would be left. The new canvas size.
fn crop_to(doc: &mut Document, base: (u32, u32), r: [i64; 4]) -> Option<(u32, u32)> {
    // Non-destructive (EDT-050): the canvas is a window over the original pixels, and
    // can grow back up to their edges, or to the canvas grown for objects (EDT-071).
    let old = doc.crop;
    let (ex0, ey0) = (old.x.min(0), old.y.min(0));
    let ex1 = (old.x + old.width as i64).max(base.0 as i64);
    let ey1 = (old.y + old.height as i64).max(base.1 as i64);
    let nx0 = (old.x + r[0]).clamp(ex0, ex1);
    let ny0 = (old.y + r[1]).clamp(ey0, ey1);
    let nx1 = (old.x + r[2]).clamp(ex0, ex1);
    let ny1 = (old.y + r[3]).clamp(ey0, ey1);
    if nx1 - nx0 < 1 || ny1 - ny0 < 1 {
        return None;
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
    // The crop is what the user chose; what it leaves outside stays there.
    doc.frame = None;
    doc.outside = fit::outside_canvas(doc);
    Some((doc.canvas.width, doc.canvas.height))
}

/// The pixels of a floating image's picture.
fn picture_pixels(
    core: &Core,
    id: &str,
    doc: &Document,
    oid: &str,
    lang: Lang,
) -> Result<image::RgbaImage, CommandError> {
    let object = doc.objects.iter().find(|o| o.id == oid).ok_or_else(|| {
        CommandError::new(
            ErrorCode::NotFound,
            tr(lang, "errors.object_not_found", &[("id", oid)]),
        )
    })?;
    let name = match (
        object.kind.as_str(),
        object.props.get("image").and_then(Value::as_str),
    ) {
        ("image", Some(name)) => name.to_string(),
        _ => {
            return Err(invalid(tr(
                lang,
                "errors.invalid_param",
                &[
                    ("param", "objectId"),
                    ("reason", "only a floating image has pixels"),
                ],
            )))
        }
    };
    editor(core)
        .store
        .image_file(id, &name)
        .and_then(|bytes| xs_image::encode::decode(&bytes).ok())
        .ok_or_else(|| CommandError::internal("picture missing"))
}

/// Stores `img` in the document's `images/` folder under a name of its own.
fn store_picture(core: &Core, id: &str, img: &image::RgbaImage, lang: Lang) -> Result<String, CommandError> {
    let png = xs_image::encode::png(img, 0.0, true).map_err(|e| internal(lang, "", e))?;
    let name = format!("{}{:016x}.png", store::INSERTED, content_hash(&png));
    editor(core)
        .store
        .add_images(id, &BTreeMap::from([(name.clone(), png)]))
        .map_err(|e| internal(lang, "", e))?;
    Ok(name)
}

/// `document.select_pixels` (EDT-043, EDT-046): draws, combines, inverts or drops the
/// pixel selection.
fn select_pixels(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    use xs_image::selection::{Combine, Shape};
    let lang = ctx.lang;
    let id = resolve(core, lang, p)?;
    let bad = |param: &str, reason: &str| {
        invalid(tr(
            lang,
            "errors.invalid_param",
            &[("param", param), ("reason", reason)],
        ))
    };
    let ed = editor(core);
    let current = ed.store.with(&id, |d| d.pixel_selection.clone()).flatten();
    let shape = p.get("shape").and_then(Value::as_str).unwrap_or("rectangle");
    let done = |sel: Option<pixels::PixelSelection>| -> CommandResult {
        let info = ed
            .store
            .with(&id, |d| {
                d.pixel_selection = sel.filter(|s| !s.mask.is_empty());
                d.pixel_selection.as_ref().and_then(|s| s.info(&d.document))
            })
            .flatten();
        emit_document(core, &id);
        Ok(json!({ "bounds": info.as_ref().map(|i| i["bounds"].clone()) }))
    };
    match shape {
        "none" => return done(None),
        "invert" => {
            let cur = current.ok_or_else(|| bad("shape", "there is no selection to invert"))?;
            return done(Some(pixels::PixelSelection::new(
                cur.target.clone(),
                cur.mask.inverted(),
                None,
            )));
        }
        _ => {}
    }
    let target = p.get("objectId").and_then(Value::as_str).map(str::to_string);
    let doc = ed
        .store
        .with(&id, |d| d.document.clone())
        .ok_or_else(|| CommandError::internal("document closed"))?;
    let base = ed.store.base(&id).map_err(|e| internal(lang, "", e))?;
    let picture = match &target {
        Some(oid) => Some(picture_pixels(core, &id, &doc, oid, lang)?),
        None => None,
    };
    let pixels = pixels::target_pixels(&doc, &base, target.as_deref(), picture)
        .ok_or_else(|| CommandError::internal("no pixels"))?;
    let to_target = pixels::to_canvas(&doc, target.as_deref(), pixels.width(), pixels.height())
        .and_then(|t| t.invert())
        .ok_or_else(|| bad("objectId", "this floating image has no size"))?;
    let point = |v: &Value| Some([v.get(0)?.as_f64()?, v.get(1)?.as_f64()?]);
    let rect = || -> Result<[f64; 4], CommandError> {
        let r = p.get("rect").ok_or_else(|| bad("rect", "give {x, y, w, h}"))?;
        let n = |k: &str| r.get(k).and_then(Value::as_f64).unwrap_or(0.0);
        let (x, y, w, h) = (n("x"), n("y"), n("w"), n("h"));
        // A box dragged up or left.
        Ok([x.min(x + w), y.min(y + h), w.abs(), h.abs()])
    };
    let mut drawn: Option<(String, [f64; 4])> = None;
    let part = match shape {
        "rectangle" | "ellipse" => {
            let mut r = rect()?;
            // Snap to object (EDT-043), on the image.
            if p.get("snap").and_then(Value::as_bool).unwrap_or(false) && target.is_none() {
                let level = p.get("level").and_then(Value::as_f64).unwrap_or(50.0);
                let (cx, cy) = (doc.crop.x as f64, doc.crop.y as f64);
                let inside = [
                    (r[0] + cx).max(0.0).floor() as u32,
                    (r[1] + cy).max(0.0).floor() as u32,
                    r[2].ceil().max(0.0) as u32,
                    r[3].ceil().max(0.0) as u32,
                ];
                if let Some(b) = xs_image::selection::snap(&pixels, inside, level) {
                    r = [b[0] as f64 - cx, b[1] as f64 - cy, b[2] as f64, b[3] as f64];
                }
            }
            drawn = Some((shape.to_string(), r));
            pixels::Part::Shape(if shape == "ellipse" {
                Shape::Ellipse(r)
            } else {
                Shape::Rectangle(r)
            })
        }
        "polygon" | "freehand" => {
            let points: Vec<[f64; 2]> = p
                .get("points")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(point).collect())
                .unwrap_or_default();
            if points.len() < 3 {
                return Err(bad("points", "give at least three [x, y]"));
            }
            pixels::Part::Shape(Shape::Polygon(points))
        }
        "wand" => pixels::Part::Color {
            at: p
                .get("at")
                .and_then(point)
                .ok_or_else(|| bad("at", "give [x, y]"))?,
            tolerance: p.get("tolerance").and_then(Value::as_f64).unwrap_or(15.0),
            global: p.get("global").and_then(Value::as_bool).unwrap_or(false),
        },
        "all" => pixels::Part::All,
        other => return Err(bad("shape", other)),
    };
    let antialias = p.get("antialias").and_then(Value::as_bool).unwrap_or(true);
    let mut mask = pixels::part_mask(&part, &pixels, to_target, antialias);
    let feather = p.get("feather").and_then(Value::as_f64).unwrap_or(0.0);
    if feather > 0.0 {
        mask = mask.feathered(feather);
    }
    let how = match p.get("mode").and_then(Value::as_str) {
        Some("add") => Combine::Add,
        Some("subtract") => Combine::Subtract,
        _ => Combine::Replace,
    };
    let joined = match current.filter(|c| c.target == target && how != Combine::Replace) {
        Some(c) => {
            drawn = None;
            c.mask.combine(&mask, how)
        }
        None if how == Combine::Subtract => Mask::empty(mask.width, mask.height),
        None => mask,
    };
    done(Some(pixels::PixelSelection::new(target, joined, drawn)))
}

/// `document.selection_action` (EDT-044, EDT-045): what to do with the selected pixels.
fn selection_action(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    use xs_image::selection::Gap;
    let lang = ctx.lang;
    let id = resolve(core, lang, p)?;
    let bad = |param: &str, reason: &str| {
        invalid(tr(
            lang,
            "errors.invalid_param",
            &[("param", param), ("reason", reason)],
        ))
    };
    let ed = editor(core);
    let sel = ed
        .store
        .with(&id, |d| d.pixel_selection.clone())
        .flatten()
        .ok_or_else(|| bad("documentId", "there is no pixel selection"))?;
    let action = p.get("action").and_then(Value::as_str).unwrap_or_default();
    let doc = ed
        .store
        .with(&id, |d| d.document.clone())
        .ok_or_else(|| CommandError::internal("document closed"))?;
    let gap = match p.get("gapFill").and_then(Value::as_str) {
        Some("transparent") => Gap::Transparent,
        Some("canvas") => doc
            .canvas
            .color
            .as_deref()
            .and_then(xs_image::color::Rgba::parse)
            .map(Gap::Color)
            .unwrap_or(Gap::Transparent),
        _ if p.get("quality").and_then(Value::as_str) == Some("photo") => Gap::Photo,
        _ => Gap::Smart,
    };
    let base = ed.store.base(&id).map_err(|e| internal(lang, "", e))?;
    let picture = match &sel.target {
        Some(oid) => Some(picture_pixels(core, &id, &doc, oid, lang)?),
        None => None,
    };
    let pixels = pixels::target_pixels(&doc, &base, sel.target.as_deref(), picture)
        .ok_or_else(|| CommandError::internal("no pixels"))?;
    let mask = sel.mask.as_ref();
    if (pixels.width(), pixels.height()) != (mask.width, mask.height) {
        return Err(bad("documentId", "the selection no longer fits the image"));
    }
    let base_only = || {
        if sel.target.is_some() {
            Err(bad("action", "this works on the image, not on a floating image"))
        } else {
            Ok(())
        }
    };
    // Copy, and the first half of Cut (EDT-044).
    if action == "copy" || action == "cut" {
        let (cut, _) = xs_image::selection::selected_pixels(&pixels, mask)
            .ok_or_else(|| bad("documentId", "nothing is selected"))?;
        copy_image(&cut).map_err(|e| internal(lang, "", e))?;
        if action == "copy" {
            return Ok(json!({ "changed": false }));
        }
    }
    let mut next = Some(sel.clone());
    let mut made: Vec<String> = Vec::new();
    // A Photo fill can take a while: after 300 ms the editor shows its progress, and
    // `editor.cancel` stops it (EDT-045).
    let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    *ed.filling.lock().unwrap() = Some(stop.clone());
    let started = Instant::now();
    let mut shown = Instant::now();
    let mut go_on = |done: f32| {
        if started.elapsed() >= Duration::from_millis(300)
            && (shown.elapsed() >= Duration::from_millis(100) || done >= 1.0)
        {
            shown = Instant::now();
            core.host()
                .editor_event("progress", json!({ "task": "fill", "done": done }));
        }
        !stop.load(std::sync::atomic::Ordering::Relaxed)
    };
    // The fill is over, done or stopped: the progress goes away.
    let end = || {
        *ed.filling.lock().unwrap() = None;
        if started.elapsed() >= Duration::from_millis(300) {
            core.host()
                .editor_event("progress", json!({ "task": "fill", "done": null }));
        }
    };
    let stopped =
        |_: pixels::Stopped| CommandError::new(ErrorCode::Cancelled, tr(lang, "errors.fill_stopped", &[]));
    let (changed_px, label) = match action {
        "delete" | "cut" => (
            {
                let r = pixels::delete(&pixels, mask, gap, &mut go_on);
                end();
                r.map_err(stopped)?
            },
            if action == "cut" {
                "history.cut_pixels"
            } else {
                "history.delete_pixels"
            },
        ),
        "move" => {
            let off = p
                .get("offset")
                .and_then(|v| Some([v.get(0)?.as_f64()?, v.get(1)?.as_f64()?]))
                .ok_or_else(|| bad("offset", "give [dx, dy]"))?;
            let to_target = pixels::to_canvas(&doc, sel.target.as_deref(), mask.width, mask.height)
                .and_then(|t| t.invert())
                .ok_or_else(|| CommandError::internal("no transform"))?;
            let mut v = [
                xs_image::selection::Point::from_xy(0.0, 0.0),
                xs_image::selection::Point::from_xy(off[0] as f32, off[1] as f32),
            ];
            to_target.map_points(&mut v);
            let (dx, dy) = ((v[1].x - v[0].x).round() as i64, (v[1].y - v[0].y).round() as i64);
            let r = pixels::shift(&pixels, mask, dx, dy, gap, &mut go_on);
            end();
            match r.map_err(stopped)? {
                Some((c, moved)) => {
                    let shape = sel
                        .shape
                        .clone()
                        .map(|(k, r)| (k, [r[0] + off[0], r[1] + off[1], r[2], r[3]]));
                    next = Some(pixels::PixelSelection::new(sel.target.clone(), moved, shape));
                    (Some(c), "history.move_pixels")
                }
                None => (None, "history.move_pixels"),
            }
        }
        "effect" => {
            // Fixed into the selected pixels (EDT-053): filters, color, replace and
            // the spotlight with lens (EDT-065).
            let effect = p
                .get("effect")
                .cloned()
                .ok_or_else(|| bad("effect", "give the effect"))?;
            let kind = effect.get("type").and_then(Value::as_str).unwrap_or_default();
            if !["filters", "color", "replace", "spotlight"].contains(&kind) {
                return Err(bad(
                    "effect",
                    "only filters, color, replace and spotlight work on a selection",
                ));
            }
            xs_image::fx::check(&effect).map_err(|r| bad("effect", &r))?;
            (
                pixels::selection_effect(&pixels, mask, &effect),
                "history.effect_selection",
            )
        }
        "float" | "crop" => {
            base_only()?;
            next = None;
            (
                None,
                if action == "float" {
                    "history.float_pixels"
                } else {
                    "history.crop_selection"
                },
            )
        }
        other => return Err(bad("action", other)),
    };
    // The new pixels, stored before the step that shows them.
    let stored = match (&changed_px, &sel.target) {
        (Some(c), None) => Some(store_picture(
            core,
            &id,
            &pixels::cut_area(&c.pixels, c.area),
            lang,
        )?),
        (Some(c), Some(_)) => Some(store_picture(core, &id, &c.pixels, lang)?),
        (None, _) => None,
    };
    // Float: the pixels as a new floating image where they were, and the gap filled.
    let mut float: Option<(String, Map<String, Value>)> = None;
    let mut float_gap: Option<(String, PixelRect)> = None;
    let mut crop: Option<[i64; 4]> = None;
    let mut crop_patch: Option<(String, PixelRect)> = None;
    if action == "float" || action == "crop" {
        let (cut, at) = xs_image::selection::selected_pixels(&pixels, mask)
            .ok_or_else(|| bad("documentId", "nothing is selected"))?;
        let (cx, cy) = (doc.crop.x, doc.crop.y);
        if action == "float" {
            let name = store_picture(core, &id, &cut, lang)?;
            let props = objects::image_object(
                &name,
                [
                    (at[0] as i64 - cx) as f64,
                    (at[1] as i64 - cy) as f64,
                    at[2] as f64,
                    at[3] as f64,
                ],
            )
            .map_err(invalid)?;
            float = Some((name, props));
            let r = pixels::delete(&pixels, mask, gap, &mut go_on);
            end();
            if let Some(c) = r.map_err(stopped)? {
                let gap_name = store_picture(core, &id, &pixels::cut_area(&c.pixels, c.area), lang)?;
                float_gap = Some((gap_name, pixels::rect_of(c.area)));
            }
        } else {
            crop = Some([
                at[0] as i64 - cx,
                at[1] as i64 - cy,
                (at[0] + at[2]) as i64 - cx,
                (at[1] + at[3]) as i64 - cy,
            ]);
            // Outside a shape that is not a rectangle, the pixels go (a circle crop).
            if mask.bounds().is_some_and(|b| {
                (b[1]..b[1] + b[3]).any(|y| (b[0]..b[0] + b[2]).any(|x| mask.at(x, y) < 255))
            }) {
                let name = store_picture(core, &id, &cut, lang)?;
                crop_patch = Some((name, pixels::rect_of(at)));
            }
        }
    }
    if changed_px.is_none() && float.is_none() && crop.is_none() {
        return Ok(json!({ "changed": false }));
    }
    let base_size = ed.store.with(&id, |d| d.base_dims()).unwrap_or((1, 1));
    let auto = auto_expand(core);
    ed.store
        .mutate(&id, label, |doc| {
            match (&sel.target, &changed_px, &stored) {
                (None, Some(c), Some(name)) => pixels::add_patch(doc, pixels::rect_of(c.area), name.clone()),
                (Some(oid), Some(_), Some(name)) => {
                    let o = doc
                        .objects
                        .iter_mut()
                        .find(|o| &o.id == oid)
                        .ok_or_else(|| CommandError::internal("object gone"))?;
                    o.props.insert("image".into(), json!(name));
                }
                _ => {}
            }
            if let Some((name, rect)) = &float_gap {
                pixels::add_patch(doc, *rect, name.clone());
            }
            if let Some((_, props)) = &float {
                let oid = doc.next_object_id();
                doc.objects.push(Object {
                    id: oid.clone(),
                    kind: "image".to_string(),
                    props: props.clone(),
                });
                made.push(oid);
                if auto {
                    fit::fit_canvas(doc);
                }
            }
            if let Some((name, rect)) = &crop_patch {
                pixels::add_patch(doc, *rect, name.clone());
                // Outside the shape stays transparent, not the canvas color: that is
                // how an image is cut to a circle (EDT-044, EDT-074).
                doc.canvas.color = None;
            }
            if let Some(r) = crop {
                crop_to(doc, base_size, r).ok_or_else(|| CommandError::internal("empty crop"))?;
            }
            Ok::<_, CommandError>(())
        })
        .ok_or_else(|| CommandError::internal("document closed"))??;
    ed.store.with(&id, |d| d.pixel_selection = next);
    changed(core, ctx, &id);
    Ok(json!({ "changed": true, "ids": made }))
}

/// The image's pixels as they show now, in base pixels (with every patch): what Edit
/// text reads (SMT-009).
pub fn image_pixels(core: &Core, id: &str) -> Result<image::RgbaImage, String> {
    let ed = editor(core);
    let doc = ed
        .store
        .with(id, |d| d.document.clone())
        .ok_or("document closed")?;
    let base = ed.store.base(id)?;
    pixels::target_pixels(&doc, &base, None, None).ok_or_else(|| "no pixels".to_string())
}

/// Edit text (SMT-009): the pixels of `mask` (base pixels, the text's) rebuilt from
/// what is around them, and `texts` added as Text objects, all in one undo step. The
/// pixel selection goes. Returns the new objects' IDs.
pub fn replace_text(
    core: &Core,
    ctx: &Context,
    id: &str,
    mask: &xs_image::selection::Mask,
    texts: &[Map<String, Value>],
) -> Result<Vec<String>, CommandError> {
    let lang = ctx.lang;
    let ed = editor(core);
    let pixels = image_pixels(core, id).map_err(|e| internal(lang, "", e))?;
    if (pixels.width(), pixels.height()) != (mask.width, mask.height) {
        return Err(CommandError::internal("the mask does not fit the image"));
    }
    let mut out = pixels.clone();
    xs_image::selection::fill_gap_with(&mut out, mask, xs_image::selection::Gap::Smart, &mut |_| true);
    let patch = match mask.bounds() {
        Some(area) => Some((
            store_picture(core, id, &pixels::cut_area(&out, area), lang)?,
            pixels::rect_of(area),
        )),
        None => None,
    };
    let mut checked = Vec::with_capacity(texts.len());
    for t in texts {
        checked.push(objects::checked("text", t).map_err(invalid)?);
    }
    let made = ed
        .store
        .mutate(id, "history.edit_text", |doc| {
            if let Some((name, rect)) = &patch {
                pixels::add_patch(doc, *rect, name.clone());
            }
            let mut made = Vec::new();
            for props in &checked {
                let oid = doc.next_object_id();
                doc.objects.push(Object {
                    id: oid.clone(),
                    kind: "text".to_string(),
                    props: props.clone(),
                });
                made.push(oid);
            }
            Ok::<_, CommandError>(made)
        })
        .ok_or_else(|| CommandError::internal("document closed"))??;
    ed.store.with(id, |d| d.pixel_selection = None);
    changed(core, ctx, id);
    Ok(made)
}

/// Where pixels of what Separate elements works on go on the canvas (SMT-018):
/// `canvas = origin + pixel × scale`.
#[derive(Debug, Clone, Copy)]
pub struct PixelMap {
    pub ox: f64,
    pub oy: f64,
    pub sx: f64,
    pub sy: f64,
}

/// The pixels Separate elements works on (SMT-018): the image (in base pixels) or an
/// upright floating image's picture.
pub fn separable_pixels(
    core: &Core,
    id: &str,
    object: Option<&str>,
) -> Result<(image::RgbaImage, PixelMap), String> {
    let ed = editor(core);
    let doc = ed
        .store
        .with(id, |d| d.document.clone())
        .ok_or("document closed")?;
    match object {
        None => Ok((
            image_pixels(core, id)?,
            PixelMap {
                ox: -doc.crop.x as f64,
                oy: -doc.crop.y as f64,
                sx: 1.0,
                sy: 1.0,
            },
        )),
        Some(oid) => {
            let o = doc.object(oid).ok_or("no such object")?;
            if o.kind != "image" {
                return Err("only an image".into());
            }
            let rotation = o.props.get("rotation").and_then(Value::as_f64).unwrap_or(0.0);
            if rotation.abs() > 0.01 {
                return Err("only an upright image".into());
            }
            let name = o.props.get("image").and_then(Value::as_str).ok_or("no picture")?;
            let bytes = ed.store.image_file(id, name).ok_or("no picture")?;
            let picture = image::load_from_memory(&bytes)
                .map_err(|e| e.to_string())?
                .to_rgba8();
            let r = xs_image::shapes::rect(o.props.get("rect"))?;
            Ok((
                picture.clone(),
                PixelMap {
                    ox: r.x,
                    oy: r.y,
                    sx: r.w / picture.width().max(1) as f64,
                    sy: r.h / picture.height().max(1) as f64,
                },
            ))
        }
    }
}

/// Separate elements (SMT-018): the pixels of `mask` rebuilt from their surroundings
/// (on the image, or in the floating image `object`'s picture) and each piece added as
/// a floating image where it was, marked with `mark_key` (its mark plus `fp`, the
/// piece's fingerprint by `fp`). One undo step. Returns the pieces' IDs.
#[allow(clippy::too_many_arguments)]
pub fn separate_into(
    core: &Core,
    ctx: &Context,
    id: &str,
    object: Option<&str>,
    mask: &xs_image::selection::Mask,
    pieces: &[(image::RgbaImage, [u32; 4], Map<String, Value>)],
    map: &PixelMap,
    mark_key: &str,
    fp: fn(&Map<String, Value>) -> String,
) -> Result<Vec<String>, CommandError> {
    let lang = ctx.lang;
    let (pixels, _) = separable_pixels(core, id, object).map_err(|e| internal(lang, "", e))?;
    let mut filled = pixels.clone();
    xs_image::selection::fill_gap_with(&mut filled, mask, xs_image::selection::Gap::Smart, &mut |_| true);
    let gap = match (object, mask.bounds()) {
        (None, Some(area)) => Some((
            store_picture(core, id, &pixels::cut_area(&filled, area), lang)?,
            Some(pixels::rect_of(area)),
        )),
        (Some(_), Some(_)) => Some((store_picture(core, id, &filled, lang)?, None)),
        _ => None,
    };
    let mut objects_in: Vec<Map<String, Value>> = Vec::with_capacity(pieces.len());
    for (img, r, mark) in pieces {
        let name = store_picture(core, id, img, lang)?;
        let rect = [
            map.ox + r[0] as f64 * map.sx,
            map.oy + r[1] as f64 * map.sy,
            r[2] as f64 * map.sx,
            r[3] as f64 * map.sy,
        ];
        let mut props = objects::image_object(&name, rect).map_err(invalid)?;
        let mut mark = mark.clone();
        mark.insert("fp".into(), json!(fp(&props)));
        props.insert(mark_key.into(), Value::Object(mark));
        objects_in.push(props);
    }
    let made = editor(core)
        .store
        .mutate(id, "history.separate", |doc| {
            match (&gap, object) {
                (Some((name, Some(rect))), None) => pixels::add_patch(doc, *rect, name.clone()),
                (Some((name, None)), Some(oid)) => {
                    if let Some(o) = doc.objects.iter_mut().find(|o| o.id == oid) {
                        o.props.insert("image".into(), json!(name));
                    }
                }
                _ => {}
            }
            let mut made = Vec::new();
            for props in &objects_in {
                let oid = doc.next_object_id();
                doc.objects.push(Object {
                    id: oid.clone(),
                    kind: "image".to_string(),
                    props: props.clone(),
                });
                made.push(oid);
            }
            Ok::<_, CommandError>(made)
        })
        .ok_or_else(|| CommandError::internal("document closed"))??;
    changed(core, ctx, id);
    Ok(made)
}

/// An object becomes a Text object in its place in the stack (SMT-018's pieces of
/// text). Returns the new object's ID.
pub fn object_to_text(
    core: &Core,
    ctx: &Context,
    id: &str,
    object: &str,
    text: Map<String, Value>,
) -> Result<String, CommandError> {
    let props = objects::checked("text", &text).map_err(invalid)?;
    let new_id = editor(core)
        .store
        .mutate(id, "history.edit_text", |doc| {
            let at = doc
                .objects
                .iter()
                .position(|o| o.id == object)
                .ok_or_else(|| CommandError::internal("object gone"))?;
            let oid = doc.next_object_id();
            doc.objects[at] = Object {
                id: oid.clone(),
                kind: "text".to_string(),
                props: props.clone(),
            };
            Ok::<_, CommandError>(oid)
        })
        .ok_or_else(|| CommandError::internal("document closed"))??;
    changed(core, ctx, id);
    Ok(new_id)
}

/// The pixel selection of the image (not of a floating image), in base pixels.
pub fn image_selection(core: &Core, id: &str) -> Option<std::sync::Arc<xs_image::selection::Mask>> {
    editor(core)
        .store
        .with(id, |d| d.pixel_selection.clone())
        .flatten()
        .filter(|s| s.target.is_none())
        .map(|s| s.mask)
}

/// Where the canvas starts in base pixels (its crop).
pub fn crop_origin(core: &Core, id: &str) -> Option<(i64, i64)> {
    editor(core)
        .store
        .with(id, |d| (d.document.crop.x, d.document.crop.y))
}

/// Fill (`fill`) or Eraser on the image or on the floating image `objectId` (EDT-047,
/// EDT-048): the new pixels become a patch or a new picture (ADR 0005).
fn pixel_edit(core: &Core, ctx: &Context, p: &Value, fill: bool) -> CommandResult {
    let lang = ctx.lang;
    let id = resolve(core, lang, p)?;
    let bad = |param: &str, reason: &str| {
        invalid(tr(
            lang,
            "errors.invalid_param",
            &[("param", param), ("reason", reason)],
        ))
    };
    let point = |v: &Value| Some([v.get(0)?.as_f64()?, v.get(1)?.as_f64()?]);
    let num = |key: &str, default: f64| p.get(key).and_then(Value::as_f64).unwrap_or(default);
    let tool = if fill {
        let at = p
            .get("at")
            .and_then(point)
            .ok_or_else(|| bad("at", "give [x, y]"))?;
        let color = match p.get("color") {
            None | Some(Value::Null) => None,
            Some(v) => {
                // A color or a gradient (EDT-020).
                let f: xs_image::paint::Fill = serde_json::from_value(v.clone())
                    .map_err(|_| bad("color", "use #RRGGBB, #RRGGBBAA, a gradient or null"))?;
                f.check("color").map_err(|r| bad("color", &r))?;
                Some(f)
            }
        };
        pixels::Tool::Fill {
            at,
            color,
            tolerance: num("tolerance", 10.0),
            opacity: num("opacity", 100.0),
            global: p.get("global").and_then(Value::as_bool).unwrap_or(false),
        }
    } else {
        let points: Vec<[f64; 2]> = p
            .get("points")
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(point).collect())
            .unwrap_or_default();
        if points.is_empty() {
            return Err(bad("points", "give at least one [x, y]"));
        }
        pixels::Tool::Erase {
            points,
            width: num("width", 20.0),
            square: p.get("shape").and_then(Value::as_str) == Some("square"),
            opacity: num("opacity", 100.0),
            antialias: p.get("antialias").and_then(Value::as_bool).unwrap_or(true),
        }
    };
    let ed = editor(core);
    let base = ed.store.base(&id).map_err(|e| internal(lang, "", e))?;
    let doc = ed
        .store
        .with(&id, |d| d.document.clone())
        .ok_or_else(|| CommandError::internal("document closed"))?;
    let target = p.get("objectId").and_then(Value::as_str).map(str::to_string);
    // The new pixels, and the rectangle of the image they go on (none for a picture).
    let made = match &target {
        Some(oid) => {
            let object = doc.objects.iter().find(|o| &o.id == oid).ok_or_else(|| {
                CommandError::new(
                    ErrorCode::NotFound,
                    tr(lang, "errors.object_not_found", &[("id", oid)]),
                )
            })?;
            let name = object.props.get("image").and_then(Value::as_str);
            let picture = match (object.kind.as_str(), name) {
                ("image", Some(name)) => ed.store.image_file(&id, name),
                _ => return Err(bad("objectId", "only a floating image has pixels to change")),
            }
            .and_then(|bytes| xs_image::encode::decode(&bytes).ok())
            .ok_or_else(|| CommandError::internal("picture missing"))?;
            pixels::edit_picture(object, picture, &tool).map(|img| (img, None))
        }
        None => pixels::edit_image(&doc, &base, &tool).map(|e| (e.pixels, Some(e.rect))),
    };
    let Some((img, rect)) = made else {
        return Ok(json!({ "changed": false }));
    };
    let png = xs_image::encode::png(&img, 0.0, true).map_err(|e| internal(lang, "", e))?;
    let name = format!("{}{:016x}.png", store::INSERTED, content_hash(&png));
    ed.store
        .add_images(&id, &BTreeMap::from([(name.clone(), png)]))
        .map_err(|e| internal(lang, "", e))?;
    let label = if fill { "history.fill" } else { "history.erase" };
    ed.store
        .mutate(&id, label, |doc| {
            match (&target, rect) {
                (Some(oid), _) => {
                    let o = doc
                        .objects
                        .iter_mut()
                        .find(|o| &o.id == oid)
                        .ok_or_else(|| CommandError::internal("object gone"))?;
                    o.props.insert("image".into(), json!(name));
                }
                (None, Some(rect)) => pixels::add_patch(doc, rect, name.clone()),
                (None, None) => {}
            }
            Ok::<_, CommandError>(())
        })
        .ok_or_else(|| CommandError::internal("document closed"))??;
    changed(core, ctx, &id);
    Ok(json!({ "changed": true }))
}

/// My colors (up to 24) and the last 8 used (EDT-019), shared by every window.
const COLORS_KEY: &str = "colors";
const MY_COLORS: usize = 24;
const RECENT_COLORS: usize = 8;
const MY_GRADIENTS: usize = 16;

fn colors_of(core: &Core) -> (Vec<String>, Vec<String>, Vec<Value>) {
    let v = core.recall(COLORS_KEY).unwrap_or_default();
    let list = |k: &str| -> Vec<String> {
        v.get(k)
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(|c| c.as_str().map(str::to_string)).collect())
            .unwrap_or_default()
    };
    let gradients = v
        .get("gradients")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    (list("mine"), list("recent"), gradients)
}

/// `editor.color`: save or remove one of My colors, or note a color as used.
fn color_command(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let bad = |param: &str, reason: &str| {
        invalid(tr(
            lang,
            "errors.invalid_param",
            &[("param", param), ("reason", reason)],
        ))
    };
    let (mut mine, mut recent, mut gradients) = colors_of(core);
    let action = p.get("action").and_then(Value::as_str).unwrap_or_default();
    if action == "save_gradient" || action == "remove_gradient" {
        // My gradients, up to 16 (EDT-020).
        let g = p
            .get("gradient")
            .cloned()
            .ok_or_else(|| bad("gradient", "give the gradient"))?;
        let fill: xs_image::paint::Fill =
            serde_json::from_value(g.clone()).map_err(|_| bad("gradient", "it is not a gradient"))?;
        if !matches!(fill, xs_image::paint::Fill::Gradient(_)) {
            return Err(bad("gradient", "it is not a gradient"));
        }
        fill.check("gradient").map_err(|r| bad("gradient", &r))?;
        let g = serde_json::to_value(&fill).unwrap_or_default();
        if action == "save_gradient" {
            if !gradients.contains(&g) {
                if gradients.len() >= MY_GRADIENTS {
                    return Err(bad("gradient", "My gradients holds 16: remove one first"));
                }
                gradients.push(g);
            }
        } else {
            gradients.retain(|x| x != &g);
        }
    } else {
        let color = p.get("color").and_then(Value::as_str).unwrap_or_default();
        let c =
            xs_image::color::Rgba::parse(color).ok_or_else(|| bad("color", "use #RRGGBB or #RRGGBBAA"))?;
        // One spelling per color: #RRGGBB, or #RRGGBBAA when not opaque.
        let color = if c.a == 255 {
            format!("#{:02X}{:02X}{:02X}", c.r, c.g, c.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", c.r, c.g, c.b, c.a)
        };
        match action {
            "save" => {
                if !mine.contains(&color) {
                    if mine.len() >= MY_COLORS {
                        return Err(bad("color", "My colors holds 24: remove one first"));
                    }
                    mine.push(color);
                }
            }
            "remove" => mine.retain(|x| x != &color),
            "used" => {
                recent.retain(|x| x != &color);
                recent.insert(0, color);
                recent.truncate(RECENT_COLORS);
            }
            other => return Err(bad("action", other)),
        }
    }
    let v = json!({ "mine": mine, "recent": recent, "gradients": gradients });
    core.remember(COLORS_KEY, v.clone());
    core.host().editor_event("colors", v.clone());
    Ok(v)
}

/// Saved effects (EDT-054): `[{id, name, stack}]`, kept between sessions.
const SAVED_EFFECTS_KEY: &str = "effects.saved";

/// The saved effects that come with the app (EDT-054, design decision).
fn built_in_effects(lang: Lang) -> Value {
    let es = lang == Lang::Es;
    let n = |en: &str, es_: &str| if es { es_.to_string() } else { en.to_string() };
    json!([
        {"id": "fine-frame", "name": n("Fine frame", "Marco fino"), "stack": [
            {"type": "border", "enabled": true, "width": 1, "color": "#1F2430", "position": "outside"}]},
        {"id": "floating-card", "name": n("Floating card", "Tarjeta flotante"), "stack": [
            {"type": "corners", "enabled": true, "shape": "radius", "radius": 12},
            {"type": "shadow", "enabled": true, "angle": 90, "distance": 6, "blur": 18, "opacity": 35, "color": "#000000"}]},
        {"id": "torn-sheet", "name": n("Torn sheet", "Hoja rasgada"), "stack": [
            {"type": "edges", "enabled": true, "style": "torn", "size": 16, "sides": {"bottom": true}, "randomness": 50, "seed": 4242},
            {"type": "shadow", "enabled": true, "angle": 45, "distance": 6, "blur": 12, "opacity": 40, "color": "#000000"}]},
        {"id": "soft-perspective", "name": n("Soft perspective", "Perspectiva suave"), "stack": [
            {"type": "perspective", "enabled": true, "style": "perspective", "horizontal": 12, "vertical": 0, "depth": 40},
            {"type": "shadow", "enabled": true, "angle": 90, "distance": 8, "blur": 20, "opacity": 30, "color": "#000000"}]},
        {"id": "documentation", "name": n("Documentation", "Documentación"), "stack": [
            {"type": "border", "enabled": true, "width": 1, "color": "#9AA0A6", "position": "outside"},
            {"type": "details", "enabled": true, "placement": "band", "side": "bottom", "fields": {"app": true, "url": true, "title": true, "date": true}, "size": 11, "color": "#FFFFFF", "background": "#1F2430", "align": "left", "padding": 8}]},
        {"id": "grayscale", "name": n("Grayscale", "Escala de grises"), "stack": [
            {"type": "filters", "enabled": true, "grayscale": {"on": true}}]},
    ])
}

fn saved_effects(core: &Core) -> Vec<Value> {
    core.recall(SAVED_EFFECTS_KEY)
        .and_then(|v| v.as_array().cloned())
        .unwrap_or_else(|| {
            built_in_effects(core.lang())
                .as_array()
                .cloned()
                .unwrap_or_default()
        })
}

/// `editor.saved_effect` (EDT-054): save the current stack, apply, rename, update,
/// delete, move, or add to favorites.
fn saved_effect_command(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let bad = |param: &str, reason: &str| {
        invalid(tr(
            lang,
            "errors.invalid_param",
            &[("param", param), ("reason", reason)],
        ))
    };
    let mut list = saved_effects(core);
    let id = p.get("id").and_then(Value::as_str).map(str::to_string);
    let find = |list: &Vec<Value>| -> Result<usize, CommandError> {
        let id = id.as_deref().ok_or_else(|| bad("id", "give the saved effect"))?;
        list.iter()
            .position(|e| e["id"] == id)
            .ok_or_else(|| bad("id", "there is no such saved effect"))
    };
    // The stack of the target: the document shown, or its floating image.
    let target_stack = || -> Result<(String, Vec<Value>), CommandError> {
        let doc = resolve(core, lang, p)?;
        let stack = editor(core)
            .store
            .with(&doc, |d| match p.get("objectId").and_then(Value::as_str) {
                Some(oid) => d
                    .document
                    .objects
                    .iter()
                    .find(|o| o.id == oid)
                    .and_then(|o| o.props.get("effects").and_then(Value::as_array).cloned())
                    .unwrap_or_default(),
                None => d.document.effects.clone(),
            })
            .ok_or_else(|| CommandError::internal("document closed"))?;
        Ok((doc, stack))
    };
    let mut out = json!({});
    match p.get("action").and_then(Value::as_str).unwrap_or_default() {
        "apply" => {
            let i = find(&list)?;
            let saved: Vec<Value> = list[i]["stack"].as_array().cloned().unwrap_or_default();
            let (doc, current) = target_stack()?;
            // Shift+click adds it at the end (EDT-054, design decision).
            let append = p.get("append").and_then(Value::as_bool).unwrap_or(false);
            let stack: Vec<Value> = if append {
                current.into_iter().chain(saved).collect()
            } else {
                saved
            };
            let mut params = json!({"documentId": doc, "action": "set", "effects": stack});
            if let Some(oid) = p.get("objectId") {
                params["objectId"] = oid.clone();
            }
            return core.call(ctx.source, "document.effects", params, Some(lang));
        }
        "save" => {
            let (_, stack) = target_stack()?;
            if stack.is_empty() {
                return Err(bad("documentId", "there are no effects to save"));
            }
            let name = p
                .get("name")
                .and_then(Value::as_str)
                .filter(|n| !n.trim().is_empty())
                .unwrap_or("Effect")
                .to_string();
            let new = format!("fx{}", chrono::Utc::now().timestamp_millis());
            list.push(json!({"id": new, "name": name, "stack": stack}));
            out = json!({ "id": new });
        }
        "update" => {
            let i = find(&list)?;
            let (_, stack) = target_stack()?;
            list[i]["stack"] = json!(stack);
        }
        "rename" => {
            let i = find(&list)?;
            let name = p.get("name").and_then(Value::as_str).unwrap_or_default().trim();
            if name.is_empty() {
                return Err(bad("name", "the name is empty"));
            }
            list[i]["name"] = json!(name);
        }
        "delete" => {
            let i = find(&list)?;
            list.remove(i);
        }
        "move" => {
            let i = find(&list)?;
            let to = p
                .get("to")
                .and_then(Value::as_u64)
                .ok_or_else(|| bad("to", "give the new position"))? as usize;
            let e = list.remove(i);
            list.insert(to.min(list.len()), e);
        }
        // A favorite of the Favorites tool (EDT-016): a copy of the stack.
        "favorite" => {
            let i = find(&list)?;
            let ed = editor(core);
            let mut lib = ed.themes.lock().unwrap().clone();
            let fid = lib.favorite(
                "effects",
                json!({"name": list[i]["name"], "stack": list[i]["stack"]}),
            );
            *ed.themes.lock().unwrap() = lib.clone();
            themes_changed(core, &lib);
            return Ok(json!({ "favoriteId": fid }));
        }
        other => return Err(bad("action", other)),
    }
    core.remember(SAVED_EFFECTS_KEY, json!(list));
    core.host().editor_event("saved_effects", json!({}));
    Ok(out)
}

/// Keeps the themes for the next session and tells the editor (EDT-015, EDT-017).
fn themes_changed(core: &Core, lib: &themes::Library) {
    core.remember(themes::KEY, serde_json::to_value(lib).unwrap_or_default());
    core.host().editor_event("themes", json!({}));
}

/// The sample of a style as a PNG (EDT-015): `which` is `current` (the tool's style
/// now), a saved style's id or a favorite's id.
pub fn sample_png(core: &Core, tool: &str, which: &str) -> Option<Vec<u8>> {
    let ed = editor(core);
    if tool == "effects" {
        // A saved effect or an effect favorite (EDT-054, EDT-016).
        let stack = saved_effects(core)
            .into_iter()
            .find(|e| e["id"] == which)
            .map(|e| e["stack"].clone())
            .or_else(|| {
                let lib = ed.themes.lock().unwrap();
                lib.favorites
                    .iter()
                    .find(|f| f.id == which)
                    .map(|f| f.style["stack"].clone())
            })?;
        let img = samples::effect_sample(stack.as_array()?)?;
        return xs_image::encode::png(&img, 0.0, true).ok();
    }
    let (tool, style) = if which == "current" {
        (tool.to_string(), ed.styles().of(tool)?)
    } else {
        let lib = ed.themes.lock().unwrap();
        match lib.favorites.iter().find(|f| f.id == which) {
            Some(f) => (f.tool.clone(), f.style.clone()),
            None => (tool.to_string(), lib.find(tool, which)?.style.clone()),
        }
    };
    let img = samples::sample(&tool, &style)?;
    xs_image::encode::png(&img, 0.0, true).ok()
}

/// `editor.styles`: the saved styles of the active theme, by tool, and the favorites.
fn styles_list(core: &Core) -> Value {
    let lib = editor(core).themes.lock().unwrap().clone();
    let theme = lib.active();
    json!({
        "theme": {"id": theme.id, "name": theme.name},
        "tools": theme.styles,
        "favorites": lib.favorites,
    })
}

/// A saved style applied to objects: never their position, size or text; a blur
/// takes only its type, strength and color (EDT-015).
fn style_for_objects(tool: &str, style: &Value) -> Map<String, Value> {
    let mut m = style.as_object().cloned().unwrap_or_default();
    match tool {
        "blur" => m.retain(|k, _| k == "blurType" || k == "intensity" || k == "color"),
        "step" => {
            m.remove("size");
        }
        _ => {}
    }
    m
}

/// `editor.style` (EDT-015, EDT-016): saved styles and favorites.
fn style_command(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let bad = |param: &str, reason: &str| {
        invalid(tr(
            lang,
            "errors.invalid_param",
            &[("param", param), ("reason", reason)],
        ))
    };
    let ed = editor(core);
    let action = p.get("action").and_then(Value::as_str).unwrap_or_default();
    let tool = p
        .get("tool")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let style_id = p.get("styleId").and_then(Value::as_str).map(str::to_string);
    let needs_tool = !matches!(action, "unfavorite" | "move_favorite" | "apply_favorite");
    if needs_tool && !themes::STYLED_TOOLS.contains(&tool.as_str()) {
        return Err(bad("tool", "this tool has no saved styles"));
    }
    let current = || ed.styles().of(&tool).unwrap_or_default();
    let mut lib = ed.themes.lock().unwrap().clone();
    let mut out = json!({});
    let need_id = || style_id.clone().ok_or_else(|| bad("styleId", "give the style"));
    match action {
        "apply" => {
            let id = need_id()?;
            let saved = lib
                .find(&tool, &id)
                .ok_or_else(|| bad("styleId", "there is no such style"))?
                .style
                .clone();
            set_tool_style(core, &tool, saved.as_object().unwrap_or(&Map::new()), lang)?;
            // The selected objects of that tool take it too.
            let ids: Vec<Value> = p
                .get("objectIds")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            if !ids.is_empty() {
                let doc = resolve(core, lang, p)?;
                core.call(
                    ctx.source,
                    "document.update_object",
                    json!({"documentId": doc, "objectIds": ids, "set": {"style": style_for_objects(&tool, &saved)}}),
                    Some(lang),
                )?;
            }
            return Ok(json!({ "style": saved }));
        }
        "save" => {
            let name = p.get("name").and_then(Value::as_str).map(str::to_string);
            out = json!({ "styleId": lib.add(&tool, current(), name) });
        }
        "update" => lib
            .update(&tool, &need_id()?, current())
            .map_err(|r| bad("styleId", &r))?,
        "rename" => {
            let name = p.get("name").and_then(Value::as_str).unwrap_or_default();
            lib.rename(&tool, &need_id()?, name)
                .map_err(|r| bad("styleId", &r))?
        }
        "delete" => lib.delete(&tool, &need_id()?).map_err(|r| bad("styleId", &r))?,
        "move" => {
            let to = p
                .get("to")
                .and_then(Value::as_u64)
                .ok_or_else(|| bad("to", "give the new position"))?;
            lib.move_to(&tool, &need_id()?, to as usize)
                .map_err(|r| bad("styleId", &r))?
        }
        "reset" => lib.reset(&tool),
        "favorite" => {
            let style = match &style_id {
                Some(id) => lib
                    .find(&tool, id)
                    .ok_or_else(|| bad("styleId", "there is no such style"))?
                    .style
                    .clone(),
                None => current(),
            };
            out = json!({ "favoriteId": lib.favorite(&tool, style) });
        }
        "unfavorite" | "move_favorite" | "apply_favorite" => {
            let fid = p
                .get("favoriteId")
                .and_then(Value::as_str)
                .ok_or_else(|| bad("favoriteId", "give the favorite"))?;
            match action {
                "unfavorite" => lib.unfavorite(fid).map_err(|r| bad("favoriteId", &r))?,
                "move_favorite" => {
                    let to = p
                        .get("to")
                        .and_then(Value::as_u64)
                        .ok_or_else(|| bad("to", "give the new position"))?;
                    lib.move_favorite(fid, to as usize)
                        .map_err(|r| bad("favoriteId", &r))?
                }
                _ => {
                    // Its tool, with its style (EDT-016).
                    let fav = lib
                        .favorites
                        .iter()
                        .find(|f| f.id == fid)
                        .cloned()
                        .ok_or_else(|| bad("favoriteId", "there is no such favorite"))?;
                    // A saved effect: it goes to the document shown.
                    if fav.tool == "effects" {
                        let doc = resolve(core, lang, p)?;
                        let stack = fav.style.get("stack").cloned().unwrap_or(json!([]));
                        return core.call(
                            ctx.source,
                            "document.effects",
                            json!({"documentId": doc, "action": "set", "effects": stack}),
                            Some(lang),
                        );
                    }
                    set_tool_style(
                        core,
                        &fav.tool,
                        fav.style.as_object().unwrap_or(&Map::new()),
                        lang,
                    )?;
                    core.call(
                        ctx.source,
                        "editor.select_tool",
                        json!({"tool": fav.tool}),
                        Some(lang),
                    )?;
                    return Ok(json!({ "tool": fav.tool, "style": fav.style }));
                }
            }
        }
        other => return Err(bad("action", other)),
    }
    *ed.themes.lock().unwrap() = lib.clone();
    themes_changed(core, &lib);
    Ok(out)
}

/// `editor.themes`: the themes, without their styles.
fn themes_list(core: &Core) -> Value {
    let lib = editor(core).themes.lock().unwrap().clone();
    json!({
        "active": lib.active,
        "themes": lib.themes.iter().map(|t| json!({
            "id": t.id, "name": t.name, "colors": t.colors, "font": t.font,
            "shadow": t.shadow, "shadowOff": t.shadow_off, "builtIn": t.built_in,
        })).collect::<Vec<_>>(),
    })
}

/// A theme's settings from `theme` (EDT-017).
fn theme_from(v: &Value) -> themes::Theme {
    let list = |k: &str| -> Vec<String> {
        v.get(k)
            .and_then(Value::as_array)
            .map(|a| a.iter().filter_map(|c| c.as_str().map(str::to_string)).collect())
            .unwrap_or_default()
    };
    themes::Theme {
        id: String::new(),
        name: v
            .get("name")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string(),
        colors: list("colors"),
        font: v
            .get("font")
            .and_then(Value::as_str)
            .filter(|f| !f.is_empty())
            .map(str::to_string),
        shadow: v.get("shadow").and_then(Value::as_bool).unwrap_or(true),
        shadow_off: list("shadowOff"),
        built_in: false,
        styles: BTreeMap::new(),
    }
}

/// `editor.theme` (EDT-017, EDT-018): activate, create or change, duplicate, rename,
/// delete, export or import a theme.
fn theme_command(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let bad = |param: &str, reason: &str| {
        invalid(tr(
            lang,
            "errors.invalid_param",
            &[("param", param), ("reason", reason)],
        ))
    };
    let ed = editor(core);
    let action = p.get("action").and_then(Value::as_str).unwrap_or_default();
    let id = p.get("id").and_then(Value::as_str).map(str::to_string);
    let need_id = || id.clone().ok_or_else(|| bad("id", "give the theme"));
    let mut lib = ed.themes.lock().unwrap().clone();
    let mut out = json!({});
    match action {
        "activate" => {
            let id = need_id()?;
            lib.index_of(&id)
                .ok_or_else(|| bad("id", "there is no such theme"))?;
            lib.active = id;
        }
        "preview" => {
            // How a theme not saved yet looks: its first arrow, callout, shape and step.
            use base64::Engine as _;
            let theme = theme_from(p.get("theme").ok_or_else(|| bad("theme", "give its settings"))?);
            themes::check(&theme).map_err(|r| bad("theme", &r))?;
            let mut samples = Map::new();
            for tool in ["arrow", "callout", "shape", "step"] {
                let Some(first) = themes::generated(&theme, tool).into_iter().next() else {
                    continue;
                };
                let style = themes::whole(tool, &first).map_err(|r| bad("theme", &r))?;
                if let Some(png) =
                    samples::sample(tool, &style).and_then(|i| xs_image::encode::png(&i, 0.0, true).ok())
                {
                    let data = base64::engine::general_purpose::STANDARD.encode(png);
                    samples.insert(tool.to_string(), json!(format!("data:image/png;base64,{data}")));
                }
            }
            return Ok(json!({ "samples": samples }));
        }
        "save" => {
            let theme = theme_from(p.get("theme").ok_or_else(|| bad("theme", "give its settings"))?);
            let new = lib
                .save_theme(id.as_deref(), theme)
                .map_err(|r| bad("theme", &r))?;
            lib.active = new.clone();
            out = json!({ "id": new });
        }
        "duplicate" => {
            let new = lib.duplicate(&need_id()?).map_err(|r| bad("id", &r))?;
            out = json!({ "id": new });
        }
        "rename" => {
            let name = p.get("name").and_then(Value::as_str).unwrap_or_default();
            lib.rename_theme(&need_id()?, name).map_err(|r| bad("name", &r))?
        }
        "delete" => lib.delete_theme(&need_id()?).map_err(|r| bad("id", &r))?,
        "export" => {
            let id = id.clone().unwrap_or_else(|| lib.active.clone());
            let i = lib
                .index_of(&id)
                .ok_or_else(|| bad("id", "there is no such theme"))?;
            let path = p
                .get("path")
                .and_then(Value::as_str)
                .ok_or_else(|| bad("path", "give the file"))?;
            let text = serde_json::to_string_pretty(&themes::to_file(&lib.themes[i])).unwrap_or_default();
            std::fs::write(path, text).map_err(|e| internal(lang, path, e))?;
            return Ok(json!({ "path": path }));
        }
        "import" => {
            let path = p
                .get("path")
                .and_then(Value::as_str)
                .ok_or_else(|| bad("path", "give the file"))?;
            let text = std::fs::read_to_string(path).map_err(|e| bad("path", &e.to_string()))?;
            let v: Value = serde_json::from_str(&text).map_err(|_| bad("path", "it is not a theme file"))?;
            let theme = themes::from_file(&v).map_err(|e| match e {
                themes::FileError::Newer => invalid(tr(lang, "errors.theme_newer", &[])),
                themes::FileError::Unreadable(r) => bad("path", &r),
            })?;
            // The same name as one of the user's themes: Replace or Keep both (EDT-018).
            let clash = lib.themes.iter().any(|t| t.name == theme.name && !t.built_in);
            let choice = p.get("onConflict").and_then(Value::as_str);
            if clash && choice.is_none() {
                return Ok(json!({ "status": "needs_decision", "name": theme.name }));
            }
            let missing_font = theme.font.clone().filter(|f| {
                !xs_image::text::families()
                    .iter()
                    .any(|x| x.eq_ignore_ascii_case(f))
            });
            let new = lib.import(theme, choice == Some("replace"));
            out = json!({ "status": "imported", "id": new, "missingFont": missing_font });
        }
        other => return Err(bad("action", other)),
    }
    *ed.themes.lock().unwrap() = lib.clone();
    themes_changed(core, &lib);
    Ok(out)
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
/// The library moved (LIB-028): new captures go to the new folder, and open documents
/// of the library follow their files.
/// Captures that waited for the library are in it now (LIB-030): their documents
/// follow their files.
fn waiting_arrived(core: &Core, moved: &[(PathBuf, PathBuf)]) {
    let ed = editor(core);
    for id in ed.store.ids() {
        let to = ed
            .store
            .with(&id, |d| {
                d.path.as_ref().and_then(|p| {
                    moved
                        .iter()
                        .find(|(from, _)| store::same_file(from, p) || from == p)
                        .map(|(_, to)| to.clone())
                })
            })
            .flatten();
        if let Some(to) = to {
            ed.store.set_path(&id, to);
        }
    }
    emit_list(core);
}

fn library_relocated(core: &Core, from: &Path, to: &Path) {
    let ed = editor(core);
    *ed.library.lock().unwrap() = to.to_path_buf();
    for id in ed.store.ids() {
        let moved = ed
            .store
            .with(&id, |d| {
                d.path
                    .as_ref()
                    .and_then(|p| p.strip_prefix(from).ok().map(|r| to.join(r)))
            })
            .flatten();
        if let Some(path) = moved {
            ed.store.set_path(&id, path);
        }
    }
    emit_list(core);
}

pub fn extension() -> Extension {
    Box::new(|reg, weak| {
        if let Some(core) = weak.upgrade() {
            core.add_quit_guard(Arc::new(EditorQuitGuard(weak.clone())));
            // The shortcut sheet as PDF (APP-023), with the text engine.
            core.replace_service(xs_core::sheet::PdfSheet(Some(Box::new(sheet_pdf::render))));
            // A smaller tray lets the oldest free items go at once (TRY-003).
            let weak = weak.clone();
            core.on_setting(Arc::new({
                let weak = weak.clone();
                move |key, _| {
                    if key == "tray.capacity" {
                        if let Some(core) = weak.upgrade() {
                            emit_list(&core);
                        }
                    }
                }
            }));
            // Themes, saved effects or the toolbar imported (APP-011): the editor takes
            // them at once.
            core.on_memory(Arc::new(move |key, value| {
                let Some(core) = weak.upgrade() else { return };
                match key {
                    themes::KEY => {
                        *editor(&core).themes.lock().unwrap() = themes::Library::load(Some(value.clone()));
                        core.host().editor_event("themes", json!({}));
                    }
                    SAVED_EFFECTS_KEY => core.host().editor_event("saved_effects", json!({})),
                    TOOLBAR_KEY => core.host().editor_event("toolbar", json!({})),
                    _ => {}
                }
            }));
        }
        xs_library::on_relocated(library_relocated);
        xs_library::on_arrived(waiting_arrived);
        macro_rules! on {
            ($id:literal, |$core:ident, $ctx:ident, $p:ident| $body:expr) => {{
                let weak = weak.clone();
                reg.handle($id, move |$ctx: &Context, $p: Value| {
                    let $core = upgrade(&weak)?;
                    // The documents reopened at startup come first (EDT-120).
                    editor(&$core).wait_reopened();
                    // Image tools say why not with a video (VID-065).
                    if crate::video::IMAGE_ONLY.contains(&$id) {
                        crate::video::refuse_for_video(&$core, $ctx.lang, &$p)?;
                    }
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
        // Videos in the editor (ADR 0008, VID-059 to VID-063).
        on!("video.cut", |core, ctx, p| video::cut(&core, ctx, &p));
        on!("video.combine", |core, ctx, p| video::combine(&core, ctx, &p));
        on!("video.gif", |core, ctx, p| video::gif(&core, ctx, &p));
        on!("video.gif_estimate", |core, ctx, p| video::gif_estimate(
            &core, ctx, &p
        ));
        on!("video.cancel", |core, ctx, p| video::cancel(&core, ctx, &p));
        on!("video.save_selection", |core, ctx, p| video::save_selection(
            &core, ctx, &p
        ));
        on!("video.save_frame", |core, ctx, p| video::save_frame(
            &core, ctx, &p
        ));
        on!("video.extract_text", |core, ctx, p| video::extract_text(
            &core, ctx, &p
        ));
        on!("video.silent_copy", |core, ctx, p| video::silent_copy(
            &core, ctx, &p
        ));
        on!("document.info", |core, ctx, p| {
            let id = resolve(&core, ctx.lang, &p)?;
            info(&core, &id, ctx.lang).ok_or_else(|| CommandError::internal("document closed"))
        });
        on!("document.annotate", |core, ctx, p| {
            let id = resolve(&core, ctx.lang, &p)?;
            let styles = editor(&core).styles();
            // Stamps: their drawing from the collection (EDT-031).
            let canvas = editor(&core)
                .store
                .with(&id, |d| {
                    (d.document.canvas.width as f64, d.document.canvas.height as f64)
                })
                .unwrap_or((400.0, 300.0));
            let mut inputs: Vec<Value> = p["objects"].as_array().cloned().unwrap_or_default();
            for (i, input) in inputs.iter_mut().enumerate() {
                if input["type"] != "stamp" {
                    continue;
                }
                let sid = input
                    .get("stamp")
                    .and_then(Value::as_str)
                    .map(str::to_string)
                    .unwrap_or_else(|| styles.stamp.stamp.clone());
                let filled =
                    objects::stamp_props(input.as_object().unwrap_or(&Map::new()), &sid, ctx.lang, canvas)
                        .map_err(|reason| {
                            let index = (i + 1).to_string();
                            invalid(tr(
                                ctx.lang,
                                "errors.invalid_object",
                                &[("index", &index), ("reason", &reason)],
                            ))
                        })?;
                *input = Value::Object(filled);
            }
            let mut objects = Vec::new();
            for (i, input) in inputs.iter().enumerate() {
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
            let auto = auto_expand(&core);
            // A callout without tails gets one once it fits its text (EDT-028).
            let tailless: Vec<bool> = p["objects"]
                .as_array()
                .map(|a| a.iter().map(|o| o.get("tails").is_none()).collect())
                .unwrap_or_default();
            // A step without a value takes the next one of its sequence (EDT-035).
            let unnumbered: Vec<bool> = p["objects"]
                .as_array()
                .map(|a| a.iter().map(|o| o.get("value").is_none()).collect())
                .unwrap_or_default();
            let mut step_next = editor(&core)
                .store
                .with(&id, |d| d.step_next.clone())
                .unwrap_or_default();
            let ids = editor(&core)
                .store
                .mutate(&id, &label, |doc| {
                    let mut ids = Vec::new();
                    for (i, (kind, mut props)) in objects.into_iter().enumerate() {
                        if kind == "step" {
                            let seq = steps::sequence_of(&Object {
                                id: String::new(),
                                kind: kind.clone(),
                                props: props.clone(),
                            });
                            let n = if unnumbered.get(i).copied().unwrap_or(true) {
                                let n = steps::next(doc, seq, &step_next);
                                props.insert("value".into(), json!(seq.value(n)));
                                Some(n)
                            } else {
                                props
                                    .get("value")
                                    .and_then(Value::as_str)
                                    .and_then(|v| seq.index(v))
                            };
                            if let Some(n) = n {
                                step_next.insert(seq.key().to_string(), n + 1);
                            }
                        }
                        let oid = doc.next_object_id();
                        let mut o = Object {
                            id: oid.clone(),
                            kind,
                            props,
                        };
                        objects::fit(&mut o, doc.dpi);
                        let spot = (o.kind == "spotlight").then(|| o.id.clone());
                        if tailless.get(i).copied().unwrap_or(false) {
                            objects::default_tail(&mut o, doc.canvas.height as f64);
                        }
                        doc.objects.push(o);
                        // A new spotlight's settings pass to the others (EDT-039).
                        if let Some(sid) = spot {
                            objects::share_spotlights(doc, &sid);
                        }
                        ids.push(oid);
                    }
                    // A stroke that is not editable is painted into the image: a
                    // flattened layer that cannot come back (EDT-037, EDT-091).
                    let painted: Vec<String> = doc
                        .objects
                        .iter()
                        .filter(|o| {
                            ids.contains(&o.id)
                                && o.kind == "pen"
                                && o.props["style"]["editable"] == json!(false)
                        })
                        .map(|o| o.id.clone())
                        .collect();
                    if !painted.is_empty() {
                        if let Some(made) = flatten::flatten(doc, &painted) {
                            for l in doc.flattened.iter_mut().filter(|l| made.layers.contains(&l.id)) {
                                l.permanent = Some("painted".into());
                            }
                        }
                    }
                    if auto {
                        fit::fit_canvas(doc);
                    }
                    Ok::<_, CommandError>(ids)
                })
                .ok_or_else(|| CommandError::internal("document closed"))??;
            editor(&core).store.with(&id, |d| d.step_next = step_next);
            changed(&core, ctx, &id);
            Ok(json!({ "ids": ids }))
        });
        // Simplify (EDT-041): whether the original shows under simplified shapes.
        on!("document.simplify_view", |core, ctx, p| {
            let id = resolve(&core, ctx.lang, &p)?;
            let show = p.get("showOriginal").and_then(Value::as_bool);
            let opacity = p.get("opacity").and_then(Value::as_f64);
            if opacity.is_some_and(|o| !(0.0..=100.0).contains(&o)) {
                return Err(invalid(tr(
                    ctx.lang,
                    "errors.invalid_param",
                    &[("param", "opacity"), ("reason", "must be between 0 and 100")],
                )));
            }
            let view = editor(&core)
                .store
                .mutate(&id, "history.simplify_view", |doc| {
                    let v = doc.simplify.get_or_insert(xs_core::document::SimplifyView {
                        show_original: true,
                        opacity: 100.0,
                    });
                    if let Some(s) = show {
                        v.show_original = s;
                    }
                    if let Some(o) = opacity {
                        v.opacity = o;
                    }
                    Ok::<_, CommandError>(serde_json::to_value(&*v).unwrap_or_default())
                })
                .ok_or_else(|| CommandError::internal("document closed"))??;
            changed(&core, ctx, &id);
            Ok(view)
        });
        // The stamp collection (EDT-031): categories and stamps, found by words.
        on!("editor.stamps", |_core, ctx, p| {
            let query = p.get("query").and_then(Value::as_str).unwrap_or_default();
            let category = p.get("category").and_then(Value::as_str);
            let es = ctx.lang == Lang::Es;
            let stamps: Vec<Value> = xs_image::stamps::collection()
                .iter()
                .filter(|s| category.is_none_or(|c| c == s.category) && s.matches(query))
                .map(|s| json!({"id": s.id, "category": s.category, "name": if es { s.es } else { s.en }}))
                .collect();
            let categories: Vec<Value> = xs_image::stamps::CATEGORIES
                .iter()
                .map(|c| json!({"id": c, "name": tr(ctx.lang, &format!("stamps.{c}"), &[])}))
                .collect();
            Ok(json!({ "categories": categories, "stamps": stamps }))
        });
        // Step sequence (EDT-035): Restart sequence and Renumber.
        on!("document.steps", |core, ctx, p| {
            let id = resolve(&core, ctx.lang, &p)?;
            let seq = match p.get("sequence").and_then(Value::as_str) {
                Some(k) => xs_image::steps::Sequence::from_key(k).ok_or_else(|| {
                    invalid(tr(
                        ctx.lang,
                        "errors.invalid_param",
                        &[("param", "sequence"), ("reason", "unknown sequence")],
                    ))
                })?,
                None => editor(&core).styles().step.sequence,
            };
            let action = p["action"].as_str().unwrap_or_default().to_string();
            match action.as_str() {
                "restart" => {
                    editor(&core).store.with(&id, |d| {
                        d.step_next.insert(seq.key().to_string(), 1);
                    });
                    changed(&core, ctx, &id);
                    Ok(json!({ "next": seq.value(1) }))
                }
                "renumber" => {
                    let count = editor(&core)
                        .store
                        .mutate(&id, "history.renumber", |doc| {
                            Ok::<_, CommandError>(steps::renumber(doc, seq))
                        })
                        .ok_or_else(|| CommandError::internal("document closed"))??;
                    editor(&core).store.with(&id, |d| {
                        d.step_next.insert(seq.key().to_string(), count + 1);
                    });
                    changed(&core, ctx, &id);
                    Ok(json!({ "renumbered": count, "next": seq.value(count + 1) }))
                }
                _ => Err(invalid(tr(
                    ctx.lang,
                    "errors.invalid_param",
                    &[("param", "action"), ("reason", "must be restart or renumber")],
                ))),
            }
        });
        on!("document.update_object", |core, ctx, p| {
            let id = resolve(&core, ctx.lang, &p)?;
            let oids = object_ids(&p, ctx.lang)?;
            let kinds = kinds_of(&core, ctx.lang, &id, &oids)?;
            let mut set = p["set"].as_object().cloned().unwrap_or_default();
            // Another stamp in place of a stamp: same place and height (EDT-031).
            if let (Some(sid), [oid]) = (
                set.get("stamp").and_then(Value::as_str).map(str::to_string),
                oids.as_slice(),
            ) {
                let rect = editor(&core)
                    .store
                    .with(&id, |d| {
                        d.document.object(oid).and_then(|o| o.props.get("rect").cloned())
                    })
                    .flatten();
                if let Some(rect) = rect {
                    let mut input = set.clone();
                    input.insert("rect".into(), rect);
                    set = objects::stamp_props(&input, &sid, ctx.lang, (0.0, 0.0)).map_err(|r| {
                        invalid(tr(
                            ctx.lang,
                            "errors.invalid_param",
                            &[("param", "set"), ("reason", &r)],
                        ))
                    })?;
                }
            }
            // Several objects moved together, each from where it is (EDT-079, EDT-087).
            let shift = p
                .get("move")
                .and_then(Value::as_array)
                .and_then(|a| Some((a.first()?.as_f64()?, a.get(1)?.as_f64()?)));
            // A move of its own for each object: aligning and distributing (EDT-083).
            let own_moves: BTreeMap<String, (f64, f64)> = p
                .get("moves")
                .and_then(Value::as_object)
                .map(|m| {
                    m.iter()
                        .filter_map(|(k, v)| {
                            let a = v.as_array()?;
                            Some((k.clone(), (a.first()?.as_f64()?, a.get(1)?.as_f64()?)))
                        })
                        .collect()
                })
                .unwrap_or_default();
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
            let auto = auto_expand(&core);
            let updated = editor(&core)
                .store
                .mutate(&id, &objects::label(action, &kind_refs), |doc| {
                    let dpi = doc.dpi;
                    for oid in &oids {
                        let o = doc
                            .object_mut(oid)
                            .ok_or_else(|| CommandError::internal("object vanished"))?;
                        if let Some((dx, dy)) = shift {
                            objects::translate(o, dx, dy);
                        }
                        if let Some((dx, dy)) = own_moves.get(oid) {
                            objects::translate(o, *dx, *dy);
                        }
                        objects::update(o, &set).map_err(|r| {
                            invalid(tr(
                                ctx.lang,
                                "errors.invalid_param",
                                &[("param", "set"), ("reason", &r)],
                            ))
                        })?;
                        objects::fit(o, dpi);
                    }
                    // A spotlight's changed settings pass to the others (EDT-039).
                    if set.contains_key("style") {
                        if let Some(first) = oids.first() {
                            objects::share_spotlights(doc, first);
                        }
                    }
                    // A guide's text grows its section at once, never cut (SMT-028).
                    if set.contains_key("text") {
                        if let Some(t) = templates::Template::of(doc).filter(|t| t.is_guide()) {
                            template_cmds::relayout(&core, ctx.lang, doc, &t, false);
                        }
                    }
                    if auto && templates::Template::of(doc).is_none_or(|t| !t.is_guide()) {
                        fit::fit_canvas(doc);
                    }
                    // After the canvas fits: it may have moved every object.
                    let out: Vec<Value> = oids
                        .iter()
                        .filter_map(|oid| doc.object(oid))
                        .map(|o| serde_json::to_value(o).unwrap_or_default())
                        .collect();
                    Ok::<_, CommandError>(out)
                })
                .ok_or_else(|| CommandError::internal("document closed"))??;
            // An edited step value of the sequence: the next click goes on from it (EDT-035).
            if let Some(v) = set.get("value").and_then(Value::as_str) {
                for o in updated.iter().filter(|o| o["type"] == "step") {
                    let seq = o["style"]["sequence"]
                        .as_str()
                        .and_then(xs_image::steps::Sequence::from_key)
                        .unwrap_or(xs_image::steps::Sequence::Number);
                    if let Some(n) = seq.index(v) {
                        editor(&core).store.with(&id, |d| {
                            d.step_next.insert(seq.key().to_string(), n + 1);
                        });
                    }
                }
            }
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
        // Several objects stretched, turned or given the same size as one, in one undo
        // step (EDT-080, EDT-083).
        on!("document.transform", |core, ctx, p| {
            let id = resolve(&core, ctx.lang, &p)?;
            let oids = object_ids(&p, ctx.lang)?;
            let kinds = kinds_of(&core, ctx.lang, &id, &oids)?;
            let bad = |param: &str, reason: &str| {
                invalid(tr(
                    ctx.lang,
                    "errors.invalid_param",
                    &[("param", param), ("reason", reason)],
                ))
            };
            let given = |k: &str| p.get(k).is_some_and(|v| !v.is_null());
            if ["scale", "rotate", "equal"].iter().filter(|k| given(k)).count() != 1 {
                return Err(bad("scale", "give one of scale, rotate or equal"));
            }
            // Line widths and text sizes follow, unless the option says no (EDT-080).
            let strokes = p
                .get("strokes")
                .and_then(Value::as_bool)
                .unwrap_or_else(|| core.settings.get_bool("editor.scaleStrokes"));
            let scale = p.get("scale").filter(|v| !v.is_null()).cloned();
            let rotate = p.get("rotate").and_then(Value::as_f64);
            if given("rotate") && rotate.is_none() {
                return Err(bad("rotate", "degrees must be a number"));
            }
            let around = p
                .get("around")
                .and_then(Value::as_array)
                .and_then(|a| Some([a.first()?.as_f64()?, a.get(1)?.as_f64()?]));
            let equal = match p.get("equal").and_then(Value::as_str) {
                Some("width") => Some(true),
                Some("height") => Some(false),
                Some(_) => return Err(bad("equal", "must be width or height")),
                None if given("equal") => return Err(bad("equal", "must be width or height")),
                None => None,
            };
            let size = p.get("size").and_then(Value::as_f64);
            let kind_refs: Vec<&str> = kinds.iter().map(String::as_str).collect();
            let auto = auto_expand(&core);
            let updated = editor(&core)
                .store
                .mutate(&id, &objects::label("change", &kind_refs), |doc| {
                    let dpi = doc.dpi;
                    let measure = |props: &Map<String, Value>| {
                        let t = xs_image::text::Text::from_props(props).ok()?;
                        let l = xs_image::text::layout(&t, dpi);
                        Some((f64::from(l.width), f64::from(l.height)))
                    };
                    let picked: Vec<Object> = doc
                        .objects
                        .iter()
                        .filter(|o| oids.contains(&o.id))
                        .cloned()
                        .collect();
                    let refs: Vec<&Object> = picked.iter().collect();
                    let all = transform::bounds(refs.iter().copied(), &measure)
                        .ok_or_else(|| bad("objectIds", "the objects have no size"))?;
                    let op = if let Some(s) = &scale {
                        let from = match s.get("from") {
                            Some(v) => transform::Rect::from_value(v)
                                .ok_or_else(|| bad("scale.from", "must be [x, y, w, h]"))?,
                            None => all,
                        };
                        let to = match (s.get("to"), s.get("factor")) {
                            (Some(v), _) => transform::Rect::from_value(v)
                                .ok_or_else(|| bad("scale.to", "must be [x, y, w, h]"))?,
                            // By a factor, from the box's top left corner.
                            (None, Some(f)) => {
                                let (fx, fy) = match f {
                                    Value::Array(a) => (
                                        a.first().and_then(Value::as_f64),
                                        a.get(1).and_then(Value::as_f64),
                                    ),
                                    v => (v.as_f64(), v.as_f64()),
                                };
                                let (Some(fx), Some(fy)) = (fx, fy) else {
                                    return Err(bad("scale.factor", "must be a number or [x, y]"));
                                };
                                transform::Rect {
                                    w: from.w * fx,
                                    h: from.h * fy,
                                    ..from
                                }
                            }
                            (None, None) => return Err(bad("scale", "give to or factor")),
                        };
                        if to.w < 0.0 || to.h < 0.0 || from.w < 0.0 || from.h < 0.0 {
                            return Err(bad("scale", "sizes must not be negative"));
                        }
                        transform::Op::Scale { from, to, strokes }
                    } else if let Some(degrees) = rotate {
                        transform::Op::Rotate {
                            center: around.unwrap_or_else(|| all.center()),
                            degrees,
                        }
                    } else {
                        let width = equal.unwrap_or(true);
                        let size = match size {
                            Some(s) if s >= 1.0 => s,
                            Some(_) => return Err(bad("size", "must be at least 1")),
                            None => transform::largest(&refs, width, &measure)
                                .ok_or_else(|| bad("objectIds", "none of them has that side"))?,
                        };
                        transform::Op::Equal { width, size }
                    };
                    for o in &picked {
                        let set = transform::transformed(o, &op, &measure);
                        if set.is_empty() {
                            continue;
                        }
                        let target = doc
                            .object_mut(&o.id)
                            .ok_or_else(|| CommandError::internal("object vanished"))?;
                        objects::update(target, &set).map_err(|r| bad("objectIds", &r))?;
                        objects::fit(target, dpi);
                    }
                    if auto {
                        fit::fit_canvas(doc);
                    }
                    let out: Vec<Value> = oids
                        .iter()
                        .filter_map(|oid| doc.object(oid))
                        .map(|o| serde_json::to_value(o).unwrap_or_default())
                        .collect();
                    Ok::<_, CommandError>(out)
                })
                .ok_or_else(|| CommandError::internal("document closed"))??;
            changed(&core, ctx, &id);
            Ok(json!({ "objects": updated }))
        });
        // Flatten (EDT-091): the objects become part of the image, in one undo step.
        on!("document.flatten", |core, ctx, p| {
            let id = resolve(&core, ctx.lang, &p)?;
            let all = p.get("all").and_then(Value::as_bool).unwrap_or(false);
            let oids = if all {
                Vec::new()
            } else {
                let oids = object_ids(&p, ctx.lang)?;
                kinds_of(&core, ctx.lang, &id, &oids)?;
                oids
            };
            let label = if all {
                "history.flatten_all"
            } else {
                "history.flatten"
            };
            let auto = auto_expand(&core);
            let made = editor(&core)
                .store
                .mutate(&id, label, |doc| {
                    let ids: Vec<String> = if all {
                        doc.objects.iter().map(|o| o.id.clone()).collect()
                    } else {
                        oids.clone()
                    };
                    let made = flatten::flatten(doc, &ids).ok_or_else(|| {
                        invalid(tr(
                            ctx.lang,
                            "errors.invalid_param",
                            &[("param", "objectIds"), ("reason", "there is nothing to flatten")],
                        ))
                    })?;
                    if auto {
                        fit::fit_canvas(doc);
                    }
                    Ok::<_, CommandError>(made)
                })
                .ok_or_else(|| CommandError::internal("document closed"))??;
            changed(&core, ctx, &id);
            Ok(json!({ "layers": made.layers, "permanent": made.permanent }))
        });
        // Unflatten (EDT-091): a flattened layer, or all of them, objects again.
        on!("document.unflatten", |core, ctx, p| {
            let id = resolve(&core, ctx.lang, &p)?;
            let all = p.get("all").and_then(Value::as_bool).unwrap_or(false);
            let layer = p.get("layerId").and_then(Value::as_str).map(str::to_string);
            let bad = |reason: &str| {
                invalid(tr(
                    ctx.lang,
                    "errors.invalid_param",
                    &[("param", "layerId"), ("reason", reason)],
                ))
            };
            if all == layer.is_some() {
                return Err(bad("give layerId or all"));
            }
            let auto = auto_expand(&core);
            let label = if all {
                "history.unflatten_all"
            } else {
                "history.unflatten"
            };
            let ids = editor(&core)
                .store
                .mutate(&id, label, |doc| {
                    let ids = match &layer {
                        Some(l) => flatten::unflatten(doc, l).map_err(|r| match r.as_str() {
                            "not_found" => bad("there is no such flattened layer"),
                            _ => {
                                bad("this flattened layer hides information and cannot become objects again")
                            }
                        })?,
                        None => flatten::unflatten_all(doc),
                    };
                    if ids.is_empty() {
                        return Err(bad("nothing flattened can become objects again"));
                    }
                    if auto {
                        fit::fit_canvas(doc);
                    }
                    Ok::<_, CommandError>(ids)
                })
                .ok_or_else(|| CommandError::internal("document closed"))??;
            changed(&core, ctx, &id);
            Ok(json!({ "ids": ids }))
        });
        // Pixel tools (EDT-047, EDT-048; ADR 0005): one undo step each.
        on!("document.fill", |core, ctx, p| pixel_edit(&core, ctx, &p, true));
        on!("document.erase", |core, ctx, p| pixel_edit(&core, ctx, &p, false));
        // Pixel selection (EDT-043 to EDT-046).
        on!("document.select_pixels", |core, ctx, p| select_pixels(
            &core, ctx, &p
        ));
        on!("document.selection_action", |core, ctx, p| selection_action(
            &core, ctx, &p
        ));
        // Image and canvas (EDT-069 to EDT-077; ADR 0006).
        on!("document.resize", |core, ctx, p| image_command(
            &core, ctx, &p, "resize"
        ));
        on!("document.rotate", |core, ctx, p| image_command(
            &core, ctx, &p, "rotate"
        ));
        on!("document.flip", |core, ctx, p| image_command(
            &core, ctx, &p, "flip"
        ));
        on!("document.trim", |core, ctx, p| image_command(
            &core, ctx, &p, "trim"
        ));
        on!("document.cut_out", |core, ctx, p| image_command(
            &core, ctx, &p, "cut_out"
        ));
        on!("document.fit_canvas", |core, ctx, p| image_command(
            &core,
            ctx,
            &p,
            "fit_canvas"
        ));
        on!("document.crop_to_canvas", |core, ctx, p| image_command(
            &core,
            ctx,
            &p,
            "crop_to_canvas"
        ));
        on!("document.revert", |core, ctx, p| image_command(
            &core, ctx, &p, "revert"
        ));
        on!("document.canvas", |core, ctx, p| image_command(
            &core, ctx, &p, "canvas"
        ));
        on!("document.canvas_size", |core, ctx, p| image_command(
            &core,
            ctx,
            &p,
            "canvas_size"
        ));
        // Effects (EDT-053 to EDT-068).
        on!("document.effects", |core, ctx, p| effects_command(&core, ctx, &p));
        // Saved effects (EDT-054).
        on!("editor.saved_effects", |core, _ctx, _p| Ok(
            json!({ "effects": saved_effects(&core) })
        ));
        on!("editor.saved_effect", |core, ctx, p| saved_effect_command(
            &core, ctx, &p
        ));
        // Stops a Photo fill (EDT-045): the image stays as it was.
        on!("editor.cancel", |core, _ctx, _p| {
            let running = editor(&core).filling.lock().unwrap().clone();
            if let Some(stop) = &running {
                stop.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            Ok(json!({ "cancelled": running.is_some() }))
        });
        // Stacking order (EDT-081): one undo step.
        on!("document.arrange", |core, ctx, p| {
            let id = resolve(&core, ctx.lang, &p)?;
            let oids = object_ids(&p, ctx.lang)?;
            kinds_of(&core, ctx.lang, &id, &oids)?;
            let to = p["to"].as_str().unwrap_or_default().to_string();
            let order = editor(&core)
                .store
                .mutate(&id, "history.arrange", |doc| {
                    objects::arranged(&mut doc.objects, &oids, &to);
                    Ok::<_, CommandError>(doc.objects.iter().map(|o| o.id.clone()).collect::<Vec<_>>())
                })
                .ok_or_else(|| CommandError::internal("document closed"))??;
            changed(&core, ctx, &id);
            Ok(json!({ "order": order }))
        });
        // Groups (EDT-082): one undo step each.
        on!("document.group", |core, ctx, p| {
            let id = resolve(&core, ctx.lang, &p)?;
            let oids = object_ids(&p, ctx.lang)?;
            kinds_of(&core, ctx.lang, &id, &oids)?;
            let ungroup = p.get("ungroup").and_then(Value::as_bool).unwrap_or(false);
            let group = editor(&core)
                .store
                .mutate(
                    &id,
                    if ungroup {
                        "history.ungroup"
                    } else {
                        "history.group"
                    },
                    |doc| {
                        if ungroup {
                            objects::ungroup(&mut doc.objects, &oids);
                            Ok::<_, CommandError>(None)
                        } else {
                            Ok(Some(objects::group(&mut doc.objects, &oids)))
                        }
                    },
                )
                .ok_or_else(|| CommandError::internal("document closed"))??;
            changed(&core, ctx, &id);
            Ok(json!({ "group": group }))
        });
        // Copies of objects, moved by `offset`, on top of the others (EDT-088).
        on!("document.duplicate", |core, ctx, p| {
            let id = resolve(&core, ctx.lang, &p)?;
            let oids = object_ids(&p, ctx.lang)?;
            kinds_of(&core, ctx.lang, &id, &oids)?;
            let (dx, dy) = p
                .get("offset")
                .and_then(Value::as_array)
                .and_then(|a| Some((a.first()?.as_f64()?, a.get(1)?.as_f64()?)))
                .unwrap_or((10.0, 10.0));
            let auto = auto_expand(&core);
            let ids = editor(&core)
                .store
                .mutate(&id, "history.duplicate", |doc| {
                    let mut copies: Vec<Object> = doc
                        .objects
                        .iter()
                        .filter(|o| oids.contains(&o.id))
                        .cloned()
                        .collect();
                    objects::regroup_copies(&doc.objects, &mut copies);
                    let mut ids = Vec::new();
                    for mut copy in copies {
                        copy.id = doc.next_object_id();
                        objects::translate(&mut copy, dx, dy);
                        ids.push(copy.id.clone());
                        doc.objects.push(copy);
                    }
                    if auto {
                        fit::fit_canvas(doc);
                    }
                    Ok::<_, CommandError>(ids)
                })
                .ok_or_else(|| CommandError::internal("document closed"))??;
            changed(&core, ctx, &id);
            Ok(json!({ "ids": ids }))
        });
        on!("document.delete_object", |core, ctx, p| {
            let id = resolve(&core, ctx.lang, &p)?;
            let oids = object_ids(&p, ctx.lang)?;
            let kinds = kinds_of(&core, ctx.lang, &id, &oids)?;
            let kind_refs: Vec<&str> = kinds.iter().map(String::as_str).collect();
            let auto = auto_expand(&core);
            editor(&core)
                .store
                .mutate(&id, &objects::label("delete", &kind_refs), |doc| {
                    doc.objects.retain(|o| !oids.contains(&o.id));
                    if auto {
                        fit::fit_canvas(doc);
                    }
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
            // A video: its cuts, in its file or in another (VID-059, VID-066).
            if ed.store.with(&id, |d| d.is_video()).unwrap_or(false) {
                let target = p.get("path").and_then(Value::as_str).map(PathBuf::from);
                return video::save(&core, ctx, &id, target.as_deref());
            }
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
                keep_editable_copy(&core, &id);
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
        on!("document.export", |core, ctx, p| export_command(&core, ctx, &p));
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
            let base = editor(&core).store.with(&id, |d| d.base_dims()).unwrap_or((1, 1));
            let result = editor(&core)
                .store
                .mutate(&id, "history.crop", |doc| {
                    crop_to(doc, base, [x0, y0, x1, y1]).ok_or_else(|| {
                        invalid(tr(
                            ctx.lang,
                            "errors.invalid_param",
                            &[("param", "rect"), ("reason", "the crop leaves no image")],
                        ))
                    })
                })
                .ok_or_else(|| CommandError::internal("document closed"))??;
            changed(&core, ctx, &id);
            Ok(json!({ "width": result.0, "height": result.1 }))
        });
        on!("document.copy_all", |core, ctx, p| {
            let id = resolve(&core, ctx.lang, &p)?;
            // A video: its file, with its cuts, for apps that take files (VID-066).
            if editor(&core).store.with(&id, |d| d.is_video()).unwrap_or(false) {
                let dir = core.paths.local_dir.join("clipboard");
                // Only the last copy is kept there.
                let _ = std::fs::remove_dir_all(&dir);
                let copy_failed =
                    |_| CommandError::new(ErrorCode::Internal, tr(ctx.lang, "errors.copy_failed", &[]));
                let file = video::current_file(&core, &id, &dir).map_err(copy_failed)?;
                arboard::Clipboard::new()
                    .and_then(|mut c| c.set().file_list(&[&file]))
                    .map_err(|e| copy_failed(e.to_string()))?;
                return Ok(json!({ "files": [file] }));
            }
            // A guide: its text and pictures, with their format (SMT-034).
            let guide = editor(&core)
                .store
                .with(&id, |d| {
                    templates::Template::of(&d.document).is_some_and(|t| t.is_guide())
                })
                .unwrap_or(false);
            if guide {
                guide_export::copy(&core, &id, ctx.lang).map_err(|_| {
                    CommandError::new(ErrorCode::Internal, tr(ctx.lang, "errors.copy_failed", &[]))
                })?;
                return Ok(json!({"guide": true}));
            }
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
            let images = objects
                .iter()
                .filter_map(|o| o.props.get("image").and_then(Value::as_str))
                .filter_map(|name| Some((name.to_string(), snap.images.get(name)?.clone())))
                .collect();
            *ed.copied.lock().unwrap() = Some(Copied {
                objects,
                images,
                from_document: id.clone(),
                fingerprint: image.as_ref().map(fingerprint).unwrap_or(0),
            });
            if p.get("cut").and_then(Value::as_bool).unwrap_or(false) {
                let kind_refs: Vec<&str> = kinds.iter().map(String::as_str).collect();
                let auto = auto_expand(&core);
                ed.store
                    .mutate(&id, &objects::label("cut", &kind_refs), |doc| {
                        doc.objects.retain(|o| !oids.contains(&o.id));
                        if auto {
                            fit::fit_canvas(doc);
                        }
                        Ok::<_, CommandError>(())
                    })
                    .ok_or_else(|| CommandError::internal("document closed"))??;
                changed(&core, ctx, &id);
            }
            Ok(json!({ "count": oids.len() }))
        });
        on!("document.new", |core, ctx, p| new_image(&core, ctx, &p));
        on!("editor.toolbar", |core, ctx, p| toolbar_command(&core, ctx, &p));
        on!("editor.templates", |core, ctx, p| {
            template_cmds::templates_command(&core, ctx, &p)
        });
        on!("document.from_template", |core, ctx, p| {
            template_cmds::from_template(&core, ctx, &p)
        });
        on!("document.template", |core, ctx, p| {
            template_cmds::template_command(&core, ctx, &p)
        });
        on!("document.save_as_template", |core, ctx, p| {
            template_cmds::save_as_template(&core, ctx, &p)
        });
        on!("document.apply_theme", |core, ctx, p| template_cmds::apply_theme(
            &core, ctx, &p
        ));
        on!("document.batch_export", |core, ctx, p| batch_export(
            &core, ctx, &p
        ));
        on!("library.export", |core, ctx, p| library_export::export_library(
            &core, ctx, &p
        ));
        on!("document.print_pages", |core, ctx, p| print_pages(&core, ctx, &p));
        on!("document.print", |core, ctx, p| print_document(&core, ctx, &p));
        on!("smart.guide_export", |core, ctx, p| guide_export::export_command(
            &core, ctx, &p
        ));
        on!("document.new_from_clipboard", |core, ctx, _p| new_from_clipboard(
            &core, ctx
        ));
        // What the clipboard has, for menus that need it (EDT-103, EDT-104).
        on!("editor.clipboard", |_core, _ctx, _p| {
            let (image, text) = read_clipboard();
            let files = clipboard_files();
            Ok(json!({
                "image": image.as_ref().map(|i| json!({"width": i.width(), "height": i.height()})),
                "files": files.len(),
                "text": text.is_some(),
            }))
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
                    .map(|c| (c.objects.clone(), c.from_document.clone(), c.images.clone()))
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
            let (mut objects, kind) = if let Some((objects, from, images)) = ours {
                if !images.is_empty() {
                    ed.store
                        .add_images(&id, &images)
                        .map_err(|e| internal(ctx.lang, "", e))?;
                }
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
            } else if let Some(img) = image {
                // An image from another app: a floating image centered in the view
                // (EDT-078, EDT-089).
                let png = xs_image::encode::png(&img, 0.0, true).map_err(|e| internal(ctx.lang, "", e))?;
                let pic = picture(png, "png", img.width(), img.height());
                let c = center.unwrap_or_else(|| {
                    ed.store
                        .with(&id, |d| {
                            [
                                d.document.canvas.width as f64 / 2.0,
                                d.document.canvas.height as f64 / 2.0,
                            ]
                        })
                        .unwrap_or([0.0, 0.0])
                });
                let at = [
                    (c[0] - img.width() as f64 / 2.0).round(),
                    (c[1] - img.height() as f64 / 2.0).round(),
                ];
                let ids = place_pictures(&core, ctx.lang, &id, &[pic], &[at], "paste")?;
                changed(&core, ctx, &id);
                return Ok(json!({ "pasted": "image", "documentId": id, "ids": ids }));
            } else {
                return Err(CommandError::new(
                    ErrorCode::NotFound,
                    tr(ctx.lang, "errors.nothing_to_paste", &[]),
                ));
            };
            let kinds: Vec<String> = objects.iter().map(|o| o.kind.clone()).collect();
            let kind_refs: Vec<&str> = kinds.iter().map(String::as_str).collect();
            let auto = auto_expand(&core);
            let ids = ed
                .store
                .mutate(&id, &objects::label("paste", &kind_refs), |doc| {
                    let mut ids = Vec::new();
                    for o in objects.iter_mut() {
                        o.id = doc.next_object_id();
                        ids.push(o.id.clone());
                        doc.objects.push(o.clone());
                    }
                    if auto {
                        fit::fit_canvas(doc);
                    }
                    Ok::<_, CommandError>(ids)
                })
                .ok_or_else(|| CommandError::internal("document closed"))??;
            changed(&core, ctx, &id);
            Ok(json!({ "pasted": kind, "documentId": id, "ids": ids }))
        });
        on!("document.insert_images", |core, ctx, p| insert_images(
            &core, ctx, &p
        ));
        on!("document.close", |core, ctx, p| close(&core, ctx, &p));
        on!("tray.list", |core, ctx, p| tray::command(
            &core,
            ctx,
            "tray.list",
            &p
        ));
        on!("tray.open", |core, ctx, p| tray::command(
            &core,
            ctx,
            "tray.open",
            &p
        ));
        on!("tray.activate", |core, ctx, p| tray::command(
            &core,
            ctx,
            "tray.activate",
            &p
        ));
        on!("tray.pin", |core, ctx, p| tray::command(
            &core, ctx, "tray.pin", &p
        ));
        on!("tray.unpin", |core, ctx, p| tray::command(
            &core,
            ctx,
            "tray.unpin",
            &p
        ));
        on!("tray.move", |core, ctx, p| tray::command(
            &core,
            ctx,
            "tray.move",
            &p
        ));
        on!("tray.close", |core, ctx, p| tray::command(
            &core,
            ctx,
            "tray.close",
            &p
        ));
        on!("library.duplicate", |core, ctx, p| duplicate::duplicate(
            &core, ctx, &p
        ));
        on!("document.rename", |core, ctx, p| rename(&core, ctx.lang, &p));
        on!("document.image", |core, ctx, p| document_image(
            &core, ctx.lang, &p
        ));
        on!("editor.show", |core, ctx, p| {
            if p.get("documentId").is_some() {
                let id = resolve(&core, ctx.lang, &p)?;
                activate(&core, &id);
            }
            // LIB-001: the Library view, also for a window that is not loaded yet.
            let view = p["view"].as_str().unwrap_or("editor").to_string();
            *editor(&core).requested_view.lock().unwrap() = Some(view.clone());
            core.host().editor_event("view", json!({ "view": view }));
            core.host().show_main_window()?;
            Ok(json!({}))
        });
        on!("document.locate", |core, ctx, p| {
            let id = resolve(&core, ctx.lang, &p)?;
            let path = PathBuf::from(p["path"].as_str().unwrap_or_default());
            if !path.is_file() {
                return Err(open_error(
                    ctx.lang,
                    &path,
                    OpenError::Unreadable("not found".into()),
                ));
            }
            let ed = editor(&core);
            let format = ed.store.with(&id, |d| d.format);
            if Format::from_path(&path) != format {
                return Err(open_error(ctx.lang, &path, OpenError::Unsupported));
            }
            let kind = if xs_library::contains(&ed.library(), &path) {
                Kind::Library
            } else {
                Kind::External
            };
            ed.store.with(&id, |d| {
                d.path = Some(path.clone());
                d.kind = kind;
            });
            emit_document(&core, &id);
            emit_list(&core);
            Ok(ed.store.with(&id, |d| d.summary()).unwrap_or_default())
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
                "tray": ed.store.tray(),
                "recoverable": recoverable,
                "zoomSteps": ZOOM_STEPS,
                "view": ed.requested_view.lock().unwrap().take(),
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
        // Saved styles, favorites and themes (EDT-015 to EDT-018).
        on!("editor.styles", |core, _ctx, _p| Ok(styles_list(&core)));
        // My colors and recent colors of the color selector (EDT-019).
        on!("editor.colors", |core, _ctx, _p| {
            let (mine, recent, gradients) = colors_of(&core);
            Ok(json!({ "mine": mine, "recent": recent, "gradients": gradients }))
        });
        on!("editor.color", |core, ctx, p| color_command(&core, ctx, &p));
        on!("editor.style", |core, ctx, p| style_command(&core, ctx, &p));
        on!("editor.themes", |core, _ctx, _p| Ok(themes_list(&core)));
        on!("editor.theme", |core, ctx, p| theme_command(&core, ctx, &p));
        on!("editor.set_style", |core, ctx, p| {
            let tool = p["tool"].as_str().unwrap_or_default();
            let changes = p["style"].as_object().cloned().unwrap_or_default();
            let style = set_tool_style(&core, tool, &changes, ctx.lang)?;
            Ok(json!({ "style": style }))
        });
        on!("editor.action", |core, ctx, p| {
            // The editor window does it, with its selection, view and dialogs: what its
            // menus and shortcuts do (EDT-002, EDT-116).
            let action = p["action"].as_str().unwrap_or_default();
            if !xs_core::catalog::EDITOR_ACTIONS.contains(&action) {
                return Err(invalid(tr(
                    ctx.lang,
                    "errors.invalid_param",
                    &[("param", "action"), ("reason", action)],
                )));
            }
            core.host().editor_event("action", json!({ "action": action }));
            Ok(json!({}))
        });
        // The editor window says which menu entries apply now (EDT-002): they are
        // greyed out, never hidden.
        on!("editor.menu_state", |core, _ctx, p| {
            core.host().editor_event("menu_state", p);
            Ok(json!({}))
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
        on!("editor.recovery_list", |core, ctx, _p| {
            // Other parts of the app turn what they left behind into items first (a
            // step recording cut short, SMT-027).
            let preparers = core.service(RecoveryPreparers::default).0.lock().unwrap().clone();
            for prepare in preparers {
                prepare(&core);
            }
            Ok(json!({ "items": recoverable(&core) }))
        });
        on!("editor.recovery_restore", |core, ctx, p| {
            let ids = p["ids"].as_array().cloned().unwrap_or_default();
            let mut opened = Vec::new();
            let dir = editor(&core).recovery_dir();
            for rid in ids.iter().filter_map(Value::as_str) {
                // A recording cut short (VID-022).
                if let Some(entry) = recovery::list(&dir)
                    .into_iter()
                    .find(|e| e.id == rid && e.video.is_some())
                {
                    opened.extend(restore_video(&core, ctx.lang, &dir, &entry)?);
                    continue;
                }
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

type Preparer = Arc<dyn Fn(&Arc<Core>) + Send + Sync>;

/// Work that turns what other parts of the app left behind into recovery items, run
/// before Recover work lists them (APP-047, SMT-027).
#[derive(Default)]
pub struct RecoveryPreparers(Mutex<Vec<Preparer>>);

/// Adds work to run before Recover work lists its items.
pub fn add_recovery_preparer(core: &Core, f: impl Fn(&Arc<Core>) + Send + Sync + 'static) {
    core.service(RecoveryPreparers::default)
        .0
        .lock()
        .unwrap()
        .push(Arc::new(f));
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

/// A recording cut short (VID-022): an MP4 where it would have been saved (another
/// name if that one is taken), in the library, opened if Open in said so. Returns
/// the document opened.
fn restore_video(
    core: &Arc<Core>,
    lang: Lang,
    dir: &std::path::Path,
    entry: &recovery::Entry,
) -> Result<Option<String>, CommandError> {
    let failed = |reason: &str| {
        CommandError::new(
            ErrorCode::Internal,
            tr(lang, "errors.recover_video_failed", &[("reason", reason)]),
        )
    };
    let video = entry.video.as_deref().ok_or_else(|| failed("no video"))?;
    let library = xs_library::library(core);
    let wanted = entry
        .original_path
        .clone()
        .unwrap_or_else(|| library.root().join(format!("{}.mp4", entry.name)));
    let folder = wanted
        .parent()
        .map(std::path::Path::to_path_buf)
        .unwrap_or_else(|| library.root().to_path_buf());
    let _ = std::fs::create_dir_all(&folder);
    let stem = wanted
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| entry.name.clone());
    let dst = if wanted.exists() {
        xs_library::unique(&folder, &stem, "mp4")
    } else {
        wanted
    };
    xs_video::recover(video, &dst).map_err(|e| failed(&e))?;
    recovery::remove(dir, &entry.id);
    library.scan();
    tracing::info!("a recording cut short was recovered");
    if entry.open == Some(false) {
        return Ok(None);
    }
    let opened = core.call(
        Source::Ui,
        "document.open",
        json!({ "path": dst, "show": true }),
        Some(lang),
    )?;
    Ok(opened["documentId"].as_str().map(str::to_string))
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

/// The color that fills transparency in formats without it (EDT-112).
fn flatten_background(core: &Core) -> xs_image::color::Rgba {
    core.settings
        .get("editor.flattenBackground")
        .and_then(|v| v.as_str().and_then(xs_image::color::Rgba::parse))
        .unwrap_or(xs_image::color::Rgba::WHITE)
}

/// `document.export` (EDT-110 to EDT-112): the final image in any export format, with
/// its options; several documents in one PDF or TIFF; `estimate` only measures.
fn export_command(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    use xs_image::export::{Kind, Options, Page};
    let lang = ctx.lang;
    let bad = |param: &str, reason: &str| {
        invalid(tr(
            lang,
            "errors.invalid_param",
            &[("param", param), ("reason", reason)],
        ))
    };
    let ed = editor(core);
    let kind = Kind::parse(p["format"].as_str().unwrap_or("png"))
        .ok_or_else(|| bad("format", "png, jpg, gif, bmp, tiff, pdf, ico or psd"))?;
    let mut opts: Options = serde_json::from_value(p.get("options").cloned().unwrap_or(json!({})))
        .map_err(|e| bad("options", &e.to_string()))?;
    if let Some(q) = p.get("quality").and_then(Value::as_u64) {
        opts.quality = q.clamp(1, 100) as u8;
    }
    opts.check().map_err(|e| bad("options", &e))?;
    // Several documents, one page each (EDT-111); else the one given or shown.
    let ids: Vec<String> = match p.get("documentIds").and_then(Value::as_array) {
        Some(list) if !list.is_empty() => list
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
        _ => vec![resolve(core, lang, p)?],
    };
    if ids.len() > 1 && !kind.multi_page() {
        return Err(bad("documentIds", "only PDF and TIFF hold several images"));
    }
    let mut pages = Vec::new();
    for id in &ids {
        let snap = ed.store.snapshot(id).map_err(|_| {
            CommandError::new(
                ErrorCode::NotFound,
                tr(lang, "errors.document_not_found", &[("id", id)]),
            )
        })?;
        pages.push(Page {
            image: snap.render(),
            dpi: snap.document.dpi,
        });
    }
    let (width, height) = pages
        .first()
        .map(|pg| (pg.image.width(), pg.image.height()))
        .unwrap_or((0, 0));
    let bytes = xs_image::export::encode(&pages, kind, &opts, flatten_background(core))
        .map_err(|e| internal(lang, "", e))?;
    // How big it would be, for the options dialog (EDT-112).
    if p.get("estimate").and_then(Value::as_bool) == Some(true) {
        return Ok(json!({"bytes": bytes.len(), "width": width, "height": height}));
    }
    let mut target = PathBuf::from(
        p["path"]
            .as_str()
            .ok_or_else(|| bad("path", "give the file to write"))?,
    );
    if target.extension().is_none() {
        target.set_extension(kind.ext());
    }
    let name = target
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    if target.exists() {
        match p.get("onConflict").and_then(Value::as_str).unwrap_or("keepBoth") {
            "replace" => {}
            "fail" => {
                return Err(invalid(tr(lang, "errors.file_exists", &[("name", &name)])));
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
    {
        let _guard = ed.writing.lock().unwrap();
        if let Some(parent) = target.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| internal(lang, &name, e.to_string()))?;
        }
        write_file(&target, &bytes).map_err(|e| internal(lang, &name, e.to_string()))?;
    }
    Ok(json!({
        "path": target,
        "width": width,
        "height": height,
        "bytes": bytes.len(),
        "pages": pages.len(),
    }))
}

/// The name of the `index`-th exported file (EDT-113): its own, a base name with a
/// number (`guia_01`), or one name given (`name`, Export of one library item, LIB-018),
/// between a prefix and a suffix.
pub fn batch_name(naming: &Value, original: &str, index: usize) -> String {
    let text = |k: &str| naming.get(k).and_then(Value::as_str).unwrap_or("").to_string();
    let mode = naming.get("mode").and_then(Value::as_str);
    let core = if mode == Some("name") && !text("name").trim().is_empty() {
        text("name")
    } else if mode == Some("base") {
        let start = naming.get("start").and_then(Value::as_u64).unwrap_or(1) as usize;
        let digits = naming
            .get("digits")
            .and_then(Value::as_u64)
            .unwrap_or(2)
            .clamp(1, 9) as usize;
        let base = text("base");
        let base = if base.trim().is_empty() {
            "image".to_string()
        } else {
            base
        };
        format!("{base}_{:0digits$}", start + index)
    } else {
        original.to_string()
    };
    xs_library::file_name(&format!("{}{core}{}", text("prefix"), text("suffix")))
        .unwrap_or_else(|| "image".to_string())
}

/// `document.batch_export` (EDT-113): files (or folders of them) to one format, with a
/// saved effect or a stack added, into a folder or next to each original; names kept
/// or numbered. The sources do not change. Stops with `editor.cancel`; a file that
/// fails is reported and the rest go on.
fn batch_export(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    use xs_image::export::{Kind, Options, Page};
    let lang = ctx.lang;
    let bad = |param: &str, reason: &str| {
        invalid(tr(
            lang,
            "errors.invalid_param",
            &[("param", param), ("reason", reason)],
        ))
    };
    let ed = editor(core);
    let kind = Kind::parse(p["format"].as_str().unwrap_or("png"))
        .ok_or_else(|| bad("format", "png, jpg, gif, bmp, tiff, pdf, ico or psd"))?;
    let opts: Options = serde_json::from_value(p.get("options").cloned().unwrap_or(json!({})))
        .map_err(|e| bad("options", &e.to_string()))?;
    opts.check().map_err(|e| bad("options", &e))?;
    // The files: folders bring their images, in name order.
    let readable = |f: &Path| {
        f.extension()
            .and_then(|e| e.to_str())
            .is_some_and(|e| ["xshot", "png", "jpg", "jpeg"].contains(&e.to_ascii_lowercase().as_str()))
    };
    let mut files: Vec<PathBuf> = Vec::new();
    for v in p.get("paths").and_then(Value::as_array).into_iter().flatten() {
        let Some(path) = v.as_str().map(PathBuf::from) else {
            continue;
        };
        if path.is_dir() {
            let mut inside: Vec<PathBuf> = std::fs::read_dir(&path)
                .map(|r| r.filter_map(|e| e.ok().map(|e| e.path())).collect())
                .unwrap_or_default();
            inside.retain(|f| f.is_file() && readable(f));
            inside.sort();
            files.extend(inside);
        } else {
            files.push(path);
        }
    }
    if files.is_empty() {
        return Err(bad("paths", "give files or folders with images"));
    }
    // Effects added to each one's own: a saved effect (EDT-054) or a stack.
    let extra: Vec<Value> = match p.get("effects") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::String(id)) => saved_effects(core)
            .into_iter()
            .find(|e| e["id"] == json!(id))
            .and_then(|e| e["stack"].as_array().cloned())
            .ok_or_else(|| bad("effects", "no saved effect has that id"))?,
        Some(Value::Array(stack)) => stack.clone(),
        Some(_) => return Err(bad("effects", "a saved effect's id or a stack")),
    };
    for e in &extra {
        xs_image::fx::check(e).map_err(|r| bad("effects", &r))?;
    }
    let folder = p.get("folder").and_then(Value::as_str).map(PathBuf::from);
    let next_to = p.get("nextToOriginal").and_then(Value::as_bool).unwrap_or(false);
    if folder.is_none() && !next_to {
        return Err(bad("folder", "give a folder, or nextToOriginal"));
    }
    let naming = p.get("naming").cloned().unwrap_or(json!({"mode": "keep"}));
    let conflict = p.get("onConflict").and_then(Value::as_str).unwrap_or("keepBoth");
    let background = flatten_background(core);
    // Progress after 300 ms, and editor.cancel stops between files.
    let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    *ed.filling.lock().unwrap() = Some(stop.clone());
    let started = Instant::now();
    let total = files.len();
    let (mut exported, mut failed, mut skipped) = (Vec::new(), Vec::new(), Vec::new());
    let mut cancelled = false;
    // The number of the next name: files that could not be read take none.
    let mut number = 0;
    for (i, file) in files.iter().enumerate() {
        if stop.load(std::sync::atomic::Ordering::Relaxed) {
            cancelled = true;
            break;
        }
        if started.elapsed() >= Duration::from_millis(300) {
            core.host().editor_event(
                "progress",
                json!({ "task": "batch", "done": i as f32 / total as f32, "index": i, "total": total }),
            );
        }
        // Read and encode first: a file that fails takes no number.
        let encoded = (|| -> Result<Vec<u8>, String> {
            let mut doc = DocStore::load(file, &ed.library()).map_err(|e| format!("{e:?}"))?;
            doc.document.effects.extend(extra.iter().cloned());
            let image = DocStore::render_doc(&doc)?;
            xs_image::export::encode(
                &[Page {
                    image,
                    dpi: doc.document.dpi,
                }],
                kind,
                &opts,
                background,
            )
        })();
        let bytes = match encoded {
            Ok(b) => b,
            Err(reason) => {
                failed.push(json!({"source": file, "reason": reason}));
                continue;
            }
        };
        let stem = file
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let dir = if next_to {
            file.parent().map(Path::to_path_buf).unwrap_or_default()
        } else {
            folder.clone().unwrap_or_default()
        };
        let mut target = dir.join(format!("{}.{}", batch_name(&naming, &stem, number), kind.ext()));
        number += 1;
        if target.exists() {
            match conflict {
                "replace" => {}
                "skip" => {
                    skipped.push(json!({"source": file, "path": target}));
                    continue;
                }
                _ => {
                    let s = target
                        .file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    target = xs_library::unique(&dir, &s, kind.ext());
                }
            }
        }
        let written = {
            let _guard = ed.writing.lock().unwrap();
            std::fs::create_dir_all(&dir)
                .and_then(|_| write_file(&target, &bytes))
                .map_err(|e| e.to_string())
        };
        match written {
            Ok(()) => exported.push(json!({"source": file, "path": target})),
            Err(reason) => failed.push(json!({"source": file, "reason": reason})),
        }
    }
    *ed.filling.lock().unwrap() = None;
    if started.elapsed() >= Duration::from_millis(300) {
        core.host()
            .editor_event("progress", json!({ "task": "batch", "done": null }));
    }
    Ok(json!({
        "exported": exported,
        "failed": failed,
        "skipped": skipped,
        "cancelled": cancelled,
    }))
}

/// The pages to print a document on (EDT-114, EDT-102): its final image, or the part
/// of the canvas in `rect`, laid out by `setup`. And the document's name.
fn print_layout(
    core: &Core,
    ctx: &Context,
    p: &Value,
) -> Result<(Vec<xs_image::export::PrintPage>, String), CommandError> {
    let lang = ctx.lang;
    let setup: xs_image::export::PrintSetup =
        serde_json::from_value(p.get("setup").cloned().unwrap_or(json!({}))).map_err(|e| {
            invalid(tr(
                lang,
                "errors.invalid_param",
                &[("param", "setup"), ("reason", &e.to_string())],
            ))
        })?;
    // Files of the library (LIB-017): each fitted on its own page, turned the way it
    // fits best, without opening them; their files do not change.
    if let Some(paths) = p.get("paths").and_then(Value::as_array) {
        let ed = editor(core);
        let fitted = xs_image::export::PrintSetup {
            size: "fit".to_string(),
            orientation: "auto".to_string(),
            ..setup.clone()
        };
        let mut pages = Vec::new();
        let mut first = None;
        for path in paths.iter().filter_map(Value::as_str).map(PathBuf::from) {
            let doc = DocStore::load(&path, &ed.library()).map_err(|e| open_error(lang, &path, e))?;
            let image = DocStore::render_doc(&doc).map_err(|e| internal(lang, "", e))?;
            first.get_or_insert_with(|| doc.name());
            pages.extend(xs_image::export::print_pages(&image, doc.document.dpi, &fitted));
        }
        if pages.is_empty() {
            return Err(invalid(tr(
                lang,
                "errors.invalid_param",
                &[("param", "paths"), ("reason", "give image files")],
            )));
        }
        let name = match (first, paths.len()) {
            (Some(n), 1) => n,
            (_, n) => format!("XcreenShot ({n})"),
        };
        return Ok((pages, name));
    }
    let id = resolve(core, lang, p)?;
    let snap = editor(core)
        .store
        .snapshot(&id)
        .map_err(|e| internal(lang, "", e))?;
    let (mut image, origin) = snap.render_with_origin();
    // Print a selection (EDT-114): its rectangle of the canvas.
    if let Some(r) = p.get("rect") {
        let n = |k: &str| r.get(k).and_then(Value::as_f64).unwrap_or(0.0);
        let x0 = (n("x").floor() as i64 + origin[0]).clamp(0, image.width() as i64) as u32;
        let y0 = (n("y").floor() as i64 + origin[1]).clamp(0, image.height() as i64) as u32;
        let x1 = ((n("x") + n("w")).ceil() as i64 + origin[0]).clamp(0, image.width() as i64) as u32;
        let y1 = ((n("y") + n("h")).ceil() as i64 + origin[1]).clamp(0, image.height() as i64) as u32;
        if x1 > x0 && y1 > y0 {
            image = image::imageops::crop_imm(&image, x0, y0, x1 - x0, y1 - y0).to_image();
        }
    }
    Ok((
        xs_image::export::print_pages(&image, snap.document.dpi, &setup),
        snap.name.clone(),
    ))
}

/// `document.print_pages` (EDT-114): each page's picture as PNG and where it goes, in
/// millimeters, for the preview.
fn print_pages(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    use base64::Engine as _;
    let (pages, _) = print_layout(core, ctx, p)?;
    let mut out = Vec::new();
    for page in pages {
        let png = xs_image::encode::png(&page.image, 0.0, true).map_err(|e| internal(ctx.lang, "", e))?;
        out.push(json!({
            "png": base64::engine::general_purpose::STANDARD.encode(png),
            "paper": {"w": page.paper.0, "h": page.paper.1},
            "at": {"x": page.at.0, "y": page.at.1},
            "size": {"w": page.size.0, "h": page.size.1},
        }));
    }
    Ok(json!({ "pages": out }))
}

/// `document.print` (EDT-114): the pages as a PDF with that paper, orientation and
/// margins, opened in the system's viewer to print with its dialog. `open: false`
/// only writes it.
fn print_document(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    // A guide prints as its PDF: steps never split, page numbers (SMT-033). Library
    // files and other documents print as before.
    let guide = resolve(core, lang, p).ok().filter(|id| {
        editor(core)
            .store
            .with(id, |d| {
                templates::Template::of(&d.document).is_some_and(|t| t.is_guide())
            })
            .unwrap_or(false)
    });
    let (bytes, count, name) = if let Some(id) = guide.filter(|_| p.get("paths").is_none()) {
        let bytes = guide_export::pdf_bytes(core, &id, lang, "auto").map_err(|e| internal(lang, "", e))?;
        let name = editor(core).store.with(&id, |d| d.name()).unwrap_or_default();
        (bytes, 0, name)
    } else {
        let (pages, name) = print_layout(core, ctx, p)?;
        let count = pages.len();
        let bytes = xs_image::export::print_pdf(&pages, flatten_background(core))
            .map_err(|e| internal(lang, "", e))?;
        (bytes, count, name)
    };
    let ed = editor(core);
    let dir = ed.unsaved_dir().with_file_name("print");
    let file = xs_library::file_name(&name).unwrap_or_else(|| "XcreenShot".to_string());
    let path = dir.join(format!("{file}.pdf"));
    {
        let _guard = ed.writing.lock().unwrap();
        std::fs::create_dir_all(&dir)
            .and_then(|_| write_file(&path, &bytes))
            .map_err(|e| internal(lang, &file, e.to_string()))?;
    }
    if p.get("open").and_then(Value::as_bool) != Some(false) {
        core.host().open_path(&path)?;
    }
    Ok(json!({ "path": path, "pages": count }))
}

/// The toolbar of EDT-003: the tools it shows when nobody changed it (EDT-004).
pub const TOOLBAR_DEFAULT: [&str; 10] = [
    "tool:favorites",
    "tool:move",
    "tool:arrow",
    "tool:text",
    "tool:callout",
    "tool:shape",
    "tool:step",
    "tool:blur",
    "tool:crop",
    "tool:select",
];
const TOOLBAR_KEY: &str = "editor.toolbar";
/// Commands that can go on the toolbar (EDT-004), as `action:<editor action>`.
const TOOLBAR_ACTIONS: [&str; 10] = [
    "save",
    "save_as",
    "export",
    "new_from_clipboard",
    "copy_all",
    "revert",
    "trim",
    "undo",
    "redo",
    "print",
];

/// Whether `item` can be on the toolbar: a tool of this build, a command, an effect
/// (`action:effect_…`), a separator or a flexible space (EDT-004).
fn toolbar_item_ok(item: &str) -> bool {
    match item.split_once(':') {
        Some(("tool", t)) => AVAILABLE_TOOLS.contains(&t),
        Some(("action", a)) => {
            TOOLBAR_ACTIONS.contains(&a)
                || (a.starts_with("effect_") && xs_core::catalog::EDITOR_ACTIONS.contains(&a))
        }
        _ => item == "sep" || item == "space",
    }
}

/// `editor.toolbar` (EDT-003, EDT-004): what the toolbar shows, in order; `set` keeps
/// a new order, `reset` goes back to the default. Kept between sessions.
fn toolbar_command(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let current = || -> Vec<String> {
        core.recall(TOOLBAR_KEY)
            .and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
            .map(|items| items.into_iter().filter(|i| toolbar_item_ok(i)).collect())
            .unwrap_or_else(|| TOOLBAR_DEFAULT.iter().map(|s| s.to_string()).collect())
    };
    match p.get("action").and_then(Value::as_str).unwrap_or("get") {
        "get" => {}
        "set" => {
            let items: Vec<String> = p
                .get("items")
                .cloned()
                .and_then(|v| serde_json::from_value(v).ok())
                .ok_or_else(|| {
                    invalid(tr(
                        lang,
                        "errors.invalid_param",
                        &[("param", "items"), ("reason", "a list of toolbar items")],
                    ))
                })?;
            if let Some(bad) = items.iter().find(|i| !toolbar_item_ok(i)) {
                return Err(invalid(tr(
                    lang,
                    "errors.invalid_param",
                    &[("param", "items"), ("reason", bad.as_str())],
                )));
            }
            // A tool appears once.
            let mut seen = std::collections::HashSet::new();
            let items: Vec<String> = items
                .into_iter()
                .filter(|i| i == "sep" || i == "space" || seen.insert(i.clone()))
                .collect();
            core.remember(TOOLBAR_KEY, json!(items));
        }
        "reset" => core.forget(TOOLBAR_KEY),
        other => {
            return Err(invalid(tr(
                lang,
                "errors.invalid_param",
                &[("param", "action"), ("reason", other)],
            )))
        }
    }
    let effects: Vec<&str> = xs_core::catalog::EDITOR_ACTIONS
        .iter()
        .copied()
        .filter(|a| a.starts_with("effect_"))
        .collect();
    Ok(json!({
        "items": current(),
        "default": TOOLBAR_DEFAULT,
        "available": {
            "tools": AVAILABLE_TOOLS,
            "actions": TOOLBAR_ACTIONS,
            "effects": effects,
        },
    }))
}

/// File > New image (EDT-103): a blank canvas of the size, color and resolution
/// given, or of the remembered ones; with `remember`, these become the defaults.
fn new_image(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let bad = |param: &str, reason: &str| {
        invalid(tr(
            lang,
            "errors.invalid_param",
            &[("param", param), ("reason", reason)],
        ))
    };
    let setting = |k: &str| core.settings.get(k).unwrap_or(Value::Null);
    let side = |k: &str, fallback: u64| -> Result<u32, CommandError> {
        let v = p
            .get(k)
            .and_then(Value::as_f64)
            .or_else(|| setting(&format!("editor.newImage.{k}")).as_f64())
            .unwrap_or(fallback as f64)
            .round();
        if !(1.0..=f64::from(xs_image::encode::MAX_SIDE)).contains(&v) {
            return Err(bad(k, "between 1 and 32000 px (APP-050)"));
        }
        Ok(v as u32)
    };
    let (w, h) = (side("width", 1280)?, side("height", 720)?);
    let fill = match p.get("color") {
        Some(v) => v.clone(),
        None => setting("editor.newImage.color"),
    };
    canvas_fill(&fill).map_err(|e| bad("color", &e))?;
    let dpi = p.get("dpi").and_then(Value::as_f64).unwrap_or(96.0);
    if !(1.0..=10_000.0).contains(&dpi) {
        return Err(bad("dpi", "between 1 and 10000"));
    }
    if p.get("remember").and_then(Value::as_bool) == Some(true) {
        // A gradient is not remembered: the setting keeps a color or transparent.
        let color = if fill.is_object() {
            setting("editor.newImage.color")
        } else {
            fill.clone()
        };
        for (k, v) in [
            ("editor.newImage.width", json!(w)),
            ("editor.newImage.height", json!(h)),
            ("editor.newImage.color", color),
        ] {
            core.settings.set(k, v, ctx.source, lang)?;
        }
    }
    let blank = image::RgbaImage::new(w, h);
    let name = tr(lang, "editor.new_image_name", &[]);
    let (id, path) = add_image_with(
        core,
        &blank,
        json!({ "source": "newImage" }),
        dpi / 96.0,
        Some(&name),
        Some(&fill),
    )
    .map_err(|e| internal(lang, "", e))?;
    activate(core, &id);
    let _ = core.host().show_main_window();
    Ok(json!({ "documentId": id, "path": path, "width": w, "height": h }))
}

/// File > New from clipboard (EDT-104): the clipboard's image, image file or text as a
/// new document of the library.
fn new_from_clipboard(core: &Arc<Core>, ctx: &Context) -> CommandResult {
    let lang = ctx.lang;
    let ed = editor(core);
    let (image, text) = read_clipboard();
    let from_file = clipboard_files().into_iter().find_map(|f| {
        let bytes = std::fs::read(&f).ok()?;
        xs_image::encode::decode(&bytes).ok()
    });
    let name = tr(lang, "editor.clipboard_name", &[]);
    let meta = json!({ "source": "clipboard" });
    let id = if let Some(img) = image.or(from_file) {
        add_image(core, &img, meta, 1.0, Some(&name))
            .map_err(|e| internal(lang, "", e))?
            .0
    } else if let Some(text) = text {
        // Text: a canvas that fits it, with a text object (EDT-104).
        const PAD: f64 = 20.0;
        let (kind, props) = objects::new_object(
            &json!({"type": "text", "at": [PAD, PAD], "text": text}),
            &ed.styles(),
        )
        .map_err(|r| {
            invalid(tr(
                lang,
                "errors.invalid_object",
                &[("index", "1"), ("reason", &r)],
            ))
        })?;
        let size = xs_image::text::Text::from_props(&props)
            .ok()
            .map(|t| xs_image::text::layout(&t, 96.0))
            .map(|l| (f64::from(l.width), f64::from(l.height)))
            .unwrap_or((200.0, 40.0));
        let (w, h) = (
            (size.0 + 2.0 * PAD).ceil().clamp(1.0, 50_000.0) as u32,
            (size.1 + 2.0 * PAD).ceil().clamp(1.0, 50_000.0) as u32,
        );
        let white = image::RgbaImage::from_pixel(w, h, image::Rgba([255, 255, 255, 255]));
        let (id, _) = add_image(core, &white, meta, 1.0, Some(&name)).map_err(|e| internal(lang, "", e))?;
        ed.store
            .mutate(&id, "history.paste", |d| {
                let oid = d.next_object_id();
                d.objects.push(Object { id: oid, kind, props });
                Ok::<_, CommandError>(())
            })
            .ok_or_else(|| CommandError::internal("document closed"))??;
        id
    } else {
        return Err(CommandError::new(
            ErrorCode::NotFound,
            tr(lang, "errors.nothing_to_paste", &[]),
        ));
    };
    activate(core, &id);
    let _ = core.host().show_main_window();
    let path = ed.store.with(&id, |d| d.path.clone()).flatten();
    Ok(json!({ "documentId": id, "path": path }))
}

/// Image files copied in the file manager (EDT-104).
fn clipboard_files() -> Vec<PathBuf> {
    let _lock = xs_core::clipboard_lock();
    arboard::Clipboard::new()
        .ok()
        .and_then(|mut cb| cb.get().file_list().ok())
        .unwrap_or_default()
        .into_iter()
        .filter(|p| {
            p.extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| ["png", "jpg", "jpeg"].contains(&e.to_ascii_lowercase().as_str()))
        })
        .collect()
}

fn read_clipboard() -> (Option<image::RgbaImage>, Option<String>) {
    let _lock = xs_core::clipboard_lock();
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
    let _lock = xs_core::clipboard_lock();
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
