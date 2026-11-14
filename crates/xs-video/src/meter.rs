//! The sound meters before recording (VID-006, VID-011: the meters work in Ready too,
//! to try the sound): the microphone and the system's sound are measured, never kept
//! (APP-054). It stops by itself when nobody asks for the levels for a few seconds.

use crate::mix::{peak, Agc, Gauge, Levels};
use crate::Microphone;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// How long it keeps measuring after the last time the levels were asked for.
const LINGER: Duration = Duration::from_secs(3);
/// The rate the microphone is measured at.
const RATE: u32 = 48_000;

struct Shared {
    stop: AtomicBool,
    asked: Mutex<Instant>,
    levels: Mutex<Levels>,
    ended: AtomicBool,
}

/// Measures the microphone and the system's sound while Ready.
pub struct Meter {
    shared: Arc<Shared>,
    /// What it measures.
    pub microphone: Option<Microphone>,
    pub system_audio: bool,
}

impl Meter {
    pub fn start(microphone: Option<Microphone>, system_audio: bool) -> Meter {
        let shared = Arc::new(Shared {
            stop: AtomicBool::new(false),
            asked: Mutex::new(Instant::now()),
            levels: Mutex::new(Levels::default()),
            ended: AtomicBool::new(false),
        });
        let s = shared.clone();
        let mic = microphone.clone();
        let spawned = std::thread::Builder::new()
            .name("xs-meter".into())
            .spawn(move || {
                measure(&s, mic, system_audio);
                s.ended.store(true, Ordering::Relaxed);
            });
        if spawned.is_err() {
            shared.ended.store(true, Ordering::Relaxed);
        }
        Meter {
            shared,
            microphone,
            system_audio,
        }
    }

    /// The levels now; asking keeps it measuring.
    pub fn levels(&self) -> Levels {
        *self.shared.asked.lock().unwrap() = Instant::now();
        *self.shared.levels.lock().unwrap()
    }

    /// It is still measuring (it stops by itself when nobody asks).
    pub fn running(&self) -> bool {
        !self.shared.ended.load(Ordering::Relaxed)
    }
}

impl Drop for Meter {
    fn drop(&mut self) {
        self.shared.stop.store(true, Ordering::Relaxed);
    }
}

fn measure(s: &Shared, microphone: Option<Microphone>, system_audio: bool) {
    // SAFETY: COM for this thread's audio calls.
    unsafe {
        let _ = windows::Win32::System::Com::CoInitializeEx(
            None,
            windows::Win32::System::Com::COINIT_MULTITHREADED,
        );
    }
    let device = microphone.as_ref().and_then(|m| m.device.clone());
    let auto_level = microphone.as_ref().is_some_and(|m| m.auto_level);
    let (mut mic, mut loopback) = (None, None);
    let (mut mic_tried, mut system_tried, mut mute_checked): (
        Option<Instant>,
        Option<Instant>,
        Option<Instant>,
    ) = (None, None, None);
    let mut agc = Agc::default();
    let (mut mic_gauge, mut system_gauge) = (Gauge::default(), Gauge::default());
    let mut muted = false;
    let every = Duration::from_secs(1);
    while !s.stop.load(Ordering::Relaxed) && s.asked.lock().unwrap().elapsed() < LINGER {
        let now = Instant::now();
        // Opened, and opened again when it comes back (VID-043).
        if microphone.is_some() && mic.is_none() && mic_tried.is_none_or(|t| t.elapsed() > every) {
            mic_tried = Some(now);
            mic = crate::audio::microphone(device.as_deref(), RATE).ok();
        }
        if system_audio && loopback.is_none() && system_tried.is_none_or(|t| t.elapsed() > every) {
            system_tried = Some(now);
            loopback = crate::audio::Loopback::open().ok();
        }
        if microphone.is_some() && mute_checked.is_none_or(|t| t.elapsed() > every) {
            mute_checked = Some(now);
            muted = crate::audio::input_muted(device.as_deref()).unwrap_or(false);
        }
        if let Some(m) = &mic {
            match m.read() {
                Ok(mut samples) => {
                    let raw = peak(&samples);
                    agc.process(&mut samples, RATE, auto_level);
                    mic_gauge.feed(raw, agc.peak, agc.clipped, now);
                }
                Err(e) if crate::audio::device_gone(&e) => mic = None,
                Err(_) => {}
            }
        } else {
            mic_gauge.feed(0.0, 0.0, false, now);
        }
        if let Some(l) = &loopback {
            match l.read() {
                Ok(samples) => {
                    let p = peak(&samples);
                    system_gauge.feed(p, p, p >= 0.99, now);
                }
                Err(e) if crate::audio::device_gone(&e) => loopback = None,
                Err(_) => {}
            }
        }
        *s.levels.lock().unwrap() = Levels {
            microphone: microphone.is_some().then(|| mic_gauge.reading(now)),
            system: system_audio.then(|| system_gauge.reading(now)),
            microphone_muted: microphone.is_some() && muted,
        };
        std::thread::sleep(Duration::from_millis(50));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The meter measures the default microphone, and stops by itself when nobody asks
    /// (by hand: it listens to the microphone, nothing is kept).
    #[test]
    #[ignore]
    fn a_meter_measures_and_stops_by_itself() {
        let m = Meter::start(
            Some(Microphone {
                device: None,
                auto_level: true,
            }),
            true,
        );
        for _ in 0..10 {
            std::thread::sleep(Duration::from_millis(300));
            println!("{:?}", m.levels());
        }
        assert!(m.running());
        std::thread::sleep(LINGER + Duration::from_millis(500));
        assert!(!m.running(), "it stopped when nobody asked");
    }
}
