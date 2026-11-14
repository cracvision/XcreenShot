//! `xsdoc` protocol: the editor canvas loads a document's pixels without passing them
//! through JSON (ADR 0004).
//!
//! - `/<documentId>/base` returns the original pixels as encoded in the file (PNG/JPG),
//!   or, with flattened objects (EDT-091), a PNG of them over those pixels, which starts
//!   where `document.info` says (`baseView`).
//! - `/<documentId>/thumb` returns a small PNG of the final image, for the strip of
//!   open documents.
//! - `/<documentId>/object/<objectId>` returns a text or blur object as Rust draws it.
//! - `/<documentId>/image/<name>` returns a picture of the document's `images/` folder
//!   (a floating image, EDT-078).
//! - `/sample/<tool>/<styleId | favoriteId | current>` returns the sample of a style
//!   (EDT-015); the URL carries the style's hash, so a changed one gets a new one.
//! - `/multi/<itemId>` returns the thumbnail of a multi-capture (CAP-067).
//! - `/scroll/preview` returns the joined image of the running scrolling capture, small
//!   (CAP-034).
//! - `/picker/<windowId>` returns a window's thumbnail in the window picker (CAP-030).
//! - `/library/<itemId>/thumb` returns the thumbnail of a library item (LIB-001); the
//!   URL carries the item's modified time, so a changed file gets a new one.
//! - `/<documentId>/video` returns the file of a video open in the editor, by ranges,
//!   for its player (ADR 0008). Only open videos: never any other path.

use tauri::http::{Request, Response};
use tauri::{AppHandle, Manager};

use crate::AppState;

thread_local! {
    /// Where the canvas is in the final image just made (EDT-053).
    static FINAL_ORIGIN: std::cell::Cell<[i64; 2]> = const { std::cell::Cell::new([0, 0]) };
}

/// Longest side of the strip thumbnails, in physical pixels.
const THUMB: u32 = 240;

pub fn serve(app: &AppHandle, request: &Request<Vec<u8>>) -> Response<Vec<u8>> {
    let parts: Vec<&str> = request.uri().path().trim_matches('/').split('/').collect();
    let core = app.state::<AppState>().core.clone();
    let editor = xs_editor::editor(&core);
    if let [id, "video"] = parts.as_slice() {
        let path = editor
            .store
            .with(id, |d| d.path.clone().filter(|_| d.is_video()))
            .flatten();
        return match path {
            Some(p) => video(&p, request),
            None => Response::builder()
                .status(404)
                .body(Vec::new())
                .unwrap_or_default(),
        };
    }
    let body = match parts.as_slice() {
        ["library", item, "thumb"] => xs_library::library(&core)
            .thumbnail(item)
            .map(|png| (std::sync::Arc::new(png), "image/png")),
        // A guide's step, for the quick view that walks through it (LIB-019).
        ["library", item, "step", n] => {
            xs_editor::quick_view::library_step(&core, item, n.parse().unwrap_or(1))
        }
        // The large preview (LIB-013), at most `max` pixels on its longest side.
        ["library", item, "view", max] => {
            xs_editor::quick_view::library_preview(&core, item, max.parse().unwrap_or(2560))
        }
        [id, "base"] => {
            let flat = editor
                .store
                .with(id, |d| {
                    !d.document.flattened.is_empty() || !d.document.patches.is_empty()
                })
                .unwrap_or(false);
            if flat {
                editor.store.snapshot(id).ok().and_then(|snap| {
                    let (image, _, _) = xs_image::render::flattened_view(&snap.document, &snap.base)?;
                    xs_image::encode::png(&image, 0.0, true)
                        .ok()
                        .map(|png| (std::sync::Arc::new(png), "image/png"))
                })
            } else {
                // The picture that replaced the original pixels, if any (ADR 0006).
                editor.store.with(id, |d| match &d.document.base_image {
                    Some(name) => d
                        .images_file(name)
                        .map(|b| (std::sync::Arc::new(b), "image/png"))
                        .unwrap_or((d.base_source.bytes.clone(), d.base_source.mime)),
                    None => (d.base_source.bytes.clone(), d.base_source.mime),
                })
            }
        }
        // Text and blur as Rust draws them (ADR 0004); the canvas places them.
        [id, "object", object] => editor.store.snapshot(id).ok().and_then(|snap| {
            let (image, _, _) = xs_image::render::object_image(&snap.document, &snap.base, object)?;
            xs_image::encode::png(&image, 0.0, true)
                .ok()
                .map(|png| (std::sync::Arc::new(png), "image/png"))
        }),
        // The image with its effects (EDT-053), for the editor's preview; where the
        // canvas's corner is goes in the X-Origin header.
        [id, "final"] => editor.store.snapshot(id).ok().and_then(|snap| {
            let (image, origin) = snap.render_with_origin();
            let png = xs_image::encode::png(&image, 0.0, true).ok()?;
            FINAL_ORIGIN.with(|o| o.set(origin));
            Some((std::sync::Arc::new(png), "image/png"))
        }),
        // An object's shadow (EDT-021); `document.info` says where it goes (`shadowAt`).
        [id, "shadow", object] => editor.store.snapshot(id).ok().and_then(|snap| {
            let image = xs_image::render::shadow_image(&snap.document, &snap.base, object)?;
            xs_image::encode::png(&image, 0.0, true)
                .ok()
                .map(|png| (std::sync::Arc::new(png), "image/png"))
        }),
        // A sample of a saved style, a favorite or the current style (EDT-015).
        ["sample", tool, which] => {
            xs_editor::sample_png(&core, tool, which).map(|png| (std::sync::Arc::new(png), "image/png"))
        }
        // A stamp of the collection (EDT-031), in the app's language.
        ["stamp", id] => {
            let lang = if core.lang() == xs_core::Lang::Es {
                xs_image::stamps::Lang::Es
            } else {
                xs_image::stamps::Lang::En
            };
            xs_image::stamps::thumbnail(id, lang, 96).and_then(|img| {
                xs_image::encode::png(&img, 0.0, true)
                    .ok()
                    .map(|png| (std::sync::Arc::new(png), "image/png"))
            })
        }
        // A template of the picker (EDT-095): built-in ones laid out for `n` pictures.
        ["template", id, n] => xs_editor::template_thumbnail(&core, &unescape(id), n.parse().unwrap_or(3))
            .map(|png| (std::sync::Arc::new(png), "image/png")),
        ["multi", item] => xs_capture::multi::thumbnail_png(&core, item).map(|png| (png, "image/png")),
        ["scroll", "preview"] => xs_capture::scroll::preview_png(&core).map(|png| (png, "image/png")),
        ["picker", id] => id
            .parse()
            .ok()
            .and_then(|id| xs_capture::picker::thumbnail_png(&core, id))
            .map(|png| (png, "image/png")),
        [id, "image", name] => editor.store.image_file(id, name).map(|bytes| {
            let mime = if name.ends_with(".jpg") {
                "image/jpeg"
            } else {
                "image/png"
            };
            (std::sync::Arc::new(bytes), mime)
        }),
        [id, "thumb"] => editor.store.snapshot(id).ok().and_then(|snap| {
            let small = xs_image::encode::thumbnail(&snap.render(), THUMB);
            xs_image::encode::png(&small, 0.0, true)
                .ok()
                .map(|png| (std::sync::Arc::new(png), "image/png"))
        }),
        _ => None,
    };
    let builder = Response::builder()
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Expose-Headers", "X-Origin");
    // The final image says where the canvas is in it (EDT-053).
    let builder = match parts.as_slice() {
        [_, "final"] => {
            let [x, y] = FINAL_ORIGIN.with(|o| o.get());
            builder.header("X-Origin", format!("{x},{y}"))
        }
        _ => builder,
    };
    match body {
        Some((bytes, mime)) => builder
            .header("Content-Type", mime)
            // The URL carries the document version, so a cached copy is never stale.
            .header("Cache-Control", "max-age=31536000, immutable")
            .body(bytes.as_ref().clone())
            .unwrap_or_default(),
        None => builder.status(404).body(Vec::new()).unwrap_or_default(),
    }
}

/// `%XX` escapes of a path segment decoded (an own template's name).
fn unescape(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        let hex = |b: u8| (b as char).to_digit(16);
        match (
            bytes[i],
            bytes.get(i + 1).copied().and_then(hex),
            bytes.get(i + 2).copied().and_then(hex),
        ) {
            (b'%', Some(h), Some(l)) => {
                out.push((h * 16 + l) as u8);
                i += 3;
            }
            (b, _, _) => {
                out.push(b);
                i += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// The most a video answer carries: the player asks again for the rest.
const VIDEO_CHUNK: u64 = 4 << 20;

/// `bytes=start-end` (the end optional) within a file of `len` bytes.
fn byte_range(header: &str, len: u64) -> Option<(u64, u64)> {
    let (start, end) = header.trim().strip_prefix("bytes=")?.split_once('-')?;
    let start: u64 = start.trim().parse().ok()?;
    let end: u64 = match end.trim() {
        "" => len.checked_sub(1)?,
        e => e.parse::<u64>().ok()?.min(len.checked_sub(1)?),
    };
    (start <= end).then_some((start, end))
}

/// A video file by ranges (ADR 0008): the player jumps anywhere without the whole
/// file in memory.
fn video(path: &std::path::Path, request: &Request<Vec<u8>>) -> Response<Vec<u8>> {
    use std::io::{Read, Seek, SeekFrom};
    let not_found = || {
        Response::builder()
            .status(404)
            .body(Vec::new())
            .unwrap_or_default()
    };
    let Ok(mut file) = std::fs::File::open(path) else {
        return not_found();
    };
    let len = file.metadata().map(|m| m.len()).unwrap_or(0);
    let mime = match path
        .extension()
        .map(|e| e.to_string_lossy().to_ascii_lowercase())
        .as_deref()
    {
        Some("mov") => "video/quicktime",
        Some("gif") => "image/gif",
        _ => "video/mp4",
    };
    let asked = request
        .headers()
        .get("range")
        .and_then(|v| v.to_str().ok())
        .and_then(|r| byte_range(r, len));
    let (start, end) = match asked {
        Some(r) => r,
        None if len == 0 => (0, 0),
        None => (0, len - 1),
    };
    let end = end.min(start + VIDEO_CHUNK - 1);
    let mut body = vec![0u8; if len == 0 { 0 } else { (end - start + 1) as usize }];
    if file.seek(SeekFrom::Start(start)).is_err() || file.read_exact(&mut body).is_err() {
        return not_found();
    }
    let partial = asked.is_some() || end + 1 < len;
    let builder = Response::builder()
        .status(if partial { 206 } else { 200 })
        .header("Content-Type", mime)
        .header("Accept-Ranges", "bytes")
        .header("Content-Length", body.len().to_string())
        .header("Access-Control-Allow-Origin", "*")
        .header("Cache-Control", "no-store");
    let builder = if partial {
        builder.header("Content-Range", format!("bytes {start}-{end}/{len}"))
    } else {
        builder
    };
    builder.body(body).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::byte_range;

    #[test]
    fn ranges_of_a_video() {
        assert_eq!(byte_range("bytes=0-", 1000), Some((0, 999)));
        assert_eq!(byte_range("bytes=100-199", 1000), Some((100, 199)));
        assert_eq!(byte_range("bytes=900-5000", 1000), Some((900, 999)));
        assert_eq!(byte_range("bytes=1200-", 1000), None);
        assert_eq!(byte_range("items=0-1", 1000), None);
        assert_eq!(byte_range("bytes=0-", 0), None);
    }
}
