// SPDX-License-Identifier: MIT OR Apache-2.0
//! Inventory-pinned, mutually authenticated TLS 1.3 for broker links.
//!
//! There is no CA validation: the accepted signed inventory is the authority.
//! Certificate validity dates are ignored: half-open inventory epoch windows
//! are the validity policy. Hostnames are ignored: the dialled member is pinned.
//! Extensions are not used for authorisation. A certificate is only a public
//! key container; CertificateVerify proves possession of that exact SPKI.
//! No resumption, tickets or early data may bypass the live inventory check.
//!
//! Configs are reusable. After a successful handshake, call verified_member
//! on that connection's peer certificate to obtain its typed identity (and
//! fail closed if a reload has already revoked it). Never record peer identity
//! in a shared verifier: concurrent handshakes would overwrite one another.

use std::sync::{Arc, RwLock};
use ed25519_dalek::Signer as _;
use mesh_trust::identity::{TlsAuthority, TlsMember};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::server::danger::{ClientCertVerified, ClientCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, Error, SignatureScheme};
use zeroize::Zeroizing;

/// Replace the immutable cache only after accepting a signed inventory.
pub type LiveAuthority = Arc<RwLock<Arc<TlsAuthority>>>;
pub type Seed = Zeroizing<[u8; 32]>;

/// Raw 32-byte TLS seed, normally mix:etc/noded/bus-tls.seed.
/// Metadata is checked on the opened file, avoiding a stat/open race.
pub fn load_seed(path: &std::path::Path) -> std::io::Result<Seed> {
    use std::io::Read;
    use std::os::unix::fs::PermissionsExt;
    let mut file = std::fs::File::open(path)?;
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.permissions().mode() & 0o077 != 0 {
        return Err(std::io::Error::new(std::io::ErrorKind::PermissionDenied, "TLS seed must be private"));
    }
    let mut bytes = Zeroizing::new([0; 33]);
    let mut len = 0;
    while len < bytes.len() {
        let n = file.read(&mut bytes[len..])?;
        if n == 0 { break; }
        len += n;
    }
    if len != 32 {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "TLS seed must be exactly 32 bytes"));
    }
    let mut seed = Zeroizing::new([0; 32]);
    seed.copy_from_slice(&bytes[..32]);
    Ok(seed)
}

// Dalek's zeroize feature erases the signing key on final drop. Keeping rcgen
// on its remote-signing API avoids creating non-zeroizing ring private keys.
struct Signing {
    key: ed25519_dalek::SigningKey,
    public: [u8; 32],
}
impl std::fmt::Debug for Signing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str("BusTlsSigningKey") }
}
struct CertificateSigner(Arc<Signing>);
impl rcgen::RemoteKeyPair for CertificateSigner {
    fn public_key(&self) -> &[u8] { &self.0.public }
    fn sign(&self, msg: &[u8]) -> Result<Vec<u8>, rcgen::Error> {
        Ok(self.0.key.sign(msg).to_bytes().to_vec())
    }
    fn algorithm(&self) -> &'static rcgen::SignatureAlgorithm { &rcgen::PKCS_ED25519 }
}
#[derive(Debug)]
struct HandshakeSigner(Arc<Signing>);
impl rustls::sign::Signer for HandshakeSigner {
    fn sign(&self, msg: &[u8]) -> Result<Vec<u8>, Error> {
        Ok(self.0.key.sign(msg).to_bytes().to_vec())
    }
    fn scheme(&self) -> SignatureScheme { SignatureScheme::ED25519 }
}
#[derive(Debug)]
struct TlsSigningKey(Arc<Signing>);
impl rustls::sign::SigningKey for TlsSigningKey {
    fn choose_scheme(&self, offered: &[SignatureScheme]) -> Option<Box<dyn rustls::sign::Signer>> {
        offered.contains(&SignatureScheme::ED25519)
            .then(|| Box::new(HandshakeSigner(self.0.clone())) as Box<dyn rustls::sign::Signer>)
    }
    fn algorithm(&self) -> rustls::SignatureAlgorithm { rustls::SignatureAlgorithm::ED25519 }
}

/// Owns public certificate and zeroizing private material. Private bytes are
/// never passed into rcgen or the ring rustls key loader.
pub struct Credential {
    pub certificate: CertificateDer<'static>,
    certified: Arc<rustls::sign::CertifiedKey>,
    _seed: Seed,
    _pkcs8: Zeroizing<Vec<u8>>,
}
impl Credential {
    pub fn new(seed: Seed) -> anyhow::Result<Self> {
        let key = ed25519_dalek::SigningKey::from_bytes(&seed);
        let public = key.verifying_key().to_bytes();
        let signing = Arc::new(Signing { key, public });
        // RFC 8410 PKCS#8 v1, kept once in an erasing allocation.
        let mut pkcs8 = Zeroizing::new(vec![0x30,0x2e,0x02,0x01,0,0x30,0x05,0x06,0x03,0x2b,0x65,0x70,0x04,0x22,0x04,0x20]);
        pkcs8.extend_from_slice(seed.as_ref());
        let pair = rcgen::KeyPair::from_remote(Box::new(CertificateSigner(signing.clone())))?;
        let params = rcgen::CertificateParams::new(Vec::<String>::new())?;
        let certificate = params.self_signed(&pair)?.der().clone();
        let certified = Arc::new(rustls::sign::CertifiedKey::new(
            vec![certificate.clone()], Arc::new(TlsSigningKey(signing))));
        Ok(Self { certificate, certified, _seed: seed, _pkcs8: pkcs8 })
    }
    pub fn server_config(&self, authority: LiveAuthority) -> Result<Arc<rustls::ServerConfig>, Error> {
        let mut config = rustls::ServerConfig::builder_with_provider(provider())
            .with_protocol_versions(&[&rustls::version::TLS13])?
            .with_client_cert_verifier(Arc::new(Verifier { authority, expected: None }))
            .with_cert_resolver(Arc::new(Resolver(self.certified.clone())));
        config.max_early_data_size = 0;
        config.send_tls13_tickets = 0;
        config.session_storage = Arc::new(rustls::server::NoServerSessionStorage {});
        Ok(Arc::new(config))
    }
    pub fn client_config(&self, authority: LiveAuthority, expected: String) -> Result<Arc<rustls::ClientConfig>, Error> {
        let mut config = rustls::ClientConfig::builder_with_provider(provider())
            .with_protocol_versions(&[&rustls::version::TLS13])?
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(Verifier { authority, expected: Some(expected) }))
            .with_client_cert_resolver(Arc::new(Resolver(self.certified.clone())));
        config.enable_early_data = false;
        config.resumption = rustls::client::Resumption::disabled();
        Ok(Arc::new(config))
    }
}
fn provider() -> Arc<rustls::crypto::CryptoProvider> {
    Arc::new(rustls::crypto::ring::default_provider())
}
#[derive(Debug)]
struct Resolver(Arc<rustls::sign::CertifiedKey>);
impl rustls::server::ResolvesServerCert for Resolver {
    fn resolve(&self, _: rustls::server::ClientHello<'_>) -> Option<Arc<rustls::sign::CertifiedKey>> { Some(self.0.clone()) }
}
impl rustls::client::ResolvesClientCert for Resolver {
    fn resolve(&self, _: &[&[u8]], schemes: &[SignatureScheme]) -> Option<Arc<rustls::sign::CertifiedKey>> {
        schemes.contains(&SignatureScheme::ED25519).then(|| self.0.clone())
    }
    fn has_certs(&self) -> bool { true }
}
fn refused(message: &str) -> Error { Error::General(message.into()) }

/// Parse the actual TBSCertificate SPKI, never a byte search. The canonical
/// SPKI encoding is compared in full, rejecting non-minimal lengths and any
/// extra algorithm fields even if the X.509 parser accepts BER equivalents.
pub fn public_key(cert: &CertificateDer<'_>) -> Result<[u8; 32], Error> {
    use x509_parser::prelude::{FromDer, X509Certificate};
    let (rest, _) = x509_parser::der_parser::der::parse_der(cert.as_ref())
        .map_err(|_| refused("non-canonical DER certificate"))?;
    if !rest.is_empty() { return Err(refused("trailing certificate data")); }
    let (rest, parsed) = X509Certificate::from_der(cert.as_ref()).map_err(|_| refused("invalid certificate"))?;
    if !rest.is_empty() { return Err(refused("trailing certificate data")); }
    let spki = parsed.public_key();
    if spki.algorithm.algorithm.to_id_string() != "1.3.101.112"
        || spki.algorithm.parameters.is_some()
        || spki.subject_public_key.unused_bits != 0 {
        return Err(refused("expected parameter-free Ed25519 SPKI"));
    }
    let key: [u8; 32] = spki.subject_public_key.data.as_ref().try_into().map_err(|_| refused("invalid Ed25519 key length"))?;
    let mut canonical = vec![0x30,0x2a,0x30,0x05,0x06,0x03,0x2b,0x65,0x70,0x03,0x21,0];
    canonical.extend_from_slice(&key);
    if spki.raw != canonical { return Err(refused("non-canonical SPKI")); }
    Ok(key)
}

/// Obtain a typed identity from the exact certificate rustls authenticated.
/// Call only after handshake completion; no certificate-only API proves possession.
pub fn verified_member(authority: &LiveAuthority, certificates: &[CertificateDer<'_>], expected: Option<&str>) -> Result<TlsMember, Error> {
    let [certificate] = certificates else { return Err(refused("exactly one certificate required")); };
    let key = public_key(certificate)?;
    let member = authority.read().map_err(|_| refused("authority lock poisoned"))?
        .member_for(&key).ok_or_else(|| refused("key absent or ambiguous in accepted inventory"))?;
    if expected.is_some_and(|expected| expected != member.member) { return Err(refused("wrong dialled member")); }
    Ok(member)
}
#[derive(Debug)]
struct Verifier { authority: LiveAuthority, expected: Option<String> }
impl Verifier {
    fn check(&self, cert: &CertificateDer<'_>, chain: &[CertificateDer<'_>]) -> Result<(), Error> {
        if !chain.is_empty() { return Err(refused("certificate chain refused")); }
        verified_member(&self.authority, std::slice::from_ref(cert), self.expected.as_deref()).map(|_| ())
    }
}
fn signature(msg: &[u8], cert: &CertificateDer<'_>, dss: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, Error> {
    if dss.scheme != SignatureScheme::ED25519 { return Err(refused("only ED25519 is allowed")); }
    public_key(cert)?;
    rustls::crypto::verify_tls13_signature(msg, cert, dss, &provider().signature_verification_algorithms)
}
impl ServerCertVerifier for Verifier {
    fn verify_server_cert(&self, cert: &CertificateDer<'_>, chain: &[CertificateDer<'_>], _: &ServerName<'_>, _: &[u8], _: UnixTime) -> Result<ServerCertVerified, Error> {
        self.check(cert, chain)?;
        Ok(ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(&self, _: &[u8], _: &CertificateDer<'_>, _: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, Error> { Err(refused("TLS 1.2 disabled")) }
    fn verify_tls13_signature(&self, msg: &[u8], cert: &CertificateDer<'_>, dss: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, Error> { signature(msg, cert, dss) }
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> { vec![SignatureScheme::ED25519] }
}
impl ClientCertVerifier for Verifier {
    fn root_hint_subjects(&self) -> &[rustls::DistinguishedName] { &[] }
    fn client_auth_mandatory(&self) -> bool { true }
    fn verify_client_cert(&self, cert: &CertificateDer<'_>, chain: &[CertificateDer<'_>], _: UnixTime) -> Result<ClientCertVerified, Error> {
        self.check(cert, chain)?;
        Ok(ClientCertVerified::assertion())
    }
    fn verify_tls12_signature(&self, _: &[u8], _: &CertificateDer<'_>, _: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, Error> { Err(refused("TLS 1.2 disabled")) }
    fn verify_tls13_signature(&self, msg: &[u8], cert: &CertificateDer<'_>, dss: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, Error> { signature(msg, cert, dss) }
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> { vec![SignatureScheme::ED25519] }
}

/// RFC 9266 tls-exporter channel binding. Fails before handshake completion.
pub fn channel_binding(conn: &rustls::Connection) -> Result<[u8; 32], Error> {
    conn.export_keying_material([0; 32], b"EXPORTER-Channel-Binding", Some(&[]))
}
