// SPDX-License-Identifier: MIT OR Apache-2.0
use testkit::Harness;

#[test]
fn agent_focus_excludes_xwayland_selection_bridge() {
    let mut h = Harness::new();
    h.client.bind_primary_devices();
    let (target, ..) = h.mapped_toplevel(64, 48);
    h.agent_press(&target, 0x110);
    let _source = h.client.set_primary_selection(1, b"agent-private");
    h.roundtrip();
    let bridge = h.add_client();
    h.extra_clients[bridge].bind_primary_devices();
    let surface = h.extra_clients[bridge].create_surface();
    h.extra_clients[bridge].commit(&surface);
    h.roundtrip_client(bridge);
    h.mark_client_xwayland(bridge);
    h.focus_agent_keyboard_client(bridge, &surface);
    h.roundtrip_client(bridge);
    assert!(
        h.extra_clients[bridge]
            .state
            .clipboard
            .primary_selections
            .get(&1)
            .is_none_or(|offer| offer.is_none()),
        "X11 must not receive agent PRIMARY"
    );
    // The same identity can still receive the human seat's offer.
    h.focus_keyboard(&target);
    let _human = h.client.set_primary_selection(0, b"human-primary");
    h.roundtrip();
    h.focus_keyboard_client(bridge, &surface);
    h.roundtrip_client(bridge);
    assert!(h.extra_clients[bridge].state.clipboard.primary_selections[&0].is_some());
}
