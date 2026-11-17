//! Import a PDF (CAP-041): its pages become captures, one per page or all of them in
//! one image. It takes the place of a virtual printer (SPEC D-03): any app prints to
//! the system's PDF printer, and the file is imported here.
//!
//! Pages are drawn by the system (Windows.Data.Pdf), as they print: text, pictures,
//! comments and filled-in form fields. The Import PDF window chooses the pages, the
//! resolution, the result and the background; `capture.pdf` with `dialog: false` (or
//! from automation) imports at once.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use image::RgbaImage;
use serde_json::{json, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode, Lang};

use crate::{finish, Options, Rect, Shot};

/// Most pages in one import (CAP-041).
pub const MAX_PAGES: usize = 200;
/// The resolutions offered, in dots per inch.
pub const DPIS: [u32; 5] = [72, 96, 150, 200, 300];
/// Longest side of an image, as a scrolling capture (CAP-036).
pub const MOST: u32 = 60_000;
/// "All pages in one image": the gap between pages and the light gray around them.
const GAP: u32 = 20;
const SHEET: [u8; 4] = [230, 232, 236, 255];

/// Why a PDF could not be read.
#[derive(Debug, Clone, PartialEq)]
pub enum PdfError {
    /// It has a password, and none or a wrong one was given.
    Password,
    /// Not a PDF, damaged, or unreadable.
    Unreadable(String),
    /// No PDF engine on this system yet.
    Unsupported,
}

/// The pages to import from a range like `1-3, 7` (1-based, in the order written,
/// each once); empty is every page.
pub fn parse_pages(text: &str, count: u32) -> Result<Vec<u32>, String> {
    if text.trim().is_empty() {
        return Ok((1..=count).collect());
    }
    let mut pages = Vec::new();
    for part in text.split([',', ';']) {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (a, b) = match part.split_once('-') {
            Some((a, b)) => (a.trim(), b.trim()),
            None => (part, part),
        };
        let a: u32 = a.parse().map_err(|_| format!("\"{part}\""))?;
        let b: u32 = if b.is_empty() {
            count
        } else {
            b.parse().map_err(|_| format!("\"{part}\""))?
        };
        if a == 0 || b == 0 || a > count || b > count || a > b {
            return Err(format!("\"{part}\": 1-{count}"));
        }
        for p in a..=b {
            if !pages.contains(&p) {
                pages.push(p);
            }
        }
    }
    if pages.is_empty() {
        return Err("no pages".into());
    }
    Ok(pages)
}

/// The size in pixels of a page `size` device-independent pixels (1/96 inch) at `dpi`.
pub fn page_pixels(size: (f32, f32), dpi: u32) -> (u32, u32) {
    let k = dpi as f64 / 96.0;
    let px = |v: f32| ((v as f64 * k).round() as u32).clamp(1, MOST);
    (px(size.0), px(size.1))
}

/// "All pages in one image" (CAP-041): one under the other, 20 px apart on light
/// gray; when they do not fit in the largest size, in several images.
pub fn stack(pages: &[RgbaImage]) -> Vec<RgbaImage> {
    let mut groups: Vec<Vec<&RgbaImage>> = vec![Vec::new()];
    let mut height = GAP;
    for page in pages {
        let need = page.height() + GAP;
        let group = groups.last_mut().expect("a group");
        if !group.is_empty() && height + need > MOST {
            groups.push(Vec::new());
            height = GAP;
        }
        groups.last_mut().expect("a group").push(page);
        height += need;
    }
    groups
        .into_iter()
        .filter(|g| !g.is_empty())
        .map(|group| {
            let width = group.iter().map(|p| p.width()).max().unwrap_or(1) + 2 * GAP;
            let height = (group.iter().map(|p| p.height() + GAP).sum::<u32>() + GAP).min(MOST);
            let mut out = RgbaImage::from_pixel(width, height, image::Rgba(SHEET));
            let mut y = GAP;
            for page in group {
                let x = (width - page.width()) / 2;
                image::imageops::overlay(&mut out, page, x as i64, y as i64);
                y += page.height() + GAP;
            }
            out
        })
        .collect()
}

// ---------------------------------------------------------------------------
// The engine

#[cfg(windows)]
mod imp {
    use std::path::Path;

    use image::RgbaImage;
    use windows::core::HSTRING;
    use windows::Data::Pdf::{PdfDocument, PdfPageRenderOptions};
    use windows::Storage::StorageFile;
    use windows::Storage::Streams::{DataReader, InMemoryRandomAccessStream};
    use windows::UI::Color;

    use super::PdfError;

    /// ERROR_WRONG_PASSWORD: a password is needed, or the one given is wrong.
    const WRONG_PASSWORD: i32 = 0x8007_052B_u32 as i32;

    fn unreadable(e: windows::core::Error) -> PdfError {
        if e.code().0 == WRONG_PASSWORD {
            PdfError::Password
        } else {
            PdfError::Unreadable(e.message().to_string())
        }
    }

    /// A PDF, and how much bigger than asked the system draws (it applies the
    /// display's scale to the size given): measured once.
    pub struct Doc(PdfDocument, std::cell::Cell<Option<f64>>);

    pub fn open(path: &Path, password: Option<&str>) -> Result<Doc, PdfError> {
        let path = std::path::absolute(path).map_err(|e| PdfError::Unreadable(e.to_string()))?;
        let file = StorageFile::GetFileFromPathAsync(&HSTRING::from(path.to_string_lossy().as_ref()))
            .and_then(|op| op.join())
            .map_err(unreadable)?;
        let doc = match password.filter(|p| !p.is_empty()) {
            Some(p) => PdfDocument::LoadFromFileWithPasswordAsync(&file, &HSTRING::from(p)),
            None => PdfDocument::LoadFromFileAsync(&file),
        }
        .and_then(|op| op.join())
        .map_err(unreadable)?;
        Ok(Doc(doc, std::cell::Cell::new(None)))
    }

    impl Doc {
        pub fn pages(&self) -> u32 {
            self.0.PageCount().unwrap_or(0)
        }

        /// A page's size in device-independent pixels (1/96 inch).
        pub fn size(&self, index: u32) -> Option<(f32, f32)> {
            let s = self.0.GetPage(index).ok()?.Size().ok()?;
            Some((s.Width, s.Height))
        }

        /// A page drawn at `width` × `height` pixels, on white or on nothing.
        pub fn render(
            &self,
            index: u32,
            width: u32,
            height: u32,
            transparent: bool,
        ) -> Result<RgbaImage, PdfError> {
            // The system scales the size asked: ask for less so it comes out exact.
            let scale = match self.1.get() {
                Some(s) => s,
                None => {
                    let probe = self.draw(index, 100, 100, transparent)?;
                    let s = (probe.width() as f64 / 100.0).max(0.1);
                    self.1.set(Some(s));
                    s
                }
            };
            let ask = |v: u32| ((v as f64 / scale).round() as u32).max(1);
            let image = self.draw(index, ask(width), ask(height), transparent)?;
            if image.dimensions() == (width, height) {
                return Ok(image);
            }
            Ok(image::imageops::resize(
                &image,
                width,
                height,
                image::imageops::FilterType::Lanczos3,
            ))
        }

        fn draw(
            &self,
            index: u32,
            width: u32,
            height: u32,
            transparent: bool,
        ) -> Result<RgbaImage, PdfError> {
            let page = self.0.GetPage(index).map_err(unreadable)?;
            let options = PdfPageRenderOptions::new().map_err(unreadable)?;
            options.SetDestinationWidth(width).map_err(unreadable)?;
            options.SetDestinationHeight(height).map_err(unreadable)?;
            let background = if transparent {
                Color {
                    A: 0,
                    R: 255,
                    G: 255,
                    B: 255,
                }
            } else {
                Color {
                    A: 255,
                    R: 255,
                    G: 255,
                    B: 255,
                }
            };
            options.SetBackgroundColor(background).map_err(unreadable)?;
            let stream = InMemoryRandomAccessStream::new().map_err(unreadable)?;
            page.RenderWithOptionsToStreamAsync(&stream, &options)
                .and_then(|op| op.join())
                .map_err(unreadable)?;
            // The page arrives as a PNG.
            let size = stream.Size().map_err(unreadable)? as u32;
            let input = stream.GetInputStreamAt(0).map_err(unreadable)?;
            let reader = DataReader::CreateDataReader(&input).map_err(unreadable)?;
            reader
                .LoadAsync(size)
                .and_then(|op| op.join())
                .map_err(unreadable)?;
            let mut bytes = vec![0u8; size as usize];
            reader.ReadBytes(&mut bytes).map_err(unreadable)?;
            image::load_from_memory(&bytes)
                .map(|i| i.to_rgba8())
                .map_err(|e| PdfError::Unreadable(e.to_string()))
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use std::path::Path;

    use image::RgbaImage;

    use super::PdfError;

    pub struct Doc;

    pub fn open(_path: &Path, _password: Option<&str>) -> Result<Doc, PdfError> {
        Err(PdfError::Unsupported)
    }

    impl Doc {
        pub fn pages(&self) -> u32 {
            0
        }
        pub fn size(&self, _index: u32) -> Option<(f32, f32)> {
            None
        }
        pub fn render(&self, _i: u32, _w: u32, _h: u32, _t: bool) -> Result<RgbaImage, PdfError> {
            Err(PdfError::Unsupported)
        }
    }
}

pub use imp::Doc;

/// Opens a PDF, with its password if it has one.
pub fn open(path: &Path, password: Option<&str>) -> Result<Doc, PdfError> {
    imp::open(path, password)
}

// ---------------------------------------------------------------------------
// The Import PDF windows and imports under way

#[derive(Default)]
pub struct PdfState {
    next: AtomicU64,
    /// Import PDF windows: their id and file.
    windows: Mutex<HashMap<String, PathBuf>>,
    /// Imports under way: their id and Cancel.
    running: Mutex<HashMap<String, Arc<AtomicBool>>>,
}

fn service(core: &Core) -> Arc<PdfState> {
    core.service(PdfState::default)
}

/// A new Import PDF window for `path`: its id.
pub fn new_window(core: &Core, path: &Path) -> String {
    let svc = service(core);
    let id = format!("{}", svc.next.fetch_add(1, Ordering::Relaxed) + 1);
    svc.windows.lock().unwrap().insert(id.clone(), path.to_path_buf());
    id
}

/// The file of a request: `path`, or the one of the window `id`.
fn file_of(core: &Core, p: &Value, lang: Lang) -> Result<PathBuf, CommandError> {
    if let Some(path) = p.get("path").and_then(Value::as_str) {
        return Ok(PathBuf::from(path));
    }
    let id = p.get("id").and_then(Value::as_str).unwrap_or_default();
    service(core)
        .windows
        .lock()
        .unwrap()
        .get(id)
        .cloned()
        .ok_or_else(|| CommandError::new(ErrorCode::NotFound, tr(lang, "errors.pdf_not_found", &[])))
}

fn error(lang: Lang, e: PdfError, path: &Path) -> CommandError {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    match e {
        PdfError::Password => CommandError::new(
            ErrorCode::PermissionDenied,
            tr(lang, "errors.pdf_password", &[("name", &name)]),
        )
        .with_details(json!({ "reason": "password" })),
        PdfError::Unreadable(why) => CommandError::new(
            ErrorCode::InvalidParams,
            tr(lang, "errors.pdf_unreadable", &[("name", &name)]),
        )
        .with_details(json!({ "reason": "unreadable", "detail": why })),
        PdfError::Unsupported => CommandError::new(
            ErrorCode::NotSupportedOnPlatform,
            tr(lang, "errors.pdf_unsupported", &[]),
        ),
    }
}

fn password(p: &Value) -> Option<&str> {
    p.get("password").and_then(Value::as_str)
}

/// `capture.pdf_info`: its pages and their sizes, or that it needs a password.
pub fn info(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let path = file_of(core, p, ctx.lang)?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    match open(&path, password(p)) {
        Ok(doc) => {
            let sizes: Vec<Value> = (0..doc.pages())
                .map(|i| json!(doc.size(i).map(|(w, h)| [w, h])))
                .collect();
            Ok(json!({
                "path": path,
                "name": name,
                "pages": doc.pages(),
                "sizes": sizes,
                "needsPassword": false,
            }))
        }
        Err(PdfError::Password) => Ok(json!({
            "path": path,
            "name": name,
            "pages": 0,
            "needsPassword": true,
            "wrongPassword": password(p).is_some_and(|p| !p.is_empty()),
        })),
        Err(e) => Err(error(ctx.lang, e, &path)),
    }
}

/// `capture.pdf_page`: one page as a small PNG (the window's thumbnails).
pub fn page_png(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    use base64::Engine;
    let path = file_of(core, p, ctx.lang)?;
    let doc = open(&path, password(p)).map_err(|e| error(ctx.lang, e, &path))?;
    let page = p.get("page").and_then(Value::as_u64).unwrap_or(1).max(1) as u32;
    if page > doc.pages() {
        return Err(CommandError::new(
            ErrorCode::InvalidParams,
            format!("page: 1-{}", doc.pages()),
        ));
    }
    let size = doc.size(page - 1).unwrap_or((816.0, 1056.0));
    let width = p
        .get("width")
        .and_then(Value::as_u64)
        .unwrap_or(160)
        .clamp(16, 1600) as u32;
    let height = ((size.1 / size.0.max(1.0)) * width as f32).round().max(1.0) as u32;
    let image = doc
        .render(page - 1, width, height, false)
        .map_err(|e| error(ctx.lang, e, &path))?;
    let png = xs_image::encode::png(&image, 0.0, false).map_err(CommandError::internal)?;
    Ok(json!({
        "page": page,
        "width": width,
        "height": height,
        "png": base64::engine::general_purpose::STANDARD.encode(png),
    }))
}

/// `capture.pdf_cancel`: stops an import, or all of them.
pub fn cancel(core: &Core, id: Option<&str>) -> Value {
    let svc = service(core);
    let running = svc.running.lock().unwrap();
    let mut n = 0;
    for (k, flag) in running.iter() {
        if id.is_none_or(|id| id == k) {
            flag.store(true, Ordering::Relaxed);
            n += 1;
        }
    }
    json!({ "cancelled": n })
}

/// Imports the pages (CAP-041): one capture each, or all in one image (several if
/// they do not fit). With Open in = Editor the first one opens and the others wait in
/// the recent tray. Cancel stops between pages: what is done stays, nothing is left
/// half made.
pub fn import(core: &Core, ctx: &Context, opts: &Options, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let path = file_of(core, p, lang)?;
    let doc = open(&path, password(p)).map_err(|e| error(lang, e, &path))?;
    let count = doc.pages();
    let pages = parse_pages(p.get("pages").and_then(Value::as_str).unwrap_or(""), count).map_err(|e| {
        CommandError::new(
            ErrorCode::InvalidParams,
            tr(lang, "errors.pdf_pages", &[("pages", &e)]),
        )
    })?;
    if pages.len() > MAX_PAGES {
        return Err(CommandError::new(
            ErrorCode::InvalidParams,
            tr(lang, "errors.pdf_too_many", &[("most", &MAX_PAGES.to_string())]),
        ));
    }
    let dpi = p.get("dpi").and_then(Value::as_u64).unwrap_or(150) as u32;
    let dpi = if DPIS.contains(&dpi) { dpi } else { 150 };
    let combine = p.get("combine").and_then(Value::as_bool).unwrap_or(false);
    let transparent = p.get("background").and_then(Value::as_str) == Some("transparent");
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();

    let job = p
        .get("id")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| format!("cli-{}", service(core).next.fetch_add(1, Ordering::Relaxed) + 1));
    let stop = Arc::new(AtomicBool::new(false));
    service(core)
        .running
        .lock()
        .unwrap()
        .insert(job.clone(), stop.clone());
    let progress = |done: usize| {
        core.host()
            .pdf_event(json!({ "id": job, "done": done, "total": pages.len() }));
    };
    let result = (|| {
        let shot_of = |image: RgbaImage, page: Option<u32>| Shot {
            rect: Rect {
                x: 0,
                y: 0,
                width: image.width(),
                height: image.height(),
            },
            image,
            mode: "pdf",
            scale: dpi as f32 / 96.0,
            monitor: String::new(),
            app: None,
            window_title: page.map(|n| format!("{name} · {n}")),
            window_id: None,
            pointer: None,
            warnings: Vec::new(),
        };
        let mut documents = Vec::new();
        let mut rendered = Vec::new();
        for (i, &page) in pages.iter().enumerate() {
            if stop.load(Ordering::Relaxed) {
                return Ok(None);
            }
            progress(i);
            let size = doc.size(page - 1).unwrap_or((816.0, 1056.0));
            let (w, h) = page_pixels(size, dpi);
            let image = doc
                .render(page - 1, w, h, transparent)
                .map_err(|e| error(lang, e, &path))?;
            if combine {
                rendered.push(image);
                continue;
            }
            let mut page_opts = opts.clone();
            page_opts.open_editor = opts.open_editor && documents.is_empty();
            page_opts.copy = opts.copy && documents.is_empty();
            page_opts.meta = Some(json!({ "pdf": { "file": name, "path": path, "page": page } }));
            let r = finish(core, ctx, shot_of(image, Some(page)), &page_opts)?;
            documents.push(r["documentId"].clone());
        }
        if combine {
            if stop.load(Ordering::Relaxed) {
                return Ok(None);
            }
            let images = stack(&rendered);
            if images.len() > 1 {
                core.host().notify(
                    "captures",
                    &tr(lang, "app.name", &[]),
                    &tr(lang, "notify.pdf_split", &[("count", &images.len().to_string())]),
                );
            }
            for image in images {
                let mut page_opts = opts.clone();
                page_opts.open_editor = opts.open_editor && documents.is_empty();
                page_opts.copy = opts.copy && documents.is_empty();
                page_opts.meta = Some(json!({ "pdf": { "file": name, "path": path, "pages": pages } }));
                let r = finish(core, ctx, shot_of(image, None), &page_opts)?;
                documents.push(r["documentId"].clone());
            }
        }
        progress(pages.len());
        Ok(Some(documents))
    })();
    service(core).running.lock().unwrap().remove(&job);
    match result {
        Ok(Some(documents)) => {
            tracing::info!(
                pages = pages.len(),
                captures = documents.len(),
                dpi,
                combine,
                "pdf imported"
            );
            Ok(json!({ "status": "completed", "count": documents.len(), "documents": documents }))
        }
        Ok(None) => Err(crate::cancelled(lang)),
        Err(e) => Err(e),
    }
}

/// Forgets an Import PDF window that closed.
pub fn forget_window(core: &Core, id: &str) {
    service(core).windows.lock().unwrap().remove(id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_ranges() {
        assert_eq!(parse_pages("", 4).unwrap(), vec![1, 2, 3, 4]);
        assert_eq!(parse_pages("2-3", 5).unwrap(), vec![2, 3]);
        assert_eq!(parse_pages("1-3, 7", 9).unwrap(), vec![1, 2, 3, 7]);
        assert_eq!(parse_pages("4, 2, 4", 5).unwrap(), vec![4, 2]);
        assert_eq!(parse_pages("3-", 5).unwrap(), vec![3, 4, 5]);
        assert!(parse_pages("0", 5).is_err());
        assert!(parse_pages("6", 5).is_err());
        assert!(parse_pages("3-2", 5).is_err());
        assert!(parse_pages("a", 5).is_err());
    }

    /// CAP-041: an A4 page (595 × 842 pt) at 150 dpi is 1240 × 1754 px.
    #[test]
    fn a4_at_150_dpi() {
        let a4 = (595.276 * 96.0 / 72.0, 841.89 * 96.0 / 72.0);
        assert_eq!(page_pixels(a4, 150), (1240, 1754));
        assert_eq!(page_pixels(a4, 72), (595, 842));
    }

    #[test]
    fn pages_in_one_image_and_split_when_too_tall() {
        let page = RgbaImage::from_pixel(100, 200, image::Rgba([255, 255, 255, 255]));
        let one = stack(&vec![page.clone(); 5]);
        assert_eq!(one.len(), 1);
        assert_eq!(one[0].width(), 140);
        assert_eq!(one[0].height(), 20 + 5 * 220);
        assert_eq!(one[0].get_pixel(5, 5).0, SHEET);
        let tall = RgbaImage::from_pixel(10, 25_000, image::Rgba([255, 255, 255, 255]));
        let split = stack(&vec![tall; 5]);
        assert_eq!(split.len(), 3);
        assert!(split.iter().all(|i| i.height() <= MOST));
    }
}
