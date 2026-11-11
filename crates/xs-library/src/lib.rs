//! The library (LIB-): its location (LIB-027), where each capture is saved as `.xshot`
//! (LIB-020), the index kept in step with the folder (LIB-022), thumbnails, and the
//! `library.*` commands: list, get, delete with Undo, rename and show in the folder.

pub mod index;
pub mod recycle;
pub mod scan;
pub mod service;
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
    core.service(|| Library::open(location(), &core.paths.local_dir))
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
    let ids = ids_of(p);
    for id in &ids {
        if lib.get(id).is_none() {
            return Err(not_found(ctx.lang, id));
        }
    }
    let files: Vec<PathBuf> = ids
        .iter()
        .filter_map(|id| lib.get(id))
        .map(|i| lib.abs(&i.path))
        .collect();
    let permanent = p["permanent"].as_bool().unwrap_or(false);
    let open = close_open(core, ctx.source, &files);
    match lib.delete(&ids, permanent) {
        Ok((undo, items)) => {
            if let Some(undo) = &undo {
                lib.remember_open(undo, open);
            }
            tracing::info!(count = items.len(), permanent, "library items deleted");
            changed(core);
            Ok(json!({ "deleted": items.len(), "undo": undo }))
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

fn rename(core: &Core, ctx: &Context, p: &Value) -> Result<Value, CommandError> {
    let lib = library(core);
    let id = p["id"].as_str().unwrap_or_default();
    let name = p["name"].as_str().unwrap_or_default();
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
                None => Library::open(location(), &core.paths.local_dir),
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
        on!("library.list", |core, ctx, p| {
            let lib = library(&core);
            let all = lib.list();
            let offset = p["offset"].as_u64().unwrap_or(0) as usize;
            let limit = p["limit"].as_u64().map(|l| l as usize).unwrap_or(usize::MAX);
            let items: Vec<Value> = all
                .iter()
                .skip(offset)
                .take(limit)
                .map(|i| i.to_json(lib.root()))
                .collect();
            Ok(json!({
                "root": lib.root(),
                "synced": synced(lib.root()),
                "total": all.len(),
                "items": items,
            }))
        });
        on!("library.get", |core, ctx, p| {
            let lib = library(&core);
            let id = p["id"].as_str().unwrap_or_default();
            lib.get(id)
                .map(|i| i.to_json(lib.root()))
                .ok_or_else(|| not_found(ctx.lang, id))
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
