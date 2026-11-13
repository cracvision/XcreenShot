//! The receive folder (CAP-043): every new file in a chosen folder becomes a capture
//! once it has finished arriving (its size the same for 2 s). Images as with a file
//! (CAP-040), PDFs with all their pages at 150 dpi (CAP-041). Then the file moves to
//! the `Importados` subfolder, unless the user keeps files where they are. It serves
//! anything that leaves files in a folder: nearby sharing, phone photo apps, synced
//! folders, printing to PDF.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant, SystemTime};

use serde_json::{json, Value};
use xs_core::i18n::tr;
use xs_core::{Context, Core, Source};

use crate::filesource::{kind_of, Kind};

/// How often the folder is looked at.
const TICK: Duration = Duration::from_secs(1);
/// A file is complete when its size has not changed for this long.
const SETTLED: Duration = Duration::from_secs(2);
/// The subfolder imported files move to.
pub const DONE_FOLDER: &str = "Importados";
/// Files already imported and left in place (fingerprints), and the folder they
/// belong to.
const DONE_KEY: &str = "capture.inbox.done";
const BASELINE_KEY: &str = "capture.inbox.baseline";
/// How many fingerprints are kept.
const KEEP: usize = 2000;

/// What makes a file the same file: its name, size and time.
fn fingerprint(path: &Path) -> Option<String> {
    let meta = std::fs::metadata(path).ok()?;
    let modified = meta
        .modified()
        .ok()?
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()?
        .as_secs();
    Some(format!(
        "{}|{}|{}",
        path.file_name()?.to_string_lossy(),
        meta.len(),
        modified
    ))
}

/// The files of the folder that could become captures (not its subfolders).
fn candidates(folder: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut list: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.is_file() && matches!(kind_of(p), Kind::Image | Kind::Pdf))
        .collect();
    list.sort();
    list
}

/// Watches a file until its size stops changing.
#[derive(Default)]
pub struct Settling {
    seen: HashMap<PathBuf, (u64, Instant)>,
}

impl Settling {
    /// Whether `path`, now `size` bytes, has finished arriving at `now`.
    pub fn settled(&mut self, path: &Path, size: u64, now: Instant) -> bool {
        match self.seen.get(path) {
            Some((s, since)) if *s == size => now.duration_since(*since) >= SETTLED,
            _ => {
                self.seen.insert(path.to_path_buf(), (size, now));
                false
            }
        }
    }

    pub fn forget(&mut self, path: &Path) {
        self.seen.remove(path);
    }
}

pub fn install(core: &Arc<Core>) {
    if cfg!(test) {
        return;
    }
    let weak = Arc::downgrade(core);
    let _ = std::thread::Builder::new()
        .name("receive-folder".into())
        .spawn(move || watch(weak));
}

fn done_list(core: &Core) -> Vec<String> {
    core.recall(DONE_KEY)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

fn remember_done(core: &Core, mut list: Vec<String>) {
    if list.len() > KEEP {
        list.drain(..list.len() - KEEP);
    }
    core.remember(DONE_KEY, json!(list));
}

fn watch(weak: Weak<Core>) {
    let mut settling = Settling::default();
    loop {
        std::thread::sleep(TICK);
        let Some(core) = weak.upgrade() else { return };
        let folder = core.settings.get_str("capture.inbox.folder");
        if folder.trim().is_empty() {
            continue;
        }
        let folder = PathBuf::from(folder.trim());
        if !folder.is_dir() {
            continue;
        }
        // A folder just chosen: what is already in it stays (only new files count).
        let baseline = core
            .recall(BASELINE_KEY)
            .and_then(|v| v.as_str().map(str::to_string));
        if baseline.as_deref() != Some(&folder.to_string_lossy()) {
            let mut done = done_list(&core);
            done.extend(candidates(&folder).iter().filter_map(|p| fingerprint(p)));
            remember_done(&core, done);
            core.remember(BASELINE_KEY, json!(folder.to_string_lossy()));
            continue;
        }
        let done = done_list(&core);
        let now = Instant::now();
        for path in candidates(&folder) {
            let Some(print) = fingerprint(&path) else { continue };
            if done.contains(&print) {
                continue;
            }
            let size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);
            if !settling.settled(&path, size, now) {
                continue;
            }
            // While XcreenShot captures, it waits for the next look.
            let Ok(busy) = crate::Busy::take(core.lang()) else {
                break;
            };
            import(&core, &path);
            drop(busy);
            settling.forget(&path);
            let leave = core.settings.get_bool("capture.inbox.leave");
            if !leave && move_done(&folder, &path).is_ok() {
                continue;
            }
            // Left where it is: remembered, so it is imported only once.
            let mut done = done_list(&core);
            done.push(print);
            remember_done(&core, done);
        }
    }
}

/// Moves an imported file to `Importados`, with a number if the name is taken.
fn move_done(folder: &Path, path: &Path) -> std::io::Result<PathBuf> {
    let target_dir = folder.join(DONE_FOLDER);
    std::fs::create_dir_all(&target_dir)?;
    let name = path.file_name().unwrap_or_default();
    let mut target = target_dir.join(name);
    let stem = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    let mut n = 2;
    while target.exists() {
        target = target_dir.join(format!("{stem} ({n}){ext}"));
        n += 1;
    }
    std::fs::rename(path, &target)?;
    Ok(target)
}

/// One file of the receive folder as captures: to the library (with a notification)
/// or into the editor, as the preference says.
fn import(core: &Core, path: &Path) {
    let lang = core.lang();
    let ctx = Context {
        source: Source::Scheduled,
        lang,
    };
    let open = core.settings.get_bool("capture.inbox.open");
    let mut opts = crate::Options::from_params(&json!({}), Source::Ui);
    opts.open_editor = open;
    opts.copy = false;
    opts.cursor = false;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    let result = match kind_of(path) {
        // All pages at 150 dpi, without asking (CAP-043).
        Kind::Pdf => crate::pdf::import(core, &ctx, &opts, &json!({ "path": path, "dpi": 150 })),
        _ => crate::filesource::run(core, &ctx, &opts, &[path.to_path_buf()]),
    };
    match result {
        Ok(r) => {
            tracing::info!(file = %name, captures = r["count"].as_u64().unwrap_or(1), "receive folder: imported")
        }
        Err(e) => {
            tracing::warn!(file = %name, error = %e.message, "receive folder: not imported");
            let password = e.details.as_ref().and_then(|d| d.get("reason")) == Some(&Value::from("password"));
            let body = if password {
                tr(lang, "notify.inbox_password", &[("name", &name)])
            } else {
                tr(lang, "notify.inbox_failed", &[("name", &name)])
            };
            core.host().notify("problems", &tr(lang, "app.name", &[]), &body);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CAP-043: a file is taken once its size has stayed the same for 2 s.
    #[test]
    fn a_file_still_being_copied_waits() {
        let mut s = Settling::default();
        let p = Path::new("big.png");
        let t = Instant::now();
        assert!(!s.settled(p, 100, t));
        assert!(!s.settled(p, 200, t + Duration::from_secs(1)), "it grew");
        assert!(!s.settled(p, 200, t + Duration::from_millis(2500)));
        assert!(s.settled(p, 200, t + Duration::from_millis(3100)));
    }

    #[test]
    fn imported_files_move_to_their_subfolder() {
        let dir = std::env::temp_dir().join(format!("xs-inbox-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        for name in ["a.png", "b.pdf", "notes.txt"] {
            std::fs::write(dir.join(name), b"x").unwrap();
        }
        std::fs::create_dir_all(dir.join(DONE_FOLDER)).unwrap();
        std::fs::write(dir.join(DONE_FOLDER).join("a.png"), b"old").unwrap();
        let found = candidates(&dir);
        assert_eq!(found, vec![dir.join("a.png"), dir.join("b.pdf")]);
        let moved = move_done(&dir, &dir.join("a.png")).unwrap();
        assert_eq!(moved, dir.join(DONE_FOLDER).join("a (2).png"));
        assert!(!dir.join("a.png").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
