//! A file as the source (CAP-040): images that already exist go down the capture
//! route (effects, destinations, library) without touching the original. A PDF opens
//! Import PDF (CAP-041) and an `.xshot` opens in the editor, without a new capture.
//! Files that cannot be read are skipped and listed at the end.

use std::path::{Path, PathBuf};

use image::RgbaImage;
use serde_json::{json, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode, Source};

use crate::{finish, Options, Rect, Shot};

/// Image files offered in the file picker.
pub const IMAGE_FILES: [&str; 11] = [
    "png", "jpg", "jpeg", "gif", "bmp", "tif", "tiff", "webp", "heic", "heif", "ico",
];

/// What a file is, for the file source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Image,
    Pdf,
    Document,
    Other,
}

pub fn kind_of(path: &Path) -> Kind {
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "pdf" => Kind::Pdf,
        "xshot" => Kind::Document,
        e if IMAGE_FILES.contains(&e) => Kind::Image,
        _ => Kind::Other,
    }
}

/// Reads an image: with the app's own decoders (PNG, JPG, BMP, TIFF, ICO), else with
/// the system's (GIF, WebP, HEIC when Windows has it).
pub fn decode_file(path: &Path) -> Result<RgbaImage, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    match xs_image::encode::decode(&bytes) {
        Ok(image) => Ok(image),
        Err(own) => imp::decode(&bytes).map_err(|_| own),
    }
}

#[cfg(windows)]
mod imp {
    use image::RgbaImage;
    use windows::Graphics::Imaging::{BitmapAlphaMode, BitmapDecoder, BitmapPixelFormat};
    use windows::Storage::Streams::{Buffer, DataReader, DataWriter, InMemoryRandomAccessStream};

    /// The first frame of any image Windows can read, as RGBA.
    pub fn decode(bytes: &[u8]) -> Result<RgbaImage, String> {
        let run = || -> windows::core::Result<RgbaImage> {
            let stream = InMemoryRandomAccessStream::new()?;
            let writer = DataWriter::CreateDataWriter(&stream)?;
            writer.WriteBytes(bytes)?;
            writer.StoreAsync()?.join()?;
            writer.FlushAsync()?.join()?;
            writer.DetachStream()?;
            stream.Seek(0)?;
            let decoder = BitmapDecoder::CreateAsync(&stream)?.join()?;
            let bitmap = decoder
                .GetSoftwareBitmapConvertedAsync(BitmapPixelFormat::Rgba8, BitmapAlphaMode::Straight)?
                .join()?;
            let (w, h) = (bitmap.PixelWidth()? as u32, bitmap.PixelHeight()? as u32);
            let buffer = Buffer::Create(w * h * 4)?;
            bitmap.CopyToBuffer(&buffer)?;
            let reader = DataReader::FromBuffer(&buffer)?;
            let mut pixels = vec![0u8; (w * h * 4) as usize];
            reader.ReadBytes(&mut pixels)?;
            RgbaImage::from_raw(w, h, pixels)
                .ok_or_else(|| windows::core::Error::from_hresult(windows::core::HRESULT(-1)))
        };
        run().map_err(|e| e.message().to_string())
    }
}

#[cfg(not(windows))]
mod imp {
    use image::RgbaImage;

    pub fn decode(_bytes: &[u8]) -> Result<RgbaImage, String> {
        Err("not supported".into())
    }
}

/// `capture.file` (CAP-040): each file in order; PDFs to Import PDF, `.xshot` to the
/// editor; what cannot be read is skipped and listed in a notification.
pub fn run(core: &Core, ctx: &Context, opts: &Options, paths: &[PathBuf]) -> CommandResult {
    let lang = ctx.lang;
    let mut documents = Vec::new();
    let mut skipped: Vec<Value> = Vec::new();
    let mut dialogs = Vec::new();
    for path in paths {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        match kind_of(path) {
            Kind::Pdf => {
                if ctx.source.is_automation() {
                    // From automation every page comes in at once (capture.pdf).
                    match crate::pdf::import(core, ctx, opts, &json!({ "path": path })) {
                        Ok(r) => documents.extend(r["documents"].as_array().cloned().unwrap_or_default()),
                        Err(e) => skipped.push(json!({ "path": path, "reason": e.message })),
                    }
                } else {
                    let id = crate::pdf::new_window(core, path);
                    core.host().pdf_window(&id)?;
                    dialogs.push(id);
                }
            }
            Kind::Document => {
                let opened = core.call(
                    Source::Ui,
                    "document.open",
                    json!({ "path": path, "show": true }),
                    None,
                );
                if let Err(e) = opened {
                    skipped.push(json!({ "path": path, "reason": e.message }));
                }
            }
            Kind::Image | Kind::Other => match decode_file(path) {
                Ok(image) => {
                    let modified = std::fs::metadata(path)
                        .and_then(|m| m.modified())
                        .ok()
                        .map(|t| chrono::DateTime::<chrono::Local>::from(t).to_rfc3339());
                    let mut file_opts = opts.clone();
                    // With Open in = Editor, the first one opens (the rest wait in the
                    // recent tray), and the clipboard gets the last one.
                    file_opts.open_editor = opts.open_editor && documents.is_empty();
                    file_opts.cursor = false;
                    file_opts.meta =
                        Some(json!({ "file": { "name": name, "path": path, "modified": modified } }));
                    let shot = Shot {
                        rect: Rect {
                            x: 0,
                            y: 0,
                            width: image.width(),
                            height: image.height(),
                        },
                        image,
                        mode: "file",
                        scale: 1.0,
                        monitor: String::new(),
                        app: None,
                        window_title: Some(name.clone()),
                        window_id: None,
                        pointer: None,
                        warnings: Vec::new(),
                    };
                    match finish(core, ctx, shot, &file_opts) {
                        Ok(r) => documents.push(r["documentId"].clone()),
                        Err(e) => skipped.push(json!({ "path": path, "reason": e.message })),
                    }
                }
                Err(why) => {
                    tracing::info!(file = %name, error = %why, "file source: skipped");
                    skipped.push(json!({ "path": path, "reason": tr(lang, "errors.file_unreadable", &[]) }));
                }
            },
        }
    }
    if !skipped.is_empty() {
        let names: Vec<String> = skipped
            .iter()
            .filter_map(|s| s["path"].as_str())
            .map(|p| {
                Path::new(p)
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default()
            })
            .collect();
        core.host().notify(
            "problems",
            &tr(lang, "app.name", &[]),
            &tr(lang, "notify.files_skipped", &[("names", &names.join(", "))]),
        );
    }
    if documents.is_empty() && dialogs.is_empty() && !skipped.is_empty() && paths.len() == skipped.len() {
        return Err(
            CommandError::new(ErrorCode::InvalidParams, tr(lang, "errors.file_unreadable", &[]))
                .with_details(json!({ "skipped": skipped })),
        );
    }
    Ok(json!({
        "status": "completed",
        "count": documents.len(),
        "documents": documents,
        "pdfWindows": dialogs,
        "skipped": skipped,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kinds() {
        assert_eq!(kind_of(Path::new("a.PNG")), Kind::Image);
        assert_eq!(kind_of(Path::new("b.webp")), Kind::Image);
        assert_eq!(kind_of(Path::new("c.pdf")), Kind::Pdf);
        assert_eq!(kind_of(Path::new("d.xshot")), Kind::Document);
        assert_eq!(kind_of(Path::new("e.txt")), Kind::Other);
    }

    /// CAP-040: the original is read, never changed.
    #[test]
    fn reading_leaves_the_file_as_it_was() {
        let dir = std::env::temp_dir().join(format!("xs-file-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("one.png");
        let img = RgbaImage::from_pixel(40, 30, image::Rgba([10, 20, 30, 255]));
        img.save(&path).unwrap();
        let before = std::fs::read(&path).unwrap();
        let read = decode_file(&path).unwrap();
        assert_eq!(read.dimensions(), (40, 30));
        assert_eq!(std::fs::read(&path).unwrap(), before);
        assert!(decode_file(&dir.join("missing.png")).is_err());
        std::fs::write(dir.join("broken.png"), b"not an image").unwrap();
        assert!(decode_file(&dir.join("broken.png")).is_err());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
