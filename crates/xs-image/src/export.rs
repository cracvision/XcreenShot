//! Export formats and their options (EDT-110 to EDT-112), and pages for PDF (EDT-102).
//! GIF, PDF and PSD are written here; TIFF, BMP and ICO use the codecs the project
//! already has. Nothing here needs a library of its own.

use std::collections::HashMap;
use std::io::{Cursor, Write};

use image::{imageops, ImageEncoder, RgbaImage};
use serde::Deserialize;

use crate::color::Rgba;

/// A file format to export to (EDT-110, EDT-111).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Png,
    Jpg,
    Gif,
    Bmp,
    Tiff,
    Pdf,
    Ico,
    Psd,
}

impl Kind {
    pub fn parse(s: &str) -> Option<Kind> {
        Some(match s.to_ascii_lowercase().as_str() {
            "png" => Kind::Png,
            "jpg" | "jpeg" => Kind::Jpg,
            "gif" => Kind::Gif,
            "bmp" => Kind::Bmp,
            "tif" | "tiff" => Kind::Tiff,
            "pdf" => Kind::Pdf,
            "ico" => Kind::Ico,
            "psd" => Kind::Psd,
            _ => return None,
        })
    }

    pub fn ext(self) -> &'static str {
        match self {
            Kind::Png => "png",
            Kind::Jpg => "jpg",
            Kind::Gif => "gif",
            Kind::Bmp => "bmp",
            Kind::Tiff => "tif",
            Kind::Pdf => "pdf",
            Kind::Ico => "ico",
            Kind::Psd => "psd",
        }
    }

    /// Formats that hold several images in one file (EDT-111).
    pub fn multi_page(self) -> bool {
        matches!(self, Kind::Tiff | Kind::Pdf)
    }
}

/// Options of each format (EDT-112). Unknown ones are ignored, so the same object can
/// carry the options of several formats.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Options {
    /// JPG quality, and PDF's when not lossless: 1–100.
    pub quality: u8,
    /// PNG: "fast" or "max".
    pub png_compression: String,
    /// PNG: an indexed image of at most 256 colors.
    pub png_colors256: bool,
    /// GIF: transparent pixels stay transparent (else they take the background).
    pub gif_transparent: bool,
    /// TIFF: "none", "lzw" or "zip".
    pub tiff_compression: String,
    /// PDF page (EDT-102): "auto" (the image's own size), "a4", "letter", "legal", "a3".
    pub pdf_page: String,
    /// "auto", "portrait" or "landscape".
    pub pdf_orientation: String,
    /// Margins in millimeters.
    pub pdf_margin: f64,
    /// Fit to the page width (else the image's own size, never wider than the page).
    pub pdf_fit_width: bool,
    /// The whole image on one page, made smaller.
    pub pdf_single_page: bool,
    /// Pictures without loss (else JPG at `quality`).
    pub pdf_lossless: bool,
    /// ICO sizes, 16 to 256 px.
    pub ico_sizes: Vec<u32>,
    /// All: 25–400 %.
    pub scale: f64,
    /// PNG: mark the colors as sRGB.
    pub color_profile: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            quality: 90,
            png_compression: "fast".into(),
            png_colors256: false,
            gif_transparent: true,
            tiff_compression: "lzw".into(),
            pdf_page: "auto".into(),
            pdf_orientation: "auto".into(),
            pdf_margin: 10.0,
            pdf_fit_width: true,
            pdf_single_page: false,
            pdf_lossless: true,
            ico_sizes: vec![16, 32, 48, 256],
            scale: 100.0,
            color_profile: true,
        }
    }
}

impl Options {
    pub fn check(&self) -> Result<(), String> {
        if !(1..=100).contains(&self.quality) {
            return Err("quality must be between 1 and 100".into());
        }
        if !(25.0..=400.0).contains(&self.scale) {
            return Err("scale must be between 25 and 400 %".into());
        }
        if !["fast", "max"].contains(&self.png_compression.as_str()) {
            return Err("pngCompression is fast or max".into());
        }
        if !["none", "lzw", "zip"].contains(&self.tiff_compression.as_str()) {
            return Err("tiffCompression is none, lzw or zip".into());
        }
        if !["auto", "a4", "letter", "legal", "a3"].contains(&self.pdf_page.as_str()) {
            return Err("pdfPage is auto, a4, letter, legal or a3".into());
        }
        if !["auto", "portrait", "landscape"].contains(&self.pdf_orientation.as_str()) {
            return Err("pdfOrientation is auto, portrait or landscape".into());
        }
        if !(0.0..=100.0).contains(&self.pdf_margin) {
            return Err("pdfMargin must be between 0 and 100 mm".into());
        }
        if self.ico_sizes.is_empty() || self.ico_sizes.iter().any(|s| !(16..=256).contains(s)) {
            return Err("icoSizes are 16 to 256 px".into());
        }
        Ok(())
    }
}

/// One image to export, with its resolution.
pub struct Page {
    pub image: RgbaImage,
    pub dpi: f64,
}

/// Encodes `pages` as `kind`. Single-image formats take the first one. `background`
/// fills transparency in formats without it (JPG, BMP, PDF; EDT-112).
pub fn encode(pages: &[Page], kind: Kind, o: &Options, background: Rgba) -> Result<Vec<u8>, String> {
    o.check()?;
    let scaled: Vec<Page> = pages
        .iter()
        .map(|p| Page {
            image: scaled(&p.image, o.scale),
            dpi: p.dpi,
        })
        .collect();
    let first = scaled.first().ok_or("nothing to export")?;
    match kind {
        Kind::Png => {
            if o.png_colors256 {
                indexed_png(&first.image, first.dpi)
            } else {
                png_with(
                    &first.image,
                    first.dpi,
                    o.png_compression == "max",
                    o.color_profile,
                )
            }
        }
        Kind::Jpg => crate::encode::jpg(&first.image, first.dpi, o.quality, background),
        Kind::Gif => gif(&first.image, o.gif_transparent, background),
        Kind::Bmp => {
            let rgb = flatten(&first.image, background);
            let mut out = Vec::new();
            image::codecs::bmp::BmpEncoder::new(&mut out)
                .write_image(
                    rgb.as_raw(),
                    rgb.width(),
                    rgb.height(),
                    image::ExtendedColorType::Rgb8,
                )
                .map_err(|e| e.to_string())?;
            Ok(out)
        }
        Kind::Tiff => tiff(&scaled, &o.tiff_compression),
        Kind::Pdf => pdf(&scaled, o, background),
        Kind::Ico => ico(&first.image, &o.ico_sizes),
        Kind::Psd => Ok(psd(&first.image)),
    }
}

fn scaled(img: &RgbaImage, percent: f64) -> RgbaImage {
    if (percent - 100.0).abs() < 1e-9 {
        return img.clone();
    }
    let k = percent / 100.0;
    let w = ((img.width() as f64 * k).round() as u32).max(1);
    let h = ((img.height() as f64 * k).round() as u32).max(1);
    imageops::resize(img, w, h, imageops::FilterType::Lanczos3)
}

/// Straight RGB over `bg`.
fn flatten(img: &RgbaImage, bg: Rgba) -> image::RgbImage {
    image::RgbImage::from_fn(img.width(), img.height(), |x, y| {
        let [r, g, b, a] = img.get_pixel(x, y).0;
        let a = a as u32;
        let mix = |c: u8, k: u8| ((c as u32 * a + k as u32 * (255 - a) + 127) / 255) as u8;
        image::Rgb([mix(r, bg.r), mix(g, bg.g), mix(b, bg.b)])
    })
}

fn png_with(img: &RgbaImage, dpi: f64, max: bool, srgb: bool) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, img.width(), img.height());
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_compression(if max {
            png::Compression::Best
        } else {
            png::Compression::Fast
        });
        set_dpi(&mut enc, dpi);
        if srgb {
            enc.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
        }
        let mut w = enc.write_header().map_err(|e| e.to_string())?;
        w.write_image_data(img.as_raw()).map_err(|e| e.to_string())?;
    }
    Ok(out)
}

fn set_dpi<W: Write>(enc: &mut png::Encoder<W>, dpi: f64) {
    if dpi > 0.0 {
        let ppm = (dpi / 0.0254).round() as u32;
        enc.set_pixel_dims(Some(png::PixelDimensions {
            xppu: ppm,
            yppu: ppm,
            unit: png::Unit::Meter,
        }));
    }
}

// ---------------------------------------------------------------------------
// 256 colors: median cut over a 15-bit histogram, then a lookup table.

/// A palette of at most `max` colors for `img`'s opaque-enough pixels.
fn palette(img: &RgbaImage, max: usize) -> Vec<[u8; 3]> {
    let mut hist: HashMap<u16, u32> = HashMap::new();
    for p in img.pixels().filter(|p| p.0[3] >= 128) {
        let key = ((p.0[0] as u16 >> 3) << 10) | ((p.0[1] as u16 >> 3) << 5) | (p.0[2] as u16 >> 3);
        *hist.entry(key).or_default() += 1;
    }
    let mut boxes: Vec<Vec<([u8; 3], u32)>> = vec![hist
        .into_iter()
        .map(|(k, n)| {
            let c = [((k >> 10) & 31) as u8, ((k >> 5) & 31) as u8, (k & 31) as u8];
            (c, n)
        })
        .collect()];
    if boxes[0].is_empty() {
        return vec![[0, 0, 0]];
    }
    while boxes.len() < max {
        // Split the box with the widest channel range at its median (by count).
        let Some((i, ch)) = boxes
            .iter()
            .enumerate()
            .filter(|(_, b)| b.len() > 1)
            .map(|(i, b)| {
                let (ch, range) = (0..3)
                    .map(|c| {
                        let lo = b.iter().map(|e| e.0[c]).min().unwrap_or(0);
                        let hi = b.iter().map(|e| e.0[c]).max().unwrap_or(0);
                        (c, hi - lo)
                    })
                    .max_by_key(|e| e.1)
                    .unwrap_or((0, 0));
                (i, ch, range)
            })
            .max_by_key(|e| e.2)
            .map(|(i, ch, _)| (i, ch))
        else {
            break;
        };
        let mut b = boxes.swap_remove(i);
        b.sort_by_key(|e| e.0[ch]);
        let total: u64 = b.iter().map(|e| e.1 as u64).sum();
        let mut acc = 0u64;
        let mut cut = 1;
        for (j, e) in b.iter().enumerate() {
            acc += e.1 as u64;
            if acc * 2 >= total {
                cut = (j + 1).clamp(1, b.len() - 1);
                break;
            }
        }
        let rest = b.split_off(cut);
        boxes.push(b);
        boxes.push(rest);
    }
    boxes
        .iter()
        .map(|b| {
            let total: u64 = b.iter().map(|e| e.1 as u64).sum::<u64>().max(1);
            let avg = |c: usize| {
                let s: u64 = b.iter().map(|e| (e.0[c] as u64 * 8 + 4) * e.1 as u64).sum();
                (s / total).min(255) as u8
            };
            [avg(0), avg(1), avg(2)]
        })
        .collect()
}

/// For each 15-bit color, the nearest palette entry.
fn lookup(pal: &[[u8; 3]]) -> Vec<u8> {
    (0..32768u32)
        .map(|k| {
            let c = [
                ((k >> 10) & 31) * 8 + 4,
                ((k >> 5) & 31) * 8 + 4,
                (k & 31) * 8 + 4,
            ];
            pal.iter()
                .enumerate()
                .min_by_key(|(_, p)| {
                    (0..3)
                        .map(|i| {
                            let d = c[i] as i32 - p[i] as i32;
                            d * d
                        })
                        .sum::<i32>()
                })
                .map(|(i, _)| i as u8)
                .unwrap_or(0)
        })
        .collect()
}

fn index_of(table: &[u8], p: &[u8; 4]) -> u8 {
    table[(((p[0] as usize) >> 3) << 10) | (((p[1] as usize) >> 3) << 5) | ((p[2] as usize) >> 3)]
}

/// An indexed PNG of at most 256 colors; transparency through its tRNS chunk
/// (EDT-112).
fn indexed_png(img: &RgbaImage, dpi: f64) -> Result<Vec<u8>, String> {
    let transparent = img.pixels().any(|p| p.0[3] < 128);
    let mut pal = palette(img, if transparent { 255 } else { 256 });
    let table = lookup(&pal);
    let clear = pal.len() as u8;
    if transparent {
        pal.push([0, 0, 0]);
    }
    let data: Vec<u8> = img
        .pixels()
        .map(|p| {
            if p.0[3] < 128 {
                clear
            } else {
                index_of(&table, &p.0)
            }
        })
        .collect();
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, img.width(), img.height());
        enc.set_color(png::ColorType::Indexed);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_palette(pal.iter().flatten().copied().collect::<Vec<u8>>());
        if transparent {
            let mut trns = vec![255u8; pal.len()];
            trns[clear as usize] = 0;
            enc.set_trns(trns);
        }
        enc.set_compression(png::Compression::Best);
        set_dpi(&mut enc, dpi);
        let mut w = enc.write_header().map_err(|e| e.to_string())?;
        w.write_image_data(&data).map_err(|e| e.to_string())?;
    }
    Ok(out)
}

/// A still GIF89a: at most 256 colors, one of them transparent when kept.
fn gif(img: &RgbaImage, keep_transparent: bool, bg: Rgba) -> Result<Vec<u8>, String> {
    let src = if keep_transparent {
        img.clone()
    } else {
        let flat = flatten(img, bg);
        RgbaImage::from_fn(img.width(), img.height(), |x, y| {
            let [r, g, b] = flat.get_pixel(x, y).0;
            image::Rgba([r, g, b, 255])
        })
    };
    let transparent = src.pixels().any(|p| p.0[3] < 128);
    let mut pal = palette(&src, if transparent { 255 } else { 256 });
    let table = lookup(&pal);
    let clear = pal.len() as u8;
    if transparent {
        pal.push([0, 0, 0]);
    }
    // The table holds 2^n colors.
    let bits = (1..=8).find(|b| (1usize << b) >= pal.len()).unwrap_or(8);
    pal.resize(1 << bits, [0, 0, 0]);
    let data: Vec<u8> = src
        .pixels()
        .map(|p| {
            if p.0[3] < 128 {
                clear
            } else {
                index_of(&table, &p.0)
            }
        })
        .collect();
    let (w, h) = (src.width().min(65535) as u16, src.height().min(65535) as u16);
    let mut out = Vec::new();
    out.extend_from_slice(b"GIF89a");
    out.extend_from_slice(&w.to_le_bytes());
    out.extend_from_slice(&h.to_le_bytes());
    out.push(0x80 | ((bits as u8 - 1) << 4) | (bits as u8 - 1));
    out.extend_from_slice(&[0, 0]);
    for c in &pal {
        out.extend_from_slice(c);
    }
    if transparent {
        // Graphic control extension: which index is transparent.
        out.extend_from_slice(&[0x21, 0xF9, 4, 0x01, 0, 0, clear, 0]);
    }
    out.push(0x2C);
    out.extend_from_slice(&[0, 0, 0, 0]);
    out.extend_from_slice(&w.to_le_bytes());
    out.extend_from_slice(&h.to_le_bytes());
    out.push(0);
    let min_code = bits.max(2) as u8;
    out.push(min_code);
    let lzw = weezl::encode::Encoder::new(weezl::BitOrder::Lsb, min_code)
        .encode(&data)
        .map_err(|e| e.to_string())?;
    for chunk in lzw.chunks(255) {
        out.push(chunk.len() as u8);
        out.extend_from_slice(chunk);
    }
    out.push(0);
    out.push(0x3B);
    Ok(out)
}

fn tiff(pages: &[Page], compression: &str) -> Result<Vec<u8>, String> {
    use tiff::encoder::{colortype, Compression, DeflateLevel, Rational, TiffEncoder};
    let mut out = Cursor::new(Vec::new());
    {
        let comp = match compression {
            "none" => Compression::Uncompressed,
            "zip" => Compression::Deflate(DeflateLevel::Balanced),
            _ => Compression::Lzw,
        };
        let mut enc = TiffEncoder::new(&mut out)
            .map_err(|e| e.to_string())?
            .with_compression(comp);
        for p in pages {
            let mut img = enc
                .new_image::<colortype::RGBA8>(p.image.width(), p.image.height())
                .map_err(|e| e.to_string())?;
            let dpi = p.dpi.round().max(1.0) as u32;
            img.resolution(tiff::tags::ResolutionUnit::Inch, Rational { n: dpi, d: 1 });
            img.write_data(p.image.as_raw()).map_err(|e| e.to_string())?;
        }
    }
    Ok(out.into_inner())
}

/// Several sizes in one ICO, each a PNG; the picture fits a square, centered.
fn ico(img: &RgbaImage, sizes: &[u32]) -> Result<Vec<u8>, String> {
    let mut sizes = sizes.to_vec();
    sizes.sort_unstable();
    sizes.dedup();
    let mut frames = Vec::new();
    for s in sizes {
        let k = s as f64 / img.width().max(img.height()) as f64;
        let (w, h) = (
            ((img.width() as f64 * k).round() as u32).clamp(1, s),
            ((img.height() as f64 * k).round() as u32).clamp(1, s),
        );
        let small = imageops::resize(img, w, h, imageops::FilterType::Lanczos3);
        let mut square = RgbaImage::new(s, s);
        imageops::overlay(&mut square, &small, ((s - w) / 2) as i64, ((s - h) / 2) as i64);
        let png = png_with(&square, 0.0, true, false)?;
        frames.push(
            image::codecs::ico::IcoFrame::with_encoded(png, s, s, image::ExtendedColorType::Rgba8)
                .map_err(|e| e.to_string())?,
        );
    }
    let mut out = Vec::new();
    image::codecs::ico::IcoEncoder::new(&mut out)
        .encode_images(&frames)
        .map_err(|e| e.to_string())?;
    Ok(out)
}

/// A PSD with one layer holding the final image, and the same image as the
/// composite (EDT-111).
fn psd(img: &RgbaImage) -> Vec<u8> {
    let (w, h) = (img.width(), img.height());
    let plane = |c: usize| -> Vec<u8> { img.pixels().map(|p| p.0[c]).collect() };
    let mut out = Vec::new();
    let be16 = |v: u16| v.to_be_bytes();
    let be32 = |v: u32| v.to_be_bytes();
    // Header: 4 channels (RGB + alpha), 8 bits, RGB.
    out.extend_from_slice(b"8BPS");
    out.extend_from_slice(&be16(1));
    out.extend_from_slice(&[0; 6]);
    out.extend_from_slice(&be16(4));
    out.extend_from_slice(&be32(h));
    out.extend_from_slice(&be32(w));
    out.extend_from_slice(&be16(8));
    out.extend_from_slice(&be16(3));
    // No color mode data, no image resources.
    out.extend_from_slice(&be32(0));
    out.extend_from_slice(&be32(0));
    // Layer and mask information: one layer.
    let channel_len = 2 + w as usize * h as usize;
    let name = b"XcreenShot";
    let mut padded_name = vec![name.len() as u8];
    padded_name.extend_from_slice(name);
    while padded_name.len() % 4 != 0 {
        padded_name.push(0);
    }
    let mut record = Vec::new();
    record.extend_from_slice(&be32(0));
    record.extend_from_slice(&be32(0));
    record.extend_from_slice(&be32(h));
    record.extend_from_slice(&be32(w));
    record.extend_from_slice(&be16(4));
    for id in [-1i16, 0, 1, 2] {
        record.extend_from_slice(&id.to_be_bytes());
        record.extend_from_slice(&be32(channel_len as u32));
    }
    record.extend_from_slice(b"8BIMnorm");
    record.extend_from_slice(&[255, 0, 0, 0]);
    let extra_len = 4 + 4 + padded_name.len();
    record.extend_from_slice(&be32(extra_len as u32));
    record.extend_from_slice(&be32(0));
    record.extend_from_slice(&be32(0));
    record.extend_from_slice(&padded_name);
    let mut layer_info = Vec::new();
    layer_info.extend_from_slice(&be16(1));
    layer_info.extend_from_slice(&record);
    for c in [3usize, 0, 1, 2] {
        layer_info.extend_from_slice(&be16(0));
        layer_info.extend_from_slice(&plane(c));
    }
    if layer_info.len() % 2 != 0 {
        layer_info.push(0);
    }
    let mut section = Vec::new();
    section.extend_from_slice(&be32(layer_info.len() as u32));
    section.extend_from_slice(&layer_info);
    section.extend_from_slice(&be32(0));
    out.extend_from_slice(&be32(section.len() as u32));
    out.extend_from_slice(&section);
    // Composite image, raw: R, G, B, A planes.
    out.extend_from_slice(&be16(0));
    for c in 0..4 {
        out.extend_from_slice(&plane(c));
    }
    out
}

// ---------------------------------------------------------------------------
// PDF (EDT-102, EDT-111)

/// Page sizes in points, portrait.
fn page_size(name: &str) -> Option<(f64, f64)> {
    Some(match name {
        "a4" => (595.28, 841.89),
        "letter" => (612.0, 792.0),
        "legal" => (612.0, 1008.0),
        "a3" => (841.89, 1190.55),
        _ => return None,
    })
}

/// Where a page break goes near `end` (a row of the image): a row of one color
/// within `back` rows above it, so text is not cut through; else `end` (EDT-102).
pub fn break_row(img: &RgbaImage, start: u32, end: u32, back: u32) -> u32 {
    let uniform = |y: u32| {
        let first = img.get_pixel(0, y).0;
        img.rows()
            .nth(y as usize)
            .is_some_and(|mut row| row.all(|p| (0..3).all(|i| (p.0[i] as i32 - first[i] as i32).abs() <= 8)))
    };
    let lowest = end.saturating_sub(back).max(start + 1);
    (lowest..end)
        .rev()
        .find(|&y| uniform(y))
        .map(|y| y + 1)
        .unwrap_or(end)
}

struct PdfImage {
    width: u32,
    height: u32,
    filter: &'static str,
    data: Vec<u8>,
}

fn pdf_image(img: &RgbaImage, bg: Rgba, lossless: bool, quality: u8) -> Result<PdfImage, String> {
    let rgb = flatten(img, bg);
    if lossless {
        let mut z = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        z.write_all(rgb.as_raw()).map_err(|e| e.to_string())?;
        Ok(PdfImage {
            width: rgb.width(),
            height: rgb.height(),
            filter: "/FlateDecode",
            data: z.finish().map_err(|e| e.to_string())?,
        })
    } else {
        Ok(PdfImage {
            width: rgb.width(),
            height: rgb.height(),
            filter: "/DCTDecode",
            data: crate::encode::jpg(img, 0.0, quality, bg)?,
        })
    }
}

/// The pages of one image: each a slice of rows, its size on the page and the page.
struct Placed {
    image: RgbaImage,
    page: (f64, f64),
    at: (f64, f64),
    size: (f64, f64),
}

fn layout(p: &Page, o: &Options) -> Vec<Placed> {
    let dpi = if p.dpi > 0.0 { p.dpi } else { 96.0 };
    let pt = 72.0 / dpi;
    let (iw, ih) = (p.image.width() as f64 * pt, p.image.height() as f64 * pt);
    let Some((mut pw, mut ph)) = page_size(&o.pdf_page) else {
        // Auto: a page of the image's own size.
        return vec![Placed {
            image: p.image.clone(),
            page: (iw, ih),
            at: (0.0, 0.0),
            size: (iw, ih),
        }];
    };
    let landscape = match o.pdf_orientation.as_str() {
        "landscape" => true,
        "portrait" => false,
        _ => iw > ih,
    };
    if landscape {
        std::mem::swap(&mut pw, &mut ph);
    }
    let m = o.pdf_margin * 72.0 / 25.4;
    let (aw, ah) = ((pw - 2.0 * m).max(1.0), (ph - 2.0 * m).max(1.0));
    let k = if o.pdf_single_page {
        (aw / iw).min(ah / ih)
    } else if o.pdf_fit_width {
        aw / iw
    } else {
        (aw / iw).min(1.0)
    };
    // Rows of the image that fit on one page.
    let rows_per_page = ((ah / (k * pt)).floor() as u32).max(1);
    let mut out = Vec::new();
    let mut y = 0;
    while y < p.image.height() {
        let end = (y + rows_per_page).min(p.image.height());
        let cut = if end < p.image.height() {
            break_row(&p.image, y, end, rows_per_page / 6)
        } else {
            end
        };
        let slice = imageops::crop_imm(&p.image, 0, y, p.image.width(), cut - y).to_image();
        let size = (iw * k, (cut - y) as f64 * pt * k);
        out.push(Placed {
            image: slice,
            page: (pw, ph),
            at: (m + (aw - size.0) / 2.0, ph - m - size.1),
            size,
        });
        y = cut;
    }
    out
}

fn pdf(pages: &[Page], o: &Options, bg: Rgba) -> Result<Vec<u8>, String> {
    let placed: Vec<Placed> = pages.iter().flat_map(|p| layout(p, o)).collect();
    write_pdf(&placed, bg, o.pdf_lossless, o.quality)
}

/// A PDF of the printed pages (EDT-114): each paper with its picture where it goes,
/// without loss, for the system's PDF viewer to print.
pub fn print_pdf(pages: &[PrintPage], bg: Rgba) -> Result<Vec<u8>, String> {
    let pt = 72.0 / 25.4;
    let placed: Vec<Placed> = pages
        .iter()
        .map(|p| Placed {
            image: p.image.clone(),
            page: (p.paper.0 * pt, p.paper.1 * pt),
            // PDF measures from the bottom.
            at: (p.at.0 * pt, (p.paper.1 - p.at.1 - p.size.1) * pt),
            size: (p.size.0 * pt, p.size.1 * pt),
        })
        .collect();
    write_pdf(&placed, bg, true, 90)
}

fn write_pdf(placed: &[Placed], bg: Rgba, lossless: bool, quality: u8) -> Result<Vec<u8>, String> {
    let mut out: Vec<u8> = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets: Vec<usize> = Vec::new();
    let mut object = |out: &mut Vec<u8>, body: &[u8]| {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", offsets.len()).as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    };
    // 1: catalog, 2: pages; each page takes three objects (page, image, contents).
    let kids: Vec<String> = (0..placed.len()).map(|i| format!("{} 0 R", 3 + i * 3)).collect();
    object(&mut out, b"<< /Type /Catalog /Pages 2 0 R >>");
    object(
        &mut out,
        format!(
            "<< /Type /Pages /Kids [{}] /Count {} >>",
            kids.join(" "),
            placed.len()
        )
        .as_bytes(),
    );
    for (i, p) in placed.iter().enumerate() {
        let (img, content) = (4 + i * 3, 5 + i * 3);
        object(
            &mut out,
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {:.2} {:.2}] /Resources << /XObject << /Im{i} {img} 0 R >> >> /Contents {content} 0 R >>",
                p.page.0, p.page.1
            )
            .as_bytes(),
        );
        let pic = pdf_image(&p.image, bg, lossless, quality)?;
        let mut body = format!(
            "<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter {} /Length {} >>\nstream\n",
            pic.width,
            pic.height,
            pic.filter,
            pic.data.len()
        )
        .into_bytes();
        body.extend_from_slice(&pic.data);
        body.extend_from_slice(b"\nendstream");
        object(&mut out, &body);
        let draw = format!(
            "q {:.3} 0 0 {:.3} {:.3} {:.3} cm /Im{i} Do Q",
            p.size.0, p.size.1, p.at.0, p.at.1
        );
        let mut body = format!("<< /Length {} >>\nstream\n", draw.len()).into_bytes();
        body.extend_from_slice(draw.as_bytes());
        body.extend_from_slice(b"\nendstream");
        object(&mut out, &body);
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len() + 1).as_bytes());
    for off in &offsets {
        out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            offsets.len() + 1
        )
        .as_bytes(),
    );
    Ok(out)
}

// ---------------------------------------------------------------------------
// Printing (EDT-114): pages in millimeters, with the same breaks as PDF (EDT-102).

/// How to print: paper, orientation, margins, size and position (EDT-114).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PrintSetup {
    /// "a4", "letter", "legal" or "a3".
    pub paper: String,
    /// "auto", "portrait" or "landscape".
    pub orientation: String,
    /// Millimeters.
    pub margin: f64,
    /// "fit" (the whole image on the page), "actual" (by its resolution) or "percent".
    pub size: String,
    pub percent: f64,
    /// "center" or "topLeft".
    pub position: String,
}

impl Default for PrintSetup {
    fn default() -> Self {
        PrintSetup {
            paper: "a4".into(),
            orientation: "auto".into(),
            margin: 10.0,
            size: "fit".into(),
            percent: 100.0,
            position: "center".into(),
        }
    }
}

/// One printed page: its picture, the paper and where the picture goes, in mm from
/// the paper's top-left corner.
pub struct PrintPage {
    pub image: RgbaImage,
    pub paper: (f64, f64),
    pub at: (f64, f64),
    pub size: (f64, f64),
}

fn paper_mm(name: &str) -> (f64, f64) {
    match name {
        "letter" => (215.9, 279.4),
        "legal" => (215.9, 355.6),
        "a3" => (297.0, 420.0),
        _ => (210.0, 297.0),
    }
}

/// The pages to print `img` on (EDT-114): fitted to one page, or at its own size (or
/// a percentage) split into pages between lines of text when taller than the paper
/// (EDT-102). Never wider than the paper.
pub fn print_pages(img: &RgbaImage, dpi: f64, s: &PrintSetup) -> Vec<PrintPage> {
    let mm = 25.4 / if dpi > 0.0 { dpi } else { 96.0 };
    let (iw, ih) = (img.width() as f64 * mm, img.height() as f64 * mm);
    let (mut pw, mut ph) = paper_mm(&s.paper);
    let landscape = match s.orientation.as_str() {
        "landscape" => true,
        "portrait" => false,
        _ => iw > ih,
    };
    if landscape {
        std::mem::swap(&mut pw, &mut ph);
    }
    let m = s.margin.clamp(0.0, pw.min(ph) / 3.0);
    let (aw, ah) = (pw - 2.0 * m, ph - 2.0 * m);
    let k = match s.size.as_str() {
        "actual" => (aw / iw).min(1.0),
        "percent" => (aw / iw).min(s.percent.clamp(1.0, 1000.0) / 100.0),
        _ => (aw / iw).min(ah / ih),
    };
    let rows = ((ah / (k * mm)).floor() as u32).max(1);
    let mut out = Vec::new();
    let mut y = 0;
    while y < img.height() {
        let end = (y + rows).min(img.height());
        let cut = if end < img.height() {
            break_row(img, y, end, rows / 6)
        } else {
            end
        };
        let slice = imageops::crop_imm(img, 0, y, img.width(), cut - y).to_image();
        let size = (iw * k, (cut - y) as f64 * mm * k);
        let at = if s.position == "topLeft" || !out.is_empty() || cut < img.height() {
            // A page of a longer image starts at the top.
            (
                if s.position == "topLeft" {
                    m
                } else {
                    m + (aw - size.0) / 2.0
                },
                m,
            )
        } else {
            (m + (aw - size.0) / 2.0, m + (ah - size.1) / 2.0)
        };
        out.push(PrintPage {
            image: slice,
            paper: (pw, ph),
            at,
            size,
        });
        y = cut;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(w: u32, h: u32) -> RgbaImage {
        RgbaImage::from_fn(w, h, |x, y| {
            if (x / 10 + y / 10) % 2 == 0 {
                image::Rgba([200, 30, 30, 255])
            } else {
                image::Rgba([20, 90, 220, 255])
            }
        })
    }

    fn page(img: RgbaImage) -> Page {
        Page {
            image: img,
            dpi: 96.0,
        }
    }

    fn opts(v: serde_json::Value) -> Options {
        serde_json::from_value(v).unwrap()
    }

    #[test]
    fn every_format_writes_its_signature() {
        let p = [page(sample(64, 48))];
        let o = Options::default();
        let starts = |k: Kind, sig: &[u8]| {
            let out = encode(&p, k, &o, Rgba::WHITE).unwrap();
            assert!(out.starts_with(sig), "{k:?}");
            out
        };
        starts(Kind::Png, b"\x89PNG");
        starts(Kind::Jpg, b"\xFF\xD8");
        starts(Kind::Gif, b"GIF89a");
        starts(Kind::Bmp, b"BM");
        starts(Kind::Tiff, b"II*\0");
        starts(Kind::Pdf, b"%PDF-1.4");
        starts(Kind::Ico, b"\0\0\x01\0");
        starts(Kind::Psd, b"8BPS");
        // The image codecs read back what is theirs.
        for k in [Kind::Png, Kind::Bmp, Kind::Tiff, Kind::Ico] {
            let out = encode(&p, k, &o, Rgba::WHITE).unwrap();
            let back = image::load_from_memory(&out).unwrap();
            assert!(back.width() > 0, "{k:?}");
        }
    }

    #[test]
    fn jpg_quality_changes_the_size() {
        let p = [page(sample(200, 150))];
        let low = encode(
            &p,
            Kind::Jpg,
            &opts(serde_json::json!({"quality": 50})),
            Rgba::WHITE,
        )
        .unwrap();
        let high = encode(
            &p,
            Kind::Jpg,
            &opts(serde_json::json!({"quality": 95})),
            Rgba::WHITE,
        )
        .unwrap();
        assert!(low.len() * 10 < high.len() * 8, "{} vs {}", low.len(), high.len());
    }

    #[test]
    fn png_with_256_colors_is_indexed() {
        let out = encode(
            &[page(sample(80, 60))],
            Kind::Png,
            &opts(serde_json::json!({"pngColors256": true})),
            Rgba::WHITE,
        )
        .unwrap();
        let reader = png::Decoder::new(Cursor::new(&out)).read_info().unwrap();
        assert_eq!(reader.info().color_type, png::ColorType::Indexed);
    }

    #[test]
    fn ico_holds_each_size() {
        let out = encode(
            &[page(sample(300, 200))],
            Kind::Ico,
            &opts(serde_json::json!({"icoSizes": [16, 32, 256]})),
            Rgba::WHITE,
        )
        .unwrap();
        assert_eq!(u16::from_le_bytes([out[4], out[5]]), 3, "three images");
        // Directory entries: width 16, 32 and 0 (= 256).
        assert_eq!([out[6], out[22], out[38]], [16, 32, 0]);
    }

    #[test]
    fn tiff_and_pdf_take_several_pages() {
        let pages = [page(sample(40, 30)), page(sample(50, 30)), page(sample(60, 30))];
        let tif = encode(&pages, Kind::Tiff, &Options::default(), Rgba::WHITE).unwrap();
        let mut dec = tiff::decoder::Decoder::new(Cursor::new(&tif)).unwrap();
        let mut n = 1;
        while dec.more_images() {
            dec.next_image().unwrap();
            n += 1;
        }
        assert_eq!(n, 3);
        let pdf = encode(&pages, Kind::Pdf, &Options::default(), Rgba::WHITE).unwrap();
        let text = String::from_utf8_lossy(&pdf);
        assert_eq!(text.matches("/Type /Page ").count(), 3);
        assert!(text.contains("/Count 3"));
    }

    #[test]
    fn a_long_image_splits_into_pages_between_lines() {
        // Text-like lines: 12 rows of "letters" (dark and light), 8 white rows.
        let img = RgbaImage::from_fn(1200, 9000, |x, y| {
            if y % 20 < 12 && (x / 3) % 2 == 0 {
                image::Rgba([30, 30, 30, 255])
            } else {
                image::Rgba([255, 255, 255, 255])
            }
        });
        let p = Page {
            image: img,
            dpi: 96.0,
        };
        let o = opts(serde_json::json!({"pdfPage": "a4"}));
        let placed = layout(&p, &o);
        assert!(placed.len() > 1, "several pages");
        // Every page ends on a white row: no line is cut through.
        let mut y = 0;
        for slice in &placed[..placed.len() - 1] {
            y += slice.image.height();
            assert!((y - 1) % 20 >= 12, "cut at row {y}");
        }
        let one = layout(
            &p,
            &opts(serde_json::json!({"pdfPage": "a4", "pdfSinglePage": true})),
        );
        assert_eq!(one.len(), 1);
    }

    #[test]
    fn psd_has_one_layer() {
        let out = psd(&sample(10, 8));
        assert_eq!(&out[0..4], b"8BPS");
        assert_eq!(u16::from_be_bytes([out[12], out[13]]), 4, "RGBA");
        // After the header (26), color mode (4) and resources (4): the layer section;
        // its layer info begins with the layer count.
        assert_eq!(i16::from_be_bytes([out[42], out[43]]), 1);
    }

    #[test]
    fn options_are_checked() {
        assert!(opts(serde_json::json!({"scale": 500})).check().is_err());
        assert!(opts(serde_json::json!({"icoSizes": [8]})).check().is_err());
        assert!(Options::default().check().is_ok());
    }

    #[test]
    fn printing_fits_one_page_or_splits() {
        // 1920 × 1080 fitted to A4 landscape: one page, centered.
        let img = sample(1920, 1080);
        let one = print_pages(&img, 96.0, &PrintSetup::default());
        assert_eq!(one.len(), 1);
        assert!(one[0].paper.0 > one[0].paper.1, "landscape");
        assert!(
            (one[0].at.0 * 2.0 + one[0].size.0 - one[0].paper.0).abs() < 0.01,
            "centered"
        );
        // A tall image at its own size: several pages from the top.
        let tall = sample(800, 6000);
        let setup = PrintSetup {
            size: "actual".into(),
            ..PrintSetup::default()
        };
        let pages = print_pages(&tall, 96.0, &setup);
        assert!(pages.len() > 1);
        assert_eq!(pages.iter().map(|p| p.image.height()).sum::<u32>(), 6000);
        assert!((pages[1].at.1 - 10.0).abs() < 0.01, "from the top margin");
    }
}
