// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! The `org.freedesktop.portal.ScreenCast` interface (version 5): monitor
//! sources from viud's screencopy, delivered as PipeWire video nodes.
//!
//! Scope of this first cut, deliberately narrow:
//! - **Monitor sources only**, one stream per session: the first output with a
//!   current mode. `multiple` is accepted and ignored.
//! - **Consent** ([`consent`]): every Start is granted by policy, a
//!   standing grant, or the person answering a notifyd question; an agent can
//!   answer or pre-grant over the Bus (`portald.screencast.*`, local callers
//!   only).
//! - **No restore tokens**: `persist_mode` is accepted and ignored.
//! - **shm frames**, copied twice (screencopy buffer, PipeWire buffer).
#[cfg(feature = "screencast")]
pub mod capture;
pub mod consent;
#[cfg(feature = "screencast")]
mod enabled;
pub mod handle;
pub mod remote;
pub mod request;
#[cfg(feature = "screencast")]
pub mod session;
#[cfg(feature = "screencast")]
pub mod stream;
#[cfg(feature = "screencast")]
pub use enabled::*;
use zbus::Connection;
pub const INTERFACE: &str = "org.freedesktop.portal.ScreenCast";
pub const VERSION: u32 = 5;
pub const SOURCE_MONITOR: u32 = 1;
pub const CURSOR_HIDDEN: u32 = 1;
pub const CURSOR_EMBEDDED: u32 = 2;
/// What `--screencast` serves with. `portald serve` resolves both from its
/// flags or the installed settings (`MIX_PORTALD_CONSENT`,
/// `MIX_PORTALD_MAX_FRAMERATE`) at start-up; there is no built-in default.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Config {
    pub consent: consent::Policy,
    pub max_framerate: MaxFramerate,
}

/// A validated frame-rate ceiling offered to consumers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaxFramerate(u32);

impl MaxFramerate {
    pub fn get(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for MaxFramerate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl std::str::FromStr for MaxFramerate {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value
            .parse::<u32>()
            .ok()
            .filter(|rate| (1..=120).contains(rate))
            .map(Self)
            .ok_or_else(|| {
                "SCREENCAST_MAX_FRAMERATE: --screencast-max-framerate must be in 1..=120".into()
            })
    }
}

/// The caller's process id, for naming it in the prompt and matching grants.
pub(crate) async fn caller_pid(connection: &Connection, owner: &str) -> Option<u32> {
    let proxy = zbus::fdo::DBusProxy::new(connection).await.ok()?;
    let name = zbus::names::BusName::try_from(owner.to_owned()).ok()?;
    proxy.get_connection_unix_process_id(name).await.ok()
}
