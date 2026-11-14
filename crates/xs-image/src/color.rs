//! Colors as written in documents: `#RRGGBB` or `#RRGGBBAA` (EDT-019).

/// Straight (not premultiplied) RGBA.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgba {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Rgba {
    pub const WHITE: Rgba = Rgba {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    };

    pub fn parse(s: &str) -> Option<Rgba> {
        let hex = s.trim().strip_prefix('#')?;
        if !hex.is_ascii() || !(hex.len() == 6 || hex.len() == 8) {
            return None;
        }
        let byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
        Some(Rgba {
            r: byte(0)?,
            g: byte(2)?,
            b: byte(4)?,
            a: if hex.len() == 8 { byte(6)? } else { 255 },
        })
    }

    /// `#RRGGBB`, or `#RRGGBBAA` when not opaque.
    pub fn to_hex(self) -> String {
        if self.a == 255 {
            format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
        } else {
            format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_hex_with_and_without_alpha() {
        assert_eq!(
            Rgba::parse("#2D7FF9"),
            Some(Rgba {
                r: 45,
                g: 127,
                b: 249,
                a: 255
            })
        );
        assert_eq!(Rgba::parse("#00000080").map(|c| c.a), Some(128));
        assert_eq!(Rgba::parse("2D7FF9"), None);
        assert_eq!(Rgba::parse("#XYZ123"), None);
        assert_eq!(
            Rgba::parse("#2d7ff9").map(Rgba::to_hex).as_deref(),
            Some("#2D7FF9")
        );
    }
}
