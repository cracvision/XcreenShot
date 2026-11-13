//! Animated GIF (VID-068 to VID-070): a video's frames as a GIF89a, and reading back a
//! GIF's size, length and first frame (thumbnails, the editor).
//!
//! Each frame keeps only the rectangle that changed since the one before (screen
//! recordings change little), with its own palette ("dynamic colors") or one shared by
//! all; Floyd–Steinberg dithering is optional. A frame equal to the one before only
//! makes that one last longer.

use image::RgbaImage;

use crate::export::{index_of, lookup, palette};

/// How many times the GIF plays (VID-070).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Loops {
    Forever,
    /// Plays this many times in all (1: once), then stays on its last frame.
    Times(u16),
}

pub struct Animation {
    out: Vec<u8>,
    width: u32,
    height: u32,
    shared: Option<(Vec<[u8; 3]>, Vec<u8>)>,
    dither: bool,
    /// The frame as last written, to find what changed.
    last: Option<RgbaImage>,
    /// The newest frame, waiting to know how long it lasts (centiseconds).
    pending: Option<(RgbaImage, u32)>,
    frames: u32,
}

/// The table holds 2^n colors: n for `len` colors.
fn bits_for(len: usize) -> u8 {
    (1..=8).find(|b| (1usize << b) >= len).unwrap_or(8)
}

fn write_table(out: &mut Vec<u8>, pal: &[[u8; 3]], bits: u8) {
    for i in 0..(1usize << bits) {
        out.extend_from_slice(pal.get(i).unwrap_or(&[0, 0, 0]));
    }
}

/// One palette for a whole GIF, from some of its frames (VID-070, colors not dynamic).
pub fn shared_palette(samples: &[RgbaImage]) -> Vec<[u8; 3]> {
    let w = samples.iter().map(|s| s.width()).max().unwrap_or(1);
    let h: u32 = samples.iter().map(|s| s.height()).sum::<u32>().max(1);
    let mut mosaic = RgbaImage::new(w, h);
    let mut y = 0;
    for s in samples {
        image::imageops::overlay(&mut mosaic, s, 0, y as i64);
        y += s.height();
    }
    palette(&mosaic, 256)
}

/// The palette indices of `img`, spreading each pixel's error to its neighbours
/// (Floyd–Steinberg) when `dither`.
fn indices(img: &RgbaImage, pal: &[[u8; 3]], table: &[u8], dither: bool) -> Vec<u8> {
    if !dither {
        return img.pixels().map(|p| index_of(table, &p.0)).collect();
    }
    let (w, h) = (img.width() as usize, img.height() as usize);
    let mut work: Vec<[i32; 3]> = img
        .pixels()
        .map(|p| [p.0[0] as i32, p.0[1] as i32, p.0[2] as i32])
        .collect();
    let mut out = vec![0u8; w * h];
    for y in 0..h {
        for x in 0..w {
            let i = y * w + x;
            let c = work[i].map(|v| v.clamp(0, 255));
            let idx = index_of(table, &[c[0] as u8, c[1] as u8, c[2] as u8, 255]);
            out[i] = idx;
            let chosen = pal.get(idx as usize).copied().unwrap_or([0, 0, 0]);
            let e = [0, 1, 2].map(|k| c[k] - chosen[k] as i32);
            let mut spread = |dx: isize, dy: usize, k: i32| {
                let nx = x as isize + dx;
                if nx >= 0 && (nx as usize) < w && y + dy < h {
                    let j = (y + dy) * w + nx as usize;
                    for ch in 0..3 {
                        work[j][ch] += e[ch] * k / 16;
                    }
                }
            };
            spread(1, 0, 7);
            spread(-1, 1, 3);
            spread(0, 1, 5);
            spread(1, 1, 1);
        }
    }
    out
}

/// The smallest rectangle where `b` differs from `a`: x, y, width, height.
fn changed(a: &RgbaImage, b: &RgbaImage) -> Option<(u32, u32, u32, u32)> {
    let (w, h) = b.dimensions();
    let (mut x0, mut y0, mut x1, mut y1) = (w, h, 0, 0);
    for y in 0..h {
        for x in 0..w {
            if a.get_pixel(x, y) != b.get_pixel(x, y) {
                x0 = x0.min(x);
                y0 = y0.min(y);
                x1 = x1.max(x);
                y1 = y1.max(y);
            }
        }
    }
    (x0 <= x1).then(|| (x0, y0, x1 - x0 + 1, y1 - y0 + 1))
}

impl Animation {
    /// A GIF of `width` × `height` px; `shared`: one palette for all frames.
    pub fn new(
        width: u32,
        height: u32,
        loops: Loops,
        shared: Option<Vec<[u8; 3]>>,
        dither: bool,
    ) -> Animation {
        let (w, h) = (width.clamp(1, 65535), height.clamp(1, 65535));
        let mut out = Vec::new();
        out.extend_from_slice(b"GIF89a");
        out.extend_from_slice(&(w as u16).to_le_bytes());
        out.extend_from_slice(&(h as u16).to_le_bytes());
        let shared = shared.map(|pal| {
            let table = lookup(&pal);
            (pal, table)
        });
        match &shared {
            Some((pal, _)) => {
                let bits = bits_for(pal.len());
                out.push(0x80 | 0x70 | (bits - 1));
                out.extend_from_slice(&[0, 0]);
                write_table(&mut out, pal, bits);
            }
            None => out.extend_from_slice(&[0x70, 0, 0]),
        }
        // NETSCAPE2.0: the repeats. Without it a GIF plays once.
        let count = match loops {
            Loops::Forever => Some(0u16),
            Loops::Times(n) if n > 1 => Some(n - 1),
            Loops::Times(_) => None,
        };
        if let Some(n) = count {
            out.extend_from_slice(&[0x21, 0xFF, 11]);
            out.extend_from_slice(b"NETSCAPE2.0");
            out.extend_from_slice(&[3, 1]);
            out.extend_from_slice(&n.to_le_bytes());
            out.push(0);
        }
        Animation {
            out,
            width: w,
            height: h,
            shared,
            dither,
            last: None,
            pending: None,
            frames: 0,
        }
    }

    /// Adds a frame shown for `delay` hundredths of a second; it must have the GIF's
    /// size.
    pub fn push(&mut self, frame: RgbaImage, delay: u32) -> Result<(), String> {
        if frame.dimensions() != (self.width, self.height) {
            return Err("a frame of another size".into());
        }
        match &mut self.pending {
            Some((p, d)) if *p == frame => *d += delay,
            _ => {
                if let Some((p, d)) = self.pending.take() {
                    self.write(p, d)?;
                }
                self.pending = Some((frame, delay));
            }
        }
        Ok(())
    }

    /// Frames written so far.
    pub fn frames(&self) -> u32 {
        self.frames
    }

    /// Bytes written so far.
    pub fn len(&self) -> usize {
        self.out.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames == 0
    }

    pub fn finish(mut self) -> Result<Vec<u8>, String> {
        if let Some((p, d)) = self.pending.take() {
            self.write(p, d)?;
        }
        self.out.push(0x3B);
        Ok(self.out)
    }

    fn write(&mut self, frame: RgbaImage, delay: u32) -> Result<(), String> {
        // Only what changed; nothing changed (a pause) still needs a frame for its time.
        let (x, y, w, h) = match &self.last {
            Some(prev) => changed(prev, &frame).unwrap_or((0, 0, 1, 1)),
            None => (0, 0, self.width, self.height),
        };
        let part = image::imageops::crop_imm(&frame, x, y, w, h).to_image();
        let (pal, table, local) = match &self.shared {
            Some((pal, table)) => (pal.clone(), table.clone(), false),
            None => {
                let pal = palette(&part, 256);
                let table = lookup(&pal);
                (pal, table, true)
            }
        };
        let data = indices(&part, &pal, &table, self.dither);
        // Graphic control: keep this frame under the next (disposal 1) and its time.
        let delay = delay.min(65535) as u16;
        self.out.extend_from_slice(&[0x21, 0xF9, 4, 0x04]);
        self.out.extend_from_slice(&delay.to_le_bytes());
        self.out.extend_from_slice(&[0, 0]);
        self.out.push(0x2C);
        for v in [x, y, w, h] {
            self.out.extend_from_slice(&(v as u16).to_le_bytes());
        }
        let bits = bits_for(pal.len());
        if local {
            self.out.push(0x80 | (bits - 1));
            write_table(&mut self.out, &pal, bits);
        } else {
            self.out.push(0);
        }
        let min_code = bits.max(2);
        self.out.push(min_code);
        let lzw = weezl::encode::Encoder::new(weezl::BitOrder::Lsb, min_code)
            .encode(&data)
            .map_err(|e| e.to_string())?;
        for chunk in lzw.chunks(255) {
            self.out.push(chunk.len() as u8);
            self.out.extend_from_slice(chunk);
        }
        self.out.push(0);
        self.last = Some(frame);
        self.frames += 1;
        Ok(())
    }
}

/// What a GIF holds: its size, its frames and how long one play lasts (seconds).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Info {
    pub width: u32,
    pub height: u32,
    pub frames: u32,
    pub duration: f64,
}

/// Walks a GIF: calls `image` for each frame with its descriptor's offset, its color
/// table, its transparent index and its delay; `image` returns whether to go on.
fn walk(
    bytes: &[u8],
    mut image: impl FnMut(&[u8], usize, Option<&[u8]>, Option<u8>, u16) -> bool,
) -> Option<(u32, u32)> {
    if bytes.len() < 13 || !bytes.starts_with(b"GIF8") {
        return None;
    }
    let width = u16::from_le_bytes([bytes[6], bytes[7]]) as u32;
    let height = u16::from_le_bytes([bytes[8], bytes[9]]) as u32;
    let packed = bytes[10];
    let mut at = 13;
    let global = if packed & 0x80 != 0 {
        let size = 3 * (1usize << ((packed & 7) + 1));
        let t = bytes.get(at..at + size)?;
        at += size;
        Some(t)
    } else {
        None
    };
    let skip_blocks = |mut at: usize| -> Option<usize> {
        loop {
            let n = *bytes.get(at)? as usize;
            at += 1;
            if n == 0 {
                return Some(at);
            }
            at += n;
        }
    };
    let (mut transparent, mut delay) = (None, 0u16);
    loop {
        match *bytes.get(at)? {
            0x21 => {
                let label = *bytes.get(at + 1)?;
                if label == 0xF9 && bytes.get(at + 2) == Some(&4) {
                    let flags = *bytes.get(at + 3)?;
                    delay = u16::from_le_bytes([*bytes.get(at + 4)?, *bytes.get(at + 5)?]);
                    transparent = (flags & 1 != 0).then(|| bytes[at + 6]);
                }
                at = skip_blocks(at + 2)?;
            }
            0x2C => {
                let desc = at;
                let flags = *bytes.get(at + 9)?;
                at += 10;
                let local = if flags & 0x80 != 0 {
                    let size = 3 * (1usize << ((flags & 7) + 1));
                    let t = bytes.get(at..at + size)?;
                    at += size;
                    Some(t)
                } else {
                    None
                };
                let go_on = image(bytes, desc, local.or(global), transparent, delay);
                at = skip_blocks(at + 1)?;
                (transparent, delay) = (None, 0);
                if !go_on {
                    break;
                }
            }
            _ => break,
        }
    }
    Some((width, height))
}

/// A GIF's size, frames and length (VID-064, the library).
pub fn info(bytes: &[u8]) -> Option<Info> {
    let (mut frames, mut hundredths) = (0u32, 0u64);
    let (width, height) = walk(bytes, |_, _, _, _, delay| {
        frames += 1;
        // Browsers show frames of 0 or 1 hundredths for 10.
        hundredths += if delay <= 1 { 10 } else { delay as u64 };
        true
    })?;
    Some(Info {
        width,
        height,
        frames,
        duration: hundredths as f64 / 100.0,
    })
}

/// A GIF's first frame, over transparency (thumbnails, the editor's picture).
pub fn first_frame(bytes: &[u8]) -> Option<RgbaImage> {
    let mut first: Option<(RgbaImage, i64, i64)> = None;
    let (width, height) = walk(bytes, |b, desc, table, transparent, _| {
        let x = u16::from_le_bytes([b[desc + 1], b[desc + 2]]) as i64;
        let y = u16::from_le_bytes([b[desc + 3], b[desc + 4]]) as i64;
        first = decode_frame(b, desc, table, transparent).map(|f| (f, x, y));
        false
    })?;
    let (part, x, y) = first?;
    let mut canvas = RgbaImage::new(width.max(1), height.max(1));
    image::imageops::overlay(&mut canvas, &part, x, y);
    Some(canvas)
}

fn decode_frame(
    bytes: &[u8],
    desc: usize,
    table: Option<&[u8]>,
    transparent: Option<u8>,
) -> Option<RgbaImage> {
    let w = u16::from_le_bytes([*bytes.get(desc + 5)?, *bytes.get(desc + 6)?]) as u32;
    let h = u16::from_le_bytes([*bytes.get(desc + 7)?, *bytes.get(desc + 8)?]) as u32;
    let flags = *bytes.get(desc + 9)?;
    let mut at = desc + 10;
    if flags & 0x80 != 0 {
        at += 3 * (1usize << ((flags & 7) + 1));
    }
    let min_code = *bytes.get(at)?;
    at += 1;
    let mut data = Vec::new();
    loop {
        let n = *bytes.get(at)? as usize;
        at += 1;
        if n == 0 {
            break;
        }
        data.extend_from_slice(bytes.get(at..at + n)?);
        at += n;
    }
    let mut decoder = weezl::decode::Decoder::new(weezl::BitOrder::Lsb, min_code.clamp(2, 11));
    let indices = decoder.decode(&data).ok()?;
    let table = table?;
    let interlaced = flags & 0x40 != 0;
    // Interlaced rows come in four passes.
    let rows: Vec<u32> = if interlaced {
        let mut r = Vec::with_capacity(h as usize);
        for (start, step) in [(0, 8), (4, 8), (2, 4), (1, 2)] {
            r.extend((start..h).step_by(step));
        }
        r
    } else {
        (0..h).collect()
    };
    let mut img = RgbaImage::new(w.max(1), h.max(1));
    for (n, &row) in rows.iter().enumerate() {
        for x in 0..w {
            let Some(&i) = indices.get(n * w as usize + x as usize) else {
                continue;
            };
            if Some(i) == transparent {
                continue;
            }
            let c = table
                .get(i as usize * 3..i as usize * 3 + 3)
                .unwrap_or(&[0, 0, 0]);
            img.put_pixel(x, row, image::Rgba([c[0], c[1], c[2], 255]));
        }
    }
    Some(img)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: u32, h: u32, c: [u8; 3]) -> RgbaImage {
        RgbaImage::from_pixel(w, h, image::Rgba([c[0], c[1], c[2], 255]))
    }

    #[test]
    fn frames_play_and_read_back() {
        let mut a = Animation::new(40, 30, Loops::Forever, None, false);
        a.push(solid(40, 30, [200, 30, 30]), 50).unwrap();
        a.push(solid(40, 30, [200, 30, 30]), 50).unwrap();
        let mut b = solid(40, 30, [200, 30, 30]);
        for x in 10..20 {
            for y in 5..15 {
                b.put_pixel(x, y, image::Rgba([30, 60, 210, 255]));
            }
        }
        a.push(b, 100).unwrap();
        let gif = a.finish().unwrap();
        let i = info(&gif).unwrap();
        // The two equal frames are one of 1 s; then the changed one.
        assert_eq!((i.width, i.height, i.frames), (40, 30, 2));
        assert!((i.duration - 2.0).abs() < 1e-9);
        let first = first_frame(&gif).unwrap();
        let p = first.get_pixel(20, 20).0;
        assert!((p[0] as i32 - 200).abs() <= 8 && p[2] < 50, "{p:?}");
        // The second frame only holds the blue square.
        let mut sizes = Vec::new();
        walk(&gif, |b, desc, _, _, _| {
            sizes.push(u16::from_le_bytes([b[desc + 5], b[desc + 6]]));
            true
        });
        assert_eq!(sizes, vec![40, 10]);
    }

    #[test]
    fn repeats_are_written() {
        let netscape = |l: Loops| {
            let mut a = Animation::new(4, 4, l, None, false);
            a.push(solid(4, 4, [0, 0, 0]), 10).unwrap();
            let g = a.finish().unwrap();
            g.windows(11)
                .position(|w| w == b"NETSCAPE2.0")
                .map(|i| u16::from_le_bytes([g[i + 13], g[i + 14]]))
        };
        assert_eq!(netscape(Loops::Forever), Some(0));
        // Three plays: the first and two repeats.
        assert_eq!(netscape(Loops::Times(3)), Some(2));
        assert_eq!(netscape(Loops::Times(1)), None);
    }

    #[test]
    fn a_shared_palette_and_dithering() {
        let pal = shared_palette(&[solid(8, 8, [255, 0, 0]), solid(8, 8, [0, 0, 255])]);
        let mut a = Animation::new(8, 8, Loops::Forever, Some(pal), true);
        let mut grad = RgbaImage::new(8, 8);
        for (x, _, p) in grad.enumerate_pixels_mut() {
            *p = image::Rgba([(x * 32) as u8, 0, 255 - (x * 32) as u8, 255]);
        }
        a.push(grad, 10).unwrap();
        let gif = a.finish().unwrap();
        assert_eq!(info(&gif).unwrap().frames, 1);
        assert!(first_frame(&gif).is_some());
    }
}
