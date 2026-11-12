//! On-device text recognition (SMT-001, SMT-003, SMT-004): one engine for every feature
//! that reads text. On Windows it is the system's (`Windows.Media.Ocr`); nothing leaves
//! the computer and nothing is downloaded. The text lives in memory only while it is
//! used, and is never logged.
//!
//! The result is blocks (paragraphs) of lines of words, each with its rectangle in
//! image pixels, in reading order: top to bottom, a whole column before the next.

use std::sync::atomic::{AtomicBool, Ordering};

use image::{GrayImage, RgbaImage};
use serde::Serialize;

pub mod layout;

#[cfg(windows)]
mod win;

pub use layout::{html, layout, plain};

/// A rectangle in image pixels: x, y, width, height.
pub type Rect = [f64; 4];

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Word {
    pub text: String,
    pub rect: Rect,
    /// 0 to 1. The system engine does not say, so its words count as sure (1).
    pub confidence: f32,
    /// Between 0.5 and 0.8: shown as doubtful (SMT-003).
    pub doubtful: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Line {
    pub text: String,
    pub rect: Rect,
    pub words: Vec<Word>,
    /// Estimated from the pixels: the text's color and the background's, `#RRGGBB`.
    pub color: String,
    pub background: String,
    /// Height of the line in pixels: its letter size.
    pub height: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Block {
    pub rect: Rect,
    pub lines: Vec<Line>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Recognized {
    pub blocks: Vec<Block>,
    /// The language whose reading was kept (`es-MX`, `en-US`…).
    pub language: String,
}

impl Recognized {
    pub fn words(&self) -> usize {
        self.blocks
            .iter()
            .flat_map(|b| &b.lines)
            .map(|l| l.words.len())
            .sum()
    }
}

/// A recognition language of this computer (SMT-004).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Language {
    pub tag: String,
    pub name: String,
}

#[derive(Debug, thiserror::Error)]
pub enum OcrError {
    #[error("text recognition is not available on this system yet")]
    NotSupported,
    #[error("no recognition language is installed: add one in the system's language settings")]
    NoLanguage,
    #[error("cancelled")]
    Cancelled,
    #[error("{0}")]
    Engine(String),
}

/// The languages this computer can recognize.
pub fn languages() -> Vec<Language> {
    #[cfg(windows)]
    {
        win::languages()
    }
    #[cfg(not(windows))]
    {
        Vec::new()
    }
}

/// A line as the engine gives it, in the pixels of the image it was given.
#[derive(Debug, Clone, PartialEq)]
pub struct RawLine {
    pub words: Vec<(String, Rect)>,
}

impl RawLine {
    fn rect(&self) -> Rect {
        union(self.words.iter().map(|(_, r)| *r))
    }
}

pub fn union(rects: impl Iterator<Item = Rect>) -> Rect {
    let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
    for [x, y, w, h] in rects {
        x0 = x0.min(x);
        y0 = y0.min(y);
        x1 = x1.max(x + w);
        y1 = y1.max(y + h);
    }
    if x0 > x1 {
        return [0.0; 4];
    }
    [x0, y0, x1 - x0, y1 - y0]
}

/// The scale the image is given to the engine at: 1.5, smaller when it would be wider
/// than the engine reads. Measured on the system's engine: at 2× small interface text
/// read worse ("Ayude" for "Ayuda"), and at 1× narrow digits ran together ("111" as
/// "1 11"); 1.5× read both right.
fn scale_for(width: u32, _height: u32, max_side: u32) -> f64 {
    (max_side as f64 / width as f64).min(SCALE)
}

const SCALE: f64 = 1.5;

/// Gray pixels for the engine, light text on dark turned dark on light.
fn prepare(image: &RgbaImage, scale: f64) -> GrayImage {
    let gray = image::DynamicImage::ImageRgba8(image.clone()).to_luma8();
    let mean = gray.pixels().map(|p| p.0[0] as u64).sum::<u64>() / gray.len().max(1) as u64;
    let mut gray = if (scale - 1.0).abs() > 0.01 {
        let w = ((image.width() as f64 * scale).round() as u32).max(1);
        let h = ((image.height() as f64 * scale).round() as u32).max(1);
        image::imageops::resize(&gray, w, h, image::imageops::FilterType::CatmullRom)
    } else {
        gray
    };
    if mean < 100 {
        image::imageops::invert(&mut gray);
    }
    stretch(&mut gray);
    gray
}

/// Levels stretched so that the background is white and the text black: thin letters
/// of interface text come out darker (SMT-003, small text).
fn stretch(gray: &mut GrayImage) {
    let mut hist = [0u64; 256];
    for p in gray.pixels() {
        hist[p.0[0] as usize] += 1;
    }
    let total: u64 = hist.iter().sum();
    let at = |q: f64| {
        let want = (total as f64 * q) as u64;
        let mut seen = 0;
        for (v, n) in hist.iter().enumerate() {
            seen += n;
            if seen > want {
                return v as f64;
            }
        }
        255.0
    };
    let (low, high) = (at(0.01), at(0.6));
    if high - low < 32.0 {
        return;
    }
    for p in gray.pixels_mut() {
        let v = ((p.0[0] as f64 - low) / (high - low) * 255.0).clamp(0.0, 255.0);
        p.0[0] = v as u8;
    }
}

/// Rows of the (scaled) image the engine reads at a time, with an overlap so that no
/// line is cut, and the rows each tile owns: a line belongs to the tile its middle is
/// in (SMT-003, tall images).
pub fn tiles(height: u32, max_side: u32, overlap: u32) -> Vec<(u32, u32, u32, u32)> {
    if height <= max_side {
        return vec![(0, height, 0, height)];
    }
    let step = max_side - overlap;
    let mut out = Vec::new();
    let mut top = 0;
    loop {
        let bottom = (top + max_side).min(height);
        let own_from = if top == 0 { 0 } else { top + overlap / 2 };
        let own_to = if bottom == height {
            height
        } else {
            bottom - overlap / 2
        };
        out.push((top, bottom, own_from, own_to));
        if bottom == height {
            break;
        }
        top += step;
    }
    out
}

/// Recognizes the text of an image (SMT-003) with up to 3 languages: tags, or `auto`
/// for the system's preferred ones (empty means `auto`). The reading with more letters
/// wins; on a tie, the first.
pub fn recognize(
    image: &RgbaImage,
    languages: &[String],
    cancel: &AtomicBool,
) -> Result<Recognized, OcrError> {
    #[cfg(not(windows))]
    {
        let _ = (image, languages, cancel);
        Err(OcrError::NotSupported)
    }
    #[cfg(windows)]
    {
        let max_side = win::max_dimension();
        let scale = scale_for(image.width(), image.height(), max_side);
        let gray = prepare(image, scale);
        // "auto" is the system's preferred languages; a tag, that language. In order:
        // when two readings have as many letters, the first wins.
        let mut wanted: Vec<Option<String>> = Vec::new();
        for l in languages.iter().filter(|l| !l.is_empty()).take(3) {
            let item = (l.as_str() != "auto").then(|| l.clone());
            if !wanted.contains(&item) {
                wanted.push(item);
            }
        }
        if wanted.is_empty() {
            wanted.push(None);
        }
        let mut best: Option<(usize, Vec<RawLine>, String)> = None;
        let mut last_error = None;
        for lang in &wanted {
            if cancel.load(Ordering::Relaxed) {
                return Err(OcrError::Cancelled);
            }
            let mut lines = Vec::new();
            let mut used = String::new();
            let mut failed = false;
            for (top, bottom, own_from, own_to) in tiles(gray.height(), max_side, 400) {
                if cancel.load(Ordering::Relaxed) {
                    return Err(OcrError::Cancelled);
                }
                let tile = image::imageops::crop_imm(&gray, 0, top, gray.width(), bottom - top).to_image();
                match win::recognize(&tile, lang.as_deref()) {
                    Ok((raw, tag)) => {
                        used = tag;
                        for line in raw {
                            let [_, y, _, h] = line.rect();
                            let middle = top as f64 + y + h / 2.0;
                            if middle < own_from as f64 || middle >= own_to as f64 {
                                continue;
                            }
                            // Back to the image's own pixels.
                            lines.push(RawLine {
                                words: line
                                    .words
                                    .into_iter()
                                    .map(|(t, [x, y, w, h])| {
                                        (t, [x / scale, (y + top as f64) / scale, w / scale, h / scale])
                                    })
                                    .collect(),
                            });
                        }
                    }
                    Err(e) => {
                        last_error = Some(e);
                        failed = true;
                        break;
                    }
                }
            }
            if failed {
                continue;
            }
            let letters: usize = lines
                .iter()
                .flat_map(|l| &l.words)
                .map(|(t, _)| t.chars().filter(|c| c.is_alphanumeric()).count())
                .sum();
            if best.as_ref().is_none_or(|(n, _, _)| letters > *n) {
                best = Some((letters, lines, used));
            }
        }
        let Some((_, raw, language)) = best else {
            return Err(last_error.unwrap_or(OcrError::NoLanguage));
        };
        Ok(Recognized {
            blocks: layout(lines_with_colors(image, raw)),
            language,
        })
    }
}

/// The lines with their text, words and colors read from the image.
fn lines_with_colors(image: &RgbaImage, raw: Vec<RawLine>) -> Vec<Line> {
    raw.into_iter()
        .filter(|l| !l.words.is_empty())
        .map(|l| {
            let rect = l.rect();
            let (color, background) = colors(image, rect);
            Line {
                text: join_words(l.words.iter().map(|(t, _)| t.as_str())),
                rect,
                words: l
                    .words
                    .into_iter()
                    .map(|(text, rect)| Word {
                        text,
                        rect,
                        confidence: 1.0,
                        doubtful: false,
                    })
                    .collect(),
                color,
                background,
                height: rect[3],
            }
        })
        .collect()
}

/// Words joined with spaces, except between characters of scripts without them.
fn join_words<'a>(words: impl Iterator<Item = &'a str>) -> String {
    let cjk =
        |c: char| matches!(c as u32, 0x3040..=0x30FF | 0x3400..=0x4DBF | 0x4E00..=0x9FFF | 0xF900..=0xFAFF);
    let mut out = String::new();
    for w in words {
        let glue = match (out.chars().last(), w.chars().next()) {
            (Some(a), Some(b)) => !(cjk(a) && cjk(b)),
            _ => false,
        };
        if glue {
            out.push(' ');
        }
        out.push_str(w);
    }
    out
}

/// The average color of `n` pixels whose channels add up to `sum`.
fn mean(sum: [u64; 3], n: u64) -> Option<[u8; 3]> {
    let [r, g, b] = sum;
    Some([
        r.checked_div(n)? as u8,
        g.checked_div(n)? as u8,
        b.checked_div(n)? as u8,
    ])
}

fn hex([r, g, b]: [u8; 3]) -> String {
    format!("#{r:02X}{g:02X}{b:02X}")
}

/// The background (the most common shade) and the text (the shade farthest from it)
/// of a line's rectangle (SMT-003).
fn colors(image: &RgbaImage, [x, y, w, h]: Rect) -> (String, String) {
    let (iw, ih) = (image.width() as f64, image.height() as f64);
    let (x0, y0) = (x.max(0.0) as u32, y.max(0.0) as u32);
    let (x1, y1) = ((x + w).min(iw) as u32, (y + h).min(ih) as u32);
    if x1 <= x0 || y1 <= y0 {
        return ("#000000".into(), "#FFFFFF".into());
    }
    let lum = |p: &image::Rgba<u8>| 0.299 * p.0[0] as f64 + 0.587 * p.0[1] as f64 + 0.114 * p.0[2] as f64;
    let mut bins = [0u32; 16];
    for yy in y0..y1 {
        for xx in x0..x1 {
            bins[(lum(image.get_pixel(xx, yy)) as usize / 16).min(15)] += 1;
        }
    }
    let bg_bin = (0..16).max_by_key(|&i| bins[i]).unwrap_or(15);
    let (mut bg, mut nbg) = ([0u64; 3], 0u64);
    let mut far = 0.0f64;
    for yy in y0..y1 {
        for xx in x0..x1 {
            let p = image.get_pixel(xx, yy);
            let l = lum(p);
            if (l as usize / 16).min(15) == bg_bin {
                for (acc, v) in bg.iter_mut().zip(p.0) {
                    *acc += v as u64;
                }
                nbg += 1;
            }
            far = far.max((l - (bg_bin as f64 * 16.0 + 8.0)).abs());
        }
    }
    let bg_rgb = mean(bg, nbg).unwrap_or([255, 255, 255]);
    let bg_l = 0.299 * bg_rgb[0] as f64 + 0.587 * bg_rgb[1] as f64 + 0.114 * bg_rgb[2] as f64;
    let (mut fg, mut nfg) = ([0u64; 3], 0u64);
    for yy in y0..y1 {
        for xx in x0..x1 {
            let p = image.get_pixel(xx, yy);
            if (lum(p) - bg_l).abs() >= far * 0.6 && far > 20.0 {
                for (acc, v) in fg.iter_mut().zip(p.0) {
                    *acc += v as u64;
                }
                nfg += 1;
            }
        }
    }
    let fg_rgb = mean(fg, nfg).unwrap_or(if bg_l > 128.0 { [0, 0, 0] } else { [255, 255, 255] });
    (hex(fg_rgb), hex(bg_rgb))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tall_images_go_in_overlapping_tiles_that_own_each_row_once() {
        assert_eq!(tiles(800, 10_000, 400), vec![(0, 800, 0, 800)]);
        let t = tiles(25_000, 10_000, 400);
        assert_eq!(t.first().unwrap().0, 0);
        assert_eq!(t.last().unwrap().1, 25_000);
        // Every row is owned by exactly one tile.
        for row in [0u32, 9_799, 9_800, 9_801, 19_400, 24_999] {
            let owners = t.iter().filter(|(_, _, a, b)| row >= *a && row < *b).count();
            assert_eq!(owners, 1, "row {row} in {t:?}");
        }
        // Tiles overlap, so a line cut at one edge is whole in the other tile.
        assert!(t.windows(2).all(|w| w[1].0 < w[0].1));
    }

    #[test]
    fn images_are_read_a_bit_bigger_unless_too_wide() {
        assert_eq!(scale_for(1920, 1080, 10_000), 1.5);
        assert_eq!(scale_for(10_000, 30_000, 10_000), 1.0);
        assert_eq!(scale_for(20_000, 1_000, 10_000), 0.5);
    }

    #[test]
    fn words_join_with_spaces_except_in_cjk() {
        assert_eq!(join_words(["Hola", "mundo"].into_iter()), "Hola mundo");
        assert_eq!(join_words(["日本", "語"].into_iter()), "日本語");
    }

    #[test]
    fn colors_of_dark_text_on_white_and_light_text_on_dark() {
        let mut img = RgbaImage::from_pixel(40, 10, image::Rgba([255, 255, 255, 255]));
        for x in 5..15 {
            for y in 2..8 {
                img.put_pixel(x, y, image::Rgba([200, 0, 0, 255]));
            }
        }
        let (fg, bg) = colors(&img, [0.0, 0.0, 40.0, 10.0]);
        assert_eq!(bg, "#FFFFFF");
        assert_eq!(fg, "#C80000");
        let dark = RgbaImage::from_pixel(10, 10, image::Rgba([20, 20, 30, 255]));
        let (fg, bg) = colors(&dark, [0.0, 0.0, 10.0, 10.0]);
        assert_eq!(bg, "#14141E");
        assert_eq!(fg, "#FFFFFF");
    }
}
