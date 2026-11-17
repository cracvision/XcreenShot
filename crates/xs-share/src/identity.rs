//! This computer's identity on the local network (SHR-042): an Ed25519 key pair kept
//! in the system keychain, its public key as a TLS raw public key (RFC 7250), its
//! fingerprint and the verification code two computers compare when pairing (SHR-043).

use base64::Engine as _;
use ring::signature::{Ed25519KeyPair, KeyPair};
use xs_core::keychain::Vault;

/// The DER prefix of an Ed25519 SubjectPublicKeyInfo; the 32 bytes of the key follow.
const SPKI_PREFIX: [u8; 12] = [
    0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0x00,
];

pub struct Identity {
    pkcs8: Vec<u8>,
    public: Vec<u8>,
}

impl Identity {
    /// The key pair in the keychain, made and stored the first time.
    pub fn load_or_create(vault: &dyn Vault) -> Result<Identity, String> {
        if let Some(stored) = vault.get()? {
            if let Ok(pkcs8) = base64::engine::general_purpose::STANDARD.decode(stored.trim()) {
                if let Ok(id) = Identity::from_pkcs8(pkcs8) {
                    return Ok(id);
                }
            }
        }
        let rng = ring::rand::SystemRandom::new();
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng)
            .map_err(|_| "the key could not be made")?
            .as_ref()
            .to_vec();
        vault.set(&base64::engine::general_purpose::STANDARD.encode(&pkcs8))?;
        Identity::from_pkcs8(pkcs8)
    }

    fn from_pkcs8(pkcs8: Vec<u8>) -> Result<Identity, String> {
        let pair = Ed25519KeyPair::from_pkcs8(&pkcs8).map_err(|_| "the stored key is not valid")?;
        let public = pair.public_key().as_ref().to_vec();
        Ok(Identity { pkcs8, public })
    }

    pub fn pkcs8(&self) -> &[u8] {
        &self.pkcs8
    }

    /// The raw public key (32 bytes).
    pub fn public(&self) -> &[u8] {
        &self.public
    }

    pub fn spki(&self) -> Vec<u8> {
        spki_of(&self.public)
    }

    pub fn fingerprint(&self) -> String {
        fingerprint(&self.public)
    }
}

/// An Ed25519 public key as a SubjectPublicKeyInfo (the raw public key TLS sends).
pub fn spki_of(public: &[u8]) -> Vec<u8> {
    let mut v = SPKI_PREFIX.to_vec();
    v.extend_from_slice(public);
    v
}

/// The 32-byte key inside a SubjectPublicKeyInfo of Ed25519, if it is one.
pub fn key_of_spki(spki: &[u8]) -> Option<Vec<u8>> {
    (spki.len() == 44 && spki[..12] == SPKI_PREFIX).then(|| spki[12..].to_vec())
}

fn sha256(bytes: &[u8]) -> Vec<u8> {
    ring::digest::digest(&ring::digest::SHA256, bytes)
        .as_ref()
        .to_vec()
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

/// What a computer announces of its key: 16 hex characters of its SHA-256.
pub fn fingerprint(public: &[u8]) -> String {
    hex(&sha256(public)[..8])
}

/// The 6-digit code both people compare when pairing (SHR-043): the same on both
/// computers, whichever starts.
pub fn pairing_code(a: &[u8], b: &[u8]) -> String {
    let (first, second) = if a <= b { (a, b) } else { (b, a) };
    let mut both = b"XcreenShot pairing v1".to_vec();
    both.extend_from_slice(first);
    both.extend_from_slice(second);
    let d = sha256(&both);
    let n = u32::from_be_bytes([d[0], d[1], d[2], d[3]]) % 1_000_000;
    format!("{n:06}")
}

/// Hex SHA-256 of a file's bytes, to check what arrived (SHR-044).
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex(&sha256(bytes))
}

pub fn b64(bytes: &[u8]) -> String {
    base64::engine::general_purpose::STANDARD.encode(bytes)
}

pub fn unb64(s: &str) -> Option<Vec<u8>> {
    base64::engine::general_purpose::STANDARD.decode(s).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use xs_core::keychain::MemoryVault;

    #[test]
    fn the_key_is_kept_and_codes_match() {
        let vault = MemoryVault::default();
        let a = Identity::load_or_create(&vault).unwrap();
        let again = Identity::load_or_create(&vault).unwrap();
        assert_eq!(a.public(), again.public(), "the same key next time");
        assert_eq!(a.fingerprint().len(), 16);
        assert_eq!(key_of_spki(&a.spki()).unwrap(), a.public());
        let b = Identity::load_or_create(&MemoryVault::default()).unwrap();
        let code = pairing_code(a.public(), b.public());
        assert_eq!(code, pairing_code(b.public(), a.public()));
        assert_eq!(code.len(), 6);
    }
}
