//! SVG files as the source (CAP-040): drawn by the system's web engine, in the same
//! private, off-screen view as Web page (CAP-037), at the size the file says, with its
//! transparency. Small drawings are drawn at twice their size, so they stay sharp.

use std::path::Path;

use base64::Engine as _;
use image::RgbaImage;
use serde_json::json;
use xs_core::Core;

/// Largest side drawn, in CSS pixels (the view's widest).
const MOST: f64 = 3840.0;
/// What a browser gives an SVG that says no size.
const DEFAULT: (f64, f64) = (300.0, 150.0);

/// A length of the root element in CSS pixels; percentages and unknown units: none.
fn length(value: &str) -> Option<f64> {
    let v = value.trim();
    let split = v
        .find(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-' || c == '+' || c == 'e' || c == 'E'))
        .unwrap_or(v.len());
    let (number, unit) = v.split_at(split);
    let n: f64 = number.parse().ok()?;
    let factor = match unit.trim() {
        "" | "px" => 1.0,
        "pt" => 4.0 / 3.0,
        "pc" => 16.0,
        "in" => 96.0,
        "cm" => 96.0 / 2.54,
        "mm" => 96.0 / 25.4,
        "em" => 16.0,
        _ => return None,
    };
    (n > 0.0 && n.is_finite()).then_some(n * factor)
}

/// The value of attribute `name` in `tag` (the root's start tag).
fn attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let mut rest = tag;
    while let Some(at) = rest.find(name) {
        let before = rest[..at].chars().last();
        let after = &rest[at + name.len()..];
        let after_trim = after.trim_start();
        if before.is_some_and(char::is_whitespace) && after_trim.starts_with('=') {
            let value = after_trim[1..].trim_start();
            let quote = value.chars().next()?;
            if quote == '"' || quote == '\'' {
                let end = value[1..].find(quote)?;
                return Some(&value[1..1 + end]);
            }
        }
        rest = &rest[at + name.len()..];
    }
    None
}

/// The size an SVG asks for, in CSS pixels: its width and height, or its viewBox, or
/// the browsers' 300 × 150.
pub fn size(svg: &str) -> (f64, f64) {
    let Some(start) = svg.find("<svg") else {
        return DEFAULT;
    };
    let tag = &svg[start..];
    let tag = &tag[..tag.find('>').unwrap_or(tag.len())];
    let w = attribute(tag, "width").and_then(length);
    let h = attribute(tag, "height").and_then(length);
    let view = attribute(tag, "viewBox").and_then(|v| {
        let n: Vec<f64> = v
            .split(|c: char| c.is_whitespace() || c == ',')
            .filter(|s| !s.is_empty())
            .filter_map(|s| s.parse().ok())
            .collect();
        (n.len() == 4 && n[2] > 0.0 && n[3] > 0.0).then(|| (n[2], n[3]))
    });
    match (w, h, view) {
        (Some(w), Some(h), _) => (w, h),
        (Some(w), None, Some((vw, vh))) => (w, w * vh / vw),
        (None, Some(h), Some((vw, vh))) => (h * vw / vh, h),
        (None, None, Some(v)) => v,
        (Some(w), None, None) => (w, DEFAULT.1),
        (None, Some(h), None) => (DEFAULT.0, h),
        (None, None, None) => DEFAULT,
    }
}

/// The drawing size (CSS pixels) and scale for an SVG of `size`.
pub fn plan((w, h): (f64, f64)) -> (u32, u32, f64) {
    let fit = (MOST / w.max(h)).min(1.0);
    let (w, h) = ((w * fit).round().max(1.0), (h * fit).round().max(1.0));
    let scale = if w.max(h) * 2.0 <= MOST { 2.0 } else { 1.0 };
    (w as u32, h as u32, scale)
}

fn file_url(path: &Path) -> String {
    let mut out = String::from("file:///");
    for c in path.to_string_lossy().replace('\\', "/").chars() {
        match c {
            ' ' => out.push_str("%20"),
            '%' => out.push_str("%25"),
            '#' => out.push_str("%23"),
            '?' => out.push_str("%3F"),
            c => out.push(c),
        }
    }
    out
}

/// Draws the SVG at `path`: the image and its scale (2 for small drawings).
pub fn render(core: &Core, path: &Path) -> Result<(RgbaImage, f32), String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let text = String::from_utf8_lossy(&bytes);
    if !text.contains("<svg") {
        return Err("not an SVG".into());
    }
    let (w, h, scale) = plan(size(&text));
    // A page with only the drawing, at its size, over nothing.
    let data = base64::engine::general_purpose::STANDARD.encode(&bytes);
    let html = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><style>html,body{{margin:0;background:transparent}}img{{display:block}}</style></head><body><img src=\"data:image/svg+xml;base64,{data}\" width=\"{w}\" height=\"{h}\"></body></html>"
    );
    let tmp = core.paths.local_dir.join("tmp");
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    let page = tmp.join(format!(
        "svg-{}.html",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::write(&page, html).map_err(|e| e.to_string())?;
    let shot = core.host().load_webpage(&json!({
        "url": file_url(&page),
        "width": w,
        "scale": scale,
        "wait": 0.2,
        "lazy": false,
        "transparent": true,
    }));
    let _ = std::fs::remove_file(&page);
    let shot = shot?;
    let png = base64::engine::general_purpose::STANDARD
        .decode(shot["png"].as_str().unwrap_or_default())
        .map_err(|e| e.to_string())?;
    let image = xs_image::encode::decode(&png)?;
    // The view is at least 320 wide and one screen high: keep the drawing only.
    let (cw, ch) = (
        ((w as f64 * scale).round() as u32).min(image.width()),
        ((h as f64 * scale).round() as u32).min(image.height()),
    );
    let image = image::imageops::crop_imm(&image, 0, 0, cw, ch).to_image();
    Ok((image, scale as f32))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_size_comes_from_the_root_element() {
        assert_eq!(size(r#"<svg width="120" height="80">"#), (120.0, 80.0));
        assert_eq!(
            size(r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 12">"#),
            (24.0, 12.0)
        );
        assert_eq!(size(r#"<svg width="48px" viewBox="0,0,24,12">"#), (48.0, 24.0));
        assert_eq!(size(r#"<svg width="1in" height="72pt">"#), (96.0, 96.0));
        // Percentages say nothing: the viewBox does.
        assert_eq!(
            size(r#"<svg width="100%" height="100%" viewBox="0 0 640 480">"#),
            (640.0, 480.0)
        );
        assert_eq!(size(r#"<?xml version="1.0"?><svg>"#), DEFAULT);
        // `stroke-width` is not `width`.
        assert_eq!(
            size(r#"<svg stroke-width="3" viewBox="0 0 10 20">"#),
            (10.0, 20.0)
        );
    }

    #[test]
    fn small_drawings_double_and_big_ones_fit() {
        assert_eq!(plan((24.0, 12.0)), (24, 12, 2.0));
        assert_eq!(plan((3000.0, 1000.0)), (3000, 1000, 1.0));
        assert_eq!(plan((7680.0, 100.0)), (3840, 50, 1.0));
    }

    #[test]
    fn paths_become_file_addresses() {
        assert_eq!(
            file_url(Path::new(r"C:\Users\a b\tmp\svg#1.html")),
            "file:///C:/Users/a%20b/tmp/svg%231.html"
        );
    }
}
