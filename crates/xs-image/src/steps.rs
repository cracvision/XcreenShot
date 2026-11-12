//! Step markers (EDT-035): a shape with a value of a sequence (1, 2, 3…; A, B, C…;
//! I, II, III…) whose text stays upright when the marker turns.
//!
//! The marker is square: `rect` keeps `w == h`, the size of `style.size`.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::color::Rgba;
use crate::shapes::{rect, Rect};

/// Sizes a marker can have, in pixels.
pub const MIN_SIZE: f64 = 16.0;
pub const MAX_SIZE: f64 = 256.0;
/// Longest value, in characters.
pub const MAX_VALUE: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Sequence {
    /// 1, 2, 3… up to 9999.
    Number,
    /// A, B, C… Z, AA, AB…
    Upper,
    /// a, b, c… z, aa, ab…
    Lower,
    /// I, II, III… up to 3999.
    RomanUpper,
    /// i, ii, iii…
    RomanLower,
}

impl Sequence {
    pub const ALL: [Sequence; 5] = [
        Sequence::Number,
        Sequence::Upper,
        Sequence::Lower,
        Sequence::RomanUpper,
        Sequence::RomanLower,
    ];

    pub fn key(self) -> &'static str {
        match self {
            Sequence::Number => "number",
            Sequence::Upper => "upper",
            Sequence::Lower => "lower",
            Sequence::RomanUpper => "romanUpper",
            Sequence::RomanLower => "romanLower",
        }
    }

    pub fn from_key(key: &str) -> Option<Sequence> {
        Sequence::ALL.into_iter().find(|s| s.key() == key)
    }

    fn last(self) -> u64 {
        match self {
            Sequence::Number => 9999,
            Sequence::RomanUpper | Sequence::RomanLower => 3999,
            // ZZZZZZ: six letters.
            Sequence::Upper | Sequence::Lower => 321_272_406,
        }
    }

    /// The `n`th value (from 1); past the last one the sequence starts again.
    pub fn value(self, n: u64) -> String {
        let n = (n.max(1) - 1) % self.last() + 1;
        match self {
            Sequence::Number => n.to_string(),
            Sequence::Upper => letters(n),
            Sequence::Lower => letters(n).to_lowercase(),
            Sequence::RomanUpper => roman(n),
            Sequence::RomanLower => roman(n).to_lowercase(),
        }
    }

    /// Which value of the sequence `text` is, if it is one.
    pub fn index(self, text: &str) -> Option<u64> {
        let t = text.trim();
        if t.is_empty() {
            return None;
        }
        let n = match self {
            Sequence::Number => t.parse::<u64>().ok().filter(|n| *n >= 1)?,
            Sequence::Upper if t.chars().all(|c| c.is_ascii_uppercase()) => from_letters(t)?,
            Sequence::Lower if t.chars().all(|c| c.is_ascii_lowercase()) => from_letters(&t.to_uppercase())?,
            Sequence::RomanUpper if t.chars().all(|c| "IVXLCDM".contains(c)) => from_roman(t)?,
            Sequence::RomanLower if t.chars().all(|c| "ivxlcdm".contains(c)) => {
                from_roman(&t.to_uppercase())?
            }
            _ => return None,
        };
        (n <= self.last()).then_some(n)
    }
}

/// A, B… Z, AA, AB… (bijective base 26).
fn letters(mut n: u64) -> String {
    let mut out = Vec::new();
    while n > 0 {
        n -= 1;
        out.push(b'A' + (n % 26) as u8);
        n /= 26;
    }
    out.reverse();
    String::from_utf8(out).unwrap_or_default()
}

fn from_letters(t: &str) -> Option<u64> {
    if t.len() > MAX_VALUE {
        return None;
    }
    t.bytes()
        .try_fold(0u64, |n, b| Some(n * 26 + (b.checked_sub(b'A')? as u64 + 1)))
}

const ROMAN: [(u64, &str); 13] = [
    (1000, "M"),
    (900, "CM"),
    (500, "D"),
    (400, "CD"),
    (100, "C"),
    (90, "XC"),
    (50, "L"),
    (40, "XL"),
    (10, "X"),
    (9, "IX"),
    (5, "V"),
    (4, "IV"),
    (1, "I"),
];

fn roman(mut n: u64) -> String {
    let mut out = String::new();
    for (v, s) in ROMAN {
        while n >= v {
            out.push_str(s);
            n -= v;
        }
    }
    out
}

fn from_roman(t: &str) -> Option<u64> {
    let mut n = 0;
    let mut rest = t;
    for (v, s) in ROMAN {
        while let Some(r) = rest.strip_prefix(s) {
            n += v;
            rest = r;
        }
    }
    // Only the usual way of writing it counts ("IIII" is not 4).
    (rest.is_empty() && n > 0 && roman(n) == t).then_some(n)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StepShape {
    Circle,
    Square,
    RoundedSquare,
    Diamond,
    /// A drop, pointing down.
    Marker,
    /// The value alone.
    Text,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct StepStyle {
    /// Shadow (EDT-021); none when absent.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shadow: Option<crate::shadows::Shadow>,
    pub sequence: Sequence,
    pub shape: StepShape,
    /// `None` is Transparent.
    pub fill: Option<String>,
    pub text_color: String,
    pub outline: String,
    /// 0–10 px.
    pub outline_width: f64,
    /// 16–256 px.
    pub size: f64,
    pub opacity: f64,
}

impl Default for StepStyle {
    /// A coral circle with a white number, from the default theme (EDT-017).
    fn default() -> Self {
        StepStyle {
            shadow: None,
            sequence: Sequence::Number,
            shape: StepShape::Circle,
            fill: Some("#F2545B".to_string()),
            text_color: "#FFFFFF".to_string(),
            outline: "#FFFFFF".to_string(),
            outline_width: 0.0,
            size: 32.0,
            opacity: 100.0,
        }
    }
}

fn check_color(name: &str, c: &str) -> Result<(), String> {
    Rgba::parse(c)
        .map(|_| ())
        .ok_or_else(|| format!("{name} must be a color like #RRGGBB"))
}

impl StepStyle {
    pub fn merged(base: &StepStyle, changes: &Map<String, Value>) -> Result<StepStyle, String> {
        let mut v = serde_json::to_value(base).map_err(|e| e.to_string())?;
        if let Some(m) = v.as_object_mut() {
            for (k, val) in changes {
                m.insert(k.clone(), val.clone());
            }
        }
        let s: StepStyle = serde_json::from_value(v).map_err(|e| e.to_string())?;
        if let Some(f) = &s.fill {
            check_color("fill", f)?;
        }
        check_color("textColor", &s.text_color)?;
        check_color("outline", &s.outline)?;
        if !(0.0..=10.0).contains(&s.outline_width) {
            return Err("outlineWidth must be between 0 and 10".to_string());
        }
        if !(MIN_SIZE..=MAX_SIZE).contains(&s.size) {
            return Err("size must be between 16 and 256".to_string());
        }
        if !(0.0..=100.0).contains(&s.opacity) {
            return Err("opacity must be between 0 and 100".to_string());
        }
        Ok(s)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct StepObject {
    pub rect: Rect,
    pub rotation: f64,
    pub value: String,
    pub style: StepStyle,
}

/// A square of side `size` (kept between 16 and 256) centered where `r` is.
pub fn square(r: Rect, size: f64) -> Rect {
    let s = size.clamp(MIN_SIZE, MAX_SIZE);
    let (cx, cy) = (r.x + r.w / 2.0, r.y + r.h / 2.0);
    Rect {
        x: cx - s / 2.0,
        y: cy - s / 2.0,
        w: s,
        h: s,
    }
}

/// The value as stored: trimmed, at most six characters.
pub fn check_value(v: &str) -> Result<String, String> {
    let t = v.trim();
    if t.is_empty() {
        return Err("value must not be empty".to_string());
    }
    if t.chars().count() > MAX_VALUE {
        return Err("value must be at most 6 characters".to_string());
    }
    Ok(t.to_string())
}

impl StepObject {
    pub fn from_props(props: &Map<String, Value>) -> Result<StepObject, String> {
        let style = match props.get("style") {
            Some(Value::Object(s)) => StepStyle::merged(&StepStyle::default(), s)?,
            Some(_) => return Err("style must be an object".to_string()),
            None => StepStyle::default(),
        };
        let value = check_value(props.get("value").and_then(Value::as_str).unwrap_or("1"))?;
        Ok(StepObject {
            rect: rect(props.get("rect"))?,
            rotation: props.get("rotation").and_then(Value::as_f64).unwrap_or(0.0),
            value,
            style,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequences_count_and_read_back() {
        use Sequence::*;
        assert_eq!(Number.value(3), "3");
        assert_eq!(Upper.value(26), "Z");
        assert_eq!(Upper.value(27), "AA");
        assert_eq!(Lower.value(28), "ab");
        assert_eq!(RomanUpper.value(14), "XIV");
        assert_eq!(RomanLower.value(9), "ix");
        assert_eq!(Number.value(10_000), "1", "past 9999 it starts again");
        for s in Sequence::ALL {
            for n in [1, 2, 9, 26, 27, 49, 702, 703, 1999] {
                assert_eq!(s.index(&s.value(n)), Some(n), "{s:?} {n}");
            }
        }
        assert_eq!(Number.index("B2"), None);
        assert_eq!(RomanUpper.index("IIII"), None);
        assert_eq!(Upper.index("a"), None);
    }

    #[test]
    fn values_and_sizes_are_checked() {
        assert!(check_value("1234567").is_err());
        assert_eq!(check_value(" B2 ").unwrap(), "B2");
        let s = square(
            Rect {
                x: 0.0,
                y: 0.0,
                w: 10.0,
                h: 10.0,
            },
            8.0,
        );
        assert_eq!((s.w, s.x), (16.0, -3.0), "at least 16, around the same center");
    }
}
