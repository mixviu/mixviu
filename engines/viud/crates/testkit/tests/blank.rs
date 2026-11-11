// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Real keyboard delivery and binding queues behind the blank policy gate.
//! TestHost models backend-free wake; production Loop timer/backend wiring is
//! exercised separately, rather than pretending this harness owns a DRM device.

use testkit::Harness;

const EV_A: u32 = 30;
const EV_F5: u32 = 63;

fn harness() -> Harness {
    let mut h = Harness::new();
    h.client.bind_keyboards();
    h.roundtrip();
    let (surface, _, _) = h.mapped_toplevel(64, 48);
    h.focus_keyboard(&surface);
    h.client.state.keys.clear();
    h
}

#[test]
fn waking_key_press_repeat_and_release_reach_no_client_then_next_key_does() {
    let mut h = harness();
    h.wire.inner.comp.blank.blanked = true;
    assert!(h.key(EV_A + 8, true, false));
    assert!(!h.comp().blank.blanked);
    assert!(
        h.key(EV_A + 8, true, false),
        "repeat belongs to waking press"
    );
    assert!(h.key(EV_A + 8, false, false));
    assert!(h.client.state.keys.is_empty(), "{:?}", h.client.state.keys);
    assert!(h.wire.inner.comp.bindings.take_comp_fires().is_empty());
    assert!(h.wire.inner.comp.bindings.take_bus_fires().is_empty());
    // Positive delivery control: swallowing ends with the matched release.
    assert!(!h.key(EV_A + 8, true, false));
    assert!(!h.key(EV_A + 8, false, false));
    assert_eq!(h.client.state.keys, [(EV_A, true), (EV_A, false)]);
}

#[test]
fn waking_bound_key_fires_no_binding_and_release_stays_owned_after_bus_unblank() {
    let mut h = harness();
    let root = std::env::temp_dir().join(format!("viud-blank-keymap-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("keymap.json");
    std::fs::write(
        &path,
        r#"{"version":1,"physical":[{"stroke":{"code":63},"action":"user.f05"}]}"#,
    )
    .unwrap();
    h.wire.inner.comp.bindings.reload_user_keymap(&path);
    h.wire.inner.comp.blank.blanked = true;
    assert!(h.key(EV_F5 + 8, true, false));
    // A separate Bus off edge must not relinquish ownership of the release.
    h.wire.inner.comp.blank.blanked = false;
    assert!(h.key(EV_F5 + 8, false, false));
    assert_eq!(h.wire.inner.comp.bindings.user_fired, 0);
    assert!(h.wire.inner.comp.bindings.take_comp_fires().is_empty());
    assert!(h.wire.inner.comp.bindings.take_bus_fires().is_empty());
    assert!(h.client.state.keys.is_empty());
    // The binding is live: its next normal press fires once.
    assert!(h.key(EV_F5 + 8, true, false));
    assert!(h.key(EV_F5 + 8, false, false));
    let fired = h.wire.inner.comp.bindings.take_bus_fires();
    assert_eq!(fired.len(), 1);
    assert_eq!(fired[0].command, "user.f05");
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn blank_entry_releases_preheld_client_key_and_modifier_then_consumes_waking_release() {
    for (evdev, modifier) in [(EV_A, false), (42, true)] {
        let mut h = harness();
        assert!(!h.key(evdev + 8, true, false));
        assert_eq!(h.client.state.keys, [(evdev, true)]);
        let keyboard = h.wire.state.seat.seat.get_keyboard().unwrap();
        assert_eq!(keyboard.modifier_state().shift, modifier);
        h.blank(true);
        assert_eq!(h.client.state.keys, [(evdev, true), (evdev, false)]);
        assert!(keyboard.pressed_keys().is_empty());
        assert!(!keyboard.modifier_state().shift);
        assert!(h.key(evdev + 8, true, false), "preheld repeat is consumed");
        assert!(!h.comp().blank.blanked);
        assert!(h.key(evdev + 8, false, false));
        assert_eq!(
            h.client.state.keys.len(),
            2,
            "no second release reaches client"
        );
        assert!(h.wire.inner.comp.bindings.take_comp_fires().is_empty());
        assert!(h.wire.inner.comp.bindings.take_bus_fires().is_empty());
        assert!(!h.key(EV_A + 8, true, false));
        assert!(!h.key(EV_A + 8, false, false));
        assert_eq!(&h.client.state.keys[2..], [(EV_A, true), (EV_A, false)]);
    }
}

#[test]
fn blank_entry_releases_preheld_pointer_button_and_consumes_its_physical_release() {
    let mut h = harness();
    let (surface, _, _) = h.mapped_toplevel(64, 48);
    let button = 0x110;
    h.press(&surface, button);
    let pointer = h.wire.state.seat.seat.get_pointer().unwrap();
    assert_eq!(pointer.current_pressed(), [button]);
    assert_eq!(h.client.state.buttons.len(), 1);
    h.blank(true);
    assert!(pointer.current_pressed().is_empty());
    assert_eq!(h.client.state.buttons.len(), 2);
    assert_eq!(h.client.state.buttons[1].1, button);
    assert!(!h.client.state.buttons[1].2);
    assert_eq!(
        h.release(&surface, button),
        0,
        "waking physical release is consumed"
    );
    assert!(!h.comp().blank.blanked);
    assert_eq!(h.client.state.buttons.len(), 2);
    h.press(&surface, button);
    h.release(&surface, button);
    assert_eq!(h.client.state.buttons.len(), 4);
}

#[test]
fn blank_entry_ends_an_existing_user_binding_repeat_without_client_release() {
    let mut h = harness();
    let root = std::env::temp_dir().join(format!("viud-blank-repeat-{}", std::process::id()));
    std::fs::create_dir_all(&root).unwrap();
    let path = root.join("keymap.json");
    std::fs::write(
        &path,
        r#"{"version":1,"physical":[{"stroke":{"code":63},"action":"user.f05","repeat":"allow"}]}"#,
    )
    .unwrap();
    h.wire.inner.comp.bindings.reload_user_keymap(&path);
    // As in the real seat, update xkb before asking the binding layer for
    // the repeat token. This fixture does not arm a production Loop timer.
    let keyboard = h.wire.state.seat.seat.get_keyboard().unwrap();
    keyboard.input_intercept(
        &mut h.wire.state,
        smithay::input::keyboard::Keycode::new(EV_F5 + 8),
        smithay::backend::input::KeyState::Pressed,
        |_, _, _| (),
    );
    let edge = world::comp::bindings::user_key(
        &mut h.wire.inner.comp.bindings,
        EV_F5 + 8,
        true,
        false,
        false,
    );
    let token = edge.repeat.expect("repeat-enabled key");
    assert!(h.wire.inner.comp.bindings.user.repeat(token).is_some());
    h.wire.inner.comp.bindings.take_bus_fires();
    h.blank(true);
    assert!(keyboard.pressed_keys().is_empty());
    assert!(
        h.client.state.keys.is_empty(),
        "client saw no press or synthetic release"
    );
    assert!(h.wire.inner.comp.bindings.user.repeat(token).is_none());
    assert!(h.key(EV_F5 + 8, false, false));
    assert!(h.wire.inner.comp.bindings.take_bus_fires().is_empty());
    assert!(h.wire.inner.comp.bindings.user.repeat(token).is_none());
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn blank_entry_never_sends_a_release_for_a_native_table_intercepted_key() {
    let mut h = harness();
    h.wire.inner.comp.bindings.ensure("headless");
    assert!(!h.key(125 + 8, true, false)); // Left Super.
    assert!(h.key(16 + 8, true, false)); // Super+q is compositor close.
    assert_eq!(h.wire.inner.comp.bindings.take_pending().len(), 1);
    h.blank(true);
    assert_eq!(h.client.state.keys, [(125, true), (125, false)]);
    assert!(h.key(16 + 8, false, false));
    assert!(h.key(125 + 8, false, false));
    assert_eq!(h.client.state.keys.len(), 2);
    assert!(h.wire.inner.comp.bindings.take_pending().is_empty());
}

#[test]
fn tablet_pointer_alias_does_not_suppress_the_next_mouse_click() {
    use smithay::backend::input::{TabletToolCapabilities, TabletToolDescriptor, TabletToolType};
    use world::comp::blank::{Edge, Hold};
    let mut h = harness();
    let (surface, _, _) = h.mapped_toplevel(64, 48);
    let button = 0x110;
    h.press(&surface, button);
    // The seat has a mouse-button alias, whose physical owner is the pen tip.
    h.wire.state.tablet.stroke = dispatcher::wire::tablet::Stroke::Pointer;
    h.wire.state.tablet.stroke_owner = Some(TabletToolDescriptor {
        tool_type: TabletToolType::Pen,
        hardware_serial: 1,
        hardware_id_wacom: 2,
        capabilities: TabletToolCapabilities::empty(),
    });
    h.blank(true);
    assert_eq!(h.client.state.buttons.len(), 2, "alias is released once");
    assert!(h.wire.inner.comp.blank.consume_edge(Edge::Release(Hold::TabletTip)));
    h.blank(false);
    h.press(&surface, button);
    h.release(&surface, button);
    assert_eq!(h.client.state.buttons.len(), 4, "next mouse click is delivered");
}

#[test]
fn injected_holds_are_separate_from_physical_holds_and_release_all_clears_them() {
    let mut h = harness();
    let (surface, _, _) = h.mapped_toplevel(64, 48);
    h.focus_keyboard(&surface);
    let button = 0x110;
    assert!(!h.injected_key(EV_A + 8, true));
    h.injected_button(&surface, button, true);
    h.blank(true);
    h.blank(false);
    // A later physical pair must not inherit injection's suppressed release.
    assert!(!h.key(EV_A + 8, true, false));
    assert!(!h.key(EV_A + 8, false, false));
    h.press(&surface, button);
    h.release(&surface, button);
    assert_eq!(h.client.state.keys, [(EV_A, true), (EV_A, false), (EV_A, true), (EV_A, false)]);
    assert_eq!(h.client.state.buttons.len(), 4);
    // The actual injected release still belongs to blank entry.
    assert!(h.injected_key(EV_A + 8, false));
    assert_eq!(h.injected_button(&surface, button, false), 0);
    // ReleaseAll must also clear waking injected presses that never reached xkb.
    h.blank(true);
    assert!(h.injected_key(EV_A + 8, true));
    h.wire.inner.comp.blank.clear_injected_holds();
    assert!(!h.injected_key(EV_A + 8, true));
    assert!(!h.injected_key(EV_A + 8, false));
}

#[test]
fn waking_injected_pair_stays_consumed_through_its_release() {
    let mut h = harness();
    h.blank(true);
    assert!(h.injected_key(EV_A + 8, true));
    assert!(h.injected_key(EV_A + 8, false));
    assert!(h.client.state.keys.is_empty());
    assert!(!h.injected_key(EV_A + 8, true));
    assert!(!h.injected_key(EV_A + 8, false));
    assert_eq!(h.client.state.keys, [(EV_A, true), (EV_A, false)]);
}
