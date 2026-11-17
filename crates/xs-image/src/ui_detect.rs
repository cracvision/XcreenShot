//! Interface elements in the pixels of a screenshot (SMT-019, SMT-018): buttons, fields,
//! icons and pictures, found with a classical method (edges grouped by distance), with
//! no model. Text is read apart (SMT-003) and left out here; its lines group into
//! blocks with [`text_blocks`]. Good for screenshots; a photo gives little of use.
//!
//! `detail` (1–10) sets how far apart two things may be and still be one element: with
//! little detail a row of buttons is one element; with a lot, each button is one.

use image::RgbaImage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElementShape {
    Rect,
    Rounded,
    Circle,
    /// Small and about square: an icon.
    Icon,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    /// `[x, y, w, h]` in the image's pixels.
    pub rect: [u32; 4],
    pub shape: ElementShape,
    /// The color most of it has.
    pub color: [u8; 3],
}

/// How far apart, in pixels, two things still join at this detail.
pub fn reach(detail: u8) -> u32 {
    match detail.clamp(1, 10) {
        1 => 14,
        2 => 11,
        3 => 9,
        4 => 7,
        5 => 5,
        6 => 4,
        7 => 3,
        8 => 2,
        _ => 1,
    }
}

fn px(img: &RgbaImage, x: u32, y: u32) -> [u8; 3] {
    let p = img.get_pixel(x, y).0;
    // Over white, as it shows.
    let a = p[3] as u32;
    let mix = |c: u8| ((c as u32 * a + 255 * (255 - a)) / 255) as u8;
    [mix(p[0]), mix(p[1]), mix(p[2])]
}

fn differ(a: [u8; 3], b: [u8; 3], by: i32) -> bool {
    (0..3).any(|i| (a[i] as i32 - b[i] as i32).abs() > by)
}

/// How much a channel changes at an edge: subtle enough for a light gray button on a
/// lighter background (screenshots have exact pixels).
const EDGE: i32 = 14;

/// Pixels where the color changes, both sides of the change: edges of things. A
/// group's box is one pixel larger than the thing on every side.
fn edges(img: &RgbaImage) -> Vec<bool> {
    let (w, h) = img.dimensions();
    let mut out = vec![false; (w * h) as usize];
    for y in 0..h {
        for x in 0..w {
            let c = px(img, x, y);
            if x + 1 < w && differ(c, px(img, x + 1, y), EDGE) {
                out[(y * w + x) as usize] = true;
                out[(y * w + x + 1) as usize] = true;
            }
            if y + 1 < h && differ(c, px(img, x, y + 1), EDGE) {
                out[(y * w + x) as usize] = true;
                out[((y + 1) * w + x) as usize] = true;
            }
        }
    }
    out
}

/// Grows the marked pixels by `r` in every direction (a square).
fn dilate(mask: &[bool], w: u32, h: u32, r: u32) -> Vec<bool> {
    let (w, h, r) = (w as usize, h as usize, r as usize);
    // Rows, then columns, with a running count of marked pixels in the window.
    let mut rows = vec![false; w * h];
    for y in 0..h {
        let line = &mask[y * w..(y + 1) * w];
        let mut count = 0usize;
        for x in 0..(w + r) {
            if x < w && line[x] {
                count += 1;
            }
            if x > 2 * r && line[x - 2 * r - 1] {
                count -= 1;
            }
            if x >= r && x - r < w {
                rows[y * w + x - r] = count > 0;
            }
        }
    }
    let mut out = vec![false; w * h];
    for x in 0..w {
        let mut count = 0usize;
        for y in 0..(h + r) {
            if y < h && rows[y * w + x] {
                count += 1;
            }
            if y > 2 * r && rows[(y - 2 * r - 1) * w + x] {
                count -= 1;
            }
            if y >= r && y - r < h {
                out[(y - r) * w + x] = count > 0;
            }
        }
    }
    out
}

/// The bounding boxes of the groups of marked pixels, `[x0, y0, x1, y1]` inclusive.
fn groups(mask: &[bool], w: u32, h: u32) -> Vec<[u32; 4]> {
    let (wu, hu) = (w as usize, h as usize);
    let mut seen = vec![false; wu * hu];
    let mut out = Vec::new();
    let mut stack: Vec<usize> = Vec::new();
    for start in 0..wu * hu {
        if !mask[start] || seen[start] {
            continue;
        }
        let mut b = [u32::MAX, u32::MAX, 0, 0];
        seen[start] = true;
        stack.push(start);
        while let Some(i) = stack.pop() {
            let (x, y) = ((i % wu) as u32, (i / wu) as u32);
            b = [b[0].min(x), b[1].min(y), b[2].max(x), b[3].max(y)];
            let mut visit = |j: usize| {
                if mask[j] && !seen[j] {
                    seen[j] = true;
                    stack.push(j);
                }
            };
            if x > 0 {
                visit(i - 1);
            }
            if x + 1 < w {
                visit(i + 1);
            }
            if y > 0 {
                visit(i - wu);
            }
            if y + 1 < h {
                visit(i + wu);
            }
        }
        out.push(b);
    }
    out
}

/// The color most of a rectangle has, by groups of near colors.
fn main_color(img: &RgbaImage, r: [u32; 4]) -> [u8; 3] {
    let [x, y, w, h] = r;
    let mut counts: std::collections::HashMap<[u8; 3], (u32, [u32; 3])> = Default::default();
    let step = ((w * h) / 4000).max(1);
    let mut i = 0u32;
    for yy in y..y + h {
        for xx in x..x + w {
            i += 1;
            if !i.is_multiple_of(step) {
                continue;
            }
            let c = px(img, xx, yy);
            let key = [c[0] >> 4, c[1] >> 4, c[2] >> 4];
            let e = counts.entry(key).or_insert((0, [0; 3]));
            e.0 += 1;
            for (sum, v) in e.1.iter_mut().zip(c) {
                *sum += v as u32;
            }
        }
    }
    counts
        .values()
        .max_by_key(|(n, _)| *n)
        .map(|(n, sum)| [(sum[0] / n) as u8, (sum[1] / n) as u8, (sum[2] / n) as u8])
        .unwrap_or([255, 255, 255])
}

/// What shape the element at `r` has: its corners tell a rounded box or a circle.
fn shape_of(img: &RgbaImage, r: [u32; 4]) -> ElementShape {
    let [x, y, w, h] = r;
    let (big, small) = (w.max(h), w.min(h));
    let squarish = small * 5 >= big * 4;
    // What is just outside, and the very corner pixels.
    let outside = |cx: u32, cy: u32| px(img, cx, cy);
    let corner_is_outside =
        |cx: u32, cy: u32, ox: u32, oy: u32| !differ(px(img, cx, cy), outside(ox, oy), 24);
    let (iw, ih) = img.dimensions();
    let (x1, y1) = (x + w - 1, y + h - 1);
    let ox0 = x.saturating_sub(2);
    let oy0 = y.saturating_sub(2);
    let ox1 = (x1 + 2).min(iw - 1);
    let oy1 = (y1 + 2).min(ih - 1);
    let corners = [
        corner_is_outside(x, y, ox0, oy0),
        corner_is_outside(x1, y, ox1, oy0),
        corner_is_outside(x, y1, ox0, oy1),
        corner_is_outside(x1, y1, ox1, oy1),
    ]
    .iter()
    .filter(|c| **c)
    .count();
    // A circle leaves its box's corners empty well inside: up to 0.146 of its side
    // along the diagonal; a rounded box, much less.
    let deep = (small * 12 / 100).max(1);
    let deep_corner =
        corner_is_outside(x + deep, y + deep, ox0, oy0) && corner_is_outside(x1 - deep, y1 - deep, ox1, oy1);
    if squarish && corners >= 3 && deep_corner && small >= 10 {
        return ElementShape::Circle;
    }
    if squarish && big <= 40 {
        return ElementShape::Icon;
    }
    if corners >= 3 {
        ElementShape::Rounded
    } else {
        ElementShape::Rect
    }
}

/// The interface elements of `img`, leaving out the `text` rectangles (`[x, y, w, h]`,
/// lines or words) and anything as big as a panel.
pub fn detect(img: &RgbaImage, text: &[[f64; 4]], detail: u8) -> Vec<Element> {
    let (w, h) = img.dimensions();
    if w < 4 || h < 4 {
        return Vec::new();
    }
    let mut mask = edges(img);
    // Text is simplified apart: its pixels do not make elements.
    for t in text {
        let x0 = (t[0] - 1.0).max(0.0) as u32;
        let y0 = (t[1] - 1.0).max(0.0) as u32;
        let x1 = ((t[0] + t[2] + 1.0).max(0.0) as u32).min(w);
        let y1 = ((t[1] + t[3] + 1.0).max(0.0) as u32).min(h);
        for y in y0..y1 {
            for x in x0..x1 {
                mask[(y * w + x) as usize] = false;
            }
        }
    }
    let r = reach(detail);
    let grown = dilate(&mask, w, h, r);
    let area = (w as u64) * (h as u64);
    let mut out = Vec::new();
    for b in groups(&grown, w, h) {
        // Back to the thing's own extent: less the growth and the edges' outer pixel.
        let x0 = (b[0] + r + 1).min(b[2]);
        let y0 = (b[1] + r + 1).min(b[3]);
        let x1 = b[2].saturating_sub(r + 1).max(x0);
        let y1 = b[3].saturating_sub(r + 1).max(y0);
        let (ew, eh) = (x1 - x0 + 1, y1 - y0 + 1);
        if ew < 6 || eh < 6 {
            continue;
        }
        // A panel or a window around the rest is not an element.
        if (ew as u64) * (eh as u64) * 4 > area {
            continue;
        }
        let rect = [x0, y0, ew, eh];
        let inner = if ew > 6 && eh > 6 {
            [x0 + 2, y0 + 2, ew - 4, eh - 4]
        } else {
            rect
        };
        out.push(Element {
            rect,
            shape: shape_of(img, rect),
            color: main_color(img, inner),
        });
    }
    out.sort_by_key(|e| (e.rect[1], e.rect[0]));
    out
}

/// A block of text lines that simplify as one: a line, or with little detail a
/// paragraph of lines close together and aligned on the left.
#[derive(Debug, Clone, PartialEq)]
pub struct TextBlock {
    /// `[x, y, w, h]`.
    pub rect: [f64; 4],
    pub lines: usize,
    /// The lines' height, on average.
    pub line_height: f64,
    /// From one line's top to the next's, times the line height.
    pub spacing: f64,
}

/// Groups text lines (`[x, y, w, h]`, top to bottom) into blocks: with detail 6 or more,
/// each line alone; with less, lines that follow each other closely and start at about
/// the same left edge join.
pub fn text_blocks(lines: &[[f64; 4]], detail: u8) -> Vec<TextBlock> {
    let mut sorted: Vec<[f64; 4]> = lines.to_vec();
    sorted.sort_by(|a, b| a[1].total_cmp(&b[1]).then(a[0].total_cmp(&b[0])));
    let alone = |l: [f64; 4]| TextBlock {
        rect: l,
        lines: 1,
        line_height: l[3],
        spacing: 1.8,
    };
    if detail >= 6 {
        return sorted.into_iter().map(alone).collect();
    }
    let mut groups: Vec<Vec<[f64; 4]>> = Vec::new();
    for l in sorted {
        let joined = groups.iter_mut().rev().take(3).find(|g| {
            let last = g[g.len() - 1];
            let gap = l[1] - (last[1] + last[3]);
            gap >= -2.0 && gap <= last[3] * 1.2 && (l[0] - g[0][0]).abs() <= last[3] * 1.5
        });
        match joined {
            Some(g) => g.push(l),
            None => groups.push(vec![l]),
        }
    }
    groups
        .into_iter()
        .map(|g| {
            if g.len() == 1 {
                return alone(g[0]);
            }
            let x0 = g.iter().map(|l| l[0]).fold(f64::MAX, f64::min);
            let y0 = g.iter().map(|l| l[1]).fold(f64::MAX, f64::min);
            let x1 = g.iter().map(|l| l[0] + l[2]).fold(f64::MIN, f64::max);
            let y1 = g.iter().map(|l| l[1] + l[3]).fold(f64::MIN, f64::max);
            let height = g.iter().map(|l| l[3]).sum::<f64>() / g.len() as f64;
            let pitch = (g[g.len() - 1][1] - g[0][1]) / (g.len() - 1) as f64;
            TextBlock {
                rect: [x0, y0, x1 - x0, y1 - y0],
                lines: g.len(),
                line_height: height,
                spacing: (pitch / height.max(1.0)).clamp(1.0, 5.0),
            }
        })
        .collect()
}

/// The palette color nearest to `c`.
pub fn nearest(c: [u8; 3], palette: &[String]) -> Option<&String> {
    palette.iter().min_by_key(|p| {
        let q = crate::color::Rgba::parse(p)
            .map(|q| [q.r, q.g, q.b])
            .unwrap_or([0, 0, 0]);
        (0..3)
            .map(|i| {
                let d = c[i] as i32 - q[i] as i32;
                (d * d) as u32
            })
            .sum::<u32>()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn canvas(w: u32, h: u32) -> RgbaImage {
        RgbaImage::from_pixel(w, h, Rgba([245, 246, 248, 255]))
    }

    fn fill(img: &mut RgbaImage, r: [u32; 4], c: [u8; 3]) {
        for y in r[1]..r[1] + r[3] {
            for x in r[0]..r[0] + r[2] {
                img.put_pixel(x, y, Rgba([c[0], c[1], c[2], 255]));
            }
        }
    }

    fn button(img: &mut RgbaImage, x: u32, y: u32) {
        fill(img, [x, y, 90, 32], [60, 120, 220]);
    }

    #[test]
    fn three_buttons_by_detail() {
        let mut img = canvas(500, 200);
        for i in 0..3 {
            button(&mut img, 40 + i * 100, 80);
        }
        // Gaps of 10 px: with little detail one element, with a lot three.
        let low = detect(&img, &[], 1);
        assert_eq!(low.len(), 1, "{low:?}");
        assert_eq!(low[0].rect, [40, 80, 290, 32]);
        let high = detect(&img, &[], 10);
        assert_eq!(high.len(), 3, "{high:?}");
        assert!(high.iter().all(|e| e.color == [60, 120, 220]), "{high:?}");
        assert!(high.iter().all(|e| e.shape == ElementShape::Rect), "{high:?}");
    }

    #[test]
    fn shapes_and_text_left_out() {
        let mut img = canvas(400, 300);
        // A circle of 30 px radius.
        for y in 0..300u32 {
            for x in 0..400u32 {
                let (dx, dy) = (x as f64 - 100.0, y as f64 - 100.0);
                if dx * dx + dy * dy <= 900.0 {
                    img.put_pixel(x, y, Rgba([220, 60, 60, 255]));
                }
            }
        }
        // An icon.
        fill(&mut img, [250, 85, 24, 24], [40, 40, 40]);
        // "Text" that must not be an element.
        fill(&mut img, [40, 220, 200, 14], [20, 20, 20]);
        let found = detect(&img, &[[40.0, 220.0, 200.0, 14.0]], 10);
        let shapes: Vec<ElementShape> = found.iter().map(|e| e.shape).collect();
        assert_eq!(shapes, [ElementShape::Circle, ElementShape::Icon], "{found:?}");
    }

    #[test]
    fn a_panel_is_not_an_element() {
        let mut img = canvas(300, 300);
        // A big bordered panel with one button inside.
        for i in 10..290 {
            for (x, y) in [(i, 10), (i, 289), (10, i), (289, i)] {
                img.put_pixel(x, y, Rgba([150, 150, 150, 255]));
            }
        }
        button(&mut img, 100, 130);
        let found = detect(&img, &[], 8);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_eq!(found[0].rect, [100, 130, 90, 32]);
    }

    #[test]
    fn lines_join_with_little_detail() {
        let lines = [
            [20.0, 10.0, 300.0, 14.0],
            [20.0, 30.0, 280.0, 14.0],
            [20.0, 50.0, 120.0, 14.0],
            [20.0, 120.0, 200.0, 14.0],
        ];
        let low = text_blocks(&lines, 3);
        assert_eq!(low.len(), 2, "{low:?}");
        assert_eq!((low[0].lines, low[0].rect), (3, [20.0, 10.0, 300.0, 54.0]));
        assert!((low[0].spacing - 20.0 / 14.0).abs() < 1e-9);
        assert_eq!(text_blocks(&lines, 8).len(), 4);
    }

    #[test]
    fn the_nearest_palette_color() {
        let palette: Vec<String> = ["#D9DDE3", "#5B6472", "#2D7FF9"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(nearest([20, 20, 20], &palette).unwrap(), "#5B6472");
        assert_eq!(nearest([60, 120, 220], &palette).unwrap(), "#2D7FF9");
    }
}
