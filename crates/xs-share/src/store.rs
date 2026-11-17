//! What the local network keeps between runs (SHR-043, SHR-044): the contacts with
//! their public keys, and the sends with their state. Both in the settings folder; the
//! private key is in the keychain, never here.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Contact {
    pub fingerprint: String,
    /// The public key, base64.
    pub key: String,
    pub name: String,
    pub device: String,
    /// A name the user gave them; theirs when empty.
    #[serde(default)]
    pub alias: String,
    #[serde(default)]
    pub always_accept: bool,
    #[serde(default)]
    pub last_seen: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Contacts {
    #[serde(default)]
    pub contacts: Vec<Contact>,
    /// Blocked fingerprints: their requests and sends are ignored without a word.
    #[serde(default)]
    pub blocked: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub path: String,
    pub name: String,
    pub size: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    Pending,
    Sending,
    Delivered,
    Rejected,
    Expired,
    Cancelled,
    Failed,
}

impl Status {
    pub fn done(self) -> bool {
        !matches!(self, Status::Pending | Status::Sending)
    }
}

/// A send (SHR-044) and where it is.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Send {
    pub id: String,
    pub to: String,
    pub items: Vec<Item>,
    pub created: String,
    pub status: Status,
    #[serde(default)]
    pub updated: String,
    #[serde(default)]
    pub reason: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Outbox {
    #[serde(default)]
    pub sends: Vec<Send>,
}

/// Finished sends kept in the history.
const KEEP_DONE: usize = 100;

impl Outbox {
    /// Old finished sends leave; pending ones always stay.
    pub fn trim(&mut self) {
        let done = self.sends.iter().filter(|s| s.status.done()).count();
        if done <= KEEP_DONE {
            return;
        }
        let mut drop = done - KEEP_DONE;
        self.sends.retain(|s| {
            if drop > 0 && s.status.done() {
                drop -= 1;
                false
            } else {
                true
            }
        });
    }
}

pub fn read<T: for<'de> Deserialize<'de> + Default>(path: &Path) -> T {
    std::fs::read(path)
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

pub fn write<T: Serialize>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let tmp: PathBuf = path.with_extension("tmp");
    std::fs::write(&tmp, serde_json::to_vec_pretty(value).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}
