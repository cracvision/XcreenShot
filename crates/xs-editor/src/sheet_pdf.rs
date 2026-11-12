//! The shortcut sheet as a PDF (APP-023): A4 pages drawn with the app's text engine,
//! two columns, the groups kept whole where they fit; one or two pages.

use image::{Rgba, RgbaImage};
use serde_json::Value;
use xs_image::export::{Kind, Options, Page};
use xs_image::fx_marks::label;

/// A4 at 150 dots per inch.
const DPI: f64 = 150.0;
const W: u32 = 1240;
// A little under the page, so the PDF never spills a sliver onto a second one.
const H: u32 = 1740;
const MARGIN: i64 = 70;
const GAP: i64 = 50;
const LINE: i64 = 27;

fn put(page: &mut RgbaImage, img: &RgbaImage, x: i64, y: i64) {
    image::imageops::overlay(page, img, x, y);
}

/// One line of a column: a group's title or a row (title on the left, keys on the
/// right).
enum Line<'a> {
    Head(&'a str),
    Row {
        title: &'a str,
        keys: &'a str,
        inactive: bool,
    },
}

pub fn render(sheet: &Value) -> Result<Vec<u8>, String> {
    let text = |k: &str| sheet[k].as_str().unwrap_or_default().to_string();
    let mut lines: Vec<Line> = Vec::new();
    for g in sheet["groups"].as_array().into_iter().flatten() {
        lines.push(Line::Head(g["title"].as_str().unwrap_or_default()));
        for r in g["rows"].as_array().into_iter().flatten() {
            lines.push(Line::Row {
                title: r["title"].as_str().unwrap_or_default(),
                keys: r["keys"].as_str().unwrap_or_default(),
                inactive: r["inactive"] == Value::Bool(true),
            });
        }
    }
    let col_w = (W as i64 - 2 * MARGIN - GAP) / 2;
    let top_first = MARGIN + 110;
    let bottom = H as i64 - MARGIN;
    let mut pages: Vec<RgbaImage> = Vec::new();
    let mut page = RgbaImage::from_pixel(W, H, Rgba([255, 255, 255, 255]));
    // The title and the profile, platform and date, on the first page.
    if let Some(t) = label(&text("title"), 16.0, true, "#111111", DPI) {
        put(&mut page, &t, MARGIN, MARGIN);
    }
    let meta = format!("{} · {} · {}", text("profile"), text("platform"), text("date"));
    if let Some(t) = label(&meta, 9.0, false, "#555555", DPI) {
        put(&mut page, &t, MARGIN, MARGIN + 52);
    }
    let (mut col, mut y, mut top) = (0, top_first, top_first);
    let inactive = text("inactive");
    for (i, line) in lines.iter().enumerate() {
        let need = match line {
            // A group's title does not stay alone at the foot of a column.
            Line::Head(_) => LINE * 3,
            Line::Row { .. } => LINE,
        };
        if y + need > bottom {
            col += 1;
            y = top;
            if col == 2 {
                pages.push(std::mem::replace(
                    &mut page,
                    RgbaImage::from_pixel(W, H, Rgba([255, 255, 255, 255])),
                ));
                col = 0;
                top = MARGIN;
                y = top;
            }
        }
        let x = MARGIN + col * (col_w + GAP);
        match line {
            Line::Head(title) => {
                if i > 0 && y > top {
                    y += 8;
                }
                if let Some(t) = label(title, 10.5, true, "#111111", DPI) {
                    put(&mut page, &t, x, y);
                }
                let rule_y = (y + LINE + 2) as u32;
                for xx in x..x + col_w {
                    if (xx as u32) < W && rule_y < H {
                        page.put_pixel(xx as u32, rule_y, Rgba([150, 150, 150, 255]));
                    }
                }
                y += LINE + 8;
            }
            Line::Row {
                title,
                keys,
                inactive: off,
            } => {
                let color = if *off { "#888888" } else { "#111111" };
                let keys = if *off {
                    format!("{keys} ({inactive})")
                } else {
                    keys.to_string()
                };
                let k = label(&keys, 8.5, true, color, DPI);
                let kw = k.as_ref().map_or(0, |k| k.width() as i64);
                if let Some(mut t) = label(title, 8.5, false, color, DPI) {
                    // A long name is cut so it never runs into the keys.
                    let room = (col_w - kw - 16).max(40) as u32;
                    if t.width() > room {
                        t = image::imageops::crop_imm(&t, 0, 0, room, t.height()).to_image();
                    }
                    put(&mut page, &t, x, y);
                }
                if let Some(k) = k {
                    put(&mut page, &k, x + col_w - kw, y);
                }
                y += LINE;
            }
        }
    }
    pages.push(page);
    let opts = Options {
        pdf_page: "a4".to_string(),
        pdf_orientation: "portrait".to_string(),
        pdf_margin: 0.0,
        pdf_fit_width: true,
        pdf_lossless: true,
        ..Options::default()
    };
    let pages: Vec<Page> = pages.into_iter().map(|image| Page { image, dpi: DPI }).collect();
    xs_image::export::encode(&pages, Kind::Pdf, &opts, xs_image::color::Rgba::WHITE)
}
