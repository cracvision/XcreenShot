//! Text objects (EDT-026): layout and drawing with the system fonts (`cosmic-text`).
//!
//! Rust draws text for the editor canvas too (ADR 0004, point 4): the canvas shows the
//! bitmap made here, so line breaks and glyphs are the same on screen and in files.
//! Sizes are points: at the document's resolution, `pixels = points × dpi / 72`.

use std::sync::{Mutex, OnceLock};

use cosmic_text::{
    Align, Attrs, Buffer, Color, Family, FontSystem, Metrics, Shaping, Style, SwashCache, Weight, Wrap,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tiny_skia::Pixmap;

use crate::color::Rgba;
use crate::geometry::point;

/// Line height as a multiple of the font size (EDT-026 default line spacing).
pub const LINE_SPACING: f32 = 1.2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TextAlign {
    Left,
    Center,
    Right,
    Justify,
}

/// Text properties of M1 (EDT-026), applied to the whole box.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct TextStyle {
    /// Family name; a family that is not installed is drawn with a fallback and kept.
    pub font: String,
    /// Points, 1–1000, decimals allowed.
    pub size: f64,
    pub bold: bool,
    pub italic: bool,
    pub color: String,
    pub align: TextAlign,
    /// 0–100 %.
    pub opacity: f64,
}

/// The system's interface font, the default of the Base theme (EDT-017).
pub fn default_font() -> &'static str {
    if cfg!(target_os = "macos") {
        "Helvetica Neue"
    } else {
        "Segoe UI"
    }
}

impl Default for TextStyle {
    fn default() -> Self {
        TextStyle {
            font: default_font().to_string(),
            size: 14.0,
            bold: false,
            italic: false,
            color: "#F2545B".to_string(),
            align: TextAlign::Left,
            opacity: 100.0,
        }
    }
}

impl TextStyle {
    pub fn validate(&self) -> Result<(), String> {
        if Rgba::parse(&self.color).is_none() {
            return Err(format!("color \"{}\" is not #RRGGBB or #RRGGBBAA", self.color));
        }
        if !(1.0..=1000.0).contains(&self.size) {
            return Err("size must be between 1 and 1000".to_string());
        }
        if !(0.0..=100.0).contains(&self.opacity) {
            return Err("opacity must be between 0 and 100".to_string());
        }
        if self.font.trim().is_empty() {
            return Err("font must not be empty".to_string());
        }
        Ok(())
    }

    /// `base` with the properties in `changes`; unknown or invalid ones are an error.
    pub fn merged(base: &TextStyle, changes: &Map<String, Value>) -> Result<TextStyle, String> {
        let mut v = serde_json::to_value(base).map_err(|e| e.to_string())?;
        if let Some(obj) = v.as_object_mut() {
            for (k, val) in changes {
                obj.insert(k.clone(), val.clone());
            }
        }
        let style: TextStyle = serde_json::from_value(v).map_err(|e| format!("style: {e}"))?;
        style.validate()?;
        Ok(style)
    }

    /// Font size in image pixels at `dpi`.
    pub fn pixels(&self, dpi: f64) -> f32 {
        (self.size * dpi.max(1.0) / 72.0) as f32
    }
}

/// A text box: its top-left corner, a fixed width (wraps) or none (grows with the text).
#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub at: [f64; 2],
    pub width: Option<f64>,
    pub text: String,
    /// Degrees, clockwise, around the box center.
    pub rotation: f64,
    pub style: TextStyle,
}

impl Text {
    pub fn from_props(props: &Map<String, Value>) -> Result<Text, String> {
        let style = match props.get("style") {
            Some(Value::Object(s)) => TextStyle::merged(&TextStyle::default(), s)?,
            Some(_) => return Err("style must be an object".to_string()),
            None => TextStyle::default(),
        };
        let width = match props.get("width") {
            None | Some(Value::Null) => None,
            Some(v) => Some(
                v.as_f64()
                    .filter(|w| *w >= 1.0)
                    .ok_or("width must be a number of pixels (at least 1) or null")?,
            ),
        };
        Ok(Text {
            at: point(props.get("at"), "at")?,
            width,
            text: props
                .get("text")
                .and_then(Value::as_str)
                .ok_or("text is required")?
                .to_string(),
            rotation: props.get("rotation").and_then(Value::as_f64).unwrap_or(0.0),
            style,
        })
    }
}

/// Size of the laid-out box, in image pixels.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Laid {
    pub width: f32,
    pub height: f32,
}

struct Fonts {
    system: FontSystem,
    cache: SwashCache,
}

/// The system fonts, loaded once (it takes a moment: see [`warm_up`]).
fn fonts() -> &'static Mutex<Fonts> {
    static FONTS: OnceLock<Mutex<Fonts>> = OnceLock::new();
    FONTS.get_or_init(|| {
        Mutex::new(Fonts {
            system: FontSystem::new(),
            cache: SwashCache::new(),
        })
    })
}

/// Loads the system fonts in the background so the first text does not wait.
pub fn warm_up() {
    std::thread::spawn(|| {
        drop(fonts().lock());
    });
}

/// Installed font families, sorted, for the font list (EDT-026).
pub fn families() -> Vec<String> {
    let f = fonts().lock().unwrap();
    let mut names: Vec<String> = f
        .system
        .db()
        .faces()
        .filter_map(|face| face.families.first().map(|(name, _)| name.clone()))
        .collect();
    names.sort_by_key(|n| n.to_lowercase());
    names.dedup();
    names
}

fn align(a: TextAlign) -> Align {
    match a {
        TextAlign::Left => Align::Left,
        TextAlign::Center => Align::Center,
        TextAlign::Right => Align::Right,
        TextAlign::Justify => Align::Justified,
    }
}

/// Lays out `text` and returns the buffer and the box size.
fn shape(system: &mut FontSystem, text: &Text, dpi: f64) -> (Buffer, Laid) {
    let px = text.style.pixels(dpi).max(1.0);
    let line = px * LINE_SPACING;
    let mut buffer = Buffer::new(system, Metrics::new(px, line));
    let attrs = Attrs::new()
        .family(Family::Name(&text.style.font))
        .weight(if text.style.bold {
            Weight::BOLD
        } else {
            Weight::NORMAL
        })
        .style(if text.style.italic {
            Style::Italic
        } else {
            Style::Normal
        });
    buffer.set_wrap(Wrap::WordOrGlyph);
    buffer.set_size(text.width.map(|w| w as f32), None);
    buffer.set_text(
        &text.text,
        &attrs,
        Shaping::Advanced,
        Some(align(text.style.align)),
    );
    buffer.shape_until_scroll(system, false);
    let widest = buffer.layout_runs().map(|r| r.line_w).fold(0.0f32, f32::max);
    let width = match text.width {
        Some(w) => w as f32,
        None => {
            // An auto-width box is as wide as its widest line; align within that.
            buffer.set_size(Some(widest.max(1.0)), None);
            buffer.shape_until_scroll(system, false);
            widest
        }
    };
    let lines = buffer.layout_runs().count().max(1);
    let laid = Laid {
        width: width.max(1.0),
        height: lines as f32 * line,
    };
    (buffer, laid)
}

/// Size of a text box in image pixels.
pub fn layout(text: &Text, dpi: f64) -> Laid {
    let mut f = fonts().lock().unwrap();
    shape(&mut f.system, text, dpi).1
}

/// The text drawn unrotated in a box of its laid-out size, premultiplied, at full
/// opacity (the object's opacity and rotation apply when it is composed).
pub fn rasterize(text: &Text, dpi: f64) -> Option<(Pixmap, Laid)> {
    let mut guard = fonts().lock().unwrap();
    let Fonts { system, cache } = &mut *guard;
    let (mut buffer, laid) = shape(system, text, dpi);
    let (w, h) = (laid.width.ceil() as u32, laid.height.ceil() as u32);
    let mut pm = Pixmap::new(w.max(1), h.max(1))?;
    let c = Rgba::parse(&text.style.color).unwrap_or(Rgba::WHITE);
    let stride = pm.width() as i32;
    let data = pm.data_mut();
    buffer.draw(
        system,
        cache,
        Color::rgba(c.r, c.g, c.b, c.a),
        |x, y, gw, gh, color| {
            let a = color.a() as u32;
            if a == 0 {
                return;
            }
            let premul = |v: u8| (v as u32 * a + 127) / 255;
            let src = [premul(color.r()), premul(color.g()), premul(color.b()), a];
            for yy in y..y + gh as i32 {
                for xx in x..x + gw as i32 {
                    if xx < 0 || yy < 0 || xx >= stride || yy >= h as i32 {
                        continue;
                    }
                    let i = ((yy * stride + xx) * 4) as usize;
                    for (k, s) in src.iter().enumerate() {
                        let d = data[i + k] as u32;
                        data[i + k] = (s + d * (255 - a) / 255).min(255) as u8;
                    }
                }
            }
        },
    );
    Some((pm, laid))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn text(props: Value) -> Text {
        Text::from_props(props.as_object().unwrap()).unwrap()
    }

    #[test]
    fn auto_width_grows_with_the_text() {
        let short = layout(&text(json!({"at": [0, 0], "text": "Hola"})), 96.0);
        let long = layout(&text(json!({"at": [0, 0], "text": "Hola, mundo entero"})), 96.0);
        assert!(long.width > short.width);
        assert_eq!(short.height, long.height, "one line each");
    }

    #[test]
    fn fixed_width_wraps_and_grows_in_height() {
        let t = text(json!({
            "at": [0, 0], "width": 200,
            "text": "Una frase bastante larga que no cabe en doscientos píxeles de ancho"
        }));
        let laid = layout(&t, 96.0);
        assert_eq!(laid.width, 200.0);
        let one_line = 14.0 * 96.0 / 72.0 * LINE_SPACING;
        assert!(laid.height > 1.5 * one_line, "{laid:?}");
    }

    #[test]
    fn points_follow_the_document_resolution() {
        let t = text(json!({"at": [0, 0], "text": "A"}));
        assert!(layout(&t, 192.0).height > 1.9 * layout(&t, 96.0).height);
    }

    #[test]
    fn draws_glyphs_in_the_text_color() {
        let t = text(json!({"at": [0, 0], "text": "Hola", "style": {"color": "#0000FF", "size": 30}}));
        let (pm, _) = rasterize(&t, 96.0).unwrap();
        let blue = pm
            .pixels()
            .iter()
            .filter(|p| p.alpha() == 255 && p.blue() == 255 && p.red() == 0)
            .count();
        assert!(blue > 20, "solid blue glyph pixels: {blue}");
    }

    #[test]
    fn rejects_bad_text_objects() {
        let bad = |v: Value| Text::from_props(v.as_object().unwrap()).is_err();
        assert!(bad(json!({"at": [0, 0]})));
        assert!(bad(json!({"text": "x"})));
        assert!(bad(json!({"at": [0, 0], "text": "x", "width": 0})));
        assert!(bad(json!({"at": [0, 0], "text": "x", "style": {"size": 0}})));
        assert!(bad(
            json!({"at": [0, 0], "text": "x", "style": {"weight": "bold"}})
        ));
        assert!(!bad(
            json!({"at": [0, 0], "text": "x", "width": null, "style": {"bold": true}})
        ));
    }
}
