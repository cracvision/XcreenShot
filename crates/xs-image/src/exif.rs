//! A small EXIF block for JPG files (CAP-053): the capture's details in the standard
//! fields photo tools and Windows' file properties read. Little-endian TIFF with:
//! - IFD0: ImageDescription (window title), Software, DateTime and the Windows fields
//!   XPTitle (title), XPSubject (app) and XPComment (address), in UTF-16;
//! - the Exif IFD: DateTimeOriginal and OffsetTimeOriginal.

/// What goes into the block. Empty texts are left out.
#[derive(Debug, Default, Clone)]
pub struct Details {
    pub title: String,
    pub app: String,
    pub url: String,
    /// `YYYY:MM:DD HH:MM:SS`, local time.
    pub taken: String,
    /// `+HH:MM`, the offset of `taken`.
    pub offset: String,
}

const ASCII: u16 = 2;
const BYTE: u16 = 1;
const LONG: u16 = 4;

struct Entry {
    tag: u16,
    kind: u16,
    count: u32,
    data: Vec<u8>,
}

fn ascii(tag: u16, text: &str) -> Entry {
    // UTF-8 bytes: what most writers put in "ASCII" fields for other languages.
    let mut data = text.as_bytes().to_vec();
    data.push(0);
    Entry {
        tag,
        kind: ASCII,
        count: data.len() as u32,
        data,
    }
}

fn utf16(tag: u16, text: &str) -> Entry {
    let mut data: Vec<u8> = text.encode_utf16().flat_map(u16::to_le_bytes).collect();
    data.extend_from_slice(&[0, 0]);
    Entry {
        tag,
        kind: BYTE,
        count: data.len() as u32,
        data,
    }
}

/// Writes one IFD at `start` (its offset in the TIFF): entries, next-IFD 0 and the
/// values that do not fit in an entry, after it.
fn ifd(mut entries: Vec<Entry>, start: u32) -> Vec<u8> {
    entries.sort_by_key(|e| e.tag);
    let head = 2 + 12 * entries.len() as u32 + 4;
    let mut out = Vec::new();
    let mut extra = Vec::new();
    out.extend_from_slice(&(entries.len() as u16).to_le_bytes());
    for e in &entries {
        out.extend_from_slice(&e.tag.to_le_bytes());
        out.extend_from_slice(&e.kind.to_le_bytes());
        out.extend_from_slice(&e.count.to_le_bytes());
        if e.data.len() <= 4 {
            let mut inline = e.data.clone();
            inline.resize(4, 0);
            out.extend_from_slice(&inline);
        } else {
            let at = start + head + extra.len() as u32;
            out.extend_from_slice(&at.to_le_bytes());
            extra.extend_from_slice(&e.data);
            // Values start on an even offset.
            if extra.len() % 2 == 1 {
                extra.push(0);
            }
        }
    }
    out.extend_from_slice(&0u32.to_le_bytes());
    out.extend_from_slice(&extra);
    out
}

/// The TIFF data of the block (what follows `Exif\0\0`).
pub fn tiff(d: &Details) -> Vec<u8> {
    let mut first = vec![ascii(0x0131, "XcreenShot")];
    if !d.title.is_empty() {
        first.push(ascii(0x010E, &d.title));
        first.push(utf16(0x9C9B, &d.title));
    }
    if !d.app.is_empty() {
        first.push(utf16(0x9C9F, &d.app));
    }
    if !d.url.is_empty() {
        first.push(utf16(0x9C9C, &d.url));
    }
    let mut exif = Vec::new();
    if !d.taken.is_empty() {
        first.push(ascii(0x0132, &d.taken));
        exif.push(ascii(0x9003, &d.taken));
        if !d.offset.is_empty() {
            exif.push(ascii(0x9011, &d.offset));
        }
    }
    let mut out = b"II\x2A\x00\x08\x00\x00\x00".to_vec();
    if exif.is_empty() {
        out.extend(ifd(first, 8));
        return out;
    }
    // The pointer to the Exif IFD needs its offset: lay IFD0 out once to know its size.
    first.push(Entry {
        tag: 0x8769,
        kind: LONG,
        count: 1,
        data: vec![0; 4],
    });
    let size = ifd(
        first
            .iter()
            .map(|e| Entry {
                tag: e.tag,
                kind: e.kind,
                count: e.count,
                data: e.data.clone(),
            })
            .collect(),
        8,
    )
    .len() as u32;
    let exif_at = 8 + size;
    if let Some(p) = first.iter_mut().find(|e| e.tag == 0x8769) {
        p.data = exif_at.to_le_bytes().to_vec();
    }
    out.extend(ifd(first, 8));
    out.extend(ifd(exif, exif_at));
    out
}

/// `jpeg` with an APP1 Exif segment for `d`, after the JFIF header if there is one.
/// A block too big for a segment (titles of thousands of characters) is left out.
pub fn add_to_jpeg(jpeg: &[u8], d: &Details) -> Vec<u8> {
    let mut payload = b"Exif\0\0".to_vec();
    payload.extend(tiff(d));
    if payload.len() + 2 > u16::MAX as usize || !jpeg.starts_with(&[0xFF, 0xD8]) {
        return jpeg.to_vec();
    }
    // After SOI, and after APP0 (JFIF) when it is there.
    let mut at = 2;
    if jpeg.len() > 4 && jpeg[2] == 0xFF && jpeg[3] == 0xE0 {
        let len = u16::from_be_bytes([jpeg[4], jpeg[5]]) as usize;
        at = 4 + len;
    }
    let mut out = Vec::with_capacity(jpeg.len() + payload.len() + 4);
    out.extend_from_slice(&jpeg[..at]);
    out.extend_from_slice(&[0xFF, 0xE1]);
    out.extend_from_slice(&((payload.len() + 2) as u16).to_be_bytes());
    out.extend_from_slice(&payload);
    out.extend_from_slice(&jpeg[at..]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_u16(b: &[u8], at: usize) -> u16 {
        u16::from_le_bytes([b[at], b[at + 1]])
    }
    fn read_u32(b: &[u8], at: usize) -> u32 {
        u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
    }
    /// The value of `tag` in the IFD at `at`, as raw bytes.
    fn value(t: &[u8], at: usize, tag: u16) -> Option<Vec<u8>> {
        let n = read_u16(t, at) as usize;
        (0..n).find_map(|i| {
            let e = at + 2 + 12 * i;
            (read_u16(t, e) == tag).then(|| {
                let count = read_u32(t, e + 4) as usize;
                let kind = read_u16(t, e + 2);
                let size = if kind == LONG { 4 * count } else { count };
                if size <= 4 {
                    t[e + 8..e + 8 + size].to_vec()
                } else {
                    let off = read_u32(t, e + 8) as usize;
                    t[off..off + size].to_vec()
                }
            })
        })
    }

    #[test]
    fn the_fields_can_be_read_back() {
        let d = Details {
            title: "Informe – año".into(),
            app: "Notepad".into(),
            url: "https://example.com".into(),
            taken: "2026:11:13 01:52:50".into(),
            offset: "-04:00".into(),
        };
        let t = tiff(&d);
        assert_eq!(&t[..4], b"II\x2A\x00");
        let ifd0 = read_u32(&t, 4) as usize;
        assert_eq!(value(&t, ifd0, 0x0131).unwrap(), b"XcreenShot\0");
        assert_eq!(value(&t, ifd0, 0x010E).unwrap(), "Informe – año\0".as_bytes());
        let xp = value(&t, ifd0, 0x9C9B).unwrap();
        let units: Vec<u16> = xp.chunks(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
        assert_eq!(
            String::from_utf16_lossy(&units[..units.len() - 1]),
            "Informe – año"
        );
        let exif = read_u32(&value(&t, ifd0, 0x8769).unwrap(), 0) as usize;
        assert_eq!(value(&t, exif, 0x9003).unwrap(), b"2026:11:13 01:52:50\0");
        assert_eq!(value(&t, exif, 0x9011).unwrap(), b"-04:00\0");
        // Tags in order, as TIFF asks.
        let n = read_u16(&t, ifd0) as usize;
        let tags: Vec<u16> = (0..n).map(|i| read_u16(&t, ifd0 + 2 + 12 * i)).collect();
        assert!(tags.windows(2).all(|w| w[0] < w[1]), "{tags:?}");
    }

    #[test]
    fn the_segment_goes_after_the_jfif_header() {
        let jpeg = [0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x04, 0xAA, 0xBB, 0xFF, 0xDA, 0x01];
        let out = add_to_jpeg(&jpeg, &Details::default());
        assert_eq!(&out[..8], &jpeg[..8]);
        assert_eq!(&out[8..10], &[0xFF, 0xE1]);
        let len = u16::from_be_bytes([out[10], out[11]]) as usize;
        assert_eq!(&out[12..18], b"Exif\0\0");
        assert_eq!(&out[10 + len..], &jpeg[8..]);
        // Not a JPEG: unchanged.
        assert_eq!(add_to_jpeg(b"nope", &Details::default()), b"nope");
    }
}
