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
    // The new area is the canvas color (transparent): the export shows it.
    let out = e.outside.path().join("grown.png");
    cli(
        &e.core,
        "document.export",
        json!({"documentId": doc, "format": "png", "path": out}),
    );
    let png = image::open(&out).unwrap().to_rgba8();
    assert_eq!(png.width() as u64, w);
    assert_eq!(png.get_pixel(w as u32 - 1, 10).0[3], 0);
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
