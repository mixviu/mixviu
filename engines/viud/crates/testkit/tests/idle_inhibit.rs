// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
use std::time::Duration;
use testkit::Harness;

#[test]
fn inhibitor_child() {
    let Some(path) = std::env::args().find_map(|a| a.strip_prefix("inhibitor-socket=").map(str::to_owned)) else {
        return;
    };
    let mut client = testkit::client::TestClient::new(std::os::unix::net::UnixStream::connect(path).unwrap());
    let sync = client.sync();
    while client.state.syncs < sync {
        client.flush();
        client.read();
        std::thread::sleep(Duration::from_millis(1));
    }
    client.bind_globals();
    let surface = client.create_surface();
    let _inhibitor = client.inhibit(&surface);
    client.flush();
    loop {
        std::thread::park();
    }
}

#[test]
fn killed_client_releases_its_live_inhibitor() {
    use std::os::unix::net::UnixListener;
    use std::process::{Command, Stdio};
    use std::time::Instant;
    let mut h = Harness::new();
    let path = std::env::temp_dir().join(format!("viud-inhibitor-{}.sock", std::process::id()));
    let listener = UnixListener::bind(&path).unwrap();
    listener.set_nonblocking(true).unwrap();
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "inhibitor_child", "--skip"])
        .arg(format!("inhibitor-socket={}", path.display()))
        .stdout(Stdio::null())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match listener.accept() {
            Ok((stream, _)) => {
                h.attach_external(stream);
                break;
            }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                if Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    panic!("inhibitor child never connected");
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            Err(e) => panic!("accept: {e}"),
        }
    }
    while h.wire.state.inhibitors.surfaces.is_empty() && Instant::now() < deadline {
        h.pump();
        std::thread::sleep(Duration::from_millis(1));
    }
    let live = h.wire.state.inhibitors.surfaces.len();
    child.kill().unwrap();
    child.wait().unwrap();
    assert_eq!(live, 1, "child must own a live resource before SIGKILL");
    h.roundtrip();
    assert!(h.wire.state.inhibitors.surfaces.is_empty());
    std::fs::remove_file(path).unwrap();
}

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
