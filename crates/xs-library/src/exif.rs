//! The date a photo or picture was taken, from its EXIF block (LIB-024): a JPEG's APP1
//! segment or a PNG's `eXIf` chunk. Only `DateTimeOriginal` (or `DateTime`), read
//! as local time; anything else in the block is skipped.

use std::io::Read;
use std::path::Path;

use chrono::{Local, NaiveDateTime, TimeZone};

/// What is read of a file at most: the EXIF block is near the start.
const HEAD: u64 = 256 * 1024;

/// Milliseconds since 1970 of the file's EXIF date, if it has one.
pub fn taken(path: &Path) -> Option<i64> {
    let mut head = Vec::new();
    std::fs::File::open(path)
        .ok()?
        .take(HEAD)
        .read_to_end(&mut head)
        .ok()?;
    let tiff = if head.starts_with(&[0xFF, 0xD8]) {
        jpeg_exif(&head)?
    } else if head.starts_with(b"\x89PNG\r\n\x1a\n") {
        png_exif(&head)?
    } else {
        return None;
    };
    let text = date_in(tiff)?;
    let naive =
        NaiveDateTime::parse_from_str(text.trim_end_matches('\0').trim(), "%Y:%m:%d %H:%M:%S").ok()?;
    Local
        .from_local_datetime(&naive)
        .earliest()
        .map(|t| t.timestamp_millis())
}

fn jpeg_exif(b: &[u8]) -> Option<&[u8]> {
    let mut i = 2;
    while i + 4 <= b.len() {
        if b[i] != 0xFF {
            return None;
        }
        let marker = b[i + 1];
        // Start of scan or end of image: no more metadata segments.
        if marker == 0xDA || marker == 0xD9 {
            return None;
        }
        let len = u16::from_be_bytes([b[i + 2], b[i + 3]]) as usize;
        let body = b.get(i + 4..i + 2 + len)?;
        if marker == 0xE1 && body.starts_with(b"Exif\0\0") {
            return Some(&body[6..]);
        }
        i += 2 + len;
    }
    None
}

fn png_exif(b: &[u8]) -> Option<&[u8]> {
    let mut i = 8;
    while i + 8 <= b.len() {
        let len = u32::from_be_bytes(b[i..i + 4].try_into().ok()?) as usize;
        let kind = &b[i + 4..i + 8];
        if kind == b"eXIf" {
            return b.get(i + 8..i + 8 + len);
        }
        if kind == b"IDAT" || kind == b"IEND" {
            return None;
        }
        i += 12 + len;
    }
    None
}

/// `DateTimeOriginal` of the EXIF sub-IFD, else `DateTime` of IFD0.
fn date_in(t: &[u8]) -> Option<String> {
    let little = match t.get(0..2)? {
        b"II" => true,
        b"MM" => false,
        _ => return None,
    };
    let u16_at = |o: usize| -> Option<u16> {
        let s: [u8; 2] = t.get(o..o + 2)?.try_into().ok()?;
        Some(if little {
            u16::from_le_bytes(s)
        } else {
            u16::from_be_bytes(s)
        })
    };
    let u32_at = |o: usize| -> Option<u32> {
        let s: [u8; 4] = t.get(o..o + 4)?.try_into().ok()?;
        Some(if little {
            u32::from_le_bytes(s)
        } else {
            u32::from_be_bytes(s)
        })
    };
    // An IFD's entries: (tag, type, count, value or offset).
    let entries = |ifd: usize| -> Vec<(u16, u16, u32, usize)> {
        let n = u16_at(ifd).unwrap_or(0) as usize;
        (0..n.min(512))
            .filter_map(|k| {
                let e = ifd + 2 + k * 12;
                Some((u16_at(e)?, u16_at(e + 2)?, u32_at(e + 4)?, e + 8))
            })
            .collect()
    };
    let ascii = |count: u32, at: usize| -> Option<String> {
        let off = u32_at(at)? as usize;
        let bytes = t.get(off..off + count as usize)?;
        Some(String::from_utf8_lossy(bytes).into_owned())
    };
    let ifd0 = u32_at(4)? as usize;
    let first = entries(ifd0);
    let sub = first
        .iter()
        .find(|e| e.0 == 0x8769)
        .and_then(|e| u32_at(e.3))
        .map(|o| o as usize);
    if let Some(sub) = sub {
        if let Some(e) = entries(sub).into_iter().find(|e| e.0 == 0x9003 && e.1 == 2) {
            return ascii(e.2, e.3);
        }
    }
    first
        .into_iter()
        .find(|e| e.0 == 0x0132 && e.1 == 2)
        .and_then(|e| ascii(e.2, e.3))
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A little-endian TIFF block with `DateTimeOriginal` in the EXIF sub-IFD.
    pub fn tiff_with_date(date: &str) -> Vec<u8> {
        let mut t = Vec::new();
        t.extend_from_slice(b"II*\0");
        t.extend_from_slice(&8u32.to_le_bytes());
        // IFD0 at 8: one entry, the EXIF pointer; then no next IFD.
        t.extend_from_slice(&1u16.to_le_bytes());
        t.extend_from_slice(&0x8769u16.to_le_bytes());
        t.extend_from_slice(&4u16.to_le_bytes());
        t.extend_from_slice(&1u32.to_le_bytes());
        t.extend_from_slice(&26u32.to_le_bytes());
        t.extend_from_slice(&0u32.to_le_bytes());
        // EXIF IFD at 26: DateTimeOriginal, ASCII, 20 bytes at 44.
        t.extend_from_slice(&1u16.to_le_bytes());
        t.extend_from_slice(&0x9003u16.to_le_bytes());
        t.extend_from_slice(&2u16.to_le_bytes());
        t.extend_from_slice(&20u32.to_le_bytes());
        t.extend_from_slice(&44u32.to_le_bytes());
        t.extend_from_slice(&0u32.to_le_bytes());
        assert_eq!(t.len(), 44);
        t.extend_from_slice(date.as_bytes());
        t.push(0);
        t
    }

    /// A JPEG made by the `image` crate with an EXIF APP1 segment put after SOI.
    pub fn jpeg_with_date(date: &str) -> Vec<u8> {
        let img = image::RgbImage::from_pixel(4, 4, image::Rgb([200, 100, 50]));
        let mut jpg = Vec::new();
        image::codecs::jpeg::JpegEncoder::new(&mut jpg)
            .encode_image(&img)
            .unwrap();
        let mut body = b"Exif\0\0".to_vec();
        body.extend(tiff_with_date(date));
        let mut out = jpg[..2].to_vec();
        out.extend_from_slice(&[0xFF, 0xE1]);
        out.extend_from_slice(&((body.len() + 2) as u16).to_be_bytes());
        out.extend(body);
        out.extend_from_slice(&jpg[2..]);
        out
    }

    #[test]
    fn reads_the_date_a_photo_was_taken() {
        let dir = tempfile::tempdir().unwrap();
        let p = dir.path().join("old.jpg");
        std::fs::write(&p, jpeg_with_date("2019:06:15 10:30:00")).unwrap();
        let ms = taken(&p).unwrap();
        let t = Local.timestamp_millis_opt(ms).unwrap();
        assert_eq!(t.format("%Y-%m-%d %H:%M").to_string(), "2019-06-15 10:30");
        let plain = dir.path().join("plain.jpg");
        let img = image::RgbImage::from_pixel(4, 4, image::Rgb([0, 0, 0]));
        img.save(&plain).unwrap();
        assert_eq!(taken(&plain), None, "no EXIF, no date");
    }
}
