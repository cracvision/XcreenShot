//! Dragging tray and library items out to other apps, the desktop or a folder
//! (TRY-015). The files are prepared first — a `.xshot` or a document with unsaved
//! changes becomes a flat PNG — and then handed to the system's drag and drop.

use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::time::{Duration, SystemTime};

use serde_json::{json, Value};
use tauri::{AppHandle, WebviewWindow};
use xs_core::{CommandError, Core};

/// Side of the picture that follows the pointer, in pixels.
const DRAG_IMAGE: u32 = 160;
/// Prepared files stay this long, so a slow app can still read them after the drop.
const KEEP: Duration = Duration::from_secs(60 * 60);

/// The folder for one drag, under the local data folder. Older ones are removed.
fn drag_dir(core: &Core) -> PathBuf {
    let root = core.paths.local_dir.join("drag");
    if let Ok(entries) = std::fs::read_dir(&root) {
        for e in entries.flatten() {
            let old = e
                .metadata()
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| SystemTime::now().duration_since(t).ok())
                .is_some_and(|age| age > KEEP);
            if old {
                let _ = std::fs::remove_dir_all(e.path());
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    let stamp = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or_default();
    root.join(stamp.to_string())
}

/// A small picture of the first file for the pointer, saved next to the files.
fn drag_image(first: &Path, dir: &Path) -> Option<PathBuf> {
    let bytes = std::fs::read(first).ok()?;
    let png = match xs_image::encode::decode(&bytes) {
        Ok(image) => {
            let small = xs_image::encode::thumbnail(&image, DRAG_IMAGE);
            xs_image::encode::png(&small, 96.0, true).ok()?
        }
        // A video: its first frame (VID-066).
        Err(_) => xs_editor::video::poster_png(first, DRAG_IMAGE)?,
    };
    std::fs::create_dir_all(dir).ok()?;
    let path = dir.join(".drag-image.png");
    std::fs::write(&path, png).ok()?;
    Some(path)
}

/// Prepares the files of a drag: open documents by ID and files by path.
pub fn prepare(core: &Core, docs: &[String], paths: &[String]) -> Result<Value, CommandError> {
    let dir = drag_dir(core);
    let paths: Vec<PathBuf> = paths.iter().map(PathBuf::from).collect();
    let files = xs_editor::files_for_drag(core, docs, &paths, &dir).map_err(|e| {
        tracing::warn!(error = %e, "drag out: files not prepared");
        CommandError::internal(e)
    })?;
    let image = files.first().and_then(|f| drag_image(f, &dir));
    tracing::info!(files = files.len(), "drag out prepared");
    Ok(json!({ "files": files, "image": image }))
}

/// Hands the files to the system's drag and drop, from the main thread, and waits
/// until they are dropped or the drag is cancelled. Returns "dropped" or "cancelled".
pub fn start(
    app: &AppHandle,
    window: WebviewWindow,
    files: Vec<PathBuf>,
    image: Option<PathBuf>,
) -> Result<&'static str, CommandError> {
    if files.is_empty() {
        return Err(CommandError::internal("nothing to drag"));
    }
    // Without a picture of its own, the first file is the picture.
    let picture = drag::Image::File(image.unwrap_or_else(|| files[0].clone()));
    let (tx, rx) = mpsc::channel::<Result<bool, String>>();
    let done = tx.clone();
    app.run_on_main_thread(move || {
        let started = drag::start_drag(
            &window,
            drag::DragItem::Files(files),
            picture,
            move |result, _| {
                let _ = done.send(Ok(matches!(result, drag::DragResult::Dropped)));
            },
            drag::Options::default(),
        );
        if let Err(e) = started {
            let _ = tx.send(Err(e.to_string()));
        }
    })
    .map_err(|e| CommandError::internal(e.to_string()))?;
    match rx.recv() {
        Ok(Ok(true)) => Ok("dropped"),
        Ok(Ok(false)) => Ok("cancelled"),
        Ok(Err(e)) => {
            tracing::warn!(error = %e, "drag out failed");
            Err(CommandError::internal(e))
        }
        Err(_) => Err(CommandError::internal("drag ended without a result")),
    }
}
