//! Walking the library folder (LIB-022): which files are items, and what the index
//! keeps about each one.

use std::path::{Path, PathBuf};

use xs_core::document::Xshot;

/// Formats the editor opens (M1). Videos, GIF and the rest arrive with their features.
pub const FORMATS: [&str; 4] = ["xshot", "png", "jpg", "jpeg"];

/// A file found in the library folder.
#[derive(Debug, Clone, PartialEq)]
pub struct Found {
    /// Relative to the library folder, with `/` separators.
    pub rel: String,
    pub abs: PathBuf,
    pub size: u64,
    /// Milliseconds since 1970.
    pub modified: i64,
}

/// What the index keeps about a file besides its ID.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Info {
    pub name: String,
    pub format: String,
    pub captured: i64,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub app: Option<String>,
    pub url: Option<String>,
    /// Its annotations' text, one per line (LIB-007).
    pub text: String,
}

/// The format of an item file, or `None` for anything else (`.xmeta`, the index,
/// temporary and hidden files, other types).
pub fn format_of(path: &Path) -> Option<&'static str> {
    let name = path.file_name()?.to_string_lossy();
    if name.starts_with('.') || name.starts_with("~$") {
        return None;
    }
    let ext = path.extension()?.to_string_lossy().to_ascii_lowercase();
    FORMATS
        .iter()
        .find(|f| **f == ext)
        .map(|f| if *f == "jpeg" { "jpg" } else { f })
}

fn hidden(path: &Path, meta: &std::fs::Metadata) -> bool {
    let dot = path
        .file_name()
        .is_some_and(|n| n.to_string_lossy().starts_with('.'));
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const HIDDEN_OR_SYSTEM: u32 = 0x2 | 0x4;
        dot || meta.file_attributes() & HIDDEN_OR_SYSTEM != 0
    }
    #[cfg(not(windows))]
    {
        let _ = meta;
        dot
    }
}

pub fn millis(time: std::io::Result<std::time::SystemTime>) -> i64 {
    time.ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as i64)
        .unwrap_or_default()
}

pub fn relative(root: &Path, abs: &Path) -> Option<String> {
    let rel = abs.strip_prefix(root).ok()?;
    let parts: Vec<String> = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect();
    (!parts.is_empty()).then(|| parts.join("/"))
}

/// Every item file under `root`, in its subfolders too.
pub fn walk(root: &Path) -> Vec<Found> {
    let mut out = Vec::new();
    let mut dirs = vec![root.to_path_buf()];
    while let Some(dir) = dirs.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Ok(meta) = entry.metadata() else { continue };
            if hidden(&path, &meta) {
                continue;
            }
            if meta.is_dir() {
                dirs.push(path);
            } else if format_of(&path).is_some() {
                if let Some(rel) = relative(root, &path) {
                    out.push(Found {
                        rel,
                        size: meta.len(),
                        modified: millis(meta.modified()),
                        abs: path,
                    });
                }
            }
        }
    }
    out
}

/// Reads what the library shows about a file: name, size of the image, and for a
/// `.xshot` its capture metadata (CAP-056). A loose file's capture time is its own.
pub fn info(found: &Found) -> Info {
    let format = format_of(&found.abs).unwrap_or("png").to_string();
    let name = found
        .abs
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut info = Info {
        name,
        format: format.clone(),
        captured: found.modified,
        ..Info::default()
    };
    if format == "xshot" {
        if let Ok(summary) = Xshot::read_summary(&found.abs) {
            info.width = Some(summary.width);
            info.height = Some(summary.height);
            let text = |key: &str| {
                summary.meta[key]
                    .as_str()
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
            };
            // CAP-056 keeps the app as {name, version, id}; older files, as its name.
            info.app = summary.meta["app"]["name"]
                .as_str()
                .or_else(|| summary.meta["app"].as_str())
                .filter(|s| !s.trim().is_empty())
                .map(str::to_string);
            info.url = text("url");
            info.text = summary.texts.join(
                "
",
            );
            if let Some(t) = text("capturedAt").and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()) {
                info.captured = t.timestamp_millis();
            }
        }
    } else if let Ok((w, h)) = image::ImageReader::open(&found.abs)
        .and_then(|r| r.with_guessed_format())
        .map_err(|e| e.to_string())
        .and_then(|r| r.into_dimensions().map_err(|e| e.to_string()))
    {
        info.width = Some(w);
        info.height = Some(h);
    }
    info
}
