// SPDX-License-Identifier: MIT OR Apache-2.0
//! Explicit test-support input fixture. No Bus command or production API.
use smithay::input::pointer::{ButtonEvent, MotionEvent};
use smithay::backend::input::ButtonState;
use smithay::utils::SERIAL_COUNTER;
use smithay::wayland::seat::WaylandFocus;
use world::state::Loop;
thread_local! { static PREVIOUS: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) }; }
pub fn before_frame(state: &mut Loop, output: &smithay::output::Output) {
    let Some(root) = std::env::var_os("MIX_TEST_RECORD_FIXTURE") else {
        return;
    };
    let mode = std::fs::read_to_string(std::path::Path::new(&root).join("record-input"))
        .unwrap_or_default();
    state.inner.cursor_output = match mode.as_str() {
        "unknown" => None,
        "other" => Some(comp_model::snapshot::output_key("headless-1")),
        _ => Some(world::state::output_key(output)),
    };
    state.inner.comp.injection.agent_pointer = None;
    if PREVIOUS.with_borrow(|last| *last == mode) {
        return;
    }
    let app = if mode == "private" {
        "cursor-private"
    } else {
        "cursor-listed"
    };
    let Some(row) = state
        .inner
        .comp
        .registry
        .surface_rows()
        .find(|r| r.app_id().is_some_and(|s| s == app))
    else {
        return;
    };
    let id = row.id();
    let generation = row.generation();
    let Ok(window) = world::window::draw::frame::scene::capture_window(state, id.0, generation)
    else {
        return;
    };
    let Some(surface) = window.wl_surface().map(|s| s.into_owned()) else {
        return;
    };
    let Some(pointer) = state.state.seat.seat.get_pointer() else {
        return;
    };
    let location = state
        .inner
        .space_state()
        .state
        .element_location(&window)
        .unwrap_or_default()
        .to_f64();
    pointer.button(
        &mut state.state,
        &ButtonEvent {
            serial: SERIAL_COUNTER.next_serial(),
            time: 0,
            button: 0x110,
            state: ButtonState::Released,
        },
    );
    pointer.motion(
        &mut state.state,
        Some((surface, location)),
        &MotionEvent {
            location: location + (20.0, 20.0).into(),
            serial: SERIAL_COUNTER.next_serial(),
            time: 0,
        },
    );
    pointer.button(
        &mut state.state,
        &ButtonEvent {
            serial: SERIAL_COUNTER.next_serial(),
            time: 0,
            button: 0x110,
            state: ButtonState::Pressed,
        },
    );
    pointer.frame(&mut state.state);
    PREVIOUS.with_borrow_mut(|last| *last = mode);
}
