//! Screen recording and encoding (VID-). The engine records an area of one monitor to
//! an MP4 (H.264) file with the system's encoders (D-04): on Windows, Desktop
//! Duplication gives a new picture only when the screen changes (variable frame rate,
//! VID-025), the GPU crops, scales and converts it (VID-028), and Media Foundation
//! encodes it, with the hardware encoder when there is one and software otherwise
//! (VID-029).

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use serde::Serialize;

#[cfg(windows)]
mod audio;
#[cfg(test)]
mod demo;
pub mod mix;
#[cfg(windows)]
mod power;
#[cfg(windows)]
pub use power::{stay_awake, watch_suspend, StayAwake};

/// A sound input (only listed on Windows yet).
#[cfg(not(windows))]
#[derive(Debug, Clone, Serialize)]
pub struct Input {
    pub id: String,
    pub name: String,
    pub wireless: bool,
}

/// The sound inputs (none listed on this platform yet).
#[cfg(not(windows))]
pub fn inputs() -> Result<(Vec<Input>, Option<String>), String> {
    Ok((Vec::new(), None))
}

/// Whether an input is muted (not told on this platform yet).
#[cfg(not(windows))]
pub fn input_muted(_device: Option<&str>) -> Option<bool> {
    None
}

#[cfg(windows)]
pub mod camera;
#[cfg(windows)]
pub mod campreview;
#[cfg(windows)]
mod meter;
#[cfg(windows)]
pub use meter::Meter;

/// The meters before recording (not on this platform yet).
#[cfg(not(windows))]
pub struct Meter {
    pub microphone: Option<Microphone>,
    pub system_audio: bool,
}

#[cfg(not(windows))]
impl Meter {
    pub fn start(microphone: Option<Microphone>, system_audio: bool) -> Meter {
        Meter {
            microphone,
            system_audio,
        }
    }

    pub fn levels(&self) -> mix::Levels {
        mix::Levels::default()
    }

    pub fn running(&self) -> bool {
        false
    }
}

/// Keeps the computer awake (nothing to do on this platform yet).
#[cfg(not(windows))]
pub struct StayAwake;

#[cfg(not(windows))]
pub fn stay_awake() -> Option<StayAwake> {
    None
}

/// Sleep and wake (not watched on this platform yet).
#[cfg(not(windows))]
pub fn watch_suspend(_f: impl Fn(bool) + Send + Sync + 'static) {}
pub mod camlook;
pub mod effects;
pub mod timeline;
#[cfg(windows)]
mod win;
#[cfg(windows)]
mod win_edit;
#[cfg(windows)]
mod win_fx;
/// What a video file holds: width, height, frames and the last frame's time (s).
#[cfg(windows)]
pub use audio::{input_muted, inputs, Input};
#[cfg(windows)]
pub use win::{audio_info, first_frame, free_space, info, probe, size, warm_up};
/// Editing (ADR 0008): the frame at a moment and a new MP4 from parts of a file.
#[cfg(windows)]
pub use win_edit::{combine, export, facts, frame_at, frames, recover, recovered_length, remux, Piece};

/// Gets each frame of [`frames`]: its time in the result, width, height and BGRA
/// pixels; returns false to stop.
pub type FrameSink<'a> = dyn FnMut(f64, u32, u32, &[u8]) -> bool + 'a;

/// What the editor needs to open a video (ADR 0008).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Facts {
    pub width: u32,
    pub height: u32,
    /// Seconds.
    pub duration: f64,
    /// The frame rate the file says it has.
    pub fps: u32,
    pub codec: String,
    pub sound: bool,
}

/// What a video holds, quickly (not available on this platform yet).
#[cfg(not(windows))]
pub fn facts(_path: &std::path::Path) -> Result<Facts, String> {
    Err("not available on this platform yet".into())
}

/// The frame at a moment of a video (not available on this platform yet).
#[cfg(not(windows))]
pub fn frame_at(_path: &std::path::Path, _seconds: f64) -> Result<(u32, u32, Vec<u8>), String> {
    Err("not available on this platform yet".into())
}

/// One video of a combination (VID-067): its file and the parts kept.
#[cfg(not(windows))]
#[derive(Debug, Clone)]
pub struct Piece {
    pub path: std::path::PathBuf,
    pub keep: Vec<timeline::Span>,
}

/// Videos one after the other (not available on this platform yet).
#[cfg(not(windows))]
pub fn combine(
    _pieces: &[Piece],
    _dst: &std::path::Path,
    _progress: &mut dyn FnMut(f64),
    _cancel: &AtomicBool,
) -> Result<(u32, u32), String> {
    Err("not available on this platform yet".into())
}

/// The frames of parts of a video at a rate (not available on this platform yet).
#[cfg(not(windows))]
pub fn frames(
    _path: &std::path::Path,
    _spans: &[timeline::Span],
    _fps: f64,
    _each: &mut FrameSink,
) -> Result<(), String> {
    Err("not available on this platform yet".into())
}

/// A new video from parts of a file (not available on this platform yet).
#[cfg(not(windows))]
pub fn export(
    _src: &std::path::Path,
    _dst: &std::path::Path,
    _keep: &[timeline::Span],
    _sound: bool,
    _progress: &mut dyn FnMut(f64),
    _cancel: &AtomicBool,
) -> Result<(), String> {
    Err("not available on this platform yet".into())
}

/// Free bytes on the disk of `path` (not available on this platform yet).
#[cfg(not(windows))]
pub fn free_space(_path: &std::path::Path) -> Option<u64> {
    None
}

/// What a video holds (not available on this platform yet).
#[cfg(not(windows))]
pub fn info(_path: &std::path::Path) -> Result<Info, String> {
    Err("not available on this platform yet".into())
}

/// A video's size (not available on this platform yet).
#[cfg(not(windows))]
pub fn size(_path: &std::path::Path) -> Result<(u32, u32), String> {
    Err("not available on this platform yet".into())
}

/// A recording cut short as an MP4 (not available on this platform yet).
#[cfg(not(windows))]
pub fn recover(_fragmented: &std::path::Path, _dst: &std::path::Path) -> Result<f64, String> {
    Err("not available on this platform yet".into())
}

/// How long a recording cut short is (not available on this platform yet).
#[cfg(not(windows))]
pub fn recovered_length(_fragmented: &std::path::Path) -> Option<f64> {
    None
}

/// Gets the recording engine ready (nothing to do on this platform yet).
#[cfg(not(windows))]
pub fn warm_up(_area: Area, _software_only: bool) {}

/// The first frame of a video (not available on this platform yet).
#[cfg(not(windows))]
pub fn first_frame(_path: &std::path::Path) -> Result<(u32, u32, Vec<u8>), String> {
    Err("not available on this platform yet".into())
}

/// Rectangle in physical pixels of the virtual screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Area {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Video quality (VID-025): the most frames per second.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

impl Quality {
    pub fn parse(s: &str) -> Option<Quality> {
        Some(match s {
            "very_low" | "veryLow" => Quality::VeryLow,
            "low" => Quality::Low,
            "medium" => Quality::Medium,
            "high" => Quality::High,
            "very_high" | "veryHigh" => Quality::VeryHigh,
            _ => return None,
        })
    }

    pub fn fps(self) -> u32 {
        match self {
            Quality::VeryLow => 5,
            Quality::Low => 10,
            Quality::Medium => 15,
            Quality::High => 30,
            Quality::VeryHigh => 60,
        }
    }

    /// Bits per second for `width` × `height`: proportional to the pixels and the
    /// frames, so text stays readable in High and Very high (VID-025).
    pub fn bitrate(self, width: u32, height: u32) -> u32 {
        let per_pixel = match self {
            Quality::VeryLow | Quality::Low => 0.06,
            Quality::Medium => 0.07,
            Quality::High => 0.08,
            Quality::VeryHigh => 0.07,
        };
        let bits = width as f64 * height as f64 * self.fps() as f64 * per_pixel;
        bits.clamp(500_000.0, 60_000_000.0) as u32
    }
}

/// How big the video is (VID-028).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MaxSize {
    /// The area's physical pixels.
    Original,
    /// The area's logical size on monitors scaled above 100 % (the default).
    ReduceHighDensity,
    /// At most 1920 × 1080, in either orientation.
    Fit1080,
}

impl MaxSize {
    pub fn parse(s: &str) -> Option<MaxSize> {
        Some(match s {
            "original" => MaxSize::Original,
            "reduce_high_density" | "reduceHighDensity" => MaxSize::ReduceHighDensity,
            "fit_1080" | "fit1080" => MaxSize::Fit1080,
            _ => return None,
        })
    }
}

/// Free space under which a recording stops and saves itself (VID-023).
pub const MIN_FREE: u64 = 1 << 30;

/// Whether `free` bytes are too few to go on recording (VID-023).
pub fn low_disk(free: u64) -> bool {
    free < MIN_FREE
}

/// What a video file holds (VID-064).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Info {
    pub width: u32,
    pub height: u32,
    /// Seconds.
    pub duration: f64,
    pub frames: u64,
    /// Frames per second on average and in the busiest second.
    pub fps_average: f64,
    pub fps_max: u32,
    /// `H.264`, `H.265` or another name.
    pub codec: String,
    /// Bits per second of the whole file.
    pub bitrate: u64,
    /// Channels and rate of its sound track, if it has one.
    pub audio: Option<(u32, u32)>,
}

/// The most frames inside any one second of `times` (seconds, in order).
pub fn busiest_second(times: &[f64]) -> u32 {
    let mut best = 0;
    let mut start = 0;
    for (end, t) in times.iter().enumerate() {
        // A frame exactly one second before is outside (with room for rounding).
        while times[start] <= t - 1.0 + 1e-6 {
            start += 1;
        }
        best = best.max(end - start + 1);
    }
    best as u32
}

/// The largest video the engine writes.
pub const MAX_VIDEO: (u32, u32) = (3840, 2160);
/// The smallest area (VID-003).
pub const MIN_AREA: u32 = 64;

/// The video's size for an area of `width` × `height` physical pixels on a monitor
/// at `scale`: reduced as `max` says, within [`MAX_VIDEO`], always even (VID-028).
pub fn video_size(width: u32, height: u32, scale: f64, max: MaxSize) -> (u32, u32) {
    let (mut w, mut h) = (width as f64, height as f64);
    match max {
        MaxSize::Original => {}
        MaxSize::ReduceHighDensity => {
            if scale > 1.0 {
                w /= scale;
                h /= scale;
            }
        }
        MaxSize::Fit1080 => {
            let (lw, lh) = if w >= h {
                (1920.0, 1080.0)
            } else {
                (1080.0, 1920.0)
            };
            let f = (lw / w).min(lh / h).min(1.0);
            w *= f;
            h *= f;
        }
    }
    // Within the limit, either orientation.
    let (lw, lh) = if w >= h {
        (MAX_VIDEO.0 as f64, MAX_VIDEO.1 as f64)
    } else {
        (MAX_VIDEO.1 as f64, MAX_VIDEO.0 as f64)
    };
    let f = (lw / w).min(lh / h).min(1.0);
    let even = |v: f64| ((v * f).round() as u32 / 2 * 2).max(2);
    (even(w), even(h))
}

/// What to record.
#[derive(Debug, Clone)]
pub struct Options {
    /// Within one monitor.
    pub area: Area,
    /// That monitor's scale (1.5 at 150 %).
    pub scale: f64,
    pub quality: Quality,
    pub max_size: MaxSize,
    /// Stops by itself after this long (VID-015).
    pub limit: Option<Duration>,
    /// Never use the hardware encoder (VID-029).
    pub software_only: bool,
    /// The pointer, with its shape at each moment (VID-044).
    pub cursor: bool,
    /// What the computer plays (VID-040).
    pub system_audio: bool,
    /// Cursor and keyboard effects (VID-045 to VID-047); they can change while it
    /// records.
    pub effects: effects::Effects,
    /// A window to follow (its handle): the area moves and resizes with it
    /// (VID-005, Window area mode).
    pub window: Option<isize>,
    pub path: PathBuf,
    /// Where it is written while it records (VID-022): a fragmented MP4 that keeps
    /// what was recorded if the app dies. When it stops it becomes the MP4 at `path`.
    pub recovery: Option<PathBuf>,
    /// The microphone, when it records too (VID-039).
    pub microphone: Option<Microphone>,
}

/// The microphone of a recording (VID-039).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Microphone {
    /// Its id ([`inputs`]), or the system's default input.
    pub device: Option<String>,
    /// The automatic level (on by default).
    pub auto_level: bool,
}

/// How often a recording into a fragmented file gets a key frame while something
/// new waits in its open fragment (VID-022): what can be lost if the app dies.
pub(crate) const KEY_EVERY: f64 = 3.0;

/// The file next to a recording's fragmented file with the time recorded so far
/// (VID-022): a recording cut short keeps its length.
pub fn time_file(fragmented: &std::path::Path) -> PathBuf {
    let mut s = fragmented.as_os_str().to_owned();
    s.push(".time");
    PathBuf::from(s)
}

/// The time noted next to a fragmented recording, in seconds.
pub(crate) fn noted_time(fragmented: &std::path::Path) -> Option<f64> {
    std::fs::read_to_string(time_file(fragmented))
        .ok()?
        .trim()
        .parse()
        .ok()
}

/// Recording into a fragmented file (VID-022): when a key frame is due and when to
/// note the time recorded.
#[derive(Debug, Default)]
pub(crate) struct Keeper {
    last_key: f64,
    pending: bool,
    last_note: Option<Instant>,
}

impl Keeper {
    /// A key frame is due at `now` (seconds recorded): something was written since
    /// the last one, `KEY_EVERY` seconds ago or more.
    pub fn key_due(&self, now: f64) -> bool {
        self.pending && now - self.last_key >= KEY_EVERY
    }

    /// A frame was written at `t`: a key frame or not, with something `new` on it
    /// or the same picture again.
    pub fn wrote(&mut self, t: f64, key: bool, new: bool) {
        if key {
            self.last_key = t;
            self.pending = false;
        } else if new {
            self.pending = true;
        }
    }

    /// Notes `recorded` next to `fragmented`, once a second.
    pub fn note(&mut self, fragmented: &std::path::Path, recorded: f64) {
        if self
            .last_note
            .is_none_or(|t| t.elapsed() >= Duration::from_secs(1))
        {
            self.last_note = Some(Instant::now());
            let _ = std::fs::write(time_file(fragmented), format!("{recorded:.3}"));
        }
    }
}

/// Where a recording is (the states of `03-video.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Recording,
    Paused,
    Finalizing,
    Done,
}

/// How a finished recording came out.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Summary {
    pub path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    /// Seconds recorded (pauses left out).
    pub duration: f64,
    pub frames: u64,
    pub bytes: u64,
    /// The encoder: `hardware` or `software`.
    pub encoder: &'static str,
    /// Every frame was black (VID-029): the screen could not be read.
    pub all_black: bool,
    /// The video has the system's sound (VID-040).
    pub audio: bool,
    /// It stopped by itself because the disk was almost full (VID-023).
    pub low_disk: bool,
    /// It stopped because the window it followed closed (VID-005).
    pub window_closed: bool,
    /// The microphone is in it (VID-039).
    pub microphone: bool,
    /// The system's sound is in it: an output was there to listen to (VID-040).
    pub system_audio: bool,
    pub cancelled: bool,
    /// When the first frame was written (VID-030: the start's delay).
    #[serde(skip)]
    pub first_frame_at: Option<Instant>,
    /// Frames lost: moments a change on screen waited past its frame's turn
    /// (VID-030: at most 1 % of the frames).
    pub late_frames: u64,
}

/// Keeps a sound in step with the picture (VID-030). `written` is where the sound
/// written so far ends and `target` where the next `frames` should start (seconds):
/// behind by more than 20 ms, silence fills the gap; ahead by more than 30 ms (the
/// sound card's clock runs fast), the start of the new sound is dropped down to
/// 10 ms ahead. Returns (silence frames, frames dropped).
pub(crate) fn audio_fit(written: f64, target: f64, rate: u32, frames: usize) -> (usize, usize) {
    let rate = rate as f64;
    if target - written > 0.02 {
        (((target - written) * rate) as usize, 0)
    } else if written - target > 0.03 {
        let skip = ((written - target - 0.01) * rate) as usize;
        (0, skip.min(frames))
    } else {
        (0, 0)
    }
}

/// What the recording thread and its owner share.
pub(crate) struct Shared {
    pub paused: AtomicBool,
    /// The cursor and keyboard effects now (VID-045 to VID-047).
    pub effects: Mutex<effects::Effects>,
    /// The pointer is drawn (VID-044); it can change while recording.
    pub cursor: AtomicBool,
    pub stop: AtomicBool,
    pub cancel: AtomicBool,
    pub state: AtomicU8,
    /// Time recorded so far, pauses left out.
    pub clock: Mutex<Clock>,
    /// The meters now (VID-011, VID-039, VID-040).
    pub levels: Mutex<mix::Levels>,
    /// The microphone wanted now: it can be switched on or off while recording, and to
    /// another device while paused (VID-011). Its number changes with it.
    pub microphone: Mutex<Option<Microphone>>,
    pub microphone_changed: std::sync::atomic::AtomicU32,
    /// The system's sound is kept (VID-011: switched off, the rest is silent).
    pub system_audio: AtomicBool,
    /// The camera (VID-031 to VID-036): where its frames come from, whether it shows
    /// and how; each can change while recording.
    pub camera_feed: Mutex<Option<camlook::Feed>>,
    pub camera_on: AtomicBool,
    pub camera_look: Mutex<camlook::Look>,
}

#[derive(Default)]
pub(crate) struct Clock {
    pub started: Option<Instant>,
    pub paused_at: Option<Instant>,
    pub paused_total: Duration,
}

impl Clock {
    pub fn recorded(&self, now: Instant) -> Duration {
        let Some(start) = self.started else {
            return Duration::ZERO;
        };
        let until = self.paused_at.unwrap_or(now);
        until
            .saturating_duration_since(start)
            .saturating_sub(self.paused_total)
    }
}

impl Shared {
    fn state(&self) -> State {
        match self.state.load(Ordering::Relaxed) {
            0 => State::Recording,
            1 => State::Paused,
            2 => State::Finalizing,
            _ => State::Done,
        }
    }
    pub(crate) fn set_state(&self, s: State) {
        self.state.store(
            match s {
                State::Recording => 0,
                State::Paused => 1,
                State::Finalizing => 2,
                State::Done => 3,
            },
            Ordering::Relaxed,
        );
    }
}

/// A recording under way.
pub struct Recorder {
    shared: Arc<Shared>,
    thread: Option<std::thread::JoinHandle<Result<Summary, String>>>,
    pub width: u32,
    pub height: u32,
    pub fps: u32,
    /// The video has a sound track: the microphone and the system's sound can be
    /// switched on and off while it records (VID-011).
    pub audio: bool,
}

impl Recorder {
    /// Starts recording; returns once the first frame is on its way (or why not).
    pub fn start(options: Options) -> Result<Recorder, String> {
        if options.area.width < MIN_AREA || options.area.height < MIN_AREA {
            return Err(format!("the area is smaller than {MIN_AREA} × {MIN_AREA} px"));
        }
        let (width, height) = video_size(
            options.area.width,
            options.area.height,
            options.scale,
            options.max_size,
        );
        let fps = options.quality.fps();
        let shared = Arc::new(Shared {
            paused: AtomicBool::new(false),
            effects: Mutex::new(options.effects.clone()),
            cursor: AtomicBool::new(options.cursor),
            stop: AtomicBool::new(false),
            cancel: AtomicBool::new(false),
            state: AtomicU8::new(0),
            clock: Mutex::new(Clock::default()),
            levels: Mutex::new(mix::Levels::default()),
            microphone: Mutex::new(options.microphone.clone()),
            microphone_changed: std::sync::atomic::AtomicU32::new(0),
            system_audio: AtomicBool::new(options.system_audio),
            camera_feed: Mutex::new(None),
            camera_on: AtomicBool::new(false),
            camera_look: Mutex::new(camlook::Look::default()),
        });
        // A sound track when some sound was asked for, so either can be switched on later.
        let audio = options.system_audio || options.microphone.is_some();
        let (ready_tx, ready_rx) = std::sync::mpsc::channel::<Result<(), String>>();
        let thread_shared = shared.clone();
        let thread = std::thread::Builder::new()
            .name("xs-record".into())
            .spawn(move || run(options, (width, height), thread_shared, ready_tx))
            .map_err(|e| e.to_string())?;
        match ready_rx.recv_timeout(Duration::from_secs(10)) {
            Ok(Ok(())) => Ok(Recorder {
                shared,
                thread: Some(thread),
                width,
                height,
                fps,
                audio,
            }),
            Ok(Err(e)) => {
                let _ = thread.join();
                Err(e)
            }
            Err(_) => {
                shared.cancel.store(true, Ordering::Relaxed);
                shared.stop.store(true, Ordering::Relaxed);
                Err("the recording did not start in time".into())
            }
        }
    }

    pub fn pause(&self) {
        if !self.shared.paused.swap(true, Ordering::Relaxed) {
            self.shared.clock.lock().unwrap().paused_at = Some(Instant::now());
            self.shared.set_state(State::Paused);
        }
    }

    pub fn resume(&self) {
        if self.shared.paused.swap(false, Ordering::Relaxed) {
            let mut c = self.shared.clock.lock().unwrap();
            if let Some(at) = c.paused_at.take() {
                c.paused_total += at.elapsed();
            }
            self.shared.set_state(State::Recording);
        }
    }

    pub fn state(&self) -> State {
        self.shared.state()
    }

    /// Shows or hides the pointer from now on (VID-044).
    pub fn set_cursor(&self, on: bool) {
        self.shared.cursor.store(on, Ordering::Relaxed);
    }

    pub fn cursor(&self) -> bool {
        self.shared.cursor.load(Ordering::Relaxed)
    }

    /// The cursor and keyboard effects from now on (VID-045 to VID-047).
    pub fn set_effects(&self, effects: effects::Effects) {
        *self.shared.effects.lock().unwrap() = effects;
    }

    /// The cursor and keyboard effects now.
    pub fn effects(&self) -> effects::Effects {
        self.shared.effects.lock().unwrap().clone()
    }

    /// Time recorded so far, pauses left out.
    pub fn recorded(&self) -> Duration {
        self.shared.clock.lock().unwrap().recorded(Instant::now())
    }

    /// The meters now (VID-011, VID-039, VID-040).
    pub fn levels(&self) -> mix::Levels {
        *self.shared.levels.lock().unwrap()
    }

    /// The microphone from now on (VID-011): on, off, or another device (only while
    /// paused, which the caller checks).
    pub fn set_microphone(&self, microphone: Option<Microphone>) {
        let mut m = self.shared.microphone.lock().unwrap();
        if *m != microphone {
            *m = microphone;
            self.shared.microphone_changed.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn microphone(&self) -> Option<Microphone> {
        self.shared.microphone.lock().unwrap().clone()
    }

    /// The camera's frames (VID-031): from the camera the preview reads, or none.
    pub fn set_camera(&self, feed: Option<camlook::Feed>) {
        *self.shared.camera_feed.lock().unwrap() = feed;
    }

    /// The camera shows from now on, or not (VID-011, VID-031).
    pub fn set_camera_on(&self, on: bool) {
        self.shared.camera_on.store(on, Ordering::Relaxed);
    }

    pub fn camera_on(&self) -> bool {
        self.shared.camera_on.load(Ordering::Relaxed)
    }

    /// Where and how the camera shows from now on (VID-032 to VID-036).
    pub fn set_camera_look(&self, look: camlook::Look) {
        *self.shared.camera_look.lock().unwrap() = look;
    }

    /// The system's sound from now on (VID-011).
    pub fn set_system_audio(&self, on: bool) {
        self.shared.system_audio.store(on, Ordering::Relaxed);
    }

    pub fn system_audio(&self) -> bool {
        self.shared.system_audio.load(Ordering::Relaxed)
    }

    /// The thread ended by itself (the limit, or an error).
    pub fn finished(&self) -> bool {
        self.thread.as_ref().is_none_or(|t| t.is_finished())
    }

    /// Stops and waits for the file.
    pub fn stop(mut self) -> Result<Summary, String> {
        self.resume();
        self.shared.stop.store(true, Ordering::Relaxed);
        self.join()
    }

    /// Stops without keeping anything (VID-014).
    pub fn cancel(mut self) -> Result<Summary, String> {
        self.shared.cancel.store(true, Ordering::Relaxed);
        self.shared.stop.store(true, Ordering::Relaxed);
        self.join()
    }

    fn join(&mut self) -> Result<Summary, String> {
        match self.thread.take() {
            Some(t) => t
                .join()
                .map_err(|_| "the recording stopped unexpectedly".to_string())?,
            None => Err("already stopped".into()),
        }
    }
}

impl Drop for Recorder {
    fn drop(&mut self) {
        if let Some(t) = self.thread.take() {
            self.shared.cancel.store(true, Ordering::Relaxed);
            self.shared.stop.store(true, Ordering::Relaxed);
            let _ = t.join();
        }
    }
}

#[cfg(windows)]
fn run(
    options: Options,
    size: (u32, u32),
    shared: Arc<Shared>,
    ready: std::sync::mpsc::Sender<Result<(), String>>,
) -> Result<Summary, String> {
    win::record(options, size, shared, ready)
}

#[cfg(not(windows))]
fn run(
    _options: Options,
    _size: (u32, u32),
    _shared: Arc<Shared>,
    ready: std::sync::mpsc::Sender<Result<(), String>>,
) -> Result<Summary, String> {
    // macOS: ScreenCaptureKit and VideoToolbox arrive with the Mac work.
    let why = "video recording is not available on this platform yet".to_string();
    let _ = ready.send(Err(why.clone()));
    Err(why)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn key_frames_come_while_something_waits() {
        let mut k = Keeper::default();
        // The first frame is a key frame.
        k.wrote(0.0, true, true);
        k.wrote(0.5, false, false);
        assert!(!k.key_due(10.0), "nothing new: no key frame, however long");
        k.wrote(1.0, false, true);
        assert!(!k.key_due(2.9));
        assert!(k.key_due(3.0));
        k.wrote(3.0, true, false);
        assert!(!k.key_due(7.0));
        // The time is noted next to the file, once a second.
        let dir = std::env::temp_dir().join(format!("xs-keeper-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("rec.mp4");
        k.note(&file, 4.25);
        k.note(&file, 4.5);
        assert_eq!(noted_time(&file), Some(4.25));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn the_sound_stays_in_step() {
        // In step: as it comes.
        assert_eq!(audio_fit(10.0, 10.01, 48_000, 480), (0, 0));
        assert_eq!(audio_fit(10.02, 10.0, 48_000, 480), (0, 0));
        // Behind (nothing played for a while): silence up to the target.
        assert_eq!(audio_fit(10.0, 10.5, 48_000, 480), (24_000, 0));
        // Ahead by 50 ms (a fast clock): 40 ms of the new sound dropped.
        assert_eq!(audio_fit(10.05, 10.0, 48_000, 4_800), (0, 1_920));
        // Never more than what came.
        assert_eq!(audio_fit(11.0, 10.0, 48_000, 480), (0, 480));
        // A clock 100 ppm fast over 30 minutes stays within 30 ms.
        let (rate, packet) = (48_000u32, 480usize);
        let fast = rate as f64 * 1.0001;
        let mut written = 0u64;
        let mut worst: f64 = 0.0;
        for n in 0..(30 * 60 * 100) {
            let now = (n + 1) as f64 * packet as f64 / fast;
            let target = now - packet as f64 / rate as f64;
            let at = written as f64 / rate as f64;
            let (gap, skip) = audio_fit(at, target, rate, packet);
            written += (gap + packet - skip) as u64;
            worst = worst.max((written as f64 / rate as f64 - now).abs());
        }
        assert!(worst <= 0.031, "{worst}");
    }

    #[test]
    fn sizes_follow_the_setting_and_stay_even() {
        // 150 %: the logical size, even.
        assert_eq!(
            video_size(1501, 901, 1.5, MaxSize::ReduceHighDensity),
            (1000, 600)
        );
        assert_eq!(
            video_size(1000, 600, 1.0, MaxSize::ReduceHighDensity),
            (1000, 600)
        );
        assert_eq!(video_size(1501, 901, 1.5, MaxSize::Original), (1500, 900));
        assert_eq!(video_size(2560, 1440, 1.0, MaxSize::Fit1080), (1920, 1080));
        assert_eq!(video_size(1440, 2560, 1.0, MaxSize::Fit1080), (1080, 1920));
        // Over the limit: reduced to fit.
        assert_eq!(video_size(5120, 2880, 1.0, MaxSize::Original), (3840, 2160));
        assert_eq!(video_size(5120, 2160, 1.0, MaxSize::Original), (3840, 1620));
    }

    #[test]
    fn quality_sets_fps_and_bitrate() {
        assert_eq!(Quality::High.fps(), 30);
        assert_eq!(Quality::VeryHigh.fps(), 60);
        let hd = Quality::High.bitrate(1920, 1080);
        assert!((4_000_000..6_000_000).contains(&hd), "{hd}");
        assert!(Quality::Low.bitrate(64, 64) >= 500_000);
    }

    #[test]
    fn the_busiest_second_counts_frames() {
        assert_eq!(busiest_second(&[]), 0);
        assert_eq!(busiest_second(&[0.0, 0.5, 0.9, 2.0]), 3);
        let sixty: Vec<f64> = (0..120).map(|i| i as f64 / 60.0).collect();
        assert_eq!(busiest_second(&sixty), 60);
    }

    #[test]
    fn it_stops_under_a_gigabyte() {
        assert!(low_disk(500 << 20));
        assert!(!low_disk(3 << 30));
    }

    #[test]
    fn the_clock_leaves_pauses_out() {
        let t0 = Instant::now();
        let c = Clock {
            started: Some(t0),
            paused_at: Some(t0 + Duration::from_secs(5)),
            paused_total: Duration::from_secs(2),
        };
        assert_eq!(c.recorded(t0 + Duration::from_secs(9)), Duration::from_secs(3));
    }
}
