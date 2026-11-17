//! The encrypted connection between two computers (SHR-043, SHR-044): TLS 1.3 with raw
//! public keys (RFC 7250), no certificates. Each side presents its Ed25519 key; the one
//! that connects checks the other's against what it expects, and the one that listens
//! reads who connected and decides what that person may do.

use std::net::TcpStream;
use std::sync::Arc;

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::client::AlwaysResolvesClientRawPublicKeys;
use rustls::crypto::{verify_tls13_signature_with_raw_key, CryptoProvider, WebPkiSupportedAlgorithms};
use rustls::pki_types::{
    CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName, SubjectPublicKeyInfoDer, UnixTime,
};
use rustls::server::danger::{ClientCertVerified, ClientCertVerifier};
use rustls::server::AlwaysResolvesServerRawPublicKeys;
use rustls::sign::CertifiedKey;
use rustls::{
    ClientConfig, ClientConnection, DigitallySignedStruct, DistinguishedName, Error, ServerConfig,
    ServerConnection, SignatureScheme, StreamOwned,
};

use crate::identity::{key_of_spki, Identity};

pub type ClientStream = StreamOwned<ClientConnection, TcpStream>;
pub type ServerStream = StreamOwned<ServerConnection, TcpStream>;

fn provider() -> Arc<CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}

fn certified(id: &Identity, provider: &CryptoProvider) -> Result<Arc<CertifiedKey>, String> {
    let key = provider
        .key_provider
        .load_private_key(PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(
            id.pkcs8().to_vec(),
        )))
        .map_err(|e| e.to_string())?;
    Ok(Arc::new(CertifiedKey::new(
        vec![CertificateDer::from(id.spki())],
        key,
    )))
}

fn verify13(
    algs: &WebPkiSupportedAlgorithms,
    message: &[u8],
    cert: &CertificateDer<'_>,
    dss: &DigitallySignedStruct,
) -> Result<HandshakeSignatureValid, Error> {
    verify_tls13_signature_with_raw_key(message, &SubjectPublicKeyInfoDer::from(cert.as_ref()), dss, algs)
}

/// What the side that connects accepts as the other's key.
type Expect = dyn Fn(&[u8]) -> bool + Send + Sync;

struct ServerKey {
    algs: WebPkiSupportedAlgorithms,
    expect: Box<Expect>,
}

impl std::fmt::Debug for ServerKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ServerKey")
    }
}

impl ServerCertVerifier for ServerKey {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, Error> {
        match key_of_spki(end_entity.as_ref()) {
            Some(key) if (self.expect)(&key) => Ok(ServerCertVerified::assertion()),
            _ => Err(Error::General("not the expected computer".into())),
        }
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Err(Error::General("TLS 1.3 only".into()))
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        verify13(&self.algs, message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.algs.supported_schemes()
    }

    fn requires_raw_public_keys(&self) -> bool {
        true
    }
}

/// The listening side takes any Ed25519 key; what its owner may do is decided after.
struct AnyClientKey {
    algs: WebPkiSupportedAlgorithms,
}

impl std::fmt::Debug for AnyClientKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("AnyClientKey")
    }
}

impl ClientCertVerifier for AnyClientKey {
    fn root_hint_subjects(&self) -> &[DistinguishedName] {
        &[]
    }

    fn verify_client_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _now: UnixTime,
    ) -> Result<ClientCertVerified, Error> {
        key_of_spki(end_entity.as_ref())
            .map(|_| ClientCertVerified::assertion())
            .ok_or_else(|| Error::General("not an XcreenShot key".into()))
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Err(Error::General("TLS 1.3 only".into()))
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        verify13(&self.algs, message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.algs.supported_schemes()
    }

    fn requires_raw_public_keys(&self) -> bool {
        true
    }
}

/// The listening side's configuration.
pub fn server_config(id: &Identity) -> Result<Arc<ServerConfig>, String> {
    let provider = provider();
    let ck = certified(id, &provider)?;
    let algs = provider.signature_verification_algorithms;
    let config = ServerConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .map_err(|e| e.to_string())?
        .with_client_cert_verifier(Arc::new(AnyClientKey { algs }))
        .with_cert_resolver(Arc::new(AlwaysResolvesServerRawPublicKeys::new(ck)));
    Ok(Arc::new(config))
}

/// Connects to `addr`, accepting only a computer whose key passes `expect`. The
/// handshake is finished when this returns.
pub fn connect(
    id: &Identity,
    stream: TcpStream,
    expect: impl Fn(&[u8]) -> bool + Send + Sync + 'static,
) -> Result<(ClientStream, Vec<u8>), String> {
    let provider = provider();
    let ck = certified(id, &provider)?;
    let algs = provider.signature_verification_algorithms;
    let config = ClientConfig::builder_with_provider(provider)
        .with_protocol_versions(&[&rustls::version::TLS13])
        .map_err(|e| e.to_string())?
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(ServerKey {
            algs,
            expect: Box::new(expect),
        }))
        .with_client_cert_resolver(Arc::new(AlwaysResolvesClientRawPublicKeys::new(ck)));
    let name = ServerName::try_from("xcreenshot.local").map_err(|e| e.to_string())?;
    let conn = ClientConnection::new(Arc::new(config), name).map_err(|e| e.to_string())?;
    let mut s = StreamOwned::new(conn, stream);
    while s.conn.is_handshaking() {
        s.conn.complete_io(&mut s.sock).map_err(|e| e.to_string())?;
    }
    let key = s
        .conn
        .peer_certificates()
        .and_then(|c| c.first())
        .and_then(|c| key_of_spki(c.as_ref()))
        .ok_or("the other computer sent no key")?;
    Ok((s, key))
}

/// Accepts a connection: the handshake, and the key of who connected.
pub fn accept(config: Arc<ServerConfig>, stream: TcpStream) -> Result<(ServerStream, Vec<u8>), String> {
    let conn = ServerConnection::new(config).map_err(|e| e.to_string())?;
    let mut s = StreamOwned::new(conn, stream);
    while s.conn.is_handshaking() {
        s.conn.complete_io(&mut s.sock).map_err(|e| e.to_string())?;
    }
    let key = s
        .conn
        .peer_certificates()
        .and_then(|c| c.first())
        .and_then(|c| key_of_spki(c.as_ref()))
        .ok_or("the other computer sent no key")?;
    Ok((s, key))
}
