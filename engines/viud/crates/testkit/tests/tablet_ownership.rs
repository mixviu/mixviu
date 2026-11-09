// SPDX-License-Identifier: MIT OR Apache-2.0
use dispatcher::wire::tablet::Stroke;
use smithay::backend::input::{TabletToolDescriptor, TabletToolCapabilities, TabletToolType};
use smithay::input::tablet::TabletDescriptor;
use smithay::utils::SERIAL_COUNTER;

fn tool(n: u64) -> TabletToolDescriptor {
    TabletToolDescriptor { tool_type: TabletToolType::Pen, hardware_serial:n,
        hardware_id_wacom:n, capabilities:TabletToolCapabilities::empty() }
}

#[test]
fn unrelated_proximity_and_tip_down_preserve_the_live_owner_and_its_lift() {
    let mut h = testkit::Harness::new();
    let dh = h.wire.state.output.display_handle.clone();
    let (a,b) = (tool(1),tool(2));
    let state = &mut h.wire.state.tablet;
    for t in [&a,&b] { state.add_tool::<dispatcher::state::Dispatch>(&dh,t); }
    assert!(!state.begin_stroke(&b,Stroke::None));
    assert!(state.stroke_owner.is_none());
    assert!(state.begin_stroke(&a,Stroke::Tablet));
    state.tool_tip_down(&a,SERIAL_COUNTER.next_serial(),1);
    assert!(state.tool_is_down(&a));
    assert!(!state.begin_stroke(&b,Stroke::Tablet));
    state.tool_tip_down(&b,SERIAL_COUNTER.next_serial(),2);
    assert!(!state.tool_is_down(&b));
    state.tool_proximity_out(&b,3);
    assert_eq!(state.stroke_owner,Some(a.clone()));
    assert!(state.tool_is_down(&a));
    state.tool_tip_up(&a,4);
    assert!(!state.tool_is_down(&a),"A's lift must still reach its tool");
    state.remove_tool(&a,5);
    assert_eq!(state.stroke,Stroke::None);
    assert!(state.stroke_owner.is_none());
}

#[test]
fn removal_revokes_the_owner_and_queues_real_emulation_releases_once() {
    let mut h = testkit::Harness::new();
    let dh = h.wire.state.output.display_handle.clone();
    let a=tool(1);
    let state=&mut h.wire.state.tablet;
    state.add_tool::<dispatcher::state::Dispatch>(&dh,&a);
    assert!(state.begin_stroke(&a,Stroke::Pointer));
    state.push_emulated_button(&a,0x14b,0x111);
    state.remove_tool(&a,1);
    assert!(state.stroke_owner.is_none());
    assert_eq!(state.stroke,Stroke::None);
    assert_eq!(state.take_revoked_buttons(),vec![0x110,0x111]);
    assert!(state.take_revoked_buttons().is_empty());
    state.add_tool::<dispatcher::state::Dispatch>(&dh,&a);
    assert!(state.begin_stroke(&a,Stroke::Pointer),"the next stroke can acquire the seat");
    state.clear_tools();
    assert!(state.stroke_owner.is_none());
}

#[test]
fn tablet_device_removal_clears_its_live_stroke() {
    let mut h=testkit::Harness::new();
    let (surface,_,_)=h.mapped_toplevel(64,48);
    let server=h.handle_of(&surface);
    let surface=protocols::window::ident::surface(h.wire.inner.space.state.elements().next().unwrap()).unwrap();
    let dh=h.wire.state.output.display_handle.clone();
    let a=tool(1);
    let tablet=TabletDescriptor { name:"removed".into(),usb_id:None,syspath:None };
    let state=&mut h.wire.state.tablet;
    state.add_tool::<dispatcher::state::Dispatch>(&dh,&a);
    state.tool_proximity_in(&a,&tablet,(0.0,0.0).into(),(surface,(0.0,0.0).into()),SERIAL_COUNTER.next_serial(),0);
    assert!(state.begin_stroke(&a,Stroke::Tablet));
    state.tool_tip_down(&a,SERIAL_COUNTER.next_serial(),1);
    state.remove_tablet(&tablet);
    assert_eq!(state.stroke,Stroke::None);
    assert!(state.stroke_owner.is_none());
    assert!(!state.tool_is_down(&a));
    let _=server;
}
