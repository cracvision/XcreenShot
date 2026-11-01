//! Multi-capture (CAP-067): while it is on, every click captures the window under the
//! pointer as it looked just before the click and then reaches the app; a drag
//! captures a rectangle. The captures of the session wait in a small panel, from where
//! they are inserted in a document (EDT-078). `Esc` or the panel's Finish ends it.
//!
//! [`Gesture`] tells clicks from drags and double clicks, without any OS call, so it
//! is tested here. The input hook, the highlight and the replay of clicks are in
//! `multi_win.rs` (Windows); other platforms answer that it is not available yet.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use image::RgbaImage;
use serde_json::{json, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode, Lang};

use crate::{failed, grab, metadata, monitors, platform, Rect, Shot};

/// How far the pointer moves with the button down before a click becomes a drag.
pub const DRAG_THRESHOLD: i32 = 5;
/// Longest side of the panel thumbnails, in physical pixels.
const THUMB: u32 = 320;

// ---------------------------------------------------------------------------
// Gestures

/// What the pointer is over when the button goes down.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Over {
    /// An app window or the desktop: the press is held back and captured.
    App,
    /// A window of XcreenShot (the panel): left alone, no capture.
    Own,
    /// A window of an app running with more rights than XcreenShot: Windows does not
    /// let XcreenShot give it a click back, so the click goes through and is captured
    /// at once, without waiting (CAP-067, APP-026).
    Elevated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    Move(i32, i32),
    /// Position and time in milliseconds (the system's input clock).
    Down(i32, i32, u32),
    Up(i32, i32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Effect {
    /// Capture what is under the point now, before the click reaches it.
    Shoot { seq: u64, at: (i32, i32) },
    /// The press became a drag: forget that capture.
    Drop { seq: u64 },
    /// Show the rectangle being dragged.
    Rubber(Rect),
    /// A click to give back to the app, after capture `seq` (none for the second
    /// click of a double click: one double click, one capture).
    Click {
        seq: Option<u64>,
        down: (i32, i32),
        up: (i32, i32),
    },
    /// A drag ended: capture this region.
    Region(Rect),
    /// A click that went through on its own (Over::Elevated): capture now.
    Snap { at: (i32, i32) },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reaction {
    /// The event does not reach the app (it is given back later, or never for drags).
    pub swallow: bool,
    pub effects: Vec<Effect>,
}

impl Reaction {
    fn pass(effects: Vec<Effect>) -> Reaction {
        Reaction {
            swallow: false,
            effects,
        }
    }

    fn swallow(effects: Vec<Effect>) -> Reaction {
        Reaction {
            swallow: true,
            effects,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    Idle,
    /// The button is down over an app and held back.
    Pressed {
        at: (i32, i32),
        seq: Option<u64>,
    },
    Dragging {
        from: (i32, i32),
        to: (i32, i32),
    },
    /// The button went down over something left alone: its release too.
    Passing,
}

/// Left button gestures during a multi-capture session.
#[derive(Debug, Clone)]
pub struct Gesture {
    state: State,
    /// Where and when the last click went down, for double clicks.
    last_click: Option<((i32, i32), u32)>,
    next_seq: u64,
    /// The system's double-click time (ms) and area (px).
    double_time: u32,
    double_size: (i32, i32),
}

/// The rectangle between two corners, at least one pixel wide and high.
pub fn span(a: (i32, i32), b: (i32, i32)) -> Rect {
    Rect {
        x: a.0.min(b.0),
        y: a.1.min(b.1),
        width: (a.0 - b.0).unsigned_abs().max(1),
        height: (a.1 - b.1).unsigned_abs().max(1),
    }
}

impl Gesture {
    pub fn new(double_time: u32, double_size: (i32, i32)) -> Gesture {
        Gesture {
            state: State::Idle,
            last_click: None,
            next_seq: 0,
            double_time,
            double_size,
        }
    }

    /// Whether the button is down (held back or dragging).
    pub fn busy(&self) -> bool {
        !matches!(self.state, State::Idle)
    }

    fn second_click(&self, at: (i32, i32), time: u32) -> bool {
        self.last_click.is_some_and(|(p, t)| {
            time.wrapping_sub(t) <= self.double_time
                && (at.0 - p.0).abs() <= self.double_size.0 / 2
                && (at.1 - p.1).abs() <= self.double_size.1 / 2
        })
    }

    pub fn on(&mut self, input: Input, over: Over) -> Reaction {
        match (input, self.state) {
            (Input::Down(x, y, time), State::Idle) => match over {
                Over::Own => {
                    self.state = State::Passing;
                    Reaction::pass(Vec::new())
                }
                Over::Elevated => {
                    self.state = State::Passing;
                    self.last_click = None;
                    Reaction::pass(vec![Effect::Snap { at: (x, y) }])
                }
                Over::App => {
                    let second = self.second_click((x, y), time);
                    // The second click of a double click is not a new capture, and a
                    // third one starts counting again.
                    self.last_click = if second { None } else { Some(((x, y), time)) };
                    let seq = (!second).then(|| {
                        self.next_seq += 1;
                        self.next_seq
                    });
                    self.state = State::Pressed { at: (x, y), seq };
                    let effects = seq
                        .map(|seq| vec![Effect::Shoot { seq, at: (x, y) }])
                        .unwrap_or_default();
                    Reaction::swallow(effects)
                }
            },
            (Input::Move(x, y), State::Pressed { at, seq }) => {
                if (x - at.0).abs() <= DRAG_THRESHOLD && (y - at.1).abs() <= DRAG_THRESHOLD {
                    return Reaction::pass(Vec::new());
                }
                self.state = State::Dragging { from: at, to: (x, y) };
                self.last_click = None;
                let mut effects: Vec<Effect> = seq.map(|seq| Effect::Drop { seq }).into_iter().collect();
                effects.push(Effect::Rubber(span(at, (x, y))));
                Reaction::pass(effects)
            }
            (Input::Move(x, y), State::Dragging { from, .. }) => {
                self.state = State::Dragging { from, to: (x, y) };
                Reaction::pass(vec![Effect::Rubber(span(from, (x, y)))])
            }
            (Input::Up(x, y), State::Pressed { at, seq }) => {
                self.state = State::Idle;
                Reaction::swallow(vec![Effect::Click {
                    seq,
                    down: at,
                    up: (x, y),
                }])
            }
            (Input::Up(x, y), State::Dragging { from, .. }) => {
                self.state = State::Idle;
                Reaction::swallow(vec![Effect::Region(span(from, (x, y)))])
            }
            (Input::Up(..), State::Passing) => {
                self.state = State::Idle;
                Reaction::pass(Vec::new())
            }
            // A press that began before the session, moves without a button, a second
            // press while one is held: untouched.
            _ => Reaction::pass(Vec::new()),
        }
    }
}

// ---------------------------------------------------------------------------
// Targets and captures

/// What a click captures (CAP-067): the window under the pointer, or the monitor
/// when there is none. Physical pixels of the virtual desktop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Target {
    pub rect: Rect,
    /// The window's process; `None` for a monitor.
    pub pid: Option<u32>,
    pub title: Option<String>,
}

/// Pixels of `rect` as the screen shows them now, ready for the capture route.
pub fn shoot(
    rect: Rect,
    mode: &'static str,
    pid: Option<u32>,
    title: Option<String>,
    lang: Lang,
) -> Result<Shot, CommandError> {
    let list = monitors().map_err(|e| failed(lang, e))?;
    let hit: Vec<_> = list
        .iter()
        .map(|(_, i)| i)
        .filter(|i| i.rect.intersect(&rect).is_some())
        .collect();
    let main = hit
        .iter()
        .max_by_key(|i| i.rect.intersect(&rect).map(|r| r.area()).unwrap_or(0))
        .map(|i| i.name.clone());
    let scale = hit.iter().map(|i| i.scale).fold(1.0f32, f32::max);
    let image = grab(&list, rect).map_err(|e| failed(lang, e))?;
    Ok(Shot {
        image,
        rect,
        mode,
        scale,
        monitor: main.unwrap_or_default(),
        app: pid.map(|p| platform::app_info(p, "")),
        window_title: title,
        window_id: None,
        warnings: Vec::new(),
    })
}

// ---------------------------------------------------------------------------
// The session

/// One capture of the session, saved in the library (LIB-020).
#[derive(Clone)]
struct Item {
    id: String,
    path: PathBuf,
    width: u32,
    height: u32,
    thumb: Arc<Vec<u8>>,
}

#[derive(Default)]
struct Inner {
    items: Vec<Item>,
    next: u64,
    /// The running input hook; `None` once capturing stops (the panel may stay).
    running: Option<Running>,
}

/// State of multi-capture, one per app.
#[derive(Default)]
pub struct Multi {
    inner: Mutex<Inner>,
}

pub fn multi(core: &Core) -> Arc<Multi> {
    core.service(Multi::default)
}

#[cfg(windows)]
type Running = crate::multi_win::Session;
#[cfg(not(windows))]
type Running = ();

fn state(core: &Core) -> Value {
    let m = multi(core);
    let inner = m.inner.lock().unwrap();
    json!({
        "capturing": inner.running.is_some(),
        "items": inner.items.iter().map(|i| json!({
            "id": i.id,
            "path": i.path,
            "width": i.width,
            "height": i.height,
        })).collect::<Vec<_>>(),
    })
}

/// Tells the panel what changed.
fn emit(core: &Core) {
    core.host().multicapture_event(state(core));
}

/// A capture of the session: saved in the library without opening it, without the
/// clipboard and without a notification, and added to the panel (CAP-067).
pub fn keep(core: &Core, shot: Shot) {
    let now = chrono::Local::now();
    let meta = metadata(core, &shot, now);
    let app = shot.app.as_ref().map(|a| a.name.clone());
    let path = match xs_editor::save_capture(core, &shot.image, meta, shot.scale as f64, app.as_deref()) {
        Ok(p) => p,
        Err(e) => {
            tracing::error!(error = %e, "multi-capture: could not save");
            return;
        }
    };
    let thumb = thumbnail(&shot.image);
    tracing::info!(
        mode = shot.mode,
        width = shot.image.width(),
        height = shot.image.height(),
        "multi-capture: capture kept"
    );
    {
        let m = multi(core);
        let mut inner = m.inner.lock().unwrap();
        inner.next += 1;
        let id = format!("m{}", inner.next);
        inner.items.push(Item {
            id,
            path,
            width: shot.image.width(),
            height: shot.image.height(),
            thumb: Arc::new(thumb),
        });
    }
    emit(core);
}

fn thumbnail(image: &RgbaImage) -> Vec<u8> {
    let small = xs_image::encode::thumbnail(image, THUMB);
    xs_image::encode::png(&small, 0.0, true).unwrap_or_default()
}

/// PNG thumbnail of a capture of the session, for the panel.
pub fn thumbnail_png(core: &Core, id: &str) -> Option<Arc<Vec<u8>>> {
    let m = multi(core);
    let inner = m.inner.lock().unwrap();
    inner.items.iter().find(|i| i.id == id).map(|i| i.thumb.clone())
}

pub fn capturing(core: &Core) -> bool {
    multi(core).inner.lock().unwrap().running.is_some()
}

#[cfg(windows)]
fn start_input(core: &Arc<Core>, lang: Lang) -> Result<Running, CommandError> {
    crate::multi_win::Session::start(core.clone()).map_err(|e| failed(lang, e))
}

#[cfg(not(windows))]
fn start_input(_core: &Arc<Core>, lang: Lang) -> Result<Running, CommandError> {
    Err(CommandError::new(
        ErrorCode::NotSupportedOnPlatform,
        tr(lang, "errors.multicapture_platform", &[]),
    ))
}

/// `capture.multi_start`: starts capturing and shows the panel.
pub fn start(core: &Arc<Core>, ctx: &Context) -> CommandResult {
    if super::CAPTURING.load(std::sync::atomic::Ordering::SeqCst) {
        return Err(
            CommandError::new(ErrorCode::Busy, tr(ctx.lang, "errors.capture_busy", &[]))
                .with_details(json!({ "status": "busy" })),
        );
    }
    let m = multi(core);
    {
        let mut inner = m.inner.lock().unwrap();
        if inner.running.is_none() {
            inner.running = Some(start_input(core, ctx.lang)?);
            tracing::info!("multi-capture started");
        }
    }
    core.host().multicapture_panel(true);
    emit(core);
    Ok(state(core))
}

/// `capture.multi_stop`: stops capturing; the panel stays, so the captures can still
/// be inserted, unless `close` also ends the session.
pub fn stop(core: &Core, close: bool) -> Value {
    let running = multi(core).inner.lock().unwrap().running.take();
    if let Some(r) = running {
        #[cfg(windows)]
        r.stop();
        #[cfg(not(windows))]
        let () = r;
        tracing::info!("multi-capture stopped");
    }
    if close {
        multi(core).inner.lock().unwrap().items.clear();
        core.host().multicapture_panel(false);
    }
    emit(core);
    state(core)
}

pub fn get_state(core: &Core) -> Value {
    state(core)
}

/// `capture.multi_remove`: takes captures out of the panel; they stay in the library.
pub fn remove(core: &Core, ids: &[String]) -> Value {
    multi(core)
        .inner
        .lock()
        .unwrap()
        .items
        .retain(|i| !ids.contains(&i.id));
    emit(core);
    state(core)
}

/// `capture.multi_clear`: empties the panel; the captures stay in the library.
pub fn clear(core: &Core) -> Value {
    multi(core).inner.lock().unwrap().items.clear();
    emit(core);
    state(core)
}

/// Message for an unknown id, for the commands that take ids.
pub fn unknown(lang: Lang, id: &str) -> CommandError {
    CommandError::new(
        ErrorCode::NotFound,
        tr(lang, "errors.multicapture_item", &[("id", id)]),
    )
}

pub fn has_item(core: &Core, id: &str) -> bool {
    multi(core).inner.lock().unwrap().items.iter().any(|i| i.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn g() -> Gesture {
        Gesture::new(500, (4, 4))
    }

    #[test]
    fn a_click_is_captured_first_and_then_given_back() {
        let mut g = g();
        let down = g.on(Input::Down(100, 100, 0), Over::App);
        assert!(down.swallow);
        assert_eq!(
            down.effects,
            vec![Effect::Shoot {
                seq: 1,
                at: (100, 100)
            }]
        );
        let jitter = g.on(Input::Move(103, 98), Over::App);
        assert!(
            !jitter.swallow && jitter.effects.is_empty(),
            "3 px is still a click"
        );
        let up = g.on(Input::Up(103, 98), Over::App);
        assert!(up.swallow);
        assert_eq!(
            up.effects,
            vec![Effect::Click {
                seq: Some(1),
                down: (100, 100),
                up: (103, 98)
            }]
        );
        assert!(!g.busy());
    }

    #[test]
    fn a_drag_captures_its_rectangle_and_never_reaches_the_app() {
        let mut g = g();
        g.on(Input::Down(10, 20, 0), Over::App);
        let moved = g.on(Input::Move(310, 220), Over::App);
        assert_eq!(
            moved.effects,
            vec![
                Effect::Drop { seq: 1 },
                Effect::Rubber(Rect {
                    x: 10,
                    y: 20,
                    width: 300,
                    height: 200
                })
            ]
        );
        let up = g.on(Input::Up(310, 220), Over::App);
        assert!(up.swallow);
        assert_eq!(
            up.effects,
            vec![Effect::Region(Rect {
                x: 10,
                y: 20,
                width: 300,
                height: 200
            })]
        );
        // Dragging up and to the left gives the same rectangle.
        g.on(Input::Down(310, 220, 5000), Over::App);
        g.on(Input::Move(10, 20), Over::App);
        assert_eq!(
            g.on(Input::Up(10, 20), Over::App).effects,
            vec![Effect::Region(Rect {
                x: 10,
                y: 20,
                width: 300,
                height: 200
            })]
        );
    }

    #[test]
    fn a_double_click_is_one_capture_and_two_clicks() {
        let mut g = g();
        g.on(Input::Down(50, 50, 1000), Over::App);
        g.on(Input::Up(50, 50), Over::App);
        let second = g.on(Input::Down(51, 50, 1200), Over::App);
        assert!(second.swallow, "held so it reaches the app after the first");
        assert!(second.effects.is_empty(), "no second capture");
        assert_eq!(
            g.on(Input::Up(51, 50), Over::App).effects,
            vec![Effect::Click {
                seq: None,
                down: (51, 50),
                up: (51, 50)
            }]
        );
        // A third click is a new capture, and so is a slow second click.
        assert_eq!(
            g.on(Input::Down(51, 50, 1300), Over::App).effects,
            vec![Effect::Shoot { seq: 2, at: (51, 50) }]
        );
        g.on(Input::Up(51, 50), Over::App);
        assert_eq!(
            g.on(Input::Down(51, 50, 2000), Over::App).effects,
            vec![Effect::Shoot { seq: 3, at: (51, 50) }]
        );
    }

    #[test]
    fn clicks_far_apart_are_two_captures() {
        let mut g = g();
        g.on(Input::Down(50, 50, 1000), Over::App);
        g.on(Input::Up(50, 50), Over::App);
        assert_eq!(
            g.on(Input::Down(80, 50, 1100), Over::App).effects,
            vec![Effect::Shoot { seq: 2, at: (80, 50) }]
        );
    }

    #[test]
    fn xcreenshot_windows_are_left_alone_and_elevated_ones_captured_at_once() {
        let mut g = g();
        let own = g.on(Input::Down(5, 5, 0), Over::Own);
        assert!(!own.swallow && own.effects.is_empty());
        assert!(!g.on(Input::Move(100, 100), Over::Own).swallow);
        assert!(!g.on(Input::Up(100, 100), Over::Own).swallow);
        let admin = g.on(Input::Down(5, 5, 0), Over::Elevated);
        assert!(!admin.swallow);
        assert_eq!(admin.effects, vec![Effect::Snap { at: (5, 5) }]);
        assert!(!g.on(Input::Up(5, 5), Over::Elevated).swallow);
        assert!(!g.busy());
    }

    #[test]
    fn a_release_without_a_press_goes_through() {
        let mut g = g();
        assert!(!g.on(Input::Up(5, 5), Over::App).swallow);
    }
}
