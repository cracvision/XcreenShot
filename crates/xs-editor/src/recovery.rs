//! Recovery copies (EDT-012, APP-047): while a document has unsaved changes, a copy
//! is written every 30 s (only if it changed since the last one). After an
//! unexpected exit, the next start offers them in Recover work.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// How often unsaved documents are copied (EDT-012).
pub const INTERVAL_SECS: u64 = 30;
/// Items nobody recovers or discards are deleted after this long (APP-047).
const KEEP_DAYS: i64 = 30;

/// `<id>.json`, next to `<id>.xshot` in the recovery folder.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    pub name: String,
    /// The file the document belongs to, to save it there after recovering.
    pub original_path: Option<PathBuf>,
    /// `library` or `external`.
    pub kind: String,
    /// `xshot`, `png` or `jpg`.
    pub format: String,
    pub saved_at: chrono::DateTime<chrono::Utc>,
    /// A recording cut short (VID-022): its fragmented file, which takes the place of
    /// the document copy, its length (s), its first frame (a PNG data URL) and
    /// whether it opens when recovered (Open in, VID-052).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub video: Option<PathBuf>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open: Option<bool>,
}

pub fn dir(local_dir: &Path) -> PathBuf {
    local_dir.join("recovery")
}

pub fn document_path(dir: &Path, id: &str) -> PathBuf {
    dir.join(format!("{id}.xshot"))
}

fn entry_path(dir: &Path, id: &str) -> PathBuf {
    dir.join(format!("{id}.json"))
}

/// A new recovery id: time-based, so names sort by age and never collide in practice.
pub fn new_id() -> String {
    let now = chrono::Utc::now();
    format!("{}-{}", now.format("%Y%m%d%H%M%S%3f"), std::process::id())
}

pub fn write_entry(dir: &Path, entry: &Entry) -> std::io::Result<()> {
    let bytes = serde_json::to_vec_pretty(entry).map_err(std::io::Error::other)?;
    xs_core::paths::write_atomic(&entry_path(dir, &entry.id), &bytes)
}

pub fn remove(dir: &Path, id: &str) {
    // A recording cut short: its files go too (VID-022).
    let video = std::fs::read(entry_path(dir, id))
        .ok()
        .and_then(|b| serde_json::from_slice::<Entry>(&b).ok())
        .and_then(|e| e.video);
    if let Some(v) = video {
        let _ = std::fs::remove_file(xs_video::time_file(&v));
        let _ = std::fs::remove_file(v);
    }
    let _ = std::fs::remove_file(document_path(dir, id));
    let _ = std::fs::remove_file(entry_path(dir, id));
}

/// Every complete item, newest first. Items older than 30 days are deleted.
pub fn list(dir: &Path) -> Vec<Entry> {
    let Ok(read) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let cutoff = chrono::Utc::now() - chrono::Duration::days(KEEP_DAYS);
    let mut out = Vec::new();
    for e in read.flatten() {
        let path = e.path();
        if path.extension().is_none_or(|x| x != "json") {
            continue;
        }
        let Some(entry) = std::fs::read(&path)
            .ok()
            .and_then(|b| serde_json::from_slice::<Entry>(&b).ok())
        else {
            continue;
        };
        if entry.saved_at < cutoff {
            remove(dir, &entry.id);
        } else if document_path(dir, &entry.id).exists() || entry.video.as_ref().is_some_and(|v| v.exists()) {
            out.push(entry);
        }
    }
    out.sort_by_key(|e| std::cmp::Reverse(e.saved_at));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(id: &str, days_ago: i64) -> Entry {
        Entry {
            id: id.into(),
            name: "doc".into(),
            original_path: None,
            kind: "external".into(),
            format: "png".into(),
            saved_at: chrono::Utc::now() - chrono::Duration::days(days_ago),
            ..Default::default()
        }
    }

    #[test]
    fn lists_complete_items_and_drops_old_ones() {
        let dir = tempfile::tempdir().unwrap();
        for (id, age) in [("new", 1), ("old", 40), ("half", 0)] {
            write_entry(dir.path(), &entry(id, age)).unwrap();
            if id != "half" {
                std::fs::write(document_path(dir.path(), id), b"x").unwrap();
            }
        }
        let items = list(dir.path());
        assert_eq!(items.iter().map(|e| e.id.as_str()).collect::<Vec<_>>(), ["new"]);
        assert!(!document_path(dir.path(), "old").exists());
        remove(dir.path(), "new");
        assert!(list(dir.path()).is_empty());
    }
}
