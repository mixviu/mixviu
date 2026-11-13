// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Explicit inherited-descriptor fake seat. Only enabled by the registered
//! test variable; it never opens a seat or performs a KMS ioctl. Renderer
//! transfer, GPU_BINDING borrowing, frame timers and session edges are real.
use smithay::{backend::{allocator::gbm::GbmDevice, drm::{DrmDeviceFd, DrmNode}}, output::Output, reexports::calloop::{generic::Generic, Interest, Mode, PostAction, timer::{Timer, TimeoutAction}}, utils::DeviceFd};
use std::{cell::RefCell, fs::OpenOptions, os::{fd::{FromRawFd, OwnedFd}, unix::net::UnixDatagram}, rc::Rc};
use world::state::{Loop, StateDRMBinding};

struct Seat {
    output: Output,
    render_node: String,
    reply: UnixDatagram,
    binding: Option<Rc<RefCell<StateDRMBinding>>>,
    failures: usize,
    retry: super::activation::Backoff,
}

fn activate(seat: &Rc<RefCell<Seat>>, state: &mut Loop, ack: u8) {
    let mut fixture = seat.borrow_mut();
    if fixture.failures > 0 {
        fixture.failures -= 1;
        let delay = fixture.retry.next();
        let seat = seat.clone();
        state.loop_handle.insert_source(Timer::from_duration(delay), move |_, _, state| {
            activate(&seat, state, ack);
            TimeoutAction::Drop
        }).expect("fake seat retry");
        return;
    }
    if fixture.binding.is_none() {
        let renderer = headless::wire::take_renderer().expect("fixture takes the original renderer once");
        let node = DrmNode::from_path(&fixture.render_node).expect("fixture render node");
        let file = OpenOptions::new().read(true).write(true).open(&fixture.render_node).expect("fixture render fd");
        let fd = DrmDeviceFd::new(DeviceFd::from(OwnedFd::from(file)));
        let gbm = GbmDevice::new(fd).expect("fixture GBM");
        let mut gpus = render_gles::multigpu::factory::create_with(Some(renderer));
        render_gles::multigpu::factory::add_node(&mut gpus, node, gbm);
        drop(render_gles::multigpu::factory::single_renderer(&mut gpus, &node));
        let binding = Rc::new(RefCell::new(StateDRMBinding { gpus, primary: node }));
        *state.inner.kernel.get_mut(&world::state::GPU_BINDING_MUT) = Some(binding.clone());
        fixture.binding = Some(binding);
        info!("fake-seat: renderer handed over once");
    }
    native::wire::session::transition(state, true);
    headless::wire::follow_output(fixture.output.clone());
    headless::wire::fixture_presentation(state, true);
    let _ = fixture.reply.send(&[ack]);
}

pub fn install(state: &mut Loop, output: Output, render_node: &str, descriptor: &str) -> Result<(), Box<dyn std::error::Error>> {
    let fd: i32 = descriptor.parse()?;
    if fd < 3 { return Err("invalid fake-seat descriptor".into()); }
    // SAFETY: explicitly transferred by the owned test child via fd:N.
    let socket = unsafe { UnixDatagram::from_raw_fd(fd) };
    socket.set_nonblocking(true)?;
    let seat = Rc::new(RefCell::new(Seat { output, render_node: render_node.into(), reply: socket.try_clone()?, binding: None, failures: 2, retry: Default::default() }));
    state.loop_handle.insert_source(Generic::new(socket, Interest::READ, Mode::Level), move |_, socket, state| {
        let mut command = [0];
        if socket.recv(&mut command).is_err() { return Ok(PostAction::Continue); }
        match command[0] {
            b'A' | b'Q' => {
                // Q arms an offscreen timer immediately before activation.
                if command[0] == b'Q' { state.state.schedule_redraw(protocols::redraw::schedule::RedrawReason::Resume); headless::wire::arm_inactive(state); }
                activate(&seat, state, command[0]);
                return Ok(PostAction::Continue);
            }
            b'P' => {
                native::wire::session::transition(state, false);
                headless::wire::follow_output(seat.borrow().output.clone());
                headless::wire::fixture_presentation(state, false);
            }
            b'D' => { *state.inner.kernel.get_mut(&world::state::GPU_BINDING_MUT) = None; }
            b'B' => {
                *state.inner.kernel.get_mut(&world::state::GPU_BINDING_MUT) = seat.borrow().binding.clone();
                state.state.schedule_redraw(protocols::redraw::schedule::RedrawReason::Rescue);
            }
            b'F' => {
                super::activation::fixture_watcher_failed(true);
                state.loop_handle.insert_source(Timer::from_duration(std::time::Duration::from_secs(1)), |_, _, _| {
                    super::activation::fixture_watcher_failed(false);
                    TimeoutAction::Drop
                }).expect("fake watcher recovery");
            }
            _ => return Ok(PostAction::Continue),
        }
        let _ = socket.send(&command);
        Ok(PostAction::Continue)
    })?;
    Ok(())
}
