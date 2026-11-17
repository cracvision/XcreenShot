//! Smart fill, Photo quality (EDT-045): rebuilds a gap with pieces of the picture
//! around it, so textures and photos continue through it. Coarse to fine: at each
//! size, every patch over the gap finds the most alike patch outside it (randomized
//! search that spreads good matches to neighbors), and the gap's pixels become the
//! mean of the patches that cover them.

use image::RgbaImage;

use crate::selection::Mask;

/// Half the side of a patch: patches are 7 × 7.
const R: i64 = 3;

/// A picture as rows of colors, with what is in the gap.
#[derive(Clone)]
struct Level {
    w: i64,
    h: i64,
    px: Vec<[f32; 4]>,
    hole: Vec<bool>,
}

impl Level {
    fn at(&self, x: i64, y: i64) -> usize {
        (y * self.w + x) as usize
    }

    /// Half the size: known pixels are the mean of the known ones below; a pixel
    /// with any of them in the gap is in the gap.
    fn half(&self) -> Level {
        let (w, h) = ((self.w + 1) / 2, (self.h + 1) / 2);
        let mut px = vec![[0f32; 4]; (w * h) as usize];
        let mut hole = vec![false; (w * h) as usize];
        for y in 0..h {
            for x in 0..w {
                let mut sum = [0f32; 4];
                let mut n = 0.0;
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    let (sx, sy) = (2 * x + dx, 2 * y + dy);
                    if sx >= self.w || sy >= self.h {
                        continue;
                    }
                    let i = self.at(sx, sy);
                    if self.hole[i] {
                        hole[(y * w + x) as usize] = true;
                    } else {
                        for (acc, v) in sum.iter_mut().zip(self.px[i]) {
                            *acc += v;
                        }
                        n += 1.0;
                    }
                }
                if n > 0.0 {
                    px[(y * w + x) as usize] = sum.map(|v| v / n);
                }
            }
        }
        Level { w, h, px, hole }
    }
}

/// Fills the selected place of `img` from the pixels around it. `go_on` hears the
/// progress (0–1) and says whether to continue: `false` stops and leaves `img` as it
/// was. Returns whether it finished.
pub fn inpaint(img: &mut RgbaImage, mask: &Mask, go_on: &mut dyn FnMut(f32) -> bool) -> bool {
    let Some(b) = mask.bounds() else {
        return true;
    };
    // A window around the gap: what the patches can come from.
    let margin = b[2].max(b[3]).max(48) as i64;
    let x0 = (b[0] as i64 - margin).max(0);
    let y0 = (b[1] as i64 - margin).max(0);
    let x1 = (b[0] as i64 + b[2] as i64 + margin).min(img.width() as i64);
    let y1 = (b[1] as i64 + b[3] as i64 + margin).min(img.height() as i64);
    let (w, h) = (x1 - x0, y1 - y0);
    let mut fine = Level {
        w,
        h,
        px: Vec::with_capacity((w * h) as usize),
        hole: Vec::with_capacity((w * h) as usize),
    };
    for y in y0..y1 {
        for x in x0..x1 {
            let p = img.get_pixel(x as u32, y as u32).0;
            fine.px.push(p.map(|c| c as f32));
            fine.hole.push(mask.at(x as u32, y as u32) > 0);
        }
    }
    // Sizes, finest first, down to about four patches across.
    let mut levels = vec![fine];
    while levels.len() < 7 {
        let last = levels.last().expect("a level");
        if last.w.min(last.h) < 8 * (2 * R + 1) {
            break;
        }
        levels.push(last.half());
    }
    // The work at each size, for the progress.
    let work: Vec<f32> = levels
        .iter()
        .map(|l| l.hole.iter().filter(|&&x| x).count() as f32)
        .collect();
    let total: f32 = work
        .iter()
        .map(|w| w * iterations(0) as f32)
        .sum::<f32>()
        .max(1.0);
    let mut done = 0.0;
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);

    // The coarsest size starts smooth.
    let coarsest = levels.len() - 1;
    {
        let l = &mut levels[coarsest];
        let known: Vec<f32> = l.hole.iter().map(|&x| if x { 0.0 } else { 1.0 }).collect();
        l.px = crate::selection::membrane(&l.px, &known, l.w as usize, l.h as usize);
    }
    let mut nnf: Option<(Vec<(i64, i64)>, i64)> = None;
    for li in (0..levels.len()).rev() {
        // From the coarser size: the gap's pixels start from its result.
        if li < coarsest {
            let (coarse, rest) = levels.split_at_mut(li + 1);
            let (l, c) = (&mut coarse[li], &rest[0]);
            for y in 0..l.h {
                for x in 0..l.w {
                    let i = l.at(x, y);
                    if l.hole[i] {
                        l.px[i] = c.px[c.at((x / 2).min(c.w - 1), (y / 2).min(c.h - 1))];
                    }
                }
            }
        }
        let l = &mut levels[li];
        let sources = sources_of(l);
        if sources.is_empty() {
            continue;
        }
        // Where each patch over the gap takes its pixels from.
        let mut map: Vec<(i64, i64)> = (0..l.w * l.h)
            .map(|i| {
                let (x, y) = (i % l.w, i / l.w);
                let from = nnf.as_ref().and_then(|(m, cw)| {
                    let (sx, sy) = m[((y / 2) * cw + (x / 2)) as usize];
                    let s = (sx * 2 + (x % 2), sy * 2 + (y % 2));
                    valid_source(l, s.0, s.1).then_some(s)
                });
                from.unwrap_or_else(|| sources[rng.below(sources.len())])
            })
            .collect();
        let targets = targets_of(l);
        for it in 0..iterations(li) {
            search(l, &targets, &mut map, &sources, &mut rng, it);
            vote(l, &targets, &map);
            done += work[li];
            if !go_on((done / total).min(0.99)) {
                return false;
            }
        }
        nnf = Some((map, l.w));
    }
    // The result, where the mask says and as much as it says.
    let l = &levels[0];
    for y in 0..h {
        for x in 0..w {
            let i = l.at(x, y);
            if !l.hole[i] {
                continue;
            }
            let (ix, iy) = ((x0 + x) as u32, (y0 + y) as u32);
            let k = mask.at(ix, iy) as f32 / 255.0;
            let p = img.get_pixel_mut(ix, iy);
            for c in 0..4 {
                p.0[c] = (p.0[c] as f32 * (1.0 - k) + l.px[i][c].clamp(0.0, 255.0) * k).round() as u8;
            }
        }
    }
    go_on(1.0);
    true
}

/// Rounds of search and mean at a size: more at the coarse ones, where they are cheap.
fn iterations(level: usize) -> usize {
    if level == 0 {
        3
    } else {
        5
    }
}

/// Whether the patch centered at `x, y` is whole and outside the gap.
fn valid_source(l: &Level, x: i64, y: i64) -> bool {
    if x < R || y < R || x >= l.w - R || y >= l.h - R {
        return false;
    }
    (-R..=R).all(|dy| (-R..=R).all(|dx| !l.hole[l.at(x + dx, y + dy)]))
}

fn sources_of(l: &Level) -> Vec<(i64, i64)> {
    let mut out = Vec::new();
    for y in R..l.h - R {
        for x in R..l.w - R {
            if valid_source(l, x, y) {
                out.push((x, y));
            }
        }
    }
    out
}

/// The patch centers whose patch touches the gap.
fn targets_of(l: &Level) -> Vec<(i64, i64)> {
    let mut near = vec![false; l.hole.len()];
    for y in 0..l.h {
        for x in 0..l.w {
            if !l.hole[l.at(x, y)] {
                continue;
            }
            for ty in (y - R).max(0)..=(y + R).min(l.h - 1) {
                for tx in (x - R).max(0)..=(x + R).min(l.w - 1) {
                    near[l.at(tx, ty)] = true;
                }
            }
        }
    }
    (0..l.h)
        .flat_map(|y| (0..l.w).map(move |x| (x, y)))
        .filter(|&(x, y)| near[(y * l.w + x) as usize])
        .collect()
}

/// How unlike the patch at `t` is to the source patch at `s`; stops counting past
/// `best`.
fn distance(l: &Level, t: (i64, i64), s: (i64, i64), best: f32) -> f32 {
    let mut d = 0.0;
    for dy in -R..=R {
        let (ty, sy) = (t.1 + dy, s.1 + dy);
        if ty < 0 || ty >= l.h {
            continue;
        }
        for dx in -R..=R {
            let tx = t.0 + dx;
            if tx < 0 || tx >= l.w {
                continue;
            }
            let (a, b) = (l.px[l.at(tx, ty)], l.px[l.at(s.0 + dx, sy)]);
            for k in 0..4 {
                let e = a[k] - b[k];
                d += e * e;
            }
        }
        if d > best {
            return d;
        }
    }
    d
}

/// A round of the randomized search: each patch tries its neighbors' matches moved by
/// one, then random places ever closer to its own.
fn search(
    l: &Level,
    targets: &[(i64, i64)],
    map: &mut [(i64, i64)],
    sources: &[(i64, i64)],
    rng: &mut Rng,
    round: usize,
) {
    let forward = round.is_multiple_of(2);
    let step: i64 = if forward { 1 } else { -1 };
    let order: Box<dyn Iterator<Item = &(i64, i64)>> = if forward {
        Box::new(targets.iter())
    } else {
        Box::new(targets.iter().rev())
    };
    for &t in order {
        let ti = l.at(t.0, t.1);
        let mut best = map[ti];
        if !valid_source(l, best.0, best.1) {
            best = sources[rng.below(sources.len())];
        }
        let mut best_d = distance(l, t, best, f32::INFINITY);
        // From the neighbors already seen this round.
        for (nx, ny) in [(t.0 - step, t.1), (t.0, t.1 - step)] {
            if nx < 0 || ny < 0 || nx >= l.w || ny >= l.h {
                continue;
            }
            let (sx, sy) = map[l.at(nx, ny)];
            let cand = if ny == t.1 {
                (sx + step, sy)
            } else {
                (sx, sy + step)
            };
            if valid_source(l, cand.0, cand.1) {
                let d = distance(l, t, cand, best_d);
                if d < best_d {
                    best = cand;
                    best_d = d;
                }
            }
        }
        // Random tries around the best, closer each time.
        let mut radius = l.w.max(l.h);
        while radius >= 1 {
            let cand = (
                best.0 + rng.between(-radius, radius),
                best.1 + rng.between(-radius, radius),
            );
            if valid_source(l, cand.0, cand.1) {
                let d = distance(l, t, cand, best_d);
                if d < best_d {
                    best = cand;
                    best_d = d;
                }
            }
            radius /= 2;
        }
        map[ti] = best;
    }
}

/// Each pixel of the gap becomes the mean of what the patches covering it say.
fn vote(l: &mut Level, targets: &[(i64, i64)], map: &[(i64, i64)]) {
    let mut sum = vec![[0f32; 4]; l.px.len()];
    let mut weight = vec![0f32; l.px.len()];
    for &t in targets {
        let s = map[l.at(t.0, t.1)];
        for dy in -R..=R {
            let y = t.1 + dy;
            if y < 0 || y >= l.h {
                continue;
            }
            for dx in -R..=R {
                let x = t.0 + dx;
                if x < 0 || x >= l.w {
                    continue;
                }
                let i = l.at(x, y);
                if !l.hole[i] {
                    continue;
                }
                let c = l.px[l.at(s.0 + dx, s.1 + dy)];
                for k in 0..4 {
                    sum[i][k] += c[k];
                }
                weight[i] += 1.0;
            }
        }
    }
    for i in 0..l.px.len() {
        if l.hole[i] && weight[i] > 0.0 {
            l.px[i] = sum[i].map(|v| v / weight[i]);
        }
    }
}

/// A small, fixed random sequence: the same gap fills the same way every time.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }

    fn between(&mut self, lo: i64, hi: i64) -> i64 {
        lo + (self.next() % (hi - lo + 1) as u64) as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selection::Shape;
    use tiny_skia::Transform;

    /// Stripes, a texture a smooth fill would blur: Photo continues them.
    #[test]
    fn stripes_continue_through_the_gap() {
        let mut img = RgbaImage::from_fn(96, 96, |x, _| {
            if (x / 4) % 2 == 0 {
                image::Rgba([240, 240, 240, 255])
            } else {
                image::Rgba([30, 30, 30, 255])
            }
        });
        let before = img.clone();
        let m = Mask::of_shape(
            96,
            96,
            &Shape::Rectangle([36.0, 36.0, 24.0, 24.0]),
            Transform::identity(),
            false,
        );
        // Paint the gap gray first: the fill must not rely on what was there.
        for y in 36..60 {
            for x in 36..60 {
                img.put_pixel(x, y, image::Rgba([128, 128, 128, 255]));
            }
        }
        assert!(inpaint(&mut img, &m, &mut |_| true));
        // Each pixel of the gap is light or dark again, not gray.
        let mut gray = 0;
        let mut right = 0;
        for y in 36..60 {
            for x in 36..60 {
                let v = img.get_pixel(x, y).0[0] as i32;
                if (60..200).contains(&v) {
                    gray += 1;
                }
                if (v - before.get_pixel(x, y).0[0] as i32).abs() < 60 {
                    right += 1;
                }
            }
        }
        assert!(gray < 24 * 24 / 10, "{gray} gray pixels");
        assert!(right > 24 * 24 * 7 / 10, "{right} pixels follow the stripes");
    }

    #[test]
    fn it_can_be_stopped_and_leaves_the_image_as_it_was() {
        let mut img = RgbaImage::from_pixel(80, 80, image::Rgba([10, 200, 10, 255]));
        let before = img.clone();
        let m = Mask::of_shape(
            80,
            80,
            &Shape::Rectangle([30.0, 30.0, 20.0, 20.0]),
            Transform::identity(),
            false,
        );
        assert!(!inpaint(&mut img, &m, &mut |_| false));
        assert_eq!(img, before);
    }
}
