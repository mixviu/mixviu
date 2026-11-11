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
