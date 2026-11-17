//! The small formatting of a guide's texts (SMT-025, SMT-029): `**bold**`, `*italic*`
//! or `_italic_`, links as `[label](https://…)`, and lines that start with `- ` or
//! `* ` as bullets. The marks are not drawn; a link shows its label. The same reading
//! serves the canvas and every export, so they all agree.

/// A piece of text with one look.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Run {
    pub text: String,
    pub bold: bool,
    pub italic: bool,
    /// A link's address; `text` is its label.
    pub link: Option<String>,
}

/// Whether `text` has any mark worth reading.
pub fn has_marks(text: &str) -> bool {
    text.contains('*') || text.contains('_') || text.contains("](") || bulleted(text) != text
}

/// Lines that start with `- ` or `* ` (after their indentation) start with `• `.
pub fn bulleted(text: &str) -> String {
    text.split('\n')
        .map(|line| {
            let trimmed = line.trim_start();
            match trimmed.strip_prefix("- ").or_else(|| trimmed.strip_prefix("* ")) {
                Some(rest) => format!("{}• {rest}", &line[..line.len() - trimmed.len()]),
                None => line.to_string(),
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn word(c: Option<&char>) -> bool {
    c.is_some_and(|c| c.is_alphanumeric())
}

fn space(c: Option<&char>) -> bool {
    c.is_none_or(|c| c.is_whitespace())
}

/// `[label](address)` from `at`: its label, address and where it ends.
fn link_at(chars: &[char], at: usize) -> Option<(String, String, usize)> {
    let close = (at + 1..chars.len()).find(|&i| chars[i] == ']')?;
    if chars.get(close + 1) != Some(&'(') {
        return None;
    }
    let end = (close + 2..chars.len()).find(|&i| chars[i] == ')')?;
    let label: String = chars[at + 1..close].iter().collect();
    let url: String = chars[close + 2..end].iter().collect();
    if label.trim().is_empty() || url.is_empty() || url.contains(char::is_whitespace) {
        return None;
    }
    Some((label, url, end + 1))
}

/// The runs of `text` (bullets included: see [`bulleted`]).
pub fn parse(text: &str) -> Vec<Run> {
    let text = bulleted(text);
    let chars: Vec<char> = text.chars().collect();
    let mut out: Vec<Run> = Vec::new();
    let (mut bold, mut italic) = (false, false);
    let mut buf = String::new();
    let flush = |buf: &mut String, out: &mut Vec<Run>, bold: bool, italic: bool| {
        if !buf.is_empty() {
            out.push(Run {
                text: std::mem::take(buf),
                bold,
                italic,
                link: None,
            });
        }
    };
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let (before, after) = (i.checked_sub(1).and_then(|j| chars.get(j)), chars.get(i + 1));
        if c == '*' && after == Some(&'*') {
            flush(&mut buf, &mut out, bold, italic);
            bold = !bold;
            i += 2;
            continue;
        }
        if c == '*' || c == '_' {
            // Opens before a word and not inside one (file_name stays); closes after one.
            let opens = !italic && !space(after) && !word(before);
            let closes = italic && !space(before) && !word(after);
            if opens || closes {
                flush(&mut buf, &mut out, bold, italic);
                italic = !italic;
                i += 1;
                continue;
            }
        }
        if c == '[' {
            if let Some((label, url, end)) = link_at(&chars, i) {
                flush(&mut buf, &mut out, bold, italic);
                out.push(Run {
                    text: label,
                    bold,
                    italic,
                    link: Some(url),
                });
                i = end;
                continue;
            }
        }
        buf.push(c);
        i += 1;
    }
    flush(&mut buf, &mut out, bold, italic);
    out
}

/// The text without its marks; a link as its label.
pub fn plain(text: &str) -> String {
    parse(text).into_iter().map(|r| r.text).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn r(text: &str, bold: bool, italic: bool) -> Run {
        Run {
            text: text.into(),
            bold,
            italic,
            link: None,
        }
    }

    /// SMT-029: bold, italic, links and bullets; marks inside words and lone stars stay.
    #[test]
    fn the_marks_of_a_guide_text() {
        assert_eq!(
            parse("Clic en **Guardar** y *luego* _cerrar_"),
            [
                r("Clic en ", false, false),
                r("Guardar", true, false),
                r(" y ", false, false),
                r("luego", false, true),
                r(" ", false, false),
                r("cerrar", false, true),
            ]
        );
        assert_eq!(
            parse("Ver [la ayuda](https://example.com/a) ahora"),
            [
                r("Ver ", false, false),
                Run {
                    text: "la ayuda".into(),
                    link: Some("https://example.com/a".into()),
                    ..Run::default()
                },
                r(" ahora", false, false),
            ]
        );
        assert_eq!(plain("file_name y 2 * 3"), "file_name y 2 * 3");
        assert_eq!(plain("- uno\n  * dos\ntres"), "• uno\n  • dos\ntres");
        assert_eq!(plain("**Texto** [sin enlace]"), "Texto [sin enlace]");
        assert!(has_marks("- a") && has_marks("*a*") && !has_marks("normal"));
    }
}
