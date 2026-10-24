//! `xsdoc` protocol: the editor canvas loads a document's pixels without passing them
//! through JSON (ADR 0004).
//!
//! - `/<documentId>/base` returns the original pixels as encoded in the file (PNG/JPG).
//! - `/<documentId>/thumb` returns a small PNG of the final image, for the strip of
//!   open documents.

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
        [id, "base"] => editor
            .store
            .with(id, |d| (d.base_source.bytes.clone(), d.base_source.mime)),
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
