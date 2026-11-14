//! The clipboard as a source (CAP-039): what was copied in another app goes down the
//! capture route (effects, destinations, file name, library), without selecting on
//! screen.
//!
//! What it takes, in this order: an image (its transparency kept); image files copied in
//! the Explorer, one capture each; text, as an editable Text object on a white canvas
//! 1040 px wide (lines wrapped at 1000 px, 20 px margins), cut at 60 000 px with a
//! notice. Anything else, or nothing, says so and creates no capture.

use std::path::PathBuf;

use image::RgbaImage;
use serde_json::{json, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode};

use crate::{finish, Options, Rect, Shot};

/// Text is wrapped at this width, with this margin around it.
const TEXT_WIDTH: f64 = 1000.0;
const MARGIN: u32 = 20;
/// Tallest text capture (as a scrolling capture, CAP-036).
const MOST: u32 = 60_000;
/// Image files that become captures.
const IMAGE_FILES: [&str; 7] = ["png", "jpg", "jpeg", "bmp", "gif", "tif", "tiff"];

/// What the clipboard holds.
pub enum Content {
    Image(RgbaImage),
    Files(Vec<PathBuf>),
    Text(String),
    Nothing,
}

/// Reads the clipboard, in the order of preference of CAP-039.
pub fn read() -> Content {
    // One use of the clipboard at a time in the process (see xs_core::clipboard_lock).
    let _lock = xs_core::clipboard_lock();
    let Ok(mut cb) = arboard::Clipboard::new() else {
        return Content::Nothing;
    };
    if let Ok(img) = cb.get_image() {
        if let Some(image) = RgbaImage::from_raw(img.width as u32, img.height as u32, img.bytes.into_owned())
        {
            return Content::Image(image);
        }
    }
    if let Ok(files) = cb.get().file_list() {
        let images: Vec<PathBuf> = files
            .into_iter()
            .filter(|p| {
                p.extension()
                    .map(|e| e.to_string_lossy().to_lowercase())
                    .is_some_and(|e| IMAGE_FILES.contains(&e.as_str()))
            })
            .collect();
        if !images.is_empty() {
            return Content::Files(images);
        }
    }
    match cb.get_text() {
        Ok(text) if !text.trim().is_empty() => Content::Text(text),
        _ => Content::Nothing,
    }
}

/// The white canvas and the Text object for `text`, and whether it had to be cut.
pub fn text_capture(text: &str) -> (RgbaImage, Value, bool) {
    let object = |t: &str| json!({ "at": [MARGIN, MARGIN], "width": TEXT_WIDTH, "text": t });
    let height_of = |t: &str| -> u32 {
        let props = object(t);
        xs_image::text::Text::from_props(props.as_object().expect("an object"))
            .map(|parsed| xs_image::text::layout(&parsed, 96.0).height.ceil() as u32)
            .unwrap_or(40)
    };
    let mut kept = text.trim_end().to_string();
    let mut cut = false;
    // Too tall: keep the lines that fit.
    while height_of(&kept) + 2 * MARGIN > MOST {
        cut = true;
        let lines: Vec<&str> = kept.lines().collect();
        if lines.len() <= 1 {
            kept = kept.chars().take(kept.chars().count() / 2).collect();
        } else {
            kept = lines[..lines.len() * 9 / 10].join("\n");
        }
    }
    let height = (height_of(&kept) + 2 * MARGIN).min(MOST);
    let width = TEXT_WIDTH as u32 + 2 * MARGIN;
    let canvas = RgbaImage::from_pixel(width, height, image::Rgba([255, 255, 255, 255]));
    let mut obj = object(&kept);
    obj["type"] = json!("text");
    (canvas, obj, cut)
}

fn shot(image: RgbaImage) -> Shot {
    Shot {
        rect: Rect {
            x: 0,
            y: 0,
            width: image.width(),
            height: image.height(),
        },
        image,
        mode: "clipboard",
        scale: 1.0,
        monitor: String::new(),
        // The app that copied it, when the system says (CAP-039, CAP-056).
        app: crate::platform::clipboard_app(),
        window_title: None,
        window_id: None,
        pointer: None,
        warnings: Vec::new(),
    }
}

/// `capture.clipboard`: the clipboard's content as a capture (CAP-039).
pub fn run(core: &Core, ctx: &Context, opts: &Options) -> CommandResult {
    let lang = ctx.lang;
    // No pointer in a capture from the clipboard.
    let mut opts = opts.clone();
    opts.cursor = false;
    match read() {
        Content::Image(image) => finish(core, ctx, shot(image), &opts),
        Content::Files(paths) => {
            let mut last = None;
            for path in paths {
                let image = std::fs::read(&path)
                    .map_err(|e| e.to_string())
                    .and_then(|b| xs_image::encode::decode(&b));
                match image {
                    Ok(image) => last = Some(finish(core, ctx, shot(image), &opts)?),
                    Err(e) => tracing::warn!(error = %e, "clipboard: a file was not read"),
                }
            }
            last.ok_or_else(|| empty(lang))
        }
        Content::Text(text) => {
            let (canvas, object, cut) = text_capture(&text);
            if cut {
                core.host().notify(
                    "problems",
                    &tr(lang, "app.name", &[]),
                    &tr(lang, "notify.clipboard_text_cut", &[]),
                );
            }
            opts.objects = vec![object];
            finish(core, ctx, shot(canvas), &opts)
        }
        Content::Nothing => {
            // A series (CAP-049) just waits for something to be copied.
            if opts.series.is_none() {
                core.host().notify(
                    "problems",
                    &tr(lang, "app.name", &[]),
                    &tr(lang, "errors.clipboard_empty", &[]),
                );
            }
            Err(empty(lang))
        }
    }
}

fn empty(lang: xs_core::Lang) -> CommandError {
    CommandError::new(ErrorCode::NotFound, tr(lang, "errors.clipboard_empty", &[]))
        .with_details(json!({ "status": "failed" }))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// CAP-039: three paragraphs, all of them, on a white canvas with margins.
    #[test]
    fn text_becomes_a_white_canvas_with_a_text_object() {
        let text = "Primer párrafo.\n\nSegundo párrafo, algo más largo que el primero.\n\nTercero y último.";
        let (canvas, object, cut) = text_capture(text);
        assert!(!cut);
        assert_eq!(canvas.width(), 1040);
        assert!(canvas.height() > 2 * MARGIN);
        assert_eq!(canvas.get_pixel(5, 5).0, [255, 255, 255, 255]);
        assert_eq!(object["type"], "text");
        assert_eq!(object["text"], text);
        assert_eq!(object["width"], 1000.0);
    }

    #[test]
    fn very_long_text_is_cut_to_the_largest_size() {
        let text = "línea de texto\n".repeat(20_000);
        let (canvas, object, cut) = text_capture(&text);
        assert!(cut);
        assert!(canvas.height() <= MOST);
        assert!(object["text"].as_str().unwrap().len() < text.len());
    }
}
