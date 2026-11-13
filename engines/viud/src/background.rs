// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Start the render-node desk before acquiring a VT-bound seat.
use std::{cell::RefCell, ffi::OsString, rc::Rc};
use smithay::reexports::calloop::{EventLoop, ping::make_ping};
use world::state::{Loop, StatusSession};

pub type Handles = Rc<RefCell<Option<native::wire::entry::NativeHandles>>>;

pub fn wire(
    state: &mut Loop,
    event_loop: &mut EventLoop<'static, Loop>,
    socket: OsString,
    size: (i32, i32),
    scale: f64,
    render_node: &str,
) -> Result<Handles, Box<dyn std::error::Error>> {
    state.inner.status_session = StatusSession::Paused;
    let output = headless::wire::install(
        state,
        &event_loop.handle(),
        headless::wire::Options { size: size.into(), scale, render_node },
        None,
    )?;
    headless::wire::vt_session();
    let handles = Rc::new(RefCell::new(None));
    let target = handles.clone();
    let handle = event_loop.handle();
    let (ping, source) = make_ping()?;
    let token = event_loop.handle().insert_source(source, move |_, _, state| {
        if target.borrow().is_some() {
            return;
        }
        let native = native::wire::entry::wire(state, socket.clone(), &handle, Some(output.clone()));
        // Headless focus stays, but the human pointer/touch sources belong to KMS.
        state.state.seat.seat.add_pointer();
        state.inner.status_session = StatusSession::Active;
        *target.borrow_mut() = Some(native);
        state.state.schedule_redraw(protocols::redraw::schedule::RedrawReason::Resume);
    })?;
    // Only this blocking, event-driven waiter runs off-thread. No seat/device
    // or Wayland state crosses threads. It exits after the first activation;
    // libseat owns subsequent pause/resume notifications.
    let _ = token;
    std::thread::Builder::new().name("viud-vt-wait".into()).spawn(move || {
        kms::seat::session::vt_wait::wait_for_own_vt();
        ping.ping();
    })?;
    Ok(handles)
}
