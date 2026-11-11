//! Deleting library files to the system's trash, with Undo for a while (LIB-015).
//!
//! Windows: the files go to the Recycle Bin at once, and Undo restores them from it.
//! Elsewhere the trash cannot be restored from code, so the files wait in a folder of
//! the app's data during the Undo time and then go to the trash.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

/// How long Undo stays available (the notice shows it for 10 s).
pub const UNDO: Duration = Duration::from_secs(30);

#[derive(Debug, thiserror::Error)]
pub enum TrashError {
    /// The location has no trash (some network folders): deleting would be final.
    #[error("{0}")]
    NoTrash(String),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

/// Files sent to the trash together, for Undo.
#[derive(Debug, Clone)]
pub struct Batch {
    pub id: String,
    pub at: Instant,
    /// Seconds since 1970 when it happened (the Recycle Bin dates in seconds).
    pub at_unix: i64,
    /// Every file, with its `.xmeta`.
    pub files: Vec<PathBuf>,
    /// Where each file waits for Undo (not on Windows).
    pub held: Vec<(PathBuf, PathBuf)>,
}

fn now_unix() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or_default()
}

/// Sends `files` to the trash, or deletes them for good with `permanent`. With
/// `system_bin` (Windows) they go to the Recycle Bin at once; otherwise they wait in
/// `hold` during the Undo time.
pub fn delete(
    files: &[PathBuf],
    permanent: bool,
    system_bin: bool,
    hold: &Path,
    id: String,
) -> Result<Batch, TrashError> {
    let mut batch = Batch {
        id,
        at: Instant::now(),
        at_unix: now_unix(),
        files: files.to_vec(),
        held: Vec::new(),
    };
    if permanent {
        for f in files {
            std::fs::remove_file(f)?;
        }
        return Ok(batch);
    }
    if system_bin {
        trash::delete_all(files).map_err(|e| TrashError::NoTrash(e.to_string()))?;
    } else {
        let dir = hold.join(&batch.id);
        std::fs::create_dir_all(&dir)?;
        for (i, f) in files.iter().enumerate() {
            let name = f.file_name().map(|n| n.to_os_string()).unwrap_or_default();
            let to = dir.join(format!("{i}-{}", name.to_string_lossy()));
            move_file(f, &to)?;
            batch.held.push((f.clone(), to));
        }
    }
    Ok(batch)
}

/// Brings a batch back where it was. Returns the files restored.
pub fn restore(batch: &Batch) -> Result<Vec<PathBuf>, TrashError> {
    if !batch.held.is_empty() {
        let mut back = Vec::new();
        for (orig, held) in &batch.held {
            if held.exists() {
                move_file(held, orig)?;
                back.push(orig.clone());
            }
        }
        return Ok(back);
    }
    restore_from_bin(batch)
}

#[cfg(any(windows, all(unix, not(target_os = "macos"))))]
fn restore_from_bin(batch: &Batch) -> Result<Vec<PathBuf>, TrashError> {
    let listed = trash::os_limited::list().map_err(|e| TrashError::NoTrash(e.to_string()))?;
    let mut picked = Vec::new();
    let mut back = Vec::new();
    for file in &batch.files {
        let (Some(parent), Some(name)) = (file.parent(), file.file_name()) else {
            continue;
        };
        // The newest copy of that file deleted since the batch (a little slack for clocks).
        let best = listed
            .iter()
            .filter(|t| t.original_parent == parent && t.name == name && t.time_deleted >= batch.at_unix - 5)
            .max_by_key(|t| t.time_deleted);
        if let Some(item) = best {
            picked.push(item.clone());
            back.push(file.clone());
        }
    }
    if !picked.is_empty() {
        trash::os_limited::restore_all(picked).map_err(|e| TrashError::NoTrash(e.to_string()))?;
    }
    Ok(back)
}

#[cfg(not(any(windows, all(unix, not(target_os = "macos")))))]
fn restore_from_bin(_batch: &Batch) -> Result<Vec<PathBuf>, TrashError> {
    Ok(Vec::new())
}

/// The files of a batch whose Undo time is over go to the trash (not Windows).
pub fn finish(batch: &Batch) {
    let waiting: Vec<&PathBuf> = batch.held.iter().map(|(_, h)| h).filter(|h| h.exists()).collect();
    if !waiting.is_empty() {
        if let Err(e) = trash::delete_all(&waiting) {
            tracing::warn!(error = %e, "could not move held files to the trash");
        }
    }
    if let Some(dir) = batch.held.first().and_then(|(_, h)| h.parent()) {
        let _ = std::fs::remove_dir(dir);
    }
}

/// Files left waiting by a previous run (it ended during an Undo time).
pub fn finish_leftovers(hold: &Path) {
    let Ok(dirs) = std::fs::read_dir(hold) else { return };
    for dir in dirs.flatten() {
        let files: Vec<PathBuf> = std::fs::read_dir(dir.path())
            .map(|e| e.flatten().map(|f| f.path()).collect())
            .unwrap_or_default();
        if !files.is_empty() && trash::delete_all(&files).is_err() {
            continue;
        }
        let _ = std::fs::remove_dir(dir.path());
    }
}

/// Renames, or copies and deletes across volumes.
fn move_file(from: &Path, to: &Path) -> std::io::Result<()> {
    if let Some(dir) = to.parent() {
        std::fs::create_dir_all(dir)?;
    }
    if std::fs::rename(from, to).is_err() {
        std::fs::copy(from, to)?;
        std::fs::remove_file(from)?;
    }
    Ok(())
}
