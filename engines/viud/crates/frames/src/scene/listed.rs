// SPDX-License-Identifier: MIT OR Apache-2.0
//! Output-sized recording scene. No presentation pixels or screen UI enter it.
use super::*;
use smithay::backend::renderer::damage::OutputDamageTracker;
use smithay::output::Output;
use std::cell::RefCell;

thread_local! {
    // Only one Bus take is active. A fresh secret resets damage at take start.
    static TRACKER: RefCell<Option<(String, OutputDamageTracker)>> = const { RefCell::new(None) };
}

pub fn service(state: &mut Loop, renderer: &mut GlesRenderer, output: &Output) {
    let old_output = state.inner.render_output.clone();
    let old_target = state.inner.render_target;
    let key = world::state::output_key(output);
    state.inner.render_output = Some(key.clone());
    screencopy::file::service_listed(renderer, output, |renderer, spec| {
        let size = output
            .current_mode()
            .ok_or_else(|| screencopy::file::capture_failed("output has no mode"))?
            .size;
        let scale = output.current_scale().fractional_scale();
        let locked = world::comp::session_lock::active(state);
        let listed = spec.listed_windows.as_ref().expect("listed capture");
        let mut allowed = std::collections::HashSet::new();
        for row in state.inner.comp.registry.surface_rows() {
            if !locked
                && row.role().managed_toplevel()
                && row.mapped()
                && listed.iter().any(|w| {
                    w.id == row.id().0
                        && w.generation == row.generation()
                        && world::comp::recording::on_output(state, *w, output)
                })
            {
                if let Some(uuid) = state.inner.comp.registry.uuid_for(row.id()) {
                    allowed.insert(uuid);
                }
            }
        }
        let mut plan = Plan::new();
        if !locked
            && world::comp::recording::cursor_on_output(
                spec.cursor,
                state.inner.cursor_output.as_deref(),
                &key,
            )
        {
            let allow_client_cursor =
                world::comp::recording::cursor_focus_allowed(state, listed, output);
            plan.extend(
                layer::POINTER,
                seat::pointer::draw::scene::recording_element(state, renderer, size, allow_client_cursor)
                    .into_iter()
                    .map(DrawNode::Pointer),
            );
        }
        // Only the backdrop's Background roots, never panels or their popups.
        // Namespace is routing metadata, not a peer-authorisation claim.
        let map = smithay::desktop::layer_map_for_output(output);
        for surface in map
            .layers()
            .rev()
            .filter(|s| !locked && s.layer() == WlrLayer::Background && s.namespace() == "backdrop")
        {
            if world::comp::panels::concealed(surface.wl_surface()) {
                continue;
            }
            if let Some(geometry) = map.layer_geometry(surface) {
                plan.push(
                    layer::LAYER_BACKGROUND,
                    DrawNode::Surface(SurfaceNode {
                        surface: surface.wl_surface().clone(),
                        location: geometry.loc.to_f64().to_physical(scale).to_i32_round(),
                        scale,
                        alpha: 1.0,
                    }),
                );
            }
        }
        drop(map);
        let computed = world::viewport::layout::compute(
            state.inner.viewports(),
            smithay::utils::Rectangle::from_size(size),
        );
        if locked {
            plan.push(
                layer::BACKGROUND,
                DrawNode::Solid(SolidColorRenderElement::new(
                    background_id(0),
                    smithay::utils::Rectangle::from_size(size),
                    CommitCounter::default(),
                    [0.0, 0.0, 0.0, 1.0],
                    Kind::Unspecified,
                )),
            );
        }
        for (index, region) in computed
            .regions
            .iter()
            .enumerate()
            .rev()
            .filter(|_| !locked)
        {
            state.inner.render_target = Some(world::state::RenderTarget {
                slot: region.slot,
                origin_logical: (
                    region.rect.loc.x as f64 / scale,
                    region.rect.loc.y as f64 / scale,
                ),
                size_physical: (region.rect.size.w as f64, region.rect.size.h as f64),
            });
            let context = world::canvas::draw::scene::context(state, renderer, size);
            let mut occ = world::window::draw::occlude::Occluders::new();
            let windows: std::collections::HashMap<_, _> = state
                .inner
                .space_state()
                .state
                .elements()
                .filter_map(|w| {
                    w.uuid()
                        .filter(|u| allowed.contains(u))
                        .map(|u| (u, w.clone()))
                })
                .collect();
            for uuid in state.inner.drawable_order() {
                if let Some(window) = windows.get(&uuid) {
                    let (elements, drawn) = world::window::draw::frame::scene::scene(
                        state, renderer, size, window, &context, &mut occ,
                    );
                    occ.extend(&drawn.opaque);
                    if drawn.visible {
                        window.send_frame(
                            output,
                            state.inner.start_time.elapsed(),
                            Some(std::time::Duration::from_millis(16)),
                            |_, _| Some(output.clone()),
                        );
                    }
                    for elem in elements {
                        plan.push(
                            layer::CANVAS,
                            DrawNode::Canvas {
                                elem: world::canvas::draw::element::Element::Window(elem),
                            },
                        );
                    }
                }
            }
            plan.push(
                layer::BACKGROUND,
                DrawNode::Solid(SolidColorRenderElement::new(
                    background_id(index),
                    region.rect,
                    CommitCounter::default(),
                    BACKGROUND_COLOR,
                    Kind::Unspecified,
                )),
            );
        }
        state.inner.render_target = old_target;
        let (elements, _) = plan.lower(renderer);
        let session_key = spec.take_id.as_deref().ok_or_else(|| {
            screencopy::file::capture_failed("listed capture has no take identity")
        })?;
        let changed = TRACKER
            .with_borrow_mut(|cache| {
                if cache.as_ref().is_none_or(|(key, _)| key != session_key) {
                    *cache = Some((
                        session_key.to_owned(),
                        OutputDamageTracker::from_output(output),
                    ));
                }
                cache
                    .as_mut()
                    .unwrap()
                    .1
                    .damage_output(1, &elements)
                    .map(|(damage, _)| damage.is_some_and(|d| !d.is_empty()))
            })
            .map_err(|e| screencopy::file::capture_failed(e.to_string()))?;
        if !changed && !spec.force_capture {
            return Ok(None);
        }
        screencopy::render_offscreen(renderer, &elements, |_| true, size, scale)
            .map(Some)
            .map_err(screencopy::file::capture_failed)
    });
    state.inner.render_target = old_target;
    state.inner.render_output = old_output;
}
