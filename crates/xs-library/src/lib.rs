//! The library (LIB-): its location (LIB-027), where each capture is saved as `.xshot`
//! (LIB-020), the index kept in step with the folder (LIB-022), thumbnails, and the
//! `library.*` commands: list, get, delete with Undo, rename and show in the folder.

pub mod conflict;
pub mod exif;
pub mod import;
pub mod index;
pub mod merge;
pub mod query;
pub mod recycle;
pub mod relocate;
pub mod scan;
pub mod service;
pub mod sync;
pub mod xmeta;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Weak};

use serde_json::{json, Value};
use xs_core::core::Extension;
use xs_core::i18n::tr;
use xs_core::{CommandError, Context, Core, ErrorCode, Lang, Source};

pub use index::Item;
pub use service::{synced, Library};

/// The library of `core`, opened at its location the first time.
pub fn library(core: &Core) -> Arc<Library> {
    core.service(|| Library::open(location_for(core), &core.paths.local_dir))
}

/// Where the user put the library (LIB-028), kept by the core.
const LOCATION_KEY: &str = "library.location";

/// The library folder of `core`: the test override, the one the user chose (LIB-028),
/// or the default (LIB-027).
pub fn location_for(core: &Core) -> PathBuf {
    if let Some(dir) = std::env::var_os("XCREENSHOT_LIBRARY_DIR") {
        return PathBuf::from(dir);
    }
    core.recall(LOCATION_KEY)
        .and_then(|v| v.as_str().map(PathBuf::from))
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(location)
}

type Relocated = Box<dyn Fn(&Core, &Path, &Path) + Send + Sync>;
static RELOCATED: std::sync::OnceLock<Relocated> = std::sync::OnceLock::new();

/// What else follows the library to a new folder (the editor's open documents): called
/// with the old and the new folder (LIB-028).
pub fn on_relocated(f: impl Fn(&Core, &Path, &Path) + Send + Sync + 'static) {
    let _ = RELOCATED.set(Box::new(f));
}

/// Indexes the folder in the background, then follows changes made outside
/// XcreenShot (LIB-022). The shell calls it once at startup.
pub fn start(core: &Arc<Core>) {
    let weak = Arc::downgrade(core);
    std::thread::spawn(move || {
        let Some(core) = weak.upgrade() else { return };
        let lib = library(&core);
        let started = std::time::Instant::now();
        lib.scan();
        tracing::info!(
            items = lib.list().len(),
            elapsed_ms = started.elapsed().as_millis() as u64,
            synced = synced(lib.root()),
            "library indexed"
        );
        changed(&core);
        let weak = Arc::downgrade(&core);
        drop(core);
        lib.watch(move || {
            if let Some(core) = weak.upgrade() {
                changed(&core);
            }
        });
    });
}

/// Tells the main window that the library changed: the Library view and the strip
/// (a file that disappeared) refresh.
fn changed(core: &Core) {
    core.host().editor_event("library", json!({}));
}

/// Library folder: `Documents\XcreenShot` on Windows (the real Documents folder, also
/// when OneDrive redirects it), `~/Pictures/XcreenShot` on macOS (LIB-027).
/// `XCREENSHOT_LIBRARY_DIR` overrides it for isolated test instances.
pub fn location() -> PathBuf {
    if let Some(dir) = std::env::var_os("XCREENSHOT_LIBRARY_DIR") {
        return PathBuf::from(dir);
    }
    let dirs = directories::UserDirs::new();
    let base = if cfg!(target_os = "macos") {
        dirs.as_ref().and_then(|d| d.picture_dir().map(Path::to_path_buf))
    } else {
        dirs.as_ref()
            .and_then(|d| d.document_dir().map(Path::to_path_buf))
    };
    base.unwrap_or_else(std::env::temp_dir).join("XcreenShot")
}

/// Whether `path` is inside the library folder `root`. Library documents save
/// themselves; files from elsewhere ask (EDT-012).
pub fn contains(root: &Path, path: &Path) -> bool {
    let canon = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let file_dir = path.parent().map(canon).unwrap_or_default();
    file_dir.starts_with(canon(root))
}

/// Characters that no file name may contain on Windows or macOS.
const INVALID: &[char] = &['\\', '/', ':', '*', '?', '"', '<', '>', '|'];

/// A source app name that is safe as part of a file name, or `XcreenShot`.
pub fn safe_name(name: Option<&str>) -> String {
    let cleaned: String = name
        .unwrap_or_default()
        .chars()
        .filter(|c| !c.is_control() && !INVALID.contains(c))
        .take(60)
        .collect();
    let cleaned = cleaned.trim().trim_end_matches('.').trim().to_string();
    if cleaned.is_empty() {
        "XcreenShot".to_string()
    } else {
        cleaned
    }
}

/// A name the user (or a suggestion, SMT-041) gave a file: invalid characters become
/// spaces, runs of spaces one, no trailing dots, at most 100 characters, and never a
/// name Windows reserves (`CON`, `NUL`, `COM1`…). `None` if nothing is left.
pub fn file_name(name: &str) -> Option<String> {
    let spaced: String = name
        .chars()
        .map(|c| {
            if c.is_control() || INVALID.contains(&c) {
                ' '
            } else {
                c
            }
        })
        .collect();
    let joined = spaced.split_whitespace().collect::<Vec<_>>().join(" ");
    let cut: String = joined.chars().take(100).collect();
    let cleaned = cut.trim().trim_end_matches('.').trim().to_string();
    if cleaned.is_empty() {
        return None;
    }
    let upper = cleaned.to_ascii_uppercase();
    let reserved = ["CON", "PRN", "AUX", "NUL"].contains(&upper.as_str())
        || ((upper.starts_with("COM") || upper.starts_with("LPT"))
            && upper.len() == 4
            && upper.as_bytes()[3].is_ascii_digit());
    Some(if reserved { format!("{cleaned}_") } else { cleaned })
}

/// Where a new library item goes (LIB-020): a `AAAA-MM` folder per month and a name
/// like `Chrome 2026-10-12 14.03.22.xshot`, with ` (2)`, ` (3)`… if it exists.
/// Provisional pattern until SHR-006 (file names) is written.
pub fn new_item_path(
    root: &Path,
    app: Option<&str>,
    now: chrono::DateTime<chrono::Local>,
    extension: &str,
) -> PathBuf {
    let dir = root.join(now.format("%Y-%m").to_string());
    let stem = format!("{} {}", safe_name(app), now.format("%Y-%m-%d %H.%M.%S"));
    unique(&dir, &stem, extension)
}

/// `dir/stem.ext`, or `dir/stem (2).ext`… if taken (also by a half-written file).
pub fn unique(dir: &Path, stem: &str, extension: &str) -> PathBuf {
    let taken = |p: &Path| {
        let mut partial = p.as_os_str().to_os_string();
        partial.push(".partial");
        p.exists() || Path::new(&partial).exists()
    };
    let mut path = dir.join(format!("{stem}.{extension}"));
    let mut n = 2;
    while taken(&path) {
        path = dir.join(format!("{stem} ({n}).{extension}"));
        n += 1;
    }
    path
}

// ---------------------------------------------------------------------------
// Commands (LIB-034 brings the rest of the `library` area in M2)

fn not_found(lang: Lang, id: &str) -> CommandError {
    CommandError::new(
        ErrorCode::NotFound,
        tr(lang, "errors.library_not_found", &[("id", id)]),
    )
}

fn ids_of(p: &Value) -> Vec<String> {
    p["ids"]
        .as_array()
        .map(|a| a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect())
        .unwrap_or_default()
}

/// `library.list` (LIB-004 to LIB-007, LIB-034): the items that pass the filters and
/// the search, in order, with the counts of the filter pane.
fn list(core: &Core, ctx: &Context, p: &Value) -> Result<Value, CommandError> {
    let lang = ctx.lang;
    let bad = |param: &str, reason: &str| {
        CommandError::new(
            ErrorCode::InvalidParams,
            tr(
                lang,
                "errors.invalid_param",
                &[("param", param), ("reason", reason)],
            ),
        )
    };
    let lib = library(core);
    let all = lib.list();
    let sort: query::Sort = match p.get("sort") {
        None | Some(Value::Null) => query::Sort::Modified,
        Some(v) => serde_json::from_value(v.clone())
            .map_err(|_| bad("sort", "modified, captured, name, size, type or app"))?,
    };
    // Newest and largest first unless asked; names and the rest, A to Z.
    let descending = match p.get("order").and_then(Value::as_str) {
        Some("asc") => false,
        Some("desc") => true,
        Some(_) => return Err(bad("order", "asc or desc")),
        None => matches!(
            sort,
            query::Sort::Modified | query::Sort::Captured | query::Sort::Size
        ),
    };
    let filters: query::Filters = match p.get("filters") {
        None | Some(Value::Null) => query::Filters::default(),
        Some(v) => serde_json::from_value(v.clone()).map_err(|e| bad("filters", &e.to_string()))?,
    };
    let search = p.get("query").and_then(Value::as_str).unwrap_or("");
    let found = query::run(&all, &filters, search, sort, descending, query::today());
    let offset = p["offset"].as_u64().unwrap_or(0) as usize;
    let limit = p["limit"].as_u64().map(|l| l as usize).unwrap_or(usize::MAX);
    let conflicts = conflicts_of(&all);
    let items: Vec<Value> = found
        .items
        .iter()
        .skip(offset)
        .take(limit)
        .map(|i| with_conflict(i.to_json(lib.root()), &conflicts, &i.id))
        .collect();
    let (place, service) = place_of(lib.root());
    Ok(json!({
        "root": lib.root(),
        "synced": synced(lib.root()),
        // LIB-026: local, synced (with its service), network or unavailable.
        "place": place,
        "service": service,
        "total": all.len(),
        "count": found.items.len(),
        "items": items,
        "facets": found.facets,
    }))
}

fn ctx_is_relocate(p: &Value) -> bool {
    p.get("relocate").and_then(Value::as_bool) == Some(true)
}

/// The last move of the library, for Retry of what it left behind (LIB-028).
static LAST_MOVE: std::sync::Mutex<Option<(PathBuf, PathBuf)>> = std::sync::Mutex::new(None);
/// The move in progress, to cancel it (LIB-028).
static MOVING: std::sync::Mutex<Option<Arc<std::sync::atomic::AtomicBool>>> = std::sync::Mutex::new(None);
/// The merge in progress, to cancel it (LIB-031).
static MERGING: std::sync::Mutex<Option<Arc<std::sync::atomic::AtomicBool>>> = std::sync::Mutex::new(None);

/// `library.merge` (LIB-031): copies another library's items into this one, in the
/// background with `library_merge` progress events, or waiting for the report.
fn merge_command(core: &Arc<Core>, ctx: &Context, p: &Value) -> Result<Value, CommandError> {
    let lang = ctx.lang;
    let bad = |param: &str, reason: &str| {
        CommandError::new(
            ErrorCode::InvalidParams,
            tr(
                lang,
                "errors.invalid_param",
                &[("param", param), ("reason", reason)],
            ),
        )
    };
    if p.get("cancel").and_then(Value::as_bool) == Some(true) {
        let running = MERGING.lock().unwrap().clone();
        if let Some(flag) = &running {
            flag.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        return Ok(json!({ "cancelled": running.is_some() }));
    }
    let other = p
        .get("path")
        .and_then(Value::as_str)
        .map(PathBuf::from)
        .filter(|p| p.is_absolute())
        .ok_or_else(|| bad("path", "give the full path of the other library's folder"))?;
    if !other.is_dir() {
        return Err(CommandError::new(
            ErrorCode::NotFound,
            tr(
                lang,
                "errors.library_not_found",
                &[("id", &other.to_string_lossy())],
            ),
        ));
    }
    let lib = library(core);
    if merge::overlaps(&other, lib.root()) {
        return Err(bad("path", "choose a folder outside this library"));
    }
    if MOVING.lock().unwrap().is_some() {
        return Err(CommandError::new(
            ErrorCode::Busy,
            tr(lang, "errors.library_moving", &[]),
        ));
    }
    let mut merging = MERGING.lock().unwrap();
    if merging.is_some() {
        return Err(CommandError::new(
            ErrorCode::Busy,
            tr(lang, "errors.library_merging", &[]),
        ));
    }
    let flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
    *merging = Some(flag.clone());
    drop(merging);
    let report = |r: &merge::Merged| {
        json!({
            "added": r.added,
            "duplicates": r.duplicates,
            "enriched": r.enriched,
            "renamed": r.renamed.iter().map(|(from, to)| json!({ "from": from, "path": to })).collect::<Vec<_>>(),
            "failed": r.failed,
            "cancelled": r.cancelled,
        })
    };
    if p.get("background").and_then(Value::as_bool) != Some(true) {
        let r = merge::merge(&lib, &other, &flag, |_, _| {});
        *MERGING.lock().unwrap() = None;
        changed(core);
        return Ok(report(&r));
    }
    let total = scan::walk(&other).len();
    let weak = Arc::downgrade(core);
    std::thread::spawn(move || {
        let Some(core) = weak.upgrade() else {
            *MERGING.lock().unwrap() = None;
            return;
        };
        let host = core.host();
        // The same progress events as the exports, for the Library's status bar.
        let r = merge::merge(&lib, &other, &flag, |done, total| {
            host.editor_event(
                "progress",
                json!({ "task": "library_merge", "done": done as f32 / total.max(1) as f32, "index": done, "total": total }),
            );
        });
        *MERGING.lock().unwrap() = None;
        changed(&core);
        host.editor_event("progress", json!({ "task": "library_merge", "done": null }));
        let mut end = report(&r);
        end["finished"] = json!(true);
        end["path"] = json!(other);
        host.editor_event("library_merge", end);
    });
    Ok(json!({ "started": true, "total": total }))
}

/// Uses `to` as the library from now on: kept, opened, indexed and followed; open
/// documents follow it (LIB-028).
fn switch_to(core: &Arc<Core>, from: &Path, to: &Path) {
    core.remember(LOCATION_KEY, json!(to));
    core.replace_service(Library::open(to.to_path_buf(), &core.paths.local_dir));
    if let Some(f) = RELOCATED.get() {
        f(core, from, to);
    }
    start(core);
}

/// `library.location` (LIB-028): where the library is; what a folder holds; and
/// changing it (moving the items, starting an empty library or using one that is
/// there), in the background with progress events (`library_move`) and Cancel.
fn location_command(core: &Arc<Core>, ctx: &Context, p: &Value) -> Result<Value, CommandError> {
    let lang = ctx.lang;
    let bad = |param: &str, reason: &str| {
        CommandError::new(
            ErrorCode::InvalidParams,
            tr(
                lang,
                "errors.invalid_param",
                &[("param", param), ("reason", reason)],
            ),
        )
    };
    let lib = library(core);
    let path = || {
        p.get("path")
            .and_then(Value::as_str)
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .ok_or_else(|| bad("path", "give a full folder path"))
    };
    let action = if ctx_is_relocate(p) {
        p.get("action").and_then(Value::as_str).unwrap_or("set")
    } else {
        p.get("action").and_then(Value::as_str).unwrap_or("get")
    };
    match action {
        "get" => {
            // The sync services' folders, each with XcreenShot's folder inside it, and
            // whether a library is already there (LIB-029).
            let services: Vec<Value> = sync::folders()
                .into_iter()
                .map(|f| {
                    let path = f.path.join("XcreenShot");
                    json!({
                        "service": f.service,
                        "account": f.account,
                        "folder": f.path,
                        "path": path,
                        "library": relocate::looks_like_library(&path),
                        "current": same_path(&path, lib.root()),
                    })
                })
                .collect();
            Ok(json!({
                "path": lib.root(),
                "default": location(),
                "synced": synced(lib.root()),
                "moving": MOVING.lock().unwrap().is_some(),
                "services": services,
            }))
        }
        "inspect" => {
            let dir = path()?;
            let i = relocate::inspect(&dir);
            // LIB-028: is there room for a move? On the same disk it is a rename.
            let size: u64 = lib.list().iter().map(|i| i.size).sum();
            let free = xs_core::platform::free_space(&dir);
            let same_disk = xs_core::platform::same_volume(&dir, lib.root());
            let enough = same_disk || free.is_none_or(|f| f > size + size / 20);
            Ok(json!({
                "path": dir,
                "exists": i.exists,
                "items": i.items,
                "library": i.library,
                "writable": i.writable,
                "synced": synced(&dir),
                "same": same_path(&dir, lib.root()),
                "size": size,
                "free": free,
                "sameDisk": same_disk,
                "enough": enough,
                // LIB-029: more than 5 GB into a synced folder takes room in the service.
                "bigSync": synced(&dir) && size > 5 * 1024 * 1024 * 1024,
            }))
        }
        "retry" => {
            // LIB-028: the files a move left behind, tried again into the library.
            let Some((from, to)) = LAST_MOVE.lock().unwrap().clone() else {
                return Ok(json!({ "moved": 0, "failed": [] }));
            };
            let files: Vec<PathBuf> = p
                .get("retry")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(Value::as_str)
                .map(PathBuf::from)
                .collect();
            let r = relocate::move_files(&from, &to, &files);
            changed(core);
            Ok(json!({ "moved": r.moved, "failed": r.failed }))
        }
        "cancel" => {
            if let Some(flag) = MOVING.lock().unwrap().as_ref() {
                flag.store(true, std::sync::atomic::Ordering::Relaxed);
            }
            Ok(json!({}))
        }
        "set" => {
            let to = path()?;
            let from = lib.root().to_path_buf();
            if same_path(&to, &from) || to.starts_with(&from) || from.starts_with(&to) {
                return Err(bad("path", "choose a folder outside the library"));
            }
            let mode = p.get("mode").and_then(Value::as_str).unwrap_or("move");
            if MERGING.lock().unwrap().is_some() {
                return Err(CommandError::new(
                    ErrorCode::Busy,
                    tr(lang, "errors.library_merging", &[]),
                ));
            }
            let mut moving = MOVING.lock().unwrap();
            if moving.is_some() {
                return Err(CommandError::new(
                    ErrorCode::Busy,
                    tr(lang, "errors.library_moving", &[]),
                ));
            }
            std::fs::create_dir_all(&to).map_err(|e| CommandError::internal(e.to_string()))?;
            match mode {
                "empty" | "use" => {
                    drop(moving);
                    switch_to(core, &from, &to);
                    Ok(json!({ "path": to, "started": false }))
                }
                "move" => {
                    let flag = Arc::new(std::sync::atomic::AtomicBool::new(false));
                    *moving = Some(flag.clone());
                    drop(moving);
                    let weak = Arc::downgrade(core);
                    let total = lib.list().len();
                    let target = to.clone();
                    std::thread::spawn(move || {
                        let to = target;
                        let Some(core) = weak.upgrade() else { return };
                        let host = core.host();
                        let r = relocate::move_library(&from, &to, &flag, |done, total| {
                            host.editor_event("library_move", json!({ "done": done, "total": total }));
                        });
                        *MOVING.lock().unwrap() = None;
                        if !r.cancelled {
                            switch_to(&core, &from, &to);
                            *LAST_MOVE.lock().unwrap() = Some((from.clone(), to.clone()));
                        }
                        changed(&core);
                        let at = if r.cancelled { &from } else { &to };
                        core.host().editor_event(
                            "library_move",
                            json!({
                                "finished": true,
                                "cancelled": r.cancelled,
                                "moved": r.moved,
                                "failed": r.failed,
                                "path": at,
                            }),
                        );
                    });
                    Ok(json!({ "path": to, "started": true, "total": total }))
                }
                other => Err(bad("mode", other)),
            }
        }
        other => Err(bad("action", other)),
    }
}

/// Where the library is (LIB-026): `local`, `synced` (with the service), `network` or
/// `unavailable`. The services are looked up once per folder.
fn place_of(root: &Path) -> (&'static str, Option<&'static str>) {
    static SEEN: std::sync::Mutex<Option<(PathBuf, Option<&'static str>)>> = std::sync::Mutex::new(None);
    if !root.is_dir() {
        return ("unavailable", None);
    }
    let text = root.to_string_lossy();
    if text.starts_with("\\\\") || text.starts_with("//") {
        return ("network", None);
    }
    let mut seen = SEEN.lock().unwrap();
    let service = match &*seen {
        Some((path, service)) if path == root => *service,
        _ => {
            let service = sync::folders()
                .into_iter()
                .find(|f| root.starts_with(&f.path))
                .map(|f| f.service);
            *seen = Some((root.to_path_buf(), service));
            service
        }
    };
    match service {
        Some(s) => ("synced", Some(s)),
        None if synced(root) => ("synced", None),
        None => ("local", None),
    }
}

/// Conflict copies of a sync service beside their original (LIB-029): copy ID →
/// original ID.
fn conflicts_of(all: &[Item]) -> std::collections::HashMap<String, String> {
    let split = |path: &str| {
        let (dir, file) = path.rsplit_once('/').unwrap_or(("", path));
        let (stem, ext) = file.rsplit_once('.').unwrap_or((file, ""));
        (dir.to_string(), stem.to_string(), ext.to_lowercase())
    };
    let by_name: std::collections::HashMap<(String, String, String), &str> = all
        .iter()
        .map(|i| {
            let (d, s, e) = split(&i.path);
            ((d, s.to_lowercase(), e), i.id.as_str())
        })
        .collect();
    let mut out = std::collections::HashMap::new();
    for i in all {
        let (dir, stem, ext) = split(&i.path);
        for base in conflict::original_stems(&stem) {
            if let Some(orig) = by_name.get(&(dir.clone(), base.to_lowercase(), ext.clone())) {
                if *orig != i.id {
                    out.insert(i.id.clone(), (*orig).to_string());
                    break;
                }
            }
        }
    }
    out
}

fn with_conflict(mut v: Value, conflicts: &std::collections::HashMap<String, String>, id: &str) -> Value {
    if let Some(orig) = conflicts.get(id) {
        v["conflict"] = json!(true);
        v["conflictOf"] = json!(orig);
    }
    v
}

/// `library.resolve` (LIB-029): a conflict copy and its original become one: keep the
/// copy (the original goes to the trash and the copy takes its name), keep the
/// original (the copy goes to the trash) or keep both (the copy gets a name of its own).
fn resolve(core: &Core, ctx: &Context, p: &Value) -> Result<Value, CommandError> {
    let lang = ctx.lang;
    let lib = library(core);
    let id = p
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let all = lib.list();
    let orig_id = conflicts_of(&all)
        .remove(&id)
        .ok_or_else(|| not_found(lang, &id))?;
    let orig = lib.get(&orig_id).ok_or_else(|| not_found(lang, &orig_id))?;
    let trash = |ids: &[String]| {
        lib.delete(ids, false)
            .map(|(undo, _)| undo)
            .map_err(|e| CommandError::internal(e.to_string()))
    };
    let free_name = |base: &str| -> Result<Item, CommandError> {
        for n in 2..100 {
            match lib.rename(&id, &format!("{base} ({n})")) {
                Ok(item) => return Ok(item),
                Err(service::RenameError::Exists(_)) => continue,
                Err(e) => return Err(CommandError::internal(e.to_string())),
            }
        }
        Err(CommandError::internal("no free name"))
    };
    let (kept, undo) = match p.get("keep").and_then(Value::as_str).unwrap_or("both") {
        "original" => (orig_id.clone(), trash(std::slice::from_ref(&id))?),
        "this" => {
            let undo = trash(std::slice::from_ref(&orig_id))?;
            let item = lib
                .rename(&id, &orig.name)
                .map_err(|e| CommandError::internal(e.to_string()))?;
            (item.id, undo)
        }
        "both" => (free_name(&orig.name)?.id, None),
        other => {
            return Err(CommandError::new(
                ErrorCode::InvalidParams,
                tr(
                    lang,
                    "errors.invalid_param",
                    &[("param", "keep"), ("reason", other)],
                ),
            ))
        }
    };
    changed(core);
    Ok(json!({ "kept": kept, "undo": undo }))
}

/// What Details shows of a file besides the index (LIB-012): its capture metadata
/// (CAP-056) and the file's own dates. A file only in the cloud is not read.
fn details(file: &Path, cloud: bool) -> Value {
    let fs = std::fs::metadata(file).ok();
    let time =
        |t: Option<std::time::SystemTime>| t.map(|t| chrono::DateTime::<chrono::Local>::from(t).to_rfc3339());
    let created = time(fs.as_ref().and_then(|m| m.created().ok()));
    let modified = time(fs.as_ref().and_then(|m| m.modified().ok()));
    let meta = if cloud || file.extension().is_none_or(|e| e != "xshot") {
        Value::Null
    } else {
        xs_core::document::Xshot::read_summary(file)
            .map(|s| s.meta)
            .unwrap_or(Value::Null)
    };
    let app = &meta["app"];
    json!({
        "created": created,
        "modified": modified,
        "capturedAt": meta["capturedAt"],
        "app": app["name"].as_str().or(app.as_str()),
        "appVersion": app["version"],
        "url": meta["url"],
        "os": meta["os"]["name"].as_str().map(|n| match meta["os"]["version"].as_str() {
            Some(v) => format!("{n} {v}"),
            None => n.to_string(),
        }),
        "user": meta["user"],
        "scale": meta["display"]["scale"],
        "monitor": meta["display"]["monitor"],
    })
}

/// `library.import` (LIB-024): files and folders copied into the library, each in the
/// folder of its month; a folder's name becomes the tag of what came from it.
fn import_command(core: &Core, ctx: &Context, p: &Value) -> Result<Value, CommandError> {
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
        return Err(CommandError::new(
            ErrorCode::InvalidParams,
            tr(
                lang,
                "errors.invalid_param",
                &[("param", "paths"), ("reason", "give files or folders")],
            ),
        ));
    }
    let tag_folders = p.get("tagFolders").and_then(Value::as_bool).unwrap_or(true);
    let lib = library(core);
    let report = import::import(&lib, &paths, tag_folders);
    lib.scan();
    let mut by_tag: std::collections::BTreeMap<&str, Vec<String>> = std::collections::BTreeMap::new();
    for (path, tag) in &report.tags {
        if let Some(item) = lib.find(path) {
            by_tag.entry(tag.as_str()).or_default().push(item.id);
        }
    }
    for (tag, ids) in by_tag {
        // A name too long for a tag is no reason to fail the import.
        let _ = lib.tag(&ids, &[tag.to_string()], &[]);
    }
    let imported: Vec<Value> = report
        .imported
        .iter()
        .map(|(source, path)| {
            json!({
                "source": source,
                "path": path,
                "id": lib.find(path).map(|i| i.id),
            })
        })
        .collect();
    let skipped: Vec<Value> = report
        .skipped
        .iter()
        .map(|(source, reason)| json!({ "source": source, "reason": reason }))
        .collect();
    changed(core);
    Ok(json!({ "imported": imported, "skipped": skipped }))
}

/// `library.search` (LIB-034): `library.list` with a parameter for each field (LIB-005,
/// LIB-007): text, tags, type, app, site, a range of days and favorites.
fn search(core: &Core, ctx: &Context, p: &Value) -> Result<Value, CommandError> {
    let words = |key: &str| strings(p, key);
    let mut query: Vec<String> = p
        .get("query")
        .and_then(Value::as_str)
        .map(|q| vec![q.to_string()])
        .unwrap_or_default();
    // An app or a site is matched by part of its name, as in the search field.
    for (key, prefix) in [("app", "app:"), ("site", "site:")] {
        for v in words(key) {
            query.push(format!("{prefix}\"{v}\""));
        }
    }
    let mut tags = words("tag");
    tags.extend(words("tags"));
    let mut types = words("type");
    types.extend(words("types"));
    let filters = json!({
        "tags": tags,
        "types": types,
        "favorites": p.get("favorite").and_then(Value::as_bool).unwrap_or(false),
        "from": p.get("from"),
        "to": p.get("to"),
    });
    let mut params = json!({ "query": query.join(" "), "filters": filters });
    for key in ["sort", "order", "offset", "limit"] {
        if let Some(v) = p.get(key) {
            params[key] = v.clone();
        }
    }
    let mut r = list(core, ctx, &params)?;
    if let Some(o) = r.as_object_mut() {
        o.remove("facets");
    }
    Ok(r)
}

fn mark_error(lang: Lang, e: service::MarkError) -> CommandError {
    use service::MarkError;
    match e {
        MarkError::NotFound(id) => not_found(lang, &id),
        MarkError::InvalidTag(t) => CommandError::new(
            ErrorCode::InvalidParams,
            tr(lang, "errors.invalid_tag", &[("tag", &t)]),
        ),
        MarkError::TooMany(name) => CommandError::new(
            ErrorCode::InvalidParams,
            tr(
                lang,
                "errors.too_many_tags",
                &[("name", &name), ("max", &service::MAX_TAGS.to_string())],
            ),
        ),
        MarkError::Exists(tag) => CommandError::new(
            ErrorCode::InvalidParams,
            tr(lang, "errors.tag_exists", &[("tag", &tag)]),
        )
        .with_details(json!({ "reason": "exists", "tag": tag })),
    }
}

fn strings(p: &Value, key: &str) -> Vec<String> {
    match p.get(key) {
        Some(Value::String(s)) => vec![s.clone()],
        Some(Value::Array(a)) => a.iter().filter_map(|v| v.as_str().map(str::to_string)).collect(),
        _ => Vec::new(),
    }
}

/// `library.tag` (LIB-009): adds and removes tags on items; their files and dates
/// stay as they are.
fn tag(core: &Core, ctx: &Context, p: &Value) -> Result<Value, CommandError> {
    let lib = library(core);
    let items = lib
        .tag(&ids_of(p), &strings(p, "add"), &strings(p, "remove"))
        .map_err(|e| mark_error(ctx.lang, e))?;
    changed(core);
    Ok(json!({ "items": items.iter().map(|i| i.to_json(lib.root())).collect::<Vec<_>>() }))
}

/// `library.tags` (LIB-010): the tags with their counts, and renaming, merging and
/// deleting a tag on all its items.
fn tags(core: &Core, ctx: &Context, p: &Value) -> Result<Value, CommandError> {
    let lang = ctx.lang;
    let lib = library(core);
    let tag = || {
        p.get("tag").and_then(Value::as_str).ok_or_else(|| {
            CommandError::new(
                ErrorCode::InvalidParams,
                tr(
                    lang,
                    "errors.invalid_param",
                    &[("param", "tag"), ("reason", "give a tag")],
                ),
            )
        })
    };
    let changed_items = match p.get("action").and_then(Value::as_str).unwrap_or("list") {
        "list" => None,
        "rename" => {
            let to = p.get("to").and_then(Value::as_str).unwrap_or_default();
            let merge = p.get("merge").and_then(Value::as_bool).unwrap_or(false);
            Some(
                lib.rename_tag(tag()?, to, merge)
                    .map_err(|e| mark_error(lang, e))?,
            )
        }
        "merge" => {
            let into = p.get("into").and_then(Value::as_str).unwrap_or_default();
            Some(
                lib.rename_tag(tag()?, into, true)
                    .map_err(|e| mark_error(lang, e))?,
            )
        }
        "delete" => Some(lib.delete_tag(tag()?)),
        other => {
            return Err(CommandError::new(
                ErrorCode::InvalidParams,
                tr(
                    lang,
                    "errors.invalid_param",
                    &[("param", "action"), ("reason", other)],
                ),
            ))
        }
    };
    if changed_items.is_some() {
        changed(core);
    }
    let mut list = lib.tags();
    if p.get("sort").and_then(Value::as_str) == Some("count") {
        list.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| query::natural(&a.0, &b.0)));
    } else {
        list.sort_by(|a, b| query::natural(&a.0, &b.0));
    }
    Ok(json!({
        "tags": list.iter().map(|(t, n)| json!({ "tag": t, "count": n })).collect::<Vec<_>>(),
        "changed": changed_items,
    }))
}

fn same_path(a: &Path, b: &Path) -> bool {
    let norm = |p: &Path| {
        let s = p.to_string_lossy().replace('/', "\\");
        if cfg!(windows) {
            s.to_lowercase()
        } else {
            s
        }
    };
    norm(a) == norm(b)
}

/// Closes in the editor the documents of files about to be deleted: their unsaved
/// changes go with them (LIB-015). Returns the files that were open.
fn close_open(core: &Core, source: Source, files: &[PathBuf]) -> Vec<PathBuf> {
    let Ok(list) = core.call(source, "document.list", json!({}), None) else {
        return Vec::new();
    };
    let mut ids = Vec::new();
    let mut paths = Vec::new();
    for doc in list["documents"].as_array().into_iter().flatten() {
        let (Some(id), Some(path)) = (doc["documentId"].as_str(), doc["path"].as_str()) else {
            continue;
        };
        if files.iter().any(|f| same_path(f, Path::new(path))) {
            ids.push(id.to_string());
            if doc["shown"] == true {
                paths.push(PathBuf::from(path));
            }
        }
    }
    if !ids.is_empty() {
        let _ = core.call(
            source,
            "document.close",
            json!({ "documentIds": ids, "discard": ids }),
            None,
        );
    }
    paths
}

fn delete(core: &Core, ctx: &Context, p: &Value) -> Result<Value, CommandError> {
    let lib = library(core);
    let mut ids = ids_of(p);
    for id in &ids {
        if lib.get(id).is_none() {
            return Err(not_found(ctx.lang, id));
        }
    }
    // Files by path: the library's become their items; others are files opened from
    // elsewhere (TRY-009).
    let mut others = Vec::new();
    for path in p["paths"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
    {
        let path = PathBuf::from(path);
        match lib.find(&path) {
            Some(item) if !ids.contains(&item.id) => ids.push(item.id),
            Some(_) => {}
            None if path.is_file() && scan::format_of(&path).is_some() => others.push(path),
            None => return Err(not_found(ctx.lang, &path.to_string_lossy())),
        }
    }
    if ids.is_empty() && others.is_empty() {
        return Err(CommandError::new(
            ErrorCode::InvalidParams,
            tr(
                ctx.lang,
                "errors.invalid_param",
                &[("param", "ids"), ("reason", "empty")],
            ),
        ));
    }
    let mut files: Vec<PathBuf> = ids
        .iter()
        .filter_map(|id| lib.get(id))
        .map(|i| lib.abs(&i.path))
        .collect();
    files.extend(others.iter().cloned());
    let permanent = p["permanent"].as_bool().unwrap_or(false);
    let open = close_open(core, ctx.source, &files);
    match lib.delete_with(&ids, &others, permanent) {
        Ok((undo, items)) => {
            if let Some(undo) = &undo {
                lib.remember_open(undo, open);
            }
            tracing::info!(count = items.len(), permanent, "library items deleted");
            changed(core);
            Ok(json!({ "deleted": items.len() + others.len(), "undo": undo }))
        }
        Err(e) => {
            // Back in the editor what was closed for nothing.
            for path in open {
                let _ = core.call(
                    ctx.source,
                    "document.open",
                    json!({ "path": path, "show": true }),
                    None,
                );
            }
            let name = files
                .first()
                .and_then(|f| f.file_name())
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            Err(match e {
                recycle::TrashError::NoTrash(reason) => CommandError::new(
                    ErrorCode::NotSupportedOnPlatform,
                    tr(ctx.lang, "errors.library_no_trash", &[("name", &name)]),
                )
                .with_details(json!({ "reason": "no_trash", "detail": reason })),
                recycle::TrashError::Io(io) => CommandError::internal(io.to_string()),
            })
        }
    }
}

fn restore(core: &Core, ctx: &Context, p: &Value) -> Result<Value, CommandError> {
    let lib = library(core);
    let undo = p["undo"].as_str().unwrap_or_default();
    let back = lib.restore(undo).map_err(|_| {
        CommandError::new(
            ErrorCode::NotFound,
            tr(ctx.lang, "errors.library_undo_expired", &[]),
        )
    })?;
    for path in lib.take_open(undo) {
        let _ = core.call(
            ctx.source,
            "document.open",
            json!({ "path": path, "show": true }),
            None,
        );
    }
    changed(core);
    Ok(json!({ "restored": back.len() }))
}

/// The open document of `file`, if any.
fn open_document(core: &Core, source: Source, file: &Path) -> Option<String> {
    let list = core.call(source, "document.list", json!({}), None).ok()?;
    list["documents"]
        .as_array()?
        .iter()
        .find(|d| d["path"].as_str().is_some_and(|p| same_path(file, Path::new(p))))
        .and_then(|d| d["documentId"].as_str().map(str::to_string))
}

fn rename(core: &Core, ctx: &Context, p: &Value) -> Result<Value, CommandError> {
    let lib = library(core);
    let id = p["id"].as_str().unwrap_or_default();
    let name = p["name"].as_str().unwrap_or_default();
    // An item open in the editor is renamed by the editor, so the document follows
    // its file (TRY-006).
    if let Some(item) = lib.get(id) {
        if let Some(doc) = open_document(core, ctx.source, &lib.abs(&item.path)) {
            let r = core.call(
                ctx.source,
                "document.rename",
                json!({ "documentId": doc, "name": name }),
                Some(ctx.lang),
            )?;
            lib.scan();
            let path = PathBuf::from(r["path"].as_str().unwrap_or_default());
            return lib
                .find(&path)
                .map(|i| i.to_json(lib.root()))
                .ok_or_else(|| not_found(ctx.lang, id));
        }
    }
    let item = lib.rename(id, name).map_err(|e| match e {
        service::RenameError::NotFound => not_found(ctx.lang, id),
        service::RenameError::Invalid => CommandError::new(
            ErrorCode::InvalidParams,
            tr(
                ctx.lang,
                "errors.invalid_param",
                &[("param", "name"), ("reason", "empty")],
            ),
        ),
        service::RenameError::Exists(n) => CommandError::new(
            ErrorCode::InvalidParams,
            tr(ctx.lang, "errors.file_exists", &[("name", &n)]),
        ),
        service::RenameError::Io(io) => CommandError::new(
            ErrorCode::Internal,
            tr(
                ctx.lang,
                "errors.rename_failed",
                &[("name", name), ("reason", &io.to_string())],
            ),
        ),
    })?;
    changed(core);
    Ok(item.to_json(lib.root()))
}

/// Registers the `library.*` commands at the default location.
pub fn extension() -> Extension {
    extension_with(None)
}

/// Registers the `library.*` commands, with the library at `root` (tests: deleted
/// files then wait in the app's data and never reach the real Recycle Bin).
pub fn extension_with(root: Option<PathBuf>) -> Extension {
    Box::new(move |reg, weak: Weak<Core>| {
        if let Some(core) = weak.upgrade() {
            core.service(|| match &root {
                Some(root) => Library::open(root.clone(), &core.paths.local_dir).holding_deletes(),
                None => Library::open(location_for(&core), &core.paths.local_dir),
            });
        }
        macro_rules! on {
            ($id:literal, |$core:ident, $ctx:ident, $p:ident| $body:expr) => {{
                let weak = weak.clone();
                reg.handle($id, move |$ctx: &Context, $p: Value| {
                    let $core = weak
                        .upgrade()
                        .ok_or_else(|| CommandError::internal("shutting down"))?;
                    #[allow(unused_variables)]
                    let $ctx = $ctx;
                    $body
                });
            }};
        }
        on!("library.list", |core, ctx, p| list(&core, ctx, &p));
        on!("library.search", |core, ctx, p| search(&core, ctx, &p));
        on!("library.resolve", |core, ctx, p| resolve(&core, ctx, &p));
        on!("library.import", |core, ctx, p| import_command(&core, ctx, &p));
        on!("library.merge", |core, ctx, p| merge_command(&core, ctx, &p));
        on!("library.reveal", |core, ctx, p| {
            let lib = library(&core);
            let item = match (
                p.get("id").and_then(Value::as_str),
                p.get("path").and_then(Value::as_str),
            ) {
                (Some(id), _) => lib.get(id).ok_or_else(|| not_found(ctx.lang, id))?,
                (None, Some(path)) => lib
                    .find(Path::new(path))
                    .ok_or_else(|| not_found(ctx.lang, path))?,
                _ => return Err(not_found(ctx.lang, "")),
            };
            core.host()
                .editor_event("library_reveal", json!({ "id": item.id }));
            let _ = core.host().show_main_window();
            Ok(item.to_json(lib.root()))
        });
        on!("library.tag", |core, ctx, p| tag(&core, ctx, &p));
        on!("library.location", |core, ctx, p| {
            match p.get("action").and_then(Value::as_str) {
                None | Some("get") | Some("inspect") => location_command(&core, ctx, &p),
                Some(other) => Err(CommandError::new(
                    ErrorCode::InvalidParams,
                    tr(
                        ctx.lang,
                        "errors.invalid_param",
                        &[("param", "action"), ("reason", other)],
                    ),
                )),
            }
        });
        on!("library.relocate", |core, ctx, p| {
            let mut p = p;
            p["relocate"] = json!(true);
            if p.get("cancel").and_then(Value::as_bool) == Some(true) {
                p["action"] = json!("cancel");
            }
            // LIB-028: Retry what a move left behind.
            if p.get("retry").is_some() {
                p["action"] = json!("retry");
            }
            location_command(&core, ctx, &p)
        });
        on!("library.rebuild", |core, ctx, _p| {
            let lib = library(&core);
            let items = lib.rebuild();
            changed(&core);
            Ok(json!({ "items": items }))
        });
        on!("library.check", |core, ctx, p| {
            let lib = library(&core);
            let cleaned = match p.get("action").and_then(Value::as_str) {
                Some("clean") => Some(lib.clean_orphans()),
                _ => None,
            };
            let report = lib.check();
            Ok(json!({
                "damaged": report.damaged,
                "orphans": report.orphans.len(),
                "writable": report.writable,
                "cleaned": cleaned,
            }))
        });
        on!("library.tags", |core, ctx, p| tags(&core, ctx, &p));
        on!("library.favorite", |core, ctx, p| {
            let lib = library(&core);
            let ids = ids_of(&p);
            let value = p.get("value").and_then(Value::as_bool);
            let set = lib
                .set_favorite(&ids, value)
                .map_err(|e| mark_error(ctx.lang, e))?;
            changed(&core);
            Ok(json!({ "favorite": set, "count": ids.len() }))
        });
        on!("library.get", |core, ctx, p| {
            let lib = library(&core);
            let id = p["id"].as_str().unwrap_or_default();
            let item = lib.get(id).ok_or_else(|| not_found(ctx.lang, id))?;
            let mut v = item.to_json(lib.root());
            if p.get("details").and_then(Value::as_bool) == Some(true) {
                v["details"] = details(&lib.abs(&item.path), item.cloud);
            }
            Ok(v)
        });
        on!("library.open_url", |core, ctx, p| {
            let lib = library(&core);
            let id = p["id"].as_str().unwrap_or_default();
            let item = lib.get(id).ok_or_else(|| not_found(ctx.lang, id))?;
            let url = item.url.ok_or_else(|| not_found(ctx.lang, id))?;
            service::open_url(&url).map_err(|e| {
                CommandError::new(
                    ErrorCode::InvalidParams,
                    tr(
                        ctx.lang,
                        "errors.invalid_param",
                        &[("param", "url"), ("reason", &e.to_string())],
                    ),
                )
            })?;
            Ok(json!({ "url": url }))
        });
        on!("library.delete", |core, ctx, p| delete(&core, ctx, &p));
        on!("library.restore", |core, ctx, p| restore(&core, ctx, &p));
        on!("library.rename", |core, ctx, p| rename(&core, ctx, &p));
        on!("library.show_in_folder", |core, ctx, p| {
            let lib = library(&core);
            let path = match (p["id"].as_str(), p["path"].as_str()) {
                (Some(id), _) => lib
                    .get(id)
                    .map(|i| lib.abs(&i.path))
                    .ok_or_else(|| not_found(ctx.lang, id))?,
                (None, Some(path)) => PathBuf::from(path),
                (None, None) => lib.root().to_path_buf(),
            };
            service::show_in_folder(&path).map_err(|e| CommandError::internal(e.to_string()))?;
            Ok(json!({}))
        });
    })
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod name_tests {
    use super::*;
    use chrono::TimeZone;

    fn at() -> chrono::DateTime<chrono::Local> {
        chrono::Local.with_ymd_and_hms(2026, 10, 12, 14, 3, 22).unwrap()
    }

    #[test]
    fn file_names_are_cleaned() {
        assert_eq!(file_name("  Sales: Q3/2026.  ").as_deref(), Some("Sales Q3 2026"));
        assert_eq!(file_name("a\tb\nc").as_deref(), Some("a b c"));
        assert_eq!(file_name(" ?* ").as_deref(), None);
        assert_eq!(file_name("con").as_deref(), Some("con_"));
        assert_eq!(file_name("COM1").as_deref(), Some("COM1_"));
        assert_eq!(file_name("Company").as_deref(), Some("Company"));
        assert_eq!(file_name(&"x".repeat(300)).unwrap().len(), 100);
    }

    #[test]
    fn names_by_app_and_time_in_a_month_folder() {
        let root = tempfile::tempdir().unwrap();
        let p = new_item_path(root.path(), Some("Chrome"), at(), "xshot");
        assert_eq!(
            p,
            root.path()
                .join("2026-10")
                .join("Chrome 2026-10-12 14.03.22.xshot")
        );
    }

    #[test]
    fn two_items_in_the_same_second_get_distinct_names() {
        let root = tempfile::tempdir().unwrap();
        let first = new_item_path(root.path(), Some("Excel"), at(), "xshot");
        std::fs::create_dir_all(first.parent().unwrap()).unwrap();
        std::fs::write(&first, b"x").unwrap();
        let second = new_item_path(root.path(), Some("Excel"), at(), "xshot");
        assert_ne!(first, second);
        assert!(second.ends_with("Excel 2026-10-12 14.03.22 (2).xshot"));
    }

    #[test]
    fn unsafe_app_names_are_cleaned() {
        assert_eq!(safe_name(Some("a/b:c*?.")), "abc");
        assert_eq!(safe_name(Some("  ")), "XcreenShot");
        assert_eq!(safe_name(None), "XcreenShot");
    }

    #[test]
    fn knows_what_is_inside_the_library() {
        let root = tempfile::tempdir().unwrap();
        let inside = root.path().join("2026-10").join("a.xshot");
        std::fs::create_dir_all(inside.parent().unwrap()).unwrap();
        assert!(contains(root.path(), &inside));
        let other = tempfile::tempdir().unwrap();
        assert!(!contains(root.path(), &other.path().join("b.png")));
    }
}
