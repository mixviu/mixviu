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

#[test]
#[ignore = "needs a GPU render node, ffmpeg and built viud/noded/testkit binaries"]
fn record_windows_only_content_and_multiple_targets() {
    let bus = Bus::start("record-windows");
    let mut process = process::spawn(1, None, "record-windows", Some(&bus.url));
    let mut b = Pattern::named(connect(&mut process), "record-window-b", false);
    let mut a = Pattern::named(connect(&mut process), "record-window-a", false);
    bus.ready();
    a.change_buffer();
    a.commit(true);
    pump_patterns(&mut [&mut a, &mut b]);
    let ar = bus.window_for("record-window-a");
    let br = bus.window_for("record-window-b");
    for (row, z) in [(&br, 0), (&ar, 10)] {
        bus.call(
            "comp.window.place",
            place_args(row, json!({"x":100,"y":100,"z":z})),
        );
    }
    let start=bus.call("comp.record.start",json!({"request_id":"windows","target":{"windows":[target(&br),target(&ar)]},"cursor":false,"path":process.root.join("windows.mkv"),"max_seconds":5}));
    assert_eq!(
        start["width"], 400,
        "no server frame or shadow is part of a window take"
    );
    assert_eq!(start["height"], 160);
    thread::sleep(Duration::from_millis(150));
    let stop = bus.call("comp.record.stop", json!({"session":start["session"]}));
    assert_eq!(stop["error"], Value::Null, "{stop}");
    let decoded = Command::new("ffmpeg")
        .args(["-v", "error", "-i"])
        .arg(start["path"].as_str().unwrap())
        .args(["-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgb24", "-"])
        .output()
        .unwrap();
    assert!(decoded.status.success());
    assert_eq!(decoded.stdout.len(), 400 * 160 * 3);
    for (x, expected) in [(50, rgb(COLOURS[0])), (250, [0xaa, 0x33, 0x77])] {
        let at = (40 * 400 + x) * 3;
        for (actual, expected) in decoded.stdout[at..at + 3].iter().zip(expected) {
            assert!((i16::from(*actual) - i16::from(expected)).abs() < 20);
        }
    }
}

#[test]
#[ignore = "needs a GPU render node, ffmpeg and built viud/noded binaries"]
fn record_vaapi_capability_or_clear_failure() {
    let bus = Bus::start("record-vaapi");
    let mut process = process::spawn(1, None, "record-vaapi", Some(&bus.url));
    drop(connect(&mut process));
    bus.ready();
    let probe = Command::new("ffmpeg")
        .args(["-hide_banner", "-encoders"])
        .output()
        .unwrap();
    let reported = String::from_utf8_lossy(&probe.stdout)
        .split_whitespace()
        .any(|word| word == "h264_vaapi");
    let args = json!({"request_id":"vaapi","target":{"output":"headless-0"},"encoder":"vaapi","path":process.root.join("vaapi.mkv"),"max_seconds":5});
    if !reported {
        let error = bus.call_rc("comp.record.start", args, 10);
        assert!(error["message"].as_str().unwrap().contains("h264_vaapi"));
        println!("VAAPI not reported by ffmpeg; explicit refusal verified");
        return;
    }
    let start = bus.call("comp.record.start", args);
    thread::sleep(Duration::from_millis(300));
    let stop = bus.call("comp.record.stop", json!({"session":start["session"]}));
    if let Some(error) = stop["error"].as_str() {
        assert!(
            error.contains("ffmpeg"),
            "device failure must include ffmpeg stderr: {stop}"
        );
        println!("VAAPI reported, device unavailable: {error}");
    } else {
        assert_eq!(stop["encoder"], "vaapi");
        assert!(fs::metadata(start["path"].as_str().unwrap()).unwrap().len() > 0);
        println!("VAAPI recording completed: {stop}");
    }
}

#[test]
#[ignore = "needs a GPU render node, ffmpeg/libx264 and built viud/noded/testkit binaries"]
fn record_4k_output_windows_pointer_and_damage_repeat() {
    let bus = Bus::start("record-4k");
    let mut process = process::spawn_4k("record-4k", &bus.url);
    let mut a = Pattern::named(connect(&mut process), "record-a", false);
    let mut b = Pattern::named(connect(&mut process), "record-b", false);
    bus.ready();
    pump_patterns(&mut [&mut a, &mut b]);
    for (name, x) in [("record-a", 100), ("record-b", 600)] {
        let row = bus.window_for(name);
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
    let path = process.root.join("4k.mkv");
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
    thread::sleep(Duration::from_millis(200));
    let moved = bus.call("comp.record.status", json!({"session":session}));
    assert!(
        moved["readbacks"].as_u64() > idle["readbacks"].as_u64(),
        "pointer damage must cause readback"
    );
    let marker = bus.call(
        "comp.record.marker",
        json!({"session":session,"label":"4k-done"}),
    );
    let stop = bus.call("comp.record.stop", json!({"session":session}));
    assert!(
        stop["repeated"].as_u64() >= stop["dropped"].as_u64(),
        "overruns must be repeated on the timeline: {stop}"
    );
    assert!(marker["frame"].as_u64() < stop["frames"].as_u64());
    let decoded = Command::new("ffmpeg")
        .args(["-v", "error", "-i"])
        .arg(&path)
        .args(["-frames:v", "1", "-f", "rawvideo", "-pix_fmt", "rgb24", "-"])
        .output()
        .unwrap();
    assert!(
        decoded.status.success(),
        "{}",
        String::from_utf8_lossy(&decoded.stderr)
    );
    assert_eq!(decoded.stdout.len(), 3840 * 2160 * 3);
    let counted = Command::new("ffmpeg")
        .args(["-v", "error", "-i"])
        .arg(&path)
        .args(["-progress", "pipe:1", "-nostats", "-f", "null", "-"])
        .output()
        .unwrap();
    assert!(
        counted.status.success(),
        "{}",
        String::from_utf8_lossy(&counted.stderr)
    );
    let encoded_frames = String::from_utf8_lossy(&counted.stdout)
        .lines()
        .filter_map(|line| line.strip_prefix("frame=")?.trim().parse::<u64>().ok())
        .max()
        .unwrap();
    assert_eq!(
        encoded_frames,
        stop["frames"].as_u64().unwrap(),
        "queue overruns must not shorten the encoded stream"
    );
    let at = |x: usize, y: usize| {
        let i = (y * 3840 + x) * 3;
        &decoded.stdout[i..i + 3]
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
    let duration = frames.last().unwrap()["t_us"].as_u64().unwrap() as f64 / 1_000_000.0;
    assert!(
        (frames.len() as f64 - duration * 30.0).abs() < 5.0,
        "{} frames in {duration}s",
        frames.len()
    );
    println!("4K30: {stop}; idle status: {idle}");
}

#[test]
#[ignore = "needs a GPU render node, ffmpeg/libx264 and built viud/noded binaries"]
fn record_idle_output_lifecycle_and_clock() {
    let bus = Bus::start("record-idle");
    let mut process = process::spawn(1, None, "record-idle", Some(&bus.url));
    drop(connect(&mut process));
    bus.ready();
    let observer = BlankObserver::start(&bus);
    let path = process.root.join("take.mkv");
    let args = json!({"request_id":"take-1","target":{"output":"headless-0"},"path":path,"fps":30,"max_seconds":10});
    let start = bus.call("comp.record.start", args.clone());
    assert_eq!(start, bus.call("comp.record.start", args.clone()));
    let session = start["session"].clone();
    assert_eq!(start["clock"], "monotonic");
    assert_eq!(start["width"], 1280);
    assert_eq!(start["height"], 720);
    let busy = bus.call_rc("comp.record.start",json!({"request_id":"other","target":{"output":"headless-0"},"path":process.root.join("other.mkv"),"max_seconds":10}),10);
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
    assert!(frames >= 10, "{stop}");
    assert!(fs::metadata(&path).unwrap().len() > 0);
    let decoded = Command::new("ffmpeg")
        .args(["-v", "error", "-i"])
        .arg(&path)
        .args(["-f", "rawvideo", "-pix_fmt", "rgb24", "-"])
        .output()
        .unwrap();
    assert!(
        decoded.status.success(),
        "{}",
        String::from_utf8_lossy(&decoded.stderr)
    );
    assert_eq!(decoded.stdout.len() as u64, frames * 1280 * 720 * 3);
    let rows: Vec<Value> = fs::read_to_string(start["frames_path"].as_str().unwrap())
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(rows.len() as u64, frames);
    let duration = rows.last().unwrap()["t_us"].as_u64().unwrap() as f64 / 1_000_000.0;
    assert!(
        (frames as f64 - 30.0 * duration).abs() < 5.0,
        "{frames} frames in {duration}s"
    );
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
    let next=bus.call("comp.record.start",json!({"request_id":"bounded","target":{"output":"headless-0"},"path":process.root.join("bounded.mkv"),"max_seconds":1}));
    thread::sleep(Duration::from_millis(1300));
    let bounded = bus.call("comp.record.stop", json!({"session":next["session"]}));
    assert!(
        (28..=32).contains(&bounded["frames"].as_u64().unwrap()),
        "max_seconds did not bound the take: {bounded}"
    );
    assert_eq!(start, bus.call("comp.record.start", args));
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
#[ignore = "needs a GPU render node, ffmpeg/libx264 and built viud/noded/testkit binaries"]
fn record_window_target_gone_fails() {
    let bus = Bus::start("record-gone");
    let mut process = process::spawn(1, None, "record-gone", Some(&bus.url));
    let mut pattern = Pattern::new(connect(&mut process));
    bus.ready();
    pattern.wait_callback(0);
    let window = bus.window();
    let start=bus.call("comp.record.start",json!({"request_id":"gone","target":{"windows":[target(&window)]},"path":process.root.join("gone.mkv"),"max_seconds":10}));
    assert!(start["width"].as_u64().unwrap() > 0);
    drop(pattern);
    thread::sleep(Duration::from_millis(250));
    let status = bus.call("comp.record.status", json!({"session":start["session"]}));
    assert_eq!(status["state"], "failed", "{status}");
    assert!(status["error"].as_str().unwrap().contains("gone"));
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
            std::env::var_os("MIX_TEST_MIX_EXE").unwrap_or_else(|| "/opt/mixos/bin/mix".into()),
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
            std::env::var_os("MIX_TEST_MIX_EXE").unwrap_or_else(|| "/opt/mixos/bin/mix".into()),
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
    let row = bus.window();
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
    let row = bus.window();
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
    let row = bus.window();
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
