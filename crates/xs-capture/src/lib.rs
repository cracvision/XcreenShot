//! Screen and window capture (CAP-). This crate contributes the handlers of the
//! `capture.*` commands declared in `xs_core::catalog` (CAP-008).
//!
//! Build 1 of M1 covers the non-interactive captures: full screen (CAP-028, CAP-021),
//! region by rectangle (CAP-025), window by target (CAP-027), the capture route
//! (CAP-054: save, clipboard, file, notify) and capture metadata (CAP-056). The
//! on-screen selection arrives in build 2.

pub mod compose;
pub mod platform;

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Weak};
use std::time::Duration;

use image::RgbaImage;
use serde::Serialize;
use serde_json::{json, Value};
use xs_core::core::Extension;
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode, Lang, Source};

pub use compose::Rect;

// ---------------------------------------------------------------------------
// Monitors

#[derive(Debug, Clone, Serialize)]
pub struct MonitorInfo {
    pub id: u32,
    pub name: String,
    /// Physical pixels on the virtual desktop.
    pub rect: Rect,
    pub scale: f32,
    pub primary: bool,
}

fn failed(lang: Lang, reason: impl std::fmt::Display) -> CommandError {
    CommandError::new(
        ErrorCode::Internal,
        tr(lang, "errors.capture_failed", &[("reason", &reason.to_string())]),
    )
}

fn monitor_info(m: &xcap::Monitor) -> Option<MonitorInfo> {
    let friendly = m.friendly_name().unwrap_or_default();
    Some(MonitorInfo {
        id: m.id().ok()?,
        name: if friendly.trim().is_empty() {
            m.name().ok()?
        } else {
            friendly
        },
        rect: Rect {
            x: m.x().ok()?,
            y: m.y().ok()?,
            width: m.width().ok()?,
            height: m.height().ok()?,
        },
        scale: m.scale_factor().unwrap_or(1.0),
        primary: m.is_primary().unwrap_or(false),
    })
}

pub fn monitors() -> Result<Vec<(xcap::Monitor, MonitorInfo)>, xcap::XCapError> {
    Ok(xcap::Monitor::all()?
        .into_iter()
        .filter_map(|m| monitor_info(&m).map(|i| (m, i)))
        .collect())
}

// ---------------------------------------------------------------------------
// Captures

/// Result of taking the pixels, before the capture route.
pub struct Shot {
    pub image: RgbaImage,
    pub rect: Rect,
    pub mode: &'static str,
    pub scale: f32,
    pub monitor: String,
    pub app: Option<platform::AppInfo>,
    pub window_title: Option<String>,
    pub warnings: Vec<String>,
}

fn grab(list: &[(xcap::Monitor, MonitorInfo)], area: Rect) -> Result<RgbaImage, xcap::XCapError> {
    let mut pieces = Vec::new();
    for (m, info) in list {
        if let Some(part) = info.rect.intersect(&area) {
            let local_x = (part.x - info.rect.x) as u32;
            let local_y = (part.y - info.rect.y) as u32;
            let image = m.capture_region(local_x, local_y, part.width, part.height)?;
            pieces.push(compose::Piece { rect: part, image });
        }
    }
    Ok(compose::compose(area, &pieces))
}

/// Visible application windows, excluding XcreenShot's own, minimized windows and
/// overlays (CAP-016, CAP-056), top of the stack first.
fn app_windows() -> Vec<xcap::Window> {
    let own = std::process::id();
    let mut list: Vec<xcap::Window> = xcap::Window::all()
        .unwrap_or_default()
        .into_iter()
        .filter(|w| {
            w.pid().ok() != Some(own)
                && !w.is_minimized().unwrap_or(true)
                && w.id().is_ok_and(platform::is_regular_window)
                && w.width().unwrap_or(0) > 1
                && w.height().unwrap_or(0) > 1
        })
        .collect();
    list.sort_by_key(|w| w.z().unwrap_or(i32::MAX));
    list
}

/// Window that covers most of `area`, for "Aplicación" in region captures (CAP-056).
fn dominant_window(area: Rect) -> Option<xcap::Window> {
    app_windows()
        .into_iter()
        .filter_map(|w| {
            let r = Rect {
                x: w.x().ok()?,
                y: w.y().ok()?,
                width: w.width().ok()?,
                height: w.height().ok()?,
            };
            Some((r.intersect(&area)?.area(), w))
        })
        .max_by_key(|(a, _)| *a)
        .map(|(_, w)| w)
}

fn window_app(w: &xcap::Window) -> platform::AppInfo {
    platform::app_info(w.pid().unwrap_or(0), &w.app_name().unwrap_or_default())
}

/// The active window among `windows` (top of the stack first): the focused one, else
/// a window of the foreground process, else the topmost app window. The focus can sit
/// on something that is not an app window (XcreenShot itself, the desktop, a console
/// with no window when called from a script), and the user still means the window
/// in front (CAP-027, CAP-056).
fn active_index(windows: &[xcap::Window]) -> Option<usize> {
    let pid = platform::foreground_pid();
    let same_pid = |w: &xcap::Window| pid.is_some() && w.pid().ok() == pid;
    windows
        .iter()
        .position(|w| same_pid(w) && w.is_focused().unwrap_or(false))
        .or_else(|| windows.iter().position(same_pid))
        .or_else(|| (!windows.is_empty()).then_some(0))
}

/// The app in the foreground for full screen captures (CAP-056).
fn foreground_app() -> (Option<platform::AppInfo>, Option<String>) {
    let windows = app_windows();
    let window = active_index(&windows).map(|i| &windows[i]);
    match window {
        Some(w) => (Some(window_app(w)), w.title().ok()),
        None => (None, None),
    }
}

/// Full screen of one monitor, the primary one, or all (CAP-028, CAP-021).
pub fn capture_fullscreen(which: &str, lang: Lang) -> Result<Shot, CommandError> {
    let list = monitors().map_err(|e| failed(lang, e))?;
    let mut warnings = Vec::new();
    let pointer = platform::cursor_position();
    let pick =
        |pred: &dyn Fn(&MonitorInfo) -> bool| list.iter().find(|(_, i)| pred(i)).map(|(_, i)| i.clone());
    let primary = pick(&|i| i.primary).or_else(|| list.first().map(|(_, i)| i.clone()));
    let (area, scale, name) = match which {
        "all" => {
            let rects: Vec<Rect> = list.iter().map(|(_, i)| i.rect).collect();
            let area = Rect::bounding(&rects).ok_or_else(|| failed(lang, "no monitors"))?;
            let scale = list.iter().map(|(_, i)| i.scale).fold(1.0f32, f32::max);
            (area, scale, "all".to_string())
        }
        other => {
            let chosen = match other {
                "" | "pointer" => pointer
                    .and_then(|(x, y)| pick(&|i| i.rect.contains(x, y)))
                    .or_else(|| primary.clone()),
                "primary" => primary.clone(),
                name => match pick(&|i| i.name.eq_ignore_ascii_case(name)) {
                    Some(m) => Some(m),
                    None => {
                        warnings.push(tr(lang, "errors.monitor_not_found", &[("name", name)]));
                        primary.clone()
                    }
                },
            }
            .ok_or_else(|| failed(lang, "no monitors"))?;
            (chosen.rect, chosen.scale, chosen.name)
        }
    };
    let image = grab(&list, area).map_err(|e| failed(lang, e))?;
    let (app, window_title) = foreground_app();
    Ok(Shot {
        image,
        rect: area,
        mode: "fullscreen",
        scale,
        monitor: name,
        app,
        window_title,
        warnings,
    })
}

/// A rectangle of the desktop in physical pixels (CAP-025, CAP-008).
pub fn capture_region(area: Rect, lang: Lang) -> Result<Shot, CommandError> {
    let list = monitors().map_err(|e| failed(lang, e))?;
    let hit: Vec<&MonitorInfo> = list
        .iter()
        .map(|(_, i)| i)
        .filter(|i| i.rect.intersect(&area).is_some())
        .collect();
    if hit.is_empty() {
        return Err(CommandError::new(
            ErrorCode::InvalidParams,
            tr(lang, "errors.region_outside", &[]),
        ));
    }
    // The monitor holding most of the region names it; the highest density sets the scale.
    let main = hit
        .iter()
        .max_by_key(|i| i.rect.intersect(&area).map(|r| r.area()).unwrap_or(0))
        .copied()
        .cloned();
    let scale = hit.iter().map(|i| i.scale).fold(1.0f32, f32::max);
    let image = grab(&list, area).map_err(|e| failed(lang, e))?;
    let window = dominant_window(area);
    Ok(Shot {
        image,
        rect: area,
        mode: "region",
        scale,
        monitor: main.map(|m| m.name).unwrap_or_default(),
        app: window.as_ref().map(window_app),
        window_title: window.and_then(|w| w.title().ok()),
        warnings: Vec::new(),
    })
}

pub enum WindowTarget {
    Active,
    Point(i32, i32),
    Title(String),
}

/// A whole window, even if another one covers it (CAP-027).
pub fn capture_window(target: &WindowTarget, lang: Lang) -> Result<Shot, CommandError> {
    let windows = app_windows();
    let rect_of = |w: &xcap::Window| -> Option<Rect> {
        Some(Rect {
            x: w.x().ok()?,
            y: w.y().ok()?,
            width: w.width().ok()?,
            height: w.height().ok()?,
        })
    };
    let window = match target {
        WindowTarget::Active => {
            let index = active_index(&windows).ok_or_else(|| {
                CommandError::new(ErrorCode::NotFound, tr(lang, "errors.no_active_window", &[]))
            })?;
            windows.into_iter().nth(index).expect("index in range")
        }
        WindowTarget::Point(x, y) => windows
            .into_iter()
            .filter(|w| rect_of(w).is_some_and(|r| r.contains(*x, *y)))
            .min_by_key(|w| w.z().unwrap_or(i32::MAX))
            .ok_or_else(|| {
                CommandError::new(
                    ErrorCode::NotFound,
                    tr(lang, "errors.window_not_found", &[("title", &format!("{x},{y}"))]),
                )
            })?,
        WindowTarget::Title(needle) => {
            let n = needle.to_lowercase();
            windows
                .into_iter()
                .find(|w| {
                    w.title().unwrap_or_default().to_lowercase().contains(&n)
                        || w.app_name().unwrap_or_default().to_lowercase().contains(&n)
                })
                .ok_or_else(|| {
                    CommandError::new(
                        ErrorCode::NotFound,
                        tr(lang, "errors.window_not_found", &[("title", needle)]),
                    )
                })?
        }
    };
    let rect = rect_of(&window).ok_or_else(|| failed(lang, "window has no size"))?;
    let image = window.capture_image().map_err(|e| failed(lang, e))?;
    let monitor = window.current_monitor().ok();
    Ok(Shot {
        rect: Rect {
            width: image.width(),
            height: image.height(),
            ..rect
        },
        image,
        mode: "window",
        scale: monitor
            .as_ref()
            .and_then(|m| m.scale_factor().ok())
            .unwrap_or(1.0),
        monitor: monitor
            .and_then(|m| monitor_info(&m))
            .map(|i| i.name)
            .unwrap_or_default(),
        app: Some(window_app(&window)),
        window_title: window.title().ok(),
        warnings: Vec::new(),
    })
}

// ---------------------------------------------------------------------------
// Capture route (CAP-054)

/// Options shared by every capture command (CAP-008).
#[derive(Debug, Clone)]
pub struct Options {
    pub cursor: bool,
    pub copy: bool,
    pub open_editor: bool,
    pub out: Option<PathBuf>,
    pub delay: f64,
}

impl Options {
    /// Panel defaults (CAP-003); automation never opens windows unless asked (APP-030).
    pub fn from_params(p: &Value, source: Source) -> Options {
        Options {
            cursor: p.get("cursor").and_then(Value::as_bool).unwrap_or(false),
            copy: p.get("copy").and_then(Value::as_bool).unwrap_or(true),
            open_editor: match p.get("open").and_then(Value::as_str) {
                Some("editor") => true,
                Some(_) => false,
                None => !source.is_automation(),
            },
            out: p.get("out").and_then(Value::as_str).map(PathBuf::from),
            delay: p.get("delay").and_then(Value::as_f64).unwrap_or(0.0),
        }
    }
}

/// Where captures are kept until the library (LIB-020) stores `.xshot` documents:
/// `Documents\XcreenShot` on Windows, `~/Pictures/XcreenShot` on macOS (LIB-027).
pub fn library_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("XCREENSHOT_LIBRARY_DIR") {
        return PathBuf::from(dir);
    }
    let dirs = directories::UserDirs::new();
    let base = if cfg!(target_os = "macos") {
        dirs.as_ref().and_then(|d| d.picture_dir().map(Path::to_path_buf))
    } else {
        dirs.as_ref()
            .and_then(|d| d.document_dir().map(Path::to_path_buf))
    };
    base.unwrap_or_else(std::env::temp_dir).join("XcreenShot")
}

/// `XcreenShot_2026-10-12_14-05-33.png`, with ` (2)`… if it exists (CAP-053 default).
pub fn unique_name(dir: &Path, now: chrono::DateTime<chrono::Local>) -> PathBuf {
    let stem = format!("XcreenShot_{}", now.format("%Y-%m-%d_%H-%M-%S"));
    let mut path = dir.join(format!("{stem}.png"));
    let mut n = 2;
    while path.exists() {
        path = dir.join(format!("{stem} ({n}).png"));
        n += 1;
    }
    path
}

/// Capture metadata (CAP-056). Field names are the same on every platform.
pub fn metadata(core: &Core, shot: &Shot, now: chrono::DateTime<chrono::Local>) -> Value {
    let (user, machine) = platform::user_and_machine();
    let with_people = core.settings.get_bool("capture.metadata.userAndMachine");
    let with_title = core.settings.get_bool("capture.metadata.windowTitle");
    json!({
        "capturedAt": now.to_rfc3339(),
        "mode": shot.mode,
        "source": "screen",
        "app": shot.app,
        "window": { "title": if with_title { shot.window_title.clone() } else { None } },
        "url": null,
        "os": platform::os_info(),
        "user": if with_people { Some(user) } else { None },
        "machine": if with_people { Some(machine) } else { None },
        "display": {
            "monitor": shot.monitor,
            "scale": shot.scale,
            "width": shot.image.width(),
            "height": shot.image.height(),
            "hdr": false,
        },
        "xcreenshotVersion": xs_core::platform::VERSION,
    })
}

/// Small PNG preview for the UI (at most 640 px wide).
fn thumbnail_png(image: &RgbaImage) -> Option<Vec<u8>> {
    let w = image.width().min(640);
    let h = ((image.height() as u64 * w as u64) / image.width().max(1) as u64).max(1) as u32;
    let small = image::imageops::thumbnail(image, w, h);
    let mut out = std::io::Cursor::new(Vec::new());
    small.write_to(&mut out, image::ImageFormat::Png).ok()?;
    Some(out.into_inner())
}

fn copy_to_clipboard(image: &RgbaImage) -> Result<(), arboard::Error> {
    let data = arboard::ImageData {
        width: image.width() as usize,
        height: image.height() as usize,
        bytes: std::borrow::Cow::Borrowed(image.as_raw()),
    };
    // Another app may hold the clipboard: retry for up to 1 s (CAP-046).
    let mut last = None;
    for _ in 0..10 {
        match arboard::Clipboard::new().and_then(|mut c| c.set_image(data.clone())) {
            Ok(()) => return Ok(()),
            Err(e) => last = Some(e),
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Err(last.unwrap_or(arboard::Error::ClipboardOccupied))
}

/// Runs the capture route: save, clipboard, extra file, notify, open (CAP-054).
pub fn finish(core: &Core, ctx: &Context, shot: Shot, opts: &Options) -> CommandResult {
    let lang = ctx.lang;
    let now = chrono::Local::now();
    let host = core.host();
    let mut warnings = shot.warnings.clone();
    if opts.cursor {
        // The pointer becomes an editable object of the document (CAP-047), which
        // needs the editor's document model (EDT-, next M1 build).
        warnings.push("cursor: arrives with the editor".to_string());
    }

    // 1-2. Keep it safe first.
    let dir = library_dir();
    std::fs::create_dir_all(&dir).map_err(|e| failed(lang, e))?;
    let path = unique_name(&dir, now);
    shot.image.save(&path).map_err(|e| failed(lang, e))?;
    let mut files = vec![path.to_string_lossy().into_owned()];
    let meta = metadata(core, &shot, now);

    // 3. Clipboard.
    let mut copied = false;
    if opts.copy {
        match copy_to_clipboard(&shot.image) {
            Ok(()) => copied = true,
            Err(e) => {
                tracing::warn!(error = %e, "clipboard copy failed");
                host.notify(
                    "problems",
                    &tr(lang, "app.name", &[]),
                    &tr(lang, "errors.clipboard_failed", &[]),
                );
            }
        }
    }

    // Extra file requested by the command (`--out`).
    if let Some(out) = &opts.out {
        if let Some(parent) = out.parent().filter(|p| !p.as_os_str().is_empty()) {
            std::fs::create_dir_all(parent).map_err(|e| failed(lang, e))?;
        }
        std::fs::copy(&path, out).map_err(|e| failed(lang, e))?;
        files.push(out.to_string_lossy().into_owned());
    }

    tracing::info!(
        mode = shot.mode,
        width = shot.image.width(),
        height = shot.image.height(),
        source = ?ctx.source,
        "capture completed"
    );
    let result = json!({
        "status": "completed",
        "path": files.last(),
        "files": files,
        "width": shot.image.width(),
        "height": shot.image.height(),
        "rect": shot.rect,
        "copied": copied,
        "metadata": meta,
        "warnings": warnings,
    });

    // 4-6. Open, confirm.
    host.capture_finished(&result, thumbnail_png(&shot.image).as_deref(), opts.open_editor);
    if ctx.source.is_automation() && core.settings.get_bool("automation.notifyOnCapture") {
        let source = tr(lang, &format!("source.{}", source_key(ctx.source)), &[]);
        host.notify(
            "automation",
            &tr(lang, "app.name", &[]),
            &tr(lang, "notify.automation_capture", &[("source", &source)]),
        );
    } else if !opts.open_editor {
        let body = if copied {
            tr(lang, "notify.capture_copied", &[])
        } else {
            tr(
                lang,
                "notify.capture_saved",
                &[("folder", &dir.to_string_lossy())],
            )
        };
        host.notify("captures", &tr(lang, "app.name", &[]), &body);
    }
    Ok(result)
}

fn source_key(s: Source) -> &'static str {
    match s {
        Source::Ui => "ui",
        Source::Shortcut => "shortcut",
        Source::Cli => "cli",
        Source::Mcp => "mcp",
        Source::Scheduled => "scheduled",
    }
}

// ---------------------------------------------------------------------------
// Commands

/// One capture at a time: others answer `busy` at once (CAP-008).
static CAPTURING: AtomicBool = AtomicBool::new(false);

struct Busy;

impl Busy {
    fn take(lang: Lang) -> Result<Busy, CommandError> {
        if CAPTURING.swap(true, Ordering::SeqCst) {
            return Err(
                CommandError::new(ErrorCode::Busy, tr(lang, "errors.capture_busy", &[]))
                    .with_details(json!({ "status": "busy" })),
            );
        }
        Ok(Busy)
    }
}

impl Drop for Busy {
    fn drop(&mut self) {
        CAPTURING.store(false, Ordering::SeqCst);
    }
}

fn run_capture(
    weak: &Weak<Core>,
    ctx: &Context,
    p: &Value,
    take: impl FnOnce(Lang) -> Result<Shot, CommandError>,
) -> CommandResult {
    let core: Arc<Core> = weak
        .upgrade()
        .ok_or_else(|| CommandError::internal("shutting down"))?;
    let _busy = Busy::take(ctx.lang)?;
    let opts = Options::from_params(p, ctx.source);
    if opts.delay > 0.0 {
        std::thread::sleep(Duration::from_secs_f64(opts.delay));
    }
    // CAP-023: keep XcreenShot's own windows out of the image.
    let hide = core.settings.get_bool("capture.hideOnCapture");
    if hide {
        core.host().hide_for_capture(true);
    }
    let shot = take(ctx.lang);
    if hide {
        core.host().hide_for_capture(false);
    }
    finish(&core, ctx, shot?, &opts)
}

fn interactive_later(lang: Lang) -> CommandError {
    CommandError::new(
        ErrorCode::InvalidParams,
        tr(lang, "errors.capture_interactive_later", &[]),
    )
}

/// Handlers for the `capture.*` commands, for `Core::with_extensions`.
pub fn extension() -> Extension {
    Box::new(|reg, weak| {
        reg.handle("capture.monitors", move |ctx, _p| {
            let list = monitors().map_err(|e| failed(ctx.lang, e))?;
            Ok(
                serde_json::to_value(list.into_iter().map(|(_, i)| i).collect::<Vec<_>>())
                    .unwrap_or_default(),
            )
        });
        let w = weak.clone();
        reg.handle("capture.fullscreen", move |ctx, p| {
            let which = p
                .get("monitor")
                .and_then(Value::as_str)
                .unwrap_or("pointer")
                .to_string();
            run_capture(&w, ctx, &p, |lang| capture_fullscreen(&which, lang))
        });
        let w = weak.clone();
        reg.handle("capture.region", move |ctx, p| {
            let Some(r) = p.get("rect") else {
                return Err(interactive_later(ctx.lang));
            };
            let area = Rect {
                x: r["x"].as_i64().unwrap_or(0) as i32,
                y: r["y"].as_i64().unwrap_or(0) as i32,
                width: r["width"].as_u64().unwrap_or(1) as u32,
                height: r["height"].as_u64().unwrap_or(1) as u32,
            };
            run_capture(&w, ctx, &p, |lang| capture_region(area, lang))
        });
        let w = weak;
        reg.handle("capture.window", move |ctx, p| {
            let target = match p.get("target").and_then(Value::as_str).unwrap_or("active") {
                "active" => WindowTarget::Active,
                "point" => WindowTarget::Point(
                    p.get("x").and_then(Value::as_i64).unwrap_or(0) as i32,
                    p.get("y").and_then(Value::as_i64).unwrap_or(0) as i32,
                ),
                "title" => WindowTarget::Title(
                    p.get("title")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                ),
                _ => return Err(interactive_later(ctx.lang)),
            };
            run_capture(&w, ctx, &p, |lang| capture_window(&target, lang))
        });
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use xs_core::platform::Platform;

    #[test]
    fn extension_handles_every_capture_command() {
        let core = Core::for_tests_with(Platform::current(), vec![extension()]);
        let reg = core.registry();
        for spec in reg.specs().iter().filter(|s| s.area() == "capture") {
            assert!(reg.has_handler(spec.id), "{} has no handler", spec.id);
        }
    }

    #[test]
    fn options_defaults_follow_the_source() {
        let ui = Options::from_params(&json!({}), Source::Ui);
        assert!(ui.copy && ui.open_editor && !ui.cursor);
        let cli = Options::from_params(&json!({}), Source::Cli);
        assert!(
            cli.copy && !cli.open_editor,
            "automation never opens windows by default"
        );
        let cli = Options::from_params(&json!({"open": "editor", "copy": false}), Source::Cli);
        assert!(!cli.copy && cli.open_editor);
    }

    #[test]
    fn unique_names_do_not_overwrite() {
        let dir = tempfile::tempdir().unwrap();
        let now = chrono::Local::now();
        let first = unique_name(dir.path(), now);
        std::fs::write(&first, b"x").unwrap();
        let second = unique_name(dir.path(), now);
        assert_ne!(first, second);
        assert!(second.to_string_lossy().ends_with(" (2).png"));
        assert!(first
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("XcreenShot_"));
    }

    #[test]
    fn interactive_capture_is_not_available_yet() {
        let core = Core::for_tests_with(Platform::current(), vec![extension()]);
        let e = core
            .call(Source::Cli, "capture.region", json!({}), Some(Lang::En))
            .unwrap_err();
        assert_eq!(e.code, ErrorCode::InvalidParams);
    }

    #[test]
    fn metadata_has_the_spec_fields() {
        let core = Core::for_tests(Platform::current());
        let shot = Shot {
            image: RgbaImage::new(3, 2),
            rect: Rect {
                x: 0,
                y: 0,
                width: 3,
                height: 2,
            },
            mode: "region",
            scale: 2.0,
            monitor: "Test".into(),
            app: Some(platform::AppInfo {
                name: "Notepad".into(),
                version: Some("1.0".into()),
                id: None,
            }),
            window_title: Some("notes.txt".into()),
            warnings: vec![],
        };
        let m = metadata(&core, &shot, chrono::Local::now());
        for key in [
            "capturedAt",
            "mode",
            "source",
            "app",
            "window",
            "os",
            "user",
            "machine",
            "display",
            "xcreenshotVersion",
        ] {
            assert!(m.get(key).is_some(), "{key}");
        }
        assert_eq!(m["display"]["scale"], 2.0);
        assert!(m["capturedAt"].as_str().unwrap().contains('T'));
        core.call(
            Source::Ui,
            "settings.set",
            json!({"key": "capture.metadata.windowTitle", "value": false}),
            None,
        )
        .unwrap();
        assert!(metadata(&core, &shot, chrono::Local::now())["window"]["title"].is_null());
    }

    /// Real screen capture; run with `cargo test -p xs-capture -- --ignored`.
    #[test]
    #[ignore]
    fn captures_the_primary_monitor_at_native_size() {
        let shot = capture_fullscreen("primary", Lang::En).unwrap();
        let list = monitors().unwrap();
        let primary = list.iter().find(|(_, i)| i.primary).unwrap();
        assert_eq!(
            (shot.image.width(), shot.image.height()),
            (primary.1.rect.width, primary.1.rect.height)
        );
    }
}
