// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Protocol ownership only; visibility is decided by the policy lane.
use crate::state::Dispatch;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::idle_inhibit::IdleInhibitHandler;

#[derive(Default)]
pub struct Inhibitors {
    // One entry per resource, including multiple inhibitors on one surface.
    pub surfaces: Vec<WlSurface>,
    pub revision: u64,
    pub projected: Option<(u64, u64, u64, bool)>,
}

impl IdleInhibitHandler for Dispatch {
    fn inhibit(&mut self, surface: WlSurface) {
        self.inhibitors.surfaces.push(surface);
        self.inhibitors.revision = self.inhibitors.revision.wrapping_add(1);
    }

    fn uninhibit(&mut self, surface: WlSurface) {
        if let Some(index) = self.inhibitors.surfaces.iter().position(|s| s == &surface) {
            self.inhibitors.surfaces.swap_remove(index);
            self.inhibitors.revision = self.inhibitors.revision.wrapping_add(1);
        }
    }
}
