// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! CBC-only owned headless processes; never a desktop or VT.
use serde_json::{Value, json};
use std::{
    fs,
    os::unix::net::UnixStream,
    path::{Path, PathBuf},
    process::{Child, Command},
    time::{Duration, Instant},
};
#[path = "../../viud/crates/headless/tests/support/pattern.rs"]
#[allow(dead_code)]
mod pattern;
struct Owned {
    child: Child,
    log: PathBuf,
    stop: std::sync::mpsc::Sender<()>,
    watchdog: Option<std::thread::JoinHandle<()>>,
}
impl Owned {
    fn new(child: Child, log: PathBuf) -> Self {
        let pid = child.id();
        let (stop, receiver) = std::sync::mpsc::channel();
        let watchdog = std::thread::spawn(move || {
            if receiver.recv_timeout(Duration::from_secs(60)).is_err() {
                unsafe {
                    libc::kill(pid as i32, libc::SIGKILL);
                }
            }
        });
        Self {
            child,
            log,
            stop,
            watchdog: Some(watchdog),
        }
    }
}
impl Drop for Owned {
    fn drop(&mut self) {
        let _ = self.stop.send(());
        if let Some(w) = self.watchdog.take() {
            let _ = w.join();
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        if std::thread::panicking() {
            eprintln!(
                "{}:\n{}",
                self.log.display(),
                fs::read_to_string(&self.log).unwrap_or_default()
            );
        }
    }
}
fn spawn(root: &Path, binary: &Path, old: bool, url: &str, parent: Option<&Path>) -> Owned {
    let config = model::environment::config::base::default_settings();
    fs::write(
        root.join("settings.json"),
        serde_json::to_vec(&config).unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("preferences.json"),
        json!({"scene_host":old}).to_string(),
    )
    .unwrap();
    let log = root.join("viud.log");
    let file = fs::File::create(&log).unwrap();
    let mut command = Command::new(binary);
    if let Some(parent) = parent {
        command
            .args(["--nested", "--bus-service", "comp-headless"])
            .env("WAYLAND_DISPLAY", parent.join("wayland-furniture-test"));
    } else {
        command
            .args([
                "--headless",
                "--size",
                "1280x720",
                "--render-node",
                "/dev/dri/renderD128",
            ])
            .env_remove("WAYLAND_DISPLAY")
            .env_remove("DISPLAY");
    }
    let child = command
        .args(["--socket", "wayland-furniture-test", "--config-file"])
        .arg(root.join("settings.json"))
        .env("XDG_RUNTIME_DIR", root)
        .env("XDG_CONFIG_HOME", root.join("config"))
        .env("XDG_STATE_HOME", root.join("state"))
        .env("XDG_DATA_HOME", root.join("data"))
        .env("XDG_CACHE_HOME", root.join("cache"))
        .env("HOME", root)
        .env("MIX_ETC", root.join("etc"))
        .env("MIX_RUN", root.join("run"))
        .env("MIX_VAR", root.join("var"))
        .env("MIX_NODED_URL", url)
        .env("VIUD_FRAME_TRACE", "1")
        .env("VIUD_FRAME_TRACE_LIMIT", "0")
        .env("VIUD_FRAME_TRACE_FILE", root.join("frames.log"))
        .env_remove("NOTIFY_SOCKET")
        .env_remove("MIX_VT")
        .stdout(file.try_clone().unwrap())
        .stderr(file)
        .spawn()
        .unwrap();
    Owned::new(child, log)
}
async fn ready(client: &bus::SupervisedClient, to: &str, command: &str) -> Value {
    let until = Instant::now() + Duration::from_secs(30);
    loop {
        if let Ok(Ok(value)) = tokio::time::timeout(
            Duration::from_millis(300),
            client.call(to, command, json!({})),
        )
        .await
        {
            return value;
        }
        assert!(Instant::now() < until, "{command} not ready");
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
fn ticks(pid: u32) -> u64 {
    let s = fs::read_to_string(format!("/proc/{pid}/stat")).unwrap();
    let words: Vec<_> = s.rsplit_once(')').unwrap().1.split_whitespace().collect();
    words[11].parse::<u64>().unwrap() + words[12].parse::<u64>().unwrap()
}
async fn exercise(old: bool, parent: Option<&Path>) {
    let broker = test_broker::Broker::start();
    let root = tempfile::tempdir().unwrap();
    let binary = PathBuf::from(
        std::env::var(if old {
            "COMPD_INPROC_TEST_BINARY"
        } else {
            "VIUD_NATIVE_TEST_BINARY"
        })
        .expect("build gate supplies exact binaries"),
    );
    let comp = spawn(root.path(), &binary, old, &broker.url, parent);
    let socket = root.path().join("wayland-furniture-test");
    let until = Instant::now() + Duration::from_secs(30);
    while !socket.exists() {
        assert!(Instant::now() < until, "headless socket timed out");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let client = bus::SupervisedClient::connect_options("furniture-gate", &broker.url)
        .connect()
        .await
        .unwrap();
    ready(&client, "comp-headless", "comp.ping").await;
    let mut viu = None;
    if !old {
        let log = root.path().join("viu.log");
        let file = fs::File::create(&log).unwrap();
        let child = Command::new(env!("CARGO_BIN_EXE_viu"))
            .arg("--host")
            .env("WAYLAND_DISPLAY", "wayland-furniture-test")
            .env("XDG_RUNTIME_DIR", root.path())
            .env("HOME", root.path())
            .env("XDG_CONFIG_HOME", root.path().join("config"))
            .env("XDG_STATE_HOME", root.path().join("state"))
            .env("XDG_DATA_HOME", root.path().join("data"))
            .env("MIX_NODED_URL", &broker.url)
            .env("VIU_COMP_SERVICE", "comp-headless")
            .env_remove("DISPLAY")
            .env_remove("NOTIFY_SOCKET")
            .env_remove("MIX_VT")
            .stdout(file.try_clone().unwrap())
            .stderr(file)
            .spawn()
            .unwrap();
        viu = Some(Owned::new(child, log));
    }
    println!(
        "SHELL_PING old={old} {}",
        ready(&client, "shell", "shell.ping").await
    );
    let start = Instant::now();
    let source = include_str!("fixtures/panel-render.scene.mix");
    let loaded = client
        .call(
            "shell",
            "shell.scene.load",
            json!({"source":source,"output":if parent.is_some() {"winit"} else {"headless-0"}}),
        )
        .await
        .unwrap();
    println!("SCENE_LOAD old={old} {loaded}");
    let until = Instant::now() + Duration::from_secs(20);
    loop {
        let surfaces = client
            .call(
                "comp-headless",
                "comp.props.get",
                json!({"path":"surfaces"}),
            )
            .await
            .unwrap();
        if surfaces.as_object().is_some_and(|rows| {
            rows.values().any(|row| {
                row["mapped"] == true && row["layer"]["exclusive_zone"].as_i64().unwrap_or(0) > 0
            })
        }) {
            println!(
                "MAPPED_LAYER old={old} elapsed_ms={:.3} {surfaces}",
                start.elapsed().as_secs_f64() * 1000.0
            );
            break;
        }
        if old {
            let scenes = client
                .call("shell", "shell.scene.get", json!({"scene":"panel"}))
                .await
                .unwrap();
            if scenes["applied_revision"].as_u64().unwrap_or(0) > 0 {
                println!(
                    "INPROC_APPLIED elapsed_ms={:.3}",
                    start.elapsed().as_secs_f64() * 1000.0
                );
                break;
            }
        }
        assert!(Instant::now() < until, "panel did not map");
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    let outputs = client
        .call("comp-headless", "comp.props.get", json!({"path":"outputs"}))
        .await
        .unwrap();
    println!("OUTPUT_RESERVATIONS old={old} {outputs}");
    assert!(
        outputs
            .as_object()
            .unwrap()
            .values()
            .any(|row| row["usable"]["height"].as_f64().unwrap_or(f64::MAX)
                < row["height"].as_f64().unwrap_or(0.0))
    );
    if !old && parent.is_some() {
        client
            .subscribe_topic("comp-headless.corner.clicked.v2")
            .await
            .unwrap();
        let mut corner_input = client.incoming().unwrap();
        client.call("shell","shell.scene.load",json!({"source":include_str!("../../../share/scenes/launcher/scene.mix"),"output":"winit"})).await.unwrap();
        let before = client
            .call("shell", "shell.props.get", json!({}))
            .await
            .unwrap()["corner_events"]
            .as_u64()
            .unwrap();
        client
            .call(
                "comp-headless",
                "comp.input.pointer.move",
                json!({"seat":"human","output":"winit","x":1,"y":1,"corners":true}),
            )
            .await
            .unwrap();
        client
            .call(
                "comp-headless",
                "comp.input.pointer.button",
                json!({"seat":"human","action":"click"}),
            )
            .await
            .unwrap();
        let clicked = tokio::time::timeout(Duration::from_secs(3), corner_input.recv())
            .await
            .unwrap()
            .unwrap();
        println!(
            "CORNER_CLICK_WIRE headers={:?} body={}",
            clicked.headers, clicked.body
        );
        let until = Instant::now() + Duration::from_secs(5);
        loop {
            let props = client
                .call("shell", "shell.props.get", json!({}))
                .await
                .unwrap();
            let count = props["corner_events"].as_u64().unwrap();
            if count > before
                && ["consumed", "clicked", "released"].iter().all(|phase| {
                    props["corner_phases"]
                        .as_array()
                        .unwrap()
                        .iter()
                        .any(|v| v == phase)
                })
            {
                println!("SCRIPTED_CORNER_DELIVERED before={before} after={count}");
                println!("EARLY_CLICK_PHASES {}", props["corner_phases"]);
                println!("CORNER_FURNITURE_STATE {props}");
                break;
            }
            assert!(Instant::now() < until, "corner topic missing: {props}");
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        let until = Instant::now() + Duration::from_secs(10);
        loop {
            let surfaces = client
                .call(
                    "comp-headless",
                    "comp.props.get",
                    json!({"path":"surfaces"}),
                )
                .await
                .unwrap();
            if surfaces.as_object().unwrap().values().any(|s| {
                s["mapped"] == true
                    && s["role"] == "layer"
                    && s["width"] == 440.0
                    && s["focused"] == true
            }) {
                println!("LAUNCHER_MAPPED_AND_FOCUSED {surfaces}");
                break;
            }
            assert!(
                Instant::now() < until,
                "launcher failed to map/take one-shot pinned focus: {surfaces}"
            );
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        for (mode, want_mapped) in [("hidden", false), ("pinned", true)] {
            client
                .call(
                    "shell",
                    "shell.panel.mode",
                    json!({"edge":"left","output":"o_winit","mode":mode}),
                )
                .await
                .unwrap();
            let until = Instant::now() + Duration::from_secs(10);
            loop {
                let surfaces = client
                    .call(
                        "comp-headless",
                        "comp.props.get",
                        json!({"path":"surfaces"}),
                    )
                    .await
                    .unwrap();
                let visible = surfaces.as_object().unwrap().values().any(|s| {
                    s["mapped"] == true
                        && s["role"] == "layer"
                        && s["width"] == 440.0
                        && (!want_mapped || s["focused"] == true)
                });
                if visible == want_mapped {
                    println!("LAUNCHER_REMAP mode={mode} {surfaces}");
                    break;
                }
                assert!(
                    Instant::now() < until,
                    "launcher remap failed mode={mode}: {surfaces}"
                );
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        }
        client.call("shell", "shell.scene.load", json!({"source":include_str!("../../../share/scenes/toast/scene.mix"),"output":"o_winit"})).await.unwrap();
        for visible in [true, false, true, false] {
            client
                .call(
                    "shell",
                    if visible {
                        "shell.toast.show"
                    } else {
                        "shell.toast.hide"
                    },
                    json!({"scene":"toast","output":"o_winit","h":132}),
                )
                .await
                .unwrap();
            let until = Instant::now() + Duration::from_secs(10);
            loop {
                let surfaces = client
                    .call(
                        "comp-headless",
                        "comp.props.get",
                        json!({"path":"surfaces"}),
                    )
                    .await
                    .unwrap();
                let mapped = surfaces.as_object().unwrap().values().any(|s| {
                    s["mapped"] == true
                        && s["width"] == 360.0
                        && s["height"] == 132.0
                        && s["focused"] == false
                });
                if mapped == visible {
                    println!("TOAST_MAPPED visible={visible}");
                    break;
                }
                assert!(
                    Instant::now() < until,
                    "toast mapping failed visible={visible}: {surfaces}"
                );
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        }
    }
    if parent.is_some() {
        return;
    }
    // Keep the furniture idle; a separate shm client requests paced frames.
    let mut pattern = pattern::Pattern::new(UnixStream::connect(&socket).unwrap());
    pattern.wait_callback(0);
    tokio::time::sleep(Duration::from_millis(400)).await;
    let trace = root.path().join("frames.log");
    let offset = fs::metadata(&trace).map(|m| m.len() as usize).unwrap_or(0);
    let begin_ticks = ticks(comp.child.id());
    let wall = Instant::now();
    for _ in 0..180 {
        let before = pattern.state.callbacks;
        pattern.commit(true);
        pattern.wait_callback(before);
    }
    let cpu_ticks = ticks(comp.child.id()) - begin_ticks;
    let seconds = wall.elapsed().as_secs_f64();
    tokio::time::sleep(Duration::from_millis(150)).await;
    let data = fs::read_to_string(trace).unwrap();
    let mut cpu = 0_u64;
    let mut frames = 0_u64;
    for line in data
        .get(offset..)
        .unwrap_or("")
        .lines()
        .filter(|l| l.contains("stage=headless_frame_cpu "))
    {
        if let Some(value) = line
            .split_whitespace()
            .find_map(|part| part.strip_prefix("cpu_us="))
            .and_then(|s| s.parse::<u64>().ok())
        {
            cpu += value;
            frames += 1;
        }
    }
    assert!(frames > 0, "frame instrumentation absent");
    let hz = unsafe { libc::sysconf(libc::_SC_CLK_TCK) } as f64;
    println!(
        "PANEL_IDLE_CPU old={old} frames={frames} wall_s={seconds:.3} main_thread_us_per_frame={:.3} process_ms_per_frame={:.3} proc_tick_hz={hz}",
        cpu as f64 / frames as f64,
        cpu_ticks as f64 * 1000.0 / hz / frames as f64
    );
    if let Some(viu) = &viu {
        println!("VIU_FULL_LOG\n{}", fs::read_to_string(&viu.log).unwrap());
    }
    drop(pattern);
    drop(viu);
    drop(comp);
    client.close().await;
}
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "CBC GPU process gate; requires exact native/default viud binaries"]
async fn native_client_maps_reserves_pings_receives_corners_and_measures_idle_cpu() {
    exercise(false, None).await;
    exercise(true, None).await;
    // A headless human seat deliberately has no device sources. Exercise the
    // real early input path in an owned nested compositor inside a headless
    // parent, preserving the existing headless seat refusal contract.
    let broker = test_broker::Broker::start();
    let root = tempfile::tempdir().unwrap();
    let binary = PathBuf::from(std::env::var("VIUD_NATIVE_TEST_BINARY").unwrap());
    let _parent = spawn(root.path(), &binary, false, &broker.url, None);
    let until = Instant::now() + Duration::from_secs(15);
    while !root.path().join("wayland-furniture-test").exists() {
        assert!(Instant::now() < until);
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    exercise(false, Some(root.path())).await;
}
