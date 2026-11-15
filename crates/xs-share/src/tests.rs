//! The local network between two XcreenShot instances in this process (SHR-042 to
//! SHR-044): over 127.0.0.1 and a pretend discovery, so nothing opens towards the
//! network.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use xs_core::keychain::MemoryVault;
use xs_core::platform::Platform;
use xs_core::{Core, Lang, Source};

use crate::discovery::MemoryNet;
use crate::lan::{Env, EnvService};

struct Peer {
    core: Arc<Core>,
    library: tempfile::TempDir,
}

fn peer(net: &MemoryNet, name: &str) -> Peer {
    let library = tempfile::tempdir().unwrap();
    let core = Core::for_tests_with(
        Platform::Windows,
        vec![
            xs_library::extension_with(Some(library.path().to_path_buf())),
            crate::extension(),
        ],
    );
    core.replace_service(EnvService(Arc::new(Env {
        vault: Arc::new(MemoryVault::default()),
        discovery: Arc::new(net.join()),
        bind: std::net::IpAddr::from([127, 0, 0, 1]),
        retry: Duration::from_millis(200),
    })));
    let p = Peer { core, library };
    p.call("settings.set", json!({"key": "share.lanName", "value": name}));
    p
}

impl Peer {
    fn call(&self, id: &str, p: Value) -> Value {
        self.core
            .call(Source::Ui, id, p, Some(Lang::En))
            .unwrap_or_else(|e| panic!("{id}: {}", e.message))
    }

    fn on(&self, on: bool) {
        self.call("settings.set", json!({"key": "share.lan", "value": on}));
        wait(|| self.call("share.lan_status", json!({}))["running"] == on);
    }

    fn fingerprint(&self) -> String {
        self.call("share.lan_status", json!({}))["fingerprint"]
            .as_str()
            .unwrap()
            .to_string()
    }

    fn sees(&self, other: &Peer) -> bool {
        self.sees_fp(&other.fingerprint())
    }

    fn sees_fp(&self, fp: &str) -> bool {
        self.call("share.lan_people", json!({}))["people"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["fingerprint"] == fp && p["online"] == true)
    }

    fn library_files(&self) -> Vec<PathBuf> {
        fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
            for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
                let p = e.path();
                if p.is_dir() {
                    walk(&p, out);
                } else if p.extension().is_some_and(|x| x == "png") {
                    out.push(p);
                }
            }
        }
        let mut out = Vec::new();
        walk(self.library.path(), &mut out);
        out
    }

    fn send_status(&self, id: &str) -> String {
        self.call("share.lan_sends", json!({}))["sends"]
            .as_array()
            .unwrap()
            .iter()
            .find(|s| s["id"] == id)
            .map(|s| s["status"].as_str().unwrap_or_default().to_string())
            .unwrap_or_default()
    }
}

fn wait(mut f: impl FnMut() -> bool) {
    let until = Instant::now() + Duration::from_secs(15);
    while !f() {
        assert!(Instant::now() < until, "timed out");
        std::thread::sleep(Duration::from_millis(50));
    }
}

/// Pairs `a` with `b`, both accepting.
fn pair(a: &Peer, b: &Peer) {
    let r = a.call("share.lan_pair", json!({"fingerprint": b.fingerprint()}));
    let code = r["code"].as_str().unwrap().to_string();
    let mut theirs = Value::Null;
    wait(|| {
        theirs = b.call("share.lan_pairings", json!({}))["pairings"][0].clone();
        !theirs.is_null()
    });
    assert_eq!(theirs["code"], code.as_str(), "the same code on both");
    assert_eq!(theirs["direction"], "in");
    b.call(
        "share.lan_pair_answer",
        json!({"id": theirs["id"], "accept": true}),
    );
    a.call("share.lan_pair_answer", json!({"id": r["id"], "accept": true}));
    let contact = |p: &Peer, of: &Peer| {
        let fp = of.fingerprint();
        p.call("share.lan_people", json!({}))["people"]
            .as_array()
            .unwrap()
            .iter()
            .any(|x| x["fingerprint"] == fp.as_str() && x["contact"] == true)
    };
    wait(|| contact(a, b) && contact(b, a));
}

fn picture(dir: &Path) -> (PathBuf, Vec<u8>) {
    // Any bytes do for the transfer; a real PNG for the library.
    let mut png = Vec::new();
    {
        let img = xs_core_png();
        png.extend_from_slice(&img);
    }
    let path = dir.join("captura.png");
    std::fs::write(&path, &png).unwrap();
    (path, png)
}

/// A tiny valid PNG (1×1, red).
fn xs_core_png() -> Vec<u8> {
    vec![
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52, 0x00,
        0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00, 0x00, 0x90, 0x77, 0x53, 0xde, 0x00,
        0x00, 0x00, 0x0c, 0x49, 0x44, 0x41, 0x54, 0x08, 0xd7, 0x63, 0xf8, 0xcf, 0xc0, 0x00, 0x00, 0x03, 0x01,
        0x01, 0x00, 0x18, 0xdd, 0x8d, 0xb0, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60,
        0x82,
    ]
}

#[test]
fn off_announces_nothing_and_listens_nowhere() {
    let net = MemoryNet::default();
    let a = peer(&net, "Ana");
    let s = a.call("share.lan_status", json!({}));
    assert_eq!(s["running"], false);
    assert!(s["port"].is_null());
    let err = a
        .core
        .call(
            Source::Ui,
            "share.lan_pair",
            json!({"fingerprint": "x"}),
            Some(Lang::En),
        )
        .unwrap_err();
    assert_eq!(err.code, xs_core::ErrorCode::FeatureDisabled);
}

#[test]
fn pair_send_and_receive_into_the_library() {
    let net = MemoryNet::default();
    let a = peer(&net, "Ana");
    let b = peer(&net, "Beto");
    a.on(true);
    b.on(true);
    wait(|| a.sees(&b) && b.sees(&a));
    pair(&a, &b);
    let dir = tempfile::tempdir().unwrap();
    let (path, bytes) = picture(dir.path());
    let sent = a.call(
        "share.lan_send",
        json!({"to": [b.fingerprint()], "paths": [path.to_string_lossy()]}),
    );
    let id = sent["sends"][0]["id"].as_str().unwrap().to_string();
    let mut offer = Value::Null;
    wait(|| {
        offer = b.call("share.lan_incoming", json!({}))["incoming"][0].clone();
        !offer.is_null()
    });
    assert_eq!(offer["name"], "Ana");
    assert_eq!(offer["items"][0]["name"], "captura.png");
    b.call("share.lan_answer", json!({"id": offer["id"], "accept": true}));
    wait(|| a.send_status(&id) == "delivered");
    let files = b.library_files();
    assert_eq!(files.len(), 1);
    assert_eq!(std::fs::read(&files[0]).unwrap(), bytes, "the same content");
    let tagged = b.call("library.list", json!({"filters": {"tags": ["From: Ana"]}}));
    assert_eq!(tagged["count"], 1);

    // Rejected: nothing stays.
    let sent = a.call(
        "share.lan_send",
        json!({"to": [b.fingerprint()], "paths": [path.to_string_lossy()]}),
    );
    let id = sent["sends"][0]["id"].as_str().unwrap().to_string();
    wait(|| !b.call("share.lan_incoming", json!({}))["incoming"][0].is_null());
    let offer = b.call("share.lan_incoming", json!({}))["incoming"][0].clone();
    b.call("share.lan_answer", json!({"id": offer["id"], "accept": false}));
    wait(|| a.send_status(&id) == "rejected");
    assert_eq!(b.library_files().len(), 1);
}

#[test]
fn pending_until_the_other_is_back_and_always_accept() {
    let net = MemoryNet::default();
    let a = peer(&net, "Ana");
    let b = peer(&net, "Beto");
    a.on(true);
    b.on(true);
    wait(|| a.sees(&b));
    pair(&a, &b);
    b.call(
        "share.lan_contact",
        json!({"fingerprint": a.fingerprint(), "action": "alwaysAccept", "value": true}),
    );
    let fp_b = b.fingerprint();
    b.on(false);
    wait(|| !a.sees_fp(&fp_b));
    let dir = tempfile::tempdir().unwrap();
    let (path, _) = picture(dir.path());
    let sent = a.call(
        "share.lan_send",
        json!({"to": [fp_b], "paths": [path.to_string_lossy()]}),
    );
    let id = sent["sends"][0]["id"].as_str().unwrap().to_string();
    std::thread::sleep(Duration::from_millis(600));
    assert_eq!(a.send_status(&id), "pending");
    b.on(true);
    wait(|| a.send_status(&id) == "delivered");
    assert_eq!(b.library_files().len(), 1, "accepted without asking");
}

#[test]
fn only_contacts_send_and_a_declined_pairing_leaves_nothing() {
    let net = MemoryNet::default();
    let a = peer(&net, "Ana");
    let b = peer(&net, "Beto");
    a.on(true);
    b.on(true);
    wait(|| a.sees(&b));
    // Declined by one: neither is the other's contact.
    let r = a.call("share.lan_pair", json!({"fingerprint": b.fingerprint()}));
    wait(|| !b.call("share.lan_pairings", json!({}))["pairings"][0].is_null());
    let theirs = b.call("share.lan_pairings", json!({}))["pairings"][0].clone();
    b.call(
        "share.lan_pair_answer",
        json!({"id": theirs["id"], "accept": false}),
    );
    a.call("share.lan_pair_answer", json!({"id": r["id"], "accept": true}));
    wait(|| {
        a.call("share.lan_pairings", json!({}))["pairings"]
            .as_array()
            .unwrap()
            .is_empty()
    });
    let contacts = |p: &Peer| {
        p.call("share.lan_people", json!({}))["people"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|x| x["contact"] == true)
            .count()
    };
    assert_eq!((contacts(&a), contacts(&b)), (0, 0));
    // Without pairing, nothing can be sent.
    let dir = tempfile::tempdir().unwrap();
    let (path, _) = picture(dir.path());
    assert!(a
        .core
        .call(
            Source::Ui,
            "share.lan_send",
            json!({"to": [b.fingerprint()], "paths": [path.to_string_lossy()]}),
            Some(Lang::En)
        )
        .is_err());
    // Paired, then removed by the other: the send never reaches them.
    pair(&a, &b);
    b.call(
        "share.lan_contact",
        json!({"fingerprint": a.fingerprint(), "action": "remove"}),
    );
    let sent = a.call(
        "share.lan_send",
        json!({"to": [b.fingerprint()], "paths": [path.to_string_lossy()]}),
    );
    let id = sent["sends"][0]["id"].as_str().unwrap().to_string();
    std::thread::sleep(Duration::from_millis(800));
    assert!(b.call("share.lan_incoming", json!({}))["incoming"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(a.send_status(&id), "pending");
    a.call("share.lan_cancel", json!({"id": id}));
    assert_eq!(a.send_status(&id), "cancelled");
}

#[test]
fn agents_need_the_preference_to_accept() {
    let net = MemoryNet::default();
    let a = peer(&net, "Ana");
    let err = a
        .core
        .call(
            Source::Mcp,
            "share.lan_answer",
            json!({"id": "r1", "accept": true}),
            Some(Lang::En),
        )
        .unwrap_err();
    assert_eq!(err.code, xs_core::ErrorCode::PermissionDenied);
    a.call(
        "settings.set",
        json!({"key": "share.lanAutomation", "value": true}),
    );
    let err = a
        .core
        .call(
            Source::Mcp,
            "share.lan_answer",
            json!({"id": "r1", "accept": true}),
            Some(Lang::En),
        )
        .unwrap_err();
    assert_eq!(
        err.code,
        xs_core::ErrorCode::NotFound,
        "allowed now; there is just nothing to answer"
    );
}
