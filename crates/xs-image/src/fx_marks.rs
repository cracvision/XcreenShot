//! Effects that write on the image (EDT-066, EDT-067): a watermark and the capture's
//! details.

use image::{imageops, Rgba as Px, RgbaImage};
use serde_json::Value;
use tiny_skia::{Pixmap, PixmapPaint, Transform};

use crate::color::Rgba;
use crate::fx::{over, Ctx};
use crate::text::{Text, TextAlign, TextStyle};

fn f(v: &Value, k: &str, d: f64) -> f64 {
    v.get(k).and_then(Value::as_f64).unwrap_or(d)
}
fn s<'a>(v: &'a Value, k: &str, d: &'a str) -> &'a str {
    v.get(k).and_then(Value::as_str).unwrap_or(d)
}

fn straight(pm: &Pixmap) -> RgbaImage {
    let mut out = Vec::with_capacity(pm.data().len());
    for px in pm.pixels() {
        let c = px.demultiply();
        out.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    RgbaImage::from_raw(pm.width(), pm.height(), out).expect("buffer size")
}

fn premultiplied(img: &RgbaImage) -> Option<Pixmap> {
    let mut pm = Pixmap::new(img.width().max(1), img.height().max(1))?;
    for (p, q) in pm.pixels_mut().iter_mut().zip(img.pixels()) {
        let c = tiny_skia::ColorU8::from_rgba(q.0[0], q.0[1], q.0[2], q.0[3]);
        *p = c.premultiply();
    }
    Some(pm)
}

/// Text drawn with the app's text engine (the same as text objects).
fn text_image(
    text: &str,
    font: &str,
    size: f64,
    color: &str,
    width: Option<f64>,
    align: TextAlign,
    dpi: f64,
) -> Option<RgbaImage> {
    let t = Text {
        at: [0.0, 0.0],
        width,
        text: text.to_string(),
        rotation: 0.0,
        style: TextStyle {
            font: if font.is_empty() {
                TextStyle::default().font
            } else {
                font.to_string()
            },
            size,
            color: color.to_string(),
            align,
            ..TextStyle::default()
        },
    };
    let (pm, _) = crate::text::rasterize(&t, dpi)?;
    Some(straight(&pm))
}

/// `img` turned `deg` degrees around its center, in a box that holds it.
fn turned(img: &RgbaImage, deg: f64) -> RgbaImage {
    turn(img, deg, None)
}

/// `img` turned `deg` degrees clockwise in a box that holds it, smoothly; the new
/// corners transparent or `background` (EDT-072).
pub fn turn(img: &RgbaImage, deg: f64, background: Option<[u8; 4]>) -> RgbaImage {
    if deg.abs() < 0.01 && background.is_none() {
        return img.clone();
    }
    let (w, h) = (img.width() as f64, img.height() as f64);
    let (s, c) = deg.to_radians().sin_cos();
    let (bw, bh) = (
        (w * c.abs() + h * s.abs()).ceil(),
        (w * s.abs() + h * c.abs()).ceil(),
    );
    let (Some(src), Some(mut dst)) = (
        premultiplied(img),
        Pixmap::new(bw.max(1.0) as u32, bh.max(1.0) as u32),
    ) else {
        return img.clone();
    };
    if let Some([r, g, b, a]) = background {
        dst.fill(tiny_skia::Color::from_rgba8(r, g, b, a));
    }
    let t = Transform::from_translate((bw / 2.0) as f32, (bh / 2.0) as f32)
        .pre_rotate(deg as f32)
        .pre_translate((-w / 2.0) as f32, (-h / 2.0) as f32);
    let paint = PixmapPaint {
        quality: tiny_skia::FilterQuality::Bicubic,
        ..PixmapPaint::default()
    };
    dst.draw_pixmap(0, 0, src.as_ref(), &paint, t, None);
    straight(&dst)
}

/// A watermark (EDT-066): a picture of the document or a text, scaled, turned and
/// faded, in a corner, a side, the middle, or tiled.
pub fn watermark(mut img: RgbaImage, e: &Value, ctx: &Ctx) -> RgbaImage {
    let mark = match s(e, "source", "text") {
        "image" => e
            .get("image")
            .and_then(Value::as_str)
            .and_then(|n| (ctx.picture)(n)),
        _ => text_image(
            s(e, "text", "XcreenShot"),
            s(e, "font", ""),
            f(e, "size", 36.0),
            s(e, "color", "#FFFFFF"),
            None,
            TextAlign::Left,
            ctx.dpi,
        ),
    };
    let Some(mark) = mark else {
        return img;
    };
    let k = (f(e, "scale", 100.0).clamp(5.0, 200.0) / 100.0).max(0.01);
    let (mw, mh) = (
        ((mark.width() as f64 * k).round() as u32).max(1),
        ((mark.height() as f64 * k).round() as u32).max(1),
    );
    let mark = if (mw, mh) == mark.dimensions() {
        mark
    } else {
        imageops::resize(&mark, mw, mh, imageops::FilterType::Lanczos3)
    };
    let mut mark = turned(&mark, f(e, "rotation", 0.0).clamp(-90.0, 90.0));
    let opacity = (f(e, "opacity", 30.0) / 100.0).clamp(0.0, 1.0);
    for p in mark.pixels_mut() {
        p.0[3] = (p.0[3] as f64 * opacity).round() as u8;
    }
    let (w, h) = (img.width() as i64, img.height() as i64);
    let (mw, mh) = (mark.width() as i64, mark.height() as i64);
    let margin = f(e, "margin", 20.0).clamp(0.0, 200.0) as i64;
    let position = s(e, "position", "bottomRight");
    let mut place = |x: i64, y: i64| {
        for (dx, dy, p) in mark.enumerate_pixels() {
            let (tx, ty) = (x + dx as i64, y + dy as i64);
            if tx >= 0 && ty >= 0 && tx < w && ty < h {
                let q = img.get_pixel_mut(tx as u32, ty as u32);
                *q = over(*q, *p);
            }
        }
    };
    if position == "tile" {
        let (sx, sy) = (mw + margin.max(8), mh + margin.max(8));
        let mut row = 0;
        let mut y = -mh / 2;
        while y < h {
            // Every other row shifted: a diagonal pattern.
            let mut x = if row % 2 == 0 { 0 } else { -sx / 2 };
            while x < w {
                place(x, y);
                x += sx;
            }
            y += sy;
            row += 1;
        }
        return img;
    }
    let x = if position.ends_with("Left") || position == "left" {
        margin
    } else if position.ends_with("Right") || position == "right" {
        w - mw - margin
    } else {
        (w - mw) / 2
    };
    let y = if position.starts_with("top") {
        margin
    } else if position.starts_with("bottom") {
        h - mh - margin
    } else {
        (h - mh) / 2
    };
    place(x, y);
    img
}

/// The lines of the capture's details that are on (EDT-067).
fn detail_lines(e: &Value, meta: &Value) -> Vec<String> {
    let on = |k: &str, d: bool| {
        e.get("fields")
            .and_then(|v| v.get(k))
            .and_then(Value::as_bool)
            .unwrap_or(d)
    };
    let text = |v: Option<&Value>| -> Option<String> {
        match v? {
            Value::String(s) if !s.is_empty() => Some(s.clone()),
            Value::Object(m) => {
                let name = m.get("name").and_then(Value::as_str).unwrap_or("");
                let version = m.get("version").and_then(Value::as_str).unwrap_or("");
                let t = format!("{name} {version}").trim().to_string();
                (!t.is_empty()).then_some(t)
            }
            _ => None,
        }
    };
    let mut lines = Vec::new();
    // The URL stands for the browser's name when there is one.
    let url = text(meta.get("url"));
    if on("app", true) {
        if let Some(a) = url
            .clone()
            .filter(|_| on("url", true))
            .or_else(|| text(meta.get("app")))
        {
            lines.push(a);
        }
    } else if on("url", true) {
        if let Some(u) = url {
            lines.push(u);
        }
    }
    if on("title", true) {
        if let Some(t) = text(meta.get("window").and_then(|w| w.get("title"))) {
            lines.push(t);
        }
    }
    if on("date", true) {
        if let Some(d) = meta.get("capturedAt").and_then(Value::as_str) {
            let shown = match (
                s(e, "dateFormat", "system"),
                chrono::DateTime::parse_from_rfc3339(d),
            ) {
                ("iso", Ok(t)) => t.format("%Y-%m-%dT%H:%M:%S%:z").to_string(),
                ("custom", Ok(t)) => t.format(s(e, "dateCustom", "%Y-%m-%d %H:%M")).to_string(),
                (_, Ok(t)) => t.format("%x %X").to_string(),
                _ => d.to_string(),
            };
            lines.push(shown);
        }
    }
    if on("os", false) {
        if let Some(o) = text(meta.get("os")) {
            lines.push(o);
        }
    }
    if on("user", false) {
        if let Some(u) = text(meta.get("user")) {
            lines.push(u);
        }
    }
    if on("computer", false) {
        if let Some(c) = text(meta.get("machine")) {
            lines.push(c);
        }
    }
    if let Some(t) = e
        .get("customText")
        .and_then(Value::as_str)
        .filter(|t| !t.trim().is_empty())
    {
        lines.push(t.to_string());
    }
    lines
}

/// Capture details (EDT-067): in a band above or below the image (it grows), or over
/// a corner. Returns the image and how far down the original went.
pub fn details(img: RgbaImage, e: &Value, ctx: &Ctx) -> (RgbaImage, i64) {
    let lines = detail_lines(e, ctx.meta);
    if lines.is_empty() {
        return (img, 0);
    }
    let size = f(e, "size", 12.0);
    let pad = f(e, "padding", 8.0).clamp(0.0, 100.0);
    let align = match s(e, "align", "left") {
        "center" => TextAlign::Center,
        "right" => TextAlign::Right,
        _ => TextAlign::Left,
    };
    let bg = s(e, "background", "#1F2430CC");
    let bgc = Rgba::parse(bg).unwrap_or(Rgba {
        r: 31,
        g: 36,
        b: 48,
        a: 204,
    });
    let band = s(e, "placement", "band") == "band";
    let (w, h) = (img.width(), img.height());
    let text_w = if band {
        (w as f64 - 2.0 * pad).max(10.0)
    } else {
        (w as f64 * 0.6).max(10.0)
    };
    let Some(text) = text_image(
        &lines.join("\n"),
        s(e, "font", ""),
        size,
        s(e, "color", "#FFFFFF"),
        Some(text_w),
        align,
        ctx.dpi,
    ) else {
        return (img, 0);
    };
    let box_h = text.height() + (2.0 * pad) as u32;
    if band {
        let top = s(e, "side", "bottom") == "top";
        let mut out = RgbaImage::new(w, h + box_h);
        let band_y = if top { 0 } else { h };
        for y in band_y..band_y + box_h {
            for x in 0..w {
                out.put_pixel(x, y, Px([bgc.r, bgc.g, bgc.b, bgc.a]));
            }
        }
        imageops::overlay(&mut out, &img, 0, if top { box_h as i64 } else { 0 });
        let tx = match align {
            TextAlign::Center => (w as i64 - text.width() as i64) / 2,
            TextAlign::Right => w as i64 - text.width() as i64 - pad as i64,
            _ => pad as i64,
        };
        for (x, y, p) in text.enumerate_pixels() {
            let (ox, oy) = (tx + x as i64, band_y as i64 + pad as i64 + y as i64);
            if ox >= 0 && oy >= 0 && (ox as u32) < w && (oy as u32) < out.height() {
                let q = out.get_pixel_mut(ox as u32, oy as u32);
                *q = over(*q, *p);
            }
        }
        return (out, if top { box_h as i64 } else { 0 });
    }
    // Over a corner.
    let mut out = img;
    let box_w = text.width() + (2.0 * pad) as u32;
    let corner = s(e, "corner", "bottomRight");
    let bx = if corner.ends_with("Left") {
        0
    } else {
        w.saturating_sub(box_w)
    } as i64;
    let by = if corner.starts_with("top") {
        0
    } else {
        h.saturating_sub(box_h)
    } as i64;
    for y in 0..box_h as i64 {
        for x in 0..box_w as i64 {
            let (ox, oy) = (bx + x, by + y);
            if ox >= 0 && oy >= 0 && (ox as u32) < w && (oy as u32) < h {
                let q = out.get_pixel_mut(ox as u32, oy as u32);
                *q = over(*q, Px([bgc.r, bgc.g, bgc.b, bgc.a]));
            }
        }
    }
    for (x, y, p) in text.enumerate_pixels() {
        let (ox, oy) = (bx + pad as i64 + x as i64, by + pad as i64 + y as i64);
        if ox >= 0 && oy >= 0 && (ox as u32) < w && (oy as u32) < h {
            let q = out.get_pixel_mut(ox as u32, oy as u32);
            *q = over(*q, *p);
        }
    }
    (out, 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    type Pictures = Box<dyn Fn(&str) -> Option<RgbaImage>>;

    fn ctx_with(meta: &Value) -> (Value, Pictures) {
        (meta.clone(), Box::new(|_| None))
    }

    #[test]
    fn a_text_watermark_shows_in_its_corner_and_tiles() {
        let (meta, pic) = ctx_with(&json!({}));
        let ctx = Ctx {
            canvas_color: None,
            dpi: 96.0,
            meta: &meta,
            picture: &*pic,
        };
        let img = RgbaImage::from_pixel(400, 300, Px([0, 0, 0, 255]));
        let out = watermark(
            img.clone(),
            &json!({"text": "WATERMARK", "color": "#FFFFFF", "opacity": 100, "position": "bottomRight", "margin": 10}),
            &ctx,
        );
        let bright = |img: &RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32| {
            (y0..y1).any(|y| (x0..x1).any(|x| img.get_pixel(x, y).0[0] > 128))
        };
        assert!(bright(&out, 200, 200, 400, 300), "bottom right");
        assert!(!bright(&out, 0, 0, 150, 100), "not top left");
        let tiled = watermark(
            img,
            &json!({"text": "W", "color": "#FFFFFF", "opacity": 100, "position": "tile", "rotation": -30}),
            &ctx,
        );
        assert!(bright(&tiled, 0, 0, 120, 120) && bright(&tiled, 250, 180, 400, 300));
    }

    #[test]
    fn details_add_a_band_with_the_chosen_fields() {
        let meta = json!({"capturedAt": "2026-11-12T10:00:00+01:00", "url": "https://example.com", "app": {"name": "Browser"}, "user": "ana"});
        let (meta, pic) = ctx_with(&meta);
        let ctx = Ctx {
            canvas_color: None,
            dpi: 96.0,
            meta: &meta,
            picture: &*pic,
        };
        let img = RgbaImage::from_pixel(400, 200, Px([255, 255, 255, 255]));
        let (out, dy) = details(
            img.clone(),
            &json!({"placement": "band", "side": "bottom", "fields": {"user": false}}),
            &ctx,
        );
        assert!(out.height() > 200);
        assert_eq!(dy, 0);
        let lines = detail_lines(&json!({"fields": {"user": false}}), &meta);
        assert!(lines.iter().any(|l| l.contains("example.com")));
        assert!(!lines.iter().any(|l| l == "ana"));
        // A file opened from a folder: only the custom text.
        let none = detail_lines(&json!({"customText": "Mine"}), &json!({}));
        assert_eq!(none, vec!["Mine".to_string()]);
    }
}
