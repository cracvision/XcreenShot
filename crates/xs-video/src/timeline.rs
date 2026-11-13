//! A video's timeline in the editor (VID-058, VID-059, ADR 0008): the parts cut out,
//! in seconds of the file, and the video that is left. `app/src/editor/videoTimeline.ts`
//! is its copy for the player; both are tested with the same cases.

use serde::{Deserialize, Serialize};

/// Shorter than this is nothing.
const EPS: f64 = 1e-6;

/// A stretch of time, in seconds: from `start` to `end`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Span {
    pub start: f64,
    pub end: f64,
}

impl Span {
    pub fn new(start: f64, end: f64) -> Span {
        Span { start, end }
    }

    pub fn len(&self) -> f64 {
        (self.end - self.start).max(0.0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() < EPS
    }
}

/// The cuts inside the video, sorted, joined where they touch and without empty ones.
pub fn normalize(cuts: &[Span], duration: f64) -> Vec<Span> {
    let mut spans: Vec<Span> = cuts
        .iter()
        .map(|c| Span::new(c.start.max(0.0), c.end.min(duration)))
        .filter(|c| !c.is_empty())
        .collect();
    spans.sort_by(|a, b| a.start.total_cmp(&b.start));
    let mut out: Vec<Span> = Vec::new();
    for s in spans {
        match out.last_mut() {
            Some(last) if s.start <= last.end + EPS => last.end = last.end.max(s.end),
            _ => out.push(s),
        }
    }
    out
}

/// The parts of the file that are kept, in order.
pub fn kept(duration: f64, cuts: &[Span]) -> Vec<Span> {
    let mut out = Vec::new();
    let mut at = 0.0;
    for c in normalize(cuts, duration) {
        if c.start - at > EPS {
            out.push(Span::new(at, c.start));
        }
        at = c.end;
    }
    if duration - at > EPS {
        out.push(Span::new(at, duration));
    }
    out
}

/// How long the video is once cut.
pub fn length(kept: &[Span]) -> f64 {
    kept.iter().map(Span::len).sum()
}

/// A time of the cut video, in the file.
pub fn to_source(kept: &[Span], t: f64) -> f64 {
    let mut left = t.max(0.0);
    for k in kept {
        if left <= k.len() + EPS {
            return (k.start + left).min(k.end);
        }
        left -= k.len();
    }
    kept.last().map(|k| k.end).unwrap_or(0.0)
}

/// A time of the file, in the cut video; a time that was cut goes to the place of the
/// cut.
pub fn to_edited(kept: &[Span], t: f64) -> f64 {
    let mut before = 0.0;
    for k in kept {
        if t < k.start {
            return before;
        }
        if t <= k.end {
            return before + (t - k.start);
        }
        before += k.len();
    }
    before
}

/// The parts of the file that `start..end` of the cut video shows.
pub fn source_spans(kept: &[Span], start: f64, end: f64) -> Vec<Span> {
    let (start, end) = (start.max(0.0), end.min(length(kept)));
    let mut out = Vec::new();
    let mut at = 0.0;
    for k in kept {
        let (a, b) = (at, at + k.len());
        let (s, e) = (start.max(a), end.min(b));
        if e - s > EPS {
            out.push(Span::new(k.start + (s - a), k.start + (e - a)));
        }
        at = b;
    }
    out
}

/// The cuts after cutting `start..end` of the cut video (VID-059), or `None` when that
/// would leave less than `frame` seconds (one frame).
pub fn cut(duration: f64, cuts: &[Span], start: f64, end: f64, frame: f64) -> Option<Vec<Span>> {
    let k = kept(duration, cuts);
    let mut all: Vec<Span> = normalize(cuts, duration);
    all.extend(source_spans(&k, start, end));
    let joined = normalize(&all, duration);
    let left = length(&kept(duration, &joined));
    (left > EPS && left + EPS >= frame).then_some(joined)
}

/// The cut points shown on the timeline (VID-059): where each cut was, in the cut video.
pub fn marks(duration: f64, cuts: &[Span]) -> Vec<f64> {
    let k = kept(duration, cuts);
    normalize(cuts, duration)
        .iter()
        .map(|c| to_edited(&k, c.start))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-9
    }

    #[test]
    fn nothing_cut_keeps_everything() {
        assert_eq!(kept(60.0, &[]), vec![Span::new(0.0, 60.0)]);
        assert!(close(to_source(&kept(60.0, &[]), 12.5), 12.5));
    }

    /// VID-059: cutting 0:10–0:20 of 60 s leaves 50 s, and 0:10 shows what was at 0:20.
    #[test]
    fn a_cut_joins_both_sides() {
        let cuts = cut(60.0, &[], 10.0, 20.0, 0.0).unwrap();
        assert_eq!(cuts, vec![Span::new(10.0, 20.0)]);
        let k = kept(60.0, &cuts);
        assert!(close(length(&k), 50.0));
        assert!(close(to_source(&k, 9.5), 9.5));
        assert!(close(to_source(&k, 10.5), 20.5));
        assert!(close(to_edited(&k, 25.0), 15.0));
        assert!(close(to_edited(&k, 15.0), 10.0), "a cut time goes to the cut");
        assert_eq!(marks(60.0, &cuts), vec![10.0]);
    }

    /// A second cut is chosen on the cut video: it maps to the file.
    #[test]
    fn cuts_on_the_cut_video() {
        let first = cut(60.0, &[], 10.0, 20.0, 0.0).unwrap();
        // 5..15 of the cut video: 5..10 and 20..25 of the file.
        let second = cut(60.0, &first, 5.0, 15.0, 0.0).unwrap();
        assert_eq!(second, vec![Span::new(5.0, 25.0)]);
        let k = kept(60.0, &second);
        assert!(close(length(&k), 40.0));
        assert_eq!(
            source_spans(&kept(60.0, &first), 5.0, 15.0),
            vec![Span::new(5.0, 10.0), Span::new(20.0, 25.0)]
        );
    }

    /// The start and the end are cut the same way (trimming).
    #[test]
    fn trimming_both_ends() {
        let cuts = cut(30.0, &[], 0.0, 5.0, 0.0).unwrap();
        let cuts = cut(30.0, &cuts, 20.0, 25.0, 0.0).unwrap();
        assert_eq!(kept(30.0, &cuts), vec![Span::new(5.0, 25.0)]);
    }

    /// Edge case: the whole video cannot be cut.
    #[test]
    fn everything_cannot_be_cut() {
        assert_eq!(cut(30.0, &[], 0.0, 30.0, 0.0), None);
        let cuts = cut(30.0, &[], 0.0, 10.0, 0.0).unwrap();
        assert_eq!(cut(30.0, &cuts, 0.0, 20.0, 0.0), None);
        // Less than a frame left (a 30 fps video of 30.01 s) is nothing either.
        assert_eq!(cut(30.01, &[], 0.0, 30.0, 1.0 / 30.0), None);
        assert!(cut(30.1, &[], 0.0, 30.0, 1.0 / 30.0).is_some());
    }

    #[test]
    fn overlapping_and_outside_cuts_are_tidied() {
        let n = normalize(
            &[
                Span::new(20.0, 25.0),
                Span::new(-5.0, 2.0),
                Span::new(24.0, 28.0),
                Span::new(40.0, 50.0),
                Span::new(3.0, 3.0),
            ],
            30.0,
        );
        assert_eq!(n, vec![Span::new(0.0, 2.0), Span::new(20.0, 28.0)]);
    }
}
