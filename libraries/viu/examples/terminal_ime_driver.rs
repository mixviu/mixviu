// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native input-method fixture for isolated terminal conformance captures.
//! Connect only to an owned compositor. Args: preedit, commit, hold milliseconds,
//! optionally `late` to deliver the original activation's serial after cancellation.
//! Emits JSON witnesses; never injects an application-side IME event.
use std::{
    io::ErrorKind,
    os::fd::AsRawFd,
    time::{Duration, Instant},
};
use wayland_client::{
    Connection, Dispatch, QueueHandle, delegate_noop,
    protocol::{wl_registry, wl_seat},
};
use wayland_protocols_misc::zwp_input_method_v2::client::{
    zwp_input_method_manager_v2::ZwpInputMethodManagerV2,
    zwp_input_method_v2::{self, ZwpInputMethodV2},
};

struct Driver {
    seat: Option<wl_seat::WlSeat>,
    manager: Option<ZwpInputMethodManagerV2>,
    active: bool,
    serial: u32,
    preedit: String,
    sent: bool,
    due: Option<Instant>,
    hold: Duration,
    late: bool,
    original_serial: Option<u32>,
}
impl Dispatch<wl_registry::WlRegistry, ()> for Driver {
    fn event(
        s: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            match interface.as_str() {
                "wl_seat" if s.seat.is_none() => {
                    s.seat = Some(registry.bind(name, version.min(7), qh, ()))
                }
                "zwp_input_method_manager_v2" => s.manager = Some(registry.bind(name, 1, qh, ())),
                _ => {}
            }
        }
    }
}
delegate_noop!(Driver: ignore wl_seat::WlSeat);
delegate_noop!(Driver: ignore ZwpInputMethodManagerV2);
impl Dispatch<ZwpInputMethodV2, ()> for Driver {
    fn event(
        s: &mut Self,
        method: &ZwpInputMethodV2,
        event: zwp_input_method_v2::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            zwp_input_method_v2::Event::Activate => {
                s.active = true;
                println!("{}", serde_json::json!({"event":"activate"}));
            }
            zwp_input_method_v2::Event::Deactivate => {
                s.active = false;
                if !s.late {
                    s.due = None;
                }
                println!("{}", serde_json::json!({"event":"deactivate"}));
            }
            zwp_input_method_v2::Event::Done => {
                s.serial = s.serial.wrapping_add(1);
                println!(
                    "{}",
                    serde_json::json!({"event":"done","serial":s.serial,"active":s.active})
                );
                if s.active && !s.sent {
                    method.set_preedit_string(
                        s.preedit.clone(),
                        s.preedit.len() as i32,
                        s.preedit.len() as i32,
                    );
                    method.commit(s.serial);
                    s.sent = true;
                    s.original_serial = Some(s.serial);
                    s.due = Some(Instant::now() + s.hold);
                    println!(
                        "{}",
                        serde_json::json!({"event":"preedit","serial":s.serial,"text":s.preedit})
                    );
                }
            }
            zwp_input_method_v2::Event::Unavailable => {
                panic!("owned compositor refused input method")
            }
            _ => {}
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if !(args.len() == 3 || (args.len() == 4 && args[3] == "late"))
        || args[0].len() > 4000
        || args[1].len() > 4000
    {
        return Err(
            "expected preedit, commit, hold milliseconds and optional late; text <=4000 bytes"
                .into(),
        );
    }
    let hold: u64 = args[2].parse()?;
    if hold > 20_000 {
        return Err("hold exceeds fixture budget".into());
    }
    let connection = Connection::connect_to_env()?;
    let mut queue = connection.new_event_queue();
    let qh = queue.handle();
    let mut state = Driver {
        seat: None,
        manager: None,
        active: false,
        serial: 0,
        preedit: args[0].clone(),
        sent: false,
        due: None,
        hold: Duration::from_millis(hold),
        late: args.len() == 4,
        original_serial: None,
    };
    let _registry = connection.display().get_registry(&qh, ());
    queue.roundtrip(&mut state)?;
    let method = state
        .manager
        .as_ref()
        .ok_or("no input-method global")?
        .get_input_method(state.seat.as_ref().ok_or("no seat")?, &qh, ());
    connection.flush()?;
    println!("{}", serde_json::json!({"event":"ready"}));
    let until = Instant::now() + Duration::from_secs(30);
    while Instant::now() < until {
        queue.dispatch_pending(&mut state)?;
        if (state.active || state.late) && state.due.is_some_and(|due| Instant::now() >= due) {
            method.set_preedit_string(String::new(), 0, 0);
            method.commit_string(args[1].clone());
            let serial = if state.late {
                state
                    .original_serial
                    .ok_or("late commit without original activation")?
            } else {
                state.serial
            };
            let active = state.active;
            let current_serial = state.serial;
            method.commit(serial);
            queue.roundtrip(&mut state)?;
            println!(
                "{}",
                serde_json::json!({"event":"commit","serial":serial,"current_serial":current_serial,"active":active,"late":state.late,"text":args[1]})
            );
            return Ok(());
        }
        connection.flush()?;
        if let Some(guard) = connection.prepare_read() {
            let mut poll = libc::pollfd {
                fd: guard.connection_fd().as_raw_fd(),
                events: libc::POLLIN,
                revents: 0,
            };
            // This owns the Wayland read guard and bounds every wait to 20ms.
            let ready = unsafe { libc::poll(&mut poll, 1, 20) };
            if ready < 0 {
                let error = std::io::Error::last_os_error();
                if error.kind() != ErrorKind::Interrupted {
                    return Err(error.into());
                }
            } else if ready > 0 {
                match guard.read() {
                    Ok(_) => {}
                    Err(wayland_client::backend::WaylandError::Io(e))
                        if e.kind() == ErrorKind::WouldBlock => {}
                    Err(error) => return Err(error.into()),
                }
            }
        }
    }
    Err("native IME fixture timed out without a committed activation".into())
}
