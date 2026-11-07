// SPDX-License-Identifier: MIT OR Apache-2.0
//! Reusable native terminal facilities. Mix owns application composition.
//! Session descriptor wire constants are retained for the existing Mix login
//! bootstrap; importing them does not grant authority to an unverified child.
pub mod session_fd;

#[cfg(test)]
#[path = "../../../vendor/teletypewriter/patch_guard.rs"]
mod teletypewriter_patch_guard;
