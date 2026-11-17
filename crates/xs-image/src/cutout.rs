//! Cut out a strip and add space (EDT-052). A strip of the whole width (or the whole
//! height) goes and the two parts join: straight, or with a gap of the canvas color
//! (transparent pixels) whose edges are torn, wavy or zigzag; or a band of space is
//! added. Edges that reach into the gap are made of the removed strip's own pixels,
//! so the parts look continuous up to the edge.

use image::{Rgba, RgbaImage};

/// How the two parts join.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Join {
    Straight,
    Torn,
    Wavy,
    Zigzag,
}

impl Join {
    pub fn parse(s: &str) -> Option<Join> {
        Some(match s {
            "straight" => Join::Straight,
            "torn" => Join::Torn,
            "wavy" => Join::Wavy,
            "zigzag" => Join::Zigzag,
            _ => return None,
        })
    }
}

/// Which way the strip runs: `Across` takes rows (the whole width, so the height
/// changes); `Down` takes columns (the whole height, so the width changes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Strip {
    Across,
    Down,
}

fn dims(img: &RgbaImage, strip: Strip) -> (u32, u32) {
    // (along the cut axis, across it)
    match strip {
        Strip::Across => (img.height(), img.width()),
        Strip::Down => (img.width(), img.height()),
    }
}

fn get(img: &RgbaImage, strip: Strip, along: u32, across: u32) -> Rgba<u8> {
    match strip {
        Strip::Across => *img.get_pixel(across, along),
        Strip::Down => *img.get_pixel(along, across),
    }
}

fn new_image(strip: Strip, along: u32, across: u32) -> RgbaImage {
    match strip {
        Strip::Across => RgbaImage::new(across, along),
        Strip::Down => RgbaImage::new(along, across),
    }
}

fn put(img: &mut RgbaImage, strip: Strip, along: u32, across: u32, p: Rgba<u8>) {
    match strip {
        Strip::Across => img.put_pixel(across, along, p),
        Strip::Down => img.put_pixel(along, across, p),
    }
}

/// How much of the join band the cut takes: `band` along the axis, and for each
/// position across it where the gap starts and ends inside the band.
pub struct Layout {
    pub band: u32,
    edges: Vec<(u32, u32)>,
}

/// The join's shape for a strip `strip_len` long, `across` wide, with a gap of `gap`.
pub fn layout(join: Join, gap: u32, strip_len: u32, across: u32, seed: u64) -> Layout {
    if join == Join::Straight || gap == 0 {
        return Layout {
            band: 0,
            edges: Vec::new(),
        };
    }
    // How far an edge reaches into the gap: half the gap, and never more than the
    // strip can supply on each side.
    let reach = (gap / 2).min(strip_len / 2);
    let band = gap + reach;
    let wave = (4 * reach).max(24) as f64;
    let mut state = seed | 1;
    let mut rnd = move || {
        // A small LCG: the same cut always tears the same way.
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        ((state >> 33) as f64) / ((1u64 << 31) as f64)
    };
    let (mut top, mut bottom) = (reach as f64 / 2.0, reach as f64 / 2.0);
    let edges = (0..across)
        .map(|u| {
            let t = u as f64 / wave;
            let (start, end) = match join {
                Join::Wavy => {
                    let s = reach as f64 * (0.5 + 0.5 * (t * std::f64::consts::TAU).sin());
                    (s, s + gap as f64)
                }
                Join::Zigzag => {
                    let s = reach as f64 * (1.0 - ((t.fract() * 2.0) - 1.0).abs());
                    (s, s + gap as f64)
                }
                _ => {
                    // Torn: two rough edges that wander on their own.
                    let step = (reach as f64 / 3.0).max(1.0);
                    top = (top + (rnd() - 0.5) * step * 2.0).clamp(0.0, reach as f64);
                    bottom = (bottom + (rnd() - 0.5) * step * 2.0).clamp(0.0, reach as f64);
                    (top, band as f64 - bottom)
                }
            };
            (start.round() as u32, (end.round() as u32).min(band))
        })
        .collect();
    Layout { band, edges }
}

/// Removes `[from, to)` along the axis of `strip` and joins the parts with `join`
/// (EDT-052). Returns the new image; it is `to - from - band` shorter.
pub fn cut(img: &RgbaImage, strip: Strip, from: u32, to: u32, join: Join, gap: u32, seed: u64) -> RgbaImage {
    let (len, across) = dims(img, strip);
    let (from, to) = (from.min(len), to.min(len).max(from.min(len)));
    let lay = layout(join, gap, to - from, across, seed);
    let new_len = len - (to - from) + lay.band;
    let mut out = new_image(strip, new_len.max(1), across);
    for u in 0..across {
        let (start, end) = lay.edges.get(u as usize).copied().unwrap_or((0, 0));
        for v in 0..new_len {
            let src = if v < from {
                Some(v)
            } else if v < from + lay.band {
                let k = v - from;
                if k < start {
                    Some(from + k)
                } else if k < end {
                    None
                } else {
                    Some(to - (lay.band - k))
                }
            } else {
                Some(to + (v - from - lay.band))
            };
            if let Some(s) = src.filter(|s| *s < len) {
                put(&mut out, strip, v, u, get(img, strip, s, u));
            }
        }
    }
    out
}

/// Inserts `size` transparent pixels at `at` along the axis of `strip` (EDT-052): the
/// canvas color shows there.
pub fn add(img: &RgbaImage, strip: Strip, at: u32, size: u32) -> RgbaImage {
    let (len, across) = dims(img, strip);
    let at = at.min(len);
    let mut out = new_image(strip, len + size, across);
    for u in 0..across {
        for v in 0..len {
            let to = if v < at { v } else { v + size };
            put(&mut out, strip, to, u, get(img, strip, v, u));
        }
    }
    out
}

/// Where a position along the axis goes after cutting `[from, to)` with a band of
/// `band`: before the strip it stays; after it, it moves back; inside, to the cut.
pub fn moved(pos: f64, from: f64, to: f64, band: f64) -> f64 {
    if pos <= from {
        pos
    } else if pos >= to {
        pos - (to - from) + band
    } else {
        from
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stripes(w: u32, h: u32) -> RgbaImage {
        // Each row has its own red value, so where a row went is easy to see.
        RgbaImage::from_fn(w, h, |_, y| Rgba([(y % 256) as u8, 0, 0, 255]))
    }

    #[test]
    fn a_straight_cut_joins_the_parts() {
        let img = stripes(10, 1000);
        let out = cut(&img, Strip::Across, 300, 500, Join::Straight, 16, 1);
        assert_eq!(out.dimensions(), (10, 800), "200 px less");
        assert_eq!(out.get_pixel(5, 299).0[0], 43, "row 299 stays");
        assert_eq!(
            out.get_pixel(5, 300).0[0],
            (500 % 256) as u8,
            "row 500 follows it"
        );
    }

    #[test]
    fn a_wavy_cut_leaves_a_wavy_gap_of_its_size() {
        let img = stripes(200, 1000);
        let out = cut(&img, Strip::Across, 300, 500, Join::Wavy, 20, 1);
        // 200 px go; the band is the gap and the reach of the waves (20 + 10).
        assert_eq!(out.height(), 1000 - 200 + 30);
        // In each column a run of exactly 20 transparent pixels, starting at
        // different heights: the gap waves.
        let mut starts = std::collections::BTreeSet::new();
        for x in 0..200 {
            let gap: Vec<u32> = (300..330).filter(|&y| out.get_pixel(x, y).0[3] == 0).collect();
            assert_eq!(gap.len(), 20, "column {x}");
            starts.insert(gap[0]);
        }
        assert!(starts.len() > 3, "it waves");
    }

    #[test]
    fn vertical_strips_take_columns() {
        let img = RgbaImage::from_fn(100, 10, |x, _| Rgba([x as u8, 0, 0, 255]));
        let out = cut(&img, Strip::Down, 20, 40, Join::Straight, 0, 1);
        assert_eq!(out.dimensions(), (80, 10));
        assert_eq!(out.get_pixel(20, 5).0[0], 40);
        let torn = cut(&img, Strip::Down, 20, 60, Join::Torn, 16, 7);
        assert!(torn.pixels().any(|p| p.0[3] == 0), "a gap");
        assert_eq!(
            torn,
            cut(&img, Strip::Down, 20, 60, Join::Torn, 16, 7),
            "the same tear"
        );
    }

    #[test]
    fn adding_space_pushes_the_rest() {
        let img = RgbaImage::from_fn(100, 10, |x, _| Rgba([x as u8, 0, 0, 255]));
        let out = add(&img, Strip::Down, 50, 150);
        assert_eq!(out.dimensions(), (250, 10));
        assert_eq!(out.get_pixel(100, 5).0[3], 0, "the new space is transparent");
        assert_eq!(out.get_pixel(200, 5).0[0], 50, "column 50 moved 150 px");
    }

    #[test]
    fn positions_move_with_the_cut() {
        assert_eq!(moved(100.0, 300.0, 500.0, 0.0), 100.0);
        assert_eq!(moved(700.0, 300.0, 500.0, 0.0), 500.0);
        assert_eq!(moved(700.0, 300.0, 500.0, 30.0), 530.0);
    }
}
