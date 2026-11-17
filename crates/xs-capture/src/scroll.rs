//! Scrolling capture (CAP-034 manual, CAP-035 automatic): frames of an area taken while
//! its content scrolls, joined as they come (CAP-036, [`crate::stitch`]).
//!
//! - **Manual:** the user scrolls the content any way they like; a panel beside the area
//!   shows the joined image live, with Done (`Enter`) and Cancel (`Esc`).
//! - **Automatic:** XcreenShot turns the wheel over the area itself, one step at a time,
//!   waiting between steps for content that loads late, until the content stops moving;
//!   Stop here (`Enter`) keeps what it has and Cancel (`Esc`) drops it. The user's own
//!   click or wheel inside the area also stops it. At the end the content goes back to
//!   where it was.
//!
//! The way the automatic capture drives the content is behind [`Driver`], so it is
//! tested here against a simulated page.

use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use image::RgbaImage;
use serde_json::{json, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode, Lang};

use crate::stitch::{Direction, Pushed, Stitcher};
use crate::{failed, finish, monitors, Options, Rect, Shot};

/// Smallest area a scrolling capture accepts (CAP-034), in physical pixels.
pub const MIN_SIDE: u32 = 50;
/// Time between frames of a manual capture.
const MANUAL_EVERY: Duration = Duration::from_millis(70);
/// Time between preview updates.
const PREVIEW_EVERY: Duration = Duration::from_millis(300);
/// Largest preview image, in pixels.
const PREVIEW_W: u32 = 360;
const PREVIEW_H: u32 = 1400;
/// Most wheel steps an automatic capture takes (a safety net for endless pages).
const MAX_STEPS: usize = 2000;

/// What the user asks of a running scrolling capture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// Start capturing (with "Ask before scrolling", CAP-034).
    Start,
    /// Done, or Stop here: keep what has been captured.
    Done,
    /// Cancel: no capture.
    Cancel,
}

/// How an automatic capture moves (CAP-035).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Auto {
    pub direction: Direction,
    /// Start from the beginning (top, left or top left), not from where the content is.
    pub from_start: bool,
    /// Wait between steps, for content that loads late.
    pub wait: Duration,
    /// Put the content back where it was at the end.
    pub restore: bool,
    /// Detect fixed parts (CAP-036).
    pub fixed: bool,
}

impl Auto {
    /// The settings of the Image tab's Scrolling selection (CAP-035), with `direction`
    /// when the user picked an arrow.
    pub fn from_settings(core: &Core, direction: Option<Direction>) -> Auto {
        let direction =
            direction.unwrap_or(match core.settings.get_str("capture.scroll.direction").as_str() {
                "horizontal" => Direction::Horizontal,
                "both" => Direction::Both,
                _ => Direction::Vertical,
            });
        let wait = core
            .settings
            .get("capture.scroll.wait")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.3)
            .clamp(0.0, 5.0);
        Auto {
            direction,
            from_start: core.settings.get_str("capture.scroll.startFrom") != "current",
            wait: Duration::from_millis((wait * 1000.0) as u64),
            restore: core.settings.get_bool("capture.scroll.restore"),
            fixed: crate::panel::load(core).image.scroll.fixed,
        }
    }
}

/// Manual or automatic (CAP-034, CAP-035).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Manual,
    Auto(Auto),
}

/// What the capture route needs to know about where the area is (CAP-056).
pub struct Place {
    pub scale: f32,
    pub monitor: String,
    pub app: Option<crate::platform::AppInfo>,
    pub window_title: Option<String>,
}

// ---------------------------------------------------------------------------
// Shared state: the panel reads it, its buttons and the tray menu act on it.

#[derive(Default)]
struct Shared {
    /// Where actions go while a capture runs.
    actions: Option<Sender<Action>>,
    preview: Option<Arc<Vec<u8>>>,
    state: Value,
}

#[derive(Default)]
pub struct ScrollState {
    shared: Mutex<Shared>,
}

fn service(core: &Core) -> Arc<ScrollState> {
    core.service(ScrollState::default)
}

/// Whether a scrolling capture is running (the tray offers "Finish scrolling", CAP-034).
pub fn running(core: &Core) -> bool {
    service(core).shared.lock().unwrap().actions.is_some()
}

/// Done, Cancel or Start for the running capture; false if none runs.
pub fn act(core: &Core, action: Action) -> bool {
    let s = service(core);
    let shared = s.shared.lock().unwrap();
    shared.actions.as_ref().is_some_and(|tx| tx.send(action).is_ok())
}

/// The panel's view of the running capture (`capture.scroll_state`).
pub fn state(core: &Core) -> Value {
    let s = service(core);
    let shared = s.shared.lock().unwrap();
    if shared.actions.is_none() {
        return json!({ "running": false });
    }
    shared.state.clone()
}

/// The latest preview, as PNG.
pub fn preview_png(core: &Core) -> Option<Arc<Vec<u8>>> {
    service(core).shared.lock().unwrap().preview.clone()
}

fn publish(core: &Core, state: Value, preview: Option<Vec<u8>>) {
    {
        let s = service(core);
        let mut shared = s.shared.lock().unwrap();
        shared.state = state.clone();
        if let Some(png) = preview {
            shared.preview = Some(Arc::new(png));
        }
    }
    core.host().scroll_event(state);
}

// ---------------------------------------------------------------------------
// Driving the content

/// How a capture sees the area and moves its content.
pub trait Driver {
    /// The area as it is now.
    fn grab(&mut self) -> Result<RgbaImage, String>;
    /// Turns the wheel `notches` times (negative: up or left).
    fn wheel(&mut self, notches: i32, horizontal: bool);
    /// Waits, for the content to settle.
    fn wait(&mut self, time: Duration);
}

/// What an automatic capture reports while it runs.
pub trait Progress {
    /// A new joined size; `false` stops the capture and keeps what it has.
    fn step(&mut self, stitcher: &Stitcher) -> Flow;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Flow {
    Go,
    Stop,
    Cancel,
}

/// How an automatic capture ended.
#[derive(Debug, PartialEq, Eq)]
pub enum Ended {
    /// It reached the end (or was stopped, or the result is full): keep the image.
    Keep,
    /// Cancelled: no capture.
    Cancelled,
}

/// Runs an automatic capture (CAP-035): back to the beginning if asked, then one wheel
/// step at a time until the content stops moving, along `auto.direction` (for both
/// directions: down one column, a step right, up the next…). At the end the content goes
/// back where it was, measured in pixels by the joins. Returns the stitcher with what it
/// joined.
pub fn drive(driver: &mut dyn Driver, auto: Auto, progress: &mut dyn Progress) -> (Stitcher, Ended) {
    let primary_h = auto.direction == Direction::Horizontal;
    let both = auto.direction == Direction::Both;
    // Where the start position is, seen from the beginning (pixels), when known.
    let mut start_from_top = (0i64, 0i64);
    let mut pace = Pace::default();
    if auto.from_start {
        let back = to_end(driver, primary_h, auto.wait, &mut pace);
        start_from_top.1 = -back.1;
        start_from_top.0 = -back.0;
        if both {
            let side = to_end(driver, true, auto.wait, &mut pace);
            start_from_top.0 = -side.0;
        }
    }

    let mut stitcher = Stitcher::new(auto.direction).keep_fixed(auto.fixed);
    let Ok(first) = driver.grab() else {
        return (stitcher, Ended::Keep);
    };
    stitcher.push(first);
    let (fw, fh) = stitcher.frame_size();

    let horizontal = primary_h;
    let mut sign = 1;
    let mut still = 0;
    let mut lost = 0;
    let mut ended = Ended::Keep;
    for _ in 0..MAX_STEPS {
        let notches = pace.step(horizontal, if horizontal { fw } else { fh });
        driver.wheel(sign * notches, horizontal);
        driver.wait(auto.wait);
        let Ok(frame) = driver.grab() else {
            break;
        };
        let pushed = stitcher.push(frame);
        if let Pushed::Joined { dx, dy, .. } = pushed {
            pace.learn(horizontal, notches, if horizontal { dx } else { dy });
        }
        match progress.step(&stitcher) {
            Flow::Go => {}
            Flow::Stop => break,
            Flow::Cancel => {
                ended = Ended::Cancelled;
                break;
            }
        }
        match pushed {
            Pushed::Joined { .. } => {
                still = 0;
                lost = 0;
            }
            Pushed::Same | Pushed::First => {
                still += 1;
                if still < 2 {
                    continue;
                }
                // The end along this axis.
                if !both || horizontal {
                    break;
                }
                // Both directions: a step sideways, then the other way along.
                still = 0;
                let side = pace.step(true, fw);
                driver.wheel(side, true);
                driver.wait(auto.wait);
                let Ok(frame) = driver.grab() else {
                    break;
                };
                match stitcher.push(frame) {
                    Pushed::Joined { dx, .. } => {
                        pace.learn(true, side, dx);
                        sign = -sign;
                    }
                    _ => break,
                }
            }
            Pushed::Lost => {
                // Too far: back, and smaller steps.
                driver.wheel(-sign * notches, horizontal);
                driver.wait(auto.wait);
                pace.shrink(horizontal);
                lost += 1;
                if lost > 4 {
                    break;
                }
            }
            Pushed::Full => break,
        }
    }
    if auto.restore {
        // From where the last frame is to where the content was at the start.
        let (x, y) = stitcher.position();
        let (to_x, to_y) = start_from_top;
        driver.wheel(pace.notches_for(false, to_y - y), false);
        driver.wheel(pace.notches_for(true, to_x - x), true);
    }
    (stitcher, ended)
}

/// How many pixels a wheel notch moves the content, learned as the capture goes.
#[derive(Default)]
struct Pace {
    /// Pixels per notch, vertical and sideways.
    per: [Option<f32>; 2],
    /// A cap on the step after a jump that lost the content.
    cap: [Option<i32>; 2],
}

impl Pace {
    fn axis(horizontal: bool) -> usize {
        usize::from(horizontal)
    }

    /// Notches for the next step: about half the area once the pace is known, 3 before.
    fn step(&self, horizontal: bool, span: u32) -> i32 {
        let a = Self::axis(horizontal);
        let n = match self.per[a] {
            Some(per) if per > 0.0 => ((span as f32 * 0.5 / per).floor() as i32).clamp(1, 30),
            _ => 3,
        };
        self.cap[a].map_or(n, |c| n.min(c))
    }

    /// A step moved the content `moved` pixels. At the ends the content moves less than
    /// the wheel asks for, so the pace is the largest seen.
    fn learn(&mut self, horizontal: bool, notches: i32, moved: i32) {
        if notches != 0 && moved != 0 {
            let per = (moved as f32 / notches as f32).abs();
            let a = Self::axis(horizontal);
            self.per[a] = Some(self.per[a].map_or(per, |p| p.max(per)));
        }
    }

    fn shrink(&mut self, horizontal: bool) {
        let a = Self::axis(horizontal);
        let now = self.step(horizontal, 0).max(self.cap[a].unwrap_or(3));
        self.cap[a] = Some((now / 2).max(1));
    }

    /// Notches that move the content by `pixels` (positive: down or right).
    fn notches_for(&self, horizontal: bool, pixels: i64) -> i32 {
        match self.per[Self::axis(horizontal)] {
            Some(per) if per > 0.0 => (pixels as f32 / per).round() as i32,
            _ => 0,
        }
    }
}

/// Turns the wheel toward the beginning until the content stops moving. Returns how far
/// it moved, in pixels (negative: up or left), as far as the joins could tell.
fn to_end(driver: &mut dyn Driver, horizontal: bool, wait: Duration, pace: &mut Pace) -> (i64, i64) {
    let mut s = Stitcher::new(if horizontal {
        Direction::Horizontal
    } else {
        Direction::Vertical
    });
    let Ok(first) = driver.grab() else {
        return (0, 0);
    };
    s.push(first);
    let (fw, fh) = s.frame_size();
    let mut moved = (0i64, 0i64);
    let mut lost = false;
    let mut still = 0;
    for _ in 0..400 {
        let notches = pace.step(horizontal, if horizontal { fw } else { fh }) * if lost { 3 } else { 1 };
        driver.wheel(-notches, horizontal);
        driver.wait(wait.min(Duration::from_millis(150)));
        let Ok(now) = driver.grab() else {
            break;
        };
        match s.push(now.clone()) {
            Pushed::Joined { dx, dy, .. } => {
                still = 0;
                moved = (moved.0 + dx as i64, moved.1 + dy as i64);
                pace.learn(horizontal, notches, if horizontal { dx } else { dy });
            }
            Pushed::Same => {
                still += 1;
                if still >= 2 {
                    break;
                }
            }
            // Out of reach of the joins: keep going up, the distance is no longer known.
            _ => {
                lost = true;
                s = Stitcher::new(if horizontal {
                    Direction::Horizontal
                } else {
                    Direction::Vertical
                });
                s.push(now);
            }
        }
    }
    if lost {
        (0, 0)
    } else {
        moved
    }
}

// ---------------------------------------------------------------------------
// The session

/// The screen behind a capture: the area's pixels and, on Windows, the wheel.
struct Screen {
    list: Vec<(xcap::Monitor, crate::MonitorInfo)>,
    area: Rect,
}

impl Driver for Screen {
    fn grab(&mut self) -> Result<RgbaImage, String> {
        crate::grab(&self.list, self.area).map_err(|e| e.to_string())
    }

    fn wheel(&mut self, notches: i32, horizontal: bool) {
        #[cfg(windows)]
        {
            let center = (
                self.area.x + self.area.width as i32 / 2,
                self.area.y + self.area.height as i32 / 2,
            );
            // Several small turns: some apps ignore one big one.
            let step = notches.signum() * 3;
            let mut left = notches;
            while left != 0 {
                let now = if left.abs() >= 3 { step } else { left };
                crate::scroll_win::wheel(center, now, horizontal);
                left -= now;
                std::thread::sleep(Duration::from_millis(8));
            }
        }
        #[cfg(not(windows))]
        let _ = (notches, horizontal);
    }

    fn wait(&mut self, time: Duration) {
        std::thread::sleep(time);
    }
}

/// Reports an automatic capture's progress to the panel and reads the user's actions.
struct Reporter<'a> {
    core: &'a Core,
    actions: &'a Receiver<Action>,
    last: Instant,
    cancelled: bool,
}

impl Progress for Reporter<'_> {
    fn step(&mut self, stitcher: &Stitcher) -> Flow {
        if self.last.elapsed() >= PREVIEW_EVERY {
            self.last = Instant::now();
            report(self.core, stitcher, true, true);
        }
        match self.actions.try_recv() {
            Ok(Action::Done) => Flow::Stop,
            Ok(Action::Cancel) => {
                self.cancelled = true;
                Flow::Cancel
            }
            _ => Flow::Go,
        }
    }
}

fn report(core: &Core, stitcher: &Stitcher, started: bool, auto: bool) {
    let (w, h) = stitcher.size();
    let png = stitcher
        .preview(PREVIEW_W, PREVIEW_H)
        .and_then(|img| xs_image::encode::png(&img, 0.0, true).ok());
    let seq = service(core)
        .shared
        .lock()
        .unwrap()
        .state
        .get("preview")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        + 1;
    publish(
        core,
        json!({
            "running": true,
            "auto": auto,
            "started": started,
            "width": w,
            "height": h,
            "full": stitcher.is_full(),
            "preview": seq,
        }),
        png,
    );
}

/// Runs a scrolling capture of `area` and sends the joined image down the capture route
/// (CAP-034, CAP-035, CAP-036). Blocks until it ends.
pub fn run(
    core: &Core,
    ctx: &Context,
    opts: &Options,
    area: Rect,
    kind: Kind,
    place: Place,
) -> CommandResult {
    let lang = ctx.lang;
    if area.width < MIN_SIDE || area.height < MIN_SIDE {
        return Err(CommandError::new(
            ErrorCode::InvalidParams,
            tr(lang, "errors.scroll_too_small", &[("min", &MIN_SIDE.to_string())]),
        ));
    }
    let list = monitors().map_err(|e| failed(lang, e))?;
    let (tx, rx) = mpsc::channel();
    {
        let s = service(core);
        let mut shared = s.shared.lock().unwrap();
        shared.actions = Some(tx.clone());
        shared.preview = None;
        shared.state = json!({});
    }
    let auto = matches!(kind, Kind::Auto(_));
    let ask = !auto && core.settings.get_bool("capture.scroll.askStart");
    publish(
        core,
        json!({ "running": true, "auto": auto, "started": !ask, "width": area.width, "height": area.height, "preview": 0 }),
        None,
    );
    core.host()
        .scroll_panel(Some(json!({ "rect": area, "auto": auto })));
    let hooks = start_hooks(tx, if auto { Some(area) } else { None });
    if let Err(e) = &hooks {
        tracing::warn!(error = %e, "scrolling capture: no Enter/Esc keys");
    }
    let mut screen = Screen { list, area };
    let outcome = match kind {
        Kind::Manual => manual(core, &mut screen, &rx, ask, lang),
        Kind::Auto(a) => {
            let mut reporter = Reporter {
                core,
                actions: &rx,
                last: Instant::now(),
                cancelled: false,
            };
            // Our panel must not be what the wheel lands on: the pointer goes back after.
            #[cfg(windows)]
            let pointer = crate::scroll_win::pointer();
            let (stitcher, ended) = drive(&mut screen, a, &mut reporter);
            #[cfg(windows)]
            if let Some(p) = pointer {
                crate::scroll_win::move_pointer(p);
            }
            match ended {
                Ended::Keep => Ok(stitcher),
                Ended::Cancelled => Err(()),
            }
        }
    };
    drop(hooks);
    {
        let s = service(core);
        let mut shared = s.shared.lock().unwrap();
        shared.actions = None;
        shared.preview = None;
    }
    core.host().scroll_panel(None);
    core.host().scroll_event(json!({ "running": false }));

    let Ok(stitcher) = outcome else {
        tracing::info!("scrolling capture cancelled");
        return Err(
            CommandError::new(ErrorCode::Cancelled, tr(lang, "errors.capture_cancelled", &[]))
                .with_details(json!({ "status": "cancelled" })),
        );
    };
    let (joins, doubtful) = stitcher.joins();
    let full = stitcher.is_full();
    let Some(image) = stitcher.finish() else {
        return Err(failed(lang, "no frames"));
    };
    tracing::info!(
        width = image.width(),
        height = image.height(),
        joins,
        doubtful,
        full,
        "scrolling capture joined"
    );
    let title = tr(lang, "app.name", &[]);
    if full {
        core.host()
            .notify("problems", &title, &tr(lang, "notify.scroll_full", &[]));
    } else if auto && joins == 0 {
        core.host()
            .notify("problems", &title, &tr(lang, "notify.scroll_still", &[]));
    }
    if doubtful > 0 {
        core.host().notify(
            "problems",
            &title,
            &tr(
                lang,
                "notify.scroll_doubtful",
                &[("count", &doubtful.to_string())],
            ),
        );
    }
    let shot = Shot {
        image,
        rect: area,
        mode: "scrolling",
        scale: place.scale,
        monitor: place.monitor,
        app: place.app,
        window_title: place.window_title,
        window_id: None,
        pointer: None,
        warnings: Vec::new(),
    };
    finish(core, ctx, shot, opts)
}

/// A manual capture: frames while the user scrolls, until Done or Cancel (CAP-034).
fn manual(
    core: &Core,
    screen: &mut Screen,
    actions: &Receiver<Action>,
    ask: bool,
    lang: Lang,
) -> Result<Stitcher, ()> {
    let _ = lang;
    if ask {
        // Start (or Done) begins; Cancel, or the panel gone, ends it.
        if matches!(actions.recv(), Ok(Action::Cancel) | Err(_)) {
            return Err(());
        }
        report_start(core);
    }
    let mut stitcher = Stitcher::new(Direction::Both).keep_fixed(crate::panel::load(core).image.scroll.fixed);
    let mut last_preview = Instant::now() - PREVIEW_EVERY;
    let mut changed = true;
    loop {
        let began = Instant::now();
        match actions.try_recv() {
            Ok(Action::Done) => return Ok(stitcher),
            Ok(Action::Cancel) => return Err(()),
            Ok(Action::Start) | Err(mpsc::TryRecvError::Empty) => {}
            Err(mpsc::TryRecvError::Disconnected) => return Err(()),
        }
        let frame = match screen.grab() {
            Ok(f) => f,
            Err(e) => {
                tracing::warn!(error = %e, "scrolling capture: frame not taken");
                std::thread::sleep(MANUAL_EVERY);
                continue;
            }
        };
        match stitcher.push(frame) {
            Pushed::First | Pushed::Joined { .. } => changed = true,
            Pushed::Full => {
                report(core, &stitcher, true, false);
                return Ok(stitcher);
            }
            Pushed::Same | Pushed::Lost => {}
        }
        if changed && last_preview.elapsed() >= PREVIEW_EVERY {
            report(core, &stitcher, true, false);
            last_preview = Instant::now();
            changed = false;
        }
        if let Some(rest) = MANUAL_EVERY.checked_sub(began.elapsed()) {
            std::thread::sleep(rest);
        }
    }
}

fn report_start(core: &Core) {
    let mut state = service(core).shared.lock().unwrap().state.clone();
    state["started"] = json!(true);
    publish(core, state, None);
}

#[cfg(windows)]
fn start_hooks(tx: Sender<Action>, watch: Option<Rect>) -> Result<crate::scroll_win::Hooks, String> {
    crate::scroll_win::Hooks::start(tx, watch)
}

#[cfg(not(windows))]
fn start_hooks(_tx: Sender<Action>, _watch: Option<Rect>) -> Result<(), String> {
    Err("not on this platform".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    /// A page seen through a window: the wheel moves the view, 40 px a notch, never past
    /// the page's ends.
    struct Page {
        content: RgbaImage,
        view: (u32, u32),
        at: (i64, i64),
        per_notch: i64,
        wheels: Vec<(i32, bool)>,
    }

    impl Page {
        fn new(content: RgbaImage, view: (u32, u32), at: (i64, i64)) -> Page {
            Page {
                content,
                view,
                at,
                per_notch: 40,
                wheels: Vec::new(),
            }
        }
    }

    impl Driver for Page {
        fn grab(&mut self) -> Result<RgbaImage, String> {
            Ok(image::imageops::crop_imm(
                &self.content,
                self.at.0 as u32,
                self.at.1 as u32,
                self.view.0,
                self.view.1,
            )
            .to_image())
        }

        fn wheel(&mut self, notches: i32, horizontal: bool) {
            self.wheels.push((notches, horizontal));
            let (max_x, max_y) = (
                (self.content.width() - self.view.0) as i64,
                (self.content.height() - self.view.1) as i64,
            );
            let d = notches as i64 * self.per_notch;
            if horizontal {
                self.at.0 = (self.at.0 + d).clamp(0, max_x);
            } else {
                self.at.1 = (self.at.1 + d).clamp(0, max_y);
            }
        }

        fn wait(&mut self, _time: Duration) {}
    }

    struct Quiet {
        steps: usize,
        stop_after: Option<usize>,
    }

    impl Progress for Quiet {
        fn step(&mut self, _s: &Stitcher) -> Flow {
            self.steps += 1;
            match self.stop_after {
                Some(n) if self.steps >= n => Flow::Stop,
                _ => Flow::Go,
            }
        }
    }

    fn page(w: u32, h: u32) -> RgbaImage {
        crate::stitch::tests::page(w, h, 21)
    }

    fn auto(direction: Direction, from_start: bool) -> Auto {
        Auto {
            direction,
            from_start,
            wait: Duration::ZERO,
            restore: true,
            fixed: true,
        }
    }

    #[test]
    fn captures_a_long_page_from_the_top_even_if_it_started_halfway() {
        let content = page(500, 4000);
        let mut p = Page::new(content.clone(), (500, 700), (0, 1800));
        let (s, ended) = drive(
            &mut p,
            auto(Direction::Vertical, true),
            &mut Quiet {
                steps: 0,
                stop_after: None,
            },
        );
        assert_eq!(ended, Ended::Keep);
        let out = s.finish().unwrap();
        assert_eq!(out.dimensions(), (500, 4000));
        assert!(out.as_raw() == content.as_raw());
        // Back where it was, give or take a wheel notch.
        assert!((p.at.1 - 1800).abs() <= p.per_notch, "{:?}", p.at);
    }

    #[test]
    fn captures_sideways() {
        let content = image::imageops::rotate90(&page(400, 2600));
        let mut p = Page::new(content.clone(), (900, 400), (0, 0));
        let (s, _) = drive(
            &mut p,
            auto(Direction::Horizontal, true),
            &mut Quiet {
                steps: 0,
                stop_after: None,
            },
        );
        let out = s.finish().unwrap();
        assert_eq!(out.dimensions(), (2600, 400));
        assert!(out.as_raw() == content.as_raw());
    }

    #[test]
    fn stop_here_keeps_what_was_captured() {
        let content = page(500, 6000);
        let mut p = Page::new(content.clone(), (500, 700), (0, 0));
        let (s, ended) = drive(
            &mut p,
            auto(Direction::Vertical, true),
            &mut Quiet {
                steps: 0,
                stop_after: Some(4),
            },
        );
        assert_eq!(ended, Ended::Keep);
        let out = s.finish().unwrap();
        assert!(out.height() > 700 && out.height() < 6000);
        let h = out.height();
        assert!(
            out.as_raw()
                == image::imageops::crop_imm(&content, 0, 0, 500, h)
                    .to_image()
                    .as_raw()
        );
        assert_eq!(p.at, (0, 0));
    }

    #[test]
    fn from_the_current_position_starts_there() {
        let content = page(500, 3000);
        let mut p = Page::new(content.clone(), (500, 700), (0, 1000));
        let (s, _) = drive(
            &mut p,
            auto(Direction::Vertical, false),
            &mut Quiet {
                steps: 0,
                stop_after: None,
            },
        );
        let out = s.finish().unwrap();
        assert_eq!(out.height(), 2000);
        assert!(
            out.as_raw()
                == image::imageops::crop_imm(&content, 0, 1000, 500, 2000)
                    .to_image()
                    .as_raw()
        );
        assert!((p.at.1 - 1000).abs() <= p.per_notch, "{:?}", p.at);
    }

    #[test]
    fn content_that_does_not_scroll_gives_the_visible_area() {
        let still = RgbaImage::from_pixel(300, 200, Rgba([10, 20, 30, 255]));
        let mut p = Page::new(still.clone(), (300, 200), (0, 0));
        let (s, ended) = drive(
            &mut p,
            auto(Direction::Vertical, true),
            &mut Quiet {
                steps: 0,
                stop_after: None,
            },
        );
        assert_eq!(ended, Ended::Keep);
        assert_eq!(s.joins(), (0, 0));
        assert!(s.finish().unwrap().as_raw() == still.as_raw());
    }

    #[test]
    fn both_directions_cover_a_wide_and_tall_page() {
        let content = page(1300, 1800);
        let mut p = Page::new(content.clone(), (600, 700), (0, 0));
        let (s, _) = drive(
            &mut p,
            auto(Direction::Both, true),
            &mut Quiet {
                steps: 0,
                stop_after: None,
            },
        );
        let out = s.finish().unwrap();
        assert_eq!(out.dimensions(), (1300, 1800));
        assert!(out.as_raw() == content.as_raw());
    }
}
