// SPDX-License-Identifier: MIT OR Apache-2.0
//! Capture buffer leases only at an impending unmap/destruction, before the
//! renderer drops its current buffer. Import remains on the render path.
use smithay::backend::renderer::element::Id;
use smithay::backend::renderer::utils::{Buffer, with_renderer_surface_state};
use smithay::reexports::wayland_server::{Resource, Weak, protocol::wl_surface::WlSurface};
use smithay::wayland::compositor::{
    BufferAssignment, SurfaceAttributes, add_destruction_hook, add_pre_commit_hook, with_states,
};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};

thread_local! {
    static SOURCES: RefCell<HashMap<Id, Weak<WlSurface>>> = RefCell::new(HashMap::new());
    static ARMED: RefCell<HashSet<Id>> = RefCell::new(HashSet::new());
    static CAPTURED: RefCell<HashMap<Id, Buffer>> = RefCell::new(HashMap::new());
    static ACTIVE: RefCell<HashSet<Id>> = RefCell::new(HashSet::new());
    static LAST_METRICS: RefCell<Option<(usize, u64)>> = const { RefCell::new(None) };
}

fn report() {
    let metrics = CAPTURED.with(|captured| {
        let captured = captured.borrow();
        let bytes = captured
            .values()
            .filter_map(|buffer| smithay::backend::renderer::buffer_dimensions(buffer))
            .map(|size| size.w.max(0) as u64 * size.h.max(0) as u64 * 4)
            .sum();
        (captured.len(), bytes)
    });
    let changed = LAST_METRICS.with(|last| last.borrow_mut().replace(metrics) != Some(metrics));
    if changed {
        info!(
            "window_exit_buffers buffers={} pixel_bytes={}",
            metrics.0, metrics.1
        );
    }
}

pub fn register<D: 'static>(surface: &WlSurface) {
    let id = Id::from_wayland_resource(surface);
    SOURCES.with(|sources| {
        sources.borrow_mut().insert(id, surface.downgrade());
    });
    add_pre_commit_hook::<D, _>(surface, |_, _, surface| {
        let (removed, replaced) = with_states(surface, |states| {
            let mut cached = states.cached_state.get::<SurfaceAttributes>();
            (
                matches!(cached.pending().buffer, Some(BufferAssignment::Removed)),
                matches!(
                    cached.pending().buffer,
                    Some(BufferAssignment::NewBuffer(_))
                ),
            )
        });
        let id = Id::from_wayland_resource(surface);
        if replaced && !ACTIVE.with(|active| active.borrow().contains(&id)) {
            CAPTURED.with(|captured| {
                captured.borrow_mut().remove(&id);
            });
        }
        if removed {
            capture_surface(surface);
        }
    });
    // Registered in new_surface, before on_commit_buffer_handler installs its
    // renderer-state reset hook. Also covers abrupt client disconnects.
    add_destruction_hook::<D, _>(surface, |_, surface| {
        capture_surface(surface);
        SOURCES.with(|sources| {
            sources
                .borrow_mut()
                .remove(&Id::from_wayland_resource(surface));
        });
    });
}

pub fn arm(id: Id) {
    ARMED.with(|armed| {
        armed.borrow_mut().insert(id);
    });
}

fn capture_surface(surface: &WlSurface) {
    let id = Id::from_wayland_resource(surface);
    if !ARMED.with(|armed| armed.borrow().contains(&id)) {
        return;
    }
    if CAPTURED.with(|captured| captured.borrow().contains_key(&id)) {
        return;
    }
    if let Some(buffer) =
        with_renderer_surface_state(surface, |state| state.buffer().cloned()).flatten()
    {
        CAPTURED.with(|captured| {
            captured.borrow_mut().insert(id, buffer);
        });
        report();
    }
}

pub fn capture(id: &Id) {
    let surface = SOURCES.with(|sources| {
        sources
            .borrow()
            .get(id)
            .and_then(|surface| surface.upgrade().ok())
    });
    if let Some(surface) = surface {
        capture_surface(&surface);
    }
    ACTIVE.with(|active| {
        active.borrow_mut().insert(id.clone());
    });
}

pub fn buffer(id: &Id) -> Option<Buffer> {
    CAPTURED.with(|captured| captured.borrow().get(id).cloned())
}

// Id hashes immutable identity fields and addresses; Wayland's atomic liveness is ignored by Hash/Eq.
#[allow(clippy::mutable_key_type)]
pub fn prune(armed: &HashSet<Id>, exiting: &HashSet<Id>) {
    ARMED.with(|sources| sources.borrow_mut().retain(|id| armed.contains(id)));
    CAPTURED.with(|captured| captured.borrow_mut().retain(|id, _| exiting.contains(id)));
    ACTIVE.with(|active| active.borrow_mut().retain(|id| exiting.contains(id)));
    report();
}

pub fn clear() {
    ARMED.with(|armed| armed.borrow_mut().clear());
    CAPTURED.with(|captured| captured.borrow_mut().clear());
    ACTIVE.with(|active| active.borrow_mut().clear());
    report();
}
