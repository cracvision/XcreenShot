//! Open documents (EDT-001, TRY-002), their undo history (EDT-092), saving (EDT-012,
//! EDT-106, EDT-107) and the final image (EDT-090, EDT-110).
//!
//! The store only holds state; it never blocks on the disk or on rendering while
//! locked. Callers take a [`Snapshot`], do the slow work, then report back.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde_json::{json, Map, Value};
use xs_core::document::{Document, Manifest, Xshot, XshotError, THUMB_SIZE};
use xs_image::Base;

/// At least 100 steps (EDT-092).
pub const UNDO_LIMIT: usize = 200;
/// Decoded images kept for documents that are not on screen.
const DECODED_LIMIT: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Xshot,
    Png,
    Jpg,
}

impl Format {
    pub fn from_path(path: &Path) -> Option<Format> {
        let ext = path.extension()?.to_string_lossy().to_ascii_lowercase();
        Format::from_name(&ext)
    }

    pub fn from_name(name: &str) -> Option<Format> {
        match name {
            "xshot" => Some(Format::Xshot),
            "png" => Some(Format::Png),
            "jpg" | "jpeg" => Some(Format::Jpg),
            _ => None,
        }
    }

    pub fn ext(self) -> &'static str {
        match self {
            Format::Xshot => "xshot",
            Format::Png => "png",
            Format::Jpg => "jpg",
        }
    }
}

/// Library documents save themselves; the rest ask before losing changes (EDT-012).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Library,
    External,
}

impl Kind {
    pub fn as_str(self) -> &'static str {
        match self {
            Kind::Library => "library",
            Kind::External => "external",
        }
    }
}

/// Encoded original pixels: the `base.png` of a `.xshot`, or the bytes of an opened
/// PNG or JPG. The editor canvas loads them as they are.
#[derive(Clone)]
pub struct BaseSource {
    pub bytes: Arc<Vec<u8>>,
    pub mime: &'static str,
}

struct Step {
    label: String,
    before: Document,
    before_version: u64,
}

pub struct Doc {
    pub id: String,
    pub path: Option<PathBuf>,
    pub kind: Kind,
    pub format: Format,
    pub document: Document,
    pub meta: Value,
    pub base_source: BaseSource,
    /// Size of the original pixels, known without decoding them.
    pub base_size: (u32, u32),
    base: Option<Base>,
    /// Parts of the `.xshot` this build only carries along.
    manifest_extra: Map<String, Value>,
    images: BTreeMap<String, Vec<u8>>,
    undo: Vec<Step>,
    redo: Vec<Step>,
    /// Identifies the current state; a save records it.
    pub version: u64,
    saved_version: u64,
    /// State last written to the recovery folder, and its file id (APP-047).
    pub recovery_version: u64,
    pub recovery_id: Option<String>,
    /// Shown in the editor's strip (opened by the user or with `show`).
    pub shown: bool,
    /// Zoom and position the editor reported (EDT-007).
    pub view: Option<Value>,
    last_used: u64,
}

impl Doc {
    fn new(
        path: Option<PathBuf>,
        kind: Kind,
        format: Format,
        document: Document,
        meta: Value,
        base_source: BaseSource,
    ) -> Doc {
        let base_size = xs_image::encode::png_size(&base_source.bytes)
            .unwrap_or((document.crop.width, document.crop.height));
        Doc {
            id: String::new(),
            path,
            kind,
            format,
            document,
            meta,
            base_source,
            base_size,
            base: None,
            manifest_extra: Map::new(),
            images: BTreeMap::new(),
            undo: Vec::new(),
            redo: Vec::new(),
            version: 0,
            saved_version: 0,
            recovery_version: 0,
            recovery_id: None,
            shown: false,
            view: None,
            last_used: 0,
        }
    }

    pub fn dirty(&self) -> bool {
        self.version != self.saved_version
    }

    pub fn name(&self) -> String {
        self.path
            .as_ref()
            .and_then(|p| p.file_stem())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| "XcreenShot".to_string())
    }

    pub fn undo_label(&self) -> Option<&str> {
        self.undo.last().map(|s| s.label.as_str())
    }

    pub fn redo_label(&self) -> Option<&str> {
        self.redo.last().map(|s| s.label.as_str())
    }

    /// Short description for lists (`document.list`).
    pub fn summary(&self) -> Value {
        json!({
            "documentId": self.id,
            "name": self.name(),
            "path": self.path,
            "kind": self.kind.as_str(),
            "format": self.format.ext(),
            "width": self.document.canvas.width,
            "height": self.document.canvas.height,
            "objects": self.document.objects.len(),
            "dirty": self.dirty(),
            "shown": self.shown,
            "version": self.version,
        })
    }
}

/// Everything needed to save, export or render a document without holding the lock.
#[derive(Clone)]
pub struct Snapshot {
    pub id: String,
    pub path: Option<PathBuf>,
    pub kind: Kind,
    pub format: Format,
    pub name: String,
    pub document: Document,
    pub meta: Value,
    pub base_source: BaseSource,
    pub base: Base,
    pub manifest_extra: Map<String, Value>,
    pub images: BTreeMap<String, Vec<u8>>,
    pub version: u64,
}

impl Snapshot {
    pub fn render(&self) -> image::RgbaImage {
        xs_image::render(&self.document, &self.base)
    }

    /// The `.xshot` of this state (EDT-106). With `drop_cropped`, the original pixels
    /// outside the crop are left out of the file (`editor.deleteCroppedContent`, EDT-050).
    pub fn to_xshot(&self, drop_cropped: bool) -> Result<Xshot, String> {
        let c = self.document.crop;
        let whole = c.x == 0 && c.y == 0 && (c.width, c.height) == (self.base.width, self.base.height);
        let mut document = self.document.clone();
        // The part of the original pixels under the canvas; the canvas may also reach
        // past them, where it grew to fit objects (EDT-071).
        let (kx0, ky0) = (c.x.max(0), c.y.max(0));
        let kx1 = (c.x + c.width as i64).min(self.base.width as i64);
        let ky1 = (c.y + c.height as i64).min(self.base.height as i64);
        let base_png = if drop_cropped && !whole && kx1 > kx0 && ky1 > ky0 {
            let kept = image::imageops::crop_imm(
                &self.base.to_image(),
                kx0 as u32,
                ky0 as u32,
                (kx1 - kx0) as u32,
                (ky1 - ky0) as u32,
            )
            .to_image();
            for r in std::iter::once(&mut document.crop).chain(document.frame.as_mut()) {
                r.x -= kx0;
                r.y -= ky0;
            }
            document.crop_history.clear();
            xs_image::encode::png(&kept, self.document.dpi, true)?
        } else if self.base_source.mime == "image/png" {
            self.base_source.bytes.as_ref().clone()
        } else {
            xs_image::encode::png(&self.base.to_image(), self.document.dpi, true)?
        };
        let thumb = xs_image::encode::thumbnail(&self.render(), THUMB_SIZE);
        let mut manifest = Manifest::current();
        manifest.extra = self.manifest_extra.clone();
        Ok(Xshot {
            manifest,
            document,
            base_png,
            meta: self.meta.clone(),
            thumb_png: xs_image::encode::png(&thumb, 0.0, true)?,
            images: self.images.clone(),
        })
    }
}

#[derive(Debug)]
pub enum OpenError {
    Unsupported,
    Newer,
    Unreadable(String),
}

#[derive(Default)]
struct Inner {
    docs: Vec<Doc>,
    active: Option<String>,
    next_id: u64,
    clock: u64,
}

impl Inner {
    fn get_mut(&mut self, id: &str) -> Option<&mut Doc> {
        self.docs.iter_mut().find(|d| d.id == id)
    }

    fn touch(&mut self, id: &str) {
        self.clock += 1;
        let now = self.clock;
        if let Some(d) = self.get_mut(id) {
            d.last_used = now;
        }
    }

    /// Frees decoded pixels of documents not used lately (they decode again on use).
    fn evict(&mut self, keep: &str) {
        let active = self.active.clone();
        let mut decoded: Vec<(u64, String)> = self
            .docs
            .iter()
            .filter(|d| d.base.is_some() && d.id != keep && Some(&d.id) != active.as_ref())
            .map(|d| (d.last_used, d.id.clone()))
            .collect();
        if decoded.len() <= DECODED_LIMIT {
            return;
        }
        decoded.sort();
        let excess = decoded.len() - DECODED_LIMIT;
        for (_, id) in decoded.into_iter().take(excess) {
            if let Some(d) = self.get_mut(&id) {
                d.base = None;
            }
        }
    }
}

#[derive(Default)]
pub struct DocStore {
    inner: Mutex<Inner>,
}

/// Same file, whatever the spelling of its path.
fn same_file(a: &Path, b: &Path) -> bool {
    match (std::fs::canonicalize(a), std::fs::canonicalize(b)) {
        (Ok(x), Ok(y)) => x == y,
        _ => a == b,
    }
}

impl DocStore {
    /// Reads a document from disk; does not add it to the store.
    pub fn load(path: &Path, library: &Path) -> Result<Doc, OpenError> {
        let format = Format::from_path(path).ok_or(OpenError::Unsupported)?;
        let unreadable = |e: &dyn std::fmt::Display| OpenError::Unreadable(e.to_string());
        match format {
            Format::Xshot => {
                let x = Xshot::read(path).map_err(|e| match e {
                    XshotError::Newer(_) => OpenError::Newer,
                    e => unreadable(&e),
                })?;
                let kind = if xs_library::contains(library, path) {
                    Kind::Library
                } else {
                    Kind::External
                };
                let mut doc = Doc::new(
                    Some(path.to_path_buf()),
                    kind,
                    format,
                    x.document,
                    x.meta,
                    BaseSource {
                        bytes: Arc::new(x.base_png),
                        mime: "image/png",
                    },
                );
                doc.manifest_extra = x.manifest.extra;
                doc.images = x.images;
                Ok(doc)
            }
            Format::Png | Format::Jpg => {
                let bytes = std::fs::read(path).map_err(|e| unreadable(&e))?;
                let (w, h) = image::ImageReader::new(std::io::Cursor::new(&bytes))
                    .with_guessed_format()
                    .map_err(|e| unreadable(&e))?
                    .into_dimensions()
                    .map_err(|e| unreadable(&e))?;
                let (mime, dpi) = if format == Format::Png {
                    ("image/png", xs_image::encode::png_dpi(&bytes).unwrap_or(96.0))
                } else {
                    ("image/jpeg", 96.0)
                };
                let mut doc = Doc::new(
                    Some(path.to_path_buf()),
                    Kind::External,
                    format,
                    Document::new(w, h, dpi),
                    json!({}),
                    BaseSource {
                        bytes: Arc::new(bytes),
                        mime,
                    },
                );
                doc.base_size = (w, h);
                Ok(doc)
            }
        }
    }

    /// A new document from pixels already in memory (a capture).
    pub fn from_image(
        path: PathBuf,
        kind: Kind,
        document: Document,
        meta: Value,
        png: Vec<u8>,
        base: Base,
    ) -> Doc {
        let mut doc = Doc::new(
            Some(path),
            kind,
            Format::Xshot,
            document,
            meta,
            BaseSource {
                bytes: Arc::new(png),
                mime: "image/png",
            },
        );
        doc.base = Some(base);
        doc
    }

    /// Adds a document and returns its id. A file already open is not opened twice:
    /// its existing id is returned (TRY-002).
    pub fn insert(&self, mut doc: Doc, show: bool) -> (String, bool) {
        let mut inner = self.inner.lock().unwrap();
        if let Some(path) = &doc.path {
            if let Some(existing) = inner
                .docs
                .iter_mut()
                .find(|d| d.path.as_deref().is_some_and(|p| same_file(p, path)))
            {
                existing.shown |= show;
                let id = existing.id.clone();
                inner.touch(&id);
                return (id, false);
            }
        }
        inner.next_id += 1;
        doc.id = format!("d{}", inner.next_id);
        doc.shown = show;
        let id = doc.id.clone();
        inner.docs.push(doc);
        inner.touch(&id);
        (id, true)
    }

    pub fn find_by_path(&self, path: &Path) -> Option<String> {
        let inner = self.inner.lock().unwrap();
        inner
            .docs
            .iter()
            .find(|d| d.path.as_deref().is_some_and(|p| same_file(p, path)))
            .map(|d| d.id.clone())
    }

    pub fn active(&self) -> Option<String> {
        self.inner.lock().unwrap().active.clone()
    }

    /// Shows `id` in the editor. Returns the document that was shown before.
    pub fn set_active(&self, id: Option<&str>) -> Option<String> {
        let mut inner = self.inner.lock().unwrap();
        let previous = inner.active.clone();
        if let Some(id) = id {
            if let Some(d) = inner.get_mut(id) {
                d.shown = true;
            }
            inner.touch(id);
        }
        inner.active = id.map(str::to_string);
        previous
    }

    pub fn exists(&self, id: &str) -> bool {
        self.inner.lock().unwrap().docs.iter().any(|d| d.id == id)
    }

    /// Runs `f` on a document.
    pub fn with<R>(&self, id: &str, f: impl FnOnce(&mut Doc) -> R) -> Option<R> {
        let mut inner = self.inner.lock().unwrap();
        inner.get_mut(id).map(f)
    }

    /// Ids in strip order.
    pub fn ids(&self) -> Vec<String> {
        self.inner
            .lock()
            .unwrap()
            .docs
            .iter()
            .map(|d| d.id.clone())
            .collect()
    }

    pub fn summaries(&self) -> Vec<Value> {
        self.inner.lock().unwrap().docs.iter().map(Doc::summary).collect()
    }

    /// Changes a document as one undo step named `label` (an i18n key). `f` works on a
    /// copy: if it fails, nothing changes.
    pub fn mutate<T, E>(
        &self,
        id: &str,
        label: &str,
        f: impl FnOnce(&mut Document) -> Result<T, E>,
    ) -> Option<Result<T, E>> {
        let mut inner = self.inner.lock().unwrap();
        inner.clock += 1;
        let clock = inner.clock;
        let doc = inner.get_mut(id)?;
        let mut next = doc.document.clone();
        let out = f(&mut next);
        if out.is_ok() {
            let before = std::mem::replace(&mut doc.document, next);
            doc.undo.push(Step {
                label: label.to_string(),
                before,
                before_version: doc.version,
            });
            if doc.undo.len() > UNDO_LIMIT {
                doc.undo.remove(0);
            }
            doc.redo.clear();
            doc.version = clock;
            doc.last_used = clock;
        }
        Some(out)
    }

    /// Undoes (`redo == false`) or redoes the last step. Returns its label.
    pub fn step(&self, id: &str, redo: bool) -> Option<Option<String>> {
        let mut inner = self.inner.lock().unwrap();
        let doc = inner.get_mut(id)?;
        let (from, to) = if redo {
            (&mut doc.redo, &mut doc.undo)
        } else {
            (&mut doc.undo, &mut doc.redo)
        };
        let Some(step) = from.pop() else {
            return Some(None);
        };
        let label = step.label.clone();
        let current = std::mem::replace(&mut doc.document, step.before);
        to.push(Step {
            label: step.label,
            before: current,
            before_version: doc.version,
        });
        doc.version = step.before_version;
        Some(Some(label))
    }

    /// Decoded pixels of a document, decoding them if needed (outside the lock).
    pub fn base(&self, id: &str) -> Result<Base, String> {
        let source = {
            let mut inner = self.inner.lock().unwrap();
            inner.touch(id);
            let doc = inner.get_mut(id).ok_or("document closed")?;
            if let Some(b) = &doc.base {
                return Ok(b.clone());
            }
            doc.base_source.bytes.clone()
        };
        let base = Base::from_image(xs_image::encode::decode(&source)?);
        let mut inner = self.inner.lock().unwrap();
        if let Some(doc) = inner.get_mut(id) {
            doc.base = Some(base.clone());
        }
        inner.evict(id);
        Ok(base)
    }

    pub fn snapshot(&self, id: &str) -> Result<Snapshot, String> {
        let base = self.base(id)?;
        let inner = self.inner.lock().unwrap();
        let doc = inner.docs.iter().find(|d| d.id == id).ok_or("document closed")?;
        Ok(Snapshot {
            id: doc.id.clone(),
            path: doc.path.clone(),
            kind: doc.kind,
            format: doc.format,
            name: doc.name(),
            document: doc.document.clone(),
            meta: doc.meta.clone(),
            base_source: doc.base_source.clone(),
            base,
            manifest_extra: doc.manifest_extra.clone(),
            images: doc.images.clone(),
            version: doc.version,
        })
    }

    /// Records that `version` of the document is now in its file.
    pub fn mark_saved(&self, id: &str, version: u64) {
        self.with(id, |d| d.saved_version = version);
    }

    /// The document's state is not in its file (a recovered copy, APP-047).
    pub fn mark_unsaved(&self, id: &str) {
        self.with(id, |d| d.saved_version = u64::MAX);
    }

    /// Links the document to another file (Save as of a file from elsewhere, EDT-107).
    pub fn relink(&self, id: &str, path: PathBuf, kind: Kind, format: Format, version: u64) {
        self.with(id, |d| {
            d.path = Some(path);
            d.kind = kind;
            d.format = format;
            d.saved_version = version;
        });
    }

    /// Removes documents from the store. Returns the ones that were there.
    pub fn remove(&self, ids: &[String]) -> Vec<Doc> {
        let mut inner = self.inner.lock().unwrap();
        let (removed, kept): (Vec<Doc>, Vec<Doc>) = std::mem::take(&mut inner.docs)
            .into_iter()
            .partition(|d| ids.contains(&d.id));
        inner.docs = kept;
        if inner.active.as_ref().is_some_and(|a| ids.contains(a)) {
            inner.active = None;
        }
        removed
    }

    /// Info for the editor and `document.info`; labels are i18n keys.
    pub fn info(&self, id: &str) -> Option<Value> {
        let inner = self.inner.lock().unwrap();
        let d = inner.docs.iter().find(|d| d.id == id)?;
        let (bw, bh) = d.base_size;
        let mut v = serde_json::to_value(&d.document).unwrap_or_default();
        // Text boxes: their laid-out size, for the canvas's handles (EDT-026).
        if let Some(objects) = v.get_mut("objects").and_then(Value::as_array_mut) {
            for o in objects.iter_mut().filter(|o| o["type"] == "text") {
                if let Some(t) = o
                    .as_object()
                    .and_then(|m| xs_image::text::Text::from_props(m).ok())
                {
                    let laid = xs_image::text::layout(&t, d.document.dpi);
                    o["layout"] = json!({ "width": laid.width, "height": laid.height });
                }
            }
        }
        let extra = json!({
            "documentId": d.id,
            "name": d.name(),
            "path": d.path,
            "kind": d.kind.as_str(),
            "format": d.format.ext(),
            "width": d.document.canvas.width,
            "height": d.document.canvas.height,
            "base": {"width": bw, "height": bh, "mime": d.base_source.mime},
            "metadata": d.meta,
            "dirty": d.dirty(),
            "canUndo": !d.undo.is_empty(),
            "canRedo": !d.redo.is_empty(),
            "undoLabel": d.undo_label(),
            "redoLabel": d.redo_label(),
            "version": d.version,
            "shown": d.shown,
            "view": d.view,
        });
        if let (Some(v), Some(e)) = (v.as_object_mut(), extra.as_object()) {
            v.extend(e.clone());
        }
        Some(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store_with_doc() -> (DocStore, String) {
        let store = DocStore::default();
        let img = image::RgbaImage::new(10, 10);
        let doc = DocStore::from_image(
            PathBuf::from("a.xshot"),
            Kind::Library,
            Document::new(10, 10, 96.0),
            json!({}),
            xs_image::encode::png(&img, 96.0, true).unwrap(),
            Base::from_image(img),
        );
        let (id, _) = store.insert(doc, true);
        (store, id)
    }

    fn add(store: &DocStore, id: &str, n: usize) {
        store
            .mutate(id, "history.draw_arrow", |d| {
                d.canvas.corner_radius = n as f64;
                Ok::<_, ()>(())
            })
            .unwrap()
            .unwrap();
    }

    #[test]
    fn undo_and_redo_many_steps() {
        let (store, id) = store_with_doc();
        for n in 1..=5 {
            add(&store, &id, n);
        }
        for _ in 0..5 {
            assert_eq!(
                store.step(&id, false).unwrap().as_deref(),
                Some("history.draw_arrow")
            );
        }
        assert_eq!(store.step(&id, false).unwrap(), None);
        assert_eq!(store.with(&id, |d| d.document.canvas.corner_radius).unwrap(), 0.0);
        for _ in 0..5 {
            store.step(&id, true).unwrap().unwrap();
        }
        assert_eq!(store.with(&id, |d| d.document.canvas.corner_radius).unwrap(), 5.0);
    }

    #[test]
    fn keeps_at_least_100_steps() {
        let (store, id) = store_with_doc();
        for n in 1..=150 {
            add(&store, &id, n);
        }
        let mut undone = 0;
        while store.step(&id, false).unwrap().is_some() {
            undone += 1;
        }
        assert!(undone >= 100);
    }

    #[test]
    fn dirty_follows_the_saved_state_through_undo() {
        let (store, id) = store_with_doc();
        assert!(!store.with(&id, |d| d.dirty()).unwrap());
        add(&store, &id, 1);
        assert!(store.with(&id, |d| d.dirty()).unwrap());
        let v = store.with(&id, |d| d.version).unwrap();
        store.mark_saved(&id, v);
        assert!(!store.with(&id, |d| d.dirty()).unwrap());
        store.step(&id, false);
        assert!(store.with(&id, |d| d.dirty()).unwrap());
        store.step(&id, true);
        assert!(!store.with(&id, |d| d.dirty()).unwrap());
    }

    #[test]
    fn a_failed_change_changes_nothing() {
        let (store, id) = store_with_doc();
        let r = store.mutate(&id, "x", |d| {
            d.canvas.corner_radius = 9.0;
            Err::<(), _>("no")
        });
        assert!(r.unwrap().is_err());
        assert_eq!(store.with(&id, |d| d.document.canvas.corner_radius).unwrap(), 0.0);
        assert_eq!(store.step(&id, false).unwrap(), None);
    }

    #[test]
    fn the_same_file_is_not_opened_twice() {
        let (store, id) = store_with_doc();
        let again = DocStore::from_image(
            PathBuf::from("a.xshot"),
            Kind::Library,
            Document::new(1, 1, 96.0),
            json!({}),
            Vec::new(),
            Base::from_image(image::RgbaImage::new(1, 1)),
        );
        let (id2, added) = store.insert(again, true);
        assert_eq!((id2, added), (id, false));
    }
}
