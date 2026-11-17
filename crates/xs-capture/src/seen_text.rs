//! The text of library images, read for the search (LIB-008): in the background, on
//! this computer, with the system's recognition, newest first. It goes slowly and
//! steps aside: paused while capturing, recording or recording steps, and on a low
//! battery; its thread runs at the lowest priority, so a capture is never delayed
//! (APP-048, APP-049). Images and guides, not videos. Switched off
//! (`library.ocrIndex`), the index forgets what was read.

use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Weak};
use std::time::Duration;

use xs_core::Core;

/// Something else needs the computer now: a capture, a recording, steps.
fn busy(core: &Core) -> bool {
    crate::capturing()
        || crate::record::running(core)
        || crate::steps_rec::running(core)
        || crate::scroll::running(core)
}

/// On battery and below 20 %.
#[cfg(windows)]
fn low_battery() -> bool {
    use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};
    // SAFETY: a plain struct filled by the system.
    unsafe {
        let mut s: SYSTEM_POWER_STATUS = std::mem::zeroed();
        GetSystemPowerStatus(&mut s) != 0 && s.ACLineStatus == 0 && s.BatteryLifePercent < 20
    }
}

#[cfg(not(windows))]
fn low_battery() -> bool {
    false
}

#[cfg(windows)]
fn lowest_priority() {
    use windows_sys::Win32::System::Threading::{
        GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_LOWEST,
    };
    // SAFETY: this thread's own priority.
    unsafe {
        SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_LOWEST);
    }
}

#[cfg(not(windows))]
fn lowest_priority() {}

/// The picture of a library file as it shows: a document rendered, an image decoded.
pub fn picture(path: &Path, library: &Path) -> Option<image::RgbaImage> {
    let is_doc = path
        .extension()
        .is_some_and(|e| e.to_string_lossy().eq_ignore_ascii_case("xshot"));
    if is_doc {
        let doc = xs_editor::store::DocStore::load(path, library).ok()?;
        return xs_editor::store::DocStore::render_doc(&doc).ok();
    }
    image::open(path).ok().map(|i| i.to_rgba8())
}

/// The text of a picture, line by line.
pub fn text_of(picture: &image::RgbaImage, languages: &[String]) -> Result<String, xs_ocr::OcrError> {
    let r = xs_ocr::recognize(picture, languages, &AtomicBool::new(false))?;
    let lines: Vec<String> = r
        .blocks
        .iter()
        .flat_map(|b| b.lines.iter().map(|l| l.text.clone()))
        .collect();
    Ok(lines.join("\n"))
}

/// Reads the next image waiting, if any. Returns whether there was one.
pub fn step(core: &Core) -> Result<bool, String> {
    let lib = xs_library::library(core);
    let Some(item) = lib.unseen(1).into_iter().next() else {
        return Ok(false);
    };
    let abs = lib
        .root()
        .join(item.path.replace('/', std::path::MAIN_SEPARATOR_STR));
    let text = match picture(&abs, lib.root()) {
        Some(p) => {
            let languages = crate::extract::wanted(core, None);
            match text_of(&p, &languages) {
                Ok(t) => t,
                // Nothing can be read on this computer: try again later.
                Err(xs_ocr::OcrError::NoLanguage) | Err(xs_ocr::OcrError::NotSupported) => {
                    return Err("no recognition language".into())
                }
                Err(_) => String::new(),
            }
        }
        // A file that cannot be read now: nothing to find in it.
        None => String::new(),
    };
    lib.set_seen(&item, &text);
    // Counts only, never the text.
    tracing::debug!(chars = text.chars().count(), "library text read");
    Ok(true)
}

/// Starts the reader. It lives as long as the core.
pub fn start(weak: Weak<Core>) {
    let _ = std::thread::Builder::new()
        .name("library-text".into())
        .spawn(move || {
            lowest_priority();
            // Let the app start first.
            std::thread::sleep(Duration::from_secs(20));
            loop {
                let wait = {
                    let Some(core): Option<Arc<Core>> = weak.upgrade() else {
                        return;
                    };
                    if !core.settings.get_bool("library.ocrIndex") {
                        let lib = xs_library::library(&core);
                        if lib.seen_counts().0 > 0 {
                            lib.clear_seen();
                        }
                        Duration::from_secs(10)
                    } else if busy(&core) {
                        Duration::from_secs(3)
                    } else if low_battery() {
                        Duration::from_secs(60)
                    } else {
                        match step(&core) {
                            Ok(true) => Duration::from_millis(400),
                            Ok(false) => Duration::from_secs(10),
                            Err(_) => Duration::from_secs(120),
                        }
                    }
                };
                std::thread::sleep(wait);
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use xs_core::Source;

    /// LIB-008: a capture with a generic name that shows "Error 404" is found by "404"
    /// once read, marked "in the image"; switched off, it is not.
    #[test]
    fn the_text_in_images_is_searched() {
        if xs_ocr::languages().is_empty() {
            eprintln!("no recognition language on this computer: skipped");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path().join("lib");
        std::fs::create_dir_all(&root).unwrap();
        let core = Core::for_tests_with(
            xs_core::platform::Platform::Windows,
            vec![
                xs_library::extension_with(Some(root.clone())),
                crate::extension(),
                xs_editor::extension(),
            ],
        );
        xs_editor::editor(&core).set_folders(root.clone(), dir.path().join("rec"));
        let image = crate::redact::tests::form(&["Error 404", "Página no encontrada"]);
        let file = root.join("Captura 2026-11-14.png");
        image.save(&file).unwrap();
        let call = |cmd: &str, p: serde_json::Value| {
            core.call(Source::Cli, cmd, p, Some(xs_core::Lang::Es)).unwrap()
        };
        call("library.rebuild", json!({}));
        let search = |q: &str| call("library.list", json!({"query": q}));
        assert_eq!(search("404")["count"], 0, "not read yet");
        assert_eq!(call("library.seen_status", json!({}))["waiting"], 1);
        while step(&core).unwrap() {}
        let r = search("404");
        assert_eq!(r["count"], 1, "{r}");
        assert_eq!(r["items"][0]["inImage"], true, "{r}");
        assert_eq!(
            search("captura")["items"][0].get("inImage"),
            None,
            "found by its name"
        );
        // Kept in the sidecar: a rebuild does not read it again.
        call("library.rebuild", json!({}));
        assert_eq!(call("library.seen_status", json!({}))["waiting"], 0);
        assert_eq!(search("404")["count"], 1);
        // Switched off: not found.
        call("settings.set", json!({"key": "library.ocrIndex", "value": false}));
        assert_eq!(search("404")["count"], 0);
    }
}
