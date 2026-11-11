// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
use std::time::Duration;
use testkit::Harness;

#[test]
fn disconnect_retires_each_resource_without_an_explicit_destroy() {
    let mut h = Harness::new();
    let i = h.add_client();
    let surface = h.extra_clients[i].create_surface();
    let _a = h.extra_clients[i].inhibit(&surface);
    let _b = h.extra_clients[i].inhibit(&surface);
    h.roundtrip_client(i);
    assert_eq!(h.wire.state.inhibitors.surfaces.len(), 2);
    drop(h.extra_clients.remove(i));
    h.roundtrip();
    assert!(h.wire.state.inhibitors.surfaces.is_empty());
}

#[test]
fn real_resources_are_counted_individually_and_surface_death_cannot_keep_them_live() {
    use smithay::reexports::wayland_server::Resource;
    let mut h = Harness::new();
    let (surface, xdg, top) = h.mapped_toplevel(64, 48);
    let a = h.client.inhibit(&surface);
    let b = h.client.inhibit(&surface);
    h.roundtrip();
    assert_eq!(h.wire.state.inhibitors.surfaces.len(), 2);
    a.destroy();
    h.roundtrip();
    assert_eq!(h.wire.state.inhibitors.surfaces.len(), 1);
    top.destroy();
    xdg.destroy();
    surface.destroy();
    h.roundtrip();
    assert!(!h.wire.state.inhibitors.surfaces[0].is_alive());
    b.destroy();
    h.roundtrip();
    assert!(h.wire.state.inhibitors.surfaces.is_empty());
}

#[test]
fn idle_notification_is_held_then_rearmed_on_the_real_calloop() {
    let mut h = Harness::new();
    dispatcher::idle::set_inhibited(&mut h.wire.state, true);
    let notification = h.client.idle_notification(25);
    h.roundtrip();
    std::thread::sleep(Duration::from_millis(60));
    h.roundtrip();
    assert!(h.client.state.idle_events.is_empty());
    dispatcher::idle::set_inhibited(&mut h.wire.state, false);
    h.roundtrip();
    assert!(h.client.state.idle_events.is_empty());
    std::thread::sleep(Duration::from_millis(60));
    h.roundtrip();
    assert_eq!(h.client.state.idle_events, [true]);
    notification.destroy();
}
