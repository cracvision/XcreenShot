//! The edge launcher (CAP-004): a thin tab on the top, left or right edge of a monitor.
//! The pointer over it unfolds Capture and Presets; it is dragged along the edges, also
//! to another monitor, and remembers where it is. It never shows in a capture, and it
//! steps aside while an app fills its monitor (a presentation, a video).
//!
//! The window is small and changes size with what it shows: the tab, the buttons, or
//! the buttons and the presets. Dragging is the app's own (the pointer's position is
//! read here), so the drop can snap to the nearest edge.

use std::sync::Mutex;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};

use crate::AppState;

pub const LAUNCHER: &str = "launcher";
/// Where the launcher is kept between runs.
const MEMORY_KEY: &str = "capture.launcher";
/// Sizes in logical pixels: the tab, and the unfolded launcher.
/// Shorter than the open launcher is tall, so the pointer stays on it as it opens.
const TAB_LONG: f64 = 48.0;
const TAB_THIN: f64 = 12.0;
const OPEN_WIDTH: f64 = 236.0;
const OPEN_HEIGHT: f64 = 52.0;
const ROW: f64 = 34.0;
/// How often a full-screen app is looked for.
const WATCH: Duration = Duration::from_millis(1500);

/// The launcher's place: a monitor, an edge and where along it (0 to 1, its center).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Place {
    pub monitor: String,
    pub edge: String,
    pub at: f64,
}

impl Default for Place {
    fn default() -> Self {
        Place {
            monitor: String::new(),
            edge: "top".into(),
            at: 0.5,
        }
    }
}

/// How the launcher looks now: `tab`, `open` or `list` (with this many rows).
#[derive(Debug, Clone, Copy)]
enum Mode {
    Tab,
    Open,
    List(u32),
}

#[derive(Default)]
pub struct LauncherState {
    /// The pointer's offset in the window while it is dragged.
    grab: Mutex<Option<(i32, i32)>>,
    /// Hidden because an app fills its monitor.
    stepped_aside: Mutex<bool>,
}

fn state(app: &AppHandle) -> &LauncherState {
    app.state::<LauncherState>().inner()
}

fn wanted(app: &AppHandle) -> bool {
    let core = &app.state::<AppState>().core;
    matches!(
        core.settings.get_str("capture.interface").as_str(),
        "launcher" | "both"
    )
}

fn place(app: &AppHandle) -> Place {
    app.state::<AppState>()
        .core
        .recall(MEMORY_KEY)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default()
}

/// The monitor of a place: its name, or the primary when that one is gone (CAP-004).
fn monitor_of(app: &AppHandle, place: &Place) -> Option<(tauri::Monitor, bool)> {
    let monitors = app.available_monitors().ok()?;
    if let Some(m) = monitors
        .iter()
        .find(|m| m.name().is_some_and(|n| *n == place.monitor))
    {
        return Some((m.clone(), true));
    }
    app.primary_monitor()
        .ok()
        .flatten()
        .or_else(|| monitors.into_iter().next())
        .map(|m| (m, false))
}

/// Size and position (physical) of the launcher in `mode` at `place` on `monitor`.
fn geometry(
    place: &Place,
    monitor: &tauri::Monitor,
    mode: Mode,
) -> (PhysicalPosition<i32>, PhysicalSize<u32>) {
    let scale = monitor.scale_factor();
    let (mx, my) = (monitor.position().x, monitor.position().y);
    let (mw, mh) = (monitor.size().width as i32, monitor.size().height as i32);
    let px = |v: f64| (v * scale).round() as i32;
    let vertical = place.edge != "top";
    let (w, h) = match mode {
        Mode::Tab if vertical => (px(TAB_THIN), px(TAB_LONG)),
        Mode::Tab => (px(TAB_LONG), px(TAB_THIN)),
        Mode::Open => (px(OPEN_WIDTH), px(OPEN_HEIGHT)),
        Mode::List(rows) => (
            px(OPEN_WIDTH),
            px(OPEN_HEIGHT + 8.0 + ROW * rows.clamp(1, 10) as f64),
        ),
    };
    let at = place.at.clamp(0.0, 1.0);
    let (x, y) = match place.edge.as_str() {
        "left" => (
            mx,
            my + ((mh as f64 * at) as i32 - h / 2).clamp(0, (mh - h).max(0)),
        ),
        "right" => (
            mx + mw - w,
            my + ((mh as f64 * at) as i32 - h / 2).clamp(0, (mh - h).max(0)),
        ),
        _ => (
            mx + ((mw as f64 * at) as i32 - w / 2).clamp(0, (mw - w).max(0)),
            my,
        ),
    };
    (
        PhysicalPosition::new(x, y),
        PhysicalSize::new(w.max(1) as u32, h.max(1) as u32),
    )
}

fn apply(app: &AppHandle, mode: Mode) -> Option<String> {
    let w = app.get_webview_window(LAUNCHER)?;
    let mut place = place(app);
    let (monitor, found) = monitor_of(app, &place)?;
    if !found {
        // Its monitor is gone: the top edge of the primary one (CAP-004).
        place = Place {
            monitor: monitor.name().cloned().unwrap_or_default(),
            ..Place::default()
        };
    }
    let (pos, size) = geometry(&place, &monitor, mode);
    let _ = w.set_size(size);
    let _ = w.set_position(pos);
    Some(place.edge)
}

/// Opens or closes the launcher as Preferences > Capture > Capture interface says.
pub fn sync(app: &AppHandle) {
    let on = wanted(app);
    let existing = app.get_webview_window(LAUNCHER);
    match (existing, on) {
        (Some(w), false) => {
            let _ = w.destroy();
        }
        (None, true) => {
            let built = WebviewWindowBuilder::new(app, LAUNCHER, WebviewUrl::App("index.html".into()))
                .title("XcreenShot")
                .decorations(false)
                .transparent(true)
                .shadow(false)
                .resizable(false)
                .always_on_top(true)
                .skip_taskbar(true)
                .focused(false)
                .visible(false)
                // Never in a capture (CAP-023).
                .content_protected(true)
                .theme(crate::windows::theme(app))
                .build();
            match built {
                Ok(w) => {
                    apply(app, Mode::Tab);
                    let _ = w.show();
                }
                Err(e) => tracing::warn!(error = %e, "edge launcher not opened"),
            }
        }
        _ => {}
    }
}

/// Starts the launcher if it is on, and the watch for full-screen apps.
pub fn start(app: &AppHandle) {
    app.manage(LauncherState::default());
    sync(app);
    let app = app.clone();
    std::thread::spawn(move || loop {
        std::thread::sleep(WATCH);
        let a = app.clone();
        let _ = app.run_on_main_thread(move || step_aside(&a));
    });
}

/// Hides the launcher while an app fills its monitor; shows it again after.
fn step_aside(app: &AppHandle) {
    let Some(w) = app.get_webview_window(LAUNCHER) else {
        return;
    };
    let Ok(pos) = w.outer_position() else { return };
    let full = fullscreen::monitor_filled()
        .is_some_and(|(x, y, r, b)| pos.x >= x && pos.x < r && pos.y >= y && pos.y < b);
    let mut aside = state(app).stepped_aside.lock().unwrap();
    if full && !*aside {
        let _ = w.hide();
        *aside = true;
    } else if !full && *aside {
        let _ = w.show();
        *aside = false;
    }
}

/// `xs_launcher`: the launcher as a tab, open, or open with `rows` presets; returns
/// its edge.
#[tauri::command]
pub fn xs_launcher(app: AppHandle, mode: String, rows: Option<u32>) -> Value {
    let mode = match mode.as_str() {
        "open" => Mode::Open,
        "list" => Mode::List(rows.unwrap_or(1)),
        _ => Mode::Tab,
    };
    json!({ "edge": apply(&app, mode) })
}

/// `xs_launcher_grab`: a drag starts; the pointer's offset in the window is kept.
#[tauri::command]
pub fn xs_launcher_grab(app: AppHandle) {
    let (Some(w), Ok(p)) = (app.get_webview_window(LAUNCHER), app.cursor_position()) else {
        return;
    };
    if let Ok(pos) = w.outer_position() {
        *state(&app).grab.lock().unwrap() = Some((p.x as i32 - pos.x, p.y as i32 - pos.y));
    }
}

/// `xs_launcher_drag`: the launcher follows the pointer.
#[tauri::command]
pub fn xs_launcher_drag(app: AppHandle) {
    let grab = *state(&app).grab.lock().unwrap();
    let (Some(w), Ok(p), Some((dx, dy))) = (app.get_webview_window(LAUNCHER), app.cursor_position(), grab)
    else {
        return;
    };
    let _ = w.set_position(PhysicalPosition::new(p.x as i32 - dx, p.y as i32 - dy));
}

/// `xs_launcher_drop`: the drag ends; the launcher sticks to the nearest edge (top,
/// left or right) of the monitor under the pointer, and remembers it.
#[tauri::command]
pub fn xs_launcher_drop(app: AppHandle) -> Value {
    state(&app).grab.lock().unwrap().take();
    let Ok(p) = app.cursor_position() else {
        return json!({});
    };
    let Some(monitor) = app
        .monitor_from_point(p.x, p.y)
        .ok()
        .flatten()
        .or_else(|| app.primary_monitor().ok().flatten())
    else {
        return json!({});
    };
    let (mx, my) = (monitor.position().x as f64, monitor.position().y as f64);
    let (mw, mh) = (monitor.size().width as f64, monitor.size().height as f64);
    let (fx, fy) = (
        ((p.x - mx) / mw).clamp(0.0, 1.0),
        ((p.y - my) / mh).clamp(0.0, 1.0),
    );
    let place = nearest_edge(fx, fy, monitor.name().cloned().unwrap_or_default());
    app.state::<AppState>()
        .core
        .remember(MEMORY_KEY, serde_json::to_value(&place).unwrap_or_default());
    json!({ "edge": apply(&app, Mode::Open) })
}

/// The edge nearest to a point (fractions of its monitor), among top, left and right.
pub fn nearest_edge(fx: f64, fy: f64, monitor: String) -> Place {
    let (edge, at) = if fy <= fx.min(1.0 - fx) {
        ("top", fx)
    } else if fx < 1.0 - fx {
        ("left", fy)
    } else {
        ("right", fy)
    };
    Place {
        monitor,
        edge: edge.into(),
        at,
    }
}

#[cfg(windows)]
mod fullscreen {
    use windows_sys::Win32::Foundation::RECT;
    use windows_sys::Win32::Graphics::Gdi::{
        GetMonitorInfoW, MonitorFromWindow, MONITORINFO, MONITOR_DEFAULTTONEAREST,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{GetClassNameW, GetForegroundWindow, GetWindowRect};

    /// The monitor (left, top, right, bottom) that the foreground app fills, if any.
    /// The desktop and the taskbar do not count.
    pub fn monitor_filled() -> Option<(i32, i32, i32, i32)> {
        // SAFETY: plain queries on the foreground window; a null one returns early.
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                return None;
            }
            let mut class = [0u16; 64];
            let n = GetClassNameW(hwnd, class.as_mut_ptr(), class.len() as i32);
            let class = String::from_utf16_lossy(&class[..n.max(0) as usize]);
            if matches!(
                class.as_str(),
                "Progman" | "WorkerW" | "Shell_TrayWnd" | "Shell_SecondaryTrayWnd"
            ) {
                return None;
            }
            let mut r = RECT {
                left: 0,
                top: 0,
                right: 0,
                bottom: 0,
            };
            if GetWindowRect(hwnd, &mut r) == 0 {
                return None;
            }
            let mut mi: MONITORINFO = std::mem::zeroed();
            mi.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
            if GetMonitorInfoW(MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST), &mut mi) == 0 {
                return None;
            }
            let m = mi.rcMonitor;
            (r.left <= m.left && r.top <= m.top && r.right >= m.right && r.bottom >= m.bottom)
                .then_some((m.left, m.top, m.right, m.bottom))
        }
    }
}

#[cfg(not(windows))]
mod fullscreen {
    pub fn monitor_filled() -> Option<(i32, i32, i32, i32)> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_drop_sticks_to_the_nearest_edge() {
        assert_eq!(nearest_edge(0.5, 0.02, "A".into()).edge, "top");
        assert_eq!(nearest_edge(0.03, 0.5, "A".into()).edge, "left");
        assert_eq!(nearest_edge(0.97, 0.4, "A".into()).edge, "right");
        // Near the bottom it goes to the closest side, never the bottom.
        assert_eq!(nearest_edge(0.2, 0.95, "A".into()).edge, "left");
        let p = nearest_edge(0.97, 0.4, "A".into());
        assert!((p.at - 0.4).abs() < 1e-9);
    }
}
