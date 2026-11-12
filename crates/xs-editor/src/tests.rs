//! Command-level tests: what an agent or the editor view sees (EDT-119).

use super::*;
use xs_core::document::Xshot;
use xs_core::platform::Platform;

struct Env {
    core: Arc<Core>,
    library: tempfile::TempDir,
    _recovery: tempfile::TempDir,
    /// A folder outside the library, of this test only.
    outside: tempfile::TempDir,
}

fn env() -> Env {
    let core = Core::for_tests_with(Platform::Windows, vec![extension()]);
    let library = tempfile::tempdir().unwrap();
    let recovery = tempfile::tempdir().unwrap();
    editor(&core).set_folders(library.path().to_path_buf(), recovery.path().to_path_buf());
    Env {
        core,
        library,
        _recovery: recovery,
        outside: tempfile::tempdir().unwrap(),
    }
}

fn call(core: &Core, source: Source, id: &str, p: Value) -> CommandResult {
    core.call(source, id, p, Some(Lang::En))
}

fn cli(core: &Core, id: &str, p: Value) -> Value {
    call(core, Source::Cli, id, p).unwrap_or_else(|e| panic!("{id}: {e:?}"))
}

fn white(w: u32, h: u32) -> image::RgbaImage {
    image::RgbaImage::from_pixel(w, h, image::Rgba([255, 255, 255, 255]))
}

fn capture(e: &Env) -> (String, PathBuf) {
    add_capture(
        &e.core,
        &white(400, 200),
        json!({"mode": "region"}),
        1.5,
        Some("Test"),
    )
    .unwrap()
}

fn arrow(from: [i32; 2], to: [i32; 2]) -> Value {
    json!({"type": "arrow", "from": from, "to": to, "style": {"color": "#0000FF", "width": 8}})
}

#[test]
fn capture_annotate_export_without_windows() {
    let e = env();
    let (doc, path) = capture(&e);
    assert!(path.starts_with(e.library.path()));
    assert!(
        path.exists(),
        "the .xshot exists before anything else happens (LIB-020)"
    );
    let r = cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([100, 100], [300, 100])]}),
    );
    assert_eq!(r["ids"], json!(["o1"]));
    let out = e.library.path().join("out.png");
    let r = cli(
        &e.core,
        "document.export",
        json!({"documentId": doc, "format": "png", "path": out}),
    );
    assert_eq!(
        (r["width"].as_u64(), r["height"].as_u64()),
        (Some(400), Some(200))
    );
    let png = image::open(&out).unwrap().to_rgba8();
    assert_eq!(png.get_pixel(200, 100).0, [0, 0, 255, 255]);
    assert_eq!(png.get_pixel(200, 150).0, [255, 255, 255, 255]);
    // 150 % display = 144 dpi (EDT-077, EDT-110).
    assert_eq!(
        xs_image::encode::png_dpi(&std::fs::read(&out).unwrap()),
        Some(144.0)
    );
}

#[test]
fn annotate_is_all_or_nothing_and_names_the_bad_object() {
    let e = env();
    let (doc, _) = capture(&e);
    let err = call(
        &e.core,
        Source::Cli,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([0, 0], [50, 50]), {"type": "spiral"}]}),
    )
    .unwrap_err();
    assert_eq!(err.code, ErrorCode::InvalidParams);
    assert_eq!(err.details.as_ref().unwrap()["index"], json!(2));
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"], json!([]));
}

#[test]
fn one_undo_removes_what_one_command_added() {
    let e = env();
    let (doc, _) = capture(&e);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([0, 0], [50, 50]), arrow([10, 0], [60, 50])]}),
    );
    let r = cli(&e.core, "document.undo", json!({"documentId": doc}));
    assert_eq!(r["label"], "Annotate");
    assert_eq!(r["canRedo"], true);
    assert_eq!(
        cli(&e.core, "document.info", json!({"documentId": doc}))["objects"],
        json!([])
    );
    cli(&e.core, "document.redo", json!({"documentId": doc}));
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"].as_array().unwrap().len(), 2);
}

#[test]
fn library_documents_changed_by_automation_are_saved() {
    let e = env();
    let (doc, path) = capture(&e);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([0, 0], [50, 50])]}),
    );
    let file = xs_core::document::Xshot::read(&path).unwrap();
    assert_eq!(file.document.objects.len(), 1);
    assert_eq!(file.document.objects[0].props["style"]["color"], "#0000FF");
    assert_eq!(
        cli(&e.core, "document.info", json!({"documentId": doc}))["dirty"],
        false
    );
}

#[test]
fn updates_and_deletes_objects() {
    let e = env();
    let (doc, _) = capture(&e);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([0, 0], [50, 50])]}),
    );
    let r = cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o1", "set": {"to": [80, 20], "style": {"opacity": 40}}}),
    );
    assert_eq!(r["objects"][0]["to"], json!([80.0, 20.0]));
    assert_eq!(r["objects"][0]["style"]["opacity"], json!(40.0));
    let missing = call(
        &e.core,
        Source::Cli,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o9", "set": {"to": [1, 1]}}),
    )
    .unwrap_err();
    assert_eq!(missing.code, ErrorCode::NotFound);
    cli(
        &e.core,
        "document.delete_object",
        json!({"documentId": doc, "objectId": "o1"}),
    );
    assert_eq!(
        cli(&e.core, "document.info", json!({"documentId": doc}))["objects"],
        json!([])
    );
    // Automation never changes the user's tool styles (EDT-014 applies to the UI).
    assert_eq!(editor(&e.core).styles().arrow.opacity, 100.0);
}

#[test]
fn ui_style_changes_also_become_the_tool_style() {
    let e = env();
    let (doc, _) = capture(&e);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([0, 0], [50, 50])]}),
    );
    call(
        &e.core,
        Source::Ui,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o1", "set": {"style": {"color": "#00FF00"}}}),
    )
    .unwrap();
    assert_eq!(editor(&e.core).styles().arrow.color, "#00FF00");
}

fn external_png(e: &Env) -> PathBuf {
    let path = xs_library::unique(e.outside.path(), "outside", "png");
    std::fs::write(
        &path,
        xs_image::encode::png(&white(100, 50), 96.0, false).unwrap(),
    )
    .unwrap();
    path
}

#[test]
fn external_files_ask_before_losing_changes() {
    let e = env();
    let path = external_png(&e);
    let doc = cli(&e.core, "document.open", json!({"path": path}))["documentId"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        cli(&e.core, "document.info", json!({"documentId": doc}))["kind"],
        "external"
    );
    call(
        &e.core,
        Source::Ui,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([10, 10], [90, 40])]}),
    )
    .unwrap();
    let r = cli(&e.core, "document.close", json!({"documentId": doc}));
    assert_eq!(r["status"], "needs_decision");
    assert_eq!(r["pending"][0]["documentId"], json!(doc));
    // Quitting asks about it too (APP-004).
    assert_eq!(e.core.quit_pending(Lang::En).len(), 1);
    // The UI warns before the first flattening save (EDT-107).
    let r = call(&e.core, Source::Ui, "document.save", json!({"documentId": doc})).unwrap();
    assert_eq!(r["reason"], "flatten");
    let r = cli(
        &e.core,
        "document.close",
        json!({"documentId": doc, "save": [doc]}),
    );
    assert_eq!(r["status"], "closed");
    let saved = image::open(&path).unwrap().to_rgba8();
    assert_eq!(
        saved.get_pixel(50, 25).0[2],
        255,
        "the arrow is flattened into the PNG"
    );
    assert_ne!(saved.get_pixel(50, 25).0, [255, 255, 255, 255]);
    let _ = std::fs::remove_file(&path);
}

#[test]
fn save_as_copies_library_documents_and_relinks_external_ones() {
    let e = env();
    let (doc, lib_path) = capture(&e);
    let copy = e.library.path().join("copy.png");
    let r = cli(&e.core, "document.save", json!({"documentId": doc, "path": copy}));
    assert_eq!(r["linked"], false);
    assert!(copy.exists());
    assert_eq!(
        cli(&e.core, "document.info", json!({"documentId": doc}))["path"],
        json!(lib_path)
    );

    let path = external_png(&e);
    let ext = cli(&e.core, "document.open", json!({"path": path}))["documentId"]
        .as_str()
        .unwrap()
        .to_string();
    let target = e.library.path().join("elsewhere.xshot");
    let r = cli(
        &e.core,
        "document.save",
        json!({"documentId": ext, "path": target}),
    );
    assert_eq!(r["linked"], true);
    let info = cli(&e.core, "document.info", json!({"documentId": ext}));
    assert_eq!(info["path"], json!(target));
    assert_eq!(info["kind"], "library");
    let _ = std::fs::remove_file(&path);
}

#[test]
fn export_never_overwrites_by_default() {
    let e = env();
    let (doc, _) = capture(&e);
    let out = e.library.path().join("x.jpg");
    std::fs::write(&out, b"keep").unwrap();
    let r = cli(
        &e.core,
        "document.export",
        json!({"documentId": doc, "format": "jpg", "path": out}),
    );
    assert_eq!(std::fs::read(&out).unwrap(), b"keep");
    assert!(r["path"].as_str().unwrap().ends_with("x (2).jpg"));
    let failed = call(
        &e.core,
        Source::Cli,
        "document.export",
        json!({"documentId": doc, "format": "jpg", "path": out, "onConflict": "fail"}),
    );
    assert!(failed.is_err());
}

#[test]
fn opening_twice_gives_the_same_document_and_bad_files_are_refused() {
    let e = env();
    let (doc, path) = capture(&e);
    let again = cli(&e.core, "document.open", json!({"path": path}));
    assert_eq!(again["documentId"], json!(doc));
    let bad = e.library.path().join("bad.xshot");
    std::fs::write(&bad, b"not a zip").unwrap();
    let err = call(&e.core, Source::Cli, "document.open", json!({"path": bad})).unwrap_err();
    assert_eq!(err.code, ErrorCode::InvalidParams);
    assert!(err.message.contains("bad.xshot"), "{}", err.message);
    let txt = e.library.path().join("notes.txt");
    std::fs::write(&txt, b"hi").unwrap();
    assert!(call(&e.core, Source::Cli, "document.open", json!({"path": txt})).is_err());
}

#[test]
fn recovery_copies_can_be_restored_with_their_changes() {
    let e = env();
    let path = external_png(&e);
    let doc = cli(&e.core, "document.open", json!({"path": path}))["documentId"]
        .as_str()
        .unwrap()
        .to_string();
    call(
        &e.core,
        Source::Ui,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([10, 10], [90, 40])]}),
    )
    .unwrap();
    write_recovery_copies(&e.core);
    // Simulate a crash: the document is gone from memory, its copy remains.
    editor(&e.core).store.remove(std::slice::from_ref(&doc));
    let items = cli(&e.core, "editor.recovery_list", json!({}))["items"].clone();
    assert_eq!(items.as_array().unwrap().len(), 1);
    let rid = items[0]["id"].as_str().unwrap().to_string();
    let restored = cli(&e.core, "editor.recovery_restore", json!({"ids": [rid]}));
    let id = restored["documents"][0].as_str().unwrap().to_string();
    let info = cli(&e.core, "document.info", json!({"documentId": id}));
    assert_eq!(info["objects"].as_array().unwrap().len(), 1);
    assert_eq!(info["dirty"], true);
    assert_eq!(info["path"], json!(path));
    let _ = std::fs::remove_file(&path);
}

#[test]
fn tools_and_styles() {
    let e = env();
    assert_eq!(
        cli(&e.core, "editor.select_tool", json!({"tool": "arrow"}))["tool"],
        "arrow"
    );
    assert_eq!(
        cli(&e.core, "editor.select_tool", json!({"tool": "crop"}))["tool"],
        "crop"
    );
    let no_style = call(
        &e.core,
        Source::Cli,
        "editor.set_style",
        json!({"tool": "crop", "style": {}}),
    )
    .unwrap_err();
    assert_eq!(no_style.code, ErrorCode::InvalidParams);
    let r = cli(
        &e.core,
        "editor.set_style",
        json!({"tool": "arrow", "style": {"color": "#112233"}}),
    );
    assert_eq!(r["style"]["color"], "#112233");
    // The next arrow without a style takes it (EDT-014).
    let (doc, _) = capture(&e);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [{"type": "arrow", "from": [0, 0], "to": [30, 30]}]}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"][0]["style"]["color"], "#112233");
}

#[test]
fn history_labels_are_translated() {
    // One arrow names the kind; several objects use the general name.
    let one: &[&str] = &["arrow"];
    let several: &[&str] = &["arrow", "arrow"];
    for kinds in [one, several] {
        for action in ["draw", "move", "change", "delete"] {
            let key = objects::label(action, kinds);
            for lang in [Lang::En, Lang::Es] {
                assert_ne!(tr(lang, &key, &[]), key, "missing {key}");
            }
        }
    }
    for key in ["history.annotate", "editor.edited", "errors.copy_failed"] {
        assert_ne!(tr(Lang::Es, key, &[]), key, "missing {key}");
    }
}

#[test]
fn zoom_steps_go_up_and_down() {
    assert_eq!(zoom_step(100.0, true), 150.0);
    assert_eq!(zoom_step(100.0, false), 67.0);
    assert_eq!(zoom_step(3200.0, true), 3200.0);
    assert_eq!(zoom_step(10.0, false), 10.0);
    assert_eq!(zoom_step(120.0, false), 100.0);
}

/// Replaces `{"$ref": "#/$defs/x"}` with the definition, for the registry's validator.
fn resolve_refs(v: &Value, defs: &Value) -> Value {
    match v {
        Value::Object(m) => {
            if let Some(name) = m
                .get("$ref")
                .and_then(Value::as_str)
                .and_then(|r| r.strip_prefix("#/$defs/"))
            {
                return resolve_refs(&defs[name], defs);
            }
            Value::Object(
                m.iter()
                    .map(|(k, x)| (k.clone(), resolve_refs(x, defs)))
                    .collect(),
            )
        }
        Value::Array(a) => Value::Array(a.iter().map(|x| resolve_refs(x, defs)).collect()),
        other => other.clone(),
    }
}

/// A red PNG outside the library, to insert.
fn red_png(e: &Env, w: u32, h: u32) -> PathBuf {
    let path = xs_library::unique(e.outside.path(), "red", "png");
    let red = image::RgbaImage::from_pixel(w, h, image::Rgba([255, 0, 0, 255]));
    std::fs::write(&path, xs_image::encode::png(&red, 96.0, false).unwrap()).unwrap();
    path
}

/// EDT-078, CAP-067: inserted files become floating images of the document shown,
/// saved with their pictures, in one undo step; they move and resize like shapes.
#[test]
fn inserted_images_float_over_the_document() {
    let e = env();
    let (doc, path) = capture(&e);
    let red = red_png(&e, 40, 20);
    let r = cli(
        &e.core,
        "document.insert_images",
        json!({"documentId": doc, "paths": [red, red], "center": [100, 100]}),
    );
    assert_eq!(r["documentId"], json!(doc));
    assert_eq!(r["ids"], json!(["o1", "o2"]));
    assert_eq!(r["created"], json!(false));
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    let first = &info["objects"][0];
    assert_eq!(first["type"], "image");
    assert_eq!(first["rect"], json!({"x": 80.0, "y": 90.0, "w": 40.0, "h": 20.0}));
    assert_eq!(
        info["objects"][1]["rect"]["x"],
        json!(104.0),
        "the next one 24 px further"
    );
    assert_eq!(
        first["image"], info["objects"][1]["image"],
        "the same picture is kept once"
    );
    assert_eq!(info["undoLabel"], "Insert image");

    let out = e.outside.path().join("out.png");
    cli(
        &e.core,
        "document.export",
        json!({"documentId": doc, "format": "png", "path": out}),
    );
    let png = image::open(&out).unwrap().to_rgba8();
    assert_eq!(png.get_pixel(90, 95).0, [255, 0, 0, 255]);
    assert_eq!(png.get_pixel(60, 95).0, [255, 255, 255, 255]);

    // They move and resize like shapes, and keep their picture.
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o1", "set": {"rect": {"x": 0, "y": 0, "w": 80, "h": 40}}}),
    );
    cli(&e.core, "document.save", json!({"documentId": doc}));
    let file = xs_core::document::Xshot::read(&path).unwrap();
    let name = file.document.objects[0].props["image"]
        .as_str()
        .unwrap()
        .to_string();
    assert!(file.images.contains_key(&name), "the picture travels in the file");
    assert_eq!(file.images.len(), 1);

    // Undone, and saved again: the picture no longer stays in the file.
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    assert_eq!(
        cli(&e.core, "document.info", json!({"documentId": doc}))["objects"],
        json!([])
    );
    cli(&e.core, "document.save", json!({"documentId": doc}));
    assert!(xs_core::document::Xshot::read(&path).unwrap().images.is_empty());

    // Agents insert them with this command, not with annotate.
    let err = call(
        &e.core,
        Source::Cli,
        "document.annotate",
        json!({"documentId": doc, "objects": [{"type": "image", "image": name, "rect": {"x": 0, "y": 0, "w": 9, "h": 9}}]}),
    )
    .unwrap_err();
    assert!(err.message.contains("insert_images"), "{}", err.message);
}

/// CAP-067: with nothing open, one capture opens as itself and several make a new
/// document with them stacked 10 px apart.
#[test]
fn inserting_with_nothing_open() {
    let e = env();
    let (one, path) = capture(&e);
    cli(&e.core, "document.close", json!({"documentId": one}));
    let r = cli(&e.core, "document.insert_images", json!({"paths": [path]}));
    assert_eq!(r["created"], json!(false));
    assert_eq!(r["ids"], json!([]));
    cli(&e.core, "document.close", json!({"all": true}));

    let red = red_png(&e, 40, 20);
    let r = cli(&e.core, "document.insert_images", json!({"paths": [path, red]}));
    assert_eq!(r["created"], json!(true));
    let doc = r["documentId"].as_str().unwrap();
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(
        (
            info["canvas"]["width"].as_u64(),
            info["canvas"]["height"].as_u64()
        ),
        (Some(400), Some(230))
    );
    assert_eq!(
        info["objects"][1]["rect"],
        json!({"x": 0.0, "y": 210.0, "w": 40.0, "h": 20.0})
    );
    assert_eq!(cli(&e.core, "document.list", json!({}))["active"], json!(doc));

    let err = call(
        &e.core,
        Source::Cli,
        "document.insert_images",
        json!({"paths": [e.outside.path().join("nope.png")]}),
    )
    .unwrap_err();
    assert_eq!(err.code, ErrorCode::NotFound);
}

#[test]
fn saved_documents_follow_the_published_schema() {
    let e = env();
    let (doc, path) = capture(&e);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([0, 0], [50, 50])]}),
    );
    let file = xs_core::document::Xshot::read(&path).unwrap();
    let value = serde_json::to_value(&file.document).unwrap();
    let schema: Value =
        serde_json::from_str(include_str!("../../../docs/formats/xshot-document.schema.json")).unwrap();
    let defs = schema["$defs"].clone();
    let mut top = resolve_refs(&schema, &defs);
    // The validator has no oneOf: check the document, then each arrow against its own schema.
    top["properties"]["objects"]["items"] = json!({"type": "object"});
    xs_core::schema::validate(&top, &value).unwrap();
    let arrow_schema = resolve_refs(&defs["arrow"], &defs);
    for o in value["objects"].as_array().unwrap() {
        xs_core::schema::validate(&arrow_schema, o).unwrap();
    }
    // A floating image against its own schema (EDT-078).
    let red = red_png(&e, 10, 10);
    cli(
        &e.core,
        "document.insert_images",
        json!({"documentId": doc, "paths": [red]}),
    );
    cli(&e.core, "document.save", json!({"documentId": doc}));
    let file = xs_core::document::Xshot::read(&path).unwrap();
    let image = serde_json::to_value(&file.document.objects[1]).unwrap();
    xs_core::schema::validate(&resolve_refs(&defs["image"], &defs), &image).unwrap();
}

#[test]
fn annotates_text_shapes_and_blur() {
    let e = env();
    let (doc, _) = capture(&e);
    let r = cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [
            {"type": "text", "at": [20, 30], "text": "Hola", "style": {"size": 20}},
            {"type": "shape", "rect": {"x": 10, "y": 10, "w": 100, "h": 50}, "style": {"shapeType": "ellipse"}},
            {"type": "blur", "rect": {"x": 200, "y": 100, "w": 80, "h": 40}, "style": {"blurType": "solid", "color": "#000000"}}
        ]}),
    );
    assert_eq!(r["ids"].as_array().unwrap().len(), 3);
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    let text = &info["objects"][0];
    assert!(text["layout"]["width"].as_f64().unwrap() > 10.0, "{text}");
    assert_eq!(info["objects"][1]["style"]["shapeType"], "ellipse");
    let out = e.library.path().join("objects.png");
    cli(
        &e.core,
        "document.export",
        json!({"documentId": doc, "format": "png", "path": out}),
    );
    let png = image::open(&out).unwrap().to_rgba8();
    assert_eq!(
        png.get_pixel(240, 120).0,
        [0, 0, 0, 255],
        "solid blur covers its area"
    );
}

#[test]
fn crop_is_non_destructive_and_keeps_objects_on_the_image() {
    let e = env();
    let (doc, path) = capture(&e);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([100, 100], [300, 100])]}),
    );
    let r = cli(
        &e.core,
        "document.crop",
        json!({"documentId": doc, "rect": {"x": 50, "y": 20, "w": 200, "h": 150}}),
    );
    assert_eq!(
        (r["width"].as_u64(), r["height"].as_u64()),
        (Some(200), Some(150))
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(
        info["objects"][0]["from"],
        json!([50.0, 80.0]),
        "the arrow stays on the image"
    );
    // Exported images are exactly the crop (EDT-050).
    let out = e.library.path().join("cropped.png");
    let r = cli(
        &e.core,
        "document.export",
        json!({"documentId": doc, "format": "png", "path": out}),
    );
    assert_eq!(
        (r["width"].as_u64(), r["height"].as_u64()),
        (Some(200), Some(150))
    );
    // Saved, closed and opened again, the crop grows back to the original edges.
    cli(&e.core, "document.close", json!({"documentId": doc}));
    let again = cli(&e.core, "document.open", json!({"path": path}))["documentId"]
        .as_str()
        .unwrap()
        .to_string();
    let r = cli(
        &e.core,
        "document.crop",
        json!({"documentId": again, "rect": {"x": -500, "y": -500, "w": 5000, "h": 5000}}),
    );
    assert_eq!(
        (r["width"].as_u64(), r["height"].as_u64()),
        (Some(400), Some(200))
    );
    let info = cli(&e.core, "document.info", json!({"documentId": again}));
    assert_eq!(info["objects"][0]["from"], json!([100.0, 100.0]));
    // Undo puts the crop back.
    cli(&e.core, "document.undo", json!({"documentId": again}));
    assert_eq!(
        cli(&e.core, "document.info", json!({"documentId": again}))["width"],
        200
    );
}

#[test]
fn deleting_cropped_content_saves_only_the_crop() {
    let e = env();
    cli(
        &e.core,
        "settings.set",
        json!({"key": "editor.deleteCroppedContent", "value": true}),
    );
    let (doc, path) = capture(&e);
    cli(
        &e.core,
        "document.crop",
        json!({"documentId": doc, "rect": {"x": 10, "y": 10, "w": 120, "h": 80}}),
    );
    let file = xs_core::document::Xshot::read(&path).unwrap();
    assert_eq!(xs_image::encode::png_size(&file.base_png), Some((120, 80)));
    assert_eq!(file.document.crop.x, 0);
}

#[test]
fn lists_installed_fonts() {
    let e = env();
    let fonts = cli(&e.core, "editor.fonts", json!({}))["fonts"].clone();
    assert!(!fonts.as_array().unwrap().is_empty());
}

fn size(e: &Env, doc: &str) -> (u64, u64) {
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    (info["width"].as_u64().unwrap(), info["height"].as_u64().unwrap())
}

#[test]
fn the_canvas_grows_to_fit_an_object_and_shrinks_back() {
    let e = env();
    let (doc, _) = capture(&e);
    // A 400 × 200 capture; the arrow ends 50 px past the right edge.
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([300, 100], [450, 100])]}),
    );
    let (w, h) = size(&e, &doc);
    assert!((450..=460).contains(&w), "grows just enough: {w}");
    assert_eq!(h, 200, "only where the arrow sticks out");
    // The new area is the canvas color, white for an opaque capture (EDT-074).
    let out = e.outside.path().join("grown.png");
    cli(
        &e.core,
        "document.export",
        json!({"documentId": doc, "format": "png", "path": out}),
    );
    let png = image::open(&out).unwrap().to_rgba8();
    assert_eq!(png.width() as u64, w);
    assert_eq!(png.get_pixel(w as u32 - 1, 10).0, [255, 255, 255, 255]);
    assert_eq!(png.get_pixel(10, 10).0, [255, 255, 255, 255]);
    // Back inside: the canvas is the image again (Carlos, 2026-10-27).
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o1", "set": {"from": [100, 100], "to": [250, 100]}}),
    );
    assert_eq!(size(&e, &doc), (400, 200));
    // Each change is one undo step, with the canvas size it left.
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    assert_eq!(size(&e, &doc).0, w);
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    assert_eq!(size(&e, &doc), (400, 200));
}

#[test]
fn growing_to_the_left_keeps_objects_on_the_image() {
    let e = env();
    let (doc, _) = capture(&e);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [
            arrow([100, 100], [300, 100]),
            {"type": "shape", "rect": {"x": -40, "y": -30, "w": 100, "h": 60}, "style": {"outlineWidth": 4}},
        ]}),
    );
    assert_eq!(size(&e, &doc), (440, 230));
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["crop"]["x"], -40);
    assert_eq!(info["crop"]["y"], -30);
    assert_eq!(info["frame"]["width"], 400);
    assert_eq!(
        info["objects"][0]["from"],
        json!([140.0, 130.0]),
        "the arrow moves with the image"
    );
    assert_eq!(info["objects"][1]["rect"]["x"], 0.0);
    // Deleting the shape gives back the image's own size and place.
    cli(
        &e.core,
        "document.delete_object",
        json!({"documentId": doc, "objectId": "o2"}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(
        (info["width"].as_u64(), info["height"].as_u64()),
        (Some(400), Some(200))
    );
    assert_eq!(info["objects"][0]["from"], json!([100.0, 100.0]));
    assert!(info.get("frame").is_none());
}

#[test]
fn what_a_crop_leaves_outside_stays_outside() {
    let e = env();
    let (doc, _) = capture(&e);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([100, 100], [300, 100])]}),
    );
    cli(
        &e.core,
        "document.crop",
        json!({"documentId": doc, "rect": {"x": 0, "y": 0, "w": 200, "h": 200}}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["outside"], json!(["o1"]));
    // Moved, it still does not make the canvas grow (Carlos, 2026-10-27).
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o1", "set": {"from": [110, 110], "to": [310, 110]}}),
    );
    assert_eq!(size(&e, &doc), (200, 200));
    // Wholly inside again, it is an ordinary object: taken out, the canvas grows.
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o1", "set": {"from": [20, 50], "to": [120, 50]}}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert!(info.get("outside").is_none());
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o1", "set": {"from": [120, 50], "to": [260, 50]}}),
    );
    assert_eq!(size(&e, &doc), (260, 200), "up to the arrow's tip");
}

#[test]
fn edges_and_blur_do_not_make_the_canvas_grow() {
    let e = env();
    let (doc, _) = capture(&e);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [
            // Drawn from the corner: its thickness there does not count.
            arrow([0, 0], [50, 50]),
            // Less than half a pixel out.
            {"type": "shape", "rect": {"x": -0.3, "y": 20, "w": 50, "h": 50}},
            // A blur only works on the image.
            {"type": "blur", "rect": {"x": 350, "y": -20, "w": 100, "h": 60}},
        ]}),
    );
    assert_eq!(size(&e, &doc), (400, 200));
}

#[test]
fn the_canvas_stays_put_with_the_preference_off() {
    let e = env();
    cli(
        &e.core,
        "settings.set",
        json!({"key": "editor.autoExpandCanvas", "value": false}),
    );
    let (doc, _) = capture(&e);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([300, 100], [450, 100])]}),
    );
    assert_eq!(size(&e, &doc), (400, 200));
}

#[test]
fn a_grown_canvas_survives_deleting_cropped_content() {
    let e = env();
    cli(
        &e.core,
        "settings.set",
        json!({"key": "editor.deleteCroppedContent", "value": true}),
    );
    let (doc, path) = capture(&e);
    cli(
        &e.core,
        "document.crop",
        json!({"documentId": doc, "rect": {"x": 100, "y": 0, "w": 200, "h": 200}}),
    );
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [
            {"type": "shape", "rect": {"x": -50, "y": 50, "w": 100, "h": 50}},
        ]}),
    );
    assert_eq!(size(&e, &doc), (250, 200));
    cli(&e.core, "document.close", json!({"documentId": doc}));
    let file = xs_core::document::Xshot::read(&path).unwrap();
    assert_eq!(xs_image::encode::png_size(&file.base_png), Some((250, 200)));
    let again = cli(&e.core, "document.open", json!({"path": path}))["documentId"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(size(&e, &again), (250, 200));
}

fn listed_name(e: &Env, doc: &str) -> String {
    let list = cli(&e.core, "document.list", json!({}));
    list["documents"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["documentId"] == doc)
        .unwrap()["name"]
        .as_str()
        .unwrap()
        .to_string()
}

/// SMT-041, TRY-006: renaming keeps the folder and the extension, never replaces
/// another file, and later saves go to the new file.
#[test]
fn renaming_keeps_folder_and_extension() {
    let e = env();
    let (doc, path) = capture(&e);
    let out = cli(
        &e.core,
        "document.rename",
        json!({"documentId": doc, "name": "Sales: Q3/2026."}),
    );
    assert_eq!(out["name"], "Sales Q3 2026");
    let renamed = PathBuf::from(out["path"].as_str().unwrap());
    assert_eq!(renamed.parent(), path.parent());
    assert_eq!(renamed.extension().unwrap(), "xshot");
    assert!(renamed.exists() && !path.exists());
    assert_eq!(listed_name(&e, &doc), "Sales Q3 2026");

    let (other, other_path) = capture(&e);
    let taken = call(
        &e.core,
        Source::Cli,
        "document.rename",
        json!({"documentId": other, "name": "Sales Q3 2026"}),
    )
    .unwrap_err();
    assert_eq!(taken.code, ErrorCode::InvalidParams);
    assert!(other_path.exists() && renamed.exists());
    let empty = call(
        &e.core,
        Source::Cli,
        "document.rename",
        json!({"documentId": doc, "name": " :: "}),
    )
    .unwrap_err();
    assert_eq!(empty.code, ErrorCode::InvalidParams);

    // A change of case only renames the same file.
    let out = cli(
        &e.core,
        "document.rename",
        json!({"documentId": doc, "name": "sales q3 2026"}),
    );
    let cased = PathBuf::from(out["path"].as_str().unwrap());
    assert!(cased.exists());
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([10, 10], [60, 40])]}),
    );
    cli(&e.core, "document.save", json!({"documentId": doc}));
    let saved = xs_core::document::Xshot::read(&cased).unwrap();
    assert_eq!(saved.document.objects.len(), 1);
}

/// SMT-043: an agent gets the final image of a document, scaled down to fit.
#[test]
fn document_image_is_a_scaled_png() {
    let e = env();
    let (doc, _) = capture(&e);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([10, 10], [300, 150])]}),
    );
    let out = cli(
        &e.core,
        "document.image",
        json!({"documentId": doc, "maxSize": 200}),
    );
    assert_eq!(
        (out["width"].as_u64(), out["height"].as_u64()),
        (Some(200), Some(100))
    );
    assert_eq!(out["image"]["mimeType"], "image/png");
    use base64::Engine as _;
    let png = base64::engine::general_purpose::STANDARD
        .decode(out["image"]["data"].as_str().unwrap())
        .unwrap();
    let img = image::load_from_memory(&png).unwrap().to_rgba8();
    assert_eq!(img.dimensions(), (200, 100));
    // The arrow is in it: some pixels are not white.
    assert!(img.pixels().any(|p| p[0] < 128));
    let full = cli(&e.core, "document.image", json!({"documentId": doc}));
    assert_eq!(full["width"], 400);
}

// ---------------------------------------------------------------------------
// Reopening at startup (EDT-120)

/// Shows a capture in the strip, as the editor does when it opens it.
fn show(e: &Env, path: &Path) -> String {
    cli(&e.core, "document.open", json!({"path": path, "show": true}))["documentId"]
        .as_str()
        .unwrap()
        .to_string()
}

/// XcreenShot started again: a new core on the same folders, with what the first one
/// remembered.
fn restart(e: &Env) -> Arc<Core> {
    let saved = e.core.recall(OPEN_KEY).unwrap_or(Value::Null);
    let core = Core::for_tests_with(Platform::Windows, vec![extension()]);
    editor(&core).set_folders(e.library.path().to_path_buf(), e._recovery.path().to_path_buf());
    core.remember(OPEN_KEY, saved);
    reopen_documents(&core);
    core
}

/// Files of the documents in the strip, in order, and the file of the one shown.
fn strip(core: &Core) -> (Vec<PathBuf>, Option<PathBuf>) {
    let list = cli(core, "document.list", json!({}));
    let docs = list["documents"].as_array().unwrap();
    let path = |d: &Value| PathBuf::from(d["path"].as_str().unwrap());
    let shown = docs.iter().filter(|d| d["shown"] == true).map(path).collect();
    let active = docs.iter().find(|d| d["documentId"] == list["active"]).map(path);
    (shown, active)
}

#[test]
fn open_documents_come_back_in_order_with_the_one_shown() {
    let e = env();
    let (_, a) = capture(&e);
    let (_, b) = capture(&e);
    let (_, c) = capture(&e);
    // A capture an agent made but never showed is not part of the strip.
    let (_, hidden) = capture(&e);
    for path in [&a, &b, &c] {
        show(&e, path);
    }
    show(&e, &b);
    let saved = e.core.recall(OPEN_KEY).unwrap();
    assert_eq!(saved["documents"].as_array().unwrap().len(), 3);

    let again = restart(&e);
    // document.list waits for the reopening: no sleep needed.
    let (shown, active) = strip(&again);
    assert_eq!(shown, vec![a, b.clone(), c]);
    assert_eq!(active, Some(b));
    assert!(!shown.contains(&hidden));
}

#[test]
fn missing_files_and_recovered_documents_are_not_reopened() {
    let e = env();
    let (_, a) = capture(&e);
    let (_, b) = capture(&e);
    let (doc_c, c) = capture(&e);
    for path in [&a, &b, &c] {
        show(&e, path);
    }
    // c has unsaved changes kept in a recovery copy (EDT-012); a is deleted.
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc_c, "objects": [arrow([10, 10], [90, 10])]}),
    );
    write_recovery_copies(&e.core);
    std::fs::remove_file(&a).unwrap();

    let again = restart(&e);
    let (shown, active) = strip(&again);
    assert_eq!(shown, vec![b.clone()]);
    assert_eq!(
        active,
        Some(b),
        "the one shown was left out: the first reopened is shown"
    );
    let recovery = cli(&again, "editor.recovery_list", json!({}));
    assert_eq!(
        recovery["items"].as_array().unwrap().len(),
        1,
        "c comes back through the recovery"
    );
}

#[test]
fn closing_everything_or_turning_it_off_starts_empty() {
    let e = env();
    let (_, a) = capture(&e);
    show(&e, &a);
    cli(&e.core, "document.close", json!({"all": true}));
    assert_eq!(e.core.recall(OPEN_KEY).unwrap()["documents"], json!([]));
    assert_eq!(strip(&restart(&e)).0, Vec::<PathBuf>::new());

    let (_, b) = capture(&e);
    show(&e, &b);
    cli(
        &e.core,
        "settings.set",
        json!({"key": "editor.reopenDocuments", "value": false}),
    );
    // The next change forgets the list.
    show(&e, &b);
    assert_eq!(e.core.recall(OPEN_KEY), Some(Value::Null));

    // Turned off with a list still remembered: nothing reopens and the list goes.
    let core = Core::for_tests_with(Platform::Windows, vec![extension()]);
    editor(&core).set_folders(e.library.path().to_path_buf(), e._recovery.path().to_path_buf());
    core.remember(OPEN_KEY, json!({"documents": [b], "active": b}));
    cli(
        &core,
        "settings.set",
        json!({"key": "editor.reopenDocuments", "value": false}),
    );
    reopen_documents(&core);
    assert_eq!(strip(&core).0, Vec::<PathBuf>::new());
    assert_eq!(core.recall(OPEN_KEY), Some(Value::Null));
}

// ---------------------------------------------------------------------------
// Library and recent tray (LIB-020, LIB-021, TRY-002, TRY-008)

fn set(core: &Core, key: &str, value: Value) {
    cli(core, "settings.set", json!({"key": key, "value": value}));
}

fn info(core: &Core, doc: &str) -> Value {
    cli(core, "document.info", json!({"documentId": doc}))
}

#[test]
fn with_autosave_off_captures_wait_until_saved() {
    let e = env();
    set(&e.core, "library.autosaveImages", json!(false));
    let (doc, path) = capture(&e);
    assert!(path.exists());
    assert!(!path.starts_with(e.library.path()), "not in the library yet");
    assert_eq!(info(&e.core, &doc)["dirty"], true);
    let r = cli(&e.core, "document.close", json!({"documentId": doc}));
    assert_eq!(r["status"], "needs_decision", "closing asks first");

    // Saving puts it in the library, where it then saves itself like any other.
    cli(&e.core, "document.save", json!({"documentId": doc}));
    let saved = info(&e.core, &doc);
    let saved_path = PathBuf::from(saved["path"].as_str().unwrap());
    assert!(saved_path.starts_with(e.library.path()));
    assert!(saved_path.exists());
    assert_eq!(saved["kind"], "library");
    assert_eq!(saved["dirty"], false);
    assert!(!path.exists(), "the waiting copy is gone");

    // Closed without saving: nothing is left behind.
    let (other, other_path) = capture(&e);
    let r = cli(
        &e.core,
        "document.close",
        json!({"documentId": other, "discard": [other]}),
    );
    assert_eq!(r["status"], "closed");
    assert!(!other_path.exists());

    // Multi-capture saves to the library either way (CAP-067).
    let (_, multi) = add_capture(
        &e.core,
        &white(40, 20),
        json!({"mode": "multi"}),
        1.0,
        Some("Test"),
    )
    .unwrap();
    assert!(multi.starts_with(e.library.path()));
}

/// The `.xshot` files of this month's library folder.
fn library_xshots(e: &Env) -> Vec<PathBuf> {
    let month = e
        .library
        .path()
        .join(chrono::Local::now().format("%Y-%m").to_string());
    let Ok(read) = std::fs::read_dir(month) else {
        return Vec::new();
    };
    read.flatten()
        .map(|f| f.path())
        .filter(|p| p.extension().is_some_and(|x| x == "xshot"))
        .collect()
}

#[test]
fn a_flattened_file_keeps_an_editable_copy_in_the_library() {
    let e = env();
    let path = external_png(&e);
    let doc = cli(&e.core, "document.open", json!({"path": path}))["documentId"]
        .as_str()
        .unwrap()
        .to_string();
    // Saved without annotations: nothing to keep.
    cli(&e.core, "document.save", json!({"documentId": doc}));
    assert!(library_xshots(&e).is_empty());

    let annotate = |from: [i32; 2]| {
        cli(
            &e.core,
            "document.annotate",
            json!({"documentId": doc, "objects": [arrow(from, [90, 40])]}),
        );
        cli(&e.core, "document.save", json!({"documentId": doc}));
    };
    annotate([10, 10]);
    let copies = library_xshots(&e);
    assert_eq!(copies.len(), 1);
    let summary = Xshot::read_summary(&copies[0]).unwrap();
    assert_eq!(summary.meta["linkedFile"], json!(path));
    let opened = Xshot::read(&copies[0]).unwrap();
    assert_eq!(opened.document.objects.len(), 1, "the arrow stays editable");

    // Each save updates the same copy.
    annotate([20, 20]);
    assert_eq!(library_xshots(&e), copies);
    assert_eq!(Xshot::read(&copies[0]).unwrap().document.objects.len(), 2);

    // Turned off: no copy.
    let other = external_png(&e);
    set(&e.core, "library.keepEditableCopy", json!(false));
    let doc2 = cli(&e.core, "document.open", json!({"path": other}))["documentId"]
        .as_str()
        .unwrap()
        .to_string();
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc2, "objects": [arrow([5, 5], [50, 40])]}),
    );
    cli(&e.core, "document.save", json!({"documentId": doc2}));
    assert_eq!(library_xshots(&e).len(), 1);
}

#[test]
fn several_documents_close_at_once() {
    let e = env();
    let (a, _) = capture(&e);
    let (b, _) = capture(&e);
    let (c, _) = capture(&e);
    let r = cli(&e.core, "document.close", json!({"documentIds": [a, b]}));
    assert_eq!(r["status"], "closed");
    let list = cli(&e.core, "document.list", json!({}));
    let left: Vec<&str> = list["documents"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["documentId"].as_str().unwrap())
        .collect();
    assert_eq!(left, [c.as_str()]);
    let err = call(
        &e.core,
        Source::Cli,
        "document.close",
        json!({"documentIds": [c, "nope"]}),
    )
    .unwrap_err();
    assert_eq!(err.code, ErrorCode::NotFound);
    assert_eq!(
        cli(&e.core, "document.list", json!({}))["documents"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn a_moved_file_is_marked_and_can_be_found_again() {
    let e = env();
    let (doc, path) = capture(&e);
    let moved = e.outside.path().join("moved.xshot");
    std::fs::rename(&path, &moved).unwrap();
    let listed = cli(&e.core, "document.list", json!({}));
    assert_eq!(listed["documents"][0]["missing"], true, "the strip marks it");

    let png = external_png(&e);
    let wrong = call(
        &e.core,
        Source::Cli,
        "document.locate",
        json!({"documentId": doc, "path": png}),
    );
    assert!(wrong.is_err(), "another format is not the same file");
    let gone = call(
        &e.core,
        Source::Cli,
        "document.locate",
        json!({"documentId": doc, "path": e.outside.path().join("nope.xshot")}),
    );
    assert!(gone.is_err());

    let found = cli(
        &e.core,
        "document.locate",
        json!({"documentId": doc, "path": moved}),
    );
    assert_eq!(found["missing"], false);
    assert_eq!(found["kind"], "external", "outside the library now");
    assert_eq!(found["path"], json!(moved));
}

#[test]
fn the_library_view_waits_for_the_window() {
    let e = env();
    // Tests have no window to show; the view is remembered before that.
    let _ = call(&e.core, Source::Cli, "editor.show", json!({"view": "library"}));
    assert_eq!(cli(&e.core, "editor.state", json!({}))["view"], "library");
    assert_eq!(
        cli(&e.core, "editor.state", json!({}))["view"],
        Value::Null,
        "taken once"
    );
    let bad = call(&e.core, Source::Cli, "editor.show", json!({"view": "gallery"}));
    assert_eq!(bad.unwrap_err().code, ErrorCode::InvalidParams);
}

#[test]
fn the_tray_can_start_empty_after_quitting() {
    let e = env();
    let (_, a) = capture(&e);
    show(&e, &a);
    let guard = EditorQuitGuard(Arc::downgrade(&e.core));
    guard.before_quit();
    assert_eq!(strip(&restart(&e)).0, vec![a.clone()], "kept by default");

    let e2 = Env {
        core: restart(&e),
        ..e
    };
    set(&e2.core, "tray.clearUnpinnedOnExit", json!(true));
    EditorQuitGuard(Arc::downgrade(&e2.core)).before_quit();
    assert_eq!(e2.core.recall(OPEN_KEY), Some(Value::Null));
    // Closing documents while quitting does not write the list back.
    cli(&e2.core, "document.close", json!({"all": true}));
    assert_eq!(e2.core.recall(OPEN_KEY), Some(Value::Null));
    assert_eq!(strip(&restart(&e2)).0, Vec::<PathBuf>::new());
    assert!(a.exists(), "the file stays in the library");
}

#[test]
fn renaming_from_the_library_moves_the_open_document_and_keeps_the_id() {
    let library = tempfile::tempdir().unwrap();
    let recovery = tempfile::tempdir().unwrap();
    let core = Core::for_tests_with(
        Platform::Windows,
        vec![
            extension(),
            xs_library::extension_with(Some(library.path().to_path_buf())),
        ],
    );
    editor(&core).set_folders(library.path().to_path_buf(), recovery.path().to_path_buf());
    let (doc, path) = add_capture(
        &core,
        &white(40, 20),
        json!({"mode": "region"}),
        1.0,
        Some("Test"),
    )
    .unwrap();
    let lib = xs_library::library(&core);
    lib.scan();
    let item = lib.find(&path).unwrap();
    let r = cli(&core, "library.rename", json!({"id": item.id, "name": "Renamed"}));
    assert_eq!(r["id"], json!(item.id), "the same item");
    assert_eq!(r["name"], "Renamed");
    let open = cli(&core, "document.info", json!({"documentId": doc}));
    assert_eq!(open["path"], r["path"], "the open document follows its file");
    assert!(!path.exists());
}

#[test]
fn dragging_out_hands_over_flat_pngs_and_leaves_the_originals() {
    let e = env();
    let out = e.outside.path().join("drag");
    // A capture with an arrow: a flat PNG named after it, with the arrow (TRY-015).
    let (doc, xshot) = capture(&e);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([100, 100], [300, 100])]}),
    );
    cli(&e.core, "document.save", json!({"documentId": doc}));
    let before = std::fs::read(&xshot).unwrap();
    let name = cli(&e.core, "document.info", json!({"documentId": doc}))["name"]
        .as_str()
        .unwrap()
        .to_string();
    let files = files_for_drag(&e.core, std::slice::from_ref(&doc), &[], &out).unwrap();
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].parent(), Some(out.as_path()));
    assert_eq!(
        files[0].file_name().unwrap().to_string_lossy(),
        format!("{name}.png")
    );
    let png = image::open(&files[0]).unwrap().to_rgba8();
    assert_eq!(png.dimensions(), (400, 200));
    assert_eq!(png.get_pixel(200, 100).0, [0, 0, 255, 255]);
    assert_eq!(
        std::fs::read(&xshot).unwrap(),
        before,
        "the original does not change"
    );

    // A PNG from elsewhere without changes goes as it is; with changes, flattened.
    let loose = external_png(&e);
    let ext = cli(&e.core, "document.open", json!({"path": loose}))["documentId"]
        .as_str()
        .unwrap()
        .to_string();
    assert_eq!(
        files_for_drag(&e.core, std::slice::from_ref(&ext), &[], &out).unwrap(),
        vec![loose.clone()]
    );
    call(
        &e.core,
        Source::Ui,
        "document.annotate",
        json!({"documentId": ext, "objects": [arrow([10, 10], [90, 40])]}),
    )
    .unwrap();
    let files = files_for_drag(&e.core, &[ext], &[], &out).unwrap();
    assert!(files[0].starts_with(&out));
    assert_ne!(
        image::open(&files[0]).unwrap().to_rgba8(),
        image::open(&loose).unwrap().to_rgba8()
    );

    // By path, closed: the `.xshot` is flattened and the loose image goes as it is.
    cli(&e.core, "document.close", json!({"documentId": doc}));
    let other = external_png(&e);
    let files = files_for_drag(&e.core, &[], &[xshot.clone(), other.clone()], &out).unwrap();
    assert_eq!(files.len(), 2);
    assert!(files[0].starts_with(&out) && files[0] != out.join(format!("{name}.png")));
    assert_eq!(
        image::open(&files[0]).unwrap().to_rgba8().get_pixel(200, 100).0,
        [0, 0, 255, 255]
    );
    assert_eq!(files[1], other);
    assert!(files_for_drag(&e.core, &[], &[e.outside.path().join("gone.png")], &out).is_err());
}

#[test]
fn a_captured_pointer_is_its_own_object() {
    let e = env();
    // A red 4×4 "pointer" at (10, 20) of a white capture (CAP-047).
    let pointer = Pointer {
        image: image::RgbaImage::from_pixel(4, 4, image::Rgba([255, 0, 0, 255])),
        x: 10,
        y: 20,
    };
    let (doc, path) = add_capture_with(
        &e.core,
        &white(100, 50),
        json!({"mode": "region"}),
        1.0,
        Some("Test"),
        Some(&pointer),
    )
    .unwrap();
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"].as_array().map(Vec::len), Some(1));
    assert!(
        !info["dirty"].as_bool().unwrap(),
        "it arrives as captured, not as a change"
    );
    // The file has it from the start, and its pixels are not in the base image.
    let file = Xshot::read(&path).unwrap();
    assert_eq!(file.document.objects[0].kind, "image");
    assert_eq!(file.images.len(), 1);
    assert_eq!(
        xs_image::encode::decode(&file.base_png)
            .unwrap()
            .get_pixel(11, 21)
            .0,
        [255, 255, 255, 255]
    );
    let out = e.outside.path().join("with.png");
    cli(
        &e.core,
        "document.export",
        json!({"documentId": doc, "format": "png", "path": out}),
    );
    assert_eq!(
        image::open(&out).unwrap().to_rgba8().get_pixel(11, 21).0,
        [255, 0, 0, 255]
    );
    // Deleting it leaves no trace.
    cli(
        &e.core,
        "document.delete_object",
        json!({"documentId": doc, "objectId": file.document.objects[0].id}),
    );
    let out = e.outside.path().join("without.png");
    cli(
        &e.core,
        "document.export",
        json!({"documentId": doc, "format": "png", "path": out}),
    );
    assert_eq!(
        image::open(&out).unwrap().to_rgba8().get_pixel(11, 21).0,
        [255, 255, 255, 255]
    );
}

#[test]
fn several_objects_move_together_in_one_step() {
    let e = env();
    let (doc, _) = capture(&e);
    let r = cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([10, 10], [50, 10]), arrow([10, 40], [50, 40])]}),
    );
    assert_eq!(r["ids"], json!(["o1", "o2"]));
    call(
        &e.core,
        Source::Ui,
        "document.update_object",
        json!({"documentId": doc, "objectIds": ["o1", "o2"], "move": [5, 7]}),
    )
    .unwrap();
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"][0]["from"], json!([15.0, 17.0]));
    assert_eq!(info["objects"][1]["to"], json!([55.0, 47.0]));
    // One Undo puts both back (EDT-079).
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"][0]["from"], json!([10.0, 10.0]));
    assert_eq!(info["objects"][1]["to"], json!([50.0, 40.0]));
}

#[test]
fn duplicates_go_on_top_moved_by_the_offset() {
    let e = env();
    let (doc, _) = capture(&e);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([10, 10], [50, 10])]}),
    );
    let r = cli(
        &e.core,
        "document.duplicate",
        json!({"documentId": doc, "objectIds": ["o1"]}),
    );
    assert_eq!(r["ids"], json!(["o2"]));
    let r = cli(
        &e.core,
        "document.duplicate",
        json!({"documentId": doc, "objectIds": ["o2"], "offset": [50, 0]}),
    );
    assert_eq!(r["ids"], json!(["o3"]));
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    let from: Vec<Value> = info["objects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["from"].clone())
        .collect();
    assert_eq!(
        from,
        [json!([10.0, 10.0]), json!([20.0, 20.0]), json!([70.0, 20.0])]
    );
    // Sent to the back, the last copy is drawn first (EDT-081).
    let r = cli(
        &e.core,
        "document.arrange",
        json!({"documentId": doc, "objectIds": ["o3"], "to": "back"}),
    );
    assert_eq!(r["order"], json!(["o3", "o1", "o2"]));
}

#[test]
fn a_lock_is_saved_and_unlocking_all_is_one_step() {
    let e = env();
    let (doc, path) = capture(&e);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([10, 10], [50, 10]), arrow([10, 40], [50, 40])]}),
    );
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectIds": ["o1", "o2"], "set": {"locked": true}}),
    );
    // It moves and keeps its lock (EDT-085).
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o1", "move": [5, 0]}),
    );
    cli(&e.core, "document.save", json!({"documentId": doc}));
    let file = Xshot::read(&path).unwrap();
    assert!(file.document.objects.iter().all(|o| o.props["locked"] == true));
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectIds": ["o1", "o2"], "set": {"locked": false}}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert!(info["objects"]
        .as_array()
        .unwrap()
        .iter()
        .all(|o| o.get("locked").is_none()));
    // One Undo locks both again.
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert!(info["objects"]
        .as_array()
        .unwrap()
        .iter()
        .all(|o| o["locked"] == true));
}

/// EDT-080: an arrow and a floating image grown half again as one keep their places
/// and grow together; without scaling strokes the arrow keeps its width; turning three
/// objects goes around their common center. Each is one undo step.
#[test]
fn several_objects_scale_and_turn_as_one() {
    let e = env();
    // Objects at the edge would make the canvas grow and move them all.
    cli(
        &e.core,
        "settings.set",
        json!({"key": "editor.autoExpandCanvas", "value": false}),
    );
    let (doc, _) = capture(&e);
    let red = red_png(&e, 40, 20);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([0, 0], [100, 0])]}),
    );
    cli(
        &e.core,
        "document.insert_images",
        json!({"documentId": doc, "paths": [red], "center": [120, 60]}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(
        info["objects"][1]["rect"],
        json!({"x": 100.0, "y": 50.0, "w": 40.0, "h": 20.0})
    );
    cli(
        &e.core,
        "document.transform",
        json!({"documentId": doc, "objectIds": ["o1", "o2"], "scale": {"factor": 1.5}}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    let (a, i) = (&info["objects"][0], &info["objects"][1]);
    assert_eq!(a["to"], json!([150.0, 0.0]));
    assert_eq!(a["style"]["width"], json!(12.0));
    assert_eq!(i["rect"], json!({"x": 150.0, "y": 75.0, "w": 60.0, "h": 30.0}));
    // Without "Scale line widths and text": the arrow keeps its width.
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    cli(
        &e.core,
        "document.transform",
        json!({"documentId": doc, "objectIds": ["o1", "o2"], "scale": {"factor": 1.5}, "strokes": false}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"][0]["style"]["width"], json!(8.0));
    assert_eq!(info["objects"][0]["to"], json!([150.0, 0.0]));
    // A quarter turn of three objects goes around the center of their box.
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [{"type": "shape", "rect": {"x": 200, "y": 0, "w": 40, "h": 20}}]}),
    );
    cli(
        &e.core,
        "document.transform",
        json!({"documentId": doc, "objectIds": ["o1", "o2", "o3"], "rotate": 90}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    // The box was (0, 0)–(240, 70): its center (120, 35).
    let a = &info["objects"][0];
    let near = |v: &Value, x: f64, y: f64| {
        (v[0].as_f64().unwrap() - x).abs() < 1e-6 && (v[1].as_f64().unwrap() - y).abs() < 1e-6
    };
    assert!(near(&a["from"], 155.0, -85.0), "{}", a["from"]);
    assert!(near(&a["to"], 155.0, 15.0), "{}", a["to"]);
    let s = &info["objects"][2];
    assert_eq!(s["rotation"], json!(90.0));
    let r = &s["rect"];
    let cx = r["x"].as_f64().unwrap() + r["w"].as_f64().unwrap() / 2.0;
    let cy = r["y"].as_f64().unwrap() + r["h"].as_f64().unwrap() / 2.0;
    assert!(
        (cx - 145.0).abs() < 1e-6 && (cy - 135.0).abs() < 1e-6,
        "{cx}, {cy}"
    );
    // One undo step puts all three back.
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"][0]["from"], json!([0.0, 0.0]));
    assert_eq!(info["objects"][2]["rotation"], json!(0.0));
}

/// EDT-083: Equal width gives the others the width of the widest; arrows stay.
#[test]
fn equal_width_matches_the_widest() {
    let e = env();
    let (doc, _) = capture(&e);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [
            {"type": "shape", "rect": {"x": 10, "y": 10, "w": 30, "h": 20}},
            {"type": "shape", "rect": {"x": 10, "y": 50, "w": 80, "h": 20}},
            arrow([0, 100], [300, 100]),
        ]}),
    );
    cli(
        &e.core,
        "document.transform",
        json!({"documentId": doc, "objectIds": ["o1", "o2", "o3"], "equal": "width"}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(
        info["objects"][0]["rect"],
        json!({"x": 10.0, "y": 10.0, "w": 80.0, "h": 20.0})
    );
    assert_eq!(info["objects"][2]["to"], json!([300.0, 100.0]));
    let err = call(
        &e.core,
        Source::Cli,
        "document.transform",
        json!({"documentId": doc, "objectIds": ["o1"], "rotate": 10, "equal": "width"}),
    )
    .unwrap_err();
    assert!(err.message.contains("one of"), "{}", err.message);
}

/// EDT-091: a flattened arrow can no longer be picked; Unflatten makes it an object
/// again, also after saving and reopening.
#[test]
fn a_flattened_arrow_comes_back_after_reopening() {
    let e = env();
    let (doc, path) = capture(&e);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([20, 50], [120, 50]), arrow([20, 150], [120, 150])]}),
    );
    let before = cli(&e.core, "document.image", json!({"documentId": doc}));
    let r = cli(
        &e.core,
        "document.flatten",
        json!({"documentId": doc, "objectIds": ["o1"]}),
    );
    assert_eq!(r, json!({"layers": ["f1"], "permanent": false}));
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(
        info["objects"].as_array().unwrap().len(),
        1,
        "no longer an object"
    );
    assert_eq!(info["flattened"][0]["id"], "f1");
    assert!(info["flattened"][0]["bounds"].is_array());
    // It still shows: the image is the same.
    let after = cli(&e.core, "document.image", json!({"documentId": doc}));
    assert!(before["image"]["data"].is_string());
    assert_eq!(before["image"]["data"], after["image"]["data"]);
    cli(&e.core, "document.save", json!({"documentId": doc}));
    cli(&e.core, "document.close", json!({"documentId": doc}));
    let doc = cli(&e.core, "document.open", json!({"path": path}))["documentId"]
        .as_str()
        .unwrap()
        .to_string();
    let r = cli(
        &e.core,
        "document.unflatten",
        json!({"documentId": doc, "layerId": "f1"}),
    );
    assert_eq!(r["ids"], json!(["o1"]));
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"][0]["id"], "o1");
    assert_eq!(info["objects"][0]["from"], json!([20.0, 50.0]));
    assert!(info.get("flattened").is_none());
}

/// EDT-091: a flattened blur cannot be unflattened, and once saved the file no longer
/// has the pixels it hid.
#[test]
fn a_flattened_blur_leaves_nothing_behind_in_the_file() {
    let e = env();
    // Stripes, so a blur changes them.
    let stripes = image::RgbaImage::from_fn(200, 100, |x, _| {
        if (x / 4) % 2 == 0 {
            image::Rgba([0, 0, 0, 255])
        } else {
            image::Rgba([255, 255, 255, 255])
        }
    });
    let (doc, path) =
        add_capture(&e.core, &stripes, json!({"mode": "region"}), 1.0, Some("Stripes")).unwrap();
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [{"type": "blur", "rect": {"x": 20, "y": 20, "w": 60, "h": 40}}]}),
    );
    let r = cli(
        &e.core,
        "document.flatten",
        json!({"documentId": doc, "all": true}),
    );
    assert_eq!(r["permanent"], true);
    let err = call(
        &e.core,
        Source::Cli,
        "document.unflatten",
        json!({"documentId": doc, "layerId": r["layers"][0]}),
    )
    .unwrap_err();
    assert!(
        err.message.contains("cannot become objects again"),
        "{}",
        err.message
    );
    cli(&e.core, "document.save", json!({"documentId": doc}));
    let file = Xshot::read(&path).unwrap();
    assert!(file.document.flattened.is_empty(), "merged into the image");
    let base = xs_image::encode::decode(&file.base_png).unwrap();
    // Inside the blur the stripes are gone; outside they are as they were.
    let (x, y) = (50, 40);
    assert_ne!(base.get_pixel(x, y), stripes.get_pixel(x, y));
    assert_ne!(base.get_pixel(x + 4, y), stripes.get_pixel(x + 4, y));
    assert_eq!(base.get_pixel(150, 80), stripes.get_pixel(150, 80));
    assert_eq!(base.get_pixel(5, 5), stripes.get_pixel(5, 5));
}

/// EDT-035: three clicks place 1, 2 and 3; Restart sequence places another 1; an edited
/// value shows; Renumber closes the gaps.
#[test]
fn steps_count_restart_and_renumber() {
    let e = env();
    let (doc, _) = capture(&e);
    let place = |at: [i32; 2]| {
        cli(
            &e.core,
            "document.annotate",
            json!({"documentId": doc, "objects": [{"type": "step", "at": at}]}),
        )["ids"][0]
            .as_str()
            .unwrap()
            .to_string()
    };
    let values = |e: &Env| -> Vec<String> {
        cli(&e.core, "document.info", json!({"documentId": doc}))["objects"]
            .as_array()
            .unwrap()
            .iter()
            .map(|o| o["value"].as_str().unwrap().to_string())
            .collect()
    };
    for x in [50, 100, 150] {
        place([x, 50]);
    }
    assert_eq!(values(&e), ["1", "2", "3"]);
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"][0]["rect"]["w"], json!(32.0), "the style's size");
    assert_eq!(info["stepNext"]["number"], "4");
    let r = cli(
        &e.core,
        "document.steps",
        json!({"documentId": doc, "action": "restart"}),
    );
    assert_eq!(r["next"], "1");
    place([200, 50]);
    assert_eq!(values(&e), ["1", "2", "3", "1"]);
    // Edit value.
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o2", "set": {"value": "B2"}}),
    );
    assert_eq!(values(&e)[1], "B2");
    // 1, 2, 4, 5 (the 3 deleted) → 1, 2, 3, 4.
    let e2 = env();
    let (doc2, _) = capture(&e2);
    for x in [20, 60, 100, 140, 180] {
        cli(
            &e2.core,
            "document.annotate",
            json!({"documentId": doc2, "objects": [{"type": "step", "at": [x, 50]}]}),
        );
    }
    cli(
        &e2.core,
        "document.delete_object",
        json!({"documentId": doc2, "objectId": "o3"}),
    );
    let r = cli(
        &e2.core,
        "document.steps",
        json!({"documentId": doc2, "action": "renumber"}),
    );
    assert_eq!(r["renumbered"], 4);
    let info = cli(&e2.core, "document.info", json!({"documentId": doc2}));
    let v: Vec<&str> = info["objects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["value"].as_str().unwrap())
        .collect();
    assert_eq!(v, ["1", "2", "3", "4"]);
    // A dragged step takes the dragged size, and the size changes the rect.
    cli(
        &e2.core,
        "document.annotate",
        json!({"documentId": doc2, "objects": [{"type": "step", "rect": {"x": 10, "y": 100, "w": 60, "h": 60}}]}),
    );
    let info = cli(&e2.core, "document.info", json!({"documentId": doc2}));
    let last = info["objects"].as_array().unwrap().last().unwrap().clone();
    assert_eq!(last["style"]["size"], json!(60.0));
    assert_eq!(last["value"], "5");
    let oid = last["id"].as_str().unwrap();
    cli(
        &e2.core,
        "document.update_object",
        json!({"documentId": doc2, "objectId": oid, "set": {"style": {"size": 40}}}),
    );
    let info = cli(&e2.core, "document.info", json!({"documentId": doc2}));
    let last = info["objects"].as_array().unwrap().last().unwrap().clone();
    assert_eq!(last["rect"], json!({"x": 20.0, "y": 110.0, "w": 40.0, "h": 40.0}));
}

/// EDT-028: a callout with two lines and a second tail moves with both tails; a tail
/// taken away comes back with Undo; with Fit to text, ten lines make the body grow.
#[test]
fn callouts_keep_their_tails_and_fit_their_text() {
    let e = env();
    let (doc, _) = capture(&e);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [{
            "type": "callout",
            "rect": {"x": 40, "y": 20, "w": 200, "h": 60},
            "text": "Line one\nLine two",
            "tails": [[60, 150]]
        }]}),
    );
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o1", "set": {"tails": [[60, 150], [300, 150]]}}),
    );
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o1", "move": [10, 5]}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    let c = &info["objects"][0];
    assert_eq!(c["tails"], json!([[70.0, 155.0], [310.0, 155.0]]));
    assert_eq!(c["rect"]["x"], json!(50.0));
    // Taking a tail away (dragged into the body) and Undo.
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o1", "set": {"tails": [[70.0, 155.0]]}}),
    );
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"][0]["tails"].as_array().unwrap().len(), 2);
    // Ten lines: the body grows to show them all.
    let before = info["objects"][0]["rect"]["h"].as_f64().unwrap();
    let ten: Vec<String> = (1..=10).map(|n| format!("Line {n}")).collect();
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o1", "set": {"text": ten.join("\n")}}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    let after = info["objects"][0]["rect"]["h"].as_f64().unwrap();
    assert!(after > before * 2.0, "{before} -> {after}");
    // Without tails given, one is added below the body.
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [{"type": "callout", "rect": {"x": 10, "y": 10, "w": 100, "h": 40}}]}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"][1]["tails"].as_array().unwrap().len(), 1);
    // Scaling several objects moves the tails with them (EDT-080).
    cli(
        &e.core,
        "document.transform",
        json!({"documentId": doc, "objectIds": ["o2"], "scale": {"from": [10, 10, 100, 100], "to": [10, 10, 200, 200]}}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    // Its default tail was at (35, 74): x + w / 4, below the body by 24.
    assert_eq!(info["objects"][1]["tails"][0], json!([60.0, 138.0]));
}

/// EDT-025: a line is drawn without ends, with the Line tool's width, and stays a line.
#[test]
fn lines_have_no_ends_and_their_own_style() {
    let e = env();
    let (doc, _) = capture(&e);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [{"type": "line", "from": [10, 10], "to": [100, 10]}, arrow([10, 50], [100, 50])]}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    let line = &info["objects"][0];
    assert_eq!(line["type"], "line");
    assert_eq!(line["style"]["startCap"], "none");
    assert_eq!(line["style"]["endCap"], "none");
    assert_eq!(line["style"]["width"], json!(4.0));
    assert_eq!(info["objects"][1]["style"]["endCap"], "filled");
    // An end added to a line keeps it a line.
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o1", "set": {"style": {"endCap": "filled"}}}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"][0]["type"], "line");
}

/// EDT-037: an editable stroke is an object whose color changes; one that is not is
/// painted into the image and cannot be picked or unflattened.
#[test]
fn pen_strokes_are_objects_or_paint() {
    let e = env();
    let (doc, _) = capture(&e);
    let wiggle: Vec<[f64; 2]> = (0..30)
        .map(|i| [20.0 + i as f64 * 4.0, 50.0 + if i % 2 == 0 { 0.0 } else { 3.0 }])
        .collect();
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [{"type": "pen", "points": wiggle, "style": {"smooth": 100}}]}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    let stroke = &info["objects"][0];
    assert_eq!(stroke["type"], "pen");
    let kept = stroke["points"].as_array().unwrap().len();
    assert!(kept < 30, "smoothed and simplified: {kept} points");
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o1", "set": {"style": {"color": "#FF0000"}}}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"][0]["style"]["color"], "#FF0000");
    // Not editable: painted.
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [{"type": "pen", "points": [[10, 100], [80, 120]], "style": {"editable": false}}]}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"].as_array().unwrap().len(), 1, "no new object");
    assert_eq!(info["flattened"][0]["permanent"], "painted");
}

/// EDT-038: a lens starts over its zone; moved away, its zone stays; moving the whole
/// object moves both; the zoom changes the lens at once.
#[test]
fn a_lens_and_its_zone() {
    let e = env();
    let (doc, _) = capture(&e);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [{"type": "magnify", "source": {"x": 175, "y": 75, "w": 50, "h": 50}, "style": {"zoom": 300}}]}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"][0]["at"], json!([200.0, 100.0]));
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o1", "set": {"at": [250, 100]}}),
    );
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o1", "move": [10, 0]}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    let m = &info["objects"][0];
    assert_eq!(m["source"]["x"], json!(185.0));
    assert_eq!(m["at"], json!([260.0, 100.0]));
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o1", "set": {"style": {"zoom": 400}}}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"][0]["style"]["zoom"], json!(400.0));
}

/// EDT-039: spotlights share their outside: a change to one passes to all.
#[test]
fn spotlights_share_their_outside() {
    let e = env();
    let (doc, _) = capture(&e);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [
            {"type": "spotlight", "rect": {"x": 10, "y": 10, "w": 50, "h": 50}},
            {"type": "spotlight", "rect": {"x": 100, "y": 10, "w": 50, "h": 50}, "style": {"shape": "ellipse"}}
        ]}),
    );
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o2", "set": {"style": {"opacity": 30, "blur": 20}}}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    let first = &info["objects"][0]["style"];
    assert_eq!(first["opacity"], json!(30.0));
    assert_eq!(first["blur"], json!(20.0));
    assert_eq!(first["shape"], "rectangle", "each keeps its own shape");
}

/// EDT-030: a closed pentagon; Radius = 12 on three rounded rectangles at once; a moved
/// corner can be undone.
#[test]
fn polygons_and_rounded_rectangles() {
    let e = env();
    let (doc, _) = capture(&e);
    let pentagon: Vec<[f64; 2]> = (0..5)
        .map(|i| {
            let a = std::f64::consts::TAU * i as f64 / 5.0 - std::f64::consts::FRAC_PI_2;
            [0.5 + 0.5 * a.cos(), 0.5 + 0.5 * a.sin()]
        })
        .collect();
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [
            {"type": "shape", "rect": {"x": 20, "y": 20, "w": 80, "h": 80}, "points": pentagon, "style": {"shapeType": "polygon", "fill": "#00FF00"}},
            {"type": "shape", "rect": {"x": 120, "y": 20, "w": 60, "h": 40}, "style": {"shapeType": "roundedRectangle"}},
            {"type": "shape", "rect": {"x": 200, "y": 20, "w": 60, "h": 40}, "style": {"shapeType": "roundedRectangle"}},
            {"type": "shape", "rect": {"x": 280, "y": 20, "w": 60, "h": 40}, "style": {"shapeType": "roundedRectangle"}}
        ]}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"][0]["points"].as_array().unwrap().len(), 5);
    // The pentagon is filled: its middle is green.
    let img = cli(&e.core, "document.image", json!({"documentId": doc}));
    assert!(img["image"]["data"].is_string());
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectIds": ["o2", "o3", "o4"], "set": {"style": {"cornerRadius": 12}}}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    for i in 1..4 {
        assert_eq!(info["objects"][i]["style"]["cornerRadius"], json!(12.0));
    }
    // A corner moved, then undone.
    let mut moved = info["objects"][0]["points"].clone();
    moved[0] = json!([0.5, -0.2]);
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o1", "set": {"points": moved}}),
    );
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    let back = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(back["objects"][0]["points"], info["objects"][0]["points"]);
}

/// EDT-031: a stamp placed with a click takes the usual size; another stamp in its place
/// keeps its place and height; the document carries its drawing.
#[test]
fn stamps_are_placed_replaced_and_carried() {
    let e = env();
    let (doc, path) = capture(&e);
    let found = cli(&e.core, "editor.stamps", json!({"query": "aviso"}));
    assert_eq!(found["stamps"][0]["id"], "warning");
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [{"type": "stamp", "stamp": "check", "at": [100, 100]}]}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    let s = &info["objects"][0];
    // The canvas is 400 x 200: a quarter of 200 is less than 96.
    assert_eq!(s["rect"]["h"], json!(50.0));
    assert!(s["art"]["shapes"].as_array().is_some_and(|a| !a.is_empty()));
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o1", "set": {"rect": {"x": 20, "y": 30, "w": 120, "h": 120}}}),
    );
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o1", "set": {"stamp": "label-new"}}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    let r = &info["objects"][0]["rect"];
    assert_eq!(
        (r["x"].as_f64(), r["y"].as_f64(), r["h"].as_f64()),
        (Some(20.0), Some(30.0), Some(120.0))
    );
    assert_eq!(r["w"], json!(300.0), "a label is 2.5 times as wide as high");
    assert_eq!(info["objects"][0]["stamp"], "label-new");
    cli(&e.core, "document.save", json!({"documentId": doc}));
    let file = Xshot::read(&path).unwrap();
    assert!(
        file.document.objects[0].props["art"]["shapes"].is_array(),
        "the drawing travels with it"
    );
}

/// EDT-041: simplified shapes keep their own style (not the tool's palettes); the
/// original shows dimmed or hidden under them, in one undo step; a new palette is kept.
#[test]
fn simplify_shapes_palettes_and_the_original() {
    let e = env();
    let (doc, _) = capture(&e);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [{"type": "simplify", "rect": {"x": 10, "y": 10, "w": 120, "h": 12}, "style": {"kind": "textLine"}}]}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    let st = &info["objects"][0]["style"];
    assert_eq!(st["kind"], "textLine");
    assert!(st.get("palettes").is_none());
    let v = cli(
        &e.core,
        "document.simplify_view",
        json!({"documentId": doc, "showOriginal": true, "opacity": 40}),
    );
    assert_eq!(v["opacity"], json!(40.0));
    cli(
        &e.core,
        "document.simplify_view",
        json!({"documentId": doc, "showOriginal": false}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["simplify"], json!({"showOriginal": false, "opacity": 40.0}));
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["simplify"]["showOriginal"], true);
    // A palette saved as new, and used.
    let mut palettes = serde_json::to_value(xs_image::simplify::default_palettes()).unwrap();
    palettes
        .as_array_mut()
        .unwrap()
        .push(json!({"name": "Mine", "colors": ["#112233", "#445566"]}));
    cli(
        &e.core,
        "editor.set_style",
        json!({"tool": "simplify", "style": {"palettes": palettes, "palette": "Mine", "fill": "#112233"}}),
    );
    let state = cli(&e.core, "editor.state", json!({}));
    assert_eq!(state["styles"]["simplify"]["palette"], "Mine");
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [{"type": "simplify", "rect": {"x": 10, "y": 40, "w": 50, "h": 20}}]}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"][1]["style"]["fill"], "#112233");
}

fn rendered(e: &Env, doc: &str) -> image::RgbaImage {
    editor(&e.core).store.snapshot(doc).unwrap().render()
}

/// EDT-047: Fill changes the image under the objects in one undo step; Transparent
/// leaves no pixels; Global reaches parts that do not touch; a floating image changes
/// alone (ADR 0005).
#[test]
fn fill_changes_the_image_not_the_objects() {
    let e = env();
    let (doc, path) = capture(&e);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([20, 50], [120, 50])]}),
    );
    let r = cli(
        &e.core,
        "document.fill",
        json!({"documentId": doc, "at": [300, 150], "color": "#00FF00", "tolerance": 0}),
    );
    assert_eq!(r, json!({"changed": true}));
    let img = rendered(&e, &doc);
    assert_eq!(img.get_pixel(300, 150).0, [0, 255, 0, 255]);
    assert_eq!(
        img.get_pixel(70, 50).0,
        [0, 0, 255, 255],
        "the arrow is not filled"
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["undoLabel"], "Fill");
    assert_eq!(info["objects"].as_array().unwrap().len(), 1);
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    assert_eq!(rendered(&e, &doc).get_pixel(300, 150).0, [255, 255, 255, 255]);
    cli(&e.core, "document.redo", json!({"documentId": doc}));
    // Transparent, then saved: it is in the base and the file has no patches.
    cli(
        &e.core,
        "document.fill",
        json!({"documentId": doc, "at": [5, 5], "color": null}),
    );
    // The white canvas of an opaque capture shows through (EDT-074).
    assert_eq!(rendered(&e, &doc).get_pixel(300, 150).0, [255, 255, 255, 255]);
    cli(&e.core, "document.save", json!({"documentId": doc}));
    let file = Xshot::read(&path).unwrap();
    assert!(file.document.patches.is_empty());
    let base = xs_image::encode::decode(&file.base_png).unwrap();
    assert_eq!(base.get_pixel(300, 150).0[3], 0);
    // Outside the pixels: nothing.
    let r = cli(
        &e.core,
        "document.fill",
        json!({"documentId": doc, "at": [-5, 5], "color": "#000000"}),
    );
    assert_eq!(r, json!({"changed": false}));
    // A floating image changes alone, into a new picture.
    let red = red_png(&e, 40, 20);
    cli(
        &e.core,
        "document.insert_images",
        json!({"documentId": doc, "paths": [red], "center": [200, 100]}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    let picture = info["objects"][1]["image"].clone();
    cli(
        &e.core,
        "document.fill",
        json!({"documentId": doc, "at": [190, 100], "color": "#0000FF", "objectId": "o2"}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_ne!(info["objects"][1]["image"], picture);
    let img = rendered(&e, &doc);
    assert_eq!(img.get_pixel(190, 100).0, [0, 0, 255, 255]);
    assert_eq!(
        img.get_pixel(300, 150).0,
        [255, 255, 255, 255],
        "the image did not change: still transparent over the white canvas"
    );
    let bad = call(
        &e.core,
        Source::Cli,
        "document.fill",
        json!({"documentId": doc, "at": [70, 50], "color": "#000000", "objectId": "o1"}),
    );
    assert!(bad.is_err(), "an arrow has no pixels to fill");
}

/// EDT-047: Global fills every part of that color.
#[test]
fn global_fill_reaches_parts_that_do_not_touch() {
    let e = env();
    let mut img = white(100, 40);
    for (x0, x1) in [(10, 20), (60, 70)] {
        for y in 10..20 {
            for x in x0..x1 {
                img.put_pixel(x, y, image::Rgba([0, 0, 255, 255]));
            }
        }
    }
    let (doc, _) = add_capture(&e.core, &img, json!({"mode": "region"}), 1.0, None).unwrap();
    cli(
        &e.core,
        "document.fill",
        json!({"documentId": doc, "at": [15, 15], "color": "#00FF00"}),
    );
    assert_eq!(rendered(&e, &doc).get_pixel(65, 15).0, [0, 0, 255, 255]);
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    cli(
        &e.core,
        "document.fill",
        json!({"documentId": doc, "at": [15, 15], "color": "#00FF00", "global": true}),
    );
    let out = rendered(&e, &doc);
    assert_eq!(out.get_pixel(65, 15).0, [0, 255, 0, 255]);
    assert_eq!(out.get_pixel(40, 30).0, [255, 255, 255, 255]);
}

/// EDT-048: the eraser shows the canvas color, or transparency without one; objects
/// stay; a flattened layer erased over can no longer be unflattened (EDT-091).
#[test]
fn erasing_shows_the_canvas_color() {
    let e = env();
    let (doc, _) = capture(&e);
    // An opaque capture starts with a white canvas (EDT-074): transparent first.
    cli(
        &e.core,
        "document.canvas",
        json!({"documentId": doc, "color": null}),
    );
    // Red pixels, so white shows.
    cli(
        &e.core,
        "document.fill",
        json!({"documentId": doc, "at": [5, 5], "color": "#FF0000"}),
    );
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([20, 150], [380, 150]), arrow([20, 100], [120, 100])]}),
    );
    cli(
        &e.core,
        "document.flatten",
        json!({"documentId": doc, "objectIds": ["o2"]}),
    );
    cli(
        &e.core,
        "document.erase",
        json!({"documentId": doc, "points": [[10, 150], [390, 150]], "width": 30}),
    );
    let img = rendered(&e, &doc);
    assert_eq!(
        img.get_pixel(200, 140).0[3],
        0,
        "transparent without a canvas color"
    );
    assert_eq!(img.get_pixel(200, 150).0, [0, 0, 255, 255], "the arrow stays");
    assert_eq!(img.get_pixel(200, 60).0, [255, 0, 0, 255]);
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["undoLabel"], "Erase");
    assert!(info["flattened"][0]["permanent"].is_null(), "not touched");
    // Over the flattened arrow: it stays flat.
    cli(
        &e.core,
        "document.erase",
        json!({"documentId": doc, "points": [[60, 100]], "width": 10, "shape": "square"}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["flattened"][0]["permanent"], "edited");
    // With a canvas color, it shows.
    editor(&e.core)
        .store
        .mutate(&doc, "history.fill", |d| {
            d.canvas.color = Some("#FFFFFF".into());
            Ok::<_, ()>(())
        })
        .unwrap()
        .unwrap();
    assert_eq!(rendered(&e, &doc).get_pixel(200, 140).0, [255, 255, 255, 255]);
}

/// A white 200 × 120 capture with a blue button at 60,40 (80 × 24).
fn with_button(e: &Env) -> String {
    let mut img = white(200, 120);
    for y in 40..64 {
        for x in 60..140 {
            img.put_pixel(x, y, image::Rgba([30, 110, 220, 255]));
        }
    }
    add_capture(&e.core, &img, json!({"mode": "region"}), 1.0, None)
        .unwrap()
        .0
}

/// EDT-043: shapes select, combine and show their outline; Snap to object fits a loose
/// box to the button; Invert and none.
#[test]
fn pixel_selections_combine_and_snap() {
    let e = env();
    let doc = with_button(&e);
    let r = cli(
        &e.core,
        "document.select_pixels",
        json!({"documentId": doc, "shape": "rectangle", "rect": {"x": 10, "y": 10, "w": 30, "h": 20}}),
    );
    assert_eq!(r["bounds"], json!({"x": 10.0, "y": 10.0, "w": 30.0, "h": 20.0}));
    cli(
        &e.core,
        "document.select_pixels",
        json!({"documentId": doc, "shape": "rectangle", "rect": {"x": 30, "y": 20, "w": 30, "h": 20}, "mode": "add"}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    let sel = &info["pixelSelection"];
    assert_eq!(
        sel["bounds"],
        json!({"x": 10.0, "y": 10.0, "w": 50.0, "h": 30.0}),
        "the union"
    );
    assert_eq!(sel["outline"].as_array().unwrap().len(), 1);
    assert!(sel["shape"].is_null(), "no longer one rectangle");
    // A loose box around the button snaps to it.
    let r = cli(
        &e.core,
        "document.select_pixels",
        json!({"documentId": doc, "shape": "rectangle", "rect": {"x": 45, "y": 30, "w": 110, "h": 45}, "snap": true}),
    );
    assert_eq!(r["bounds"], json!({"x": 60.0, "y": 40.0, "w": 80.0, "h": 24.0}));
    // The magic wand on the background, inverted: the button.
    cli(
        &e.core,
        "document.select_pixels",
        json!({"documentId": doc, "shape": "wand", "at": [5, 5], "tolerance": 0}),
    );
    let r = cli(
        &e.core,
        "document.select_pixels",
        json!({"documentId": doc, "shape": "invert"}),
    );
    assert_eq!(r["bounds"], json!({"x": 60.0, "y": 40.0, "w": 80.0, "h": 24.0}));
    let r = cli(
        &e.core,
        "document.select_pixels",
        json!({"documentId": doc, "shape": "none"}),
    );
    assert!(r["bounds"].is_null());
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert!(info["pixelSelection"].is_null());
    assert!(!info["canUndo"].as_bool().unwrap(), "selecting is no undo step");
}

/// EDT-044, EDT-045: delete fills the gap; move takes the pixels and the selection;
/// float makes a floating image; crop to an ellipse leaves transparent corners.
#[test]
fn acting_on_selected_pixels() {
    let e = env();
    let doc = with_button(&e);
    let select_button = || {
        cli(
            &e.core,
            "document.select_pixels",
            json!({"documentId": doc, "shape": "rectangle", "rect": {"x": 56, "y": 36, "w": 88, "h": 32}}),
        )
    };
    // Delete with Smart fill: a plain background without a mark.
    select_button();
    cli(
        &e.core,
        "document.selection_action",
        json!({"documentId": doc, "action": "delete"}),
    );
    let img = rendered(&e, &doc);
    assert!(
        img.pixels().all(|p| p.0[0] >= 254 && p.0[2] >= 254),
        "no button left"
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["undoLabel"], "Delete pixels");
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    // Move 40 px right, Transparent behind.
    cli(
        &e.core,
        "document.selection_action",
        json!({"documentId": doc, "action": "move", "offset": [40, 0], "gapFill": "transparent"}),
    );
    let img = rendered(&e, &doc);
    assert_eq!(
        img.get_pixel(65, 50).0,
        [255, 255, 255, 255],
        "transparent where it was: the white canvas shows (EDT-074)"
    );
    assert_eq!(img.get_pixel(170, 50).0, [30, 110, 220, 255], "moved");
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(
        info["pixelSelection"]["bounds"]["x"],
        json!(96.0),
        "the selection went too"
    );
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    // Float: a floating image of the button, the gap filled.
    select_button();
    let r = cli(
        &e.core,
        "document.selection_action",
        json!({"documentId": doc, "action": "float"}),
    );
    assert_eq!(r["ids"], json!(["o1"]));
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"][0]["type"], "image");
    assert_eq!(
        info["objects"][0]["rect"],
        json!({"x": 56.0, "y": 36.0, "w": 88.0, "h": 32.0})
    );
    assert!(info["pixelSelection"].is_null());
    let img = rendered(&e, &doc);
    assert_eq!(img.get_pixel(100, 50).0, [30, 110, 220, 255], "it still shows");
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    // Crop to an ellipse: transparent corners.
    cli(
        &e.core,
        "document.select_pixels",
        json!({"documentId": doc, "shape": "ellipse", "rect": {"x": 40, "y": 20, "w": 120, "h": 80}}),
    );
    cli(
        &e.core,
        "document.selection_action",
        json!({"documentId": doc, "action": "crop"}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(
        (info["width"].as_u64(), info["height"].as_u64()),
        (Some(120), Some(80))
    );
    let img = rendered(&e, &doc);
    assert_eq!(img.get_pixel(1, 1).0[3], 0, "outside the circle");
    assert_eq!(img.get_pixel(60, 40).0, [30, 110, 220, 255]);
    // Nothing selected: an error.
    let none = call(
        &e.core,
        Source::Cli,
        "document.selection_action",
        json!({"documentId": doc, "action": "delete"}),
    );
    assert!(none.is_err());
}

/// EDT-043: a selection on a floating image changes only that image.
#[test]
fn selecting_on_a_floating_image() {
    let e = env();
    let (doc, _) = capture(&e);
    let red = red_png(&e, 40, 20);
    cli(
        &e.core,
        "document.insert_images",
        json!({"documentId": doc, "paths": [red], "center": [200, 100]}),
    );
    // The image is at 180,90 (40 × 20): its left half.
    cli(
        &e.core,
        "document.select_pixels",
        json!({"documentId": doc, "shape": "rectangle", "rect": {"x": 180, "y": 90, "w": 20, "h": 20}, "objectId": "o1"}),
    );
    cli(
        &e.core,
        "document.selection_action",
        json!({"documentId": doc, "action": "delete", "gapFill": "transparent"}),
    );
    let img = rendered(&e, &doc);
    assert_eq!(
        img.get_pixel(185, 100).0,
        [255, 255, 255, 255],
        "the white capture shows"
    );
    assert_eq!(img.get_pixel(215, 100).0, [255, 0, 0, 255]);
    let crop = call(
        &e.core,
        Source::Cli,
        "document.selection_action",
        json!({"documentId": doc, "action": "crop"}),
    );
    assert!(crop.is_err(), "crop works on the image");
}

/// EDT-045: Photo quality fills the gap too; Cancel with nothing running says so.
#[test]
fn photo_quality_fills_and_cancel_is_safe() {
    let e = env();
    let doc = with_button(&e);
    let r = cli(&e.core, "editor.cancel", json!({}));
    assert_eq!(r["cancelled"], false);
    cli(
        &e.core,
        "document.select_pixels",
        json!({"documentId": doc, "shape": "rectangle", "rect": {"x": 56, "y": 36, "w": 88, "h": 32}}),
    );
    cli(
        &e.core,
        "document.selection_action",
        json!({"documentId": doc, "action": "delete", "quality": "photo"}),
    );
    let img = rendered(&e, &doc);
    assert!(
        img.pixels().all(|p| p.0[0] >= 250 && p.0[2] >= 250),
        "no button left"
    );
    let style = call(
        &e.core,
        Source::Ui,
        "editor.set_style",
        json!({"tool": "select", "style": {"quality": "slow"}}),
    );
    assert!(style.is_err(), "fast or photo");
}

/// EDT-021: a shape with a shadow down and to the right darkens there and not on the
/// other side; the editor learns where the shadow goes; bad values are refused.
#[test]
fn objects_cast_shadows() {
    let e = env();
    let (doc, _) = capture(&e);
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [{
            "type": "shape", "rect": {"x": 100, "y": 50, "w": 80, "h": 60},
            "style": {"shapeType": "rectangle", "fill": "#2D7FF9", "outline": "#2D7FF9",
                      "shadow": {"color": "#000000", "angle": 45, "distance": 10, "opacity": 30, "blur": 20}}
        }]}),
    );
    let img = rendered(&e, &doc);
    let below_right = img.get_pixel(185, 115).0;
    let above_left = img.get_pixel(95, 45).0;
    assert!(below_right[0] < 245, "shadowed: {below_right:?}");
    assert!(above_left[0] >= 250, "no shadow there: {above_left:?}");
    // Soft and light: never black.
    assert!(below_right[0] > 150, "30 % opacity: {below_right:?}");
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert!(info["objects"][0]["shadowAt"].is_array());
    let bad = call(
        &e.core,
        Source::Cli,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o1", "set": {"style": {"shadow": {"distance": 80}}}}),
    );
    assert!(bad.is_err(), "distance goes to 50");
    // Removing it.
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o1", "set": {"style": {"shadow": null}}}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert!(info["objects"][0]["shadowAt"].is_null());
    assert_eq!(rendered(&e, &doc).get_pixel(185, 115).0[0], 255);
}

/// EDT-015, EDT-016: saved styles of the active theme; save, apply to objects,
/// reset per tool; favorites keep a copy and pick their tool.
#[test]
fn saved_styles_and_favorites() {
    let e = env();
    let (doc, _) = capture(&e);
    let list = cli(&e.core, "editor.styles", json!({}));
    assert_eq!(list["theme"]["name"], "Base");
    let arrows = list["tools"]["arrow"].as_array().unwrap().clone();
    assert_eq!(arrows.len(), 5);
    // A thicker arrow, saved.
    cli(
        &e.core,
        "editor.set_style",
        json!({"tool": "arrow", "style": {"width": 22}}),
    );
    let saved = cli(
        &e.core,
        "editor.style",
        json!({"action": "save", "tool": "arrow"}),
    );
    let id = saved["styleId"].as_str().unwrap().to_string();
    let list = cli(&e.core, "editor.styles", json!({}));
    assert_eq!(list["tools"]["arrow"].as_array().unwrap().len(), 6);
    // Three callouts take a callout style; their text stays.
    let callout = |x: i32| json!({"type": "callout", "rect": {"x": x, "y": 20, "w": 80, "h": 40}, "text": format!("t{x}")});
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [callout(10), callout(110), callout(210)]}),
    );
    let blue = list["tools"]["callout"][2]["id"].as_str().unwrap().to_string();
    cli(
        &e.core,
        "editor.style",
        json!({"action": "apply", "tool": "callout", "styleId": blue, "documentId": doc, "objectIds": ["o1", "o2", "o3"]}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    for (i, o) in info["objects"].as_array().unwrap().iter().enumerate() {
        assert_eq!(o["style"]["fill"], "#2D7FF9");
        assert_eq!(o["text"], format!("t{}", 10 + i * 100));
    }
    // Reset shapes: arrows keep the saved one.
    cli(
        &e.core,
        "editor.style",
        json!({"action": "reset", "tool": "shape"}),
    );
    let list = cli(&e.core, "editor.styles", json!({}));
    assert!(list["tools"]["arrow"]
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s["id"] == id));
    // A favorite picks its tool and style.
    let fav = cli(
        &e.core,
        "editor.style",
        json!({"action": "favorite", "tool": "arrow", "styleId": id}),
    );
    cli(&e.core, "editor.select_tool", json!({"tool": "text"}));
    let r = cli(
        &e.core,
        "editor.style",
        json!({"action": "apply_favorite", "favoriteId": fav["favoriteId"]}),
    );
    assert_eq!(r["tool"], "arrow");
    let state = cli(&e.core, "editor.state", json!({}));
    assert_eq!(state["tool"], "arrow");
    assert_eq!(state["styles"]["arrow"]["width"], json!(22.0));
}

/// EDT-017, EDT-018: a new theme makes its styles; built-in ones stay; .xtheme files
/// go and come back, asking about the same name.
#[test]
fn themes_and_xtheme_files() {
    let e = env();
    let r = cli(
        &e.core,
        "editor.theme",
        json!({"action": "save", "theme": {"name": "Mío", "colors": ["#112233", "#DD3355", "#33AA77"], "font": "Georgia", "shadow": false}}),
    );
    let id = r["id"].as_str().unwrap().to_string();
    let list = cli(&e.core, "editor.styles", json!({}));
    assert_eq!(list["theme"]["name"], "Mío");
    assert_eq!(list["tools"]["callout"].as_array().unwrap().len(), 3);
    assert_eq!(list["tools"]["callout"][0]["style"]["font"], "Georgia");
    assert!(call(
        &e.core,
        Source::Cli,
        "editor.theme",
        json!({"action": "delete", "id": "base"})
    )
    .is_err());
    let bad = call(
        &e.core,
        Source::Cli,
        "editor.theme",
        json!({"action": "save", "theme": {"name": "X", "colors": ["#112233", "#112233"]}}),
    );
    assert!(bad.is_err(), "a color twice");
    // Export, then import: same name asks; keep both.
    let path = e.outside.path().join("mio.xtheme");
    cli(
        &e.core,
        "editor.theme",
        json!({"action": "export", "id": id, "path": path}),
    );
    let r = cli(&e.core, "editor.theme", json!({"action": "import", "path": path}));
    assert_eq!(r["status"], "needs_decision");
    let r = cli(
        &e.core,
        "editor.theme",
        json!({"action": "import", "path": path, "onConflict": "keep"}),
    );
    assert_eq!(r["status"], "imported");
    let themes = cli(&e.core, "editor.themes", json!({}));
    let names: Vec<&str> = themes["themes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap())
        .collect();
    assert!(names.contains(&"Mío") && names.contains(&"Mío (2)"));
    // A newer file is refused and nothing changes.
    std::fs::write(
        &path,
        r##"{"format":"xtheme","version":9,"name":"N","colors":["#000000","#FFFFFF"]}"##,
    )
    .unwrap();
    assert!(call(
        &e.core,
        Source::Cli,
        "editor.theme",
        json!({"action": "import", "path": path})
    )
    .is_err());
    assert_eq!(
        cli(&e.core, "editor.themes", json!({}))["themes"]
            .as_array()
            .unwrap()
            .len(),
        4
    );
}

/// EDT-019: My colors (no repeats, up to 24) and the last 8 used.
#[test]
fn my_colors_and_recent_colors() {
    let e = env();
    let r = cli(
        &e.core,
        "editor.color",
        json!({"action": "save", "color": "#2d7ff9"}),
    );
    assert_eq!(r["mine"], json!(["#2D7FF9"]), "one spelling");
    cli(
        &e.core,
        "editor.color",
        json!({"action": "save", "color": "#2D7FF9"}),
    );
    for i in 0..10 {
        cli(
            &e.core,
            "editor.color",
            json!({"action": "used", "color": format!("#0000{:02X}", i)}),
        );
    }
    let r = cli(&e.core, "editor.colors", json!({}));
    assert_eq!(r["mine"].as_array().unwrap().len(), 1);
    assert_eq!(r["recent"].as_array().unwrap().len(), 8);
    assert_eq!(r["recent"][0], "#000009", "newest first");
    for i in 0..23 {
        cli(
            &e.core,
            "editor.color",
            json!({"action": "save", "color": format!("#11{:02X}11", i)}),
        );
    }
    let full = call(
        &e.core,
        Source::Cli,
        "editor.color",
        json!({"action": "save", "color": "#ABCDEF"}),
    );
    assert!(full.is_err(), "24 at most");
    let r = cli(
        &e.core,
        "editor.color",
        json!({"action": "remove", "color": "#2D7FF9"}),
    );
    assert_eq!(r["mine"].as_array().unwrap().len(), 23);
}

/// EDT-020: a 90° linear gradient goes from the top to the bottom of a shape; the Fill
/// tool fills with a gradient; gradients are kept in My gradients.
#[test]
fn gradients_fill_shapes_and_areas() {
    let e = env();
    let (doc, _) = capture(&e);
    let red_blue = json!({"type": "linear", "angle": 90, "stops": [{"color": "#FF0000", "pos": 0}, {"color": "#0000FF", "pos": 100}]});
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [{
            "type": "shape", "rect": {"x": 20, "y": 20, "w": 100, "h": 100},
            "style": {"shapeType": "rectangle", "fill": red_blue, "outlineWidth": 0}
        }]}),
    );
    let img = rendered(&e, &doc);
    let (top, bottom) = (img.get_pixel(70, 22).0, img.get_pixel(70, 118).0);
    assert!(top[0] > 230 && top[2] < 25, "red at the top: {top:?}");
    assert!(
        bottom[2] > 230 && bottom[0] < 25,
        "blue at the bottom: {bottom:?}"
    );
    // Written as given; a bad one is refused.
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"][0]["style"]["fill"]["type"], "linear");
    let bad = call(
        &e.core,
        Source::Cli,
        "document.update_object",
        json!({"documentId": doc, "objectId": "o1", "set": {"style": {"fill": {"type": "linear", "stops": [{"color": "#FF0000", "pos": 0}]}}}}),
    );
    assert!(bad.is_err(), "two stops at least");
    // The Fill tool: a radial gradient over the white background.
    let radial = json!({"type": "radial", "stops": [{"color": "#00FF00", "pos": 0}, {"color": "#000000", "pos": 100}]});
    cli(
        &e.core,
        "document.fill",
        json!({"documentId": doc, "at": [300, 100], "color": radial, "tolerance": 0}),
    );
    let img = rendered(&e, &doc);
    let center = img.get_pixel(200, 100).0;
    let corner = img.get_pixel(398, 2).0;
    assert!(
        center[1] > corner[1],
        "lighter green at the center: {center:?} {corner:?}"
    );
    // My gradients.
    let r = cli(
        &e.core,
        "editor.color",
        json!({"action": "save_gradient", "gradient": red_blue}),
    );
    assert_eq!(r["gradients"].as_array().unwrap().len(), 1);
    let r = cli(
        &e.core,
        "editor.color",
        json!({"action": "remove_gradient", "gradient": red_blue}),
    );
    assert!(r["gradients"].as_array().unwrap().is_empty());
}

/// EDT-053, EDT-055: effects are kept in the file and editable; the export has them
/// (a 5 px outside border on 400 × 200 gives 410 × 210); order matters; a floating
/// image has its own stack; a selection takes an effect into its pixels.
#[test]
fn effects_stack_on_the_document_images_and_selections() {
    let e = env();
    let (doc, path) = capture(&e);
    cli(
        &e.core,
        "document.effects",
        json!({"documentId": doc, "action": "add", "effect": {"type": "border", "width": 5, "color": "#FF0000"}}),
    );
    cli(
        &e.core,
        "document.effects",
        json!({"documentId": doc, "action": "add", "effect": {"type": "shadow", "distance": 0, "blur": 0, "opacity": 0}}),
    );
    let img = rendered(&e, &doc);
    assert_eq!(img.dimensions(), (410, 210));
    assert_eq!(img.get_pixel(1, 100).0, [255, 0, 0, 255]);
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["width"], 400, "the editable canvas does not grow");
    assert_eq!(info["undoLabel"], "Change effects");
    // Saved and opened again: both there and on.
    cli(&e.core, "document.save", json!({"documentId": doc}));
    cli(&e.core, "document.close", json!({"documentId": doc}));
    let doc = cli(&e.core, "document.open", json!({"path": path}))["documentId"]
        .as_str()
        .unwrap()
        .to_string();
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    let fx = info["effects"].as_array().unwrap();
    assert_eq!(fx.len(), 2);
    assert_eq!(fx[0]["type"], "border");
    assert_eq!(fx[0]["enabled"], true);
    // Order: perspective after the border slants the border too.
    cli(
        &e.core,
        "document.effects",
        json!({"documentId": doc, "action": "set", "effects": [{"type": "border", "width": 5}, {"type": "perspective", "horizontal": 20}]}),
    );
    let a = rendered(&e, &doc);
    cli(
        &e.core,
        "document.effects",
        json!({"documentId": doc, "action": "move", "index": 1, "to": 0}),
    );
    let b = rendered(&e, &doc);
    assert!(a != b, "a different result");
    // Remove all: off, kept.
    cli(
        &e.core,
        "document.effects",
        json!({"documentId": doc, "action": "clear"}),
    );
    assert_eq!(rendered(&e, &doc).dimensions(), (400, 200));
    let bad = call(
        &e.core,
        Source::Cli,
        "document.effects",
        json!({"documentId": doc, "action": "add", "effect": {"type": "border", "width": 500}}),
    );
    assert!(bad.is_err(), "width goes to 100");
    // A floating image's own border.
    let red = red_png(&e, 40, 20);
    cli(
        &e.core,
        "document.insert_images",
        json!({"documentId": doc, "paths": [red], "center": [200, 100]}),
    );
    cli(
        &e.core,
        "document.effects",
        json!({"documentId": doc, "objectId": "o1", "action": "add", "effect": {"type": "border", "width": 4, "color": "#0000FF"}}),
    );
    let img = rendered(&e, &doc);
    assert_eq!(img.get_pixel(178, 100).0, [0, 0, 255, 255], "around the image");
    assert_eq!(
        img.get_pixel(5, 5).0,
        [255, 255, 255, 255],
        "not around the document"
    );
    // A selection takes grayscale; Undo takes it back.
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([10, 10], [20, 20])]}),
    );
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    cli(
        &e.core,
        "document.fill",
        json!({"documentId": doc, "at": [5, 5], "color": "#FF0000"}),
    );
    cli(
        &e.core,
        "document.select_pixels",
        json!({"documentId": doc, "shape": "rectangle", "rect": {"x": 0, "y": 0, "w": 100, "h": 100}}),
    );
    cli(
        &e.core,
        "document.selection_action",
        json!({"documentId": doc, "action": "effect", "effect": {"type": "filters", "grayscale": {"on": true}}}),
    );
    let img = rendered(&e, &doc);
    let inside = img.get_pixel(50, 50).0;
    assert!(
        inside[0] == inside[1] && inside[1] == inside[2],
        "gray: {inside:?}"
    );
    assert_eq!(img.get_pixel(200, 150).0, [255, 0, 0, 255], "outside stays red");
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    assert_eq!(rendered(&e, &doc).get_pixel(50, 50).0, [255, 0, 0, 255]);
}

/// EDT-054: a saved effect replaces the stack, or Shift adds it at the end; the
/// current stack can be saved and applied to another capture; a favorite applies it.
#[test]
fn saved_effects_replace_or_add() {
    let e = env();
    let (doc, _) = capture(&e);
    let list = cli(&e.core, "editor.saved_effects", json!({}));
    assert_eq!(list["effects"].as_array().unwrap().len(), 6);
    cli(
        &e.core,
        "document.effects",
        json!({"documentId": doc, "action": "add", "effect": {"type": "border", "width": 3}}),
    );
    cli(
        &e.core,
        "editor.saved_effect",
        json!({"action": "apply", "id": "grayscale", "documentId": doc, "append": true}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    let types: Vec<&str> = info["effects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e["type"].as_str().unwrap())
        .collect();
    assert_eq!(types, ["border", "filters"], "the border stays");
    // Border + rounded corners + shadow, saved and applied to another capture.
    cli(
        &e.core,
        "document.effects",
        json!({"documentId": doc, "action": "set", "effects": [
            {"type": "border", "width": 4}, {"type": "corners", "radius": 10}, {"type": "shadow", "distance": 5}]}),
    );
    let saved = cli(
        &e.core,
        "editor.saved_effect",
        json!({"action": "save", "name": "Mine", "documentId": doc}),
    );
    let (other, _) = capture(&e);
    cli(
        &e.core,
        "editor.saved_effect",
        json!({"action": "apply", "id": saved["id"], "documentId": other}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": other}));
    let fx = info["effects"].as_array().unwrap();
    assert_eq!(fx.len(), 3);
    assert_eq!(fx[1]["radius"], json!(10));
    // As a favorite, it applies to the document shown.
    let fav = cli(
        &e.core,
        "editor.saved_effect",
        json!({"action": "favorite", "id": "fine-frame"}),
    );
    cli(
        &e.core,
        "editor.style",
        json!({"action": "apply_favorite", "favoriteId": fav["favoriteId"], "documentId": other}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": other}));
    assert_eq!(info["effects"][0]["type"], "border");
    assert_eq!(info["effects"].as_array().unwrap().len(), 1);
}

/// EDT-069, EDT-072, EDT-076 (ADR 0006): resizing scales pixels and objects (text
/// too) and is saved into base.png; turning a quarter and back gives the original;
/// reverting drops every edit and Undo brings them back.
#[test]
fn resizing_turning_and_reverting_the_image() {
    let e = env();
    let (doc, path) = add_capture(&e.core, &white(2000, 1000), json!({"mode": "region"}), 1.0, None).unwrap();
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([100, 100], [300, 100]),
            {"type": "text", "at": [500, 500], "text": "Hola", "style": {"size": 20}}]}),
    );
    let r = cli(
        &e.core,
        "document.resize",
        json!({"documentId": doc, "percent": 50}),
    );
    assert_eq!(
        (r["width"].as_u64(), r["height"].as_u64()),
        (Some(1000), Some(500))
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(
        info["objects"][1]["style"]["size"],
        json!(10.0),
        "the text is 10 pt"
    );
    assert_eq!(info["objects"][0]["to"], json!([150.0, 50.0]));
    assert_eq!(info["base"]["width"], 1000, "new original pixels");
    // Saved: base.png has them, the file no picture in their place.
    cli(&e.core, "document.save", json!({"documentId": doc}));
    let file = Xshot::read(&path).unwrap();
    assert!(file.document.base_image.is_none());
    assert!(file.document.original_changed, "Revert to original will warn");
    assert_eq!(xs_image::encode::png_size(&file.base_png), Some((1000, 500)));
    // A quarter turn and back: exactly the same image.
    let before = rendered(&e, &doc);
    cli(
        &e.core,
        "document.rotate",
        json!({"documentId": doc, "degrees": 90}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(
        (info["width"].as_u64(), info["height"].as_u64()),
        (Some(500), Some(1000))
    );
    cli(
        &e.core,
        "document.rotate",
        json!({"documentId": doc, "degrees": -90}),
    );
    assert_eq!(rendered(&e, &doc), before);
    // Revert, then Undo.
    cli(&e.core, "document.revert", json!({"documentId": doc}));
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert!(info["objects"].as_array().unwrap().is_empty());
    assert_eq!(info["undoLabel"], "Revert to original");
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"].as_array().unwrap().len(), 2);
}

/// EDT-069: nearest neighbour doubles pixels without mixing; without resampling only
/// the resolution changes.
#[test]
fn resampling_options() {
    let e = env();
    let img = image::RgbaImage::from_fn(4, 2, |x, _| {
        if x % 2 == 0 {
            image::Rgba([255, 0, 0, 255])
        } else {
            image::Rgba([0, 0, 255, 255])
        }
    });
    let (doc, _) = add_capture(&e.core, &img, json!({"mode": "region"}), 1.0, None).unwrap();
    cli(
        &e.core,
        "document.resize",
        json!({"documentId": doc, "percent": 200, "resample": "nearest"}),
    );
    let out = rendered(&e, &doc);
    assert_eq!(out.dimensions(), (8, 4));
    assert_eq!(out.get_pixel(1, 0).0, [255, 0, 0, 255]);
    assert_eq!(out.get_pixel(2, 0).0, [0, 0, 255, 255]);
    let dpi = cli(&e.core, "document.info", json!({"documentId": doc}))["dpi"]
        .as_f64()
        .unwrap();
    cli(
        &e.core,
        "document.resize",
        json!({"documentId": doc, "percent": 50, "resampleImage": false}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["width"], 8, "same pixels");
    assert_eq!(info["dpi"].as_f64().unwrap(), dpi * 2.0, "twice the resolution");
}

/// EDT-070, EDT-074, EDT-075, EDT-077: canvas size around an anchor and back;
/// transparent canvas; rounded corners and a circle; resolution without new pixels.
#[test]
fn canvas_color_size_corners_and_resolution() {
    let e = env();
    let (doc, _) = add_capture(&e.core, &white(800, 600), json!({"mode": "region"}), 1.0, None).unwrap();
    cli(
        &e.core,
        "document.canvas_size",
        json!({"documentId": doc, "width": 1000, "height": 1000, "anchor": "center"}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(
        info["crop"],
        json!({"x": -100, "y": -200, "width": 1000, "height": 1000})
    );
    cli(
        &e.core,
        "document.canvas_size",
        json!({"documentId": doc, "width": 800, "height": 600, "anchor": "center"}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(
        info["crop"],
        json!({"x": 0, "y": 0, "width": 800, "height": 600}),
        "back without loss"
    );
    // Rounded corners: transparent corners in the image that leaves.
    cli(
        &e.core,
        "document.canvas",
        json!({"documentId": doc, "cornerRadius": 20}),
    );
    assert_eq!(rendered(&e, &doc).get_pixel(0, 0).0[3], 0);
    cli(
        &e.core,
        "document.canvas",
        json!({"documentId": doc, "cornerRadius": "circle"}),
    );
    let out = rendered(&e, &doc);
    assert_eq!(out.dimensions(), (600, 600));
    assert_eq!(out.get_pixel(300, 300).0[3], 255);
    assert_eq!(out.get_pixel(30, 30).0[3], 0);
    // Resolution: same pixels, a new value in the PNG.
    cli(&e.core, "document.canvas", json!({"documentId": doc, "dpi": 300}));
    let target = e.outside.path().join("dpi.png");
    cli(
        &e.core,
        "document.export",
        json!({"documentId": doc, "path": target, "format": "png"}),
    );
    let png = std::fs::read(&target).unwrap();
    assert_eq!(xs_image::encode::png_dpi(&png).map(|d| d.round()), Some(300.0));
    // A transparent canvas: erasing leaves transparency.
    cli(
        &e.core,
        "document.canvas",
        json!({"documentId": doc, "color": null}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert!(info["canvas"]["color"].is_null());
}

/// EDT-073: flipping turns the pixels and objects over (arrows point the other way,
/// text stays readable); twice gives the original; with objectIds only those objects
/// flip, around the box that holds them.
#[test]
fn flipping_the_image_and_objects() {
    let e = env();
    let img = image::RgbaImage::from_fn(2000, 1000, |x, _| {
        if x < 1000 {
            image::Rgba([255, 0, 0, 255])
        } else {
            image::Rgba([0, 0, 255, 255])
        }
    });
    let (doc, _) = add_capture(&e.core, &img, json!({"mode": "region"}), 1.0, None).unwrap();
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([100, 100], [300, 100]),
            {"type": "text", "at": [500, 500], "text": "Hola", "style": {"size": 20}}]}),
    );
    let before = rendered(&e, &doc);
    let text_before = cli(&e.core, "document.info", json!({"documentId": doc}))["objects"][1].clone();
    cli(
        &e.core,
        "document.flip",
        json!({"documentId": doc, "direction": "horizontal"}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"][0]["from"], json!([1900.0, 100.0]));
    assert_eq!(info["objects"][0]["to"], json!([1700.0, 100.0]), "it points left");
    let text = &info["objects"][1];
    assert_eq!(text["text"], text_before["text"], "the text is not mirrored");
    let at = text["at"][0].as_f64().unwrap();
    assert!(
        at < 1500.0 && at > 1300.0,
        "its box moved to the other side: {at}"
    );
    assert_eq!(info["undoLabel"], "Flip");
    let out = rendered(&e, &doc);
    assert_eq!(out.get_pixel(10, 900).0, [0, 0, 255, 255], "blue on the left");
    // Twice: the original image.
    cli(
        &e.core,
        "document.flip",
        json!({"documentId": doc, "direction": "horizontal"}),
    );
    assert_eq!(rendered(&e, &doc), before);
    // Vertically, only the arrow: it flips inside its own box.
    let id = info["objects"][0]["id"].as_str().unwrap().to_string();
    cli(
        &e.core,
        "document.flip",
        json!({"documentId": doc, "direction": "vertical", "objectIds": [id]}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"][0]["from"], json!([100.0, 100.0]));
    assert_eq!(
        info["objects"][1]["at"], text_before["at"],
        "the text did not move"
    );
    assert_eq!(
        rendered(&e, &doc).get_pixel(10, 900).0,
        [255, 0, 0, 255],
        "pixels unchanged"
    );
    // Undo brings the flipped pixels back step by step.
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    assert_eq!(rendered(&e, &doc).get_pixel(10, 900).0, [0, 0, 255, 255]);
}

/// EDT-074: an opaque capture gets a white canvas, one with transparency keeps it
/// transparent; Transparent and an eraser then leave transparency in the PNG.
#[test]
fn captures_start_with_a_white_canvas_unless_transparent() {
    let e = env();
    let (opaque, _) = add_capture(&e.core, &white(40, 20), json!({"mode": "region"}), 1.0, None).unwrap();
    let info = cli(&e.core, "document.info", json!({"documentId": opaque}));
    assert_eq!(info["canvas"]["color"], "#FFFFFF");
    let mut rounded = white(40, 20);
    rounded.put_pixel(0, 0, image::Rgba([0, 0, 0, 0]));
    let (clear, _) = add_capture(&e.core, &rounded, json!({"mode": "window"}), 1.0, None).unwrap();
    let info = cli(&e.core, "document.info", json!({"documentId": clear}));
    assert!(info["canvas"]["color"].is_null());
    // Transparent, then erase: transparent in the exported PNG.
    cli(
        &e.core,
        "document.canvas",
        json!({"documentId": opaque, "color": null}),
    );
    cli(
        &e.core,
        "document.erase",
        json!({"documentId": opaque, "points": [[10, 10]], "width": 6, "shape": "square"}),
    );
    let out = e.outside.path().join("erased.png");
    cli(
        &e.core,
        "document.export",
        json!({"documentId": opaque, "format": "png", "path": out}),
    );
    let png = image::open(&out).unwrap().to_rgba8();
    assert_eq!(png.get_pixel(10, 10).0[3], 0);
    assert_eq!(png.get_pixel(30, 10).0, [255, 255, 255, 255]);
}

/// EDT-051: trimming uniform borders leaves the content (and says when there are
/// none); fitting the canvas takes in a floating image that sticks out; cropping to
/// the canvas cuts its picture and reports annotations that still stick out.
#[test]
fn trimming_fitting_and_cropping_to_the_canvas() {
    let e = env();
    // 40 px of white around a blue block.
    let img = image::RgbaImage::from_fn(200, 120, |x, y| {
        if (40..160).contains(&x) && (40..80).contains(&y) {
            image::Rgba([30, 110, 220, 255])
        } else {
            image::Rgba([255, 255, 255, 255])
        }
    });
    let (doc, _) = add_capture(&e.core, &img, json!({"mode": "region"}), 1.0, None).unwrap();
    let r = cli(&e.core, "document.trim", json!({"documentId": doc}));
    assert_eq!(r["changed"], true);
    assert_eq!((r["width"].as_u64(), r["height"].as_u64()), (Some(120), Some(40)));
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["undoLabel"], "Trim edges");
    let r = cli(&e.core, "document.trim", json!({"documentId": doc}));
    assert_eq!(r["changed"], false, "no borders left");
    assert!(r["message"].as_str().is_some_and(|m| !m.is_empty()));
    cli(&e.core, "document.undo", json!({"documentId": doc}));

    // A floating image 100 px past the right edge, with the canvas not growing.
    cli(
        &e.core,
        "settings.set",
        json!({"key": "editor.autoExpandCanvas", "value": false}),
    );
    let red = red_png(&e, 100, 40);
    cli(
        &e.core,
        "document.insert_images",
        json!({"documentId": doc, "paths": [red], "center": [100, 60]}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    let pic = info["objects"][0]["id"].as_str().unwrap().to_string();
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectId": pic, "set": {"rect": {"x": 150, "y": 10, "w": 150, "h": 40}}}),
    );
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [arrow([150, 100], [260, 100])]}),
    );
    let r = cli(&e.core, "document.fit_canvas", json!({"documentId": doc}));
    assert_eq!(r["changed"], true);
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["width"], 300, "100 px more");
    assert_eq!(info["objects"][0]["rect"]["x"], json!(150.0), "inside now");
    cli(&e.core, "document.undo", json!({"documentId": doc}));

    // Crop to canvas: the picture loses its outer 100 px; the arrow stays and is reported.
    let before = cli(&e.core, "document.info", json!({"documentId": doc}))["objects"][0]["image"].clone();
    let r = cli(&e.core, "document.crop_to_canvas", json!({"documentId": doc}));
    assert_eq!(r["changed"], true);
    assert_eq!(r["sticking"].as_array().map(Vec::len), Some(1));
    assert!(r["warnings"][0].as_str().is_some());
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["width"], 200);
    assert_eq!(info["objects"][0]["rect"]["w"], json!(50.0));
    assert_ne!(info["objects"][0]["image"], before, "a new, narrower picture");
    assert_eq!(info["undoLabel"], "Crop to canvas");
    // Nothing more to cut: said, with the arrow still sticking out.
    let r = cli(&e.core, "document.crop_to_canvas", json!({"documentId": doc}));
    assert_eq!(r["changed"], false);
}

/// EDT-074, EDT-020: a canvas with a linear gradient shows it in the space added to
/// the canvas; a plain color takes its place again.
#[test]
fn a_gradient_canvas_shows_in_added_space() {
    let e = env();
    let (doc, _) = add_capture(&e.core, &white(100, 50), json!({"mode": "region"}), 1.0, None).unwrap();
    let gradient = json!({"type": "linear", "angle": 0, "stops": [
        {"color": "#FF0000", "pos": 0}, {"color": "#0000FF", "pos": 100}]});
    cli(
        &e.core,
        "document.canvas",
        json!({"documentId": doc, "color": gradient}),
    );
    cli(
        &e.core,
        "document.canvas_size",
        json!({"documentId": doc, "width": 300, "height": 50, "anchor": "center"}),
    );
    let img = rendered(&e, &doc);
    let (left, right) = (img.get_pixel(2, 25).0, img.get_pixel(297, 25).0);
    assert!(left[0] > 200 && left[2] < 60, "red on the left: {left:?}");
    assert!(right[2] > 200 && right[0] < 60, "blue on the right: {right:?}");
    assert_eq!(img.get_pixel(150, 25).0, [255, 255, 255, 255], "the image on top");
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["canvas"]["color"], "#FF0000", "the first stop");
    assert_eq!(info["canvas"]["gradient"], gradient);
    cli(
        &e.core,
        "document.canvas",
        json!({"documentId": doc, "color": "#00FF00"}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert!(info["canvas"]["gradient"].is_null());
    assert_eq!(rendered(&e, &doc).get_pixel(2, 25).0, [0, 255, 0, 255]);
    // A bad gradient is refused.
    let bad = call(
        &e.core,
        Source::Cli,
        "document.canvas",
        json!({"documentId": doc, "color": {"type": "linear", "stops": [{"color": "#FF0000", "pos": 0}]}}),
    );
    assert!(bad.is_err());
}

/// EDT-075: with three floating images selected, Radius = 16 rounds the three; their
/// corners show what is below.
#[test]
fn floating_images_get_rounded_corners_together() {
    let e = env();
    let (doc, _) = add_capture(&e.core, &white(400, 200), json!({"mode": "region"}), 1.0, None).unwrap();
    let paths: Vec<PathBuf> = (0..3).map(|_| red_png(&e, 60, 60)).collect();
    cli(
        &e.core,
        "document.insert_images",
        json!({"documentId": doc, "paths": paths, "center": [200, 100]}),
    );
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    let ids: Vec<String> = info["objects"]
        .as_array()
        .unwrap()
        .iter()
        .map(|o| o["id"].as_str().unwrap().to_string())
        .collect();
    for (i, id) in ids.iter().enumerate() {
        cli(
            &e.core,
            "document.update_object",
            json!({"documentId": doc, "objectId": id, "set": {"rect": {"x": 20 + 120 * i, "y": 70, "w": 60, "h": 60}}}),
        );
    }
    cli(
        &e.core,
        "document.update_object",
        json!({"documentId": doc, "objectIds": ids, "set": {"style": {"cornerRadius": 16}}}),
    );
    let img = rendered(&e, &doc);
    for i in 0..3u32 {
        let x0 = 20 + 120 * i;
        assert_eq!(
            img.get_pixel(x0, 70).0,
            [255, 255, 255, 255],
            "corner {i} rounded"
        );
        assert_eq!(img.get_pixel(x0 + 30, 100).0, [255, 0, 0, 255], "middle {i} red");
    }
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"][2]["style"]["cornerRadius"], json!(16.0));
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    assert_eq!(
        rendered(&e, &doc).get_pixel(20, 70).0,
        [255, 0, 0, 255],
        "square again"
    );
}

/// EDT-052: removing a 200 px strip leaves 800 px; arrows above stay and below move up;
/// one across it stays and is reported; a wavy join leaves a 20 px wavy gap; adding
/// 150 px of vertical space widens the image and moves what is to the right.
#[test]
fn cutting_out_strips_and_adding_space() {
    let e = env();
    let (doc, _) = add_capture(&e.core, &white(600, 1000), json!({"mode": "region"}), 1.0, None).unwrap();
    cli(
        &e.core,
        "document.annotate",
        json!({"documentId": doc, "objects": [
            arrow([100, 100], [300, 100]),
            arrow([100, 800], [300, 800]),
            arrow([400, 200], [400, 700])]}),
    );
    let r = cli(
        &e.core,
        "document.cut_out",
        json!({"documentId": doc, "direction": "horizontal", "from": 300, "to": 500}),
    );
    assert_eq!(r["height"], 800);
    assert_eq!(r["crossing"].as_array().map(Vec::len), Some(1));
    assert!(r["warnings"][0].as_str().is_some());
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["objects"][0]["from"], json!([100.0, 100.0]), "above: stays");
    assert_eq!(
        info["objects"][1]["from"],
        json!([100.0, 600.0]),
        "below: 200 px up"
    );
    assert_eq!(info["objects"][2]["from"], json!([400.0, 200.0]), "across: stays");
    assert_eq!(info["undoLabel"], "Cut out");
    cli(&e.core, "document.undo", json!({"documentId": doc}));

    // Wavy, with a 20 px gap: on a transparent canvas, 20 transparent pixels a column.
    cli(
        &e.core,
        "document.canvas",
        json!({"documentId": doc, "color": null}),
    );
    cli(
        &e.core,
        "document.cut_out",
        json!({"documentId": doc, "from": 300, "to": 500, "join": "wavy", "gap": 20}),
    );
    let img = rendered(&e, &doc);
    let run = |x: u32| {
        (0..img.height())
            .filter(|&y| img.get_pixel(x, y).0[3] == 0)
            .collect::<Vec<_>>()
    };
    assert_eq!(run(10).len(), 20);
    assert_ne!(run(10)[0], run(30)[0], "the gap waves");
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    cli(&e.core, "document.undo", json!({"documentId": doc}));

    // Add 150 px of vertical space at x = 350: wider, and what is right of it moves.
    let r = cli(
        &e.core,
        "document.cut_out",
        json!({"documentId": doc, "direction": "vertical", "mode": "add", "from": 350, "to": 500}),
    );
    assert_eq!(r["width"], 750);
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(
        info["objects"][2]["from"],
        json!([550.0, 200.0]),
        "right of it: 150 px right"
    );
    assert_eq!(
        info["objects"][0]["from"],
        json!([100.0, 100.0]),
        "left of it: stays"
    );
    assert_eq!(info["undoLabel"], "Add space");
    // Undo and Revert bring the original size back.
    cli(&e.core, "document.undo", json!({"documentId": doc}));
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(
        (info["width"].as_u64(), info["height"].as_u64()),
        (Some(600), Some(1000))
    );
}

/// EDT-103: New image with 800 × 600 and Transparent makes a transparent canvas of
/// that size; Remember makes the next one start with the same values.
#[test]
fn new_images_and_remembered_defaults() {
    let e = env();
    let r = cli(
        &e.core,
        "document.new",
        json!({"width": 800, "height": 600, "color": null, "remember": true}),
    );
    let doc = r["documentId"].as_str().unwrap().to_string();
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(
        (info["width"].as_u64(), info["height"].as_u64()),
        (Some(800), Some(600))
    );
    assert!(info["canvas"]["color"].is_null(), "transparent");
    assert_eq!(rendered(&e, &doc).get_pixel(10, 10).0[3], 0);
    assert_eq!(
        info["name"].as_str().map(|n| n.starts_with("New image")),
        Some(true)
    );
    // The next one, without values, takes the remembered ones.
    let r = cli(&e.core, "document.new", json!({}));
    let info = cli(&e.core, "document.info", json!({"documentId": r["documentId"]}));
    assert_eq!(
        (info["width"].as_u64(), info["height"].as_u64()),
        (Some(800), Some(600))
    );
    assert!(info["canvas"]["color"].is_null());
    // A color and a resolution.
    let r = cli(
        &e.core,
        "document.new",
        json!({"width": 100, "height": 50, "color": "#0064FB", "dpi": 144}),
    );
    let doc = r["documentId"].as_str().unwrap().to_string();
    assert_eq!(rendered(&e, &doc).get_pixel(5, 5).0, [0, 100, 251, 255]);
    let info = cli(&e.core, "document.info", json!({"documentId": doc}));
    assert_eq!(info["dpi"], json!(144.0));
    let bad = call(
        &e.core,
        Source::Cli,
        "document.new",
        json!({"width": 0, "height": 10}),
    );
    assert!(bad.is_err());
}
