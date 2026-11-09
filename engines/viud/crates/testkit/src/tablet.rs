// SPDX-License-Identifier: MIT OR Apache-2.0
//! Tablet-v2 client observations for the real dispatcher fixtures.
use super::{ClientState, TestClient};
use wayland_client::{Connection, Dispatch, Proxy, QueueHandle, delegate_noop, event_created_child};
use wayland_protocols::wp::tablet::zv2::client::{
    zwp_tablet_manager_v2::ZwpTabletManagerV2,
    zwp_tablet_seat_v2::{self, ZwpTabletSeatV2},
    zwp_tablet_v2::ZwpTabletV2,
    zwp_tablet_tool_v2::{self, ZwpTabletToolV2},
    zwp_tablet_pad_v2::ZwpTabletPadV2,
};

#[derive(Default)]
pub struct State {
    pub tools: Vec<ZwpTabletToolV2>,
    pub tips: Vec<(u32, bool)>,
}

impl TestClient {
    pub fn bind_tablet(&mut self) {
        let (name,_,version)=self.state.globals.iter()
            .find(|(_,i,_)| i=="zwp_tablet_manager_v2").unwrap();
        let manager: ZwpTabletManagerV2=self.registry.bind(*name,(*version).min(1),&self.qh,());
        manager.get_tablet_seat(self.seat(0),&self.qh,());
    }
}

impl Dispatch<ZwpTabletSeatV2,()> for ClientState {
    fn event(state:&mut Self,_:&ZwpTabletSeatV2,event:zwp_tablet_seat_v2::Event,_:&(),_:&Connection,_:&QueueHandle<Self>) {
        if let zwp_tablet_seat_v2::Event::ToolAdded {id}=event { state.tablet.tools.push(id); }
    }
    event_created_child!(ClientState,ZwpTabletSeatV2,[
        0 => (ZwpTabletV2,()),
        1 => (ZwpTabletToolV2,()),
        2 => (ZwpTabletPadV2,())
    ]);
}

impl Dispatch<ZwpTabletToolV2,()> for ClientState {
    fn event(state:&mut Self,tool:&ZwpTabletToolV2,event:zwp_tablet_tool_v2::Event,_:&(),_:&Connection,_:&QueueHandle<Self>) {
        match event {
            zwp_tablet_tool_v2::Event::Down {..} => state.tablet.tips.push((tool.id().protocol_id(),true)),
            zwp_tablet_tool_v2::Event::Up => state.tablet.tips.push((tool.id().protocol_id(),false)),
            _=>{}
        }
    }
}
delegate_noop!(ClientState: ignore ZwpTabletManagerV2);
delegate_noop!(ClientState: ignore ZwpTabletV2);
delegate_noop!(ClientState: ignore ZwpTabletPadV2);
