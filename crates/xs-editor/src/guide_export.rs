//! `smart.guide_export` (SMT-030, SMT-032 to SMT-035): a guide of steps in Word,
//! PowerPoint, Markdown, HTML, PDF, pictures apart or the clipboard. Each step's picture
//! is drawn with its annotations, as the guide shows it.

use base64::Engine;

use super::*;
use crate::guide_out::{self, Guide, Item, Pic};
use crate::templates::Template;

fn text_of(doc: &Document, slot: &str) -> String {
    doc.objects
        .iter()
        .find(|o| o.props.get("slot").and_then(Value::as_str) == Some(slot))
        .and_then(|o| o.props.get("text").and_then(Value::as_str))
        .unwrap_or_default()
        .to_string()
}

fn rect(o: &Object) -> Option<[f64; 4]> {
    let r = o.props.get("rect")?;
    Some([
        r.get("x")?.as_f64()?,
        r.get("y")?.as_f64()?,
        r.get("w")?.as_f64()?,
        r.get("h")?.as_f64()?,
    ])
}

/// A section's picture with what is drawn on it: the guide's band drawn alone, cut to
/// the picture's rectangle.
fn picture_of(snap: &store::Snapshot, top: f64, height: f64, zone: [f64; 4]) -> Option<Pic> {
    let mut doc = snap.document.clone();
    let dpi = doc.dpi;
    doc.objects
        .retain(|o| xs_image::object_bounds(o, dpi).is_some_and(|b| b[3] > top && b[1] < top + height));
    for o in doc.objects.iter_mut() {
        o.translate(0.0, -top);
    }
    let h = height.ceil().max(1.0) as u32;
    doc.canvas.height = h;
    doc.crop.height = h;
    let (img, origin) = xs_image::render::render_final(&doc, &snap.base, &snap.meta);
    let x = (zone[0] + origin[0] as f64).round().max(0.0) as u32;
    let y = (zone[1] - top + origin[1] as f64).round().max(0.0) as u32;
    let w = (zone[2].round() as u32).min(img.width().saturating_sub(x));
    let hh = (zone[3].round() as u32).min(img.height().saturating_sub(y));
    if w == 0 || hh == 0 {
        return None;
    }
    let cut = image::imageops::crop_imm(&img, x, y, w, hh).to_image();
    Some(Pic {
        png: xs_image::encode::png(&cut, 0.0, true).ok()?,
        width: w,
        height: hh,
    })
}

/// The content of an open guide (SMT-028), its pictures drawn.
pub fn content(core: &Core, id: &str, lang: Lang) -> Result<Guide, String> {
    let snap = editor(core).store.snapshot(id)?;
    let doc = &snap.document;
    let t = Template::of(doc)
        .filter(|t| t.is_guide())
        .ok_or_else(|| tr(lang, "errors.not_a_guide", &[]))?;
    let mut items = Vec::new();
    let mut n = 0;
    let mut date = String::new();
    for (s, top) in templates::bands(&t) {
        let zone = doc
            .objects
            .iter()
            .find(|o| o.kind == "zone" && o.props.get("section").and_then(Value::as_str) == Some(&s.id));
        let filled = zone.is_some_and(|z| z.props.get("image").is_some_and(Value::is_string));
        let pic = || {
            zone.filter(|_| filled)
                .and_then(rect)
                .and_then(|r| picture_of(&snap, top, s.height, r))
        };
        match s.kind.as_str() {
            "guideHeader" => date = text_of(doc, &format!("date:{}", s.id)),
            "guideStep" => {
                let text = text_of(doc, &format!("caption:{}:0", s.id));
                let pic = pic();
                if pic.is_some() {
                    n += 1;
                }
                if pic.is_some() || !text.trim().is_empty() {
                    items.push(Item::Step {
                        n: n.max(1),
                        text,
                        pic,
                    });
                }
            }
            "guideText" => {
                let text = text_of(doc, &format!("text:{}", s.id));
                if !text.trim().is_empty() {
                    items.push(Item::Text(text));
                }
            }
            // An empty picture is left out (SMT-029).
            "guideImage" => {
                if let Some(p) = pic() {
                    items.push(Item::Picture(p));
                }
            }
            _ => {}
        }
    }
    Ok(Guide {
        title: text_of(doc, "title"),
        date,
        footer: text_of(doc, "footer"),
        items,
        page: tr(lang, "steps.export.page", &[]),
        of: tr(lang, "steps.export.of", &[]),
        step: tr(lang, "steps.export.step", &[]),
    })
}

fn plain_title(t: &str) -> String {
    guide_out::plain(t).replace(['\n', '\r'], " ")
}

/// A text's width in points, with Arial: the metrics of the PDF's Helvetica.
fn measure(s: &str, bold: bool, size: f64) -> f64 {
    let props = json!({"at": [0, 0], "width": null, "text": s.replace(' ', "\u{a0}"),
        "style": {"font": "Arial", "size": size, "bold": bold}});
    match xs_image::text::Text::from_props(props.as_object().expect("object")) {
        Ok(t) if !s.is_empty() => xs_image::text::layout(&t, 72.0).width as f64,
        _ => 0.0,
    }
}

/// The paper of the system's region: Letter in the Americas that use it, else A4.
fn system_paper() -> &'static str {
    #[cfg(windows)]
    {
        let size = xs_core::winreg::read_string(
            xs_core::winreg::HKCU,
            r"Control Panel\International",
            "iPaperSize",
        );
        if size.as_deref() == Some("1") {
            return "letter";
        }
    }
    "a4"
}

/// The guide as a PDF (SMT-033).
pub fn pdf_bytes(core: &Core, id: &str, lang: Lang, paper: &str) -> Result<Vec<u8>, String> {
    let g = content(core, id, lang)?;
    let paper = if paper == "auto" || paper.is_empty() {
        system_paper()
    } else {
        paper
    };
    guide_out::pdf(&g, guide_out::paper(paper), &measure)
}

/// A free folder `parent/name`, or `name-2`, `name-3`… (SMT-032), unless `replace`.
fn folder_for(parent: &Path, name: &str, replace: bool) -> PathBuf {
    let first = parent.join(name);
    if replace || !first.exists() {
        return first;
    }
    (2..)
        .map(|i| parent.join(format!("{name}-{i}")))
        .find(|p| !p.exists())
        .expect("a free name")
}

fn as_jpg(p: &Pic) -> Result<Vec<u8>, String> {
    let img = xs_image::encode::decode(&p.png)?;
    xs_image::encode::jpg(&img, 0.0, 90, xs_image::color::Rgba::WHITE)
}

/// The guide's pictures as files, `stem-01.png`… (or `.jpg`), with their names.
fn pictures(g: &Guide, folder: &Path, stem: &str, jpg: bool) -> Result<Vec<String>, String> {
    let ext = if jpg { "jpg" } else { "png" };
    let names: Vec<String> = guide_out::picture_names(g, stem)
        .into_iter()
        .map(|n| n.replace(".png", &format!(".{ext}")))
        .collect();
    let pics = g.items.iter().filter_map(|i| match i {
        Item::Step { pic, .. } => pic.as_ref(),
        Item::Picture(p) => Some(p),
        Item::Text(_) => None,
    });
    for (p, name) in pics.zip(&names) {
        let bytes = if jpg { as_jpg(p)? } else { p.png.clone() };
        let path = folder.join(name);
        if let Some(dir) = path.parent() {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
        }
        write_file(&path, &bytes).map_err(|e| e.to_string())?;
    }
    Ok(names)
}

/// The guide on the clipboard with its format (text and pictures) and as plain text
/// for apps that take no format (SMT-034).
pub fn copy(core: &Core, id: &str, lang: Lang) -> Result<(), String> {
    let g = content(core, id, lang)?;
    let srcs: Vec<String> = g
        .items
        .iter()
        .filter_map(|i| match i {
            Item::Step { pic, .. } => pic.as_ref(),
            Item::Picture(p) => Some(p),
            Item::Text(_) => None,
        })
        .map(|p| {
            format!(
                "data:image/png;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(&p.png)
            )
        })
        .collect();
    let html = guide_out::html(&g, &srcs, lang.code());
    // The fragment Windows' clipboard wants: the body.
    let body = html
        .split("<body>")
        .nth(1)
        .and_then(|b| b.split("</body>").next())
        .unwrap_or(&html)
        .to_string();
    let _lock = xs_core::clipboard_lock();
    arboard::Clipboard::new()
        .and_then(|mut c| c.set_html(body, Some(guide_out::text(&g))))
        .map_err(|e| e.to_string())
}

/// `smart.guide_export` (SMT-035): an open guide (`documentId`) or a guide file
/// (`path`) in `format`, written to `out`. Returns the paths written.
pub fn export_command(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let bad = |param: &str, reason: &str| {
        invalid(tr(
            lang,
            "errors.invalid_param",
            &[("param", param), ("reason", reason)],
        ))
    };
    // A file not open is opened apart, and closed after.
    let (id, opened) = match p.get("path").and_then(Value::as_str) {
        Some(path) => {
            let path = PathBuf::from(path);
            match editor(core).store.find_by_path(&path) {
                Some(id) => (id, false),
                None => (open(core, lang, &path, false)?, true),
            }
        }
        None => (resolve(core, lang, p)?, false),
    };
    let done = (|| -> CommandResult {
        let format = p
            .get("format")
            .and_then(Value::as_str)
            .ok_or_else(|| bad("format", "docx, pptx, md, html, pdf, images or clipboard"))?;
        let fail = |e: String| internal(lang, "", e);
        if format == "clipboard" {
            copy(core, &id, lang).map_err(fail)?;
            return Ok(json!({"format": format, "paths": []}));
        }
        let out = p
            .get("out")
            .and_then(Value::as_str)
            .map(PathBuf::from)
            .ok_or_else(|| bad("out", "give where to write it"))?;
        // Named after the guide's title (SMT-032, SMT-034), else its file.
        let name = editor(core)
            .store
            .with(&id, |d| {
                let title = plain_title(&text_of(&d.document, "title"));
                if title.trim().is_empty() {
                    d.name()
                } else {
                    title
                }
            })
            .and_then(|n| xs_library::file_name(&n))
            .unwrap_or_else(|| "Guide".into());
        let replace = p.get("onConflict").and_then(Value::as_str) == Some("replace");
        let jpg = p.get("imageFormat").and_then(Value::as_str) == Some("jpg");
        let g = || content(core, &id, lang).map_err(fail);
        let write = |path: &Path, bytes: &[u8]| -> Result<(), CommandError> {
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir).map_err(|e| fail(e.to_string()))?;
            }
            write_file(path, bytes).map_err(|e| fail(e.to_string()))
        };
        let paths: Vec<PathBuf> = match format {
            "docx" => {
                write(&out, &guide_out::docx(&g()?).map_err(fail)?)?;
                vec![out]
            }
            "pptx" => {
                write(&out, &guide_out::pptx(&g()?).map_err(fail)?)?;
                vec![out]
            }
            "pdf" => {
                let paper = p.get("paper").and_then(Value::as_str).unwrap_or("auto");
                write(&out, &pdf_bytes(core, &id, lang, paper).map_err(fail)?)?;
                vec![out]
            }
            "md" => {
                let g = g()?;
                let folder = folder_for(&out, &name, replace);
                let names = pictures(&g, &folder, "images/paso", jpg).map_err(fail)?;
                let file = folder.join(format!("{name}.md"));
                write(&file, guide_out::markdown(&g, &names).as_bytes())?;
                vec![file]
            }
            "html" => {
                let g = g()?;
                if p.get("singleFile").and_then(Value::as_bool) == Some(true) {
                    // The pictures inside the page: it can travel alone (SMT-032).
                    let srcs: Vec<String> = g
                        .items
                        .iter()
                        .filter_map(|i| match i {
                            Item::Step { pic, .. } => pic.as_ref(),
                            Item::Picture(p) => Some(p),
                            Item::Text(_) => None,
                        })
                        .map(|p| {
                            let (mime, bytes) = if jpg {
                                ("image/jpeg", as_jpg(p).unwrap_or_default())
                            } else {
                                ("image/png", p.png.clone())
                            };
                            format!(
                                "data:{mime};base64,{}",
                                base64::engine::general_purpose::STANDARD.encode(bytes)
                            )
                        })
                        .collect();
                    let file = if replace || !out.join(format!("{name}.html")).exists() {
                        out.join(format!("{name}.html"))
                    } else {
                        (2..)
                            .map(|i| out.join(format!("{name}-{i}.html")))
                            .find(|p| !p.exists())
                            .expect("a free name")
                    };
                    write(&file, guide_out::html(&g, &srcs, lang.code()).as_bytes())?;
                    vec![file]
                } else {
                    let folder = folder_for(&out, &name, replace);
                    let names = pictures(&g, &folder, "images/paso", jpg).map_err(fail)?;
                    let file = folder.join("index.html");
                    write(&file, guide_out::html(&g, &names, lang.code()).as_bytes())?;
                    vec![file]
                }
            }
            "images" => {
                let g = g()?;
                let names = pictures(&g, &out, &name.replace(' ', "_"), jpg).map_err(fail)?;
                let mut written: Vec<PathBuf> = names.iter().map(|n| out.join(n)).collect();
                if p.get("withText").and_then(Value::as_bool) == Some(true) {
                    let file = out.join(tr(lang, "steps.export.text_file", &[]));
                    write(&file, guide_out::text(&g).as_bytes())?;
                    written.push(file);
                }
                written
            }
            other => return Err(bad("format", &format!("not {other}"))),
        };
        Ok(json!({"format": format, "paths": paths}))
    })();
    if opened {
        let _ = core.call(
            ctx.source,
            "document.close",
            json!({"documentId": id, "discard": [id]}),
            Some(lang),
        );
    }
    done
}
