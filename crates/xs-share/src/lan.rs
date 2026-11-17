//! The local network service (SHR-042 to SHR-044): it runs only while the preference
//! is on. It listens on a port of its own, announces itself, keeps the people it sees,
//! pairs, sends what is pending and receives what is offered.

use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{IpAddr, SocketAddr, TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, Weak};
use std::time::{Duration, Instant};

use chrono::{DateTime, Local, Utc};
use serde_json::{json, Value};
use xs_core::keychain::Vault;
use xs_core::{Core, Source};

use crate::discovery::{Announce, Discovery, Event};
use crate::identity::{self, Identity};
use crate::store::{self, Contact, Contacts, Item, Outbox, Send, Status};
use crate::tls;
use crate::wire;

/// How long a person has to answer a pairing or an offer.
const ANSWER: Duration = Duration::from_secs(180);
const OFFER_ANSWER: Duration = Duration::from_secs(600);
/// A pending send expires after (SHR-044).
const EXPIRES: chrono::Duration = chrono::Duration::days(7);
const CHUNK: usize = 64 * 1024;

/// What the service needs from the system: tests give their own.
pub struct Env {
    pub vault: Arc<dyn Vault>,
    pub discovery: Arc<dyn Discovery>,
    /// Where to listen: every interface, or 127.0.0.1 in tests.
    pub bind: IpAddr,
    /// How often pending sends are tried.
    pub retry: Duration,
}

impl Env {
    pub fn system() -> Env {
        let instance = std::env::var("XCREENSHOT_DATA_DIR").ok();
        let vault: Arc<dyn Vault> = Arc::new(xs_core::keychain::SystemVault::named(
            "lan-identity",
            instance.as_deref(),
        ));
        // Development builds: two instances through a folder, at 127.0.0.1.
        #[cfg(debug_assertions)]
        if let Some(dir) = std::env::var_os("XCREENSHOT_LAN_DIR") {
            return Env {
                vault,
                discovery: Arc::new(crate::discovery::FolderDiscovery::new(PathBuf::from(dir))),
                bind: IpAddr::from([127, 0, 0, 1]),
                retry: Duration::from_secs(3),
            };
        }
        Env {
            vault,
            discovery: Arc::new(crate::discovery::Mdns::new()),
            bind: IpAddr::from([0, 0, 0, 0]),
            retry: Duration::from_secs(15),
        }
    }
}

/// The environment in use (a core service tests replace before turning it on).
pub struct EnvService(pub Arc<Env>);

#[derive(Debug, Clone)]
struct Peer {
    announce: Announce,
    addrs: Vec<IpAddr>,
    online: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Out,
    In,
}

#[derive(Debug, Clone)]
struct Pairing {
    id: String,
    fingerprint: String,
    name: String,
    device: String,
    code: String,
    direction: Direction,
    mine: Option<bool>,
    done: bool,
}

#[derive(Debug, Clone)]
struct Incoming {
    id: String,
    fingerprint: String,
    name: String,
    items: Vec<(String, u64)>,
    total: u64,
    decision: Option<bool>,
    cancel: Arc<AtomicBool>,
    receiving: bool,
}

struct Running {
    identity: Arc<Identity>,
    port: u16,
    stop: Arc<AtomicBool>,
    discovery: Arc<dyn Discovery>,
    server: Arc<rustls::ServerConfig>,
}

pub struct Lan {
    core: Weak<Core>,
    dir: PathBuf,
    staging: PathBuf,
    running: Mutex<Option<Running>>,
    peers: Mutex<HashMap<String, Peer>>,
    contacts: Mutex<Contacts>,
    outbox: Mutex<Outbox>,
    pairings: Mutex<HashMap<String, Pairing>>,
    incoming: Mutex<HashMap<String, Incoming>>,
    /// Sends in flight, with their cancel flag.
    flying: Mutex<HashMap<String, Arc<AtomicBool>>>,
    error: Mutex<Option<String>>,
    next: AtomicU64,
}

/// The service; [`install`] makes it, with its way back to the core, at start.
pub fn lan(core: &Core) -> Arc<Lan> {
    install(core, Weak::new())
}

pub fn install(core: &Core, weak: Weak<Core>) -> Arc<Lan> {
    let dir = core.paths.config_dir.join("lan");
    let staging = core.paths.local_dir.join("lan-incoming");
    core.service(move || Lan::new(weak, dir, staging))
}

fn now() -> String {
    Utc::now().to_rfc3339()
}

fn user_name() -> String {
    std::env::var("USERNAME")
        .or_else(|_| std::env::var("USER"))
        .unwrap_or_else(|_| "XcreenShot".into())
}

fn device_name() -> String {
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "XcreenShot".into())
}

impl Lan {
    fn new(core: Weak<Core>, dir: PathBuf, staging: PathBuf) -> Lan {
        let contacts = store::read(&dir.join("contacts.json"));
        let outbox = store::read(&dir.join("outbox.json"));
        Lan {
            core,
            dir,
            staging,
            running: Mutex::new(None),
            peers: Mutex::new(HashMap::new()),
            contacts: Mutex::new(contacts),
            outbox: Mutex::new(outbox),
            pairings: Mutex::new(HashMap::new()),
            incoming: Mutex::new(HashMap::new()),
            flying: Mutex::new(HashMap::new()),
            error: Mutex::new(None),
            next: AtomicU64::new(1),
        }
    }

    fn id(&self, prefix: &str) -> String {
        format!(
            "{prefix}{:x}{:x}",
            Utc::now().timestamp_millis(),
            self.next.fetch_add(1, Ordering::Relaxed)
        )
    }

    fn core(&self) -> Option<Arc<Core>> {
        self.core.upgrade()
    }

    fn event(&self, kind: &str, payload: Value) {
        if let Some(core) = self.core() {
            let mut p = payload;
            p["kind"] = json!(kind);
            core.host().editor_event("lan", p);
        }
    }

    fn save_contacts(&self, c: &Contacts) {
        let _ = store::write(&self.dir.join("contacts.json"), c);
    }

    fn save_outbox(&self, o: &Outbox) {
        let _ = store::write(&self.dir.join("outbox.json"), o);
    }

    pub fn name(&self) -> String {
        let set = self
            .core()
            .map(|c| c.settings.get_str("share.lanName"))
            .unwrap_or_default();
        if set.trim().is_empty() {
            user_name()
        } else {
            set.trim().to_string()
        }
    }

    // -----------------------------------------------------------------------
    // On and off

    /// Starts or stops as the preference says (SHR-042).
    pub fn apply(self: &Arc<Self>) {
        let Some(core) = self.core() else { return };
        let on = core.settings.get_bool("share.lan");
        let running = self.running.lock().unwrap().is_some();
        if on && !running {
            match self.start(&core) {
                Ok(()) => *self.error.lock().unwrap() = None,
                Err(e) => {
                    tracing::warn!(error = %e, "local network could not start");
                    *self.error.lock().unwrap() = Some(e);
                }
            }
        } else if !on && running {
            self.stop();
        } else if on && running {
            // The visible name changed: announce again.
            self.stop();
            let _ = self.start(&core);
        }
        self.event("status", json!({}));
    }

    fn start(self: &Arc<Self>, core: &Core) -> Result<(), String> {
        let env = core.service(|| EnvService(Arc::new(Env::system()))).0.clone();
        let identity = Arc::new(Identity::load_or_create(env.vault.as_ref())?);
        let listener = TcpListener::bind(SocketAddr::new(env.bind, 0)).map_err(|e| e.to_string())?;
        let port = listener.local_addr().map_err(|e| e.to_string())?.port();
        listener.set_nonblocking(true).map_err(|e| e.to_string())?;
        let server = tls::server_config(&identity)?;
        let stop = Arc::new(AtomicBool::new(false));
        // Connections.
        {
            let me = self.clone();
            let stop = stop.clone();
            let server = server.clone();
            let identity = identity.clone();
            std::thread::Builder::new()
                .name("xs-lan-listen".into())
                .spawn(move || {
                    while !stop.load(Ordering::Relaxed) {
                        match listener.accept() {
                            Ok((tcp, _)) => {
                                let me = me.clone();
                                let server = server.clone();
                                let identity = identity.clone();
                                std::thread::spawn(move || {
                                    let _ = tcp.set_nonblocking(false);
                                    if let Err(e) = me.serve(server, &identity, tcp) {
                                        tracing::debug!(error = %e, "local network connection ended");
                                    }
                                });
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                std::thread::sleep(Duration::from_millis(100));
                            }
                            Err(_) => std::thread::sleep(Duration::from_millis(200)),
                        }
                    }
                })
                .map_err(|e| e.to_string())?;
        }
        // Discovery.
        let me = Arc::downgrade(self);
        env.discovery.start(
            Announce {
                fingerprint: identity.fingerprint(),
                name: self.name(),
                device: device_name(),
                port,
            },
            Arc::new(move |e| {
                if let Some(me) = me.upgrade() {
                    me.on_discovery(e);
                }
            }),
        )?;
        // Pending sends (SHR-044).
        {
            let me = self.clone();
            let stop = stop.clone();
            let every = env.retry;
            std::thread::Builder::new()
                .name("xs-lan-outbox".into())
                .spawn(move || {
                    let mut last = Instant::now() - every;
                    while !stop.load(Ordering::Relaxed) {
                        if last.elapsed() >= every {
                            last = Instant::now();
                            me.tick();
                        }
                        std::thread::sleep(Duration::from_millis(100));
                    }
                })
                .map_err(|e| e.to_string())?;
        }
        *self.running.lock().unwrap() = Some(Running {
            identity,
            port,
            stop,
            discovery: env.discovery.clone(),
            server,
        });
        Ok(())
    }

    pub fn stop(&self) {
        let Some(r) = self.running.lock().unwrap().take() else {
            return;
        };
        r.stop.store(true, Ordering::Relaxed);
        r.discovery.stop();
        self.peers.lock().unwrap().clear();
        let _ = &r.server;
    }

    fn running(&self) -> Option<(Arc<Identity>, u16)> {
        self.running
            .lock()
            .unwrap()
            .as_ref()
            .map(|r| (r.identity.clone(), r.port))
    }

    // -----------------------------------------------------------------------
    // People (SHR-042)

    fn on_discovery(self: &Arc<Self>, e: Event) {
        match e {
            Event::Seen { announce, addrs } => {
                let fp = announce.fingerprint.clone();
                self.peers.lock().unwrap().insert(
                    fp.clone(),
                    Peer {
                        announce: announce.clone(),
                        addrs,
                        online: true,
                    },
                );
                let mut c = self.contacts.lock().unwrap();
                if let Some(contact) = c.contacts.iter_mut().find(|x| x.fingerprint == fp) {
                    contact.last_seen = Some(now());
                    contact.name = announce.name.clone();
                    contact.device = announce.device.clone();
                    let copy = c.clone();
                    drop(c);
                    self.save_contacts(&copy);
                }
            }
            Event::Gone { fingerprint } => {
                if let Some(p) = self.peers.lock().unwrap().get_mut(&fingerprint) {
                    p.online = false;
                }
                let mut c = self.contacts.lock().unwrap();
                if let Some(contact) = c.contacts.iter_mut().find(|x| x.fingerprint == fingerprint) {
                    contact.last_seen = Some(now());
                    let copy = c.clone();
                    drop(c);
                    self.save_contacts(&copy);
                }
            }
        }
        self.event("people", json!({}));
    }

    pub fn status(&self) -> Value {
        let running = self.running();
        let on = self
            .core()
            .map(|c| c.settings.get_bool("share.lan"))
            .unwrap_or(false);
        json!({
            "enabled": on,
            "running": running.is_some(),
            "name": self.name(),
            "device": device_name(),
            "fingerprint": running.as_ref().map(|(id, _)| id.fingerprint()),
            "port": running.as_ref().map(|(_, p)| *p),
            "error": self.error.lock().unwrap().clone(),
        })
    }

    /// The people on the network and the contacts, online or not.
    pub fn people(&self) -> Vec<Value> {
        let peers = self.peers.lock().unwrap().clone();
        let c = self.contacts.lock().unwrap().clone();
        let mut out: Vec<Value> = Vec::new();
        for contact in &c.contacts {
            let p = peers.get(&contact.fingerprint);
            out.push(json!({
                "fingerprint": contact.fingerprint,
                "name": if contact.alias.is_empty() { &contact.name } else { &contact.alias },
                "theirName": contact.name,
                "device": contact.device,
                "online": p.is_some_and(|p| p.online),
                "contact": true,
                "alwaysAccept": contact.always_accept,
                "blocked": false,
                "lastSeen": contact.last_seen,
            }));
        }
        for (fp, p) in &peers {
            if !p.online || c.contacts.iter().any(|x| &x.fingerprint == fp) {
                continue;
            }
            out.push(json!({
                "fingerprint": fp,
                "name": p.announce.name,
                "device": p.announce.device,
                "online": true,
                "contact": false,
                "blocked": c.blocked.contains(fp),
            }));
        }
        out
    }

    fn peer(&self, fp: &str) -> Option<Peer> {
        self.peers.lock().unwrap().get(fp).cloned().filter(|p| p.online)
    }

    fn contact(&self, fp: &str) -> Option<Contact> {
        let c = self.contacts.lock().unwrap();
        if c.blocked.iter().any(|b| b == fp) {
            return None;
        }
        c.contacts.iter().find(|x| x.fingerprint == fp).cloned()
    }

    fn dial(&self, peer: &Peer) -> Result<TcpStream, String> {
        let mut last = "unreachable".to_string();
        for ip in &peer.addrs {
            match TcpStream::connect_timeout(
                &SocketAddr::new(*ip, peer.announce.port),
                Duration::from_secs(5),
            ) {
                Ok(s) => {
                    let _ = s.set_nodelay(true);
                    return Ok(s);
                }
                Err(e) => last = e.to_string(),
            }
        }
        Err(last)
    }

    // -----------------------------------------------------------------------
    // Contacts (SHR-043)

    pub fn contact_action(&self, fp: &str, action: &str, value: &Value) -> Result<(), String> {
        let mut c = self.contacts.lock().unwrap();
        match action {
            "rename" => {
                let x = c
                    .contacts
                    .iter_mut()
                    .find(|x| x.fingerprint == fp)
                    .ok_or("missing")?;
                x.alias = value.as_str().unwrap_or_default().trim().to_string();
            }
            "alwaysAccept" => {
                let x = c
                    .contacts
                    .iter_mut()
                    .find(|x| x.fingerprint == fp)
                    .ok_or("missing")?;
                x.always_accept = value.as_bool().unwrap_or(false);
            }
            "remove" => {
                let before = c.contacts.len();
                c.contacts.retain(|x| x.fingerprint != fp);
                if c.contacts.len() == before {
                    return Err("missing".into());
                }
            }
            "block" => {
                c.contacts.retain(|x| x.fingerprint != fp);
                if !c.blocked.iter().any(|b| b == fp) {
                    c.blocked.push(fp.to_string());
                }
            }
            "unblock" => c.blocked.retain(|b| b != fp),
            _ => return Err("action".into()),
        }
        let copy = c.clone();
        drop(c);
        self.save_contacts(&copy);
        self.event("people", json!({}));
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Pairing (SHR-043)

    /// Starts pairing with a person on the network: the code both see.
    pub fn pair(self: &Arc<Self>, fp: &str) -> Result<(String, String), String> {
        let (identity, _) = self.running().ok_or("off")?;
        let peer = self.peer(fp).ok_or("offline")?;
        let tcp = self.dial(&peer)?;
        tcp.set_read_timeout(Some(Duration::from_secs(10))).ok();
        let want = fp.to_string();
        let (mut s, key) = tls::connect(&identity, tcp, move |k| identity::fingerprint(k) == want)?;
        let code = identity::pairing_code(identity.public(), &key);
        let id = self.id("p");
        let pairing = Pairing {
            id: id.clone(),
            fingerprint: fp.to_string(),
            name: peer.announce.name.clone(),
            device: peer.announce.device.clone(),
            code: code.clone(),
            direction: Direction::Out,
            mine: None,
            done: false,
        };
        self.pairings.lock().unwrap().insert(id.clone(), pairing.clone());
        self.event("pair", pairing_json(&pairing));
        wire::send(
            &mut s,
            &json!({"t": "pair", "name": self.name(), "device": device_name()}),
        )
        .map_err(|e| e.to_string())?;
        let me = self.clone();
        let pid = id.clone();
        std::thread::spawn(move || {
            let r = (|| -> Result<bool, String> {
                s.sock.set_read_timeout(Some(ANSWER)).ok();
                let theirs = wire::read(&mut s).map_err(|e| e.to_string())?;
                let theirs = theirs["t"] == "pair_answer" && theirs["accept"] == true;
                let mine = me.wait_mine(&pid);
                let ok = theirs && mine;
                wire::send(&mut s, &json!({"t": "pair_final", "accept": ok})).map_err(|e| e.to_string())?;
                Ok(ok)
            })();
            let ok = r.unwrap_or(false);
            if ok {
                me.add_contact(&key, &peer.announce.name, &peer.announce.device);
            }
            me.finish_pairing(&pid, ok);
        });
        Ok((id, code))
    }

    /// Waits until this side answers a pairing (or the time is up: no).
    fn wait_mine(&self, id: &str) -> bool {
        let until = Instant::now() + ANSWER;
        while Instant::now() < until {
            if let Some(m) = self.pairings.lock().unwrap().get(id).and_then(|p| p.mine) {
                return m;
            }
            std::thread::sleep(Duration::from_millis(100));
        }
        false
    }

    pub fn answer_pairing(&self, id: &str, accept: bool) -> Result<(), String> {
        let mut p = self.pairings.lock().unwrap();
        let x = p.get_mut(id).filter(|x| !x.done).ok_or("missing")?;
        x.mine = Some(accept);
        Ok(())
    }

    pub fn pairings(&self) -> Vec<Value> {
        self.pairings
            .lock()
            .unwrap()
            .values()
            .filter(|p| !p.done)
            .map(pairing_json)
            .collect()
    }

    fn finish_pairing(&self, id: &str, ok: bool) {
        let p = self.pairings.lock().unwrap().remove(id);
        if let Some(mut p) = p {
            p.done = true;
            let mut v = pairing_json(&p);
            v["ok"] = json!(ok);
            self.event("pair_done", v);
        }
        self.event("people", json!({}));
    }

    fn add_contact(&self, key: &[u8], name: &str, device: &str) {
        let fp = identity::fingerprint(key);
        let mut c = self.contacts.lock().unwrap();
        c.contacts.retain(|x| x.fingerprint != fp);
        c.contacts.push(Contact {
            fingerprint: fp,
            key: identity::b64(key),
            name: name.to_string(),
            device: device.to_string(),
            alias: String::new(),
            always_accept: false,
            last_seen: Some(now()),
        });
        let copy = c.clone();
        drop(c);
        self.save_contacts(&copy);
    }

    // -----------------------------------------------------------------------
    // The listening side

    fn serve(
        self: &Arc<Self>,
        server: Arc<rustls::ServerConfig>,
        identity: &Identity,
        tcp: TcpStream,
    ) -> Result<(), String> {
        tcp.set_read_timeout(Some(Duration::from_secs(20))).ok();
        let (mut s, key) = tls::accept(server, tcp)?;
        let fp = identity::fingerprint(&key);
        if self.contacts.lock().unwrap().blocked.contains(&fp) {
            // Blocked: ignored without a word.
            return Ok(());
        }
        let first = wire::read(&mut s).map_err(|e| e.to_string())?;
        match first["t"].as_str() {
            Some("pair") => self.pair_in(s, identity, &key, &first),
            Some("offer") => {
                // Not a contact, or a contact whose key changed: nothing arrives.
                let Some(contact) = self.contact(&fp) else {
                    return Ok(());
                };
                if identity::unb64(&contact.key).as_deref() != Some(&key[..]) {
                    return Ok(());
                }
                self.offer_in(s, contact, &first)
            }
            _ => Ok(()),
        }
    }

    fn pair_in(
        self: &Arc<Self>,
        mut s: tls::ServerStream,
        identity: &Identity,
        key: &[u8],
        msg: &Value,
    ) -> Result<(), String> {
        let fp = identity::fingerprint(key);
        let id = self.id("p");
        let pairing = Pairing {
            id: id.clone(),
            fingerprint: fp,
            name: msg["name"]
                .as_str()
                .unwrap_or_default()
                .chars()
                .take(80)
                .collect(),
            device: msg["device"]
                .as_str()
                .unwrap_or_default()
                .chars()
                .take(80)
                .collect(),
            code: identity::pairing_code(identity.public(), key),
            direction: Direction::In,
            mine: None,
            done: false,
        };
        self.pairings.lock().unwrap().insert(id.clone(), pairing.clone());
        self.event("pair", pairing_json(&pairing));
        if let Some(core) = self.core() {
            let lang = core.lang();
            core.host().notify(
                "share",
                &xs_core::i18n::tr(lang, "notify.lan_pair_title", &[]),
                &xs_core::i18n::tr(
                    lang,
                    "notify.lan_pair",
                    &[("name", &pairing.name), ("device", &pairing.device)],
                ),
            );
        }
        let r = (|| -> Result<bool, String> {
            let mine = self.wait_mine(&id);
            wire::send(&mut s, &json!({"t": "pair_answer", "accept": mine})).map_err(|e| e.to_string())?;
            if !mine {
                return Ok(false);
            }
            s.sock.set_read_timeout(Some(ANSWER)).ok();
            let fin = wire::read(&mut s).map_err(|e| e.to_string())?;
            Ok(fin["t"] == "pair_final" && fin["accept"] == true)
        })();
        let ok = r.unwrap_or(false);
        if ok {
            self.add_contact(key, &pairing.name, &pairing.device);
        }
        self.finish_pairing(&id, ok);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Receiving (SHR-044)

    fn offer_in(
        self: &Arc<Self>,
        mut s: tls::ServerStream,
        contact: Contact,
        msg: &Value,
    ) -> Result<(), String> {
        let items: Vec<(String, u64)> = msg["items"]
            .as_array()
            .map(|a| {
                a.iter()
                    .map(|i| {
                        (
                            safe_name(i["name"].as_str().unwrap_or("file")),
                            i["size"].as_u64().unwrap_or(0),
                        )
                    })
                    .collect()
            })
            .unwrap_or_default();
        if items.is_empty() {
            return Ok(());
        }
        let total = items.iter().map(|i| i.1).sum();
        let shown = if contact.alias.is_empty() {
            contact.name.clone()
        } else {
            contact.alias.clone()
        };
        let id = self.id("r");
        let cancel = Arc::new(AtomicBool::new(false));
        let inc = Incoming {
            id: id.clone(),
            fingerprint: contact.fingerprint.clone(),
            name: shown.clone(),
            items: items.clone(),
            total,
            decision: contact.always_accept.then_some(true),
            cancel: cancel.clone(),
            receiving: false,
        };
        self.incoming.lock().unwrap().insert(id.clone(), inc.clone());
        if !contact.always_accept {
            self.event("incoming", incoming_json(&inc));
            if let Some(core) = self.core() {
                let lang = core.lang();
                core.host().notify(
                    "share",
                    &xs_core::i18n::tr(lang, "notify.lan_offer_title", &[]),
                    &xs_core::i18n::tr(
                        lang,
                        "notify.lan_offer",
                        &[
                            ("name", &shown),
                            ("count", &items.len().to_string()),
                            ("size", &size_text(total)),
                        ],
                    ),
                );
            }
        }
        let until = Instant::now() + OFFER_ANSWER;
        let accept = loop {
            if let Some(d) = self.incoming.lock().unwrap().get(&id).and_then(|i| i.decision) {
                break d;
            }
            if Instant::now() > until || cancel.load(Ordering::Relaxed) {
                break false;
            }
            std::thread::sleep(Duration::from_millis(100));
        };
        let r = wire::send(&mut s, &json!({"t": "answer", "accept": accept}));
        if !accept || r.is_err() {
            self.incoming.lock().unwrap().remove(&id);
            self.event("incoming_done", json!({"id": id, "status": "rejected"}));
            return Ok(());
        }
        if let Some(i) = self.incoming.lock().unwrap().get_mut(&id) {
            i.receiving = true;
        }
        let folder = self.staging.join(&id);
        let result = self.receive_files(&mut s, &folder, &id, total, &cancel);
        let status = match result {
            Ok(paths) => match self.to_library(&paths, &shown) {
                Ok(()) => {
                    let _ = wire::send(&mut s, &json!({"t": "delivered"}));
                    if contact.always_accept {
                        if let Some(core) = self.core() {
                            let lang = core.lang();
                            core.host().notify(
                                "share",
                                &xs_core::i18n::tr(lang, "notify.lan_received_title", &[]),
                                &xs_core::i18n::tr(
                                    lang,
                                    "notify.lan_received",
                                    &[("name", &shown), ("count", &paths.len().to_string())],
                                ),
                            );
                        }
                    }
                    "delivered"
                }
                Err(e) => {
                    tracing::warn!(error = %e, "received files could not go to the library");
                    let _ = wire::send(&mut s, &json!({"t": "failed", "reason": e}));
                    "failed"
                }
            },
            Err(e) if e == "cancelled" => {
                let _ = wire::send(&mut s, &json!({"t": "cancel"}));
                "cancelled"
            }
            Err(_) => "failed",
        };
        // Nothing half-received stays behind.
        let _ = std::fs::remove_dir_all(&folder);
        self.incoming.lock().unwrap().remove(&id);
        self.event("incoming_done", json!({"id": id, "status": status}));
        Ok(())
    }

    fn receive_files(
        &self,
        s: &mut tls::ServerStream,
        folder: &Path,
        id: &str,
        total: u64,
        cancel: &AtomicBool,
    ) -> Result<Vec<PathBuf>, String> {
        std::fs::create_dir_all(folder).map_err(|e| e.to_string())?;
        s.sock.set_read_timeout(Some(Duration::from_secs(60))).ok();
        let mut paths = Vec::new();
        let mut done: u64 = 0;
        let mut shown = Instant::now();
        loop {
            let msg = wire::read(s).map_err(|e| e.to_string())?;
            match msg["t"].as_str() {
                Some("file") => {
                    let name = safe_name(msg["name"].as_str().unwrap_or("file"));
                    let size = msg["size"].as_u64().unwrap_or(0);
                    let path = unique(folder, &name);
                    let mut out = std::fs::File::create(&path).map_err(|e| e.to_string())?;
                    let mut left = size;
                    let mut buf = vec![0u8; CHUNK];
                    let mut hash = ring::digest::Context::new(&ring::digest::SHA256);
                    while left > 0 {
                        if cancel.load(Ordering::Relaxed) {
                            return Err("cancelled".into());
                        }
                        let n = (left as usize).min(CHUNK);
                        s.read_exact(&mut buf[..n]).map_err(|e| e.to_string())?;
                        hash.update(&buf[..n]);
                        out.write_all(&buf[..n]).map_err(|e| e.to_string())?;
                        left -= n as u64;
                        done += n as u64;
                        if shown.elapsed() > Duration::from_millis(250) {
                            shown = Instant::now();
                            self.event(
                                "progress",
                                json!({"id": id, "direction": "in", "done": done, "total": total}),
                            );
                        }
                    }
                    drop(out);
                    let got: String = hash
                        .finish()
                        .as_ref()
                        .iter()
                        .map(|b| format!("{b:02x}"))
                        .collect();
                    if Some(got.as_str()) != msg["sha256"].as_str() {
                        return Err("a file arrived damaged".into());
                    }
                    paths.push(path);
                }
                Some("end") => return Ok(paths),
                Some("cancel") => return Err("cancelled".into()),
                _ => return Err("unexpected message".into()),
            }
        }
    }

    /// Received files to the library with the tag "From: <name>" (SHR-044).
    fn to_library(&self, paths: &[PathBuf], from: &str) -> Result<(), String> {
        let core = self.core().ok_or("closing")?;
        let lang = core.lang();
        let r = core
            .call(
                Source::Ui,
                "library.import",
                json!({"paths": paths, "tagFolders": false}),
                Some(lang),
            )
            .map_err(|e| e.message)?;
        let ids: Vec<Value> = r["imported"]
            .as_array()
            .map(|a| a.iter().map(|i| i["id"].clone()).collect())
            .unwrap_or_default();
        if ids.len() < paths.len() {
            return Err(r["skipped"][0]["reason"]
                .as_str()
                .unwrap_or("not imported")
                .to_string());
        }
        let tag = xs_core::i18n::tr(lang, "share.lan_from", &[("name", from)]);
        let _ = core.call(
            Source::Ui,
            "library.tag",
            json!({"ids": ids, "add": [tag]}),
            Some(lang),
        );
        Ok(())
    }

    pub fn incoming(&self) -> Vec<Value> {
        self.incoming
            .lock()
            .unwrap()
            .values()
            .map(incoming_json)
            .collect()
    }

    pub fn answer_incoming(&self, id: &str, accept: bool) -> Result<(), String> {
        let mut i = self.incoming.lock().unwrap();
        let x = i.get_mut(id).filter(|x| x.decision.is_none()).ok_or("missing")?;
        x.decision = Some(accept);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Sending (SHR-044)

    pub fn send(&self, to: &[String], paths: &[String]) -> Result<Vec<Value>, String> {
        let mut items = Vec::new();
        for p in paths {
            let meta = std::fs::metadata(p).map_err(|_| format!("{p}: not found"))?;
            if !meta.is_file() {
                return Err(format!("{p}: not a file"));
            }
            let name = Path::new(p)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            items.push(Item {
                path: p.clone(),
                name,
                size: meta.len(),
            });
        }
        let mut made = Vec::new();
        {
            let mut o = self.outbox.lock().unwrap();
            for fp in to {
                if self.contact(fp).is_none() {
                    return Err(format!("{fp}: not a contact"));
                }
                let id = self.id("s");
                o.sends.push(Send {
                    id: id.clone(),
                    to: fp.clone(),
                    items: items.clone(),
                    created: now(),
                    status: Status::Pending,
                    updated: now(),
                    reason: String::new(),
                });
                made.push(json!({"id": id, "to": fp}));
            }
            o.trim();
            let copy = o.clone();
            drop(o);
            self.save_outbox(&copy);
        }
        self.event("sends", json!({}));
        Ok(made)
    }

    pub fn sends(&self) -> Vec<Value> {
        let names: HashMap<String, String> = self
            .contacts
            .lock()
            .unwrap()
            .contacts
            .iter()
            .map(|c| {
                (
                    c.fingerprint.clone(),
                    if c.alias.is_empty() {
                        c.name.clone()
                    } else {
                        c.alias.clone()
                    },
                )
            })
            .collect();
        self.outbox
            .lock()
            .unwrap()
            .sends
            .iter()
            .rev()
            .map(|s| {
                let mut v = serde_json::to_value(s).unwrap_or_default();
                v["toName"] = json!(names.get(&s.to).cloned().unwrap_or_default());
                v
            })
            .collect()
    }

    fn set_status(&self, id: &str, status: Status, reason: &str) {
        let mut o = self.outbox.lock().unwrap();
        if let Some(s) = o.sends.iter_mut().find(|s| s.id == id) {
            // A cancelled send stays cancelled.
            if s.status == Status::Cancelled && status != Status::Cancelled {
                return;
            }
            s.status = status;
            s.updated = now();
            s.reason = reason.to_string();
        }
        let copy = o.clone();
        drop(o);
        self.save_outbox(&copy);
        self.event("sends", json!({"id": id, "status": status}));
    }

    /// Cancels a send (pending or in flight) or a reception in progress.
    pub fn cancel(&self, id: &str) -> Result<(), String> {
        if let Some(i) = self.incoming.lock().unwrap().get_mut(id) {
            i.cancel.store(true, Ordering::Relaxed);
            if i.decision.is_none() {
                i.decision = Some(false);
            }
            return Ok(());
        }
        let known = self
            .outbox
            .lock()
            .unwrap()
            .sends
            .iter()
            .any(|s| s.id == id && !s.status.done());
        if !known {
            return Err("missing".into());
        }
        if let Some(flag) = self.flying.lock().unwrap().get(id) {
            flag.store(true, Ordering::Relaxed);
        }
        self.set_status(id, Status::Cancelled, "");
        Ok(())
    }

    /// Expires old pending sends and starts those whose person is online.
    fn tick(self: &Arc<Self>) {
        let pending: Vec<Send> = self
            .outbox
            .lock()
            .unwrap()
            .sends
            .iter()
            .filter(|s| s.status == Status::Pending)
            .cloned()
            .collect();
        for s in pending {
            let old = DateTime::parse_from_rfc3339(&s.created)
                .map(|t| Utc::now() - t.with_timezone(&Utc) > EXPIRES)
                .unwrap_or(false);
            if old {
                self.set_status(&s.id, Status::Expired, "");
                if let Some(core) = self.core() {
                    let lang = core.lang();
                    core.host().notify(
                        "share",
                        &xs_core::i18n::tr(lang, "notify.lan_expired_title", &[]),
                        &xs_core::i18n::tr(
                            lang,
                            "notify.lan_expired",
                            &[("count", &s.items.len().to_string())],
                        ),
                    );
                }
                continue;
            }
            if self.flying.lock().unwrap().contains_key(&s.id) || self.peer(&s.to).is_none() {
                continue;
            }
            let flag = Arc::new(AtomicBool::new(false));
            self.flying.lock().unwrap().insert(s.id.clone(), flag.clone());
            let me = self.clone();
            std::thread::spawn(move || {
                let r = me.deliver(&s, &flag);
                me.flying.lock().unwrap().remove(&s.id);
                match r {
                    Ok(status) => me.set_status(&s.id, status, ""),
                    // Not reached, or the connection broke: it stays pending.
                    Err(e) => {
                        tracing::debug!(error = %e, "local network send will be tried again");
                        me.set_status(&s.id, Status::Pending, &e)
                    }
                }
            });
        }
    }

    fn deliver(&self, send: &Send, cancel: &AtomicBool) -> Result<Status, String> {
        let (identity, _) = self.running().ok_or("off")?;
        let contact = self.contact(&send.to).ok_or("not a contact")?;
        let peer = self.peer(&send.to).ok_or("offline")?;
        // Every file is still there.
        for i in &send.items {
            if !Path::new(&i.path).is_file() {
                return Ok(Status::Failed);
            }
        }
        let key = identity::unb64(&contact.key).ok_or("bad key")?;
        let tcp = self.dial(&peer)?;
        tcp.set_read_timeout(Some(Duration::from_secs(20))).ok();
        let (mut s, _) = tls::connect(&identity, tcp, move |k| k == key.as_slice())?;
        let total: u64 = send.items.iter().map(|i| i.size).sum();
        wire::send(
            &mut s,
            &json!({
                "t": "offer",
                "id": send.id,
                "items": send.items.iter().map(|i| json!({"name": i.name, "size": i.size})).collect::<Vec<_>>(),
                "total": total,
            }),
        )
        .map_err(|e| e.to_string())?;
        s.sock.set_read_timeout(Some(OFFER_ANSWER)).ok();
        let answer = wire::read(&mut s).map_err(|e| e.to_string())?;
        if answer["accept"] != true {
            return Ok(Status::Rejected);
        }
        self.set_status(&send.id, Status::Sending, "");
        let mut done: u64 = 0;
        let mut shown = Instant::now();
        let mut buf = vec![0u8; CHUNK];
        let r = (|| -> Result<(), String> {
            for item in &send.items {
                let bytes = std::fs::read(&item.path).map_err(|e| e.to_string())?;
                wire::send(
                    &mut s,
                    &json!({"t": "file", "name": item.name, "size": bytes.len(), "sha256": identity::sha256_hex(&bytes)}),
                )
                .map_err(|e| e.to_string())?;
                let mut r = std::io::Cursor::new(bytes);
                loop {
                    if cancel.load(Ordering::Relaxed) {
                        let _ = wire::send(&mut s, &json!({"t": "cancel"}));
                        return Err("cancelled".into());
                    }
                    let n = r.read(&mut buf).map_err(|e| e.to_string())?;
                    if n == 0 {
                        break;
                    }
                    s.write_all(&buf[..n]).map_err(|e| e.to_string())?;
                    done += n as u64;
                    if shown.elapsed() > Duration::from_millis(250) {
                        shown = Instant::now();
                        self.event(
                            "progress",
                            json!({"id": send.id, "direction": "out", "done": done, "total": total}),
                        );
                    }
                }
            }
            wire::send(&mut s, &json!({"t": "end"})).map_err(|e| e.to_string())
        })();
        match r {
            Err(e) if e == "cancelled" => return Ok(Status::Cancelled),
            Err(e) => {
                // The other side may have cancelled.
                s.sock.set_read_timeout(Some(Duration::from_secs(2))).ok();
                if let Ok(m) = wire::read(&mut s) {
                    if m["t"] == "cancel" {
                        return Ok(Status::Cancelled);
                    }
                }
                return Err(e);
            }
            Ok(()) => {}
        }
        s.sock.set_read_timeout(Some(Duration::from_secs(120))).ok();
        let last = wire::read(&mut s).map_err(|e| e.to_string())?;
        Ok(match last["t"].as_str() {
            Some("delivered") => Status::Delivered,
            Some("cancel") => Status::Cancelled,
            _ => Status::Failed,
        })
    }
}

fn pairing_json(p: &Pairing) -> Value {
    json!({
        "id": p.id,
        "fingerprint": p.fingerprint,
        "name": p.name,
        "device": p.device,
        "code": p.code,
        "direction": if p.direction == Direction::Out { "out" } else { "in" },
        "answered": p.mine.is_some(),
    })
}

fn incoming_json(i: &Incoming) -> Value {
    json!({
        "id": i.id,
        "fingerprint": i.fingerprint,
        "name": i.name,
        "items": i.items.iter().map(|(n, s)| json!({"name": n, "size": s})).collect::<Vec<_>>(),
        "total": i.total,
        "receiving": i.receiving,
    })
}

/// A file name from the other computer, made safe: no folders, no odd characters.
fn safe_name(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or("file");
    let clean: String = base
        .chars()
        .filter(|c| !c.is_control() && !"<>:\"|?*".contains(*c))
        .collect();
    let clean = clean.trim().trim_start_matches('.').trim_end_matches('.');
    if clean.is_empty() {
        "file".into()
    } else {
        clean.chars().take(120).collect()
    }
}

fn unique(dir: &Path, name: &str) -> PathBuf {
    let mut p = dir.join(name);
    let (stem, ext) = match name.rsplit_once('.') {
        Some((s, e)) => (s.to_string(), format!(".{e}")),
        None => (name.to_string(), String::new()),
    };
    let mut n = 2;
    while p.exists() {
        p = dir.join(format!("{stem} ({n}){ext}"));
        n += 1;
    }
    p
}

fn size_text(bytes: u64) -> String {
    if bytes >= 1 << 20 {
        format!("{:.1} MB", bytes as f64 / (1u64 << 20) as f64)
    } else {
        format!("{} KB", bytes.div_ceil(1024))
    }
}

#[allow(dead_code)]
fn local_time(s: &str) -> Option<DateTime<Local>> {
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|t| t.with_timezone(&Local))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn names_from_elsewhere_are_made_safe() {
        assert_eq!(safe_name("../../evil.exe"), "evil.exe");
        assert_eq!(safe_name("C:\\x\\a:b.png"), "ab.png");
        assert_eq!(safe_name(".."), "file");
        assert_eq!(size_text(3 * 1024 * 1024 + 400 * 1024), "3.4 MB");
    }
}
