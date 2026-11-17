//! Merging another library into this one (LIB-031): its items are **copied** here with
//! their `.xmeta` (tags, favorite, recognized text); the other folder is not touched.
//!
//! - A file with the same content as one already here is not copied; the item here
//!   gains the tags it lacks and its favorite.
//! - A name already taken by a different file gets " (2)".
//! - Tags join without regard to case: "cliente" joins "Cliente" (LIB-010).
//! - Cancel removes what was copied, and nothing else changes.

use std::collections::HashMap;
use std::hash::Hasher;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use crate::scan;
use crate::service::Library;
use crate::xmeta::{self, Xmeta};

/// How a merge ended.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Merged {
    /// Items copied into this library.
    pub added: usize,
    /// Items not copied because the same file is already here.
    pub duplicates: usize,
    /// Items here that gained tags or the favorite from a duplicate.
    pub enriched: usize,
    /// Copies that got another name: (its name in the other library, its new path).
    pub renamed: Vec<(String, PathBuf)>,
    /// Files that could not be copied, as they are in the other library.
    pub failed: Vec<PathBuf>,
    pub cancelled: bool,
}

/// A fingerprint of a file's content (not cryptographic: two files are the same when
/// their size and this match).
fn fingerprint(path: &Path) -> Option<u64> {
    let mut file = std::fs::File::open(path).ok()?;
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    let mut buf = vec![0u8; 256 * 1024];
    loop {
        let n = file.read(&mut buf).ok()?;
        if n == 0 {
            break;
        }
        hasher.write(&buf[..n]);
    }
    Some(hasher.finish())
}

/// A file of this library that a copy could duplicate.
struct Known {
    abs: PathBuf,
    /// Its library ID, or none for a copy of this merge (indexed afterwards).
    id: Option<String>,
    print: Option<Option<u64>>,
}

impl Known {
    fn print(&mut self) -> Option<u64> {
        if self.print.is_none() {
            self.print = Some(fingerprint(&self.abs));
        }
        self.print.flatten()
    }
}

/// Whether `a` and `b` are the same folder, or one is inside the other.
pub fn overlaps(a: &Path, b: &Path) -> bool {
    let canon = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let (a, b) = (canon(a), canon(b));
    a.starts_with(&b) || b.starts_with(&a)
}

/// Copies the items of `other` into `lib`. `progress` gets (done, total). Nothing in
/// `other` changes. The index is brought up to date at the end.
pub fn merge(lib: &Library, other: &Path, cancel: &AtomicBool, progress: impl Fn(usize, usize)) -> Merged {
    let mut out = Merged::default();
    let incoming = scan::walk(other);
    let total = incoming.len();

    // What is here, by size: only files on this computer (reading one that is only in
    // the cloud would download it, LIB-029).
    let mut by_size: HashMap<u64, Vec<Known>> = HashMap::new();
    let mut ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    for item in lib.list() {
        ids.insert(item.id.clone());
        if item.cloud {
            continue;
        }
        by_size.entry(item.size).or_default().push(Known {
            abs: lib.abs(&item.path),
            id: Some(item.id.clone()),
            print: None,
        });
    }
    // Tag spellings: the ones here win; a new tag keeps the first spelling it comes with.
    let mut spelling: HashMap<String, String> = lib
        .tags()
        .into_iter()
        .map(|(t, _)| (t.to_lowercase(), t))
        .collect();
    let mut unify = |tags: &[String]| -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for t in tags {
            let t = t.trim();
            if t.is_empty() {
                continue;
            }
            let s = spelling
                .entry(t.to_lowercase())
                .or_insert_with(|| t.to_string())
                .clone();
            if !out.iter().any(|x| x.to_lowercase() == s.to_lowercase()) {
                out.push(s);
            }
        }
        out
    };

    let mut copies: Vec<PathBuf> = Vec::new();
    // Existing item ID → (tags to add, favorite).
    let mut gains: HashMap<String, (Vec<String>, bool)> = HashMap::new();
    for (i, f) in incoming.iter().enumerate() {
        if cancel.load(Ordering::Relaxed) {
            for c in copies.iter().rev() {
                let _ = std::fs::remove_file(xmeta::path_of(c));
                let _ = std::fs::remove_file(c);
                // The folder it made, if nothing else is in it.
                if let Some(dir) = c.parent().filter(|d| *d != lib.root()) {
                    let _ = std::fs::remove_dir(dir);
                }
            }
            return Merged {
                cancelled: true,
                ..Merged::default()
            };
        }
        let meta = xmeta::read(&f.abs);
        let tags = unify(meta.as_ref().map(|m| m.tags.as_slice()).unwrap_or_default());
        let favorite = meta.as_ref().is_some_and(|m| m.favorite);

        // The same content already here?
        let mut twin: Option<Option<String>> = None;
        if let Some(candidates) = by_size.get_mut(&f.size) {
            if let Some(print) = fingerprint(&f.abs) {
                for k in candidates.iter_mut() {
                    if k.print() == Some(print) {
                        twin = Some(k.id.clone());
                        break;
                    }
                }
            }
        }
        if let Some(twin) = twin {
            out.duplicates += 1;
            if let Some(id) = twin {
                let gain = gains.entry(id).or_default();
                for t in tags {
                    if !gain.0.contains(&t) {
                        gain.0.push(t);
                    }
                }
                gain.1 |= favorite;
            }
            progress(i + 1, total);
            continue;
        }

        let rel = f.rel.replace('/', std::path::MAIN_SEPARATOR_STR);
        let wanted = lib.root().join(&rel);
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
        let copied = std::fs::create_dir_all(&dir).and_then(|_| {
            let target = crate::unique(&dir, &stem, &ext);
            let partial = xs_core::document::partial_path(&target);
            let done = std::fs::copy(&f.abs, &partial).and_then(|_| std::fs::rename(&partial, &target));
            if done.is_err() {
                let _ = std::fs::remove_file(&partial);
            }
            done.map(|_| target)
        });
        let target = match copied {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!(error = %e, "an item could not be merged");
                out.failed.push(f.abs.clone());
                progress(i + 1, total);
                continue;
            }
        };
        // Its ID comes along unless this library already uses it.
        let id = meta
            .as_ref()
            .map(|m| m.id.clone())
            .filter(|id| !ids.contains(id))
            .unwrap_or_else(xmeta::new_id);
        ids.insert(id.clone());
        let side = Xmeta {
            id,
            tags,
            favorite,
            extra: meta.map(|m| m.extra).unwrap_or_default(),
        };
        if let Err(e) = xmeta::write(&target, &side) {
            tracing::warn!(error = %e, "could not write a merged item's sidecar");
        }
        if target != wanted {
            out.renamed.push((f.rel.clone(), target.clone()));
        }
        by_size.entry(f.size).or_default().push(Known {
            abs: target.clone(),
            id: None,
            print: None,
        });
        copies.push(target);
        out.added += 1;
        progress(i + 1, total);
    }

    lib.scan();
    for (id, (tags, favorite)) in gains {
        let Some(item) = lib.get(&id) else { continue };
        let new: Vec<String> = tags
            .into_iter()
            .filter(|t| !item.tags.iter().any(|x| x.to_lowercase() == t.to_lowercase()))
            .collect();
        let mut gained = false;
        if !new.is_empty() && lib.tag(std::slice::from_ref(&id), &new, &[]).is_ok() {
            gained = true;
        }
        if favorite && !item.favorite && lib.set_favorite(std::slice::from_ref(&id), Some(true)).is_ok() {
            gained = true;
        }
        if gained {
            out.enriched += 1;
        }
    }
    out
}
