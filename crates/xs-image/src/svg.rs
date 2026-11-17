//! SVG pictures drawn to pixels (EDT-032, EDT-034): a stamp made from an SVG file is
//! drawn once, large enough to stay sharp at the sizes stamps are used.

use image::RgbaImage;

/// The SVG `bytes` drawn with its longest side at most `max` px (and at least
/// `max / 4`, so tiny drawings are not blurry), with its transparency.
pub fn rasterize(bytes: &[u8], max: u32) -> Result<RgbaImage, String> {
    use resvg::{tiny_skia, usvg};
    let mut options = usvg::Options::default();
    options.fontdb_mut().load_system_fonts();
    let tree = usvg::Tree::from_data(bytes, &options).map_err(|e| e.to_string())?;
    let size = tree.size();
    let (w, h) = (size.width(), size.height());
    if !(w > 0.0 && h > 0.0) {
        return Err("the drawing has no size".into());
    }
    let longest = w.max(h);
    let k = (max as f32 / longest).min(4.0).max((max as f32 / 4.0) / longest);
    let (pw, ph) = ((w * k).round().max(1.0) as u32, (h * k).round().max(1.0) as u32);
    let mut pm = tiny_skia::Pixmap::new(pw, ph).ok_or("the drawing is too large")?;
    resvg::render(&tree, tiny_skia::Transform::from_scale(k, k), &mut pm.as_mut());
    let mut out = Vec::with_capacity(pm.data().len());
    for px in pm.pixels() {
        let c = px.demultiply();
        out.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    RgbaImage::from_raw(pw, ph, out).ok_or_else(|| "the drawing could not be read".into())
}

/// Whether `bytes` look like an SVG file (text with an `<svg` element).
pub fn is_svg(bytes: &[u8]) -> bool {
    let head = &bytes[..bytes.len().min(4096)];
    let text = String::from_utf8_lossy(head);
    text.contains("<svg")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_circle_keeps_its_transparency() {
        let svg = br##"<svg xmlns="http://www.w3.org/2000/svg" width="20" height="10">
            <circle cx="5" cy="5" r="5" fill="#ff0000"/></svg>"##;
        assert!(is_svg(svg));
        let img = rasterize(svg, 400).unwrap();
        assert_eq!(
            (img.width(), img.height()),
            (100, 50),
            "small drawings grow to a quarter of max"
        );
        assert_eq!(img.get_pixel(25, 25).0, [255, 0, 0, 255]);
        assert_eq!(img.get_pixel(80, 25).0[3], 0);
        assert!(rasterize(b"not svg", 100).is_err());
    }
}
