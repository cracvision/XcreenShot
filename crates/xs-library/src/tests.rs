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
        meta: json!({ "capturedAt": captured, "app": app, "url": "https://example.com/a" }),
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
