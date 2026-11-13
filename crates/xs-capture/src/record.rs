//! Video recording commands (VID-024): `record.start`, `record.pause`,
//! `record.resume`, `record.stop`, `record.cancel` and `record.status`, over the engine
//! of `xs-video`. One recording at a time. While it runs, the icon's menu starts with
//! Stop recording (VID-020) and, when automation started it, a notification says so
//! (APP-032). The file goes to the library's month folder (VID-054).

use std::path::PathBuf;
use std::sync::{Arc, Mutex, Weak};
use std::time::Duration;

use serde_json::{json, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode, Lang, Source};
use xs_video::{Area, MaxSize, Quality, Recorder, State};

use crate::{monitors, platform, Rect};

struct Running {
    recorder: Recorder,
    source: Value,
    path: PathBuf,
    /// The area recorded (virtual-screen pixels), for its border and bar.
    rect: Rect,
}

/// An area chosen and waiting for Record (the *Ready* state, VID-002).
#[derive(Clone)]
struct Ready {
    rect: Rect,
    scale: f64,
    source: Value,
}

#[derive(Default)]
pub struct Recording {
    current: Mutex<Option<Running>>,
    ready: Mutex<Option<Ready>>,
    /// The countdown runs (VID-008).
    counting: std::sync::atomic::AtomicBool,
}

fn service(core: &Core) -> Arc<Recording> {
    core.service(Recording::default)
}

fn not_recording(lang: Lang) -> CommandError {
    CommandError::new(ErrorCode::NotFound, tr(lang, "errors.not_recording", &[]))
        .with_details(json!({ "reason": "not_recording" }))
}

/// Whether a recording runs (for the icon's clock and menu).
pub fn running(core: &Core) -> bool {
    service(core).current.lock().unwrap().is_some()
}

/// The recording bar and the area's border: shown with `Some(state)`, gone with
/// `None` (VID-011, VID-020).
fn bar(core: &Core, state: Option<Value>) {
    core.host().recording_bar(state);
}

/// The video size an area gives, as the settings say (VID-028).
fn planned(core: &Core, rect: Rect, scale: f64) -> (u32, u32) {
    let max = core
        .settings
        .get("video.maxSize")
        .and_then(|v| v.as_str().and_then(MaxSize::parse))
        .unwrap_or(MaxSize::ReduceHighDensity);
    xs_video::video_size(rect.width, rect.height, scale, max)
}

/// `record.video` (VID-002): during a recording it stops it; otherwise the area is
/// chosen on screen and the recording waits, Ready, for Record on its bar.
pub fn video(core: &Arc<Core>, ctx: &Context) -> CommandResult {
    if running(core) {
        return stop(core, ctx);
    }
    let svc = service(core);
    if let Some(r) = svc.ready.lock().unwrap().clone() {
        // Already Ready: show its bar again.
        bar(core, Some(ready_state(core, &r)));
        return Ok(ready_state(core, &r));
    }
    let (monitor, rect) = crate::interactive::pick_area(core, ctx)?;
    if rect.width < xs_video::MIN_AREA || rect.height < xs_video::MIN_AREA {
        return Err(CommandError::new(
            ErrorCode::InvalidParams,
            tr(ctx.lang, "errors.record_too_small", &[]),
        ));
    }
    let r = Ready {
        rect,
        scale: monitor.scale as f64,
        source: json!({ "type": "region", "rect": rect }),
    };
    *svc.ready.lock().unwrap() = Some(r.clone());
    let state = ready_state(core, &r);
    bar(core, Some(state.clone()));
    tell(core, ctx.source, "record.ready", state.clone());
    Ok(state)
}

fn ready_state(core: &Core, r: &Ready) -> Value {
    let (width, height) = planned(core, r.rect, r.scale);
    json!({
        "status": "ready",
        "elapsed": 0.0,
        "source": r.source,
        "area": r.rect,
        "width": width,
        "height": height,
        "cursor": core.settings.get_bool("video.cursor"),
    })
}

/// `record.toggle` (VID-012, VID-075): Record when Ready, Pause while recording,
/// Resume when paused; with nothing going on, Video capture.
pub fn toggle(core: &Arc<Core>, ctx: &Context) -> CommandResult {
    let state = service(core)
        .current
        .lock()
        .unwrap()
        .as_ref()
        .map(|r| r.recorder.state());
    match state {
        Some(State::Recording) => pause(core, ctx, true),
        Some(State::Paused) => pause(core, ctx, false),
        Some(_) => Ok(status(core)),
        None if service(core).ready.lock().unwrap().is_some() => start(core, ctx, &json!({})),
        None => video(core, ctx),
    }
}

/// The icon menu's entry while recording: (id, "Stop recording").
pub fn menu_items(core: &Core) -> Vec<(String, String)> {
    if running(core) {
        vec![("recording".into(), tr(core.lang(), "tray.stop_recording", &[]))]
    } else {
        Vec::new()
    }
}

/// The monitor holding the middle of `r`: its place and scale.
fn monitor_of(r: Rect) -> Option<(Rect, f64)> {
    let (cx, cy) = (r.x + r.width as i32 / 2, r.y + r.height as i32 / 2);
    monitors()
        .ok()?
        .into_iter()
        .map(|(_, i)| i)
        .find(|i| i.rect.contains(cx, cy))
        .map(|i| (i.rect, i.scale as f64))
}

/// The area `p` asks for (VID-024): `source` `screen` (a `monitor`: `primary`,
/// `pointer` or its number) or `region` (`x`, `y`, `width`, `height`, physical).
fn area(p: &Value, lang: Lang) -> Result<(Rect, f64, Value), CommandError> {
    let bad = |what: &str| CommandError::new(ErrorCode::InvalidParams, what.to_string());
    let list = monitors().map_err(|e| crate::failed(lang, e))?;
    match p.get("source").and_then(Value::as_str).unwrap_or("screen") {
        "screen" => {
            let which = p.get("monitor").cloned().unwrap_or(json!("primary"));
            let found = match &which {
                Value::Number(n) => list
                    .iter()
                    .find(|(_, i)| i.id as u64 == n.as_u64().unwrap_or(u64::MAX)),
                Value::String(s) if s == "pointer" => {
                    let (x, y) = platform::cursor_position().unwrap_or((0, 0));
                    list.iter().find(|(_, i)| i.rect.contains(x, y))
                }
                Value::String(s) if s == "primary" => list.iter().find(|(_, i)| i.primary),
                _ => return Err(bad("monitor: primary, pointer or a monitor's number")),
            }
            .or_else(|| list.first())
            .map(|(_, i)| i.clone())
            .ok_or_else(|| crate::failed(lang, "no monitor"))?;
            Ok((
                found.rect,
                found.scale as f64,
                json!({ "type": "screen", "monitor": found.id, "name": found.name }),
            ))
        }
        "region" => {
            let n = |k: &str| p.get(k).and_then(Value::as_f64);
            let (Some(x), Some(y), Some(w), Some(h)) = (n("x"), n("y"), n("width"), n("height")) else {
                return Err(bad("region: x, y, width and height"));
            };
            let r = Rect {
                x: x as i32,
                y: y as i32,
                width: w.max(0.0) as u32,
                height: h.max(0.0) as u32,
            };
            if r.width < xs_video::MIN_AREA || r.height < xs_video::MIN_AREA {
                return Err(CommandError::new(
                    ErrorCode::InvalidParams,
                    tr(lang, "errors.record_too_small", &[]),
                ));
            }
            let (monitor, scale) = monitor_of(r).ok_or_else(|| bad("region: outside every monitor"))?;
            // A recording stays on one monitor (VID-003).
            let r = r
                .intersect(&monitor)
                .ok_or_else(|| bad("region: outside every monitor"))?;
            Ok((r, scale, json!({ "type": "region", "rect": r })))
        }
        _ => Err(bad("source: screen or region")),
    }
}

/// Where the video goes: the path asked for, or the library's month folder.
fn file_path(core: &Core, p: &Value, lang: Lang) -> PathBuf {
    if let Some(path) = p.get("path").and_then(Value::as_str).filter(|s| !s.is_empty()) {
        return PathBuf::from(path);
    }
    let now = chrono::Local::now();
    let folder = xs_library::library(core)
        .root()
        .join(now.format("%Y-%m").to_string());
    let _ = std::fs::create_dir_all(&folder);
    let base = format!(
        "{} {}",
        tr(lang, "record.file_name", &[]),
        now.format("%Y-%m-%d %H.%M.%S")
    );
    let first = folder.join(format!("{base}.mp4"));
    if !first.exists() {
        return first;
    }
    (2..)
        .map(|n| folder.join(format!("{base} ({n}).mp4")))
        .find(|p| !p.exists())
        .unwrap_or(first)
}

fn status_of(r: &Running) -> Value {
    json!({
        "status": match r.recorder.state() {
            State::Recording => "recording",
            State::Paused => "paused",
            State::Finalizing => "finalizing",
            State::Done => "finalizing",
        },
        "elapsed": (r.recorder.recorded().as_secs_f64() * 10.0).round() / 10.0,
        "source": r.source,
        "area": r.rect,
        "path": r.path,
        "width": r.recorder.width,
        "height": r.recorder.height,
        "fps": r.recorder.fps,
        "cursor": r.recorder.cursor(),
    })
}

/// `record.cursor` (VID-044): shows or hides the pointer from now on; without `on`,
/// switches it. It also becomes the setting for the next recordings.
pub fn cursor(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let svc = service(core);
    let current = svc.current.lock().unwrap();
    let now = current
        .as_ref()
        .map(|r| r.recorder.cursor())
        .unwrap_or_else(|| core.settings.get_bool("video.cursor"));
    let on = p.get("on").and_then(Value::as_bool).unwrap_or(!now);
    if let Some(r) = current.as_ref() {
        r.recorder.set_cursor(on);
    }
    drop(current);
    let _ = core.settings.set("video.cursor", json!(on), ctx.source, ctx.lang);
    let state = status(core);
    if state["status"] != "idle" {
        bar(core, Some(state.clone()));
    }
    Ok(json!({ "cursor": on, "status": state["status"] }))
}

fn tell(core: &Core, source: Source, event: &str, data: Value) {
    core.events.publish(event, source, data.clone());
    core.host().recording_event(data);
}

/// `record.start`.
pub fn start(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let svc = service(core);
    let busy = || CommandError::new(ErrorCode::Busy, tr(lang, "errors.record_busy", &[]));
    if running(core) {
        return Err(busy());
    }
    // Without an area in `p`, the one chosen on screen (Ready).
    let asks_area = p.get("source").is_some() || p.get("x").is_some() || p.get("monitor").is_some();
    let ready = if asks_area {
        None
    } else {
        svc.ready.lock().unwrap().take()
    };
    // The countdown (VID-008): from the bar, the setting's; from automation only when
    // asked (VID-024). Esc goes back to Ready.
    let countdown = p.get("countdown").and_then(Value::as_f64).unwrap_or_else(|| {
        if ready.is_some() && !ctx.source.is_automation() {
            core.settings
                .get("video.countdown")
                .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok()))
                .unwrap_or(0.0)
        } else {
            0.0
        }
    });
    if countdown > 0.0 {
        if let Some(r) = &ready {
            let mut shown = ready_state(core, r);
            shown["status"] = json!("countdown");
            bar(core, Some(shown));
        }
        use std::sync::atomic::Ordering;
        svc.counting.store(true, Ordering::Relaxed);
        let done = core.host().countdown(countdown);
        svc.counting.store(false, Ordering::Relaxed);
        if !done {
            if let Some(r) = ready {
                bar(core, Some(ready_state(core, &r)));
                *svc.ready.lock().unwrap() = Some(r);
            }
            return Err(crate::cancelled(lang));
        }
    }
    let (rect, scale, source) = match ready {
        Some(r) => (r.rect, r.scale, r.source),
        None => area(p, lang)?,
    };
    let mut current = svc.current.lock().unwrap();
    if current.is_some() {
        return Err(busy());
    }
    let setting = |key: &str| {
        core.settings
            .get(key)
            .and_then(|v| v.as_str().map(str::to_string))
    };
    let quality = p
        .get("quality")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| setting("video.quality"))
        .and_then(|q| Quality::parse(&q))
        .unwrap_or(Quality::High);
    let max_size = p
        .get("maxSize")
        .and_then(Value::as_str)
        .map(str::to_string)
        .or_else(|| setting("video.maxSize"))
        .and_then(|m| MaxSize::parse(&m))
        .unwrap_or(MaxSize::ReduceHighDensity);
    let limit = p
        .get("limit")
        .and_then(Value::as_f64)
        .filter(|s| *s > 0.0)
        .map(Duration::from_secs_f64);
    let path = file_path(core, p, lang);
    let recorder = Recorder::start(xs_video::Options {
        area: Area {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: rect.height,
        },
        scale,
        quality,
        max_size,
        limit,
        software_only: core.settings.get_bool("video.softwareEncoding"),
        cursor: p
            .get("cursor")
            .and_then(Value::as_bool)
            .unwrap_or_else(|| core.settings.get_bool("video.cursor")),
        path: path.clone(),
    })
    .map_err(|e| {
        CommandError::new(
            ErrorCode::Internal,
            tr(lang, "errors.record_failed", &[("reason", &e)]),
        )
    })?;
    let running = Running {
        recorder,
        source,
        path,
        rect,
    };
    let status = status_of(&running);
    *current = Some(running);
    drop(current);
    bar(core, Some(status.clone()));
    tracing::info!(
        width = status["width"].as_u64(),
        height = status["height"].as_u64(),
        fps = status["fps"].as_u64(),
        "recording started"
    );
    // Automation never records without the user seeing it (APP-032).
    if ctx.source.is_automation() {
        core.host().notify(
            "automation",
            &tr(lang, "notify.recording_title", &[]),
            &tr(lang, "notify.recording_automation", &[]),
        );
    }
    tell(core, ctx.source, "record.started", status.clone());
    watch(Arc::downgrade(core), ctx.source);
    Ok(status)
}

/// Finishes the recording when it ends by itself (its time limit, or a failure).
fn watch(core: Weak<Core>, source: Source) {
    std::thread::spawn(move || loop {
        std::thread::sleep(Duration::from_millis(250));
        let Some(core) = core.upgrade() else { return };
        let svc = service(&core);
        let mut current = svc.current.lock().unwrap();
        match current.as_ref() {
            None => return,
            Some(r) if r.recorder.finished() => {
                let r = current.take().expect("checked");
                drop(current);
                let _ = finish(&core, source, r, false, core.lang());
                return;
            }
            Some(_) => {}
        }
    });
}

/// Stops (or cancels) `r` and reports the file.
fn finish(core: &Core, source: Source, r: Running, cancel: bool, lang: Lang) -> CommandResult {
    let summary = if cancel {
        r.recorder.cancel()
    } else {
        r.recorder.stop()
    };
    bar(core, None);
    let result = match summary {
        Ok(s) if s.cancelled => {
            tell(core, source, "record.cancelled", json!({ "status": "cancelled" }));
            return Ok(json!({ "status": "cancelled" }));
        }
        Ok(s) => s,
        Err(e) => {
            tell(
                core,
                source,
                "record.failed",
                json!({ "status": "failed", "reason": e }),
            );
            return Err(CommandError::new(
                ErrorCode::Internal,
                tr(lang, "errors.record_failed", &[("reason", &e)]),
            ));
        }
    };
    // The library picks the file up (VID-054).
    xs_library::library(core).scan();
    let value = json!({
        "status": "done",
        "path": result.path,
        "duration": (result.duration * 100.0).round() / 100.0,
        "width": result.width,
        "height": result.height,
        "fps": result.fps,
        "frames": result.frames,
        "bytes": result.bytes,
        "encoder": result.encoder,
    });
    tracing::info!(
        frames = result.frames,
        bytes = result.bytes,
        encoder = result.encoder,
        "recording saved"
    );
    core.host().notify_opening(
        "captures",
        &tr(lang, "notify.recording_saved_title", &[]),
        &tr(
            lang,
            "notify.recording_saved_body",
            &[("seconds", &format!("{:.0}", result.duration.round()))],
        ),
        json!({ "path": result.path, "folder": true }),
    );
    core.events
        .publish("file.written", source, json!({ "path": result.path }));
    tell(core, source, "record.stopped", value.clone());
    Ok(value)
}

fn take(core: &Core, lang: Lang) -> Result<Running, CommandError> {
    service(core)
        .current
        .lock()
        .unwrap()
        .take()
        .ok_or_else(|| not_recording(lang))
}

/// `record.stop`: waits for the file.
pub fn stop(core: &Core, ctx: &Context) -> CommandResult {
    let r = take(core, ctx.lang)?;
    finish(core, ctx.source, r, false, ctx.lang)
}

/// `record.cancel`: ends without a file (VID-014); when Ready, nothing is recorded.
pub fn cancel(core: &Core, ctx: &Context) -> CommandResult {
    if service(core).ready.lock().unwrap().take().is_some() {
        bar(core, None);
        tell(
            core,
            ctx.source,
            "record.cancelled",
            json!({ "status": "cancelled" }),
        );
        return Ok(json!({ "status": "cancelled" }));
    }
    let r = take(core, ctx.lang)?;
    finish(core, ctx.source, r, true, ctx.lang)
}

/// `record.pause` and `record.resume` (VID-012).
pub fn pause(core: &Core, ctx: &Context, pause: bool) -> CommandResult {
    let svc = service(core);
    let current = svc.current.lock().unwrap();
    let r = current.as_ref().ok_or_else(|| not_recording(ctx.lang))?;
    if pause {
        r.recorder.pause();
    } else {
        r.recorder.resume();
    }
    let status = status_of(r);
    drop(current);
    bar(core, Some(status.clone()));
    tell(
        core,
        ctx.source,
        if pause { "record.paused" } else { "record.resumed" },
        status.clone(),
    );
    Ok(status)
}

/// `record.status`.
pub fn status(core: &Core) -> Value {
    let svc = service(core);
    if let Some(r) = svc.current.lock().unwrap().as_ref() {
        return status_of(r);
    }
    if svc.counting.load(std::sync::atomic::Ordering::Relaxed) {
        return json!({ "status": "countdown" });
    }
    let ready = svc.ready.lock().unwrap().clone();
    match ready {
        Some(r) => ready_state(core, &r),
        None => json!({ "status": "idle" }),
    }
}
