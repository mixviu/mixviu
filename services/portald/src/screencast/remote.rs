// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! `OpenPipeWireRemote`: a connected socket to the session's PipeWire, the
//! same instance the cast nodes live on. The caller speaks the PipeWire
//! protocol over it.
//!
//! The session's callers are trusted and unsandboxed, so the remote is not
//! restricted to the session's nodes; a sandbox-facing portal needs a
//! permission-limited remote first.
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

/// The PipeWire socket, resolved as libpipewire resolves it: an absolute
/// `PIPEWIRE_REMOTE`, else `PIPEWIRE_REMOTE` (default `pipewire-0`) in
/// `PIPEWIRE_RUNTIME_DIR`, then `XDG_RUNTIME_DIR`.
pub fn socket_path() -> anyhow::Result<PathBuf> {
    let name = paths::env::setting(paths::env::PIPEWIRE_REMOTE)?;
    if name.starts_with('/') {
        return Ok(PathBuf::from(name));
    }
    let dir = paths::env::raw(paths::env::PIPEWIRE_RUNTIME_DIR)
        .or_else(|| paths::env::raw(paths::env::XDG_RUNTIME_DIR))
        .ok_or_else(|| anyhow::anyhow!("neither PIPEWIRE_RUNTIME_DIR nor XDG_RUNTIME_DIR is set"))?;
    Ok(PathBuf::from(dir).join(name))
}

pub fn connect() -> anyhow::Result<UnixStream> {
    let path = socket_path()?;
    UnixStream::connect(&path).map_err(|error| anyhow::anyhow!("{}: {error}", path.display()))
}
