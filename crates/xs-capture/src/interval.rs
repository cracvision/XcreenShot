//! Interval capture (CAP-049): the same capture again every few seconds, until the
//! user stops it, after a number of captures, after a time or at an hour.
//!
//! A series picks its area or window once, then runs the capture command of its
//! selection on a thread of its own, through the command registry (so each one is
//! logged and published like any other). A capture just like the last one kept is
//! dropped when the series says so ([`same`]). Several series can run at once; none
//! survives quitting the app (the quit dialog says so, APP-004).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, Weak};
use std::time::Duration;

use chrono::{DateTime, Local, NaiveTime, TimeDelta};
use image::RgbaImage;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use xs_core::core::QuitGuard;
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode, Lang, Source};

/// Shortest and longest time between captures, in seconds (CAP-049).
pub const MIN_EVERY: u32 = 3;
pub const MAX_EVERY: u32 = 86_400;
/// "Ignore small changes": fewer than 5 changed pixels in 1000.
const SMALL_PER_MILLE: u64 = 5;
/// The first capture waits this long for the selection that started it to end.
const FIRST_WAIT: Duration = Duration::from_secs(3);

/// The Interval options of the Image tab (CAP-049).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct IntervalTab {
    /// Seconds between captures.
    pub every: u32,
    /// `manual`, `count`, `duration` or `time`.
    pub stop: String,
    /// Stop after this many captures.
    pub count: u32,
    /// Stop after this many minutes.
    pub minutes: u32,
    /// Stop at this hour, `HH:MM` (local time).
    pub until: String,
    /// Drop a capture just like the last one kept.
    pub discard: bool,
    /// `exact` (any pixel counts) or `small` (fewer than 0.5 % of the pixels do not).
    pub sensitivity: String,
}

impl Default for IntervalTab {
    fn default() -> Self {
        IntervalTab {
            every: 60,
            stop: "manual".into(),
            count: 10,
            minutes: 60,
            until: "18:00".into(),
            discard: true,
            sensitivity: "exact".into(),
        }
    }
}

impl IntervalTab {
    pub fn validate(&self) -> Result<(), String> {
        if !(MIN_EVERY..=MAX_EVERY).contains(&self.every) {
            return Err(format!(
                "image.interval.every: between {MIN_EVERY} and {MAX_EVERY} seconds"
            ));
        }
        if !["manual", "count", "duration", "time"].contains(&self.stop.as_str()) {
            return Err("image.interval.stop: manual, count, duration or time".into());
        }
        if !(1..=100_000).contains(&self.count) {
            return Err("image.interval.count: between 1 and 100000".into());
        }
        if !(1..=10_080).contains(&self.minutes) {
            return Err("image.interval.minutes: between 1 and 10080".into());
        }
        parse_hour(&self.until).ok_or("image.interval.until: HH:MM")?;
        if !["exact", "small"].contains(&self.sensitivity.as_str()) {
            return Err("image.interval.sensitivity: exact or small".into());
        }
        Ok(())
    }

    /// As parameters of `capture.interval`.
    pub fn params(&self) -> Value {
        json!({
            "every": self.every,
            "stop": self.stop,
            "count": self.count,
            "minutes": self.minutes,
            "until": self.until,
            "discard": self.discard,
            "sensitivity": self.sensitivity,
        })
    }
}

fn parse_hour(s: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(s.trim(), "%H:%M").ok()
}

/// When a series ends by itself.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Stop {
    Manual,
    Count(u32),
    At(DateTime<Local>),
}

/// How a series runs.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plan {
    pub every: Duration,
    pub stop: Stop,
    pub discard: bool,
    pub small: bool,
}

impl Plan {
    /// From `capture.interval`'s parameters, started at `now`. Fewer than 3 s become
    /// 3 s (CAP-049).
    pub fn from_params(p: &Value, now: DateTime<Local>) -> Result<Plan, String> {
        let every = p["every"].as_f64().unwrap_or(60.0);
        let every = (every.round() as i64).clamp(MIN_EVERY as i64, MAX_EVERY as i64) as u64;
        let stop = match p["stop"].as_str().unwrap_or("manual") {
            "manual" => Stop::Manual,
            "count" => Stop::Count(p["count"].as_u64().unwrap_or(10).clamp(1, 100_000) as u32),
            "duration" => {
                let minutes = p["minutes"].as_u64().unwrap_or(60).clamp(1, 10_080) as i64;
                Stop::At(now + TimeDelta::minutes(minutes))
            }
            "time" => {
                let hour = p["until"].as_str().and_then(parse_hour).ok_or("until: HH:MM")?;
                Stop::At(next_time(now, hour))
            }
            other => return Err(format!("stop: unknown value {other}")),
        };
        Ok(Plan {
            every: Duration::from_secs(every),
            stop,
            discard: p["discard"].as_bool().unwrap_or(true),
            small: p["sensitivity"].as_str() == Some("small"),
        })
    }

    /// The moment the series ends by time, if it does.
    pub fn deadline(&self) -> Option<DateTime<Local>> {
        match self.stop {
            Stop::At(t) => Some(t),
            _ => None,
        }
    }
}

/// The next time the clock shows `hour` after `now` (today or tomorrow).
pub fn next_time(now: DateTime<Local>, hour: NaiveTime) -> DateTime<Local> {
    let mut day = now.date_naive();
    loop {
        if let Some(t) = day.and_time(hour).and_local_timezone(Local).earliest() {
            if t > now {
                return t;
            }
        }
        day = day.succ_opt().unwrap_or(day);
    }
}

/// Whether two captures count as the same for a series (CAP-049): every pixel, or all
/// but fewer than 0.5 % of them.
pub fn same(a: &RgbaImage, b: &RgbaImage, small: bool) -> bool {
    if a.dimensions() != b.dimensions() {
        return false;
    }
    if !small {
        return a.as_raw() == b.as_raw();
    }
    let total = a.width() as u64 * a.height() as u64;
    let mut changed = 0u64;
    for (p, q) in a.pixels().zip(b.pixels()) {
        if p != q {
            changed += 1;
            if changed * 1000 >= total * SMALL_PER_MILLE {
                return false;
            }
        }
    }
    true
}

/// The next capture after one planned at `planned` and taken at `now`: one interval
/// later, but turns missed while the computer slept are not made up (CAP-049).
pub fn next_after(planned: DateTime<Local>, now: DateTime<Local>, every: Duration) -> DateTime<Local> {
    let step = TimeDelta::from_std(every).unwrap_or(TimeDelta::seconds(60));
    let next = planned + step;
    if next <= now {
        now + step
    } else {
        next
    }
}

// ---------------------------------------------------------------------------
// Series

struct Inner {
    stopped: bool,
    saved: u32,
    next: DateTime<Local>,
    last: Option<RgbaImage>,
}

/// One running series.
pub struct Series {
    pub id: String,
    /// The preset's name, or none ("Interval").
    pub name: Option<String>,
    /// The preset that started it: its shortcut stops it (CAP-062).
    pub preset: Option<String>,
    command: String,
    params: Value,
    plan: Plan,
    started: DateTime<Local>,
    inner: Mutex<Inner>,
    wake: Condvar,
}

impl Series {
    /// CAP-049: the number of a new capture of the series, or none when it is just
    /// like the last one kept.
    pub fn accept(&self, image: &RgbaImage) -> Option<u32> {
        let mut inner = self.inner.lock().unwrap();
        if self.plan.discard {
            if inner
                .last
                .as_ref()
                .is_some_and(|last| same(last, image, self.plan.small))
            {
                return None;
            }
            inner.last = Some(image.clone());
        }
        inner.saved += 1;
        Some(inner.saved)
    }

    fn stop(&self) {
        self.inner.lock().unwrap().stopped = true;
        self.wake.notify_all();
    }

    fn saved(&self) -> u32 {
        self.inner.lock().unwrap().saved
    }

    /// An id that no other series has, also after a restart: for the library to
    /// group a series' captures (CAP-049, CAP-056).
    pub fn group(&self) -> String {
        format!("{}-{}", self.started.format("%Y%m%dT%H%M%S"), self.id)
    }

    pub fn title(&self, lang: Lang) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| tr(lang, "interval.name", &[]))
    }

    fn summary(&self, lang: Lang) -> Value {
        let inner = self.inner.lock().unwrap();
        json!({
            "id": self.id,
            "name": self.title(lang),
            "preset": self.preset,
            "command": self.command,
            "count": inner.saved,
            "every": self.plan.every.as_secs(),
            "startedAt": self.started.to_rfc3339(),
            "nextAt": inner.next.to_rfc3339(),
            "endsAt": self.plan.deadline().map(|d| d.to_rfc3339()),
            "stopAfter": match self.plan.stop { Stop::Count(n) => Some(n), _ => None },
        })
    }
}

/// The running series.
pub struct Intervals {
    core: Weak<Core>,
    list: Mutex<Vec<Arc<Series>>>,
    next_id: AtomicU64,
}

impl Intervals {
    pub fn new(core: Weak<Core>) -> Intervals {
        Intervals {
            core,
            list: Mutex::new(Vec::new()),
            next_id: AtomicU64::new(1),
        }
    }
}

fn service(core: &Core) -> Arc<Intervals> {
    core.service(|| Intervals::new(Weak::new()))
}

/// Sets up the series service with a way back to the core (from the extension).
pub fn install(core: &Arc<Core>) {
    core.service(|| Intervals::new(Arc::downgrade(core)));
    core.add_quit_guard(Arc::new(IntervalQuitGuard(Arc::downgrade(core))));
}

/// The series with this id, while it runs.
pub fn find(core: &Core, id: &str) -> Option<Arc<Series>> {
    service(core)
        .list
        .lock()
        .unwrap()
        .iter()
        .find(|s| s.id == id)
        .cloned()
}

/// Whether a series runs (the icon shows a clock, APP-002).
pub fn running(core: &Core) -> bool {
    !service(core).list.lock().unwrap().is_empty()
}

/// The running series: their id and title, for the icon's menu (CAP-005).
pub fn menu_items(core: &Core) -> Vec<(String, String)> {
    let lang = core.lang();
    service(core)
        .list
        .lock()
        .unwrap()
        .iter()
        .map(|s| {
            (
                s.id.clone(),
                tr(lang, "tray.stop_series", &[("name", &s.title(lang))]),
            )
        })
        .collect()
}

/// `capture.interval_state`.
pub fn state(core: &Core, lang: Lang) -> Value {
    let list: Vec<Value> = service(core)
        .list
        .lock()
        .unwrap()
        .iter()
        .map(|s| s.summary(lang))
        .collect();
    json!({
        "running": !list.is_empty(),
        "series": list,
        // The schedules (CAP-050), for the panel that shows both.
        "schedules": crate::schedule::list(core, lang)["schedules"],
    })
}

fn emit(core: &Core) {
    core.host().interval_event(state(core, core.lang()));
}

/// `capture.interval_stop`: one series, or all of them.
pub fn stop(core: &Core, lang: Lang, id: Option<&str>) -> CommandResult {
    let svc = service(core);
    let stopped: Vec<Arc<Series>> = {
        let mut list = svc.list.lock().unwrap();
        let (gone, kept): (Vec<_>, Vec<_>) = list.drain(..).partition(|s| id.is_none_or(|id| s.id == id));
        *list = kept;
        gone
    };
    if let (Some(id), true) = (id, stopped.is_empty()) {
        return Err(CommandError::new(
            ErrorCode::NotFound,
            tr(lang, "errors.series_not_found", &[("id", id)]),
        ));
    }
    for s in &stopped {
        s.stop();
    }
    emit(core);
    Ok(json!({ "stopped": stopped.iter().map(|s| s.id.clone()).collect::<Vec<_>>() }))
}

/// The series a preset started, if it still runs (CAP-062: its shortcut stops it).
pub fn of_preset(core: &Core, preset: &str) -> Option<String> {
    service(core)
        .list
        .lock()
        .unwrap()
        .iter()
        .find(|s| s.preset.as_deref() == Some(preset))
        .map(|s| s.id.clone())
}

/// Starts a series that runs `command` with `params` (the selection already made).
pub fn start(
    core: &Core,
    ctx: &Context,
    command: &str,
    params: Value,
    plan: Plan,
    name: Option<String>,
    preset: Option<String>,
) -> CommandResult {
    let svc = service(core);
    let now = Local::now();
    let n = svc.next_id.fetch_add(1, Ordering::Relaxed);
    let series = Arc::new(Series {
        id: format!("s{n}"),
        name,
        preset,
        command: command.to_string(),
        params,
        plan,
        started: now,
        inner: Mutex::new(Inner {
            stopped: false,
            saved: 0,
            next: now,
            last: None,
        }),
        wake: Condvar::new(),
    });
    svc.list.lock().unwrap().push(series.clone());
    tracing::info!(
        series = %series.id,
        command,
        every_s = plan.every.as_secs(),
        "interval series started"
    );
    let weak = svc.core.clone();
    let s = series.clone();
    let lang = ctx.lang;
    std::thread::Builder::new()
        .name(format!("interval-{}", series.id))
        .spawn(move || run_series(weak, s, lang))
        .map_err(|e| CommandError::internal(e.to_string()))?;
    emit(core);
    Ok(json!({ "status": "started", "series": series.summary(lang) }))
}

/// Why a series ended by itself.
enum End {
    Done,
    Problem(String),
}

fn run_series(weak: Weak<Core>, s: Arc<Series>, lang: Lang) {
    let mut first = true;
    let end = loop {
        // Wait for the next turn, the end time or a stop.
        {
            let mut inner = s.inner.lock().unwrap();
            loop {
                if inner.stopped {
                    break;
                }
                let now = Local::now();
                let until = match s.plan.deadline() {
                    Some(d) if d < inner.next => d,
                    _ => inner.next,
                };
                if until <= now {
                    break;
                }
                let wait = (until - now).to_std().unwrap_or(Duration::from_millis(10));
                inner = s.wake.wait_timeout(inner, wait).unwrap().0;
            }
            if inner.stopped {
                return;
            }
        }
        let Some(core) = weak.upgrade() else { return };
        if s.plan.deadline().is_some_and(|d| d <= Local::now()) {
            break End::Done;
        }
        let planned = s.inner.lock().unwrap().next;
        if let Err(problem) = turn(&core, &s, first, lang) {
            break End::Problem(problem);
        }
        first = false;
        if let Stop::Count(n) = s.plan.stop {
            if s.saved() >= n {
                break End::Done;
            }
        }
        s.inner.lock().unwrap().next = next_after(planned, Local::now(), s.plan.every);
        emit(&core);
    };
    let Some(core) = weak.upgrade() else { return };
    // Gone from the list, unless a stop already took it.
    let svc = service(&core);
    let was_listed = {
        let mut list = svc.list.lock().unwrap();
        let before = list.len();
        list.retain(|x| x.id != s.id);
        list.len() != before
    };
    if !was_listed {
        return;
    }
    tracing::info!(series = %s.id, saved = s.saved(), "interval series ended");
    let lang = core.lang();
    let body = match end {
        End::Done => tr(
            lang,
            "notify.series_done",
            &[("name", &s.title(lang)), ("count", &s.saved().to_string())],
        ),
        End::Problem(why) => why,
    };
    core.host().notify("captures", &tr(lang, "app.name", &[]), &body);
    emit(&core);
}

/// One turn: the capture, unless the session is locked or another capture is under
/// way (then it is skipped). An error that means the window or monitor is gone ends
/// the series with its reason.
fn turn(core: &Core, s: &Series, first: bool, lang: Lang) -> Result<(), String> {
    if crate::platform::session_locked() {
        tracing::info!(series = %s.id, "interval: session locked, turn skipped");
        return Ok(());
    }
    let mut params = s.params.clone();
    params["series"] = json!(s.id);
    let started = std::time::Instant::now();
    loop {
        match core.call(Source::Scheduled, &s.command, params.clone(), None) {
            Ok(_) => return Ok(()),
            // The selection that started the series may still be closing.
            Err(e) if e.code == ErrorCode::Busy && first && started.elapsed() < FIRST_WAIT => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) if e.code == ErrorCode::Busy => {
                tracing::info!(series = %s.id, "interval: another capture under way, turn skipped");
                return Ok(());
            }
            Err(e) if s.command == "capture.window" && e.code == ErrorCode::NotFound => {
                return Err(tr(lang, "notify.series_window_gone", &[("name", &s.title(lang))]));
            }
            Err(e)
                if s.command == "capture.region"
                    && e.details.as_ref().is_some_and(|d| d.get("monitor").is_some()) =>
            {
                return Err(tr(
                    lang,
                    "notify.series_monitor_gone",
                    &[("name", &s.title(lang))],
                ));
            }
            Err(e) => {
                tracing::warn!(series = %s.id, code = e.code.as_str(), error = %e.message, "interval: turn failed");
                return Ok(());
            }
        }
    }
}

/// Quitting ends every series (APP-004): the quit dialog lists them.
struct IntervalQuitGuard(Weak<Core>);

impl QuitGuard for IntervalQuitGuard {
    fn pending(&self, lang: Lang) -> Vec<Value> {
        let Some(core) = self.0.upgrade() else {
            return Vec::new();
        };
        service(&core)
            .list
            .lock()
            .unwrap()
            .iter()
            .map(|s| json!({ "kind": "series", "id": s.id, "name": s.title(lang) }))
            .collect()
    }

    fn before_quit(&self) {
        if let Some(core) = self.0.upgrade() {
            for s in service(&core).list.lock().unwrap().drain(..) {
                s.stop();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(h: u32, m: u32, s: u32) -> DateTime<Local> {
        Local.with_ymd_and_hms(2026, 11, 12, h, m, s).single().unwrap()
    }

    #[test]
    fn fewer_than_three_seconds_become_three() {
        let p = Plan::from_params(&json!({"every": 1}), at(10, 0, 0)).unwrap();
        assert_eq!(p.every, Duration::from_secs(3));
        let p = Plan::from_params(&json!({"every": 999_999}), at(10, 0, 0)).unwrap();
        assert_eq!(p.every, Duration::from_secs(86_400));
    }

    #[test]
    fn stop_conditions() {
        let now = at(10, 0, 0);
        assert_eq!(Plan::from_params(&json!({}), now).unwrap().stop, Stop::Manual);
        assert_eq!(
            Plan::from_params(&json!({"stop": "count", "count": 12}), now)
                .unwrap()
                .stop,
            Stop::Count(12)
        );
        assert_eq!(
            Plan::from_params(&json!({"stop": "duration", "minutes": 30}), now)
                .unwrap()
                .stop,
            Stop::At(at(10, 30, 0))
        );
        // An hour already gone today is tomorrow's.
        let tomorrow = Plan::from_params(&json!({"stop": "time", "until": "09:15"}), now).unwrap();
        assert_eq!(tomorrow.deadline().unwrap(), at(9, 15, 0) + TimeDelta::days(1));
        let today = Plan::from_params(&json!({"stop": "time", "until": "18:00"}), now).unwrap();
        assert_eq!(today.deadline().unwrap(), at(18, 0, 0));
        assert!(Plan::from_params(&json!({"stop": "time", "until": "6pm"}), now).is_err());
    }

    #[test]
    fn missed_turns_are_not_made_up() {
        let every = Duration::from_secs(5);
        assert_eq!(next_after(at(10, 0, 0), at(10, 0, 1), every), at(10, 0, 5));
        // Asleep for a minute: the next one is one interval after waking up.
        assert_eq!(next_after(at(10, 0, 0), at(10, 1, 0), every), at(10, 1, 5));
    }

    #[test]
    fn same_images_exact_and_with_small_changes() {
        let a = RgbaImage::from_pixel(100, 100, image::Rgba([10, 20, 30, 255]));
        let mut b = a.clone();
        assert!(same(&a, &b, false));
        // 40 pixels of 10 000 (0.4 %): a change for Exact, not for Small.
        for x in 0..40 {
            b.put_pixel(x, 0, image::Rgba([200, 0, 0, 255]));
        }
        assert!(!same(&a, &b, false));
        assert!(same(&a, &b, true));
        // 60 pixels (0.6 %): a change for both.
        for x in 40..60 {
            b.put_pixel(x, 0, image::Rgba([200, 0, 0, 255]));
        }
        assert!(!same(&a, &b, true));
        assert!(!same(&a, &RgbaImage::new(100, 99), true));
    }

    #[test]
    fn a_series_keeps_only_what_changed() {
        let plan = Plan::from_params(&json!({"every": 5}), at(10, 0, 0)).unwrap();
        let s = Series {
            id: "s1".into(),
            name: None,
            preset: None,
            command: "capture.fullscreen".into(),
            params: json!({}),
            plan,
            started: at(10, 0, 0),
            inner: Mutex::new(Inner {
                stopped: false,
                saved: 0,
                next: at(10, 0, 0),
                last: None,
            }),
            wake: Condvar::new(),
        };
        let a = RgbaImage::from_pixel(8, 8, image::Rgba([1, 2, 3, 255]));
        let b = RgbaImage::from_pixel(8, 8, image::Rgba([9, 9, 9, 255]));
        assert_eq!(s.accept(&a), Some(1));
        assert_eq!(s.accept(&a), None);
        assert_eq!(s.accept(&b), Some(2));
        assert_eq!(s.accept(&a), Some(3));
    }
}
