//! Sensitive data in recognized text (SMT-011, SMT-012): where in the image each email,
//! phone number, card number, identity number, postal address, IP address, URL or date
//! is, and the user's own words and patterns. Pure: it reads what the recognition gave
//! and returns rectangles that cover only the data, not its label.

use regex::Regex;
use serde::Serialize;

use crate::{Recognized, Rect};

/// The categories, in the panel's order (SMT-011). Faces are found apart.
pub const CATEGORIES: [&str; 8] = ["email", "phone", "card", "id", "address", "ip", "url", "date"];

/// When two categories find the same text, the more specific one keeps it (SMT-011).
const PRIORITY: [&str; 8] = ["email", "card", "id", "ip", "url", "date", "phone", "address"];

/// A word or a pattern of the user's (SMT-012).
#[derive(Debug, Clone, PartialEq)]
pub struct Term {
    /// Its name in the panel: the words themselves, or the pattern's name.
    pub name: String,
    pub kind: TermKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TermKind {
    /// Words or a phrase; `exact` keeps case, accents and spaces.
    Words { text: String, exact: bool },
    /// A regular expression.
    Pattern(String),
}

/// One thing found: its category (or `term:<name>`), the text and where it is (one
/// rectangle per line it covers).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Found {
    pub category: String,
    pub text: String,
    pub rects: Vec<Rect>,
}

/// A line's text, with where each character comes from: (word, character in it).
struct Line<'a> {
    text: String,
    from: Vec<Option<(usize, usize)>>,
    words: &'a [crate::Word],
}

impl<'a> Line<'a> {
    fn new(words: &'a [crate::Word]) -> Line<'a> {
        let mut text = String::new();
        let mut from = Vec::new();
        for (wi, w) in words.iter().enumerate() {
            if wi > 0 {
                text.push(' ');
                from.push(None);
            }
            for (ci, c) in w.text.chars().enumerate() {
                text.push(c);
                from.push(Some((wi, ci)));
            }
        }
        Line { text, from, words }
    }

    /// The rectangle of the characters `start..end` (char indices): whole words, or the
    /// share of a word they take.
    fn rect(&self, start: usize, end: usize) -> Option<Rect> {
        let mut out: Option<Rect> = None;
        let mut add = |r: Rect| {
            out = Some(match out {
                None => r,
                Some(o) => {
                    let (x0, y0) = (o[0].min(r[0]), o[1].min(r[1]));
                    let (x1, y1) = ((o[0] + o[2]).max(r[0] + r[2]), (o[1] + o[3]).max(r[1] + r[3]));
                    [x0, y0, x1 - x0, y1 - y0]
                }
            })
        };
        let mut i = start;
        while i < end {
            let Some((wi, ci)) = self.from.get(i).copied().flatten() else {
                i += 1;
                continue;
            };
            let w = &self.words[wi];
            let n = w.text.chars().count().max(1);
            // The run of this word's characters inside the match.
            let mut last = ci;
            while i + 1 < end
                && self
                    .from
                    .get(i + 1)
                    .copied()
                    .flatten()
                    .is_some_and(|(wj, _)| wj == wi)
            {
                i += 1;
                last = self.from[i].map(|(_, c)| c).unwrap_or(last);
            }
            let r = w.rect;
            let (a, b) = (ci as f64 / n as f64, (last + 1) as f64 / n as f64);
            add([r[0] + r[2] * a, r[1], r[2] * (b - a), r[3]]);
            i += 1;
        }
        out
    }
}

/// Byte offset to char index in `s`.
fn char_index(s: &str, byte: usize) -> usize {
    s[..byte].chars().count()
}

fn luhn(digits: &str) -> bool {
    let d: Vec<u32> = digits.chars().filter_map(|c| c.to_digit(10)).collect();
    if !(13..=19).contains(&d.len()) {
        return false;
    }
    let sum: u32 = d
        .iter()
        .rev()
        .enumerate()
        .map(|(i, &x)| {
            if i % 2 == 1 {
                if x * 2 > 9 {
                    x * 2 - 9
                } else {
                    x * 2
                }
            } else {
                x
            }
        })
        .sum();
    sum.is_multiple_of(10)
}

/// Spain's DNI and NIE: their control letter.
fn dni_ok(s: &str) -> bool {
    let s = s.to_ascii_uppercase();
    let (body, letter) = s.split_at(s.len() - 1);
    let body = body
        .replacen('X', "0", 1)
        .replacen('Y', "1", 1)
        .replacen('Z', "2", 1);
    let Ok(n) = body.parse::<u64>() else {
        return false;
    };
    "TRWAGMYFPDXBNJZSQVHLCKE"
        .chars()
        .nth((n % 23) as usize)
        .map(String::from)
        == Some(letter.to_string())
}

/// Brazil's CPF: its two check digits.
fn cpf_ok(s: &str) -> bool {
    let d: Vec<u32> = s.chars().filter_map(|c| c.to_digit(10)).collect();
    if d.len() != 11 || d.iter().all(|&x| x == d[0]) {
        return false;
    }
    let check = |n: usize| {
        let sum: u32 = (0..n).map(|i| d[i] * (n as u32 + 1 - i as u32)).sum();
        let r = (sum * 10) % 11;
        if r == 10 {
            0
        } else {
            r
        }
    };
    check(9) == d[9] && check(10) == d[10]
}

/// Portugal's NIF: its check digit.
fn nif_ok(s: &str) -> bool {
    let d: Vec<u32> = s.chars().filter_map(|c| c.to_digit(10)).collect();
    if d.len() != 9 {
        return false;
    }
    let sum: u32 = (0..8).map(|i| d[i] * (9 - i as u32)).sum();
    let r = 11 - sum % 11;
    (if r >= 10 { 0 } else { r }) == d[8]
}

struct Detector {
    category: &'static str,
    re: Regex,
    check: fn(&str) -> bool,
}

fn always(_: &str) -> bool {
    true
}

fn detectors() -> Vec<Detector> {
    let d = |category: &'static str, re: &str, check: fn(&str) -> bool| Detector {
        category,
        re: Regex::new(re).expect("a valid built-in pattern"),
        check,
    };
    const MONTHS: &str = "enero|febrero|marzo|abril|mayo|junio|julio|agosto|septiembre|setiembre|octubre|noviembre|diciembre|january|february|march|april|may|june|july|august|september|october|november|december|janeiro|fevereiro|março|maio|junho|julho|setembro|outubro|novembro|dezembro|janvier|février|mars|avril|mai|juin|juillet|août|septembre|octobre|novembre|décembre|januar|februar|märz|juni|juli|oktober|dezember|ene|feb|mar|abr|jun|jul|ago|sep|sept|oct|nov|dic|jan|apr|aug|dec";
    vec![
        d("email", r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}", always),
        // 13 to 19 digits, grouped or not, that pass the cards' check.
        d("card", r"\b\d(?:[ -]?\d){12,18}\b", luhn),
        d("id", r"\b\d{3}-\d{2}-\d{4}\b", always),
        d("id", r"(?i)\b\d{8}[A-Z]\b", dni_ok),
        d("id", r"(?i)\b[XYZ]\d{7}[A-Z]\b", dni_ok),
        d("id", r"\b\d{3}\.?\d{3}\.?\d{3}-?\d{2}\b", cpf_ok),
        d("id", r"\b[1235689]\d{8}\b", nif_ok),
        d("id", r"\b[A-Z]{4}\d{6}[HM][A-Z]{5}[0-9A-Z]\d\b", always),
        d(
            "ip",
            r"\b(?:(?:25[0-5]|2[0-4]\d|1?\d?\d)\.){3}(?:25[0-5]|2[0-4]\d|1?\d?\d)\b",
            always,
        ),
        d(
            "ip",
            r"(?i)\b(?:[0-9a-f]{1,4}:){2,7}[0-9a-f]{1,4}\b|(?i)\b(?:[0-9a-f]{1,4}:){1,6}:[0-9a-f]{1,4}\b",
            always,
        ),
        d("url", r"(?i)\b(?:https?://|www\.)[^\s<>]+[^\s<>.,;:!?)]", always),
        d(
            "date",
            r"\b\d{4}-\d{2}-\d{2}\b|\b\d{1,2}[/.-]\d{1,2}[/.-]\d{2,4}\b",
            always,
        ),
        d(
            "date",
            &format!(
                r"(?i)\b\d{{1,2}}\.?(?:\s+de)?\s+(?:{MONTHS})\.?(?:\s+(?:de\s+)?\d{{4}})?\b|(?i)\b(?:{MONTHS})\.?\s+\d{{1,2}}(?:st|nd|rd|th)?,?\s+\d{{4}}\b"
            ),
            always,
        ),
        // Phone numbers: an optional +country, then groups, 7 to 15 digits in all.
        d(
            "phone",
            r"(?:\+\d{1,3}[\s.-]?)?(?:\(\d{1,4}\)[\s.-]?)?\d{2,4}(?:[\s.-]?\d{2,4}){1,4}",
            |s| (7..=15).contains(&s.chars().filter(char::is_ascii_digit).count()),
        ),
        // Postal addresses: a street and its number, or a postal code and a town.
        d(
            "address",
            r"(?i)\b(?:calle|c/|avenida|avda\.?|av\.|plaza|pza\.?|paseo|carrera|camino|rua|r\.|travessa|rue|avenue|boulevard|bd|chemin|straße|strasse|str\.|weg|platz)\s+[^\d,\n]{2,40}?,?\s*(?:n[ºo°]\.?\s*)?\d{1,5}[a-z]?\b",
            always,
        ),
        d(
            "address",
            r"\b\d{1,5}\s+(?:[A-Z][a-z]+\s+){1,3}(?:Street|St\.?|Avenue|Ave\.?|Road|Rd\.?|Boulevard|Blvd\.?|Lane|Ln\.?|Drive|Dr\.?|Court|Ct\.?|Way)\b",
            always,
        ),
        d(
            r"address",
            r"\b\d{5}(?:-\d{3})?\s+[A-ZÁÉÍÓÚÀÂÊÔÃÕÇÄÖÜ][a-záéíóúàâêôãõçäöüñ]+(?:[ -][A-ZÁÉÍÓÚ][a-záéíóúñ]+)*\b",
            always,
        ),
    ]
}

/// Lowercase, without accents and with single spaces: for words that are not exact.
pub fn fold(s: &str) -> String {
    let mut out = String::new();
    let mut space = false;
    for c in s.chars() {
        if c.is_whitespace() {
            space = !out.is_empty();
            continue;
        }
        if space {
            out.push(' ');
            space = false;
        }
        let base = match c {
            'á' | 'à' | 'â' | 'ä' | 'ã' | 'Á' | 'À' | 'Â' | 'Ä' | 'Ã' => 'a',
            'é' | 'è' | 'ê' | 'ë' | 'É' | 'È' | 'Ê' | 'Ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' | 'Í' | 'Ì' | 'Î' | 'Ï' => 'i',
            'ó' | 'ò' | 'ô' | 'ö' | 'õ' | 'Ó' | 'Ò' | 'Ô' | 'Ö' | 'Õ' => 'o',
            'ú' | 'ù' | 'û' | 'ü' | 'Ú' | 'Ù' | 'Û' | 'Ü' => 'u',
            'ñ' | 'Ñ' => 'n',
            'ç' | 'Ç' => 'c',
            c => c,
        };
        out.extend(base.to_lowercase());
    }
    out
}

/// A user's term as a regular expression over the line text.
fn term_regex(t: &Term) -> Result<Regex, String> {
    match &t.kind {
        TermKind::Pattern(p) => Regex::new(p).map_err(|e| e.to_string()),
        TermKind::Words { text, exact } => {
            // Not exact: compared without case, accents or extra spaces, on both sides.
            let text = if *exact { text.clone() } else { fold(text) };
            let words: Vec<String> = text.split_whitespace().map(regex::escape).collect();
            if words.is_empty() {
                return Err("empty".into());
            }
            let body = words.join(r"\s+");
            let re = if *exact { body } else { format!("(?i){body}") };
            Regex::new(&format!(r"\b{re}\b")).map_err(|e| e.to_string())
        }
    }
}

/// Everything sensitive in `rec` of the `categories` asked for and the `terms`; a text
/// is found once, by the most specific category. Terms count as `term:<name>`.
pub fn find(rec: &Recognized, categories: &[&str], terms: &[Term]) -> Vec<Found> {
    let detectors: Vec<Detector> = detectors()
        .into_iter()
        .filter(|d| categories.contains(&d.category))
        .collect();
    let user: Vec<(String, Regex, bool)> = terms
        .iter()
        .filter_map(|t| {
            let folded = matches!(&t.kind, TermKind::Words { exact: false, .. });
            term_regex(t)
                .ok()
                .map(|re| (format!("term:{}", t.name), re, folded))
        })
        .collect();
    let mut out = Vec::new();
    for line in rec.blocks.iter().flat_map(|b| &b.lines) {
        let l = Line::new(&line.words);
        let folded = fold(&l.text);
        // Candidates: (start char, end char, category, text).
        let mut hits: Vec<(usize, usize, String, String)> = Vec::new();
        for d in &detectors {
            for m in d.re.find_iter(&l.text) {
                if (d.check)(m.as_str()) {
                    hits.push((
                        char_index(&l.text, m.start()),
                        char_index(&l.text, m.end()),
                        d.category.to_string(),
                        m.as_str().to_string(),
                    ));
                }
            }
        }
        for (name, re, on_folded) in &user {
            // Folded text keeps one character per character when the line has no
            // repeated spaces, as built here.
            let hay = if *on_folded && folded.chars().count() == l.text.chars().count() {
                &folded
            } else {
                &l.text
            };
            for m in re.find_iter(hay) {
                let (a, b) = (char_index(hay, m.start()), char_index(hay, m.end()));
                let text: String = l.text.chars().skip(a).take(b - a).collect();
                hits.push((a, b, name.clone(), text));
            }
        }
        // The most specific first; the user's terms before all; no overlaps.
        let rank = |c: &str| {
            if c.starts_with("term:") {
                0
            } else {
                1 + PRIORITY.iter().position(|p| *p == c).unwrap_or(PRIORITY.len())
            }
        };
        hits.sort_by_key(|h| (rank(&h.2), h.0));
        let mut taken: Vec<(usize, usize)> = Vec::new();
        for (a, b, category, text) in hits {
            if taken.iter().any(|&(x, y)| a < y && x < b) {
                continue;
            }
            if let Some(r) = l.rect(a, b) {
                taken.push((a, b));
                out.push(Found {
                    category,
                    text,
                    rects: vec![r],
                });
            }
        }
    }
    out
}

/// Whether a user's pattern works (SMT-012): an invalid one is marked and not applied.
pub fn check_pattern(p: &str) -> Result<(), String> {
    Regex::new(p).map(drop).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Block, Line as RLine, Word};

    /// A line of words, each 10 px per character, one space (10 px) apart.
    fn rec(lines: &[&str]) -> Recognized {
        let mut blocks = Vec::new();
        for (li, text) in lines.iter().enumerate() {
            let y = li as f64 * 30.0;
            let mut x = 0.0;
            let words: Vec<Word> = text
                .split(' ')
                .map(|w| {
                    let width = w.chars().count() as f64 * 10.0;
                    let word = Word {
                        text: w.to_string(),
                        rect: [x, y, width, 20.0],
                        confidence: 1.0,
                        doubtful: false,
                    };
                    x += width + 10.0;
                    word
                })
                .collect();
            blocks.push(Block {
                rect: [0.0, y, x, 20.0],
                lines: vec![RLine {
                    text: text.to_string(),
                    rect: [0.0, y, x, 20.0],
                    words,
                    color: "#000000".into(),
                    background: "#FFFFFF".into(),
                    height: 20.0,
                }],
            });
        }
        Recognized {
            blocks,
            language: "es".into(),
        }
    }

    fn cats(found: &[Found]) -> Vec<(String, String)> {
        found
            .iter()
            .map(|f| (f.category.clone(), f.text.clone()))
            .collect()
    }

    /// SMT-010: a form with an email, a phone and a card: each once.
    #[test]
    fn a_form_with_three_data() {
        let r = rec(&[
            "Correo: ana.perez@example.com",
            "Teléfono: +34 612 345 678",
            "Tarjeta: 4111 1111 1111 1111",
        ]);
        let f = find(&r, &CATEGORIES, &[]);
        assert_eq!(
            cats(&f),
            vec![
                ("email".into(), "ana.perez@example.com".into()),
                ("phone".into(), "+34 612 345 678".into()),
                ("card".into(), "4111 1111 1111 1111".into()),
            ]
        );
        // Only the data: the label stays visible.
        assert!(f[0].rects[0][0] >= 80.0, "{:?}", f[0].rects);
    }

    /// SMT-011: 16 digits that fail the cards' check are not a card.
    #[test]
    fn a_number_that_is_not_a_card() {
        let f = find(&rec(&["Pedido 4111 1111 1111 1112"]), &["card"], &[]);
        assert!(f.is_empty(), "{f:?}");
    }

    #[test]
    fn identity_numbers_with_their_checks() {
        let f = find(
            &rec(&["DNI 12345678Z NIE X1234567L SSN 123-45-6789 CPF 529.982.247-25 NIF 123456789"]),
            &["id"],
            &[],
        );
        let texts: Vec<String> = f.iter().map(|f| f.text.clone()).collect();
        assert!(texts.contains(&"12345678Z".to_string()), "{texts:?}");
        assert!(texts.contains(&"X1234567L".to_string()), "{texts:?}");
        assert!(texts.contains(&"123-45-6789".to_string()));
        assert!(texts.contains(&"529.982.247-25".to_string()));
        assert!(texts.contains(&"123456789".to_string()));
        // A wrong letter is not a DNI.
        assert!(find(&rec(&["12345678A"]), &["id"], &[]).is_empty());
    }

    #[test]
    fn ip_url_date_and_address() {
        let f = find(
            &rec(&[
                "Servidor 192.168.1.20 en www.example.com/panel",
                "Fecha: 12/03/2026 y 3 de marzo de 2026",
                "Calle Mayor 15, 28013 Madrid",
            ]),
            &CATEGORIES,
            &[],
        );
        let c = cats(&f);
        assert!(c.contains(&("ip".into(), "192.168.1.20".into())), "{c:?}");
        assert!(
            c.contains(&("url".into(), "www.example.com/panel".into())),
            "{c:?}"
        );
        assert!(c.contains(&("date".into(), "12/03/2026".into())), "{c:?}");
        assert!(c.contains(&("date".into(), "3 de marzo de 2026".into())), "{c:?}");
        assert!(
            c.iter()
                .any(|(k, t)| k == "address" && t.starts_with("Calle Mayor 15")),
            "{c:?}"
        );
        assert!(
            c.iter().any(|(k, t)| k == "address" && t == "28013 Madrid"),
            "{c:?}"
        );
        // An IP is not a phone or a date (SMT-011).
        assert_eq!(c.iter().filter(|(_, t)| t.contains("192.168")).count(), 1);
    }

    /// SMT-012: words without exact match ignore case and accents; a pattern finds
    /// only what it says.
    #[test]
    fn the_users_terms() {
        let words = |exact| Term {
            name: "Proyecto Orión".into(),
            kind: TermKind::Words {
                text: "Proyecto Orión".into(),
                exact,
            },
        };
        let r = rec(&["Plan del proyecto orion para 2027"]);
        assert_eq!(find(&r, &[], &[words(false)]).len(), 1);
        assert!(find(&r, &[], &[words(true)]).is_empty());
        let po = Term {
            name: "Pedidos".into(),
            kind: TermKind::Pattern(r"PO-\d{6}\b".into()),
        };
        let f = find(&rec(&["PO-123456 y PO-12345"]), &[], &[po]);
        assert_eq!(cats(&f), vec![("term:Pedidos".into(), "PO-123456".into())]);
        assert!(check_pattern("PO-(").is_err());
    }

    #[test]
    fn folding() {
        assert_eq!(fold("  Proyecto   ORIÓN "), "proyecto orion");
    }
}
