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
//! - `/multi/<itemId>` returns the thumbnail of a multi-capture (CAP-067).
//! - `/library/<itemId>/thumb` returns the thumbnail of a library item (LIB-001); the
//!   URL carries the item's modified time, so a changed file gets a new one.

use tauri::http::{Request, Response};
use tauri::{AppHandle, Manager};

use crate::AppState;

/// Longest side of the strip thumbnails, in physical pixels.
const THUMB: u32 = 240;

pub fn serve(app: &AppHandle, request: &Request<Vec<u8>>) -> Response<Vec<u8>> {
    let parts: Vec<&str> = request.uri().path().trim_matches('/').split('/').collect();
    let core = app.state::<AppState>().core.clone();
    let editor = xs_editor::editor(&core);
    let body = match parts.as_slice() {
        ["library", item, "thumb"] => xs_library::library(&core)
            .thumbnail(item)
            .map(|png| (std::sync::Arc::new(png), "image/png")),
        [id, "base"] => {
            let flat = editor
                .store
                .with(id, |d| !d.document.flattened.is_empty())
                .unwrap_or(false);
            if flat {
                editor.store.snapshot(id).ok().and_then(|snap| {
                    let (image, _, _) = xs_image::render::flattened_view(&snap.document, &snap.base)?;
                    xs_image::encode::png(&image, 0.0, true)
                        .ok()
                        .map(|png| (std::sync::Arc::new(png), "image/png"))
                })
            } else {
                editor
                    .store
                    .with(id, |d| (d.base_source.bytes.clone(), d.base_source.mime))
            }
        }
        // Text and blur as Rust draws them (ADR 0004); the canvas places them.
        [id, "object", object] => editor.store.snapshot(id).ok().and_then(|snap| {
            let (image, _, _) = xs_image::render::object_image(&snap.document, &snap.base, object)?;
            xs_image::encode::png(&image, 0.0, true)
                .ok()
                .map(|png| (std::sync::Arc::new(png), "image/png"))
        }),
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
        ["multi", item] => xs_capture::multi::thumbnail_png(&core, item).map(|png| (png, "image/png")),
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
    let builder = Response::builder().header("Access-Control-Allow-Origin", "*");
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
