//! Extract text (SMT-004 to SMT-006): from a zone of the screen (a capture mode and
//! its own shortcut), from a file, or again from the same zone in another language.
//! The text and the zone's image live in memory only while their results window is
//! open; nothing is written to the logs or the events but counts.

use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

use image::RgbaImage;
use serde_json::{json, Value};
use xs_core::i18n::tr;
use xs_core::{CommandError, CommandResult, Context, Core, ErrorCode, Lang};

/// The languages every computer should read (SMT-004), and the extra ones offered.
pub const REQUIRED: [&str; 5] = ["en", "es", "pt", "fr", "de"];
pub const EXTRA: [&str; 3] = ["ja", "zh-Hans", "zh-Hant"];

/// Where the system adds a recognition language (SMT-004).
pub const SETTINGS_URI: &str = if cfg!(windows) {
    "ms-settings:regionlanguage"
} else {
    "x-apple.systempreferences:com.apple.Localization-Settings.extension"
};

struct Kept {
    image: RgbaImage,
    result: Value,
}

static KEPT: Mutex<Option<HashMap<String, Kept>>> = Mutex::new(None);

fn new_id() -> String {
    use std::sync::atomic::{AtomicU32, Ordering};
    static N: AtomicU32 = AtomicU32::new(1);
    format!("t{}", N.fetch_add(1, Ordering::Relaxed))
}

/// The recognition languages: what this computer has, and for each required or extra
/// one whether it is there (SMT-004).
pub fn languages(lang: Lang) -> Value {
    let installed = xs_ocr::languages();
    let has = |code: &str| {
        installed
            .iter()
            .find(|l| l.tag == code || l.tag.starts_with(&format!("{code}-")))
            .map(|l| l.tag.clone())
    };
    let row = |code: &str, required: bool| {
        json!({
            "code": code,
            "name": tr(lang, &format!("ui.extract.lang.{}", code.replace('-', "_")), &[]),
            "tag": has(code),
            "available": has(code).is_some(),
            "required": required,
        })
    };
    let mut list: Vec<Value> = REQUIRED.iter().map(|c| row(c, true)).collect();
    list.extend(EXTRA.iter().map(|c| row(c, false)));
    json!({
        "languages": list,
        "installed": installed,
        "settingsUri": SETTINGS_URI,
    })
}

/// The languages a recognition uses: the ones asked for, else the setting; `auto` is
/// the interface's language (when this computer reads it) and the system's ones.
pub fn wanted(core: &Core, asked: Option<&Value>) -> Vec<String> {
    let list: Vec<String> = match asked {
        Some(Value::String(s)) => vec![s.clone()],
        Some(Value::Array(a)) => a.iter().filter_map(Value::as_str).map(str::to_string).collect(),
        _ => core
            .settings
            .get("smart.ocr.languages")
            .and_then(|v| v.as_array().cloned())
            .unwrap_or_default()
            .iter()
            .filter_map(Value::as_str)
            .map(str::to_string)
            .collect(),
    };
    let installed = xs_ocr::languages();
    let tag_of = |code: &str| {
        installed
            .iter()
            .find(|l| l.tag == code || l.tag.starts_with(&format!("{code}-")))
            .map(|l| l.tag.clone())
    };
    let mut out = Vec::new();
    for item in if list.is_empty() {
        vec!["auto".to_string()]
    } else {
        list
    } {
        if item == "auto" {
            if let Some(tag) = tag_of(core.lang().code()) {
                out.push(tag);
            }
            out.push("auto".to_string());
        } else if let Some(tag) = tag_of(&item) {
            out.push(tag);
        }
    }
    out.dedup();
    out.truncate(3);
    out
}

fn ocr_error(lang: Lang, e: xs_ocr::OcrError) -> CommandError {
    match e {
        xs_ocr::OcrError::Cancelled => {
            CommandError::new(ErrorCode::Cancelled, tr(lang, "errors.text_cancelled", &[]))
        }
        xs_ocr::OcrError::NotSupported => CommandError::new(
            ErrorCode::NotSupportedOnPlatform,
            tr(lang, "errors.ocr_not_supported", &[]),
        ),
        xs_ocr::OcrError::NoLanguage => CommandError::new(
            ErrorCode::FeatureDisabled,
            tr(lang, "errors.ocr_no_language", &[]),
        )
        .with_details(json!({ "settingsUri": SETTINGS_URI })),
        xs_ocr::OcrError::Engine(m) => CommandError::internal(m),
    }
}

/// Recognizes an image and keeps the result for its window: the text plain and with
/// format, with the lines of each paragraph apart or joined.
pub fn recognize(
    core: &Core,
    image: RgbaImage,
    asked: Option<&Value>,
    lang: Lang,
) -> Result<Value, CommandError> {
    let languages = wanted(core, asked);
    let started = std::time::Instant::now();
    let r = xs_ocr::recognize(&image, &languages, &AtomicBool::new(false)).map_err(|e| ocr_error(lang, e))?;
    // Counts only: never the text (SMT-001).
    tracing::info!(
        words = r.words(),
        elapsed_ms = started.elapsed().as_millis() as u64,
        language = %r.language,
        "text extracted"
    );
    let id = new_id();
    let result = json!({
        "id": id,
        "found": r.words() > 0,
        "words": r.words(),
        "language": r.language,
        "text": xs_ocr::plain(&r.blocks, false),
        "textJoined": xs_ocr::plain(&r.blocks, true),
        "html": xs_ocr::html(&r.blocks, false),
        "htmlJoined": xs_ocr::html(&r.blocks, true),
        "blocks": r.blocks,
        "width": image.width(),
        "height": image.height(),
    });
    KEPT.lock().unwrap().get_or_insert_with(HashMap::new).insert(
        id,
        Kept {
            image,
            result: result.clone(),
        },
    );
    Ok(result)
}

/// The same zone again, in another language (SMT-006): the window keeps its id.
pub fn again(core: &Core, id: &str, language: &Value, lang: Lang) -> CommandResult {
    let image = KEPT
        .lock()
        .unwrap()
        .as_ref()
        .and_then(|m| m.get(id))
        .map(|k| k.image.clone())
        .ok_or_else(|| not_kept(lang, id))?;
    let mut fresh = recognize(core, image, Some(language), lang)?;
    let new_id = fresh["id"].as_str().unwrap_or_default().to_string();
    let mut kept = KEPT.lock().unwrap();
    let map = kept.get_or_insert_with(HashMap::new);
    if let Some(mut k) = map.remove(&new_id) {
        fresh["id"] = json!(id);
        k.result = fresh.clone();
        map.insert(id.to_string(), k);
    }
    Ok(fresh)
}

fn not_kept(lang: Lang, id: &str) -> CommandError {
    CommandError::new(
        ErrorCode::NotFound,
        tr(lang, "errors.text_result_gone", &[("id", id)]),
    )
}

pub fn kept(id: &str, lang: Lang) -> CommandResult {
    KEPT.lock()
        .unwrap()
        .as_ref()
        .and_then(|m| m.get(id))
        .map(|k| k.result.clone())
        .ok_or_else(|| not_kept(lang, id))
}

/// Its window closed: the text and the image go (SMT-001).
pub fn forget(id: &str) {
    if let Some(m) = KEPT.lock().unwrap().as_mut() {
        m.remove(id);
    }
}

/// Puts text on the clipboard, with its format when there is one, so that each app
/// pastes what it takes (SMT-006).
pub fn copy_text(text: &str, html: Option<&str>) -> Result<(), String> {
    let _lock = xs_core::clipboard_lock();
    let mut last = String::new();
    for _ in 0..10 {
        let done = arboard::Clipboard::new().and_then(|mut c| match html {
            Some(h) => c.set_html(h, Some(text)),
            None => c.set_text(text),
        });
        match done {
            Ok(()) => return Ok(()),
            Err(e) => last = e.to_string(),
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    Err(last)
}

/// What a capture in Extract text mode does with its image (SMT-005): recognize, copy
/// the text, and show the results window or a notification. Automation gets the text
/// back and opens no window unless it asks (`show`).
pub fn deliver(
    core: &Core,
    ctx: &Context,
    image: RgbaImage,
    copy: bool,
    show: Option<bool>,
) -> CommandResult {
    let lang = ctx.lang;
    let result = recognize(core, image, None, lang)?;
    let id = result["id"].as_str().unwrap_or_default().to_string();
    let found = result["found"] == json!(true);
    let plain = core.settings.get_bool("smart.extractText.plain");
    if found && copy {
        let html = (!plain).then(|| result["html"].as_str().unwrap_or_default().to_string());
        if let Err(e) = copy_text(result["text"].as_str().unwrap_or_default(), html.as_deref()) {
            tracing::warn!(error = %e, "extracted text not copied");
        }
    }
    let automation = ctx.source.is_automation();
    let show = show.unwrap_or(!automation && core.settings.get_bool("smart.extractText.showResults"));
    let host = core.host();
    if show {
        let _ = host.open_text_result(&id);
    } else if automation {
        // Nothing to tell: the result is the answer.
    } else if found {
        let words = result["words"].as_u64().unwrap_or(0).to_string();
        host.notify_opening(
            "captures",
            &tr(lang, "app.name", &[]),
            &tr(lang, "notify.text_copied", &[("count", &words)]),
            json!({ "textResult": id }),
        );
    } else {
        host.notify_opening(
            "captures",
            &tr(lang, "app.name", &[]),
            &tr(lang, "notify.text_none", &[]),
            json!({ "textResult": id }),
        );
    }
    // The text goes back to whoever asked (an agent, a script), not to the logs.
    let mut out = result;
    if let Some(o) = out.as_object_mut() {
        o.remove("blocks");
        o.remove("htmlJoined");
        o.remove("textJoined");
        o.insert("copied".into(), json!(found && copy));
    }
    Ok(out)
}

fn upgrade(weak: &std::sync::Weak<Core>) -> Result<std::sync::Arc<Core>, CommandError> {
    weak.upgrade()
        .ok_or_else(|| CommandError::internal("shutting down"))
}

fn id_of(p: &Value) -> String {
    p.get("id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

/// `smart.extract_text` (SMT-005): from the screen (a zone the user selects, or `rect`)
/// or from an image file.
fn extract_command(core: &Core, ctx: &Context, p: &Value) -> CommandResult {
    let lang = ctx.lang;
    let copy = p.get("copy").and_then(Value::as_bool).unwrap_or(true);
    let show = p.get("show").and_then(Value::as_bool);
    match p.get("source").and_then(Value::as_str).unwrap_or("screen") {
        "file" => {
            let path = p.get("path").and_then(Value::as_str).unwrap_or_default();
            let image = image::open(path)
                .map_err(|e| {
                    CommandError::new(
                        ErrorCode::InvalidParams,
                        tr(
                            lang,
                            "errors.invalid_param",
                            &[("param", "path"), ("reason", &e.to_string())],
                        ),
                    )
                })?
                .to_rgba8();
            deliver(core, ctx, image, copy, show)
        }
        _ => {
            let mut params = json!({
                "extract": true,
                "cursor": false,
                "copy": copy,
                "delay": p.get("delay").and_then(Value::as_f64).unwrap_or(0.0),
            });
            if let Some(s) = show {
                params["show"] = json!(s);
            }
            if let Some(rect) = p.get("rect") {
                params["rect"] = rect.clone();
            }
            crate::dispatch(core, ctx, "capture.region", &params, json!({}))
        }
    }
}

/// Handlers of the `smart` area that read text (SMT-004 to SMT-006).
pub fn register(reg: &mut xs_core::command::Registry, weak: &std::sync::Weak<Core>) {
    macro_rules! on {
        ($id:literal, $f:expr) => {{
            let w = weak.clone();
            reg.handle($id, move |ctx, p| {
                let core = upgrade(&w)?;
                $f(&core, ctx, &p)
            });
        }};
    }
    on!("smart.extract_text", extract_command);
    on!(
        "smart.ocr_languages",
        |_core: &Core, ctx: &Context, _p: &Value| Ok(languages(ctx.lang))
    );
    on!(
        "smart.text_result",
        |_core: &Core, ctx: &Context, p: &Value| kept(&id_of(p), ctx.lang)
    );
    on!("smart.text_again", |core: &Core, ctx: &Context, p: &Value| again(
        core,
        &id_of(p),
        &p["language"],
        ctx.lang
    ));
    on!("smart.text_close", |_core: &Core, _ctx: &Context, p: &Value| {
        forget(&id_of(p));
        Ok(json!({}))
    });
    on!("smart.show_text", |core: &Core, _ctx: &Context, p: &Value| {
        core.host().open_text_result(&id_of(p))?;
        Ok(json!({}))
    });
    on!("smart.copy_text", |_core: &Core, _ctx: &Context, p: &Value| {
        let text = p["text"].as_str().unwrap_or_default();
        copy_text(text, p["html"].as_str()).map_err(CommandError::internal)?;
        Ok(json!({}))
    });
    on!(
        "smart.language_settings",
        |_core: &Core, _ctx: &Context, _p: &Value| {
            // The system's own page; XcreenShot downloads nothing itself (SMT-001).
            #[cfg(windows)]
            let opened = std::process::Command::new("explorer.exe")
                .arg(SETTINGS_URI)
                .spawn();
            #[cfg(not(windows))]
            let opened = std::process::Command::new("open").arg(SETTINGS_URI).spawn();
            opened.map_err(|e| CommandError::internal(e.to_string()))?;
            Ok(json!({ "uri": SETTINGS_URI }))
        }
    );
    on!("smart.save_text", |_core: &Core, _ctx: &Context, p: &Value| {
        let path = std::path::PathBuf::from(p["path"].as_str().unwrap_or_default());
        let text = p["text"].as_str().unwrap_or_default();
        xs_core::paths::write_atomic(&path, text.as_bytes())
            .map_err(|e| CommandError::internal(e.to_string()))?;
        Ok(json!({ "path": path }))
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use xs_core::document::{Document, Object};
    use xs_core::platform::Platform;
    use xs_core::Source;

    /// A PNG with two paragraphs of text drawn by the app's text engine.
    fn text_png(dir: &std::path::Path) -> std::path::PathBuf {
        let mut doc = Document::new(800, 220, 96.0);
        for (i, (y, text)) in [
            (20.0, "Primer párrafo de prueba"),
            (120.0, "Segundo párrafo aquí"),
        ]
        .iter()
        .enumerate()
        {
            let props = json!({"at": [20, y], "width": null, "text": text, "rotation": 0,
                "style": {"font": "Segoe UI", "size": 18, "bold": false, "italic": false,
                          "color": "#000000", "align": "left", "opacity": 100}});
            doc.objects.push(Object {
                id: format!("o{i}"),
                kind: "text".into(),
                props: props.as_object().unwrap().clone(),
            });
        }
        let base = xs_image::render::Base {
            width: 800,
            height: 220,
            rgba: Arc::new(vec![255; 800 * 220 * 4]),
            images: Arc::new(Default::default()),
        };
        let path = dir.join("texto.png");
        xs_image::render::render(&doc, &base).save(&path).unwrap();
        path
    }

    /// SMT-005, SMT-006: an image file's text comes back to automation without any
    /// window or clipboard; its result stays while its window is open, then goes.
    #[test]
    fn text_of_an_image_file() {
        if xs_ocr::languages().is_empty() {
            eprintln!("no recognition language on this computer: skipped");
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let path = text_png(dir.path());
        let core = Core::for_tests_with(Platform::Windows, vec![crate::extension()]);
        let call = |id: &str, p: Value| core.call(Source::Cli, id, p, Some(Lang::Es));
        let r = call(
            "smart.extract_text",
            json!({"source": "file", "path": path, "copy": false, "language": "es"}),
        )
        .unwrap();
        assert_eq!(r["found"], true);
        assert_eq!(r["copied"], false);
        let text = r["text"].as_str().unwrap();
        assert!(text.contains("Primer"), "{text}");
        assert!(text.contains("\n\n"), "two paragraphs: {text:?}");
        let id = r["id"].as_str().unwrap();
        let kept = core
            .call(Source::Ui, "smart.text_result", json!({"id": id}), None)
            .unwrap();
        assert_eq!(kept["text"], r["text"]);
        assert!(kept["htmlJoined"].as_str().unwrap().contains("<p"));
        core.call(Source::Ui, "smart.text_close", json!({"id": id}), None)
            .unwrap();
        assert!(core
            .call(Source::Ui, "smart.text_result", json!({"id": id}), None)
            .is_err());
    }

    /// SMT-004: the required languages say whether this computer has them, and where
    /// the system adds one.
    #[test]
    fn languages_say_what_is_missing() {
        let v = languages(Lang::Es);
        let list = v["languages"].as_array().unwrap();
        assert_eq!(list.len(), 8);
        assert_eq!(list[1]["code"], "es");
        assert_eq!(list[1]["name"], "Español");
        assert!(list.iter().take(5).all(|l| l["required"] == true));
        assert!(v["settingsUri"].as_str().unwrap().starts_with("ms-settings:"));
    }

    /// SMT-005: Selection = Extract text in the Image tab reads a zone; the pointer
    /// never comes, the delay does.
    #[test]
    fn the_image_tab_plans_extract_text() {
        let mut panel = crate::panel::PanelSettings::default();
        panel.image.selection = "text".into();
        panel.image.cursor = true;
        panel.image.timing = "delay".into();
        panel.image.delay = 3;
        assert!(panel.validate().is_ok());
        let (command, params, _) = panel.plan("image");
        assert_eq!(command, "capture.region");
        assert_eq!(params["extract"], true);
        assert_eq!(params["cursor"], false);
        assert_eq!(params["delay"], 3);
        let mut universal = crate::panel::PanelSettings::default();
        universal.universal.selection = "text".into();
        assert!(universal.validate().is_err(), "only the Image tab has it");
    }
}
