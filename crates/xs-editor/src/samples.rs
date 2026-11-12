//! Samples of saved styles (EDT-015): a small object of the tool in that style, drawn
//! by the same engine as the documents, for the Styles tab.

use image::RgbaImage;
use serde_json::{json, Map, Value};
use xs_core::document::{Document, Object};
use xs_image::Base;

/// Size of a sample, in pixels (shown at half size, for sharp screens).
pub const W: u32 = 128;
pub const H: u32 = 88;

/// A light page with a few lines of "text": what blurs, lenses and highlighters work
/// on.
fn page() -> RgbaImage {
    let mut img = RgbaImage::from_pixel(W, H, image::Rgba([246, 246, 248, 255]));
    for (i, y) in [18u32, 34, 50, 66].into_iter().enumerate() {
        let len = [96u32, 80, 104, 64][i];
        for yy in y..y + 6 {
            for x in 12..12 + len {
                img.put_pixel(x, yy, image::Rgba([90, 96, 110, 255]));
            }
        }
    }
    img
}

/// The object a sample of `tool` shows, with `style`.
fn object(tool: &str, style: &Value) -> Option<Map<String, Value>> {
    let (w, h) = (W as f64, H as f64);
    let mut style = style.clone();
    let v = match tool {
        "arrow" | "line" => json!({"from": [18, h - 18.0], "to": [w - 18.0, 18], "style": style}),
        "text" => {
            // Big enough to read, small enough to fit.
            let size = style["size"].as_f64().unwrap_or(24.0).min(30.0);
            style["size"] = json!(size);
            json!({"at": [16, 18], "width": null, "text": "Aa", "rotation": 0, "style": style})
        }
        "callout" => {
            let size = style["size"].as_f64().unwrap_or(18.0).min(18.0);
            style["size"] = json!(size);
            json!({"rect": {"x": 14, "y": 8, "w": 100, "h": 48}, "rotation": 0, "text": "Aa", "tails": [[34, 80]], "style": style})
        }
        "shape" => json!({"rect": {"x": 22, "y": 16, "w": 84, "h": 56}, "rotation": 0, "style": style}),
        "step" => {
            style["size"] = json!(52);
            json!({"rect": {"x": 38, "y": 18, "w": 52, "h": 52}, "rotation": 0, "value": "1", "style": style})
        }
        "highlight" | "blur" | "spotlight" => {
            json!({"rect": {"x": 22, "y": 14, "w": 84, "h": 44}, "rotation": 0, "style": style})
        }
        "pen" => {
            let points: Vec<[f64; 2]> = (0..=24)
                .map(|i| {
                    let t = i as f64 / 24.0;
                    [
                        16.0 + t * (w - 32.0),
                        h / 2.0 + (t * std::f64::consts::TAU).sin() * 18.0,
                    ]
                })
                .collect();
            json!({"points": points, "style": style})
        }
        "magnify" => json!({"source": {"x": 14, "y": 14, "w": 30, "h": 30}, "at": [86, 44], "style": style}),
        _ => return None,
    };
    v.as_object().cloned()
}

/// A saved effect's sample (EDT-054): a small page with the stack applied.
pub fn effect_sample(stack: &[Value]) -> Option<RgbaImage> {
    let mut img = page();
    // A little color, so filters and color changes show.
    for y in 10..30 {
        for x in 80..116 {
            img.put_pixel(x, y, image::Rgba([45, 127, 249, 255]));
        }
    }
    let meta = json!({"capturedAt": "2026-01-01T09:00:00+00:00", "app": {"name": "XcreenShot"}});
    let none = |_: &str| None;
    let ctx = xs_image::fx::Ctx {
        canvas_color: None,
        dpi: 96.0,
        meta: &meta,
        picture: &none,
    };
    let out = xs_image::fx::apply(stack, img, &ctx).image;
    // Back to the sample's size, keeping its proportions, on a transparent box.
    let (w, h) = out.dimensions();
    let k = (W as f64 / w as f64).min(H as f64 / h as f64).min(1.0);
    let (nw, nh) = (((w as f64 * k) as u32).max(1), ((h as f64 * k) as u32).max(1));
    let small = image::imageops::resize(&out, nw, nh, image::imageops::FilterType::Triangle);
    let mut boxed = RgbaImage::new(W, H);
    image::imageops::overlay(&mut boxed, &small, ((W - nw) / 2) as i64, ((H - nh) / 2) as i64);
    Some(boxed)
}

/// The sample of `tool` in `style`.
pub fn sample(tool: &str, style: &Value) -> Option<RgbaImage> {
    let props = object(tool, style)?;
    let mut doc = Document::new(W, H, 96.0);
    doc.objects.push(Object {
        id: "o1".into(),
        kind: tool.to_string(),
        props,
    });
    // Tools that work on what is below show the page; the rest a light plain one, so
    // dark and light colors both show in either theme of the app.
    let below = matches!(tool, "highlight" | "blur" | "spotlight" | "magnify");
    let base = if below {
        page()
    } else {
        RgbaImage::from_pixel(W, H, image::Rgba([228, 231, 237, 255]))
    };
    Some(xs_image::render(&doc, &Base::from_image(base)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::objects::Styles;

    #[test]
    fn every_styled_tool_has_a_sample() {
        let styles = Styles::default();
        for tool in crate::themes::STYLED_TOOLS {
            let style = styles.of(tool).unwrap();
            let img = sample(tool, &style).unwrap_or_else(|| panic!("{tool}"));
            assert!(img.pixels().any(|p| p.0[3] > 0), "{tool} draws something");
        }
    }
}
