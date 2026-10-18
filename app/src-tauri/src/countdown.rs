//! Delay countdown (CAP-048): a small counter at the bottom-right of the monitor under
//! the pointer. It never takes focus, so menus and tooltips the user opens meanwhile
//! stay open; Cancel or Esc ends the capture. It is never part of a capture.

use std::str::FromStr;
use std::sync::mpsc;
use std::sync::Mutex;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

use crate::AppState;

pub const LABEL: &str = "countdown";
/// Logical size of the counter and its distance to the work area edges.
const WIDTH: f64 = 248.0;
const HEIGHT: f64 = 76.0;
const MARGIN: f64 = 16.0;

#[derive(Default)]
pub struct Countdown {
    cancel: Mutex<Option<mpsc::Sender<()>>>,
    /// What the window shows: `{ "seconds", "endsAt" }` while running.
    current: Mutex<Option<Value>>,
}

/// Creates the (hidden) counter window so it appears without delay.
pub fn prewarm(app: &AppHandle) {
    if app.get_webview_window(LABEL).is_some() {
        return;
    }
    let built = WebviewWindowBuilder::new(app, LABEL, WebviewUrl::App("index.html".into()))
        .title("XcreenShot")
        .decorations(false)
        .resizable(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .shadow(true)
        .focused(false)
        // Clicking Cancel must not take the focus from the menu being prepared.
        .focusable(false)
        .visible(false)
        // Never part of any capture or screen share (CAP-023).
        .content_protected(true)
        .inner_size(WIDTH, HEIGHT)
        .build();
    if let Err(e) = built {
        tracing::error!(error = %e, "could not create the countdown window");
    }
}

/// Bottom-right corner of the work area of the monitor under the pointer.
fn place(app: &AppHandle) -> Option<(PhysicalPosition<i32>, PhysicalSize<u32>)> {
    let pointer = app.cursor_position().ok();
    let monitor = pointer
        .and_then(|p| app.monitor_from_point(p.x, p.y).ok().flatten())
        .or_else(|| app.primary_monitor().ok().flatten())?;
    let scale = monitor.scale_factor();
    let area = monitor.work_area();
    let size = PhysicalSize::new((WIDTH * scale) as u32, (HEIGHT * scale) as u32);
    let margin = (MARGIN * scale) as i32;
    let x = area.position.x + area.size.width as i32 - size.width as i32 - margin;
    let y = area.position.y + area.size.height as i32 - size.height as i32 - margin;
    Some((PhysicalPosition::new(x, y), size))
}

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Shows the countdown and blocks until it ends. `true` = time is up, `false` = cancelled.
pub fn run(app: &AppHandle, seconds: f64) -> bool {
    let seconds = seconds.max(0.0);
    let state = app.state::<AppState>();
    let (tx, rx) = mpsc::channel::<()>();
    *state.countdown.cancel.lock().unwrap() = Some(tx.clone());
    let info = json!({ "seconds": seconds, "endsAt": now_millis() + (seconds * 1000.0) as u64 });
    *state.countdown.current.lock().unwrap() = Some(info.clone());

    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        prewarm(&handle);
        let Some(w) = handle.get_webview_window(LABEL) else {
            return;
        };
        if let Some((pos, size)) = place(&handle) {
            // Position first: moving to a monitor with another scale resizes the window.
            let _ = w.set_position(pos);
            let _ = w.set_size(size);
        }
        let _ = w.emit("xs://countdown", &info);
        let _ = w.show();
    });
    // `Esc` cancels while the counter runs, whatever window has the focus.
    let esc = Shortcut::from_str("Escape").ok();
    if let Some(esc) = esc {
        let cancel = tx.clone();
        let registered = app.global_shortcut().on_shortcut(esc, move |_, _, event| {
            if event.state() == ShortcutState::Pressed {
                let _ = cancel.send(());
            }
        });
        if let Err(e) = registered {
            tracing::warn!(error = %e, "Esc is not available to cancel the countdown");
        }
    }

    let finished = rx.recv_timeout(Duration::from_secs_f64(seconds)).is_err();

    *state.countdown.cancel.lock().unwrap() = None;
    *state.countdown.current.lock().unwrap() = None;
    if let Some(esc) = esc {
        let _ = app.global_shortcut().unregister(esc);
    }
    let (done_tx, done_rx) = mpsc::channel();
    let handle = app.clone();
    let queued = app.run_on_main_thread(move || {
        if let Some(w) = handle.get_webview_window(LABEL) {
            let _ = w.hide();
        }
        let _ = done_tx.send(());
    });
    // The counter must be gone before the screens are frozen.
    if queued.is_ok() {
        let _ = done_rx.recv_timeout(Duration::from_millis(500));
    }
    tracing::info!(seconds, finished, "countdown ended");
    finished
}

/// What the counter shows right now (the window asks when it loads).
pub fn current(app: &AppHandle) -> Option<Value> {
    app.state::<AppState>().countdown.current.lock().unwrap().clone()
}

/// Cancel button of the counter, or the icon menu.
pub fn cancel(app: &AppHandle) {
    if let Some(tx) = app.state::<AppState>().countdown.cancel.lock().unwrap().as_ref() {
        let _ = tx.send(());
    }
}
