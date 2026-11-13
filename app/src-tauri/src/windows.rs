//! Windows are created on demand and can close without ending the process (APP-001, APP-004).

use std::sync::atomic::Ordering;

use serde_json::json;
use tauri::{
    AppHandle, Emitter, LogicalSize, Manager, Theme, WebviewUrl, WebviewWindow, WebviewWindowBuilder,
};

use crate::AppState;

pub const MAIN: &str = "main";
pub const PREFERENCES: &str = "preferences";
pub const CAPTURE_PANEL: &str = "capture-panel";
pub const MULTI_PANEL: &str = "multicapture";
pub const SCROLL_PANEL: &str = "scroll";
pub const PICKER: &str = "picker";
pub const HELP: &str = "help";
pub const WELCOME: &str = "welcome";
pub const WEBPAGE: &str = "webpage";
pub const WORKING: &str = "working";
pub const ASK_TEXT: &str = "ask-text";
/// Logical width of the capture panel; its height follows the content.
const PANEL_WIDTH: f64 = 400.0;

pub fn theme(app: &AppHandle) -> Option<Theme> {
    match app
        .state::<AppState>()
        .core
        .settings
        .get_str("general.theme")
        .as_str()
    {
        "light" => Some(Theme::Light),
        "dark" => Some(Theme::Dark),
        _ => None,
    }
}

fn focus(w: &WebviewWindow) {
    let _ = w.show();
    let _ = w.unminimize();
    let _ = w.set_focus();
}

/// On macOS the Dock icon only shows while a main window is open (APP-001).
fn dock(app: &AppHandle, visible: bool) {
    #[cfg(target_os = "macos")]
    {
        let policy = if visible {
            tauri::ActivationPolicy::Regular
        } else {
            tauri::ActivationPolicy::Accessory
        };
        let _ = app.set_activation_policy(policy);
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (app, visible);
}

pub fn show_main(app: &AppHandle) -> tauri::Result<()> {
    dock(app, true);
    if let Some(w) = app.get_webview_window(MAIN) {
        memory_target(&w, false);
        focus(&w);
        return Ok(());
    }
    let title = xs_core::i18n::tr(app.state::<AppState>().core.lang(), "app.name", &[]);
    #[allow(unused_mut)]
    let mut builder = WebviewWindowBuilder::new(app, MAIN, WebviewUrl::App("index.html".into()))
        .title(title)
        .inner_size(1280.0, 800.0)
        // EDT-001: the editor stays usable down to 900 × 600.
        .min_inner_size(900.0, 600.0)
        .theme(theme(app));
    // On Windows the editor draws its own menu bar (EDT-002, `xs_editor_menus`): a
    // native one cannot animate like the rest of the app (APP-012).
    let w = builder.build()?;
    focus(&w);
    Ok(())
}

/// Asks WebView2 to keep little memory for a window while it is hidden (the editor
/// kept loaded, APP-004), and the usual again when it shows (APP-051). The textures of
/// large captures go back to the system this way.
pub fn memory_target(w: &WebviewWindow, low: bool) {
    #[cfg(windows)]
    {
        let _ = w.with_webview(move |webview| {
            use webview2_com::Microsoft::Web::WebView2::Win32::{
                ICoreWebView2_19, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW,
                COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
            };
            use windows_core::Interface;
            let level = if low {
                COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW
            } else {
                COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL
            };
            // SAFETY: COM calls on the webview's own controller, on its thread; an older
            // WebView2 without ICoreWebView2_19 is left as it is.
            unsafe {
                if let Ok(core) = webview.controller().CoreWebView2() {
                    if let Ok(v19) = core.cast::<ICoreWebView2_19>() {
                        let _ = v19.SetMemoryUsageTargetLevel(level);
                    }
                }
            }
        });
    }
    #[cfg(not(windows))]
    let _ = (w, low);
}

/// Title of the main window: the document shown and its unsaved state (EDT-012).
pub fn set_main_title(app: &AppHandle, title: &str) {
    if let Some(w) = app.get_webview_window(MAIN) {
        let _ = w.set_title(title);
    }
}

pub fn open_preferences(app: &AppHandle, section: Option<&str>, key: Option<&str>) -> tauri::Result<()> {
    let nav = json!({ "section": section, "key": key });
    if let Some(w) = app.get_webview_window(PREFERENCES) {
        focus(&w);
        let _ = w.emit("xs://navigate", nav);
        return Ok(());
    }
    *app.state::<AppState>().pending_navigation.lock().unwrap() = Some(nav);
    let title = xs_core::i18n::tr(app.state::<AppState>().core.lang(), "ui.prefs.title", &[]);
    let w = WebviewWindowBuilder::new(app, PREFERENCES, WebviewUrl::App("index.html".into()))
        .title(title)
        .inner_size(820.0, 600.0)
        .min_inner_size(640.0, 420.0)
        .theme(theme(app))
        .build()?;
    focus(&w);
    Ok(())
}

/// The welcome of the first run (APP-005): a small fixed window in the middle.
pub fn open_welcome(app: &AppHandle) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window(WELCOME) {
        focus(&w);
        return Ok(());
    }
    let title = xs_core::i18n::tr(app.state::<AppState>().core.lang(), "ui.welcome.title", &[]);
    let w = WebviewWindowBuilder::new(app, WELCOME, WebviewUrl::App("index.html".into()))
        .title(title)
        .inner_size(560.0, 520.0)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .center()
        .theme(theme(app))
        .build()?;
    focus(&w);
    Ok(())
}

/// Where the size and place of the last results window are kept (SMT-006).
pub const TEXT_WINDOW_KEY: &str = "smart.textWindow";

/// Keeps the size and place of a results window that closes (SMT-006).
pub fn remember_text_window(app: &AppHandle, w: &tauri::Window) {
    let (Ok(pos), Ok(size), Ok(scale)) = (w.outer_position(), w.inner_size(), w.scale_factor()) else {
        return;
    };
    app.state::<AppState>().core.remember(
        TEXT_WINDOW_KEY,
        json!({
            "x": pos.x,
            "y": pos.y,
            "width": size.width as f64 / scale,
            "height": size.height as f64 / scale,
        }),
    );
}

/// A results window of Extract text (SMT-006): one per extraction, near the pointer,
/// resizable, without a minimize button. Its label carries the result's id.
pub fn open_text_result(app: &AppHandle, id: &str) -> tauri::Result<()> {
    let label = format!("text-{id}");
    if let Some(w) = app.get_webview_window(&label) {
        focus(&w);
        return Ok(());
    }
    let core = &app.state::<AppState>().core;
    let title = xs_core::i18n::tr(core.lang(), "ui.extract.title", &[]);
    // The size and place it had last (SMT-006).
    let kept = core.recall(TEXT_WINDOW_KEY);
    let dim = |k: &str, d: f64| kept.as_ref().and_then(|v| v[k].as_f64()).unwrap_or(d);
    let (width, height) = (dim("width", 560.0).max(360.0), dim("height", 420.0).max(260.0));
    let w = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html".into()))
        .title(title)
        .inner_size(width, height)
        .min_inner_size(360.0, 260.0)
        .minimizable(false)
        .visible(false)
        .theme(theme(app))
        .build()?;
    // Where it was last, if that is still on a monitor.
    if let Some((x, y)) = kept
        .as_ref()
        .and_then(|v| Some((v["x"].as_i64()? as i32, v["y"].as_i64()? as i32)))
    {
        let on_screen = app
            .monitor_from_point(x as f64 + 20.0, y as f64 + 20.0)
            .ok()
            .flatten()
            .is_some();
        if on_screen {
            let _ = w.set_position(tauri::PhysicalPosition::new(x, y));
            focus(&w);
            return Ok(());
        }
    }
    // Near the zone, which ends under the pointer: below and to the right of it, kept
    // inside its monitor.
    let pointer = app.cursor_position().ok();
    let monitor = pointer
        .and_then(|p| app.monitor_from_point(p.x, p.y).ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten());
    if let (Some(p), Some(m)) = (pointer, monitor) {
        let scale = m.scale_factor();
        let area = m.work_area();
        let (w_px, h_px) = ((width * scale) as i32, (height * scale) as i32);
        let max_x = area.position.x + area.size.width as i32 - w_px;
        let max_y = area.position.y + area.size.height as i32 - h_px;
        let x = (p.x as i32 + 16).clamp(area.position.x, max_x.max(area.position.x));
        let y = (p.y as i32 + 16).clamp(area.position.y, max_y.max(area.position.y));
        let _ = w.set_position(tauri::PhysicalPosition::new(x, y));
    }
    focus(&w);
    Ok(())
}

/// "Working" next to a zone of the screen (SMT-005): small, in front, without taking
/// the focus, never in a capture. `None` closes it.
pub fn working(app: &AppHandle, zone: Option<serde_json::Value>) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window(WORKING) {
        w.destroy()?;
    }
    let Some(zone) = zone else { return Ok(()) };
    let n = |k: &str| zone[k].as_f64().unwrap_or(0.0);
    let (x, y, w, h) = (n("x"), n("y"), n("width"), n("height"));
    let monitor = app
        .monitor_from_point(x + w / 2.0, y + h / 2.0)
        .ok()
        .flatten()
        .or_else(|| app.primary_monitor().ok().flatten());
    let scale = monitor.as_ref().map(|m| m.scale_factor()).unwrap_or(1.0);
    let (width, height) = (190.0, 40.0);
    // Under the zone's bottom left, or inside it at the bottom when there is no room.
    let mut px = x as i32;
    let mut py = (y + h) as i32 + (8.0 * scale) as i32;
    if let Some(m) = &monitor {
        let area = m.work_area();
        let right = area.position.x + area.size.width as i32 - (width * scale) as i32;
        let bottom = area.position.y + area.size.height as i32 - (height * scale) as i32;
        px = px.clamp(area.position.x, right.max(area.position.x));
        if py > bottom {
            py = ((y + h) as i32 - ((height + 8.0) * scale) as i32).max(area.position.y);
        }
    }
    let win = WebviewWindowBuilder::new(app, WORKING, WebviewUrl::App("index.html".into()))
        .title("XcreenShot")
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focused(false)
        .visible(false)
        .content_protected(true)
        .inner_size(width, height)
        .theme(theme(app))
        .build()?;
    win.set_position(tauri::PhysicalPosition::new(px, py))?;
    win.show()?;
    Ok(())
}

/// The web address box (CAP-037): small, centered, in front; it shows the loading and
/// Cancel too.
pub fn open_webpage_prompt(app: &AppHandle) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window(WEBPAGE) {
        focus(&w);
        return Ok(());
    }
    let title = xs_core::i18n::tr(app.state::<AppState>().core.lang(), "ui.webpage.title", &[]);
    let w = WebviewWindowBuilder::new(app, WEBPAGE, WebviewUrl::App("index.html".into()))
        .title(title)
        .inner_size(520.0, 190.0)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .always_on_top(true)
        .center()
        .theme(theme(app))
        .build()?;
    focus(&w);
    Ok(())
}

/// The box that asks for the text of Capture details when a capture ends (CAP-051):
/// centered and on top; `open: false` closes it.
pub fn ask_text(app: &AppHandle, open: bool) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window(ASK_TEXT) {
        if open {
            focus(&w);
            return Ok(());
        }
        return w.destroy();
    }
    if !open {
        return Ok(());
    }
    let title = xs_core::i18n::tr(app.state::<AppState>().core.lang(), "ui.ask_text.title", &[]);
    let w = WebviewWindowBuilder::new(app, ASK_TEXT, WebviewUrl::App("index.html".into()))
        .title(title)
        .inner_size(440.0, 176.0)
        .resizable(false)
        .maximizable(false)
        .minimizable(false)
        .always_on_top(true)
        .center()
        .theme(theme(app))
        .build()?;
    focus(&w);
    Ok(())
}

/// Import PDF (CAP-041): one window per file, centered, resizable. Its label carries
/// the import's id.
pub fn open_pdf(app: &AppHandle, id: &str) -> tauri::Result<()> {
    let label = format!("pdf-{id}");
    if let Some(w) = app.get_webview_window(&label) {
        focus(&w);
        return Ok(());
    }
    let title = xs_core::i18n::tr(app.state::<AppState>().core.lang(), "ui.pdf.title", &[]);
    let w = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html".into()))
        .title(title)
        .inner_size(760.0, 640.0)
        .min_inner_size(520.0, 420.0)
        .minimizable(false)
        .center()
        .theme(theme(app))
        .build()?;
    focus(&w);
    Ok(())
}

/// The Help window (APP-015) at a topic: the page goes in the address, and an open
/// window is told with an event.
pub fn open_help(app: &AppHandle, topic: Option<&str>) -> tauri::Result<()> {
    let topic = topic.unwrap_or("start");
    if let Some(w) = app.get_webview_window(HELP) {
        focus(&w);
        let _ = w.emit("xs://help", topic);
        return Ok(());
    }
    let title = xs_core::i18n::tr(app.state::<AppState>().core.lang(), "ui.help.title", &[]);
    let w = WebviewWindowBuilder::new(app, HELP, WebviewUrl::App(format!("index.html#{topic}").into()))
        .title(title)
        .inner_size(860.0, 640.0)
        .min_inner_size(560.0, 400.0)
        .theme(theme(app))
        .build()?;
    focus(&w);
    Ok(())
}

/// Capture panel (CAP-001): a normal window with the native title bar that remembers
/// its position. `None` toggles it (its shortcut, CAP-065): it hides when it is the
/// window in front, and otherwise comes to the front.
pub fn capture_panel(app: &AppHandle, show: Option<bool>) -> tauri::Result<bool> {
    let existing = app.get_webview_window(CAPTURE_PANEL);
    let show = show.unwrap_or_else(|| {
        !existing.as_ref().is_some_and(|w| {
            w.is_visible().unwrap_or(false)
                && !w.is_minimized().unwrap_or(false)
                && w.is_focused().unwrap_or(false)
        })
    });
    match (existing, show) {
        (Some(w), true) => focus(&w),
        (Some(w), false) => w.hide()?,
        (None, true) => {
            let title = xs_core::i18n::tr(app.state::<AppState>().core.lang(), "ui.panel.title", &[]);
            let w = WebviewWindowBuilder::new(app, CAPTURE_PANEL, WebviewUrl::App("index.html".into()))
                .title(title)
                .inner_size(PANEL_WIDTH, 460.0)
                .resizable(false)
                .maximizable(false)
                .theme(theme(app))
                .build()?;
            focus(&w);
        }
        (None, false) => {}
    }
    Ok(show)
}

/// Logical size of the multi-capture panel and its distance to the work area edges.
const MULTI_WIDTH: f64 = 340.0;
const MULTI_HEIGHT: f64 = 420.0;
const MULTI_MARGIN: f64 = 16.0;

/// Multi-capture panel (CAP-067): always in front and never part of a capture. It
/// appears without taking the focus, at the bottom right of the monitor under the
/// pointer, so the app being captured keeps it. `false` closes it.
pub fn multicapture_panel(app: &AppHandle, show: bool) -> tauri::Result<()> {
    let existing = app.get_webview_window(MULTI_PANEL);
    match (existing, show) {
        (Some(w), true) => {
            w.show()?;
            w.unminimize()?;
        }
        (Some(w), false) => w.destroy()?,
        (None, true) => {
            let title = xs_core::i18n::tr(app.state::<AppState>().core.lang(), "ui.multi.title", &[]);
            let w = WebviewWindowBuilder::new(app, MULTI_PANEL, WebviewUrl::App("index.html".into()))
                .title(title)
                .inner_size(MULTI_WIDTH, MULTI_HEIGHT)
                .min_inner_size(260.0, 240.0)
                .always_on_top(true)
                .maximizable(false)
                .minimizable(false)
                .focused(false)
                .visible(false)
                .content_protected(true)
                .theme(theme(app))
                .build()?;
            let pointer = app.cursor_position().ok();
            let monitor = pointer
                .and_then(|p| app.monitor_from_point(p.x, p.y).ok().flatten())
                .or_else(|| app.primary_monitor().ok().flatten());
            if let Some(m) = monitor {
                let scale = m.scale_factor();
                let area = m.work_area();
                let (width, height) = ((MULTI_WIDTH * scale) as i32, (MULTI_HEIGHT * scale) as i32);
                let margin = (MULTI_MARGIN * scale) as i32;
                // Position first: moving to a monitor with another scale resizes it.
                w.set_position(tauri::PhysicalPosition::new(
                    area.position.x + area.size.width as i32 - width - margin,
                    area.position.y + area.size.height as i32 - height - margin,
                ))?;
                w.set_size(tauri::PhysicalSize::new(width as u32, height as u32))?;
            }
            w.show()?;
        }
        (None, false) => {}
    }
    Ok(())
}

/// Logical size of the scrolling capture's panel (CAP-034, CAP-035).
const SCROLL_WIDTH: f64 = 300.0;
const SCROLL_HEIGHT: f64 = 520.0;
const SCROLL_AUTO_HEIGHT: f64 = 300.0;
const SCROLL_GAP: f64 = 12.0;

/// A rectangle in physical pixels of the desktop.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Area {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

/// Where the scrolling capture's panel goes (CAP-034): beside the area, on the side of its
/// monitor with most room; if it fits on no side, on another monitor; at worst in a corner
/// of the area's monitor (it never shows in the capture anyway). `size` is the panel's,
/// `gap` the space it keeps from the area and the edges; `monitors` are work areas, the
/// area's first.
pub fn scroll_panel_place(area: Area, size: (i32, i32), gap: i32, monitors: &[Area]) -> (i32, i32) {
    let (pw, ph) = size;
    let Some(m) = monitors.first() else {
        return (area.x + area.w + gap, area.y);
    };
    let clamp_y = |y: i32, m: &Area| y.clamp(m.y + gap, (m.y + m.h - ph - gap).max(m.y + gap));
    let clamp_x = |x: i32, m: &Area| x.clamp(m.x + gap, (m.x + m.w - pw - gap).max(m.x + gap));
    // Room on each side of the area, inside its monitor.
    let right = m.x + m.w - (area.x + area.w);
    let left = area.x - m.x;
    let below = m.y + m.h - (area.y + area.h);
    let above = area.y - m.y;
    let mut sides = [
        (right - pw - 2 * gap, (area.x + area.w + gap, clamp_y(area.y, m))),
        (left - pw - 2 * gap, (area.x - gap - pw, clamp_y(area.y, m))),
        (below - ph - 2 * gap, (clamp_x(area.x, m), area.y + area.h + gap)),
        (above - ph - 2 * gap, (clamp_x(area.x, m), area.y - gap - ph)),
    ];
    sides.sort_by_key(|(room, _)| -room);
    if sides[0].0 >= 0 {
        return sides[0].1;
    }
    // Another monitor: its top right corner.
    if let Some(other) = monitors
        .iter()
        .skip(1)
        .find(|o| o.w >= pw + 2 * gap && o.h >= ph + 2 * gap)
    {
        return (other.x + other.w - pw - gap, other.y + gap);
    }
    (m.x + m.w - pw - gap, m.y + m.h - ph - gap)
}

/// The scrolling capture's panel (CAP-034, CAP-035): always in front, never part of a
/// capture, without taking the focus from the app being scrolled. `None` closes it.
pub fn scroll_panel(app: &AppHandle, place: Option<serde_json::Value>) -> tauri::Result<()> {
    let existing = app.get_webview_window(SCROLL_PANEL);
    let Some(place) = place else {
        if let Some(w) = existing {
            w.destroy()?;
        }
        return Ok(());
    };
    if let Some(w) = existing {
        w.destroy()?;
    }
    let auto = place["auto"].as_bool().unwrap_or(false);
    let rect = &place["rect"];
    let area = Area {
        x: rect["x"].as_i64().unwrap_or(0) as i32,
        y: rect["y"].as_i64().unwrap_or(0) as i32,
        w: rect["width"].as_i64().unwrap_or(0) as i32,
        h: rect["height"].as_i64().unwrap_or(0) as i32,
    };
    let height = if auto { SCROLL_AUTO_HEIGHT } else { SCROLL_HEIGHT };
    let title = xs_core::i18n::tr(app.state::<AppState>().core.lang(), "ui.scroll.title", &[]);
    let w = WebviewWindowBuilder::new(app, SCROLL_PANEL, WebviewUrl::App("index.html".into()))
        .title(title)
        .inner_size(SCROLL_WIDTH, height)
        .resizable(false)
        .always_on_top(true)
        .maximizable(false)
        .minimizable(false)
        .skip_taskbar(true)
        .focused(false)
        .visible(false)
        .content_protected(true)
        .theme(theme(app))
        .build()?;
    // The area's monitor first, then the others.
    let mut monitors: Vec<(Area, f64)> = app
        .available_monitors()?
        .into_iter()
        .map(|m| {
            let wa = m.work_area();
            (
                Area {
                    x: wa.position.x,
                    y: wa.position.y,
                    w: wa.size.width as i32,
                    h: wa.size.height as i32,
                },
                m.scale_factor(),
            )
        })
        .collect();
    let (cx, cy) = (area.x + area.w / 2, area.y + area.h / 2);
    monitors.sort_by_key(|(m, _)| !(cx >= m.x && cx < m.x + m.w && cy >= m.y && cy < m.y + m.h));
    let scale = monitors.first().map(|(_, s)| *s).unwrap_or(1.0);
    let size = ((SCROLL_WIDTH * scale) as i32, (height * scale) as i32);
    let areas: Vec<Area> = monitors.iter().map(|(m, _)| *m).collect();
    let (x, y) = scroll_panel_place(area, size, (SCROLL_GAP * scale) as i32, &areas);
    w.set_position(tauri::PhysicalPosition::new(x, y))?;
    w.set_size(tauri::PhysicalSize::new(size.0 as u32, size.1 as u32))?;
    w.show()?;
    Ok(())
}

/// The window picker (CAP-030): the work area of the monitor under the pointer, in
/// front of everything and never part of a capture. `false` closes it.
pub fn picker_window(app: &AppHandle, show: bool) -> tauri::Result<()> {
    if let Some(w) = app.get_webview_window(PICKER) {
        w.destroy()?;
    }
    if !show {
        return Ok(());
    }
    let title = xs_core::i18n::tr(app.state::<AppState>().core.lang(), "ui.picker.title", &[]);
    let w = WebviewWindowBuilder::new(app, PICKER, WebviewUrl::App("index.html".into()))
        .title(title)
        .decorations(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .visible(false)
        .content_protected(true)
        .theme(theme(app))
        .build()?;
    let pointer = app.cursor_position().ok();
    let monitor = pointer
        .and_then(|p| app.monitor_from_point(p.x, p.y).ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten());
    if let Some(m) = monitor {
        let area = m.work_area();
        w.set_position(tauri::PhysicalPosition::new(area.position.x, area.position.y))?;
        w.set_size(tauri::PhysicalSize::new(area.size.width, area.size.height))?;
    }
    w.show()?;
    w.set_focus()?;
    Ok(())
}

/// The panel fits its content (compact or expanded view, CAP-001).
pub fn fit_capture_panel(app: &AppHandle, height: f64) {
    if let Some(w) = app.get_webview_window(CAPTURE_PANEL) {
        let _ = w.set_size(LogicalSize::new(PANEL_WIDTH, height.clamp(120.0, 900.0)));
    }
}

/// While a capture runs the panel steps aside, and comes back afterwards in the state
/// it had (CAP-001). Runs on the UI thread and waits, so it is gone before the screen
/// is frozen.
pub fn panel_capture_session(app: &AppHandle, active: bool) {
    let (tx, rx) = std::sync::mpsc::channel();
    let handle = app.clone();
    let queued = app.run_on_main_thread(move || {
        let state = handle.state::<AppState>();
        if active {
            let shown = handle
                .get_webview_window(CAPTURE_PANEL)
                .filter(|w| w.is_visible().unwrap_or(false) && !w.is_minimized().unwrap_or(false));
            if let Some(w) = shown {
                let _ = w.hide();
                state.panel_stepped_aside.store(true, Ordering::SeqCst);
            }
        } else {
            restore_capture_panel(&handle);
        }
        let _ = tx.send(());
    });
    if queued.is_ok() {
        let _ = rx.recv_timeout(std::time::Duration::from_millis(500));
    }
}

/// Brings back the panel if a capture had put it aside (UI thread).
pub fn restore_capture_panel(app: &AppHandle) {
    let state = app.state::<AppState>();
    if state.panel_stepped_aside.swap(false, Ordering::SeqCst) {
        if let Some(w) = app.get_webview_window(CAPTURE_PANEL) {
            let _ = w.show();
        }
    }
}

/// "Hide XcreenShot when capturing" (CAP-023): while a capture is taken the OS leaves
/// our windows out of it (Windows display affinity, macOS sharing type), without
/// hiding them on screen. Only during the capture, so screen sharing in meeting apps
/// still shows XcreenShot.
pub fn exclude_from_capture(app: &AppHandle, exclude: bool) {
    for w in app.webview_windows().values() {
        // Selection windows and the countdown are always excluded.
        if w.label().starts_with(crate::overlay::LABEL_PREFIX) || w.label() == crate::countdown::LABEL {
            continue;
        }
        let _ = w.set_content_protected(exclude);
    }
}

pub fn apply_theme(app: &AppHandle) {
    let t = theme(app);
    for w in app.webview_windows().values() {
        let _ = w.set_theme(t);
    }
}

/// Retitles open windows after a language change.
pub fn retitle(app: &AppHandle) {
    let lang = app.state::<AppState>().core.lang();
    if let Some(w) = app.get_webview_window(MAIN) {
        let _ = w.set_title(&xs_core::i18n::tr(lang, "app.name", &[]));
    }
    if let Some(w) = app.get_webview_window(PREFERENCES) {
        let _ = w.set_title(&xs_core::i18n::tr(lang, "ui.prefs.title", &[]));
    }
    if let Some(w) = app.get_webview_window(CAPTURE_PANEL) {
        let _ = w.set_title(&xs_core::i18n::tr(lang, "ui.panel.title", &[]));
    }
}

/// Called after a window closed or hid. Hides the Dock icon when nothing is visible
/// and quits if "keep running in the background" is off (APP-004).
pub fn after_window_gone(app: &AppHandle, closing_label: &str) {
    let visible = app
        .webview_windows()
        .values()
        .any(|w| w.label() != closing_label && w.is_visible().unwrap_or(false));
    let main_visible = app
        .get_webview_window(MAIN)
        .is_some_and(|w| w.label() != closing_label && w.is_visible().unwrap_or(false));
    dock(app, main_visible);
    if !visible
        && !app
            .state::<AppState>()
            .core
            .settings
            .get_bool("general.keepRunning")
    {
        tracing::info!("last window closed and keepRunning is off: quitting");
        // Through the command, so unsaved documents are asked about first (APP-004).
        let core = app.state::<AppState>().core.clone();
        std::thread::spawn(move || {
            let _ = core.call(xs_core::Source::Ui, "app.quit", json!({}), None);
        });
    }
}

pub fn close_focused(app: &AppHandle) {
    if let Some(w) = app
        .webview_windows()
        .values()
        .find(|w| w.is_focused().unwrap_or(false))
    {
        let _ = w.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCREEN: Area = Area {
        x: 0,
        y: 0,
        w: 1920,
        h: 1040,
    };

    /// CAP-034: the panel goes beside the area, on the side with most room.
    #[test]
    fn the_scroll_panel_goes_on_the_side_with_most_room() {
        let area = Area {
            x: 100,
            y: 100,
            w: 900,
            h: 700,
        };
        assert_eq!(scroll_panel_place(area, (300, 520), 12, &[SCREEN]), (1012, 100));
        let right = Area { x: 1000, ..area };
        assert_eq!(scroll_panel_place(right, (300, 520), 12, &[SCREEN]), (688, 100));
    }

    /// With no room beside the area, another monitor; with none, a corner.
    #[test]
    fn a_full_screen_area_sends_the_panel_elsewhere() {
        let other = Area {
            x: 1920,
            y: 0,
            w: 1280,
            h: 1000,
        };
        assert_eq!(
            scroll_panel_place(SCREEN, (300, 520), 12, &[SCREEN, other]),
            (2888, 12)
        );
        assert_eq!(scroll_panel_place(SCREEN, (300, 520), 12, &[SCREEN]), (1608, 508));
    }
}
