// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! The workspace suppression on the engine's input path (the workspace
//! hit-test gap).
//!
//! `CompState::hidden` (carries a workspace, and minimised or off the
//! current workspace) already gates the draw through `DrawWindow::visible`,
//! but the central hit driver (`surface_under_filtered_cx`, used by touch,
//! tablet and the canvas press) reads only the window itself.
//! [`sync_hidden`] stamps the decision on each window (`ident::set_hidden`),
//! and the driver reads `ident::is_shown` (`is_drawn && !is_hidden`).
//! `is_drawn` is untouched: it feeds the wire-visible `windows.*.visible`
//! (whose projection already adds the hidden term), the truth lists and the
//! chrome. The foreign-toplevel mirror stays on `is_drawn` too: it exports
//! minimised and off-workspace windows, so a dock can restore them.

use std::cell::Cell;

use dispatcher::wire::trait_::surface_event::SurfaceHandle;
use dispatcher::wire::trait_::wire_trait::WireTrait;
use protocols::window::ident;

use crate::state::Loop;

thread_local! {
    /// The CompState revision the last stamp ran at.
    static STAMPED: Cell<Option<u64>> = const { Cell::new(None) };
}

/// One loop pass: stamp every world window with the policy's hidden decision,
/// when CompState moved since the last stamp. Returns whether any stamp
/// changed.
pub fn sync_hidden(lp: &mut Loop) -> bool {
    // Space ownership may change without a visibility revision. Reconcile the
    // persistent request in the owning world's order before draw or hit testing.
    let updates: Vec<_> = lp
        .inner
        .all_world_spaces()
        .into_iter()
        .flat_map(|s| s.state.elements())
        .filter_map(|w| {
            let id = comp_id(lp, w)?;
            Some((
                lp.inner.world_of_window(w)?,
                lp.inner.comp.registry.uuid_for(id)?,
                lp.inner.comp.visual(id).z,
            ))
        })
        .collect();
    for (owner, uuid, z) in updates {
        lp.inner
            .worlds
            .get_mut(owner)
            .storage_mut()
            .get_mut(&crate::order::track::base::DRAW_ORDER_MUT)
            .set_z(crate::order::track::base::ComponentId(uuid), z);
    }
    let revision = lp.inner.comp.revision();
    if STAMPED.with(Cell::get) == Some(revision) {
        return false;
    }
    STAMPED.with(|stamped| stamped.set(Some(revision)));
    let comp = &lp.inner.comp;
    let mut changed = false;
    for space in lp.inner.all_world_spaces() {
        for window in space.state.elements() {
            let visual = SurfaceHandle::of_window(window)
                .and_then(|h| comp.registry.id_for_handle(&h))
                .map(|id| comp.visual(id))
                .unwrap_or_default();
            *window
                .user_data()
                .get_or_insert(|| {
                    std::sync::Mutex::new(comp_model::request::VisualAttributes::default())
                })
                .lock()
                .unwrap() = visual;
            let hidden =
                SurfaceHandle::of_window(window).is_some_and(|handle| comp.hidden(&handle));
            changed |= ident::set_hidden(window, hidden);
        }
    }
    changed
}

fn comp_id(lp: &Loop, window: &smithay::desktop::Window) -> Option<surfaces::SurfaceId> {
    let id = lp
        .inner
        .comp
        .registry
        .id_for_handle(&SurfaceHandle::of_window(window)?)?;
    lp.inner
        .comp
        .registry
        .get(id)?
        .role()
        .managed_toplevel()
        .then_some(id)
}
