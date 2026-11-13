//! Joining the frames of a scrolling capture (CAP-036).
//!
//! Frames of the same area arrive one after another while its content scrolls, by hand
//! (CAP-034) or by XcreenShot (CAP-035). Each new frame is matched against the one before
//! it to find how far the content moved, and its moving part is painted onto a growing
//! canvas at that place. Fixed elements are found on the way: rows (or columns) that stay
//! the same at the same place in two frames while the rest moves.
//!
//! - Headers and left bars come once, from the frame that reached furthest up (or left);
//!   footers and right bars from the one that reached furthest down (or right).
//! - Fixed side bars and floating buttons are painted with every frame; later frames cover
//!   the earlier ones, so a floating button stays where the last view had it.
//! - Matching compares hashes of whole lines, counting only lines that differ from their
//!   neighbor (blank stretches match anywhere); when no shift matches well, it compares
//!   coarse gray profiles of the lines, which tolerate the small differences of content
//!   drawn again while it scrolls. A join found that way is "doubtful".
//!
//! Limits (CAP-036): 60 000 px on the longest side or 250 megapixels; past them the
//! stitcher stops taking frames and keeps what it has.

use image::RgbaImage;

/// Which way the content may move (CAP-034, CAP-035).
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Vertical,
    Horizontal,
    Both,
}

/// Longest side of a result (CAP-036).
pub const MAX_SIDE: u32 = 60_000;
/// Largest result, in pixels (CAP-036).
pub const MAX_PIXELS: u64 = 250_000_000;

/// Fewest lines two frames must share to be joined.
const MIN_OVERLAP: usize = 12;
/// A join whose exact matching scores at least this is sure.
const GOOD_SCORE: f32 = 0.85;
/// From this score up, an exact join is taken (doubtful below GOOD_SCORE unless the gray
/// profiles agree); below it, the gray profiles decide.
const MIN_SCORE: f32 = 0.6;
/// Gray levels (0-255) two profiles may differ on average and still be the same content.
const PROFILE_TOLERANCE: f32 = 3.0;
/// Bins of a line's gray profile.
const PROFILE_BINS: usize = 48;

/// What happened to a frame given to [`Stitcher::push`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pushed {
    /// The first frame: the result starts with it.
    First,
    /// The content did not move: nothing to add.
    Same,
    /// Joined at this shift (content moved by `dx`, `dy` px; positive is down or right).
    Joined { dx: i32, dy: i32, doubtful: bool },
    /// No overlap with the previous frame was found (the content jumped too far, or
    /// changed): the frame was left out.
    Lost,
    /// The result reached its largest size (CAP-036): the frame was left out and so will
    /// every later one.
    Full,
}

/// The shift between two frames and what stayed fixed in them.
#[derive(Debug, Clone, Copy, PartialEq)]
struct Shift {
    dx: i32,
    dy: i32,
    /// Lines that stayed at the start and at the end (rows for a vertical shift, columns
    /// for a horizontal one).
    lead: usize,
    trail: usize,
    score: f32,
    doubtful: bool,
}

/// A frame's fixed start or end, kept to paint at the edge of the result.
struct Edge {
    /// Where it goes on the result (content coordinates).
    x: i64,
    y: i64,
    image: RgbaImage,
}

/// Joins the frames of one scrolling capture.
pub struct Stitcher {
    direction: Direction,
    width: u32,
    height: u32,
    prev: Option<RgbaImage>,
    /// Where the previous frame sits, in content coordinates.
    at: (i64, i64),
    canvas: Canvas,
    /// The fixed start (header or left bar) of the frame furthest up or left, and the
    /// fixed end (footer or right bar) of the one furthest down or right.
    /// With both directions there is one for each column (or row) of frames.
    heads: Vec<Keyed>,
    tails: Vec<Keyed>,
    /// The first frame waits until the first join tells what in it is fixed.
    pending_first: bool,
    joins: usize,
    doubtful: usize,
    full: bool,
}

impl Stitcher {
    pub fn new(direction: Direction) -> Self {
        Self {
            direction,
            width: 0,
            height: 0,
            prev: None,
            at: (0, 0),
            canvas: Canvas::default(),
            heads: Vec::new(),
            tails: Vec::new(),
            pending_first: false,
            joins: 0,
            doubtful: 0,
            full: false,
        }
    }

    /// The size of one frame.
    pub fn frame_size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Where the last frame joined sits, from the first one (pixels; positive: down or
    /// right).
    pub fn position(&self) -> (i64, i64) {
        self.at
    }

    /// How many joins were made, and how many of them are doubtful.
    pub fn joins(&self) -> (usize, usize) {
        (self.joins, self.doubtful)
    }

    /// Size of the result so far, in pixels.
    pub fn size(&self) -> (u32, u32) {
        if self.pending_first || self.joins == 0 {
            return (self.width, self.height);
        }
        let (x0, y0, x1, y1) = self.bounds();
        ((x1 - x0) as u32, (y1 - y0) as u32)
    }

    /// Whether the result reached its largest size.
    pub fn is_full(&self) -> bool {
        self.full
    }

    /// Adds the next frame. Frames must all have the size of the first one.
    pub fn push(&mut self, frame: RgbaImage) -> Pushed {
        if self.full {
            return Pushed::Full;
        }
        let Some(prev) = self.prev.as_ref() else {
            self.width = frame.width();
            self.height = frame.height();
            self.prev = Some(frame);
            self.pending_first = true;
            return Pushed::First;
        };
        if frame.dimensions() != (self.width, self.height) {
            return Pushed::Lost;
        }
        let Some(shift) = find_shift(prev, &frame, self.direction) else {
            return if prev.as_raw() == frame.as_raw() || nothing_moved(prev, &frame) {
                Pushed::Same
            } else {
                Pushed::Lost
            };
        };
        if shift.dx == 0 && shift.dy == 0 {
            return Pushed::Same;
        }
        let at = (self.at.0 + shift.dx as i64, self.at.1 + shift.dy as i64);
        // Would the result grow past its largest size?
        let (bx0, by0, bx1, by1) = self.bounds_with(at);
        let (w, h) = ((bx1 - bx0) as u64, (by1 - by0) as u64);
        if w.max(h) > MAX_SIDE as u64 || w * h > MAX_PIXELS {
            self.full = true;
            return Pushed::Full;
        }
        if self.pending_first {
            self.pending_first = false;
            let first = self.prev.take().expect("the first frame");
            self.paint_band(&first, self.at, shift, true);
            self.keep_edges(&first, self.at, shift);
            self.prev = Some(first);
        }
        self.paint_band(&frame, at, shift, false);
        self.keep_edges(&frame, at, shift);
        self.at = at;
        self.prev = Some(frame);
        self.joins += 1;
        if shift.doubtful {
            self.doubtful += 1;
        }
        Pushed::Joined {
            dx: shift.dx,
            dy: shift.dy,
            doubtful: shift.doubtful,
        }
    }

    /// The joined image: everything the frames covered, with the fixed start and end
    /// painted once at the edges.
    pub fn finish(mut self) -> Option<RgbaImage> {
        if self.pending_first || self.joins == 0 {
            return self.prev.take();
        }
        let edges = std::mem::take(&mut self.heads)
            .into_iter()
            .chain(std::mem::take(&mut self.tails));
        for keyed in edges {
            let edge = keyed.edge;
            self.canvas.paint(&edge.image, edge.x, edge.y, None);
        }
        let (x0, y0, x1, y1) = self.bounds();
        Some(self.canvas.crop(x0, y0, x1, y1))
    }

    /// A small copy of the result so far, for a live preview: at most `max_w` × `max_h`.
    pub fn preview(&self, max_w: u32, max_h: u32) -> Option<RgbaImage> {
        if self.pending_first || self.joins == 0 {
            let p = self.prev.as_ref()?;
            return Some(shrink(p, max_w, max_h));
        }
        let (x0, y0, x1, y1) = self.bounds();
        Some(shrink(&self.canvas.crop(x0, y0, x1, y1), max_w, max_h))
    }

    fn bounds(&self) -> (i64, i64, i64, i64) {
        let mut b = self.canvas.painted;
        for e in self.heads.iter().chain(&self.tails).map(|k| &k.edge) {
            let (x1, y1) = (e.x + e.image.width() as i64, e.y + e.image.height() as i64);
            b = (b.0.min(e.x), b.1.min(e.y), b.2.max(x1), b.3.max(y1));
        }
        b
    }

    fn bounds_with(&self, at: (i64, i64)) -> (i64, i64, i64, i64) {
        let (x0, y0, x1, y1) = if self.joins == 0 {
            (
                self.at.0,
                self.at.1,
                self.at.0 + self.width as i64,
                self.at.1 + self.height as i64,
            )
        } else {
            self.bounds()
        };
        (
            x0.min(at.0),
            y0.min(at.1),
            x1.max(at.0 + self.width as i64),
            y1.max(at.1 + self.height as i64),
        )
    }

    /// Paints the part of `frame` that moves (the first frame keeps its fixed start: it
    /// is where the result begins).
    fn paint_band(&mut self, frame: &RgbaImage, at: (i64, i64), shift: Shift, first: bool) {
        let (w, h) = (self.width as usize, self.height as usize);
        let vertical = shift.dy != 0;
        let lead = if first { 0 } else { shift.lead };
        let rect = if vertical {
            (0, lead, w, h - shift.trail)
        } else {
            (lead, 0, w - shift.trail, h)
        };
        if rect.0 >= rect.2 || rect.1 >= rect.3 {
            return;
        }
        self.canvas.paint(frame, at.0, at.1, Some(rect));
    }

    /// Keeps the fixed start of the frame furthest up (or left) and the fixed end of the
    /// one furthest down (or right), for each column (or row) of frames.
    fn keep_edges(&mut self, frame: &RgbaImage, at: (i64, i64), shift: Shift) {
        let (w, h) = (self.width, self.height);
        let vertical = shift.dy != 0;
        let along = if vertical { at.1 } else { at.0 };
        let key = (vertical, if vertical { at.0 } else { at.1 });
        if shift.lead > 0 {
            let image = if vertical {
                image::imageops::crop_imm(frame, 0, 0, w, shift.lead as u32).to_image()
            } else {
                image::imageops::crop_imm(frame, 0, 0, shift.lead as u32, h).to_image()
            };
            let edge = Edge {
                x: at.0,
                y: at.1,
                image,
            };
            keep(
                &mut self.heads,
                key,
                edge,
                |e| -(if vertical { e.y } else { e.x }),
                -along,
            );
        }
        if shift.trail > 0 {
            let end = along + if vertical { h as i64 } else { w as i64 };
            let t = shift.trail as u32;
            let (image, x, y) = if vertical {
                let img = image::imageops::crop_imm(frame, 0, h - t, w, t).to_image();
                (img, at.0, at.1 + (h - t) as i64)
            } else {
                let img = image::imageops::crop_imm(frame, w - t, 0, t, h).to_image();
                (img, at.0 + (w - t) as i64, at.1)
            };
            let end_of = move |e: &Edge| {
                if vertical {
                    e.y + e.image.height() as i64
                } else {
                    e.x + e.image.width() as i64
                }
            };
            keep(&mut self.tails, key, Edge { x, y, image }, end_of, end);
        }
    }
}

/// A fixed start or end, for one column (vertical moves, keyed by x) or row of frames.
struct Keyed {
    key: (bool, i64),
    edge: Edge,
}

/// Keeps `edge` for `key` if it reaches further (`reach`, larger is further) than the one
/// kept, or if none is.
fn keep(list: &mut Vec<Keyed>, key: (bool, i64), edge: Edge, reach_of: impl Fn(&Edge) -> i64, reach: i64) {
    match list.iter_mut().find(|k| k.key == key) {
        Some(k) if reach >= reach_of(&k.edge) => k.edge = edge,
        Some(_) => {}
        None => list.push(Keyed { key, edge }),
    }
}

/// Two frames that differ in only a few lines (a blinking caret, a clock): not a scroll.
fn nothing_moved(a: &RgbaImage, b: &RgbaImage) -> bool {
    let rows_a = line_hashes(a, true, None);
    let rows_b = line_hashes(b, true, None);
    let differ = rows_a.iter().zip(&rows_b).filter(|(x, y)| x != y).count();
    differ * 20 < rows_a.len().max(1)
}

/// The shift between two frames along the allowed directions, or None if they do not
/// overlap enough to tell.
fn find_shift(a: &RgbaImage, b: &RgbaImage, direction: Direction) -> Option<Shift> {
    let vertical = matches!(direction, Direction::Vertical | Direction::Both)
        .then(|| shift_along(a, b, true))
        .flatten();
    let horizontal = matches!(direction, Direction::Horizontal | Direction::Both)
        .then(|| shift_along(a, b, false))
        .flatten();
    match (vertical, horizontal) {
        (Some(v), Some(h)) => {
            let moved = |s: &Shift| s.dx != 0 || s.dy != 0;
            Some(match (moved(&v), moved(&h)) {
                (true, false) => v,
                (false, true) => h,
                _ if h.score > v.score => h,
                _ => v,
            })
        }
        (v, h) => v.or(h),
    }
}

/// The shift along one axis: rows move for `vertical`, columns otherwise.
fn shift_along(a: &RgbaImage, b: &RgbaImage, vertical: bool) -> Option<Shift> {
    // Lines across the motion that do not change at all (a fixed side bar while scrolling
    // down, a fixed toolbar while scrolling sideways) say nothing about the shift.
    let across_a = line_hashes(a, !vertical, None);
    let across_b = line_hashes(b, !vertical, None);
    let moving: Vec<bool> = across_a.iter().zip(&across_b).map(|(x, y)| x != y).collect();
    if !moving.iter().any(|m| *m) {
        return Some(Shift {
            dx: 0,
            dy: 0,
            lead: 0,
            trail: 0,
            score: 1.0,
            doubtful: false,
        });
    }
    let la = line_hashes(a, vertical, Some(&moving));
    let lb = line_hashes(b, vertical, Some(&moving));
    let n = la.len();
    let lead = la.iter().zip(&lb).take_while(|(x, y)| x == y).count();
    if lead == n {
        return Some(Shift {
            dx: 0,
            dy: 0,
            lead: 0,
            trail: 0,
            score: 1.0,
            doubtful: false,
        });
    }
    let trail = la
        .iter()
        .rev()
        .zip(lb.iter().rev())
        .take_while(|(x, y)| x == y)
        .count();
    let (start, end) = (lead, n - trail);
    if end <= start + MIN_OVERLAP {
        return None;
    }
    let band = end - start;
    let (mut best_d, mut best_score, mut best_overlap) = (0i64, -1.0f32, 0usize);
    for d in -(band as i64 - MIN_OVERLAP as i64)..=(band as i64 - MIN_OVERLAP as i64) {
        if d == 0 {
            continue;
        }
        if let Some((score, overlap)) = exact_score(&la, &lb, start, end, d) {
            if score > best_score || (score == best_score && overlap > best_overlap) {
                (best_d, best_score, best_overlap) = (d, score, overlap);
            }
        }
    }
    let mut doubtful = false;
    if best_score < GOOD_SCORE {
        // Content drawn again while it scrolls is not always the same to the bit, and a
        // floating button spoils a few lines: compare coarse gray profiles of the lines.
        let pa = profiles(a, vertical, &moving);
        let pb = profiles(b, vertical, &moving);
        if best_score >= MIN_SCORE {
            // The exact join stands; it is sure if the profiles agree with it.
            doubtful = profile_error(&pa, &pb, start, end, best_d) > PROFILE_TOLERANCE;
        } else {
            let (mut fuzzy_d, mut fuzzy_err) = (0i64, f32::MAX);
            for d in -(band as i64 - MIN_OVERLAP as i64)..=(band as i64 - MIN_OVERLAP as i64) {
                if d == 0 {
                    continue;
                }
                let err = profile_error(&pa, &pb, start, end, d);
                if err < fuzzy_err {
                    (fuzzy_d, fuzzy_err) = (d, err);
                }
            }
            if fuzzy_err > PROFILE_TOLERANCE {
                return None;
            }
            best_d = fuzzy_d;
            doubtful = true;
        }
    }
    let (dx, dy) = if vertical {
        (0, best_d as i32)
    } else {
        (best_d as i32, 0)
    };
    Some(Shift {
        dx,
        dy,
        lead,
        trail,
        score: best_score.max(0.0),
        doubtful,
    })
}

/// How well `b` matches `a` moved by `d` lines: the share of `b`'s telling lines (those
/// that differ from the line before them) found in `a`, and how many lines overlap. The
/// share is damped when there are few telling lines, so a short overlap of blank lines
/// never beats the real join.
fn exact_score(la: &[u64], lb: &[u64], start: usize, end: usize, d: i64) -> Option<(f32, usize)> {
    let (mut telling, mut matched, mut overlap) = (0usize, 0usize, 0usize);
    for y in start..end {
        let ya = y as i64 + d;
        if ya < start as i64 || ya >= end as i64 {
            continue;
        }
        overlap += 1;
        if y > start && lb[y] == lb[y - 1] {
            continue;
        }
        telling += 1;
        if la[ya as usize] == lb[y] {
            matched += 1;
        }
    }
    if overlap < MIN_OVERLAP || telling < 3 {
        return None;
    }
    Some((matched as f32 / (telling + 3) as f32, overlap))
}

/// Mean difference of the gray profiles of `b` and `a` moved by `d` lines.
fn profile_error(
    pa: &[[u8; PROFILE_BINS]],
    pb: &[[u8; PROFILE_BINS]],
    start: usize,
    end: usize,
    d: i64,
) -> f32 {
    let (mut sum, mut count, mut overlap) = (0u64, 0u64, 0usize);
    for (y, q) in pb.iter().enumerate().take(end).skip(start) {
        let ya = y as i64 + d;
        if ya < start as i64 || ya >= end as i64 {
            continue;
        }
        overlap += 1;
        // Flat lines (blank stretches) match anywhere: they say nothing.
        let (lo, hi) = (
            q.iter().min().copied().unwrap_or(0),
            q.iter().max().copied().unwrap_or(0),
        );
        if hi - lo < 12 {
            continue;
        }
        let p = &pa[ya as usize];
        sum += p
            .iter()
            .zip(q)
            .map(|(x, y)| (*x as i32 - *y as i32).unsigned_abs() as u64)
            .sum::<u64>();
        count += PROFILE_BINS as u64;
    }
    // A fair part of the band must overlap, with enough telling lines.
    if overlap * 4 < end - start || count < (8 * PROFILE_BINS) as u64 {
        return f32::MAX;
    }
    sum as f32 / count as f32
}

/// A hash of each line: rows when `rows`, columns otherwise; only over the cells `mask`
/// keeps across the line.
fn line_hashes(img: &RgbaImage, rows: bool, mask: Option<&[bool]>) -> Vec<u64> {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let raw = img.as_raw();
    let px = |x: usize, y: usize| -> u32 {
        let i = (y * w + x) * 4;
        u32::from_le_bytes([raw[i], raw[i + 1], raw[i + 2], 0])
    };
    let (lines, across) = if rows { (h, w) } else { (w, h) };
    (0..lines)
        .map(|l| {
            let mut hash = 0xcbf2_9ce4_8422_2325u64;
            for c in 0..across {
                if mask.is_some_and(|m| !m[c]) {
                    continue;
                }
                let v = if rows { px(c, l) } else { px(l, c) };
                hash ^= v as u64;
                hash = hash.wrapping_mul(0x0100_0000_01b3);
            }
            hash
        })
        .collect()
}

/// The coarse gray profile of each line: its moving cells split into bins, averaged.
fn profiles(img: &RgbaImage, rows: bool, mask: &[bool]) -> Vec<[u8; PROFILE_BINS]> {
    let (w, h) = (img.width() as usize, img.height() as usize);
    let raw = img.as_raw();
    let (lines, across) = if rows { (h, w) } else { (w, h) };
    let cells: Vec<usize> = (0..across).filter(|c| mask[*c]).collect();
    (0..lines)
        .map(|l| {
            let mut sums = [0u32; PROFILE_BINS];
            let mut counts = [0u32; PROFILE_BINS];
            for (k, &c) in cells.iter().enumerate() {
                let (x, y) = if rows { (c, l) } else { (l, c) };
                let i = (y * w + x) * 4;
                let gray = (raw[i] as u32 * 77 + raw[i + 1] as u32 * 150 + raw[i + 2] as u32 * 29) >> 8;
                let bin = k * PROFILE_BINS / cells.len().max(1);
                sums[bin] += gray;
                counts[bin] += 1;
            }
            let mut out = [0u8; PROFILE_BINS];
            for b in 0..PROFILE_BINS {
                out[b] = sums[b].checked_div(counts[b]).unwrap_or(0) as u8;
            }
            out
        })
        .collect()
}

/// An image scaled down to fit `max_w` × `max_h` (never up).
fn shrink(img: &RgbaImage, max_w: u32, max_h: u32) -> RgbaImage {
    let (w, h) = img.dimensions();
    let k = (max_w as f64 / w as f64).min(max_h as f64 / h as f64).min(1.0);
    if k >= 1.0 {
        return img.clone();
    }
    let (nw, nh) = (
        ((w as f64 * k).round() as u32).max(1),
        ((h as f64 * k).round() as u32).max(1),
    );
    image::imageops::resize(img, nw, nh, image::imageops::FilterType::Triangle)
}

/// A canvas that grows in any direction, in content coordinates.
#[derive(Default)]
struct Canvas {
    /// Content coordinates of the canvas's top left pixel.
    x: i64,
    y: i64,
    w: usize,
    h: usize,
    data: Vec<u8>,
    /// What has been painted: x0, y0, x1, y1 (content coordinates).
    painted: (i64, i64, i64, i64),
}

impl Canvas {
    /// Makes room for the rectangle, with slack in the direction it grows.
    fn ensure(&mut self, x0: i64, y0: i64, x1: i64, y1: i64) {
        if self.w > 0 && x0 >= self.x && y0 >= self.y {
            let (cx1, cy1) = (self.x + self.w as i64, self.y + self.h as i64);
            if x1 <= cx1 && y1 <= cy1 {
                return;
            }
        }
        let slack = 2048;
        let (nx0, ny0, nx1, ny1) = if self.w == 0 {
            (x0, y0, x1, y1)
        } else {
            let (cx1, cy1) = (self.x + self.w as i64, self.y + self.h as i64);
            (
                if x0 < self.x { x0 - slack } else { self.x },
                if y0 < self.y { y0 - slack } else { self.y },
                if x1 > cx1 { x1 + slack } else { cx1 },
                if y1 > cy1 { y1 + slack } else { cy1 },
            )
        };
        let (nw, nh) = ((nx1 - nx0) as usize, (ny1 - ny0) as usize);
        let mut data = vec![0u8; nw * nh * 4];
        for row in 0..self.h {
            let src = row * self.w * 4;
            let dy = (self.y - ny0) as usize + row;
            let dx = (self.x - nx0) as usize;
            let dst = (dy * nw + dx) * 4;
            data[dst..dst + self.w * 4].copy_from_slice(&self.data[src..src + self.w * 4]);
        }
        if self.w == 0 {
            self.painted = (x0, y0, x0, y0);
        }
        (self.x, self.y, self.w, self.h, self.data) = (nx0, ny0, nw, nh, data);
    }

    /// Paints `img` (or the part `rect` of it: x0, y0, x1, y1) with its top left at
    /// content point (`x`, `y`).
    fn paint(&mut self, img: &RgbaImage, x: i64, y: i64, rect: Option<(usize, usize, usize, usize)>) {
        let (iw, ih) = (img.width() as usize, img.height() as usize);
        let (rx0, ry0, rx1, ry1) = rect.unwrap_or((0, 0, iw, ih));
        let (px0, py0) = (x + rx0 as i64, y + ry0 as i64);
        let (px1, py1) = (x + rx1 as i64, y + ry1 as i64);
        self.ensure(px0, py0, px1, py1);
        let raw = img.as_raw();
        let len = (rx1 - rx0) * 4;
        for row in ry0..ry1 {
            let src = (row * iw + rx0) * 4;
            let cy = (y + row as i64 - self.y) as usize;
            let cx = (px0 - self.x) as usize;
            let dst = (cy * self.w + cx) * 4;
            self.data[dst..dst + len].copy_from_slice(&raw[src..src + len]);
        }
        let p = &mut self.painted;
        *p = (p.0.min(px0), p.1.min(py0), p.2.max(px1), p.3.max(py1));
    }

    fn crop(&self, x0: i64, y0: i64, x1: i64, y1: i64) -> RgbaImage {
        let (w, h) = ((x1 - x0) as usize, (y1 - y0) as usize);
        let mut out = vec![0u8; w * h * 4];
        for row in 0..h {
            let src = (((y0 - self.y) as usize + row) * self.w + (x0 - self.x) as usize) * 4;
            out[row * w * 4..(row + 1) * w * 4].copy_from_slice(&self.data[src..src + w * 4]);
        }
        RgbaImage::from_raw(w as u32, h as u32, out).expect("the canvas size")
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use image::Rgba;

    /// A tall "page" with varied content: text-like lines, boxes and blank stretches.
    pub(crate) fn page(w: u32, h: u32, seed: u32) -> RgbaImage {
        let mut img = RgbaImage::from_pixel(w, h, Rgba([250, 250, 252, 255]));
        let mut s = seed;
        let mut rnd = || {
            s = s.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            (s >> 16) & 0x7fff
        };
        let mut y = 10;
        while y + 30 < h {
            let kind = rnd() % 4;
            let lh = 8 + rnd() % 18;
            if kind != 3 {
                let x0 = 10 + rnd() % 40;
                let x1 = (x0 + 60 + rnd() % (w - 120).max(1)).min(w - 5);
                let c = [(rnd() % 200) as u8, (rnd() % 200) as u8, (rnd() % 200) as u8, 255];
                for yy in y..(y + lh).min(h) {
                    for xx in x0..x1 {
                        // Text-like: broken strokes.
                        if (xx + yy * 3 + seed) % 7 < 4 || kind == 1 {
                            img.put_pixel(xx, yy, Rgba(c));
                        }
                    }
                }
            }
            y += lh + 4 + rnd() % 30;
        }
        img
    }

    fn view(src: &RgbaImage, x: u32, y: u32, w: u32, h: u32) -> RgbaImage {
        image::imageops::crop_imm(src, x, y, w, h).to_image()
    }

    fn with_bars(mut frame: RgbaImage, header: &RgbaImage, footer: &RgbaImage) -> RgbaImage {
        image::imageops::replace(&mut frame, header, 0, 0);
        let fy = frame.height() - footer.height();
        image::imageops::replace(&mut frame, footer, 0, fy as i64);
        frame
    }

    #[test]
    fn joins_a_page_scrolled_down_without_gaps_or_repeats() {
        let content = page(400, 3000, 7);
        let mut s = Stitcher::new(Direction::Vertical);
        let mut y = 0;
        while y + 600 <= 3000 {
            s.push(view(&content, 0, y, 400, 600));
            y += 173;
        }
        // The last view, at the very bottom.
        s.push(view(&content, 0, 2400, 400, 600));
        let out = s.finish().expect("an image");
        assert_eq!(out.dimensions(), (400, 3000));
        assert!(out.as_raw() == content.as_raw());
    }

    #[test]
    fn a_fixed_header_and_footer_appear_once() {
        let content = page(400, 2400, 3);
        let header = RgbaImage::from_fn(400, 80, |x, y| {
            Rgba([30, 60, (x % 200) as u8, (200 + y % 50) as u8])
        });
        let footer = RgbaImage::from_fn(400, 40, |x, _| Rgba([200, (x % 90) as u8, 40, 255]));
        let mut s = Stitcher::new(Direction::Vertical);
        // The page scrolls under the bars: views of 600 px, 480 of them content.
        let mut y = 0;
        let mut last = 0;
        while y + 480 <= 2400 {
            s.push(with_bars(view(&content, 0, y, 400, 600), &header, &footer));
            last = y;
            y += 150;
        }
        let out = s.finish().expect("an image");
        // Header + the content seen (from 80 to the last view's 80 + 480) + footer.
        assert_eq!(out.dimensions(), (400, last + 600));
        assert_eq!(view(&out, 0, 0, 400, 80).as_raw(), header.as_raw());
        let fy = out.height() - 40;
        assert_eq!(view(&out, 0, fy, 400, 40).as_raw(), footer.as_raw());
        // The content between them, once and in order.
        let seen = view(&out, 0, 80, 400, fy - 80);
        assert_eq!(seen.as_raw(), view(&content, 0, 80, 400, fy - 80).as_raw());
    }

    #[test]
    fn joins_sideways() {
        let tall = page(300, 2000, 11);
        let wide = image::imageops::rotate90(&tall);
        let mut s = Stitcher::new(Direction::Horizontal);
        for x in (0..=1500).step_by(125) {
            s.push(view(&wide, x, 0, 500, 300));
        }
        let out = s.finish().expect("an image");
        assert_eq!(out.dimensions(), (2000, 300));
        assert!(out.as_raw() == wide.as_raw());
    }

    #[test]
    fn down_then_right_grows_both_ways() {
        let tall = page(900, 1400, 5);
        let mut s = Stitcher::new(Direction::Both);
        for y in (0..=600).step_by(150) {
            s.push(view(&tall, 0, y, 500, 800));
        }
        for x in (100..=400).step_by(100) {
            s.push(view(&tall, x, 600, 500, 800));
        }
        let out = s.finish().expect("an image");
        assert_eq!(out.dimensions(), (900, 1400));
        // What the views saw is in place.
        assert_eq!(
            view(&out, 400, 600, 500, 800).as_raw(),
            view(&tall, 400, 600, 500, 800).as_raw()
        );
        assert_eq!(
            view(&out, 0, 0, 500, 800).as_raw(),
            view(&tall, 0, 0, 500, 800).as_raw()
        );
    }

    #[test]
    fn a_frame_that_did_not_move_adds_nothing() {
        let content = page(300, 1200, 2);
        let mut s = Stitcher::new(Direction::Vertical);
        assert_eq!(s.push(view(&content, 0, 0, 300, 400)), Pushed::First);
        assert_eq!(s.push(view(&content, 0, 0, 300, 400)), Pushed::Same);
        assert!(matches!(
            s.push(view(&content, 0, 100, 300, 400)),
            Pushed::Joined { dy: 100, .. }
        ));
        assert_eq!(s.size(), (300, 500));
    }

    #[test]
    fn scrolling_back_up_keeps_the_result() {
        let content = page(300, 1500, 9);
        let mut s = Stitcher::new(Direction::Vertical);
        for y in [0, 200, 400, 600, 400, 200, 500] {
            s.push(view(&content, 0, y, 300, 500));
        }
        let out = s.finish().expect("an image");
        assert_eq!(out.dimensions(), (300, 1100));
        assert!(out.as_raw() == view(&content, 0, 0, 300, 1100).as_raw());
    }

    #[test]
    fn a_jump_too_far_is_left_out() {
        let content = page(300, 3000, 4);
        let mut s = Stitcher::new(Direction::Vertical);
        s.push(view(&content, 0, 0, 300, 400));
        assert_eq!(s.push(view(&content, 0, 1500, 300, 400)), Pushed::Lost);
        assert!(matches!(
            s.push(view(&content, 0, 200, 300, 400)),
            Pushed::Joined { dy: 200, .. }
        ));
    }

    #[test]
    fn small_redraw_differences_still_join_but_are_doubtful() {
        let content = page(400, 1600, 8);
        let mut s = Stitcher::new(Direction::Vertical);
        s.push(view(&content, 0, 0, 400, 600));
        // The next view, drawn again with every pixel one level off.
        let mut next = view(&content, 0, 250, 400, 600);
        for p in next.pixels_mut() {
            p.0[0] = p.0[0].saturating_add(1);
            p.0[2] = p.0[2].saturating_sub(1);
        }
        assert_eq!(
            s.push(next),
            Pushed::Joined {
                dx: 0,
                dy: 250,
                doubtful: true
            }
        );
        assert_eq!(s.joins(), (1, 1));
    }

    #[test]
    fn stops_at_the_largest_size() {
        let content = page(300, 600, 6);
        let mut s = Stitcher::new(Direction::Vertical);
        s.push(view(&content, 0, 0, 300, 200));
        // Pretend the result is already 60 000 px tall.
        s.joins = 1;
        s.canvas.painted = (0, 0, 300, MAX_SIDE as i64 - 50);
        s.at = (0, MAX_SIDE as i64 - 250);
        s.pending_first = false;
        assert_eq!(s.push(view(&content, 0, 100, 300, 200)), Pushed::Full);
        assert!(s.is_full());
        assert_eq!(s.push(view(&content, 0, 150, 300, 200)), Pushed::Full);
    }

    #[test]
    fn a_floating_button_stays_where_the_last_view_had_it() {
        let content = page(400, 2000, 12);
        let button = RgbaImage::from_pixel(40, 40, Rgba([220, 20, 60, 255]));
        let mut s = Stitcher::new(Direction::Vertical);
        let mut last = 0;
        for y in (0..=1400).step_by(140) {
            let mut v = view(&content, 0, y, 400, 600);
            image::imageops::replace(&mut v, &button, 340, 520);
            s.push(v);
            last = y;
        }
        let out = s.finish().expect("an image");
        assert_eq!(out.dimensions(), (400, 2000));
        // Once, at its place in the last view; the content elsewhere is clean.
        assert_eq!(view(&out, 340, last + 520, 40, 40).as_raw(), button.as_raw());
        assert_eq!(
            view(&out, 0, 0, 400, 1900).as_raw(),
            view(&content, 0, 0, 400, 1900).as_raw()
        );
    }
}
