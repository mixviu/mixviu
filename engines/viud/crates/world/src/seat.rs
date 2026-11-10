// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
pub mod gesture;
pub mod pointer;
pub mod system;

/// Timestamp for an input event the compositor produces itself, on libinput's
/// clock (`CLOCK_MONOTONIC` milliseconds, wrapped to u32).
///
/// Every event a seat sends must share one time base. Real device events carry
/// libinput's; so must synthesized releases, replayed motion, Bus-injected input
/// and focus-loss key releases. Not wall-clock time and not time since viud
/// started: either puts a synthesized event at an arbitrary offset from the
/// real ones around it.
pub fn event_time_ms() -> u32 {
    smithay::utils::Clock::<smithay::utils::Monotonic>::new()
        .now()
        .as_millis()
}
