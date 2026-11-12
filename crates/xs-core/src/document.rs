//! Editable image document and its `.xshot` container (SPEC §7, EDT-106).
//!
//! `document.json` keeps the canvas, the crop over the original pixels (`base.png`,
//! never cropped, EDT-050) and the ordered list of objects. Positions are image
//! pixels with the origin at the top-left corner of the canvas. Fields this version
//! does not know are kept and written back (forward compatibility within a major
//! version).

use std::collections::BTreeMap;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Format version written by this build. A file with a higher major is refused.
pub const FORMAT_MAJOR: u32 = 1;
pub const FORMAT_MINOR: u32 = 0;
/// Longest side of `thumb.png` (EDT-106).
pub const THUMB_SIZE: u32 = 512;

/// Rectangle in whole pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PixelRect {
    pub x: i64,
    pub y: i64,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    /// `#RRGGBB` or `#RRGGBBAA`; `None` is transparent (EDT-074).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// A gradient instead of the color, `{type, angle, stops}` over the whole canvas
    /// (EDT-020, EDT-074). `color` then holds its first stop, for what needs one color.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub gradient: Option<Value>,
    #[serde(default)]
    pub corner_radius: f64,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// An annotation or floating image. `props` holds its geometry (`from`, `to`, `rect`…)
/// and its `style`; each tool reads them with its own defaults.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Object {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(flatten)]
    pub props: Map<String, Value>,
}

impl Object {
    /// Moves it by (`dx`, `dy`) pixels, whatever its type: its points (`from`, `to`,
    /// `at`, `points`, `tails`) and its `rect`.
    pub fn translate(&mut self, dx: f64, dy: f64) {
        translate_props(&mut self.props, dx, dy);
    }
}

/// [`Object::translate`] on the properties alone.
pub fn translate_props(props: &mut Map<String, Value>, dx: f64, dy: f64) {
    let shift = |v: &mut Value| {
        if let Some(a) = v.as_array_mut() {
            if let (Some(x), Some(y)) = (
                a.first().and_then(Value::as_f64),
                a.get(1).and_then(Value::as_f64),
            ) {
                *a = vec![Value::from(x + dx), Value::from(y + dy)];
            }
        }
    };
    for key in ["from", "to", "at"] {
        if let Some(v) = props.get_mut(key) {
            shift(v);
        }
    }
    // Polylines' points and callouts' tails (EDT-028).
    for key in ["points", "tails"] {
        if let Some(points) = props.get_mut(key).and_then(Value::as_array_mut) {
            points.iter_mut().for_each(shift);
        }
    }
    // A box, and a lens's zone (EDT-038).
    for key in ["rect", "source"] {
        if let Some(r) = props.get_mut(key).and_then(Value::as_object_mut) {
            for (k, d) in [("x", dx), ("y", dy)] {
                if let Some(v) = r.get(k).and_then(Value::as_f64) {
                    r.insert(k.into(), Value::from(v + d));
                }
            }
        }
    }
}

/// An edit of pixels (ADR 0005): a rectangle of the image, in base pixels, whose pixels
/// (a picture of `images/`) replace what is there after the flattened layers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Patch {
    pub rect: PixelRect,
    pub image: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Simplify (EDT-041): whether the original image shows under the simplified shapes,
/// and how strongly. Without it the image shows as usual.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimplifyView {
    pub show_original: bool,
    /// 0–100 %.
    pub opacity: f64,
}

/// Objects merged into the image (EDT-091). They are drawn over `base.png` and under
/// the objects, and can become objects again: the file keeps them as they were.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Flattened {
    /// `f1`, `f2`…
    pub id: String,
    /// The objects as they were, in base pixels: a crop does not move them.
    pub objects: Vec<Object>,
    /// Why it cannot become objects again: `blur` (its purpose is to hide what is
    /// below) or `covered` (a blur was flattened over it later). Such a layer is merged
    /// into `base.png` when the file is written, so the file no longer has what it hid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permanent: Option<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    /// `image`, or `guide` for the step recorder (SMT-).
    #[serde(rename = "type", default = "image_kind")]
    pub kind: String,
    pub canvas: Canvas,
    /// Where the canvas sits over `base.png`, in base pixels (EDT-049, EDT-050).
    pub crop: PixelRect,
    /// Earlier crops, oldest first (EDT-050, EDT-076).
    #[serde(default)]
    pub crop_history: Vec<PixelRect>,
    /// The crop the user chose, in base pixels, while the canvas is larger so that
    /// objects sticking out of it fit (EDT-071). `None` when the canvas is that crop.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub frame: Option<PixelRect>,
    /// Objects a crop left wholly or partly outside the canvas. They stay as the crop
    /// left them and do not make the canvas grow (EDT-049, EDT-071).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outside: Vec<String>,
    /// Resolution written to exported files (EDT-077).
    pub dpi: f64,
    #[serde(default)]
    pub objects: Vec<Object>,
    /// Objects merged into the image, oldest first (EDT-091).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub flattened: Vec<Flattened>,
    /// Edits of pixels, oldest first (ADR 0005). A written file has none: they merge
    /// into `base.png`.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub patches: Vec<Patch>,
    /// The original pixels are this picture of `images/` instead of `base.png`: after
    /// resizing, turning or flipping the image (ADR 0006). A written file has none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub base_image: Option<String>,
    /// `base.png` is no longer the image as captured or opened: a save merged edits
    /// into it or left out cropped pixels (EDT-050, EDT-076, ADR 0006). Revert to
    /// original warns about it.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub original_changed: bool,
    /// A template's settings and sections (EDT-095 to EDT-101, ADR 0007); none in an
    /// ordinary document.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<Value>,
    /// How the original shows under simplified shapes (EDT-041).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub simplify: Option<SimplifyView>,
    /// Document effects (EDT-053, M2).
    #[serde(default)]
    pub effects: Vec<Value>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

fn image_kind() -> String {
    "image".to_string()
}

impl Document {
    /// A new document showing all of a `width` × `height` base image.
    pub fn new(width: u32, height: u32, dpi: f64) -> Document {
        Document {
            kind: image_kind(),
            canvas: Canvas {
                width,
                height,
                color: None,
                gradient: None,
                corner_radius: 0.0,
                extra: Map::new(),
            },
            crop: PixelRect {
                x: 0,
                y: 0,
                width,
                height,
            },
            crop_history: Vec::new(),
            frame: None,
            outside: Vec::new(),
            dpi,
            objects: Vec::new(),
            flattened: Vec::new(),
            patches: Vec::new(),
            base_image: None,
            original_changed: false,
            template: None,
            simplify: None,
            effects: Vec::new(),
            extra: Map::new(),
        }
    }

    pub fn object(&self, id: &str) -> Option<&Object> {
        self.objects.iter().find(|o| o.id == id)
    }

    pub fn object_mut(&mut self, id: &str) -> Option<&mut Object> {
        self.objects.iter_mut().find(|o| o.id == id)
    }

    /// An object id not used in this document, flattened objects included: `o1`,
    /// `o2`…
    pub fn next_object_id(&self) -> String {
        let max = self
            .objects
            .iter()
            .chain(self.flattened.iter().flat_map(|f| &f.objects))
            .filter_map(|o| o.id.strip_prefix('o').and_then(|n| n.parse::<u64>().ok()))
            .max()
            .unwrap_or(0);
        format!("o{}", max + 1)
    }

    /// A flattened layer id not used yet: `f1`, `f2`…
    pub fn next_flattened_id(&self) -> String {
        let max = self
            .flattened
            .iter()
            .filter_map(|f| f.id.strip_prefix('f').and_then(|n| n.parse::<u64>().ok()))
            .max()
            .unwrap_or(0);
        format!("f{}", max + 1)
    }
}

/// `manifest.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    pub format: String,
    /// `major.minor`.
    pub version: String,
    /// App that wrote the file, for support.
    #[serde(default)]
    pub app: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Manifest {
    pub fn current() -> Manifest {
        Manifest {
            format: "xshot".to_string(),
            version: format!("{FORMAT_MAJOR}.{FORMAT_MINOR}"),
            app: format!("XcreenShot {}", crate::platform::VERSION),
            extra: Map::new(),
        }
    }

    fn major(&self) -> Option<u32> {
        self.version.split('.').next()?.trim().parse().ok()
    }
}

/// Everything inside a `.xshot` file.
#[derive(Debug, Clone, PartialEq)]
pub struct Xshot {
    pub manifest: Manifest,
    pub document: Document,
    /// Original pixels, never cropped (EDT-050).
    pub base_png: Vec<u8>,
    /// Capture metadata (CAP-056).
    pub meta: Value,
    pub thumb_png: Vec<u8>,
    /// Floating images and stamps, by name without the `images/` prefix (M2).
    pub images: BTreeMap<String, Vec<u8>>,
}

/// What the library needs from a `.xshot` without its pixels (LIB-022).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct XshotSummary {
    /// Size of the final image: the canvas.
    pub width: u32,
    pub height: u32,
    /// Capture metadata (CAP-056).
    pub meta: Value,
    pub thumb_png: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum XshotError {
    #[error("{0}")]
    Io(#[from] std::io::Error),
    /// Not a ZIP, or a ZIP without the required parts.
    #[error("not a valid .xshot file: {0}")]
    Invalid(String),
    /// Written by a newer XcreenShot with an incompatible format (EDT-106).
    #[error("format version {0} is newer than this XcreenShot")]
    Newer(String),
}

fn invalid(e: impl std::fmt::Display) -> XshotError {
    XshotError::Invalid(e.to_string())
}

fn read_entry<R: Read + Seek>(
    zip: &mut zip::ZipArchive<R>,
    name: &str,
) -> Result<Option<Vec<u8>>, XshotError> {
    match zip.by_name(name) {
        Ok(mut f) => {
            let mut buf = Vec::with_capacity(f.size() as usize);
            f.read_to_end(&mut buf)?;
            Ok(Some(buf))
        }
        Err(zip::result::ZipError::FileNotFound) => Ok(None),
        Err(e) => Err(invalid(e)),
    }
}

fn required<R: Read + Seek>(zip: &mut zip::ZipArchive<R>, name: &str) -> Result<Vec<u8>, XshotError> {
    read_entry(zip, name)?.ok_or_else(|| XshotError::Invalid(format!("missing {name}")))
}

/// The manifest, if this build can read the file.
fn manifest_of<R: Read + Seek>(zip: &mut zip::ZipArchive<R>) -> Result<Manifest, XshotError> {
    let manifest: Manifest = serde_json::from_slice(&required(zip, "manifest.json")?).map_err(invalid)?;
    if manifest.format != "xshot" {
        return Err(XshotError::Invalid(format!("format {}", manifest.format)));
    }
    match manifest.major() {
        Some(major) if major > FORMAT_MAJOR => Err(XshotError::Newer(manifest.version)),
        Some(_) => Ok(manifest),
        None => Err(XshotError::Invalid(format!("version {}", manifest.version))),
    }
}

impl Xshot {
    pub fn read(path: &Path) -> Result<Xshot, XshotError> {
        let file = std::fs::File::open(path)?;
        Xshot::from_reader(std::io::BufReader::new(file))
    }

    /// Reads only the small parts, without the pixels: the canvas size, the capture
    /// metadata and the thumbnail (LIB-022).
    pub fn read_summary(path: &Path) -> Result<XshotSummary, XshotError> {
        let file = std::fs::File::open(path)?;
        let mut zip = zip::ZipArchive::new(std::io::BufReader::new(file)).map_err(invalid)?;
        manifest_of(&mut zip)?;
        let document: Document =
            serde_json::from_slice(&required(&mut zip, "document.json")?).map_err(invalid)?;
        let meta = match read_entry(&mut zip, "meta.json")? {
            Some(b) => serde_json::from_slice(&b).map_err(invalid)?,
            None => Value::Object(Map::new()),
        };
        Ok(XshotSummary {
            width: document.canvas.width,
            height: document.canvas.height,
            meta,
            thumb_png: read_entry(&mut zip, "thumb.png")?.unwrap_or_default(),
        })
    }

    pub fn from_reader<R: Read + Seek>(reader: R) -> Result<Xshot, XshotError> {
        let mut zip = zip::ZipArchive::new(reader).map_err(invalid)?;
        let manifest = manifest_of(&mut zip)?;
        let document = serde_json::from_slice(&required(&mut zip, "document.json")?).map_err(invalid)?;
        let base_png = required(&mut zip, "base.png")?;
        let meta = match read_entry(&mut zip, "meta.json")? {
            Some(b) => serde_json::from_slice(&b).map_err(invalid)?,
            None => Value::Object(Map::new()),
        };
        let thumb_png = read_entry(&mut zip, "thumb.png")?.unwrap_or_default();
        let names: Vec<String> = zip
            .file_names()
            .filter_map(Result::ok)
            .filter(|n| n.starts_with("images/") && n.len() > "images/".len())
            .map(|n| n.into_owned())
            .collect();
        let mut images = BTreeMap::new();
        for name in names {
            let bytes = required(&mut zip, &name)?;
            images.insert(name["images/".len()..].to_string(), bytes);
        }
        Ok(Xshot {
            manifest,
            document,
            base_png,
            meta,
            thumb_png,
            images,
        })
    }

    pub fn to_writer<W: Write + Seek>(&self, writer: W) -> Result<W, XshotError> {
        use zip::write::SimpleFileOptions;
        use zip::CompressionMethod;
        let json = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
        // PNG data is already compressed.
        let stored = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
        let mut zip = zip::ZipWriter::new(writer);
        let mut put = |name: &str, bytes: &[u8], opts: SimpleFileOptions| -> Result<(), XshotError> {
            zip.start_file(name, opts).map_err(invalid)?;
            zip.write_all(bytes)?;
            Ok(())
        };
        put("manifest.json", &json_bytes(&self.manifest)?, json)?;
        put("document.json", &json_bytes(&self.document)?, json)?;
        put("meta.json", &json_bytes(&self.meta)?, json)?;
        put("base.png", &self.base_png, stored)?;
        if !self.thumb_png.is_empty() {
            put("thumb.png", &self.thumb_png, stored)?;
        }
        for (name, bytes) in &self.images {
            put(&format!("images/{name}"), bytes, stored)?;
        }
        zip.finish().map_err(invalid)
    }

    /// Writes the file safely: a partial file next to it, renamed over the old one
    /// only when complete, so a crash never leaves a damaged document (EDT-106).
    pub fn write(&self, path: &Path) -> Result<(), XshotError> {
        if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent)?;
        }
        let partial = partial_path(path);
        let result = (|| {
            let file = std::fs::File::create(&partial)?;
            let file = self.to_writer(std::io::BufWriter::new(file))?;
            let file = file.into_inner().map_err(|e| e.into_error())?;
            file.sync_all()?;
            drop(file);
            std::fs::rename(&partial, path)?;
            Ok(())
        })();
        if result.is_err() {
            let _ = std::fs::remove_file(&partial);
        }
        result
    }
}

/// `name.xshot` → `name.xshot.partial`.
pub fn partial_path(path: &Path) -> PathBuf {
    let mut s = path.as_os_str().to_os_string();
    s.push(".partial");
    PathBuf::from(s)
}

fn json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, XshotError> {
    serde_json::to_vec_pretty(value).map_err(invalid)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn sample() -> Xshot {
        let mut document = Document::new(4, 3, 144.0);
        document.objects.push(Object {
            id: "o1".into(),
            kind: "arrow".into(),
            props: json!({"from": [0, 0], "to": [3, 2], "style": {"color": "#F2545B"}})
                .as_object()
                .unwrap()
                .clone(),
        });
        Xshot {
            manifest: Manifest::current(),
            document,
            base_png: vec![1, 2, 3],
            meta: json!({"mode": "region"}),
            thumb_png: vec![4, 5],
            images: BTreeMap::new(),
        }
    }

    #[test]
    fn round_trips_through_a_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("a.xshot");
        let x = sample();
        x.write(&path).unwrap();
        assert!(!partial_path(&path).exists());
        let back = Xshot::read(&path).unwrap();
        assert_eq!(back, x);
        assert_eq!(back.document.next_object_id(), "o2");
    }

    #[test]
    fn keeps_unknown_fields_of_a_newer_minor_version() {
        let mut x = sample();
        x.manifest.version = "1.7".into();
        let mut doc = serde_json::to_value(&x.document).unwrap();
        doc["futureField"] = json!({"a": 1});
        doc["objects"][0]["futureProp"] = json!(true);
        x.document = serde_json::from_value(doc).unwrap();
        let mut buf = std::io::Cursor::new(Vec::new());
        x.to_writer(&mut buf).unwrap();
        buf.set_position(0);
        let back = Xshot::from_reader(buf).unwrap();
        let v = serde_json::to_value(&back.document).unwrap();
        assert_eq!(v["futureField"], json!({"a": 1}));
        assert_eq!(v["objects"][0]["futureProp"], json!(true));
    }

    #[test]
    fn refuses_a_newer_major_version() {
        let mut x = sample();
        x.manifest.version = "2.0".into();
        let mut buf = std::io::Cursor::new(Vec::new());
        x.to_writer(&mut buf).unwrap();
        buf.set_position(0);
        assert!(matches!(Xshot::from_reader(buf), Err(XshotError::Newer(v)) if v == "2.0"));
    }

    #[test]
    fn rejects_files_that_are_not_xshot() {
        let r = Xshot::from_reader(std::io::Cursor::new(b"not a zip".to_vec()));
        assert!(matches!(r, Err(XshotError::Invalid(_))));
    }

    #[test]
    fn a_failed_write_leaves_the_previous_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("keep.xshot");
        sample().write(&path).unwrap();
        let before = std::fs::read(&path).unwrap();
        // A folder where the partial file should go makes the write fail.
        std::fs::create_dir(partial_path(&path)).unwrap();
        let mut changed = sample();
        changed.document.dpi = 300.0;
        assert!(changed.write(&path).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }
}
