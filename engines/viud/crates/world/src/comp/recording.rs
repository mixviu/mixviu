// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared recording membership. Protocol output-enter membership is deliberately
//! all outputs in viud; recording instead uses the selected output's view.
use crate::state::Loop;
use comp_model::capture::CaptureWindow;
use smithay::output::Output;
use smithay::reexports::wayland_server::{Resource, protocol::wl_surface::WlSurface};
use smithay::wayland::seat::WaylandFocus;

pub fn on_output(state: &Loop, target: CaptureWindow, output: &Output) -> bool {
    let Ok(window) =
        crate::window::draw::frame::scene::capture_window(state, target.id, target.generation)
    else {
        return false;
    };
    let placement = state.inner.comp.placement(surfaces::SurfaceId(target.id));
    if state
        .inner
        .comp
        .fullscreen
        .selected(surfaces::SurfaceId(target.id))
        .is_some_and(|name| {
            name != output.name() && name != comp_model::snapshot::output_key(&output.name())
        })
    {
        return false;
    }
    if placement.output().is_some_and(|name| {
        name != output.name() && name != comp_model::snapshot::output_key(&output.name())
    }) {
        return false;
    }
    let Some(loc) = state.inner.space_state().state.element_location(&window) else {
        return false;
    };
    let Some(mode) = output.current_mode() else {
        return false;
    };
    let views = state
        .inner
        .output_views()
        .views(&crate::state::output_key(output));
    let layout =
        crate::viewport::layout::compute(views, smithay::utils::Rectangle::from_size(mode.size));
    let size = crate::camera::transform::translate::slot::expected_size(&window)
        .unwrap_or(window.geometry().size);
    let scale = output.current_scale().fractional_scale();
    layout.regions.iter().any(|region| {
        let Some(camera) = views.camera_of(region.slot) else {
            return false;
        };
        let k = camera.transform.zoom * scale;
        let x = (f64::from(loc.x) - camera.transform.position.x) * k
            + f64::from(region.rect.size.w) / 2.0;
        let y = (f64::from(loc.y) - camera.transform.position.y) * k
            + f64::from(region.rect.size.h) / 2.0;
        x < f64::from(region.rect.size.w)
            && y < f64::from(region.rect.size.h)
            && x + f64::from(size.w) * k > 0.0
            && y + f64::from(size.h) * k > 0.0
    })
}

pub fn cursor_client_allowed(
    state: &Loop,
    cursor: &WlSurface,
    listed: &[CaptureWindow],
    output: &Output,
) -> bool {
    let Some(owner) = cursor.client().map(|c| c.id()) else {
        return false;
    };
    listed.iter().any(|target| {
        on_output(state, *target, output)
            && crate::window::draw::frame::scene::capture_window(
                state,
                target.id,
                target.generation,
            )
            .ok()
            .and_then(|w| w.wl_surface().and_then(|s| s.client()).map(|c| c.id()))
                == Some(owner.clone())
    })
}

pub fn cursor_on_output(requested: bool, cursor_output: Option<&str>, recorded: &str) -> bool {
    requested && cursor_output == Some(recorded)
}

#[cfg(test)]
mod tests {
    #[test]
    fn cursor_output_fence_never_guesses() {
        for (requested, cursor, expected) in [
            (true, Some("a"), true),
            (true, Some("b"), false),
            (true, None, false),
            (false, Some("a"), false),
        ] {
            assert_eq!(super::cursor_on_output(requested, cursor, "a"), expected);
        }
    }
}
