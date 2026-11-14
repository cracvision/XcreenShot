//! `library.export` (LIB-033): the whole library, or what the filters show, as plain
//! files in a folder, to leave XcreenShot without losing anything. `.xshot` images
//! become PNG (or JPG) with their annotations; PNG and JPG files are copied as they
//! are; the subfolders are kept; a CSV lists each item's name, dates, app, website and
//! tags. It goes on in the background with progress and `editor.cancel` (LIB-018).

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, Lang};
use xs_library::index::Item;
use xs_library::query;

use crate::store::DocStore;

fn bad(lang: Lang, param: &str, reason: &str) -> CommandError {
    crate::invalid(tr(
        lang,
        "errors.invalid_param",
        &[("param", param), ("reason", reason)],
    ))
}

/// The items to export: those of `ids`, or those the filters and the search show
/// (all of them when there are none), by name.
fn chosen<'a>(all: &'a [Item], p: &Value, lang: Lang) -> Result<Vec<&'a Item>, CommandError> {
    if let Some(ids) = p.get("ids").and_then(Value::as_array) {
        let ids: HashSet<&str> = ids.iter().filter_map(Value::as_str).collect();
        return Ok(all.iter().filter(|i| ids.contains(i.id.as_str())).collect());
    }
    let filters: query::Filters = match p.get("filters") {
        None | Some(Value::Null) => query::Filters::default(),
        Some(v) => serde_json::from_value(v.clone()).map_err(|e| bad(lang, "filters", &e.to_string()))?,
    };
    let search = p.get("query").and_then(Value::as_str).unwrap_or("");
    Ok(query::run(all, &filters, search, query::Sort::Name, false, query::today()).items)
}

/// Where an item goes: its place in the library, with the new extension; never over
/// a file that was already there or that this export wrote.
fn target_of(folder: &Path, rel: &str, ext: &str, taken: &mut HashSet<PathBuf>) -> PathBuf {
    let rel = Path::new(rel);
    let dir = folder.join(rel.parent().unwrap_or(Path::new("")));
    let stem = rel
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "image".to_string());
    let mut target = dir.join(format!("{stem}.{ext}"));
    if target.exists() || taken.contains(&target) {
        let mut n = 2;
        loop {
            let next = dir.join(format!("{stem} ({n}).{ext}"));
            if !next.exists() && !taken.contains(&next) {
                target = next;
                break;
            }
            n += 1;
        }
    }
    taken.insert(target.clone());
    target
}

/// `folder` is `root` or inside it, also when it does not exist yet.
fn inside(root: &Path, folder: &Path) -> bool {
    let root = std::fs::canonicalize(root).unwrap_or_else(|_| root.to_path_buf());
    let mut probe = folder.to_path_buf();
    let mut rest = Vec::new();
    while !probe.exists() {
        match (probe.file_name().map(|n| n.to_os_string()), probe.parent()) {
            (Some(name), Some(parent)) => {
                rest.push(name);
                probe = parent.to_path_buf();
            }
            _ => break,
        }
    }
    let mut full = std::fs::canonicalize(&probe).unwrap_or(probe);
    for name in rest.into_iter().rev() {
        full.push(name);
    }
    full.starts_with(&root)
}

/// A CSV field: quoted when it needs it (RFC 4180).
fn field(s: &str) -> String {
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

fn local_time(ms: i64) -> String {
    chrono::DateTime::from_timestamp_millis(ms)
        .map(|t| {
            t.with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string()
        })
        .unwrap_or_default()
}

/// The CSV of the export: one row per item, in the app's language, with a byte-order
/// mark so that spreadsheets read the accents.
pub fn metadata_csv(lang: Lang, rows: &[(&Item, String)]) -> String {
    let head = [
        "name", "file_col", "captured", "modified", "app", "website", "tags", "favorite",
    ]
    .map(|k| tr(lang, &format!("ui.library.export.csv_{k}"), &[]));
    let mut out = String::from("\u{feff}");
    out.push_str(&head.map(|h| field(&h)).join(","));
    out.push_str("\r\n");
    for (item, file) in rows {
        let cells = [
            item.name.clone(),
            file.clone(),
            local_time(item.captured),
            local_time(item.modified),
            item.app.clone().unwrap_or_default(),
            item.url.clone().unwrap_or_default(),
            item.tags.join("; "),
            if item.favorite { "1" } else { "0" }.to_string(),
        ];
        out.push_str(&cells.map(|c| field(&c)).join(","));
        out.push_str("\r\n");
    }
    out
}

pub fn export_library(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    use xs_image::export::{Kind, Options, Page};
    let lang = ctx.lang;
    let folder = p
        .get("folder")
        .and_then(Value::as_str)
        .filter(|f| !f.trim().is_empty())
        .map(PathBuf::from)
        .ok_or_else(|| bad(lang, "folder", "give the folder to export to"))?;
    let kind = match p.get("format").and_then(Value::as_str).unwrap_or("png") {
        "png" => Kind::Png,
        "jpg" | "jpeg" => Kind::Jpg,
        _ => return Err(bad(lang, "format", "png or jpg")),
    };
    let metadata = p.get("metadata").and_then(Value::as_bool).unwrap_or(true);
    let lib = xs_library::library(core);
    let all = lib.list();
    let items = chosen(&all, p, lang)?;
    // Not into the library itself: the copies would become items.
    if inside(lib.root(), &folder) {
        return Err(bad(lang, "folder", "a folder outside the library"));
    }
    std::fs::create_dir_all(&folder).map_err(|e| crate::internal(lang, "", e))?;

    let ed = crate::editor(core);
    let background = crate::flatten_background(core);
    let opts = Options::default();
    let stop = Arc::new(AtomicBool::new(false));
    *ed.filling.lock().unwrap() = Some(stop.clone());
    let started = Instant::now();
    let total = items.len();
    let (mut exported, mut copied, mut failed) = (0usize, 0usize, Vec::new());
    let mut rows: Vec<(&Item, String)> = Vec::new();
    let mut taken = HashSet::new();
    let mut cancelled = false;
    for (i, item) in items.iter().enumerate() {
        if stop.load(Ordering::Relaxed) {
            cancelled = true;
            break;
        }
        if started.elapsed() >= Duration::from_millis(300) {
            core.host().editor_event(
                "progress",
                json!({ "task": "library_export", "done": i as f32 / total.max(1) as f32, "index": i, "total": total }),
            );
        }
        let source = lib.abs(&item.path);
        let editable = item.format == "xshot";
        let ext = if editable {
            kind.ext()
        } else {
            item.format.as_str()
        };
        let target = target_of(&folder, &item.path, ext, &mut taken);
        let result = (|| -> Result<(), String> {
            let bytes = if editable {
                let doc = DocStore::load(&source, lib.root()).map_err(|e| format!("{e:?}"))?;
                let image = DocStore::render_doc(&doc)?;
                xs_image::export::encode(
                    &[Page {
                        image,
                        dpi: doc.document.dpi,
                    }],
                    kind,
                    &opts,
                    background,
                )?
            } else {
                std::fs::read(&source).map_err(|e| e.to_string())?
            };
            if let Some(dir) = target.parent() {
                std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            }
            crate::write_file(&target, &bytes).map_err(|e| e.to_string())
        })();
        match result {
            Ok(()) => {
                if editable {
                    exported += 1;
                } else {
                    copied += 1;
                }
                let rel = target
                    .strip_prefix(&folder)
                    .unwrap_or(&target)
                    .to_string_lossy()
                    .replace('\\', "/");
                rows.push((item, rel));
            }
            Err(reason) => failed.push(json!({"source": source, "name": item.name, "reason": reason})),
        }
    }
    *ed.filling.lock().unwrap() = None;
    let csv = if metadata && !rows.is_empty() {
        let name = tr(lang, "ui.library.export.csv_file", &[]);
        let path = folder.join(name);
        crate::write_file(&path, metadata_csv(lang, &rows).as_bytes())
            .map_err(|e| crate::internal(lang, "", e))?;
        Some(path)
    } else {
        None
    };
    if started.elapsed() >= Duration::from_millis(300) {
        core.host()
            .editor_event("progress", json!({ "task": "library_export", "done": null }));
    }
    Ok(json!({
        "folder": folder,
        "exported": exported,
        "copied": copied,
        "failed": failed,
        "cancelled": cancelled,
        "csv": csv,
    }))
}
