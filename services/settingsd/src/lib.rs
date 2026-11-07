// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
pub mod authority;
pub mod contract;
pub mod downgrade;
pub mod format4;
pub mod migration;
pub mod mirror;
pub mod mirroring;
pub mod publication;
pub mod service;
pub mod store;

mod daemon;
pub use daemon::main;

/// Component version, listed by the mixd engine's `--personalities`.
pub const COMPONENT_VERSION: &str = env!("CARGO_PKG_VERSION");
