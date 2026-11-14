//! File names of the File destination (CAP-053): a pattern of text and markers in
//! braces, the same on Windows and macOS.
//!
//! Markers: `{prefix}`, `{date}`, `{date:compact}`, `{time}`, `{time:compact}`, `{app}`,
//! `{title}`, `{site}`, `{mode}`, `{width}`, `{height}`, `{user}`, `{counter}` and
//! `{counter:N}` (N digits, 1 to 9; 4 by default). A `/` written in the pattern makes
//! subfolders. What a marker brings is cleaned: characters no system allows become `-`;
//! an empty marker goes, and the separators it leaves doubled become one. Trailing
//! spaces and dots go, reserved Windows names get a `_`, and a name is at most 200
//! characters.

use chrono::{DateTime, Local};

/// What the markers stand for, for one capture.
#[derive(Debug, Clone, Default)]
pub struct Values {
    pub prefix: String,
    pub app: Option<String>,
    pub title: Option<String>,
    pub site: Option<String>,
    pub mode: String,
    pub width: u32,
    pub height: u32,
    pub user: String,
}

/// The default pattern (CAP-053).
pub const DEFAULT_PATTERN: &str = "{prefix}_{date}_{time}";

/// Longest file name, without its extension.
const MOST: usize = 200;
/// Characters that separate words in a name.
const SEPARATORS: [char; 4] = ['_', '-', ' ', '.'];

/// The markers a pattern may use, for the Insert button (CAP-053).
pub const MARKERS: [&str; 14] = [
    "prefix",
    "date",
    "date:compact",
    "time",
    "time:compact",
    "app",
    "title",
    "site",
    "mode",
    "width",
    "height",
    "user",
    "counter",
    "counter:4",
];

/// Whether the pattern numbers its files.
pub fn uses_counter(pattern: &str) -> bool {
    pattern.contains("{counter")
}

/// The subfolders and the file name (without extension) a pattern gives.
pub fn render(pattern: &str, values: &Values, now: DateTime<Local>, counter: u32) -> (Vec<String>, String) {
    let pattern = if pattern.trim().is_empty() {
        DEFAULT_PATTERN
    } else {
        pattern
    };
    let mut parts: Vec<String> = pattern
        .split(['/', '\\'])
        .map(|segment| clean_name(&render_segment(segment, values, now, counter)))
        .filter(|s| !s.is_empty())
        .collect();
    let name = parts.pop().unwrap_or_else(|| "XcreenShot".to_string());
    (parts, name)
}

fn render_segment(segment: &str, values: &Values, now: DateTime<Local>, counter: u32) -> String {
    let mut out = String::new();
    let mut rest = segment;
    while let Some(open) = rest.find('{') {
        out.push_str(&clean_text(&rest[..open]));
        let Some(close) = rest[open..].find('}') else {
            out.push_str(&clean_text(&rest[open..]));
            rest = "";
            break;
        };
        let marker = &rest[open + 1..open + close];
        match value(marker, values, now, counter) {
            // An empty marker goes, and so does the doubled separator it leaves.
            Some(v) if !v.is_empty() => out.push_str(&clean_text(&v)),
            Some(_) => out.push('\u{0}'),
            // Not a marker: kept as written.
            None => out.push_str(&clean_text(&rest[open..open + close + 1])),
        }
        rest = &rest[open + close + 1..];
    }
    out.push_str(&clean_text(rest));
    collapse(&out)
}

fn value(marker: &str, v: &Values, now: DateTime<Local>, counter: u32) -> Option<String> {
    let text = |o: &Option<String>| o.clone().unwrap_or_default();
    Some(match marker {
        "prefix" => v.prefix.clone(),
        "date" => now.format("%Y-%m-%d").to_string(),
        "date:compact" => now.format("%Y%m%d").to_string(),
        "time" => now.format("%H-%M-%S").to_string(),
        "time:compact" => now.format("%H%M%S").to_string(),
        "app" => text(&v.app),
        "title" => text(&v.title),
        "site" => text(&v.site),
        "mode" => v.mode.clone(),
        "width" => v.width.to_string(),
        "height" => v.height.to_string(),
        "user" => v.user.clone(),
        "counter" => format!("{counter:04}"),
        m => {
            let digits: usize = m.strip_prefix("counter:")?.parse().ok()?;
            let digits = digits.clamp(1, 9);
            format!("{counter:0digits$}")
        }
    })
}

/// Characters some system does not allow in a name become `-`.
fn clean_text(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_control() || matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|') {
                '-'
            } else {
                c
            }
        })
        .collect()
}

/// Where an empty marker was (`\0`), the separators around it become one.
fn collapse(s: &str) -> String {
    let mut out = String::new();
    let mut gap = false;
    for c in s.chars() {
        if c == '\u{0}' {
            gap = true;
            continue;
        }
        if gap {
            gap = false;
            if SEPARATORS.contains(&c) && out.ends_with(SEPARATORS) {
                continue;
            }
        }
        out.push(c);
    }
    out.trim_start_matches(SEPARATORS).to_string()
}

/// The finishing touches of a name: no trailing spaces or dots, not a reserved Windows
/// name, at most 200 characters.
fn clean_name(s: &str) -> String {
    let mut name: String = s.chars().take(MOST).collect();
    name = name.trim_end_matches([' ', '.', '_', '-']).trim().to_string();
    let stem = name.split('.').next().unwrap_or_default().to_ascii_uppercase();
    let reserved = matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
        || ((stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.len() == 4
            && stem.as_bytes()[3].is_ascii_digit());
    if reserved {
        name.push('_');
    }
    name
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn at() -> DateTime<Local> {
        Local.with_ymd_and_hms(2026, 10, 12, 14, 5, 33).unwrap()
    }

    fn values() -> Values {
        Values {
            prefix: "XcreenShot".into(),
            app: Some("Excel".into()),
            title: Some("Informe Q3".into()),
            site: None,
            mode: "region".into(),
            width: 1920,
            height: 1080,
            user: "carlos".into(),
        }
    }

    #[test]
    fn the_default_pattern() {
        assert_eq!(
            render(DEFAULT_PATTERN, &values(), at(), 1),
            (vec![], "XcreenShot_2026-10-12_14-05-33".to_string())
        );
    }

    /// CAP-053: `{prefix}_{date}_{counter:3}` with prefix Informe.
    #[test]
    fn the_counter_has_its_digits() {
        let v = Values {
            prefix: "Informe".into(),
            ..values()
        };
        assert_eq!(
            render("{prefix}_{date}_{counter:3}", &v, at(), 1).1,
            "Informe_2026-10-12_001"
        );
        assert_eq!(
            render("{prefix}_{date}_{counter:3}", &v, at(), 2).1,
            "Informe_2026-10-12_002"
        );
        assert_eq!(render("{counter}", &v, at(), 7).1, "0007");
        assert!(uses_counter("{prefix}_{counter:3}"));
        assert!(!uses_counter(DEFAULT_PATTERN));
    }

    /// CAP-053: a title `a/b: c?` gives `a-b- c-` in the name.
    #[test]
    fn markers_lose_the_characters_no_system_allows() {
        let v = Values {
            title: Some("a/b: c?".into()),
            ..values()
        };
        let (_, name) = render("{title}_{app}", &v, at(), 1);
        assert_eq!(name, "a-b- c-_Excel");
    }

    /// CAP-053: a `/` in the pattern makes subfolders.
    #[test]
    fn a_slash_makes_subfolders() {
        assert_eq!(
            render("{date}/{time}", &values(), at(), 1),
            (vec!["2026-10-12".to_string()], "14-05-33".to_string())
        );
    }

    #[test]
    fn an_empty_marker_and_its_separator_go() {
        assert_eq!(
            render("{prefix}_{site}_{date}", &values(), at(), 1).1,
            "XcreenShot_2026-10-12"
        );
        assert_eq!(render("{site}-{app}", &values(), at(), 1).1, "Excel");
        assert_eq!(render("{app} {site}", &values(), at(), 1).1, "Excel");
    }

    #[test]
    fn names_end_clean_and_are_never_reserved() {
        let v = Values {
            prefix: "CON".into(),
            ..values()
        };
        assert_eq!(render("{prefix}", &v, at(), 1).1, "CON_");
        let v = Values {
            prefix: "nombre. ".into(),
            ..values()
        };
        assert_eq!(render("{prefix}", &v, at(), 1).1, "nombre");
        let v = Values {
            prefix: "x".repeat(300),
            ..values()
        };
        assert_eq!(render("{prefix}", &v, at(), 1).1.len(), 200);
    }

    #[test]
    fn text_that_is_not_a_marker_stays() {
        assert_eq!(
            render("Captura {nope} {width}x{height}", &values(), at(), 1).1,
            "Captura {nope} 1920x1080"
        );
    }
}
