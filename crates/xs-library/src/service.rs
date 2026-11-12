//! The library service (LIB-): its folder, an index kept in step with the files
//! (LIB-022), thumbnails, renaming, and deleting with Undo (LIB-015).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify_debouncer_mini::notify::{RecommendedWatcher, RecursiveMode};
use notify_debouncer_mini::{new_debouncer, DebounceEventResult, Debouncer};
use xs_core::document::Xshot;

use crate::index::{Index, Item};
use crate::recycle::{self, Batch, TrashError};
use crate::scan::{self, Found};
use crate::xmeta::{self, Xmeta};

/// Longest side of a loose image's thumbnail, in pixels.
const THUMB: u32 = 320;
/// Changes outside XcreenShot settle for this long before the index follows them.
const SETTLE: Duration = Duration::from_millis(700);

/// Whether a sync service keeps the folder (OneDrive, Dropbox, Google Drive, iCloud):
/// then each computer keeps its own index in the app's data, because two computers
/// writing one database through a sync service can damage it (LIB-022, LIB-029).
pub fn synced(root: &Path) -> bool {
    let by_env = ["OneDrive", "OneDriveConsumer", "OneDriveCommercial"]
        .iter()
        .filter_map(std::env::var_os)
        .map(PathBuf::from)
        .any(|r| !r.as_os_str().is_empty() && root.starts_with(&r));
    by_env
        || root.ancestors().any(|a| {
            a.file_name().is_some_and(|n| {
                let n = n.to_string_lossy();
                n.starts_with("OneDrive")
                    || [
                        "Dropbox",
                        "Google Drive",
                        "My Drive",
                        "Mobile Documents",
                        "CloudStorage",
                    ]
                    .contains(&n.as_ref())
            })
        })
}

fn fnv(text: &str) -> u64 {
    text.bytes().fold(0xcbf29ce484222325u64, |h, b| {
        (h ^ b as u64).wrapping_mul(0x100000001b3)
    })
}

/// `library.db` in the library folder, or in the app's data for a synced folder.
pub fn index_path(root: &Path, local_dir: &Path) -> PathBuf {
    if synced(root) {
        let key = fnv(&root.to_string_lossy().to_lowercase());
        local_dir.join("library").join(format!("index-{key:016x}.db"))
    } else {
        root.join("library.db")
    }
}

/// The report of Check the library (LIB-023).
#[derive(Debug, Clone, Default)]
pub struct Check {
    pub damaged: Vec<PathBuf>,
    pub orphans: Vec<PathBuf>,
    pub writable: bool,
}

/// Most tags an item has (LIB-009).
pub const MAX_TAGS: usize = 100;

/// A tag as kept (LIB-009): trimmed, 1 to 50 characters.
pub fn clean_tag(tag: &str) -> Option<String> {
    let t = tag.trim();
    (!t.is_empty() && t.chars().count() <= 50).then(|| t.to_string())
}

#[derive(Debug, thiserror::Error)]
pub enum MarkError {
    #[error("not found: {0}")]
    NotFound(String),
    #[error("invalid tag")]
    InvalidTag(String),
    #[error("too many tags")]
    TooMany(String),
    /// Renaming onto a tag that exists: merge them?
    #[error("exists")]
    Exists(String),
}

#[derive(Debug, thiserror::Error)]
pub enum RenameError {
    #[error("not found")]
    NotFound,
    #[error("invalid name")]
    Invalid,
    #[error("exists")]
    Exists(String),
    #[error("{0}")]
    Io(#[from] std::io::Error),
}

pub struct Library {
    root: PathBuf,
    index: Mutex<Index>,
    thumbs: PathBuf,
    hold: PathBuf,
    /// The Recycle Bin restores from code on Windows; elsewhere, and in tests, files
    /// wait in `hold` during the Undo time.
    system_bin: bool,
    batches: Mutex<Vec<Batch>>,
    /// Open documents a delete closed, by batch: Undo opens them again (TRY-009).
    reopen: Mutex<HashMap<String, Vec<PathBuf>>>,
    scanning: Mutex<()>,
    watcher: Mutex<Option<Debouncer<RecommendedWatcher>>>,
}

impl Library {
    /// The library at `root`, with its index, thumbnails and Undo area under
    /// `local_dir` (the app's data). The folder is created if missing (LIB-027).
    pub fn open(root: PathBuf, local_dir: &Path) -> Library {
        let _ = std::fs::create_dir_all(&root);
        let index = Index::open(&index_path(&root, local_dir)).unwrap_or_else(|e| {
            tracing::error!(error = %e, "library index could not be opened; using a temporary one");
            Index::in_memory()
        });
        let hold = local_dir.join("deleted");
        recycle::finish_leftovers(&hold);
        Library {
            root,
            index: Mutex::new(index),
            thumbs: local_dir.join("thumbs"),
            hold,
            system_bin: cfg!(windows),
            batches: Mutex::default(),
            reopen: Mutex::default(),
            scanning: Mutex::default(),
            watcher: Mutex::default(),
        }
    }

    /// Deleted files wait in the app's data instead of going to the Recycle Bin at
    /// once (tests, so they never touch the real one).
    pub fn holding_deletes(mut self) -> Library {
        self.system_bin = false;
        self
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn abs(&self, rel: &str) -> PathBuf {
        self.root.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR))
    }

    /// Every item, newest capture first.
    pub fn list(&self) -> Vec<Item> {
        self.index.lock().unwrap().all().unwrap_or_default()
    }

    pub fn get(&self, id: &str) -> Option<Item> {
        self.index.lock().unwrap().get(id).ok().flatten()
    }

    /// Writes an item's tags and favorite to the index and its sidecar (LIB-022): its
    /// file and dates do not change (LIB-004, LIB-009, LIB-011).
    fn save_marks(&self, index: &Index, item: &Item) {
        let _ = index.set_marks(&item.id, &item.tags, item.favorite);
        let abs = self.abs(&item.path);
        let mut meta = xmeta::read(&abs).unwrap_or_else(|| Xmeta {
            id: item.id.clone(),
            ..Xmeta::default()
        });
        meta.tags = item.tags.clone();
        meta.favorite = item.favorite;
        if let Err(e) = xmeta::write(&abs, &meta) {
            tracing::warn!(error = %e, "could not write a library sidecar");
        }
    }

    /// Builds the index again from the files and their sidecars (LIB-023): IDs, tags
    /// and favorites come back from the `.xmeta`. Returns the number of items.
    pub fn rebuild(&self) -> usize {
        {
            let index = self.index.lock().unwrap();
            let _ = index.clear();
        }
        self.scan();
        self.list().len()
    }

    /// What Check the library finds (LIB-023): files that cannot be read, sidecars
    /// without their file, and whether the folder can be written.
    pub fn check(&self) -> Check {
        let mut damaged = Vec::new();
        for f in scan::walk(&self.root) {
            let ok = match scan::format_of(&f.abs) {
                Some("xshot") => Xshot::read_summary(&f.abs).is_ok(),
                _ => image::ImageReader::open(&f.abs)
                    .ok()
                    .and_then(|r| r.with_guessed_format().ok())
                    .and_then(|r| r.into_dimensions().ok())
                    .is_some(),
            };
            if !ok {
                damaged.push(f.abs);
            }
        }
        let mut orphans = Vec::new();
        let mut dirs = vec![self.root.clone()];
        while let Some(dir) = dirs.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else {
                continue;
            };
            for e in entries.flatten() {
                let path = e.path();
                if path.is_dir() {
                    dirs.push(path);
                } else if path.extension().is_some_and(|x| x == "xmeta") {
                    let file = path.with_extension("");
                    if !file.exists() {
                        orphans.push(path);
                    }
                }
            }
        }
        let probe = self.root.join(format!(".xs-write-test-{}", std::process::id()));
        let writable = std::fs::write(&probe, b"x").is_ok();
        let _ = std::fs::remove_file(&probe);
        Check {
            damaged,
            orphans,
            writable,
        }
    }

    /// Removes sidecars whose file is gone (LIB-023: Clean up).
    pub fn clean_orphans(&self) -> usize {
        let orphans = self.check().orphans;
        orphans.iter().filter(|p| std::fs::remove_file(p).is_ok()).count()
    }

    /// Every tag with its number of items, in the spelling of its first appearance
    /// (LIB-010).
    pub fn tags(&self) -> Vec<(String, usize)> {
        let all = self.list();
        let mut out: Vec<(String, usize)> = Vec::new();
        for t in all.iter().flat_map(|i| &i.tags) {
            match out.iter_mut().find(|(k, _)| k.to_lowercase() == t.to_lowercase()) {
                Some((_, n)) => *n += 1,
                None => out.push((t.clone(), 1)),
            }
        }
        out
    }

    /// Adds and removes tags on items (LIB-009). A tag keeps the spelling it already
    /// has in the library; case does not make a new one.
    pub fn tag(&self, ids: &[String], add: &[String], remove: &[String]) -> Result<Vec<Item>, MarkError> {
        let known = self.tags();
        let add: Vec<String> = add
            .iter()
            .map(|t| {
                let t = clean_tag(t).ok_or_else(|| MarkError::InvalidTag(t.clone()))?;
                Ok(known
                    .iter()
                    .find(|(k, _)| k.to_lowercase() == t.to_lowercase())
                    .map_or(t, |(k, _)| k.clone()))
            })
            .collect::<Result<_, MarkError>>()?;
        let remove: Vec<String> = remove.iter().map(|t| t.trim().to_lowercase()).collect();
        let index = self.index.lock().unwrap();
        let mut items = Vec::new();
        for id in ids {
            let mut item = index
                .get(id)
                .ok()
                .flatten()
                .ok_or_else(|| MarkError::NotFound(id.clone()))?;
            item.tags.retain(|t| !remove.contains(&t.to_lowercase()));
            for t in &add {
                if !item.tags.iter().any(|x| x.to_lowercase() == t.to_lowercase()) {
                    item.tags.push(t.clone());
                }
            }
            if item.tags.len() > MAX_TAGS {
                return Err(MarkError::TooMany(item.name.clone()));
            }
            items.push(item);
        }
        for item in &items {
            self.save_marks(&index, item);
        }
        Ok(items)
    }

    /// Renames a tag on all its items (LIB-010). Onto another tag that exists, only
    /// with `merge`; a change of case only is allowed. Returns the items changed.
    pub fn rename_tag(&self, from: &str, to: &str, merge: bool) -> Result<usize, MarkError> {
        let to = clean_tag(to).ok_or_else(|| MarkError::InvalidTag(to.to_string()))?;
        let (from_l, to_l) = (from.trim().to_lowercase(), to.to_lowercase());
        if from_l != to_l {
            if let Some((other, _)) = self.tags().into_iter().find(|(k, _)| k.to_lowercase() == to_l) {
                if !merge {
                    return Err(MarkError::Exists(other));
                }
            }
        }
        // The other tag's spelling stays when merging.
        let spelling = self
            .tags()
            .into_iter()
            .find(|(k, _)| k.to_lowercase() == to_l && from_l != to_l)
            .map_or(to, |(k, _)| k);
        let index = self.index.lock().unwrap();
        let mut changed = 0;
        for mut item in index.all().unwrap_or_default() {
            if !item.tags.iter().any(|t| t.to_lowercase() == from_l) {
                continue;
            }
            let mut tags: Vec<String> = Vec::new();
            for t in &item.tags {
                let t = if t.to_lowercase() == from_l || t.to_lowercase() == to_l {
                    spelling.clone()
                } else {
                    t.clone()
                };
                if !tags.iter().any(|x| x.to_lowercase() == t.to_lowercase()) {
                    tags.push(t);
                }
            }
            item.tags = tags;
            self.save_marks(&index, &item);
            changed += 1;
        }
        Ok(changed)
    }

    /// Removes a tag from all its items (LIB-010). Returns how many had it.
    pub fn delete_tag(&self, tag: &str) -> usize {
        let low = tag.trim().to_lowercase();
        let index = self.index.lock().unwrap();
        let mut changed = 0;
        for mut item in index.all().unwrap_or_default() {
            let before = item.tags.len();
            item.tags.retain(|t| t.to_lowercase() != low);
            if item.tags.len() != before {
                self.save_marks(&index, &item);
                changed += 1;
            }
        }
        changed
    }

    /// Marks or unmarks items as favorites (LIB-011). Without a value: all of them,
    /// unless all already are, then none. Returns the value set.
    pub fn set_favorite(&self, ids: &[String], value: Option<bool>) -> Result<bool, MarkError> {
        let index = self.index.lock().unwrap();
        let mut items = Vec::new();
        for id in ids {
            items.push(
                index
                    .get(id)
                    .ok()
                    .flatten()
                    .ok_or_else(|| MarkError::NotFound(id.clone()))?,
            );
        }
        let value = value.unwrap_or_else(|| !items.iter().all(|i| i.favorite));
        for mut item in items {
            if item.favorite != value {
                item.favorite = value;
                self.save_marks(&index, &item);
            }
        }
        Ok(value)
    }

    /// The item of a file, if it is in the library.
    pub fn find(&self, abs: &Path) -> Option<Item> {
        let rel = scan::relative(&self.root, abs)?;
        self.index.lock().unwrap().by_path(&rel).ok().flatten()
    }

    /// Brings the index in step with the folder (LIB-022): new, changed, renamed,
    /// moved and deleted files, whoever made the change. Returns whether anything
    /// changed. A file renamed outside XcreenShot keeps its ID and data: its sidecar
    /// follows it, or it is recognized by the same size and time.
    pub fn scan(&self) -> bool {
        let _one = self.scanning.lock().unwrap();
        let found = scan::walk(&self.root);
        let index = self.index.lock().unwrap();
        let known = index.all().unwrap_or_default();
        let on_disk: HashSet<&str> = found.iter().map(|f| f.rel.as_str()).collect();
        let by_path: HashMap<&str, &Item> = known.iter().map(|i| (i.path.as_str(), i)).collect();
        let mut gone: Vec<&Item> = known
            .iter()
            .filter(|i| !on_disk.contains(i.path.as_str()))
            .collect();
        let mut used: HashSet<String> = known
            .iter()
            .filter(|i| on_disk.contains(i.path.as_str()))
            .map(|i| i.id.clone())
            .collect();
        let mut changed = false;
        for f in &found {
            if let Some(item) = by_path.get(f.rel.as_str()) {
                if f.cloud && (item.size != f.size || item.modified != f.modified || !item.cloud) {
                    // Gone to the cloud: what the index knows stays; nothing is read.
                    let _ = index.upsert(&Item {
                        size: f.size,
                        modified: f.modified,
                        cloud: true,
                        ..(*item).clone()
                    });
                    changed = true;
                } else if !f.cloud && (item.size != f.size || item.modified != f.modified || item.cloud) {
                    // Saved again in place, or downloaded: read it again, same ID.
                    let _ = index.upsert(&build(f, item.id.clone(), item.favorite, item.tags.clone()));
                    changed = true;
                }
                continue;
            }
            let side = xmeta::read(&f.abs);
            let mut kept: Option<Item> = None;
            if let Some(m) = &side {
                if let Some(pos) = gone.iter().position(|g| g.id == m.id) {
                    kept = Some(gone.remove(pos).clone());
                }
            }
            if kept.is_none() {
                if let Some(pos) = gone
                    .iter()
                    .position(|g| g.size == f.size && g.modified == f.modified)
                {
                    let old = gone.remove(pos);
                    xmeta::follow(&self.abs(&old.path), &f.abs);
                    kept = Some(old.clone());
                }
            }
            let item = match kept {
                Some(old) => build(f, old.id, old.favorite, old.tags),
                None => match side.filter(|m| !used.contains(&m.id)) {
                    // Brought in with its sidecar from elsewhere: its ID and data come too.
                    Some(m) => build(f, m.id, m.favorite, m.tags),
                    None => build(f, xmeta::new_id(), false, Vec::new()),
                },
            };
            let current = xmeta::read(&f.abs);
            if current.as_ref().map(|m| m.id.as_str()) != Some(item.id.as_str()) {
                let meta = Xmeta {
                    id: item.id.clone(),
                    tags: item.tags.clone(),
                    favorite: item.favorite,
                    ..Xmeta::default()
                };
                if let Err(e) = xmeta::write(&f.abs, &meta) {
                    tracing::warn!(error = %e, "could not write a library sidecar");
                }
            }
            used.insert(item.id.clone());
            let _ = index.upsert(&item);
            changed = true;
        }
        for g in gone {
            let _ = index.remove(&g.id);
            changed = true;
        }
        changed
    }

    /// A PNG thumbnail of an item, from the cache or made now: the `.xshot` carries its
    /// own; a loose image is scaled down.
    pub fn thumbnail(&self, id: &str) -> Option<Vec<u8>> {
        let item = self.get(id)?;
        let cache = self.thumbs.join(format!("{id}-{}.png", item.modified));
        if let Ok(bytes) = std::fs::read(&cache) {
            return Some(bytes);
        }
        // Only in the cloud: the last thumbnail this computer saw, else none; reading
        // the file would download it (LIB-029).
        if item.cloud {
            return std::fs::read_dir(&self.thumbs)
                .ok()?
                .flatten()
                .find(|e| e.file_name().to_string_lossy().starts_with(&format!("{id}-")))
                .and_then(|e| std::fs::read(e.path()).ok());
        }
        let abs = self.abs(&item.path);
        let png = if item.format == "xshot" {
            Some(Xshot::read_summary(&abs).ok()?.thumb_png).filter(|b| !b.is_empty())?
        } else {
            let image = image::open(&abs).ok()?.to_rgba8();
            xs_image::encode::png(&xs_image::encode::thumbnail(&image, THUMB), 0.0, true).ok()?
        };
        let _ = std::fs::create_dir_all(&self.thumbs);
        let prefix = format!("{id}-");
        if let Ok(entries) = std::fs::read_dir(&self.thumbs) {
            for e in entries.flatten() {
                if e.file_name().to_string_lossy().starts_with(&prefix) {
                    let _ = std::fs::remove_file(e.path());
                }
            }
        }
        let _ = std::fs::write(&cache, &png);
        Some(png)
    }

    /// Sends items to the trash (or deletes them for good with `permanent`). Returns
    /// the Undo ID, if there is Undo, and the items removed.
    pub fn delete(&self, ids: &[String], permanent: bool) -> Result<(Option<String>, Vec<Item>), TrashError> {
        self.delete_with(ids, &[], permanent)
    }

    /// [`Library::delete`] of items and of `others`, files outside the library (a file
    /// opened from elsewhere and deleted from the recent tray, TRY-009).
    pub fn delete_with(
        &self,
        ids: &[String],
        others: &[PathBuf],
        permanent: bool,
    ) -> Result<(Option<String>, Vec<Item>), TrashError> {
        let items: Vec<Item> = ids.iter().filter_map(|id| self.get(id)).collect();
        let mut files: Vec<PathBuf> = others.to_vec();
        for item in &items {
            let abs = self.abs(&item.path);
            let side = xmeta::path_of(&abs);
            files.push(abs);
            if side.exists() {
                files.push(side);
            }
        }
        let batch = recycle::delete(&files, permanent, self.system_bin, &self.hold, xmeta::new_id())?;
        {
            let index = self.index.lock().unwrap();
            for item in &items {
                let _ = index.remove(&item.id);
            }
        }
        if permanent {
            return Ok((None, items));
        }
        let undo = batch.id.clone();
        if !batch.held.is_empty() {
            let later = batch.clone();
            std::thread::spawn(move || {
                std::thread::sleep(recycle::UNDO);
                recycle::finish(&later);
            });
        }
        let mut batches = self.batches.lock().unwrap();
        batches.retain(|b| b.at.elapsed() < recycle::UNDO);
        batches.push(batch);
        Ok((Some(undo), items))
    }

    /// Undo of a delete, while it lasts. Returns the files back in the library.
    pub fn restore(&self, undo: &str) -> Result<Vec<PathBuf>, TrashError> {
        let batch = {
            let mut batches = self.batches.lock().unwrap();
            let pos = batches
                .iter()
                .position(|b| b.id == undo && b.at.elapsed() < recycle::UNDO)
                .ok_or_else(|| TrashError::NoTrash("expired".into()))?;
            batches.remove(pos)
        };
        let back = recycle::restore(&batch)?;
        self.scan();
        Ok(back
            .into_iter()
            .filter(|p| scan::format_of(p).is_some())
            .collect())
    }

    pub fn remember_open(&self, undo: &str, paths: Vec<PathBuf>) {
        self.reopen.lock().unwrap().insert(undo.to_string(), paths);
    }

    pub fn take_open(&self, undo: &str) -> Vec<PathBuf> {
        self.reopen.lock().unwrap().remove(undo).unwrap_or_default()
    }

    /// Renames an item's file in its folder, keeping its extension, ID and data
    /// (TRY-006). A change of case only is allowed.
    pub fn rename(&self, id: &str, name: &str) -> Result<Item, RenameError> {
        let item = self.get(id).ok_or(RenameError::NotFound)?;
        let clean = crate::file_name(name).ok_or(RenameError::Invalid)?;
        let from = self.abs(&item.path);
        let ext = from
            .extension()
            .map(|e| e.to_string_lossy().into_owned())
            .unwrap_or_default();
        let to = from.with_file_name(format!("{clean}.{ext}"));
        let same_file = to.to_string_lossy().to_lowercase() == from.to_string_lossy().to_lowercase();
        if to.exists() && !same_file {
            return Err(RenameError::Exists(clean));
        }
        std::fs::rename(&from, &to)?;
        xmeta::follow(&from, &to);
        let rel = scan::relative(&self.root, &to).unwrap_or_default();
        let renamed = Item {
            path: rel,
            name: clean,
            ..item
        };
        let _ = self.index.lock().unwrap().upsert(&renamed);
        Ok(renamed)
    }

    /// Follows changes made outside XcreenShot (Explorer, Finder, sync services):
    /// `on_change` runs after each scan that changed something.
    /// Takes the tags and favorite of sidecars changed elsewhere (another computer of a
    /// synced library, LIB-029): the last change wins. Returns whether any changed.
    pub fn refresh_marks(&self, sidecars: &[PathBuf]) -> bool {
        let index = self.index.lock().unwrap();
        let mut changed = false;
        for side in sidecars {
            let file = side.with_extension("");
            let Some(rel) = scan::relative(&self.root, &file) else {
                continue;
            };
            let (Ok(Some(item)), Some(meta)) = (index.by_path(&rel), xmeta::read(&file)) else {
                continue;
            };
            if meta.id == item.id && (meta.tags != item.tags || meta.favorite != item.favorite) {
                let _ = index.set_marks(&item.id, &meta.tags, meta.favorite);
                changed = true;
            }
        }
        changed
    }

    pub fn watch(self: &Arc<Self>, on_change: impl Fn() + Send + 'static) {
        let weak = Arc::downgrade(self);
        let debouncer = new_debouncer(SETTLE, move |result: DebounceEventResult| {
            let Ok(events) = result else { return };
            // Sidecars changed by another computer bring their tags (LIB-029).
            let sidecars: Vec<PathBuf> = events
                .iter()
                .filter(|e| e.path.extension().is_some_and(|x| x == "xmeta"))
                .map(|e| e.path.clone())
                .collect();
            if !sidecars.is_empty() {
                if let Some(library) = weak.upgrade() {
                    if library.refresh_marks(&sidecars) {
                        on_change();
                    }
                }
            }
            // Items, and folders (renamed or moved); not the index nor the sidecars.
            let relevant = events
                .iter()
                .any(|e| scan::format_of(&e.path).is_some() || e.path.extension().is_none());
            if !relevant {
                return;
            }
            if let Some(library) = weak.upgrade() {
                if library.scan() {
                    on_change();
                }
            }
        });
        match debouncer {
            Ok(mut d) => {
                if let Err(e) = d.watcher().watch(&self.root, RecursiveMode::Recursive) {
                    tracing::warn!(error = %e, "cannot watch the library folder");
                }
                *self.watcher.lock().unwrap() = Some(d);
            }
            Err(e) => tracing::warn!(error = %e, "cannot watch the library folder"),
        }
    }
}

fn build(f: &Found, id: String, favorite: bool, tags: Vec<String>) -> Item {
    let info = scan::info(f);
    Item {
        id,
        path: f.rel.clone(),
        name: info.name,
        format: info.format,
        size: f.size,
        modified: f.modified,
        captured: info.captured,
        width: info.width,
        height: info.height,
        app: info.app,
        url: info.url,
        favorite,
        tags,
        text: info.text,
        cloud: f.cloud,
    }
}

/// Opens the system file manager with the file selected (EDT-109).
pub fn show_in_folder(path: &Path) -> std::io::Result<()> {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        // `/select,"…"` must reach Explorer as written, quotes included.
        std::process::Command::new("explorer.exe")
            .raw_arg(format!("/select,\"{}\"", path.display()))
            .spawn()
            .map(|_| ())
    }
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg("-R")
            .arg(path)
            .spawn()
            .map(|_| ())
    }
    #[cfg(not(any(windows, target_os = "macos")))]
    {
        let dir = path.parent().unwrap_or(path);
        std::process::Command::new("xdg-open")
            .arg(dir)
            .spawn()
            .map(|_| ())
    }
}
