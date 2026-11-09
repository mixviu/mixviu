// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! `zwp_tablet_tool_v2` tip down/up — the stroke boundary. The role is latched at
//! tip-down from the active tool:
//!   * **Hand** (canvas grab or touch pane) → `Pan`: the tip-drag glide-pans the world.
//!   * **Select** (touch pane) → arm the transient select grab + press → rubber-band.
//!   * **Light touch** → a pure hovering cursor: the tip never presses (clicks come
//!     only from the barrel buttons).
//!   * otherwise → `Tablet` draw over a tablet-aware window, a `Pointer` click over any
//!     other window / UI overlay, or NOTHING over bare canvas (a plain pen tip must
//!     not start a canvas pan — that's the Hand tool's job — so a spurious pen-down
//!     from a bad driver can't pan).
//!
//! Mirrors how `touch/session.rs` maps `TouchMode` onto the same canvas mechanisms.

use crate::pointer::input::tablet::{client, coords, cursor, hand_active, select_active};
use dispatcher::wire::tablet::Stroke;
use smithay::backend::input::{
    Event, InputBackend, TabletToolDescriptor, TabletToolEvent, TabletToolTipEvent,
    TabletToolTipState,
};
use smithay::utils::{Logical, Physical, Point, SERIAL_COUNTER};
use world::state::Loop;
use world::state::export::{CanvasGrab, TargetOption};

/// Begin a stroke at `world` (`screen` = the physical cursor position): a native tool
/// tip on a tablet-aware surface; else a `Pointer` click over any other window / UI
/// overlay (the pen as a mouse); else — over bare canvas — NOTHING, so a plain pen tip
/// never initiates a canvas pan. Only reached outside the Hand/Select/Light-touch modes.
pub fn start_draw(
    _loop: &mut Loop,
    tool: &TabletToolDescriptor,
    screen: Point<f64, Physical>,
    world: Point<f64, Logical>,
    time: u32,
) {
    super::revoke::flush(_loop, time);
    if _loop.state.tablet.stroke_owner.is_some() {
        return;
    }
    if let Some(focus) = client::tablet_focus(_loop, world) {
        if !_loop
            .state
            .tablet
            .focus_stroke(tool, world, focus, SERIAL_COUNTER.next_serial(), time)
        {
            return;
        }
        super::revoke::flush(_loop, time);
        client::apply_focus(_loop, world);
        let serial = SERIAL_COUNTER.next_serial();
        if _loop.state.tablet.begin_stroke(tool, Stroke::Tablet) {
            _loop.state.tablet.tool_tip_down(tool, serial, time);
        }
    } else if let Some(hit) = client::topmost(_loop, world) {
        // A window / layer / UI overlay (NOT bare canvas) → the pen acts as a mouse.
        if !_loop.state.tablet.begin_stroke(tool, Stroke::Pointer) {
            return;
        }
        _loop.state.tablet.stroke_surface = client::focus_of(&hit).map(|(surface, _)| surface);
        cursor::follow(_loop, screen, world, time);
        crate::pointer::input::touch::emulate::press(_loop, time);
    } else {
        // Bare canvas → inert: no press means no canvas pan from a plain pen tip.
        _loop.state.tablet.stroke = Stroke::None;
    }
}

/// End the latched stroke (no-op when none / a pan is active — the pan just stops).
pub fn end_draw(_loop: &mut Loop, tool: &TabletToolDescriptor, time: u32) {
    if _loop.state.tablet.stroke_owner.as_ref() != Some(tool) {
        return;
    }
    match _loop.state.tablet.stroke {
        Stroke::Tablet => _loop.state.tablet.tool_tip_up(tool, time),
        Stroke::Pointer => crate::pointer::input::touch::emulate::release(_loop, time),
        Stroke::Pan | Stroke::None => {}
    }
    _loop.state.tablet.stroke = Stroke::None;
    _loop.state.tablet.stroke_owner = None;
    _loop.state.tablet.stroke_surface = None;
}

/// Clear the transient Select grab on lift, mirroring `touch/session.rs::disarm_select`.
pub(super) fn disarm_select(_loop: &mut Loop) {
    if matches!(
        _loop.inner.canvas().grab,
        CanvasGrab::Target(TargetOption::Select { .. })
    ) {
        _loop.inner.canvas_mut().grab = CanvasGrab::None;
    }
}

pub fn tip<I: InputBackend>(event: &I::TabletToolTipEvent, _loop: &mut Loop) {
    let tool = event.tool();
    let time = event.time_msec();
    let (screen, world) = coords::world::<I, _>(event, _loop);
    match event.tip_state() {
        TabletToolTipState::Down => {
            super::revoke::flush(_loop, time);
            if _loop.state.tablet.stroke_owner.is_some() {
                return;
            }
            _loop.state.tablet.last_pen_screen = Some(screen);
            if hand_active(_loop) {
                if !_loop.state.tablet.begin_stroke(&tool, Stroke::Pan) {
                    return;
                }
                // Hand tool: the tip-drag pans (axis.rs); no draw, no click.
                cursor::follow(_loop, screen, world, time);
            } else if select_active(_loop) {
                if !_loop.state.tablet.begin_stroke(&tool, Stroke::Pointer) {
                    return;
                }
                // Select tool: arm the transient grab (disarmed on lift) so the
                // emulated press + drag rubber-band selects, like the touch session.
                _loop.inner.canvas_mut().grab =
                    CanvasGrab::Target(TargetOption::Select { append: true });
                cursor::follow(_loop, screen, world, time);
                crate::pointer::input::touch::emulate::press(_loop, time);
            } else if _loop.inner.preference.pen.below_threshold_cursor {
                // Light touch: a pure hovering cursor — the tip never presses.
                cursor::follow(_loop, screen, world, time);
            } else {
                start_draw(_loop, &tool, screen, world, time);
            }
        }
        TabletToolTipState::Up => {
            if _loop.state.tablet.stroke_owner.as_ref() != Some(&tool) {
                return;
            }
            end_draw(_loop, &tool, time);
            disarm_select(_loop);
        }
    }
}
