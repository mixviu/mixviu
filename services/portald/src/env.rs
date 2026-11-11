// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! The environment portald is started with. The unit passes the session bus
//! address through `EnvironmentFile=`; portald keeps it.
use std::path::PathBuf;

/// The session bus address, required and non-empty.
pub fn session_address() -> anyhow::Result<String> {
    let address = paths::env::setting(paths::env::DBUS_SESSION_BUS_ADDRESS)?;
    anyhow::ensure!(
        !address.trim().is_empty(),
        "DBUS_SESSION_BUS_ADDRESS is empty"
    );
    Ok(address)
}

/// The portal cache follows the selected paths state root.
pub fn state_directory() -> anyhow::Result<PathBuf> {
    paths::resolve("mix:var/portald").map_err(anyhow::Error::msg)
}
