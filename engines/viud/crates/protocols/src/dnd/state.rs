// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;

pub struct DNDState {
    pub icon: Option<WlSurface>,
    /// Includes iconless drags: a grab permits the source to set the cursor
    /// even when the pointer is over another client's target.
    pub active: bool,
}
