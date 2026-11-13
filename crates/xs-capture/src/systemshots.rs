//! Captures from the system's own tool (CAP-042): with Preferences > Capture > "Open
//! system captures in XcreenShot" on, each new capture of the Snipping Tool (also
//! `Win+Shift+S` and `PrtScn` when the system gives that key to it) arrives as one
//! more capture: library, recent tray, and the editor or only the library.
//!
//! Two places are watched, every half second: the clipboard, when the image on it
//! comes from the system tool (images copied in other apps never count), and the
//! system's Screenshots folder, for the files it saves. The same capture seen in both
//! is imported once. Nothing made before the option was on is imported, and while
//! XcreenShot captures the new ones wait their turn. The system's shortcuts are never
//! touched.

use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Weak};
use std::time::{Duration, Instant, SystemTime};

use image::RgbaImage;
use serde_json::json;
use xs_core::{Context, Core, Source};

use crate::{finish, Options, Rect, Shot};

/// How often the clipboard and the folder are looked at.
const TICK: Duration = Duration::from_millis(500);
/// A capture seen on the clipboard and as a file within this time is one capture.
const SAME_WINDOW: Duration = Duration::from_secs(15);
/// Image files the system tool saves.
const IMAGE_FILES: [&str; 3] = ["png", "jpg", "jpeg"];

/// A capture that arrived: its pixels' fingerprint, to know it again.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Print {
    pub width: u32,
    pub height: u32,
    pub hash: u64,
}

impl Print {
    pub fn of(image: &RgbaImage) -> Print {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        image.as_raw().hash(&mut h);
        Print {
            width: image.width(),
            height: image.height(),
            hash: h.finish(),
        }
    }
}

/// The captures imported lately, to import each one once (CAP-042).
#[derive(Default)]
pub struct Recent {
    seen: VecDeque<(Instant, Print)>,
}

impl Recent {
    /// Whether `print` arrived already (within [`SAME_WINDOW`]); if not, it is now.
    pub fn first_time(&mut self, print: Print, now: Instant) -> bool {
        while self
            .seen
            .front()
            .is_some_and(|(t, _)| now.duration_since(*t) > SAME_WINDOW)
        {
            self.seen.pop_front();
        }
        if self.seen.iter().any(|(_, p)| *p == print) {
            return false;
        }
        self.seen.push_back((now, print));
        true
    }
}

/// The process names of the system's capture tool on Windows.
pub fn is_system_tool(process: &str) -> bool {
    let name = Path::new(process)
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    matches!(name.as_str(), "screenclippinghost.exe" | "snippingtool.exe")
}

/// New image files in `folder` modified after `since` and not seen yet.
fn new_files(folder: &Path, since: SystemTime, seen: &mut Vec<PathBuf>) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(folder) else {
        return Vec::new();
    };
    let mut found: Vec<(SystemTime, PathBuf)> = entries
        .filter_map(Result::ok)
        .filter_map(|e| {
            let path = e.path();
            let ext = path.extension()?.to_string_lossy().to_lowercase();
            if !IMAGE_FILES.contains(&ext.as_str()) {
                return None;
            }
            let modified = e.metadata().ok()?.modified().ok()?;
            (modified > since && !seen.contains(&path)).then_some((modified, path))
        })
        .collect();
    found.sort();
    found
        .into_iter()
        .map(|(_, p)| {
            seen.push(p.clone());
            p
        })
        .collect()
}

/// The folder where the system tool saves its captures. A test instance can point
/// it elsewhere with `XCREENSHOT_SCREENSHOTS_DIR`.
fn screenshots_folder() -> Option<PathBuf> {
    std::env::var_os("XCREENSHOT_SCREENSHOTS_DIR")
        .map(PathBuf::from)
        .or_else(imp::screenshots_folder)
}

/// Something that arrived from the system tool.
enum Arrival {
    Clipboard(RgbaImage),
    File(PathBuf),
}

/// Starts watching, for as long as the app runs (from the extension). It does
/// nothing while the option is off.
pub fn install(core: &Arc<Core>) {
    if cfg!(test) || !cfg!(windows) {
        return;
    }
    let weak = Arc::downgrade(core);
    let _ = std::thread::Builder::new()
        .name("system-captures".into())
        .spawn(move || watch(weak));
}

fn watch(weak: Weak<Core>) {
    let mut on_since: Option<SystemTime> = None;
    let mut clip_seq = 0u32;
    let mut seen_files: Vec<PathBuf> = Vec::new();
    let mut recent = Recent::default();
    let mut waiting: VecDeque<Arrival> = VecDeque::new();
    loop {
        std::thread::sleep(TICK);
        let Some(core) = weak.upgrade() else { return };
        if !core.settings.get_bool("capture.systemCaptures") {
            on_since = None;
            waiting.clear();
            continue;
        }
        // Only what comes after the option was turned on (CAP-042).
        let since = *on_since.get_or_insert_with(|| {
            clip_seq = imp::clipboard_sequence();
            seen_files.clear();
            SystemTime::now()
        });
        let seq = imp::clipboard_sequence();
        if seq != clip_seq {
            clip_seq = seq;
            if imp::clipboard_owner().is_some_and(|p| is_system_tool(&p)) {
                if let Some(image) = read_clipboard_image() {
                    waiting.push_back(Arrival::Clipboard(image));
                }
            }
        }
        if let Some(folder) = screenshots_folder() {
            for path in new_files(&folder, since, &mut seen_files) {
                waiting.push_back(Arrival::File(path));
            }
        }
        // One at a time, and not while XcreenShot captures (then they wait).
        while let Some(next) = waiting.pop_front() {
            match import(&core, next, &mut recent) {
                Ok(()) => {}
                Err(back) => {
                    waiting.push_front(back);
                    break;
                }
            }
        }
    }
}

fn read_clipboard_image() -> Option<RgbaImage> {
    let _lock = xs_core::clipboard_lock();
    let img = arboard::Clipboard::new().ok()?.get_image().ok()?;
    RgbaImage::from_raw(img.width as u32, img.height as u32, img.bytes.into_owned())
}

/// Reads a file the tool is still writing: a few tries.
fn read_file(path: &Path) -> Option<RgbaImage> {
    for _ in 0..10 {
        if let Ok(image) = std::fs::read(path)
            .map_err(|e| e.to_string())
            .and_then(|b| xs_image::encode::decode(&b))
        {
            return Some(image);
        }
        std::thread::sleep(Duration::from_millis(200));
    }
    None
}

/// Imports one arrival; gives it back if a capture is under way.
fn import(core: &Core, arrival: Arrival, recent: &mut Recent) -> Result<(), Arrival> {
    let Ok(_busy) = crate::Busy::take(core.lang()) else {
        return Err(arrival);
    };
    let keep = core.settings.get_bool("capture.systemCaptures.keepOriginal");
    let (image, file) = match arrival {
        Arrival::Clipboard(image) => (image, None),
        Arrival::File(path) => match read_file(&path) {
            Some(image) => (image, Some(path)),
            None => {
                tracing::warn!(path = %path.display(), "system capture: the file could not be read");
                return Ok(());
            }
        },
    };
    let first = recent.first_time(Print::of(&image), Instant::now());
    // Without Keep the original file, the file goes to the library: it leaves its
    // folder, also when the clipboard brought the same capture first.
    let remove_file = |f: &Path| {
        if !keep {
            if let Err(e) = std::fs::remove_file(f) {
                tracing::warn!(error = %e, "system capture: the original file stays");
            }
        }
    };
    if !first {
        if let Some(f) = &file {
            remove_file(f);
        }
        return Ok(());
    }
    let ctx = Context {
        source: Source::Scheduled,
        lang: core.lang(),
    };
    let mut opts = Options::from_params(&json!({}), Source::Ui);
    opts.copy = false;
    opts.cursor = false;
    opts.open_editor = core.settings.get_str("capture.systemCaptures.open") != "library";
    opts.meta = Some(json!({
        "systemTool": {
            "file": file.as_ref().map(|f| f.to_string_lossy().into_owned()),
            "fileDate": file
                .as_ref()
                .and_then(|f| std::fs::metadata(f).ok()?.modified().ok())
                .map(|t| chrono::DateTime::<chrono::Local>::from(t).to_rfc3339()),
        }
    }));
    let shot = Shot {
        rect: Rect {
            x: 0,
            y: 0,
            width: image.width(),
            height: image.height(),
        },
        image,
        mode: "system",
        scale: 1.0,
        monitor: String::new(),
        app: None,
        window_title: None,
        window_id: None,
        pointer: None,
        warnings: Vec::new(),
    };
    match finish(core, &ctx, shot, &opts) {
        Ok(_) => {
            tracing::info!(from_file = file.is_some(), "system capture imported");
            if let Some(f) = &file {
                remove_file(f);
            }
        }
        Err(e) => tracing::warn!(error = %e.message, "system capture: not imported"),
    }
    Ok(())
}

#[cfg(windows)]
mod imp {
    use std::path::PathBuf;
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::DataExchange::{GetClipboardOwner, GetClipboardSequenceNumber};
    use windows_sys::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::GetWindowThreadProcessId;

    pub fn clipboard_sequence() -> u32 {
        // SAFETY: a plain query.
        unsafe { GetClipboardSequenceNumber() }
    }

    /// The executable of the process that put the clipboard's content there.
    pub fn clipboard_owner() -> Option<String> {
        // SAFETY: plain queries; a closed window or process yields nothing.
        unsafe {
            let hwnd = GetClipboardOwner();
            if hwnd.is_null() {
                return None;
            }
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, &mut pid);
            if pid == 0 {
                return None;
            }
            let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if process.is_null() {
                return None;
            }
            let mut buf = [0u16; 1024];
            let mut len = buf.len() as u32;
            let ok = QueryFullProcessImageNameW(process, PROCESS_NAME_WIN32, buf.as_mut_ptr(), &mut len);
            CloseHandle(process);
            (ok != 0).then(|| String::from_utf16_lossy(&buf[..len as usize]))
        }
    }

    /// The system's Screenshots folder (Pictures\Screenshots unless moved).
    pub fn screenshots_folder() -> Option<PathBuf> {
        use windows_sys::Win32::System::Com::CoTaskMemFree;
        use windows_sys::Win32::UI::Shell::{FOLDERID_Screenshots, SHGetKnownFolderPath, KF_FLAG_DEFAULT};
        // SAFETY: the path is freed with CoTaskMemFree, as the API asks.
        unsafe {
            let mut raw: windows_sys::core::PWSTR = std::ptr::null_mut();
            let hr = SHGetKnownFolderPath(
                &FOLDERID_Screenshots,
                KF_FLAG_DEFAULT as _,
                std::ptr::null_mut(),
                &mut raw,
            );
            if hr != 0 || raw.is_null() {
                return None;
            }
            let mut n = 0;
            while *raw.add(n) != 0 {
                n += 1;
            }
            let path = String::from_utf16_lossy(std::slice::from_raw_parts(raw, n));
            CoTaskMemFree(raw as _);
            Some(PathBuf::from(path))
        }
    }
}

#[cfg(not(windows))]
mod imp {
    use std::path::PathBuf;

    pub fn clipboard_sequence() -> u32 {
        0
    }

    pub fn clipboard_owner() -> Option<String> {
        None
    }

    pub fn screenshots_folder() -> Option<PathBuf> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_system_tool_counts() {
        assert!(is_system_tool(r"C:\Windows\SystemApps\ScreenClippingHost.exe"));
        assert!(is_system_tool(
            r"C:\Program Files\WindowsApps\Microsoft.ScreenSketch\SnippingTool\SnippingTool.exe"
        ));
        assert!(!is_system_tool(r"C:\Program Files\Mozilla Firefox\firefox.exe"));
        assert!(!is_system_tool(""));
    }

    /// CAP-042: the same capture on the clipboard and as a file is imported once.
    #[test]
    fn the_same_capture_arrives_once() {
        let a = RgbaImage::from_pixel(20, 10, image::Rgba([1, 2, 3, 255]));
        let b = RgbaImage::from_pixel(20, 10, image::Rgba([3, 2, 1, 255]));
        let mut recent = Recent::default();
        let t = Instant::now();
        assert!(recent.first_time(Print::of(&a), t));
        assert!(!recent.first_time(Print::of(&a), t + Duration::from_secs(2)));
        assert!(recent.first_time(Print::of(&b), t + Duration::from_secs(3)));
        // Much later, the same picture is a new capture.
        assert!(recent.first_time(Print::of(&a), t + Duration::from_secs(60)));
    }

    #[test]
    fn only_new_image_files() {
        let dir = std::env::temp_dir().join(format!("xs-sys-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let old = dir.join("old.png");
        std::fs::write(&old, b"x").unwrap();
        std::thread::sleep(Duration::from_millis(30));
        let since = SystemTime::now();
        std::thread::sleep(Duration::from_millis(30));
        std::fs::write(dir.join("new.png"), b"x").unwrap();
        std::fs::write(dir.join("notes.txt"), b"x").unwrap();
        let mut seen = Vec::new();
        let found = new_files(&dir, since, &mut seen);
        assert_eq!(found, vec![dir.join("new.png")]);
        assert!(new_files(&dir, since, &mut seen).is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
