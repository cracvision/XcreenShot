//! Reading and writing image files (EDT-105, EDT-110).

use image::RgbaImage;

use crate::color::Rgba;

/// Decodes a PNG or JPG (any format the `image` features allow) to straight RGBA.
pub fn decode(bytes: &[u8]) -> Result<RgbaImage, String> {
    image::load_from_memory(bytes)
        .map(|i| i.to_rgba8())
        .map_err(|e| e.to_string())
}

/// Width and height from a PNG header, without decoding the pixels.
pub fn png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
    let reader = decoder.read_info().ok()?;
    let info = reader.info();
    Some((info.width, info.height))
}

/// Resolution a PNG declares in its pHYs chunk (in dots per inch), if any.
pub fn png_dpi(bytes: &[u8]) -> Option<f64> {
    let reader = png::Decoder::new(std::io::Cursor::new(bytes)).read_info().ok()?;
    let dims = reader.info().pixel_dims?;
    (dims.unit == png::Unit::Meter && dims.xppu > 0).then(|| (dims.xppu as f64 * 0.0254).round())
}

/// PNG with transparency, the resolution (pHYs) and the sRGB mark (EDT-110).
/// `fast` trades size for speed (the `base.png` of a new capture).
pub fn png(image: &RgbaImage, dpi: f64, fast: bool) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    {
        let mut enc = png::Encoder::new(&mut out, image.width(), image.height());
        enc.set_color(png::ColorType::Rgba);
        enc.set_depth(png::BitDepth::Eight);
        enc.set_compression(if fast {
            png::Compression::Fast
        } else {
            png::Compression::Default
        });
        if dpi > 0.0 {
            let ppm = (dpi / 0.0254).round() as u32;
            enc.set_pixel_dims(Some(png::PixelDimensions {
                xppu: ppm,
                yppu: ppm,
                unit: png::Unit::Meter,
            }));
        }
        enc.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
        let mut writer = enc.write_header().map_err(|e| e.to_string())?;
        writer
            .write_image_data(image.as_raw())
            .map_err(|e| e.to_string())?;
    }
    Ok(out)
}

/// JPG: transparent areas over `background` (EDT-112 default: white), with the
/// resolution in its header.
pub fn jpg(image: &RgbaImage, dpi: f64, quality: u8, background: Rgba) -> Result<Vec<u8>, String> {
    let mut rgb = Vec::with_capacity((image.width() * image.height() * 3) as usize);
    for px in image.pixels() {
        let [r, g, b, a] = px.0;
        let a = a as u32;
        let mix = |c: u8, bg: u8| ((c as u32 * a + bg as u32 * (255 - a) + 127) / 255) as u8;
        rgb.extend_from_slice(&[mix(r, background.r), mix(g, background.g), mix(b, background.b)]);
    }
    let mut out = Vec::new();
    let mut enc = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut out, quality.clamp(1, 100));
    if dpi > 0.0 {
        let d = dpi.round().clamp(1.0, u16::MAX as f64) as u16;
        enc.set_pixel_density(image::codecs::jpeg::PixelDensity {
            density: (d, d),
            unit: image::codecs::jpeg::PixelDensityUnit::Inches,
        });
    }
    enc.encode(
        &rgb,
        image.width(),
        image.height(),
        image::ExtendedColorType::Rgb8,
    )
    .map_err(|e| e.to_string())?;
    Ok(out)
}

/// Scaled-down copy whose longest side is at most `max` (never enlarged).
pub fn thumbnail(image: &RgbaImage, max: u32) -> RgbaImage {
    let (w, h) = image.dimensions();
    let longest = w.max(h).max(1);
    if longest <= max {
        return image.clone();
    }
    let tw = ((w as u64 * max as u64) / longest as u64).max(1) as u32;
    let th = ((h as u64 * max as u64) / longest as u64).max(1) as u32;
    image::imageops::thumbnail(image, tw, th)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn half_transparent() -> RgbaImage {
        let mut img = RgbaImage::from_pixel(8, 4, image::Rgba([0, 0, 0, 0]));
        img.put_pixel(0, 0, image::Rgba([255, 0, 0, 255]));
        img
    }

    #[test]
    fn png_keeps_transparency_and_declares_the_resolution() {
        let bytes = png(&half_transparent(), 144.0, false).unwrap();
        let back = decode(&bytes).unwrap();
        assert_eq!(back, half_transparent());
        let reader = png::Decoder::new(std::io::Cursor::new(&bytes))
            .read_info()
            .unwrap();
        let dims = reader.info().pixel_dims.unwrap();
        // 144 dpi = 5669 pixels per meter.
        assert_eq!(dims.xppu, 5669);
        assert!(reader.info().srgb.is_some());
        assert_eq!(png_size(&bytes), Some((8, 4)));
        assert_eq!(png_dpi(&bytes), Some(144.0));
    }

    #[test]
    fn jpg_fills_transparency_with_the_background() {
        let bytes = jpg(&half_transparent(), 96.0, 95, Rgba::WHITE).unwrap();
        assert_eq!(&bytes[..2], &[0xFF, 0xD8]);
        let back = decode(&bytes).unwrap();
        let px = back.get_pixel(6, 3).0;
        assert!(px[0] > 245 && px[1] > 245 && px[2] > 245, "{px:?}");
    }

    #[test]
    fn thumbnails_fit_without_enlarging() {
        let big = RgbaImage::new(2000, 1000);
        assert_eq!(thumbnail(&big, 512).dimensions(), (512, 256));
        let small = RgbaImage::new(100, 50);
        assert_eq!(thumbnail(&small, 512).dimensions(), (100, 50));
    }
}
