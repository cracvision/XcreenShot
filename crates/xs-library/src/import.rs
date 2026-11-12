//! Import to the library (LIB-024): files and folders are **copied** into the
//! library, each in the folder of the month of its date (its capture time, its EXIF
//! date, or the file's own); the originals do not change. A folder's name can become a
//! tag of what came from it.

use std::path::{Path, PathBuf};

use chrono::{Local, TimeZone};

use crate::scan::format_of;
use crate::service::Library;

/// A file the import found, and the tag of the folder it came from.
#[derive(Debug, Clone)]
struct Source {
    path: PathBuf,
    tag: Option<String>,
}

#[derive(Debug, Default)]
pub struct Report {
    /// Source → its copy in the library.
    pub imported: Vec<(PathBuf, PathBuf)>,
    /// Source → why not.
    pub skipped: Vec<(PathBuf, &'static str)>,
    /// The tag each copy got, by copy.
    pub tags: Vec<(PathBuf, String)>,
}

/// The images of `paths`: files as they are, folders with their subfolders.
fn sources(paths: &[PathBuf], tag_folders: bool, skipped: &mut Vec<(PathBuf, &'static str)>) -> Vec<Source> {
    let mut out = Vec::new();
    for path in paths {
        if path.is_dir() {
            let tag = tag_folders
                .then(|| path.file_name().map(|n| n.to_string_lossy().trim().to_string()))
                .flatten()
                .filter(|t| !t.is_empty());
            let mut dirs = vec![path.clone()];
            let mut found = Vec::new();
            while let Some(dir) = dirs.pop() {
                let Ok(entries) = std::fs::read_dir(&dir) else {
                    continue;
                };
                for e in entries.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        dirs.push(p);
                    } else if format_of(&p).is_some() {
                        found.push(p);
                    }
                }
            }
            found.sort();
            out.extend(found.into_iter().map(|p| Source {
                path: p,
                tag: tag.clone(),
            }));
        } else if format_of(path).is_some() {
            out.push(Source {
                path: path.clone(),
                tag: None,
            });
        } else {
            skipped.push((path.clone(), "unsupported"));
        }
    }
    out
}

/// When a file was taken: a capture's own time (CAP-056), the EXIF date of a picture,
/// else the file's modification time.
pub fn date_of(path: &Path) -> i64 {
    let file_time = || {
        std::fs::metadata(path)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as i64)
            .unwrap_or_else(|| Local::now().timestamp_millis())
    };
    if format_of(path) == Some("xshot") {
        return xs_core::document::Xshot::read_summary(path)
            .ok()
            .and_then(|s| s.meta["capturedAt"].as_str().map(str::to_string))
            .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok())
            .map(|t| t.timestamp_millis())
            .unwrap_or_else(file_time);
    }
    crate::exif::taken(path).unwrap_or_else(file_time)
}

/// Copies `paths` into the library. Files already in it are left alone.
pub fn import(lib: &Library, paths: &[PathBuf], tag_folders: bool) -> Report {
    let mut report = Report::default();
    for source in sources(paths, tag_folders, &mut report.skipped) {
        let file = &source.path;
        if crate::contains(lib.root(), file) {
            report.skipped.push((file.clone(), "in_library"));
            continue;
        }
        let when = Local
            .timestamp_millis_opt(date_of(file))
            .single()
            .unwrap_or_else(Local::now);
        let dir = lib.root().join(when.format("%Y-%m").to_string());
        let stem = file
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "image".to_string());
        let ext = file
            .extension()
            .map(|e| e.to_string_lossy().to_ascii_lowercase())
            .unwrap_or_else(|| "png".to_string());
        let copied = std::fs::create_dir_all(&dir).and_then(|_| {
            let target = crate::unique(&dir, &stem, &ext);
            let partial = xs_core::document::partial_path(&target);
            let done = std::fs::copy(file, &partial).and_then(|_| std::fs::rename(&partial, &target));
            if done.is_err() {
                let _ = std::fs::remove_file(&partial);
            }
            done.map(|_| target)
        });
        match copied {
            Ok(target) => {
                if let Some(tag) = &source.tag {
                    report.tags.push((target.clone(), tag.clone()));
                }
                report.imported.push((file.clone(), target));
            }
            Err(_) => report.skipped.push((file.clone(), "unreadable")),
        }
    }
    report
}
