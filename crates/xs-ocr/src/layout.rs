//! Reading order and paragraphs (SMT-003), and the text as plain text or HTML
//! (SMT-006).
//!
//! - Lines much wider than the rest (a title across the page) split the page into
//!   bands; inside a band, lines that never overlap horizontally form columns, read
//!   left to right, each one top to bottom.
//! - Two paragraphs part where the space between lines is bigger than the line
//!   spacing, or where the letter size changes a lot (a heading).

use crate::{union, Block, Line};

fn median(v: Vec<f64>) -> f64 {
    quantile(v, 0.5)
}

fn quantile(mut v: Vec<f64>, q: f64) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.sort_by(f64::total_cmp);
    v[((v.len() as f64 * q) as usize).min(v.len() - 1)]
}

/// The lines, in reading order, as paragraphs.
pub fn layout(mut lines: Vec<Line>) -> Vec<Block> {
    if lines.is_empty() {
        return Vec::new();
    }
    let left = lines.iter().map(|l| l.rect[0]).fold(f64::MAX, f64::min);
    let right = lines
        .iter()
        .map(|l| l.rect[0] + l.rect[2])
        .fold(f64::MIN, f64::max);
    let width = (right - left).max(1.0);
    lines.sort_by(|a, b| a.rect[1].total_cmp(&b.rect[1]));
    // Bands split by the lines that cross the page, when the rest stands in columns.
    let narrow: Vec<Line> = lines
        .iter()
        .filter(|l| l.rect[2] <= width * 0.6)
        .cloned()
        .collect();
    let in_columns = columns(narrow).len() > 1;
    let wide = |l: &Line| in_columns && l.rect[2] > width * 0.6;
    let mut ordered: Vec<(usize, Line)> = Vec::new(); // (group, line)
    let mut group = 0usize;
    let mut band: Vec<Line> = Vec::new();
    let flush = |band: &mut Vec<Line>, group: &mut usize, ordered: &mut Vec<(usize, Line)>| {
        for column in columns(std::mem::take(band)) {
            *group += 1;
            ordered.extend(column.into_iter().map(|l| (*group, l)));
        }
    };
    for line in lines {
        if wide(&line) {
            flush(&mut band, &mut group, &mut ordered);
            group += 1;
            ordered.push((group, line));
        } else {
            band.push(line);
        }
    }
    flush(&mut band, &mut group, &mut ordered);

    // Paragraphs.
    let heights = median(ordered.iter().map(|(_, l)| l.rect[3]).collect());
    // The usual space between the lines of a paragraph: most gaps are that one, even
    // when paragraphs are short.
    let gaps = quantile(
        ordered
            .windows(2)
            .filter(|w| w[0].0 == w[1].0)
            .map(|w| w[1].1.rect[1] - (w[0].1.rect[1] + w[0].1.rect[3]))
            .filter(|g| *g >= 0.0)
            .collect(),
        0.25,
    );
    let mut blocks: Vec<Block> = Vec::new();
    let mut last: Option<(usize, Line)> = None;
    for (g, line) in ordered {
        let new_paragraph = match &last {
            None => true,
            Some((lg, prev)) => {
                let gap = line.rect[1] - (prev.rect[1] + prev.rect[3]);
                let size = line.rect[3].max(1.0) / prev.rect[3].max(1.0);
                *lg != g || gap > (gaps * 1.8).max(heights * 0.6) || !(0.72..=1.4).contains(&size)
            }
        };
        if new_paragraph || blocks.is_empty() {
            blocks.push(Block {
                rect: line.rect,
                lines: Vec::new(),
            });
        }
        let block = blocks.last_mut().expect("a block");
        block.lines.push(line.clone());
        block.rect = union(block.lines.iter().map(|l| l.rect));
        last = Some((g, line));
    }
    blocks
}

/// Lines of a band in columns: groups whose horizontal extents never touch, left to
/// right, each top to bottom.
fn columns(lines: Vec<Line>) -> Vec<Vec<Line>> {
    if lines.is_empty() {
        return Vec::new();
    }
    let mut spans: Vec<(f64, f64)> = lines.iter().map(|l| (l.rect[0], l.rect[0] + l.rect[2])).collect();
    spans.sort_by(|a, b| a.0.total_cmp(&b.0));
    let mut merged: Vec<(f64, f64)> = Vec::new();
    for (a, b) in spans {
        match merged.last_mut() {
            Some(last) if a <= last.1 + 4.0 => last.1 = last.1.max(b),
            _ => merged.push((a, b)),
        }
    }
    let mut out: Vec<Vec<Line>> = vec![Vec::new(); merged.len()];
    for line in lines {
        let x = line.rect[0];
        let i = merged
            .iter()
            .position(|(a, b)| x >= *a - 0.5 && x <= *b)
            .unwrap_or(0);
        out[i].push(line);
    }
    // A "column" of a single short line beside a real one is the same text broken by
    // a big space: keep the reading by rows then.
    if out.len() > 1 && out.iter().any(|c| c.len() < 2) {
        let mut all: Vec<Line> = out.into_iter().flatten().collect();
        all.sort_by(|a, b| {
            let (ya, yb) = (a.rect[1], b.rect[1]);
            if (ya - yb).abs() < a.rect[3].min(b.rect[3]) * 0.5 {
                a.rect[0].total_cmp(&b.rect[0])
            } else {
                ya.total_cmp(&yb)
            }
        });
        return vec![all];
    }
    for c in &mut out {
        c.sort_by(|a, b| a.rect[1].total_cmp(&b.rect[1]));
    }
    out
}

/// The text: paragraphs apart by an empty line; the lines of a paragraph on their own
/// lines, or joined (`join`, with a word broken by a hyphen made whole again).
pub fn plain(blocks: &[Block], join: bool) -> String {
    blocks
        .iter()
        .map(|b| {
            if !join {
                return b
                    .lines
                    .iter()
                    .map(|l| l.text.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
            }
            let mut out = String::new();
            for l in &b.lines {
                if out.ends_with('-') && l.text.chars().next().is_some_and(char::is_lowercase) {
                    out.pop();
                } else if !out.is_empty() {
                    out.push(' ');
                }
                out.push_str(&l.text);
            }
            out
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The text with its approximate format (SMT-006): each line keeps its color and its
/// size relative to the rest; the paragraphs are paragraphs.
pub fn html(blocks: &[Block], join: bool) -> String {
    let base = median(blocks.iter().flat_map(|b| &b.lines).map(|l| l.height).collect()).max(1.0);
    let mut out = String::from("<div style=\"font-family: 'Segoe UI', Arial, sans-serif\">");
    for b in blocks {
        out.push_str("<p style=\"margin: 0 0 1em\">");
        for (i, l) in b.lines.iter().enumerate() {
            if i > 0 {
                out.push_str(if join { " " } else { "<br>" });
            }
            let mut style = String::new();
            // Black-ish text keeps the reader's own color; colored text keeps its own.
            if l.color != "#000000" && !is_dark(&l.color) {
                style.push_str(&format!("color: {};", l.color));
            }
            let rel = l.height / base;
            if !(0.85..=1.15).contains(&rel) {
                style.push_str(&format!("font-size: {:.2}em;", rel));
            }
            if style.is_empty() {
                out.push_str(&escape(&l.text));
            } else {
                out.push_str(&format!("<span style=\"{style}\">{}</span>", escape(&l.text)));
            }
        }
        out.push_str("</p>");
    }
    out.push_str("</div>");
    out
}

fn channels(hex: &str) -> [u8; 3] {
    let v = u32::from_str_radix(hex.trim_start_matches('#'), 16).unwrap_or(0);
    [(v >> 16) as u8, (v >> 8) as u8, v as u8]
}

/// Black-ish text keeps the reader's own color.
fn is_dark(hex: &str) -> bool {
    let [r, g, b] = channels(hex);
    (r as u32 + g as u32 + b as u32) < 150 && !is_colored(hex)
}

fn is_colored(hex: &str) -> bool {
    let [r, g, b] = channels(hex);
    let (max, min) = (r.max(g).max(b), r.min(g).min(b));
    max as i32 - min as i32 > 60
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Word;

    fn line(text: &str, x: f64, y: f64, w: f64, h: f64) -> Line {
        Line {
            text: text.into(),
            rect: [x, y, w, h],
            words: vec![Word {
                text: text.into(),
                rect: [x, y, w, h],
                confidence: 1.0,
                doubtful: false,
            }],
            color: "#000000".into(),
            background: "#FFFFFF".into(),
            height: h,
        }
    }

    fn texts(blocks: &[Block]) -> Vec<Vec<String>> {
        blocks
            .iter()
            .map(|b| b.lines.iter().map(|l| l.text.clone()).collect())
            .collect()
    }

    /// SMT-006 criterion: three paragraphs give three paragraphs, apart by an empty line.
    #[test]
    fn paragraphs_part_where_the_space_grows() {
        let lines = vec![
            line("Uno a", 10.0, 10.0, 300.0, 16.0),
            line("uno b", 10.0, 30.0, 280.0, 16.0),
            line("Dos a", 10.0, 70.0, 300.0, 16.0),
            line("dos b", 10.0, 90.0, 250.0, 16.0),
            line("Tres", 10.0, 130.0, 200.0, 16.0),
        ];
        let blocks = layout(lines);
        assert_eq!(
            texts(&blocks),
            [vec!["Uno a", "uno b"], vec!["Dos a", "dos b"], vec!["Tres"]]
        );
        assert_eq!(plain(&blocks, false), "Uno a\nuno b\n\nDos a\ndos b\n\nTres");
        assert_eq!(plain(&blocks, true), "Uno a uno b\n\nDos a dos b\n\nTres");
    }

    /// SMT-003 criterion: on a page in two columns, the left one is read first.
    #[test]
    fn a_column_is_read_whole_before_the_next() {
        let mut lines = vec![line("Título de la página", 10.0, 0.0, 600.0, 24.0)];
        for i in 0..4 {
            let y = 40.0 + i as f64 * 20.0;
            lines.push(line(&format!("izq {i}"), 10.0, y, 260.0, 16.0));
            lines.push(line(&format!("der {i}"), 340.0, y, 260.0, 16.0));
        }
        let text = plain(&layout(lines), false);
        let left = text.find("izq 3").unwrap();
        let right = text.find("der 0").unwrap();
        assert!(left < right, "{text}");
        assert!(text.starts_with("Título de la página"));
    }

    #[test]
    fn a_heading_is_its_own_paragraph_and_hyphens_join() {
        let lines = vec![
            line("Título grande", 10.0, 10.0, 300.0, 30.0),
            line("texto que se cor-", 10.0, 44.0, 300.0, 16.0),
            line("ta aquí", 10.0, 62.0, 100.0, 16.0),
        ];
        let blocks = layout(lines);
        assert_eq!(blocks.len(), 2);
        assert_eq!(plain(&blocks, true), "Título grande\n\ntexto que se corta aquí");
    }

    /// SMT-006 criterion: a red word stays red in the formatted copy; sizes are relative.
    #[test]
    fn the_html_keeps_color_and_size() {
        let mut red = line("rojo", 10.0, 10.0, 60.0, 16.0);
        red.color = "#D01010".into();
        let blocks = layout(vec![
            line("<Título>", 10.0, 0.0, 200.0, 32.0),
            red,
            line("normal", 10.0, 30.0, 60.0, 16.0),
        ]);
        let h = html(&blocks, false);
        assert!(h.contains("color: #D01010"), "{h}");
        assert!(h.contains("font-size: 2.00em"), "{h}");
        assert!(h.contains("&lt;Título&gt;"), "{h}");
        assert!(!h.contains("color: #000000"));
    }
}
