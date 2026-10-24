//! Command-level tests: what an agent or the editor view sees (EDT-119).

use super::*;
use xs_core::platform::Platform;

struct Env {
    core: Arc<Core>,
    library: tempfile::TempDir,
    _recovery: tempfile::TempDir,
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
    let dir = e
        .library
        .path()
        .parent()
        .unwrap()
        .join(format!("ext-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = xs_library::unique(&dir, "outside", "png");
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
    let later = call(
        &e.core,
        Source::Cli,
        "editor.select_tool",
        json!({"tool": "crop"}),
    )
    .unwrap_err();
    assert_eq!(later.code, ErrorCode::NotFound);
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
    for kind in ["arrow", "objects"] {
        for action in ["draw", "move", "change", "delete"] {
            let key = objects::label(
                action,
                &[if kind == "arrow" { "arrow" } else { "a" }, "b"][..if kind == "arrow" { 1 } else { 2 }],
            );
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
}
