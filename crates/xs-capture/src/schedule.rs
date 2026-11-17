//! Scheduled capture (CAP-050): a capture at a day and minute, once or again every
//! day, every weekday or every week.
//!
//! Schedules are kept in the app's memory, so they outlive a restart. One thread
//! waits for the next one and fires it at second 0 of its minute. A capture that the
//! app missed (closed, or the computer asleep) is still made if the app is back within
//! five minutes, and says how late it was; later than that it is skipped with a
//! notification. A locked session or a window that is gone skip it too.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex, Weak};
use std::time::Duration;

use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveTime, TimeDelta, Weekday};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode, Lang, Source};

/// A capture missed by less than this is made late; by more, skipped (CAP-050).
pub const LATE_LIMIT: TimeDelta = TimeDelta::minutes(5);
/// Where the schedules are kept between runs.
const MEMORY_KEY: &str = "capture.schedules";
/// The longest wait before looking again (the clock may change, the computer sleep).
const RECHECK: Duration = Duration::from_secs(30);

/// The Scheduled options of the Image tab (CAP-050).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ScheduleTab {
    /// `YYYY-MM-DD`; empty is today.
    pub date: String,
    /// `HH:MM`, local time.
    pub time: String,
    /// `once`, `daily`, `weekdays` or `weekly`.
    pub repeat: String,
}

impl Default for ScheduleTab {
    fn default() -> Self {
        ScheduleTab {
            date: String::new(),
            time: "09:00".into(),
            repeat: "once".into(),
        }
    }
}

impl ScheduleTab {
    pub fn validate(&self) -> Result<(), String> {
        if !self.date.is_empty() && parse_date(&self.date).is_none() {
            return Err("image.schedule.date: YYYY-MM-DD".into());
        }
        parse_time(&self.time).ok_or("image.schedule.time: HH:MM")?;
        Repeat::parse(&self.repeat).ok_or("image.schedule.repeat: once, daily, weekdays or weekly")?;
        Ok(())
    }

    /// As parameters of `capture.schedule`.
    pub fn params(&self) -> Value {
        json!({ "date": self.date, "time": self.time, "repeat": self.repeat })
    }
}

fn parse_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").ok()
}

fn parse_time(s: &str) -> Option<NaiveTime> {
    NaiveTime::parse_from_str(s.trim(), "%H:%M").ok()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Repeat {
    Once,
    Daily,
    Weekdays,
    Weekly,
}

impl Repeat {
    pub fn parse(s: &str) -> Option<Repeat> {
        Some(match s {
            "once" | "" => Repeat::Once,
            "daily" => Repeat::Daily,
            "weekdays" => Repeat::Weekdays,
            "weekly" => Repeat::Weekly,
            _ => return None,
        })
    }
}

/// The moment of a date and minute, local time. A minute that does not exist (the
/// clock jumps forward) moves to the next one that does.
fn local(date: NaiveDate, time: NaiveTime) -> Option<DateTime<Local>> {
    (0..120).find_map(|m| {
        date.and_time(time)
            .checked_add_signed(TimeDelta::minutes(m))?
            .and_local_timezone(Local)
            .earliest()
    })
}

/// When to capture from `capture.schedule`'s parameters: `date` (empty: today) and
/// `time`, at second 0. A moment already gone is refused (CAP-050).
pub fn first_time(p: &Value, now: DateTime<Local>) -> Result<DateTime<Local>, String> {
    let date = match p["date"].as_str().unwrap_or("") {
        "" => now.date_naive(),
        d => parse_date(d).ok_or("date: YYYY-MM-DD")?,
    };
    let time = p["time"].as_str().and_then(parse_time).ok_or("time: HH:MM")?;
    let at = local(date, time).ok_or("that time does not exist")?;
    if at <= now {
        return Err("past".into());
    }
    Ok(at)
}

/// The next capture after one at `at`, or none for Once.
pub fn following(at: DateTime<Local>, repeat: Repeat) -> Option<DateTime<Local>> {
    let time = at.time();
    let mut day = at.date_naive();
    let step = match repeat {
        Repeat::Once => return None,
        Repeat::Weekly => 7,
        Repeat::Daily | Repeat::Weekdays => 1,
    };
    loop {
        day = day.checked_add_days(chrono::Days::new(step))?;
        if repeat == Repeat::Weekdays && matches!(day.weekday(), Weekday::Sat | Weekday::Sun) {
            continue;
        }
        return local(day, time);
    }
}

/// What to do with a capture planned at `at`, now.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Due {
    /// Not yet: wait this long.
    Wait(Duration),
    /// Now, this many seconds after its time.
    Now(i64),
    /// Too late: it is skipped.
    Missed,
}

pub fn due(at: DateTime<Local>, now: DateTime<Local>) -> Due {
    if now < at {
        return Due::Wait((at - now).to_std().unwrap_or(Duration::from_millis(1)));
    }
    let late = now - at;
    if late <= LATE_LIMIT {
        Due::Now(late.num_seconds())
    } else {
        Due::Missed
    }
}

/// The next time of a repeating schedule after one that was made or skipped: the
/// first one still to come (or within the late limit), without the ones the app
/// missed meanwhile.
pub fn next_after(at: DateTime<Local>, repeat: Repeat, now: DateTime<Local>) -> Option<DateTime<Local>> {
    let mut next = following(at, repeat)?;
    while next + LATE_LIMIT < now {
        next = following(next, repeat)?;
    }
    Some(next)
}

// ---------------------------------------------------------------------------
// The schedules

/// One scheduled capture, as kept between runs.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub id: String,
    /// The preset's name, or none ("Scheduled").
    pub name: Option<String>,
    /// The preset that made it: its shortcut cancels it (CAP-062).
    pub preset: Option<String>,
    pub command: String,
    pub params: Value,
    /// The next capture, local time (RFC 3339).
    pub at: String,
    pub repeat: Repeat,
}

impl Entry {
    fn when(&self) -> Option<DateTime<Local>> {
        DateTime::parse_from_rfc3339(&self.at)
            .ok()
            .map(|t| t.with_timezone(&Local))
    }

    pub fn title(&self, lang: Lang) -> String {
        self.name
            .clone()
            .unwrap_or_else(|| tr(lang, "schedule.name", &[]))
    }

    fn summary(&self, lang: Lang) -> Value {
        json!({
            "id": self.id,
            "name": self.title(lang),
            "preset": self.preset,
            "command": self.command,
            "at": self.at,
            "repeat": self.repeat,
        })
    }
}

pub struct Schedules {
    core: Weak<Core>,
    list: Mutex<Vec<Entry>>,
    wake: Condvar,
    next_id: AtomicU64,
}

fn service(core: &Core) -> Arc<Schedules> {
    core.service(|| Schedules {
        core: Weak::new(),
        list: Mutex::new(Vec::new()),
        wake: Condvar::new(),
        next_id: AtomicU64::new(1),
    })
}

/// Loads the schedules kept from the last run and starts waiting for them (from the
/// extension).
pub fn install(core: &Arc<Core>) {
    let kept: Vec<Entry> = core
        .recall(MEMORY_KEY)
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();
    let next = kept
        .iter()
        .filter_map(|e| e.id.trim_start_matches('p').parse::<u64>().ok())
        .max()
        .unwrap_or(0)
        + 1;
    let svc = core.service(|| Schedules {
        core: Arc::downgrade(core),
        list: Mutex::new(kept),
        wake: Condvar::new(),
        next_id: AtomicU64::new(next),
    });
    // Only a running app waits: tests build cores without one.
    if svc.core.upgrade().is_some() && !cfg!(test) {
        let weak = svc.core.clone();
        let _ = std::thread::Builder::new()
            .name("schedules".into())
            .spawn(move || wait_loop(weak));
    }
}

fn save(core: &Core, list: &[Entry]) {
    if list.is_empty() {
        core.forget(MEMORY_KEY);
    } else {
        core.remember(MEMORY_KEY, json!(list));
    }
}

fn emit(core: &Core) {
    core.host()
        .interval_event(crate::interval::state(core, core.lang()));
}

/// Whether a capture is scheduled (the icon shows a clock, APP-002).
pub fn any(core: &Core) -> bool {
    !service(core).list.lock().unwrap().is_empty()
}

/// The schedules for the icon's menu: id and "Stop capture: <name> (next: <when>)".
pub fn menu_items(core: &Core) -> Vec<(String, String)> {
    let lang = core.lang();
    service(core)
        .list
        .lock()
        .unwrap()
        .iter()
        .map(|e| {
            let when = e
                .when()
                .map(|t| t.format("%Y-%m-%d %H:%M").to_string())
                .unwrap_or_default();
            (
                e.id.clone(),
                tr(
                    lang,
                    "tray.stop_schedule",
                    &[("name", &e.title(lang)), ("when", &when)],
                ),
            )
        })
        .collect()
}

/// `capture.schedule_list`.
pub fn list(core: &Core, lang: Lang) -> Value {
    let list: Vec<Value> = service(core)
        .list
        .lock()
        .unwrap()
        .iter()
        .map(|e| e.summary(lang))
        .collect();
    json!({ "schedules": list })
}

/// The schedule a preset (or the panel) made, if it is still pending.
pub fn of_preset(core: &Core, preset: &str) -> Option<String> {
    service(core)
        .list
        .lock()
        .unwrap()
        .iter()
        .find(|e| e.preset.as_deref() == Some(preset))
        .map(|e| e.id.clone())
}

/// `capture.schedule_cancel`: one schedule, or all.
pub fn cancel(core: &Core, lang: Lang, id: Option<&str>) -> CommandResult {
    let svc = service(core);
    let gone: Vec<String> = {
        let mut list = svc.list.lock().unwrap();
        let (gone, kept): (Vec<_>, Vec<_>) = list.drain(..).partition(|e| id.is_none_or(|id| e.id == id));
        *list = kept;
        save(core, &list);
        gone.into_iter().map(|e| e.id).collect()
    };
    if let (Some(id), true) = (id, gone.is_empty()) {
        return Err(CommandError::new(
            ErrorCode::NotFound,
            tr(lang, "errors.schedule_not_found", &[("id", id)]),
        ));
    }
    svc.wake.notify_all();
    emit(core);
    Ok(json!({ "cancelled": gone }))
}

/// Adds a schedule for `command` with `params` (the selection already made).
#[allow(clippy::too_many_arguments)]
pub fn add(
    core: &Core,
    ctx: &Context,
    command: &str,
    params: Value,
    at: DateTime<Local>,
    repeat: Repeat,
    name: Option<String>,
    preset: Option<String>,
) -> CommandResult {
    let svc = service(core);
    let n = svc.next_id.fetch_add(1, Ordering::Relaxed);
    let entry = Entry {
        id: format!("p{n}"),
        name,
        preset,
        command: command.to_string(),
        params,
        at: at.to_rfc3339(),
        repeat,
    };
    let summary = entry.summary(ctx.lang);
    {
        let mut list = svc.list.lock().unwrap();
        list.push(entry);
        save(core, &list);
    }
    svc.wake.notify_all();
    tracing::info!(at = %at, ?repeat, command, "capture scheduled");
    emit(core);
    Ok(json!({ "status": "scheduled", "schedule": summary }))
}

/// Time for the app to finish starting (its windows and icon) before a capture that
/// is already due is made.
const SETTLE: Duration = Duration::from_secs(3);

fn wait_loop(weak: Weak<Core>) {
    // The commands exist once the core is built; the app needs a moment more.
    loop {
        match weak.upgrade() {
            None => return,
            Some(core) if core.is_ready() => break,
            Some(_) => std::thread::sleep(Duration::from_millis(100)),
        }
    }
    std::thread::sleep(SETTLE);
    loop {
        let Some(core) = weak.upgrade() else { return };
        let svc = service(&core);
        // The first schedule that is due, if any; otherwise wait.
        let fired: Option<(Entry, Due)> = {
            let list = svc.list.lock().unwrap();
            let now = Local::now();
            let mut soonest: Option<Duration> = None;
            let mut found = None;
            for e in list.iter() {
                let Some(at) = e.when() else { continue };
                match due(at, now) {
                    Due::Wait(d) => soonest = Some(soonest.map_or(d, |s| s.min(d))),
                    other => {
                        found = Some((e.clone(), other));
                        break;
                    }
                }
            }
            if found.is_none() {
                let wait = soonest.unwrap_or(RECHECK).min(RECHECK);
                drop(core);
                let _ = svc.wake.wait_timeout(list, wait).unwrap();
                continue;
            }
            found
        };
        let Some((entry, due)) = fired else { continue };
        let lang = core.lang();
        match due {
            Due::Now(late) => fire(&core, &entry, late, lang),
            _ => {
                tracing::info!(schedule = %entry.id, at = %entry.at, "scheduled capture missed");
                core.host().notify(
                    "problems",
                    &tr(lang, "app.name", &[]),
                    &tr(lang, "notify.schedule_missed", &[("name", &entry.title(lang))]),
                );
            }
        }
        // Next time, or gone.
        {
            let mut list = svc.list.lock().unwrap();
            let next = entry
                .when()
                .and_then(|at| next_after(at, entry.repeat, Local::now()));
            match (list.iter().position(|e| e.id == entry.id), next) {
                (Some(i), Some(next)) => list[i].at = next.to_rfc3339(),
                (Some(i), None) => {
                    list.remove(i);
                }
                // Cancelled meanwhile.
                (None, _) => {}
            }
            save(&core, &list);
        }
        emit(&core);
    }
}

/// Makes one scheduled capture now, `late` seconds after its time; a locked session,
/// a window that is gone or a capture that cannot start skip it, with a notice.
fn fire(core: &Core, e: &Entry, late: i64, lang: Lang) {
    let skip = |why: &str| {
        tracing::info!(schedule = %e.id, why, "scheduled capture skipped");
        core.host().notify(
            "problems",
            &tr(lang, "app.name", &[]),
            &tr(lang, why, &[("name", &e.title(lang))]),
        );
    };
    if crate::platform::session_locked() {
        return skip("notify.schedule_locked");
    }
    let mut params = e.params.clone();
    params["schedule"] = json!({ "id": e.id, "at": e.at, "late": late });
    let started = std::time::Instant::now();
    loop {
        match core.call(Source::Scheduled, &e.command, params.clone(), None) {
            Ok(_) => return,
            // Another capture is finishing: wait a little.
            Err(err) if err.code == ErrorCode::Busy && started.elapsed() < Duration::from_secs(10) => {
                std::thread::sleep(Duration::from_millis(200));
            }
            Err(err) if e.command == "capture.window" && err.code == ErrorCode::NotFound => {
                return skip("notify.schedule_window_gone");
            }
            Err(err) => {
                tracing::warn!(schedule = %e.id, code = err.code.as_str(), error = %err.message, "scheduled capture failed");
                return skip("notify.schedule_failed");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at(d: u32, h: u32, m: u32, s: u32) -> DateTime<Local> {
        // November 2026: the 12th is a Thursday.
        Local.with_ymd_and_hms(2026, 11, d, h, m, s).single().unwrap()
    }

    #[test]
    fn the_first_time_is_at_second_zero_and_not_in_the_past() {
        let now = at(12, 10, 14, 20);
        let t = first_time(&json!({"time": "10:15"}), now).unwrap();
        assert_eq!(t, at(12, 10, 15, 0));
        let t = first_time(&json!({"date": "2026-11-13", "time": "08:00"}), now).unwrap();
        assert_eq!(t, at(13, 8, 0, 0));
        assert_eq!(first_time(&json!({"time": "10:14"}), now).unwrap_err(), "past");
        assert!(first_time(&json!({"time": "25:00"}), now).is_err());
    }

    #[test]
    fn repeats() {
        let thu = at(12, 9, 0, 0);
        assert_eq!(following(thu, Repeat::Once), None);
        assert_eq!(following(thu, Repeat::Daily), Some(at(13, 9, 0, 0)));
        assert_eq!(following(thu, Repeat::Weekly), Some(at(19, 9, 0, 0)));
        // Friday's weekday is Monday.
        assert_eq!(
            following(at(13, 9, 0, 0), Repeat::Weekdays),
            Some(at(16, 9, 0, 0))
        );
    }

    #[test]
    fn late_within_five_minutes_still_captures() {
        let t = at(12, 10, 15, 0);
        assert!(matches!(due(t, at(12, 10, 14, 59)), Due::Wait(_)));
        assert_eq!(due(t, at(12, 10, 15, 0)), Due::Now(0));
        assert_eq!(due(t, at(12, 10, 19, 30)), Due::Now(270));
        assert_eq!(due(t, at(12, 10, 25, 0)), Due::Missed);
    }

    #[test]
    fn a_daily_schedule_missed_for_days_goes_on_from_the_next_one() {
        let t = at(9, 9, 0, 0);
        assert_eq!(
            next_after(t, Repeat::Daily, at(12, 10, 0, 0)),
            Some(at(13, 9, 0, 0))
        );
        assert_eq!(
            next_after(t, Repeat::Daily, at(9, 9, 0, 5)),
            Some(at(10, 9, 0, 0))
        );
        assert_eq!(next_after(t, Repeat::Once, at(12, 10, 0, 0)), None);
    }

    #[test]
    fn tab_validation() {
        assert!(ScheduleTab::default().validate().is_ok());
        let bad = ScheduleTab {
            time: "9am".into(),
            ..ScheduleTab::default()
        };
        assert!(bad.validate().is_err());
    }
}
