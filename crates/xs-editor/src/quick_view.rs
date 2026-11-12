//! The large preview of the Library (LIB-013): a library item's final image, with
//! its annotations, fitted to the screen, without opening it in the editor.

use std::sync::{Arc, Mutex};

use xs_core::Core;

use crate::store::DocStore;

/// Previews kept in memory: the one shown and its neighbours, which the view asks
/// for ahead so that ← and → are instant.
const KEEP: usize = 6;

/// Pictures up to this many pixels go to the web view as they are: it decodes them
/// faster than reading and shrinking them here.
const AS_IS: u64 = 40_000_000;

type Preview = (Arc<Vec<u8>>, &'static str);

static CACHE: Mutex<Vec<(String, Preview)>> = Mutex::new(Vec::new());

/// The image of library item `id` with its annotations, at most `max` pixels on its
/// longest side (PNG), or a PNG or JPG file as it is.
pub fn library_preview(core: &Arc<Core>, id: &str, max: u32) -> Option<Preview> {
    let lib = xs_library::library(core);
    let item = lib.get(id)?;
    let max = max.clamp(64, 8192);
    let key = format!("{id}|{}|{max}", item.modified);
    if let Some(hit) = CACHE.lock().unwrap().iter().find(|(k, _)| *k == key) {
        return Some(hit.1.clone());
    }
    let path = lib.abs(&item.path);
    let pixels = item.width.unwrap_or(0) as u64 * item.height.unwrap_or(0) as u64;
    let preview: Preview = if item.format != "xshot" && pixels <= AS_IS {
        let mime = if item.format == "png" {
            "image/png"
        } else {
            "image/jpeg"
        };
        (Arc::new(std::fs::read(&path).ok()?), mime)
    } else {
        let doc = DocStore::load(&path, lib.root()).ok()?;
        let image = DocStore::render_doc(&doc).ok()?;
        let small = xs_image::encode::thumbnail(&image, max);
        drop(image);
        (
            Arc::new(xs_image::encode::png(&small, 0.0, true).ok()?),
            "image/png",
        )
    };
    let mut cache = CACHE.lock().unwrap();
    cache.retain(|(k, _)| !k.starts_with(&format!("{id}|")));
    cache.push((key, preview.clone()));
    if cache.len() > KEEP {
        cache.remove(0);
    }
    Some(preview)
}
