//! QR codes (SMT-036, SMT-038): the QR object of a document, drawn as square modules at
//! its own size (sharp however big it gets), its picture as PNG or SVG, and the codes
//! read from the pixels of an image. Everything happens on this computer.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tiny_skia::{FillRule, PathBuilder, Pixmap};

use crate::color::Rgba;
use crate::render::{paint, turned};
use crate::shapes::{rect, Rect};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Ecc {
    L,
    M,
    Q,
    H,
}

impl Ecc {
    fn level(self) -> qrcode::EcLevel {
        match self {
            Ecc::L => qrcode::EcLevel::L,
            Ecc::M => qrcode::EcLevel::M,
            Ecc::Q => qrcode::EcLevel::Q,
            Ecc::H => qrcode::EcLevel::H,
        }
    }

    /// The most bytes a QR code holds with this correction (version 40, bytes).
    pub fn capacity(self) -> usize {
        match self {
            Ecc::L => 2953,
            Ecc::M => 2331,
            Ecc::Q => 1663,
            Ecc::H => 1273,
        }
    }
}

/// What a QR object shows and how.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct QrStyle {
    /// `link` or `text`: how the panel shows the content.
    pub kind: String,
    pub ecc: Ecc,
    pub color: String,
    /// `None` is transparent.
    pub background: Option<String>,
    /// Around the code, in modules (0–8).
    pub margin: u32,
}

impl Default for QrStyle {
    fn default() -> Self {
        QrStyle {
            kind: "link".into(),
            ecc: Ecc::M,
            color: "#000000".into(),
            background: Some("#FFFFFF".into()),
            margin: 4,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct QrObject {
    pub rect: Rect,
    pub rotation: f64,
    pub content: String,
    pub style: QrStyle,
}

impl QrObject {
    pub fn from_props(props: &Map<String, Value>) -> Result<QrObject, String> {
        let style: QrStyle = match props.get("style") {
            Some(v @ Value::Object(_)) => serde_json::from_value(v.clone()).map_err(|e| e.to_string())?,
            Some(_) => return Err("style must be an object".into()),
            None => QrStyle::default(),
        };
        check_style(&style)?;
        let content = props
            .get("content")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        if content.is_empty() {
            return Err("content must not be empty".into());
        }
        if content.len() > style.ecc.capacity() {
            return Err(format!(
                "content is too long: at most {} bytes with this correction",
                style.ecc.capacity()
            ));
        }
        Ok(QrObject {
            rect: rect(props.get("rect"))?,
            rotation: props.get("rotation").and_then(Value::as_f64).unwrap_or(0.0),
            content,
            style,
        })
    }
}

pub fn check_style(s: &QrStyle) -> Result<(), String> {
    Rgba::parse(&s.color).ok_or("color must be a color like #RRGGBB")?;
    if let Some(b) = &s.background {
        Rgba::parse(b).ok_or("background must be a color like #RRGGBB or null")?;
    }
    if s.margin > 8 {
        return Err("margin must be between 0 and 8 modules".into());
    }
    if !["link", "text"].contains(&s.kind.as_str()) {
        return Err("kind must be link or text".into());
    }
    Ok(())
}

/// A link without a scheme gets `https://` (SMT-036).
pub fn link(text: &str) -> String {
    let t = text.trim();
    let has_scheme = t.split_once(':').is_some_and(|(s, _)| {
        !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || "+-.".contains(c))
    });
    if t.is_empty() || has_scheme {
        t.to_string()
    } else {
        format!("https://{t}")
    }
}

/// The modules of a code: its side and, row by row, which are dark.
pub fn matrix(content: &str, ecc: Ecc) -> Result<(usize, Vec<bool>), String> {
    let code = qrcode::QrCode::with_error_correction_level(content.as_bytes(), ecc.level())
        .map_err(|e| e.to_string())?;
    let n = code.width();
    let dark = code
        .to_colors()
        .into_iter()
        .map(|c| c == qrcode::Color::Dark)
        .collect();
    Ok((n, dark))
}

/// Draws a QR object: its background, then its dark modules, edges kept sharp.
pub fn draw(pm: &mut Pixmap, q: &QrObject) {
    let Ok((n, dark)) = matrix(&q.content, q.style.ecc) else {
        return;
    };
    let t = turned(q.rect, q.rotation);
    if let Some(bg) = q.style.background.as_deref().and_then(Rgba::parse) {
        if let Some(r) =
            tiny_skia::Rect::from_xywh(q.rect.x as f32, q.rect.y as f32, q.rect.w as f32, q.rect.h as f32)
        {
            let mut p = paint(bg);
            p.anti_alias = false;
            pm.fill_rect(r, &p, t, None);
        }
    }
    let Some(fg) = Rgba::parse(&q.style.color) else {
        return;
    };
    let total = (n + 2 * q.style.margin as usize) as f64;
    let (cw, ch) = (q.rect.w / total, q.rect.h / total);
    let mut pb = PathBuilder::new();
    for y in 0..n {
        for x in 0..n {
            if dark[y * n + x] {
                let px = q.rect.x + (x as f64 + q.style.margin as f64) * cw;
                let py = q.rect.y + (y as f64 + q.style.margin as f64) * ch;
                // Whole pixels where it can: neighbors touch without seams.
                let (x0, y0) = (px.round(), py.round());
                let (x1, y1) = ((px + cw).round(), (py + ch).round());
                if let Some(r) = tiny_skia::Rect::from_ltrb(
                    x0 as f32,
                    y0 as f32,
                    x1.max(x0 + 1.0) as f32,
                    y1.max(y0 + 1.0) as f32,
                ) {
                    pb.push_rect(r);
                }
            }
        }
    }
    if let Some(path) = pb.finish() {
        let mut p = paint(fg);
        p.anti_alias = false;
        pm.fill_path(&path, &p, FillRule::Winding, t, None);
    }
}

/// The code alone as a picture of `size` pixels a side.
pub fn picture(content: &str, style: &QrStyle, size: u32) -> Result<image::RgbaImage, String> {
    let size = size.clamp(16, 8192);
    let mut pm = Pixmap::new(size, size).ok_or("no picture")?;
    let q = QrObject {
        rect: Rect {
            x: 0.0,
            y: 0.0,
            w: size as f64,
            h: size as f64,
        },
        rotation: 0.0,
        content: content.to_string(),
        style: style.clone(),
    };
    matrix(content, style.ecc)?;
    draw(&mut pm, &q);
    let mut out = image::RgbaImage::new(size, size);
    for (o, p) in out.pixels_mut().zip(pm.pixels()) {
        let c = p.demultiply();
        o.0 = [c.red(), c.green(), c.blue(), c.alpha()];
    }
    Ok(out)
}

/// The code as SVG, one square per module (a vector picture).
pub fn svg(content: &str, style: &QrStyle) -> Result<String, String> {
    let (n, dark) = matrix(content, style.ecc)?;
    let m = style.margin as usize;
    let side = n + 2 * m;
    let mut d = String::new();
    for y in 0..n {
        for x in 0..n {
            if dark[y * n + x] {
                d.push_str(&format!("M{} {}h1v1h-1z", x + m, y + m));
            }
        }
    }
    let bg = style
        .background
        .as_ref()
        .map(|b| format!("<rect width=\"{side}\" height=\"{side}\" fill=\"{b}\"/>"))
        .unwrap_or_default();
    Ok(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {side} {side}\" shape-rendering=\"crispEdges\">{bg}<path fill=\"{}\" d=\"{d}\"/></svg>",
        style.color
    ))
}

/// A code read from pixels: its content and where it is (`[x, y, w, h]`).
#[derive(Debug, Clone, PartialEq)]
pub struct Found {
    pub text: String,
    pub rect: [f64; 4],
}

/// The QR codes in `img`, read on this computer. Light codes on dark backgrounds too.
pub fn read(img: &image::RgbaImage) -> Vec<Found> {
    let (w, h) = img.dimensions();
    let luma = |x: usize, y: usize| -> u8 {
        let p = img.get_pixel(x as u32, y as u32).0;
        let a = p[3] as u32;
        let mix = |c: u8| (c as u32 * a + 255 * (255 - a)) / 255;
        ((mix(p[0]) * 299 + mix(p[1]) * 587 + mix(p[2]) * 114) / 1000) as u8
    };
    let mut out: Vec<Found> = Vec::new();
    for invert in [false, true] {
        let mut prepared = rqrr::PreparedImage::prepare_from_greyscale(w as usize, h as usize, |x, y| {
            let v = luma(x, y);
            if invert {
                255 - v
            } else {
                v
            }
        });
        for grid in prepared.detect_grids() {
            let Ok((_, text)) = grid.decode() else { continue };
            let xs = grid.bounds.iter().map(|p| p.x as f64);
            let ys = grid.bounds.iter().map(|p| p.y as f64);
            let (x0, x1) = xs
                .clone()
                .fold((f64::MAX, f64::MIN), |(a, b), v| (a.min(v), b.max(v)));
            let (y0, y1) = ys.fold((f64::MAX, f64::MIN), |(a, b), v| (a.min(v), b.max(v)));
            let rect = [x0, y0, x1 - x0, y1 - y0];
            let same = out.iter().any(|f| {
                f.text == text && (f.rect[0] - rect[0]).abs() < 4.0 && (f.rect[1] - rect[1]).abs() < 4.0
            });
            if !same {
                out.push(Found { text, rect });
            }
        }
    }
    out.sort_by(|a, b| {
        a.rect[1]
            .total_cmp(&b.rect[1])
            .then(a.rect[0].total_cmp(&b.rect[0]))
    });
    out
}

/// The relative luminance contrast of two colors (WCAG), 1 to 21.
pub fn contrast(a: &str, b: &str) -> f64 {
    let lum = |hex: &str| {
        let c = Rgba::parse(hex).unwrap_or(Rgba {
            r: 0,
            g: 0,
            b: 0,
            a: 255,
        });
        let lin = |v: u8| {
            let v = v as f64 / 255.0;
            if v <= 0.03928 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * lin(c.r) + 0.7152 * lin(c.g) + 0.0722 * lin(c.b)
    };
    let (la, lb) = (lum(a), lum(b));
    (la.max(lb) + 0.05) / (la.min(lb) + 0.05)
}

/// Some readers may not read it: low contrast, or a code lighter than its background.
pub fn hard_to_read(style: &QrStyle) -> bool {
    let bg = style.background.as_deref().unwrap_or("#FFFFFF");
    let c = Rgba::parse(&style.color).unwrap_or(Rgba {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    });
    let b = Rgba::parse(bg).unwrap_or(Rgba {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    });
    let light = |c: Rgba| c.r as u32 * 299 + c.g as u32 * 587 + c.b as u32 * 114;
    contrast(&style.color, bg) < 4.0 || light(c) > light(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_link_reads_back() {
        let style = QrStyle::default();
        let content = link("example.com");
        assert_eq!(content, "https://example.com");
        let img = picture(&content, &style, 400).unwrap();
        let found = read(&img);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].text, "https://example.com");
        // Light on dark reads too.
        let inverted = QrStyle {
            color: "#FFFFFF".into(),
            background: Some("#000000".into()),
            ..QrStyle::default()
        };
        assert_eq!(read(&picture("hola", &inverted, 300).unwrap())[0].text, "hola");
    }

    #[test]
    fn modules_stay_sharp_when_big() {
        let style = QrStyle {
            margin: 0,
            ..QrStyle::default()
        };
        let small = picture("https://example.com", &style, 100).unwrap();
        let big = picture("https://example.com", &style, 400).unwrap();
        let (n, _) = matrix("https://example.com", Ecc::M).unwrap();
        // Only black and white, no gray edges, at 400 px too.
        for img in [&small, &big] {
            assert!(img.pixels().all(|p| p.0[0] == 0 || p.0[0] == 255), "sharp");
        }
        assert!(n > 20);
    }

    #[test]
    fn limits_and_contrast() {
        assert_eq!(link("mailto:a@b.c"), "mailto:a@b.c");
        assert_eq!(link("https://x.y"), "https://x.y");
        assert!(matrix(&"a".repeat(2954), Ecc::L).is_err());
        assert!(matrix(&"a".repeat(2953), Ecc::L).is_ok());
        let light = QrStyle {
            color: "#D0D0D0".into(),
            ..QrStyle::default()
        };
        assert!(hard_to_read(&light));
        assert!(!hard_to_read(&QrStyle::default()));
        let reversed = QrStyle {
            color: "#FFFFFF".into(),
            background: Some("#202020".into()),
            ..QrStyle::default()
        };
        assert!(hard_to_read(&reversed), "lighter than its background");
        assert!(svg("hola", &QrStyle::default()).unwrap().starts_with("<svg"));
    }
}
