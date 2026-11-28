// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>

//! The Bus: MixViu's message bus wire format and broker client.
//!
//! - [`wire`] is the frozen wire format every Bus peer speaks: `---`-fenced
//!   headers plus an optional body ([`BusMessage`], [`parse`]). It needs
//!   only `serde_json` and is always compiled.
//! - [`client`] (the default `client` feature) is the WebSocket client that
//!   registers a service with the local broker, noded, and keeps it
//!   registered across broker restarts ([`SupervisedClient`]).
//!
//! The wire format and the commands the client speaks (`noded.register`,
//! `topic.subscribe`, …) are contracts with a running broker: their bytes
//! never change.

pub mod wire;
// Source compatibility names refer to the same message and parser implementation.
pub use wire as bus;
pub mod native_session;
mod protocol;
pub mod service_info;
pub use protocol::*;

/// Manifest limits in broker return-code precedence order.
pub const MAX_MANIFEST_NAMES: usize = 512;
pub const MAX_MANIFEST_NAME_BYTES: usize = 64;
pub const MAX_MANIFEST_NAME_SEGMENTS: usize = 6;

/// A rejected name or manifest position, in broker return-code precedence order.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ManifestNameError { Count, Bytes, Segments, Charset }

impl ManifestNameError {
    pub fn rc(self) -> &'static str {
        match self { Self::Count => "21", Self::Bytes => "22", Self::Segments => "23", Self::Charset => "24" }
    }
}

/// Validate one command name at its zero-based position in the manifest.
pub fn validate_manifest_name(name: &str, index: usize) -> Result<(), ManifestNameError> {
    if index >= MAX_MANIFEST_NAMES { return Err(ManifestNameError::Count); }
    if name.len() > MAX_MANIFEST_NAME_BYTES { return Err(ManifestNameError::Bytes); }
    if name.split('.').count() > MAX_MANIFEST_NAME_SEGMENTS { return Err(ManifestNameError::Segments); }
    if name.split('.').any(|part| part.is_empty() || !part.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')) {
        return Err(ManifestNameError::Charset);
    }
    Ok(())
}

#[cfg(test)]
mod manifest_name_tests {
    use super::*;
    #[test]
    fn shared_limits_and_boundaries() {
        for name in ["a".repeat(64), "a.b.c.d.e.f".into()] {
            assert_eq!(validate_manifest_name(&name, 511), Ok(()));
        }
        for (name, index, error) in [
            ("example.read".into(), 512, ManifestNameError::Count),
            ("a".repeat(65), 0, ManifestNameError::Bytes),
            ("a.b.c.d.e.f.g".into(), 0, ManifestNameError::Segments),
            ("example@bad".into(), 0, ManifestNameError::Charset),
            ("a..b".into(), 0, ManifestNameError::Charset),
        ] { assert_eq!(validate_manifest_name(&name, index), Err(error)); }
    }
}
pub use service_info::{
    NodeInfo, REGISTRATION_REJECTION_NAME_TAKEN, REGISTRATION_REJECTION_SCHEMA, RegisterProvenance,
    SCHEMA_VERSION, ServiceInfo,
};

#[cfg(feature = "native")]
pub mod agent_client;
#[cfg(feature = "client")]
pub mod client_helpers;
#[cfg(feature = "client")]
pub mod native_client;

pub use wire::{
    BusMessage, EMPTY_MESSAGE, MAX_HEADERS, MAX_MESSAGE_BYTES, ParseError, ParseReport,
    WS_MAX_FRAME_BYTES, parse, parse_lenient, parse_strict,
};

/// Return codes carried in the `rc` header. The convention is ARexx's:
/// 0 succeeded, 5 succeeded with a warning, 10 is an error the caller can
/// act on, 20 a failure of the responder itself.
pub const RC_SUCCESS: u8 = 0;
pub const RC_WARNING: u8 = 5;
pub const RC_ERROR: u8 = 10;
pub const RC_FAILURE: u8 = 20;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "client")]
pub use client::{
    ClientError, ConnState, Connection, DEFAULT_NODED_URL, IncomingCommand, MAX_INITIAL_ATTEMPTS,
    RegistrationRejected, RegistrationRejectionKind, SubscriptionRegistry, SupervisedClient,
    SupervisedConnectOptions, SupervisedDelivery, SupervisedError, noded_url,
};

#[cfg(all(feature = "client", unix))]
mod native_port;
#[cfg(all(feature = "client", unix))]
pub use native_port::call_port_typed;
