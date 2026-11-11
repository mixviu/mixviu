// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! The environment portald is started with. The unit passes the session bus
//! address through `EnvironmentFile=`; portald keeps it.
use std::path::PathBuf;

/// The session bus address, required and non-empty.
pub fn session_address() -> anyhow::Result<String> {
    let address = paths::env::raw(paths::env::DBUS_SESSION_BUS_ADDRESS)
        .and_then(|value| value.into_string().ok())
        .ok_or_else(|| anyhow::anyhow!("DBUS_SESSION_BUS_ADDRESS is not set"))?;
    anyhow::ensure!(
        !address.trim().is_empty(),
        "DBUS_SESSION_BUS_ADDRESS is empty"
    );
    Ok(address)
}

/// The first directory systemd provides through `StateDirectory=`. Without
/// one (and without `--state-dir`) the portal runs cache-less.
pub fn state_directory() -> Option<PathBuf> {
    paths::env::raw(paths::env::STATE_DIRECTORY)?
        .into_string()
        .ok()?
        .split(':')
        .find(|part| !part.is_empty())
        .map(PathBuf::from)
}
