//! Pixel selections (EDT-043 to EDT-046): how much of each pixel of an image is
//! selected, what the selected pixels are, and what fills the gap they leave.

use std::collections::HashMap;

use image::RgbaImage;
use tiny_skia::{FillRule, Paint, PathBuilder, Pixmap, Rect as SkRect, Transform};

use crate::color::Rgba;

pub use tiny_skia::Point;

/// How much of each pixel is selected, 0–255, over a whole image.
#[derive(Debug, Clone, PartialEq)]
pub struct Mask {
    pub width: u32,
    pub height: u32,
    pub alpha: Vec<u8>,
}

/// A drawn selection shape, in the coordinates `to_image` takes to the image's pixels.
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    Rectangle([f64; 4]),
    Ellipse([f64; 4]),
    /// Polygon or freehand loop: closed by itself.
    Polygon(Vec<[f64; 2]>),
}

/// How a new part joins the selection (EDT-043).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Combine {
    Replace,
    Add,
    Subtract,
}

/// What fills the place selected pixels leave (EDT-045).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Gap {
    /// The canvas color shows.
    Transparent,
    /// A solid color.
    Color(Rgba),
    /// Rebuilt from the pixels around it.
    Smart,
}

impl Mask {
    pub fn empty(width: u32, height: u32) -> Mask {
        Mask {
            width,
            height,
            alpha: vec![0; (width * height) as usize],
        }
    }

    pub fn full(width: u32, height: u32) -> Mask {
        Mask {
            width,
            height,
            alpha: vec![255; (width * height) as usize],
        }
    }

    pub fn at(&self, x: u32, y: u32) -> u8 {
        self.alpha[(y * self.width + x) as usize]
    }

    pub fn is_empty(&self) -> bool {
        self.alpha.iter().all(|&a| a == 0)
    }

    /// `shape` drawn into an image of `width` × `height`, with `to_image` taking its
    /// points there. Smooth edges with `antialias`.
    pub fn of_shape(width: u32, height: u32, shape: &Shape, to_image: Transform, antialias: bool) -> Mask {
        let mut out = Mask::empty(width, height);
        let Some(mut pm) = Pixmap::new(width.max(1), height.max(1)) else {
            return out;
        };
        let path = match shape {
            Shape::Rectangle([x, y, w, h]) => {
                SkRect::from_xywh(*x as f32, *y as f32, *w as f32, *h as f32).map(PathBuilder::from_rect)
            }
            Shape::Ellipse([x, y, w, h]) => {
                SkRect::from_xywh(*x as f32, *y as f32, *w as f32, *h as f32).and_then(PathBuilder::from_oval)
            }
            Shape::Polygon(points) => {
                let mut pb = PathBuilder::new();
                for (i, p) in points.iter().enumerate() {
                    if i == 0 {
                        pb.move_to(p[0] as f32, p[1] as f32);
                    } else {
                        pb.line_to(p[0] as f32, p[1] as f32);
                    }
                }
                pb.close();
                pb.finish()
            }
        };
        let Some(path) = path else {
            return out;
        };
        let mut paint = Paint::default();
        paint.set_color_rgba8(0, 0, 0, 255);
        paint.anti_alias = antialias;
        pm.fill_path(&path, &paint, FillRule::Winding, to_image, None);
        for (a, px) in out.alpha.iter_mut().zip(pm.pixels()) {
            *a = px.alpha();
        }
        out
    }

    /// The pixels of a color like the one at `seed`, touching it or anywhere with
    /// `global` (EDT-046). Empty if `seed` is outside.
    pub fn of_color(img: &RgbaImage, seed: (u32, u32), tolerance: f64, global: bool) -> Mask {
        let mut out = Mask::empty(img.width(), img.height());
        if let Some((mask, _)) = crate::pixels::reach(img, seed, tolerance, global) {
            for (a, on) in out.alpha.iter_mut().zip(mask) {
                *a = if on { 255 } else { 0 };
            }
        }
        out
    }

    /// This selection with `part` joined as `how` says.
    pub fn combine(&self, part: &Mask, how: Combine) -> Mask {
        let alpha = match how {
            Combine::Replace => part.alpha.clone(),
            Combine::Add => self
                .alpha
                .iter()
                .zip(&part.alpha)
                .map(|(a, b)| *a.max(b))
                .collect(),
            Combine::Subtract => self
                .alpha
                .iter()
                .zip(&part.alpha)
                .map(|(a, b)| ((*a as u32 * (255 - *b as u32)) / 255) as u8)
                .collect(),
        };
        Mask { alpha, ..*self }
    }

    /// Everything that was not selected (EDT-044).
    pub fn inverted(&self) -> Mask {
        Mask {
            alpha: self.alpha.iter().map(|a| 255 - a).collect(),
            ..*self
        }
    }

    /// Softer edges over about `radius` px (EDT-043 Feather): three box blurs.
    pub fn feathered(&self, radius: f64) -> Mask {
        let r = (radius / 2.0).round() as i64;
        if r < 1 {
            return self.clone();
        }
        let (w, h) = (self.width as i64, self.height as i64);
        let mut cur: Vec<f32> = self.alpha.iter().map(|&a| a as f32).collect();
        let mut tmp = vec![0f32; cur.len()];
        for _ in 0..3 {
            for (horizontal, len, lines) in [(true, w, h), (false, h, w)] {
                for line in 0..lines {
                    let idx = |i: i64| -> usize {
                        let i = i.clamp(0, len - 1);
                        (if horizontal { line * w + i } else { i * w + line }) as usize
                    };
                    let mut sum: f32 = (-r..=r).map(|i| cur[idx(i)]).sum();
                    for i in 0..len {
                        tmp[idx(i)] = sum / (2 * r + 1) as f32;
                        sum += cur[idx(i + r + 1)] - cur[idx(i - r)];
                    }
                }
                std::mem::swap(&mut cur, &mut tmp);
            }
        }
        Mask {
            alpha: cur.iter().map(|v| v.round().clamp(0.0, 255.0) as u8).collect(),
            ..*self
        }
    }

    /// Moved by whole pixels; what goes past the edges is lost.
    pub fn shifted(&self, dx: i64, dy: i64) -> Mask {
        let mut out = Mask::empty(self.width, self.height);
        for y in 0..self.height as i64 {
            let sy = y - dy;
            if sy < 0 || sy >= self.height as i64 {
                continue;
            }
            for x in 0..self.width as i64 {
                let sx = x - dx;
                if sx >= 0 && sx < self.width as i64 {
                    out.alpha[(y * self.width as i64 + x) as usize] = self.at(sx as u32, sy as u32);
                }
            }
        }
        out
    }

    /// The rectangle around what is selected, `[x, y, w, h]`.
    pub fn bounds(&self) -> Option<[u32; 4]> {
        let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0, 0);
        for y in 0..self.height {
            let row = &self.alpha[(y * self.width) as usize..((y + 1) * self.width) as usize];
            let Some(first) = row.iter().position(|&a| a > 0) else {
                continue;
            };
            let last = row.iter().rposition(|&a| a > 0).unwrap_or(first);
            x0 = x0.min(first as u32);
            x1 = x1.max(last as u32);
            y0 = y0.min(y);
            y1 = y;
        }
        (x0 != u32::MAX).then(|| [x0, y0, x1 - x0 + 1, y1 - y0 + 1])
    }

    /// The edges of what is at least half selected, as closed loops of pixel corners:
    /// what the marching ants follow (EDT-043). Straight runs are one segment.
    pub fn outline(&self) -> Vec<Vec<[u32; 2]>> {
        let (w, h) = (self.width as i64, self.height as i64);
        let inside =
            |x: i64, y: i64| x >= 0 && y >= 0 && x < w && y < h && self.at(x as u32, y as u32) >= 128;
        // Directed edges with the inside on their right: start corner -> end corner.
        let mut next: HashMap<(i64, i64), Vec<(i64, i64)>> = HashMap::new();
        for y in 0..h {
            for x in 0..w {
                if !inside(x, y) {
                    continue;
                }
                if !inside(x, y - 1) {
                    next.entry((x, y)).or_default().push((x + 1, y));
                }
                if !inside(x + 1, y) {
                    next.entry((x + 1, y)).or_default().push((x + 1, y + 1));
                }
                if !inside(x, y + 1) {
                    next.entry((x + 1, y + 1)).or_default().push((x, y + 1));
                }
                if !inside(x - 1, y) {
                    next.entry((x, y + 1)).or_default().push((x, y));
                }
            }
        }
        let mut loops = Vec::new();
        let mut starts: Vec<(i64, i64)> = next.keys().copied().collect();
        starts.sort_unstable();
        for start in starts {
            while let Some(first) = next.get_mut(&start).and_then(Vec::pop) {
                let mut corners = vec![start];
                let mut at = first;
                while at != start {
                    corners.push(at);
                    let Some(n) = next.get_mut(&at).and_then(Vec::pop) else {
                        break;
                    };
                    at = n;
                }
                // Only the corners where the direction turns.
                let n = corners.len();
                let kept: Vec<[u32; 2]> = (0..n)
                    .filter(|&i| {
                        let (a, b, c) = (corners[(i + n - 1) % n], corners[i], corners[(i + 1) % n]);
                        (b.0 - a.0) * (c.1 - b.1) != (b.1 - a.1) * (c.0 - b.0)
                    })
                    .map(|i| [corners[i].0 as u32, corners[i].1 as u32])
                    .collect();
                if kept.len() >= 3 {
                    loops.push(kept);
                }
            }
        }
        loops
    }
}

/// The selected pixels of `img` over the selection's bounds, transparent where they
/// are not selected (EDT-044 Copy), and where that rectangle is.
pub fn selected_pixels(img: &RgbaImage, mask: &Mask) -> Option<(RgbaImage, [u32; 4])> {
    let b = mask.bounds()?;
    let out = RgbaImage::from_fn(b[2], b[3], |x, y| {
        let mut p = *img.get_pixel(b[0] + x, b[1] + y);
        p.0[3] = (p.0[3] as u32 * mask.at(b[0] + x, b[1] + y) as u32 / 255) as u8;
        p
    });
    Some((out, b))
}

/// Fills the place of the selected pixels as `gap` says (EDT-045).
pub fn fill_gap(img: &mut RgbaImage, mask: &Mask, gap: Gap) {
    match gap {
        Gap::Transparent => {
            for (p, &m) in img.pixels_mut().zip(&mask.alpha) {
                p.0[3] = (p.0[3] as u32 * (255 - m as u32) / 255) as u8;
            }
        }
        Gap::Color(c) => {
            for (p, &m) in img.pixels_mut().zip(&mask.alpha) {
                let k = m as f32 / 255.0;
                let mix = |old: u8, new: u8| (old as f32 * (1.0 - k) + new as f32 * k).round() as u8;
                p.0 = [
                    mix(p.0[0], c.r),
                    mix(p.0[1], c.g),
                    mix(p.0[2], c.b),
                    mix(p.0[3], c.a),
                ];
            }
        }
        Gap::Smart => smart_fill(img, mask),
    }
}

/// Rebuilds the selected place from its surroundings (EDT-045 Smart fill, Fast): the
/// smoothest surface that meets the pixels around it, solved coarse to fine. Flat
/// backgrounds and gradients come back without a mark.
fn smart_fill(img: &mut RgbaImage, mask: &Mask) {
    let Some(b) = mask.bounds() else {
        return;
    };
    // Work on the selection's bounds and a margin of known pixels around it.
    let m = 8;
    let x0 = b[0].saturating_sub(m);
    let y0 = b[1].saturating_sub(m);
    let x1 = (b[0] + b[2] + m).min(img.width());
    let y1 = (b[1] + b[3] + m).min(img.height());
    let (w, h) = ((x1 - x0) as usize, (y1 - y0) as usize);
    // Known pixels weigh 1, selected ones 0 (premultiplied colors, so transparency
    // fills in too).
    let mut known = vec![0f32; w * h];
    let mut color = vec![[0f32; 4]; w * h];
    for y in 0..h {
        for x in 0..w {
            let (ix, iy) = (x0 + x as u32, y0 + y as u32);
            let k = 1.0 - mask.at(ix, iy) as f32 / 255.0;
            let p = img.get_pixel(ix, iy).0;
            let a = p[3] as f32 / 255.0;
            known[y * w + x] = if k >= 0.999 { 1.0 } else { 0.0 };
            color[y * w + x] = [p[0] as f32 * a, p[1] as f32 * a, p[2] as f32 * a, p[3] as f32];
        }
    }
    let mut filled = membrane(&color, &known, w, h);
    // Interfaces: where the pixels on both sides of the gap, across a row or a column,
    // are the same color, the gap takes it, sharp; the shorter crossing wins.
    let crisp = sides_alike(&color, &known, w, h);
    for (f, c) in filled.iter_mut().zip(crisp) {
        if let Some(c) = c {
            *f = c;
        }
    }
    for y in 0..h {
        for x in 0..w {
            let (ix, iy) = (x0 + x as u32, y0 + y as u32);
            let k = mask.at(ix, iy) as f32 / 255.0;
            if k <= 0.0 {
                continue;
            }
            let f = filled[y * w + x];
            let a = f[3].clamp(0.0, 255.0);
            let un = |c: f32| {
                if a > 0.0 {
                    (c * 255.0 / a).clamp(0.0, 255.0)
                } else {
                    0.0
                }
            };
            let p = img.get_pixel_mut(ix, iy);
            let mix = |old: u8, new: f32| (old as f32 * (1.0 - k) + new * k).round() as u8;
            p.0 = [
                mix(p.0[0], un(f[0])),
                mix(p.0[1], un(f[1])),
                mix(p.0[2], un(f[2])),
                mix(p.0[3], a),
            ];
        }
    }
}

/// For each unknown cell, the color of the known cells at both ends of its row or
/// column run when they match, from the shorter run.
fn sides_alike(color: &[[f32; 4]], known: &[f32], w: usize, h: usize) -> Vec<Option<[f32; 4]>> {
    let alike = |a: [f32; 4], b: [f32; 4]| (0..4).all(|k| (a[k] - b[k]).abs() <= 8.0);
    let mut best: Vec<Option<([f32; 4], usize)>> = vec![None; w * h];
    // Rows, then columns: the same walk over a line of cells.
    for (lines, len, at) in [
        (
            h,
            w,
            &(|line: usize, i: usize| line * w + i) as &dyn Fn(usize, usize) -> usize,
        ),
        (
            w,
            h,
            &(|line: usize, i: usize| i * w + line) as &dyn Fn(usize, usize) -> usize,
        ),
    ] {
        for line in 0..lines {
            let mut i = 0;
            while i < len {
                if known[at(line, i)] > 0.0 {
                    i += 1;
                    continue;
                }
                let start = i;
                while i < len && known[at(line, i)] == 0.0 {
                    i += 1;
                }
                let (before, after) = (start.checked_sub(1), (i < len).then_some(i));
                let (Some(b), Some(a)) = (before, after) else {
                    continue;
                };
                let (cb, ca) = (color[at(line, b)], color[at(line, a)]);
                if !alike(cb, ca) {
                    continue;
                }
                let span = i - start;
                for j in start..i {
                    let cell = &mut best[at(line, j)];
                    if cell.is_none_or(|(_, s)| span < s) {
                        *cell = Some((cb, span));
                    }
                }
            }
        }
    }
    best.into_iter().map(|c| c.map(|(c, _)| c)).collect()
}

/// Fills the unknown cells (`known` 0) of a grid so each is the mean of its
/// neighbors: a coarser grid gives the start, then a few relaxations at this one.
fn membrane(color: &[[f32; 4]], known: &[f32], w: usize, h: usize) -> Vec<[f32; 4]> {
    let mut out = color.to_vec();
    if known.iter().all(|&k| k > 0.0) || known.iter().all(|&k| k == 0.0) {
        return out;
    }
    if w > 2 && h > 2 {
        // Coarser: the mean of the known cells of each 2 × 2 block.
        let (cw, ch) = (w.div_ceil(2), h.div_ceil(2));
        let mut ccolor = vec![[0f32; 4]; cw * ch];
        let mut cknown = vec![0f32; cw * ch];
        for y in 0..h {
            for x in 0..w {
                let (i, c) = (y * w + x, (y / 2) * cw + x / 2);
                if known[i] > 0.0 {
                    for k in 0..4 {
                        ccolor[c][k] += color[i][k];
                    }
                    cknown[c] += 1.0;
                }
            }
        }
        for (cc, ck) in ccolor.iter_mut().zip(cknown.iter_mut()) {
            if *ck > 0.0 {
                for v in cc.iter_mut() {
                    *v /= *ck;
                }
                *ck = 1.0;
            }
        }
        let coarse = membrane(&ccolor, &cknown, cw, ch);
        for y in 0..h {
            for x in 0..w {
                if known[y * w + x] == 0.0 {
                    out[y * w + x] = coarse[(y / 2) * cw + x / 2];
                }
            }
        }
    }
    let iterations = if w * h <= 64 { 200 } else { 12 };
    for _ in 0..iterations {
        for y in 0..h {
            for x in 0..w {
                let i = y * w + x;
                if known[i] > 0.0 {
                    continue;
                }
                let mut sum = [0f32; 4];
                let mut n = 0.0;
                for (nx, ny) in [
                    (x.wrapping_sub(1), y),
                    (x + 1, y),
                    (x, y.wrapping_sub(1)),
                    (x, y + 1),
                ] {
                    if nx < w && ny < h {
                        let q = out[ny * w + nx];
                        for k in 0..4 {
                            sum[k] += q[k];
                        }
                        n += 1.0;
                    }
                }
                for k in 0..4 {
                    out[i][k] = sum[k] / n;
                }
            }
        }
    }
    out
}

/// Snap to object (EDT-043): the rectangle `rect` (`[x, y, w, h]` in `img`'s pixels)
/// shrunk to the visual element inside it: what differs from the color around the
/// rectangle's border. `level` 0–100 goes from the smallest element (at the center,
/// or the biggest) to the widest group: parts closer than a few pixels join.
pub fn snap(img: &RgbaImage, rect: [u32; 4], level: f64) -> Option<[u32; 4]> {
    let (iw, ih) = img.dimensions();
    let x0 = rect[0].min(iw);
    let y0 = rect[1].min(ih);
    let x1 = (rect[0] + rect[2]).min(iw);
    let y1 = (rect[1] + rect[3]).min(ih);
    if x1 <= x0 + 1 || y1 <= y0 + 1 {
        return None;
    }
    let (w, h) = ((x1 - x0) as usize, (y1 - y0) as usize);
    // The background: the most common color along the border.
    let mut counts: HashMap<[u8; 4], usize> = HashMap::new();
    for x in x0..x1 {
        *counts.entry(img.get_pixel(x, y0).0).or_default() += 1;
        *counts.entry(img.get_pixel(x, y1 - 1).0).or_default() += 1;
    }
    for y in y0..y1 {
        *counts.entry(img.get_pixel(x0, y).0).or_default() += 1;
        *counts.entry(img.get_pixel(x1 - 1, y).0).or_default() += 1;
    }
    let bg = counts.into_iter().max_by_key(|(_, n)| *n)?.0;
    let differs = |p: [u8; 4]| (0..4).any(|c| (p[c] as i32 - bg[c] as i32).abs() > 24);
    let content: Vec<bool> = (0..w * h)
        .map(|i| differs(img.get_pixel(x0 + (i % w) as u32, y0 + (i / w) as u32).0))
        .collect();
    if !content.iter().any(|&c| c) {
        return None;
    }
    // Parts closer than `gap` px are one element.
    let gap = (1.0 + level.clamp(0.0, 100.0) / 100.0 * 23.0).round() as usize;
    let grown = grow(&content, w, h, gap / 2);
    // Pieces of the grown content.
    let mut label = vec![0usize; w * h];
    let mut pieces: Vec<(usize, [usize; 4])> = Vec::new();
    for start in 0..w * h {
        if !grown[start] || label[start] != 0 {
            continue;
        }
        let id = pieces.len() + 1;
        let mut stack = vec![start];
        label[start] = id;
        let mut b = [usize::MAX, usize::MAX, 0, 0];
        let mut size = 0;
        while let Some(i) = stack.pop() {
            let (x, y) = (i % w, i / w);
            if content[i] {
                size += 1;
                b = [b[0].min(x), b[1].min(y), b[2].max(x), b[3].max(y)];
            }
            for (nx, ny) in [
                (x.wrapping_sub(1), y),
                (x + 1, y),
                (x, y.wrapping_sub(1)),
                (x, y + 1),
            ] {
                if nx < w && ny < h {
                    let j = ny * w + nx;
                    if grown[j] && label[j] == 0 {
                        label[j] = id;
                        stack.push(j);
                    }
                }
            }
        }
        if size > 0 {
            pieces.push((size, b));
        } else {
            pieces.push((0, [0, 0, 0, 0]));
        }
    }
    let center = label[(h / 2) * w + w / 2];
    let chosen = if center > 0 && pieces[center - 1].0 > 0 {
        pieces[center - 1]
    } else {
        *pieces.iter().max_by_key(|p| p.0)?
    };
    let b = chosen.1;
    Some([
        x0 + b[0] as u32,
        y0 + b[1] as u32,
        (b[2] - b[0] + 1) as u32,
        (b[3] - b[1] + 1) as u32,
    ])
}

/// `on` grown by `r` cells in every direction (a square).
fn grow(on: &[bool], w: usize, h: usize, r: usize) -> Vec<bool> {
    if r == 0 {
        return on.to_vec();
    }
    let mut rows = vec![false; w * h];
    for y in 0..h {
        let mut last: Option<usize> = None;
        let mut next = (0..w).find(|&x| on[y * w + x]);
        for x in 0..w {
            if on[y * w + x] {
                last = Some(x);
            }
            if next.is_some_and(|n| n < x) {
                next = (x..w).find(|&k| on[y * w + k]);
            }
            let near_last = last.is_some_and(|l| x - l <= r);
            let near_next = next.is_some_and(|n| n - x <= r);
            rows[y * w + x] = near_last || near_next;
        }
    }
    let mut out = vec![false; w * h];
    for x in 0..w {
        let mut last: Option<usize> = None;
        let mut next = (0..h).find(|&y| rows[y * w + x]);
        for y in 0..h {
            if rows[y * w + x] {
                last = Some(y);
            }
            if next.is_some_and(|n| n < y) {
                next = (y..h).find(|&k| rows[k * w + x]);
            }
            out[y * w + x] = last.is_some_and(|l| y - l <= r) || next.is_some_and(|n| n - y <= r);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shapes_combine_into_one_selection() {
        let a = Mask::of_shape(
            100,
            60,
            &Shape::Rectangle([10.0, 10.0, 30.0, 20.0]),
            Transform::identity(),
            false,
        );
        assert_eq!(a.bounds(), Some([10, 10, 30, 20]));
        let b = Mask::of_shape(
            100,
            60,
            &Shape::Rectangle([30.0, 20.0, 30.0, 20.0]),
            Transform::identity(),
            false,
        );
        let both = a.combine(&b, Combine::Add);
        assert_eq!(both.bounds(), Some([10, 10, 50, 30]), "the union");
        let cut = both.combine(&b, Combine::Subtract);
        assert_eq!(cut.at(35, 25), 0);
        assert_eq!(cut.at(15, 15), 255);
        assert_eq!(both.inverted().at(0, 0), 255);
        // An ellipse leaves its corners out.
        let e = Mask::of_shape(
            100,
            60,
            &Shape::Ellipse([0.0, 0.0, 100.0, 60.0]),
            Transform::identity(),
            true,
        );
        assert_eq!(e.at(1, 1), 0);
        assert_eq!(e.at(50, 30), 255);
    }

    #[test]
    fn the_outline_follows_the_edges() {
        let mut m = Mask::of_shape(
            20,
            20,
            &Shape::Rectangle([2.0, 3.0, 5.0, 4.0]),
            Transform::identity(),
            false,
        );
        assert_eq!(m.outline(), vec![vec![[2, 3], [7, 3], [7, 7], [2, 7]]]);
        // Two pieces, two loops; a hole is a loop too.
        m = m.combine(
            &Mask::of_shape(
                20,
                20,
                &Shape::Rectangle([10.0, 10.0, 6.0, 6.0]),
                Transform::identity(),
                false,
            ),
            Combine::Add,
        );
        m = m.combine(
            &Mask::of_shape(
                20,
                20,
                &Shape::Rectangle([12.0, 12.0, 2.0, 2.0]),
                Transform::identity(),
                false,
            ),
            Combine::Subtract,
        );
        assert_eq!(m.outline().len(), 3);
    }

    #[test]
    fn selected_pixels_keep_the_shape() {
        let img = RgbaImage::from_pixel(40, 40, image::Rgba([200, 10, 10, 255]));
        let e = Mask::of_shape(
            40,
            40,
            &Shape::Ellipse([0.0, 0.0, 40.0, 40.0]),
            Transform::identity(),
            true,
        );
        let (px, at) = selected_pixels(&img, &e).unwrap();
        assert_eq!(at, [0, 0, 40, 40]);
        assert_eq!(px.get_pixel(0, 0).0[3], 0, "transparent corners");
        assert_eq!(px.get_pixel(20, 20).0, [200, 10, 10, 255]);
    }

    #[test]
    fn a_smart_fill_on_a_plain_background_leaves_no_mark() {
        // A button on a flat background, deleted.
        let mut img = RgbaImage::from_pixel(120, 80, image::Rgba([230, 235, 240, 255]));
        for y in 30..50 {
            for x in 40..80 {
                img.put_pixel(x, y, image::Rgba([20, 90, 200, 255]));
            }
        }
        let m = Mask::of_shape(
            120,
            80,
            &Shape::Rectangle([36.0, 26.0, 48.0, 28.0]),
            Transform::identity(),
            false,
        );
        fill_gap(&mut img, &m, Gap::Smart);
        for p in img.pixels() {
            for c in 0..3 {
                assert!((p.0[c] as i32 - [230, 235, 240][c]).abs() <= 1, "{:?}", p.0);
            }
        }
        // Transparent and a color.
        let mut t = RgbaImage::from_pixel(10, 10, image::Rgba([0, 0, 0, 255]));
        let all = Mask::full(10, 10);
        fill_gap(&mut t, &all, Gap::Transparent);
        assert_eq!(t.get_pixel(3, 3).0[3], 0);
        fill_gap(&mut t, &all, Gap::Color(Rgba::parse("#00FF00").unwrap()));
        assert_eq!(t.get_pixel(3, 3).0, [0, 255, 0, 255]);
    }

    #[test]
    fn a_loose_box_snaps_to_the_button() {
        let mut img = RgbaImage::from_pixel(200, 100, image::Rgba([245, 245, 245, 255]));
        // A button with its label, and a far-away mark.
        for y in 40..64 {
            for x in 60..140 {
                img.put_pixel(x, y, image::Rgba([30, 110, 220, 255]));
            }
        }
        for x in 80..120 {
            img.put_pixel(x, 52, image::Rgba([255, 255, 255, 255]));
        }
        img.put_pixel(10, 10, image::Rgba([0, 0, 0, 255]));
        assert_eq!(snap(&img, [40, 25, 120, 55], 50.0), Some([60, 40, 80, 24]));
        // Nothing but background: no snap.
        assert_eq!(snap(&img, [150, 70, 40, 25], 50.0), None);
    }

    #[test]
    fn a_gap_across_a_box_edge_stays_sharp() {
        // A blue box; the gap takes its right half and the white beyond.
        let mut img = RgbaImage::from_pixel(120, 80, image::Rgba([255, 255, 255, 255]));
        for y in 20..60 {
            for x in 20..60 {
                img.put_pixel(x, y, image::Rgba([20, 90, 200, 255]));
            }
        }
        let m = Mask::of_shape(
            120,
            80,
            &Shape::Rectangle([40.0, 10.0, 40.0, 60.0]),
            Transform::identity(),
            false,
        );
        fill_gap(&mut img, &m, Gap::Smart);
        assert_eq!(
            img.get_pixel(50, 40).0,
            [255, 255, 255, 255],
            "white above and below"
        );
        assert_eq!(img.get_pixel(45, 15).0, [255, 255, 255, 255]);
        assert_eq!(
            img.get_pixel(30, 40).0,
            [20, 90, 200, 255],
            "the rest of the box stays"
        );
    }

    #[test]
    fn feathering_softens_the_edge() {
        let m = Mask::of_shape(
            60,
            20,
            &Shape::Rectangle([20.0, 0.0, 40.0, 20.0]),
            Transform::identity(),
            false,
        );
        let f = m.feathered(10.0);
        assert!(f.at(20, 10) > 60 && f.at(20, 10) < 200, "{}", f.at(20, 10));
        assert_eq!(f.at(50, 10), 255);
        assert_eq!(f.at(5, 10), 0);
    }
}
