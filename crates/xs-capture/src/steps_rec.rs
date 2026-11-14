//! The step recorder's commands (SMT-022 to SMT-027, ADR 0009): `smart.steps_start`,
//! `_pause`, `_resume`, `_status`, `_undo`, `_stop` and `_cancel`. While it records, a
//! worker turns the clicks the hook hears into steps: the picture of the monitor as it
//! was at the press, what was clicked and its sentence. Each step is written at once to
//! a folder of its own in the recovery folder (`steps.json` and the pictures), so
//! memory does not grow with the steps and a crash loses nothing (SMT-027). Stop makes
//! the guide (SMT-028) from that folder.

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, Weak};

use serde_json::{json, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode, Lang, Source};

use crate::steps::{Framing, Step, Target};

/// The status of a recording.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The start notice is up: clicks do not count yet (SMT-022).
    Notice,
    Recording,
    Paused,
}

impl Status {
    fn as_str(self) -> &'static str {
        match self {
            Status::Notice => "notice",
            Status::Recording => "recording",
            Status::Paused => "paused",
        }
    }
}

/// What the commands and the worker share.
pub(crate) struct Shared {
    pub status: Status,
    pub steps: Vec<Step>,
    /// The click ids of the steps, for a double click (SMT-024).
    pub ids: Vec<u64>,
    pub next_n: u32,
    /// The last step already says that something was typed (SMT-024).
    pub typed: bool,
    /// Esc with steps: the bar asks first (SMT-023).
    pub confirm: bool,
    pub disk_full: bool,
    /// What the last steps were on, for a double click and for the text read later
    /// (SMT-024, SMT-025).
    pub named: Vec<Named>,
}

/// What a step was on: its target and the app when it changed.
pub(crate) struct Named {
    pub n: u32,
    pub target: Target,
    pub app: Option<String>,
}

/// One recording.
pub(crate) struct Session {
    pub shared: Mutex<Shared>,
    pub folder: PathBuf,
    pub lang: Lang,
    pub source: Source,
    pub framing: Framing,
    pub cursor: bool,
    pub keyboard: bool,
    pub open: String,
    pub copy: bool,
    pub title: Option<String>,
    /// "Record more steps here": the open guide and the section they go after (SMT-029).
    pub into: Option<(String, Option<String>)>,
    /// Scale of the pictures (the Scale image effect, SMT-022), 1 = as captured.
    pub scale: f64,
    pub started: chrono::DateTime<chrono::Local>,
    #[cfg(windows)]
    pub hook: Mutex<Option<crate::steps_win::Hook>>,
    #[cfg(windows)]
    pub worker: Mutex<Option<crate::steps_work::Worker>>,
}

#[derive(Default)]
pub struct Steps {
    current: Mutex<Option<Arc<Session>>>,
}

fn service(core: &Core) -> Arc<Steps> {
    core.service(Steps::default)
}

fn session(core: &Core) -> Option<Arc<Session>> {
    service(core).current.lock().unwrap().clone()
}

/// A step recording is going on (APP-028: video and steps never at once).
pub fn running(core: &Core) -> bool {
    session(core).is_some()
}

fn not_recording(lang: Lang) -> CommandError {
    CommandError::new(ErrorCode::NotRunning, tr(lang, "errors.steps_not_recording", &[]))
}

/// The folder of recordings in progress (SMT-027).
pub fn recovery_dir(core: &Core) -> PathBuf {
    core.paths.local_dir.join("recovery").join("steps")
}

/// What the bar, the icon and `smart.steps_status` show.
pub(crate) fn state_of(s: &Session) -> Value {
    let sh = s.shared.lock().unwrap();
    json!({
        "status": sh.status.as_str(),
        "steps": sh.steps.len(),
        "confirm": sh.confirm,
        "diskFull": sh.disk_full,
        // A hint from 200 steps on: several guides read better (SMT-027).
        "manySteps": sh.steps.len() >= 200,
        "cancelKey": "",
    })
}

/// Tells the bar, the icon and the event listeners.
pub(crate) fn tell(core: &Core, s: &Session, event: &str) {
    let mut state = state_of(s);
    state["cancelKey"] = json!(core.settings.get_str("smart.steps.cancelKey"));
    core.events.publish(event, s.source, state.clone());
    core.host().steps_bar(Some(state));
}

/// Writes the steps of a recording (SMT-027): replaced whole, atomically.
pub(crate) fn save(folder: &Path, steps: &[Step], meta: &Value) -> std::io::Result<()> {
    let body = json!({"meta": meta, "steps": steps});
    let tmp = folder.join("steps.json.tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(&body).unwrap_or_default())?;
    std::fs::rename(&tmp, folder.join("steps.json"))
}

pub(crate) fn meta_of(s: &Session) -> Value {
    json!({
        "started": s.started.to_rfc3339(),
        "title": s.title,
        "open": s.open,
        "lang": s.lang.code(),
    })
}

/// Reads a recording's folder: its steps and metadata (SMT-027).
pub fn read(folder: &Path) -> Result<(Vec<Step>, Value), String> {
    let bytes = std::fs::read(folder.join("steps.json")).map_err(|e| e.to_string())?;
    let v: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let steps: Vec<Step> = serde_json::from_value(v["steps"].clone()).map_err(|e| e.to_string())?;
    Ok((steps, v["meta"].clone()))
}

/// The scale of an enabled Scale image effect among the capture's effects (CAP-051).
fn scale_of(effects: &Value, size: (f64, f64)) -> f64 {
    let Some(list) = effects.as_array() else {
        return 1.0;
    };
    let mut k = 1.0;
    for e in list {
        if e["type"] != "scale" || e["enabled"] == json!(false) {
            continue;
        }
        let v = e["value"].as_f64().unwrap_or(100.0).max(1.0);
        k *= match e["mode"].as_str().unwrap_or("percent") {
            "longest" => v / size.0.max(size.1),
            "width" => v / size.0,
            "height" => v / size.1,
            _ => v / 100.0,
        };
    }
    k.clamp(0.05, 4.0)
}

fn bool_or(p: &Value, key: &str, default: bool) -> bool {
    p.get(key).and_then(Value::as_bool).unwrap_or(default)
}

/// `smart.steps_start` (SMT-022).
pub fn start(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let busy = || CommandError::new(ErrorCode::Busy, tr(lang, "errors.steps_busy", &[]));
    if running(core) || crate::record::running(core) {
        if crate::record::running(core) {
            core.host()
                .notify("steps", &tr(lang, "errors.steps_busy", &[]), "");
        }
        return Err(busy());
    }
    #[cfg(not(windows))]
    {
        let _ = p;
        Err(CommandError::new(
            ErrorCode::NotSupportedOnPlatform,
            tr(lang, "errors.steps_not_supported", &[]),
        ))
    }
    #[cfg(windows)]
    {
        let delay = p
            .get("delay")
            .and_then(Value::as_f64)
            .unwrap_or(0.0)
            .clamp(0.0, 60.0);
        if delay > 0.0 && !core.host().countdown(delay) {
            return Ok(json!({"status": "cancelled"}));
        }
        let area = p.get("area").and_then(Value::as_str).unwrap_or("all").to_string();
        let monitor = match area.as_str() {
            "monitor" => {
                let (x, y) = crate::platform::cursor_position().unwrap_or((0, 0));
                crate::monitors()
                    .ok()
                    .and_then(|list| list.into_iter().map(|(_, i)| i.rect).find(|r| r.contains(x, y)))
                    .map(|r| [r.x, r.y, r.width as i32, r.height as i32])
            }
            _ => None,
        };
        let size = crate::record::pointer_monitor()
            .map(|(r, _)| (r.width as f64, r.height as f64))
            .unwrap_or((1920.0, 1080.0));
        let started = chrono::Local::now();
        let folder = recovery_dir(core).join(started.format("%Y%m%d-%H%M%S").to_string());
        std::fs::create_dir_all(&folder).map_err(|e| CommandError::internal(e.to_string()))?;
        let title = p
            .get("title")
            .and_then(Value::as_str)
            .filter(|t| !t.trim().is_empty())
            .map(str::to_string);
        let s = Arc::new(Session {
            shared: Mutex::new(Shared {
                status: Status::Notice,
                steps: Vec::new(),
                ids: Vec::new(),
                next_n: 1,
                typed: false,
                confirm: false,
                disk_full: false,
                named: Vec::new(),
            }),
            folder: folder.clone(),
            lang,
            source: ctx.source,
            framing: Framing::parse(&core.settings.get_str("smart.steps.framing")),
            cursor: bool_or(p, "cursor", false),
            keyboard: core.settings.get_bool("smart.steps.keyboard"),
            // Quick annotation does not apply to a guide: it opens in the editor (SMT-022).
            open: match p.get("open").and_then(Value::as_str).unwrap_or("editor") {
                "none" => "none".into(),
                _ => "editor".into(),
            },
            copy: bool_or(p, "copy", false),
            title,
            into: p.get("into").and_then(|i| {
                Some((
                    i.get("documentId")?.as_str()?.to_string(),
                    i.get("after").and_then(Value::as_str).map(str::to_string),
                ))
            }),
            scale: scale_of(p.get("effects").unwrap_or(&Value::Null), size),
            started,
            hook: Mutex::new(None),
            worker: Mutex::new(None),
        });
        save(&folder, &[], &meta_of(&s)).map_err(|e| CommandError::internal(e.to_string()))?;
        let worker = crate::steps_work::Worker::start(Arc::downgrade(core), s.clone())
            .map_err(CommandError::internal)?;
        let filter = crate::steps_win::Filter {
            monitor,
            window: None,
            pick_window: area == "window",
        };
        let cancel = crate::steps_win::CancelKey::parse(&core.settings.get_str("smart.steps.cancelKey"));
        let hook = crate::steps_win::Hook::start(filter, s.keyboard, cancel, worker.sink())
            .map_err(CommandError::internal)?;
        *s.hook.lock().unwrap() = Some(hook);
        *s.worker.lock().unwrap() = Some(worker);
        *service(core).current.lock().unwrap() = Some(s.clone());
        // The notice for 2 s: clicks meanwhile do not count (SMT-022).
        core.host().steps_notice(true);
        tell(core, &s, "steps.started");
        let weak = Arc::downgrade(core);
        let notice_of = Arc::downgrade(&s);
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(2));
            if let (Some(core), Some(s)) = (weak.upgrade(), notice_of.upgrade()) {
                end_notice(&core, &s);
            }
        });
        tracing::info!(area, "step recording started");
        Ok(json!({"status": "notice", "steps": 0, "folder": folder}))
    }
}

fn end_notice(core: &Core, s: &Session) {
    let changed = {
        let mut sh = s.shared.lock().unwrap();
        let notice = sh.status == Status::Notice;
        if notice {
            sh.status = Status::Recording;
        }
        notice
    };
    if changed {
        core.host().steps_notice(false);
        tell(core, s, "steps.recording");
    }
}

/// The notice was clicked: it goes at once (SMT-022).
pub fn notice_closed(core: &Core) -> CommandResult {
    if let Some(s) = session(core) {
        end_notice(core, &s);
    }
    Ok(json!({}))
}

/// `smart.steps_pause` / `smart.steps_resume` (SMT-023).
pub fn pause(core: &Core, ctx: &Context, pause: bool) -> CommandResult {
    let s = session(core).ok_or_else(|| not_recording(ctx.lang))?;
    {
        let mut sh = s.shared.lock().unwrap();
        if sh.status != Status::Notice {
            sh.status = if pause { Status::Paused } else { Status::Recording };
            if !pause {
                sh.disk_full = false;
            }
        }
    }
    tell(core, &s, if pause { "steps.paused" } else { "steps.resumed" });
    Ok(state_of(&s))
}

/// `smart.steps_status` (SMT-022).
pub fn status(core: &Core) -> Value {
    match session(core) {
        Some(s) => state_of(&s),
        None => json!({"status": "idle", "steps": 0}),
    }
}

/// `smart.steps_undo`: the last step goes (SMT-023).
pub fn undo(core: &Core, ctx: &Context) -> CommandResult {
    let s = session(core).ok_or_else(|| not_recording(ctx.lang))?;
    let removed = {
        let mut sh = s.shared.lock().unwrap();
        let last = sh.steps.pop();
        sh.ids.pop();
        sh.typed = true;
        if let Some(step) = &last {
            let _ = save(&s.folder, &sh.steps, &meta_of(&s));
            let _ = std::fs::remove_file(s.folder.join(&step.image));
            if let Some(p) = &step.pointer {
                let _ = std::fs::remove_file(s.folder.join(&p.file));
            }
        }
        last.is_some()
    };
    tell(core, &s, "steps.changed");
    Ok(json!({"removed": removed, "steps": s.shared.lock().unwrap().steps.len()}))
}

/// Stops the hook and the worker, waiting for the last pictures to be written.
fn end(core: &Core, s: &Session) {
    #[cfg(windows)]
    {
        if let Some(h) = s.hook.lock().unwrap().take() {
            h.stop();
        }
        if let Some(w) = s.worker.lock().unwrap().take() {
            w.finish();
        }
    }
    service(core).current.lock().unwrap().take();
    core.host().steps_notice(false);
    core.host().steps_bar(None);
}

/// `smart.steps_cancel` (SMT-023): with steps and without `confirm`, the bar asks
/// first; with `confirm`, nothing is kept.
pub fn cancel(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let s = session(core).ok_or_else(|| not_recording(ctx.lang))?;
    let confirmed = bool_or(p, "confirm", ctx.source != Source::Ui);
    let ask = {
        let mut sh = s.shared.lock().unwrap();
        let ask = !confirmed && !sh.steps.is_empty();
        sh.confirm = ask;
        ask
    };
    if ask {
        tell(core, &s, "steps.confirm");
        return Ok(json!({"status": "confirm", "steps": s.shared.lock().unwrap().steps.len()}));
    }
    end(core, &s);
    let _ = std::fs::remove_dir_all(&s.folder);
    core.events
        .publish("steps.cancelled", s.source, json!({"status": "cancelled"}));
    tracing::info!("step recording cancelled");
    Ok(json!({"status": "cancelled"}))
}

/// The bar's confirmation was answered No: recording goes on (SMT-023).
pub fn keep(core: &Core, ctx: &Context) -> CommandResult {
    let s = session(core).ok_or_else(|| not_recording(ctx.lang))?;
    s.shared.lock().unwrap().confirm = false;
    tell(core, &s, "steps.changed");
    Ok(state_of(&s))
}

/// `smart.steps_stop` (SMT-023, SMT-028): the guide, in the library, and opened as
/// Open in says.
pub fn stop(core: &Arc<Core>, ctx: &Context) -> CommandResult {
    let s = session(core).ok_or_else(|| not_recording(ctx.lang))?;
    end(core, &s);
    let (steps, _) = read(&s.folder).map_err(CommandError::internal)?;
    if steps.is_empty() {
        let _ = std::fs::remove_dir_all(&s.folder);
        core.events
            .publish("steps.cancelled", s.source, json!({"status": "empty"}));
        return Ok(json!({"status": "empty", "steps": 0}));
    }
    // More steps for an open guide (SMT-029).
    if let Some((doc_id, after)) = &s.into {
        let g = guide_input(ctx.lang, &s.folder, &steps, None, s.started).map_err(CommandError::internal)?;
        let n = xs_editor::guide::add_steps(core, ctx, doc_id, after.as_deref(), g.steps)?;
        let _ = std::fs::remove_dir_all(&s.folder);
        let _ = core.call(
            Source::Ui,
            "editor.show",
            json!({"documentId": doc_id}),
            Some(ctx.lang),
        );
        let result = json!({"status": "done", "documentId": doc_id, "steps": n, "added": true});
        core.events.publish("steps.done", s.source, result.clone());
        return Ok(result);
    }
    let made = guide_from(core, ctx.lang, &s.folder, &steps, s.title.clone(), s.started)?;
    let _ = std::fs::remove_dir_all(&s.folder);
    let (id, path) = made;
    xs_library::library(core).scan();
    if s.open == "editor" {
        let _ = core.call(
            Source::Ui,
            "editor.show",
            json!({"documentId": id}),
            Some(ctx.lang),
        );
    }
    if s.copy {
        // The guide as one picture until it copies with its text (SMT-034).
        let _ = core.call(
            Source::Ui,
            "document.copy_all",
            json!({"documentId": id}),
            Some(ctx.lang),
        );
    }
    let result = json!({"status": "done", "documentId": id, "steps": steps.len(), "path": path});
    core.events.publish("steps.done", s.source, result.clone());
    tracing::info!(steps = steps.len(), "step recording made a guide");
    Ok(result)
}

/// The application most used in the steps: the default title (SMT-028).
fn most_used(steps: &[Step]) -> Option<String> {
    let mut counts: Vec<(String, usize)> = Vec::new();
    for s in steps.iter().filter(|s| !s.app.trim().is_empty()) {
        match counts.iter_mut().find(|(a, _)| *a == s.app) {
            Some((_, n)) => *n += 1,
            None => counts.push((s.app.clone(), 1)),
        }
    }
    counts.into_iter().max_by_key(|(_, n)| *n).map(|(a, _)| a)
}

/// A guide from a recording's folder (SMT-028).
pub fn guide_from(
    core: &Arc<Core>,
    lang: Lang,
    folder: &Path,
    steps: &[Step],
    title: Option<String>,
    started: chrono::DateTime<chrono::Local>,
) -> Result<(String, PathBuf), CommandError> {
    let g = guide_input(lang, folder, steps, title, started).map_err(CommandError::internal)?;
    xs_editor::guide::create_guide(core, lang, g)
}

/// Recordings a crash cut short become guides in Recover work (SMT-027, APP-047); their
/// folders go. The one recording now is left alone.
pub fn recover_leftovers(core: &Arc<Core>) {
    let current = session(core).map(|s| s.folder.clone());
    let Ok(entries) = std::fs::read_dir(recovery_dir(core)) else {
        return;
    };
    for dir in entries.flatten().map(|e| e.path()).filter(|p| p.is_dir()) {
        if current.as_ref() == Some(&dir) {
            continue;
        }
        let Ok((steps, meta)) = read(&dir) else { continue };
        if steps.is_empty() {
            let _ = std::fs::remove_dir_all(&dir);
            continue;
        }
        let lang = match meta["lang"].as_str() {
            Some("es") => Lang::Es,
            Some(_) => Lang::En,
            None => core.lang(),
        };
        let started = meta["started"]
            .as_str()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.with_timezone(&chrono::Local))
            .unwrap_or_else(chrono::Local::now);
        let title = meta["title"].as_str().map(str::to_string);
        let made = guide_input(lang, &dir, &steps, title, started)
            .and_then(|g| xs_editor::guide::recover_guide(core, lang, g));
        match made {
            Ok(_) => {
                tracing::info!(
                    steps = steps.len(),
                    "a step recording cut short is in Recover work"
                );
                let _ = std::fs::remove_dir_all(&dir);
            }
            Err(e) => tracing::warn!(error = %e, "a step recording cut short could not be recovered"),
        }
    }
}

/// What the guide builder needs from a recording's folder (SMT-028).
fn guide_input(
    lang: Lang,
    folder: &Path,
    steps: &[Step],
    title: Option<String>,
    started: chrono::DateTime<chrono::Local>,
) -> Result<xs_editor::guide::GuideIn, String> {
    let mut list = Vec::new();
    for st in steps {
        let png = std::fs::read(folder.join(&st.image)).map_err(|e| e.to_string())?;
        let (w, h) = xs_image::encode::png_size(&png).unwrap_or((st.frame[2] as u32, st.frame[3] as u32));
        let pointer = st.pointer.as_ref().and_then(|p| {
            let bytes = std::fs::read(folder.join(&p.file)).ok()?;
            Some((bytes, p.at, p.size))
        });
        list.push(xs_editor::guide::GuideStep {
            text: st.text.clone(),
            png,
            size: [w, h],
            frame: st.frame,
            at: st.at,
            to: st.to,
            right: st.kind == crate::steps::Kind::Right,
            ring: st.kind != crate::steps::Kind::Keys,
            pointer,
        });
    }
    let apps: Vec<String> = {
        let mut a: Vec<String> = Vec::new();
        for s in steps {
            if !s.app.is_empty() && !a.contains(&s.app) {
                a.push(s.app.clone());
            }
        }
        a
    };
    let title = title.unwrap_or_else(|| match most_used(steps) {
        Some(app) => tr(lang, "steps.guide.title_in", &[("app", &app)]),
        None => tr(lang, "steps.guide.title", &[]),
    });
    let meta = json!({
        "source": "steps",
        "guide": {
            "steps": steps.len(),
            "apps": apps,
            "started": started.to_rfc3339(),
            "ended": chrono::Local::now().to_rfc3339(),
        },
        "app": apps.first(),
    });
    Ok(xs_editor::guide::GuideIn {
        title,
        steps: list,
        meta,
    })
}

/// The icon menu's entries while recording steps (SMT-023): Pause or Resume, and
/// Finish, each with its command.
pub fn menu_items(core: &Core) -> Vec<(String, String, &'static str)> {
    let Some(s) = session(core) else { return Vec::new() };
    let paused = s.shared.lock().unwrap().status == Status::Paused;
    let lang = core.lang();
    vec![
        if paused {
            (
                "steps-resume".into(),
                tr(lang, "tray.steps_resume", &[]),
                "smart.steps_resume",
            )
        } else {
            (
                "steps-pause".into(),
                tr(lang, "tray.steps_pause", &[]),
                "smart.steps_pause",
            )
        },
        (
            "steps-stop".into(),
            tr(lang, "tray.steps_stop", &[]),
            "smart.steps_stop",
        ),
    ]
}

/// Handlers of the step recorder.
pub fn register(reg: &mut xs_core::command::Registry, weak: &Weak<Core>) {
    macro_rules! on {
        ($id:literal, $f:expr) => {{
            let w = weak.clone();
            reg.handle($id, move |ctx, p| {
                let core = w
                    .upgrade()
                    .ok_or_else(|| CommandError::internal("shutting down"))?;
                $f(&core, ctx, &p)
            });
        }};
    }
    on!("smart.steps_start", start);
    on!("smart.steps_pause", |core: &Arc<Core>,
                              ctx: &Context,
                              _p: &Value| {
        pause(core, ctx, true)
    });
    on!("smart.steps_resume", |core: &Arc<Core>,
                               ctx: &Context,
                               _p: &Value| {
        pause(core, ctx, false)
    });
    on!("smart.steps_toggle", |core: &Arc<Core>,
                               ctx: &Context,
                               _p: &Value| {
        let paused = session(core).is_some_and(|s| s.shared.lock().unwrap().status == Status::Paused);
        pause(core, ctx, !paused)
    });
    on!("smart.steps_status", |core: &Arc<Core>,
                               _ctx: &Context,
                               _p: &Value| {
        Ok(status(core))
    });
    on!("smart.steps_undo", |core: &Arc<Core>,
                             ctx: &Context,
                             _p: &Value| undo(
        core, ctx
    ));
    on!("smart.steps_stop", |core: &Arc<Core>,
                             ctx: &Context,
                             _p: &Value| stop(
        core, ctx
    ));
    on!("smart.steps_cancel", |core: &Arc<Core>,
                               ctx: &Context,
                               p: &Value| {
        cancel(core, ctx, p)
    });
    on!("smart.steps_keep", |core: &Arc<Core>,
                             ctx: &Context,
                             _p: &Value| keep(
        core, ctx
    ));
    on!(
        "smart.steps_notice_closed",
        |core: &Arc<Core>, _ctx: &Context, _p: &Value| notice_closed(core)
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    /// SMT-022 to SMT-028 on this desktop: presses and releases handed to the worker
    /// (no real clicks) at the middle of the main monitor make steps with pictures and
    /// sentences; a double click is one step; undo takes the last; stop makes a guide in
    /// the library with them, and the recording's folder goes. Run by hand: it reads the
    /// screen (into a temporary folder, deleted).
    #[cfg(windows)]
    #[test]
    #[ignore]
    fn a_recording_on_this_desktop() {
        use crate::steps::{Button, Event};
        use crate::steps_win::Heard;
        use std::time::Duration;
        let dir = tempfile::tempdir().unwrap();
        let core = Core::for_tests_with(
            xs_core::platform::Platform::Windows,
            vec![crate::extension(), xs_editor::extension()],
        );
        xs_editor::editor(&core).set_folders(dir.path().join("lib"), dir.path().join("rec"));
        let ctx = Context {
            source: Source::Cli,
            lang: Lang::Es,
        };
        let r = start(&core, &ctx, &json!({"cursor": true})).unwrap();
        assert_eq!(r["status"], "notice");
        let s = session(&core).unwrap();
        // Clicks under the notice do not count.
        let sink = s.worker.lock().unwrap().as_ref().unwrap().sink();
        let (mx, my) = crate::monitors()
            .unwrap()
            .into_iter()
            .find(|(_, i)| i.primary)
            .map(|(_, i)| {
                (
                    i.rect.x as f64 + i.rect.width as f64 / 2.0,
                    i.rect.y as f64 + i.rect.height as f64 / 2.0,
                )
            })
            .unwrap();
        let click = |t: u64, x: f64| {
            sink(Heard::Mouse(Event::Down {
                button: Button::Left,
                x,
                y: my,
                t,
            }));
            sink(Heard::Mouse(Event::Up {
                button: Button::Left,
                x,
                y: my,
                t: t + 50,
            }));
        };
        click(0, mx);
        std::thread::sleep(Duration::from_millis(400));
        assert_eq!(status(&core)["steps"], 0, "the notice is up");
        notice_closed(&core).unwrap();
        click(10_000, mx);
        // A double click on the same place: one step.
        click(20_000, mx);
        click(20_200, mx + 1.0);
        // A right click and a drag.
        sink(Heard::Mouse(Event::Down {
            button: Button::Right,
            x: mx - 100.0,
            y: my,
            t: 30_000,
        }));
        sink(Heard::Mouse(Event::Up {
            button: Button::Right,
            x: mx - 100.0,
            y: my,
            t: 30_100,
        }));
        sink(Heard::Mouse(Event::Down {
            button: Button::Left,
            x: mx,
            y: my,
            t: 40_000,
        }));
        sink(Heard::Mouse(Event::Move {
            x: mx + 60.0,
            y: my + 40.0,
        }));
        sink(Heard::Mouse(Event::Up {
            button: Button::Left,
            x: mx + 120.0,
            y: my + 80.0,
            t: 40_300,
        }));
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while status(&core)["steps"] != 4 && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(50));
        }
        let kinds: Vec<_> = s.shared.lock().unwrap().steps.iter().map(|st| st.kind).collect();
        use crate::steps::Kind;
        assert_eq!(kinds, [Kind::Click, Kind::Double, Kind::Right, Kind::Drag]);
        for st in &s.shared.lock().unwrap().steps {
            eprintln!("step {}: {} ({:?})", st.n, st.text, st.frame);
            assert!(!st.text.is_empty());
        }
        // A fifth click, undone.
        click(50_000, mx);
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        while status(&core)["steps"] != 5 && std::time::Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(50));
        }
        undo(&core, &ctx).unwrap();
        assert_eq!(status(&core)["steps"], 4);
        let folder = s.folder.clone();
        drop(s);
        let done = stop(&core, &ctx).unwrap();
        assert_eq!(done["status"], "done");
        assert_eq!(done["steps"], 4);
        assert!(!folder.exists(), "the recording's folder goes");
        let id = done["documentId"].as_str().unwrap();
        let doc = xs_editor::editor(&core)
            .store
            .with(id, |d| d.document.clone())
            .unwrap();
        let t = xs_editor::templates::Template::of(&doc).unwrap();
        assert!(t.is_guide());
        assert_eq!(t.sections.iter().filter(|s| s.kind == "guideStep").count(), 4);
        if let Ok(out) = std::env::var("XS_GUIDE_PNG") {
            xs_editor::final_image(&core, id).unwrap().save(out).unwrap();
        }
        assert_eq!(status(&core)["status"], "idle");
    }

    /// SMT-027: a recording a crash cut short (its folder, with 2 steps) is in Recover
    /// work as a guide, its folder goes, and recovering it opens the guide.
    #[test]
    fn a_recording_cut_short_is_recovered() {
        let dir = tempfile::tempdir().unwrap();
        let core = Core::for_tests_with(
            xs_core::platform::Platform::Windows,
            vec![crate::extension(), xs_editor::extension()],
        );
        xs_editor::editor(&core).set_folders(dir.path().join("lib"), dir.path().join("rec"));
        let folder = recovery_dir(&core).join("20261114-090000");
        std::fs::create_dir_all(&folder).unwrap();
        let img = image::RgbaImage::from_pixel(640, 400, image::Rgba([230, 230, 240, 255]));
        let png = xs_image::encode::png(&img, 0.0, true).unwrap();
        let mut steps = Vec::new();
        for n in 1..=2u32 {
            let image = format!("step-{n:03}.png");
            std::fs::write(folder.join(&image), &png).unwrap();
            steps.push(Step {
                n,
                kind: crate::steps::Kind::Click,
                image,
                at: [100.0, 100.0],
                to: None,
                frame: [0.0, 0.0, 640.0, 400.0],
                text: format!("Clic en **Paso {n}**"),
                app: "Bloc de notas".into(),
                pointer: None,
                time: String::new(),
            });
        }
        save(
            &folder,
            &steps,
            &json!({"started": "2026-11-14T09:00:00+00:00", "lang": "es"}),
        )
        .unwrap();
        let call = |id: &str, p: Value| core.call(Source::Ui, id, p, Some(Lang::Es)).unwrap();
        let list = call("editor.recovery_list", json!({}));
        let items = list["items"].as_array().unwrap();
        assert_eq!(items.len(), 1, "{list}");
        assert_eq!(items[0]["name"], "Pasos en Bloc de notas");
        assert!(!folder.exists(), "its folder goes");
        let restored = call("editor.recovery_restore", json!({"ids": [items[0]["id"]]}));
        let id = restored["documents"][0].as_str().unwrap();
        let doc = xs_editor::editor(&core)
            .store
            .with(id, |d| d.document.clone())
            .unwrap();
        let t = xs_editor::templates::Template::of(&doc).unwrap();
        assert_eq!(t.sections.iter().filter(|s| s.kind == "guideStep").count(), 2);
    }

    #[test]
    fn the_scale_effect() {
        let e = json!([{"type": "scale", "value": 50}]);
        assert_eq!(scale_of(&e, (3840.0, 2160.0)), 0.5);
        let off = json!([{"type": "scale", "value": 50, "enabled": false}, {"type": "border"}]);
        assert_eq!(scale_of(&off, (1920.0, 1080.0)), 1.0);
        let longest = json!([{"type": "scale", "mode": "longest", "value": 1920}]);
        assert_eq!(scale_of(&longest, (3840.0, 2160.0)), 0.5);
    }

    #[test]
    fn the_title_is_the_app_most_used() {
        let step = |app: &str| Step {
            n: 1,
            kind: crate::steps::Kind::Click,
            image: "s.png".into(),
            at: [0.0, 0.0],
            to: None,
            frame: [0.0, 0.0, 10.0, 10.0],
            text: String::new(),
            app: app.into(),
            pointer: None,
            time: String::new(),
        };
        let steps = [step("Excel"), step("Edge"), step("Excel")];
        assert_eq!(most_used(&steps).as_deref(), Some("Excel"));
    }
}
