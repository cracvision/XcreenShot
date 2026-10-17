//! On-screen selection windows (CAP-013, APP-048). One borderless, always-on-top
//! window per monitor, created hidden at startup so the crosshair appears fast. A
//! session shows them over the frozen frames, which the webviews read uncompressed
//! through the `xsframe` protocol, and waits for the user's choice.

use std::sync::mpsc;
use std::sync::Mutex;
use std::time::Instant;

use serde_json::{json, Value};
use tauri::http::{Request, Response};
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewUrl, WebviewWindowBuilder};
use xs_core::host::Frame;
use xs_core::CommandError;

use crate::AppState;

pub const LABEL_PREFIX: &str = "overlay-";

pub struct Session {
    pub seq: u64,
    pub frames: Vec<Frame>,
    pub started: Instant,
    pub shown: bool,
    pub done: Option<mpsc::Sender<Value>>,
}

#[derive(Default)]
pub struct Overlays {
    pub session: Mutex<Option<Session>>,
    pub seq: Mutex<u64>,
}

fn label(index: usize) -> String {
    format!("{LABEL_PREFIX}{index}")
}

/// Creates (hidden) one overlay window per monitor that does not have one yet.
pub fn prewarm(app: &AppHandle, count: usize) {
    for index in 0..count.max(1) {
        let label = label(index);
        if app.get_webview_window(&label).is_some() {
            continue;
        }
        let built = WebviewWindowBuilder::new(app, &label, WebviewUrl::App("index.html".into()))
            .title("XcreenShot")
            .decorations(false)
            .resizable(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .shadow(false)
            .focused(false)
            .visible(false)
            // Never part of any capture or screen share (CAP-023).
            .content_protected(true)
            // On Windows the frozen screen is painted by a native window underneath.
            .transparent(cfg!(windows))
            .build();
        if let Err(e) = built {
            tracing::error!(error = %e, label, "could not create the selection window");
        }
    }
}

/// `xsframe` protocol: `/<seq>/<index>` returns the raw RGBA of that monitor.
pub fn serve_frame(app: &AppHandle, request: &Request<Vec<u8>>) -> Response<Vec<u8>> {
    let parts: Vec<&str> = request.uri().path().trim_matches('/').split('/').collect();
    let (seq, index) = match parts.as_slice() {
        [seq, index] => (seq.parse::<u64>().ok(), index.parse::<usize>().ok()),
        _ => (None, None),
    };
    let overlays = &app.state::<AppState>().overlays;
    let body = overlays
        .session
        .lock()
        .unwrap()
        .as_ref()
        .filter(|s| Some(s.seq) == seq)
        .and_then(|s| index.and_then(|i| s.frames.get(i)))
        .map(|f| f.rgba.as_ref().clone());
    let builder = Response::builder().header("Access-Control-Allow-Origin", "*");
    match body {
        Some(bytes) => builder
            .header("Content-Type", "application/octet-stream")
            .body(bytes)
            .unwrap_or_default(),
        None => builder.status(404).body(Vec::new()).unwrap_or_default(),
    }
}

/// Blocks until the user finishes the selection (runs on a command thread).
pub fn select(app: &AppHandle, request: Value, frames: Vec<Frame>) -> Result<Value, CommandError> {
    let monitors = request["monitors"].as_array().cloned().unwrap_or_default();
    let started = Instant::now();
    let (tx, rx) = mpsc::channel();
    let state = app.state::<AppState>();
    let seq = {
        let mut s = state.overlays.seq.lock().unwrap();
        *s += 1;
        *s
    };
    *state.overlays.session.lock().unwrap() = Some(Session {
        seq,
        frames,
        started,
        shown: false,
        done: Some(tx),
    });

    // Windows: convert the frames for GDI here, in parallel and off the UI thread.
    #[cfg(windows)]
    let mut bgra: Vec<Option<Vec<u8>>> = {
        let state = app.state::<AppState>();
        let session = state.overlays.session.lock().unwrap();
        let frames = session.as_ref().map(|s| s.frames.clone()).unwrap_or_default();
        drop(session);
        std::thread::scope(|scope| {
            let jobs: Vec<_> = frames
                .iter()
                .map(|f| scope.spawn(move || xs_capture::interactive::to_bgra(&f.rgba)))
                .collect();
            jobs.into_iter().map(|j| j.join().ok()).collect()
        })
    };
    tracing::debug!(
        elapsed_ms = started.elapsed().as_millis() as u64,
        "frames converted"
    );

    // Window creation and placement belong to the UI thread.
    let (ready_tx, ready_rx) = mpsc::channel();
    let handle = app.clone();
    let req = request.clone();
    app.run_on_main_thread(move || {
        let ui_started = Instant::now();
        prewarm(&handle, monitors.len());
        for (index, monitor) in monitors.iter().enumerate() {
            let Some(w) = handle.get_webview_window(&label(index)) else {
                continue;
            };
            let r = &monitor["rect"];
            // Windows: paint the frozen screen natively under the selection (APP-048).
            #[cfg(windows)]
            {
                let x = r["x"].as_i64().unwrap_or(0) as i32;
                let y = r["y"].as_i64().unwrap_or(0) as i32;
                let rect = windows_sys::Win32::Foundation::RECT {
                    left: x,
                    top: y,
                    right: x + r["width"].as_i64().unwrap_or(1) as i32,
                    bottom: y + r["height"].as_i64().unwrap_or(1) as i32,
                };
                let width = r["width"].as_u64().unwrap_or(1) as u32;
                let height = r["height"].as_u64().unwrap_or(1) as u32;
                if let Some(pixels) = bgra.get_mut(index).and_then(Option::take) {
                    crate::freeze_win::show(index, rect, width, height, pixels);
                }
            }
            let _ = w.set_position(PhysicalPosition::new(
                r["x"].as_i64().unwrap_or(0) as i32,
                r["y"].as_i64().unwrap_or(0) as i32,
            ));
            let _ = w.set_size(PhysicalSize::new(
                r["width"].as_u64().unwrap_or(1) as u32,
                r["height"].as_u64().unwrap_or(1) as u32,
            ));
            let lang = handle.state::<AppState>().core.lang();
            let _ = w.emit(
                "xs://overlay-start",
                json!({ "seq": seq, "index": index, "lang": lang.code(), "nativeBackground": cfg!(windows), "request": req }),
            );
        }
        tracing::debug!(ui_ms = ui_started.elapsed().as_millis() as u64, "selection windows placed");
        let _ = ready_tx.send(());
    })
    .map_err(|e| CommandError::internal(e.to_string()))?;
    let _ = ready_rx.recv();

    let outcome = rx.recv().unwrap_or_else(|_| json!({ "kind": "cancelled" }));
    end(app);
    Ok(outcome)
}

/// An overlay drew its frame: show them all at once and focus the one under the pointer.
pub fn ready(app: &AppHandle, seq: u64, index: usize) {
    let state = app.state::<AppState>();
    let mut guard = state.overlays.session.lock().unwrap();
    let Some(session) = guard.as_mut().filter(|s| s.seq == seq) else {
        return;
    };
    if session.shown {
        return;
    }
    session.shown = true;
    let elapsed = session.started.elapsed().as_millis() as u64;
    let count = session.frames.len();
    drop(guard);
    tracing::info!(elapsed_ms = elapsed, first = index, "selection shown");
    for i in 0..count {
        if let Some(w) = app.get_webview_window(&label(i)) {
            let _ = w.show();
            let _ = w.set_always_on_top(true);
        }
    }
    if let Some(w) = app.get_webview_window(&label(index)) {
        let _ = w.set_focus();
    }
}

pub fn finish(app: &AppHandle, seq: u64, outcome: Value) {
    let state = app.state::<AppState>();
    let sender = state
        .overlays
        .session
        .lock()
        .unwrap()
        .as_mut()
        .filter(|s| s.seq == seq)
        .and_then(|s| s.done.take());
    if let Some(tx) = sender {
        let _ = tx.send(outcome);
    }
}

/// Hides the overlays and frees the frames.
fn end(app: &AppHandle) {
    *app.state::<AppState>().overlays.session.lock().unwrap() = None;
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        #[cfg(windows)]
        crate::freeze_win::hide_all();
        for (label, w) in handle.webview_windows() {
            if label.starts_with(LABEL_PREFIX) {
                let _ = w.hide();
                let _ = w.emit("xs://overlay-end", ());
            }
        }
    });
}

/// Cancels a running session (e.g. when the app quits).
pub fn cancel(app: &AppHandle) {
    let state = app.state::<AppState>();
    let seq = state.overlays.session.lock().unwrap().as_ref().map(|s| s.seq);
    if let Some(seq) = seq {
        finish(app, seq, json!({ "kind": "cancelled" }));
    }
}

/// Pixels around the pointer for the magnifier (CAP-015): `w`×`h` RGBA starting at
/// `x`,`y` in the monitor's own pixels; outside the frame is transparent.
pub fn patch(app: &AppHandle, seq: u64, index: usize, x: i32, y: i32, w: u32, h: u32) -> Vec<u8> {
    let (w, h) = (w.min(256), h.min(256));
    let mut out = vec![0u8; (w * h * 4) as usize];
    let state = app.state::<AppState>();
    let guard = state.overlays.session.lock().unwrap();
    let Some(frame) = guard
        .as_ref()
        .filter(|s| s.seq == seq)
        .and_then(|s| s.frames.get(index))
    else {
        return out;
    };
    for row in 0..h as i32 {
        let sy = y + row;
        if sy < 0 || sy >= frame.height as i32 {
            continue;
        }
        for col in 0..w as i32 {
            let sx = x + col;
            if sx < 0 || sx >= frame.width as i32 {
                continue;
            }
            let src = ((sy as usize) * frame.width as usize + sx as usize) * 4;
            let dst = ((row as usize) * w as usize + col as usize) * 4;
            out[dst..dst + 4].copy_from_slice(&frame.rgba[src..src + 4]);
        }
    }
    out
}
