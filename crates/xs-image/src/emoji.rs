//! The Emoji stamps (EDT-033): Noto Emoji's pictures (Apache 2.0; flags in the public
//! domain) with the names and keywords of Unicode CLDR in English and Spanish, built by
//! `scripts/make-emoji.py` and carried inside the app, so they look the same on Windows
//! and macOS.

use std::sync::OnceLock;

use serde::Deserialize;

static INDEX: &str = include_str!("../assets/emoji.json");
static PICTURES: &[u8] = include_bytes!("../assets/emoji.bin");

/// An emoji of the set.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Emoji {
    /// Its code points in hexadecimal, joined with `-` (without U+FE0F).
    pub id: String,
    #[serde(rename = "char")]
    pub character: String,
    pub en: String,
    pub es: String,
    pub keywords_en: String,
    pub keywords_es: String,
    pub w: u32,
    pub h: u32,
    at: usize,
    len: usize,
}

/// Every emoji, in the set's order.
pub fn all() -> &'static [Emoji] {
    static ALL: OnceLock<Vec<Emoji>> = OnceLock::new();
    ALL.get_or_init(|| serde_json::from_str(INDEX).unwrap_or_default())
}

/// An emoji by its id.
pub fn find(id: &str) -> Option<&'static Emoji> {
    all().iter().find(|e| e.id == id)
}

/// The emoji a text is, if it is one emoji of the set (a drop from the system's picker).
pub fn of_text(text: &str) -> Option<&'static Emoji> {
    let t: String = text.trim().chars().filter(|c| *c != '\u{FE0F}').collect();
    all().iter().find(|e| {
        e.character
            .chars()
            .filter(|c| *c != '\u{FE0F}')
            .collect::<String>()
            == t
    })
}

impl Emoji {
    /// Its picture, as stored (WebP).
    pub fn picture(&self) -> &'static [u8] {
        PICTURES.get(self.at..self.at + self.len).unwrap_or_default()
    }

    /// Its picture decoded.
    pub fn image(&self) -> Option<image::RgbaImage> {
        crate::encode::decode(self.picture()).ok()
    }

    /// Whether `query` finds it: every word in its names or keywords, either language,
    /// without accents.
    pub fn matches(&self, query: &str) -> bool {
        let hay = crate::stamps::fold(
            &format!(
                "{} {} {} {}",
                self.en, self.es, self.keywords_en, self.keywords_es
            )
            .to_lowercase(),
        );
        crate::stamps::fold(&query.to_lowercase())
            .split_whitespace()
            .all(|w| hay.contains(w))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn thumbs_up_in_both_languages() {
        assert!(all().len() > 1500);
        let up = find("1f44d").unwrap();
        assert!(up.matches("pulgar"));
        assert!(up.matches("thumbs up"));
        let img = up.image().unwrap();
        assert_eq!((img.width(), img.height()), (up.w, up.h));
        assert!(img.pixels().any(|p| p.0[3] == 255));
        assert_eq!(of_text("👍").unwrap().id, "1f44d");
        assert_eq!(of_text("❤️").unwrap().id, "2764");
        assert!(find("1f1ea-1f1f8").unwrap().matches("bandera espana"));
    }
}
