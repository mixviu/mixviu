// SPDX-License-Identifier: MIT OR Apache-2.0
use dispatcher::wire::tablet::Stroke;
use smithay::backend::input::{TabletToolCapabilities, TabletToolDescriptor, TabletToolType};
use smithay::input::tablet::TabletDescriptor;
use smithay::utils::SERIAL_COUNTER;

fn tool(n: u64) -> TabletToolDescriptor {
    TabletToolDescriptor {
        tool_type: TabletToolType::Pen,
        hardware_serial: n,
        hardware_id_wacom: n,
        capabilities: TabletToolCapabilities::empty(),
    }
}

#[test]
fn unrelated_proximity_and_tip_down_preserve_the_live_owner_and_its_lift() {
    let mut h = testkit::Harness::new();
    h.client.bind_tablet();
    h.roundtrip();
    h.mapped_toplevel(64, 48);
    h.mapped_toplevel(64, 48);
    let surfaces: Vec<_> = h
        .wire
        .inner
        .space
        .state
        .elements()
        .filter_map(protocols::window::ident::surface)
        .collect();
    let dh = h.wire.state.output.display_handle.clone();
    let (a, b) = (tool(1), tool(2));
    let state = &mut h.wire.state.tablet;
    for t in [&a, &b] {
        state.add_tool::<dispatcher::state::Dispatch>(&dh, t);
    }
    let tablet = TabletDescriptor {
        name: "two tools".into(),
        usb_id: None,
        syspath: None,
    };
    state.add_tablet::<dispatcher::state::Dispatch>(&dh, &tablet);
    for (t, s) in [(&a, &surfaces[0]), (&b, &surfaces[1])] {
        state.tool_proximity_in(
            t,
            &tablet,
            (0.0, 0.0).into(),
            (s.clone(), (0.0, 0.0).into()),
            SERIAL_COUNTER.next_serial(),
            0,
        );
    }
    assert!(!state.begin_stroke(&b, Stroke::None));
    assert!(state.stroke_owner.is_none());
    assert!(state.begin_stroke(&a, Stroke::Tablet));
    state.tool_tip_down(&a, SERIAL_COUNTER.next_serial(), 1);
    assert!(state.tool_is_down(&a));
    h.roundtrip();
    assert_eq!(h.client.state.tablet.tools.len(),2);
    let cursor=h.client.create_surface();
    h.client.state.tablet.tools[1].set_cursor(1,Some(&cursor),0,0);
    h.roundtrip();
    assert_eq!(h.wire.state.tablet.cursor_owner,Some(b.clone()));
    let state=&mut h.wire.state.tablet;
    assert!(!state.begin_stroke(&b, Stroke::Tablet));
    state.tool_tip_down(&b, SERIAL_COUNTER.next_serial(), 2);
    assert!(!state.tool_is_down(&b));
    state.tool_proximity_out(&b, 3);
    assert!(!state.tool_cursor);
    assert!(state.cursor_owner.is_none());
    assert_eq!(state.stroke_owner, Some(a.clone()));
    assert!(state.tool_is_down(&a));
    state.tool_proximity_in(&b,&tablet,(0.0,0.0).into(),
        (surfaces[1].clone(),(0.0,0.0).into()),SERIAL_COUNTER.next_serial(),3);
    h.roundtrip();
    h.client.state.tablet.tools[1].set_cursor(1,Some(&cursor),0,0);
    h.roundtrip();
    let state=&mut h.wire.state.tablet;
    assert!(state.tool_cursor);
    state.clear_surface_focus(|s|s==&surfaces[1],SERIAL_COUNTER.next_serial(),4);
    assert!(!state.tool_cursor,"hiding B's surface revokes B's hovering cursor");
    assert!(state.tool_is_down(&a));
    assert_eq!(state.stroke_owner,Some(a.clone()));
    state.tool_tip_up(&a, 4);
    assert!(
        !state.tool_is_down(&a),
        "A's lift must still reach its tool"
    );
    state.remove_tool(&a, 5);
    assert_eq!(state.stroke, Stroke::None);
    assert!(state.stroke_owner.is_none());
    h.roundtrip();
    let tips=&h.client.state.tablet.tips;
    assert_eq!(tips.len(),2,"only A's down and lift reach the real tablet client");
    assert_eq!(tips[0].0,tips[1].0);
    assert!(tips[0].1);
    assert!(!tips[1].1);
}

#[test]
fn revoked_pointer_stroke_releases_the_real_primary_seat_before_the_next_press() {
    let mut h = testkit::Harness::new();
    let (surface, _, _) = h.mapped_toplevel(64, 48);
    let dh = h.wire.state.output.display_handle.clone();
    let a = tool(1);
    h.wire
        .state
        .tablet
        .add_tool::<dispatcher::state::Dispatch>(&dh, &a);
    assert!(h.wire.state.tablet.begin_stroke(&a, Stroke::Pointer));
    h.press(&surface, 0x110);
    let pointer = h.wire.state.seat.seat.get_pointer().unwrap();
    assert!(pointer.is_grabbed());
    let server =
        protocols::window::ident::surface(h.wire.inner.space.state.elements().next().unwrap())
            .unwrap();
    h.wire.state.tablet.stroke_surface = Some(server.clone());
    h.wire
        .state
        .tablet
        .clear_surface_focus(|s| s == &server, SERIAL_COUNTER.next_serial(), 1);
    // The production tablet revoke flush feeds these same releases through
    // touch::emulate; the renderer-free harness drives the actual seat directly.
    for button in h.wire.state.tablet.take_revoked_buttons() {
        h.release(&surface, button);
    }
    assert!(!pointer.is_grabbed());
    let before = h.client.state.buttons.len();
    h.wire
        .state
        .tablet
        .add_tool::<dispatcher::state::Dispatch>(&dh, &a);
    assert!(h.wire.state.tablet.begin_stroke(&a, Stroke::Pointer));
    h.press(&surface, 0x110);
    assert_eq!(h.client.state.buttons.len(), before + 1);
    assert!(h.client.state.buttons.last().unwrap().2);
    h.release(&surface, 0x110);
    assert!(!pointer.is_grabbed());
}

#[test]
fn removal_revokes_the_owner_and_queues_real_emulation_releases_once() {
    let mut h = testkit::Harness::new();
    let dh = h.wire.state.output.display_handle.clone();
    let a = tool(1);
    let state = &mut h.wire.state.tablet;
    state.add_tool::<dispatcher::state::Dispatch>(&dh, &a);
    assert!(state.begin_stroke(&a, Stroke::Pointer));
    state.push_emulated_button(&a, 0x14b, 0x111);
    state.remove_tool(&a, 1);
    assert!(state.stroke_owner.is_none());
    assert_eq!(state.stroke, Stroke::None);
    assert_eq!(state.take_revoked_buttons(), vec![0x110, 0x111]);
    assert!(state.take_revoked_buttons().is_empty());
    state.add_tool::<dispatcher::state::Dispatch>(&dh, &a);
    assert!(
        state.begin_stroke(&a, Stroke::Pointer),
        "the next stroke can acquire the seat"
    );
    state.clear_tools();
    assert!(state.stroke_owner.is_none());
}

#[test]
fn tablet_device_removal_clears_its_live_stroke() {
    let mut h = testkit::Harness::new();
    h.client.bind_tablet();
    h.roundtrip();
    let (surface, _, _) = h.mapped_toplevel(64, 48);
    let server = h.handle_of(&surface);
    let surface =
        protocols::window::ident::surface(h.wire.inner.space.state.elements().next().unwrap())
            .unwrap();
    let dh = h.wire.state.output.display_handle.clone();
    let a = tool(1);
    let tablet = TabletDescriptor {
        name: "removed".into(),
        usb_id: None,
        syspath: None,
    };
    let state = &mut h.wire.state.tablet;
    state.add_tool::<dispatcher::state::Dispatch>(&dh, &a);
    state.add_tablet::<dispatcher::state::Dispatch>(&dh,&tablet);
    state.tool_proximity_in(
        &a,
        &tablet,
        (0.0, 0.0).into(),
        (surface, (0.0, 0.0).into()),
        SERIAL_COUNTER.next_serial(),
        0,
    );
    assert!(state.begin_stroke(&a, Stroke::Tablet));
    state.tool_tip_down(&a, SERIAL_COUNTER.next_serial(), 1);
    state.remove_tablet(&tablet);
    assert_eq!(state.stroke, Stroke::None);
    assert!(state.stroke_owner.is_none());
    assert!(!state.tool_is_down(&a));
    h.roundtrip();
    assert_eq!(h.client.state.tablet.tips.len(),2);
    assert!(h.client.state.tablet.tips[0].1);
    assert!(!h.client.state.tablet.tips[1].1);
    let _ = server;
}
