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

pub use crate::seed::{BusTlsSeed, SeedFileError};
use ed25519_dalek::Signer as _;
use mesh_trust::identity::{TlsAuthority, TlsMember};
use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::server::danger::{ClientCertVerified, ClientCertVerifier};
use rustls::{DigitallySignedStruct, Error, SignatureScheme};
use std::sync::{Arc, LazyLock, RwLock};
use zeroize::Zeroizing;

/// Replace the immutable cache only after accepting a signed inventory.
pub type LiveAuthority = Arc<RwLock<Arc<TlsAuthority>>>;

const REFUSAL_WINDOW: std::time::Duration = std::time::Duration::from_secs(60);
const MAX_REFUSAL_WINDOWS: usize = 4096;

struct RefusalWindow {
    started: std::time::Instant,
    pending: u64,
}

#[derive(Default)]
struct RefusalWindows {
    peers: std::collections::HashMap<(std::net::IpAddr, &'static str), RefusalWindow>,
    overflow: Option<RefusalWindow>,
    next_expiry: Option<std::time::Instant>,
    classes: std::collections::BTreeMap<&'static str, u64>,
    overflow_classes: std::collections::HashSet<&'static str>,
}

pub(crate) fn refusal_network(peer: std::net::SocketAddr) -> std::net::IpAddr {
    match peer.ip() {
        std::net::IpAddr::V4(ip) => ip.into(),
        std::net::IpAddr::V6(ip) => {
            // IPv4-mapped addresses must share the native IPv4 bucket.
            if let Some(ip) = ip.to_ipv4_mapped() {
                return ip.into();
            }
            let mut bytes = ip.octets();
            bytes[8..].fill(0);
            std::net::Ipv6Addr::from(bytes).into()
        }
    }
}

#[derive(Default)]
pub struct Refusals {
    total: std::sync::atomic::AtomicU64,
    overflow_total: std::sync::atomic::AtomicU64,
    peers: std::sync::Mutex<RefusalWindows>,
    admitted: std::sync::atomic::AtomicU64,
}
impl Refusals {
    pub fn admitted(&self) {
        self.admitted
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
    /// Snapshot also flushes expired coalesced windows after a silent burst.
    pub fn snapshot(&self) -> serde_json::Value {
        self.flush();
        let windows = self.peers.lock().expect("TLS refusals poisoned");
        let timeout = windows.classes.get("timeout").copied().unwrap_or(0);
        serde_json::json!({
            "classes": windows.classes, "total": self.total(),
            "overflow": self.overflow_total(),
            "outcomes": {
                "admitted": self.admitted.load(std::sync::atomic::Ordering::Relaxed),
                "refused": self.total() - timeout, "timeout": timeout
            }
        })
    }
    pub fn flush(&self) {
        self.flush_at(std::time::Instant::now());
    }
    fn flush_at(&self, now: std::time::Instant) {
        let mut windows = self.peers.lock().expect("TLS refusals poisoned");
        self.expire(&mut windows, now, self.total());
    }
    pub fn total(&self) -> u64 {
        self.total.load(std::sync::atomic::Ordering::Relaxed)
    }
    pub fn overflow_total(&self) -> u64 {
        self.overflow_total
            .load(std::sync::atomic::Ordering::Relaxed)
    }
    pub fn record(&self, peer: std::net::SocketAddr, class: &'static str) {
        self.record_at(peer, class, std::time::Instant::now());
    }
    fn record_at(&self, peer: std::net::SocketAddr, class: &'static str, now: std::time::Instant) {
        let total = self
            .total
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            + 1;
        let mut windows = self.peers.lock().expect("TLS refusals poisoned");
        *windows.classes.entry(class).or_default() += 1;
        self.expire(&mut windows, now, total);
        let network = refusal_network(peer);
        if let Some(window) = windows.peers.get_mut(&(network, class)) {
            window.pending += 1;
        } else if windows.peers.len() < MAX_REFUSAL_WINDOWS {
            tracing::warn!(event = "tls.refused", %peer, %network, class, count = 1u64, total, "TLS attempt: {class}");
            windows.peers.insert(
                (network, class),
                RefusalWindow {
                    started: now,
                    pending: 0,
                },
            );
            windows.next_expiry.get_or_insert(now + REFUSAL_WINDOW);
        } else {
            let overflow_total = self
                .overflow_total
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                + 1;
            if windows.overflow_classes.insert(class) {
                tracing::warn!(event = "tls.refused", %peer, %network, class, overflow_total, "TLS refusal map full");
            }
            if let Some(window) = &mut windows.overflow {
                window.pending += 1;
            } else {
                windows.overflow = Some(RefusalWindow {
                    started: now,
                    pending: 0,
                });
                windows.next_expiry.get_or_insert(now + REFUSAL_WINDOW);
            }
        }
    }
    fn expire(&self, windows: &mut RefusalWindows, now: std::time::Instant, total: u64) {
        // Scan only when the earliest window expires, not on every connection.
        if windows.next_expiry.is_some_and(|expiry| now >= expiry) {
            windows.peers.retain(|(network, class), window| {
            if now.duration_since(window.started) < REFUSAL_WINDOW {
                return true;
            }
            if window.pending > 0 {
                tracing::warn!(event = "tls.refused", %network, class, count = window.pending, total, "TLS refusals coalesced");
            }
            false
        });
            if windows
                .overflow
                .as_ref()
                .is_some_and(|w| now.duration_since(w.started) >= REFUSAL_WINDOW)
            {
                let window = windows.overflow.take().expect("overflow window");
                windows.overflow_classes.clear();
                if window.pending > 0 {
                    tracing::warn!(
                        event = "tls.refused",
                        class = "overflow",
                        count = window.pending,
                        total,
                        overflow_total = self.overflow_total(),
                        "TLS refusals coalesced"
                    );
                }
            }
            windows.next_expiry = windows
                .peers
                .values()
                .map(|w| w.started + REFUSAL_WINDOW)
                .chain(windows.overflow.iter().map(|w| w.started + REFUSAL_WINDOW))
                .min();
        }
    }
}
pub static DIAL_REFUSALS: LazyLock<Refusals> = LazyLock::new(Default::default);

/// Raw 32-byte TLS seed, normally mix:etc/noded/bus-tls.seed.
/// Metadata is checked on the opened file, avoiding a stat/open race.
pub fn load_seed(path: &std::path::Path) -> Result<BusTlsSeed, SeedFileError> {
    let mut opened = crate::seed::open_seed(path)?;
    if !opened.secure {
        return Err(SeedFileError::Insecure {
            mode: opened.mode,
            owner: opened.owner,
        });
    }
    let mut bytes = Zeroizing::new([0; 33]);
    let mut len = 0;
    while len < bytes.len() {
        let n = opened.read(&mut bytes[len..])?;
        if n == 0 {
            break;
        }
        len += n;
    }
    if len != 32 {
        return Err(SeedFileError::Length);
    }
    let mut seed = Zeroizing::new([0; 32]);
    seed.copy_from_slice(&bytes[..32]);
    Ok(BusTlsSeed::new(*seed))
}

// Dalek's zeroize feature erases the signing key on final drop. Keeping rcgen
// on its remote-signing API avoids creating non-zeroizing ring private keys.
struct Signing {
    key: ed25519_dalek::SigningKey,
    public: [u8; 32],
}
impl std::fmt::Debug for Signing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("BusTlsSigningKey")
    }
}
struct CertificateSigner(Arc<Signing>);
impl rcgen::RemoteKeyPair for CertificateSigner {
    fn public_key(&self) -> &[u8] {
        &self.0.public
    }
    fn sign(&self, msg: &[u8]) -> Result<Vec<u8>, rcgen::Error> {
        Ok(self.0.key.sign(msg).to_bytes().to_vec())
    }
    fn algorithm(&self) -> &'static rcgen::SignatureAlgorithm {
        &rcgen::PKCS_ED25519
    }
}
#[derive(Debug)]
struct HandshakeSigner(Arc<Signing>);
impl rustls::sign::Signer for HandshakeSigner {
    fn sign(&self, msg: &[u8]) -> Result<Vec<u8>, Error> {
        Ok(self.0.key.sign(msg).to_bytes().to_vec())
    }
    fn scheme(&self) -> SignatureScheme {
        SignatureScheme::ED25519
    }
}
#[derive(Debug)]
struct TlsSigningKey(Arc<Signing>);
impl rustls::sign::SigningKey for TlsSigningKey {
    fn choose_scheme(&self, offered: &[SignatureScheme]) -> Option<Box<dyn rustls::sign::Signer>> {
        offered
            .contains(&SignatureScheme::ED25519)
            .then(|| Box::new(HandshakeSigner(self.0.clone())) as Box<dyn rustls::sign::Signer>)
    }
    fn algorithm(&self) -> rustls::SignatureAlgorithm {
        rustls::SignatureAlgorithm::ED25519
    }
}

/// Owns public certificate and zeroizing private material. Private bytes are
/// never passed into rcgen or the ring rustls key loader.
/// D2 material cannot construct a TLS credential through the public API:
/// ```compile_fail
/// let d2 = mesh::seed::D2Seed::new([7; 32]);
/// let tls = mesh::tls::Credential::new(d2);
/// ```
pub struct Credential {
    pub certificate: CertificateDer<'static>,
    certified: Arc<rustls::sign::CertifiedKey>,
}
impl Credential {
    pub fn new(seed: BusTlsSeed) -> anyhow::Result<Self> {
        let key = ed25519_dalek::SigningKey::from_bytes(seed.bytes());
        // Erase the source immediately; only Dalek's zeroizing signing key lives on.
        drop(seed);
        let public = key.verifying_key().to_bytes();
        let signing = Arc::new(Signing { key, public });
        let pair = rcgen::KeyPair::from_remote(Box::new(CertificateSigner(signing.clone())))?;
        let params = rcgen::CertificateParams::new(Vec::<String>::new())?;
        let certificate = params.self_signed(&pair)?.der().clone();
        let certified = Arc::new(rustls::sign::CertifiedKey::new(
            vec![certificate.clone()],
            Arc::new(TlsSigningKey(signing)),
        ));
        Ok(Self {
            certificate,
            certified,
        })
    }
    pub fn server_config(
        &self,
        authority: LiveAuthority,
    ) -> Result<Arc<rustls::ServerConfig>, Error> {
        let mut config = rustls::ServerConfig::builder_with_provider(provider())
            .with_protocol_versions(&[&rustls::version::TLS13])?
            .with_client_cert_verifier(Arc::new(Verifier {
                authority,
                expected: None,
            }))
            .with_cert_resolver(Arc::new(Resolver(self.certified.clone())));
        config.max_early_data_size = 0;
        config.send_tls13_tickets = 0;
        config.session_storage = Arc::new(rustls::server::NoServerSessionStorage {});
        Ok(Arc::new(config))
    }
    pub fn client_config(
        &self,
        authority: LiveAuthority,
        expected: String,
    ) -> Result<Arc<rustls::ClientConfig>, Error> {
        let mut config = rustls::ClientConfig::builder_with_provider(provider())
            .with_protocol_versions(&[&rustls::version::TLS13])?
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(Verifier {
                authority,
                expected: Some(expected),
            }))
            .with_client_cert_resolver(Arc::new(Resolver(self.certified.clone())));
        config.enable_early_data = false;
        config.resumption = rustls::client::Resumption::disabled();
        Ok(Arc::new(config))
    }
}
fn provider() -> Arc<rustls::crypto::CryptoProvider> {
    static PROVIDER: LazyLock<Arc<rustls::crypto::CryptoProvider>> =
        LazyLock::new(|| Arc::new(rustls::crypto::ring::default_provider()));
    PROVIDER.clone()
}
#[derive(Debug)]
struct Resolver(Arc<rustls::sign::CertifiedKey>);
impl rustls::server::ResolvesServerCert for Resolver {
    fn resolve(
        &self,
        _: rustls::server::ClientHello<'_>,
    ) -> Option<Arc<rustls::sign::CertifiedKey>> {
        Some(self.0.clone())
    }
}
impl rustls::client::ResolvesClientCert for Resolver {
    fn resolve(
        &self,
        _: &[&[u8]],
        schemes: &[SignatureScheme],
    ) -> Option<Arc<rustls::sign::CertifiedKey>> {
        schemes
            .contains(&SignatureScheme::ED25519)
            .then(|| self.0.clone())
    }
    fn has_certs(&self) -> bool {
        true
    }
}
fn refused(message: &str) -> Error {
    Error::General(message.into())
}

/// Parse the actual TBSCertificate SPKI, never a byte search. The canonical
/// SPKI encoding is compared in full, rejecting non-minimal lengths and any
/// extra algorithm fields even if the X.509 parser accepts BER equivalents.
pub fn public_key(cert: &CertificateDer<'_>) -> Result<[u8; 32], Error> {
    use x509_parser::prelude::{FromDer, X509Certificate};
    let (rest, _) = x509_parser::der_parser::der::parse_der(cert.as_ref())
        .map_err(|_| refused("non-canonical DER certificate"))?;
    if !rest.is_empty() {
        return Err(refused("trailing certificate data"));
    }
    let (rest, parsed) =
        X509Certificate::from_der(cert.as_ref()).map_err(|_| refused("invalid certificate"))?;
    if !rest.is_empty() {
        return Err(refused("trailing certificate data"));
    }
    let spki = parsed.public_key();
    if spki.algorithm.algorithm.to_id_string() != "1.3.101.112"
        || spki.algorithm.parameters.is_some()
        || spki.subject_public_key.unused_bits != 0
    {
        return Err(refused("expected parameter-free Ed25519 SPKI"));
    }
    let key: [u8; 32] = spki
        .subject_public_key
        .data
        .as_ref()
        .try_into()
        .map_err(|_| refused("invalid Ed25519 key length"))?;
    let mut canonical = vec![
        0x30, 0x2a, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x03, 0x21, 0,
    ];
    canonical.extend_from_slice(&key);
    if spki.raw != canonical {
        return Err(refused("non-canonical SPKI"));
    }
    Ok(key)
}

/// Obtain a typed identity from the exact certificate rustls authenticated.
/// Call only after handshake completion; no certificate-only API proves possession.
///
/// Callers MUST call this after EVERY handshake: membership is checked in
/// certificate verification, but not re-checked inside verify_tls13_signature.
/// Diallers MUST pass the same member pin used to build the client config as
/// expected, including when a revision changes between verification and lookup.
pub fn verified_member(
    authority: &LiveAuthority,
    certificates: &[CertificateDer<'_>],
    expected: Option<&str>,
) -> Result<TlsMember, Error> {
    let [certificate] = certificates else {
        return Err(refused("exactly one certificate required"));
    };
    let key = public_key(certificate)?;
    let member = authority
        .read()
        .map_err(|_| refused("authority lock poisoned"))?
        .member_for(&key)
        .ok_or_else(|| refused("key absent or ambiguous in accepted inventory"))?;
    if expected.is_some_and(|expected| expected != member.member) {
        return Err(refused("wrong dialled member"));
    }
    Ok(member)
}
#[derive(Debug)]
struct Verifier {
    authority: LiveAuthority,
    expected: Option<String>,
}
impl Verifier {
    fn check(&self, cert: &CertificateDer<'_>, chain: &[CertificateDer<'_>]) -> Result<(), Error> {
        if !chain.is_empty() {
            return Err(refused("certificate chain refused"));
        }
        verified_member(
            &self.authority,
            std::slice::from_ref(cert),
            self.expected.as_deref(),
        )
        .map(|_| ())
    }
}
fn signature(
    msg: &[u8],
    cert: &CertificateDer<'_>,
    dss: &DigitallySignedStruct,
) -> Result<HandshakeSignatureValid, Error> {
    if dss.scheme != SignatureScheme::ED25519 {
        return Err(refused("only ED25519 is allowed"));
    }
    public_key(cert)?;
    rustls::crypto::verify_tls13_signature(
        msg,
        cert,
        dss,
        &provider().signature_verification_algorithms,
    )
}
impl ServerCertVerifier for Verifier {
    fn verify_server_cert(
        &self,
        cert: &CertificateDer<'_>,
        chain: &[CertificateDer<'_>],
        _: &ServerName<'_>,
        _: &[u8],
        _: UnixTime,
    ) -> Result<ServerCertVerified, Error> {
        self.check(cert, chain)?;
        Ok(ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Err(refused("TLS 1.2 disabled"))
    }
    fn verify_tls13_signature(
        &self,
        msg: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        signature(msg, cert, dss)
    }
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![SignatureScheme::ED25519]
    }
}
impl ClientCertVerifier for Verifier {
    fn root_hint_subjects(&self) -> &[rustls::DistinguishedName] {
        &[]
    }
    fn client_auth_mandatory(&self) -> bool {
        true
    }
    fn verify_client_cert(
        &self,
        cert: &CertificateDer<'_>,
        chain: &[CertificateDer<'_>],
        _: UnixTime,
    ) -> Result<ClientCertVerified, Error> {
        self.check(cert, chain)?;
        Ok(ClientCertVerified::assertion())
    }
    fn verify_tls12_signature(
        &self,
        _: &[u8],
        _: &CertificateDer<'_>,
        _: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Err(refused("TLS 1.2 disabled"))
    }
    fn verify_tls13_signature(
        &self,
        msg: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        signature(msg, cert, dss)
    }
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![SignatureScheme::ED25519]
    }
}

/// RFC 9266 tls-exporter channel binding. Fails before handshake completion.
pub fn channel_binding(conn: &rustls::Connection) -> Result<[u8; 32], Error> {
    conn.export_keying_material([0; 32], b"EXPORTER-Channel-Binding", Some(&[]))
}

/// The same RFC 9266 helper for tokio-rustls's borrowed connection halves.
pub fn server_channel_binding(conn: &rustls::ServerConnection) -> Result<[u8; 32], Error> {
    conn.export_keying_material([0; 32], b"EXPORTER-Channel-Binding", Some(&[]))
}
pub fn client_channel_binding(conn: &rustls::ClientConnection) -> Result<[u8; 32], Error> {
    conn.export_keying_material([0; 32], b"EXPORTER-Channel-Binding", Some(&[]))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;
    use mesh_trust::inventory::*;
    use serde_json::{Value, json};
    use std::collections::BTreeMap;

    #[test]
    fn refusal_networks_group_ipv6_by_64_and_keep_classes_separate() {
        let refusals = Refusals::default();
        for peer in [
            "[2001:db8:1:2::1]:1",
            "[2001:db8:1:2:ffff::2]:2",
            "[2001:db8:1:3::1]:1",
        ] {
            refusals.record(peer.parse().unwrap(), "timeout");
        }
        assert_eq!(refusals.peers.lock().unwrap().peers.len(), 2);
        refusals.record("[2001:db8:1:2::1]:3".parse().unwrap(), "queue-full");
        assert_eq!(refusals.peers.lock().unwrap().peers.len(), 3);
        assert_eq!(
            refusal_network("[::ffff:192.0.2.1]:1".parse().unwrap()),
            "192.0.2.1".parse::<std::net::IpAddr>().unwrap()
        );
    }

    #[test]
    fn refusal_windows_are_capped_and_pruned_with_a_counted_overflow() {
        let refusals = Refusals::default();
        let now = std::time::Instant::now();
        for n in 0..MAX_REFUSAL_WINDOWS + 1000 {
            let ip = std::net::Ipv6Addr::new(0x2001, 0xdb8, n as u16, 0, 0, 0, 0, 1);
            refusals.record_at((ip, 1).into(), "timeout", now);
            assert!(refusals.peers.lock().unwrap().peers.len() <= MAX_REFUSAL_WINDOWS);
        }
        assert_eq!(refusals.overflow_total(), 1000);
        assert_eq!(
            refusals
                .peers
                .lock()
                .unwrap()
                .overflow
                .as_ref()
                .unwrap()
                .pending,
            999
        );
        let next = "192.0.2.1:2".parse().unwrap();
        refusals.record_at(next, "timeout", now + REFUSAL_WINDOW);
        let windows = refusals.peers.lock().unwrap();
        assert_eq!(windows.peers.len(), 1);
        assert!(windows.peers.contains_key(&(next.ip(), "timeout")));
        assert!(windows.overflow.is_none());
        assert_eq!(refusals.total(), (MAX_REFUSAL_WINDOWS + 1001) as u64);
        assert_eq!(refusals.overflow_total(), 1000);
    }

    #[test]
    fn refusal_expiry_reports_pending_counts_before_pruning_or_reuse() {
        // Run alone so process-wide tracing interest cannot race other tests.
        let test = "tls::tests::refusal_expiry_reports_pending_counts_before_pruning_or_reuse";
        if std::env::var("MIXVIU_LOG_CAPTURE_TEST").as_deref() != Ok(test) {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", test, "--nocapture"])
                .env("MIXVIU_LOG_CAPTURE_TEST", test)
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{}\n{}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
            return;
        }
        #[derive(Clone, Default)]
        struct Capture(Arc<std::sync::Mutex<Vec<u8>>>);
        impl std::io::Write for Capture {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let capture = Capture::default();
        let writer = capture.clone();
        let subscriber = tracing_subscriber::fmt()
            .without_time()
            .with_ansi(false)
            .with_writer(move || writer.clone())
            .finish();
        let _guard = tracing::subscriber::set_default(subscriber);
        let refusals = Refusals::default();
        let now = std::time::Instant::now();
        for port in 1..=1000 {
            refusals.record_at(([192, 0, 2, 1], port).into(), "timeout", now);
        }
        // Expiry with another peer prunes, rather than silently losing 999.
        refusals.record_at(
            "192.0.2.2:1".parse().unwrap(),
            "timeout",
            now + REFUSAL_WINDOW,
        );
        // Reuse starts a fresh window, whose pending count is also reported.
        refusals.record_at(
            "192.0.2.2:2".parse().unwrap(),
            "timeout",
            now + REFUSAL_WINDOW,
        );
        refusals.record_at(
            "192.0.2.2:3".parse().unwrap(),
            "timeout",
            now + 2 * REFUSAL_WINDOW,
        );
        let log = String::from_utf8(capture.0.lock().unwrap().clone()).unwrap();
        assert_eq!(log.lines().count(), 5, "{log}");
        assert!(log.contains("count=999"), "{log}");
        assert!(log.contains("total=1003"), "{log}");
        assert_eq!(refusals.total(), 1003);
        // No later refusal is needed to emit the pending count.
        refusals.record_at(
            "192.0.2.2:4".parse().unwrap(),
            "timeout",
            now + 2 * REFUSAL_WINDOW,
        );
        refusals.flush_at(now + 3 * REFUSAL_WINDOW);
        let log = String::from_utf8(capture.0.lock().unwrap().clone()).unwrap();
        assert!(log.lines().last().unwrap().contains("count=1"), "{log}");
        assert!(refusals.peers.lock().unwrap().peers.is_empty());
        let metric = refusals.snapshot();
        assert_eq!(metric["classes"]["timeout"], 1004);
        assert_eq!(metric["outcomes"]["timeout"], 1004);
        assert_eq!(metric["outcomes"]["refused"], 0);
    }

    fn credential(n: u8) -> Credential {
        Credential::new(BusTlsSeed::new([n; 32])).unwrap()
    }
    fn record(c: &Credential) -> Value {
        json!({"status":"active", "bus":true, "credentials":[{
            "kind":"bus-tls", "pubkey":base64::engine::general_purpose::STANDARD.encode(public_key(&c.certificate).unwrap()),
            "from_epoch":1, "until_epoch":null
        }]})
    }
    // All handshake fixtures go through the real signed inventory verifier.
    fn authority(members: BTreeMap<String, Value>, epoch: u64) -> Arc<TlsAuthority> {
        let genesis = ed25519_dalek::SigningKey::from_bytes(&[99; 32]);
        let b64 = base64::engine::general_purpose::STANDARD;
        let public = b64.encode(genesis.verifying_key().to_bytes());
        let payload: InventoryPayload = serde_json::from_value(json!({
            "schema_version":1, "canonical_encoding":"mixos-inventory-json-v1",
            "mesh":"example.test", "subnet":"192.0.2.0/24", "epoch":epoch,
            "signed_at":"2026-11-27T00:00:00Z", "valid_until":"2026-12-04T00:00:00Z",
            "hub":["alpha"], "verify_keys":[{"key_id":"genesis","pubkey":public,"key_type":"Ed25519","status":"active"}],
            "members":members.iter().map(|(name, value)| {
                let mut value = value.clone(); value["name"] = json!(name); value
            }).collect::<Vec<_>>()
        })).unwrap();
        let mut payload = payload;
        payload.canonical_encoding = CANONICAL_ENCODING_V1.into();
        let inventory = SignedInventory {
            signatures: vec![InvSignature {
                key_id: "genesis".into(),
                alg: "Ed25519".into(),
                sig: b64.encode(genesis.sign(&payload.canonical_bytes()).to_bytes()),
            }],
            payload,
        };
        let state = NodeTrustState {
            genesis_key_id: "genesis".into(),
            trusted_keys: vec![TrustedKey {
                key_id: "genesis".into(),
                pubkey: genesis.verifying_key().to_bytes().to_vec(),
                status: KeyStatus::Active,
            }],
            last_epoch: 0,
            last_recovery_generation: 0,
            last_canonical_hash: None,
        };
        let accepted = inventory.verify(&state).unwrap();
        Arc::new(TlsAuthority::new(&members, accepted.epoch, epoch + 10))
    }
    fn live(members: BTreeMap<String, Value>, epoch: u64) -> LiveAuthority {
        Arc::new(RwLock::new(authority(members, epoch)))
    }
    fn handshake(
        client: Arc<rustls::ClientConfig>,
        server: Arc<rustls::ServerConfig>,
    ) -> Result<(rustls::Connection, rustls::Connection), Error> {
        let mut client = rustls::Connection::Client(rustls::ClientConnection::new(
            client,
            ServerName::try_from("node.example.test").unwrap(),
        )?);
        let mut server = rustls::Connection::Server(rustls::ServerConnection::new(server)?);
        for _ in 0..20 {
            for direction in 0..2 {
                let (sender, receiver) = if direction == 0 {
                    (&mut client, &mut server)
                } else {
                    (&mut server, &mut client)
                };
                let mut bytes = Vec::new();
                while sender.wants_write() {
                    sender.write_tls(&mut bytes).unwrap();
                }
                if !bytes.is_empty() {
                    let mut cursor = std::io::Cursor::new(bytes);
                    receiver.read_tls(&mut cursor).unwrap();
                    receiver.process_new_packets()?;
                }
            }
            if !client.is_handshaking() && !server.is_handshaking() {
                return Ok((client, server));
            }
        }
        Err(refused("handshake stalled"))
    }
    #[test]
    fn mutual_identity_exporters_and_live_revocation() {
        let a = credential(1);
        let b = credential(2);
        let members = BTreeMap::from([("alpha".into(), record(&a)), ("beta".into(), record(&b))]);
        let live = live(members.clone(), 1);
        let cc = b.client_config(live.clone(), "alpha".into()).unwrap();
        let sc = a.server_config(live.clone()).unwrap();
        let (client, server) = handshake(cc.clone(), sc.clone()).unwrap();
        for (conn, name) in [(&client, "alpha"), (&server, "beta")] {
            let identity =
                verified_member(&live, conn.peer_certificates().unwrap(), Some(name)).unwrap();
            assert_eq!(identity.member, name);
            assert_eq!((identity.epoch, identity.revision), (1, 11));
        }
        let binding = channel_binding(&client).unwrap();
        assert_eq!(binding, channel_binding(&server).unwrap());
        let (other, _) = handshake(cc.clone(), sc.clone()).unwrap();
        assert_ne!(binding, channel_binding(&other).unwrap());
        let mut retired = members;
        retired.get_mut("beta").unwrap()["credentials"][0]["until_epoch"] = json!(2);
        *live.write().unwrap() = authority(retired, 2);
        assert!(handshake(cc, sc).is_err()); // Same configs and handle, newer cache.
    }
    #[test]
    fn refuses_untrusted_wrong_member_ambiguous_ineligible_and_epoch_keys() {
        let a = credential(1);
        let b = credential(2);
        let base = BTreeMap::from([("alpha".into(), record(&a)), ("beta".into(), record(&b))]);
        for case in 0..7 {
            let mut members = base.clone();
            let expected = if case == 1 { "beta" } else { "alpha" };
            match case {
                0 => {
                    members.remove("beta");
                }
                1 => {}
                2 => {
                    members.insert("gamma".into(), record(&b));
                }
                3 => {
                    members.get_mut("beta").unwrap()["status"] = json!("tombstoned");
                }
                4 => {
                    members.get_mut("beta").unwrap()["bus"] = json!(false);
                }
                5 => {
                    members.get_mut("beta").unwrap()["credentials"][0]["from_epoch"] = json!(2);
                }
                6 => {
                    members.get_mut("beta").unwrap()["credentials"][0]["until_epoch"] = json!(1);
                }
                _ => unreachable!(),
            }
            let live = live(members, 1);
            assert!(
                handshake(
                    b.client_config(live.clone(), expected.into()).unwrap(),
                    a.server_config(live).unwrap()
                )
                .is_err(),
                "case {case}"
            );
        }
    }
    #[test]
    fn client_refuses_ambiguous_and_ineligible_server_keys() {
        let a = credential(1);
        let b = credential(2);
        let base = BTreeMap::from([("alpha".into(), record(&a)), ("beta".into(), record(&b))]);
        // The server trusts beta in every case, so the client verifier must
        // refuse alpha rather than relying on a server-side client refusal.
        for case in 0..3 {
            let mut members = base.clone();
            match case {
                0 => {
                    members.insert("gamma".into(), record(&a));
                }
                1 => {
                    members.get_mut("alpha").unwrap()["status"] = json!("tombstoned");
                }
                2 => {
                    members.get_mut("alpha").unwrap()["bus"] = json!(false);
                }
                _ => unreachable!(),
            }
            let client_authority = live(members, 1);
            let server_authority = live(base.clone(), 1);
            assert!(
                handshake(
                    b.client_config(client_authority, "alpha".into()).unwrap(),
                    a.server_config(server_authority).unwrap(),
                )
                .is_err(),
                "client case {case}"
            );
        }
    }
    #[test]
    fn refuses_tls12_no_certificate_and_chains() {
        let a = credential(1);
        let b = credential(2);
        let live = live(
            BTreeMap::from([("alpha".into(), record(&a)), ("beta".into(), record(&b))]),
            1,
        );
        let tls12 = rustls::ClientConfig::builder_with_provider(provider())
            .with_protocol_versions(&[&rustls::version::TLS12])
            .unwrap()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(Verifier {
                authority: live.clone(),
                expected: Some("alpha".into()),
            }))
            .with_client_cert_resolver(Arc::new(Resolver(b.certified.clone())));
        assert!(handshake(Arc::new(tls12), a.server_config(live.clone()).unwrap()).is_err());
        let no_cert = rustls::ClientConfig::builder_with_provider(provider())
            .with_protocol_versions(&[&rustls::version::TLS13])
            .unwrap()
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(Verifier {
                authority: live.clone(),
                expected: Some("alpha".into()),
            }))
            .with_no_client_auth();
        assert!(handshake(Arc::new(no_cert), a.server_config(live.clone()).unwrap()).is_err());
        let pin = Verifier {
            authority: live,
            expected: None,
        };
        assert!(
            pin.check(&a.certificate, std::slice::from_ref(&b.certificate))
                .is_err()
        );
    }
    #[test]
    fn rejects_non_ed25519_and_malformed_spki() {
        let a = credential(1);
        let pair = rcgen::KeyPair::generate().unwrap();
        let ec = rcgen::CertificateParams::new(Vec::<String>::new())
            .unwrap()
            .self_signed(&pair)
            .unwrap();
        assert!(public_key(ec.der()).is_err());
        let mut trailing = a.certificate.as_ref().to_vec();
        trailing.push(0);
        assert!(public_key(&CertificateDer::from(trailing)).is_err());
        // Mutations inside the parsed SPKI. Repair outer lengths with a
        // fixture-only helper; production never walks DER by hand.
        for replacement in [
            vec![
                0x30, 0x81, 0x2a, 0x30, 5, 6, 3, 0x2b, 0x65, 0x70, 3, 0x21, 0,
            ],
            vec![
                0x30, 0x2c, 0x30, 7, 6, 3, 0x2b, 0x65, 0x70, 5, 0, 3, 0x21, 0,
            ],
        ] {
            let mut der = a.certificate.as_ref().to_vec();
            let offset = der
                .windows(12)
                .position(|w| w == [0x30, 0x2a, 0x30, 5, 6, 3, 0x2b, 0x65, 0x70, 3, 0x21, 0])
                .unwrap();
            let extra = replacement.len() - 12;
            // rcgen emits long outer length and short TBS length for this fixture.
            fn grow_length(bytes: &mut [u8], offset: usize, extra: usize) -> usize {
                let n = bytes[offset + 1];
                if n < 128 {
                    bytes[offset + 1] += extra as u8;
                    offset + 2
                } else {
                    let count = (n & 127) as usize;
                    let last = offset + 1 + count;
                    bytes[last] = bytes[last].checked_add(extra as u8).unwrap();
                    last + 1
                }
            }
            let tbs = grow_length(&mut der, 0, extra);
            grow_length(&mut der, tbs, extra);
            der.splice(offset..offset + 12, replacement);
            assert!(public_key(&CertificateDer::from(der)).is_err());
        }
    }
    #[test]
    fn seed_loader_enforces_permissions_and_exact_size() {
        use std::io::Write;
        use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
        let path = std::env::temp_dir().join(format!("bus-tls-{}.seed", uuid::Uuid::new_v4()));
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)
            .unwrap();
        for len in [31, 32, 33] {
            file.set_len(0).unwrap();
            use std::io::Seek;
            file.rewind().unwrap();
            file.write_all(&vec![7; len]).unwrap();
            assert_eq!(load_seed(&path).is_ok(), len == 32);
        }
        file.set_len(32).unwrap();
        file.set_permissions(std::fs::Permissions::from_mode(0o640))
            .unwrap();
        assert!(matches!(
            load_seed(&path),
            Err(SeedFileError::Insecure { .. })
        ));
        for mode in [0o604, 0o644, 0o660] {
            file.set_permissions(std::fs::Permissions::from_mode(mode))
                .unwrap();
            assert!(matches!(
                load_seed(&path),
                Err(SeedFileError::Insecure { .. })
            ));
        }
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn seed_loader_refuses_symlink_and_fifo_promptly() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        let directory = std::env::temp_dir().join(format!("bus-tls-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        let target = directory.join("seed");
        std::fs::write(&target, [7; 32]).unwrap();
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o640)).unwrap();
        let link = directory.join("link");
        symlink(&target, &link).unwrap();
        assert!(
            matches!(load_seed(&link), Err(SeedFileError::Io(error)) if error.raw_os_error() == Some(libc::ELOOP))
        );
        let fifo = directory.join("fifo");
        let name = std::ffi::CString::new(fifo.as_os_str().as_encoded_bytes()).unwrap();
        // SAFETY: name is a live NUL-terminated path, and mode is valid.
        assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = tx.send(load_seed(&fifo));
        });
        assert!(matches!(
            rx.recv_timeout(std::time::Duration::from_secs(1)).unwrap(),
            Err(SeedFileError::NotRegular)
        ));
        std::fs::remove_dir_all(directory).unwrap();
    }
}
