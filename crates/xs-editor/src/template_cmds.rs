//! Template commands (EDT-095 to EDT-101, ADR 0007): `editor.templates`,
//! `document.from_template`, `document.template`, `document.save_as_template` and
//! `document.apply_theme`.

use std::collections::HashMap;
use std::sync::OnceLock;

use xs_core::document::{Manifest, Xshot, THUMB_SIZE};

use super::*;
use crate::templates::{self, Look, Template, MAX_IMAGES};

fn bad(lang: Lang, param: &str, reason: &str) -> CommandError {
    invalid(tr(
        lang,
        "errors.invalid_param",
        &[("param", param), ("reason", reason)],
    ))
}

fn closed() -> CommandError {
    CommandError::internal("document closed")
}

/// Where the user's own templates live (EDT-101).
pub(crate) fn templates_dir(core: &Core) -> PathBuf {
    core.paths.local_dir.join("templates")
}

const OWN: &str = "own:";
const EXT: &str = "xtemplate";

/// The file of an own template, `own:<name>`, if it is there.
fn own_path(core: &Core, id: &str) -> Option<PathBuf> {
    let name = id.strip_prefix(OWN)?;
    if name.is_empty() || name.contains(['/', '\\', ':']) || name.starts_with('.') {
        return None;
    }
    let path = templates_dir(core).join(format!("{name}.{EXT}"));
    path.is_file().then_some(path)
}

/// A name usable as a file name: no characters Windows or macOS refuse.
fn file_name(name: &str) -> Option<String> {
    let clean: String = name
        .chars()
        .filter(|c| !c.is_control() && !"<>:\"/\\|?*".contains(*c))
        .collect();
    let clean = clean.trim().trim_end_matches('.').trim();
    (!clean.is_empty()).then(|| clean.chars().take(80).collect())
}

/// `dir/<name>.xtemplate`, or `<name> (2)`… when taken.
fn free_path(dir: &Path, name: &str) -> PathBuf {
    let mut path = dir.join(format!("{name}.{EXT}"));
    let mut n = 2;
    while path.exists() {
        path = dir.join(format!("{name} ({n}).{EXT}"));
        n += 1;
    }
    path
}

fn id_of(path: &Path) -> String {
    format!(
        "{OWN}{}",
        path.file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default()
    )
}

pub(crate) fn active_theme(core: &Core) -> String {
    editor(core).themes.lock().unwrap().active.clone()
}

fn look_of(core: &Core, s: &templates::Settings) -> Look {
    let lib = editor(core).themes.lock().unwrap().clone();
    Look::of(&lib, s)
}

fn author_of(s: &templates::Settings) -> String {
    if s.author == "custom" {
        s.author_text.clone()
    } else {
        templates::system_author()
    }
}

/// Lays `t` out on `doc` again (ADR 0007).
pub(crate) fn relayout(core: &Core, lang: Lang, doc: &mut Document, t: &Template, restyle: bool) {
    let look = look_of(core, &t.settings);
    templates::layout(
        doc,
        t,
        &Styles::default(),
        &look,
        lang,
        &author_of(&t.settings),
        restyle,
    );
}

/// `settings` with the changes of `changes` (EDT-097), checked.
fn merged_settings(
    lang: Lang,
    settings: &templates::Settings,
    changes: &Value,
) -> Result<templates::Settings, CommandError> {
    let Some(changes) = changes.as_object() else {
        return Err(bad(lang, "settings", "must be an object"));
    };
    let mut v = serde_json::to_value(settings).map_err(|e| internal(lang, "", e))?;
    for (k, value) in changes {
        v[k] = value.clone();
    }
    let s: templates::Settings =
        serde_json::from_value(v).map_err(|e| bad(lang, "settings", &e.to_string()))?;
    if !["fit", "fill"].contains(&s.scale.as_str()) {
        return Err(bad(lang, "settings.scale", "fit or fill"));
    }
    if !["system", "custom"].contains(&s.author.as_str()) {
        return Err(bad(lang, "settings.author", "system or custom"));
    }
    if !["circle", "square", "roundedSquare", "diamond", "marker", "text"].contains(&s.number_shape.as_str())
    {
        return Err(bad(
            lang,
            "settings.numberShape",
            "circle, square, roundedSquare, diamond, marker or text",
        ));
    }
    for (k, c) in [
        ("settings.textColor", &s.text_color),
        ("settings.numberColor", &s.number_color),
    ] {
        if let Some(c) = c {
            xs_image::color::Rgba::parse(c).ok_or_else(|| bad(lang, k, "use #RRGGBB"))?;
        }
    }
    canvas_fill(&s.background).map_err(|e| bad(lang, "settings.background", &e))?;
    Ok(s)
}

/// What a picture for a zone is: a file, or an open document as it looks.
fn pictures_of(core: &Core, lang: Lang, list: &[Value]) -> Result<Vec<Picture>, CommandError> {
    list.iter()
        .map(|v| match v {
            Value::String(path) => picture_of(core, lang, Path::new(path)),
            Value::Object(o) => match (
                o.get("documentId").and_then(Value::as_str),
                o.get("path").and_then(Value::as_str),
            ) {
                (Some(id), _) => document_picture(core, lang, id),
                (None, Some(path)) => picture_of(core, lang, Path::new(path)),
                _ => Err(bad(lang, "images", "each is a path or {documentId}")),
            },
            _ => Err(bad(lang, "images", "each is a path or {documentId}")),
        })
        .collect()
}

/// An open document's final image as a picture.
fn document_picture(core: &Core, lang: Lang, id: &str) -> Result<Picture, CommandError> {
    let snap = editor(core).store.snapshot(id).map_err(|_| {
        CommandError::new(
            ErrorCode::NotFound,
            tr(lang, "errors.document_not_found", &[("id", id)]),
        )
    })?;
    let image = snap.render();
    let png = xs_image::encode::png(&image, 0.0, true).map_err(|e| internal(lang, "", e))?;
    Ok(picture(png, "png", image.width(), image.height()))
}

/// A zone with `name`, as placed (fitted or filling, no framing).
fn put_picture(zone: &mut Object, name: Option<&str>) {
    zone.props
        .insert("image".into(), name.map_or(Value::Null, |n| json!(n)));
    zone.props.insert("zoom".into(), json!(1.0));
    zone.props.insert("offset".into(), json!([0.0, 0.0]));
}

/// A new document from `document` (EDT-095): written to the library (or kept unsaved,
/// LIB-020) and opened.
pub(crate) fn create_document(
    core: &Arc<Core>,
    lang: Lang,
    document: Document,
    meta: Value,
    base_png: Vec<u8>,
    images: BTreeMap<String, Vec<u8>>,
) -> Result<(String, PathBuf), CommandError> {
    let ed = editor(core);
    let autosave = core.settings.get_bool("library.autosaveImages");
    let folder = if autosave {
        crate::library_target(core)
    } else {
        ed.unsaved_dir()
    };
    let path = xs_library::new_item_path(&folder, None, chrono::Local::now(), "xshot");
    let fail = |e: String| internal(lang, "", e);
    let decoded = xs_image::encode::decode(&base_png).map_err(fail)?;
    let mut base = Base::from_image(decoded);
    for (name, bytes) in &images {
        base.add_image(name, bytes).map_err(fail)?;
    }
    let thumb = xs_image::encode::thumbnail(&xs_image::render(&document, &base), THUMB_SIZE);
    let file = Xshot {
        manifest: Manifest::current(),
        document: document.clone(),
        base_png: base_png.clone(),
        meta: meta.clone(),
        thumb_png: xs_image::encode::png(&thumb, 0.0, true).map_err(fail)?,
        images: images.clone(),
    };
    {
        let _guard = ed.writing.lock().unwrap();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| fail(e.to_string()))?;
        }
        file.write(&path).map_err(|e| fail(e.to_string()))?;
    }
    let kind = if autosave { Kind::Library } else { Kind::External };
    let doc = DocStore::from_image(path.clone(), kind, document, meta, base_png, base).with_images(images);
    let (id, _) = ed.store.insert(doc, false);
    if !autosave {
        ed.store.mark_unsaved(&id);
    }
    emit_list(core);
    Ok((id, path))
}

pub(crate) fn transparent_pixel() -> Vec<u8> {
    xs_image::encode::png(&image::RgbaImage::new(1, 1), 96.0, true).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// The picker (EDT-095, EDT-096, EDT-101)

/// A built-in template laid out for `n` pictures, with its zones shown as placeholders:
/// what the picker shows.
fn builtin_preview(core: &Core, lang: Lang, b: &templates::Builtin, n: usize) -> Document {
    let mut t = templates::create(b, n);
    t.settings.theme = Some(active_theme(core));
    let mut doc = Document::new(1, 1, 96.0);
    relayout(core, lang, &mut doc, &t, true);
    doc
}

/// Empty zones drawn as grey boxes (thumbnails of templates).
fn with_placeholders(doc: &mut Document) {
    for o in doc.objects.iter_mut().filter(|o| o.kind == "zone") {
        let empty = !o.props.get("image").is_some_and(Value::is_string);
        if !empty {
            continue;
        }
        let rect = o.props.get("rect").cloned().unwrap_or(Value::Null);
        o.kind = "shape".into();
        o.props = serde_json::from_value(json!({
            "rect": rect,
            "rotation": 0,
            "style": {"shapeType": "roundedRectangle", "fill": "#DCE1E8", "outline": "#B8C0CC",
                      "outlineWidth": 2, "cornerRadius": 12, "lineStyle": "solid", "opacity": 100}
        }))
        .unwrap_or_default();
    }
}

fn preview_png(doc: &Document, base: &Base, side: u32) -> Option<Vec<u8>> {
    let mut doc = doc.clone();
    with_placeholders(&mut doc);
    let image = xs_image::render(&doc, base);
    xs_image::encode::png(&xs_image::encode::thumbnail(&image, side), 0.0, true).ok()
}

/// The picker's picture of a template (EDT-095): a built-in one for `n` pictures in the
/// active theme, or an own one as saved.
pub fn template_thumbnail(core: &Core, id: &str, n: usize) -> Option<Vec<u8>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Vec<u8>>>> = OnceLock::new();
    let lang = core.lang();
    if let Some(b) = templates::builtin(id) {
        let key = format!("{id}/{n}/{}/{}", active_theme(core), lang.code());
        let cache = CACHE.get_or_init(Mutex::default);
        if let Some(png) = cache.lock().unwrap().get(&key) {
            return Some(png.clone());
        }
        let doc = builtin_preview(core, lang, b, n.max(1));
        let png = preview_png(&doc, &Base::from_image(image::RgbaImage::new(1, 1)), 320)?;
        cache.lock().unwrap().insert(key, png.clone());
        return Some(png);
    }
    let path = own_path(core, id)?;
    Xshot::read_summary(&path).ok().map(|s| s.thumb_png)
}

fn builtin_name(lang: Lang, id: &str) -> String {
    tr(
        lang,
        &format!("editor.template.builtin.{}", id.replace('-', "_")),
        &[],
    )
}

/// `editor.templates` (EDT-095, EDT-096, EDT-101): the templates of the picker, and
/// the own ones' renaming, duplicating, deleting, importing and exporting.
pub(crate) fn templates_command(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let dir = templates_dir(core);
    let action = p.get("action").and_then(Value::as_str).unwrap_or("list");
    let text = |k: &str| p.get(k).and_then(Value::as_str);
    let io = |e: std::io::Error| internal(lang, "", e);
    let mut done = Value::Null;
    match action {
        "list" => {}
        "import" => {
            let from =
                PathBuf::from(text("path").ok_or_else(|| bad(lang, "path", "give the .xtemplate file"))?);
            let x = Xshot::read(&from)
                .map_err(|e| open_error(lang, &from, OpenError::Unreadable(e.to_string())))?;
            if !x.document.objects.iter().any(|o| o.kind == "zone") {
                return Err(bad(lang, "path", "it has no image zones"));
            }
            let stem = from
                .file_stem()
                .and_then(|s| file_name(&s.to_string_lossy()))
                .unwrap_or_else(|| "template".into());
            std::fs::create_dir_all(&dir).map_err(io)?;
            let to = free_path(&dir, &stem);
            std::fs::copy(&from, &to).map_err(io)?;
            done = json!(id_of(&to));
        }
        "rename" | "duplicate" | "delete" | "export" => {
            let id = text("id").ok_or_else(|| bad(lang, "id", "give an own template (own:<name>)"))?;
            let file = own_path(core, id).ok_or_else(|| {
                CommandError::new(
                    ErrorCode::NotFound,
                    tr(lang, "errors.template_not_found", &[("id", id)]),
                )
            })?;
            match action {
                "rename" => {
                    let name = text("name")
                        .and_then(file_name)
                        .ok_or_else(|| bad(lang, "name", "give a name"))?;
                    let to = dir.join(format!("{name}.{EXT}"));
                    if to.exists() && !crate::store::same_file(&to, &file) {
                        return Err(bad(lang, "name", "there is a template with that name"));
                    }
                    std::fs::rename(&file, &to).map_err(io)?;
                    done = json!(id_of(&to));
                }
                "duplicate" => {
                    let stem = file
                        .file_stem()
                        .map(|s| s.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    let to = free_path(&dir, &stem);
                    std::fs::copy(&file, &to).map_err(io)?;
                    done = json!(id_of(&to));
                }
                "delete" => {
                    std::fs::remove_file(&file).map_err(io)?;
                }
                _ => {
                    let to =
                        PathBuf::from(text("path").ok_or_else(|| bad(lang, "path", "where to export it"))?);
                    std::fs::copy(&file, &to).map_err(io)?;
                    done = json!(to);
                }
            }
        }
        other => return Err(bad(lang, "action", other)),
    }
    let mut list: Vec<Value> = templates::BUILTINS
        .iter()
        .map(|b| {
            json!({
                "id": b.id,
                "name": builtin_name(lang, b.id),
                "family": b.family,
                "min": b.min,
                "max": b.max,
                "own": false,
            })
        })
        .collect();
    let mut own: Vec<(String, Value)> = std::fs::read_dir(&dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case(EXT)))
        .filter_map(|path| {
            let x = Xshot::read(&path).ok()?;
            let zones = x.document.objects.iter().filter(|o| o.kind == "zone").count();
            let name = path.file_stem()?.to_string_lossy().into_owned();
            let family = x.meta["template"]["family"]
                .as_str()
                .unwrap_or("mine")
                .to_string();
            let modified = std::fs::metadata(&path)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                .map_or(0, |d| d.as_secs());
            Some((
                name.to_lowercase(),
                json!({"id": id_of(&path), "name": name, "family": family, "min": 1, "max": zones,
                       "zones": zones, "own": true, "path": path, "version": modified}),
            ))
        })
        .collect();
    own.sort_by(|a, b| a.0.cmp(&b.0));
    list.extend(own.into_iter().map(|(_, v)| v));
    Ok(json!({
        "templates": list,
        "max": MAX_IMAGES,
        "folder": dir,
        "theme": active_theme(core),
        "result": done,
    }))
}

// ---------------------------------------------------------------------------
// New from a template (EDT-095)

/// `document.from_template` (EDT-095): a new document from a built-in or own template
/// with `images` in its zones, in order.
pub(crate) fn from_template(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let which = p
        .get("template")
        .and_then(Value::as_str)
        .ok_or_else(|| bad(lang, "template", "give a template id or .xtemplate file"))?;
    let list = p
        .get("images")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut warnings = Vec::new();
    if list.len() > MAX_IMAGES {
        warnings.push(tr(
            lang,
            "editor.template.too_many",
            &[
                ("count", &list.len().to_string()),
                ("max", &MAX_IMAGES.to_string()),
            ],
        ));
    }
    let pictures = pictures_of(core, lang, &list[..list.len().min(MAX_IMAGES)])?;
    let overrides = p.get("settings").filter(|v| !v.is_null());
    let (mut document, meta, base_png, mut images, name) = match templates::builtin(which) {
        Some(b) => {
            let n = if pictures.is_empty() { 3 } else { pictures.len() };
            let mut t = templates::create(b, n);
            t.settings.theme = Some(active_theme(core));
            if let Some(changes) = overrides {
                t.settings = merged_settings(lang, &t.settings, changes)?;
            }
            let mut doc = Document::new(1, 1, 96.0);
            relayout(core, lang, &mut doc, &t, true);
            (
                doc,
                json!({"source": "template", "template": which}),
                transparent_pixel(),
                BTreeMap::new(),
                builtin_name(lang, which),
            )
        }
        None => {
            let path = own_path(core, which)
                .or_else(|| Some(PathBuf::from(which)).filter(|p| p.is_file()))
                .ok_or_else(|| {
                    CommandError::new(
                        ErrorCode::NotFound,
                        tr(lang, "errors.template_not_found", &[("id", which)]),
                    )
                })?;
            let x = Xshot::read(&path)
                .map_err(|e| open_error(lang, &path, OpenError::Unreadable(e.to_string())))?;
            let mut doc = x.document;
            if let (Some(changes), Some(mut t)) = (overrides, Template::of(&doc)) {
                t.settings = merged_settings(lang, &t.settings, changes)?;
                doc.template = serde_json::to_value(&t).ok();
            }
            let name = path
                .file_stem()
                .map(|s| s.to_string_lossy().into_owned())
                .unwrap_or_default();
            (
                doc,
                json!({"source": "template", "template": name}),
                x.base_png,
                x.images,
                name,
            )
        }
    };
    // The pictures go into the zones, in order.
    let mut zones = 0;
    let mut pics = pictures.iter();
    for zone in document.objects.iter_mut().filter(|o| o.kind == "zone") {
        zones += 1;
        if let Some(pic) = pics.next() {
            put_picture(zone, Some(&pic.name));
        }
    }
    let placed = pictures.len().min(zones);
    if pictures.len() > zones {
        warnings.push(tr(
            lang,
            "editor.template.only_zones",
            &[("count", &zones.to_string())],
        ));
    }
    for pic in &pictures[..placed] {
        images.insert(pic.name.clone(), pic.bytes.clone());
    }
    if let Some(t) = Template::of(&document) {
        if t.settings.reversed {
            templates::reverse_pictures(&mut document);
        }
        // Numbers on the zones now filled (EDT-097).
        relayout(core, lang, &mut document, &t, false);
    }
    let mut meta = meta;
    meta["name"] = json!(name);
    let (id, path) = create_document(core, lang, document, meta, base_png, images)?;
    activate(core, &id);
    let _ = core.host().show_main_window();
    Ok(json!({
        "documentId": id,
        "path": path,
        "zones": zones,
        "placed": placed,
        "warnings": warnings,
    }))
}

// ---------------------------------------------------------------------------
// Editing a template (EDT-097 to EDT-099)

/// What the Template tab shows (EDT-097, EDT-099): the template, its sections and its
/// zones.
fn info(doc: &Document) -> Value {
    let template = Template::of(doc);
    let sections: Vec<Value> = template
        .as_ref()
        .map(|t| {
            templates::bands(t)
                .into_iter()
                .map(|(s, top)| json!({"id": s.id, "kind": s.kind, "top": top, "height": s.height}))
                .collect()
        })
        .unwrap_or_default();
    let zones: Vec<Value> = doc
        .objects
        .iter()
        .filter(|o| o.kind == "zone")
        .map(|o| {
            json!({
                "objectId": o.id,
                "slot": o.props.get("slot"),
                "section": o.props.get("section"),
                "rect": o.props.get("rect"),
                "filled": o.props.get("image").is_some_and(Value::is_string),
                "zoom": o.props.get("zoom").and_then(Value::as_f64).unwrap_or(1.0),
            })
        })
        .collect();
    json!({
        "isTemplate": template.is_some(),
        "template": doc.template,
        "sections": sections,
        "zones": zones,
        "width": doc.canvas.width,
        "height": doc.canvas.height,
    })
}

/// The zone named by a slot or an object id.
fn zone_index(doc: &Document, key: &str) -> Option<usize> {
    doc.objects.iter().position(|o| {
        o.kind == "zone" && (o.id == key || o.props.get("slot").and_then(Value::as_str) == Some(key))
    })
}

/// The rest of a zone's slot (`<section>:<n>`), which its caption and number share.
fn zone_place(o: &Object) -> Option<String> {
    o.props
        .get("slot")
        .and_then(Value::as_str)
        .and_then(|s| s.strip_prefix("zone:"))
        .map(str::to_string)
}

fn slot_index(doc: &Document, slot: &str) -> Option<usize> {
    doc.objects
        .iter()
        .position(|o| o.props.get("slot").and_then(Value::as_str) == Some(slot))
}

/// `document.template` (EDT-097 to EDT-099, EDT-101): reads or changes a template, its
/// sections and zones, in one undo step each.
pub(crate) fn template_command(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let id = resolve(core, lang, p)?;
    let ed = editor(core);
    let action = p.get("action").and_then(Value::as_str).unwrap_or("info");
    let doc = ed.store.with(&id, |d| d.document.clone()).ok_or_else(closed)?;
    if action == "info" {
        return Ok(info(&doc));
    }
    let text = |k: &str| p.get(k).and_then(Value::as_str);
    let num = |k: &str| p.get(k).and_then(Value::as_f64);
    let not_template = || invalid(tr(lang, "errors.not_a_template", &[]));
    let mut template = Template::of(&doc);
    let mut next = doc.clone();
    let mut restyle = false;
    let mut added: Vec<Picture> = Vec::new();
    let section_of = |t: &Template| -> Result<usize, CommandError> {
        let sid = text("sectionId").ok_or_else(|| bad(lang, "sectionId", "give a section"))?;
        t.sections
            .iter()
            .position(|s| s.id == sid)
            .ok_or_else(|| bad(lang, "sectionId", "there is no such section"))
    };
    let zone_of = |doc: &Document, key: &str| -> Result<usize, CommandError> {
        let z = text(key).ok_or_else(|| bad(lang, key, "give a zone (its id or slot)"))?;
        zone_index(doc, z).ok_or_else(|| bad(lang, key, "there is no such zone"))
    };
    let label = match action {
        "settings" => {
            let t = template.as_mut().ok_or_else(not_template)?;
            let changes = p.get("settings").cloned().unwrap_or_default();
            let new = merged_settings(lang, &t.settings, &changes)?;
            if new.reversed != t.settings.reversed {
                templates::reverse_pictures(&mut next);
            }
            restyle = new.theme != t.settings.theme
                || new.font != t.settings.font
                || new.text_color != t.settings.text_color
                || new.number_color != t.settings.number_color
                || new.number_shape != t.settings.number_shape;
            t.settings = new;
            "history.template_settings"
        }
        "reverse" => {
            let t = template.as_mut().ok_or_else(not_template)?;
            t.settings.reversed = !t.settings.reversed;
            templates::reverse_pictures(&mut next);
            "history.template_settings"
        }
        "add_section" => {
            let t = template.as_mut().ok_or_else(not_template)?;
            // A guide adds steps, texts and pictures (SMT-029).
            let allowed: &[&str] = if t.is_guide() {
                &templates::GUIDE_ADDABLE
            } else {
                &templates::ADDABLE
            };
            let kind = text("kind").unwrap_or(if t.is_guide() { "guideStep" } else { "step" });
            if !allowed.contains(&kind) {
                return Err(bad(lang, "kind", &allowed.join(", ")));
            }
            let zones = match kind {
                "twoImages" => 2,
                "guideText" => 0,
                _ => 1,
            };
            let section = t.new_section(kind, zones);
            // At the end, before the footer, unless given.
            let end = t
                .sections
                .iter()
                .rposition(|s| s.kind != "footer")
                .map_or(0, |i| i + 1);
            let at = p
                .get("index")
                .and_then(Value::as_u64)
                .map_or(end, |i| i as usize)
                .min(t.sections.len());
            t.sections.insert(at, section);
            "history.add_section"
        }
        "remove_section" => {
            let t = template.as_mut().ok_or_else(not_template)?;
            // Several at once (SMT-029: Delete on the selected steps).
            let many: Vec<String> = p
                .get("sectionIds")
                .and_then(Value::as_array)
                .map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect())
                .unwrap_or_default();
            if many.is_empty() {
                let i = section_of(t)?;
                t.sections.remove(i);
            } else {
                t.sections.retain(|s| !many.contains(&s.id));
            }
            "history.remove_section"
        }
        "duplicate_section" => {
            let t = template.as_mut().ok_or_else(not_template)?;
            let i = section_of(t)?;
            duplicate_section(core, lang, &mut next, t, i)?;
            return finish(
                core,
                ctx,
                &id,
                &mut next,
                None,
                false,
                "history.duplicate_section",
                Vec::new(),
                lang,
            );
        }
        "resize_section" => {
            let t = template.as_mut().ok_or_else(not_template)?;
            let i = section_of(t)?;
            let h = num("height").ok_or_else(|| bad(lang, "height", "give the height in px"))?;
            if !(40.0..=5000.0).contains(&h) {
                return Err(bad(lang, "height", "between 40 and 5000 px"));
            }
            t.sections[i].height = h.round();
            "history.resize_section"
        }
        "move_section" => {
            let t = template.as_mut().ok_or_else(not_template)?;
            let i = section_of(t)?;
            let to = p
                .get("index")
                .and_then(Value::as_u64)
                .ok_or_else(|| bad(lang, "index", "give the new place"))?;
            let s = t.sections.remove(i);
            t.sections.insert((to as usize).min(t.sections.len()), s);
            "history.move_section"
        }
        "fill_zone" => {
            let z = zone_of(&next, "zone")?;
            let pic = if let Some(path) = text("path") {
                picture_of(core, lang, Path::new(path))?
            } else if let Some(source) = text("sourceDocumentId") {
                document_picture(core, lang, source)?
            } else if let Some(oid) = text("fromObject") {
                // A floating image goes into the zone (EDT-098).
                let i = next
                    .objects
                    .iter()
                    .position(|o| o.id == oid && o.kind == "image")
                    .ok_or_else(|| bad(lang, "fromObject", "there is no such floating image"))?;
                let name = next.objects[i]
                    .props
                    .get("image")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                next.objects.remove(i);
                let z = zone_index(&next, text("zone").unwrap_or(""))
                    .ok_or_else(|| bad(lang, "zone", "there is no such zone"))?;
                put_picture(&mut next.objects[z], Some(&name));
                grow_custom_steps(&mut template, &next);
                return finish(
                    core,
                    ctx,
                    &id,
                    &mut next,
                    template.as_ref(),
                    false,
                    "history.fill_zone",
                    Vec::new(),
                    lang,
                );
            } else if p.get("clipboard").and_then(Value::as_bool) == Some(true) {
                let (image, _) = read_clipboard();
                let image = image.ok_or_else(|| invalid(tr(lang, "errors.nothing_to_paste", &[])))?;
                let png = xs_image::encode::png(&image, 0.0, true).map_err(|e| internal(lang, "", e))?;
                picture(png, "png", image.width(), image.height())
            } else {
                return Err(bad(
                    lang,
                    "path",
                    "give path, sourceDocumentId, fromObject or clipboard",
                ));
            };
            put_picture(&mut next.objects[z], Some(&pic.name));
            // A guide's section takes the picture's size (SMT-029).
            if let (Some(t), Some(sid)) = (
                template.as_mut().filter(|t| t.is_guide()),
                next.objects[z].props.get("section").and_then(Value::as_str),
            ) {
                if let Some(s) = t.sections.iter_mut().find(|s| s.id == sid) {
                    s.natural = Some([pic.width as f64, pic.height as f64]);
                }
            }
            added.push(pic);
            grow_custom_steps(&mut template, &next);
            "history.fill_zone"
        }
        "clear_zone" => {
            let z = zone_of(&next, "zone")?;
            put_picture(&mut next.objects[z], None);
            "history.clear_zone"
        }
        "swap_zones" => {
            let (a, b) = (zone_of(&next, "zone")?, zone_of(&next, "with")?);
            if a != b {
                let take = |o: &Object| {
                    ["image", "zoom", "offset"].map(|k| o.props.get(k).cloned().unwrap_or(Value::Null))
                };
                let (va, vb) = (take(&next.objects[a]), take(&next.objects[b]));
                for (i, vals) in [(a, vb), (b, va)] {
                    for (k, v) in ["image", "zoom", "offset"].into_iter().zip(vals) {
                        next.objects[i].props.insert(k.into(), v);
                    }
                }
                // Their captions go with them (EDT-098).
                if let (Some(pa), Some(pb)) = (zone_place(&next.objects[a]), zone_place(&next.objects[b])) {
                    let (ca, cb) = (
                        slot_index(&next, &format!("caption:{pa}")),
                        slot_index(&next, &format!("caption:{pb}")),
                    );
                    if let (Some(ca), Some(cb)) = (ca, cb) {
                        let ta = next.objects[ca].props.get("text").cloned();
                        let tb = next.objects[cb].props.get("text").cloned();
                        if let (Some(ta), Some(tb)) = (ta, tb) {
                            next.objects[ca].props.insert("text".into(), tb);
                            next.objects[cb].props.insert("text".into(), ta);
                        }
                    }
                }
            }
            "history.swap_zones"
        }
        "frame_zone" => {
            let z = zone_of(&next, "zone")?;
            let o = &mut next.objects[z];
            if p.get("reset").and_then(Value::as_bool) == Some(true) {
                o.props.insert("zoom".into(), json!(1.0));
                o.props.insert("offset".into(), json!([0.0, 0.0]));
            } else {
                let zoom = o.props.get("zoom").and_then(Value::as_f64).unwrap_or(1.0);
                // Steps of 10 % (EDT-098).
                let zoom = match (num("zoom"), num("zoomBy")) {
                    (Some(z), _) => z,
                    (None, Some(steps)) => ((zoom + 0.1 * steps) * 100.0).round() / 100.0,
                    _ => zoom,
                };
                // A guide frames a small part of a large monitor (SMT-026): up to 100.
                if !(0.1..=100.0).contains(&zoom) {
                    return Err(bad(lang, "zoom", "between 0.1 and 100"));
                }
                o.props.insert("zoom".into(), json!(zoom));
                if let Some(off) = p.get("offset").and_then(Value::as_array) {
                    let x = off.first().and_then(Value::as_f64).unwrap_or(0.0);
                    let y = off.get(1).and_then(Value::as_f64).unwrap_or(0.0);
                    o.props.insert("offset".into(), json!([x.round(), y.round()]));
                }
            }
            "history.frame_zone"
        }
        "remove_caption" => {
            let t = template.as_mut().ok_or_else(not_template)?;
            let z = zone_of(&next, "zone")?;
            let place = zone_place(&next.objects[z]).ok_or_else(|| bad(lang, "zone", "it has no caption"))?;
            let slot = format!("caption:{place}");
            if !t.hidden.contains(&slot) {
                t.hidden.push(slot);
            }
            "history.remove_caption"
        }
        "convert_zone" => {
            // A floating image becomes a zone (EDT-101).
            let oid = text("objectId").ok_or_else(|| bad(lang, "objectId", "give a floating image"))?;
            let o = next
                .objects
                .iter_mut()
                .find(|o| o.id == oid && o.kind == "image")
                .ok_or_else(|| bad(lang, "objectId", "there is no such floating image"))?;
            let rect = o.props.get("rect").cloned().unwrap_or(Value::Null);
            let r = |k: &str| rect.get(k).and_then(Value::as_f64).unwrap_or(0.0);
            let image = o.props.get("image").and_then(Value::as_str).map(str::to_string);
            let props = objects::zone_props([r("x"), r("y"), r("w"), r("h")], image.as_deref(), false)
                .map_err(invalid)?;
            o.kind = "zone".into();
            let locked = o.props.get("locked").cloned();
            o.props = props;
            if let Some(l) = locked {
                o.props.insert("locked".into(), l);
            }
            "history.convert_zone"
        }
        other => return Err(bad(lang, "action", other)),
    };
    let restyle_now = restyle;
    finish(
        core,
        ctx,
        &id,
        &mut next,
        template.as_ref(),
        restyle_now,
        label,
        added,
        lang,
    )
}

/// "Custom steps" grows with each picture (EDT-096): a new empty step once every zone
/// has one.
fn grow_custom_steps(template: &mut Option<Template>, doc: &Document) {
    let Some(t) = template.as_mut().filter(|t| t.id == "steps-custom") else {
        return;
    };
    let full = doc
        .objects
        .iter()
        .filter(|o| o.kind == "zone")
        .all(|o| o.props.get("image").is_some_and(Value::is_string));
    let steps = t.sections.iter().filter(|s| s.kind == "step").count();
    if full && steps < MAX_IMAGES {
        let s = t.new_section("step", 1);
        let end = t
            .sections
            .iter()
            .rposition(|s| s.kind != "footer")
            .map_or(0, |i| i + 1);
        t.sections.insert(end, s);
    }
}

/// Duplicates a guide's section `i` (SMT-029): its picture, framing and text, and the
/// annotations on it, right after it.
fn duplicate_section(
    core: &Arc<Core>,
    lang: Lang,
    next: &mut Document,
    t: &mut Template,
    i: usize,
) -> Result<(), CommandError> {
    let orig = t.sections[i].clone();
    if !templates::GUIDE_ADDABLE.contains(&orig.kind.as_str()) {
        return Err(bad(lang, "sectionId", "a guide's step, text or picture"));
    }
    let mut copy = t.new_section(&orig.kind, orig.zones);
    copy.height = orig.height;
    copy.picture = orig.picture;
    copy.natural = orig.natural;
    let new_id = copy.id.clone();
    t.sections.insert(i + 1, copy);
    // Its places, named for the copy: the layout keeps their text and picture.
    let placed: Vec<Object> = next
        .objects
        .iter()
        .filter(|o| o.props.get("section").and_then(Value::as_str) == Some(orig.id.as_str()))
        .filter(|o| o.props.get("slot").is_some())
        .cloned()
        .collect();
    for mut o in placed {
        let slot = o
            .props
            .get("slot")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let renamed = slot.replacen(&format!(":{}", orig.id), &format!(":{new_id}"), 1);
        o.props.insert("slot".into(), json!(renamed));
        o.props.insert("section".into(), json!(new_id));
        o.id = next.next_object_id();
        next.objects.push(o);
    }
    relayout(core, lang, next, t, false);
    // The annotations on the original go on the copy too, as far down as it is.
    let zone_y = |doc: &Document, sid: &str| {
        doc.objects
            .iter()
            .find(|o| o.kind == "zone" && o.props.get("section").and_then(Value::as_str) == Some(sid))
            .and_then(|o| o.props.get("rect")?.get("y")?.as_f64())
    };
    let laid = Template::of(next).unwrap_or_else(|| t.clone());
    let band = templates::bands(&laid)
        .into_iter()
        .find(|(s, _)| s.id == orig.id)
        .map(|(s, top)| (top, s.height));
    let copy_top = templates::bands(&laid)
        .into_iter()
        .find(|(s, _)| s.id == new_id)
        .map(|(_, top)| top);
    let (Some((top, h)), Some(copy_top)) = (band, copy_top) else {
        return Ok(());
    };
    let dy = match (zone_y(next, &orig.id), zone_y(next, &new_id)) {
        (Some(a), Some(b)) => b - a,
        _ => copy_top - top,
    };
    let dpi = next.dpi;
    let copies: Vec<Object> = next
        .objects
        .iter()
        .filter(|o| o.props.get("slot").is_none())
        .filter(|o| {
            xs_image::object_bounds(o, dpi).is_some_and(|b| {
                let cy = (b[1] + b[3]) / 2.0;
                cy >= top && cy < top + h
            })
        })
        .cloned()
        .collect();
    for mut o in copies {
        o.id = next.next_object_id();
        o.translate(0.0, dy);
        next.objects.push(o);
    }
    Ok(())
}

/// Lays the template out again if there is one, keeps new pictures and makes the
/// change one undo step.
#[allow(clippy::too_many_arguments)]
fn finish(
    core: &Arc<Core>,
    ctx: &Context,
    id: &str,
    next: &mut Document,
    template: Option<&Template>,
    restyle: bool,
    label: &str,
    added: Vec<Picture>,
    lang: Lang,
) -> CommandResult {
    let ed = editor(core);
    if let Some(t) = template {
        relayout(core, lang, next, t, restyle);
    }
    if !added.is_empty() {
        let files = added.iter().map(|p| (p.name.clone(), p.bytes.clone())).collect();
        ed.store
            .add_images(id, &files)
            .map_err(|e| internal(lang, "", e))?;
    }
    let state = next.clone();
    ed.store
        .mutate(id, label, |d| {
            *d = state;
            Ok::<_, CommandError>(())
        })
        .ok_or_else(closed)??;
    changed(core, ctx, id);
    let mut out = info(next);
    out["changed"] = json!(true);
    Ok(out)
}

// ---------------------------------------------------------------------------
// Own templates (EDT-101)

/// `document.save_as_template` (EDT-101): the document as an own `.xtemplate`.
pub(crate) fn save_as_template(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let id = resolve(core, lang, p)?;
    let ed = editor(core);
    let name = p
        .get("name")
        .and_then(Value::as_str)
        .and_then(file_name)
        .ok_or_else(|| bad(lang, "name", "give a name"))?;
    let family = p.get("family").and_then(Value::as_str).unwrap_or("mine");
    if !["steps", "timeline", "compare", "collage", "mine"].contains(&family) {
        return Err(bad(lang, "family", "steps, timeline, compare, collage or mine"));
    }
    let empty = p.get("emptyZones").and_then(Value::as_bool).unwrap_or(true);
    let mut snap = ed.store.snapshot(&id).map_err(|e| internal(lang, "", e))?;
    if !snap.document.objects.iter().any(|o| o.kind == "zone") {
        return Err(invalid(tr(lang, "errors.template_without_zones", &[])));
    }
    if empty {
        for z in snap.document.objects.iter_mut().filter(|o| o.kind == "zone") {
            put_picture(z, None);
        }
        if let Some(t) = Template::of(&snap.document) {
            let mut doc = snap.document.clone();
            relayout(core, lang, &mut doc, &t, false);
            snap.document = doc;
        }
    }
    snap.meta = json!({"source": "template", "template": {"name": name, "family": family}});
    let mut file = snap.to_xshot(false).map_err(|e| internal(lang, "", e))?;
    // The picker shows the zones as placeholders.
    if let Some(png) = preview_png(&snap.document, &snap.base, THUMB_SIZE) {
        file.thumb_png = png;
    }
    let dir = templates_dir(core);
    let path = dir.join(format!("{name}.{EXT}"));
    if path.exists() && p.get("replace").and_then(Value::as_bool) != Some(true) {
        return Err(CommandError::new(
            ErrorCode::InvalidParams,
            tr(lang, "errors.template_exists", &[("name", &name)]),
        ));
    }
    {
        let _guard = ed.writing.lock().unwrap();
        std::fs::create_dir_all(&dir).map_err(|e| internal(lang, "", e))?;
        file.write(&path).map_err(|e| internal(lang, "", e))?;
    }
    Ok(json!({"id": id_of(&path), "path": path}))
}

// ---------------------------------------------------------------------------
// Theme colors on existing objects (EDT-100)

fn rgb(hex: &str) -> Option<[f64; 3]> {
    xs_image::color::Rgba::parse(hex).map(|c| [c.r as f64, c.g as f64, c.b as f64])
}

/// `color` in the new theme (EDT-100): the color in the same place as in the old theme,
/// or the closest one. Its transparency stays.
fn recolor(color: &str, from: &[String], to: &[String]) -> Option<String> {
    let c = color.trim();
    if !(c.len() == 7 || c.len() == 9) || !c.starts_with('#') || to.is_empty() {
        return None;
    }
    let (body, alpha) = c.split_at(7);
    let [r, g, b] = rgb(body)?;
    let new = match from.iter().position(|f| f.eq_ignore_ascii_case(body)) {
        Some(i) => to[i % to.len()].clone(),
        None => to
            .iter()
            .min_by(|x, y| {
                let d = |h: &str| {
                    rgb(h).map_or(f64::MAX, |[a, b2, c2]| {
                        (a - r).powi(2) + (b2 - g).powi(2) + (c2 - b).powi(2)
                    })
                };
                d(x).total_cmp(&d(y))
            })?
            .clone(),
    };
    Some(format!("{}{alpha}", new.to_uppercase()))
}

/// Every color of a style in the new theme, and its font; shadows keep theirs.
fn restyle_value(v: &mut Value, from: &[String], to: &[String], font: Option<&str>) {
    match v {
        Value::Object(map) => {
            for (k, child) in map.iter_mut() {
                if k == "shadow" {
                    continue;
                }
                if k == "font" {
                    if let Some(f) = font {
                        *child = json!(f);
                    }
                    continue;
                }
                restyle_value(child, from, to, font);
            }
        }
        Value::Array(items) => items.iter_mut().for_each(|i| restyle_value(i, from, to, font)),
        Value::String(s) => {
            if let Some(c) = recolor(s, from, to) {
                *s = c;
            }
        }
        _ => {}
    }
}

/// `document.apply_theme` (EDT-100): the colors and font of a theme on the selected
/// objects, or all, in one undo step.
pub(crate) fn apply_theme(core: &Arc<Core>, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let id = resolve(core, lang, p)?;
    let ed = editor(core);
    let lib = ed.themes.lock().unwrap().clone();
    let find = |k: &str| -> Result<Option<&themes::Theme>, CommandError> {
        match p.get(k).and_then(Value::as_str) {
            None => Ok(None),
            Some(tid) => lib
                .themes
                .iter()
                .find(|t| t.id == tid)
                .map(Some)
                .ok_or_else(|| bad(lang, k, "there is no such theme")),
        }
    };
    let to = find("themeId")?
        .ok_or_else(|| bad(lang, "themeId", "give a theme"))?
        .clone();
    let doc = ed.store.with(&id, |d| d.document.clone()).ok_or_else(closed)?;
    let mut template = Template::of(&doc);
    let from = match find("fromThemeId")? {
        Some(t) => t.clone(),
        None => template
            .as_ref()
            .and_then(|t| t.settings.theme.as_deref())
            .and_then(|tid| lib.themes.iter().find(|t| t.id == tid))
            .unwrap_or_else(|| lib.active())
            .clone(),
    };
    let only: Option<Vec<String>> = p
        .get("objectIds")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect())
        .filter(|v: &Vec<String>| !v.is_empty());
    let mut next = doc.clone();
    let mut count = 0;
    for o in next.objects.iter_mut() {
        if only.as_ref().is_some_and(|ids| !ids.contains(&o.id))
            || ["zone", "image"].contains(&o.kind.as_str())
        {
            continue;
        }
        let Some(style) = o.props.get_mut("style") else {
            continue;
        };
        let before = style.clone();
        restyle_value(style, &from.colors, &to.colors, to.font.as_deref());
        // A step's value stays readable on its new fill.
        if o.kind == "step" {
            let fill = style.get("fill").and_then(Value::as_str).map(str::to_string);
            if let Some(fill) = fill {
                let light = themes::luminance(&fill) >= 150.0;
                let darkest = to
                    .colors
                    .iter()
                    .min_by(|a, b| themes::luminance(a).total_cmp(&themes::luminance(b)))
                    .cloned()
                    .unwrap_or_else(|| "#000000".into());
                style["textColor"] = json!(if light { darkest } else { "#FFFFFF".to_string() });
            }
        }
        if *style != before {
            count += 1;
        }
    }
    // A template keeps the theme for what it lays out next.
    if only.is_none() {
        if let Some(t) = template.as_mut() {
            t.settings.theme = Some(to.id.clone());
            t.settings.number_color = None;
            t.settings.text_color = None;
            next.template = serde_json::to_value(&*t).ok();
        }
    }
    ed.store
        .mutate(&id, "history.apply_theme", |d| {
            *d = next.clone();
            Ok::<_, CommandError>(())
        })
        .ok_or_else(closed)??;
    changed(core, ctx, &id);
    Ok(json!({"changed": count}))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_follow_their_place_or_the_closest() {
        let base: Vec<String> = ["#1F2430", "#F2545B", "#2D7FF9"].map(String::from).to_vec();
        let sober: Vec<String> = ["#1F1F1F", "#6B6B6B", "#B5B5B5"].map(String::from).to_vec();
        assert_eq!(recolor("#f2545b", &base, &sober).as_deref(), Some("#6B6B6B"));
        assert_eq!(recolor("#2D7FF980", &base, &sober).as_deref(), Some("#B5B5B580"));
        // Not of the theme: the closest.
        assert_eq!(recolor("#000000", &base, &sober).as_deref(), Some("#1F1F1F"));
        assert_eq!(recolor("solid", &base, &sober), None);
    }

    #[test]
    fn file_names_lose_what_systems_refuse() {
        assert_eq!(file_name(" My: steps? ").as_deref(), Some("My steps"));
        assert_eq!(file_name("..."), None);
        assert_eq!(file_name("<>"), None);
    }
}
