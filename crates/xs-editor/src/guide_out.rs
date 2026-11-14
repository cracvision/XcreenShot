//! A guide of steps in other formats (SMT-030, SMT-032 to SMT-034, ADR 0009): Word,
//! PowerPoint, Markdown, HTML, PDF and plain text, written by XcreenShot itself, with
//! no Office installed and no new dependencies (`zip` and `flate2` were in the app).
//! Pure: it takes the guide's content, already with each picture rendered.

use std::io::Write;

/// A picture of the guide: PNG bytes and its size in pixels.
#[derive(Debug, Clone)]
pub struct Pic {
    pub png: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

/// One section of the guide, in order.
#[derive(Debug, Clone)]
pub enum Item {
    /// A numbered step: its text (bold marked `**…**`) and its picture.
    Step {
        n: usize,
        text: String,
        pic: Option<Pic>,
    },
    Text(String),
    Picture(Pic),
}

/// What every format writes.
#[derive(Debug, Clone)]
pub struct Guide {
    pub title: String,
    pub date: String,
    /// "Created by … · date".
    pub footer: String,
    pub items: Vec<Item>,
    /// Labels in the guide's language: "Page", "of", "Step".
    pub page: String,
    pub of: String,
    pub step: String,
}

/// A text in pieces, each bold or not (`**…**`, SMT-025).
pub fn runs(text: &str) -> Vec<(String, bool)> {
    text.split("**")
        .enumerate()
        .filter(|(_, p)| !p.is_empty())
        .map(|(i, p)| (p.to_string(), i % 2 == 1))
        .collect()
}

/// The text without its marks.
pub fn plain(text: &str) -> String {
    text.replace("**", "")
}

fn xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The pictures' file names (`images/paso-01.png`, SMT-032), in the items' order.
pub fn picture_names(g: &Guide, stem: &str) -> Vec<String> {
    let total = g.items.iter().filter(|i| pic_of(i).is_some()).count();
    let digits = total.to_string().len().max(2);
    (1..=total).map(|i| format!("{stem}-{i:0digits$}.png")).collect()
}

fn pic_of(i: &Item) -> Option<&Pic> {
    match i {
        Item::Step { pic, .. } => pic.as_ref(),
        Item::Picture(p) => Some(p),
        Item::Text(_) => None,
    }
}

// ---------------------------------------------------------------------------
// Markdown and HTML (SMT-032), plain text (SMT-034)

/// Markdown (CommonMark): the title, each step as an item of a numbered list with its
/// picture below, the texts as paragraphs and the footer in italics. `names` are the
/// pictures' paths.
pub fn markdown(g: &Guide, names: &[String]) -> String {
    let md = |t: &str| {
        runs(t)
            .into_iter()
            .map(|(s, b)| if b { format!("**{}**", s.trim()) } else { s })
            .collect::<String>()
    };
    let mut out = format!("# {}\n\n{}\n\n", g.title.replace('\n', " "), g.date);
    let mut names = names.iter();
    for item in &g.items {
        match item {
            Item::Step { n, text, pic } => {
                out.push_str(&format!("{n}. {}\n", md(text).replace('\n', "\n   ")));
                if pic.is_some() {
                    if let Some(name) = names.next() {
                        out.push_str(&format!(
                            "\n   ![{}]({name})\n",
                            plain(text).replace(['[', ']'], "")
                        ));
                    }
                }
                out.push('\n');
            }
            Item::Text(text) => out.push_str(&format!("{}\n\n", md(text))),
            Item::Picture(_) => {
                if let Some(name) = names.next() {
                    out.push_str(&format!("![]({name})\n\n"));
                }
            }
        }
    }
    out.push_str(&format!("*{}*\n", g.footer));
    out
}

/// A page that reads well offline and on a phone: its own styles, a numbered list of
/// steps, pictures with their alternative text. `srcs` are the pictures' addresses
/// (paths, or `data:` URIs for a single file).
pub fn html(g: &Guide, srcs: &[String], lang: &str) -> String {
    let rich = |t: &str| {
        runs(t)
            .into_iter()
            .map(|(s, b)| {
                if b {
                    format!("<strong>{}</strong>", xml(&s))
                } else {
                    xml(&s)
                }
            })
            .collect::<String>()
            .replace('\n', "<br>")
    };
    let mut body = String::new();
    let mut srcs = srcs.iter();
    let mut open = false;
    for item in &g.items {
        match item {
            Item::Step { n, text, pic } => {
                if !open {
                    body.push_str("<ol>\n");
                    open = true;
                }
                body.push_str(&format!("<li value=\"{n}\"><p>{}</p>", rich(text)));
                if pic.is_some() {
                    if let Some(src) = srcs.next() {
                        body.push_str(&format!(
                            "<img src=\"{}\" alt=\"{}\">",
                            xml(src),
                            xml(&plain(text))
                        ));
                    }
                }
                body.push_str("</li>\n");
            }
            Item::Text(text) => {
                if open {
                    body.push_str("</ol>\n");
                    open = false;
                }
                body.push_str(&format!("<p>{}</p>\n", rich(text)));
            }
            Item::Picture(_) => {
                if open {
                    body.push_str("</ol>\n");
                    open = false;
                }
                if let Some(src) = srcs.next() {
                    body.push_str(&format!("<p><img src=\"{}\" alt=\"\"></p>\n", xml(src)));
                }
            }
        }
    }
    if open {
        body.push_str("</ol>\n");
    }
    format!(
        "<!doctype html>\n<html lang=\"{lang}\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>{title}</title>\n<style>\nbody {{ max-width: 960px; margin: 0 auto; padding: 24px 16px; font: 16px/1.5 system-ui, sans-serif; color: #1f2430; background: #fff; }}\nh1 {{ font-size: 28px; margin: 0 0 4px; }}\n.date {{ color: #5b6270; margin: 0 0 24px; }}\nol {{ padding-left: 1.6em; }}\nli {{ margin: 0 0 28px; }}\nimg {{ display: block; max-width: 100%; height: auto; margin-top: 8px; border: 1px solid #d8dbe2; border-radius: 6px; }}\nfooter {{ margin-top: 32px; color: #5b6270; font-style: italic; }}\n@media (prefers-color-scheme: dark) {{ body {{ color: #e6e8ee; background: #16181d; }} .date, footer {{ color: #a3a9b6; }} img {{ border-color: #3a3f4b; }} }}\n</style>\n</head>\n<body>\n<h1>{title}</h1>\n<p class=\"date\">{date}</p>\n{body}<footer>{footer}</footer>\n</body>\n</html>\n",
        title = xml(&g.title),
        date = xml(&g.date),
        footer = xml(&g.footer),
    )
}

/// `pasos.txt` (SMT-034): one numbered line per step.
pub fn text(g: &Guide) -> String {
    let mut out = format!("{}\n\n", g.title);
    for item in &g.items {
        match item {
            Item::Step { n, text, .. } => out.push_str(&format!("{n}. {}\n", plain(text).replace('\n', " "))),
            Item::Text(t) => out.push_str(&format!("\n{}\n\n", plain(t))),
            Item::Picture(_) => {}
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Word (SMT-030)

const EMU_PER_PX: u64 = 9525;

fn zip_file(files: &[(&str, Vec<u8>)]) -> Result<Vec<u8>, String> {
    let mut out = std::io::Cursor::new(Vec::new());
    {
        let mut z = zip::ZipWriter::new(&mut out);
        let opts =
            zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        for (name, bytes) in files {
            z.start_file(*name, opts).map_err(|e| e.to_string())?;
            z.write_all(bytes).map_err(|e| e.to_string())?;
        }
        z.finish().map_err(|e| e.to_string())?;
    }
    Ok(out.into_inner())
}

fn w_runs(text: &str) -> String {
    runs(text)
        .into_iter()
        .map(|(s, b)| {
            let props = if b { "<w:rPr><w:b/></w:rPr>" } else { "" };
            format!("<w:r>{props}<w:t xml:space=\"preserve\">{}</w:t></w:r>", xml(&s))
        })
        .collect()
}

/// A Word document: the title as Heading 1, each step an item of a numbered list
/// (List Number) followed by its picture at the page's width with the step's text as
/// its alternative text, the texts as Normal paragraphs and the footer in the page's
/// footer. Word's standard styles, so an organization's template restyles it.
pub fn docx(g: &Guide) -> Result<Vec<u8>, String> {
    // Letter with 1" margins: 6.5" of width.
    let max_cx: u64 = 5_943_600;
    let mut body = String::new();
    body.push_str(&format!(
        "<w:p><w:pPr><w:pStyle w:val=\"Heading1\"/></w:pPr>{}</w:p>",
        w_runs(&g.title)
    ));
    body.push_str(&format!("<w:p>{}</w:p>", w_runs(&g.date)));
    let mut media: Vec<(String, Vec<u8>)> = Vec::new();
    let mut rels = String::new();
    let picture = |pic: &Pic, alt: &str, media: &mut Vec<(String, Vec<u8>)>, rels: &mut String| -> String {
        let n = media.len() + 1;
        let rid = format!("rIdImg{n}");
        media.push((format!("word/media/image{n}.png"), pic.png.clone()));
        rels.push_str(&format!(
            "<Relationship Id=\"{rid}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image\" Target=\"media/image{n}.png\"/>"
        ));
        let cx0 = pic.width as u64 * EMU_PER_PX;
        let cy0 = pic.height as u64 * EMU_PER_PX;
        let (cx, cy) = if cx0 > max_cx {
            (max_cx, cy0 * max_cx / cx0.max(1))
        } else {
            (cx0, cy0)
        };
        format!(
            "<w:p><w:r><w:drawing><wp:inline distT=\"0\" distB=\"0\" distL=\"0\" distR=\"0\"><wp:extent cx=\"{cx}\" cy=\"{cy}\"/><wp:docPr id=\"{n}\" name=\"Picture {n}\" descr=\"{alt}\"/><a:graphic xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\"><a:graphicData uri=\"http://schemas.openxmlformats.org/drawingml/2006/picture\"><pic:pic xmlns:pic=\"http://schemas.openxmlformats.org/drawingml/2006/picture\"><pic:nvPicPr><pic:cNvPr id=\"{n}\" name=\"image{n}.png\" descr=\"{alt}\"/><pic:cNvPicPr/></pic:nvPicPr><pic:blipFill><a:blip r:embed=\"{rid}\"/><a:stretch><a:fillRect/></a:stretch></pic:blipFill><pic:spPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"{cx}\" cy=\"{cy}\"/></a:xfrm><a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></pic:spPr></pic:pic></a:graphicData></a:graphic></wp:inline></w:drawing></w:r></w:p>",
            alt = xml(alt)
        )
    };
    for item in &g.items {
        match item {
            Item::Step { text, pic, .. } => {
                body.push_str(&format!(
                    "<w:p><w:pPr><w:pStyle w:val=\"ListNumber\"/><w:numPr><w:ilvl w:val=\"0\"/><w:numId w:val=\"1\"/></w:numPr></w:pPr>{}</w:p>",
                    w_runs(text)
                ));
                if let Some(p) = pic {
                    body.push_str(&picture(p, &plain(text), &mut media, &mut rels));
                }
            }
            Item::Text(t) => body.push_str(&format!("<w:p>{}</w:p>", w_runs(t))),
            Item::Picture(p) => body.push_str(&picture(p, "", &mut media, &mut rels)),
        }
    }
    let document = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<w:document xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" xmlns:wp=\"http://schemas.openxmlformats.org/drawingml/2006/wordprocessingDrawing\"><w:body>{body}<w:sectPr><w:footerReference w:type=\"default\" r:id=\"rIdFooter\"/><w:pgSz w:w=\"12240\" w:h=\"15840\"/><w:pgMar w:top=\"1440\" w:right=\"1440\" w:bottom=\"1440\" w:left=\"1440\" w:header=\"720\" w:footer=\"720\" w:gutter=\"0\"/></w:sectPr></w:body></w:document>"
    );
    let styles = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<w:styles xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:docDefaults><w:rPrDefault><w:rPr><w:rFonts w:ascii=\"Calibri\" w:hAnsi=\"Calibri\" w:eastAsia=\"Calibri\" w:cs=\"Calibri\"/><w:sz w:val=\"22\"/><w:szCs w:val=\"22\"/><w:lang w:val=\"es-ES\"/></w:rPr></w:rPrDefault><w:pPrDefault><w:pPr><w:spacing w:after=\"160\" w:line=\"259\" w:lineRule=\"auto\"/></w:pPr></w:pPrDefault></w:docDefaults><w:style w:type=\"paragraph\" w:default=\"1\" w:styleId=\"Normal\"><w:name w:val=\"Normal\"/><w:qFormat/></w:style><w:style w:type=\"paragraph\" w:styleId=\"Heading1\"><w:name w:val=\"heading 1\"/><w:basedOn w:val=\"Normal\"/><w:next w:val=\"Normal\"/><w:qFormat/><w:pPr><w:keepNext/><w:spacing w:before=\"240\" w:after=\"120\"/><w:outlineLvl w:val=\"0\"/></w:pPr><w:rPr><w:b/><w:sz w:val=\"36\"/><w:szCs w:val=\"36\"/></w:rPr></w:style><w:style w:type=\"paragraph\" w:styleId=\"ListNumber\"><w:name w:val=\"List Number\"/><w:basedOn w:val=\"Normal\"/><w:qFormat/><w:pPr><w:keepNext/><w:numPr><w:numId w:val=\"1\"/></w:numPr></w:pPr></w:style><w:style w:type=\"paragraph\" w:styleId=\"Footer\"><w:name w:val=\"footer\"/><w:basedOn w:val=\"Normal\"/><w:rPr><w:i/><w:sz w:val=\"18\"/></w:rPr></w:style></w:styles>";
    let numbering = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<w:numbering xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:abstractNum w:abstractNumId=\"0\"><w:multiLevelType w:val=\"singleLevel\"/><w:lvl w:ilvl=\"0\"><w:start w:val=\"1\"/><w:numFmt w:val=\"decimal\"/><w:lvlText w:val=\"%1.\"/><w:lvlJc w:val=\"left\"/><w:pPr><w:ind w:left=\"360\" w:hanging=\"360\"/></w:pPr></w:lvl></w:abstractNum><w:num w:numId=\"1\"><w:abstractNumId w:val=\"0\"/></w:num></w:numbering>";
    let footer = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<w:ftr xmlns:w=\"http://schemas.openxmlformats.org/wordprocessingml/2006/main\"><w:p><w:pPr><w:pStyle w:val=\"Footer\"/></w:pPr>{}</w:p></w:ftr>",
        w_runs(&g.footer)
    );
    let doc_rels = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rIdStyles\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles\" Target=\"styles.xml\"/><Relationship Id=\"rIdNumbering\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/numbering\" Target=\"numbering.xml\"/><Relationship Id=\"rIdFooter\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/footer\" Target=\"footer1.xml\"/>{rels}</Relationships>"
    );
    let types = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" ContentType=\"application/xml\"/><Default Extension=\"png\" ContentType=\"image/png\"/><Override PartName=\"/word/document.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml\"/><Override PartName=\"/word/styles.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.styles+xml\"/><Override PartName=\"/word/numbering.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.numbering+xml\"/><Override PartName=\"/word/footer1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.wordprocessingml.footer+xml\"/><Override PartName=\"/docProps/core.xml\" ContentType=\"application/vnd.openxmlformats-package.core-properties+xml\"/></Types>";
    let root_rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"word/document.xml\"/><Relationship Id=\"rId2\" Type=\"http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties\" Target=\"docProps/core.xml\"/></Relationships>";
    let core = core_props(&g.title);
    let mut files: Vec<(&str, Vec<u8>)> = vec![
        ("[Content_Types].xml", types.as_bytes().to_vec()),
        ("_rels/.rels", root_rels.as_bytes().to_vec()),
        ("docProps/core.xml", core.into_bytes()),
        ("word/document.xml", document.into_bytes()),
        ("word/styles.xml", styles.as_bytes().to_vec()),
        ("word/numbering.xml", numbering.as_bytes().to_vec()),
        ("word/footer1.xml", footer.into_bytes()),
        ("word/_rels/document.xml.rels", doc_rels.into_bytes()),
    ];
    for (name, bytes) in &media {
        files.push((name.as_str(), bytes.clone()));
    }
    zip_file(&files)
}

fn core_props(title: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<cp:coreProperties xmlns:cp=\"http://schemas.openxmlformats.org/package/2006/metadata/core-properties\" xmlns:dc=\"http://purl.org/dc/elements/1.1/\" xmlns:dcterms=\"http://purl.org/dc/terms/\" xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\"><dc:title>{}</dc:title><dc:creator>XcreenShot</dc:creator></cp:coreProperties>",
        xml(title)
    )
}

// ---------------------------------------------------------------------------
// PowerPoint (SMT-030)

const SLIDE_W: u64 = 12_192_000;
const SLIDE_H: u64 = 6_858_000;

fn a_runs(text: &str, size: u32) -> String {
    runs(text)
        .into_iter()
        .map(|(s, b)| {
            format!(
                "<a:r><a:rPr lang=\"es-ES\" sz=\"{size}\" b=\"{}\" dirty=\"0\"/><a:t>{}</a:t></a:r>",
                u8::from(b),
                xml(&s)
            )
        })
        .collect()
}

fn text_box(id: u32, name: &str, x: u64, y: u64, cx: u64, cy: u64, paragraphs: &[String]) -> String {
    let ps: String = paragraphs.iter().map(|p| format!("<a:p>{p}</a:p>")).collect();
    format!(
        "<p:sp><p:nvSpPr><p:cNvPr id=\"{id}\" name=\"{name}\"/><p:cNvSpPr txBox=\"1\"/><p:nvPr/></p:nvSpPr><p:spPr><a:xfrm><a:off x=\"{x}\" y=\"{y}\"/><a:ext cx=\"{cx}\" cy=\"{cy}\"/></a:xfrm><a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom><a:noFill/></p:spPr><p:txBody><a:bodyPr wrap=\"square\" rtlCol=\"0\"><a:normAutofit/></a:bodyPr><a:lstStyle/>{ps}</p:txBody></p:sp>"
    )
}

fn slide(shapes: &str) -> String {
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<p:sld xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\"><p:cSld><p:spTree><p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"\"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"0\" cy=\"0\"/><a:chOff x=\"0\" y=\"0\"/><a:chExt cx=\"0\" cy=\"0\"/></a:xfrm></p:grpSpPr>{shapes}</p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sld>"
    )
}

const THEME: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<a:theme xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" name=\"XcreenShot\"><a:themeElements><a:clrScheme name=\"XcreenShot\"><a:dk1><a:srgbClr val=\"1F2430\"/></a:dk1><a:lt1><a:srgbClr val=\"FFFFFF\"/></a:lt1><a:dk2><a:srgbClr val=\"2B2F38\"/></a:dk2><a:lt2><a:srgbClr val=\"EEF0F4\"/></a:lt2><a:accent1><a:srgbClr val=\"2D7FF9\"/></a:accent1><a:accent2><a:srgbClr val=\"F28C28\"/></a:accent2><a:accent3><a:srgbClr val=\"8E5BD9\"/></a:accent3><a:accent4><a:srgbClr val=\"1BA784\"/></a:accent4><a:accent5><a:srgbClr val=\"E5484D\"/></a:accent5><a:accent6><a:srgbClr val=\"5B6270\"/></a:accent6><a:hlink><a:srgbClr val=\"2D7FF9\"/></a:hlink><a:folHlink><a:srgbClr val=\"8E5BD9\"/></a:folHlink></a:clrScheme><a:fontScheme name=\"XcreenShot\"><a:majorFont><a:latin typeface=\"Calibri\"/><a:ea typeface=\"\"/><a:cs typeface=\"\"/></a:majorFont><a:minorFont><a:latin typeface=\"Calibri\"/><a:ea typeface=\"\"/><a:cs typeface=\"\"/></a:minorFont></a:fontScheme><a:fmtScheme name=\"XcreenShot\"><a:fillStyleLst><a:solidFill><a:schemeClr val=\"phClr\"/></a:solidFill><a:solidFill><a:schemeClr val=\"phClr\"/></a:solidFill><a:solidFill><a:schemeClr val=\"phClr\"/></a:solidFill></a:fillStyleLst><a:lnStyleLst><a:ln w=\"6350\"><a:solidFill><a:schemeClr val=\"phClr\"/></a:solidFill></a:ln><a:ln w=\"12700\"><a:solidFill><a:schemeClr val=\"phClr\"/></a:solidFill></a:ln><a:ln w=\"19050\"><a:solidFill><a:schemeClr val=\"phClr\"/></a:solidFill></a:ln></a:lnStyleLst><a:effectStyleLst><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle><a:effectStyle><a:effectLst/></a:effectStyle></a:effectStyleLst><a:bgFillStyleLst><a:solidFill><a:schemeClr val=\"phClr\"/></a:solidFill><a:solidFill><a:schemeClr val=\"phClr\"/></a:solidFill><a:solidFill><a:schemeClr val=\"phClr\"/></a:solidFill></a:bgFillStyleLst></a:fmtScheme></a:themeElements><a:objectDefaults/><a:extraClrSchemeLst/></a:theme>";

/// A PowerPoint presentation, 16:9: a title slide with the title, the date and the
/// author; one slide per step with its number and text as an editable title and its
/// picture fitted below without stretching it (its alternative text, the step's text);
/// the texts as slides of their own.
pub fn pptx(g: &Guide) -> Result<Vec<u8>, String> {
    let margin: u64 = 457_200;
    let mut slides: Vec<(String, Option<Vec<u8>>)> = Vec::new();
    // The title slide.
    slides.push((
        slide(&text_box(
            2,
            "Title",
            margin,
            2_286_000,
            SLIDE_W - 2 * margin,
            2_286_000,
            &[
                a_runs(&format!("**{}**", g.title.replace("**", "")), 4000),
                a_runs(&g.date, 1800),
                a_runs(&g.footer, 1600),
            ],
        )),
        None,
    ));
    for item in &g.items {
        match item {
            Item::Step { n, text, pic } => {
                let title_h: u64 = 1_143_000;
                let mut shapes = text_box(
                    2,
                    "Title",
                    margin,
                    margin / 2,
                    SLIDE_W - 2 * margin,
                    title_h,
                    &[a_runs(&format!("{n}. {text}"), 2400)],
                );
                if let Some(p) = pic {
                    shapes.push_str(&picture_shape(
                        p,
                        &plain(text),
                        margin,
                        margin / 2 + title_h,
                        margin,
                    ));
                }
                slides.push((slide(&shapes), pic.as_ref().map(|p| p.png.clone())));
            }
            Item::Text(t) => slides.push((
                slide(&text_box(
                    2,
                    "Text",
                    margin,
                    margin,
                    SLIDE_W - 2 * margin,
                    SLIDE_H - 2 * margin,
                    &t.split('\n').map(|l| a_runs(l, 2400)).collect::<Vec<_>>(),
                )),
                None,
            )),
            Item::Picture(p) => {
                slides.push((
                    slide(&picture_shape(p, "", margin, margin, margin)),
                    Some(p.png.clone()),
                ));
            }
        }
    }
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    let mut types = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Types xmlns=\"http://schemas.openxmlformats.org/package/2006/content-types\"><Default Extension=\"rels\" ContentType=\"application/vnd.openxmlformats-package.relationships+xml\"/><Default Extension=\"xml\" ContentType=\"application/xml\"/><Default Extension=\"png\" ContentType=\"image/png\"/><Override PartName=\"/ppt/presentation.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.presentationml.presentation.main+xml\"/><Override PartName=\"/ppt/slideMasters/slideMaster1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.presentationml.slideMaster+xml\"/><Override PartName=\"/ppt/slideLayouts/slideLayout1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.presentationml.slideLayout+xml\"/><Override PartName=\"/ppt/theme/theme1.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.theme+xml\"/><Override PartName=\"/docProps/core.xml\" ContentType=\"application/vnd.openxmlformats-package.core-properties+xml\"/>");
    let mut pres_rels = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rIdMaster\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster\" Target=\"slideMasters/slideMaster1.xml\"/><Relationship Id=\"rIdTheme\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme\" Target=\"theme/theme1.xml\"/>");
    let mut sld_ids = String::new();
    for (i, (xml_slide, png)) in slides.into_iter().enumerate() {
        let k = i + 1;
        types.push_str(&format!(
            "<Override PartName=\"/ppt/slides/slide{k}.xml\" ContentType=\"application/vnd.openxmlformats-officedocument.presentationml.slide+xml\"/>"
        ));
        pres_rels.push_str(&format!(
            "<Relationship Id=\"rIdS{k}\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/slide\" Target=\"slides/slide{k}.xml\"/>"
        ));
        sld_ids.push_str(&format!("<p:sldId id=\"{}\" r:id=\"rIdS{k}\"/>", 255 + k));
        let mut rels = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rIdLayout\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout\" Target=\"../slideLayouts/slideLayout1.xml\"/>");
        if let Some(png) = png {
            rels.push_str(&format!(
                "<Relationship Id=\"rIdPic\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/image\" Target=\"../media/image{k}.png\"/>"
            ));
            files.push((format!("ppt/media/image{k}.png"), png));
        }
        rels.push_str("</Relationships>");
        files.push((format!("ppt/slides/slide{k}.xml"), xml_slide.into_bytes()));
        files.push((format!("ppt/slides/_rels/slide{k}.xml.rels"), rels.into_bytes()));
    }
    types.push_str("</Types>");
    pres_rels.push_str("</Relationships>");
    let presentation = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<p:presentation xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\"><p:sldMasterIdLst><p:sldMasterId id=\"2147483648\" r:id=\"rIdMaster\"/></p:sldMasterIdLst><p:sldIdLst>{sld_ids}</p:sldIdLst><p:sldSz cx=\"{SLIDE_W}\" cy=\"{SLIDE_H}\"/><p:notesSz cx=\"6858000\" cy=\"9144000\"/></p:presentation>"
    );
    let master = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<p:sldMaster xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\"><p:cSld><p:bg><p:bgRef idx=\"1001\"><a:schemeClr val=\"bg1\"/></p:bgRef></p:bg><p:spTree><p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"\"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"0\" cy=\"0\"/><a:chOff x=\"0\" y=\"0\"/><a:chExt cx=\"0\" cy=\"0\"/></a:xfrm></p:grpSpPr></p:spTree></p:cSld><p:clrMap bg1=\"lt1\" tx1=\"dk1\" bg2=\"lt2\" tx2=\"dk2\" accent1=\"accent1\" accent2=\"accent2\" accent3=\"accent3\" accent4=\"accent4\" accent5=\"accent5\" accent6=\"accent6\" hlink=\"hlink\" folHlink=\"folHlink\"/><p:sldLayoutIdLst><p:sldLayoutId id=\"2147483649\" r:id=\"rIdLayout\"/></p:sldLayoutIdLst><p:txStyles><p:titleStyle><a:lvl1pPr><a:defRPr sz=\"3200\"/></a:lvl1pPr></p:titleStyle><p:bodyStyle><a:lvl1pPr><a:defRPr sz=\"2000\"/></a:lvl1pPr></p:bodyStyle><p:otherStyle><a:lvl1pPr><a:defRPr sz=\"1800\"/></a:lvl1pPr></p:otherStyle></p:txStyles></p:sldMaster>";
    let master_rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rIdLayout\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideLayout\" Target=\"../slideLayouts/slideLayout1.xml\"/><Relationship Id=\"rIdTheme\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/theme\" Target=\"../theme/theme1.xml\"/></Relationships>";
    let layout = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<p:sldLayout xmlns:a=\"http://schemas.openxmlformats.org/drawingml/2006/main\" xmlns:r=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships\" xmlns:p=\"http://schemas.openxmlformats.org/presentationml/2006/main\" type=\"blank\" preserve=\"1\"><p:cSld name=\"Blank\"><p:spTree><p:nvGrpSpPr><p:cNvPr id=\"1\" name=\"\"/><p:cNvGrpSpPr/><p:nvPr/></p:nvGrpSpPr><p:grpSpPr><a:xfrm><a:off x=\"0\" y=\"0\"/><a:ext cx=\"0\" cy=\"0\"/><a:chOff x=\"0\" y=\"0\"/><a:chExt cx=\"0\" cy=\"0\"/></a:xfrm></p:grpSpPr></p:spTree></p:cSld><p:clrMapOvr><a:masterClrMapping/></p:clrMapOvr></p:sldLayout>";
    let layout_rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rIdMaster\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/slideMaster\" Target=\"../slideMasters/slideMaster1.xml\"/></Relationships>";
    let root_rels = "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\"?>\n<Relationships xmlns=\"http://schemas.openxmlformats.org/package/2006/relationships\"><Relationship Id=\"rId1\" Type=\"http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument\" Target=\"ppt/presentation.xml\"/><Relationship Id=\"rId2\" Type=\"http://schemas.openxmlformats.org/package/2006/relationships/metadata/core-properties\" Target=\"docProps/core.xml\"/></Relationships>";
    let mut all: Vec<(&str, Vec<u8>)> = vec![
        ("[Content_Types].xml", types.into_bytes()),
        ("_rels/.rels", root_rels.as_bytes().to_vec()),
        ("docProps/core.xml", core_props(&g.title).into_bytes()),
        ("ppt/presentation.xml", presentation.into_bytes()),
        ("ppt/_rels/presentation.xml.rels", pres_rels.into_bytes()),
        ("ppt/slideMasters/slideMaster1.xml", master.as_bytes().to_vec()),
        (
            "ppt/slideMasters/_rels/slideMaster1.xml.rels",
            master_rels.as_bytes().to_vec(),
        ),
        ("ppt/slideLayouts/slideLayout1.xml", layout.as_bytes().to_vec()),
        (
            "ppt/slideLayouts/_rels/slideLayout1.xml.rels",
            layout_rels.as_bytes().to_vec(),
        ),
        ("ppt/theme/theme1.xml", THEME.as_bytes().to_vec()),
    ];
    for (name, bytes) in &files {
        all.push((name.as_str(), bytes.clone()));
    }
    zip_file(&all)
}

/// A picture fitted in the slide below `top`, centered, never stretched.
fn picture_shape(p: &Pic, alt: &str, x0: u64, top: u64, margin: u64) -> String {
    let (aw, ah) = (SLIDE_W - 2 * x0, SLIDE_H.saturating_sub(top + margin).max(1));
    let (pw, ph) = (p.width.max(1) as f64, p.height.max(1) as f64);
    let k = (aw as f64 / pw).min(ah as f64 / ph);
    let (cx, cy) = ((pw * k) as u64, (ph * k) as u64);
    let x = x0 + (aw - cx) / 2;
    format!(
        "<p:pic><p:nvPicPr><p:cNvPr id=\"3\" name=\"Picture\" descr=\"{}\"/><p:cNvPicPr><a:picLocks noChangeAspect=\"1\"/></p:cNvPicPr><p:nvPr/></p:nvPicPr><p:blipFill><a:blip r:embed=\"rIdPic\"/><a:stretch><a:fillRect/></a:stretch></p:blipFill><p:spPr><a:xfrm><a:off x=\"{x}\" y=\"{top}\"/><a:ext cx=\"{cx}\" cy=\"{cy}\"/></a:xfrm><a:prstGeom prst=\"rect\"><a:avLst/></a:prstGeom></p:spPr></p:pic>",
        xml(alt)
    )
}

// ---------------------------------------------------------------------------
// PDF (SMT-033)

/// A paper: width and height in points.
pub fn paper(name: &str) -> (f64, f64) {
    match name {
        "letter" => (612.0, 792.0),
        _ => (595.28, 841.89),
    }
}

/// A character in WinAnsi (the PDF standard fonts' encoding), or `?`.
fn win_ansi(c: char) -> u8 {
    match c {
        '\u{20}'..='\u{7E}' | '\u{A0}'..='\u{FF}' => c as u32 as u8,
        '€' => 0x80,
        '‚' => 0x82,
        '„' => 0x84,
        '…' => 0x85,
        '‘' => 0x91,
        '’' => 0x92,
        '“' => 0x93,
        '”' => 0x94,
        '•' => 0x95,
        '–' => 0x96,
        '—' => 0x97,
        '™' => 0x99,
        _ => b'?',
    }
}

fn pdf_string(s: &str) -> Vec<u8> {
    let mut out = vec![b'('];
    for c in s.chars() {
        let b = win_ansi(c);
        if matches!(b, b'(' | b')' | b'\\') {
            out.push(b'\\');
        }
        out.push(b);
    }
    out.push(b')');
    out
}

/// A text's width in points at `size`: the font's widths, measured once.
pub type Measure<'a> = &'a dyn Fn(&str, bool, f64) -> f64;

/// Lines of `(text, bold)` pieces that fit in `width` points.
fn wrap(text: &str, size: f64, width: f64, measure: Measure) -> Vec<Vec<(String, bool)>> {
    let mut lines: Vec<Vec<(String, bool)>> = Vec::new();
    for paragraph in text.split('\n') {
        let mut line: Vec<(String, bool)> = Vec::new();
        let mut used = 0.0;
        for (piece, bold) in runs(paragraph) {
            for (i, word) in piece.split(' ').enumerate() {
                let token = if i == 0 {
                    word.to_string()
                } else {
                    format!(" {word}")
                };
                let w = measure(&token, bold, size);
                if used + w > width && !line.is_empty() {
                    lines.push(std::mem::take(&mut line));
                    used = 0.0;
                    let t = token.trim_start().to_string();
                    used += measure(&t, bold, size);
                    line.push((t, bold));
                } else {
                    used += w;
                    match line.last_mut() {
                        Some((s, b)) if *b == bold => s.push_str(&token),
                        _ => line.push((token, bold)),
                    }
                }
            }
        }
        lines.push(line);
    }
    lines
}

/// A block of the PDF: lines of text, then maybe a picture, kept on one page.
struct Block {
    lines: Vec<Vec<(String, bool)>>,
    size: f64,
    pic: Option<(usize, f64, f64)>,
    /// A step's bookmark.
    mark: Option<String>,
}

/// A PDF of the guide: text that can be selected and searched (Helvetica), a bookmark
/// per step, pictures that fit the page, a step never split between two pages, the
/// header on the first page only and "title — Page N of M" at the foot of each page.
pub fn pdf(g: &Guide, page: (f64, f64), measure: Measure) -> Result<Vec<u8>, String> {
    let margin = 15.0 * 72.0 / 25.4;
    let (pw, ph) = page;
    let (aw, ah) = (pw - 2.0 * margin, ph - 2.0 * margin - 20.0);
    let line_h = |size: f64| size * 1.3;
    let mut pics: Vec<&Pic> = Vec::new();
    let mut blocks: Vec<Block> = vec![Block {
        lines: wrap(&format!("**{}**", g.title.replace("**", "")), 22.0, aw, measure),
        size: 22.0,
        pic: None,
        mark: None,
    }];
    blocks.push(Block {
        lines: wrap(&g.date, 10.0, aw, measure),
        size: 10.0,
        pic: None,
        mark: None,
    });
    let fit = |p: &Pic, room: f64| {
        let (w, h) = (p.width as f64 * 0.75, p.height as f64 * 0.75);
        let k = (aw / w).min(room / h).min(1.0);
        (w * k, h * k)
    };
    for item in &g.items {
        match item {
            Item::Step { n, text, pic } => {
                let lines = wrap(&format!("{n}. {text}"), 12.0, aw, measure);
                let text_h = lines.len() as f64 * line_h(12.0);
                let p = pic.as_ref().map(|p| {
                    pics.push(p);
                    let (w, h) = fit(p, (ah - text_h - 8.0).max(60.0));
                    (pics.len() - 1, w, h)
                });
                blocks.push(Block {
                    lines,
                    size: 12.0,
                    pic: p,
                    mark: Some(format!("{n}. {}", plain(text))),
                });
            }
            Item::Text(t) => blocks.push(Block {
                lines: wrap(t, 12.0, aw, measure),
                size: 12.0,
                pic: None,
                mark: None,
            }),
            Item::Picture(p) => {
                pics.push(p);
                let (w, h) = fit(p, ah);
                blocks.push(Block {
                    lines: Vec::new(),
                    size: 12.0,
                    pic: Some((pics.len() - 1, w, h)),
                    mark: None,
                });
            }
        }
    }
    // Pages: each block whole on one page.
    let mut pages: Vec<Vec<(usize, f64)>> = vec![Vec::new()];
    let mut y = 0.0;
    for (i, b) in blocks.iter().enumerate() {
        let h = b.lines.len() as f64 * line_h(b.size) + b.pic.map_or(0.0, |(_, _, h)| h + 6.0) + 14.0;
        if y + h > ah && !pages.last().is_none_or(Vec::is_empty) {
            pages.push(Vec::new());
            y = 0.0;
        }
        pages.last_mut().expect("page").push((i, y));
        y += h;
    }
    let total = pages.len();
    // Objects: 1 catalog, 2 pages, 3 F1, 4 F2, 5 outlines, then the pictures, then
    // per page its page and contents, then the bookmarks.
    let mut objects: Vec<Vec<u8>> = Vec::new();
    let first_pic = 6;
    let first_page = first_pic + pics.len();
    let first_mark = first_page + 2 * total;
    let marks: Vec<(usize, usize, f64)> = pages
        .iter()
        .enumerate()
        .flat_map(|(pi, list)| {
            list.iter()
                .filter(|(bi, _)| blocks[*bi].mark.is_some())
                .map(move |(bi, y)| (*bi, pi, *y))
        })
        .collect();
    let kids: Vec<String> = (0..total)
        .map(|i| format!("{} 0 R", first_page + 2 * i))
        .collect();
    objects.push(b"<< /Type /Catalog /Pages 2 0 R /Outlines 5 0 R /PageMode /UseOutlines >>".to_vec());
    objects.push(format!("<< /Type /Pages /Kids [{}] /Count {total} >>", kids.join(" ")).into_bytes());
    objects
        .push(b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>".to_vec());
    objects.push(
        b"<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>".to_vec(),
    );
    objects.push(if marks.is_empty() {
        b"<< /Type /Outlines /Count 0 >>".to_vec()
    } else {
        format!(
            "<< /Type /Outlines /First {} 0 R /Last {} 0 R /Count {} >>",
            first_mark,
            first_mark + marks.len() - 1,
            marks.len()
        )
        .into_bytes()
    });
    for p in &pics {
        let img = xs_image::encode::decode(&p.png)?;
        let rgb: Vec<u8> = img
            .pixels()
            .flat_map(|px| {
                let a = px[3] as u32;
                // On white.
                [0, 1, 2].map(|c| ((px[c] as u32 * a + 255 * (255 - a)) / 255) as u8)
            })
            .collect();
        let mut z = flate2::write::ZlibEncoder::new(Vec::new(), flate2::Compression::default());
        z.write_all(&rgb).map_err(|e| e.to_string())?;
        let data = z.finish().map_err(|e| e.to_string())?;
        let mut body = format!(
            "<< /Type /XObject /Subtype /Image /Width {} /Height {} /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /FlateDecode /Length {} >>\nstream\n",
            img.width(),
            img.height(),
            data.len()
        )
        .into_bytes();
        body.extend_from_slice(&data);
        body.extend_from_slice(b"\nendstream");
        objects.push(body);
    }
    let top = ph - margin;
    for (pi, list) in pages.iter().enumerate() {
        let mut content: Vec<u8> = Vec::new();
        for (bi, y0) in list {
            let b = &blocks[*bi];
            let mut y = top - y0;
            for line in &b.lines {
                y -= line_h(b.size);
                content.extend_from_slice(format!("BT {margin:.2} {:.2} Td ", y + b.size * 0.25).as_bytes());
                for (s, bold) in line {
                    content.extend_from_slice(
                        format!("/{} {:.1} Tf ", if *bold { "F2" } else { "F1" }, b.size).as_bytes(),
                    );
                    content.extend_from_slice(&pdf_string(s));
                    content.extend_from_slice(b" Tj ");
                }
                content.extend_from_slice(b"ET\n");
            }
            if let Some((k, w, h)) = b.pic {
                y -= h + 6.0;
                content.extend_from_slice(
                    format!("q {w:.2} 0 0 {h:.2} {margin:.2} {y:.2} cm /Im{k} Do Q\n").as_bytes(),
                );
            }
        }
        // The foot of the page.
        let foot = format!(
            "{} — {} {} {} {}",
            g.title.replace("**", ""),
            g.page,
            pi + 1,
            g.of,
            total
        );
        content.extend_from_slice(format!("BT /F1 9 Tf {margin:.2} {:.2} Td ", margin - 4.0).as_bytes());
        content.extend_from_slice(&pdf_string(&foot));
        content.extend_from_slice(b" Tj ET\n");
        let xobjects: String = (0..pics.len())
            .map(|k| format!("/Im{k} {} 0 R ", first_pic + k))
            .collect();
        objects.push(
            format!(
                "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {pw:.2} {ph:.2}] /Resources << /Font << /F1 3 0 R /F2 4 0 R >> /XObject << {xobjects}>> >> /Contents {} 0 R >>",
                first_page + 2 * pi + 1
            )
            .into_bytes(),
        );
        let mut body = format!("<< /Length {} >>\nstream\n", content.len()).into_bytes();
        body.extend_from_slice(&content);
        body.extend_from_slice(b"\nendstream");
        objects.push(body);
    }
    for (k, (bi, pi, y)) in marks.iter().enumerate() {
        let title = blocks[*bi].mark.clone().unwrap_or_default();
        let title: String = title.chars().take(120).collect();
        let mut body = b"<< /Title ".to_vec();
        body.extend_from_slice(&pdf_string(&title));
        body.extend_from_slice(
            format!(
                " /Parent 5 0 R {}{}/Dest [{} 0 R /XYZ 0 {:.2} 0] >>",
                if k > 0 {
                    format!("/Prev {} 0 R ", first_mark + k - 1)
                } else {
                    String::new()
                },
                if k + 1 < marks.len() {
                    format!("/Next {} 0 R ", first_mark + k + 1)
                } else {
                    String::new()
                },
                first_page + 2 * pi,
                top - y
            )
            .as_bytes(),
        );
        objects.push(body);
    }
    let mut out: Vec<u8> = b"%PDF-1.4\n%\xE2\xE3\xCF\xD3\n".to_vec();
    let mut offsets = Vec::new();
    for (i, body) in objects.iter().enumerate() {
        offsets.push(out.len());
        out.extend_from_slice(format!("{} 0 obj\n", i + 1).as_bytes());
        out.extend_from_slice(body);
        out.extend_from_slice(b"\nendobj\n");
    }
    let xref = out.len();
    out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", offsets.len() + 1).as_bytes());
    for off in &offsets {
        out.extend_from_slice(format!("{off:010} 00000 n \n").as_bytes());
    }
    out.extend_from_slice(
        format!(
            "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
            offsets.len() + 1
        )
        .as_bytes(),
    );
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pic() -> Pic {
        let img = image::RgbaImage::from_pixel(400, 300, image::Rgba([200, 210, 230, 255]));
        Pic {
            png: xs_image::encode::png(&img, 0.0, true).unwrap(),
            width: 400,
            height: 300,
        }
    }

    pub(crate) fn sample(steps: usize) -> Guide {
        let mut items: Vec<Item> = (1..=steps)
            .map(|n| Item::Step {
                n,
                text: format!("Clic en el botón **Guardar {n}** (año & <más>)"),
                pic: Some(pic()),
            })
            .collect();
        items.insert(1, Item::Text("Una nota".into()));
        Guide {
            title: "Pasos en Excel".into(),
            date: "2026-11-14".into(),
            footer: "Creado por Ana · 2026-11-14".into(),
            items,
            page: "Página".into(),
            of: "de".into(),
            step: "Paso".into(),
        }
    }

    fn unzip(bytes: &[u8]) -> std::collections::BTreeMap<String, String> {
        let mut z = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
        let mut out = std::collections::BTreeMap::new();
        for i in 0..z.len() {
            let mut f = z.by_index(i).unwrap();
            let mut s = Vec::new();
            std::io::Read::read_to_end(&mut f, &mut s).unwrap();
            let name = f.name().map(|n| n.to_string()).unwrap_or_default();
            out.insert(name, String::from_utf8_lossy(&s).into_owned());
        }
        out
    }

    /// SMT-032: Markdown with a numbered list and relative pictures; HTML with its own
    /// styles, an ordered list, alternative texts and nothing from the network.
    #[test]
    fn markdown_and_html() {
        let g = sample(3);
        let names = picture_names(&g, "images/paso");
        assert_eq!(
            names,
            ["images/paso-01.png", "images/paso-02.png", "images/paso-03.png"]
        );
        let md = markdown(&g, &names);
        assert!(md.starts_with("# Pasos en Excel\n"));
        assert!(md.contains("1. Clic en el botón **Guardar 1**"));
        assert!(md.contains("   ![Clic en el botón Guardar 3 (año & <más>)](images/paso-03.png)"));
        assert!(md.contains("Una nota\n"));
        let page = html(&g, &names, "es");
        assert!(page.contains("<ol>") && page.contains("<li value=\"3\">"));
        assert!(page.contains("<strong>Guardar 1</strong>"));
        assert!(page.contains("alt=\"Clic en el botón Guardar 1 (año &amp; &lt;más&gt;)\""));
        assert!(
            !page.contains("http://") && !page.contains("https://"),
            "nothing from the network"
        );
        assert!(text(&g).contains("3. Clic en el botón Guardar 3"));
    }

    /// SMT-030: a Word document with a numbered list of 6 steps and 6 pictures with
    /// their alternative texts; a presentation with the title slide and one per step.
    #[test]
    fn word_and_powerpoint() {
        let g = sample(6);
        let files = unzip(&docx(&g).unwrap());
        let doc = &files["word/document.xml"];
        assert_eq!(doc.matches("<w:numId w:val=\"1\"/>").count(), 6);
        assert_eq!(doc.matches("<w:drawing>").count(), 6);
        assert!(doc.contains("descr=\"Clic en el botón Guardar 2 (año &amp; &lt;más&gt;)\""));
        assert!(doc.contains("<w:pStyle w:val=\"Heading1\"/>"));
        assert!(files.keys().filter(|k| k.starts_with("word/media/")).count() == 6);
        assert!(files["word/styles.xml"].contains("w:styleId=\"ListNumber\""));
        let files = unzip(&pptx(&g).unwrap());
        // The title slide, 6 steps and the text: 8.
        assert_eq!(
            files.keys().filter(|k| k.starts_with("ppt/slides/slide")).count(),
            8
        );
        assert!(files["ppt/slides/slide2.xml"].contains("txBox=\"1\""));
        assert!(files["ppt/presentation.xml"].contains("cx=\"12192000\" cy=\"6858000\""));
    }

    /// SMT-033: a PDF whose steps are never split, with a bookmark per step, searchable
    /// text and "Page N of M".
    #[test]
    fn a_pdf_of_the_guide() {
        let g = sample(12);
        let measure =
            |s: &str, bold: bool, size: f64| s.chars().count() as f64 * size * if bold { 0.58 } else { 0.52 };
        let bytes = pdf(&g, paper("a4"), &measure).unwrap();
        let text = String::from_utf8_lossy(&bytes);
        assert!(text.starts_with("%PDF-1.4"));
        assert_eq!(text.matches("/Dest [").count(), 12, "a bookmark per step");
        assert!(text.contains("(Clic en el bot\u{f3}n ") || bytes.windows(10).any(|w| w == b"Clic en el"));
        let pages = text.matches("/Type /Page /Parent").count();
        assert!(pages >= 3);
        assert!(text.contains(&format!("de {pages})")), "Page N of M");
        // Each picture is drawn exactly once: no step was cut.
        assert_eq!(text.matches(" Do Q").count(), 12);
    }
}
