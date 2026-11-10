// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Real registry filtering, without a GPU, settingsd or a session desktop.
use std::{os::unix::net::{UnixListener, UnixStream}, process::{Command, Stdio}, time::{Duration, Instant}};
use comp_model::agent_seat::{self, Policy};
use testkit::Harness;
use wayland_client::{Connection, Dispatch, QueueHandle, protocol::{wl_registry, wl_seat}};

#[derive(Default)]
struct Registry { seats: Vec<(wl_seat::WlSeat, String)> }
impl Dispatch<wl_registry::WlRegistry, ()> for Registry {
    fn event(state: &mut Self, registry: &wl_registry::WlRegistry, event: wl_registry::Event, _: &(), _: &Connection, qh: &QueueHandle<Self>) {
        if let wl_registry::Event::Global { name, interface, version } = event && interface == "wl_seat" {
            state.seats.push((registry.bind(name, version.min(7), qh, ()), String::new()));
        }
    }
}
impl Dispatch<wl_seat::WlSeat, ()> for Registry {
    fn event(state: &mut Self, seat: &wl_seat::WlSeat, event: wl_seat::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {
        if let wl_seat::Event::Name { name } = event {
            state.seats.iter_mut().find(|row| &row.0 == seat).unwrap().1 = name;
        }
    }
}

#[test]
fn registry_child() {
    // The socket is supplied as a positional argument after libtest's --skip,
    // avoiding another environment policy or runtime configuration mechanism.
    let args: Vec<_> = std::env::args().collect();
    let Some(path) = args.iter().find_map(|arg| arg.strip_prefix("agent-seat-socket=")) else { return; };
    let conn = Connection::from_socket(UnixStream::connect(path).unwrap()).unwrap();
    let mut queue = conn.new_event_queue::<Registry>();
    let _registry = conn.display().get_registry(&queue.handle(), ());
    let mut state = Registry::default();
    queue.roundtrip(&mut state).unwrap();
    queue.roundtrip(&mut state).unwrap();
    let names: Vec<_> = state.seats.iter().map(|row| row.1.as_str()).collect();
    println!("SEATS={}", names.join(","));
}

#[test]
fn external_clients_only_see_the_opted_in_seat() {
    let mut harness = Harness::new();
    assert_eq!(harness.client.seat_count(), 2, "in-process exception");
    let path = std::env::temp_dir().join(format!("agent-seat-{}.sock", std::process::id()));
    let listener = UnixListener::bind(&path).unwrap();
    listener.set_nonblocking(true).unwrap();
    let exe = std::env::current_exe().unwrap();
    for allowed in [false, true] {
        agent_seat::install(Policy { allow: if allowed { vec![exe.file_name().unwrap().to_str().unwrap().into()] } else { vec![] }, ..Default::default() });
        let mut child = Command::new(&exe).args(["--exact", "registry_child", "--nocapture", "--skip"])
            .arg(format!("agent-seat-socket={}", path.display())).stdout(Stdio::piped()).stderr(Stdio::inherit()).spawn().unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            match listener.accept() {
                Ok((stream, _)) => { harness.attach_external(stream); break; }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline, "child never connected");
                    std::thread::sleep(Duration::from_millis(1));
                }
                Err(error) => panic!("accept: {error}"),
            }
        }
        loop {
            harness.pump();
            if child.try_wait().unwrap().is_some() { break; }
            assert!(Instant::now() < deadline, "child hung");
            std::thread::sleep(Duration::from_millis(1));
        }
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        let expected = if allowed { "SEATS=seat0,agent" } else { "SEATS=seat0" };
        assert!(text.lines().any(|line| line == expected), "{text}");
    }
    agent_seat::install(Policy::default());
    std::fs::remove_file(path).unwrap();
}
