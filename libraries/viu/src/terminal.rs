// SPDX-License-Identifier: MIT OR Apache-2.0
//! Reusable native terminal facilities. Mix owns application composition.
//! Session descriptor wire constants are retained for the existing Mix login
//! bootstrap; importing them does not grant authority to an unverified child.
mod admission;
mod blink;
pub(crate) mod clipboard;
mod clusters;
pub mod config;
pub mod control;
pub mod font;
pub mod gpu;
pub mod ime;
pub mod input;
mod layout_control;
mod metadata;
pub mod metrics;
pub mod model;
pub mod native_session;
mod pointer;
pub mod protocol;
pub mod raster;
mod recipient;
mod replay;
mod selection_worker;
pub(crate) use selection_worker::WRITE_ANSWER;
pub mod session_actor;
pub mod session_fd;
pub mod sessions;
pub mod view;
mod view_search;
pub use model::{Key, Screen, ScrollRequest, Terminal};

#[cfg(test)]
mod gpu_parity;

#[cfg(test)]
#[path = "../../../vendor/teletypewriter/patch_guard.rs"]
mod teletypewriter_patch_guard;
