//! Changing or moving the library (LIB-028): what a folder holds, and moving every item
//! with its `.xmeta` to another folder, with progress, and Cancel bringing back what
//! was moved.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::scan;
use crate::xmeta;

/// Whether a folder looks like a library, at a glance: an index, or a sidecar among
/// its first files and month folders. Nothing is read (a sync service's folder may hold
/// files only in the cloud, LIB-029).
pub fn looks_like_library(dir: &Path) -> bool {
    if dir.join("library.db").is_file() {
        return true;
    }
    let has_sidecar = |d: &Path| {
        std::fs::read_dir(d)
            .into_iter()
            .flatten()
            .flatten()
            .take(200)
            .any(|e| e.path().extension().is_some_and(|x| x == "xmeta"))
    };
    has_sidecar(dir)
        || std::fs::read_dir(dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter(|e| e.path().is_dir())
            .take(24)
            .any(|e| has_sidecar(&e.path()))
}

/// What a folder chosen as the new location holds.
#[derive(Debug, Clone, PartialEq)]
pub struct Inspect {
    pub exists: bool,
    /// Library items in it (captures, images).
    pub items: usize,
    /// It is a library of XcreenShot: items with their `.xmeta`, or an index.
    pub library: bool,
    pub writable: bool,
}

pub fn inspect(dir: &Path) -> Inspect {
    if !dir.is_dir() {
        return Inspect {
            exists: false,
            items: 0,
            library: false,
            writable: dir.parent().is_some_and(Path::is_dir),
        };
    }
    let found = scan::walk(dir);
    let with_meta = found.iter().any(|f| xmeta::read(&f.abs).is_some());
    let probe = dir.join(format!(".xs-write-test-{}", std::process::id()));
    let writable = std::fs::write(&probe, b"x").is_ok();
    let _ = std::fs::remove_file(&probe);
    Inspect {
        exists: true,
        items: found.len(),
        library: with_meta || dir.join("library.db").is_file(),
        writable,
    }
}

/// How a move ended.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Moved {
    pub moved: usize,
    /// Files left behind (in use, no permission), as they were.
    pub failed: Vec<PathBuf>,
    pub cancelled: bool,
}

/// Moves one file: a rename on the same disk, else a copy and a delete.
fn move_file(from: &Path, to: &Path) -> std::io::Result<()> {
    if let Some(parent) = to.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if to.exists() {
        return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "exists"));
    }
    if std::fs::rename(from, to).is_ok() {
        return Ok(());
    }
    std::fs::copy(from, to)?;
    if let Err(e) = std::fs::remove_file(from) {
        let _ = std::fs::remove_file(to);
        return Err(e);
    }
    Ok(())
}

/// Moves the files of `from` named in `files` (and their sidecars) to the same place
/// under `to`: Retry after a move that left them behind (LIB-028).
pub fn move_files(from: &Path, to: &Path, files: &[PathBuf]) -> Moved {
    let mut out = Moved::default();
    for file in files {
        let Ok(rel) = file.strip_prefix(from) else {
            out.failed.push(file.clone());
            continue;
        };
        let dst = to.join(rel);
        match move_file(file, &dst) {
            Ok(()) => {
                let side = xmeta::path_of(file);
                if side.exists() {
                    let _ = move_file(&side, &xmeta::path_of(&dst));
                }
                out.moved += 1;
            }
            Err(_) => out.failed.push(file.clone()),
        }
    }
    out
}

/// Moves every item of `from` (and its sidecar) to the same place under `to`. Items
/// that arrive meanwhile (a capture) move too. `progress` gets (done, total); when
/// `cancel` turns on, what was moved goes back and the move stops.
pub fn move_library(from: &Path, to: &Path, cancel: &AtomicBool, progress: impl Fn(usize, usize)) -> Moved {
    let mut done: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut failed: Vec<PathBuf> = Vec::new();
    loop {
        let pending: Vec<scan::Found> = scan::walk(from)
            .into_iter()
            .filter(|f| !failed.contains(&f.abs))
            .collect();
        if pending.is_empty() {
            break;
        }
        let total = done.len() + pending.len();
        for f in pending {
            if cancel.load(Ordering::Relaxed) {
                // Everything back where it was, newest first.
                for (src, dst) in done.iter().rev() {
                    let _ = move_file(dst, src);
                    let (side_dst, side_src) = (xmeta::path_of(dst), xmeta::path_of(src));
                    if side_dst.exists() {
                        let _ = move_file(&side_dst, &side_src);
                    }
                }
                return Moved {
                    moved: 0,
                    failed,
                    cancelled: true,
                };
            }
            let dst = to.join(f.rel.replace('/', std::path::MAIN_SEPARATOR_STR));
            match move_file(&f.abs, &dst) {
                Ok(()) => {
                    let side = xmeta::path_of(&f.abs);
                    if side.exists() {
                        let _ = move_file(&side, &xmeta::path_of(&dst));
                    }
                    done.push((f.abs.clone(), dst));
                }
                Err(e) => {
                    tracing::warn!(error = %e, "an item could not be moved");
                    failed.push(f.abs.clone());
                }
            }
            progress(done.len() + failed.len(), total);
        }
    }
    Moved {
        moved: done.len(),
        failed,
        cancelled: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn png(path: &Path) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        let img = image::RgbaImage::from_pixel(4, 4, image::Rgba([1, 2, 3, 255]));
        img.save(path).unwrap();
    }

    #[test]
    fn moves_items_with_sidecars_and_cancel_brings_them_back() {
        let (a, b) = (tempfile::tempdir().unwrap(), tempfile::tempdir().unwrap());
        for i in 0..5 {
            let f = a.path().join(format!("2026-11/c{i}.png"));
            png(&f);
            xmeta::write(
                &f,
                &xmeta::Xmeta {
                    id: format!("id{i}"),
                    tags: vec!["Cliente".into()],
                    ..xmeta::Xmeta::default()
                },
            )
            .unwrap();
        }
        let into = b.path().join("Lib");
        assert!(!inspect(&into).exists);
        let seen = std::sync::Mutex::new(Vec::new());
        let r = move_library(a.path(), &into, &AtomicBool::new(false), |d, t| {
            seen.lock().unwrap().push((d, t))
        });
        assert_eq!((r.moved, r.cancelled), (5, false));
        assert_eq!(seen.lock().unwrap().last(), Some(&(5, 5)));
        assert!(scan::walk(a.path()).is_empty());
        let moved = into.join("2026-11").join("c3.png");
        assert_eq!(xmeta::read(&moved).unwrap().tags, ["Cliente"]);
        let i = inspect(&into);
        assert_eq!((i.items, i.library, i.writable), (5, true, true));
        // Cancelled after two: all back.
        let back = b.path().join("Other");
        let stop = AtomicBool::new(false);
        let r = move_library(&into, &back, &stop, |d, _| {
            if d == 2 {
                stop.store(true, Ordering::Relaxed);
            }
        });
        assert!(r.cancelled);
        assert_eq!(scan::walk(&into).len(), 5);
        assert!(scan::walk(&back).is_empty());
    }
}
