//! `<file>.xmeta`: a small hidden JSON file next to each library item with what must
//! survive the index (LIB-022): its stable ID (LIB-034), and later its tags, favorite
//! and recognized text.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU32, Ordering};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Xmeta {
    pub id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub favorite: bool,
    /// Fields of a newer build, kept as they are.
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

pub fn path_of(file: &Path) -> PathBuf {
    let mut p = file.as_os_str().to_os_string();
    p.push(".xmeta");
    PathBuf::from(p)
}

pub fn read(file: &Path) -> Option<Xmeta> {
    let bytes = std::fs::read(path_of(file)).ok()?;
    serde_json::from_slice::<Xmeta>(&bytes)
        .ok()
        .filter(|m| !m.id.is_empty())
}

pub fn write(file: &Path, meta: &Xmeta) -> std::io::Result<()> {
    let bytes = serde_json::to_vec_pretty(meta).map_err(std::io::Error::other)?;
    let path = path_of(file);
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create(true).truncate(true);
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        // Hidden; Windows refuses to overwrite a hidden file opened without it.
        const FILE_ATTRIBUTE_HIDDEN: u32 = 0x2;
        options.attributes(FILE_ATTRIBUTE_HIDDEN);
    }
    use std::io::Write;
    options.open(path)?.write_all(&bytes)
}

/// Moves a sidecar along with its file (renamed or moved in the library).
pub fn follow(from: &Path, to: &Path) {
    let (old, new) = (path_of(from), path_of(to));
    if old.exists() {
        let _ = std::fs::rename(old, new);
    }
}

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// A new item ID: the time and a counter, 20 hex digits, never reused (LIB-034).
pub fn new_id() -> String {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or_default();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    let mix = (nanos as u64) ^ ((n as u64) << 48) ^ (std::process::id() as u64).rotate_left(20);
    format!(
        "{:012x}{:08x}",
        (nanos / 1_000_000) as u64 & 0xffff_ffff_ffff,
        (mix as u32) ^ n
    )
}
