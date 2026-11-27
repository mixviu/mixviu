// SPDX-License-Identifier: MIT OR Apache-2.0
//! Explicit test-support input fixture. No Bus command or production API.
use smithay::backend::input::ButtonState;
use smithay::input::pointer::{ButtonEvent, MotionEvent};
use smithay::utils::SERIAL_COUNTER;
use smithay::wayland::seat::WaylandFocus;
use world::state::Loop;
thread_local! { static PREVIOUS: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) }; }
pub fn before_frame(state: &mut Loop, output: &smithay::output::Output) {
    let Some(root) = std::env::var_os(paths::env::test::TEST_RECORD_FIXTURE) else {
        return;
    };
    let mode = std::fs::read_to_string(std::path::Path::new(&root).join("record-input"))
        .unwrap_or_default();
    // The production camera is pinned. Exercise fractional pan/zoom contracts
    // through the existing opt-in fixture, after pinning and before composition.
    if let Some(args) = mode.strip_prefix("fractional-camera:") {
        let values: Vec<f64> = args.split(':').map(|v| v.parse().expect("fixture camera number")).collect();
        assert_eq!(values.len(), 3);
        let camera = state.inner.camera_mut();
        camera.transform.position.x = values[0];
        camera.transform.position.y = values[1];
        camera.transform.zoom = values[2];
        return;
    }
    if mode == "cold-readback" {
        screencopy::file::testing::penalize_first_readback();
        return;
    }
    if mode == "missing-answer" {
        if screencopy::file::testing::forget_raw_answer() {
            eprintln!("LISTED_TEST_MISSING_ANSWER fired");
        }
        return;
    }
    state.inner.cursor_output = match mode.as_str() {
        "unknown" => None,
        "other" => Some(comp_model::snapshot::output_key("headless-1")),
        _ => Some(world::state::output_key(output)),
    };
    state.inner.comp.injection.agent_pointer = None;
    let command = mode.split(':').next().unwrap_or_default();
    state.state.seat.force_cursor = command
        .ends_with("forced")
        .then_some(smithay::input::pointer::CursorIcon::Crosshair);
    if command != "overlap" && PREVIOUS.with_borrow(|last| *last == mode) {
        return;
    }
    let app = if command.starts_with("different") {
        "cursor-different"
    } else if command.starts_with("private") {
        "cursor-private"
    } else {
        "cursor-listed"
    };
    let Some(window) = state
        .inner
        .comp
        .registry
        .surface_rows()
        .filter(|r| r.app_id().is_some_and(|s| s.as_ref() == app))
        .find_map(|r| {
            world::window::draw::frame::scene::capture_window(state, r.id().0, r.generation()).ok()
        })
    else {
        return;
    };
    let Some(mut surface) = window.wl_surface().map(|s| s.into_owned()) else {
        return;
    };
    let Some(pointer) = state.state.seat.seat.get_pointer() else {
        return;
    };
    let mut location = state
        .inner
        .space_state()
        .state
        .element_location(&window)
        .unwrap_or_default()
        .to_f64();
    let mut point = location + smithay::utils::Point::from((20.0, 20.0));
    if command == "listed-popup" || command == "listed-sub" {
        // Use Smithay's real surface hit test; the test's enter event confirms
        // which child actually received focus (not just membership in a walk).
        let local = if command == "listed-popup" {
            (190.0, 50.0)
        } else {
            (50.0, 50.0)
        };
        let Some((child, offset)) =
            window.surface_under(local, smithay::desktop::WindowSurfaceType::ALL)
        else {
            panic!("listed child not mapped: {command}");
        };
        assert_ne!(child, surface, "fixture must focus the child");
        surface = child;
        point = location + smithay::utils::Point::from(local);
        location += offset.to_f64();
    }
    if command == "overlap" {
        let hit = world::surface::interface::hit::surface_under_filtered(state, point, &|_| true);
        let Some(world::surface::interface::hit::SurfaceHit::Window {
            surface: hit_surface,
            position,
            ..
        }) = hit
        else {
            return;
        };
        surface = hit_surface;
        location = position;
        if pointer.current_focus().as_ref() == Some(&surface) {
            return;
        }
    }
    let dragging = command == "drag-listed";
    if !dragging {
        pointer.button(
            &mut state.state,
            &ButtonEvent {
                serial: SERIAL_COUNTER.next_serial(),
                time: 0,
                button: 0x110,
                state: ButtonState::Released,
            },
        );
    }
    pointer.motion(
        &mut state.state,
        Some((surface, location)),
        &MotionEvent {
            location: point,
            serial: SERIAL_COUNTER.next_serial(),
            time: 0,
        },
    );
    if !dragging {
        pointer.button(
            &mut state.state,
            &ButtonEvent {
                serial: SERIAL_COUNTER.next_serial(),
                time: 0,
                button: 0x110,
                state: ButtonState::Pressed,
            },
        );
    }
    pointer.frame(&mut state.state);
    PREVIOUS.with_borrow_mut(|last| *last = mode);
}
