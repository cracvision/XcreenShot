//! The camera of a recording (VID-031 to VID-038): opened while the recording bar shows
//! and the camera is on, with its preview over the area; the recording draws its
//! frames where the preview shows them. It is let go when the bar closes, so the
//! system's camera light goes off.

use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode};
use xs_video::camlook::{Border, Corner, Look, Placement, Shape};

use crate::record;

/// The cameras that are plugged in, at most 2 s old (VID-043).
type Cameras = Vec<CameraInfo>;

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct CameraInfo {
    pub id: String,
    pub name: String,
}

fn list() -> Cameras {
    #[cfg(windows)]
    {
        xs_video::camera::cameras()
            .unwrap_or_default()
            .into_iter()
            .map(|c| CameraInfo {
                id: c.id,
                name: c.name,
            })
            .collect()
    }
    #[cfg(not(windows))]
    {
        Vec::new()
    }
}

#[cfg(windows)]
struct Session {
    camera: Arc<xs_video::camera::Camera>,
    device: Option<String>,
    preview: Option<xs_video::campreview::Preview>,
    area: xs_video::campreview::Area,
}

#[derive(Default)]
pub struct CameraService {
    core: Mutex<Weak<Core>>,
    cameras: Mutex<Option<(Instant, Cameras)>>,
    #[cfg(windows)]
    session: Mutex<Option<Session>>,
    /// Opening now, for this showing of the bar.
    opening: Mutex<Option<u64>>,
    /// Each showing of the bar has its number: an opening that ends after the bar
    /// closed lets the camera go at once.
    showing: std::sync::atomic::AtomicU64,
    /// Why the camera gives no picture (VID-038).
    trouble: Mutex<Option<&'static str>>,
}

fn service(core: &Core) -> Arc<CameraService> {
    core.service(CameraService::default)
}

/// Keeps the core for the camera's own threads (the preview's changes, the opening).
pub fn init(core: &Arc<Core>) {
    *service(core).core.lock().unwrap() = Arc::downgrade(core);
}

/// The cameras plugged in, at most 2 s old.
pub fn cameras(core: &Core) -> Cameras {
    let svc = service(core);
    let mut cache = svc.cameras.lock().unwrap();
    match cache.as_ref() {
        Some((at, l)) if at.elapsed() < Duration::from_secs(2) => l.clone(),
        _ => {
            let l = list();
            *cache = Some((Instant::now(), l.clone()));
            l
        }
    }
}

/// The camera is wanted: switched on, and allowed in videos (VID-031).
pub fn wanted(core: &Core) -> bool {
    core.settings.get_bool("video.camera") && core.settings.get_bool("video.cameraEnabled")
}

/// The chosen camera if it is plugged in; otherwise the first (VID-043).
fn device(core: &Core) -> Option<String> {
    let chosen = core.settings.get_str("video.cameraDevice");
    (!chosen.is_empty() && cameras(core).iter().any(|c| c.id == chosen)).then_some(chosen)
}

/// The camera's look, from the settings (VID-032 to VID-036).
pub fn look(core: &Core) -> Look {
    let s = &core.settings;
    let text = |k: &str| s.get_str(k);
    let number = |k: &str, d: f64| s.get(k).and_then(|v| v.as_f64()).unwrap_or(d);
    let color = |k: &str, d| xs_video::effects::parse_color(&text(k)).unwrap_or(d);
    Look {
        placement: if text("video.cameraPlacement") == "fullscreen" {
            Placement::Fullscreen
        } else {
            Placement::Pip
        },
        corner: Corner::parse(&text("video.cameraCorner")),
        size: number("video.cameraSize", 0.25) as f32,
        shape: Shape::parse(&text("video.cameraShape")),
        wide: text("video.cameraAspect") != "4:3",
        mirror: s.get_bool("video.cameraMirror"),
        border: Border::parse(&text("video.cameraBorder")),
        border_colors: (
            color("video.cameraBorderColor", [255, 255, 255]),
            color("video.cameraBorderColor2", [47, 107, 255]),
        ),
        border_angle: number("video.cameraBorderAngle", 0.0) as f32,
        border_width: number("video.cameraBorderWidth", 4.0) as f32,
    }
}

/// Follows the recording bar: `state` is what it shows (with its `area`), or `None`
/// when it closes. Opens the camera and its preview, keeps them over the area, gives
/// the recording its frames and look, and lets it go when not wanted.
pub fn sync(core: &Core, state: Option<&Value>) {
    #[cfg(windows)]
    sync_win(core, state);
    #[cfg(not(windows))]
    let _ = (core, state);
}

#[cfg(windows)]
fn sync_win(core: &Core, state: Option<&Value>) {
    use std::sync::atomic::Ordering;
    use xs_video::campreview::Area;
    let svc = service(core);
    let Some(state) = state.filter(|_| wanted(core)) else {
        if state.is_none() {
            svc.showing.fetch_add(1, Ordering::SeqCst);
            *svc.trouble.lock().unwrap() = None;
        }
        // Let go: the light goes off (VID-031).
        svc.session.lock().unwrap().take();
        record::with_recorder(core, |r| r.set_camera_on(false));
        return;
    };
    let n = |k: &str| state["area"][k].as_i64().unwrap_or(0) as i32;
    let area = Area {
        x: n("x"),
        y: n("y"),
        width: n("width"),
        height: n("height"),
        scale: record::area_scale(core) as f32,
    };
    if area.width < 2 || area.height < 2 {
        return;
    }
    let look = look(core);
    let device = device(core);
    let mut session = svc.session.lock().unwrap();
    // Another camera chosen (in Ready or paused): the one open goes.
    if session.as_ref().is_some_and(|s| s.device != device) {
        *session = None;
    }
    if let Some(s) = session.as_mut() {
        if s.area != area {
            s.area = area;
            if let Some(p) = &s.preview {
                p.set_area(area);
            }
        }
        if let Some(p) = &s.preview {
            p.set_look(look.clone());
        }
        let feed = fresh_feed(&s.camera);
        record::with_recorder(core, |r| {
            r.set_camera(Some(feed));
            r.set_camera_look(look);
            r.set_camera_on(true);
        });
        return;
    }
    drop(session);
    if cameras(core).is_empty() {
        return;
    }
    // Opened on a thread of its own: the bar does not wait for the camera.
    let showing = svc.showing.load(Ordering::SeqCst);
    {
        let mut opening = svc.opening.lock().unwrap();
        if *opening == Some(showing) {
            return;
        }
        *opening = Some(showing);
    }
    let weak = svc.core.lock().unwrap().clone();
    let state = state.clone();
    std::thread::spawn(move || {
        let opened = xs_video::camera::open(device.as_deref());
        let Some(core) = weak.upgrade() else { return };
        let svc = service(&core);
        *svc.opening.lock().unwrap() = None;
        if svc.showing.load(Ordering::SeqCst) != showing || !wanted(&core) {
            return;
        }
        match opened {
            Ok(camera) => {
                let camera = Arc::new(camera);
                let w = Arc::downgrade(&core);
                let preview =
                    xs_video::campreview::Preview::show(camera.feed(), area, look.clone(), move |change| {
                        if let Some(core) = w.upgrade() {
                            moved(&core, change);
                        }
                    });
                *svc.trouble.lock().unwrap() = None;
                *svc.session.lock().unwrap() = Some(Session {
                    camera,
                    device,
                    preview,
                    area,
                });
                // The recording, if it started meanwhile, gets it now.
                sync_win(&core, Some(&state));
            }
            Err(t) => {
                tracing::warn!(trouble = ?t, "camera: not available");
                use xs_video::camera::Trouble;
                *svc.trouble.lock().unwrap() = Some(match t {
                    Trouble::Denied => "blocked",
                    Trouble::InUse => "in_use",
                    Trouble::Gone => "gone",
                    Trouble::Failed => "failed",
                });
            }
        }
    });
}

/// The camera's frames for the recording, only while they keep coming: a camera that
/// stops leaves the video instead of freezing in it (VID-038).
#[cfg(windows)]
fn fresh_feed(camera: &Arc<xs_video::camera::Camera>) -> xs_video::camlook::Feed {
    let c = camera.clone();
    Arc::new(move || c.latest().filter(|f| f.at.elapsed() < Duration::from_secs(1)))
}

/// The preview was dragged to a corner or resized (VID-032): it stays as the setting,
/// and the recording follows from now on.
#[cfg(windows)]
fn moved(core: &Core, change: xs_video::campreview::Change) {
    use xs_video::campreview::Change;
    let (key, value) = match change {
        Change::Corner(c) => ("video.cameraCorner", json!(c.name())),
        Change::Size(s) => ("video.cameraSize", json!((s as f64 * 100.0).round() / 100.0)),
    };
    if core
        .settings
        .set(key, value.clone(), xs_core::Source::Ui, core.lang())
        .unwrap_or(false)
    {
        core.setting_changed(key, &value);
    }
    let look = look(core);
    record::with_recorder(core, |r| r.set_camera_look(look));
}

/// The camera now, for the bar (VID-011, VID-038): on, whether there is any, which
/// one, and why it gives no picture.
pub fn status(core: &Core) -> Value {
    let svc = service(core);
    let list = cameras(core);
    let device = device(core);
    let name = device
        .as_ref()
        .and_then(|d| list.iter().find(|c| &c.id == d))
        .or(list.first())
        .map(|c| c.name.clone());
    let chosen = core.settings.get_str("video.cameraDevice");
    #[cfg(windows)]
    let (open, black) = {
        let session = svc.session.lock().unwrap();
        let frame = session.as_ref().and_then(|s| s.camera.latest());
        // A closed privacy shutter gives black pictures (VID-038).
        let black = frame.as_ref().is_some_and(|f| {
            let n = (f.bgra.len() / 4 / 97).max(1) as u64;
            let sum: u64 = f.bgra.chunks(4).step_by(97).map(|p| p[1] as u64).sum();
            sum / n < 6
        });
        (session.is_some(), black)
    };
    #[cfg(not(windows))]
    let (open, black) = (false, false);
    let trouble = (*svc.trouble.lock().unwrap()).or(black.then_some("black"));
    json!({
        "on": wanted(core),
        "enabled": core.settings.get_bool("video.cameraEnabled"),
        "available": !list.is_empty(),
        "open": open,
        "device": name,
        "missing": !chosen.is_empty() && !list.iter().any(|c| c.id == chosen),
        "trouble": trouble,
        "placement": core.settings.get_str("video.cameraPlacement"),
    })
}

/// `record.cameras` (VID-031): the cameras plugged in and the one chosen.
pub fn list_cmd(core: &Core) -> CommandResult {
    Ok(json!({
        "cameras": cameras(core),
        "chosen": core.settings.get_str("video.cameraDevice"),
    }))
}

/// `record.camera` (VID-031 to VID-036): the camera on or off, its device (in Ready or
/// paused), placement, corner, size, shape, proportion, mirror and border, as the
/// settings and, while recording, from now on. `retry` opens it again (VID-038).
pub fn set(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    if p.get("device").is_some() && record::recording_now(core) {
        return Err(CommandError::new(
            ErrorCode::Busy,
            tr(lang, "errors.camera_device_recording", &[]),
        ));
    }
    let put = |key: &str, v: Value| -> Result<(), CommandError> {
        if core.settings.set(key, v.clone(), ctx.source, lang)? {
            core.setting_changed(key, &v);
        }
        Ok(())
    };
    for (param, key) in [
        ("on", "video.camera"),
        ("mirror", "video.cameraMirror"),
        ("enabled", "video.cameraEnabled"),
    ] {
        if let Some(v) = p.get(param).and_then(Value::as_bool) {
            put(key, json!(v))?;
        }
    }
    for (param, key) in [
        ("device", "video.cameraDevice"),
        ("placement", "video.cameraPlacement"),
        ("corner", "video.cameraCorner"),
        ("shape", "video.cameraShape"),
        ("aspect", "video.cameraAspect"),
        ("border", "video.cameraBorder"),
        ("borderColor", "video.cameraBorderColor"),
        ("borderColor2", "video.cameraBorderColor2"),
    ] {
        if let Some(v) = p.get(param).filter(|v| v.is_string()) {
            put(key, v.clone())?;
        }
    }
    for (param, key) in [
        ("size", "video.cameraSize"),
        ("borderWidth", "video.cameraBorderWidth"),
        ("borderAngle", "video.cameraBorderAngle"),
    ] {
        if let Some(v) = p.get(param).filter(|v| v.is_number()) {
            put(key, v.clone())?;
        }
    }
    if p.get("retry") == Some(&json!(true)) {
        #[cfg(windows)]
        service(core).session.lock().unwrap().take();
        *service(core).trouble.lock().unwrap() = None;
    }
    record::refresh(core);
    Ok(status(core))
}

/// `record.camera_swap` (VID-035): picture in picture ⇄ full screen, also while
/// recording; the corner, size and shape stay for when it comes back.
pub fn swap(core: &Core, ctx: &Context) -> CommandResult {
    if !wanted(core) {
        return Err(CommandError::new(
            ErrorCode::NotFound,
            tr(ctx.lang, "errors.camera_off", &[]),
        ));
    }
    let next = if core.settings.get_str("video.cameraPlacement") == "fullscreen" {
        "pip"
    } else {
        "fullscreen"
    };
    set(core, ctx, &json!({ "placement": next }))
}
