// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Live staging probe: `activation_probe background|stale|foreign|unknown|handoff`.
//! Mapping gives this fixture focus. The operator uses comp.window.focus to
//! return focus to the witness during the announced five-second preparation.
use std::{
    os::unix::net::UnixStream,
    time::{Duration, Instant},
};
use testkit::client::TestClient;

fn pump(client: &mut TestClient, duration: Duration) {
    let until = Instant::now() + duration;
    while Instant::now() < until {
        client.flush();
        client.read();
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn sync(client: &mut TestClient) {
    let goal = client.sync();
    let deadline = Instant::now() + Duration::from_secs(10);
    while client.state.syncs < goal {
        assert!(Instant::now() < deadline, "compositor sync deadline");
        pump(client, Duration::from_millis(5));
    }
}

fn main() {
    let mode = std::env::args()
        .nth(1)
        .expect("background|stale|foreign|unknown|handoff");
    assert!(["background", "stale", "foreign", "unknown", "handoff"].contains(&mode.as_str()));
    let socket = std::path::PathBuf::from(std::env::var_os("XDG_RUNTIME_DIR").unwrap())
        .join(std::env::var_os("WAYLAND_DISPLAY").unwrap());
    let mut client = TestClient::new(UnixStream::connect(&socket).unwrap());
    sync(&mut client);
    client.bind_globals();
    client.bind_keyboards();
    sync(&mut client);
    let surface = client.create_surface();
    let (_xdg, top) = client.toplevel(&surface);
    top.set_app_id("testkit.activation-probe".into());
    top.set_title(format!("Activation probe {mode}"));
    client.commit(&surface);
    sync(&mut client);
    client.attach(&surface, 240, 100);
    sync(&mut client);
    pump(&mut client, Duration::from_secs(1));
    let serial = client
        .state
        .keyboard_enter_serial
        .expect("fixture must receive initial keyboard focus");
    if mode == "stale" {
        client.activation_token(Some(serial), Some(&surface));
        sync(&mut client);
    }
    // A real cross-client bearer grant, deliberately distinct from an unknown string.
    // Keep A alive while background B presents A's token after witness focus returns.
    let mut issuer = None;
    let mut foreign_token = None;
    if mode == "foreign" || mode == "handoff" {
        let mut a = TestClient::new(UnixStream::connect(&socket).unwrap());
        sync(&mut a);
        a.bind_globals();
        a.bind_keyboards();
        sync(&mut a);
        let surface_a = a.create_surface();
        let (_xdg_a, top_a) = a.toplevel(&surface_a);
        top_a.set_app_id("testkit.activation-issuer".into());
        top_a.set_title("Activation token issuer A".into());
        a.commit(&surface_a);
        sync(&mut a);
        a.attach(&surface_a, 240, 100);
        sync(&mut a);
        pump(&mut a, Duration::from_secs(1));
        let serial_a = a.state.keyboard_enter_serial.expect("A must have focus");
        a.activation_token(Some(serial_a), Some(&surface_a));
        sync(&mut a);
        foreign_token = a.state.activation_tokens.last().cloned();
        println!("ISSUED mode={mode}; A minted while keyboard-focused");
        issuer = Some((a, surface_a, _xdg_a, top_a));
    }
    if mode != "handoff" {
        println!("READY mode={mode}; return focus to witness now");
        pump(
        &mut client,
        Duration::from_secs(if mode == "stale" { 31 } else { 5 }),
        );
    }
    let token = if mode == "unknown" {
        "foreign-compositor-token".to_owned()
    } else if mode == "foreign" || mode == "handoff" {
        foreign_token.expect("A must mint a real token")
    } else {
        if mode == "background" {
            client.activation_token(Some(serial), Some(&surface));
            sync(&mut client);
        }
        client.state.activation_tokens.last().unwrap().clone()
    };
    client.activation().activate(token, &surface);
    sync(&mut client);
    println!("ACTIVATED mode={mode}");
    pump(&mut client, Duration::from_secs(3));
    drop(issuer);
}
