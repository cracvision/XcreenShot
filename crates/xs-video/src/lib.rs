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
#[cfg(windows)]
mod win;
/// What a video file holds: width, height, frames and the last frame's time (s).
#[cfg(windows)]
pub use win::{audio_info, first_frame, probe, size};

/// A video's size (not available on this platform yet).
#[cfg(not(windows))]
pub fn size(_path: &std::path::Path) -> Result<(u32, u32), String> {
    Err("not available on this platform yet".into())
}

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
    pub path: PathBuf,
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
    pub cancelled: bool,
}

/// What the recording thread and its owner share.
pub(crate) struct Shared {
    pub paused: AtomicBool,
    /// The pointer is drawn (VID-044); it can change while recording.
    pub cursor: AtomicBool,
    pub stop: AtomicBool,
    pub cancel: AtomicBool,
    pub state: AtomicU8,
    /// Time recorded so far, pauses left out.
    pub clock: Mutex<Clock>,
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
            cursor: AtomicBool::new(options.cursor),
            stop: AtomicBool::new(false),
            cancel: AtomicBool::new(false),
            state: AtomicU8::new(0),
            clock: Mutex::new(Clock::default()),
        });
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

    /// Time recorded so far, pauses left out.
    pub fn recorded(&self) -> Duration {
        self.shared.clock.lock().unwrap().recorded(Instant::now())
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
