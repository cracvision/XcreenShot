//! Finding other XcreenShot users on the local network (SHR-042): DNS-SD over mDNS,
//! only while the preference is on. The announcement says the visible name, the
//! computer and the key's fingerprint; nothing else. Tests use [`MemoryNet`] instead,
//! so they open nothing towards the network.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub const SERVICE: &str = "_xcreenshot._tcp.local.";

/// What this computer announces.
#[derive(Debug, Clone, PartialEq)]
pub struct Announce {
    pub fingerprint: String,
    pub name: String,
    pub device: String,
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Seen { announce: Announce, addrs: Vec<IpAddr> },
    Gone { fingerprint: String },
}

pub type OnEvent = Arc<dyn Fn(Event) + Send + Sync>;

pub trait Discovery: Send + Sync {
    fn start(&self, me: Announce, on_event: OnEvent) -> Result<(), String>;
    fn stop(&self);
}

// ---------------------------------------------------------------------------
// mDNS

pub struct Mdns {
    daemon: Mutex<Option<(mdns_sd::ServiceDaemon, String)>>,
}

impl Mdns {
    pub fn new() -> Mdns {
        Mdns {
            daemon: Mutex::new(None),
        }
    }
}

impl Default for Mdns {
    fn default() -> Self {
        Mdns::new()
    }
}

fn announce_of(info: &mdns_sd::ResolvedService) -> Option<Announce> {
    let get = |k: &str| info.get_property_val_str(k).map(str::to_string);
    if get("v").as_deref() != Some("1") {
        return None;
    }
    Some(Announce {
        fingerprint: get("fp")?,
        name: get("name").unwrap_or_default(),
        device: get("device").unwrap_or_default(),
        port: info.get_port(),
    })
}

impl Discovery for Mdns {
    fn start(&self, me: Announce, on_event: OnEvent) -> Result<(), String> {
        self.stop();
        let daemon = mdns_sd::ServiceDaemon::new().map_err(|e| e.to_string())?;
        let host = format!("xs-{}.local.", me.fingerprint);
        let props = [
            ("v", "1"),
            ("fp", me.fingerprint.as_str()),
            ("name", me.name.as_str()),
            ("device", me.device.as_str()),
        ];
        let info = mdns_sd::ServiceInfo::new(SERVICE, &me.fingerprint, &host, "", me.port, &props[..])
            .map_err(|e| e.to_string())?
            .enable_addr_auto();
        let fullname = info.get_fullname().to_string();
        daemon.register(info).map_err(|e| e.to_string())?;
        let events = daemon.browse(SERVICE).map_err(|e| e.to_string())?;
        // Who is where: names of instances by fingerprint, to tell when one leaves.
        let mine = me.fingerprint.clone();
        let check = daemon.clone();
        std::thread::Builder::new()
            .name("xs-lan-mdns".into())
            .spawn(move || {
                let mut known: HashMap<String, String> = HashMap::new();
                let mut last_check = std::time::Instant::now();
                loop {
                    match events.recv_timeout(Duration::from_secs(2)) {
                        Ok(mdns_sd::ServiceEvent::ServiceResolved(info)) => {
                            if let Some(a) = announce_of(&info) {
                                if a.fingerprint == mine {
                                    continue;
                                }
                                known.insert(info.get_fullname().to_string(), a.fingerprint.clone());
                                let addrs = info.get_addresses().iter().map(|a| a.to_ip_addr()).collect();
                                on_event(Event::Seen { announce: a, addrs });
                            }
                        }
                        Ok(mdns_sd::ServiceEvent::ServiceRemoved(_, fullname)) => {
                            if let Some(fp) = known.remove(&fullname) {
                                on_event(Event::Gone { fingerprint: fp });
                            }
                        }
                        Ok(mdns_sd::ServiceEvent::SearchStopped(_)) => break,
                        Ok(_) => {}
                        Err(mdns_sd::RecvTimeoutError::Disconnected) => break,
                        Err(mdns_sd::RecvTimeoutError::Timeout) => {}
                    }
                    // Asks every 10 s whether each one is still there, so one that left
                    // without saying goodbye shows as offline within 30 s.
                    if last_check.elapsed() >= Duration::from_secs(10) {
                        last_check = std::time::Instant::now();
                        for name in known.keys() {
                            let _ = check.verify(name.clone(), Duration::from_secs(8));
                        }
                    }
                }
            })
            .map_err(|e| e.to_string())?;
        *self.daemon.lock().unwrap() = Some((daemon, fullname));
        Ok(())
    }

    fn stop(&self) {
        if let Some((daemon, fullname)) = self.daemon.lock().unwrap().take() {
            // A goodbye, so the others see this computer leave at once.
            if let Ok(done) = daemon.unregister(&fullname) {
                let _ = done.recv_timeout(Duration::from_secs(1));
            }
            let _ = daemon.stop_browse(SERVICE);
            let _ = daemon.shutdown();
        }
    }
}

// ---------------------------------------------------------------------------
// A shared folder (development builds only)

/// Two development instances on this computer find each other through a folder
/// (`XCREENSHOT_LAN_DIR`), at 127.0.0.1: no multicast, no port open to the network, so
/// trying the local network never asks for the firewall.
pub struct FolderDiscovery {
    dir: std::path::PathBuf,
    stop: Mutex<Option<Arc<std::sync::atomic::AtomicBool>>>,
    me: Mutex<Option<String>>,
}

impl FolderDiscovery {
    pub fn new(dir: std::path::PathBuf) -> FolderDiscovery {
        FolderDiscovery {
            dir,
            stop: Mutex::new(None),
            me: Mutex::new(None),
        }
    }
}

impl Discovery for FolderDiscovery {
    fn start(&self, me: Announce, on_event: OnEvent) -> Result<(), String> {
        use std::sync::atomic::{AtomicBool, Ordering};
        self.stop();
        std::fs::create_dir_all(&self.dir).map_err(|e| e.to_string())?;
        let file = self.dir.join(format!("{}.json", me.fingerprint));
        let body =
            serde_json::json!({"fp": me.fingerprint, "name": me.name, "device": me.device, "port": me.port});
        std::fs::write(&file, body.to_string()).map_err(|e| e.to_string())?;
        let stop = Arc::new(AtomicBool::new(false));
        *self.stop.lock().unwrap() = Some(stop.clone());
        *self.me.lock().unwrap() = Some(me.fingerprint.clone());
        let dir = self.dir.clone();
        let mine = me.fingerprint.clone();
        std::thread::spawn(move || {
            let mut known: HashMap<String, Announce> = HashMap::new();
            while !stop.load(Ordering::Relaxed) {
                let mut now: HashMap<String, Announce> = HashMap::new();
                for e in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
                    let Ok(v) = std::fs::read(e.path())
                        .map_err(|_| ())
                        .and_then(|b| serde_json::from_slice::<serde_json::Value>(&b).map_err(|_| ()))
                    else {
                        continue;
                    };
                    let fp = v["fp"].as_str().unwrap_or_default().to_string();
                    if fp.is_empty() || fp == mine {
                        continue;
                    }
                    now.insert(
                        fp.clone(),
                        Announce {
                            fingerprint: fp,
                            name: v["name"].as_str().unwrap_or_default().into(),
                            device: v["device"].as_str().unwrap_or_default().into(),
                            port: v["port"].as_u64().unwrap_or(0) as u16,
                        },
                    );
                }
                for (fp, a) in &now {
                    if known.get(fp) != Some(a) {
                        on_event(Event::Seen {
                            announce: a.clone(),
                            addrs: vec![IpAddr::from([127, 0, 0, 1])],
                        });
                    }
                }
                for fp in known.keys() {
                    if !now.contains_key(fp) {
                        on_event(Event::Gone {
                            fingerprint: fp.clone(),
                        });
                    }
                }
                known = now;
                std::thread::sleep(Duration::from_millis(700));
            }
        });
        Ok(())
    }

    fn stop(&self) {
        if let Some(s) = self.stop.lock().unwrap().take() {
            s.store(true, std::sync::atomic::Ordering::Relaxed);
        }
        if let Some(fp) = self.me.lock().unwrap().take() {
            let _ = std::fs::remove_file(self.dir.join(format!("{fp}.json")));
        }
    }
}

// ---------------------------------------------------------------------------
// In memory (tests)

/// A pretend network: every [`MemoryDiscovery`] on it sees the others at 127.0.0.1.
#[derive(Clone, Default)]
pub struct MemoryNet(Arc<Mutex<Vec<(Announce, OnEvent)>>>);

pub struct MemoryDiscovery {
    net: MemoryNet,
    me: Mutex<Option<String>>,
}

impl MemoryNet {
    pub fn join(&self) -> MemoryDiscovery {
        MemoryDiscovery {
            net: self.clone(),
            me: Mutex::new(None),
        }
    }
}

impl Discovery for MemoryDiscovery {
    fn start(&self, me: Announce, on_event: OnEvent) -> Result<(), String> {
        self.stop();
        let local = vec![IpAddr::from([127, 0, 0, 1])];
        let others: Vec<(Announce, OnEvent)> = self.net.0.lock().unwrap().clone();
        for (a, cb) in &others {
            on_event(Event::Seen {
                announce: a.clone(),
                addrs: local.clone(),
            });
            cb(Event::Seen {
                announce: me.clone(),
                addrs: local.clone(),
            });
        }
        *self.me.lock().unwrap() = Some(me.fingerprint.clone());
        self.net.0.lock().unwrap().push((me, on_event));
        Ok(())
    }

    fn stop(&self) {
        let Some(fp) = self.me.lock().unwrap().take() else {
            return;
        };
        let others: Vec<OnEvent> = {
            let mut net = self.net.0.lock().unwrap();
            net.retain(|(a, _)| a.fingerprint != fp);
            net.iter().map(|(_, cb)| cb.clone()).collect()
        };
        for cb in others {
            cb(Event::Gone {
                fingerprint: fp.clone(),
            });
        }
    }
}
