//! Screen and window capture (CAP-). This crate contributes the handlers of the
//! `capture.*` commands declared in `xs_core::catalog` (CAP-008).
//!
//! Build 1 of M1 covers the non-interactive captures: full screen (CAP-028, CAP-021),
//! region by rectangle (CAP-025), window by target (CAP-027), the capture route
//! (CAP-054: save, clipboard, file, notify) and capture metadata (CAP-056). The
//! on-screen selection arrives in build 2.

pub mod areas;
pub mod asktext;
pub mod clipsource;
pub mod compose;
pub mod extract;
pub mod feedback;
pub mod filedest;
pub mod filename;
pub mod filesource;
pub mod freehand;
pub mod inbox;
pub mod interactive;
pub mod interval;
pub mod menus;
pub mod multi;
#[cfg(windows)]
mod multi_win;
pub mod panel;
pub mod pdf;
pub mod pending;
pub mod picker;
pub mod platform;
pub mod pointer;
pub mod preset_cmds;
pub mod presets;
pub mod record;
pub mod schedule;
pub mod scroll;
#[cfg(windows)]
mod scroll_win;
pub mod stitch;
pub mod svgsource;
pub mod systemshots;
pub mod wallpaper;
pub mod webcap;

use std::path::PathBuf;
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
    /// The window it shows, for "Repeat last area" (CAP-007).
    pub window_id: Option<u32>,
    /// The pointer when the image was taken, with "Capture cursor" (CAP-047).
    pub pointer: Option<pointer::ScreenPointer>,
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
        window_id: None,
        pointer: None,
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
        window_id: None,
        pointer: None,
        warnings: Vec::new(),
    })
}

pub enum WindowTarget {
    Active,
    Point(i32, i32),
    Title(String),
    /// Window picked in the on-screen selection.
    Id(u32),
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
        WindowTarget::Id(id) => windows
            .into_iter()
            .find(|w| w.id().ok() == Some(*id))
            .ok_or_else(|| {
                CommandError::new(
                    ErrorCode::NotFound,
                    tr(lang, "errors.window_not_found", &[("title", &id.to_string())]),
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
        window_id: window.id().ok(),
        pointer: None,
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
    /// Extract text instead of keeping an image (SMT-005).
    pub extract: bool,
    /// Extract text: show its results window (default: the setting, or no for
    /// automation).
    pub show: Option<bool>,
    /// The File destination, when Share includes it (CAP-052, CAP-053).
    pub file: Option<filedest::FileDest>,
    /// Effects for every capture (CAP-051), the editor's entries.
    pub effects: Vec<Value>,
    /// Objects the capture's document starts with (text from the clipboard, CAP-039).
    pub objects: Vec<Value>,
    /// Open in = Quick annotation (CAP-057): mark the area on screen before it leaves.
    /// Where there is no selection on screen it opens in the editor.
    pub quick: bool,
    /// Saved in the library (true) or kept unsaved (false), whatever the setting
    /// (LIB-020); `None` follows the setting.
    pub keep: Option<bool>,
    /// The interval series this capture belongs to (CAP-049).
    pub series: Option<String>,
    /// The schedule that made this capture: id, planned time, seconds late (CAP-050).
    pub schedule: Option<Value>,
    /// More metadata for the capture (a system capture's file, CAP-042).
    pub meta: Option<Value>,
    /// Each visible window as its own object over the desktop (CAP-029).
    pub windows_objects: bool,
}

impl Options {
    /// Panel defaults (CAP-003); automation never opens windows unless asked (APP-030).
    pub fn from_params(p: &Value, source: Source) -> Options {
        Options {
            cursor: p.get("cursor").and_then(Value::as_bool).unwrap_or(false),
            copy: p.get("copy").and_then(Value::as_bool).unwrap_or(true),
            open_editor: match p.get("open").and_then(Value::as_str) {
                Some("editor") | Some("quick") => true,
                Some(_) => false,
                None => !source.is_automation(),
            },
            out: p.get("out").and_then(Value::as_str).map(PathBuf::from),
            delay: p.get("delay").and_then(Value::as_f64).unwrap_or(0.0),
            extract: p.get("extract").and_then(Value::as_bool).unwrap_or(false),
            show: p.get("show").and_then(Value::as_bool),
            file: p
                .get("share")
                .and_then(Value::as_array)
                .is_some_and(|s| s.iter().any(|d| d == "file"))
                .then(|| {
                    p.get("file")
                        .and_then(|f| serde_json::from_value::<filedest::FileDest>(f.clone()).ok())
                        .filter(|f| f.validate().is_ok())
                        .unwrap_or_default()
                }),
            effects: p
                .get("effects")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter(|e| xs_image::fx::check(e).is_ok())
                        .cloned()
                        .collect()
                })
                .unwrap_or_default(),
            objects: Vec::new(),
            quick: p.get("open").and_then(Value::as_str) == Some("quick"),
            keep: None,
            series: p.get("series").and_then(Value::as_str).map(str::to_string),
            schedule: p.get("schedule").filter(|v| v.is_object()).cloned(),
            meta: None,
            windows_objects: p.get("windows").and_then(Value::as_bool) == Some(true),
        }
    }
}

/// Capture metadata (CAP-056). Field names are the same on every platform.
pub fn metadata(core: &Core, shot: &Shot, now: chrono::DateTime<chrono::Local>) -> Value {
    let (user, machine) = platform::user_and_machine();
    let with_people = core.settings.get_bool("capture.metadata.userAndMachine");
    let with_title = core.settings.get_bool("capture.metadata.windowTitle");
    json!({
        "capturedAt": now.to_rfc3339(),
        "mode": shot.mode,
        "source": match shot.mode {
            "clipboard" => "clipboard",
            // The system's capture tool (CAP-042).
            "system" => "system",
            // A file (CAP-040) or a PDF's page (CAP-041).
            "file" => "file",
            "pdf" => "pdf",
            // A web page by its address (CAP-037).
            "webpage" => "webpage",
            _ => "screen",
        },
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

/// Whether every pixel is black: what Windows returns for protected content and for
/// windows that exclude themselves from captures (CAP-024, APP-026).
pub fn is_blank(image: &RgbaImage) -> bool {
    image.width() * image.height() > 0 && image.pixels().all(|p| p.0[0] | p.0[1] | p.0[2] == 0)
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
    let _lock = xs_core::clipboard_lock();
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
    // A series (CAP-049): a capture just like the last one kept is dropped.
    let series = opts.series.as_deref().and_then(|id| interval::find(core, id));
    let mut series_index = None;
    if let Some(s) = &series {
        match s.accept(&shot.image) {
            Some(n) => series_index = Some(n),
            None => {
                tracing::info!(series = %s.id, "interval: unchanged, not kept");
                return Ok(json!({ "status": "skipped", "reason": "unchanged", "series": s.id }));
            }
        }
    }
    let now = chrono::Local::now();
    let started = std::time::Instant::now();
    let host = core.host();
    let mut warnings = shot.warnings.clone();
    // The moment of the capture: its sound and flash (CAP-055).
    let sound = core.settings.get_str("capture.sound");
    if sound != "none" {
        feedback::play(&sound);
    }
    if core.settings.get_bool("capture.flash") {
        feedback::flash(shot.rect);
    }
    // Extract text (SMT-005): the zone's text, not an image; the image is kept in the
    // library only when asked.
    if opts.extract {
        if core.settings.get_bool("smart.extractText.saveImage") {
            let meta = metadata(core, &shot, now);
            let app = shot.app.as_ref().map(|a| a.name.clone());
            if let Err(e) =
                xs_editor::save_capture(core, &shot.image, meta, shot.scale as f64, app.as_deref())
            {
                tracing::warn!(error = %e, "extract text: the image was not kept");
            }
        }
        return extract::deliver(core, ctx, shot.image, opts.copy, opts.show, Some(shot.rect));
    }
    // The pointer, if asked for and inside the image (CAP-047): a floating image in
    // the document, and laid over the image that leaves without the editor.
    let pointer = if opts.cursor {
        if cfg!(not(windows)) {
            warnings.push("cursor: not available on this system yet".to_string());
        }
        let area = Rect {
            width: shot.image.width(),
            height: shot.image.height(),
            ..shot.rect
        };
        shot.pointer.as_ref().and_then(|p| p.within(area))
    } else {
        None
    };
    // All black: the system kept protected content out (CAP-024, APP-026). The
    // capture finishes anyway, and says so.
    let blank = is_blank(&shot.image);
    if blank {
        warnings.push("blank: the system returned only black pixels (protected content)".to_string());
        host.notify(
            "problems",
            &tr(lang, "app.name", &[]),
            &tr(lang, "notify.capture_blank", &[]),
        );
    }
    let flat = pointer.as_ref().map(|p| pointer::flatten(&shot.image, p));
    let outgoing = flat.as_ref().unwrap_or(&shot.image);

    // 1-2. Keep it safe first: a `.xshot` in the library, open for the editor (LIB-020).
    let mut meta = metadata(core, &shot, now);
    // The series and its order, so the library can group them (CAP-049, CAP-056).
    if let (Some(s), Some(n)) = (&series, series_index) {
        meta["series"] = json!({ "id": s.group(), "index": n, "name": s.title(lang) });
    }
    if let Some(extra) = opts.meta.as_ref().and_then(Value::as_object) {
        for (k, v) in extra {
            meta[k] = v.clone();
        }
    }
    // A scheduled capture: when it was planned and how late it was made (CAP-050).
    if let Some(when) = &opts.schedule {
        meta["schedule"] = json!({
            "id": when["id"],
            "plannedAt": when["at"],
            "lateSeconds": when["late"],
        });
    }
    let app = shot.app.as_ref().map(|a| a.name.as_str());
    let keep = opts
        .keep
        .unwrap_or_else(|| core.settings.get_bool("library.autosaveImages"));
    // Windows as objects (CAP-029): the document's base is the desktop without the app
    // windows, which become objects; what leaves (clipboard, files) is the screen as
    // it was seen.
    let as_objects = if opts.windows_objects {
        object_windows(shot.rect)
    } else {
        Vec::new()
    };
    let desktop = if opts.windows_objects {
        desktop_base(&shot.image, shot.rect, &as_objects)
    } else {
        None
    };
    let (document_id, path) = xs_editor::add_capture_kept(
        core,
        desktop.as_ref().unwrap_or(&shot.image),
        meta.clone(),
        shot.scale as f64,
        app,
        pointer.as_ref(),
        keep,
    )
    .map_err(|e| failed(lang, e))?;
    let save_ms = started.elapsed().as_millis() as u64;
    let mut files = vec![path.to_string_lossy().into_owned()];

    if opts.windows_objects {
        windows_as_objects(core, lang, &document_id, shot.rect, as_objects);
    }

    // Objects the document starts with (CAP-039) and effects on capture (CAP-051):
    // editable in the document, already applied in what leaves for the clipboard and
    // files.
    let with_effects = if opts.effects.is_empty() && opts.objects.is_empty() {
        None
    } else {
        // Capture details may ask for its text now (CAP-051); series and schedules
        // do not ask.
        let effects = asktext::fill(
            core,
            &opts.effects,
            opts.series.is_none() && opts.schedule.is_none(),
        );
        apply_effects(core, &document_id, &opts.objects, &effects)
    };
    let outgoing = with_effects.as_ref().unwrap_or(outgoing);

    // 3. Clipboard.
    let mut copied = false;
    if opts.copy {
        match copy_to_clipboard(outgoing) {
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
        let dpi = (96.0 * shot.scale as f64).round();
        let png = xs_image::encode::png(outgoing, dpi, false).map_err(|e| failed(lang, e))?;
        std::fs::write(out, png).map_err(|e| failed(lang, e))?;
        files.push(out.to_string_lossy().into_owned());
    }

    // The File destination (CAP-052, CAP-053): also written to its folder, named by its
    // pattern. If the folder cannot be used, the capture stays in the library.
    if let Some(dest) = &opts.file {
        let values = filename::Values {
            prefix: dest.prefix.clone(),
            app: shot.app.as_ref().map(|a| a.name.clone()),
            title: shot.window_title.clone(),
            site: None,
            mode: shot.mode.to_string(),
            width: outgoing.width(),
            height: outgoing.height(),
            user: platform::user_and_machine().0,
        };
        let dpi = (96.0 * shot.scale as f64).round();
        let host_for_dialog = core.host();
        let choose = move |p: &std::path::Path| host_for_dialog.pick_save(p);
        // Into the editor: nothing is sent until Send (CAP-052). Series and schedules
        // send at once.
        let later = opts.open_editor && opts.series.is_none() && opts.schedule.is_none();
        if later {
            pending::wait(
                core,
                &document_id,
                pending::Waiting {
                    file: dest.clone(),
                    values,
                    dpi,
                    meta: meta.clone(),
                },
            );
        } else {
            match filedest::write(core, dest, outgoing, &path, &values, dpi, Some(&meta), &choose) {
                Ok(written) => files.push(written.to_string_lossy().into_owned()),
                // Cancelled in the Save dialog: the capture stays in the library only.
                Err(e) if e == "cancelled" => {}
                Err(e) => {
                    tracing::warn!(error = %e, "file destination: not written");
                    warnings.push(format!("file: {e}"));
                    host.notify(
                        "problems",
                        &tr(lang, "app.name", &[]),
                        &tr(
                            lang,
                            "notify.file_destination_failed",
                            &[("folder", &dest.folder().to_string_lossy())],
                        ),
                    );
                }
            }
        }
    }

    tracing::info!(
        mode = shot.mode,
        width = shot.image.width(),
        height = shot.image.height(),
        source = ?ctx.source,
        save_ms,
        total_ms = started.elapsed().as_millis() as u64,
        "capture completed"
    );
    // "Repeat last area" (CAP-007): the rectangle of the last region or window.
    if matches!(shot.mode, "region" | "universal" | "window") {
        core.remember(
            LAST_AREA_KEY,
            json!({ "rect": shot.rect, "windowId": shot.window_id }),
        );
    }
    let result = json!({
        "status": "completed",
        "documentId": document_id,
        "path": files.last(),
        "files": files,
        "width": outgoing.width(),
        "height": outgoing.height(),
        "rect": shot.rect,
        "copied": copied,
        "metadata": meta,
        "warnings": warnings,
    });

    // 4-6. Open, confirm.
    host.capture_finished(&result, thumbnail_png(outgoing).as_deref(), opts.open_editor);
    if ctx.source.is_automation() && core.settings.get_bool("automation.notifyOnCapture") {
        let source = tr(lang, &format!("source.{}", source_key(ctx.source)), &[]);
        host.notify(
            "automation",
            &tr(lang, "app.name", &[]),
            &tr(lang, "notify.automation_capture", &[("source", &source)]),
        );
    } else if !opts.open_editor && series.is_none() {
        // A series says so in the icon's menu and the panel, not once per capture.
        let body = if copied {
            tr(lang, "notify.capture_copied", &[])
        } else if opts.out.is_none() && path.starts_with(xs_editor::editor(core).library()) {
            tr(lang, "notify.capture_saved_library", &[])
        } else {
            let file = opts.out.as_ref().unwrap_or(&path);
            let folder = file
                .parent()
                .map(|d| d.to_string_lossy().into_owned())
                .unwrap_or_default();
            tr(lang, "notify.capture_saved", &[("folder", &folder)])
        };
        // A click opens it in the editor; Show in folder, its file (CAP-055).
        host.notify_opening(
            "captures",
            &tr(lang, "app.name", &[]),
            &body,
            json!({ "documentId": document_id, "path": path }),
        );
    }
    Ok(result)
}

/// Windows as objects (CAP-029): each visible window over `area` (not minimized, not
/// on another virtual desktop), taken whole even where another covers it, becomes an
/// image object named after its app, in its place and stacking order. The base stays
/// the screen as it looked, so the flattened image is the same as without it.
/// The windows that become objects (CAP-029): visible app windows over `area`, from
/// the topmost down, with their place and app name.
fn object_windows(area: Rect) -> Vec<(u32, Rect, String)> {
    app_windows()
        .into_iter()
        .filter_map(|w| {
            let id = w.id().ok()?;
            let r = Rect {
                x: w.x().ok()?,
                y: w.y().ok()?,
                width: w.width().ok()?,
                height: w.height().ok()?,
            };
            (r.intersect(&area).is_some() && !platform::is_cloaked(id) && platform::is_app_window(id))
                .then(|| (id, r, window_app(&w).name))
        })
        .collect()
}

/// The screen as seen with the desktop's background where the app windows were
/// (CAP-029): taskbar, icons left uncovered and other non-window things stay. `None`
/// when the background cannot be known: the base is then the screen as seen.
fn desktop_base(screen: &RgbaImage, area: Rect, windows: &[(u32, Rect, String)]) -> Option<RgbaImage> {
    if windows.is_empty() {
        return None;
    }
    let background = wallpaper::desktop(area)?;
    let mut base = screen.clone();
    for (_, r, _) in windows {
        let Some(part) = r.intersect(&area) else {
            continue;
        };
        let piece = image::imageops::crop_imm(
            &background,
            (part.x - area.x) as u32,
            (part.y - area.y) as u32,
            part.width,
            part.height,
        )
        .to_image();
        image::imageops::replace(
            &mut base,
            &piece,
            (part.x - area.x) as i64,
            (part.y - area.y) as i64,
        );
    }
    Some(base)
}

fn windows_as_objects(
    core: &Core,
    lang: Lang,
    document_id: &str,
    area: Rect,
    windows: Vec<(u32, Rect, String)>,
) {
    let tmp = core.paths.local_dir.join("tmp");
    let _ = std::fs::create_dir_all(&tmp);
    let mut placed = 0;
    // The topmost comes first: insert from the bottom up.
    for (id, _, app) in windows.into_iter().rev() {
        let Ok(shot) = capture_window(&WindowTarget::Id(id), lang) else {
            continue;
        };
        let file = tmp.join(format!("window-{id}.png"));
        let Ok(png) = xs_image::encode::png(&shot.image, 0.0, false) else {
            continue;
        };
        if std::fs::write(&file, png).is_err() {
            continue;
        }
        let x = (shot.rect.x - area.x) as f64;
        let y = (shot.rect.y - area.y) as f64;
        let center = [
            x + shot.image.width() as f64 / 2.0,
            y + shot.image.height() as f64 / 2.0,
        ];
        let inserted = core.call(
            Source::Ui,
            "document.insert_images",
            json!({ "documentId": document_id, "paths": [file], "center": center }),
            None,
        );
        let _ = std::fs::remove_file(&file);
        match inserted {
            Ok(r) => {
                placed += 1;
                if let Some(object) = r["ids"].get(0).and_then(Value::as_str) {
                    let _ = core.call(
                        Source::Ui,
                        "document.update_object",
                        json!({ "documentId": document_id, "objectId": object, "set": { "name": app } }),
                        None,
                    );
                }
            }
            Err(e) => tracing::warn!(error = %e.message, "windows as objects: a window was left out"),
        }
    }
    let _ = core.call(
        Source::Ui,
        "document.save",
        json!({ "documentId": document_id }),
        None,
    );
    tracing::info!(windows = placed, "windows as objects");
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

pub(crate) fn cancelled(lang: Lang) -> CommandError {
    CommandError::new(ErrorCode::Cancelled, tr(lang, "errors.capture_cancelled", &[]))
        .with_details(json!({ "status": "cancelled" }))
}

/// The delay countdown (CAP-048); `Esc` or Cancel ends the capture.
fn wait_delay(core: &Core, opts: &Options, lang: Lang) -> Result<(), CommandError> {
    if opts.delay > 0.0 && !core.host().countdown(opts.delay) {
        return Err(cancelled(lang));
    }
    Ok(())
}

fn run_capture(
    core: &Core,
    ctx: &Context,
    p: &Value,
    take: impl FnOnce(Lang) -> Result<Shot, CommandError>,
) -> CommandResult {
    let opts = Options::from_params(p, ctx.source);
    wait_delay(core, &opts, ctx.lang)?;
    // CAP-023: keep XcreenShot's own windows out of the image.
    let hide = core.settings.get_bool("capture.hideOnCapture");
    if hide {
        core.host().hide_for_capture(true);
    }
    let shot = take(ctx.lang);
    // Without a selection, the pointer of the moment the image is taken (CAP-047).
    let pointer = opts.cursor.then(pointer::grab).flatten();
    if hide {
        core.host().hide_for_capture(false);
    }
    let mut shot = shot?;
    shot.pointer = pointer;
    finish(core, ctx, shot, &opts)
}

/// Interactive capture (CAP-009, CAP-025, CAP-027), after the delay.
fn run_interactive(core: &Core, ctx: &Context, p: &Value, mode: &str, extra: Value) -> CommandResult {
    let opts = Options::from_params(p, ctx.source);
    wait_delay(core, &opts, ctx.lang)?;
    interactive::run(core, ctx, &opts, mode, extra)
}

const LAST_KEY: &str = "capture.last";
const LAST_AREA_KEY: &str = "capture.lastArea";

/// Runs one capture command with resolved parameters, one at a time, and remembers
/// it for "Repeat last capture" (CAP-007). The capture panel steps aside meanwhile
/// (CAP-001, CAP-023).
pub(crate) fn dispatch(core: &Core, ctx: &Context, command: &str, p: &Value, extra: Value) -> CommandResult {
    // Multi-capture holds the mouse: finish it first (CAP-067).
    if multi::capturing(core) {
        return Err(
            CommandError::new(ErrorCode::Busy, tr(ctx.lang, "errors.multicapture_busy", &[]))
                .with_details(json!({ "status": "busy" })),
        );
    }
    let _busy = Busy::take(ctx.lang)?;
    core.remember(
        LAST_KEY,
        json!({ "command": command, "params": p, "extra": extra }),
    );
    let step_aside = core.settings.get_bool("capture.hideOnCapture");
    if step_aside {
        core.host().capture_session(true);
    }
    let result = run_command(core, ctx, command, p, extra);
    if step_aside {
        core.host().capture_session(false);
    }
    result
}

fn run_command(core: &Core, ctx: &Context, command: &str, p: &Value, extra: Value) -> CommandResult {
    match command {
        "capture.universal" => run_interactive(core, ctx, p, "universal", extra),
        "capture.fullscreen" => {
            let which = p
                .get("monitor")
                .and_then(Value::as_str)
                .unwrap_or("pointer")
                .to_string();
            // Windows as objects (CAP-029) is done while finishing: `windows` in `p`.
            run_capture(core, ctx, p, |lang| capture_fullscreen(&which, lang))
        }
        "capture.region" => match p.get("rect") {
            None => {
                // Freehand (CAP-032): a shape instead of a rectangle.
                if let Some(shape) = p.get("freehand").and_then(Value::as_str) {
                    let extra = panel::with_overrides(extra, &json!({ "freehand": shape }));
                    return run_interactive(core, ctx, p, "region", extra);
                }
                // Multiple areas (CAP-033).
                if let Some(multiple) = p.get("multiple") {
                    let extra = panel::with_overrides(extra, &json!({ "multiple": multiple }));
                    return run_interactive(core, ctx, p, "multi", extra);
                }
                // A fixed size follows the pointer (CAP-026).
                let extra = match p.get("size") {
                    Some(size) => panel::with_overrides(extra, &json!({ "fixedSize": size })),
                    None => extra,
                };
                run_interactive(core, ctx, p, "region", extra)
            }
            Some(r) => {
                let area = desktop_rect(r, ctx.lang)?;
                run_capture(core, ctx, p, |lang| capture_region(area, lang))
            }
        },
        "capture.window" => {
            let target = match p.get("target").and_then(Value::as_str).unwrap_or("active") {
                "active" => WindowTarget::Active,
                "point" => WindowTarget::Point(
                    p.get("x").and_then(Value::as_i64).unwrap_or(0) as i32,
                    p.get("y").and_then(Value::as_i64).unwrap_or(0) as i32,
                ),
                // Internal: a window still open from "Repeat last area" (CAP-007).
                "id" => WindowTarget::Id(p.get("id").and_then(Value::as_u64).unwrap_or(0) as u32),
                "title" => WindowTarget::Title(
                    p.get("title")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string(),
                ),
                _ => return run_interactive(core, ctx, p, "window", extra),
            };
            run_capture(core, ctx, p, |lang| capture_window(&target, lang))
        }
        // A file as the source (CAP-040): the files given, the fixed one, or chosen now.
        "capture.file" => {
            let opts = Options::from_params(p, ctx.source);
            let mut paths: Vec<std::path::PathBuf> = p
                .get("paths")
                .and_then(Value::as_array)
                .map(|a| {
                    a.iter()
                        .filter_map(Value::as_str)
                        .map(std::path::PathBuf::from)
                        .collect()
                })
                .unwrap_or_default();
            if paths.is_empty() {
                let mut all = filesource::IMAGE_FILES
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>();
                all.extend(["pdf".to_string(), "xshot".to_string()]);
                paths = core
                    .host()
                    .pick_files(true, &[(tr(ctx.lang, "ui.panel.file_filter", &[]), all)])
                    .filter(|f| !f.is_empty())
                    .ok_or_else(|| cancelled(ctx.lang))?;
            }
            filesource::run(core, ctx, &opts, &paths)
        }
        // A web page by its address (CAP-037).
        "capture.webpage" => {
            let opts = Options::from_params(p, ctx.source);
            webcap::run(core, ctx, &opts, p)
        }
        // Quick annotation (CAP-057): select as in Universal, mark, copy.
        "capture.quick_annotate" => {
            let mut p = p.clone();
            p["open"] = json!("quick");
            // Copy is the bar's button; Open in the editor does not copy too.
            if p.get("copy").is_none() {
                p["copy"] = json!(false);
            }
            run_interactive(core, ctx, &p, "region", extra)
        }
        // The clipboard as a source (CAP-039).
        "capture.clipboard" => {
            let opts = Options::from_params(p, ctx.source);
            wait_delay(core, &opts, ctx.lang)?;
            clipsource::run(core, ctx, &opts)
        }
        "capture.window_picker" => {
            let opts = Options::from_params(p, ctx.source);
            wait_delay(core, &opts, ctx.lang)?;
            picker::run(core, ctx, &opts)
        }
        // Menu capture (CAP-031): after the delay (from the panel, at least the time to
        // open the menu), the open menus are found on the frozen screens.
        "capture.menu" => {
            let opts = Options::from_params(p, ctx.source);
            wait_delay(core, &opts, ctx.lang)?;
            let extra = panel::with_overrides(
                extra,
                &json!({ "menu": {
                    "submenus": p.get("submenus").and_then(Value::as_bool).unwrap_or(true),
                    "menubar": p.get("menubar").and_then(Value::as_bool).unwrap_or(false),
                } }),
            );
            interactive::run(core, ctx, &opts, "menu", extra)
        }
        "capture.scrolling" => {
            let opts = Options::from_params(p, ctx.source);
            wait_delay(core, &opts, ctx.lang)?;
            let kind = scroll_kind(core, p);
            match p.get("rect").and_then(interactive::rect_of) {
                Some(area) => {
                    let place = interactive::place_of(core, area);
                    scroll::run(core, ctx, &opts, area, kind, place)
                }
                None => {
                    let extra = panel::with_overrides(extra, &json!({ "scroll": p }));
                    interactive::run(core, ctx, &opts, "scrolling", extra)
                }
            }
        }
        other => Err(CommandError::internal(format!("not a capture command: {other}"))),
    }
}

/// A rectangle of the desktop from `{x, y, width, height}`, or from the top left of
/// `monitor` when it names one (CAP-026, CAP-008). A missing monitor, or a rectangle that
/// does not fit in it, is an error: never a partial capture.
pub(crate) fn desktop_rect(r: &Value, lang: Lang) -> Result<Rect, CommandError> {
    let mut area = Rect {
        x: r["x"].as_i64().unwrap_or(0) as i32,
        y: r["y"].as_i64().unwrap_or(0) as i32,
        width: r["width"].as_u64().unwrap_or(1).max(1) as u32,
        height: r["height"].as_u64().unwrap_or(1).max(1) as u32,
    };
    let Some(name) = r.get("monitor").and_then(Value::as_str).filter(|n| !n.is_empty()) else {
        return Ok(area);
    };
    let list = monitors().map_err(|e| failed(lang, e))?;
    let infos: Vec<MonitorInfo> = list.into_iter().map(|(_, i)| i).collect();
    let Some(m) = infos.iter().find(|m| m.name == name) else {
        return Err(fixed_region_problem(lang, name));
    };
    area.x += m.rect.x;
    area.y += m.rect.y;
    if area.intersect(&m.rect) != Some(area) {
        return Err(fixed_region_problem(lang, name));
    }
    Ok(area)
}

fn fixed_region_problem(lang: Lang, monitor: &str) -> CommandError {
    CommandError::new(
        ErrorCode::InvalidParams,
        tr(lang, "errors.fixed_region", &[("monitor", monitor)]),
    )
    .with_details(json!({ "status": "failed", "monitor": monitor }))
}

/// Puts the effects on the new capture's document and saves it; returns the image as
/// it leaves the editor (CAP-051). None if something failed: the capture goes on as it
/// is.
fn apply_effects(core: &Core, document_id: &str, objects: &[Value], effects: &[Value]) -> Option<RgbaImage> {
    if !objects.is_empty() {
        let added = core.call(
            Source::Ui,
            "document.annotate",
            json!({ "documentId": document_id, "objects": objects }),
            None,
        );
        if let Err(e) = added {
            tracing::warn!(error = %e.message, "capture: its objects were not added");
            return None;
        }
    }
    let set = if effects.is_empty() {
        Ok(json!({}))
    } else {
        core.call(
            Source::Ui,
            "document.effects",
            json!({ "documentId": document_id, "action": "set", "effects": effects }),
            None,
        )
    };
    if let Err(e) = set {
        tracing::warn!(error = %e.message, "effects on capture: not applied");
        return None;
    }
    if let Err(e) = core.call(
        Source::Ui,
        "document.save",
        json!({ "documentId": document_id }),
        None,
    ) {
        tracing::warn!(error = %e.message, "effects on capture: not saved");
    }
    xs_editor::editor(core)
        .store
        .snapshot(document_id)
        .ok()
        .map(|snap| snap.render())
}

/// Manual or automatic, from `capture.scrolling`'s parameters (CAP-034, CAP-035).
pub(crate) fn scroll_kind(core: &Core, p: &Value) -> scroll::Kind {
    let tab = panel::load(core).image.scroll;
    let text = |key: &str, default: &str| p.get(key).and_then(Value::as_str).unwrap_or(default).to_string();
    if text("type", &tab.kind) != "auto" {
        return scroll::Kind::Manual;
    }
    let direction = match text("direction", &tab.direction).as_str() {
        "horizontal" => stitch::Direction::Horizontal,
        "both" => stitch::Direction::Both,
        _ => stitch::Direction::Vertical,
    };
    let wait = p
        .get("wait")
        .and_then(Value::as_f64)
        .unwrap_or(tab.wait)
        .clamp(0.0, 5.0);
    scroll::Kind::Auto(scroll::Auto {
        direction,
        from_start: text("startFrom", &tab.start_from) != "current",
        wait: std::time::Duration::from_millis((wait * 1000.0) as u64),
        restore: p.get("restore").and_then(Value::as_bool).unwrap_or(tab.restore),
        fixed: p.get("fixed").and_then(Value::as_bool).unwrap_or(tab.fixed),
    })
}

/// "Repeat last area" (CAP-007): the same screen rectangle at once, without the
/// crosshair. A window that is still open is taken where it is now. Options come
/// from the Image tab when started from the app, like the other single-mode actions.
fn repeat_last_area(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let Some(last) = core.recall(LAST_AREA_KEY).filter(|l| l["rect"].is_object()) else {
        // Nothing to repeat yet: open the capture panel.
        let _ = core.host().show_capture_panel(Some(true));
        return Ok(json!({ "status": "none", "panel": true }));
    };
    let window = last["windowId"].as_u64().filter(|id| {
        app_windows()
            .iter()
            .any(|w| w.id().ok().map(u64::from) == Some(*id))
    });
    let (command, target) = match window {
        Some(id) => ("capture.window", json!({ "target": "id", "id": id })),
        None => ("capture.region", json!({ "rect": last["rect"] })),
    };
    let options = if ctx.source.is_automation() {
        p.clone()
    } else {
        panel::with_overrides(panel::load(core).image_options(), p)
    };
    dispatch(
        core,
        ctx,
        command,
        &panel::with_overrides(options, &target),
        json!({}),
    )
}

/// "Capture" with a tab's settings (CAP-006, CAP-065); explicit parameters win.
fn capture_with_tab(core: &Core, ctx: &Context, tab: &str, p: &Value) -> CommandResult {
    let settings = panel::load(core);
    let (command, base, extra) = settings.plan(tab);
    let params = panel::with_overrides(base, p);
    // Interval (CAP-049): the panel's Start; again, Stop.
    if command == "capture.interval" {
        if let Some(id) = interval::of_preset(core, PANEL_SERIES) {
            return interval::stop(core, ctx.lang, Some(&id));
        }
        return start_interval(core, ctx, &params, None, Some(PANEL_SERIES.into()));
    }
    // Scheduled (CAP-050): Schedule; again, Cancel schedule.
    if command == "capture.schedule" {
        if let Some(id) = schedule::of_preset(core, PANEL_SERIES) {
            return schedule::cancel(core, ctx.lang, Some(&id));
        }
        return start_schedule(core, ctx, &params, None, Some(PANEL_SERIES.into()));
    }
    dispatch(core, ctx, command, &params, extra)
}

/// The series the capture panel started (CAP-049).
const PANEL_SERIES: &str = "@panel";

/// Keys of `capture.interval` and `capture.schedule` that are not the capture's own.
const TIMING_KEYS: [&str; 12] = [
    "every",
    "stop",
    "count",
    "minutes",
    "until",
    "discard",
    "sensitivity",
    "selection",
    "name",
    "date",
    "time",
    "repeat",
];

/// Starts an interval series (CAP-049): the area or window is chosen once, then each
/// turn runs the capture command of the selection, without opening the editor.
pub(crate) fn start_interval(
    core: &Core,
    ctx: &Context,
    p: &Value,
    name: Option<String>,
    origin: Option<String>,
) -> CommandResult {
    let plan = interval::Plan::from_params(p, chrono::Local::now())
        .map_err(|e| CommandError::new(ErrorCode::InvalidParams, e))?;
    let (command, params) = repeated_capture(core, ctx, p)?;
    interval::start(core, ctx, command, params, plan, name, origin)
}

/// Schedules a capture (CAP-050): a moment already gone is refused before anything
/// is chosen; then the area or window is chosen, as for an interval.
pub(crate) fn start_schedule(
    core: &Core,
    ctx: &Context,
    p: &Value,
    name: Option<String>,
    origin: Option<String>,
) -> CommandResult {
    let lang = ctx.lang;
    let at = schedule::first_time(p, chrono::Local::now()).map_err(|e| {
        if e == "past" {
            CommandError::new(ErrorCode::InvalidParams, tr(lang, "errors.schedule_past", &[]))
        } else {
            CommandError::new(ErrorCode::InvalidParams, e)
        }
    })?;
    let repeat = schedule::Repeat::parse(p.get("repeat").and_then(Value::as_str).unwrap_or("once"))
        .ok_or_else(|| {
            CommandError::new(
                ErrorCode::InvalidParams,
                "repeat: once, daily, weekdays or weekly",
            )
        })?;
    let (command, params) = repeated_capture(core, ctx, p)?;
    schedule::add(core, ctx, command, params, at, repeat, name, origin)
}

/// The capture an interval or a schedule runs later (CAP-049, CAP-050): its command
/// and parameters, with the region or window already chosen, without opening the
/// editor and with only the options it was given.
fn repeated_capture(core: &Core, ctx: &Context, p: &Value) -> Result<(&'static str, Value), CommandError> {
    let lang = ctx.lang;
    let selection = p.get("selection").and_then(Value::as_str).unwrap_or("fullscreen");
    let mut params = p.clone();
    if let Some(o) = params.as_object_mut() {
        for k in TIMING_KEYS
            .iter()
            .chain(&["size", "multiple", "extract", "target"])
        {
            o.remove(*k);
        }
    }
    params["open"] = json!("none");
    params["delay"] = json!(0);
    // Each turn uses the options it started with (from the panel, all of them): the
    // panel's effects and destinations are not added later.
    for (k, v) in [
        ("copy", json!(true)),
        ("cursor", json!(false)),
        ("effects", json!([])),
        ("share", json!([])),
    ] {
        if params.get(k).is_none() {
            params[k] = v;
        }
    }
    let pick = |f: &dyn Fn() -> CommandResult| -> CommandResult {
        let _busy = Busy::take(lang)?;
        core.host().capture_session(true);
        let picked = f();
        core.host().capture_session(false);
        picked
    };
    let command = match selection {
        "region" => {
            if params.get("rect").is_none() {
                params["rect"] = pick(&|| interactive::pick(core, ctx))?;
            }
            "capture.region"
        }
        "window" => {
            // A window id from automation, or the one the user picks.
            let id = match p.get("id").filter(|v| v.is_u64()) {
                Some(id) => id.clone(),
                None => pick(&|| interactive::pick_window(core, ctx).map(|id| json!(id)))?,
            };
            params["target"] = json!("id");
            params["id"] = id;
            "capture.window"
        }
        "fullscreen" => "capture.fullscreen",
        "menu" => "capture.menu",
        "clipboard" => {
            params["cursor"] = json!(false);
            "capture.clipboard"
        }
        // A web page (CAP-037): its address, given at the start or the last one
        // captured; each turn loads it again.
        "webpage" => {
            if params
                .get("url")
                .and_then(Value::as_str)
                .is_none_or(|u| u.trim().is_empty())
            {
                let last = core
                    .recall(webcap::LAST_URL_KEY)
                    .and_then(|v| v.as_str().map(str::to_string))
                    .ok_or_else(|| {
                        CommandError::new(
                            ErrorCode::InvalidParams,
                            tr(lang, "errors.webpage_series_url", &[]),
                        )
                    })?;
                params["url"] = json!(last);
            }
            params["cursor"] = json!(false);
            "capture.webpage"
        }
        // A file (CAP-040): the fixed one, or one chosen once; each turn reads it again
        // and keeps it only if it changed.
        "file" => {
            if params
                .get("paths")
                .and_then(Value::as_array)
                .is_none_or(|a| a.is_empty())
            {
                let chosen = core
                    .host()
                    .pick_files(
                        false,
                        &[(
                            tr(lang, "ui.panel.file_filter", &[]),
                            filesource::IMAGE_FILES.iter().map(|e| e.to_string()).collect(),
                        )],
                    )
                    .and_then(|f| f.into_iter().next())
                    .ok_or_else(|| cancelled(lang))?;
                params["paths"] = json!([chosen]);
            }
            params["cursor"] = json!(false);
            "capture.file"
        }
        _ => {
            return Err(CommandError::new(
                ErrorCode::InvalidParams,
                tr(lang, "errors.interval_selection", &[]),
            ))
        }
    };
    Ok((command, params))
}

fn upgrade(weak: &Weak<Core>) -> Result<Arc<Core>, CommandError> {
    weak.upgrade()
        .ok_or_else(|| CommandError::internal("shutting down"))
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
        for id in ["capture.fullscreen", "capture.region", "capture.window"] {
            let w = weak.clone();
            reg.handle(id, move |ctx, p| {
                let core = upgrade(&w)?;
                // From the app (shortcuts, menus) the rest comes from the Image tab
                // (CAP-065); automation keeps fixed defaults so scripts stay repeatable.
                let p = if ctx.source.is_automation() {
                    p
                } else {
                    panel::with_overrides(panel::load(&core).image_options(), &p)
                };
                dispatch(&core, ctx, id, &p, json!({}))
            });
        }
        let w = weak.clone();
        reg.handle("capture.universal", move |ctx, p| {
            let core = upgrade(&w)?;
            // The edge launcher's Capture (CAP-004): always Universal into the editor
            // with the pointer, without delay, effects or destinations.
            if p.get("fixed").and_then(Value::as_bool) == Some(true) {
                let fixed =
                    json!({ "open": "editor", "cursor": true, "copy": true, "delay": 0, "share": [] });
                return dispatch(&core, ctx, "capture.universal", &fixed, json!({}));
            }
            capture_with_tab(&core, ctx, "universal", &p)
        });
        let w = weak.clone();
        reg.handle("capture.image", move |ctx, p| {
            capture_with_tab(&*upgrade(&w)?, ctx, "image", &p)
        });
        let w = weak.clone();
        reg.handle("capture.capture", move |ctx, p| {
            let core = upgrade(&w)?;
            let tab = panel::load(&core).tab;
            capture_with_tab(&core, ctx, &tab, &p)
        });
        let w = weak.clone();
        reg.handle("capture.repeat_last", move |ctx, _p| {
            let core = upgrade(&w)?;
            match core.recall(LAST_KEY) {
                Some(last) => {
                    let command = last["command"].as_str().unwrap_or_default().to_string();
                    dispatch(&core, ctx, &command, &last["params"], last["extra"].clone())
                }
                // Nothing to repeat yet: open the capture panel (CAP-007).
                None => {
                    let _ = core.host().show_capture_panel(Some(true));
                    Ok(json!({ "status": "none", "panel": true }))
                }
            }
        });
        let w = weak.clone();
        reg.handle("capture.repeat_last_area", move |ctx, p| {
            repeat_last_area(&*upgrade(&w)?, ctx, &p)
        });
        let w = weak.clone();
        reg.handle("capture.panel", move |_ctx, p| {
            let core = upgrade(&w)?;
            let visible = core
                .host()
                .show_capture_panel(p.get("show").and_then(Value::as_bool))?;
            Ok(json!({ "visible": visible }))
        });
        let w = weak.clone();
        reg.handle("capture.settings", move |_ctx, _p| {
            Ok(serde_json::to_value(panel::load(&*upgrade(&w)?)).unwrap_or_default())
        });
        let w = weak.clone();
        reg.handle("capture.configure", move |ctx, p| {
            let core = upgrade(&w)?;
            let next = panel::load(&core).patched(&p).map_err(|e| {
                let param = e.split(':').next().unwrap_or("").to_string();
                CommandError::new(
                    ErrorCode::InvalidParams,
                    tr(
                        ctx.lang,
                        "errors.invalid_param",
                        &[("param", &param), ("reason", &e)],
                    ),
                )
            })?;
            panel::save(&core, &next);
            Ok(serde_json::to_value(next).unwrap_or_default())
        });
        let w = weak.clone();
        reg.handle("capture.multi_start", move |ctx, _p| {
            multi::start(&upgrade(&w)?, ctx)
        });
        let w = weak.clone();
        reg.handle("capture.multi_stop", move |_ctx, p| {
            let close = p.get("close").and_then(Value::as_bool).unwrap_or(false);
            Ok(multi::stop(&*upgrade(&w)?, close))
        });
        let w = weak.clone();
        reg.handle("capture.multi_state", move |_ctx, _p| {
            Ok(multi::get_state(&*upgrade(&w)?))
        });
        let w = weak.clone();
        reg.handle("capture.multi_remove", move |ctx, p| {
            let core = upgrade(&w)?;
            let ids: Vec<String> = p["ids"]
                .as_array()
                .map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect())
                .unwrap_or_default();
            if let Some(missing) = ids.iter().find(|id| !multi::has_item(&core, id)) {
                return Err(multi::unknown(ctx.lang, missing));
            }
            Ok(multi::remove(&core, &ids))
        });
        let w = weak.clone();
        reg.handle("capture.multi_clear", move |_ctx, _p| {
            Ok(multi::clear(&*upgrade(&w)?))
        });
        // Interval capture (CAP-049): start a series, list them, stop one or all.
        // Scheduled capture (CAP-050): the schedules kept from the last run wait again.
        if let Some(core) = weak.upgrade() {
            interval::install(&core);
            schedule::install(&core);
            // System captures (CAP-042): watched while the option is on.
            systemshots::install(&core);
            // The receive folder (CAP-043): watched while one is chosen.
            inbox::install(&core);
        }
        // Video recording (VID-024).
        let w = weak.clone();
        reg.handle("record.start", move |ctx, p| {
            record::start(&upgrade(&w)?, ctx, &p)
        });
        let w = weak.clone();
        reg.handle("record.stop", move |ctx, _p| record::stop(&*upgrade(&w)?, ctx));
        let w = weak.clone();
        reg.handle("record.cancel", move |ctx, _p| {
            record::cancel(&*upgrade(&w)?, ctx)
        });
        let w = weak.clone();
        reg.handle("record.pause", move |ctx, _p| {
            record::pause(&*upgrade(&w)?, ctx, true)
        });
        let w = weak.clone();
        reg.handle("record.resume", move |ctx, _p| {
            record::pause(&*upgrade(&w)?, ctx, false)
        });
        let w = weak.clone();
        reg.handle("record.status", move |_ctx, _p| {
            Ok(record::status(&*upgrade(&w)?))
        });
        let w = weak.clone();
        reg.handle("record.video", move |ctx, _p| record::video(&upgrade(&w)?, ctx));
        let w = weak.clone();
        reg.handle("record.toggle", move |ctx, _p| record::toggle(&upgrade(&w)?, ctx));
        // The text box of Capture details (CAP-051).
        let w = weak.clone();
        reg.handle("capture.details_text", move |ctx, p| {
            asktext::answer(&*upgrade(&w)?, ctx.lang, &p)
        });
        // Sending later from the editor (CAP-052).
        let w = weak.clone();
        reg.handle("capture.send_pending", move |ctx, p| {
            pending::send(&*upgrade(&w)?, ctx, &p)
        });
        let w = weak.clone();
        reg.handle("capture.pending_shares", move |ctx, _p| {
            Ok(pending::list(&*upgrade(&w)?, ctx.lang))
        });
        let w = weak.clone();
        reg.add_observer(Box::new(move |_ctx, spec, result| {
            // A document closed without sending sends nothing (CAP-052).
            if spec.id == "document.close" && result.is_ok() {
                if let Some(core) = w.upgrade() {
                    pending::forget_closed(&core);
                }
            }
        }));
        // A file as the source (CAP-040) and Import PDF (CAP-041).
        let w = weak.clone();
        reg.handle("capture.file", move |ctx, p| {
            let core = upgrade(&w)?;
            let p = if ctx.source.is_automation() {
                p
            } else {
                panel::with_overrides(panel::load(&core).image_options(), &p)
            };
            dispatch(&core, ctx, "capture.file", &p, json!({}))
        });
        let w = weak.clone();
        reg.handle("capture.pdf", move |ctx, p| {
            let core = upgrade(&w)?;
            let lang = ctx.lang;
            let dialog = p
                .get("dialog")
                .and_then(Value::as_bool)
                .unwrap_or(!ctx.source.is_automation());
            let p = if ctx.source.is_automation() {
                p
            } else {
                panel::with_overrides(panel::load(&core).image_options(), &p)
            };
            if dialog {
                // The Import PDF window, for a file given or chosen now.
                let path = match p.get("path").and_then(Value::as_str) {
                    Some(path) => std::path::PathBuf::from(path),
                    None => core
                        .host()
                        .pick_files(false, &[("PDF".into(), vec!["pdf".into()])])
                        .and_then(|f| f.into_iter().next())
                        .ok_or_else(|| cancelled(lang))?,
                };
                let id = pdf::new_window(&core, &path);
                core.host().pdf_window(&id)?;
                return Ok(json!({ "status": "dialog", "id": id }));
            }
            let opts = Options::from_params(&p, ctx.source);
            pdf::import(&core, ctx, &opts, &p)
        });
        // A web page by its address (CAP-037): from the app without an address, the box
        // that asks for it.
        let w = weak.clone();
        reg.handle("capture.webpage", move |ctx, p| {
            let core = upgrade(&w)?;
            let p = if ctx.source.is_automation() {
                p
            } else {
                panel::with_overrides(panel::load(&core).image_options(), &p)
            };
            let has_url = p
                .get("url")
                .and_then(Value::as_str)
                .is_some_and(|u| !u.trim().is_empty());
            if !has_url && !ctx.source.is_automation() {
                core.host().webpage_prompt()?;
                return Ok(json!({ "status": "dialog" }));
            }
            dispatch(&core, ctx, "capture.webpage", &p, json!({}))
        });
        let w = weak.clone();
        reg.handle("capture.webpage_cancel", move |_ctx, _p| {
            upgrade(&w)?.host().cancel_webpage();
            Ok(json!({}))
        });
        let w = weak.clone();
        reg.handle("capture.webpage_suggest", move |_ctx, _p| {
            Ok(webcap::suggest(&*upgrade(&w)?))
        });
        let w = weak.clone();
        reg.handle("capture.pdf_info", move |ctx, p| {
            pdf::info(&*upgrade(&w)?, ctx, &p)
        });
        let w = weak.clone();
        reg.handle("capture.pdf_page", move |ctx, p| {
            pdf::page_png(&*upgrade(&w)?, ctx, &p)
        });
        let w = weak.clone();
        reg.handle("capture.pdf_cancel", move |_ctx, p| {
            let core = upgrade(&w)?;
            Ok(pdf::cancel(&core, p.get("id").and_then(Value::as_str)))
        });
        let w = weak.clone();
        reg.handle("capture.schedule", move |ctx, p| {
            let core = upgrade(&w)?;
            let p = if ctx.source.is_automation() {
                p
            } else {
                panel::with_overrides(panel::load(&core).image_options(), &p)
            };
            let name = p.get("name").and_then(Value::as_str).map(str::to_string);
            start_schedule(&core, ctx, &p, name, None)
        });
        let w = weak.clone();
        reg.handle("capture.schedule_list", move |ctx, _p| {
            Ok(schedule::list(&*upgrade(&w)?, ctx.lang))
        });
        let w = weak.clone();
        reg.handle("capture.schedule_cancel", move |ctx, p| {
            let core = upgrade(&w)?;
            schedule::cancel(&core, ctx.lang, p.get("id").and_then(Value::as_str))
        });
        let w = weak.clone();
        reg.handle("capture.interval", move |ctx, p| {
            let core = upgrade(&w)?;
            let p = if ctx.source.is_automation() {
                p
            } else {
                panel::with_overrides(panel::load(&core).image_options(), &p)
            };
            let name = p.get("name").and_then(Value::as_str).map(str::to_string);
            start_interval(&core, ctx, &p, name, None)
        });
        let w = weak.clone();
        reg.handle("capture.interval_state", move |ctx, _p| {
            Ok(interval::state(&*upgrade(&w)?, ctx.lang))
        });
        let w = weak.clone();
        reg.handle("capture.interval_stop", move |ctx, p| {
            let core = upgrade(&w)?;
            interval::stop(&core, ctx.lang, p.get("id").and_then(Value::as_str))
        });
        // Quick annotation (CAP-057): only while its trial is on (APP-034). From the app
        // it takes Capture cursor from the Image tab; effects and Share only come from a
        // preset that has them.
        let w = weak.clone();
        reg.handle("capture.quick_annotate", move |ctx, p| {
            let core = upgrade(&w)?;
            if !core.settings.get_bool("experimental.quickAnnotate") {
                return Err(CommandError::new(
                    ErrorCode::InvalidParams,
                    tr(ctx.lang, "errors.quick_annotate_off", &[]),
                )
                .with_details(json!({ "setting": "experimental.quickAnnotate" })));
            }
            let p = if ctx.source.is_automation() {
                p
            } else {
                let tab = panel::load(&core).image_options();
                panel::with_overrides(json!({ "cursor": tab["cursor"] }), &p)
            };
            dispatch(&core, ctx, "capture.quick_annotate", &p, json!({}))
        });
        let w = weak.clone();
        reg.handle("capture.clipboard", move |ctx, p| {
            let core = upgrade(&w)?;
            let p = if ctx.source.is_automation() {
                p
            } else {
                panel::with_overrides(panel::load(&core).image_options(), &p)
            };
            dispatch(&core, ctx, "capture.clipboard", &p, json!({}))
        });
        // Window picker (CAP-030).
        let w = weak.clone();
        reg.handle("capture.window_picker", move |ctx, p| {
            let core = upgrade(&w)?;
            let p = if ctx.source.is_automation() {
                p
            } else {
                panel::with_overrides(panel::load(&core).image_options(), &p)
            };
            dispatch(&core, ctx, "capture.window_picker", &p, json!({}))
        });
        let w = weak.clone();
        reg.handle("capture.picker_state", move |_ctx, _p| {
            Ok(picker::state(&*upgrade(&w)?))
        });
        let w = weak.clone();
        reg.handle("capture.picker_choose", move |ctx, p| {
            let core = upgrade(&w)?;
            let choice = match p.get("id").and_then(Value::as_u64) {
                Some(id) => picker::Choice::Window(id as u32),
                None => picker::Choice::Cancel,
            };
            if !picker::choose(&core, choice) {
                return Err(CommandError::new(
                    ErrorCode::NotFound,
                    tr(ctx.lang, "errors.picker_not_open", &[]),
                ));
            }
            Ok(json!({ "ok": true }))
        });
        let w = weak.clone();
        reg.handle("capture.menu", move |ctx, p| {
            let core = upgrade(&w)?;
            let p = if ctx.source.is_automation() {
                p
            } else {
                panel::with_overrides(panel::load(&core).image_options(), &p)
            };
            dispatch(&core, ctx, "capture.menu", &p, json!({}))
        });
        // The File destination's name preview and counter (CAP-053).
        let w = weak.clone();
        reg.handle("capture.file_name", move |ctx, p| {
            let core = upgrade(&w)?;
            let dest: filedest::FileDest =
                serde_json::from_value(p.get("file").cloned().unwrap_or(json!({})))
                    .map_err(|e| CommandError::new(ErrorCode::InvalidParams, e.to_string()))?;
            dest.validate().map_err(|e| {
                CommandError::new(
                    ErrorCode::InvalidParams,
                    tr(
                        ctx.lang,
                        "errors.invalid_param",
                        &[("param", "file"), ("reason", &e)],
                    ),
                )
            })?;
            if let Some(next) = p.get("next").and_then(Value::as_u64) {
                filedest::set_counter(&core, &dest, next.clamp(1, u32::MAX as u64 - 1) as u32);
            }
            let values = filename::Values {
                prefix: dest.prefix.clone(),
                app: Some("Excel".into()),
                title: Some(tr(ctx.lang, "ui.panel.file_sample_title", &[])),
                site: None,
                mode: "region".into(),
                width: 1920,
                height: 1080,
                user: platform::user_and_machine().0,
            };
            Ok(json!({
                "folder": dest.folder(),
                "name": filedest::preview(&core, &dest, &values),
                "next": filedest::next_counter(&core, &dest),
                "counter": filename::uses_counter(&dest.pattern),
                "markers": filename::MARKERS,
            }))
        });
        // Choose on screen (CAP-026): an area for the fixed region, nothing captured.
        let w = weak.clone();
        reg.handle("capture.pick_area", move |ctx, _p| {
            let core = upgrade(&w)?;
            let _busy = Busy::take(ctx.lang)?;
            core.host().capture_session(true);
            let picked = interactive::pick(&core, ctx);
            core.host().capture_session(false);
            picked
        });
        // Scrolling capture (CAP-034 to CAP-036).
        let w = weak.clone();
        reg.handle("capture.scrolling", move |ctx, p| {
            let core = upgrade(&w)?;
            let p = if ctx.source.is_automation() {
                p
            } else {
                panel::with_overrides(panel::load(&core).image_options(), &p)
            };
            dispatch(&core, ctx, "capture.scrolling", &p, json!({}))
        });
        let w = weak.clone();
        reg.handle("capture.scroll_action", move |ctx, p| {
            let core = upgrade(&w)?;
            let action = match p.get("action").and_then(Value::as_str) {
                Some("start") => scroll::Action::Start,
                Some("done") => scroll::Action::Done,
                _ => scroll::Action::Cancel,
            };
            if !scroll::act(&core, action) {
                return Err(CommandError::new(
                    ErrorCode::NotFound,
                    tr(ctx.lang, "errors.scroll_not_running", &[]),
                ));
            }
            Ok(json!({ "running": true }))
        });
        let w = weak.clone();
        reg.handle("capture.scroll_state", move |_ctx, _p| {
            Ok(scroll::state(&*upgrade(&w)?))
        });
        // CAP-058 to CAP-064; their shortcut rows exist from the start (CAP-062).
        preset_cmds::register(reg, &weak);
        // Extract text (SMT-003 to SMT-006).
        extract::register(reg, &weak);
        if let Some(core) = weak.upgrade() {
            let lang = core.lang();
            let rows = presets::load(&core)
                .presets
                .iter()
                .map(|p| (presets::slot_of(&p.id), presets::display_name(p, lang)))
                .collect();
            core.shortcuts.set_extra(rows);
        }
        let w = weak;
        reg.handle("capture.reset_settings", move |_ctx, _p| {
            let core = upgrade(&w)?;
            let defaults = panel::PanelSettings::default();
            panel::save(&core, &defaults);
            Ok(serde_json::to_value(defaults).unwrap_or_default())
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

    /// CAP-007: with no previous capture, Repeat opens the panel and is not an error.
    #[test]
    fn repeat_last_without_history_opens_the_panel() {
        let core = Core::for_tests_with(Platform::current(), vec![extension()]);
        let r = core
            .call(Source::Ui, "capture.repeat_last", json!({}), None)
            .unwrap();
        assert_eq!(r, json!({"status": "none", "panel": true}));
    }

    /// CAP-007: Repeat last area without history opens the panel too.
    #[test]
    fn repeat_last_area_without_history_opens_the_panel() {
        let core = Core::for_tests_with(Platform::current(), vec![extension()]);
        let r = core
            .call(Source::Ui, "capture.repeat_last_area", json!({}), None)
            .unwrap();
        assert_eq!(r, json!({"status": "none", "panel": true}));
    }

    /// CAP-001, CAP-003: settings persist per tab, bad values change nothing, and
    /// reset brings back every default, including Selection and the delay seconds.
    #[test]
    fn panel_settings_configure_and_reset() {
        let core = Core::for_tests_with(Platform::current(), vec![extension()]);
        let call = |id: &str, p: Value| core.call(Source::Ui, id, p, None);
        call(
            "capture.configure",
            json!({"tab": "universal", "expanded": false, "image": {"selection": "window", "timing": "delay", "delay": 9}, "universal": {"delay5": true}}),
        )
        .unwrap();
        let before = call("capture.settings", json!({})).unwrap();
        assert_eq!(before["image"]["selection"], "window");
        assert_eq!(before["tab"], "universal");
        let bad = call("capture.configure", json!({"image": {"delay": 99}})).unwrap_err();
        assert_eq!(bad.code, ErrorCode::InvalidParams);
        assert_eq!(call("capture.settings", json!({})).unwrap(), before);
        let reset = call("capture.reset_settings", json!({})).unwrap();
        assert_eq!(
            reset,
            serde_json::to_value(panel::PanelSettings::default()).unwrap()
        );
        assert_eq!(reset["image"]["delay"], 5);
        assert_eq!(reset["image"]["selection"], "region");
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
    fn black_captures_are_noticed() {
        assert!(is_blank(&RgbaImage::from_pixel(
            4,
            4,
            image::Rgba([0, 0, 0, 255])
        )));
        let mut one = RgbaImage::from_pixel(4, 4, image::Rgba([0, 0, 0, 255]));
        one.put_pixel(3, 3, image::Rgba([1, 0, 0, 255]));
        assert!(!is_blank(&one));
        assert!(!is_blank(&RgbaImage::new(0, 0)));
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
            window_id: None,
            pointer: None,
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
