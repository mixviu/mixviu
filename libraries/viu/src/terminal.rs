// SPDX-License-Identifier: MIT OR Apache-2.0
//! Reusable native terminal facilities. Mix owns application composition.
//! Session descriptor wire constants are retained for the existing Mix login
//! bootstrap; importing them does not grant authority to an unverified child.
mod blink;
mod clusters;
pub mod config;
pub mod control;
pub mod font;
pub mod gpu;
pub mod ime;
pub mod input;
pub mod metrics;
pub mod model;
pub mod native_session;
pub mod raster;
pub mod session_actor;
pub mod session_fd;
pub mod sessions;
pub mod view;
pub use model::{Key, Screen, ScrollRequest, Terminal};

#[cfg(test)]
mod gpu_parity;

#[cfg(test)]
#[path = "../../../vendor/teletypewriter/patch_guard.rs"]
mod teletypewriter_patch_guard;
