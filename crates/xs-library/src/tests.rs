//! Library tests: the index follows the folder (LIB-022), IDs survive renames made
//! outside the app, thumbnails, delete with Undo (LIB-015) and the commands.

use super::*;
use std::collections::BTreeMap;
use xs_core::document::{Document, Manifest, Xshot};
use xs_core::platform::Platform;

struct Env {
    root: tempfile::TempDir,
    data: tempfile::TempDir,
}

fn env() -> Env {
    Env {
        root: tempfile::tempdir().unwrap(),
        data: tempfile::tempdir().unwrap(),
    }
}

impl Env {
    fn open(&self) -> Library {
        Library::open(self.root.path().to_path_buf(), self.data.path()).holding_deletes()
    }

    fn path(&self, rel: &str) -> PathBuf {
        self.root.path().join(rel)
    }
}

fn png(w: u32, h: u32) -> Vec<u8> {
    let img = image::RgbaImage::from_pixel(w, h, image::Rgba([10, 120, 250, 255]));
    xs_image::encode::png(&img, 0.0, true).unwrap()
}

fn write_png(path: &Path, w: u32, h: u32) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, png(w, h)).unwrap();
}

fn write_xshot(path: &Path, app: &str, captured: &str) {
    let x = Xshot {
        manifest: Manifest::current(),
        document: Document::new(400, 300, 96.0),
        base_png: png(400, 300),
        // The app as CAP-056 writes it.
        meta: json!({ "capturedAt": captured, "app": {"name": app, "version": "1.0"}, "url": "https://example.com/a" }),
        thumb_png: png(40, 30),
        images: BTreeMap::new(),
    };
    x.write(path).unwrap();
}

fn ids(lib: &Library) -> Vec<String> {
    lib.list().into_iter().map(|i| i.id).collect()
}

#[test]
fn indexes_the_items_of_the_folder_and_nothing_else() {
    let e = env();
    write_xshot(
        &e.path("2026-11/Chrome 2026-11-10 10.00.00.xshot"),
        "Chrome",
        "2026-11-10T10:00:00+00:00",
    );
    write_png(&e.path("old capture.png"), 64, 48);
    std::fs::write(e.path("notes.txt"), b"x").unwrap();
    std::fs::write(e.path(".hidden.png"), png(2, 2)).unwrap();
    std::fs::write(e.path("library.db-journal"), b"x").unwrap();
    let lib = e.open();
    assert!(lib.scan());
    let items = lib.list();
    assert_eq!(items.len(), 2, "{items:?}");
    let xshot = items.iter().find(|i| i.format == "xshot").unwrap();
    assert_eq!(xshot.name, "Chrome 2026-11-10 10.00.00");
    assert_eq!((xshot.width, xshot.height), (Some(400), Some(300)));
    assert_eq!(xshot.app.as_deref(), Some("Chrome"));
    assert_eq!(xshot.path, "2026-11/Chrome 2026-11-10 10.00.00.xshot");
    let loose = items.iter().find(|i| i.format == "png").unwrap();
    assert_eq!((loose.width, loose.height), (Some(64), Some(48)));
    // Each item has its hidden sidecar with its ID (LIB-022, LIB-034).
    assert_eq!(xmeta::read(&e.path("old capture.png")).unwrap().id, loose.id);
    assert!(!lib.scan(), "a second scan finds nothing new");
}

#[test]
fn newest_capture_comes_first() {
    let e = env();
    write_xshot(&e.path("a.xshot"), "A", "2026-01-01T00:00:00+00:00");
    write_xshot(&e.path("b.xshot"), "B", "2026-06-01T00:00:00+00:00");
    let lib = e.open();
    lib.scan();
    let names: Vec<String> = lib.list().into_iter().map(|i| i.name).collect();
    assert_eq!(names, ["b", "a"]);
}

#[test]
fn a_rename_outside_the_app_keeps_the_id() {
    let e = env();
    write_png(&e.path("before.png"), 10, 10);
    let lib = e.open();
    lib.scan();
    let id = ids(&lib)[0].clone();
    // Explorer renames the file but leaves its hidden sidecar behind.
    std::fs::rename(e.path("before.png"), e.path("after.png")).unwrap();
    assert!(lib.scan());
    let item = lib.find(&e.path("after.png")).unwrap();
    assert_eq!(item.id, id);
    assert_eq!(item.name, "after");
    assert!(
        xmeta::path_of(&e.path("after.png")).exists(),
        "the sidecar followed"
    );
    // Moved to another folder together with its sidecar.
    std::fs::create_dir_all(e.path("sub")).unwrap();
    std::fs::rename(e.path("after.png"), e.path("sub/after.png")).unwrap();
    xmeta::follow(&e.path("after.png"), &e.path("sub/after.png"));
    lib.scan();
    assert_eq!(lib.find(&e.path("sub/after.png")).unwrap().id, id);
    assert_eq!(lib.list().len(), 1);
}

#[test]
fn saving_again_keeps_the_id_and_reads_the_new_size() {
    let e = env();
    write_png(&e.path("shot.png"), 10, 10);
    let lib = e.open();
    lib.scan();
    let id = ids(&lib)[0].clone();
    std::thread::sleep(std::time::Duration::from_millis(20));
    write_png(&e.path("shot.png"), 30, 20);
    assert!(lib.scan());
    let item = lib.get(&id).unwrap();
    assert_eq!((item.width, item.height), (Some(30), Some(20)));
}

#[test]
fn files_deleted_outside_the_app_leave_the_index() {
    let e = env();
    write_png(&e.path("a.png"), 4, 4);
    write_png(&e.path("b.png"), 5, 5);
    let lib = e.open();
    lib.scan();
    std::fs::remove_file(e.path("a.png")).unwrap();
    assert!(lib.scan());
    let names: Vec<String> = lib.list().into_iter().map(|i| i.name).collect();
    assert_eq!(names, ["b"]);
}

#[test]
fn the_index_can_be_deleted_or_damaged_and_rebuilt() {
    let e = env();
    write_png(&e.path("a.png"), 4, 4);
    write_png(&e.path("b.png"), 6, 6);
    let before = {
        let lib = e.open();
        lib.scan();
        let mut ids = ids(&lib);
        ids.sort();
        ids
    };
    let db = service::index_path(e.root.path(), e.data.path());
    std::fs::write(&db, b"this is not a database").unwrap();
    let lib = e.open();
    lib.scan();
    let mut after = ids(&lib);
    after.sort();
    assert_eq!(before, after, "IDs come back from the sidecars");
}

#[test]
fn synced_folders_keep_the_index_in_the_app_data() {
    let data = Path::new("C:/data");
    let synced_root = Path::new("C:/Users/x/OneDrive/Documents/XcreenShot");
    assert!(service::index_path(synced_root, data).starts_with(data));
    let local = Path::new("C:/Users/x/Documents/XcreenShot");
    assert_eq!(service::index_path(local, data), local.join("library.db"));
}

#[test]
fn thumbnails_come_from_the_xshot_or_are_made_and_cached() {
    let e = env();
    write_xshot(&e.path("x.xshot"), "App", "2026-11-10T10:00:00+00:00");
    write_png(&e.path("big.png"), 1200, 600);
    let lib = e.open();
    lib.scan();
    for item in lib.list() {
        let thumb = lib.thumbnail(&item.id).unwrap();
        let (w, h) = xs_image::encode::png_size(&thumb).unwrap();
        if item.format == "xshot" {
            assert_eq!((w, h), (40, 30), "the file's own thumbnail");
        } else {
            assert_eq!((w, h), (320, 160));
        }
        assert_eq!(
            lib.thumbnail(&item.id).unwrap(),
            thumb,
            "the second time from the cache"
        );
    }
}

#[test]
fn delete_and_undo_bring_back_the_file_and_its_id() {
    let e = env();
    write_png(&e.path("a.png"), 4, 4);
    write_png(&e.path("b.png"), 5, 5);
    let lib = e.open();
    lib.scan();
    let a = lib.find(&e.path("a.png")).unwrap().id;
    let (undo, gone) = lib.delete(std::slice::from_ref(&a), false).unwrap();
    assert_eq!(gone.len(), 1);
    assert!(!e.path("a.png").exists());
    assert!(!xmeta::path_of(&e.path("a.png")).exists(), "its sidecar went too");
    assert_eq!(lib.list().len(), 1);
    let back = lib.restore(&undo.unwrap()).unwrap();
    assert_eq!(back, vec![e.path("a.png")]);
    assert_eq!(lib.find(&e.path("a.png")).unwrap().id, a);
    assert!(lib.restore("nope").is_err());
}

#[test]
fn permanent_delete_has_no_undo() {
    let e = env();
    write_png(&e.path("a.png"), 4, 4);
    let lib = e.open();
    lib.scan();
    let id = ids(&lib)[0].clone();
    let (undo, _) = lib.delete(&[id], true).unwrap();
    assert!(undo.is_none());
    assert!(!e.path("a.png").exists());
}

#[test]
fn rename_keeps_folder_extension_and_id() {
    let e = env();
    write_png(&e.path("2026-11/shot.png"), 4, 4);
    write_png(&e.path("2026-11/taken.png"), 4, 4);
    let lib = e.open();
    lib.scan();
    let id = lib.find(&e.path("2026-11/shot.png")).unwrap().id;
    let item = lib.rename(&id, "Quarterly: sales").unwrap();
    assert_eq!(item.id, id);
    assert_eq!(item.path, "2026-11/Quarterly sales.png");
    assert!(e.path("2026-11/Quarterly sales.png").exists());
    assert!(xmeta::path_of(&e.path("2026-11/Quarterly sales.png")).exists());
    assert!(matches!(
        lib.rename(&id, "taken"),
        Err(service::RenameError::Exists(_))
    ));
    assert!(matches!(
        lib.rename(&id, " :: "),
        Err(service::RenameError::Invalid)
    ));
    // A change of case only renames the same file.
    assert!(lib.rename(&id, "quarterly sales").is_ok());
}

#[test]
fn commands_list_get_delete_and_restore() {
    let e = env();
    write_xshot(&e.path("2026-11/a.xshot"), "A", "2026-11-01T00:00:00+00:00");
    write_png(&e.path("b.png"), 8, 8);
    let core = Core::for_tests_with(
        Platform::Windows,
        vec![extension_with(Some(e.root.path().to_path_buf()))],
    );
    library(&core).scan();
    let cli = |id: &str, p: Value| core.call(Source::Cli, id, p, Some(Lang::En));
    let list = cli("library.list", json!({})).unwrap();
    assert_eq!(list["total"], 2);
    // The loose PNG has no capture date: its modified time (now) makes it the newest.
    assert_eq!(list["items"][0]["name"], "b");
    assert_eq!(list["items"][1]["name"], "a");
    assert_eq!(list["items"][1]["captured"], "2026-11-01T00:00:00+00:00");
    let page = cli("library.list", json!({ "offset": 1, "limit": 1 })).unwrap();
    assert_eq!(page["items"].as_array().unwrap().len(), 1);
    assert_eq!(page["items"][0]["name"], "a");
    let id = list["items"][0]["id"].as_str().unwrap().to_string();
    assert_eq!(cli("library.get", json!({ "id": id })).unwrap()["format"], "png");
    let missing = cli("library.get", json!({ "id": "nope" })).unwrap_err();
    assert_eq!(missing.code, ErrorCode::NotFound);
    let deleted = cli("library.delete", json!({ "ids": [id] })).unwrap();
    assert_eq!(deleted["deleted"], 1);
    assert_eq!(cli("library.list", json!({})).unwrap()["total"], 1);
    let undo = deleted["undo"].as_str().unwrap();
    assert_eq!(
        cli("library.restore", json!({ "undo": undo })).unwrap()["restored"],
        1
    );
    assert_eq!(cli("library.list", json!({})).unwrap()["total"], 2);
}

#[test]
fn files_outside_the_library_can_be_deleted_by_path_and_brought_back() {
    let e = env();
    write_png(&e.path("in.png"), 4, 4);
    let outside = tempfile::tempdir().unwrap();
    let other = outside.path().join("other.png");
    write_png(&other, 6, 6);
    let core = Core::for_tests_with(
        Platform::Windows,
        vec![extension_with(Some(e.root.path().to_path_buf()))],
    );
    library(&core).scan();
    let cli = |id: &str, p: Value| core.call(Source::Cli, id, p, Some(Lang::En));
    let r = cli("library.delete", json!({ "paths": [e.path("in.png"), other] })).unwrap();
    assert_eq!(r["deleted"], 2);
    assert!(!other.exists() && !e.path("in.png").exists());
    assert_eq!(cli("library.list", json!({})).unwrap()["total"], 0);
    cli("library.restore", json!({ "undo": r["undo"] })).unwrap();
    assert!(other.exists(), "the outside file is back");
    assert_eq!(cli("library.list", json!({})).unwrap()["total"], 1);
    let none = cli("library.delete", json!({})).unwrap_err();
    assert_eq!(none.code, ErrorCode::InvalidParams);
    let gone = cli(
        "library.delete",
        json!({ "paths": [outside.path().join("nope.png")] }),
    )
    .unwrap_err();
    assert_eq!(gone.code, ErrorCode::NotFound);
}

fn write_xshot_with_text(path: &Path, text: &str) {
    let mut document = Document::new(400, 300, 96.0);
    document.objects.push(xs_core::document::Object {
        id: "o1".into(),
        kind: "callout".into(),
        props: serde_json::from_value(json!({"text": text, "rect": {"x": 1, "y": 1, "w": 50, "h": 20}}))
            .unwrap(),
    });
    let x = Xshot {
        manifest: Manifest::current(),
        document,
        base_png: png(400, 300),
        meta: json!({ "app": "Notepad" }),
        thumb_png: png(40, 30),
        images: BTreeMap::new(),
    };
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    x.write(path).unwrap();
}

fn names(v: &Value) -> Vec<String> {
    v["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["name"].as_str().unwrap().to_string())
        .collect()
}

/// LIB-004, LIB-007: sorting by size both ways and names naturally; the search finds
/// names, tags and the text of a callout; prefixes cross fields.
#[test]
fn sorting_and_searching_the_library() {
    let e = env();
    write_png(&e.path("Captura 10.png"), 300, 300);
    write_png(&e.path("Captura 2.png"), 10, 10);
    write_xshot(
        &e.path("Factura cliente.xshot"),
        "Google Chrome 131",
        "2026-11-01T00:00:00+00:00",
    );
    write_xshot_with_text(&e.path("Notas.xshot"), "Pulsa el botón Guardar");
    let core = Core::for_tests_with(
        Platform::Windows,
        vec![extension_with(Some(e.root.path().to_path_buf()))],
    );
    library(&core).scan();
    let cli = |id: &str, p: Value| core.call(Source::Cli, id, p, Some(Lang::En)).unwrap();
    let by_name = cli("library.list", json!({"sort": "name"}));
    assert_eq!(
        names(&by_name),
        ["Captura 2", "Captura 10", "Factura cliente", "Notas"]
    );
    let small_first = cli("library.list", json!({"sort": "size", "order": "asc"}));
    assert_eq!(names(&small_first)[0], "Captura 2");
    let large_first = cli("library.list", json!({"sort": "size"}));
    assert_eq!(names(&large_first).last().unwrap(), "Captura 2");
    let all = cli("library.list", json!({}));
    let id_of = |name: &str| {
        all["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["name"] == name)
            .unwrap()["id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    cli(
        "library.tag",
        json!({"ids": [id_of("Captura 10")], "add": ["facturas", "Cliente"]}),
    );
    cli(
        "library.tag",
        json!({"ids": [id_of("Factura cliente")], "add": "cliente"}),
    );
    let q = |s: &str| names(&cli("library.list", json!({"query": s, "sort": "name"})));
    assert_eq!(q("factura"), ["Captura 10", "Factura cliente"]);
    assert_eq!(q("boton guardar"), ["Notas"]);
    assert_eq!(q("app:chrome tag:cliente"), ["Factura cliente"]);
    let none = cli("library.list", json!({"query": "zzz"}));
    assert_eq!(
        (none["count"].as_u64(), none["total"].as_u64()),
        (Some(0), Some(4))
    );
}

/// LIB-009, LIB-010, LIB-011: tags on several items, counted by the filter; case does
/// not duplicate; tagging and favorites keep the dates; rename, merge and delete;
/// marks survive a rebuilt index.
#[test]
fn tags_and_favorites() {
    let e = env();
    for i in 0..5 {
        write_png(&e.path(&format!("c{i}.png")), 8, 8);
    }
    let core = Core::for_tests_with(
        Platform::Windows,
        vec![extension_with(Some(e.root.path().to_path_buf()))],
    );
    library(&core).scan();
    let cli = |id: &str, p: Value| core.call(Source::Cli, id, p, Some(Lang::En));
    let list = cli("library.list", json!({"sort": "name"})).unwrap();
    let ids: Vec<String> = list["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["id"].as_str().unwrap().to_string())
        .collect();
    let modified = list["items"][0]["modified"].clone();
    cli("library.tag", json!({"ids": ids, "add": ["Cliente A"]})).unwrap();
    let tagged = cli("library.list", json!({"filters": {"tags": ["Cliente A"]}})).unwrap();
    assert_eq!(tagged["count"], 5);
    cli("library.tag", json!({"ids": &ids[..2], "remove": ["cliente a"]})).unwrap();
    let tags = cli("library.tags", json!({})).unwrap();
    assert_eq!(tags["tags"][0], json!({"tag": "Cliente A", "count": 3}));
    // Another case is the same tag.
    let r = cli("library.tag", json!({"ids": [&ids[4]], "add": ["cliente a"]})).unwrap();
    assert_eq!(r["items"][0]["tags"], json!(["Cliente A"]));
    // Too long a tag is refused.
    assert!(cli("library.tag", json!({"ids": [&ids[0]], "add": ["x".repeat(51)]})).is_err());
    // Rename; onto an existing tag only with merge, without counting twice.
    cli(
        "library.tags",
        json!({"action": "rename", "tag": "cliente a", "to": "Proyecto X"}),
    )
    .unwrap();
    cli("library.tag", json!({"ids": [&ids[0], &ids[2]], "add": ["Bug"]})).unwrap();
    cli("library.tag", json!({"ids": [&ids[2]], "add": ["Error"]})).unwrap();
    let refused = cli(
        "library.tags",
        json!({"action": "rename", "tag": "Bug", "to": "error"}),
    )
    .unwrap_err();
    assert_eq!(refused.details.unwrap()["reason"], "exists");
    cli(
        "library.tags",
        json!({"action": "merge", "tag": "Bug", "into": "Error"}),
    )
    .unwrap();
    let tags = cli("library.tags", json!({"sort": "count"})).unwrap();
    assert_eq!(
        tags["tags"],
        json!([{"tag": "Proyecto X", "count": 3}, {"tag": "Error", "count": 2}])
    );
    let deleted = cli("library.tags", json!({"action": "delete", "tag": "proyecto x"})).unwrap();
    assert_eq!(deleted["changed"], 3);
    // Favorites: three marked; marking all of them again unmarks.
    let three = &ids[..3];
    assert_eq!(
        cli("library.favorite", json!({"ids": three})).unwrap()["favorite"],
        true
    );
    let favs = cli("library.list", json!({"filters": {"favorites": true}})).unwrap();
    assert_eq!(favs["count"], 3);
    assert_eq!(favs["facets"]["favorites"], 3);
    // Dates stay.
    let again = cli("library.list", json!({"sort": "name"})).unwrap();
    assert_eq!(again["items"][0]["modified"], modified);
    // A rebuilt index keeps tags and favorites (LIB-011, LIB-022).
    drop(core);
    for suffix in ["", "-journal", "-wal", "-shm"] {
        let _ = std::fs::remove_file(e.path(&format!("library.db{suffix}")));
    }
    let lib = e.open();
    lib.scan();
    assert_eq!(lib.list().iter().filter(|i| i.favorite).count(), 3);
    assert_eq!(
        lib.list()
            .iter()
            .filter(|i| i.tags.contains(&"Error".to_string()))
            .count(),
        2
    );
}

/// LIB-023: rebuilding the index keeps IDs, tags and favorites; Check finds a
/// truncated file and a sidecar without its file, and cleans the sidecar.
#[test]
fn rebuild_and_check() {
    let e = env();
    write_xshot(&e.path("a.xshot"), "Chrome", "2026-11-01T00:00:00+00:00");
    write_png(&e.path("b.png"), 8, 8);
    let core = Core::for_tests_with(
        Platform::Windows,
        vec![extension_with(Some(e.root.path().to_path_buf()))],
    );
    library(&core).scan();
    let cli = |id: &str, p: Value| core.call(Source::Cli, id, p, Some(Lang::En)).unwrap();
    let list = cli("library.list", json!({"sort": "name"}));
    let a = list["items"][0]["id"].as_str().unwrap().to_string();
    assert_eq!(list["items"][0]["app"], "Chrome", "the app's name from CAP-056");
    cli("library.tag", json!({"ids": [a], "add": ["Cliente"]}));
    cli("library.favorite", json!({"ids": [a]}));
    assert_eq!(cli("library.rebuild", json!({}))["items"], 2);
    let again = cli("library.list", json!({"sort": "name"}));
    assert_eq!(again["items"][0]["id"], a);
    assert_eq!(again["items"][0]["tags"], json!(["Cliente"]));
    assert_eq!(again["items"][0]["favorite"], true);
    // A truncated capture and a lonely sidecar.
    std::fs::write(e.path("broken.xshot"), b"PK\x03\x04 not a zip").unwrap();
    std::fs::write(e.path("gone.png.xmeta"), br#"{"id":"x"}"#).unwrap();
    let report = cli("library.check", json!({}));
    assert_eq!(report["damaged"].as_array().unwrap().len(), 1);
    assert_eq!(report["orphans"], 1);
    assert_eq!(report["writable"], true);
    let cleaned = cli("library.check", json!({"action": "clean"}));
    assert_eq!(
        (cleaned["cleaned"].as_u64(), cleaned["orphans"].as_u64()),
        (Some(1), Some(0))
    );
    // The damaged file does not stop the rest.
    assert_eq!(cli("library.rebuild", json!({}))["items"], 3);
}

/// LIB-034: `library search --tag cliente --type image` returns only images with that
/// tag, with ID, name and path; app, site and a range of days; reveal by path.
#[test]
fn search_by_fields_and_reveal() {
    let e = env();
    write_xshot(
        &e.path("2026-11/a.xshot"),
        "Google Chrome",
        "2026-11-03T10:00:00+00:00",
    );
    write_xshot(&e.path("2026-10/b.xshot"), "Code", "2026-10-20T10:00:00+00:00");
    write_png(&e.path("c.png"), 8, 8);
    let core = Core::for_tests_with(
        Platform::Windows,
        vec![extension_with(Some(e.root.path().to_path_buf()))],
    );
    library(&core).scan();
    let cli = |id: &str, p: Value| core.call(Source::Cli, id, p, Some(Lang::En)).unwrap();
    let all = cli("library.list", json!({"sort": "name"}));
    let ids: Vec<String> = all["items"]
        .as_array()
        .unwrap()
        .iter()
        .map(|i| i["id"].as_str().unwrap().to_string())
        .collect();
    cli(
        "library.tag",
        json!({"ids": [&ids[0], &ids[2]], "add": ["Cliente"]}),
    );
    let r = cli("library.search", json!({"tag": "cliente", "type": "image"}));
    assert_eq!(r["count"], 2);
    let first = &r["items"][0];
    assert!(first["id"].is_string() && first["name"].is_string() && first["path"].is_string());
    let names = |v: &Value| -> Vec<String> {
        v["items"]
            .as_array()
            .unwrap()
            .iter()
            .map(|i| i["name"].as_str().unwrap().to_string())
            .collect()
    };
    assert_eq!(names(&cli("library.search", json!({"app": "chrome"}))), ["a"]);
    assert_eq!(
        names(&cli(
            "library.search",
            json!({"site": "example.com", "sort": "name"})
        )),
        ["a", "b"]
    );
    assert_eq!(
        names(&cli(
            "library.search",
            json!({"from": "2026-10-01", "to": "2026-10-31"})
        )),
        ["b"]
    );
    let shown = cli("library.reveal", json!({"path": e.path("2026-10/b.xshot")}));
    assert_eq!(shown["name"], "b");
    assert!(core
        .call(
            Source::Cli,
            "library.reveal",
            json!({"id": "nope"}),
            Some(Lang::En)
        )
        .is_err());
}

/// LIB-029: a tag added on another computer arrives through the item's sidecar; the
/// last change wins. Listing the library never reads a file only in the cloud.
#[test]
fn tags_from_another_computer_arrive_by_the_sidecar() {
    let e = env();
    write_png(&e.path("a.png"), 8, 8);
    let lib = e.open();
    lib.scan();
    let item = lib.list().remove(0);
    let file = e.path("a.png");
    let mut meta = xmeta::read(&file).unwrap();
    meta.tags = vec!["Desde B".into()];
    meta.favorite = true;
    xmeta::write(&file, &meta).unwrap();
    assert!(lib.refresh_marks(&[xmeta::path_of(&file)]));
    let now = lib.get(&item.id).unwrap();
    assert_eq!(
        (now.tags.clone(), now.favorite),
        (vec!["Desde B".to_string()], true)
    );
    assert!(
        !lib.refresh_marks(&[xmeta::path_of(&file)]),
        "nothing new the second time"
    );
    assert!(!now.cloud);
}

/// LIB-029: a conflict copy beside its original shows as a conflict; Resolve keeps
/// both, this copy (taking the original's name) or the original.
#[test]
fn sync_conflict_copies_are_resolved() {
    let e = env();
    write_png(&e.path("Informe.png"), 4, 4);
    write_png(&e.path("Informe (Ana's conflicted copy 2026-11-12).png"), 6, 6);
    write_png(&e.path("Plan.png"), 4, 4);
    write_png(&e.path("Plan-LAPTOP-ANA.png"), 8, 8);
    write_png(&e.path("Solo (Ana's conflicted copy).png"), 4, 4);
    let core = Core::for_tests_with(
        Platform::Windows,
        vec![extension_with(Some(e.root.path().to_path_buf()))],
    );
    library(&core).scan();
    let cli = |id: &str, p: Value| core.call(Source::Cli, id, p, Some(Lang::En)).unwrap();
    let list = cli("library.list", json!({"sort": "name"}));
    let by_name = |name: &str| {
        list["items"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["name"] == name)
            .unwrap()
            .clone()
    };
    let copy = by_name("Informe (Ana's conflicted copy 2026-11-12)");
    assert_eq!(copy["conflict"], true);
    assert_eq!(copy["conflictOf"], by_name("Informe")["id"]);
    assert_eq!(by_name("Plan-LAPTOP-ANA")["conflict"], true);
    assert!(
        by_name("Solo (Ana's conflicted copy)")["conflict"].is_null(),
        "no original beside it"
    );
    // Keep this copy: it takes the original's name, the original goes.
    let r = cli("library.resolve", json!({"id": copy["id"], "keep": "this"}));
    assert_eq!(r["kept"], copy["id"]);
    assert!(e.path("Informe.png").exists());
    assert!(!e.path("Informe (Ana's conflicted copy 2026-11-12).png").exists());
    assert_eq!(
        library(&core).get(copy["id"].as_str().unwrap()).unwrap().name,
        "Informe"
    );
    // Keep both: the copy gets a name of its own and is no longer a conflict.
    let plan = by_name("Plan-LAPTOP-ANA");
    cli("library.resolve", json!({"id": plan["id"], "keep": "both"}));
    assert!(e.path("Plan (2).png").exists() && e.path("Plan.png").exists());
    let again = cli("library.list", json!({}));
    assert!(again["items"]
        .as_array()
        .unwrap()
        .iter()
        .all(|i| i["conflict"].is_null()));
}

/// LIB-012: details bring the capture's app, version and website from its metadata,
/// even after the image changed; only web addresses open.
#[test]
fn details_come_from_the_capture_metadata() {
    let e = env();
    write_xshot(&e.path("a.xshot"), "Google Chrome", "2026-11-03T10:00:00+00:00");
    let core = Core::for_tests_with(
        Platform::Windows,
        vec![extension_with(Some(e.root.path().to_path_buf()))],
    );
    library(&core).scan();
    let cli = |id: &str, p: Value| core.call(Source::Cli, id, p, Some(Lang::En)).unwrap();
    let id = cli("library.list", json!({}))["items"][0]["id"].clone();
    let plain = cli("library.get", json!({"id": id}));
    assert!(plain["details"].is_null());
    let d = cli("library.get", json!({"id": id, "details": true}))["details"].clone();
    assert_eq!(d["app"], "Google Chrome");
    assert_eq!(d["appVersion"], "1.0");
    assert_eq!(d["url"], "https://example.com/a");
    assert_eq!(d["capturedAt"], "2026-11-03T10:00:00+00:00");
    assert!(d["modified"].is_string());
    for bad in [
        "javascript:alert(1)",
        "file:///C:/x",
        "https://a.com/\" & calc",
        "ftp://x",
    ] {
        assert!(service::open_url(bad).is_err(), "{bad}");
    }
}

/// LIB-024: importing copies files and folders into the folder of their month; a
/// folder's name becomes their tag; a photo's EXIF date files it under its year; the
/// originals stay; files already in the library are skipped.
#[test]
fn import_copies_into_the_month_with_the_folder_tag() {
    let e = env();
    let outside = tempfile::tempdir().unwrap();
    let project = outside.path().join("Proyecto X");
    for n in 1..=3 {
        write_png(&project.join(format!("p{n}.png")), 8, 8);
    }
    write_png(&project.join("deeper").join("p4.png"), 8, 8);
    std::fs::write(project.join("notes.txt"), "not an image").unwrap();
    let photo = outside.path().join("old.jpg");
    std::fs::write(&photo, crate::exif::tests::jpeg_with_date("2019:06:15 10:30:00")).unwrap();
    write_png(&e.path("2026-11/inside.png"), 8, 8);
    let core = Core::for_tests_with(
        Platform::Windows,
        vec![extension_with(Some(e.root.path().to_path_buf()))],
    );
    library(&core).scan();
    let cli = |id: &str, p: Value| core.call(Source::Cli, id, p, Some(Lang::En));
    let r = cli(
        "library.import",
        json!({"paths": [project, photo, e.path("2026-11/inside.png")]}),
    )
    .unwrap();
    assert_eq!(r["imported"].as_array().unwrap().len(), 5, "{r}");
    let reasons: Vec<&str> = r["skipped"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|s| s["reason"].as_str())
        .collect();
    assert_eq!(
        reasons,
        ["in_library"],
        "the text file is not looked at; the library file is skipped"
    );
    assert!(project.join("p1.png").exists(), "the originals stay");
    assert!(r["imported"]
        .as_array()
        .unwrap()
        .iter()
        .all(|i| i["id"].is_string()));

    let tagged = cli("library.list", json!({"filters": {"tags": ["Proyecto X"]}})).unwrap();
    assert_eq!(tagged["count"], 4, "the folder's images, subfolders too");
    let old = cli("library.list", json!({"filters": {"dates": ["2019"]}})).unwrap();
    assert_eq!(old["count"], 1, "the EXIF date files the photo under 2019");
    let path = old["items"][0]["path"].as_str().unwrap().replace('\\', "/");
    assert!(path.contains("/2019-06/"), "in the folder of its month: {path}");

    // Again, without the tag: new copies with their own names.
    let again = cli("library.import", json!({"paths": [project], "tagFolders": false})).unwrap();
    assert_eq!(again["imported"].as_array().unwrap().len(), 4);
    let tagged = cli("library.list", json!({"filters": {"tags": ["Proyecto X"]}})).unwrap();
    assert_eq!(tagged["count"], 4);
    assert!(cli("library.import", json!({"paths": []})).is_err());
}

/// LIB-028: Retry moves the files a move left behind (with their sidecars), and the
/// folder check says how big the library is and whether there is room.
#[test]
fn retry_moves_what_was_left_and_the_check_measures_room() {
    let e = env();
    let to = tempfile::tempdir().unwrap();
    write_png(&e.path("2026-11/left.png"), 8, 8);
    std::fs::write(e.path("2026-11/left.png.xmeta"), "{}").unwrap();
    let r = relocate::move_files(
        e.root.path(),
        to.path(),
        &[e.path("2026-11/left.png"), e.path("elsewhere.png")],
    );
    assert_eq!(r.moved, 1);
    assert_eq!(r.failed.len(), 1, "a file outside the old library is not moved");
    assert!(to.path().join("2026-11").join("left.png").exists());
    assert!(
        to.path().join("2026-11").join("left.png.xmeta").exists(),
        "with its sidecar"
    );

    write_png(&e.path("a.png"), 8, 8);
    let core = Core::for_tests_with(
        Platform::Windows,
        vec![extension_with(Some(e.root.path().to_path_buf()))],
    );
    library(&core).scan();
    let target = tempfile::tempdir().unwrap();
    let i = core
        .call(
            Source::Ui,
            "library.location",
            json!({"action": "inspect", "path": target.path()}),
            Some(Lang::En),
        )
        .unwrap();
    assert!(i["size"].as_u64().unwrap() > 0);
    assert_eq!(i["enough"], json!(true), "{i}");
    assert_eq!(i["bigSync"], json!(false));
}

/// A small PNG whose pixels depend on `n`, so each `n` is a different file.
fn write_png_n(path: &Path, n: u32) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let img = image::RgbaImage::from_pixel(4, 4, image::Rgba([(n % 251) as u8, (n / 251) as u8, 7, 255]));
    std::fs::write(path, xs_image::encode::png(&img, 0.0, true).unwrap()).unwrap();
}

/// LIB-031: merging a library of 300 items, 20 of them the same as items already
/// here, adds 280; "cliente" joins "Cliente"; the other folder keeps all its files; a
/// taken name gets " (2)"; a duplicate brings its tags and favorite.
#[test]
fn merging_another_library_copies_what_is_new() {
    let e = env();
    let other = tempfile::tempdir().unwrap();
    // Here: 20 items, one tagged "Cliente".
    for n in 0..20 {
        write_png_n(&e.path(&format!("2026-10/here{n}.png")), n);
    }
    let core = Core::for_tests_with(
        Platform::Windows,
        vec![extension_with(Some(e.root.path().to_path_buf()))],
    );
    let lib = library(&core);
    lib.scan();
    let first = lib.find(&e.path("2026-10/here0.png")).unwrap();
    lib.tag(std::slice::from_ref(&first.id), &["Cliente".to_string()], &[])
        .unwrap();

    // There: 280 new items and the same 20 under other names; one new item tagged
    // "cliente", one duplicate with a tag and the favorite, one new item whose name is
    // taken here by a different file.
    for n in 100..380 {
        write_png_n(&other.path().join(format!("2026-11/there{n}.png")), n);
    }
    for n in 0..20 {
        write_png_n(&other.path().join(format!("old/copy{n}.png")), n);
    }
    let side = |file: &Path, id: &str, tags: &[&str], favorite: bool| {
        xmeta::write(
            file,
            &xmeta::Xmeta {
                id: id.to_string(),
                tags: tags.iter().map(|t| t.to_string()).collect(),
                favorite,
                ..xmeta::Xmeta::default()
            },
        )
        .unwrap();
    };
    side(
        &other.path().join("2026-11/there100.png"),
        "other-100",
        &["cliente"],
        false,
    );
    side(
        &other.path().join("old/copy5.png"),
        "other-c5",
        &["Proyecto X"],
        true,
    );
    // here1.png exists here with other pixels.
    write_png_n(&other.path().join("2026-10/here1.png"), 999);
    let before: Vec<(PathBuf, u64)> = walk_files(other.path());

    let cli = |id: &str, p: Value| core.call(Source::Cli, id, p, Some(Lang::En));
    let r = cli("library.merge", json!({"path": other.path()})).unwrap();
    assert_eq!(r["added"], 281, "280 new and the one with a taken name: {r}");
    assert_eq!(r["duplicates"], 20, "{r}");
    assert_eq!(r["enriched"], 1, "{r}");
    assert_eq!(r["cancelled"], false);
    let renamed = r["renamed"].as_array().unwrap();
    assert_eq!(renamed.len(), 1);
    assert!(
        renamed[0]["path"].as_str().unwrap().ends_with("here1 (2).png"),
        "{r}"
    );
    assert_eq!(lib.list().len(), 20 + 281);

    // One "Cliente" tag with both items, not two tags.
    let tags: Vec<String> = lib.tags().into_iter().map(|(t, _)| t).collect();
    assert!(tags.contains(&"Cliente".to_string()));
    assert!(!tags.contains(&"cliente".to_string()), "{tags:?}");
    let tagged = cli("library.list", json!({"filters": {"tags": ["Cliente"]}})).unwrap();
    assert_eq!(tagged["count"], 2);
    // The new item kept its ID; the duplicate's tag and favorite went to the item here.
    assert!(lib.get("other-100").is_some());
    let five = lib.find(&e.path("2026-10/here5.png")).unwrap();
    assert_eq!(five.tags, vec!["Proyecto X".to_string()]);
    assert!(five.favorite);

    // The other library is as it was.
    assert_eq!(walk_files(other.path()), before);

    // Merging it again copies nothing.
    let again = cli("library.merge", json!({"path": other.path()})).unwrap();
    assert_eq!(again["added"], 0, "{again}");
    assert_eq!(again["duplicates"], 301);

    // Not this library itself, nor a folder inside it.
    assert!(cli("library.merge", json!({"path": e.path("2026-10")})).is_err());
}

/// LIB-031: Cancel removes what the merge copied; nothing else changes.
#[test]
fn a_cancelled_merge_leaves_the_library_as_it_was() {
    let e = env();
    let other = tempfile::tempdir().unwrap();
    for n in 0..5 {
        write_png_n(&other.path().join(format!("m/x{n}.png")), n);
    }
    write_png_n(&e.path("keep.png"), 77);
    let lib = e.open();
    lib.scan();
    let before = walk_files(e.root.path());
    let cancel = std::sync::atomic::AtomicBool::new(false);
    let r = merge::merge(&lib, other.path(), &cancel, |done, _| {
        if done == 3 {
            cancel.store(true, std::sync::atomic::Ordering::Relaxed);
        }
    });
    assert!(r.cancelled);
    assert_eq!(walk_files(e.root.path()), before, "the copies are gone");
    assert!(!e.path("m").exists(), "and the folder they needed");
    assert_eq!(lib.list().len(), 1);
    assert_eq!(walk_files(other.path()).len(), 5);
}

/// LIB-010: a tag that arrives in a sidecar joins the spelling the library has.
#[test]
fn tags_from_a_sidecar_join_the_existing_spelling() {
    let e = env();
    write_png_n(&e.path("a.png"), 1);
    let lib = e.open();
    lib.scan();
    let a = lib.find(&e.path("a.png")).unwrap();
    lib.tag(std::slice::from_ref(&a.id), &["Cliente".to_string()], &[])
        .unwrap();
    let b = e.path("b.png");
    write_png_n(&b, 2);
    xmeta::write(
        &b,
        &xmeta::Xmeta {
            id: "from-elsewhere".into(),
            tags: vec!["CLIENTE".into(), "cliente".into()],
            ..xmeta::Xmeta::default()
        },
    )
    .unwrap();
    lib.scan();
    let item = lib.get("from-elsewhere").unwrap();
    assert_eq!(item.tags, vec!["Cliente".to_string()]);
    assert_eq!(
        xmeta::read(&b).unwrap().tags,
        vec!["Cliente".to_string()],
        "the sidecar too"
    );
}

/// Every file under `dir` (sidecars too) with its size, sorted.
fn walk_files(dir: &Path) -> Vec<(PathBuf, u64)> {
    let mut out = Vec::new();
    let mut dirs = vec![dir.to_path_buf()];
    while let Some(d) = dirs.pop() {
        for e in std::fs::read_dir(&d).unwrap().flatten() {
            let p = e.path();
            if p.is_dir() {
                dirs.push(p);
            } else {
                out.push((p.clone(), e.metadata().unwrap().len()));
            }
        }
    }
    out.sort();
    out
}

/// Copies a folder with everything in it.
fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap().flatten() {
        let p = e.path();
        let target = to.join(e.file_name());
        if p.is_dir() {
            copy_dir(&p, &target);
        } else {
            std::fs::copy(&p, &target).unwrap();
        }
    }
}

/// The names of the files under `dir`, relative, without the index.
fn names_in(dir: &Path) -> Vec<String> {
    walk_files(dir)
        .into_iter()
        .map(|(p, _)| p.strip_prefix(dir).unwrap().to_string_lossy().replace('\\', "/"))
        .filter(|n| !n.starts_with("library.db"))
        .collect()
}

/// LIB-032, APP-055: a library folder copied to another computer (another place, another
/// app data folder) shows every item with its ID, tags and favorite, with or without
/// its index; no file is renamed, duplicated or added.
#[test]
fn a_copied_library_works_on_another_computer() {
    let e = env();
    write_png_n(&e.path("2026-10/a.png"), 1);
    write_xshot(&e.path("2026-11/b.xshot"), "Excel", "2026-11-02T10:00:00+00:00");
    write_png_n(&e.path("c.png"), 3);
    let lib = e.open();
    lib.scan();
    let a = lib.find(&e.path("2026-10/a.png")).unwrap();
    let b = lib.find(&e.path("2026-11/b.xshot")).unwrap();
    lib.tag(std::slice::from_ref(&a.id), &["Cliente".to_string()], &[])
        .unwrap();
    lib.set_favorite(std::slice::from_ref(&b.id), Some(true)).unwrap();
    let marks = |lib: &Library| {
        let mut v: Vec<(String, String, Vec<String>, bool)> = lib
            .list()
            .into_iter()
            .map(|i| (i.path, i.id, i.tags, i.favorite))
            .collect();
        v.sort();
        v
    };
    let before = marks(&lib);
    drop(lib);

    for keep_index in [true, false] {
        let elsewhere = tempfile::tempdir().unwrap();
        let root = elsewhere.path().join("Mi biblioteca");
        copy_dir(e.root.path(), &root);
        if !keep_index {
            for n in ["library.db", "library.db-wal", "library.db-shm"] {
                let _ = std::fs::remove_file(root.join(n));
            }
        }
        let files = names_in(&root);
        let data = tempfile::tempdir().unwrap();
        let lib = Library::open(root.clone(), data.path()).holding_deletes();
        lib.scan();
        assert_eq!(marks(&lib), before, "index kept: {keep_index}");
        assert_eq!(names_in(&root), files, "nothing renamed or added");
    }
}
