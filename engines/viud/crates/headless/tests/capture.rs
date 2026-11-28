// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Real render-node compositor, Wayland clients and an owned private Bus.
//! Build viud (backend-headless,test-support), noded and testkit binaries first.
#[path = "support/osd.rs"]
mod osd;
#[path = "support/owned_bus.rs"]
mod owned_bus;
#[path = "support/pattern.rs"]
mod pattern;
#[path = "support/process.rs"]
mod process;
#[path = "support/window_motion.rs"]
mod window_motion;
#[path = "support/window_motion_retention.rs"]
mod window_motion_retention;
#[path = "support/window_motion_review.rs"]
mod window_motion_review;

use pattern::{COLOURS, Pattern};
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    process::{Command, Output},
    thread,
    time::{Duration, Instant},
};
use testkit::TestClient;
use wayland_client::protocol::wl_shm;

#[test]
fn initial_fullscreen_before_first_buffer_fills_the_output() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("initial-fullscreen");
    let mut process = process::spawn_size("initial-fullscreen", &bus.url, "1600x900");
    // Pattern requests fullscreen before its first commit. Its initial client
    // choice is 1280x720, deliberately different from the output.
    let mut pattern = Pattern::new(connect(&mut process));
    pattern.wait_callback(0);
    bus.ready();
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut resized = false;
    loop {
        pattern.pump();
        // Unlike a normal app, Pattern does not automatically resize its shm
        // buffer when a configure arrives. Complete that client-side half of
        // the transaction before asserting the committed window geometry.
        if !resized && pattern.state.size == (1600, 900) {
            pattern.change_buffer();
            let before = pattern.state.callbacks;
            pattern.commit(true);
            pattern.wait_callback(before);
            resized = true;
        }
        let row = bus.window(&mut pattern);
        if resized && row["fullscreen"] == true && row["width"] == 1600 && row["height"] == 900 {
            assert_eq!(row["window_x"], 0);
            assert_eq!(row["window_y"], 0);
            break;
        }
        assert!(
            Instant::now() < deadline,
            "initial fullscreen slot: {row}; configured={:?}",
            pattern.state.size
        );
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(pattern.state.size, (1600, 900));
}

#[test]
fn premap_set_then_unset_fullscreen_ends_at_normal_placement() {
    premap_fullscreen_order("premap-set-unset", &[true, false], false);
}

#[test]
fn premap_unset_then_set_fullscreen_fills_the_output() {
    premap_fullscreen_order("premap-unset-set", &[false, true], true);
}

fn premap_fullscreen_order(name: &str, requests: &[bool], fullscreen: bool) {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start(name);
    let mut process = process::spawn_size(name, &bus.url, "1600x900");
    // Requests are sent before even the initial empty surface commit.
    let mut pattern = Pattern::premap_fullscreen(connect(&mut process), requests);
    pattern.wait_callback(0);
    bus.ready();
    let expected = if fullscreen { (1600, 900) } else { (1280, 720) };
    let origin = if fullscreen { (0, 0) } else { (160, 90) };
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut resized = false;
    loop {
        pattern.pump();
        if !resized && pattern.state.size == expected {
            pattern.change_buffer();
            let before = pattern.state.callbacks;
            pattern.commit(true);
            pattern.wait_callback(before);
            resized = true;
        }
        let row = bus.window(&mut pattern);
        if resized
            && row["fullscreen"] == fullscreen
            && row["width"] == expected.0
            && row["height"] == expected.1
            && row["window_x"] == origin.0
            && row["window_y"] == origin.1
        {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "last pre-map request: {row}; configured={:?}",
            pattern.state.size
        );
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(pattern.state.size, expected);
}

fn binary(name: &str) -> PathBuf {
    std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join(name)
}

struct Bus {
    _owner: owned_bus::OwnedBus,
    // These tests measure a real render node and bounded recorder queues.
    // Keep fixtures in this binary exclusive even with libtest's default
    // parallelism; unrelated test binaries still use the normal CI pool.
    _capture: std::sync::MutexGuard<'static, ()>,
    root: PathBuf,
    url: String,
}
impl Bus {
    fn window_for(&self, name: &str, patterns: &mut [&mut Pattern]) -> Value {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let reply = self.call("comp.windows.list", json!({"app_id":name}));
            if let Some(row) = reply["windows"].as_array().unwrap().first() {
                return row.clone();
            }
            if Instant::now() >= deadline {
                eprintln!(
                    "WINDOW_TIMEOUT broker={} record={}",
                    self.url,
                    fs::read_to_string(self.root.join("bus.json")).unwrap_or_default()
                );
                for (command, args) in [
                    ("comp.info", json!({})),
                    ("comp.props.get", json!({"path":"info"})),
                    ("comp.windows.list", json!({})),
                    ("comp.props.get", json!({"path":"surfaces"})),
                    ("viud.truth", json!({})),
                ] {
                    eprintln!("WINDOW_TIMEOUT {command}: {}", self.call(command, args));
                }
                panic!("timed out after 10 s waiting for mapped window app_id={name}: {reply}");
            }
            // The fast path only observes the list. Dispatch client events
            // when waiting for the compositor to map the first buffer.
            for pattern in patterns.iter_mut() {
                pattern.pump();
            }
            thread::sleep(Duration::from_millis(20));
        }
    }
    fn start(label: &str) -> Self {
        static CAPTURE: std::sync::Mutex<()> = std::sync::Mutex::new(());
        let capture = CAPTURE
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let owner = owned_bus::OwnedBus::start(label);
        Self {
            _capture: capture,
            root: owner.root.clone(),
            url: owner.url.clone(),
            _owner: owner,
        }
    }
    fn call(&self, command_name: &str, args: Value) -> Value {
        self.call_rc(command_name, args, 0)
    }
    fn call_rc(&self, command_name: &str, args: Value, expected_rc: i32) -> Value {
        let output = helper(
            "call",
            &self.root,
            &[command_name, &args.to_string()],
            Some(&self.url),
        );
        let text = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success(),
            "Bus {command_name}: {text} {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let reply: Value = serde_json::from_str(text.lines().last().expect("Bus reply")).unwrap();
        assert_eq!(reply["rc"], expected_rc, "{command_name}: {reply}");
        match &reply["body"] {
            Value::String(body) => serde_json::from_str(body).unwrap(),
            body => body.clone(),
        }
    }
    fn ready(&self) {
        let output = helper("ready", &self.root, &[], Some(&self.url));
        assert!(
            output.status.success(),
            "headless Bus readiness: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        eprintln!(
            "BUS_READY broker={} instance={}",
            self.url,
            self.call("comp.props.get", json!({"path":"info"}))
        );
    }
    fn window(&self, pattern: &mut Pattern) -> Value {
        self.window_for("headless-pattern", &mut [pattern])
    }
}

fn target(row: &Value) -> Value {
    json!({"id":row["id"],"generation":row["generation"]})
}

fn scale_pump(pattern: &mut Pattern, duration: Duration) {
    let until = Instant::now() + duration;
    while Instant::now() < until {
        pattern.pump();
        thread::sleep(Duration::from_millis(2));
    }
}

fn scale_pump_rendered(
    bus: &Bus,
    process: &process::Process,
    pattern: &mut Pattern,
    duration: Duration,
) {
    let until = Instant::now() + duration;
    while Instant::now() < until {
        pattern.pump();
        bus.call("comp.capture.frame", json!({"output":"headless-0", "path":process.root.join("scale-tick.ppm"), "format":"ppm"}));
        pattern.pump();
        thread::sleep(Duration::from_millis(100));
    }
}

#[test]
fn fractional_workspace_return_keeps_scale_and_first_buffer() {
    fractional_return(false, false, false);
}

#[test]
fn fractional_unminimize_keeps_scale_and_first_buffer() {
    fractional_return(true, false, false);
}

#[test]
fn fractional_wake_precedes_first_returned_presentation() {
    fractional_return(false, true, false);
}

#[test]
fn fractional_pan_off_and_grace_band_republish_real_scale() {
    use pattern::ScaleEvent;
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("fractional-pan");
    let mut process = process::spawn_scale_probe("fractional-pan", &bus.url);
    let mut client = Pattern::scale_probe(connect(&mut process));
    client.wait_callback(0);
    bus.ready();
    let row = bus.window(&mut client);
    scale_pump(&mut client, Duration::from_secs(2));
    let x = row["window_x"].as_f64().unwrap();
    let y = row["window_y"].as_f64().unwrap();
    let start = client.state.events.len();
    // Right edge of the slot lies more than 256 logical units off-pane.
    fractional_camera(
        &bus,
        &process,
        &mut client,
        x + 200.0 + 256.0 + 256.0,
        y + 80.0,
        1.0,
    );
    scale_pump(&mut client, Duration::from_secs(2));
    assert!(
        client.state.events[start..]
            .iter()
            .any(|(_, e)| *e == ScaleEvent::Preferred(120)),
        "{:?}",
        client.state.events
    );
    let start = client.state.events.len();
    // Slot is still outside the pane, now inside its grace band.
    fractional_camera(
        &bus,
        &process,
        &mut client,
        x + 200.0 + 256.0 + 100.0,
        y + 80.0,
        1.0,
    );
    scale_pump(&mut client, Duration::from_secs(2));
    assert!(
        client.state.events[start..]
            .iter()
            .any(|(_, e)| *e == ScaleEvent::Preferred(300)),
        "{:?}",
        client.state.events
    );
}

#[test]
fn fractional_hidden_zoom_reaches_client_before_return() {
    use pattern::ScaleEvent;
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("fractional-zoom");
    let mut process = process::spawn_scale_probe("fractional-zoom", &bus.url);
    let mut client = Pattern::scale_probe(connect(&mut process));
    client.wait_callback(0);
    bus.ready();
    let row = bus.window(&mut client);
    scale_pump(&mut client, Duration::from_secs(2));
    bus.call("comp.workspace.switch", json!({"index":2}));
    let start = client.state.events.len();
    fractional_camera(
        &bus,
        &process,
        &mut client,
        row["window_x"].as_f64().unwrap() + 100.0,
        row["window_y"].as_f64().unwrap() + 80.0,
        2.0,
    );
    scale_pump(&mut client, Duration::from_secs(2));
    assert!(
        client.state.events[start..]
            .iter()
            .any(|(_, e)| *e == ScaleEvent::Preferred(540)),
        "{:?}",
        client.state.events
    );
    assert!(
        client.state.events[start..]
            .iter()
            .any(|(_, e)| *e == ScaleEvent::Commit(900, 720, 540)),
        "{:?}",
        client.state.events
    );
}

fn fractional_camera(
    bus: &Bus,
    process: &process::Process,
    client: &mut Pattern,
    x: f64,
    y: f64,
    zoom: f64,
) {
    fs::write(
        process.root.join("record-input-next"),
        format!("fractional-camera:{x}:{y}:{zoom}"),
    )
    .unwrap();
    fs::rename(
        process.root.join("record-input-next"),
        process.root.join("record-input"),
    )
    .unwrap();
    // A real policy operation requests a composition; the fixture only changes
    // the camera the production scene and scale aggregation subsequently read.
    client.commit(true);
    // A hidden commit damages no output. Force a real output capture to render.
    bus.call(
        "comp.capture.frame",
        json!({"output":"headless-0", "path":process.root.join("camera.ppm"), "format":"ppm"}),
    );
}

#[test]
fn fractional_unminimize_wakes_before_reveal() {
    fractional_return(true, true, false);
}

#[test]
fn fractional_restore_wakes_before_reveal() {
    fractional_return(true, true, true);
}

#[test]
fn fractional_minimised_destination_stays_suspended() {
    use pattern::ScaleEvent;
    if !process::record_available() { return; }
    let bus = Bus::start("fractional-still-hidden");
    let mut process = process::spawn_scale_probe("fractional-still-hidden", &bus.url);
    let mut client = Pattern::scale_probe(connect(&mut process));
    client.wait_callback(0);
    bus.ready();
    let row = bus.window(&mut client);
    bus.call("comp.window.minimize", target(&row));
    bus.call("comp.workspace.switch", json!({"index":2}));
    scale_pump_rendered(&bus, &process, &mut client, Duration::from_secs(7));
    // xdg-shell.xml xdg_toplevel.state: suspended=9, not zero-indexed.
    assert!(client.state.events.iter().any(|(_, e)| matches!(e, ScaleEvent::Configure(_, _, s) if s.contains(&9))));
    // Fence the 1 Hz trickle, then queue a fresh committed callback. A switch
    // must not answer it early for a window that remains minimised.
    let callbacks = client.state.callbacks;
    client.commit(true); // Explicitly requests wl_surface.frame + commits it.
    let deadline = Instant::now() + Duration::from_secs(3);
    while client.state.callbacks == callbacks {
        scale_pump_rendered(&bus, &process, &mut client, Duration::from_millis(50));
        assert!(Instant::now() < deadline);
    }
    client.commit(true);
    client.pump();
    let start = client.state.events.len();
    bus.call("comp.workspace.switch", json!({"index":1}));
    scale_pump(&mut client, Duration::from_millis(250));
    assert!(!client.state.events[start..].iter().any(|(_, e)| match e { ScaleEvent::Callback => true, ScaleEvent::Configure(_, _, s) => !s.contains(&9), _ => false }));
    assert_eq!(bus.window(&mut client)["minimized"], true);
}

#[test]
fn fractional_visual_hidden_keeps_full_idle_behaviour() {
    use pattern::ScaleEvent;
    if !process::record_available() { return; }
    let bus = Bus::start("fractional-visual-hidden");
    let mut process = process::spawn_scale_probe("fractional-visual-hidden", &bus.url);
    let mut client = Pattern::scale_probe(connect(&mut process));
    client.wait_callback(0);
    bus.ready();
    let row = bus.window(&mut client);
    scale_pump(&mut client, Duration::from_secs(2));
    assert_eq!(client.state.scale120, 300);
    bus.call("comp.window.place", place_args(&row, json!({"visible":false})));
    scale_pump_rendered(&bus, &process, &mut client, Duration::from_secs(2));
    assert_eq!(client.state.scale120, 120, "{:?}", client.state.events);
    let start = client.state.events.len();
    bus.call("comp.workspace.switch", json!({"index":2}));
    bus.call("comp.workspace.switch", json!({"index":1}));
    scale_pump_rendered(&bus, &process, &mut client, Duration::from_secs(2));
    assert!(!client.state.events[start..].iter().any(|(_, e)| matches!(e, ScaleEvent::Preferred(300))));
}

#[test]
fn fractional_parked_world_is_not_retained() {
    if !process::record_available() { return; }
    let bus = Bus::start("fractional-parked");
    let mut process = process::spawn_scale_probe("fractional-parked", &bus.url);
    let mut client = Pattern::scale_probe(connect(&mut process));
    client.wait_callback(0);
    bus.ready();
    bus.window(&mut client);
    scale_pump(&mut client, Duration::from_secs(2));
    bus.call("comp.workspace.switch", json!({"index":2}));
    scale_pump_rendered(&bus, &process, &mut client, Duration::from_secs(2));
    assert_eq!(client.state.scale120, 300);
    fs::write(process.root.join("record-input"), "fractional-park").unwrap();
    scale_pump_rendered(&bus, &process, &mut client, Duration::from_secs(2));
    assert_eq!(client.state.scale120, 120, "{:?}", client.state.events);
}

fn fractional_return(minimized: bool, suspended: bool, activate: bool) {
    use pattern::ScaleEvent;
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("fractional-return");
    let mut process = process::spawn_scale_probe("fractional-return", &bus.url);
    let mut client = Pattern::scale_probe(connect(&mut process));
    client.wait_callback(0);
    bus.ready();
    let row = bus.window(&mut client);
    scale_pump(&mut client, Duration::from_secs(2));
    assert_eq!(client.state.scale120, 300, "{:?}", client.state.events);
    let start = client.state.events.len();
    if minimized {
        bus.call("comp.window.minimize", target(&row));
    } else {
        bus.call("comp.workspace.switch", json!({"index":2}));
    }
    // debounce_max is 750 ms. Also cover the five-second suspend dwell.
    scale_pump_rendered(
        &bus,
        &process,
        &mut client,
        Duration::from_secs(if suspended { 7 } else { 2 }),
    );
    assert!(
        !client.state.events[start..]
            .iter()
            .any(|(_, e)| matches!(e, ScaleEvent::Preferred(_) | ScaleEvent::BufferScale(_))),
        "{:?}",
        client.state.events
    );
    // wayland-protocols/stable/xdg-shell/xdg-shell.xml: xdg_toplevel.state
    // starts at maximized=1; tiled_bottom=8, suspended=9 (since version 6).
    if suspended {
        assert!(
            client.state.events[start..].iter().any(
                |(_, e)| matches!(e, ScaleEvent::Configure(_, _, states) if states.contains(&9))
            ),
            "missing suspended configure: {:?}",
            client.state.events
        );
    }
    // Queue a callback while hidden; the arriving-window path must answer it
    // before a presentation of the returned content, without waiting for trickle.
    client.commit(true);
    client.pump();
    let returning = client.state.events.len();
    if minimized {
        let mut args = target(&row);
        args["activate"] = json!(activate);
        bus.call("comp.window.restore", args);
    } else {
        bus.call("comp.workspace.switch", json!({"index":1}));
    }
    scale_pump(&mut client, Duration::from_secs(2));
    let events = &client.state.events[returning..];
    assert!(
        !events
            .iter()
            .any(|(_, e)| matches!(e, ScaleEvent::Preferred(_) | ScaleEvent::BufferScale(_))),
        "{events:?}"
    );
    let first = events
        .iter()
        .position(|(_, e)| matches!(e, ScaleEvent::Presented(..)))
        .expect("returned presentation");
    // wayland-protocols/stable/xdg-shell/xdg-shell.xml: xdg_toplevel.state
    // starts at maximized=1; tiled_bottom=8, suspended=9 (since version 6).
    if suspended {
        assert!(
            events[..first].iter().any(
                |(_, e)| matches!(e, ScaleEvent::Configure(_, _, states) if !states.contains(&9))
            ),
            "wake configure after reveal: {events:?}"
        );
        assert!(
            events[..first]
                .iter()
                .any(|(_, e)| matches!(e, ScaleEvent::Callback)),
            "callback after reveal: {events:?}"
        );
    }
    let ScaleEvent::Presented(commit_id, w, h, scale) = events[first].1 else { unreachable!() };
    assert!(commit_id > 0);
    assert_eq!((w, h, scale), (500, 400, 300), "first presented commit: {events:?}");
}

#[test]
fn record_cold_start_under_load_has_no_capture_drops() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("cold-record");
    let mut process = process::spawn_size("cold-record", &bus.url, "320x240");
    let mut pattern = Pattern::new(connect(&mut process));
    pattern.wait_callback(0);
    bus.ready();
    let row = bus.window(&mut pattern);
    // Fence presentation using Wayland, not a screenshot: mapping/listing can
    // precede the first usable output frame, especially in the shared pool.
    bus.call(
        "comp.window.fullscreen",
        place_args(&row, json!({"output":"headless-0"})),
    );
    let configured = Instant::now() + Duration::from_secs(5);
    while pattern.state.size != (320, 240) {
        pattern.pump();
        assert!(
            Instant::now() < configured,
            "fullscreen configure timed out"
        );
        thread::sleep(Duration::from_millis(2));
    }
    pattern.change_buffer();
    let before = pattern.state.callbacks;
    pattern.commit(true);
    pattern.wait_callback(before);
    // No screenshot/readback readiness probe: this is a fresh compositor.
    struct StopLoaders(std::sync::Arc<std::sync::atomic::AtomicBool>);
    impl Drop for StopLoaders {
        fn drop(&mut self) {
            self.0.store(false, std::sync::atomic::Ordering::Relaxed);
        }
    }
    let running = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(true));
    thread::scope(|scope| {
        let _stop = StopLoaders(running.clone());
        for _ in 0..8 {
            let running = running.clone();
            scope.spawn(move || {
                while running.load(std::sync::atomic::Ordering::Relaxed) {
                    // Keep competing CPU work through the measured second,
                    // with scheduling room for the other pool tests. The
                    // injected first-read cost isolates cold-path deferral;
                    // starving every later read would test scheduler jitter.
                    let busy_until = Instant::now() + Duration::from_millis(1);
                    while Instant::now() < busy_until {
                        std::hint::spin_loop();
                    }
                    thread::sleep(Duration::from_millis(4));
                }
            });
        }
        let begin = Instant::now();
        let starting = scope.spawn(|| bus.call("comp.record.start", json!({"request_id":"cold", "target":{"output":"headless-0"}, "path":process.root.join("cold.vfl"), "max_seconds":10, "fps":30, "cursor":false})));
        // Damage once immediately after the first completed read. In the
        // bypassed build this makes frame zero's debt observable before an
        // idle tick can repay it. Repeated commits while start is pending
        // would instead test superseding snapshots during admission.
        let first_read = Instant::now() + Duration::from_secs(3);
        while !fs::read_to_string(process.root.join("viud.log"))
            .unwrap_or_default()
            .contains("TEST_COLD_READBACK n=1 penalty_us=20000")
        {
            assert!(Instant::now() < first_read, "first read did not complete");
            pattern.pump();
            thread::sleep(Duration::from_millis(1));
        }
        pattern.change_buffer();
        pattern.commit(true);
        while !starting.is_finished() {
            pattern.pump();
            thread::sleep(Duration::from_millis(1));
        }
        let start = starting.join().unwrap();
        let start_ms = begin.elapsed().as_secs_f64() * 1000.0;
        let first_second = Instant::now();
        while first_second.elapsed() < Duration::from_secs(1) {
            // Continue changing content, leaving room to complete each GPU
            // snapshot under the shared pool. This tests cold admission; the
            // strict changing 4K30 case covers sustained capture throughput.
            thread::sleep(Duration::from_millis(200));
            pattern.change_buffer();
            pattern.commit(true);
            pattern.pump();
        }
        let status = bus.call("comp.record.status", json!({"session":start["session"]}));
        eprintln!("cold record start_ms={start_ms:.3} status={status}");
        assert!(start_ms < 3000.0, "start must be bounded: {start_ms} ms");
        // Check deferrals first so bypassing only warm-up produces a budget
        // failure, rather than merely a missing warm-up status assertion.
        assert_eq!(status["readback_deferred"], 0, "{status}");
        assert_eq!(status["warmup_done"], true, "{status}");
        assert_eq!(status["warmup_errors"], json!([]), "{status}");
        assert!(status["warmup_us"].as_u64().unwrap() >= 20_000, "{status}");
        assert!(
            status["readback_max_us"].as_u64().unwrap() < 16_667,
            "{status}"
        );
        assert!(status["readbacks"].as_u64().unwrap() > 1, "{status}");
        assert_eq!(status["dropped_capture"], 0, "{status}");
        assert_eq!(status["readback_deferred"], 0, "{status}");
        let stop = bus.call("comp.record.stop", json!({"session":start["session"]}));
        let log = fs::read_to_string(process.root.join("viud.log")).unwrap();
        let reads = log
            .lines()
            .filter(|line| line.starts_with("TEST_COLD_READBACK n="))
            .count();
        assert!(
            log.contains("n=1 penalty_us=20000"),
            "cold penalty hook did not fire"
        );
        assert_eq!(
            stop["readbacks"].as_u64().unwrap() + 1,
            reads as u64,
            "exactly one read is outside the take: {stop}"
        );
        let mut frames = 0_u64;
        vfl_visit(&process.root.join("cold.vfl"), |_| frames += 1);
        assert_eq!(stop["frames"], frames, "warm-up must not enter the take");
        let terminal = bus.call("comp.record.status", json!({"session":start["session"]}));
        assert_eq!(terminal["warmup_us"], status["warmup_us"]);
    });
}

#[test]
fn mapped_window_survives_render_before_protocol_drain() {
    if !process::record_available() {
        return;
    }
    // Re-exec so the forced source ordering cannot affect parallel tests'
    // compositor children through process-global environment mutation.
    if std::env::var_os(paths::env::test::TEST_PRE_DRAIN_CHILD).is_none() {
        let output = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "mapped_window_survives_render_before_protocol_drain",
            ])
            .env(paths::env::test::TEST_PRE_DRAIN_FRAME, "1")
            .env(paths::env::test::TEST_PRE_DRAIN_CHILD, "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "pre-drain frame: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            String::from_utf8_lossy(&output.stdout)
                .contains("test result: ok. 1 passed; 0 failed;"),
            "pre-drain child must run exactly one test: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    let bus = Bus::start("pre-drain-map");
    let mut process = process::spawn(1, None, "pre-drain-map", Some(&bus.url));
    let mut pattern = Pattern::named(connect(&mut process), "headless-pattern", false);
    pattern.wait_callback(0);
    bus.ready();
    let window = bus.window(&mut pattern);
    assert_eq!(window["app_id"], "headless-pattern");
    assert_eq!(window["pid"], std::process::id());
    assert!(
        fs::read_to_string(process.root.join("viud.log"))
            .unwrap()
            .contains("HEADLESS_TEST_PRE_DRAIN_FRAME fired"),
        "pre-drain fixture did not fire; build viud with --features test-support"
    );
}

#[test]
fn record_idle_content_clock_ignores_capture_repaints() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("record-repaints");
    let mut process = process::spawn(1, None, "record-repaints", Some(&bus.url));
    let mut pattern = Pattern::named(connect(&mut process), "headless-pattern", false);
    pattern.wait_callback(0);
    bus.ready();
    let row = bus.window(&mut pattern);
    let output = row["output"].as_str().unwrap();
    let start=bus.call("comp.record.start",json!({"request_id":"repaints","target":{"output":output},"path":process.root.join("idle.vfl"),"max_seconds":20}));
    let before = bus.call("comp.record.status", json!({"session":start["session"]}));
    for i in 0..8 {
        bus.call("comp.capture.frame",json!({"output":output,"path":process.root.join(format!("repaint-{i}.ppm")),"format":"ppm"}));
        pattern.pump();
        thread::sleep(Duration::from_millis(35));
    }
    let stop = bus.call("comp.record.stop", json!({"session":start["session"]}));
    assert_eq!(
        stop["readbacks"], before["readbacks"],
        "capture-only redraws must not resample: {stop}"
    );
    assert_eq!(
        stop["dropped"], 0,
        "idle is repeat, even with readback debt or queue pressure: {stop}"
    );
    assert!(
        stop["writer"].is_object(),
        "stage attribution must survive stop"
    );
}

// Test-only container inspection, independent of the writer's counters. R2
// owns a production reader. Retain only the first image so 4K tests stay bounded.
fn vfl_first(path: &Path) -> (Vec<u8>, u64) {
    let (bgra, count) = vfl_visit(path, |_| {});
    // The reader exposes BGRA; the older first-image assertions use RGB.
    (
        bgra.chunks_exact(4)
            .flat_map(|p| [p[2], p[1], p[0]])
            .collect(),
        count,
    )
}

fn assert_recorded_colours(path: &Path, expected: &[u32]) -> u64 {
    let manifest: Value =
        serde_json::from_slice(&fs::read(path.join("manifest.json")).unwrap()).unwrap();
    let width = manifest["width"].as_u64().unwrap() as usize;
    let height = manifest["height"].as_u64().unwrap() as usize;
    let at = ((height * 3 / 4) * width + width * 3 / 4) * 4;
    let mut found = 0;
    let mut frame = 0;
    let mut last = None;
    let (_, records) = vfl_visit(path, |pixels| {
        let colour = u32::from_le_bytes(pixels[at..at + 4].try_into().unwrap());
        if last != Some(colour) {
            println!("VFL_COLOUR frame={frame} colour={colour:08x}");
            last = Some(colour);
        }
        frame += 1;
        if found < expected.len() && pixels[at..at + 4] == expected[found].to_le_bytes() {
            found += 1;
        }
    });
    assert_eq!(found, expected.len(), "VFL lost an admitted colour change");
    records
}

fn vfl_visit(path: &Path, mut visit: impl FnMut(&[u8])) -> (Vec<u8>, u64) {
    let mut reader = media_vfl::reader::Reader::open(path).unwrap();
    let mut first = Vec::new();
    let mut count = 0;
    while let Some(frame) = reader.read().unwrap() {
        if count == 0 {
            first = frame.pixels.to_vec();
        }
        visit(frame.pixels);
        count += 1;
    }
    (first, count)
}

/// A live scan acknowledges only complete, CRC-checked records. The active
/// chunk may end in a partially written record; sealed scans remain strict.
fn vfl_scan(
    path: &Path,
    mut visit: impl FnMut(&[u8]),
    sealed: bool,
) -> std::io::Result<(Vec<u8>, u64)> {
    let manifest: Value = serde_json::from_slice(&fs::read(path.join("manifest.json"))?).unwrap();
    assert!([json!(1), json!(2)].contains(&manifest["version"]));
    let size = manifest["width"].as_u64().unwrap() as usize
        * manifest["height"].as_u64().unwrap() as usize
        * 4;
    let mut chunks: Vec<_> = fs::read_dir(path)?
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|e| e == "bin"))
        .collect();
    chunks.sort();
    let mut first = Vec::new();
    let mut count = 0;
    let mut pixels = vec![0; size];
    for chunk in chunks {
        let bytes = fs::read(chunk)?;
        let mut at = 0;
        while at < bytes.len() {
            if !sealed && bytes.len() - at < 17 {
                break;
            }
            let h = &bytes[at..at + 17];
            let len = u32::from_le_bytes(h[..4].try_into().unwrap()) as usize;
            if !sealed && bytes.len() - at - 17 < len {
                break;
            }
            let body = &bytes[at + 8..at + 17 + len];
            assert_eq!(
                crc32fast::hash(body),
                u32::from_le_bytes(h[4..8].try_into().unwrap())
            );
            assert_eq!(u64::from_le_bytes(h[8..16].try_into().unwrap()), count);
            let payload = &body[9..];
            match h[16] {
                0 => pixels = lz4_flex::block::decompress(payload, size).unwrap(),
                1 => {
                    let delta = lz4_flex::block::decompress(payload, size).unwrap();
                    for (p, d) in pixels.iter_mut().zip(delta) {
                        *p ^= d;
                    }
                }
                2 => assert!(payload.is_empty()),
                3 => {
                    let n =
                        |at| u32::from_le_bytes(payload[at..at + 4].try_into().unwrap()) as usize;
                    let (x, y, w, h) = (n(0), n(4), n(8), n(12));
                    let width = manifest["width"].as_u64().unwrap() as usize;
                    let height = manifest["height"].as_u64().unwrap() as usize;
                    assert!(x + w <= width && y + h <= height);
                    let delta = lz4_flex::block::decompress(&payload[16..], w * h * 4).unwrap();
                    for row in 0..h {
                        let at = ((y + row) * width + x) * 4;
                        for (p, d) in pixels[at..at + w * 4]
                            .iter_mut()
                            .zip(&delta[row * w * 4..(row + 1) * w * 4])
                        {
                            *p ^= d;
                        }
                    }
                }
                kind => panic!("unknown VFL record kind {kind}"),
            }
            visit(&pixels);
            if count == 0 {
                assert_eq!(h[16], 0);
                first = pixels
                    .chunks_exact(4)
                    .flat_map(|p| [p[2], p[1], p[0]])
                    .collect();
            }
            count += 1;
            at += 17 + len;
        }
    }
    if sealed {
        let complete: Value =
            serde_json::from_slice(&fs::read(path.join("complete.json")).unwrap()).unwrap();
        assert_eq!(complete["complete"], true);
        assert_eq!(complete["frames"], count);
    }
    Ok((first, count))
}

#[test]
fn record_changing_4k_ten_seconds() {
    changing_4k(false);
}

/// State-paced correctness, deliberately independent of worker frame rate.
/// The ignored 4K benchmark separately checks sustained throughput.
#[test]
fn record_small_changes_zero_drops_and_crcs() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("record-small");
    let mut process = process::spawn_size("record-small", &bus.url, "320x180");
    process.extend_watchdog(Duration::from_secs(120));
    let mut p = Pattern::new(connect(&mut process));
    p.wait_callback(0);
    bus.ready();
    let row = bus.window(&mut p);
    bus.call(
        "comp.window.fullscreen",
        place_args(&row, json!({"output":"headless-0"})),
    );
    pump_patterns(&mut [&mut p]);
    p.state.size = (320, 180);
    p.solid_buffer(0xff20d040);
    let previous = p.state.callbacks;
    p.commit(true);
    p.wait_callback(previous);
    let path = process.root.join("small.vfl");
    let start=bus.call("comp.record.start",json!({"request_id":"small","target":{"output":"headless-0"},"path":path,"fps":10,"cursor":false,"max_seconds":120}));
    for i in 0..8 {
        let colour: u32 = 0xffa03370 + i;
        p.solid_buffer(colour);
        p.commit(true);
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            p.pump();
            let status = bus.call("comp.record.status", json!({"session":start["session"]}));
            assert_eq!(status["error"], Value::Null, "{status}");
            let mut retained = false;
            match vfl_scan(
                Path::new(start["recovery_path"].as_str().unwrap()),
                |pixels| {
                    let at = (90 * 320 + 160) * 4;
                    retained |= pixels[at..at + 4] == colour.to_le_bytes();
                },
                false,
            ) {
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => panic!("live small VFL scan: {error}"),
            }
            if retained {
                break;
            }
            assert!(
                Instant::now() < deadline,
                "changed state never captured: {status}"
            );
            thread::sleep(Duration::from_millis(10));
        }
    }
    let stop = bus.call("comp.record.stop", json!({"session":start["session"]}));
    assert_eq!(stop["error"], Value::Null, "{stop}");
    assert_eq!(stop["dropped"], 0, "{stop}");
    let (_, records) = vfl_first(&path);
    assert_eq!(Some(records), stop["frames"].as_u64());
    assert!(stop["readbacks"].as_u64().unwrap() >= 9);
    assert!(stop["writer"]["delta_records"].as_u64().unwrap() > 0);
    let mut expected = 0u8;
    vfl_visit(&path, |pixels| {
        let at = (90 * 320 + 160) * 4;
        if expected < 8 && pixels[at..at + 4] == [0x70 + expected, 0x33, 0xa0, 0xff] {
            expected += 1;
        }
    });
    assert_eq!(
        expected, 8,
        "every committed colour must survive in the VFL, in order"
    );
}

/// Throughput acceptance must run alone, with release runtime binaries.
#[test]
#[ignore = "release 4K30 zero-drop acceptance on a render worker"]
fn benchmark_record_changing_4k_zero_drop() {
    #[cfg(debug_assertions)]
    panic!("use cargo test --release");
    #[cfg(not(debug_assertions))]
    {
        assert!(process::record_available());
        changing_4k(true);
    }
}

fn changing_4k(strict: bool) {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("record-changing");
    let mut process = process::spawn_4k("record-changing", &bus.url);
    process.extend_watchdog(Duration::from_secs(180));
    let seconds: u64 = std::env::var("MIX_TEST_CAPTURE_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10);
    let mut p = Pattern::new(connect(&mut process));
    p.wait_callback(0);
    bus.ready();
    let row = bus.window(&mut p);
    bus.call(
        "comp.window.fullscreen",
        place_args(&row, json!({"output":"headless-0"})),
    );
    pump_patterns(&mut [&mut p]);
    // The fixture's initial xdg configure chooses 1280x720 even on a larger
    // output. Submit full-output buffers explicitly and prove their coverage.
    p.state.size = (3840, 2160);
    p.solid_buffer(0xff20d040);
    let mut alternate = p.current_buffer();
    p.change_buffer();
    let previous = p.state.callbacks;
    p.commit(true);
    p.wait_callback(previous);
    let (width, height, pixels) = ppm(&bus, &process, "full-4k-workload", None);
    assert_eq!((width, height), (3840, 2160));
    let at = (2100 * width + 3800) * 3;
    assert_eq!(&pixels[at..at + 3], &[0xaa, 0x33, 0x77]);
    if std::env::var("MIX_TEST_CAPTURE_OSD").as_deref() == Ok("1") {
        osd::ready(&bus);
        osd::call(
            &bus,
            "osd.show",
            json!({
                "id":"record-benchmark",
                "scene":{"kind":"caption","text":"Native 4K30 OSD recording","width":480,"height":80},
                "anchor":{"output":"headless-0","edge":"bottom-right","margin":24},
                "enter":"up"
            }),
        );
    }
    for recording in [false, true] {
        let start = recording.then(|| bus.call("comp.record.start", json!({"request_id":"changing", "target":{"output":"headless-0"}, "path":process.root.join("changing.vfl"), "max_seconds":seconds+2,"fps":30,"cursor":true})));
        let begin_us = monotonic_ns() / 1000;
        let begin = Instant::now();
        let mut commits = 0;
        while begin.elapsed() < Duration::from_secs(seconds) {
            p.swap_buffer(&mut alternate);
            p.state.size = (3840, 2160);
            p.commit(true);
            commits += 1;
            let next = begin + Duration::from_nanos(commits * 1_000_000_000 / 30);
            while Instant::now() < next {
                p.pump();
                thread::sleep(Duration::from_millis(1));
            }
        }
        let end_us = monotonic_ns() / 1000;
        let stop = start.map(|s| bus.call("comp.record.stop", json!({"session":s["session"]})));
        thread::sleep(Duration::from_millis(300));
        let trace = fs::read_to_string(process.root.join("frames.log")).unwrap();
        let field = |line: &str, key: &str| -> u64 {
            line.split_whitespace()
                .find_map(|word| word.strip_prefix(key)?.parse().ok())
                .unwrap()
        };
        let mut times: Vec<u64> = trace
            .lines()
            .filter(|line| line.contains("stage=headless_frame_cpu "))
            .filter(|line| (begin_us..end_us).contains(&field(line, "start_us=")))
            .map(|line| field(line, "duration_us="))
            .collect();
        assert!(!times.is_empty());
        times.sort_unstable();
        for stage in [
            "capture_base_render_wait",
            "capture_cursor_render_wait",
            "capture_offscreen_render",
            "capture_copy_submit",
            "capture_map_wait",
            "capture_map_latency",
            "capture_rgb_copy",
            "capture_file_service",
            "headless_readback_poll_cpu",
        ] {
            let mut samples: Vec<u64> = trace
                .lines()
                .filter(|line| line.contains(&format!("stage={stage} ")))
                .filter(|line| (begin_us..end_us).contains(&field(line, "start_us=")))
                .map(|line| field(line, "duration_us="))
                .collect();
            samples.sort_unstable();
            if !samples.is_empty() {
                println!(
                    "READBACK stage={stage} n={} median_us={} p95_us={}",
                    samples.len(),
                    samples[samples.len() / 2],
                    samples[samples.len() * 95 / 100]
                );
            }
            if stage == "capture_rgb_copy" {
                for line in trace
                    .lines()
                    .filter(|line| line.contains("stage=capture_rgb_copy "))
                    .filter(|line| (begin_us..end_us).contains(&field(line, "start_us=")))
                    .filter(|line| field(line, "duration_us=") > 8000)
                {
                    println!(
                        "READBACK_BURST offset_us={} duration_us={}",
                        field(line, "start_us=") - begin_us,
                        field(line, "duration_us=")
                    );
                }
            }
        }
        println!(
            "CHANGING_4K30 recording={recording} commits={commits} compositor_frames={} median_us={} p95_us={} max_us={} recording_stats={stop:?}",
            times.len(),
            times[times.len() / 2],
            times[times.len() * 95 / 100],
            times.last().unwrap()
        );
        if let Some(stop) = stop {
            assert_eq!(stop["error"], Value::Null, "{stop}");
            if strict {
                assert_eq!(stop["dropped"], 0, "{stop}");
                let (_, frames) = vfl_first(&process.root.join("changing.vfl"));
                assert_eq!(Some(frames), stop["frames"].as_u64());
                assert!(
                    frames >= commits,
                    "retain the submitted sampling slots for the requested {seconds}s: {stop}"
                );
            }
            // Debug readback can exceed a frame budget and deliberately
            // throttle captures. This is a changing-content regression;
            // release throughput is measured separately.
            assert!(stop["readbacks"].as_u64().unwrap() > 1);
        }
    }
}

#[test]
fn record_windows_only_content_and_multiple_targets() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("record-windows");
    let mut process = process::spawn(1, None, "record-windows", Some(&bus.url));
    let mut b = Pattern::named(connect(&mut process), "record-window-b", false);
    let mut a = Pattern::named(connect(&mut process), "record-window-a", false);
    bus.ready();
    a.change_buffer();
    a.commit(true);
    pump_patterns(&mut [&mut a, &mut b]);
    let ar = bus.window_for("record-window-a", &mut [&mut a, &mut b]);
    let br = bus.window_for("record-window-b", &mut [&mut a, &mut b]);
    for (row, z) in [(&br, 0), (&ar, 10)] {
        bus.call(
            "comp.window.place",
            place_args(row, json!({"x":100,"y":100,"z":z})),
        );
    }
    let start=bus.call("comp.record.start",json!({"request_id":"windows","target":{"windows":[target(&br),target(&ar)]},"cursor":false,"path":process.root.join("windows.vfl"),"max_seconds":5}));
    assert_eq!(
        start["width"], 400,
        "no server frame or shadow is part of a window take"
    );
    assert_eq!(start["height"], 160);
    thread::sleep(Duration::from_millis(150));
    let stop = bus.call("comp.record.stop", json!({"session":start["session"]}));
    assert_eq!(stop["error"], Value::Null, "{stop}");
    let (decoded, _) = vfl_first(Path::new(start["path"].as_str().unwrap()));
    assert_eq!(decoded.len(), 400 * 160 * 3);
    for (x, expected) in [(50, rgb(COLOURS[0])), (250, [0xaa, 0x33, 0x77])] {
        let at = (40 * 400 + x) * 3;
        for (actual, expected) in decoded[at..at + 3].iter().zip(expected) {
            assert!((i16::from(*actual) - i16::from(expected)).abs() < 20);
        }
    }
}

#[test]
fn record_vaapi_capability_or_clear_failure() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("record-vaapi");
    let mut process = process::spawn(1, None, "record-vaapi", Some(&bus.url));
    drop(connect(&mut process));
    bus.ready();
    assert_eq!(
        bus.call("comp.record.status", json!({}))["encoders"],
        json!(["vfl"])
    );
    bus.call_rc("comp.record.start", json!({"request_id":"mkv","target":{},"path":process.root.join("refused.mkv"),"max_seconds":1}),10);
    bus.call_rc("comp.record.start", json!({"request_id":"vaapi","target":{},"path":process.root.join("refused.vfl"),"max_seconds":1,"encoder":"vaapi"}),10);
}

#[test]
fn record_listed_output_excludes_occluders_and_tracks_popup_and_close() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("record-listed");
    let mut process = process::spawn(1, None, "record-listed", Some(&bus.url));
    let mut a = Pattern::named(connect(&mut process), "listed", false);
    let mut b = Pattern::named(connect(&mut process), "private", false);
    use wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Layer;
    let _wallpaper = b.layer("backdrop", Layer::Background, 0xff213243);
    let _foreign_background = b.layer("private-background", Layer::Background, 0xff12abcd);
    let _overlay = b.layer("private-overlay", Layer::Overlay, 0xff12abcd);
    b.solid_buffer(0xff12abcd);
    b.commit(true);
    bus.ready();
    let ar = bus.window_for("listed", &mut [&mut a, &mut b]);
    let br = bus.window_for("private", &mut [&mut a, &mut b]);
    for (row, z) in [(&ar, 0), (&br, 10)] {
        bus.call(
            "comp.window.place",
            place_args(row, json!({"x":100,"y":100,"z":z})),
        );
    }
    pump_patterns(&mut [&mut a, &mut b]);
    let path = process.root.join("listed.vfl");
    let start = bus.call("comp.record.start", json!({"request_id":"listed", "target":{"output":ar["output"],"windows":[target(&ar)]},"path":path,"cursor":false,"max_seconds":20}));
    assert_eq!(start["width"], 1280);
    let before = bus.call("comp.record.status", json!({"session":start["session"]}));
    assert_eq!(before["windows"], json!([target(&ar)]));
    let live = process.root.join("live.ppm");
    bus.call(
        "comp.capture.frame",
        json!({"output":ar["output"],"path":live,"format":"ppm","cursor":false}),
    );
    assert!(
        fs::read(live)
            .unwrap()
            .windows(3)
            .any(|p| p == rgb(0xff12abcd)),
        "the private overlay must remain on the live screen"
    );
    let (_private_surface, private_popup) = b.popup(false);
    pump_patterns(&mut [&mut a, &mut b]);
    thread::sleep(Duration::from_millis(100));
    let excluded = bus.call("comp.record.status", json!({"session":start["session"]}));
    assert_eq!(
        excluded["readbacks"], before["readbacks"],
        "unlisted popup causes no readback"
    );
    let recovery = Path::new(start["recovery_path"].as_str().unwrap());
    // This is a live take: EOF moves as the writer appends repeat records.
    // A complete frame sidecar row follows its chunk payload. Wait for the
    // entire observed prefix before opening the reader, which stops at EOF.
    let bound = excluded["frames"].as_u64().unwrap();
    assert!(bound > 0, "the live prefix must contain decoded pixels");
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        let written = fs::read_to_string(recovery.join("frames.jsonl"))
            .unwrap()
            .split_inclusive('\n')
            .filter(|line| line.ends_with('\n'))
            .count() as u64;
        if written >= bound {
            break;
        }
        assert!(
            Instant::now() < until,
            "live prefix not written: {written}/{bound} frames"
        );
        thread::sleep(Duration::from_millis(20));
    }
    let mut reader = media_vfl::reader::Reader::open(recovery).unwrap();
    let mut inspected = 0;
    for _ in 0..bound {
        let frame = reader
            .read()
            .unwrap()
            .expect("written live prefix must be fully readable");
        inspected += 1;
        assert!(
            !frame
                .pixels
                .chunks_exact(4)
                .any(|p| p == 0xffff00ff_u32.to_le_bytes())
        );
    }
    assert_eq!(inspected, bound, "inspect the entire observed live prefix");
    private_popup.destroy();
    let mut c = Pattern::named(connect(&mut process), "opened-during-take", false);
    c.change_buffer();
    c.commit(true);
    bus.window_for("opened-during-take", &mut [&mut a, &mut b, &mut c]);
    for _ in 0..5 {
        b.change_buffer();
        b.commit(true);
        pump_patterns(&mut [&mut a, &mut b]);
    }
    let quiet = bus.call("comp.record.status", json!({"session":start["session"]}));
    assert_eq!(
        quiet["readbacks"], before["readbacks"],
        "unlisted damage: {quiet}"
    );
    let (_surface, popup) = a.popup(false);
    pump_patterns(&mut [&mut a, &mut b]);
    thread::sleep(Duration::from_millis(150));
    let with_popup = bus.call("comp.record.status", json!({"session":start["session"]}));
    assert!(with_popup["readbacks"].as_u64().unwrap() > quiet["readbacks"].as_u64().unwrap());
    popup.destroy();
    drop(a);
    thread::sleep(Duration::from_millis(150));
    let closed = bus.call("comp.record.status", json!({"session":start["session"]}));
    assert_eq!(closed["windows"], json!([]));
    assert_eq!(closed["state"], "recording");
    let stop = bus.call("comp.record.stop", json!({"session":start["session"]}));
    assert_eq!(stop["error"], Value::Null, "{stop}");
    let mut saw_app = false;
    let mut saw_popup = false;
    let mut saw_wallpaper = false;
    let mut last_had_app = false;
    vfl_visit(&path, |pixels| {
        last_had_app = false;
        for pixel in pixels.chunks_exact(4) {
            let c = u32::from_le_bytes(pixel.try_into().unwrap());
            assert_ne!(c, 0xff12abcd, "private initial pixels leaked");
            assert_ne!(c, 0xffaa3377, "private updated pixels leaked");
            saw_app |= COLOURS.contains(&c);
            last_had_app |= COLOURS.contains(&c);
            saw_popup |= c == 0xffff00ff;
            saw_wallpaper |= c == 0xff213243;
        }
    });
    assert!(saw_app && saw_popup, "listed app and popup must be decoded");
    assert!(saw_wallpaper, "the real wallpaper band must be decoded");
    assert!(
        !last_had_app,
        "closing the last listed window must clear its pixels"
    );
    let markers = fs::read_to_string(path.join("markers.jsonl")).unwrap();
    let gone: Vec<Value> = markers
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .filter(|m: &Value| m["label"] == "window_gone")
        .collect();
    assert_eq!(gone.len(), 1);
    assert_eq!(gone[0]["data"], target(&ar));
    assert!(path.join("complete.json").exists());
    assert!(
        stop["repeated"].as_u64().unwrap() > 0,
        "quiet ticks must repeat: {stop}"
    );
}

#[test]
fn record_privacy_cursor_ownership_output_and_take_identity() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("record-privacy-cursor");
    let mut process = process::spawn(1, None, "record-privacy-cursor", Some(&bus.url));
    let mut a = Pattern::named(connect(&mut process), "cursor-listed", false);
    a.solid_buffer(0xff334455);
    a.commit(true);
    // Two toplevels share one Wayland connection: client identity alone must
    // not grant the private window permission to set the recorded cursor.
    let mut b = a.another_window("cursor-private");
    b.solid_buffer(0xff667788);
    b.commit(true);
    bus.ready();
    let ar = bus.window_for("cursor-listed", &mut [&mut a, &mut b]);
    let br = bus.window_for("cursor-private", &mut [&mut a, &mut b]);
    bus.call(
        "comp.window.place",
        place_args(&ar, json!({"output":"headless-0","x":100,"y":100})),
    );
    bus.call(
        "comp.window.place",
        place_args(&br, json!({"output":"headless-0","x":400,"y":100})),
    );
    let focus = |mode: &str, a: &mut Pattern, b: &mut Pattern| {
        fs::write(process.root.join("record-input"), mode).unwrap();
        bus.call("comp.capture.frame", json!({"output":"headless-0","path":process.root.join(format!("focus-{mode}.ppm")),"format":"ppm"}));
        pump_patterns(&mut [a, b]);
    };
    let take = |name: &str, cursor: bool, a: &mut Pattern, b: &mut Pattern| {
        eprintln!("LISTED_CURSOR_TAKE {name}");
        let path = process.root.join(format!("{name}.vfl"));
        let start = bus.call("comp.record.start",json!({"request_id":name,"target":{"output":"headless-0","windows":[target(&ar)]},"cursor":cursor,"path":path,"max_seconds":10}));
        pump_patterns(&mut [a, b]);
        thread::sleep(Duration::from_millis(150));
        let stop = bus.call("comp.record.stop", json!({"session":start["session"]}));
        assert_eq!(stop["error"], Value::Null, "{stop}");
        assert!(
            stop["readbacks"].as_u64().unwrap() >= 1,
            "frame zero must be captured"
        );
        assert!(stop["repeated"].as_u64().unwrap() > 0);
        let mut frames = Vec::new();
        vfl_visit(&path, |pixels| frames.push(pixels.to_vec()));
        assert!(!frames.is_empty());
        frames
    };
    focus("listed", &mut a, &mut b);
    let baseline = take("hyphenated-take-a-1", false, &mut a, &mut b);
    let second = take("hyphenated-take-a-2", false, &mut a, &mut b);
    assert_eq!(
        baseline[0], second[0],
        "consecutive takes both have real identical first images"
    );
    let default = take("default-cursor", true, &mut a, &mut b);
    assert_ne!(default[0], baseline[0], "compositor cursor must appear");
    let contains = |frames: &[Vec<u8>], colour: u32| {
        frames
            .iter()
            .any(|f| f.chunks_exact(4).any(|p| p == colour.to_le_bytes()))
    };
    let _allowed = a.custom_cursor(0xff11eecc);
    pump_patterns(&mut [&mut a, &mut b]);
    assert!(contains(
        &take("owned-cursor", true, &mut a, &mut b),
        0xff11eecc
    ));
    focus("unknown", &mut a, &mut b);
    assert_eq!(
        take("unknown-cursor-output", true, &mut a, &mut b)[0],
        baseline[0]
    );
    focus("other", &mut a, &mut b);
    assert_eq!(
        take("other-cursor-output", true, &mut a, &mut b)[0],
        baseline[0]
    );
    focus("private", &mut a, &mut b);
    let _private_cursor = b.custom_cursor(0xffff00ff);
    pump_patterns(&mut [&mut a, &mut b]);
    let live = process.root.join("cursor-live.ppm");
    bus.call(
        "comp.capture.frame",
        json!({"output":"headless-0","path":live,"format":"ppm"}),
    );
    assert!(
        fs::read(live)
            .unwrap()
            .windows(3)
            .any(|p| p == rgb(0xffff00ff)),
        "private cursor fixture must draw live"
    );
    let denied = take("private-cursor", true, &mut a, &mut b);
    assert_ne!(
        denied[0], baseline[0],
        "fallback default cursor must appear"
    );
    assert!(
        !contains(&denied, 0xffff00ff),
        "unlisted cursor artwork leaked"
    );
    b.hide_cursor();
    pump_patterns(&mut [&mut a, &mut b]);
    let hidden_live = process.root.join("hidden-live.ppm");
    let without_cursor = process.root.join("hidden-live-no-cursor.ppm");
    for (path, cursor) in [(&hidden_live, true), (&without_cursor, false)] {
        bus.call(
            "comp.capture.frame",
            json!({"output":"headless-0","path":path,"format":"ppm","cursor":cursor}),
        );
    }
    assert_eq!(
        fs::read(hidden_live).unwrap(),
        fs::read(without_cursor).unwrap(),
        "private hidden cursor must be hidden live"
    );
    let hidden = take("private-hidden-cursor", true, &mut a, &mut b);
    assert_eq!(
        hidden[0], denied[0],
        "private Hidden state must still draw the default cursor in the take"
    );
    assert_ne!(hidden[0], baseline[0], "default cursor must remain visible");
    // Retain the private custom cursor for the independent drag exclusion case.
    let _private_cursor = b.custom_cursor(0xffff00ff);
    let _drag = b.drag_icon(0xffeedd11);
    pump_patterns(&mut [&mut a, &mut b]);
    let live = process.root.join("drag-live.ppm");
    bus.call(
        "comp.capture.frame",
        json!({"output":"headless-0","path":live,"format":"ppm"}),
    );
    assert!(
        fs::read(live)
            .unwrap()
            .windows(3)
            .any(|p| p == rgb(0xffeedd11)),
        "live drag icon fixture must draw"
    );
    let drag = take("private-drag", true, &mut a, &mut b);
    assert!(!contains(&drag, 0xffeedd11) && !contains(&drag, 0xffff00ff));
}

#[test]
fn record_privacy_cursor_enter_epoch_and_listed_children() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("record-privacy-cursor-epoch");
    // All windows, child focus points and cursor artwork fit this canvas.
    // Bound the matrix's debug GPU/encoding work alongside other pool tests.
    let mut process = process::spawn_size("record-privacy-cursor-epoch", &bus.url, "320x240");
    // This matrix seals dozens of takes, also under the width-eight pool load.
    process.extend_watchdog(Duration::from_secs(180));
    let mut a = Pattern::named(connect(&mut process), "cursor-listed", false);
    a.solid_buffer(0xff334455);
    a.commit(true);
    let mut b = a.another_window("cursor-private");
    b.solid_buffer(0xff667788);
    b.commit(true);
    let mut c = Pattern::named(connect(&mut process), "cursor-different", false);
    c.solid_buffer(0xff998877);
    c.commit(true);
    bus.ready();
    let ar = bus.window_for("cursor-listed", &mut [&mut a, &mut b, &mut c]);
    let mut br = bus.window_for("cursor-private", &mut [&mut a, &mut b, &mut c]);
    let mut cr = bus.window_for("cursor-different", &mut [&mut a, &mut b, &mut c]);
    // Same coordinates keep the take's default-cursor reference identical
    // across focus changes; only A belongs to the take.
    for row in [&ar, &br, &cr] {
        bus.call(
            "comp.window.place",
            place_args(row, json!({"output":"headless-0","x":20,"y":20})),
        );
    }
    let root = process.root.clone();
    let mut serial = 0;
    let mut focus = |mode: &str, a: &mut Pattern, b: &mut Pattern, c: &mut Pattern| {
        serial += 1;
        fs::write(root.join("record-input-next"), format!("{mode}:{serial}")).unwrap();
        fs::rename(root.join("record-input-next"), root.join("record-input")).unwrap();
        bus.call(
            "comp.capture.frame",
            json!({"output":"headless-0","path":root.join("epoch-focus.ppm"),"format":"ppm"}),
        );
        pump_patterns(&mut [a, b, c]);
    };
    let take = |name: &str, a: &mut Pattern, b: &mut Pattern, c: &mut Pattern| {
        eprintln!("CURSOR_EPOCH_TAKE {name}");
        let path = root.join(format!("{name}.vfl"));
        let start = bus.call("comp.record.start", json!({"request_id":name,"target":{"output":"headless-0","windows":[target(&ar)]},"cursor":true,"path":path,"max_seconds":10}));
        pump_patterns(&mut [a, b, c]);
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let s = bus.call("comp.record.status", json!({"session":start["session"]}));
            if s["frames"].as_u64().unwrap() >= 1 {
                break;
            }
            assert!(Instant::now() < deadline, "cursor take first frame: {s}");
        }
        let stop = bus.call("comp.record.stop", json!({"session":start["session"]}));
        assert_eq!(stop["error"], Value::Null, "{stop}");
        let mut frames = Vec::new();
        vfl_visit(&path, |p| frames.push(p.to_vec()));
        assert!(!frames.is_empty());
        frames
    };
    let all_equal = |frames: &[Vec<u8>], expected: &[u8], label: &str| {
        for (i, frame) in frames.iter().enumerate() {
            assert!(
                frame.as_slice() == expected,
                "{label}, decoded frame {i}: length={}, expected={}, first difference={:?}",
                frame.len(),
                expected.len(),
                frame.iter().zip(expected).position(|(a, b)| a != b)
            );
        }
    };
    let live = |name: &str| {
        let path = root.join(format!("{name}.ppm"));
        bus.call(
            "comp.capture.frame",
            json!({"output":"headless-0","path":path,"format":"ppm"}),
        );
        fs::read(path).unwrap()
    };
    focus("listed", &mut a, &mut b, &mut c);
    let default = take("epoch-default", &mut a, &mut b, &mut c).remove(0);
    // Exercise Surface, Hidden and Named from same-client and different-client
    // private windows, followed by a listed enter with no cursor response.
    for different in [false, true] {
        for shape in ["surface", "hidden", "named"] {
            let private = if different { "different" } else { "private" };
            focus(private, &mut a, &mut b, &mut c);
            let live_default = live("epoch-private-default");
            let source = if different { &mut c } else { &mut b };
            let _cursor = match shape {
                "surface" => Some(source.custom_cursor(0xffff00ff)),
                "hidden" => {
                    source.hide_cursor();
                    None
                }
                _ => {
                    source.named_cursor();
                    None
                }
            };
            pump_patterns(&mut [&mut a, &mut b, &mut c]);
            if shape == "surface" {
                assert!(
                    live("epoch-private-live")
                        .windows(3)
                        .any(|p| p == rgb(0xffff00ff))
                );
            } else if shape == "named" {
                assert!(
                    live("epoch-named-live") != live_default,
                    "Named fixture must change the live cursor"
                );
            }
            all_equal(
                &take(
                    &format!("epoch-denied-{different}-{shape}"),
                    &mut a,
                    &mut b,
                    &mut c,
                ),
                &default,
                "private state",
            );
            focus("listed", &mut a, &mut b, &mut c);
            all_equal(
                &take(
                    &format!("epoch-return-{different}-{shape}"),
                    &mut a,
                    &mut b,
                    &mut c,
                ),
                &default,
                "new enter must use default",
            );
            let _own = a.custom_cursor(0xff11eecc);
            pump_patterns(&mut [&mut a, &mut b, &mut c]);
            let own = take(
                &format!("epoch-own-{different}-{shape}"),
                &mut a,
                &mut b,
                &mut c,
            );
            assert_ne!(own[0], default, "new listed cursor must be honoured");
            assert!(
                own[0]
                    .chunks_exact(4)
                    .any(|p| p == 0xff11eecc_u32.to_le_bytes())
            );
            all_equal(&own, &own[0], "listed custom cursor");
        }
    }
    // A forced compositor shape must not disclose private focus metadata.
    focus("private-forced", &mut a, &mut b, &mut c);
    assert_ne!(
        live("epoch-forced-live"),
        {
            focus("private", &mut a, &mut b, &mut c);
            live("epoch-not-forced-live")
        },
        "forced shape positive control"
    );
    focus("private-forced", &mut a, &mut b, &mut c);
    all_equal(
        &take("epoch-forced-denied", &mut a, &mut b, &mut c),
        &default,
        "forced private cursor",
    );
    // Unmapping the overlapping private window enters A without A setting
    // anything. Test both client identities and all three cursor states.
    for different in [false, true] {
        for shape in ["surface", "hidden", "named"] {
            bus.call(
                "comp.window.place",
                place_args(
                    &br,
                    json!({"output":"headless-0","x":if different {120}else{20},"y":20}),
                ),
            );
            bus.call(
                "comp.window.place",
                place_args(
                    &cr,
                    json!({"output":"headless-0","x":if different {20}else{120},"y":20}),
                ),
            );
            focus("overlap", &mut a, &mut b, &mut c);
            let source = if different { &mut c } else { &mut b };
            assert_eq!(
                source.state.pointer_focus.as_ref(),
                Some(&source.surface),
                "overlapping private window must own focus before unmapping"
            );
            let _cursor = match shape {
                "surface" => Some(source.custom_cursor(0xffff00ff)),
                "hidden" => {
                    source.hide_cursor();
                    None
                }
                _ => {
                    source.named_cursor();
                    None
                }
            };
            pump_patterns(&mut [&mut a, &mut b, &mut c]);
            if different {
                c.unmap();
            } else {
                b.unmap();
            }
            pump_patterns(&mut [&mut a, &mut b, &mut c]);
            assert_eq!(
                a.state.pointer_focus.as_ref(),
                Some(&a.surface),
                "unmapping private window must enter A"
            );
            all_equal(
                &take(
                    &format!("epoch-unmap-{different}-{shape}"),
                    &mut a,
                    &mut b,
                    &mut c,
                ),
                &default,
                "unmap enter",
            );
            if different {
                c.destroy_window();
                c = Pattern::named(connect(&mut process), "cursor-different", false);
                c.solid_buffer(0xff998877);
                c.commit(true);
                cr = bus.window_for("cursor-different", &mut [&mut a, &mut b, &mut c]);
            } else {
                b.destroy_window();
                b = a.another_window("cursor-private");
                b.solid_buffer(0xff667788);
                b.commit(true);
                br = bus.window_for("cursor-private", &mut [&mut a, &mut b, &mut c]);
            }
            pump_patterns(&mut [&mut a, &mut b, &mut c]);
        }
    }
    // Confirm the real popup/subsurface hit and the allow direction with a
    // newly set cursor on that exact focused child.
    focus("listed", &mut a, &mut b, &mut c);
    let _popup = a.popup(false);
    let _sub = a.subsurface();
    for child in ["listed-popup", "listed-sub"] {
        focus(child, &mut a, &mut b, &mut c);
        assert_eq!(
            a.state.pointer_focus.as_ref(),
            Some(if child == "listed-popup" {
                &_popup.0
            } else {
                &_sub.0
            }),
            "child must receive the enter event"
        );
        let child_default =
            take(&format!("epoch-{child}-default"), &mut a, &mut b, &mut c).remove(0);
        let _cursor = a.custom_cursor(0xff11eecc);
        pump_patterns(&mut [&mut a, &mut b, &mut c]);
        let custom = take(&format!("epoch-{child}-own"), &mut a, &mut b, &mut c);
        assert_ne!(custom[0], child_default, "child cursor must be authorised");
        assert!(
            custom[0]
                .chunks_exact(4)
                .any(|p| p == 0xff11eecc_u32.to_le_bytes())
        );
        all_equal(&custom, &custom[0], "child cursor");
    }
    // A drag source keeps authority to set its cursor during the grab. It
    // must not transfer that authority to the listed drop target's take.
    bus.call(
        "comp.window.place",
        place_args(&cr, json!({"output":"headless-0","x":20,"y":20})),
    );
    focus("different", &mut a, &mut b, &mut c);
    let _cursor = c.custom_cursor(0xffff00ff);
    let _drag = c.drag_icon(0xffeedd11);
    pump_patterns(&mut [&mut a, &mut b, &mut c]);
    focus("drag-listed", &mut a, &mut b, &mut c);
    let _cursor = c.custom_cursor(0xffff00ff);
    pump_patterns(&mut [&mut a, &mut b, &mut c]);
    let drag_live = live("epoch-drag-live");
    assert!(
        drag_live.windows(3).any(|p| p == rgb(0xffeedd11)),
        "drag must remain active over listed target"
    );
    // Popup/subsurface pixels changed A's reference; establish the expected
    // default at the same pointer position before comparing all decoded bytes.
    focus("listed", &mut a, &mut b, &mut c);
    let drag_default = take("epoch-after-drag-default", &mut a, &mut b, &mut c).remove(0);
    focus("different", &mut a, &mut b, &mut c);
    let _cursor = c.custom_cursor(0xffff00ff);
    let _drag = c.drag_icon(0xffeedd11);
    pump_patterns(&mut [&mut a, &mut b, &mut c]);
    focus("drag-listed", &mut a, &mut b, &mut c);
    let _cursor = c.custom_cursor(0xffff00ff);
    pump_patterns(&mut [&mut a, &mut b, &mut c]);
    all_equal(
        &take("epoch-drag-denied", &mut a, &mut b, &mut c),
        &drag_default,
        "drag cursor and icon",
    );
}

#[test]
fn record_privacy_window_output_membership_and_marker_order() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("record-privacy-output");
    let mut process = process::spawn(1, None, "record-privacy-output", Some(&bus.url));
    let mut a = Pattern::named(connect(&mut process), "output-listed", false);
    a.solid_buffer(0xff11eecc);
    a.commit(true);
    bus.ready();
    let ar = bus.window_for("output-listed", &mut [&mut a]);
    bus.call(
        "comp.window.place",
        place_args(&ar, json!({"output":"headless-1","x":100,"y":100})),
    );
    let path = process.root.join("off-output.vfl");
    let refused = bus.call_rc("comp.record.start",json!({"request_id":"off","target":{"output":"headless-0","windows":[target(&ar)]},"path":path,"max_seconds":10}),10);
    assert_eq!(refused["error"], "window_not_on_output");
    assert_eq!(refused["id"], target(&ar)["id"]);
    assert!(!path.exists());
    bus.call(
        "comp.window.place",
        place_args(&ar, json!({"output":"headless-0","x":100,"y":100})),
    );
    let path = process.root.join("left-output.vfl");
    let start=bus.call("comp.record.start",json!({"request_id":"left","target":{"output":"headless-0","windows":[target(&ar)]},"cursor":false,"path":path,"max_seconds":10}));
    pump_patterns(&mut [&mut a]);
    bus.call(
        "comp.window.place",
        place_args(&ar, json!({"output":"headless-0","x":3000,"y":100})),
    );
    let status = bus.call("comp.record.status", json!({"session":start["session"]}));
    assert_eq!(
        status["windows"],
        json!([target(&ar)]),
        "out of view retains identity membership"
    );
    let marker_path = Path::new(start["recovery_path"].as_str().unwrap()).join("markers.jsonl");
    let until = Instant::now() + Duration::from_secs(5);
    let outside_frames = loop {
        let left = fs::read_to_string(&marker_path)
            .unwrap_or_default()
            .split_inclusive('\n')
            .filter(|line| line.ends_with('\n'))
            .map(|line| serde_json::from_str::<Value>(line).expect("complete marker row"))
            .find(|m| m["label"] == "window_left");
        let status = bus.call("comp.record.status", json!({"session":start["session"]}));
        let frames = status["frames"].as_u64().unwrap();
        if left.is_some_and(|m| frames > m["frame"].as_u64().unwrap() + 2) {
            break frames;
        }
        assert!(
            Instant::now() < until,
            "no written off-output marker/ticks: {status}"
        );
        thread::sleep(Duration::from_millis(20));
    };
    bus.call(
        "comp.window.place",
        place_args(&ar, json!({"output":"headless-0","x":100,"y":100})),
    );
    pump_patterns(&mut [&mut a]);
    // Marker publication follows the returned-state image in the writer FIFO.
    // Do not close the window until that image has actually been written.
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        let status = bus.call("comp.record.status", json!({"session":start["session"]}));
        let returned = fs::read_to_string(&marker_path)
            .unwrap_or_default()
            .split_inclusive('\n')
            .filter(|line| line.ends_with('\n'))
            .map(|line| serde_json::from_str::<Value>(line).expect("complete marker row"))
            .any(|m| m["label"] == "window_returned");
        if returned {
            break;
        }
        assert!(Instant::now() < until, "no written return marker: {status}");
        thread::sleep(Duration::from_millis(20));
    }
    drop(a);
    // A fixed delay can stop before the marker is admitted under load. Wait
    // for the written marker and a subsequent clock slot, so the decoded
    // post-marker tail assertion below observes a real quiet tick.
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        let gone = fs::read_to_string(&marker_path)
            .unwrap_or_default()
            .split_inclusive('\n')
            .filter(|line| line.ends_with('\n'))
            .map(|line| serde_json::from_str::<Value>(line).expect("complete marker row"))
            .find(|m| m["label"] == "window_gone");
        let status = bus.call("comp.record.status", json!({"session":start["session"]}));
        if gone
            .is_some_and(|m| status["frames"].as_u64().unwrap() > m["frame"].as_u64().unwrap() + 1)
        {
            break;
        }
        assert!(
            Instant::now() < until,
            "no post-close marker/tick: {status}"
        );
        thread::sleep(Duration::from_millis(20));
    }
    let stop = bus.call("comp.record.stop", json!({"session":start["session"]}));
    assert_eq!(stop["error"], Value::Null);
    assert!(stop["repeated"].as_u64().unwrap() > 0);
    let marker: Value = fs::read_to_string(path.join("markers.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str::<Value>(l).unwrap())
        .find(|m| m["label"] == "window_gone")
        .unwrap();
    let markers: Vec<Value> = fs::read_to_string(path.join("markers.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let left = markers
        .iter()
        .find(|m| m["label"] == "window_left")
        .unwrap()["frame"]
        .as_u64()
        .unwrap();
    let returned = markers
        .iter()
        .find(|m| m["label"] == "window_returned")
        .unwrap()["frame"]
        .as_u64()
        .unwrap();
    let mut reader = media_vfl::reader::Reader::open(&path).unwrap();
    let mut saw_return = false;
    assert!(outside_frames > left, "need stable off-output frames");
    assert!(left < returned && returned < marker["frame"].as_u64().unwrap());
    while let Some(f) = reader.read().unwrap() {
        let present = f
            .pixels
            .chunks_exact(4)
            .any(|p| p == 0xff11eecc_u32.to_le_bytes());
        if f.slot >= left && f.slot < outside_frames {
            assert!(!present);
        }
        if f.slot >= returned {
            saw_return |= present;
        }
    }
    assert!(saw_return);
    let frame = marker["frame"].as_u64().unwrap();
    let mut index = 0;
    let mut saw = false;
    vfl_visit(&path, |pixels| {
        let present = pixels
            .chunks_exact(4)
            .any(|p| p == 0xff11eecc_u32.to_le_bytes());
        saw |= present;
        if index > frame {
            assert!(!present, "old window frame followed its marker");
        }
        index += 1;
    });
    assert!(
        saw && index > frame + 1,
        "take must show initial window then a cleared tail"
    );
}

#[test]
fn record_privacy_lock_is_blank_and_stops_callbacks() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("record-lock");
    let mut process = process::spawn(1, None, "record-lock", Some(&bus.url));
    let mut a = Pattern::named(connect(&mut process), "lock-listed", false);
    a.solid_buffer(0xff11eecc);
    a.commit(true);
    bus.ready();
    let ar = bus.window_for("lock-listed", &mut [&mut a]);
    let path = process.root.join("lock.vfl");
    let start = bus.call("comp.record.start", json!({"request_id":"lock","target":{"output":"headless-0","windows":[target(&ar)]},"path":path,"max_seconds":20,"cursor":true}));
    let log = process.root.join("lock.log");
    let file = fs::File::create(&log).unwrap();
    let mut lock = Command::new(binary("testkit-lock-client"))
        .args(["--hold", "10"])
        .env("XDG_RUNTIME_DIR", &process.root)
        .env("WAYLAND_DISPLAY", "wayland-headless-test")
        .stdout(std::process::Stdio::from(file.try_clone().unwrap()))
        .stderr(file)
        .spawn()
        .unwrap();
    let wait = |text: &str| {
        let until = Instant::now() + Duration::from_secs(5);
        while !fs::read_to_string(&log).unwrap().contains(text) {
            assert!(Instant::now() < until, "lock client did not report {text}");
            thread::sleep(Duration::from_millis(10));
        }
    };
    let marker_path = Path::new(start["recovery_path"].as_str().unwrap()).join("markers.jsonl");
    let wait_recorded = |label: &str| {
        let until = Instant::now() + Duration::from_secs(5);
        loop {
            let status = bus.call("comp.record.status", json!({"session":start["session"]}));
            let marker = fs::read_to_string(&marker_path)
                .unwrap_or_default()
                .split_inclusive('\n')
                .filter(|line| line.ends_with('\n'))
                .map(|line| serde_json::from_str::<Value>(line).expect("complete marker row"))
                .find(|m| m["label"] == label);
            if marker.is_some_and(|m| {
                status["frames"].as_u64().unwrap() > m["frame"].as_u64().unwrap() + 1
            }) {
                break;
            }
            assert!(
                Instant::now() < until,
                "no written {label} marker and subsequent tick: {status}"
            );
            thread::sleep(Duration::from_millis(20));
        }
    };
    wait("locked");
    wait_recorded("locked");
    pump_patterns(&mut [&mut a]);
    let before = a.state.callbacks;
    a.commit(true);
    let until = Instant::now() + Duration::from_millis(500);
    while Instant::now() < until {
        a.pump();
        thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(
        a.state.callbacks, before,
        "listed window got callbacks while locked"
    );
    let quiet = bus.call("comp.record.status", json!({"session":start["session"]}));
    thread::sleep(Duration::from_millis(200));
    let quiet2 = bus.call("comp.record.status", json!({"session":start["session"]}));
    assert_eq!(quiet["readbacks"], quiet2["readbacks"]);
    assert!(quiet2["repeated"].as_u64() > quiet["repeated"].as_u64());
    assert_eq!(unsafe { libc::kill(lock.id() as i32, libc::SIGUSR1) }, 0);
    wait("unlocked");
    assert!(lock.wait().unwrap().success());
    pump_patterns(&mut [&mut a]);
    // The marker points at the transition's clock boundary. Keep recording
    // until a later slot exists for the post-unlock decoded-pixel assertion.
    wait_recorded("unlocked");
    let stop = bus.call("comp.record.stop", json!({"session":start["session"]}));
    assert_eq!(stop["error"], Value::Null);
    let markers: Vec<Value> = fs::read_to_string(path.join("markers.jsonl"))
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let locked = markers.iter().find(|m| m["label"] == "locked").unwrap()["frame"]
        .as_u64()
        .unwrap();
    let unlocked = markers.iter().find(|m| m["label"] == "unlocked").unwrap()["frame"]
        .as_u64()
        .unwrap();
    let mut reader = media_vfl::reader::Reader::open(&path).unwrap();
    let mut blank = 0;
    let mut returned = false;
    while let Some(f) = reader.read().unwrap() {
        if f.slot >= locked && f.slot < quiet2["frames"].as_u64().unwrap() {
            assert!(
                f.pixels.chunks_exact(4).all(|p| p == [0, 0, 0, 255]),
                "locked frame leaked content"
            );
            blank += 1;
        }
        if f.slot >= unlocked {
            returned |= f
                .pixels
                .chunks_exact(4)
                .any(|p| p == 0xff11eecc_u32.to_le_bytes());
        }
    }
    assert!(
        blank > 0 && returned,
        "blank={blank}, returned={returned}, locked={locked}, unlocked={unlocked}, stop={stop}"
    );
}

#[test]
fn record_privacy_generation_change_and_start_refusals() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("record-generation");
    let mut process = process::spawn(1, None, "record-generation", Some(&bus.url));
    let mut a = Pattern::named(connect(&mut process), "generation-listed", false);
    a.solid_buffer(0xff11eecc);
    a.commit(true);
    bus.ready();
    let ar = bus.window_for("generation-listed", &mut [&mut a]);
    let path = process.root.join("generation.vfl");
    let start = bus.call("comp.record.start",json!({"request_id":"generation","target":{"output":"headless-0","windows":[target(&ar)]},"path":path,"cursor":false,"max_seconds":10}));
    a.solid_buffer(0xffee9911);
    a.retake_role("generation-listed");
    let successor = bus.window_for("generation-listed", &mut [&mut a]);
    assert_eq!(successor["id"], ar["id"]);
    assert_ne!(successor["generation"], ar["generation"]);
    thread::sleep(Duration::from_millis(250));
    assert_eq!(
        bus.call("comp.record.status", json!({"session":start["session"]}))["windows"],
        json!([])
    );
    let stop = bus.call("comp.record.stop", json!({"session":start["session"]}));
    assert_eq!(stop["error"], Value::Null);
    let mut saw = false;
    let mut last = false;
    vfl_visit(&path, |pixels| {
        last = pixels
            .chunks_exact(4)
            .any(|p| p == 0xff11eecc_u32.to_le_bytes());
        saw |= last;
        assert!(
            !pixels
                .chunks_exact(4)
                .any(|p| p == 0xffee9911_u32.to_le_bytes())
        );
    });
    assert!(saw && !last);
    let markers = fs::read_to_string(path.join("markers.jsonl")).unwrap();
    assert!(
        markers
            .lines()
            .map(|l| serde_json::from_str::<Value>(l).unwrap())
            .any(|m| m["label"] == "window_gone" && m["data"] == target(&ar))
    );
    let args = |row: &Value, id: &str| json!({"request_id":id,"target":{"output":"headless-0","windows":[target(row)]},"path":process.root.join(format!("{id}.vfl")),"max_seconds":1});
    let stale = bus.call_rc("comp.record.start", args(&ar, "stale"), 10);
    assert_eq!(stale["error"], "stale_target");
    assert_eq!(stale["id"], ar["id"]);
    drop(a);
    thread::sleep(Duration::from_millis(150));
    let closed = bus.call_rc("comp.record.start", args(&successor, "closed"), 10);
    assert_eq!(closed["error"], "unknown_window");
    assert_eq!(closed["id"], ar["id"]);
}

#[test]
fn record_privacy_missing_unchanged_answer_fails_without_pending_drain() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("record-privacy-missing");
    let mut process = process::spawn(1, None, "record-privacy-missing", Some(&bus.url));
    let mut a = Pattern::named(connect(&mut process), "missing-listed", false);
    a.solid_buffer(0xff11eecc);
    a.commit(true);
    bus.ready();
    let ar = bus.window_for("missing-listed", &mut [&mut a]);
    let path = process.root.join("missing.vfl");
    let start=bus.call("comp.record.start",json!({"request_id":"missing","target":{"output":"headless-0","windows":[target(&ar)]},"cursor":false,"path":path,"max_seconds":10}));
    thread::sleep(Duration::from_millis(150));
    fs::write(process.root.join("record-input"), "missing-answer").unwrap();
    let began = Instant::now();
    loop {
        let status = bus.call("comp.record.status", json!({"session":start["session"]}));
        if status["state"] == "failed" {
            assert!(
                status["error"]
                    .as_str()
                    .unwrap()
                    .contains("unchanged listed scene has no raw answer")
            );
            break;
        }
        assert!(
            began.elapsed() < Duration::from_secs(3),
            "missing callback left a pending take: {status}"
        );
        thread::sleep(Duration::from_millis(20));
    }
    let stop = bus.call("comp.record.stop", json!({"session":start["session"]}));
    assert!(!path.exists());
    let recovery = Path::new(stop["recovery_path"].as_str().unwrap());
    let mut found = false;
    let (_, frames) = vfl_scan(
        recovery,
        |pixels| {
            found |= pixels
                .chunks_exact(4)
                .any(|p| p == 0xff11eecc_u32.to_le_bytes());
        },
        false,
    )
    .unwrap();
    assert!(
        found && frames > 0,
        "failed take retains decoded admitted pixels"
    );
    assert!(!recovery.join("complete.json").exists());
}

#[test]
fn record_4k_output_windows_pointer_and_damage_repeat() {
    record_4k();
}

/// Print submission throughput on an idle render worker:
/// cargo test -p headless --test capture benchmark_record_4k_throughput -- --ignored --nocapture
#[test]
#[ignore = "throughput measurement; run alone on an idle render worker"]
fn benchmark_record_4k_throughput() {
    record_4k();
}

#[test]
#[ignore = "10-minute generated static recording cost; run alone on a render worker"]
fn benchmark_record_ui_static_10min() {
    assert!(process::record_available());
    let bus = Bus::start("vfl-static-cost");
    let mut process = process::spawn_4k("vfl-static-cost", &bus.url);
    process.extend_watchdog(Duration::from_secs(660));
    let mut p = Pattern::new(connect(&mut process));
    p.wait_callback(0);
    bus.ready();
    let measure = |begin: u64, end: u64| -> (u64, u64, usize) {
        let trace = fs::read_to_string(process.root.join("frames.log")).unwrap();
        let field = |line: &str, key: &str| -> u64 {
            line.split_whitespace()
                .find_map(|w| w.strip_prefix(key)?.parse().ok())
                .unwrap()
        };
        let mut times: Vec<u64> = trace
            .lines()
            .filter(|l| l.contains("stage=headless_frame_cpu "))
            .filter(|l| (begin..end).contains(&field(l, "start_us=")))
            .map(|l| field(l, "duration_us="))
            .collect();
        assert!(!times.is_empty());
        times.sort_unstable();
        (
            times[times.len() / 2],
            times[times.len() * 95 / 100],
            times.len(),
        )
    };
    let begin = monotonic_ns() / 1000;
    let baseline = Instant::now();
    let mut redraw_at = Duration::ZERO;
    while baseline.elapsed() < Duration::from_secs(10) {
        if baseline.elapsed() >= redraw_at {
            // Unchanged pixels, one controlled redraw per second, equally
            // in both phases. An idle output otherwise has no CPU samples.
            p.commit(true);
            redraw_at += Duration::from_secs(1);
        }
        p.pump();
        thread::sleep(Duration::from_millis(5));
    }
    let before = measure(begin, monotonic_ns() / 1000);
    let start=bus.call("comp.record.start",json!({"request_id":"static-cost","target":{"output":"headless-0"},"path":process.root.join("static.vfl"),"max_seconds":600,"fps":30}));
    let begin = monotonic_ns() / 1000;
    let take = Instant::now();
    let mut redraw_at = Duration::ZERO;
    while take.elapsed() < Duration::from_secs(600) {
        if take.elapsed() >= redraw_at {
            p.commit(true);
            redraw_at += Duration::from_secs(1);
        }
        p.pump();
        thread::sleep(Duration::from_millis(5));
    }
    let end = monotonic_ns() / 1000;
    let stop = bus.call("comp.record.stop", json!({"session":start["session"]}));
    let during = measure(begin, end);
    assert_eq!(stop["error"], Value::Null, "{stop}");
    println!(
        "VFL_STATIC_10MIN generated_static_proxy=true baseline_median_us={} baseline_p95_us={} record_median_us={} record_p95_us={} delta_median_us={} bytes={} dropped={} stats={stop}",
        before.0,
        before.1,
        during.0,
        during.1,
        during.0 as i64 - before.0 as i64,
        stop["bytes"],
        stop["dropped"]
    );
    assert_eq!(stop["dropped"], 0, "{stop}");
    assert!(during.0 as i64 - before.0 as i64 <= 2000);
}

fn record_4k() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("record-4k");
    let mut process = process::spawn_4k("record-4k", &bus.url);
    let mut a = Pattern::named(connect(&mut process), "record-a", false);
    let mut b = Pattern::named(connect(&mut process), "record-b", false);
    bus.ready();
    pump_patterns(&mut [&mut a, &mut b]);
    for (name, x) in [("record-a", 100), ("record-b", 600)] {
        let row = bus.window_for(name, &mut [&mut a, &mut b]);
        bus.call(
            "comp.window.place",
            place_args(&row, json!({"x":x,"y":100})),
        );
    }
    bus.call(
        "comp.input.pointer.move",
        json!({"seat":"agent","output":"headless-0","x":1000,"y":500}),
    );
    pump_patterns(&mut [&mut a, &mut b]);
    let path = process.root.join("4k.vfl");
    let args = json!({"request_id":"4k","target":{"output":"headless-0"},"cursor":true,"path":path,"fps":30,"max_seconds":10});
    let start = bus.call("comp.record.start", args);
    assert_eq!(start["width"], 3840);
    assert_eq!(start["height"], 2160);
    let session = start["session"].clone();
    let before = bus.call("comp.record.status", json!({"session":session}));
    for _ in 0..50 {
        a.pump();
        b.pump();
        thread::sleep(Duration::from_millis(20));
    }
    let idle = bus.call("comp.record.status", json!({"session":session}));
    assert!(idle["repeated"].as_u64().unwrap() >= 20, "{idle}");
    assert!(
        idle["readbacks"].as_u64().unwrap() - before["readbacks"].as_u64().unwrap() <= 4,
        "idle output kept reading back: {before} -> {idle}"
    );
    assert!(idle["readback_mean_us"].as_u64().unwrap() > 0);
    bus.call(
        "comp.input.pointer.move",
        json!({"seat":"agent","output":"headless-0","x":1200,"y":500}),
    );
    // A debug renderer's first read can incur up to sixty ticks of budget
    // debt. Verify eventual pointer capture within that recovery bound.
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let moved = bus.call("comp.record.status", json!({"session":session}));
        if moved["readbacks"].as_u64() > idle["readbacks"].as_u64() {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "pointer damage must cause readback: {moved}"
        );
        thread::sleep(Duration::from_millis(50));
    }
    let marker = bus.call(
        "comp.record.marker",
        json!({"session":session,"label":"4k-done"}),
    );
    let stop = bus.call("comp.record.stop", json!({"session":session}));
    let manifest: Value =
        serde_json::from_slice(&fs::read(path.join("complete.json")).unwrap()).unwrap();
    assert_eq!(manifest["dropped"], stop["dropped"]);
    assert!(
        stop["repeated"].as_u64() >= stop["dropped"].as_u64(),
        "overruns must be repeated on the timeline: {stop}"
    );
    assert!(marker["frame"].as_u64() < stop["frames"].as_u64());
    let (decoded, encoded_frames) = vfl_first(&path);
    assert_eq!(decoded.len(), 3840 * 2160 * 3);
    assert_eq!(
        encoded_frames,
        stop["frames"].as_u64().unwrap(),
        "queue overruns must not shorten the encoded stream"
    );
    let at = |x: usize, y: usize| {
        let i = (y * 3840 + x) * 3;
        &decoded[i..i + 3]
    };
    // Each independent client must be visible in the whole-output recording.
    for x in [150, 650] {
        for (actual, expected) in at(x, 140).iter().zip(rgb(COLOURS[0])) {
            assert!((i16::from(*actual) - i16::from(expected)).abs() < 20);
        }
    }
    let amber = (500..524)
        .flat_map(|y| (1000..1024).map(move |x| (x, y)))
        .filter(|(x, y)| {
            let p = at(*x, *y);
            p[0] > 200 && p[1] > 90 && p[1] < 210 && p[2] < 80
        })
        .count();
    assert!(
        amber > 20,
        "the encoded output must include the pointer ({amber} amber pixels)"
    );
    let frames: Vec<Value> = fs::read_to_string(start["frames_path"].as_str().unwrap())
        .unwrap()
        .lines()
        .map(|s| serde_json::from_str(s).unwrap())
        .collect();
    // t_us is submission wall time, not the encoded CFR timeline. Scheduler
    // stalls may separate submissions without losing any encoded frames.
    assert_eq!(frames.len() as u64, encoded_frames);
    assert_eq!(frames[0]["t_us"], 0);
    assert!(frames.last().unwrap()["t_us"].as_u64().unwrap() > 0);
    for (index, frame) in frames.iter().enumerate() {
        assert_eq!(frame["frame"].as_u64(), Some(index as u64));
    }
    assert!(
        frames
            .windows(2)
            .all(|pair| { pair[0]["t_us"].as_u64().unwrap() <= pair[1]["t_us"].as_u64().unwrap() })
    );
    assert_eq!(
        stop["duration_us"].as_u64(),
        Some(encoded_frames * 1_000_000 / 30)
    );
    let duration = frames.last().unwrap()["t_us"].as_u64().unwrap() as f64 / 1_000_000.0;
    println!(
        "4K submission throughput: {} frames in {duration}s",
        frames.len()
    );
    println!("4K30: {stop}; idle status: {idle}");
}

#[test]
fn record_idle_output_lifecycle_and_clock() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("record-idle");
    let mut process = process::spawn(1, None, "record-idle", Some(&bus.url));
    drop(connect(&mut process));
    bus.ready();
    let observer = BlankObserver::start(&bus);
    let path = process.root.join("take.vfl");
    let args = json!({"request_id":"take-1","target":{"output":"headless-0"},"path":path,"fps":30,"max_seconds":10});
    fs::write(&path, "existing").unwrap();
    bus.call_rc("comp.record.start", args.clone(), 10);
    assert_eq!(fs::read_to_string(&path).unwrap(), "existing");
    fs::remove_file(&path).unwrap();
    let start = bus.call("comp.record.start", args.clone());
    assert!(
        !path.exists(),
        "final path stays absent until successful stop"
    );
    let mut changed = args.clone();
    changed["fps"] = json!(10);
    bus.call_rc("comp.record.start", changed, 10);
    assert_eq!(start, bus.call("comp.record.start", args.clone()));
    let session = start["session"].clone();
    assert_eq!(start["clock"], "monotonic");
    assert_eq!(start["width"], 1280);
    assert_eq!(start["height"], 720);
    for id in ["unknown", "stale-session", "mismatched-session"] {
        let refused = bus.call_rc("comp.record.stop", json!({"session":id}), 10);
        assert_eq!(refused["error"], "record_failed");
        let live = bus.call("comp.record.status", json!({"session":session}));
        assert_eq!(live["state"], "recording");
        assert_eq!(live["session"], session);
    }
    let busy = bus.call_rc("comp.record.start",json!({"request_id":"other","target":{"output":"headless-0"},"path":process.root.join("other.vfl"),"max_seconds":10}),10);
    assert_eq!(busy["error"], "record_busy");
    let mut markers = Vec::new();
    for index in 0..4 {
        thread::sleep(Duration::from_millis(100));
        markers.push(bus.call(
            "comp.record.marker",
            json!({"session":session,"label":format!("step-{index}"),"data":{"step":index}}),
        ));
    }
    let status = bus.call("comp.record.status", json!({"session":session}));
    assert_eq!(status["state"], "recording");
    assert!(status["repeated"].as_u64().unwrap() > 0);
    let stop = bus.call("comp.record.stop", json!({"session":session}));
    assert_eq!(
        stop,
        bus.call("comp.record.stop", json!({"session":session}))
    );
    let frames = stop["frames"].as_u64().unwrap();
    use std::os::unix::fs::PermissionsExt;
    for key in ["frames_path", "markers_path"] {
        assert_eq!(
            fs::metadata(stop[key].as_str().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
    }
    assert!(frames > 0, "{stop}");
    assert!(fs::metadata(&path).unwrap().len() > 0);
    let (_, encoded_frames) = vfl_first(&path);
    assert_eq!(encoded_frames, frames);
    let rows: Vec<Value> = fs::read_to_string(start["frames_path"].as_str().unwrap())
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(rows.len() as u64, frames);
    for (index, row) in rows.iter().enumerate() {
        assert_eq!(row["frame"], index as u64);
        assert_eq!(row["t_us"], index as u64 * 1_000_000 / 30);
    }
    for pair in markers.windows(2) {
        assert!(pair[0]["t_us"].as_u64() < pair[1]["t_us"].as_u64());
        assert!(pair[0]["seq"].as_u64() < pair[1]["seq"].as_u64());
    }
    assert!(
        markers
            .iter()
            .all(|m| m["frame"].as_u64().unwrap() < frames)
    );
    let logged: Vec<Value> = fs::read_to_string(start["markers_path"].as_str().unwrap())
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    for (reply, row) in markers.iter().zip(logged) {
        for key in ["seq", "frame", "t_us"] {
            assert_eq!(reply[key], row[key]);
        }
    }
    let mut bounded_pattern = Pattern::new(connect(&mut process));
    bounded_pattern.wait_callback(0);
    let row = bus.window(&mut bounded_pattern);
    bus.call(
        "comp.window.fullscreen",
        place_args(&row, json!({"output":"headless-0"})),
    );
    bounded_pattern.state.size = (1280, 720);
    bounded_pattern.solid_buffer(0xff103050);
    let previous = bounded_pattern.state.callbacks;
    bounded_pattern.commit(true);
    bounded_pattern.wait_callback(previous);
    let next=bus.call("comp.record.start",json!({"request_id":"bounded","target":{"output":"headless-0"},"path":process.root.join("bounded.vfl"),"max_seconds":1}));
    let mut admitted_colours = Vec::new();
    let mut submitted_colours = Vec::new();
    let at = ((720 * 3 / 4) * 1280 + 1280 * 3 / 4) * 4;
    'changes: for colour in [0xff103050, 0xff507090, 0xff90b0d0, 0xffd0f010] {
        let before = bus.call("comp.record.status", json!({"session":next["session"]}));
        if before["state"] != "recording" {
            break;
        }
        bounded_pattern.solid_buffer(colour);
        println!("BOUNDED_COMMIT colour={colour:08x} before={before}");
        let previous = bounded_pattern.state.callbacks;
        bounded_pattern.commit(true);
        submitted_colours.push(colour);
        bounded_pattern.wait_callback(previous);
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            bounded_pattern.pump();
            let status = bus.call("comp.record.status", json!({"session":next["session"]}));
            assert_eq!(status["error"], Value::Null, "{status}");
            let mut retained = None;
            // A completed readback can still meet a full writer queue. Only
            // decoded pixels acknowledge this particular commit. Scan both
            // names because automatic seal can publish between Bus replies.
            for path in [
                Path::new(next["recovery_path"].as_str().unwrap()),
                Path::new(next["path"].as_str().unwrap()),
            ] {
                let mut frame = 0;
                match vfl_scan(
                    path,
                    |pixels| {
                        if retained.is_none() && pixels[at..at + 4] == colour.to_le_bytes() {
                            retained = Some(frame);
                        }
                        frame += 1;
                    },
                    false,
                ) {
                    Ok(_) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => panic!("live VFL scan: {error}"),
                }
            }
            if status["state"] == "idle" {
                // Decide loss from the complete published take, never solely
                // from a live scan that may have reached a partial record.
                let mut frame = 0;
                vfl_scan(
                    Path::new(next["path"].as_str().unwrap()),
                    |pixels| {
                        if retained.is_none() && pixels[at..at + 4] == colour.to_le_bytes() {
                            retained = Some(frame);
                        }
                        frame += 1;
                    },
                    true,
                )
                .expect("sealed bounded VFL scan");
            }
            if let Some(frame) = retained {
                println!(
                    "BOUNDED_RETAINED colour={colour:08x} frame={frame} readbacks={} dropped={}",
                    status["readbacks"], status["dropped"]
                );
                admitted_colours.push(colour);
                break;
            }
            // The one-second take may seal before sampling/retaining a late
            // commit. A readback counter alone cannot promise its pixels.
            if status["state"] == "idle" {
                if status["readbacks"].as_u64().unwrap() > before["readbacks"].as_u64().unwrap() {
                    assert!(
                        status["dropped"].as_u64().unwrap() >= 1,
                        "read-back colour {colour:08x} lost without a drop: {status}"
                    );
                }
                println!("BOUNDED_SEALED colour={colour:08x} not_retained status={status}");
                break 'changes;
            }
            assert!(
                Instant::now() < deadline,
                "bounded writer never retained colour {colour:08x}: {status}"
            );
            thread::sleep(Duration::from_millis(10));
        }
    }
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        let status = bus.call("comp.record.status", json!({"session":next["session"]}));
        if status["state"] == "idle" {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "max_seconds did not finish the take: {status}"
        );
        thread::sleep(Duration::from_millis(20));
    }
    let bounded = bus.call("comp.record.stop", json!({"session":next["session"]}));
    let records = assert_recorded_colours(
        std::path::Path::new(next["path"].as_str().unwrap()),
        &admitted_colours,
    );
    assert_eq!(Some(records), bounded["frames"].as_u64());
    assert_eq!(bounded["error"], Value::Null, "{bounded}");
    // Bounded queue drops under parallel CI load are reported, not a promise
    // that every submitted state is retained. Zero-drop throughput is tested
    // separately by the isolated strict 4K30 acceptance.
    assert_eq!(
        bounded["dropped"].as_u64().unwrap(),
        bounded["dropped_capture"].as_u64().unwrap() + bounded["dropped_queue"].as_u64().unwrap()
    );
    let mut previous = 0;
    let mut final_colour = 0;
    vfl_visit(Path::new(next["path"].as_str().unwrap()), |pixels| {
        let colour = u32::from_le_bytes(pixels[at..at + 4].try_into().unwrap());
        let index = submitted_colours
            .iter()
            .position(|c| *c == colour)
            .expect("unknown recorded colour");
        assert!(index >= previous, "recorded colours went backwards");
        previous = index;
        final_colour = colour;
    });
    assert_eq!(Some(&final_colour), admitted_colours.last());
    assert!(
        records >= admitted_colours.len() as u64,
        "bounded take lost admitted slots: {bounded}"
    );
    assert_eq!(start, bus.call("comp.record.start", args));
    let discard_path = process.root.join("discard.vfl");
    let discard = bus.call("comp.record.start", json!({"request_id":"discard", "target":{"output":"headless-0"},"path":discard_path,"max_seconds":1}));
    let discarded = bus.call(
        "comp.record.stop",
        json!({"session":discard["session"], "discard":true}),
    );
    assert_eq!(discarded["frames_path"], Value::Null);
    assert!(!discard_path.exists());
    assert!(!fs::read_dir(&process.root).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .contains(".part")
    }));
    assert_eq!(
        stop,
        bus.call("comp.record.stop", json!({"session":session}))
    );
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let reply = observer.request(&bus, "observer.record_states");
        let states: Vec<_> = reply["body"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["session"] == session)
            .map(|row| row["state"].as_str().unwrap())
            .collect();
        if states == ["recording", "finishing", "idle"] {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "record.state publications: {reply}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn record_window_target_gone_fails() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("record-gone");
    let mut process = process::spawn(1, None, "record-gone", Some(&bus.url));
    let mut pattern = Pattern::new(connect(&mut process));
    bus.ready();
    pattern.wait_callback(0);
    let window = bus.window(&mut pattern);
    let start=bus.call("comp.record.start",json!({"request_id":"gone","target":{"windows":[target(&window)]},"path":process.root.join("gone.vfl"),"max_seconds":10}));
    assert!(start["width"].as_u64().unwrap() > 0);
    drop(pattern);
    thread::sleep(Duration::from_millis(250));
    let status = bus.call("comp.record.status", json!({"session":start["session"]}));
    assert_eq!(status["state"], "failed", "{status}");
    assert!(status["error"].as_str().unwrap().contains("gone"));
    let stop = bus.call("comp.record.stop", json!({"session":start["session"]}));
    assert_eq!(stop["frames_path"], Value::Null);
    assert!(!Path::new(start["path"].as_str().unwrap()).exists());
    let recovery = Path::new(stop["recovery_path"].as_str().unwrap());
    assert!(!recovery.join("complete.json").exists());
    let interrupted: Value =
        serde_json::from_slice(&fs::read(recovery.join("interrupted.json")).unwrap()).unwrap();
    assert_eq!(interrupted["complete"], false);
    assert_eq!(interrupted["dropped"], stop["dropped"]);
    assert!(interrupted["error"].as_str().unwrap().contains("gone"));
}
#[test]
fn inactive_seat_maps_clients_and_captures_output_and_window() {
    if !inactive_fixture_available() {
        return;
    }
    let bus = Bus::start("inactive-seat");
    let mut process = process::spawn_inactive("inactive-seat", &bus.url);
    let mut pattern = Pattern::new(connect(&mut process));
    bus.ready();
    pattern.wait_callback(0);
    let window = bus.window(&mut pattern);
    let session = bus.call("comp.session.get", json!({}));
    assert_eq!(session["active"], false, "{session}");
    assert_eq!(session["render_state"], "offscreen", "{session}");
    for cursor in [false, true] {
        for window in [None, Some(target(&window))] {
            let path = process
                .root
                .join(format!("inactive-{cursor}-{}.ppm", window.is_some()));
            let mut args = json!({"path":path,"format":"ppm","cursor":cursor});
            if let Some(window) = window {
                args["window"] = window;
            } else {
                args["output"] = json!("headless-0");
            }
            let reply = bus.call("comp.capture.frame", args);
            assert_eq!(reply["source"], "offscreen", "{reply}");
            assert!(fs::read(path).unwrap().starts_with(b"P6\n"));
        }
    }
    assert_ne!(
        fs::read(process.root.join("inactive-false-false.ppm")).unwrap(),
        fs::read(process.root.join("inactive-true-false.ppm")).unwrap(),
        "cursor:true must draw the retained pointer while the seat is inactive"
    );
}

fn inactive_fixture_available() -> bool {
    let node = process::render_node();
    let ready = Path::new(&node).exists()
        && binary("viud").is_file()
        && binary("noded").is_file()
        && std::env::var_os("MIX_TEST_MIX_EXE").is_some_and(|path| Path::new(&path).is_file());
    if !ready {
        eprintln!(
            "inactive-seat GPU fixture skipped: build viud/backend-native, noded and Mix; set MIX_TEST_MIX_EXE and provide a render node"
        );
    }
    ready
}

#[test]
fn inactive_seat_transitions_transfer_renderer_and_keep_captures_owned() {
    if !inactive_fixture_available() {
        return;
    }
    let bus = Bus::start("inactive-transitions");
    let mut process = process::spawn_transitions("inactive-transitions", &bus.url);
    process.extend_watchdog(Duration::from_secs(180));
    let mut pattern = Pattern::new(connect(&mut process));
    bus.ready();
    pattern.wait_callback(0);
    let window = bus.window(&mut pattern);
    let control = process.control.as_ref().unwrap().try_clone().unwrap();
    let edge = |command| {
        control.send(&[command]).unwrap();
        let mut ack = [0];
        control.recv(&mut ack).unwrap();
        assert_eq!(ack, [command]);
    };
    let before = ppm(&bus, &process, "owned-before", None);
    let start = Instant::now();
    edge(b'A'); // first two fake seat opens fail, then retry without another edge
    assert!(start.elapsed() >= Duration::from_millis(650));
    assert_eq!(bus.call("comp.session.get", json!({}))["active"], true);
    ppm(&bus, &process, "owned-active", None);
    bus.call(
        "comp.window.fullscreen",
        place_args(&window, json!({"output":"headless-0"})),
    );
    pattern.state.size = (1280, 720);
    pattern.solid_buffer(0xff103050);
    let previous = pattern.state.callbacks;
    pattern.commit(true);
    pattern.wait_callback(previous);
    let recording = bus.call(
        "comp.record.start",
        json!({
            "request_id":"pause-resume", "target":{"output":"headless-0"},
            "path":process.root.join("pause-resume.vfl"), "max_seconds":120, "cursor":false
        }),
    );
    let mut colours = vec![0xff103050];
    for index in 0..3 {
        let before = bus.call(
            "comp.record.status",
            json!({"session":recording["session"]}),
        );
        assert_eq!(
            before["state"], "recording",
            "pause take sealed before its explicit stop: {before}"
        );
        edge(b'P');
        assert_eq!(bus.call("comp.session.get", json!({}))["active"], false);
        thread::sleep(Duration::from_millis(1100));
        let paused_before = bus.call(
            "comp.record.status",
            json!({"session":recording["session"]}),
        );
        let colour = 0xff507090 + index * 0x00101010;
        pattern.solid_buffer(colour);
        pattern.commit(true);
        ppm(&bus, &process, &format!("paused-{index}"), None);
        ppm(
            &bus,
            &process,
            &format!("window-paused-{index}"),
            Some(target(&window)),
        );
        let until = Instant::now() + Duration::from_secs(10);
        loop {
            let status = bus.call(
                "comp.record.status",
                json!({"session":recording["session"]}),
            );
            assert_eq!(
                status["state"], "recording",
                "pause take unexpectedly sealed: {status}"
            );
            if status["readbacks"].as_u64().unwrap() > paused_before["readbacks"].as_u64().unwrap()
            {
                break;
            }
            assert!(
                Instant::now() < until,
                "paused change never sampled: {status}"
            );
            pattern.pump();
            thread::sleep(Duration::from_millis(10));
        }
        colours.push(colour);
        edge(b'Q'); // activation with an offscreen frame timer queued
        assert_eq!(bus.call("comp.session.get", json!({}))["active"], true);
        ppm(&bus, &process, &format!("active-{index}"), None);
        assert_eq!(bus.window(&mut pattern)["id"], window["id"]);
    }
    let stopped = bus.call("comp.record.stop", json!({"session":recording["session"]}));
    assert_eq!(stopped["error"], Value::Null, "{stopped}");
    let records = assert_recorded_colours(&process.root.join("pause-resume.vfl"), &colours);
    assert_eq!(Some(records), stopped["frames"].as_u64());
    assert!(records > 0);
    // Recording intentionally changed the scene. Freeze its final pixels
    // before probing binding loss and explicit VT watcher recovery.
    let before_recovery = ppm(&bus, &process, "before-watcher-recovery", None).2;
    edge(b'P');
    edge(b'D'); // a temporarily missing binding must back off, not panic
    let restore = control.try_clone().unwrap();
    let restore_thread = thread::spawn(move || {
        thread::sleep(Duration::from_millis(400));
        restore.send(b"B").unwrap();
        let mut ack = [0];
        restore.recv(&mut ack).unwrap();
        assert_eq!(ack, *b"B");
    });
    ppm(&bus, &process, "restored-binding", None);
    restore_thread.join().unwrap();
    edge(b'F');
    assert_eq!(
        bus.call("comp.session.get", json!({}))["render_state"],
        "offscreen-retrying"
    );
    edge(b'R');
    let until = Instant::now() + Duration::from_secs(4);
    while bus.call("comp.session.get", json!({}))["render_state"] != "offscreen" {
        assert!(Instant::now() < until);
        thread::sleep(Duration::from_millis(50));
    }
    ppm(&bus, &process, "after-watcher-recovery", None);
    assert_eq!(
        before_recovery,
        ppm(&bus, &process, "after-switch-loop", None).2,
        "the static scene survives all buffer reuse and transitions"
    );
    assert_eq!(
        before.2,
        fs::read(process.root.join("owned-before.ppm"))
            .unwrap()
            .splitn(4, |b| *b == b'\n')
            .nth(3)
            .unwrap()
    );
    control.send(b"H").unwrap();
    let mut handovers = [0];
    control.recv(&mut handovers).unwrap();
    assert_eq!(handovers, [1]);
    assert!(process.child.try_wait().unwrap().is_none());
}
fn place_args(row: &Value, extra: Value) -> Value {
    let mut args = target(row);
    args.as_object_mut()
        .unwrap()
        .extend(extra.as_object().unwrap().clone());
    args
}
fn pump_patterns(patterns: &mut [&mut Pattern]) {
    for _ in 0..30 {
        for p in patterns.iter_mut() {
            p.pump();
        }
        thread::sleep(Duration::from_millis(5));
    }
}
fn ppm(
    bus: &Bus,
    process: &process::Process,
    label: &str,
    window: Option<Value>,
) -> (usize, usize, Vec<u8>) {
    let path = process.root.join(format!("{label}.ppm"));
    let mut args = json!({"path":path,"format":"ppm","cursor":false});
    if let Some(window) = window {
        args["window"] = window;
    } else {
        args["output"] = json!("headless-0");
    }
    bus.call("comp.capture.frame", args);
    let bytes = fs::read(path).unwrap();
    let mut parts = bytes.splitn(4, |b| *b == b'\n');
    assert_eq!(parts.next().unwrap(), b"P6");
    let dims = std::str::from_utf8(parts.next().unwrap())
        .unwrap()
        .split_whitespace()
        .map(|s| s.parse::<usize>().unwrap())
        .collect::<Vec<_>>();
    assert_eq!(parts.next().unwrap(), b"255");
    (dims[0], dims[1], parts.next().unwrap().to_vec())
}

/// This observer only connects to the broker owned by this test. It records
/// actual props.watch publications, rather than sampling props.get.
struct BlankObserver(std::process::Child);

impl BlankObserver {
    fn start(bus: &Bus) -> Self {
        let child = Command::new(
            std::env::var_os("MIX_TEST_MIX_EXE").unwrap_or_else(|| "/opt/mix/bin/mix".into()),
        )
        .arg("--serve")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/blank-observer.mix"))
        .args(["--name", "blank-observer"])
        .env("MIX_NODED_URL", &bus.url)
        .env("MIX_RUN", bus.root.join("run"))
        .env("MIX_ETC", bus.root.join("etc"))
        .env("MIX_VAR", bus.root.join("var"))
        .env("MIX_NODE_CONFIG", bus.root.join("etc/node.conf.mix"))
        .spawn()
        .expect("start blank observer on private broker");
        let observer = Self(child);
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let reply = observer.request(bus, "observer.blanks");
            if reply["rc"] == 0 {
                break;
            }
            assert!(Instant::now() < deadline, "observer registration: {reply}");
            thread::sleep(Duration::from_millis(20));
        }
        assert_eq!(observer.request(bus, "observer.start")["rc"], 0);
        observer
    }

    fn request(&self, bus: &Bus, command: &str) -> Value {
        let output = helper(
            "call",
            &bus.root,
            &[command, "{}", "blank-observer"],
            Some(&bus.url),
        );
        assert!(output.status.success(), "{output:?}");
        let text = String::from_utf8_lossy(&output.stdout);
        let mut reply: Value = serde_json::from_str(text.lines().last().unwrap()).unwrap();
        if reply["rc"] == 0
            && let Some(body) = reply["body"].as_str()
        {
            reply["body"] = serde_json::from_str(body).unwrap();
        }
        reply
    }

    fn wait_edges(&self, bus: &Bus, count: usize) -> Value {
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let reply = self.request(bus, "observer.blanks");
            assert_eq!(reply["rc"], 0, "{reply}");
            if reply["body"].as_array().unwrap().len() >= count {
                return reply["body"].clone();
            }
            assert!(Instant::now() < deadline, "missing blank edges: {reply}");
            thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for BlankObserver {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Fenced settings adapter fixture. Accepted values are written beneath the
/// owned broker root; real settingsd's durability is outside this test.
struct SettingsFixture {
    child: std::process::Child,
    state: PathBuf,
}

impl SettingsFixture {
    fn start(bus: &Bus) -> Self {
        let state = bus.root.join("settings-fixture.json");
        let child = Command::new(
            std::env::var_os("MIX_TEST_MIX_EXE").unwrap_or_else(|| "/opt/mix/bin/mix".into()),
        )
        .arg("--serve")
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/settings-fixture.mix"))
        .args(["--name", "settingsd"])
        .env("MIX_NODED_URL", &bus.url)
        .env("MIX_RUN", bus.root.join("run"))
        .env("MIX_ETC", bus.root.join("etc"))
        .env("MIX_VAR", bus.root.join("var"))
        .env("MIX_NODE_CONFIG", bus.root.join("etc/node.conf.mix"))
        .env("MIX_TEST_SETTINGS_STATE", &state)
        .spawn()
        .expect("start isolated settings fixture");
        let fixture = Self { child, state };
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let output = helper(
                "call",
                &bus.root,
                &["settings.fixture", "{}", "settingsd"],
                Some(&bus.url),
            );
            assert!(output.status.success(), "{output:?}");
            let reply: Value = serde_json::from_str(
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .last()
                    .unwrap(),
            )
            .unwrap();
            if reply["rc"] == 0 {
                return fixture;
            }
            assert!(
                Instant::now() < deadline,
                "settings fixture registration: {reply}"
            );
            thread::sleep(Duration::from_millis(20));
        }
    }

    fn accepted(&self) -> Value {
        serde_json::from_slice(&fs::read(&self.state).unwrap()).unwrap()
    }
}

impl Drop for SettingsFixture {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[test]
#[ignore = "needs a GPU render node and built viud, noded and native Mix"]
fn blank_preserves_capture_callbacks_and_agent_input_and_watch_reports_both_edges() {
    let bus = Bus::start("blank");
    let mut process = process::spawn(1, None, "blank", Some(&bus.url));
    let mut pattern = Pattern::new(connect(&mut process));
    pattern.wait_callback(0);
    bus.ready();
    // Headless has no CRTC. It still exposes the policy state without
    // pretending that it can blank a physical output.
    assert_eq!(
        bus.call("comp.props.get", json!({"path":"display.blank_supported"})),
        false
    );
    let observer = BlankObserver::start(&bus);
    let before = ppm(&bus, &process, "blank-before", None);
    assert!(before.2.iter().any(|channel| *channel != 0));
    bus.call("comp.display.blank", json!({"on":true}));
    assert_eq!(
        bus.call("comp.props.get", json!({"path":"display.blanked"})),
        true
    );
    let edges = observer.wait_edges(&bus, 1);
    assert_eq!(edges[0]["old"], false);
    assert_eq!(edges[0]["new"], true);
    assert_eq!(edges[0]["cause"], "bus");
    assert_eq!(ppm(&bus, &process, "blank-during", None), before);

    // A callback-only commit and a damaged commit must both keep getting
    // callbacks. This catches reusing session_lock's empty visible set.
    for damage in [false, true, false] {
        let callbacks = pattern.state.callbacks;
        pattern.commit(damage);
        pattern.wait_callback(callbacks);
    }
    let row = bus.window(&mut pattern);
    let keys = pattern.state.keys;
    bus.call(
        "comp.input.key",
        json!({"seat":"agent","window":target(&row),"key":"a"}),
    );
    pump_patterns(&mut [&mut pattern]);
    assert_eq!(pattern.state.keys, keys + 2);
    assert_eq!(
        bus.call("comp.props.get", json!({"path":"display.blanked"})),
        true,
        "agent input must not wake the human display"
    );
    assert_eq!(ppm(&bus, &process, "blank-after-agent", None), before);
    bus.call("comp.display.blank", json!({"on":false}));
    let edges = observer.wait_edges(&bus, 2);
    assert_eq!(edges.as_array().unwrap().len(), 2);
    assert_eq!(edges[1]["old"], true);
    assert_eq!(edges[1]["new"], false);
    assert_eq!(edges[1]["cause"], "bus");
    assert_eq!(
        bus.call("comp.props.get", json!({"path":"display.blanked"})),
        false
    );
    assert_eq!(ppm(&bus, &process, "blank-after", None), before);
}

#[test]
#[ignore = "needs a GPU render node and built viud, noded and native Mix"]
fn continuous_agent_input_does_not_postpone_the_idle_blank_timer() {
    let bus = Bus::start("blank-idle");
    let settings = SettingsFixture::start(&bus);
    let mut process = process::spawn(1, None, "blank-idle", Some(&bus.url));
    let mut pattern = Pattern::new(connect(&mut process));
    pattern.wait_callback(0);
    bus.ready();
    let row = bus.window(&mut pattern);
    bus.call(
        "comp.props.set",
        json!({"path":"power.screen_off_s","value":3}),
    );
    let accepted = settings.accepted();
    assert_eq!(accepted["values"]["power.screen_off_s"], 3);
    assert_eq!(accepted["operations"].as_array().unwrap().len(), 1);
    assert_eq!(
        bus.call("comp.props.get", json!({"path":"display.blanked"})),
        false,
        "positive control: timer starts armed rather than already fired"
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    let mut commands = 0;
    loop {
        bus.call(
            "comp.input.key",
            json!({"seat":"agent","window":target(&row),"key":"a"}),
        );
        commands += 1;
        pump_patterns(&mut [&mut pattern]);
        if bus.call("comp.props.get", json!({"path":"display.blanked"})) == true {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "continuous agent input postponed the human idle deadline"
        );
    }
    assert!(commands >= 2, "exercise input while the timer is armed");
    assert_eq!(pattern.state.keys, commands * 2);
    bus.call(
        "comp.props.set",
        json!({"path":"power.screen_off_s","value":null}),
    );
    let accepted = settings.accepted();
    assert!(accepted["values"]["power.screen_off_s"].is_null());
    assert_eq!(accepted["operations"].as_array().unwrap().len(), 2);
    bus.call("comp.display.blank", json!({"on":false}));
}

fn wait_inhibited(bus: &Bus, expected: bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if bus.call("comp.props.get", json!({"path":"display.inhibited"})) == expected {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "inhibited never became {expected}"
        );
        thread::sleep(Duration::from_millis(20));
    }
}

#[test]
#[ignore = "needs a render node and built viud/noded/Mix"]
fn idle_inhibitor_holds_both_timers_then_rearms_from_release() {
    let bus = Bus::start("inhibit-timers");
    let _settings = SettingsFixture::start(&bus);
    let mut process = process::spawn(1, None, "inhibit-timers", Some(&bus.url));
    let mut pattern = Pattern::new(connect(&mut process));
    pattern.wait_callback(0);
    bus.ready();
    let inhibitor = pattern.inhibit();
    wait_inhibited(&bus, true);
    pattern.idle_notification(100);
    bus.call(
        "comp.props.set",
        json!({"path":"power.screen_off_s","value":2}),
    );
    bus.call(
        "comp.props.set",
        json!({"path":"power.screen_off_s","value":1}),
    );
    let until = Instant::now() + Duration::from_millis(1200);
    while Instant::now() < until {
        pattern.pump();
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(pattern.state.idled, 0);
    assert_eq!(
        bus.call("comp.props.get", json!({"path":"display.blanked"})),
        false
    );
    inhibitor.destroy();
    pattern.pump();
    wait_inhibited(&bus, false);
    let released = Instant::now();
    while released.elapsed() < Duration::from_millis(250) {
        pattern.pump();
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(pattern.state.idled, 1);
    assert_eq!(
        bus.call("comp.props.get", json!({"path":"display.blanked"})),
        false
    );
    loop {
        if bus.call("comp.props.get", json!({"path":"display.blanked"})) == true {
            break;
        }
        assert!(
            released.elapsed() < Duration::from_secs(3),
            "blank did not rearm"
        );
        thread::sleep(Duration::from_millis(20));
    }
    assert!(released.elapsed() >= Duration::from_millis(900));
}

#[test]
#[ignore = "needs a render node and built viud/noded/Mix"]
fn idle_inhibitor_visibility_lifecycle_watch_and_bus_blank() {
    let bus = Bus::start("inhibit");
    let mut process = process::spawn(1, None, "inhibit", Some(&bus.url));
    let mut pattern = Pattern::new(connect(&mut process));
    pattern.wait_callback(0);
    bus.ready();
    let observer = BlankObserver::start(&bus);
    wait_inhibited(&bus, false);
    let a = pattern.inhibit();
    wait_inhibited(&bus, true);
    let row = bus.window(&mut pattern);
    bus.call("comp.window.minimize", target(&row));
    wait_inhibited(&bus, false);
    bus.call("comp.window.restore", target(&row));
    wait_inhibited(&bus, true);
    let lock = pattern.lock_session();
    wait_inhibited(&bus, false);
    lock.unlock_and_destroy();
    pattern.pump();
    wait_inhibited(&bus, true);
    bus.call(
        "comp.window.send_to_workspace",
        place_args(&row, json!({"index":2})),
    );
    wait_inhibited(&bus, false);
    bus.call("comp.workspace.switch", json!({"index":2}));
    wait_inhibited(&bus, true);
    bus.call("comp.workspace.switch", json!({"index":1}));
    wait_inhibited(&bus, false);
    bus.call(
        "comp.window.send_to_workspace",
        place_args(&row, json!({"index":1})),
    );
    wait_inhibited(&bus, true);
    bus.call("comp.display.blank", json!({"on":true}));
    assert_eq!(
        bus.call("comp.props.get", json!({"path":"display.blanked"})),
        true
    );
    let b = pattern.inhibit();
    wait_inhibited(&bus, true);
    assert_eq!(
        bus.call("comp.props.get", json!({"path":"display.blanked"})),
        true
    );
    bus.call("comp.display.blank", json!({"on":false}));
    a.destroy();
    pattern.pump();
    wait_inhibited(&bus, true);
    b.destroy();
    pattern.pump();
    wait_inhibited(&bus, false);
    let _c = pattern.inhibit();
    wait_inhibited(&bus, true);
    pattern.unmap();
    wait_inhibited(&bus, false);
    // Disconnect with a live resource, then positively prove a new client
    // can hold and release inhibition without the old resource remaining.
    drop(pattern);
    let mut next = Pattern::new(connect(&mut process));
    next.wait_callback(0);
    let _d = next.inhibit();
    wait_inhibited(&bus, true);
    drop(next);
    wait_inhibited(&bus, false);
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let reply = observer.request(&bus, "observer.inhibitors");
        assert_eq!(reply["rc"], 0);
        let edges = reply["body"].as_array().unwrap();
        if edges.len() >= 14 {
            assert_eq!(edges.len(), 14);
            for (i, edge) in edges.iter().enumerate() {
                assert_eq!(edge["old"], i % 2 != 0);
                assert_eq!(edge["new"], i % 2 == 0);
                assert_eq!(edge["cause"], "wayland.idle_inhibit");
            }
            break;
        }
        assert!(
            Instant::now() < deadline,
            "missing inhibitor edges: {reply}"
        );
    }
}

#[test]
#[ignore = "needs a render node and built viud/noded/Mix"]
fn review_z_overlap_and_workspace_move_deliver_to_top_window() {
    let bus = Bus::start("z-overlap");
    let mut process = process::spawn(1, None, "z-overlap", Some(&bus.url));
    let mut a = Pattern::named(connect(&mut process), "z-a", false);
    a.wait_callback(0);
    let mut b = Pattern::named(connect(&mut process), "z-b", false);
    b.wait_callback(0);
    bus.ready();
    let ar = bus.window_for("z-a", &mut [&mut a, &mut b]);
    let br = bus.window_for("z-b", &mut [&mut a, &mut b]);
    for (row, z) in [(&ar, 10), (&br, 20)] {
        bus.call(
            "comp.window.place",
            place_args(row, json!({"x":100,"y":100,"z":z})),
        );
    }
    for moved in [false, true] {
        if moved {
            bus.call(
                "comp.window.send_to_workspace",
                place_args(&br, json!({"index":2})),
            );
            bus.call(
                "comp.window.send_to_workspace",
                place_args(&ar, json!({"index":2,"follow":true})),
            );
            assert_eq!(bus.window_for("z-b", &mut [&mut a, &mut b])["z"], 20);
        }
        bus.call("comp.window.raise", target(&ar));
        let (ac, bc) = (a.state.pointer_buttons, b.state.pointer_buttons);
        let row = bus.window_for("z-a", &mut [&mut a, &mut b]);
        bus.call("comp.input.pointer.move",json!({"output":"headless-0","x":row["window_x"].as_f64().unwrap()+40.0,"y":row["window_y"].as_f64().unwrap()+50.0}));
        bus.call(
            "comp.input.pointer.button",
            json!({"button":"left","action":"click"}),
        );
        pump_patterns(&mut [&mut a, &mut b]);
        assert_eq!(
            a.state.pointer_buttons, ac,
            "z-lower A must not receive overlap click"
        );
        assert_eq!(
            b.state.pointer_buttons,
            bc + 2,
            "z-higher B receives overlap click, moved={moved}"
        );
    }
}

#[test]
fn stationary_agent_buttons_refresh_hits_but_keep_grab_and_targeted_focus() {
    if !process::record_available() {
        return;
    }
    let bus = Bus::start("stationary-agent");
    let mut process = process::spawn(1, None, "stationary-agent", Some(&bus.url));
    let mut a = Pattern::named(connect(&mut process), "stationary-a", false);
    a.wait_callback(0);
    let mut b = Pattern::named(connect(&mut process), "stationary-b", false);
    b.wait_callback(0);
    bus.ready();
    let ar = bus.window_for("stationary-a", &mut [&mut a, &mut b]);
    let br = bus.window_for("stationary-b", &mut [&mut a, &mut b]);
    for (row, z) in [(&ar, 20), (&br, 10)] {
        bus.call(
            "comp.window.place",
            place_args(row, json!({"x":100,"y":100,"z":z})),
        );
    }
    let row = bus.window_for("stationary-a", &mut [&mut a, &mut b]);
    bus.call("comp.input.pointer.move", json!({"seat":"agent","output":"headless-0","x":row["window_x"].as_f64().unwrap()+40.0,"y":row["window_y"].as_f64().unwrap()+50.0}));
    bus.call(
        "comp.input.pointer.button",
        json!({"seat":"agent","button":"left","action":"click"}),
    );
    pump_patterns(&mut [&mut a, &mut b]);
    let (ac, bc) = (a.state.pointer_buttons, b.state.pointer_buttons);
    assert!(ac >= 2, "initial hit must positively reach A");
    let motions = a.state.pointer_motions;
    bus.call(
        "comp.input.pointer.button",
        json!({"seat":"agent","button":"left","action":"click"}),
    );
    pump_patterns(&mut [&mut a, &mut b]);
    assert_eq!(
        a.state.pointer_motions, motions,
        "unchanged hit emits no motion"
    );
    let ac = a.state.pointer_buttons;
    bus.call("comp.window.place", place_args(&br, json!({"z":30})));
    // No motion: an untargeted press must discover the new topmost window.
    bus.call(
        "comp.input.pointer.button",
        json!({"seat":"agent","button":"left","action":"click"}),
    );
    pump_patterns(&mut [&mut a, &mut b]);
    assert_eq!(a.state.pointer_buttons, ac);
    assert_eq!(b.state.pointer_buttons, bc + 2);

    bus.call("comp.window.place", place_args(&ar, json!({"z":40})));
    bus.call(
        "comp.input.pointer.button",
        json!({"seat":"agent","button":"left","action":"press"}),
    );
    pump_patterns(&mut [&mut a, &mut b]);
    let (ac, bc) = (a.state.pointer_buttons, b.state.pointer_buttons);
    bus.call("comp.window.place", place_args(&br, json!({"z":50})));
    bus.call(
        "comp.input.pointer.button",
        json!({"seat":"agent","button":"right","action":"click"}),
    );
    bus.call(
        "comp.input.pointer.button",
        json!({"seat":"agent","button":"left","action":"release"}),
    );
    pump_patterns(&mut [&mut a, &mut b]);
    assert_eq!(
        a.state.pointer_buttons,
        ac + 3,
        "chord and release retain the implicit grab owner"
    );
    assert_eq!(b.state.pointer_buttons, bc);
    let ac = a.state.pointer_buttons;
    bus.call(
        "comp.input.pointer.button",
        json!({"seat":"agent","window":target(&ar),"button":"left","action":"click"}),
    );
    pump_patterns(&mut [&mut a, &mut b]);
    assert_eq!(
        a.state.pointer_buttons,
        ac + 2,
        "explicit targeting retains the named delivery target"
    );
    assert_eq!(b.state.pointer_buttons, bc);
}

#[test]
#[ignore = "needs a render node and built viud/noded/Mix"]
fn review_popup_outside_parent_is_drawn_without_clip_in_both_capture_paths() {
    let bus = Bus::start("popup-clip");
    let mut process = process::spawn(1, None, "popup-clip", Some(&bus.url));
    let mut p = Pattern::named(connect(&mut process), "popup-clip", false);
    p.wait_callback(0);
    bus.ready();
    let row = bus.window_for("popup-clip", &mut [&mut p]);
    bus.call(
        "comp.window.place",
        place_args(&row, json!({"x":100,"y":100})),
    );
    let before = ppm(&bus, &process, "before", Some(target(&row)));
    let (w, _, pixels) = ppm(&bus, &process, "output-before", None);
    let right = pixels
        .chunks_exact(3)
        .enumerate()
        .filter(|(_, rgb)| COLOURS.iter().any(|c| *rgb == crate::rgb(*c)))
        .map(|(i, _)| i % w)
        .max()
        .expect("parent content");
    let (_surface, _popup) = p.popup(false);
    pump_patterns(&mut [&mut p]);
    let (w, _, pixels) = ppm(&bus, &process, "output-popup", None);
    assert!(
        pixels
            .chunks_exact(3)
            .enumerate()
            .any(|(i, rgb)| i % w > right && rgb == [255, 0, 255]),
        "popup pixels extend beyond parent in presentation capture"
    );
    let after = ppm(&bus, &process, "window-popup", Some(target(&row)));
    assert!(
        after.0 > before.0,
        "capture extent includes the extending popup"
    );
    assert!(
        after
            .2
            .chunks_exact(3)
            .enumerate()
            .any(|(i, rgb)| i % after.0 >= before.0 && rgb == [255, 0, 255]),
        "extending popup pixels survive window capture"
    );
}

#[test]
#[ignore = "needs a render node and built viud/noded/Mix"]
fn review_hidden_popup_keyboard_grab_receives_no_more_keys() {
    let bus = Bus::start("hide-grab");
    let mut process = process::spawn(1, None, "hide-grab", Some(&bus.url));
    let mut p = Pattern::named(connect(&mut process), "hide-grab", false);
    p.wait_callback(0);
    bus.ready();
    let row = bus.window_for("hide-grab", &mut [&mut p]);
    bus.call("comp.input.key", json!({"window":target(&row),"key":"a"}));
    bus.call("comp.input.pointer.move",json!({"output":"headless-0","x":row["window_x"].as_f64().unwrap()+40.0,"y":row["window_y"].as_f64().unwrap()+50.0}));
    bus.call(
        "comp.input.pointer.button",
        json!({"button":"left","action":"press"}),
    );
    pump_patterns(&mut [&mut p]);
    let (_surface, _popup) = p.popup(true);
    pump_patterns(&mut [&mut p]);
    let focus = bus.call(
        "comp.props.get",
        json!({"path":"input.seats.agent.keyboard_focus"}),
    );
    assert!(!focus.is_null(), "popup holds keyboard focus");
    assert_ne!(focus["id"], row["id"], "grab focused the popup");
    let before = p.state.keys;
    bus.call("comp.input.key", json!({"key":"a"}));
    pump_patterns(&mut [&mut p]);
    assert!(
        p.state.keys > before,
        "positive control: grabbed window receives keys"
    );
    bus.call(
        "comp.window.place",
        place_args(&row, json!({"visible":false})),
    );
    pump_patterns(&mut [&mut p]);
    let before = p.state.keys;
    assert!(
        bus.call(
            "comp.props.get",
            json!({"path":"input.seats.agent.keyboard_focus"})
        )
        .is_null()
    );
    let refused = bus.call_rc("comp.input.key", json!({"key":"a"}), 10);
    assert_eq!(refused["error"], "no_keyboard_target");
    pump_patterns(&mut [&mut p]);
    assert_eq!(p.state.keys, before, "hidden grab receives no keys");
}

#[test]
#[ignore = "needs a render node and built viud/noded/Mix"]
fn review_hidden_remap_does_not_focus_and_show_with_geometry_is_atomic() {
    let bus = Bus::start("hide-remap");
    let mut process = process::spawn(1, None, "hide-remap", Some(&bus.url));
    let mut p = Pattern::named(connect(&mut process), "hide-remap", false);
    p.wait_callback(0);
    bus.ready();
    let row = bus.window_for("hide-remap", &mut [&mut p]);
    let reply = bus.call(
        "comp.window.place",
        place_args(&row, json!({"visible":false})),
    );
    for field in [
        "id",
        "generation",
        "output",
        "window_x",
        "window_y",
        "requested",
        "configure_pending",
    ] {
        assert!(
            reply.get(field).is_some(),
            "visual-only reply missing {field}"
        );
    }
    pump_patterns(&mut [&mut p]);
    let enters = p.state.keyboard_enters;
    p.remap();
    pump_patterns(&mut [&mut p]);
    let row = bus.window_for("hide-remap", &mut [&mut p]);
    assert_eq!(row["requested_visible"], false);
    bus.call(
        "comp.window.place",
        place_args(&row, json!({"x":120,"y":160})),
    );
    assert_eq!(
        bus.window_for("hide-remap", &mut [&mut p])["window_x"],
        120.0
    );
    assert_eq!(
        bus.window_for("hide-remap", &mut [&mut p])["requested_visible"],
        false
    );
    bus.call(
        "comp.window.send_to_workspace",
        place_args(&row, json!({"index":2})),
    );
    bus.call(
        "comp.window.send_to_workspace",
        place_args(&row, json!({"index":1})),
    );
    assert_eq!(
        p.state.keyboard_enters, enters,
        "hidden remap must not take keyboard focus"
    );
    bus.call(
        "comp.window.place",
        place_args(&row, json!({"visible":true,"x":140,"y":180})),
    );
    let shown = bus.window_for("hide-remap", &mut [&mut p]);
    assert_eq!(shown["visible"], true);
    assert_eq!(shown["window_x"], 140.0);
    assert_eq!(shown["window_y"], 180.0);
}

#[test]
#[ignore = "needs a render node and built viud/noded/Mix"]
fn review_visual_only_place_is_refused_by_session_lock_dispatch() {
    let bus = Bus::start("locked-place");
    let mut process = process::spawn(1, None, "locked-place", Some(&bus.url));
    let mut p = Pattern::new(connect(&mut process));
    p.wait_callback(0);
    bus.ready();
    let row = bus.window(&mut p);
    let _lock = p.lock_session();
    let refusal = bus.call_rc(
        "comp.window.place",
        place_args(&row, json!({"opacity":0.5})),
        10,
    );
    assert_eq!(refusal["error"], "locked");
}

#[test]
#[ignore = "needs a render node and built viud/noded/Mix"]
fn moved_window_clip_is_local_to_surface_origin() {
    let bus = Bus::start("moved-clip");
    let mut process = process::spawn(1, None, "moved-clip", Some(&bus.url));
    let mut p = Pattern::named(connect(&mut process), "moved-clip", false);
    p.wait_callback(0);
    bus.ready();
    let row = bus.window_for("moved-clip", &mut [&mut p]);
    bus.call(
        "comp.window.place",
        place_args(&row, json!({"x":200,"y":200})),
    );
    let (w, _, original) = ppm(&bus, &process, "origin", None);
    let red = original
        .chunks_exact(3)
        .enumerate()
        .filter(|(_, rgb)| *rgb == crate::rgb(COLOURS[0]))
        .map(|(i, _)| (i % w, i / w))
        .collect::<Vec<_>>();
    let left = red.iter().map(|p| p.0).min().unwrap();
    let top = red.iter().map(|p| p.1).min().unwrap();
    bus.call(
        "comp.window.place",
        place_args(&row, json!({"clip":{"x":20,"y":20,"width":40,"height":40}})),
    );
    let (w, _, clipped) = ppm(&bus, &process, "local-clip", None);
    let at = |x: usize, y: usize| &clipped[(y * w + x) * 3..(y * w + x) * 3 + 3];
    assert_eq!(at(left + 30, top + 30), crate::rgb(COLOURS[0]));
    assert_eq!(at(left + 10, top + 30), [18, 20, 25]);
    assert_eq!(at(left + 70, top + 30), [18, 20, 25]);
}
fn helper(action: &str, root: &Path, rest: &[&str], url: Option<&str>) -> Output {
    let mut command = Command::new(
        std::env::var_os("MIX_TEST_MIX_EXE").unwrap_or_else(|| "/opt/mix/bin/mix".into()),
    );
    command
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/bus.mix"))
        .arg(action)
        .arg(root)
        .args(rest)
        .env("MIX_NODED_URL", url.unwrap_or("ws://127.0.0.1:1"))
        .env("MIX_RUN", root.join("run"))
        .env("MIX_ETC", root.join("etc"))
        .env("MIX_VAR", root.join("var"))
        .env("MIX_NODE_CONFIG", root.join("etc/node.conf.mix"));
    command.output().unwrap()
}
fn connect(process: &mut process::Process) -> UnixStream {
    let deadline = Instant::now() + Duration::from_secs(25);
    loop {
        if let Ok(stream) = UnixStream::connect(process.root.join("wayland-headless-test")) {
            use std::os::fd::AsRawFd;
            let mut cred: libc::ucred = unsafe { std::mem::zeroed() };
            let mut len = std::mem::size_of_val(&cred) as libc::socklen_t;
            assert_eq!(
                unsafe {
                    libc::getsockopt(
                        stream.as_raw_fd(),
                        libc::SOL_SOCKET,
                        libc::SO_PEERCRED,
                        (&mut cred as *mut libc::ucred).cast(),
                        &mut len,
                    )
                },
                0
            );
            if std::env::var_os(paths::env::test::TEST_MAPPING_TRACE).is_some() {
                eprintln!(
                    "WAYLAND_PEER expected={} actual={} root={}",
                    process.child.id(),
                    cred.pid,
                    process.root.display()
                );
            }
            assert_eq!(cred.pid as u32, process.child.id());
            return stream;
        }
        process.assert_running();
        assert!(Instant::now() < deadline, "viud startup timed out");
        thread::sleep(Duration::from_millis(10));
    }
}
fn pump(client: &mut TestClient, predicate: impl Fn(&TestClient) -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !predicate(client) {
        client.flush();
        client.read();
        assert!(
            Instant::now() < deadline,
            "Wayland capture timed out: {:?}",
            client.state.frames
        );
        thread::sleep(Duration::from_millis(2));
    }
}
fn roundtrip(client: &mut TestClient) {
    let sync = client.sync();
    pump(client, |c| c.state.syncs >= sync);
}
fn pixel(bytes: &[u8], width: usize, x: usize, y: usize) -> [u8; 3] {
    let at = (y * width + x) * 4;
    [bytes[at + 2], bytes[at + 1], bytes[at]]
}
fn rgb(colour: u32) -> [u8; 3] {
    [(colour >> 16) as u8, (colour >> 8) as u8, colour as u8]
}

fn monotonic_ns() -> u64 {
    let mut time = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: time is a valid writable timespec.
    assert_eq!(
        unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut time) },
        0
    );
    time.tv_sec as u64 * 1_000_000_000 + time.tv_nsec as u64
}

fn frame_count(root: &Path) -> usize {
    let trace = fs::read_to_string(root.join("frames.log")).unwrap();
    assert!(!trace.contains("stage=trace_limit "));
    for line in trace.lines() {
        assert!(
            line.contains(" dropped=0 "),
            "incomplete frame trace: {line}"
        );
    }
    trace
        .lines()
        .filter(|line| line.contains("stage=comp_frame_reasons "))
        .count()
}

#[test]
#[ignore = "needs a GPU render node and built viud, noded and testkit binaries"]
fn pattern_mixed_cursor_captures_and_presentation_ledger() {
    let bus = Bus::start("pixels");
    let mut process = process::spawn(1, None, "pixels", Some(&bus.url));
    let stream = connect(&mut process);
    bus.ready();
    let wall_start = monotonic_ns();
    let mut pattern = Pattern::new(stream);
    pattern.wait_callback(0);
    // Headless has no human sources. Move the fixed built-in 24px amber
    // agent arrow; no host cursor theme or animation enters the test.
    bus.call(
        "comp.input.pointer.move",
        json!({"seat":"agent","output":"headless-0","x":160,"y":160}),
    );
    for _ in 0..20 {
        pattern.pump();
        thread::sleep(Duration::from_millis(5));
    }
    let mut capture = TestClient::new(connect(&mut process));
    roundtrip(&mut capture);
    capture.bind_globals();
    roundtrip(&mut capture);
    let (off, oi) = capture.capture_with(None, false);
    let (on, ni) = capture.capture_with(None, true);
    pump(&mut capture, |c| {
        c.state.frames[oi].buffer_done && c.state.frames[ni].buffer_done
    });
    for index in [oi, ni] {
        assert_eq!(capture.state.frames[index].buffer.unwrap().1, 1280);
        assert_eq!(capture.state.frames[index].buffer.unwrap().2, 720);
    }
    let (off_buffer, off_file) = capture.shm_buffer_with(1280, 720, 5120, wl_shm::Format::Xrgb8888);
    let (on_buffer, on_file) = capture.shm_buffer_with(1280, 720, 5120, wl_shm::Format::Xrgb8888);
    // Both copies enter the same Wayland flush, before the one-shot frame.
    off.copy(&off_buffer);
    on.copy(&on_buffer);
    capture.flush();
    pump(&mut capture, |c| {
        c.state.frames[oi].ready && c.state.frames[ni].ready
    });
    let off_pixels = capture.read_buffer(off_file);
    let on_pixels = capture.read_buffer(on_file);
    for ((x, y), colour) in [(320, 180), (960, 180), (320, 540), (960, 540)]
        .into_iter()
        .zip(COLOURS)
    {
        assert_eq!(pixel(&off_pixels, 1280, x, y), rgb(colour));
    }
    assert_ne!(
        pixel(&off_pixels, 1280, 320, 180),
        [255, 0, 255],
        "negative colour control"
    );
    let mut changed = 0;
    let mut amber = 0;
    for y in 0..720 {
        for x in 0..1280 {
            let a = pixel(&off_pixels, 1280, x, y);
            let b = pixel(&on_pixels, 1280, x, y);
            if (160..184).contains(&x) && (160..184).contains(&y) {
                assert_eq!(a, rgb(COLOURS[0]), "cursorless pixel");
                changed += usize::from(a != b);
                amber += usize::from(b == [255, 159, 28]);
            } else {
                assert_eq!(a, b, "unexpected difference at {x},{y}");
            }
        }
    }
    let (opaque_pixels, fill_pixels) =
        world::seat::pointer::texture::pointer_load::agent_arrow_pixel_counts(24);
    assert_eq!(
        changed, opaque_pixels,
        "complete agent arrow must be captured"
    );
    assert_eq!(
        amber, fill_pixels,
        "complete agent arrow fill must be captured"
    );
    // Also exercise the shipped screenshot client against the same pattern.
    let shot = process.root.join("screenshot.ppm");
    let output = Command::new(binary("testkit-screenshot"))
        .arg(&shot)
        .env("XDG_RUNTIME_DIR", &process.root)
        .env("WAYLAND_DISPLAY", "wayland-headless-test")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "screenshot: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let ppm = fs::read(&shot).unwrap();
    let header = b"P6\n1280 720\n255\n";
    assert!(ppm.starts_with(header));
    for ((x, y), colour) in [(320, 180), (960, 180), (320, 540), (960, 540)]
        .into_iter()
        .zip(COLOURS)
    {
        let at = header.len() + (y * 1280 + x) * 3;
        assert_eq!(&ppm[at..at + 3], &rgb(colour));
    }
    println!(
        "four pattern points + negative control; screenshot; mixed cursor frame: {changed} changed pixels confined to [160,160,24,24], {amber} amber pixels"
    );

    let before = bus.window(&mut pattern)["presentation"]["presented"]
        .as_u64()
        .unwrap();
    for _ in 0..12 {
        let previous = pattern.state.callbacks;
        pattern.commit(true);
        pattern.wait_callback(previous);
    }
    thread::sleep(Duration::from_millis(150));
    let after = bus.window(&mut pattern)["presentation"]["presented"]
        .as_u64()
        .unwrap();
    assert!(
        after >= before + 10,
        "damage commits did not present: {before} -> {after}"
    );
    for _ in 0..12 {
        let previous = pattern.state.callbacks;
        pattern.commit(false);
        pattern.wait_callback(previous);
    }
    thread::sleep(Duration::from_millis(150));
    let still = bus.window(&mut pattern)["presentation"]["presented"]
        .as_u64()
        .unwrap();
    assert_eq!(after, still, "callback-only commits must not present");
    // All callbacks have been consumed and the client stops committing. Count
    // both presented and empty draws through viud's existing frame ledger.
    thread::sleep(Duration::from_millis(200));
    let frames = frame_count(&process.root);
    assert!(frames > 0, "frame tracing must be active");
    let idle_start = Instant::now();
    while idle_start.elapsed() < Duration::from_secs(1) {
        pattern.pump();
        assert_eq!(
            frame_count(&process.root),
            frames,
            "idle timer kept drawing"
        );
        thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(
        still,
        bus.window(&mut pattern)["presentation"]["presented"]
            .as_u64()
            .unwrap()
    );
    assert_eq!(pattern.state.clock, Some(libc::CLOCK_MONOTONIC as u32));
    assert!(pattern.state.presented >= 12);
    assert!(pattern.state.discarded >= 10);
    assert!(
        pattern
            .state
            .times
            .windows(2)
            .all(|pair| pair[0] <= pair[1])
    );
    let wall_end = monotonic_ns();
    assert!(!pattern.state.times.is_empty());
    for time in &pattern.state.times {
        assert!(
            (wall_start..=wall_end).contains(time),
            "presentation {time} outside monotonic test window {wall_start}..={wall_end}"
        );
    }
    println!(
        "presentation.presented {before} -> {after} -> {still}; {} callbacks, {} presented / {} discarded feedbacks; CLOCK_MONOTONIC",
        pattern.state.callbacks, pattern.state.presented, pattern.state.discarded
    );
    println!(
        "idle frame count stayed at {frames} for {:?}; timestamps within {wall_start}..={wall_end}",
        idle_start.elapsed()
    );
    // Different content still presents after the idle/callback-only stretch.
    pattern.change_buffer();
    let previous = pattern.state.callbacks;
    pattern.commit(true);
    pattern.wait_callback(previous);
}

#[test]
#[ignore = "needs a GPU render node and built viud and noded binaries"]
fn bus_capture_completes_on_idle_output() {
    let bus = Bus::start("idle");
    let mut process = process::spawn(1, None, "idle", Some(&bus.url));
    drop(connect(&mut process));
    bus.ready();
    thread::sleep(Duration::from_millis(300));
    assert_eq!(
        bus.call("comp.windows.list", json!({}))["windows"],
        json!([])
    );
    bus.call(
        "comp.input.pointer.move",
        json!({"seat":"agent","output":"headless-0","x":160,"y":160}),
    );
    for cursor in [false, true] {
        let path = process.root.join(format!("idle-{cursor}.ppm"));
        let start = Instant::now();
        let reply = bus.call(
            "comp.capture.frame",
            json!({"output":"headless-0","path":path,"format":"ppm","cursor":cursor}),
        );
        let elapsed = start.elapsed();
        assert!(
            elapsed < Duration::from_millis(1500),
            "idle capture took {elapsed:?}"
        );
        assert_eq!(reply["width"], 1280);
        assert_eq!(reply["height"], 720);
        assert_eq!(reply["source"], "offscreen");
        let ppm = fs::read(path).unwrap();
        assert!(ppm.starts_with(b"P6\n1280 720\n255\n"));
        assert_eq!(ppm.len(), "P6\n1280 720\n255\n".len() + 1280 * 720 * 3);
        // frames::scene paints the empty desktop with BACKGROUND_COLOR
        // [0.07, 0.08, 0.10, 1.0], covering the framebuffer's GL clear colour.
        for (i, pixel) in ppm[b"P6\n1280 720\n255\n".len()..]
            .chunks_exact(3)
            .enumerate()
        {
            let (x, y) = (i % 1280, i / 1280);
            if !cursor || !(160..184).contains(&x) || !(160..184).contains(&y) {
                assert_eq!(
                    pixel,
                    [18, 20, 25],
                    "empty desktop colour cursor={cursor} at {x},{y}"
                );
            }
        }
        println!("idle Bus capture cursor={cursor}: {elapsed:?}, 1280x720 offscreen");
    }
}
#[test]
#[ignore = "needs a GPU render node and built viud and noded binaries"]
fn placement_visuals_persist_capture_and_suppress_hidden_input() {
    let bus = Bus::start("place-visuals");
    let mut process = process::spawn(1, None, "place-visuals", Some(&bus.url));
    let mut pattern = Pattern::new(connect(&mut process));
    bus.ready();
    pattern.wait_callback(0);
    let window = bus.window(&mut pattern);
    let place = |extra: Value| {
        let mut args = json!({"id":window["id"],"generation":window["generation"]});
        args.as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        bus.call("comp.window.place", args)
    };
    let capture = |label: &str| {
        let path = process.root.join(format!("{label}.ppm"));
        bus.call(
            "comp.capture.frame",
            json!({"output":"headless-0","path":path,"format":"ppm","cursor":false}),
        );
        let ppm = fs::read(path).unwrap();
        let header = b"P6\n1280 720\n255\n";
        assert!(ppm.starts_with(header));
        ppm[header.len()..].to_vec()
    };
    let at = |bytes: &[u8], x: usize, y: usize| -> [u8; 3] {
        bytes[(y * 1280 + x) * 3..(y * 1280 + x) * 3 + 3]
            .try_into()
            .unwrap()
    };
    let original = capture("original");
    assert_eq!(at(&original, 320, 180), rgb(COLOURS[0]));
    place(json!({"opacity":0.5,"z":5,"clip":{"x":0,"y":0,"width":640,"height":360}}));
    let row = bus.window(&mut pattern);
    assert_eq!(row["opacity"], 0.5);
    assert_eq!(row["z"], 5);
    assert_eq!(row["clip"]["width"], 640.0);
    let blended = capture("blend-clip");
    for (actual, (fg, bg)) in at(&blended, 320, 180)
        .into_iter()
        .zip(rgb(COLOURS[0]).into_iter().zip([18, 20, 25]))
    {
        assert!(
            (i32::from(actual) - ((i32::from(fg) + bg) / 2)).abs() <= 3,
            "alpha blend: {actual}, foreground {fg}, background {bg}"
        );
    }
    assert_eq!(at(&blended, 960, 540), [18, 20, 25], "outside local clip");
    let click = || {
        bus.call(
            "comp.input.pointer.move",
            json!({"seat":"agent","output":"headless-0","x":320,"y":180}),
        );
        bus.call(
            "comp.input.pointer.button",
            json!({"seat":"agent","button":"left","action":"click"}),
        );
    };
    click();
    for _ in 0..30 {
        pattern.pump();
        thread::sleep(Duration::from_millis(5));
    }
    assert!(
        pattern.state.pointer_buttons >= 2,
        "opacity retains pointer input"
    );
    place(json!({"visible":false}));
    let hidden = bus.window(&mut pattern);
    assert_eq!(hidden["requested_visible"], false);
    assert_eq!(hidden["visible"], false);
    let focused = bus.call(
        "comp.window.focus",
        json!({"id":window["id"],"generation":window["generation"]}),
    );
    assert_eq!(
        focused["focused"], false,
        "hidden windows cannot reacquire keyboard focus"
    );
    assert_eq!(
        hidden["opacity"], 0.5,
        "omitted attributes persist across list"
    );
    assert_eq!(hidden["window_width"], window["window_width"]);
    assert_eq!(hidden["window_height"], window["window_height"]);
    for _ in 0..20 {
        pattern.pump();
        thread::sleep(Duration::from_millis(5));
    }
    let buttons = pattern.state.pointer_buttons;
    let callbacks = pattern.state.callbacks;
    let refused = bus.call_rc(
        "comp.input.pointer.button",
        json!({"seat":"agent","button":"left","action":"click"}),
        10,
    );
    assert_eq!(refused["error"], "no_pointer_target");
    let hidden_commit = Instant::now();
    pattern.commit(true);
    let blank = capture("hidden");
    assert_eq!(at(&blank, 320, 180), [18, 20, 25]);
    for _ in 0..60 {
        pattern.pump();
        thread::sleep(Duration::from_millis(5));
    }
    assert_eq!(
        pattern.state.pointer_buttons, buttons,
        "hidden window receives no pointer buttons"
    );
    assert!(
        pattern.state.callbacks - callbacks <= hidden_commit.elapsed().as_secs(),
        "hidden callbacks must follow the one-second occlusion trickle"
    );
    place(json!({"opacity":null,"z":null,"visible":null,"clip":null}));
    let reset = bus.window(&mut pattern);
    assert_eq!(reset["opacity"], 1.0);
    assert_eq!(reset["z"], 0);
    assert_eq!(reset["requested_visible"], true);
    assert!(reset["clip"].is_null());
    assert_eq!(at(&capture("reset"), 960, 540), rgb(COLOURS[3]));
}
