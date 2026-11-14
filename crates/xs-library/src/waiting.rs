//! Captures made while the library is out of reach (LIB-030): a network share
//! disconnected, a disk unplugged. They wait in a folder of the app's data, each in its
//! month's folder, and move into the library as soon as it is back.

use std::path::{Path, PathBuf};

use crate::service::Library;
use crate::{relocate, scan, xmeta};

/// Where captures wait while the library is out of reach.
pub fn waiting_dir(local_dir: &Path) -> PathBuf {
    local_dir.join("waiting")
}

static REMEMBERED: std::sync::OnceLock<PathBuf> = std::sync::OnceLock::new();

/// The app's waiting folder, so that [`contains`] knows it.
pub fn remember(local_dir: &Path) {
    let _ = REMEMBERED.set(waiting_dir(local_dir));
}

/// Whether `path` waits for the library: a capture of the library like the others, kept
/// and saved by itself (LIB-030).
pub fn contains(path: &Path) -> bool {
    REMEMBERED.get().is_some_and(|dir| crate::contains(dir, path))
}

/// Where a new item goes: the library, or the waiting folder while it is out of reach.
pub fn target_root(root: &Path, local_dir: &Path) -> PathBuf {
    if root.is_dir() {
        root.to_path_buf()
    } else {
        waiting_dir(local_dir)
    }
}

/// Whether something waits, without reading more than one entry per folder.
pub fn has_waiting(waiting: &Path) -> bool {
    !scan::walk(waiting).is_empty()
}

/// Moves what waits into the library, each into the same month's folder; a taken name
/// gets " (2)". Returns where each file went, (from, to). The index learns them.
pub fn move_waiting(lib: &Library, waiting: &Path) -> Vec<(PathBuf, PathBuf)> {
    let mut moved = Vec::new();
    if !lib.available() || !waiting.is_dir() {
        return moved;
    }
    for f in scan::walk(waiting) {
        let wanted = lib.root().join(f.rel.replace('/', std::path::MAIN_SEPARATOR_STR));
        let dir = wanted
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| lib.root().to_path_buf());
        let stem = wanted
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "image".to_string());
        let ext = wanted
            .extension()
            .map(|e| e.to_string_lossy().into_owned())
            .unwrap_or_default();
        if std::fs::create_dir_all(&dir).is_err() {
            continue;
        }
        let target = crate::unique(&dir, &stem, &ext);
        match relocate::move_file(&f.abs, &target) {
            Ok(()) => {
                let side = xmeta::path_of(&f.abs);
                if side.exists() {
                    let _ = relocate::move_file(&side, &xmeta::path_of(&target));
                }
                moved.push((f.abs.clone(), target));
            }
            Err(e) => tracing::warn!(error = %e, "a waiting capture could not reach the library"),
        }
    }
    // The emptied month folders go.
    if let Ok(entries) = std::fs::read_dir(waiting) {
        for e in entries.flatten() {
            if e.path().is_dir() {
                let _ = std::fs::remove_dir(e.path());
            }
        }
    }
    if !moved.is_empty() {
        lib.scan();
    }
    moved
}
