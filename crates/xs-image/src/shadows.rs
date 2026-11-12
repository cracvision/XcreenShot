//! Object shadows (EDT-021): a soft copy of the object's shape, in a color, moved in
//! a direction. Rust draws it for the final image and for the editor, so both match.

use serde::{Deserialize, Serialize};
use tiny_skia::{Pixmap, PremultipliedColorU8};
use xs_core::document::Object;

use crate::color::Rgba;

/// A shadow's settings, in `style.shadow`. `angle` turns clockwise from the right
/// (screen coordinates: 90° is down); `distance` 0 is a shadow all around.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct Shadow {
    pub color: String,
    /// 0–359°.
    pub angle: f64,
    /// 0–50 px.
    pub distance: f64,
    /// 0–100 %.
    pub opacity: f64,
    /// 0–50 px.
    pub blur: f64,
}

impl Default for Shadow {
    fn default() -> Self {
        Shadow {
            color: "#000000".to_string(),
            angle: 45.0,
            distance: 6.0,
            opacity: 50.0,
            blur: 8.0,
        }
    }
}

/// Object types that can have a shadow (EDT-021).
pub const WITH_SHADOW: &[&str] = &[
    "arrow", "line", "text", "callout", "shape", "stamp", "step", "pen", "image",
];

impl Shadow {
    /// The shadow of `object`, if it has one that shows.
    pub fn of(object: &Object) -> Option<Shadow> {
        if !WITH_SHADOW.contains(&object.kind.as_str()) {
            return None;
        }
        let v = object.props.get("style")?.get("shadow")?;
        let s: Shadow = serde_json::from_value(v.clone()).ok()?;
        (s.opacity > 0.0).then_some(s)
    }

    /// How far it is moved, in whole pixels.
    pub fn offset(&self) -> (i32, i32) {
        let a = self.angle.to_radians();
        (
            (a.cos() * self.distance).round() as i32,
            (a.sin() * self.distance).round() as i32,
        )
    }

    /// Room around the object for the blur.
    fn margin(&self) -> i32 {
        (self.blur.clamp(0.0, 50.0) * 1.5).ceil() as i32 + 2
    }

    /// Checks the values (for styles).
    pub fn check(&self) -> Result<(), String> {
        Rgba::parse(&self.color).ok_or_else(|| format!("{} is not a color", self.color))?;
        for (name, v, max) in [
            ("angle", self.angle, 360.0),
            ("distance", self.distance, 50.0),
            ("opacity", self.opacity, 100.0),
            ("blur", self.blur, 50.0),
        ] {
            if !(0.0..=max).contains(&v) {
                return Err(format!("shadow {name} must be between 0 and {max}"));
            }
        }
        Ok(())
    }
}

/// Where the shadow of an object with these bounds (`[x0, y0, x1, y1]`, canvas
/// pixels) goes: `[x, y, w, h]`.
pub fn place(shadow: &Shadow, bounds: [f64; 4]) -> [i32; 4] {
    let m = shadow.margin();
    let (dx, dy) = shadow.offset();
    let x0 = bounds[0].floor() as i32 - m;
    let y0 = bounds[1].floor() as i32 - m;
    let x1 = bounds[2].ceil() as i32 + m;
    let y1 = bounds[3].ceil() as i32 + m;
    [x0 + dx, y0 + dy, (x1 - x0).max(1), (y1 - y0).max(1)]
}

/// The shadow of `object` (with `bounds`), drawn by `draw` on a pixmap of its own,
/// and where it goes (top-left corner, canvas pixels).
pub fn layer(
    object: &Object,
    shadow: &Shadow,
    bounds: [f64; 4],
    draw: impl Fn(&mut Pixmap, &Object),
) -> Option<(Pixmap, i32, i32)> {
    let [x, y, w, h] = place(shadow, bounds);
    let (dx, dy) = shadow.offset();
    let (ox, oy) = (x - dx, y - dy);
    // Too big to be useful (a runaway size): no shadow.
    if (w as u64) * (h as u64) > 40_000_000 {
        return None;
    }
    let mut shape = Pixmap::new(w as u32, h as u32)?;
    let mut moved = object.clone();
    moved.translate(-ox as f64, -oy as f64);
    draw(&mut shape, &moved);
    let mut alpha: Vec<f32> = shape.pixels().iter().map(|p| p.alpha() as f32).collect();
    blur(&mut alpha, w as usize, h as usize, shadow.blur / 2.0);
    let c = Rgba::parse(&shadow.color).unwrap_or(Rgba {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    });
    let k = (shadow.opacity / 100.0).clamp(0.0, 1.0) as f32 * c.a as f32 / 255.0;
    for (px, a) in shape.pixels_mut().iter_mut().zip(alpha) {
        let a = (a * k).round().clamp(0.0, 255.0) as u8;
        let pre = |v: u8| ((v as u16 * a as u16 + 127) / 255) as u8;
        *px = PremultipliedColorU8::from_rgba(pre(c.r), pre(c.g), pre(c.b), a)
            .unwrap_or(PremultipliedColorU8::TRANSPARENT);
    }
    Some((shape, x, y))
}

/// A Gaussian blur of standard deviation `sigma`, as three box blurs.
fn blur(v: &mut [f32], w: usize, h: usize, sigma: f64) {
    if sigma < 0.3 {
        return;
    }
    // Box width whose three passes give about this sigma.
    let width = (4.0 * sigma * sigma + 1.0).sqrt().round() as i64;
    let r = (width / 2).max(1);
    let mut tmp = vec![0f32; v.len()];
    for _ in 0..3 {
        for (horizontal, len, lines) in [(true, w, h), (false, h, w)] {
            let len = len as i64;
            for line in 0..lines {
                let idx = |i: i64| -> usize {
                    if horizontal {
                        line * w + i as usize
                    } else {
                        i as usize * w + line
                    }
                };
                let get = |v: &[f32], i: i64| if i < 0 || i >= len { 0.0 } else { v[idx(i)] };
                let mut sum: f32 = (-r..=r).map(|i| get(v, i)).sum();
                for i in 0..len {
                    tmp[idx(i)] = sum / (2 * r + 1) as f32;
                    sum += get(v, i + r + 1) - get(v, i - r);
                }
            }
            v.copy_from_slice(&tmp);
        }
    }
}
