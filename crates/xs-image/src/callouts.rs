//! Callouts (EDT-028): a text box with a body of some shape, fill and outline, and
//! tails that point at something.
//!
//! The body is `rect` turned `rotation` degrees around its center; the text turns with
//! it. Tails are the canvas points their tips reach: they do not turn with the body,
//! since they point at something on the image.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use tiny_skia::{
    FillRule, GradientStop, LinearGradient, Paint, PathBuilder, Pixmap, PixmapPaint, Point, SpreadMode,
    Transform,
};

use crate::color::Rgba;
use crate::geometry::{arrow_geometry, dash, point, ArrowStyle, Cap, LineStyle};
use crate::render::{blend_layer, draw_geometry, paint, stroke, turned};
use crate::shapes::{rect, Rect};
use crate::text::{self, Text, TextAlign, TextStyle};

/// Most tails a callout has.
pub const MAX_TAILS: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CalloutShape {
    Rectangle,
    RoundedRectangle,
    Ellipse,
    Cloud,
    /// A cloud with bubbles instead of tails.
    Thought,
    Explosion,
    /// A label, pointed on the left.
    Tag,
    /// The body is an arrow pointing where the first tail does; no tail is drawn.
    Arrow,
    /// Text and tails, without body.
    Text,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TailStyle {
    Straight,
    Curved,
    /// A line with an arrowhead (EDT-024).
    Arrow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum VAlign {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct CalloutStyle {
    /// Shadow (EDT-021); none when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow: Option<crate::shadows::Shadow>,
    pub shape: CalloutShape,
    /// `None` is Transparent.
    pub fill: Option<crate::paint::Fill>,
    /// A soft gradient from the fill to a darker tone.
    pub shading: bool,
    pub outline: String,
    /// 0–20 px; 0 = no outline.
    pub outline_width: f64,
    pub line_style: LineStyle,
    /// 0–200 px, rectangular shapes.
    pub corner_radius: f64,
    pub tail_style: TailStyle,
    /// 2–200 px.
    pub tail_width: f64,
    /// `None`: the outline's.
    pub tail_color: Option<String>,
    pub font: String,
    /// Points.
    pub size: f64,
    pub bold: bool,
    pub italic: bool,
    pub text_color: String,
    pub align: TextAlign,
    pub valign: VAlign,
    /// 0–100 px between the body's edge and the text.
    pub padding: f64,
    /// The body grows to show all the text.
    pub fit_text: bool,
    pub opacity: f64,
}

impl Default for CalloutStyle {
    /// A white rounded box with a coral outline (EDT-017).
    fn default() -> Self {
        CalloutStyle {
            shadow: None,
            shape: CalloutShape::RoundedRectangle,
            fill: Some("#FFFFFF".into()),
            shading: false,
            outline: "#F2545B".to_string(),
            outline_width: 3.0,
            line_style: LineStyle::Solid,
            corner_radius: 12.0,
            tail_style: TailStyle::Straight,
            tail_width: 24.0,
            tail_color: None,
            font: text::default_font().to_string(),
            size: 14.0,
            bold: false,
            italic: false,
            text_color: "#1E1E1E".to_string(),
            align: TextAlign::Left,
            valign: VAlign::Top,
            padding: 10.0,
            fit_text: true,
            opacity: 100.0,
        }
    }
}

fn check_color(name: &str, c: &str) -> Result<(), String> {
    Rgba::parse(c)
        .map(|_| ())
        .ok_or_else(|| format!("{name} must be a color like #RRGGBB"))
}

fn within(name: &str, v: f64, lo: f64, hi: f64) -> Result<(), String> {
    if (lo..=hi).contains(&v) {
        Ok(())
    } else {
        Err(format!("{name} must be between {lo} and {hi}"))
    }
}

impl CalloutStyle {
    pub fn merged(base: &CalloutStyle, changes: &Map<String, Value>) -> Result<CalloutStyle, String> {
        let mut v = serde_json::to_value(base).map_err(|e| e.to_string())?;
        if let Some(m) = v.as_object_mut() {
            for (k, val) in changes {
                m.insert(k.clone(), val.clone());
            }
        }
        let s: CalloutStyle = serde_json::from_value(v).map_err(|e| e.to_string())?;
        if let Some(f) = &s.fill {
            f.check("fill")?;
        }
        if let Some(c) = &s.tail_color {
            check_color("tailColor", c)?;
        }
        check_color("outline", &s.outline)?;
        check_color("textColor", &s.text_color)?;
        within("outlineWidth", s.outline_width, 0.0, 20.0)?;
        within("cornerRadius", s.corner_radius, 0.0, 200.0)?;
        within("tailWidth", s.tail_width, 2.0, 200.0)?;
        within("size", s.size, 1.0, 1000.0)?;
        within("padding", s.padding, 0.0, 100.0)?;
        within("opacity", s.opacity, 0.0, 100.0)?;
        Ok(s)
    }

    /// The text of the callout as a text box style.
    pub fn text_style(&self) -> TextStyle {
        TextStyle {
            shadow: None,
            font: self.font.clone(),
            size: self.size,
            bold: self.bold,
            italic: self.italic,
            color: self.text_color.clone(),
            align: self.align,
            opacity: 100.0,
            markup: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CalloutObject {
    pub rect: Rect,
    pub rotation: f64,
    pub text: String,
    pub tails: Vec<[f64; 2]>,
    pub style: CalloutStyle,
}

impl CalloutObject {
    pub fn from_props(props: &Map<String, Value>) -> Result<CalloutObject, String> {
        let style = match props.get("style") {
            Some(Value::Object(s)) => CalloutStyle::merged(&CalloutStyle::default(), s)?,
            Some(_) => return Err("style must be an object".to_string()),
            None => CalloutStyle::default(),
        };
        let tails = match props.get("tails") {
            None => Vec::new(),
            Some(Value::Array(a)) => a
                .iter()
                .map(|t| point(Some(t), "each tail"))
                .collect::<Result<Vec<_>, _>>()?,
            Some(_) => return Err("tails must be a list of [x, y]".to_string()),
        };
        if tails.len() > MAX_TAILS {
            return Err("a callout has at most 8 tails".to_string());
        }
        Ok(CalloutObject {
            rect: rect(props.get("rect"))?,
            rotation: props.get("rotation").and_then(Value::as_f64).unwrap_or(0.0),
            text: props
                .get("text")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            tails,
            style,
        })
    }

    /// How far in from each side the text starts, as a share of the body's size, for
    /// the curve of round shapes.
    fn inset(&self) -> f64 {
        match self.style.shape {
            CalloutShape::Ellipse | CalloutShape::Cloud | CalloutShape::Thought => 0.12,
            CalloutShape::Explosion => 0.18,
            CalloutShape::Arrow => 0.1,
            _ => 0.0,
        }
    }

    fn margin(&self) -> f64 {
        self.style.padding + self.style.outline_width
    }

    /// The part of the body the text goes in, unturned: inside the padding, and inside
    /// the curve of round shapes.
    pub fn text_box(&self) -> Rect {
        let r = self.rect;
        let (k, p) = (self.inset(), self.margin());
        let (ix, iy) = (r.w * k + p, r.h * k + p);
        let left = if self.style.shape == CalloutShape::Tag {
            ix + r.h * 0.35
        } else {
            ix
        };
        Rect {
            x: r.x + left,
            y: r.y + iy,
            w: (r.w - left - ix).max(1.0),
            h: (r.h - 2.0 * iy).max(1.0),
        }
    }

    /// The text laid out in the text box, as a text object would draw it.
    pub fn text_object(&self) -> Text {
        let b = self.text_box();
        Text {
            at: [b.x, b.y],
            width: Some(b.w),
            text: self.text.clone(),
            rotation: 0.0,
            style: self.style.text_style(),
        }
    }

    /// The body height that shows all the text (EDT-028 Fit to text), or `None` when it
    /// already does.
    pub fn fitted_height(&self, dpi: f64) -> Option<f64> {
        if !self.style.fit_text || self.text.trim().is_empty() {
            return None;
        }
        let laid = text::layout(&self.text_object(), dpi).height as f64;
        let need = (laid + 2.0 * self.margin()) / (1.0 - 2.0 * self.inset());
        (need > self.rect.h + 0.5).then_some(need.ceil())
    }

    pub fn center(&self) -> [f64; 2] {
        [self.rect.x + self.rect.w / 2.0, self.rect.y + self.rect.h / 2.0]
    }
}

// ---------------------------------------------------------------------------
// Drawing

fn arc(pb: &mut PathBuilder, c: (f32, f32), r: f32, start: f32, sweep: f32) {
    let n = (sweep.abs() / std::f32::consts::FRAC_PI_2).ceil().max(1.0) as usize;
    let step = sweep / n as f32;
    let k = 4.0 / 3.0 * (step / 4.0).tan();
    let mut a = start;
    for _ in 0..n {
        let b = a + step;
        let (p0x, p0y) = (c.0 + r * a.cos(), c.1 + r * a.sin());
        let (p3x, p3y) = (c.0 + r * b.cos(), c.1 + r * b.sin());
        pb.cubic_to(
            p0x - k * r * a.sin(),
            p0y + k * r * a.cos(),
            p3x + k * r * b.sin(),
            p3y - k * r * b.cos(),
            p3x,
            p3y,
        );
        a = b;
    }
}

fn ellipse_point(r: Rect, a: f64, k: f64) -> (f32, f32) {
    let (cx, cy) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
    (
        (cx + k * r.w / 2.0 * a.cos()) as f32,
        (cy + k * r.h / 2.0 * a.sin()) as f32,
    )
}

/// Which way an arrow body points: toward its first tail, as one of four directions.
fn arrow_direction(c: &CalloutObject) -> (f64, f64) {
    let [cx, cy] = c.center();
    let Some(t) = c.tails.first() else {
        return (1.0, 0.0);
    };
    let (dx, dy) = (t[0] - cx, t[1] - cy);
    if dx.abs() >= dy.abs() {
        (dx.signum(), 0.0)
    } else {
        (0.0, dy.signum())
    }
}

/// The body, unturned, in canvas pixels. `None` for text alone.
fn body_path(c: &CalloutObject, r: Rect) -> Option<tiny_skia::Path> {
    let (x, y, w, h) = (r.x as f32, r.y as f32, r.w.max(0.01) as f32, r.h.max(0.01) as f32);
    let rect = tiny_skia::Rect::from_xywh(x, y, w, h)?;
    let mut pb = PathBuilder::new();
    match c.style.shape {
        CalloutShape::Text => return None,
        CalloutShape::Rectangle => return Some(PathBuilder::from_rect(rect)),
        CalloutShape::Ellipse => return PathBuilder::from_oval(rect),
        CalloutShape::RoundedRectangle => {
            let q = (c.style.corner_radius as f32).min(w / 2.0).min(h / 2.0);
            if q <= 0.0 {
                return Some(PathBuilder::from_rect(rect));
            }
            let half = std::f32::consts::FRAC_PI_2;
            pb.move_to(x + q, y);
            pb.line_to(x + w - q, y);
            arc(&mut pb, (x + w - q, y + q), q, -half, half);
            pb.line_to(x + w, y + h - q);
            arc(&mut pb, (x + w - q, y + h - q), q, 0.0, half);
            pb.line_to(x + q, y + h);
            arc(&mut pb, (x + q, y + h - q), q, half, half);
            pb.line_to(x, y + q);
            arc(&mut pb, (x + q, y + q), q, std::f32::consts::PI, half);
            pb.close();
        }
        CalloutShape::Cloud | CalloutShape::Thought => {
            // Bumps around an ellipse, and the ellipse inside them.
            let n = 12;
            let bump = (r.w.min(r.h) * 0.2).max(4.0) as f32;
            for i in 0..n {
                let a = i as f64 / n as f64 * std::f64::consts::TAU;
                let (px, py) = ellipse_point(r, a, 0.8);
                pb.push_circle(px, py, bump);
            }
            if let Some(inner) = tiny_skia::Rect::from_xywh(x + w * 0.1, y + h * 0.1, w * 0.8, h * 0.8) {
                pb.push_oval(inner);
            }
        }
        CalloutShape::Explosion => {
            let n = 14;
            for i in 0..2 * n {
                let a = i as f64 / (2 * n) as f64 * std::f64::consts::TAU - std::f64::consts::FRAC_PI_2;
                // Sharp points of slightly different lengths, the same every time.
                let k = if i % 2 == 0 {
                    1.0 - 0.06 * ((i * 7 % 5) as f64 / 4.0)
                } else {
                    0.68
                };
                let (px, py) = ellipse_point(r, a, k);
                if i == 0 {
                    pb.move_to(px, py);
                } else {
                    pb.line_to(px, py);
                }
            }
            pb.close();
        }
        CalloutShape::Tag => {
            let notch = (h * 0.35).min(w / 2.0);
            pb.move_to(x + notch, y);
            pb.line_to(x + w, y);
            pb.line_to(x + w, y + h);
            pb.line_to(x + notch, y + h);
            pb.line_to(x, y + h / 2.0);
            pb.close();
        }
        CalloutShape::Arrow => {
            // A block arrow along the direction of the first tail.
            let (dx, dy) = arrow_direction(c);
            let (cx, cy) = (x + w / 2.0, y + h / 2.0);
            let horizontal = dx != 0.0;
            let (len, thick) = if horizontal { (w, h) } else { (h, w) };
            let head = (len * 0.3).min(thick);
            let shaft = thick * 0.25;
            // Points for an arrow pointing to +x, centered on 0; then turned.
            let pts = [
                (-len / 2.0, -shaft),
                (len / 2.0 - head, -shaft),
                (len / 2.0 - head, -thick / 2.0),
                (len / 2.0, 0.0),
                (len / 2.0 - head, thick / 2.0),
                (len / 2.0 - head, shaft),
                (-len / 2.0, shaft),
            ];
            let map = |(px, py): (f32, f32)| -> (f32, f32) {
                match (dx as i32, dy as i32) {
                    (1, _) => (cx + px, cy + py),
                    (-1, _) => (cx - px, cy - py),
                    (_, 1) => (cx - py, cy + px),
                    _ => (cx + py, cy - px),
                }
            };
            for (i, p) in pts.into_iter().map(map).enumerate() {
                if i == 0 {
                    pb.move_to(p.0, p.1);
                } else {
                    pb.line_to(p.0, p.1);
                }
            }
            pb.close();
        }
    }
    pb.finish()
}

/// Where a tail leaves the body: its center, turned with the body.
fn tail_base(c: &CalloutObject) -> [f64; 2] {
    c.center()
}

/// A tail's shape: a wedge from the body's center to its tip (straight or curved).
fn tail_path(c: &CalloutObject, tip: [f64; 2]) -> Option<tiny_skia::Path> {
    let [bx, by] = tail_base(c);
    let (dx, dy) = (tip[0] - bx, tip[1] - by);
    let len = dx.hypot(dy);
    if len < 1.0 {
        return None;
    }
    let (nx, ny) = (-dy / len, dx / len);
    let half = c.style.tail_width / 2.0;
    let (b1, b2) = ((bx + nx * half, by + ny * half), (bx - nx * half, by - ny * half));
    let mut pb = PathBuilder::new();
    pb.move_to(b1.0 as f32, b1.1 as f32);
    match c.style.tail_style {
        TailStyle::Curved => {
            // Both sides bend the same way, like a comic's tail.
            let bend = len * 0.25;
            let m1 = (
                (b1.0 + tip[0]) / 2.0 + nx * bend,
                (b1.1 + tip[1]) / 2.0 + ny * bend,
            );
            let m2 = (
                (b2.0 + tip[0]) / 2.0 + nx * bend * 0.6,
                (b2.1 + tip[1]) / 2.0 + ny * bend * 0.6,
            );
            pb.quad_to(m1.0 as f32, m1.1 as f32, tip[0] as f32, tip[1] as f32);
            pb.quad_to(m2.0 as f32, m2.1 as f32, b2.0 as f32, b2.1 as f32);
        }
        _ => {
            pb.line_to(tip[0] as f32, tip[1] as f32);
            pb.line_to(b2.0 as f32, b2.1 as f32);
        }
    }
    pb.close();
    pb.finish()
}

/// Thought bubbles from the body toward a tip.
fn bubbles(c: &CalloutObject, tip: [f64; 2]) -> Option<tiny_skia::Path> {
    let [bx, by] = tail_base(c);
    let r = c.rect;
    let (dx, dy) = (tip[0] - bx, tip[1] - by);
    let len = dx.hypot(dy);
    if len < 1.0 {
        return None;
    }
    // From the edge of the body (an ellipse) to the tip.
    let (ux, uy) = (dx / len, dy / len);
    let edge = 1.0 / ((ux / (r.w / 2.0)).powi(2) + (uy / (r.h / 2.0)).powi(2)).sqrt();
    let start = edge.min(len);
    let mut pb = PathBuilder::new();
    let size = (c.style.tail_width / 2.0).clamp(3.0, 40.0);
    for (k, s) in [(0.25, 1.0), (0.6, 0.66), (0.92, 0.4)] {
        let d = start + (len - start) * k;
        pb.push_circle((bx + ux * d) as f32, (by + uy * d) as f32, (size * s) as f32);
    }
    pb.finish()
}

fn darker(c: Rgba) -> Rgba {
    let f = |v: u8| (v as f64 * 0.78) as u8;
    Rgba {
        r: f(c.r),
        g: f(c.g),
        b: f(c.b),
        a: c.a,
    }
}

fn fill_paint(c: &CalloutObject, fill: &crate::paint::Fill) -> Paint<'static> {
    let r = c.rect;
    // A gradient covers the body (EDT-020); a color can be shaded (EDT-028).
    let color = match fill {
        crate::paint::Fill::Gradient(_) => {
            return fill
                .paint([r.x, r.y, r.w, r.h])
                .unwrap_or_else(|| paint(Rgba::WHITE));
        }
        crate::paint::Fill::Color(c) => Rgba::parse(c).unwrap_or(Rgba::WHITE),
    };
    if !c.style.shading {
        return paint(color);
    }
    let r = c.rect;
    let d = darker(color);
    let stop = |pos: f32, k: Rgba| GradientStop::new(pos, tiny_skia::Color::from_rgba8(k.r, k.g, k.b, k.a));
    let mut p = Paint {
        anti_alias: true,
        ..Paint::default()
    };
    if let Some(shader) = LinearGradient::new(
        Point::from_xy(r.x as f32, r.y as f32),
        Point::from_xy(r.x as f32, (r.y + r.h) as f32),
        vec![stop(0.0, color), stop(1.0, d)],
        SpreadMode::Pad,
        Transform::identity(),
    ) {
        p.shader = shader;
    } else {
        p = paint(color);
    }
    p
}

/// Draws a callout (EDT-028).
pub fn draw(pm: &mut Pixmap, c: &CalloutObject, dpi: f64) {
    let t = turned(c.rect, c.rotation);
    let s = &c.style;
    let outline = Rgba::parse(&s.outline);
    let ow = s.outline_width;
    let fill = s.fill.clone();
    let tail_color = s
        .tail_color
        .as_deref()
        .and_then(Rgba::parse)
        .or(outline)
        .unwrap_or(Rgba::WHITE);
    let body = body_path(c, c.rect);
    let shape = s.shape;
    // Tails as wedges, bubbles, or none (an arrow body points by itself).
    let wedges: Vec<tiny_skia::Path> = match (shape, s.tail_style) {
        (CalloutShape::Arrow, _) | (_, TailStyle::Arrow) => Vec::new(),
        (CalloutShape::Thought, _) => c.tails.iter().filter_map(|tp| bubbles(c, *tp)).collect(),
        _ => c.tails.iter().filter_map(|tp| tail_path(c, *tp)).collect(),
    };
    blend_layer(pm, s.opacity, |pm| {
        // Arrow tails: a line from the body's center, under the body.
        if s.tail_style == TailStyle::Arrow && shape != CalloutShape::Arrow {
            let style = ArrowStyle {
                shadow: None,
                color: String::new(),
                width: ow.clamp(2.0, 100.0),
                start_cap: Cap::None,
                end_cap: Cap::Filled,
                line_style: LineStyle::Solid,
                opacity: 100.0,
            };
            for tip in &c.tails {
                let g = arrow_geometry(c.center(), *tip, &style);
                draw_geometry(pm, &g, tail_color);
            }
        }
        let text_only = shape == CalloutShape::Text;
        // One outline around body and tails together: thick strokes first, then the
        // fills cover their inner half.
        if ow > 0.0 && !text_only {
            if let Some(color) = outline {
                let d = dash(s.line_style, ow);
                let covered = fill.is_some();
                let width = if covered { 2.0 * ow } else { ow };
                let st = stroke(width, d.as_deref());
                for w in &wedges {
                    pm.stroke_path(w, &paint(tail_color), &st, Transform::identity(), None);
                }
                if let Some(b) = &body {
                    pm.stroke_path(b, &paint(color), &st, t, None);
                }
            }
        }
        if let Some(f) = &fill {
            if !text_only {
                for w in &wedges {
                    pm.fill_path(
                        w,
                        &fill_paint(c, f),
                        FillRule::Winding,
                        Transform::identity(),
                        None,
                    );
                }
                if let Some(b) = &body {
                    pm.fill_path(b, &fill_paint(c, f), FillRule::Winding, t, None);
                }
            }
        }
        if text_only {
            // Without a body, a tail is a line in the tail color.
            for w in &wedges {
                pm.fill_path(
                    w,
                    &paint(tail_color),
                    FillRule::Winding,
                    Transform::identity(),
                    None,
                );
            }
        }
        // The text turns with the body.
        if !c.text.trim().is_empty() {
            if let Some((raster, _)) = text::rasterize(&c.text_object(), dpi) {
                let b = c.text_box();
                let free = b.h - raster.height() as f64;
                let dy = match s.valign {
                    VAlign::Top => 0.0,
                    VAlign::Middle => (free / 2.0).max(0.0),
                    VAlign::Bottom => free.max(0.0),
                };
                // What does not fit is cut at the bottom of the text box.
                let shown = raster.height().min(b.h.ceil().max(1.0) as u32);
                let clip = if shown < raster.height() {
                    let mut part = Pixmap::new(raster.width(), shown);
                    if let Some(p) = part.as_mut() {
                        p.draw_pixmap(
                            0,
                            0,
                            raster.as_ref(),
                            &PixmapPaint::default(),
                            Transform::identity(),
                            None,
                        );
                    }
                    part
                } else {
                    None
                };
                let img = clip.as_ref().unwrap_or(&raster);
                let at = t.pre_translate(b.x as f32, (b.y + dy) as f32);
                let p = PixmapPaint {
                    quality: tiny_skia::FilterQuality::Bilinear,
                    ..PixmapPaint::default()
                };
                pm.draw_pixmap(0, 0, img.as_ref(), &p, at, None);
            }
        }
    });
}

/// What a callout covers, `[x0, y0, x1, y1]` in canvas pixels: its turned body and
/// outline, its tails' tips, the bulge of curved tails and the bubbles of a thought.
pub fn bounds(c: &CalloutObject) -> [f64; 4] {
    let ow = c.style.outline_width;
    let mut b = [f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY];
    let mut add = |x: f64, y: f64, m: f64| {
        b = [b[0].min(x - m), b[1].min(y - m), b[2].max(x + m), b[3].max(y + m)];
    };
    let body = extent(c);
    for [x, y] in body.iter().take(4) {
        add(*x, *y, ow);
    }
    let tip_room = ow
        + if c.style.shape == CalloutShape::Thought {
            (c.style.tail_width / 2.0).clamp(3.0, 40.0)
        } else {
            1.0
        };
    let [bx, by] = tail_base(c);
    for t in &c.tails {
        add(t[0], t[1], tip_room);
        if c.style.tail_style == TailStyle::Curved {
            let (dx, dy) = (t[0] - bx, t[1] - by);
            let len = dx.hypot(dy).max(1.0);
            let (nx, ny) = (-dy / len, dx / len);
            let bend = len * 0.25;
            add(
                (bx + t[0]) / 2.0 + nx * bend,
                (by + t[1]) / 2.0 + ny * bend,
                ow + c.style.tail_width / 2.0,
            );
        }
    }
    b
}

/// Canvas points a callout reaches: its turned body (the first four) and its tails.
pub fn extent(c: &CalloutObject) -> Vec<[f64; 2]> {
    let r = c.rect;
    let [cx, cy] = c.center();
    let (s, k) = c.rotation.to_radians().sin_cos();
    let mut pts: Vec<[f64; 2]> = [
        (r.x, r.y),
        (r.x + r.w, r.y),
        (r.x + r.w, r.y + r.h),
        (r.x, r.y + r.h),
    ]
    .iter()
    .map(|(x, y)| {
        let (dx, dy) = (x - cx, y - cy);
        [cx + dx * k - dy * s, cy + dx * s + dy * k]
    })
    .collect();
    pts.extend(c.tails.iter().copied());
    pts
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn styles_are_checked() {
        let bad = json!({"outlineWidth": 30});
        assert!(CalloutStyle::merged(&CalloutStyle::default(), bad.as_object().unwrap()).is_err());
        let ok = json!({"shape": "cloud", "tailStyle": "curved", "cornerRadius": 24});
        let s = CalloutStyle::merged(&CalloutStyle::default(), ok.as_object().unwrap()).unwrap();
        assert_eq!(s.shape, CalloutShape::Cloud);
    }

    /// The same case as `app/src/editor/callouts.test.ts`.
    #[test]
    fn text_box_matches_the_editor() {
        let props = json!({"rect": {"x": 0, "y": 0, "w": 200, "h": 60}});
        let mut c = CalloutObject::from_props(props.as_object().unwrap()).unwrap();
        let b = c.text_box();
        assert_eq!((b.x, b.y, b.w, b.h), (13.0, 13.0, 174.0, 34.0));
        c.style.shape = CalloutShape::Cloud;
        let b = c.text_box();
        let close = |a: f64, e: f64| (a - e).abs() < 1e-9;
        assert!(
            close(b.x, 37.0) && close(b.y, 20.2) && close(b.w, 126.0) && close(b.h, 19.6),
            "{b:?}"
        );
    }

    #[test]
    fn tails_are_points_and_at_most_eight() {
        let props = json!({"rect": {"x": 0, "y": 0, "w": 100, "h": 50}, "tails": [[10, 90]]});
        let c = CalloutObject::from_props(props.as_object().unwrap()).unwrap();
        assert_eq!(c.tails, [[10.0, 90.0]]);
        let many = json!({"rect": {"x": 0, "y": 0, "w": 100, "h": 50}, "tails": vec![[1, 1]; 9]});
        assert!(CalloutObject::from_props(many.as_object().unwrap()).is_err());
    }
}
