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

/// Prefix of the pictures this build inserts in `images/` (EDT-078).
pub const INSERTED: &str = "img-";

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
    /// Which pixels `base` holds: `base.png` (`None`) or a picture (ADR 0006).
    base_key: Option<String>,
    /// Parts of the `.xshot` this build only carries along.
    manifest_extra: Map<String, Value>,
    /// The `images/` folder: pictures of floating images (EDT-078) and what this
    /// build only carries along.
    images: Arc<BTreeMap<String, Vec<u8>>>,
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
    /// Pinned in the tray (TRY-004): at the left, never leaves by itself.
    pub pinned: bool,
    /// Where it goes in its part of the tray, pinned or not (TRY-005): smaller first.
    pub place: f64,
    /// When it arrived in the tray: the oldest leaves first (TRY-003).
    pub arrival: u64,
    /// Zoom and position the editor reported (EDT-007).
    pub view: Option<Value>,
    /// The editable `.xshot` kept in the library for a flat file from elsewhere
    /// (LIB-021), once there is one.
    pub editable_copy: Option<PathBuf>,
    /// The next value of each step sequence, after the last placed or edited (EDT-035).
    pub step_next: BTreeMap<String, u64>,
    /// The pixel selection (EDT-043): not in the file, not an undo step.
    pub pixel_selection: Option<crate::pixels::PixelSelection>,
    last_used: u64,
}

impl Doc {
    /// The same, with pictures already in its `images/` folder (a captured pointer,
    /// CAP-047). Its base must already know them.
    pub fn with_images(mut self, images: BTreeMap<String, Vec<u8>>) -> Doc {
        self.images = Arc::new(images);
        self
    }

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
            base_key: None,
            base: None,
            manifest_extra: Map::new(),
            images: Arc::default(),
            undo: Vec::new(),
            redo: Vec::new(),
            version: 0,
            saved_version: 0,
            recovery_version: 0,
            recovery_id: None,
            shown: false,
            pinned: false,
            place: 0.0,
            arrival: 0,
            view: None,
            editable_copy: None,
            step_next: BTreeMap::new(),
            pixel_selection: None,
            last_used: 0,
        }
    }

    /// The bytes of one picture of its `images/` folder.
    pub fn images_file(&self, name: &str) -> Option<Vec<u8>> {
        self.images.get(name).cloned()
    }

    /// Size of the original pixels: of `base.png`, or of the picture that replaced it
    /// (ADR 0006).
    pub fn base_dims(&self) -> (u32, u32) {
        self.document
            .base_image
            .as_ref()
            .and_then(|n| self.images.get(n))
            .and_then(|b| xs_image::encode::png_size(b))
            .unwrap_or(self.base_size)
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
            // Moved or deleted outside XcreenShot (TRY-002).
            "missing": self.path.as_ref().is_some_and(|p| !p.exists()),
            "shown": self.shown,
            "pinned": self.pinned,
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
    pub images: Arc<BTreeMap<String, Vec<u8>>>,
    pub version: u64,
}

impl Snapshot {
    /// The image as it leaves the editor: with its effects (EDT-053).
    pub fn render(&self) -> image::RgbaImage {
        xs_image::render::render_final(&self.document, &self.base, &self.meta).0
    }

    /// [`Self::render`] and where the canvas's corner is in it (the editor's preview).
    pub fn render_with_origin(&self) -> (image::RgbaImage, [i64; 2]) {
        xs_image::render::render_final(&self.document, &self.base, &self.meta)
    }

    /// The `.xshot` of this state (EDT-106). With `drop_cropped`, the original pixels
    /// outside the crop are left out of the file (`editor.deleteCroppedContent`, EDT-050).
    pub fn to_xshot(&self, drop_cropped: bool) -> Result<Xshot, String> {
        let c = self.document.crop;
        let whole = c.x == 0 && c.y == 0 && (c.width, c.height) == (self.base.width, self.base.height);
        let mut document = self.document.clone();
        // A flattened blur, and what is under it, merge into the original pixels: the
        // file no longer has what it hid (EDT-091).
        let baked = xs_image::render::baked_base(&self.document, &self.base);
        document.flattened.retain(|l| l.permanent.is_none());
        // Edits of pixels too (ADR 0005), and a replaced image (ADR 0006).
        document.patches.clear();
        let replaced = document.base_image.take().is_some();
        // The part of the original pixels under the canvas; the canvas may also reach
        // past them, where it grew to fit objects (EDT-071).
        let (kx0, ky0) = (c.x.max(0), c.y.max(0));
        let kx1 = (c.x + c.width as i64).min(self.base.width as i64);
        let ky1 = (c.y + c.height as i64).min(self.base.height as i64);
        let dropping = drop_cropped && !whole && kx1 > kx0 && ky1 > ky0;
        document.original_changed |= dropping || replaced || baked.is_some();
        let base_png = if dropping {
            let kept = image::imageops::crop_imm(
                baked.as_ref().unwrap_or(&self.base.to_image()),
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
            // Flattened layers keep their place on the pixels that stay.
            for o in document.flattened.iter_mut().flat_map(|l| &mut l.objects) {
                o.translate(-kx0 as f64, -ky0 as f64);
            }
            xs_image::encode::png(&kept, self.document.dpi, true)?
        } else if let Some(baked) = &baked {
            xs_image::encode::png(baked, self.document.dpi, true)?
        } else if self.base_source.mime == "image/png" && !replaced {
            self.base_source.bytes.as_ref().clone()
        } else {
            xs_image::encode::png(&self.base.to_image(), self.document.dpi, true)?
        };
        let thumb = xs_image::encode::thumbnail(&self.render(), THUMB_SIZE);
        let mut manifest = Manifest::current();
        manifest.extra = self.manifest_extra.clone();
        // Pictures this build inserted that no object shows any more (an insertion
        // undone) stay out of the file; anything else is carried along.
        let mut used: std::collections::BTreeSet<&str> = document
            .objects
            .iter()
            .chain(document.flattened.iter().flat_map(|l| &l.objects))
            .filter_map(|o| o.props.get("image").and_then(Value::as_str))
            .collect();
        // Watermark pictures of the document's and the floating images' effects (EDT-066).
        let stacks = std::iter::once(&document.effects).chain(
            document
                .objects
                .iter()
                .filter_map(|o| o.props.get("effects").and_then(Value::as_array)),
        );
        for e in stacks.flatten() {
            if let Some(name) = e.get("image").and_then(Value::as_str) {
                used.insert(name);
            }
        }
        let images = self
            .images
            .iter()
            .filter(|(name, _)| !name.starts_with(INSERTED) || used.contains(name.as_str()))
            .map(|(name, bytes)| (name.clone(), bytes.clone()))
            .collect();
        Ok(Xshot {
            manifest,
            document,
            base_png,
            meta: self.meta.clone(),
            thumb_png: xs_image::encode::png(&thumb, 0.0, true)?,
            images,
        })
    }
}

#[derive(Debug)]
pub enum OpenError {
    Unsupported,
    Newer,
    Unreadable(String),
    /// Beyond 32,000 px per side or 500 megapixels (APP-050).
    TooLarge(u32, u32),
}

#[derive(Default)]
struct Inner {
    docs: Vec<Doc>,
    active: Option<String>,
    next_id: u64,
    clock: u64,
    /// Counts arrivals in the tray (TRY-003, TRY-005).
    arrivals: u64,
}

impl Inner {
    fn get_mut(&mut self, id: &str) -> Option<&mut Doc> {
        self.docs.iter_mut().find(|d| d.id == id)
    }

    /// `id` enters the tray: the newest of the part that is not pinned (TRY-005).
    fn arrive(&mut self, id: &str) {
        self.arrivals += 1;
        let n = self.arrivals;
        if let Some(d) = self.get_mut(id) {
            if !d.shown {
                d.shown = true;
                d.arrival = n;
                d.place = -(n as f64);
            }
        }
    }

    /// The tray, left to right: the pinned ones, then the rest (TRY-004, TRY-005).
    fn tray(&self) -> Vec<String> {
        let mut shown: Vec<&Doc> = self.docs.iter().filter(|d| d.shown).collect();
        shown.sort_by(|a, b| b.pinned.cmp(&a.pinned).then(a.place.total_cmp(&b.place)));
        shown.into_iter().map(|d| d.id.clone()).collect()
    }

    /// Numbers each part of the tray 0, 1, 2… in the order given.
    fn renumber(&mut self, order: &[String]) {
        let (mut pinned, mut rest) = (0.0, 0.0);
        for id in order {
            if let Some(d) = self.get_mut(id) {
                let n = if d.pinned { &mut pinned } else { &mut rest };
                d.place = *n;
                *n += 1.0;
            }
        }
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
    /// One decode of the original pixels at a time: the canvas asks for many pieces of
    /// a new document at once (shadows, texts), and each one decoding a 300-megapixel
    /// image on its own took tens of gigabytes (APP-050).
    decoding: Mutex<()>,
}

/// Same file, whatever the spelling of its path.
pub(crate) fn same_file(a: &Path, b: &Path) -> bool {
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
                doc.images = Arc::new(x.images);
                Ok(doc)
            }
            Format::Png | Format::Jpg => {
                let bytes = std::fs::read(path).map_err(|e| unreadable(&e))?;
                let (w, h) = image::ImageReader::new(std::io::Cursor::new(&bytes))
                    .with_guessed_format()
                    .map_err(|e| unreadable(&e))?
                    .into_dimensions()
                    .map_err(|e| unreadable(&e))?;
                if !xs_image::encode::within_limits(w, h) {
                    return Err(OpenError::TooLarge(w, h));
                }
                let (mime, dpi) = if format == Format::Png {
                    ("image/png", xs_image::encode::png_dpi(&bytes).unwrap_or(96.0))
                } else {
                    ("image/jpeg", 96.0)
                };
                // A picture without transparency gets a white canvas, like a capture
                // (EDT-074); its pixels are read later, only its format is looked at.
                let has_alpha = image::ImageReader::new(std::io::Cursor::new(&bytes))
                    .with_guessed_format()
                    .ok()
                    .and_then(|r| r.into_decoder().ok())
                    .map(|d| image::ImageDecoder::color_type(&d).has_alpha())
                    .unwrap_or(true);
                let mut document = Document::new(w, h, dpi);
                if !has_alpha {
                    document.canvas.color = Some("#FFFFFF".to_string());
                }
                let mut doc = Doc::new(
                    Some(path.to_path_buf()),
                    Kind::External,
                    format,
                    document,
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
                let id = existing.id.clone();
                if show {
                    inner.arrive(&id);
                }
                inner.touch(&id);
                return (id, false);
            }
        }
        inner.next_id += 1;
        doc.id = format!("d{}", inner.next_id);
        doc.shown = false;
        let id = doc.id.clone();
        inner.docs.push(doc);
        if show {
            inner.arrive(&id);
        }
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
            inner.arrive(id);
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

    /// Puts a document in the tray without showing it in the editor (TRY-017).
    pub fn show_in_tray(&self, id: &str) {
        self.inner.lock().unwrap().arrive(id);
    }

    /// The tray, left to right (TRY-002, TRY-004, TRY-005).
    pub fn tray(&self) -> Vec<String> {
        self.inner.lock().unwrap().tray()
    }

    /// Pins or unpins (TRY-004). A pinned item goes to the right end of the pinned
    /// ones; an unpinned one back to its place by arrival. Returns whether any changed.
    pub fn set_pinned(&self, ids: &[String], pinned: bool) -> bool {
        let mut inner = self.inner.lock().unwrap();
        let mut order = inner.tray();
        let mut changed = false;
        for id in order.clone().iter().filter(|id| ids.contains(id)) {
            let Some(d) = inner.get_mut(id) else { continue };
            if d.pinned == pinned {
                continue;
            }
            d.pinned = pinned;
            changed = true;
            let arrival = d.arrival;
            order.retain(|o| o != id);
            let at = if pinned {
                order
                    .iter()
                    .rposition(|o| inner.docs.iter().any(|d| d.id == *o && d.pinned))
                    .map_or(0, |i| i + 1)
            } else {
                // Among the rest, before the first that arrived earlier.
                let first_rest = order
                    .iter()
                    .position(|o| inner.docs.iter().any(|d| d.id == *o && !d.pinned))
                    .unwrap_or(order.len());
                order[first_rest..]
                    .iter()
                    .position(|o| inner.docs.iter().any(|d| d.id == *o && d.arrival < arrival))
                    .map_or(order.len(), |i| first_rest + i)
            };
            order.insert(at, id.clone());
        }
        if changed {
            inner.renumber(&order);
        }
        changed
    }

    /// Moves `id` to `index` of the tray (TRY-005). Dropped among the pinned ones, it
    /// is pinned; among the rest, unpinned; at the border, it keeps its state.
    pub fn move_to(&self, id: &str, index: usize) -> Option<usize> {
        let mut inner = self.inner.lock().unwrap();
        let mut order = inner.tray();
        let from = order.iter().position(|o| o == id)?;
        order.remove(from);
        let index = index.min(order.len());
        let pinned_count = order
            .iter()
            .filter(|o| inner.docs.iter().any(|d| d.id == **o && d.pinned))
            .count();
        let doc = inner.get_mut(id)?;
        doc.pinned = match index.cmp(&pinned_count) {
            std::cmp::Ordering::Less => true,
            std::cmp::Ordering::Greater => false,
            std::cmp::Ordering::Equal => doc.pinned,
        };
        order.insert(index, id.to_string());
        inner.renumber(&order);
        inner.tray().iter().position(|o| o == id)
    }

    /// Puts back the pinned items and the order of a previous run (EDT-120).
    pub fn restore_tray(&self, pinned: &[PathBuf], order: &[PathBuf]) {
        let mut inner = self.inner.lock().unwrap();
        let at = |list: &[PathBuf], d: &Doc| {
            d.path
                .as_deref()
                .and_then(|p| list.iter().position(|q| same_file(q, p)))
        };
        let mut placed: Vec<(usize, String)> = Vec::new();
        for d in inner.docs.iter_mut().filter(|d| d.shown) {
            d.pinned = at(pinned, d).is_some();
            if let Some(i) = at(order, d) {
                placed.push((i, d.id.clone()));
            }
        }
        if placed.is_empty() {
            return;
        }
        placed.sort();
        let mut tray = inner.tray();
        let known: Vec<String> = placed.into_iter().map(|p| p.1).collect();
        tray.retain(|id| !known.contains(id));
        let mut order = known;
        order.extend(tray);
        let pinned_first: Vec<String> = order
            .iter()
            .filter(|id| inner.docs.iter().any(|d| d.id == **id && d.pinned))
            .chain(
                order
                    .iter()
                    .filter(|id| inner.docs.iter().any(|d| d.id == **id && !d.pinned)),
            )
            .cloned()
            .collect();
        inner.renumber(&pinned_first);
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
        if let Some(b) = self.cached_base(id)? {
            return Ok(b);
        }
        // Whoever decodes first fills the cache; the rest wait and take it.
        let _one = self
            .decoding
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let (source, images, key) = {
            let mut inner = self.inner.lock().unwrap();
            inner.touch(id);
            let doc = inner.get_mut(id).ok_or("document closed")?;
            // The original pixels: base.png, or the picture that replaced it (ADR 0006).
            let key = doc.document.base_image.clone();
            if let Some(b) = doc.base.as_ref().filter(|_| doc.base_key == key) {
                return Ok(b.clone());
            }
            let source = match &key {
                Some(name) => Arc::new(
                    doc.images
                        .get(name)
                        .cloned()
                        .ok_or("the image's picture is missing")?,
                ),
                None => doc.base_source.bytes.clone(),
            };
            (source, doc.images.clone(), key)
        };
        let mut base = Base::from_image(xs_image::encode::decode(&source)?);
        for (name, bytes) in images.iter() {
            // What is not a picture (carried along from a newer build) is skipped.
            let _ = base.add_image(name, bytes);
        }
        let mut inner = self.inner.lock().unwrap();
        if let Some(doc) = inner.get_mut(id) {
            doc.base = Some(base.clone());
            doc.base_key = key;
        }
        inner.evict(id);
        Ok(base)
    }

    /// The decoded original pixels of `id`, if they are already there.
    fn cached_base(&self, id: &str) -> Result<Option<Base>, String> {
        let mut inner = self.inner.lock().unwrap();
        inner.touch(id);
        let doc = inner.get_mut(id).ok_or("document closed")?;
        let key = doc.document.base_image.clone();
        Ok(doc.base.as_ref().filter(|_| doc.base_key == key).cloned())
    }

    /// The final image of a document read from disk and not in the store.
    pub fn render_doc(doc: &Doc) -> Result<image::RgbaImage, String> {
        let source = match &doc.document.base_image {
            Some(name) => doc.images.get(name).ok_or("the image's picture is missing")?,
            None => doc.base_source.bytes.as_ref(),
        };
        let mut base = Base::from_image(xs_image::encode::decode(source)?);
        for (name, bytes) in doc.images.iter() {
            let _ = base.add_image(name, bytes);
        }
        Ok(xs_image::render::render_final(&doc.document, &base, &doc.meta).0)
    }

    /// Adds pictures to a document's `images/` folder (EDT-078), before the objects
    /// that show them. Decodes them outside the lock.
    pub fn add_images(&self, id: &str, images: &BTreeMap<String, Vec<u8>>) -> Result<(), String> {
        let mut decoded = Base::from_image(image::RgbaImage::new(1, 1));
        for (name, bytes) in images {
            decoded.add_image(name, bytes)?;
        }
        let mut inner = self.inner.lock().unwrap();
        let doc = inner.get_mut(id).ok_or("document closed")?;
        let files = Arc::make_mut(&mut doc.images);
        for (name, bytes) in images {
            files.insert(name.clone(), bytes.clone());
        }
        if let Some(base) = &mut doc.base {
            let pictures = Arc::make_mut(&mut base.images);
            for (name, picture) in decoded.images.iter() {
                pictures.insert(name.clone(), picture.clone());
            }
        }
        Ok(())
    }

    /// The bytes of one picture of a document's `images/` folder.
    pub fn image_file(&self, id: &str, name: &str) -> Option<Vec<u8>> {
        self.with(id, |d| d.images.get(name).cloned()).flatten()
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
    /// The document's file moved (the library moved, LIB-028): nothing else changes.
    pub fn set_path(&self, id: &str, path: PathBuf) {
        self.with(id, |d| d.path = Some(path));
    }

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
        let (bw, bh) = d.base_dims();
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
        // Shadows (EDT-021): where the editor places each one.
        if let Some(objects) = v.get_mut("objects").and_then(Value::as_array_mut) {
            for (o, object) in objects.iter_mut().zip(&d.document.objects) {
                if let Some(at) = xs_image::render::shadow_place(object, d.document.dpi) {
                    o["shadowAt"] = json!(at);
                }
            }
        }
        // Flattened layers (EDT-091): where each is on the canvas, for Unflatten.
        if let Some(layers) = v.get_mut("flattened").and_then(Value::as_array_mut) {
            for (l, layer) in layers.iter_mut().zip(&d.document.flattened) {
                l["bounds"] = json!(xs_image::render::layer_bounds(&d.document, layer));
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
            // With flattened objects the canvas shows a view of the image that may start
            // before the original pixels (EDT-091), in base pixels.
            "baseView": xs_image::render::flattened_view_rect(&d.document, bw, bh),
            "metadata": d.meta,
            "dirty": d.dirty(),
            "canUndo": !d.undo.is_empty(),
            "canRedo": !d.redo.is_empty(),
            "undoLabel": d.undo_label(),
            "redoLabel": d.redo_label(),
            "version": d.version,
            // What the next step of each sequence will show (EDT-035).
            "stepNext": crate::steps::next_values(&d.document, &d.step_next),
            // The pixel selection, in canvas pixels (EDT-043).
            "pixelSelection": d.pixel_selection.as_ref().and_then(|s| s.info(&d.document)),
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
