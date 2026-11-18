// SPDX-License-Identifier: MIT OR Apache-2.0
//! LAB ONLY: inventory-pinned TLS 1.3, sharing the node's D2 signing seed.
use std::sync::Arc;
use rustls::{DigitallySignedStruct, Error, SignatureScheme};
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer, ServerName, UnixTime};
use rustls::client::danger::{ServerCertVerifier, ServerCertVerified, HandshakeSignatureValid};
use rustls::server::danger::{ClientCertVerifier, ClientCertVerified};

pub type Lookup = Arc<dyn Fn(&[u8; 32]) -> Option<String> + Send + Sync>;

#[derive(Clone)]
pub struct Material {
    pub cert: CertificateDer<'static>,
    seed: [u8; 32],
    pub lookup: Lookup,
}

impl Material {
    pub fn new(seed: [u8; 32], lookup: Lookup) -> anyhow::Result<Self> {
        let key = Self::key(seed);
        let pair = rcgen::KeyPair::from_pkcs8_der_and_sign_algo(
            &PrivatePkcs8KeyDer::from(key), &rcgen::PKCS_ED25519)?;
        let params = rcgen::CertificateParams::new(vec!["node.example.invalid".into()])?;
        let cert = params.self_signed(&pair)?.der().clone();
        Ok(Self { cert, seed, lookup })
    }
    fn key(seed: [u8; 32]) -> Vec<u8> {
        // RFC 8410 Ed25519 PKCS#8, seed inside the private-key OCTET STRING.
        let mut key = vec![0x30,0x2e,0x02,0x01,0,0x30,0x05,0x06,0x03,0x2b,0x65,0x70,0x04,0x22,0x04,0x20];
        key.extend_from_slice(&seed);
        key
    }
    fn private_key(&self) -> PrivateKeyDer<'static> {
        PrivatePkcs8KeyDer::from(Self::key(self.seed)).into()
    }
    pub fn server(&self) -> anyhow::Result<Arc<rustls::ServerConfig>> {
        let mut c = rustls::ServerConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_protocol_versions(&[&rustls::version::TLS13])?
            .with_client_cert_verifier(Arc::new(Pin { lookup: self.lookup.clone(), expected: None }))
            .with_single_cert(vec![self.cert.clone()], self.private_key())?;
        c.max_early_data_size = 0;
        c.send_tls13_tickets = 0;
        c.session_storage = Arc::new(rustls::server::NoServerSessionStorage {});
        Ok(Arc::new(c))
    }
    pub fn client(&self, expected: String) -> anyhow::Result<Arc<rustls::ClientConfig>> {
        let mut c = rustls::ClientConfig::builder_with_provider(Arc::new(rustls::crypto::ring::default_provider()))
            .with_protocol_versions(&[&rustls::version::TLS13])?
            .dangerous().with_custom_certificate_verifier(Arc::new(Pin { lookup: self.lookup.clone(), expected: Some(expected) }))
            .with_client_auth_cert(vec![self.cert.clone()], self.private_key())?;
        c.enable_early_data = false;
        c.resumption = rustls::client::Resumption::disabled();
        Ok(Arc::new(c))
    }
}

// Bounded DER walk to the actual TBSCertificate.subjectPublicKeyInfo, never a
// byte search (a hostile extension could contain a copied member SPKI).
fn tlv<'a>(input: &mut &'a [u8], tag: u8) -> Option<&'a [u8]> {
    if *input.first()? != tag { return None; }
    let first = *input.get(1)?;
    let (header, len) = if first < 128 { (2, first as usize) } else {
        let n = (first & 127) as usize;
        if n == 0 || n > 4 { return None; }
        let mut len = 0usize;
        for b in input.get(2..2+n)? { len = len.checked_mul(256)?.checked_add(*b as usize)?; }
        (2+n, len)
    };
    let value = input.get(header..header.checked_add(len)?)?;
    *input = input.get(header+len..)?;
    Some(value)
}
pub fn public_key(cert: &CertificateDer<'_>) -> Option<[u8; 32]> {
    let mut der = cert.as_ref();
    let mut outer = tlv(&mut der, 0x30)?;
    if !der.is_empty() { return None; }
    let mut tbs = tlv(&mut outer, 0x30)?;
    if tbs.first() == Some(&0xa0) { tlv(&mut tbs, 0xa0)?; }
    tlv(&mut tbs, 0x02)?; // serial
    for _ in 0..4 { tlv(&mut tbs, 0x30)?; } // signature, issuer, validity, subject
    let mut spki = tlv(&mut tbs, 0x30)?;
    if tlv(&mut spki, 0x30)? != [0x06,0x03,0x2b,0x65,0x70] { return None; }
    let bits = tlv(&mut spki, 0x03)?;
    if !spki.is_empty() || bits.len() != 33 || bits[0] != 0 { return None; }
    bits[1..].try_into().ok()
}

struct Pin { lookup: Lookup, expected: Option<String> }
impl std::fmt::Debug for Pin {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { f.write_str("InventoryPin") }
}
impl Pin {
    fn check(&self, cert: &CertificateDer<'_>, intermediates: &[CertificateDer<'_>]) -> Result<(), Error> {
        if !intermediates.is_empty() { return Err(Error::General("chain refused".into())); }
        let node = public_key(cert).and_then(|key| (self.lookup)(&key))
            .ok_or_else(|| Error::General("key absent from active inventory".into()))?;
        if self.expected.as_ref().is_some_and(|expected| expected != &node) {
            return Err(Error::General("wrong inventory member".into()));
        }
        Ok(())
    }
}
fn signature(msg: &[u8], cert: &CertificateDer<'_>, dss: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, Error> {
    rustls::crypto::verify_tls13_signature(msg, cert, dss, &rustls::crypto::ring::default_provider().signature_verification_algorithms)
}
impl ServerCertVerifier for Pin {
    fn verify_server_cert(&self, cert: &CertificateDer<'_>, chain: &[CertificateDer<'_>], _: &ServerName<'_>, _: &[u8], _: UnixTime) -> Result<ServerCertVerified, Error> {
        self.check(cert, chain)?; Ok(ServerCertVerified::assertion())
    }
    fn verify_tls12_signature(&self, _: &[u8], _: &CertificateDer<'_>, _: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, Error> { Err(Error::General("TLS 1.2 disabled".into())) }
    fn verify_tls13_signature(&self, msg: &[u8], cert: &CertificateDer<'_>, dss: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, Error> { signature(msg, cert, dss) }
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> { vec![SignatureScheme::ED25519] }
}
impl ClientCertVerifier for Pin {
    fn root_hint_subjects(&self) -> &[rustls::DistinguishedName] { &[] }
    fn verify_client_cert(&self, cert: &CertificateDer<'_>, chain: &[CertificateDer<'_>], _: UnixTime) -> Result<ClientCertVerified, Error> {
        self.check(cert, chain)?; Ok(ClientCertVerified::assertion())
    }
    fn verify_tls12_signature(&self, _: &[u8], _: &CertificateDer<'_>, _: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, Error> { Err(Error::General("TLS 1.2 disabled".into())) }
    fn verify_tls13_signature(&self, msg: &[u8], cert: &CertificateDer<'_>, dss: &DigitallySignedStruct) -> Result<HandshakeSignatureValid, Error> { signature(msg, cert, dss) }
    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> { vec![SignatureScheme::ED25519] }
}
