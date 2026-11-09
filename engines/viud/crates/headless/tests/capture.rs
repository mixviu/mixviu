// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Real render-node compositor, Wayland clients and an owned private Bus.
//! Build viud (backend-headless), noded and testkit binaries before running.
#[path = "support/pattern.rs"]
mod pattern;
#[path = "support/process.rs"]
mod process;

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
    root: PathBuf,
    url: String,
}
impl Bus {
    fn window_for(&self, name: &str) -> Value {
        self.call("comp.windows.list", json!({"app_id":name}))["windows"]
            .as_array()
            .unwrap()
            .first()
            .expect("mapped named window")
            .clone()
    }
    fn start(label: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("headless-bus-{}-{label}", std::process::id()));
        let output = helper("start", &root, &[binary("noded").to_str().unwrap()], None);
        assert!(
            output.status.success(),
            "private broker: {} {}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let record: Value =
            serde_json::from_slice(&fs::read(root.join("bus.json")).unwrap()).unwrap();
        Self {
            root,
            url: record["url"].as_str().unwrap().into(),
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
    }
    fn window(&self) -> Value {
        let reply = self.call("comp.windows.list", json!({"app_id":"headless-pattern"}));
        reply["windows"]
            .as_array()
            .unwrap()
            .first()
            .expect("mapped pattern window")
            .clone()
    }
}

fn target(row: &Value) -> Value {
    json!({"id":row["id"],"generation":row["generation"]})
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
    let ar = bus.window_for("z-a");
    let br = bus.window_for("z-b");
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
            assert_eq!(bus.window_for("z-b")["z"], 20);
        }
        bus.call("comp.window.raise", target(&ar));
        let (ac, bc) = (a.state.pointer_buttons, b.state.pointer_buttons);
        let row = bus.window_for("z-a");
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
#[ignore = "needs a render node and built viud/noded/Mix"]
fn review_popup_outside_parent_is_drawn_without_clip_in_both_capture_paths() {
    let bus = Bus::start("popup-clip");
    let mut process = process::spawn(1, None, "popup-clip", Some(&bus.url));
    let mut p = Pattern::named(connect(&mut process), "popup-clip", false);
    p.wait_callback(0);
    bus.ready();
    let row = bus.window_for("popup-clip");
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
    let row = bus.window_for("hide-grab");
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
    let row = bus.window_for("hide-remap");
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
    let row = bus.window_for("hide-remap");
    assert_eq!(row["requested_visible"], false);
    bus.call(
        "comp.window.place",
        place_args(&row, json!({"x":120,"y":160})),
    );
    assert_eq!(bus.window_for("hide-remap")["window_x"], 120.0);
    assert_eq!(bus.window_for("hide-remap")["requested_visible"], false);
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
    let shown = bus.window_for("hide-remap");
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
    let row = bus.window();
    let _lock = p.lock_session();
    let refusal = bus.call_rc(
        "comp.window.place",
        place_args(&row, json!({"opacity":0.5})),
        10,
    );
    assert_eq!(refusal["error"], "locked");
}

impl Drop for Bus {
    fn drop(&mut self) {
        let output = helper("stop", &self.root, &[], None);
        assert!(
            output.status.success(),
            "stop owned broker: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
#[ignore = "needs a render node and built viud/noded/Mix"]
fn moved_window_clip_is_local_to_surface_origin() {
    let bus = Bus::start("moved-clip");
    let mut process = process::spawn(1, None, "moved-clip", Some(&bus.url));
    let mut p = Pattern::named(connect(&mut process), "moved-clip", false);
    p.wait_callback(0);
    bus.ready();
    let row = bus.window_for("moved-clip");
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
        std::env::var_os("MIX_TEST_MIX_EXE").unwrap_or_else(|| "/opt/mixos/bin/mix".into()),
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
            return stream;
        }
        assert!(
            process.child.try_wait().unwrap().is_none(),
            "viud exited: {}",
            fs::read_to_string(process.root.join("viud.log")).unwrap()
        );
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

    let before = bus.window()["presentation"]["presented"].as_u64().unwrap();
    for _ in 0..12 {
        let previous = pattern.state.callbacks;
        pattern.commit(true);
        pattern.wait_callback(previous);
    }
    thread::sleep(Duration::from_millis(150));
    let after = bus.window()["presentation"]["presented"].as_u64().unwrap();
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
    let still = bus.window()["presentation"]["presented"].as_u64().unwrap();
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
        bus.window()["presentation"]["presented"].as_u64().unwrap()
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
    let window = bus.window();
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
    let row = bus.window();
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
    let hidden = bus.window();
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
    let reset = bus.window();
    assert_eq!(reset["opacity"], 1.0);
    assert_eq!(reset["z"], 0);
    assert_eq!(reset["requested_visible"], true);
    assert!(reset["clip"].is_null());
    assert_eq!(at(&capture("reset"), 960, 540), rgb(COLOURS[3]));
}
