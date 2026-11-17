//! A minute of a typical interface demo, drawn in memory (VID-030): the reference
//! recording for the size requirement, with no screen and no input.
//!
//! What happens, at 30 frames per second: the pointer moves between places and
//! rests; text is typed (6–14 s); the page changes (20 s and 40 s); the page
//! scrolls (26–29 s); a menu opens under the pointer (46–50 s). Only frames that
//! change are produced, like the recorder does.

/// BGRA pixels of one frame, and what changes from frame to frame.
pub(crate) struct Demo {
    pub width: u32,
    pub height: u32,
    /// The frame (BGRA).
    pub px: Vec<u8>,
    /// Title bar and sidebar, redrawn when the hovered item changes.
    chrome: Vec<u8>,
    /// The page (content area), taller than the screen for scrolling.
    page: Vec<u8>,
    page_w: usize,
    page_h: usize,
    scroll: usize,
    hovered: Option<usize>,
    menu: Option<(usize, usize, usize)>,
    typed: usize,
    page_no: u32,
    pointer: (f64, f64),
    path: Vec<(f64, f64, f64, f64)>,
    glyphs: Vec<[u8; GLYPH_W * GLYPH_H]>,
}

const GLYPH_W: usize = 7;
const GLYPH_H: usize = 11;
const TITLE: usize = 40;
const SIDEBAR: usize = 280;
const ITEM: usize = 36;

/// A small deterministic generator, so every run draws the same minute.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u32 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.0 >> 33) as u32
    }
    fn below(&mut self, n: u32) -> u32 {
        self.next() % n.max(1)
    }
}

fn glyph_set() -> Vec<[u8; GLYPH_W * GLYPH_H]> {
    let mut rng = Rng(7);
    (0..48)
        .map(|_| {
            let mut g = [0u8; GLYPH_W * GLYPH_H];
            let mut put = |x: usize, y: usize, a: u8| {
                if x < GLYPH_W && y < GLYPH_H {
                    let v = &mut g[y * GLYPH_W + x];
                    *v = (*v).max(a);
                }
            };
            for _ in 0..2 + rng.below(3) {
                match rng.below(3) {
                    0 => {
                        let x = 1 + rng.below(5) as usize;
                        let (a, b) = (rng.below(5) as usize, 6 + rng.below(5) as usize);
                        for y in a..=b {
                            put(x, y, 255);
                            put(x + 1, y, 70);
                        }
                    }
                    1 => {
                        let y = 2 + rng.below(8) as usize;
                        for x in 1..6 {
                            put(x, y, 255);
                            put(x, y + 1, 60);
                        }
                    }
                    _ => {
                        for i in 0..6 {
                            put(1 + i, 9 - i, 220);
                            put(2 + i, 9 - i, 90);
                        }
                    }
                }
            }
            g
        })
        .collect()
}

fn fill(buf: &mut [u8], stride: usize, x: usize, y: usize, w: usize, h: usize, c: [u8; 3]) {
    let rows = buf.len() / (stride * 4);
    for row in y..(y + h).min(rows) {
        for col in x..(x + w).min(stride) {
            let i = (row * stride + col) * 4;
            buf[i..i + 4].copy_from_slice(&[c[2], c[1], c[0], 255]);
        }
    }
}

/// `n` glyphs of text from `seed` at (`x`, `y`), `scale` times their size.
#[allow(clippy::too_many_arguments)]
fn text(
    buf: &mut [u8],
    stride: usize,
    glyphs: &[[u8; GLYPH_W * GLYPH_H]],
    x: usize,
    y: usize,
    n: usize,
    seed: u64,
    scale: usize,
    c: [u8; 3],
) {
    let rows = buf.len() / (stride * 4);
    let mut rng = Rng(seed);
    let mut cx = x;
    for i in 0..n {
        // Words of 2 to 9 letters.
        if i > 0 && rng.below(6) == 0 {
            cx += 4 * scale;
            continue;
        }
        let g = &glyphs[rng.below(glyphs.len() as u32) as usize];
        for gy in 0..GLYPH_H * scale {
            for gx in 0..GLYPH_W * scale {
                let a = g[(gy / scale) * GLYPH_W + gx / scale] as u32;
                let (px, py) = (cx + gx, y + gy);
                if a == 0 || px >= stride || py >= rows {
                    continue;
                }
                let i = (py * stride + px) * 4;
                for (k, ch) in [c[2], c[1], c[0]].into_iter().enumerate() {
                    let bg = buf[i + k] as u32;
                    buf[i + k] = ((bg * (255 - a) + ch as u32 * a) / 255) as u8;
                }
            }
        }
        cx += (GLYPH_W + 1) * scale;
    }
}

impl Demo {
    pub fn new(width: u32, height: u32) -> Demo {
        let (w, h) = (width as usize, height as usize);
        let page_w = w - SIDEBAR;
        let page_h = h * 3;
        let mut rng = Rng(42);
        // Places the pointer goes to, each move taking `secs` and then resting.
        let mut path = Vec::new();
        let mut t = 0.0;
        while t < 61.0 {
            let (x, y) = (
                rng.below(width - 40) as f64 + 20.0,
                rng.below(height - 40) as f64 + 20.0,
            );
            let secs = 0.8 + rng.below(12) as f64 / 10.0;
            let rest = 0.3 + rng.below(10) as f64 / 10.0;
            path.push((t, x, y, secs));
            t += secs + rest;
        }
        let mut d = Demo {
            width,
            height,
            px: vec![0; w * h * 4],
            chrome: vec![0; w * h * 4],
            page: vec![0; page_w * page_h * 4],
            page_w,
            page_h,
            scroll: 0,
            hovered: None,
            menu: None,
            typed: 0,
            page_no: 0,
            pointer: (w as f64 / 2.0, h as f64 / 2.0),
            path,
            glyphs: glyph_set(),
        };
        d.draw_chrome();
        d.draw_page();
        d
    }

    fn draw_chrome(&mut self) {
        let (w, h) = (self.width as usize, self.height as usize);
        let g = self.glyphs.clone();
        let b = &mut self.chrome;
        fill(b, w, 0, 0, w, h, [255, 255, 255]);
        fill(b, w, 0, 0, w, TITLE, [32, 33, 40]);
        text(b, w, &g, 16, 14, 18, 1, 1, [230, 230, 235]);
        for (i, x) in [1700usize, 1760, 1820].iter().enumerate() {
            fill(b, w, *x, 12, 16, 16, [90 + i as u8 * 20, 92, 110]);
        }
        fill(b, w, 0, TITLE, SIDEBAR, h - TITLE, [236, 238, 242]);
        fill(b, w, SIDEBAR - 1, TITLE, 1, h - TITLE, [210, 212, 218]);
        for item in 0..20 {
            let y = TITLE + 16 + item * ITEM;
            if Some(item) == self.hovered {
                fill(b, w, 8, y - 6, SIDEBAR - 16, ITEM - 4, [214, 226, 250]);
            }
            fill(b, w, 20, y + 2, 14, 14, [120, 130, 150]);
            text(
                b,
                w,
                &g,
                44,
                y + 3,
                8 + (item * 7) % 12,
                100 + item as u64,
                1,
                [50, 52, 60],
            );
        }
    }

    fn draw_page(&mut self) {
        let (pw, ph) = (self.page_w, self.page_h);
        let g = self.glyphs.clone();
        let seed = 1000 * (self.page_no as u64 + 1);
        let mut rng = Rng(seed);
        let b = &mut self.page;
        fill(b, pw, 0, 0, pw, ph, [255, 255, 255]);
        let mut y = 32;
        let mut block = 0;
        while y + 400 < ph {
            // A heading, then paragraphs, a field, a chart or a picture.
            text(
                b,
                pw,
                &g,
                48,
                y,
                10 + rng.below(20) as usize,
                seed + y as u64,
                2,
                [24, 26, 32],
            );
            y += 48;
            match block % 4 {
                1 => {
                    // A bar chart.
                    fill(b, pw, 48, y, 900, 260, [248, 249, 251]);
                    for i in 0..12 {
                        let bh = 40 + rng.below(200) as usize;
                        let c = [[66, 120, 220], [40, 170, 150], [230, 150, 60]][i % 3];
                        fill(b, pw, 80 + i * 70, y + 250 - bh, 40, bh, c);
                    }
                    y += 290;
                }
                3 => {
                    // A picture: smooth colours with fine grain.
                    for py in 0..240 {
                        for px in 0..420.min(pw - 48) {
                            let i = ((y + py) * pw + 48 + px) * 4;
                            let n = (rng.below(24)) as i32 - 12;
                            let r = (90 + px as i32 / 4 + n).clamp(0, 255) as u8;
                            let gch = (140 + py as i32 / 3 + n).clamp(0, 255) as u8;
                            let bl = (200 - px as i32 / 5 + n).clamp(0, 255) as u8;
                            b[i..i + 4].copy_from_slice(&[bl, gch, r, 255]);
                        }
                    }
                    y += 270;
                }
                _ => {
                    for _ in 0..4 + rng.below(5) {
                        let n = 90 + rng.below(70) as usize;
                        text(b, pw, &g, 48, y, n, seed + y as u64 * 3, 1, [60, 62, 70]);
                        y += 24;
                    }
                    y += 24;
                }
            }
            block += 1;
        }
        // The field typed into.
        fill(b, pw, 48, 96, 900, 44, [180, 184, 192]);
        fill(b, pw, 49, 97, 898, 42, [255, 255, 255]);
        self.typed = 0;
    }

    fn compose(&mut self) {
        let (w, h) = (self.width as usize, self.height as usize);
        self.px.copy_from_slice(&self.chrome);
        let pw = self.page_w;
        for y in TITLE..h {
            let src = ((self.scroll + y - TITLE) * pw) * 4;
            let dst = (y * w + SIDEBAR) * 4;
            self.px[dst..dst + pw * 4].copy_from_slice(&self.page[src..src + pw * 4]);
        }
        if let Some((mx, my, sel)) = self.menu {
            let g = self.glyphs.clone();
            fill(&mut self.px, w, mx + 6, my + 6, 300, 392, [200, 202, 208]);
            fill(&mut self.px, w, mx, my, 300, 392, [250, 250, 252]);
            for i in 0..12 {
                if i == sel {
                    fill(&mut self.px, w, mx + 4, my + 6 + i * 32, 292, 30, [214, 226, 250]);
                }
                text(
                    &mut self.px,
                    w,
                    &g,
                    mx + 16,
                    my + 14 + i * 32,
                    6 + i % 9,
                    500 + i as u64,
                    1,
                    [40, 42, 50],
                );
            }
        }
        // The pointer: an arrow, black edge and white inside.
        let (px, py) = (self.pointer.0 as usize, self.pointer.1 as usize);
        for row in 0..19usize {
            let width = (row * 2 / 3).min(11);
            for col in 0..=width {
                let (x, y) = (px + col, py + row);
                if x >= w || y >= h {
                    continue;
                }
                let edge = col == 0 || col == width || row == 18;
                let c = if edge { 0 } else { 255 };
                let i = (y * w + x) * 4;
                self.px[i..i + 4].copy_from_slice(&[c, c, c, 255]);
            }
        }
    }

    /// Advances to frame `i`; false when nothing changed (no frame then).
    pub fn step(&mut self, i: u32, fps: u32) -> bool {
        let t = i as f64 / fps as f64;
        let mut changed = i == 0;
        // The pointer: eased between places, resting after each move.
        let at = self.path.iter().rposition(|p| p.0 <= t).unwrap_or(0);
        let (t0, x1, y1, secs) = self.path[at];
        let (x0, y0) = if at == 0 {
            (self.width as f64 / 2.0, self.height as f64 / 2.0)
        } else {
            (self.path[at - 1].1, self.path[at - 1].2)
        };
        let k = ((t - t0) / secs).clamp(0.0, 1.0);
        let k = k * k * (3.0 - 2.0 * k);
        let p = ((x0 + (x1 - x0) * k).round(), (y0 + (y1 - y0) * k).round());
        if p != self.pointer {
            self.pointer = p;
            changed = true;
        }
        // The sidebar item under the pointer.
        let hovered = (p.0 < SIDEBAR as f64 && p.1 > (TITLE + 10) as f64)
            .then(|| ((p.1 as usize - TITLE - 10) / ITEM).min(19));
        if hovered != self.hovered {
            self.hovered = hovered;
            self.draw_chrome();
            changed = true;
        }
        // Typing, about ten letters a second.
        if (6.0..14.0).contains(&t) && i.is_multiple_of(3) {
            let g = self.glyphs.clone();
            let x = 60 + self.typed * 8;
            if x < 900 {
                let pw = self.page_w;
                text(
                    &mut self.page,
                    pw,
                    &g,
                    x,
                    112,
                    1,
                    9000 + self.typed as u64,
                    1,
                    [20, 20, 28],
                );
                changed = true;
            }
            self.typed += 1;
        }
        // Another page.
        if i == 20 * fps || i == 40 * fps {
            self.page_no += 1;
            self.scroll = 0;
            self.draw_page();
            changed = true;
        }
        // Scrolling.
        if (26.0..29.0).contains(&t) {
            let max = self.page_h - (self.height as usize - TITLE);
            self.scroll = (self.scroll + 6).min(max);
            changed = true;
        }
        // A menu under the pointer, its choice following it.
        let menu = (46.0..50.0).contains(&t).then(|| {
            let (mx, my) = (
                (p.0 as usize).min((self.width as usize).saturating_sub(320)),
                (p.1 as usize).min((self.height as usize).saturating_sub(410)),
            );
            let sel = ((t - 46.0) * 3.0) as usize % 12;
            (mx, my, sel)
        });
        let menu = match (menu, self.menu) {
            // Open: it stays where it opened.
            (Some((_, _, sel)), Some((mx, my, _))) => Some((mx, my, sel)),
            (m, _) => m,
        };
        if menu != self.menu {
            self.menu = menu;
            changed = true;
        }
        if changed {
            self.compose();
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_demo_changes_like_a_demo() {
        let mut d = Demo::new(640, 360);
        let frames = (0..60 * 30).filter(|i| d.step(*i, 30)).count();
        // The pointer rests now and then: not every frame changes, most do.
        assert!(frames > 1000 && frames < 1800, "{frames}");
        // The same minute every time.
        let mut again = Demo::new(640, 360);
        for i in 0..300 {
            again.step(i, 30);
        }
        let mut d = Demo::new(640, 360);
        for i in 0..300 {
            d.step(i, 30);
        }
        assert_eq!(d.px, again.px);
    }
}
