//! Video recording commands (VID-024): `record.start`, `record.pause`,
//! `record.resume`, `record.stop`, `record.cancel` and `record.status`, over the engine
//! of `xs-video`. One recording at a time. While it runs, the icon's menu starts with
//! Stop recording (VID-020) and, when automation started it, a notification says so
//! (APP-032). The file goes to the library's month folder (VID-054).

use std::path::PathBuf;
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

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
    /// Open in the editor when it stops (VID-052).
    open: bool,
    /// What Restart goes back to (VID-014).
    again: Ready,
    /// When Record was pressed or the countdown ended (VID-030).
    pressed: std::time::Instant,
    /// Its name in the recovery folder (VID-022).
    rid: String,
}

/// Open in (VID-052): `open` is `editor` or `none`; without it, the Video tab says, and
/// automation never opens windows unless asked (APP-030).
fn opens_editor(core: &Core, source: Source, p: &Value) -> bool {
    match p.get("open").and_then(Value::as_str) {
        Some(o) => o == "editor",
        None => !source.is_automation() && crate::panel::load(core).video.open == "editor",
    }
}

/// An area chosen and waiting for Record (the *Ready* state, VID-002).
#[derive(Clone)]
struct Ready {
    rect: Rect,
    scale: f64,
    source: Value,
    /// Stops by itself after this long (the Video tab's timer, VID-015).
    limit: Option<Duration>,
    /// The window followed (VID-005).
    window: Option<u32>,
    /// Open in the editor when it stops (VID-052).
    open: bool,
}

#[derive(Default)]
pub struct Recording {
    current: Mutex<Option<Running>>,
    ready: Mutex<Option<Ready>>,
    /// The countdown runs (VID-008).
    counting: std::sync::atomic::AtomicBool,
    /// Recordings being written in the recovery folder, from before their engine
    /// starts until they end: Recover work leaves them alone (VID-022).
    live: Mutex<Vec<String>>,
    /// The computer kept awake while the bar shows (VID-021).
    awake: Mutex<Option<xs_video::StayAwake>>,
    /// The recording paused because the computer went to sleep (VID-021).
    slept: std::sync::atomic::AtomicBool,
    /// The meters of Ready (VID-011): the sound measured, never kept.
    meter: Mutex<Option<xs_video::Meter>>,
    /// The inputs, looked at again every 2 s (VID-043).
    inputs: Mutex<Option<(Instant, Inputs)>>,
    /// The system's privacy blocks the microphone (VID-077), looked at every 2 s.
    mic_blocked: Mutex<Option<(Instant, bool)>>,
}

/// The sound inputs that are on, and the default one's id.
type Inputs = (Vec<xs_video::Input>, Option<String>);

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
    // VID-021: awake while the bar shows: Ready, the countdown, recording, paused.
    {
        let svc = service(core);
        let mut awake = svc.awake.lock().unwrap();
        if state.is_none() {
            *awake = None;
        } else if awake.is_none() {
            *awake = xs_video::stay_awake();
        }
    }
    core.host().recording_bar(state);
}

/// The computer goes to sleep, or wakes, during a recording (VID-021): it pauses; on
/// waking the bar shows it paused, and a notice says why.
pub fn power_changed(core: &Core, resumed: bool) {
    use std::sync::atomic::Ordering;
    let svc = service(core);
    if !resumed {
        let current = svc.current.lock().unwrap();
        if let Some(r) = current
            .as_ref()
            .filter(|r| r.recorder.state() == State::Recording)
        {
            r.recorder.pause();
            svc.slept.store(true, Ordering::SeqCst);
            tracing::info!("recording paused: the computer is going to sleep");
        }
        return;
    }
    if !svc.slept.swap(false, Ordering::SeqCst) {
        return;
    }
    let status = svc.current.lock().unwrap().as_ref().map(status_of);
    if let Some(status) = status {
        bar(core, Some(status));
        let lang = core.lang();
        core.host().notify(
            "problems",
            &tr(lang, "app.name", &[]),
            &tr(lang, "notify.recording_slept", &[]),
        );
    }
}

/// The cursor and keyboard effects as the settings say (VID-045 to VID-047).
pub fn effects(core: &Core) -> xs_video::effects::Effects {
    use xs_video::effects::*;
    let s = &core.settings;
    let text = |k: &str| {
        s.get(k)
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_default()
    };
    let number = |k: &str, d: f64| s.get(k).and_then(|v| v.as_f64()).unwrap_or(d);
    let color = |k: &str, d: Rgb| parse_color(&text(k)).unwrap_or(d);
    let highlight = s.get_bool("video.highlight").then(|| Highlight {
        color: color("video.highlightColor", [255, 208, 0]),
        diameter: number("video.highlightSize", 48.0) as f32,
        opacity: number("video.highlightOpacity", 40.0) as f32 / 100.0,
        ring: text("video.highlightShape") == "ring",
    });
    let clicks = s.get_bool("video.clicks").then(|| {
        let primary = color("video.clickColor", [47, 111, 222]);
        Clicks {
            primary,
            secondary: if s.get_bool("video.clickSameColor") {
                primary
            } else {
                color("video.clickColorSecondary", [242, 140, 40])
            },
            diameter: match text("video.clickSize").as_str() {
                "small" => 40.0,
                "large" => 96.0,
                _ => 64.0,
            },
        }
    });
    let keys = s.get_bool("video.keys").then(|| Keys {
        all: text("video.keysWhich") == "all",
        place: match text("video.keysPlace").as_str() {
            "left" => KeyPlace::Left,
            "right" => KeyPlace::Right,
            _ => KeyPlace::Center,
        },
        size: match text("video.keysSize").as_str() {
            "small" => 14.0,
            "large" => 26.0,
            _ => 18.0,
        },
        hidden: own_shortcuts(core),
    });
    Effects {
        highlight,
        clicks,
        keys,
    }
}

/// The recording's own shortcuts, never shown as keys pressed (VID-047).
fn own_shortcuts(core: &Core) -> Vec<String> {
    let platform = core.shortcuts.platform();
    core.shortcuts
        .bindings(core.profile())
        .into_iter()
        .filter(|(slot, _)| slot.command.starts_with("record."))
        .map(|(_, acc)| xs_video::effects::normalize(&acc.display(platform)))
        .collect()
}

/// Stores a setting and, when it changes, tells the windows (the panel's Video tab
/// shows the same switches).
fn store(core: &Core, ctx: &Context, key: &str, value: Value) -> Result<(), CommandError> {
    if core.settings.set(key, value.clone(), ctx.source, ctx.lang)? {
        core.setting_changed(key, &value);
    }
    Ok(())
}

/// `record.effects` (VID-045 to VID-047): switches the cursor and keyboard effects,
/// also while recording; they stay as the settings for the next recordings. A colour
/// switches its effect on (VID-045), and the pointer's effects switch the pointer on
/// (VID-044).
pub fn set_effects(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let put = |key: &str, value: Value| store(core, ctx, key, value);
    for (param, key, effect) in [
        ("highlightColor", "video.highlightColor", "video.highlight"),
        ("clickColor", "video.clickColor", "video.clicks"),
    ] {
        if let Some(color) = p.get(param).filter(|v| v.is_string()) {
            put(key, color.clone())?;
            put(effect, json!(true))?;
        }
    }
    for (param, key) in [
        ("highlight", "video.highlight"),
        ("clicks", "video.clicks"),
        ("keys", "video.keys"),
    ] {
        if let Some(v) = p.get(param).and_then(Value::as_bool) {
            put(key, json!(v))?;
        }
    }
    let pointer_fx = ["highlight", "clicks", "highlightColor", "clickColor"]
        .iter()
        .any(|k| p.get(*k).is_some_and(|v| v != &json!(false)));
    let fx = effects(core);
    let svc = service(core);
    let current = svc.current.lock().unwrap();
    if pointer_fx {
        put("video.cursor", json!(true))?;
        if let Some(r) = current.as_ref() {
            r.recorder.set_cursor(true);
        }
    }
    if let Some(r) = current.as_ref() {
        r.recorder.set_effects(fx.clone());
    }
    drop(current);
    let state = status(core);
    if state["status"] != "idle" {
        bar(core, Some(state.clone()));
    }
    Ok(json!({
        "highlight": fx.highlight.is_some(),
        "clicks": fx.clicks.is_some(),
        "keys": fx.keys.is_some(),
        "cursor": core.settings.get_bool("video.cursor"),
    }))
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
/// chosen on screen (or, with `selection: fullscreen`, the monitor under the pointer,
/// VID-004) and the recording waits, Ready, for Record on its bar.
pub fn video(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    if running(core) {
        return stop(core, ctx);
    }
    if crate::steps_rec::running(core) {
        return Err(CommandError::new(
            ErrorCode::Busy,
            tr(ctx.lang, "errors.steps_busy", &[]),
        ));
    }
    let svc = service(core);
    if let Some(r) = svc.ready.lock().unwrap().clone() {
        // Already Ready: show its bar again.
        bar(core, Some(ready_state(core, &r)));
        return Ok(ready_state(core, &r));
    }
    let limit = p
        .get("limit")
        .and_then(Value::as_f64)
        .filter(|s| *s > 0.0)
        .map(Duration::from_secs_f64);
    if p.get("selection").and_then(Value::as_str) == Some("window") {
        // A window chosen on screen, followed while it moves (VID-005).
        let id = crate::interactive::pick_window(core, ctx)?;
        let (rect, scale, source) = window_area(id, ctx.lang)?;
        let r = Ready {
            rect,
            scale,
            source,
            limit,
            window: Some(id),
            open: opens_editor(core, ctx.source, p),
        };
        *svc.ready.lock().unwrap() = Some(r.clone());
        warm(core, &r);
        let state = ready_state(core, &r);
        bar(core, Some(state.clone()));
        tell(core, ctx.source, "record.ready", state.clone());
        return start_at_once(core, ctx, state);
    }
    if p.get("selection").and_then(Value::as_str) == Some("fullscreen") {
        let (rect, scale, source) = area(&json!({ "source": "screen", "monitor": "pointer" }), ctx.lang)?;
        let r = Ready {
            rect,
            scale,
            source,
            limit,
            window: None,
            open: opens_editor(core, ctx.source, p),
        };
        *svc.ready.lock().unwrap() = Some(r.clone());
        warm(core, &r);
        let state = ready_state(core, &r);
        bar(core, Some(state.clone()));
        tell(core, ctx.source, "record.ready", state.clone());
        return start_at_once(core, ctx, state);
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
        limit,
        window: None,
        open: opens_editor(core, ctx.source, p),
    };
    *svc.ready.lock().unwrap() = Some(r.clone());
    warm(core, &r);
    let state = ready_state(core, &r);
    bar(core, Some(state.clone()));
    tell(core, ctx.source, "record.ready", state.clone());
    start_at_once(core, ctx, state)
}

/// The inputs now, at most 2 s old (VID-043: they follow what is plugged in).
fn inputs(core: &Core) -> Inputs {
    let svc = service(core);
    let mut cache = svc.inputs.lock().unwrap();
    match cache.as_ref() {
        Some((at, list)) if at.elapsed() < Duration::from_secs(2) => list.clone(),
        _ => {
            let list = xs_video::inputs().unwrap_or_default();
            *cache = Some((Instant::now(), list.clone()));
            list
        }
    }
}

/// Whether the system's privacy settings block the microphone (VID-077), at most 2 s old.
fn mic_blocked(core: &Core) -> bool {
    let svc = service(core);
    let mut cache = svc.mic_blocked.lock().unwrap();
    match *cache {
        Some((at, blocked)) if at.elapsed() < Duration::from_secs(2) => blocked,
        _ => {
            let blocked = core.host().permissions().iter().any(|p| {
                p.permission == xs_core::host::Permission::Microphone
                    && p.state == xs_core::host::PermissionState::Denied
            });
            *cache = Some((Instant::now(), blocked));
            blocked
        }
    }
}

/// The microphone a recording takes (VID-039): asked for in `p`, or as the settings
/// say; its device and automatic level from the settings. A device that is not
/// plugged in gives the system's default (VID-043).
fn microphone_of(core: &Core, p: &Value) -> Option<xs_video::Microphone> {
    let on = p
        .get("microphone")
        .and_then(Value::as_bool)
        .unwrap_or_else(|| core.settings.get_bool("video.microphone"));
    on.then(|| {
        let device = p
            .get("microphoneDevice")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| core.settings.get_str("video.microphoneDevice"));
        let (list, _) = inputs(core);
        xs_video::Microphone {
            device: Some(device).filter(|d| !d.is_empty() && list.iter().any(|i| &i.id == d)),
            auto_level: core.settings.get_bool("video.microphoneAutoLevel"),
        }
    })
}

/// The meters and what the warnings need (VID-011, VID-019, VID-043).
fn levels_json(core: &Core, l: xs_video::mix::Levels, mic_on: bool, system_on: bool, track: bool) -> Value {
    let (list, default) = inputs(core);
    let chosen = core.settings.get_str("video.microphoneDevice");
    let missing = !chosen.is_empty() && !list.iter().any(|i| i.id == chosen);
    let using = if chosen.is_empty() || missing {
        default
    } else {
        Some(chosen)
    };
    let input = using.and_then(|id| list.iter().find(|i| i.id == id));
    let mic = l.microphone.unwrap_or_default();
    let system = l.system.unwrap_or_default();
    let round = |v: f32| (v as f64 * 100.0).round() / 100.0;
    json!({
        "microphone": {
            "on": mic_on,
            "level": round(mic.level),
            "clip": mic.clip,
            "quietSeconds": round(mic.quiet_seconds),
            "muted": l.microphone_muted,
            "device": input.map(|i| i.name.clone()),
            "wireless": input.is_some_and(|i| i.wireless),
            "missing": missing,
            "none": list.is_empty(),
            "blocked": mic_on && mic_blocked(core),
        },
        "systemAudio": { "on": system_on, "level": round(system.level), "clip": system.clip },
        "track": track,
    })
}

/// `record.level` (VID-011, VID-019, VID-039, VID-040): the meters now, in Ready too
/// (the sound is measured to try it, never kept: APP-054) and while recording.
pub fn level(core: &Core) -> Value {
    let svc = service(core);
    if let Some(r) = svc.current.lock().unwrap().as_ref() {
        let mut v = levels_json(
            core,
            r.recorder.levels(),
            r.recorder.microphone().is_some(),
            r.recorder.system_audio(),
            r.recorder.audio,
        );
        v["status"] = json!(if r.recorder.state() == State::Paused {
            "paused"
        } else {
            "recording"
        });
        return v;
    }
    let ready = svc.ready.lock().unwrap().is_some();
    let mut meter = svc.meter.lock().unwrap();
    if !ready {
        *meter = None;
        return json!({ "status": "idle" });
    }
    let mic = microphone_of(core, &json!({}));
    let system = core.settings.get_bool("video.systemAudio");
    let stale = meter
        .as_ref()
        .is_none_or(|m| !m.running() || m.microphone != mic || m.system_audio != system);
    if stale {
        *meter = (mic.is_some() || system).then(|| xs_video::Meter::start(mic.clone(), system));
    }
    let l = meter.as_ref().map(|m| m.levels()).unwrap_or_default();
    drop(meter);
    let mut v = levels_json(core, l, mic.is_some(), system, true);
    v["status"] = json!("ready");
    v
}

/// `record.sound` (VID-011, VID-039, VID-040): the system's sound and the microphone
/// (on or off, its device, its automatic level), as the settings for the next
/// recordings and, while recording, from now on. The microphone's device changes
/// only in Ready or paused, so the voice is not cut.
pub fn sound(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let svc = service(core);
    let current = svc.current.lock().unwrap();
    if let Some(r) = current.as_ref() {
        if p.get("microphoneDevice").is_some() && r.recorder.state() == State::Recording {
            return Err(CommandError::new(
                ErrorCode::Busy,
                tr(lang, "errors.mic_device_recording", &[]),
            ));
        }
        let switches_on = ["systemAudio", "microphone"]
            .iter()
            .any(|k| p.get(*k) == Some(&json!(true)));
        if switches_on && !r.recorder.audio {
            return Err(CommandError::new(
                ErrorCode::Busy,
                tr(lang, "errors.no_sound_track", &[]),
            ));
        }
    }
    for (param, key) in [
        ("systemAudio", "video.systemAudio"),
        ("microphone", "video.microphone"),
        ("microphoneAutoLevel", "video.microphoneAutoLevel"),
    ] {
        if let Some(v) = p.get(param).and_then(Value::as_bool) {
            store(core, ctx, key, json!(v))?;
        }
    }
    if let Some(d) = p.get("microphoneDevice").and_then(Value::as_str) {
        store(core, ctx, "video.microphoneDevice", json!(d))?;
    }
    if let Some(r) = current.as_ref() {
        r.recorder
            .set_system_audio(core.settings.get_bool("video.systemAudio"));
        r.recorder.set_microphone(microphone_of(core, &json!({})));
    }
    drop(current);
    let state = status(core);
    if state["status"] != "idle" {
        bar(core, Some(state.clone()));
    }
    Ok(json!({
        "systemAudio": core.settings.get_bool("video.systemAudio"),
        "microphone": core.settings.get_bool("video.microphone"),
        "microphoneDevice": core.settings.get_str("video.microphoneDevice"),
        "microphoneAutoLevel": core.settings.get_bool("video.microphoneAutoLevel"),
    }))
}

/// `record.sound_settings` (VID-039, VID-019): the system's sound settings, where the
/// microphone is unmuted and its volume set.
pub fn sound_settings(core: &Core) -> CommandResult {
    let uri = if cfg!(target_os = "macos") {
        "x-apple.systempreferences:com.apple.preference.sound?input"
    } else {
        "ms-settings:sound"
    };
    core.host().open_uri(uri)?;
    Ok(json!({ "uri": uri }))
}

/// `record.devices` (VID-039): the microphones that are on, the default one, and the
/// one chosen.
pub fn devices(core: &Core, ctx: &Context) -> CommandResult {
    let (inputs, default) = xs_video::inputs().map_err(|e| {
        CommandError::new(
            ErrorCode::Internal,
            tr(ctx.lang, "errors.record_failed", &[("reason", &e)]),
        )
    })?;
    Ok(json!({
        "inputs": inputs,
        "default": default,
        "chosen": core.settings.get_str("video.microphoneDevice"),
    }))
}

/// Where recordings are written while they record (VID-022).
pub fn recovery_dir(core: &Core) -> PathBuf {
    core.paths.local_dir.join("recovery").join("video")
}

/// Recording `rid` ended: Recover work may look at what it left.
fn ended(core: &Core, rid: &str) {
    service(core).live.lock().unwrap().retain(|r| r != rid);
}

/// The files of recording `rid` in the recovery folder go.
fn forget_recovery(core: &Core, rid: &str) {
    let dir = recovery_dir(core);
    let video = dir.join(format!("{rid}.mp4"));
    let _ = std::fs::remove_file(xs_video::time_file(&video));
    let _ = std::fs::remove_file(video);
    let _ = std::fs::remove_file(dir.join(format!("{rid}.json")));
}

/// Recordings the app did not finish (VID-022): each becomes an item of Recover
/// work, with its date, length and first frame. One with nothing readable goes. A
/// recording under way, or one already offered, is left alone.
pub fn recover_leftovers(core: &Arc<Core>) {
    let live = service(core).live.lock().unwrap().clone();
    let dir = recovery_dir(core);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return;
    };
    let items = xs_editor::editor(core).recovery_dir();
    let files: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    for path in &files {
        let ext = path.extension().and_then(|x| x.to_str()).unwrap_or_default();
        let Some(rid) = path.file_stem().map(|s| s.to_string_lossy().into_owned()) else {
            continue;
        };
        let video = dir.join(format!("{rid}.mp4"));
        // Once per recording: by its video, or by its notes when it has none.
        let by_notes = ext == "json" && !video.exists();
        if !(ext == "mp4" || by_notes) || live.contains(&rid) || items.join(format!("{rid}.json")).exists() {
            continue;
        }
        let length = video
            .exists()
            .then(|| xs_video::recovered_length(&video))
            .flatten()
            .filter(|l| *l > 0.0);
        let Some(length) = length else {
            forget_recovery(core, &rid);
            continue;
        };
        let meta_file = dir.join(format!("{rid}.json"));
        let meta: Value = std::fs::read(&meta_file)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        let modified = std::fs::metadata(&video)
            .and_then(|m| m.modified())
            .map(chrono::DateTime::<chrono::Utc>::from)
            .unwrap_or_else(|_| chrono::Utc::now());
        let saved_at = meta["started"]
            .as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.with_timezone(&chrono::Utc))
            .unwrap_or(modified);
        let target = meta["path"].as_str().map(PathBuf::from);
        let name = target
            .as_ref()
            .and_then(|p| p.file_stem())
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_else(|| {
                let local = saved_at.with_timezone(&chrono::Local);
                format!("Video {}", local.format("%Y-%m-%d %H.%M.%S"))
            });
        let entry = xs_editor::recovery::Entry {
            id: rid.clone(),
            name,
            original_path: target,
            kind: "video".into(),
            format: "mp4".into(),
            saved_at,
            video: Some(video.clone()),
            duration: Some((length * 10.0).round() / 10.0),
            thumbnail: thumbnail_of(&video),
            open: meta["open"].as_bool(),
        };
        let _ = std::fs::create_dir_all(&items);
        if xs_editor::recovery::write_entry(&items, &entry).is_ok() {
            let _ = std::fs::remove_file(&meta_file);
            tracing::info!(seconds = length, "a recording cut short is in Recover work");
        }
    }
}

/// The first frame of a video, 160 px wide, as a PNG data URL.
fn thumbnail_of(video: &std::path::Path) -> Option<String> {
    use base64::Engine;
    let (w, h, mut px) = xs_video::frame_at(video, 0.0).ok()?;
    for p in px.as_chunks_mut::<4>().0 {
        p.swap(0, 2);
        p[3] = 255;
    }
    let image = image::RgbaImage::from_raw(w, h, px)?;
    let small = image::imageops::thumbnail(&image, 160, (160 * h / w.max(1)).max(1));
    let mut png = Vec::new();
    small
        .write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
        .ok()?;
    Some(format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(png)
    ))
}

/// Gets the engine ready while the bar waits for Record (VID-030: the start).
fn warm(core: &Core, r: &Ready) {
    xs_video::warm_up(
        Area {
            x: r.rect.x,
            y: r.rect.y,
            width: r.rect.width,
            height: r.rect.height,
        },
        core.settings.get_bool("video.softwareEncoding"),
    );
}

/// Where XcreenShot notes that it hid the desktop's icons, to show them again even after
/// a crash (VID-018).
const ICONS_KEY: &str = "video.iconsHidden";

/// Hides the desktop's icons for a recording, with the preference (VID-018); only when
/// they were showing, so icons the user hid stay hidden.
fn hide_icons(core: &Core) {
    if !core.settings.get_bool("video.hideDesktopIcons") || core.recall(ICONS_KEY).is_some() {
        return;
    }
    if platform::desktop_icons(false) == Some(true) {
        core.remember(ICONS_KEY, json!(true));
    }
}

/// Shows the icons hidden for a recording, where they were (VID-018).
pub fn restore_icons(core: &Core) {
    if core.recall(ICONS_KEY).is_some() {
        let _ = platform::desktop_icons(true);
        core.forget(ICONS_KEY);
    }
}

/// Start recording without waiting (VID-007): with the preference, Ready goes on to
/// the countdown and the recording by itself, as if Record were pressed.
fn start_at_once(core: &Arc<Core>, ctx: &Context, ready: Value) -> CommandResult {
    // A clean desktop already in Ready, to see the result (VID-018).
    hide_icons(core);
    if !core.settings.get_bool("video.startAtOnce") {
        return Ok(ready);
    }
    start(core, ctx, &json!({}))
}

fn ready_state(core: &Core, r: &Ready) -> Value {
    let (width, height) = planned(core, r.rect, r.scale);
    // The summary (VID-006): frames, codec and how long fits on the disk (VID-023),
    // at the most the quality's bit rate writes.
    let quality = core
        .settings
        .get("video.quality")
        .and_then(|v| v.as_str().and_then(Quality::parse))
        .unwrap_or(Quality::High);
    let per_minute = quality.bitrate(width, height) as u64 / 8 * 60;
    let folder = xs_library::library(core).root().to_path_buf();
    let minutes = xs_video::free_space(&folder.join("x"))
        .map(|free| free.saturating_sub(xs_video::MIN_FREE) / per_minute.max(1));
    json!({
        "status": "ready",
        "elapsed": 0.0,
        "source": r.source,
        "area": r.rect,
        "width": width,
        "height": height,
        "fps": quality.fps(),
        "codec": "H.264",
        "minutesFree": minutes,
        // Smaller than the area's pixels (VID-028), and where it goes after (VID-052).
        "reduced": width < r.rect.width || height < r.rect.height,
        "areaWidth": r.rect.width,
        "areaHeight": r.rect.height,
        "open": if r.open { "editor" } else { "none" },
        "cursor": core.settings.get_bool("video.cursor"),
        "clicks": core.settings.get_bool("video.clicks"),
        "systemAudio": core.settings.get_bool("video.systemAudio"),
        "microphone": core.settings.get_bool("video.microphone"),
        "audio": true,
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
        None => video(core, ctx, &json!({})),
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

/// The monitor of the recording, or of the area waiting to be recorded: live drawing
/// covers it (VID-048).
pub(crate) fn drawing_monitor(core: &Core) -> Option<(Rect, f64)> {
    let svc = service(core);
    let current = svc.current.lock().unwrap().as_ref().map(|r| r.rect);
    let rect = current.or_else(|| svc.ready.lock().unwrap().as_ref().map(|r| r.rect))?;
    monitor_of(rect)
}

/// The monitor under the pointer: its place and scale.
pub(crate) fn pointer_monitor() -> Option<(Rect, f64)> {
    let (x, y) = platform::cursor_position().unwrap_or((0, 0));
    monitors()
        .ok()?
        .into_iter()
        .map(|(_, i)| i)
        .find(|i| i.rect.contains(x, y))
        .map(|i| (i.rect, i.scale as f64))
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

/// A window's area (VID-005): its place now, its monitor's scale, and the source.
fn window_area(id: u32, lang: Lang) -> Result<(Rect, f64, Value), CommandError> {
    let w = xcap::Window::all()
        .ok()
        .and_then(|l| l.into_iter().find(|w| w.id().ok() == Some(id)))
        .ok_or_else(|| {
            CommandError::new(
                ErrorCode::NotFound,
                tr(lang, "errors.window_not_found", &[("title", &id.to_string())]),
            )
        })?;
    let rect = Rect {
        x: w.x().unwrap_or(0),
        y: w.y().unwrap_or(0),
        width: w.width().unwrap_or(0),
        height: w.height().unwrap_or(0),
    };
    let (monitor, scale) = monitor_of(rect).ok_or_else(|| crate::failed(lang, "no monitor"))?;
    let rect = rect.intersect(&monitor).unwrap_or(rect);
    let title = w.title().unwrap_or_default();
    Ok((
        rect,
        scale,
        json!({ "type": "window", "id": id, "title": title, "rect": rect }),
    ))
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
        "clicks": r.recorder.effects().clicks.is_some(),
        // The sound (VID-011): what is on now; its meters come from record.level.
        "microphone": r.recorder.microphone().is_some(),
        "systemAudio": r.recorder.system_audio(),
        "audio": r.recorder.audio,
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
    let _ = store(core, ctx, "video.cursor", json!(on));
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
    // Video and steps never record at once (APP-028, SMT-022).
    if running(core) || crate::steps_rec::running(core) {
        return Err(busy());
    }
    // Without an area in `p`, the one chosen on screen (Ready).
    let asks_area = p.get("source").is_some()
        || p.get("x").is_some()
        || p.get("monitor").is_some()
        || p.get("window").is_some();
    let ready = if asks_area {
        None
    } else {
        svc.ready.lock().unwrap().take()
    };
    // The meters of Ready let go of the devices: the recording takes them.
    svc.meter.lock().unwrap().take();
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
    // VID-030: the start's delay counts from here, Record pressed or the countdown
    // over.
    let pressed = std::time::Instant::now();
    let ready_limit = ready.as_ref().and_then(|r| r.limit);
    let open = match &ready {
        Some(r) => r.open,
        None => opens_editor(core, ctx.source, p),
    };
    let again_of = ready.clone();
    let asked_window = p.get("window").and_then(Value::as_u64).map(|w| w as u32);
    let window = ready.as_ref().and_then(|r| r.window).or(asked_window);
    let (rect, scale, source) = match (ready, asked_window) {
        (Some(r), _) => (r.rect, r.scale, r.source),
        (None, Some(id)) => window_area(id, lang)?,
        (None, None) => area(p, lang)?,
    };
    // Not held while the engine starts: the icon's menu asks whether one runs.
    if running(core) {
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
        .map(Duration::from_secs_f64)
        .or(ready_limit);
    let path = file_path(core, p, lang);
    // VID-022: written into the recovery folder while it records, with what Recover
    // work needs if the app dies.
    let rid = xs_editor::recovery::new_id();
    svc.live.lock().unwrap().push(rid.clone());
    let rdir = recovery_dir(core);
    let _ = std::fs::create_dir_all(&rdir);
    let recovery_file = rdir.join(format!("{rid}.mp4"));
    let meta = json!({
        "path": path,
        "open": open,
        "started": chrono::Utc::now().to_rfc3339(),
    });
    let _ = xs_core::paths::write_atomic(
        &rdir.join(format!("{rid}.json")),
        &serde_json::to_vec_pretty(&meta).unwrap_or_default(),
    );
    // The panel and the editor stay out of the video, still on screen (VID-017),
    // from its first picture.
    let hide = core.settings.get_bool("capture.hideOnCapture");
    if hide {
        core.host().hide_for_capture(true);
    }
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
        system_audio: p
            .get("systemAudio")
            .and_then(Value::as_bool)
            .unwrap_or_else(|| core.settings.get_bool("video.systemAudio")),
        window: window.map(|w| w as isize),
        effects: effects(core),
        path: path.clone(),
        recovery: Some(recovery_file.clone()),
        microphone: microphone_of(core, p),
    })
    .map_err(|e| {
        if hide {
            core.host().hide_for_capture(false);
        }
        forget_recovery(core, &rid);
        ended(core, &rid);
        // From the bar: it stays Ready, to try again or cancel.
        if let Some(r) = &again_of {
            *svc.ready.lock().unwrap() = Some(r.clone());
            warm(core, r);
        }
        CommandError::new(
            ErrorCode::Internal,
            tr(lang, "errors.record_failed", &[("reason", &e)]),
        )
    })?;
    let again = again_of.unwrap_or(Ready {
        rect,
        scale,
        source: source.clone(),
        limit,
        window,
        open,
    });
    let running = Running {
        recorder,
        source,
        path,
        rect,
        open,
        again,
        pressed,
        rid,
    };
    let status = status_of(&running);
    {
        let mut current = svc.current.lock().unwrap();
        if current.is_some() {
            // Another one started meanwhile: this one goes.
            drop(current);
            let rid = running.rid.clone();
            let _ = running.recorder.cancel();
            forget_recovery(core, &rid);
            ended(core, &rid);
            return Err(busy());
        }
        *current = Some(running);
    }
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
    crate::livedraw::recording_ended(core);
    restore_icons(core);
    // The sound as it was at the end (VID-011: it can be switched while recording).
    let wants_audio = r.recorder.system_audio();
    let wants_mic = r.recorder.microphone().is_some();
    let open = r.open;
    let pressed = r.pressed;
    let rid = r.rid.clone();
    let summary = if cancel {
        r.recorder.cancel()
    } else {
        r.recorder.stop()
    };
    bar(core, None);
    if core.settings.get_bool("capture.hideOnCapture") {
        core.host().hide_for_capture(false);
    }
    // VID-022: a recording that ended (or was cancelled) leaves nothing to recover;
    // one that failed stays for Recover work if its file is there.
    if summary.is_ok() || !recovery_dir(core).join(format!("{rid}.mp4")).exists() {
        forget_recovery(core, &rid);
    }
    ended(core, &rid);
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
        "audio": result.audio,
        "microphone": result.microphone,
    });
    // VID-030: from Record to the first frame, and the frames lost.
    let startup_ms = result
        .first_frame_at
        .map(|t| t.saturating_duration_since(pressed).as_millis() as u64);
    let mut value = value;
    if let Some(ms) = startup_ms {
        value["startupMs"] = json!(ms);
    }
    value["lateFrames"] = json!(result.late_frames);
    let mut warnings = Vec::new();
    if result.window_closed {
        // The window it followed closed (VID-005).
        warnings.push(tr(lang, "notify.recording_window_closed", &[]));
    }
    if result.low_disk {
        // It stopped by itself: less than 1 GB was left (VID-023).
        warnings.push(tr(lang, "notify.recording_low_disk", &[]));
    }
    if wants_audio && !result.system_audio {
        // Asked for and missing: no sound output was on (VID-040, VID-043).
        warnings.push(tr(lang, "notify.recording_no_audio", &[]));
    }
    if wants_mic && !result.microphone {
        // Asked for and never there: no microphone was plugged in (VID-043).
        warnings.push(tr(lang, "notify.recording_no_mic", &[]));
    }
    if !warnings.is_empty() {
        value["warnings"] = json!(warnings);
    }
    tracing::info!(
        frames = result.frames,
        late_frames = result.late_frames,
        startup_ms = startup_ms,
        bytes = result.bytes,
        encoder = result.encoder,
        "recording saved"
    );
    // Open in (VID-052): the editor shows it on its first frame, ready to play; with
    // None it stays in the library and the tray, and the notice opens it.
    let opened = open
        && core
            .call(
                Source::Ui,
                "document.open",
                json!({ "path": result.path, "show": true }),
                Some(lang),
            )
            .map_err(|e| tracing::warn!(error = %e.message, "the video did not open"))
            .is_ok();
    value["opened"] = json!(opened);
    let mut body = tr(
        lang,
        "notify.recording_saved_body",
        &[("seconds", &format!("{:.0}", result.duration.round()))],
    );
    if let Some(w) = value["warnings"][0].as_str() {
        body = format!("{body} {w}");
    }
    if !opened || value.get("warnings").is_some() {
        core.host().notify_opening(
            "captures",
            &tr(lang, "notify.recording_saved_title", &[]),
            &body,
            json!({ "path": result.path, "folder": true }),
        );
    }
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
        service(core).meter.lock().unwrap().take();
        crate::livedraw::recording_ended(core);
        restore_icons(core);
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

/// `record.restart` (VID-014): drops what was recorded and goes back to Ready, same
/// area and settings; Record counts down again.
pub fn restart(core: &Core, ctx: &Context) -> CommandResult {
    let r = take(core, ctx.lang)?;
    let again = r.again.clone();
    let rid = r.rid.clone();
    let _ = r.recorder.cancel();
    forget_recovery(core, &rid);
    ended(core, &rid);
    if core.settings.get_bool("capture.hideOnCapture") {
        core.host().hide_for_capture(false);
    }
    *service(core).ready.lock().unwrap() = Some(again.clone());
    warm(core, &again);
    let state = ready_state(core, &again);
    bar(core, Some(state.clone()));
    tell(core, ctx.source, "record.ready", state.clone());
    Ok(state)
}

/// `record.image` (VID-010): an image of the area instead of a video. When Ready it
/// closes the recording; while recording it drops what was recorded first. The image
/// goes the normal way of a capture (CAP-054).
pub fn image(core: &Core, ctx: &Context) -> CommandResult {
    let svc = service(core);
    let ready = svc.ready.lock().unwrap().take();
    let rect = match ready {
        Some(r) => r.rect,
        None => {
            let r = take(core, ctx.lang)?;
            let rect = r.rect;
            let rid = r.rid.clone();
            let _ = r.recorder.cancel();
            forget_recovery(core, &rid);
            ended(core, &rid);
            if core.settings.get_bool("capture.hideOnCapture") {
                core.host().hide_for_capture(false);
            }
            rect
        }
    };
    crate::livedraw::recording_ended(core);
    restore_icons(core);
    bar(core, None);
    tell(
        core,
        ctx.source,
        "record.cancelled",
        json!({ "status": "cancelled" }),
    );
    core.call(
        ctx.source,
        "capture.region",
        json!({ "rect": rect, "open": "editor" }),
        Some(ctx.lang),
    )
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    /// VID-022: a recording the app did not finish (here the sample video stands for
    /// its file) is in Recover work with its length and first frame; Recover makes
    /// it an MP4 where it would have been saved and opens it; Discard removes its
    /// files; one with nothing readable is not offered.
    #[test]
    fn a_video_cut_short_is_offered_and_recovered() {
        let dir = tempfile::tempdir().unwrap();
        let core = Core::for_tests_with(
            xs_core::platform::Platform::Windows,
            vec![crate::extension(), xs_editor::extension()],
        );
        let lib = dir.path().join("lib");
        core.remember("library.location", json!(lib));
        xs_editor::editor(&core).set_folders(lib.clone(), dir.path().join("rec"));
        let sample =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/video/three-colors.mp4");
        let rdir = recovery_dir(&core);
        std::fs::create_dir_all(&rdir).unwrap();
        let leave = |rid: &str, target: &Path, open: bool| {
            std::fs::copy(&sample, rdir.join(format!("{rid}.mp4"))).unwrap();
            let meta = json!({"path": target, "open": open, "started": "2026-11-14T09:00:00+00:00"});
            std::fs::write(rdir.join(format!("{rid}.json")), meta.to_string()).unwrap();
        };
        let target = lib.join("2026-11").join("Video 2026-11-14 09.00.00.mp4");
        leave("a", &target, true);
        leave("b", &lib.join("other.mp4"), false);
        // Nothing readable: not offered, and its files go.
        std::fs::write(rdir.join("c.json"), "{}").unwrap();
        std::fs::write(rdir.join("c.mp4"), b"not a video").unwrap();
        // A recording under way (its file still empty) is left alone.
        service(&core).live.lock().unwrap().push("d".into());
        std::fs::write(rdir.join("d.json"), "{}").unwrap();
        std::fs::write(rdir.join("d.mp4"), b"").unwrap();
        // A video without its notes is offered too, named by its time.
        std::fs::copy(&sample, rdir.join("e.mp4")).unwrap();

        let call = |id: &str, p: Value| core.call(Source::Ui, id, p, Some(Lang::Es));
        let list = call("editor.recovery_list", json!({})).unwrap();
        let items = list["items"].as_array().unwrap();
        assert_eq!(items.len(), 3, "{list}");
        assert!(rdir.join("d.json").exists() && rdir.join("d.mp4").exists());
        let e = items.iter().find(|i| i["id"] == "e").unwrap();
        assert!(e["name"].as_str().unwrap().starts_with("Video "), "{e}");
        // Asked again: the same items, not twice.
        let again = call("editor.recovery_list", json!({})).unwrap();
        assert_eq!(again["items"].as_array().unwrap().len(), 3);
        let a = items.iter().find(|i| i["id"] == "a").unwrap();
        assert_eq!(a["kind"], "video");
        assert_eq!(a["name"], "Video 2026-11-14 09.00.00");
        assert!(a["duration"].as_f64().unwrap() > 0.5, "{a}");
        assert!(a["thumbnail"]
            .as_str()
            .unwrap()
            .starts_with("data:image/png;base64,"));
        assert!(!rdir.join("c.mp4").exists() && !rdir.join("c.json").exists());
        assert!(!rdir.join("a.json").exists(), "the item took its place");

        // Recover: the MP4 where it would have been saved, opened.
        let restored = call("editor.recovery_restore", json!({"ids": ["a"]})).unwrap();
        assert!(target.exists());
        assert!(restored["documents"][0].is_string(), "{restored}");
        assert!(!rdir.join("a.mp4").exists());
        // Discard: its files go.
        call("editor.recovery_discard", json!({"ids": ["b", "e"]})).unwrap();
        assert!(!rdir.join("b.mp4").exists() && !rdir.join("e.mp4").exists());
        assert!(call("editor.recovery_list", json!({})).unwrap()["items"]
            .as_array()
            .unwrap()
            .is_empty());
    }
}
