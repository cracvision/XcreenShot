//! Remove background (SMT-020), the classical way (decided by Carlos on 2026-11-14; a
//! local segmentation model comes later): the background is what reaches the image's
//! edge and looks like the colors that fill that edge, plain or a gradient. From the
//! edge inwards, a pixel is background while it changes little from its neighbour and
//! stays near those colors; the rest is the subject. Its outline is softened. Enclosed
//! areas of the background color (the inside of an "O") stay: the magic wand
//! (EDT-046) takes them out.

use std::collections::VecDeque;

use image::RgbaImage;

/// How much a pixel may differ from the background pixel beside it (RGB distance).
const LOCAL: f32 = 26.0;
/// How far a background pixel may be from the colors of the edge.
const GLOBAL: f32 = 90.0;
/// A border pixel starts the background when this near one of the edge's colors.
const SEED: f32 = 40.0;
/// The edge's colors: the most common until this much of it is covered (a subject
/// that reaches the edge covers less, and its colors stay out).
const COVER: f32 = 0.7;
/// Less background than this, or more, is no clear subject.
const LEAST: f64 = 0.02;
const MOST: f64 = 0.98;

fn rgb(p: &image::Rgba<u8>) -> [f32; 3] {
    [p[0] as f32, p[1] as f32, p[2] as f32]
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

/// Transparent pixels are background whatever their color.
fn clear(p: &image::Rgba<u8>) -> bool {
    p[3] < 16
}

/// The colors that fill the image's edge: the average of each common group of them.
fn edge_colors(img: &RgbaImage) -> Vec<[f32; 3]> {
    let (w, h) = img.dimensions();
    let mut groups: std::collections::HashMap<(u8, u8, u8), (u32, [f32; 3])> = Default::default();
    let mut total = 0u32;
    let mut add = |x: u32, y: u32| {
        let p = img.get_pixel(x, y);
        if clear(p) {
            return;
        }
        total += 1;
        let c = rgb(p);
        let g = groups
            .entry((p[0] >> 4, p[1] >> 4, p[2] >> 4))
            .or_insert((0, [0.0; 3]));
        g.0 += 1;
        for (sum, v) in g.1.iter_mut().zip(c) {
            *sum += v;
        }
    };
    for x in 0..w {
        add(x, 0);
        add(x, h - 1);
    }
    for y in 1..h.saturating_sub(1) {
        add(0, y);
        add(w - 1, y);
    }
    if total == 0 {
        return Vec::new();
    }
    let mut list: Vec<(u32, [f32; 3])> = groups
        .into_values()
        .map(|(n, s)| (n, [s[0] / n as f32, s[1] / n as f32, s[2] / n as f32]))
        .collect();
    list.sort_by_key(|g| std::cmp::Reverse(g.0));
    let mut covered = 0u32;
    let mut out = Vec::new();
    // A gradient spreads over many small groups: they count until enough is covered.
    for (n, c) in list {
        if covered as f32 >= COVER * total as f32 {
            break;
        }
        out.push(c);
        covered += n;
    }
    out
}

/// Whether the image's edge changes smoothly around it, as a plain or gradient
/// background does (most neighbours alike), not as a busy photo or noise.
fn smooth_edge(img: &RgbaImage) -> bool {
    let (w, h) = img.dimensions();
    let mut ring: Vec<(u32, u32)> = (0..w).map(|x| (x, 0)).collect();
    ring.extend((1..h).map(|y| (w - 1, y)));
    ring.extend((0..w - 1).rev().map(|x| (x, h - 1)));
    ring.extend((1..h - 1).rev().map(|y| (0, y)));
    let alike = ring
        .windows(2)
        .filter(|p| {
            let (a, b) = (img.get_pixel(p[0].0, p[0].1), img.get_pixel(p[1].0, p[1].1));
            (clear(a) && clear(b)) || distance(rgb(a), rgb(b)) <= LOCAL
        })
        .count();
    alike as f32 >= 0.6 * (ring.len() - 1) as f32
}

/// The subject's opacity (0 to 255, a byte per pixel, rows from the top) when the
/// image has a clear one on a plain or gradient background; none when it has not.
pub fn subject(img: &RgbaImage) -> Option<Vec<u8>> {
    let (w, h) = img.dimensions();
    if w < 8 || h < 8 {
        return None;
    }
    if !smooth_edge(img) {
        return None;
    }
    let colors = edge_colors(img);
    let near_edge = |c: [f32; 3], limit: f32| colors.iter().any(|e| distance(c, *e) <= limit);
    let (wu, hu) = (w as usize, h as usize);
    let mut background = vec![false; wu * hu];
    let mut queue = VecDeque::new();
    let seed = |x: u32, y: u32, background: &mut Vec<bool>, queue: &mut VecDeque<(u32, u32)>| {
        let i = y as usize * wu + x as usize;
        let p = img.get_pixel(x, y);
        if !background[i] && (clear(p) || near_edge(rgb(p), SEED)) {
            background[i] = true;
            queue.push_back((x, y));
        }
    };
    for x in 0..w {
        seed(x, 0, &mut background, &mut queue);
        seed(x, h - 1, &mut background, &mut queue);
    }
    for y in 1..h - 1 {
        seed(0, y, &mut background, &mut queue);
        seed(w - 1, y, &mut background, &mut queue);
    }
    while let Some((x, y)) = queue.pop_front() {
        let here = img.get_pixel(x, y);
        let from = rgb(here);
        let from_clear = clear(here);
        let around = [
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
        ];
        for (nx, ny) in around {
            if nx >= w || ny >= h {
                continue;
            }
            let i = ny as usize * wu + nx as usize;
            if background[i] {
                continue;
            }
            let p = img.get_pixel(nx, ny);
            let c = rgb(p);
            let goes = clear(p) || ((from_clear || distance(c, from) <= LOCAL) && near_edge(c, GLOBAL));
            if goes {
                background[i] = true;
                queue.push_back((nx, ny));
            }
        }
    }
    let share = background.iter().filter(|b| **b).count() as f64 / (wu * hu) as f64;
    if !(LEAST..=MOST).contains(&share) {
        return None;
    }
    // Opaque subject, clear background; the subject's outline half to fully opaque by
    // how much of the subject is around it, so it has no hard edge.
    let mut alpha = vec![0u8; wu * hu];
    for y in 0..hu {
        for x in 0..wu {
            let i = y * wu + x;
            if background[i] {
                continue;
            }
            let mut inside = 0u32;
            let mut edge = false;
            for dy in -1i32..=1 {
                for dx in -1i32..=1 {
                    let (nx, ny) = (x as i32 + dx, y as i32 + dy);
                    if nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32 {
                        inside += 1;
                        continue;
                    }
                    if background[ny as usize * wu + nx as usize] {
                        edge = true;
                    } else {
                        inside += 1;
                    }
                }
            }
            alpha[i] = if edge {
                (128 + 127 * (inside.saturating_sub(1)) / 8) as u8
            } else {
                255
            };
        }
    }
    Some(alpha)
}

/// `img` with `alpha` (from [`subject`]) applied: the subject kept, or with `invert`
/// the background kept and the subject removed.
pub fn apply(img: &RgbaImage, alpha: &[u8], invert: bool) -> RgbaImage {
    let mut out = img.clone();
    for (p, a) in out.pixels_mut().zip(alpha) {
        let keep = if invert { 255 - *a } else { *a };
        p[3] = ((p[3] as u32 * keep as u32) / 255) as u8;
    }
    out
}

/// The smallest rectangle (x, y, width, height) with something visible in `img`.
pub fn visible_bounds(img: &RgbaImage) -> Option<[u32; 4]> {
    let (w, h) = img.dimensions();
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
    for (x, y, p) in img.enumerate_pixels() {
        if p[3] > 0 {
            x0 = x0.min(x);
            y0 = y0.min(y);
            x1 = x1.max(x);
            y1 = y1.max(y);
        }
    }
    (x1 >= x0 && y1 >= y0 && x0 < w).then(|| [x0, y0, x1 - x0 + 1, y1 - y0 + 1])
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn alpha_at(a: &[u8], w: u32, x: u32, y: u32) -> u8 {
        a[(y * w + x) as usize]
    }

    /// A red disc on white: the white goes, the disc stays, its outline is soft.
    #[test]
    fn a_disc_on_white() {
        let (w, h) = (120u32, 90u32);
        let img = RgbaImage::from_fn(w, h, |x, y| {
            let (dx, dy) = (x as f32 - 60.0, y as f32 - 45.0);
            if dx * dx + dy * dy <= 30.0 * 30.0 {
                Rgba([210, 40, 40, 255])
            } else {
                Rgba([255, 255, 255, 255])
            }
        });
        let a = subject(&img).expect("a subject");
        assert_eq!(alpha_at(&a, w, 0, 0), 0);
        assert_eq!(alpha_at(&a, w, 10, 45), 0);
        assert_eq!(alpha_at(&a, w, 60, 45), 255);
        // The outline is neither clear nor fully opaque.
        let edge = alpha_at(&a, w, 30, 45);
        assert!((128..255).contains(&edge), "{edge}");
        let out = apply(&img, &a, false);
        assert_eq!(out.get_pixel(5, 5)[3], 0);
        assert_eq!(out.get_pixel(60, 45)[3], 255);
        // Inverted: the white stays, the disc goes.
        let inv = apply(&img, &a, true);
        assert_eq!(inv.get_pixel(5, 5)[3], 255);
        assert_eq!(inv.get_pixel(60, 45)[3], 0);
        let b = visible_bounds(&out).unwrap();
        assert!(b[0] >= 28 && b[0] <= 31 && b[2] >= 59 && b[2] <= 63, "{b:?}");
    }

    /// A blue square on a grey gradient from top to bottom: the gradient goes.
    #[test]
    fn a_square_on_a_gradient() {
        let (w, h) = (100u32, 100u32);
        let img = RgbaImage::from_fn(w, h, |x, y| {
            if (35..65).contains(&x) && (35..65).contains(&y) {
                Rgba([30, 60, 200, 255])
            } else {
                let v = (230 - y * 120 / h) as u8;
                Rgba([v, v, v, 255])
            }
        });
        let a = subject(&img).expect("a subject");
        assert_eq!(alpha_at(&a, w, 50, 5), 0);
        assert_eq!(alpha_at(&a, w, 50, 95), 0);
        assert_eq!(alpha_at(&a, w, 50, 50), 255);
        let opaque = a.iter().filter(|v| **v > 0).count();
        assert!((900..=1000).contains(&opaque), "{opaque}");
    }

    /// A subject cut by the bottom edge is still the subject.
    #[test]
    fn a_subject_touching_the_edge() {
        let (w, h) = (100u32, 80u32);
        let img = RgbaImage::from_fn(w, h, |x, y| {
            if (40..60).contains(&x) && y >= 30 {
                Rgba([20, 120, 40, 255])
            } else {
                Rgba([245, 240, 230, 255])
            }
        });
        let a = subject(&img).expect("a subject");
        assert_eq!(alpha_at(&a, w, 50, 79), 255);
        assert_eq!(alpha_at(&a, w, 50, 50), 255);
        assert_eq!(alpha_at(&a, w, 10, 79), 0);
    }

    /// No background to find: noise everywhere, or nothing but the background.
    #[test]
    fn no_clear_subject() {
        let mut seed = 7u32;
        let noise = RgbaImage::from_fn(80, 60, |_, _| {
            seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            let v = seed.to_le_bytes();
            Rgba([v[1], v[2], v[3], 255])
        });
        assert!(subject(&noise).is_none());
        let plain = RgbaImage::from_pixel(80, 60, Rgba([250, 250, 250, 255]));
        assert!(subject(&plain).is_none());
        assert!(subject(&RgbaImage::new(4, 4)).is_none());
    }

    /// The inside of a ring is enclosed: it stays with the ring.
    #[test]
    fn enclosed_background_stays() {
        let (w, h) = (100u32, 100u32);
        let img = RgbaImage::from_fn(w, h, |x, y| {
            let (dx, dy) = (x as f32 - 50.0, y as f32 - 50.0);
            let r = (dx * dx + dy * dy).sqrt();
            if (20.0..30.0).contains(&r) {
                Rgba([0, 0, 0, 255])
            } else {
                Rgba([255, 255, 255, 255])
            }
        });
        let a = subject(&img).expect("a subject");
        assert_eq!(alpha_at(&a, w, 50, 50), 255);
        assert_eq!(alpha_at(&a, w, 2, 2), 0);
    }
}
