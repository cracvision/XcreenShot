//! The sound of a recording (VID-039, VID-040): the system's sound and the microphone
//! put each at the moment it was heard, added together and written in order. And
//! the microphone's automatic level (VID-039): quiet voices up, loud ones down,
//! never clipping.

use std::collections::VecDeque;
use std::time::Instant;

/// One source's meter now (VID-011, VID-039, VID-040): its level, 0 to 1; whether it
/// reached the top; and how long nothing has come from it (VID-019).
#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Level {
    pub level: f32,
    pub clip: bool,
    pub quiet_seconds: f32,
}

/// The meters of a recording, or of Ready before it.
#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Levels {
    /// The microphone, when it is on.
    pub microphone: Option<Level>,
    /// The system's sound, when it is on.
    pub system: Option<Level>,
    /// The microphone is muted in the system, or its volume is at zero (VID-019).
    pub microphone_muted: bool,
}

/// Below this (about -80 dBFS) nothing came: a muted or dead microphone gives zeros,
/// a working one at least its own hiss.
const NOTHING: f32 = 1e-4;

/// The loudest of `samples`, 0 to 1.
pub fn peak(samples: &[i16]) -> f32 {
    samples
        .iter()
        .map(|s| s.unsigned_abs())
        .max()
        .map_or(0.0, |p| p as f32 / 32768.0)
}

/// A level meter: it falls back slowly, so a short sound is seen; the top stays marked
/// for a second; and it counts how long nothing has come.
#[derive(Debug, Default)]
pub struct Gauge {
    level: f32,
    clip_until: Option<Instant>,
    quiet_since: Option<Instant>,
}

impl Gauge {
    /// What came now: `raw` is its peak as it came (for "nothing came"), `shown` the
    /// level that is recorded (after the automatic level).
    pub fn feed(&mut self, raw: f32, shown: f32, clipped: bool, now: Instant) {
        self.level = shown.max(self.level * 0.75);
        if clipped {
            self.clip_until = Some(now + std::time::Duration::from_secs(1));
        }
        if raw > NOTHING {
            self.quiet_since = None;
        } else if self.quiet_since.is_none() {
            self.quiet_since = Some(now);
        }
    }

    pub fn reading(&self, now: Instant) -> Level {
        Level {
            level: (self.level * 100.0).round() / 100.0,
            clip: self.clip_until.is_some_and(|t| now < t),
            quiet_seconds: self.quiet_since.map_or(0.0, |t| {
                (now.saturating_duration_since(t).as_secs_f32() * 10.0).round() / 10.0
            }),
        }
    }
}

/// One source of sound: where its next sample goes, in frames from the start.
#[derive(Debug, Default)]
pub struct Track {
    pub cursor: u64,
}

/// Sound waiting to be written: stereo frames from `base` on, as sums.
#[derive(Debug)]
pub struct Mixer {
    pub rate: u32,
    base: u64,
    frames: VecDeque<[i32; 2]>,
}

impl Mixer {
    pub fn new(rate: u32) -> Mixer {
        Mixer {
            rate,
            base: 0,
            frames: VecDeque::new(),
        }
    }

    /// Where the sound written so far ends, in frames.
    pub fn written(&self) -> u64 {
        self.base
    }

    /// Puts `samples` (interleaved stereo) of `track`, which should start at `target`
    /// seconds: silence fills a gap, a clock that runs fast loses a little of its
    /// start (`audio_fit`). Sound older than what was already written is dropped.
    pub fn place(&mut self, track: &mut Track, target: f64, samples: &[i16]) {
        let frames = samples.len() / 2;
        let (gap, skip) = crate::audio_fit(track.cursor as f64 / self.rate as f64, target, self.rate, frames);
        track.cursor += gap as u64;
        for (i, f) in samples.as_chunks::<2>().0.iter().enumerate().skip(skip) {
            let at = track.cursor + (i - skip) as u64;
            if at < self.base {
                continue;
            }
            let k = (at - self.base) as usize;
            while self.frames.len() <= k {
                self.frames.push_back([0, 0]);
            }
            self.frames[k][0] += f[0] as i32;
            self.frames[k][1] += f[1] as i32;
        }
        track.cursor += (frames - skip.min(frames)) as u64;
    }

    /// The frames before `upto` (silence where nothing was placed), ready to write at
    /// [`written`](Self::written): the sum, held within 16 bits.
    pub fn take(&mut self, upto: u64) -> Vec<i16> {
        let n = upto.saturating_sub(self.base) as usize;
        let mut out = Vec::with_capacity(n * 2);
        for _ in 0..n {
            let f = self.frames.pop_front().unwrap_or([0, 0]);
            out.push(f[0].clamp(-32768, 32767) as i16);
            out.push(f[1].clamp(-32768, 32767) as i16);
        }
        self.base += n as u64;
        out
    }
}

/// The microphone's automatic level (VID-039): it follows the voice's loudness in
/// short blocks and moves the gain slowly towards a steady level, faster down than
/// up; it never raises background noise and never clips.
#[derive(Debug)]
pub struct Agc {
    gain: f32,
    /// The loudest sample of the last block, 0 to 1, for the level meter.
    pub peak: f32,
    /// A sample reached the top in the last block (the meter's mark).
    pub clipped: bool,
}

/// The level a voice is brought to: about -20 dBFS.
const TARGET: f32 = 0.1;
/// At most +20 dB, at least -12 dB.
const MAX_GAIN: f32 = 10.0;
const MIN_GAIN: f32 = 0.25;
/// Below this (about -50 dBFS) it is not a voice: the gain does not rise.
const QUIET: f32 = 0.003;

impl Default for Agc {
    fn default() -> Self {
        Agc {
            gain: 1.0,
            peak: 0.0,
            clipped: false,
        }
    }
}

impl Agc {
    /// Levels `samples` (interleaved stereo) in place when `on`; measures them always.
    pub fn process(&mut self, samples: &mut [i16], rate: u32, on: bool) {
        let block = (rate as usize / 100).max(1) * 2; // 10 ms
        self.peak = 0.0;
        self.clipped = false;
        if !on {
            // As it came: only measured.
            for s in samples.iter() {
                let v = (*s as f32 / 32768.0).abs();
                self.peak = self.peak.max(v);
                self.clipped |= v >= 0.99;
            }
            return;
        }
        for chunk in samples.chunks_mut(block) {
            let rms = (chunk.iter().map(|s| (*s as f32 / 32768.0).powi(2)).sum::<f32>()
                / chunk.len().max(1) as f32)
                .sqrt();
            if on {
                let wanted = if rms > QUIET {
                    (TARGET / rms).clamp(MIN_GAIN, MAX_GAIN)
                } else {
                    self.gain.min(1.0)
                };
                // Down in about 50 ms, up in about 1 s.
                let speed = if wanted < self.gain { 0.2 } else { 0.01 };
                self.gain += (wanted - self.gain) * speed;
                // A sudden loud sound: down at once, so it never clips.
                let top = chunk
                    .iter()
                    .map(|s| (*s as f32 / 32768.0).abs())
                    .fold(0.0, f32::max);
                if top * self.gain > 0.9 {
                    self.gain = 0.9 / top.max(1e-6);
                }
            }
            let gain = if on { self.gain } else { 1.0 };
            for s in chunk.iter_mut() {
                let v = *s as f32 * gain / 32768.0;
                // A soft top instead of clipping.
                let v = if v.abs() > 0.9 {
                    v.signum() * (0.9 + (v.abs() - 0.9).min(0.4) * 0.25)
                } else {
                    v
                };
                let v = v.clamp(-1.0, 1.0);
                if v.abs() >= 0.99 {
                    self.clipped = true;
                }
                self.peak = self.peak.max(v.abs());
                *s = (v * 32767.0) as i16;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tone(frames: usize, amplitude: f32, rate: u32) -> Vec<i16> {
        (0..frames)
            .flat_map(|n| {
                let v = ((n as f32 / rate as f32 * 300.0 * std::f32::consts::TAU).sin() * amplitude * 32767.0)
                    as i16;
                [v, v]
            })
            .collect()
    }

    fn db(samples: &[i16]) -> f32 {
        let rms = (samples.iter().map(|s| (*s as f32 / 32768.0).powi(2)).sum::<f32>() / samples.len() as f32)
            .sqrt();
        20.0 * rms.log10()
    }

    #[test]
    fn two_sources_are_added_at_their_moment() {
        let mut m = Mixer::new(48_000);
        let (mut system, mut mic) = (Track::default(), Track::default());
        // The system plays 2 s from 0 s; the microphone hears 1 s from 0.5 s.
        m.place(&mut system, 0.0, &[1000i16; 192_000]);
        m.place(&mut mic, 0.5, &[500i16; 96_000]);
        let out = m.take(48_000 * 2);
        assert_eq!(out.len(), 96_000 * 2);
        assert_eq!(out[0], 1000);
        assert_eq!(out[24_000 * 2], 1500, "both at 0.5 s");
        assert_eq!(out[71_999 * 2], 1500);
        assert_eq!(out[72_000 * 2], 1000, "the microphone stopped at 1.5 s");
        assert_eq!(m.written(), 96_000);
        // Nothing placed: silence; and the sum never wraps.
        m.place(&mut system, 2.5, &[30_000i16; 2]);
        m.place(&mut mic, 2.5, &[30_000i16; 2]);
        let out = m.take(48_000 * 3);
        assert_eq!(out[0], 0);
        assert_eq!(out[24_000 * 2], 32_767);
    }

    #[test]
    fn the_automatic_level_evens_out_voices() {
        let rate = 48_000;
        let mut agc = Agc::default();
        // A quiet sentence (-36 dBFS) then a loud one (-12 dBFS), 2 s each.
        let mut quiet = tone(96_000, 0.016, rate);
        let mut loud = tone(96_000, 0.25, rate);
        agc.process(&mut quiet, rate, true);
        agc.process(&mut loud, rate, true);
        // Their second halves, once the level settled: within 6 dB.
        let (a, b) = (db(&quiet[96_000..]), db(&loud[96_000..]));
        assert!((a - b).abs() <= 6.0, "{a} vs {b}");
        assert!(!agc.clipped);
        // Off: as it came.
        let mut same = tone(4_800, 0.25, rate);
        let before = same.clone();
        Agc::default().process(&mut same, rate, false);
        assert_eq!(same, before);
    }

    #[test]
    fn the_meter_counts_how_long_nothing_came() {
        use std::time::Duration;
        let t0 = Instant::now();
        let mut g = Gauge::default();
        g.feed(0.3, 0.3, false, t0);
        assert_eq!(g.reading(t0).quiet_seconds, 0.0);
        assert!(g.reading(t0).level > 0.29);
        // Zeros from a muted microphone: counted from the first.
        for i in 1..=100 {
            g.feed(0.0, 0.0, false, t0 + Duration::from_millis(100 * i));
        }
        let r = g.reading(t0 + Duration::from_secs(10));
        assert!(r.quiet_seconds >= 9.8, "{r:?}");
        assert!(r.level < 0.01, "it fell back: {r:?}");
        // Hiss is something: the count starts again.
        g.feed(0.0005, 0.0005, true, t0 + Duration::from_secs(11));
        let r = g.reading(t0 + Duration::from_secs(11));
        assert_eq!(r.quiet_seconds, 0.0);
        assert!(r.clip);
        assert!(
            !g.reading(t0 + Duration::from_secs(13)).clip,
            "the mark goes after a second"
        );
        assert_eq!(peak(&[0, -16384, 100]), 0.5);
    }

    #[test]
    fn silence_is_not_raised() {
        let rate = 48_000;
        let mut agc = Agc::default();
        let mut hiss = tone(96_000, 0.001, rate);
        agc.process(&mut hiss, rate, true);
        assert!(db(&hiss) < -55.0, "{}", db(&hiss));
    }
}
