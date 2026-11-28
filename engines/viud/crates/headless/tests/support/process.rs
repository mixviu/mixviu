// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
use std::{
    fs,
    os::unix::{fs::PermissionsExt, process::ExitStatusExt},
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};

#[allow(dead_code)]
pub fn render_node() -> String {
    screencopy::file::testing::render_node()
}

#[allow(dead_code)] // Only recording tests need these optional host facilities.
pub fn record_available() -> bool {
    let node = render_node();
    let mut missing = Vec::new();
    if let Err(error) = fs::OpenOptions::new().read(true).write(true).open(&node) {
        missing.push(format!("render node {node}: {error}"));
    }
    if missing.is_empty() {
        return true;
    }
    let reason = missing.join("; ");
    assert!(
        std::env::var("MIX_TEST_REQUIRE_RECORD").as_deref() != Ok("1"),
        "recording tests required by MIX_TEST_REQUIRE_RECORD=1: {reason}"
    );
    eprintln!("recording test skipped: {reason}");
    false
}
pub struct Process {
    pub child: Child,
    pub root: PathBuf,
    #[allow(dead_code)]
    pub control: Option<std::os::unix::net::UnixDatagram>,
    stop: mpsc::Sender<()>,
    watchdog: Option<thread::JoinHandle<()>>,
    watchdog_fired: Arc<AtomicBool>,
    started: Instant,
    oom_before: String,
}

fn diagnostic_excerpt(path: impl AsRef<std::path::Path>) -> String {
    use std::io::{Read, Seek, SeekFrom};
    const LIMIT: usize = 16 * 1024;
    let Ok(mut file) = fs::File::open(path) else {
        return String::new();
    };
    let mut head = Vec::new();
    let _ = file.by_ref().take(LIMIT as u64).read_to_end(&mut head);
    let length = file.metadata().map(|m| m.len()).unwrap_or(0);
    if length <= (2 * LIMIT) as u64 {
        let _ = file.by_ref().take(LIMIT as u64).read_to_end(&mut head);
        return String::from_utf8_lossy(&head).into_owned();
    }
    let tail_start = (length.saturating_sub(LIMIT as u64)).max(LIMIT as u64);
    let _ = file.seek(SeekFrom::Start(tail_start));
    let mut tail = Vec::new();
    let _ = file.take(LIMIT as u64).read_to_end(&mut tail);
    format!(
        "{}\n... {} bytes omitted ...\n{}",
        String::from_utf8_lossy(&head),
        tail_start - LIMIT as u64,
        String::from_utf8_lossy(&tail)
    )
}

impl Drop for Process {
    fn drop(&mut self) {
        if thread::panicking() {
            if let Ok(Some(status)) = self.child.try_wait() {
                eprintln!(
                    "viud child exit before cleanup: pid={} status={status} code={:?} signal={:?} core_dumped={} watchdog_fired={} OOM_plausible={} oom_before={} oom_after={}",
                    self.child.id(),
                    status.code(),
                    status.signal(),
                    status.core_dumped(),
                    self.watchdog_fired.load(Ordering::SeqCst),
                    status.signal() == Some(libc::SIGKILL),
                    self.oom_before,
                    oom_evidence()
                );
            }
            eprintln!(
                "viud child status before cleanup: {:?}",
                self.child.try_wait()
            );
            eprintln!(
                "frame trace: {}",
                diagnostic_excerpt(self.root.join("frames.log"))
            );
            eprintln!(
                "viud log: {}",
                diagnostic_excerpt(self.root.join("viud.log"))
            );
        }
        let _ = self.stop.send(());
        if let Some(watchdog) = self.watchdog.take() {
            let _ = watchdog.join();
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        if thread::panicking()
            && std::env::var("MIX_TEST_CAPTURE_KEEP_FAILED").as_deref() == Ok("1")
        {
            eprintln!("failed capture artifacts retained: {}", self.root.display());
        } else {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

impl Process {
    /// Shared startup/liveness check: a signal exit has no Rust panic to log.
    #[allow(dead_code)]
    pub fn assert_running(&mut self) {
        if let Some(status) = self.child.try_wait().expect("query viud exit status") {
            panic!(
                "viud exited: pid={} status={status} code={:?} signal={:?} core_dumped={} elapsed={:?} watchdog_fired={} OOM_plausible={} (SIGKILL alone does not prove OOM); oom_before={} oom_after={}\n{}",
                self.child.id(),
                status.code(),
                status.signal(),
                status.core_dumped(),
                self.started.elapsed(),
                self.watchdog_fired.load(Ordering::SeqCst),
                status.signal() == Some(libc::SIGKILL),
                self.oom_before,
                oom_evidence(),
                diagnostic_excerpt(self.root.join("viud.log"))
            );
        }
    }

    #[allow(dead_code)] // Only the long presentation measurement extends it.
    pub fn extend_watchdog(&mut self, duration: Duration) {
        let _ = self.stop.send(());
        if let Some(watchdog) = self.watchdog.take() {
            let _ = watchdog.join();
        }
        let pid = self.child.id();
        self.watchdog_fired.store(false, Ordering::SeqCst);
        let fired = self.watchdog_fired.clone();
        let (stop, stopped) = mpsc::channel();
        self.stop = stop;
        self.watchdog = Some(thread::spawn(move || {
            if stopped.recv_timeout(duration).is_err() {
                fired.store(true, Ordering::SeqCst);
                // SAFETY: killing only our owned test child.
                unsafe {
                    libc::kill(pid as i32, libc::SIGKILL);
                }
            }
        }));
    }
}

fn oom_evidence() -> String {
    let vm = fs::read_to_string("/proc/vmstat").unwrap_or_default();
    let kills = vm
        .lines()
        .find(|line| line.starts_with("oom_kill "))
        .unwrap_or("oom_kill unavailable");
    let cgroup = fs::read_to_string("/proc/self/cgroup").unwrap_or_default();
    let events = cgroup
        .lines()
        .find_map(|line| line.strip_prefix("0::"))
        .and_then(|path| fs::read_to_string(format!("/sys/fs/cgroup{path}/memory.events")).ok())
        .unwrap_or_else(|| "cgroup memory.events unavailable".into());
    format!("{kills}; cgroup: {}", events.replace('\n', ","))
}

#[allow(dead_code)] // The timing target uses spawn_measurement instead.
pub fn spawn(scale: i32, explicit_node: Option<&str>, label: &str, bus: Option<&str>) -> Process {
    spawn_options(Options {
        scale: scale as f64,
        explicit_node,
        label,
        bus,
        ..Options::default()
    })
}

#[allow(dead_code)]
pub fn spawn_service(label: &str, bus: &str, service: &str) -> Process {
    spawn_options(Options {
        label,
        bus: Some(bus),
        service: Some(service),
        ..Options::default()
    })
}

#[allow(dead_code)] // Used only by boot.rs; each integration target shares this module.
pub fn spawn_at_refresh(refresh: &str) -> Process {
    spawn_options(Options {
        label: "refresh",
        refresh,
        ..Options::default()
    })
}

#[allow(dead_code)] // Only the timing integration target disables tracing.
pub fn spawn_measurement() -> Process {
    spawn_options(Options {
        label: "presentation",
        trace: false,
        ..Options::default()
    })
}

struct Options<'a> {
    scale: f64,
    explicit_node: Option<&'a str>,
    label: &'a str,
    bus: Option<&'a str>,
    refresh: &'a str,
    service: Option<&'a str>,
    trace: bool,
    agent: bool,
    seat: Option<&'a str>,
    size: &'a str,
    preferences: Option<serde_json::Value>,
    osd: bool,
}

impl Default for Options<'_> {
    fn default() -> Self {
        Self {
            scale: 1.0,
            explicit_node: None,
            label: "",
            bus: None,
            refresh: "60",
            service: None,
            trace: true,
            agent: true,
            seat: None,
            size: "1280x720",
            preferences: None,
            osd: false,
        }
    }
}

fn spawn_options(options: Options<'_>) -> Process {
    let Options {
        scale,
        explicit_node,
        label,
        bus,
        refresh,
        service,
        trace,
        agent,
        seat,
        size,
        preferences,
        osd,
    } = options;
    assert!(
        !label.is_empty(),
        "headless process label must be non-empty"
    );
    let binary = std::env::var_os("MIX_TEST_VIUD_EXE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            std::env::current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("viud")
        });
    assert!(
        binary.is_file(),
        "build viud with backend-headless before this test: {}",
        binary.display()
    );
    let root = std::env::temp_dir().join(format!("viud-headless-{}-{label}", std::process::id()));
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).unwrap();
    let node = render_node();
    let mut config = model::environment::config::base::default_settings();
    if std::env::var_os(paths::env::test::TEST_MAPPING_TRACE).is_some() {
        config.log_level = "info,warn,error,trace,debug".into();
    }
    // Default priority is realtime: headless must override the startup policy.
    assert_eq!(config.priority, "realtime");
    config.render_node = if explicit_node.is_some() {
        "/dev/dri/nonexistent-render-node".into()
    } else {
        node.clone()
    };
    if seat.is_some() {
        config.priority = "inherited".into();
    }
    let settings = root.join("settings.json");
    fs::write(&settings, serde_json::to_vec(&config).unwrap()).unwrap();
    if let Some(preferences) = preferences {
        if preferences["_test_two_outputs"] == true {
            fs::write(root.join("two-outputs"), b"fixture").unwrap();
        }
        if preferences["_test_two_panes"] == true {
            fs::write(root.join("two-panes"), b"fixture").unwrap();
        }
        fs::write(
            root.join("preferences.json"),
            serde_json::to_vec(&preferences).unwrap(),
        )
        .unwrap();
    }
    let log = root.join("viud.log");
    let stderr = fs::File::create(&log).unwrap();
    let mut command = Command::new(binary);
    if label.starts_with("record-privacy-") || label.starts_with("fractional-") {
        command.env(paths::env::test::TEST_RECORD_FIXTURE, &root);
    }
    if label == "cold-record" {
        fs::write(root.join("record-input"), "cold-readback").unwrap();
        command.env(paths::env::test::TEST_RECORD_FIXTURE, &root);
    }
    if label == "record-privacy-cursor-epoch" {
        // Most tests deliberately have no theme, so every Named shape falls
        // back to the same built-in arrow. Give this test one generated shape
        // to make Named/forced privacy assertions discriminate decoded pixels.
        let theme = root.join("cursors/headless-test");
        fs::create_dir_all(theme.join("cursors")).unwrap();
        fs::write(
            theme.join("index.theme"),
            "Inherits=missing-headless-theme\n",
        )
        .unwrap();
        let mut bytes = b"Xcur".to_vec();
        for word in [
            16_u32, 1, 1, 0xfffd0002, 24, 28, 36, 0xfffd0002, 24, 1, 24, 24, 0, 0, 10,
        ] {
            bytes.extend_from_slice(&word.to_le_bytes());
        }
        for _ in 0..24 * 24 {
            bytes.extend_from_slice(&0xff3399ee_u32.to_le_bytes());
        }
        fs::write(theme.join("cursors/crosshair"), bytes).unwrap();
    }
    if osd {
        let workspace = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../..");
        let viu = std::env::var_os(paths::env::test::TEST_VIU_EXE)
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                std::env::current_exe()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .parent()
                    .unwrap()
                    .join("viu")
            });
        assert!(
            viu.is_file(),
            "build viu before the OSD tests: {}",
            viu.display()
        );
        command
            .env(
                paths::env::test::TEST_VIUD_OSD_BUNDLE,
                workspace.join("apps/osd"),
            )
            .env(paths::env::test::TEST_VIU_EXE, viu)
            .env("MIX_VIU_COMP_SERVICE", "comp-headless");
    } else {
        command
            .env_remove(paths::env::test::TEST_VIUD_OSD_BUNDLE)
            .env_remove(paths::env::test::TEST_VIU_EXE);
    }
    if std::env::var_os(paths::env::test::TEST_PRE_DRAIN_CHILD).is_none() {
        command.env_remove(paths::env::test::TEST_PRE_DRAIN_FRAME);
    }
    command
        .args([
            "--socket",
            "wayland-headless-test",
            "--config-file",
        ])
        .arg(settings)
        .env("XDG_RUNTIME_DIR", &root)
        .env("XDG_CONFIG_HOME", root.join("config"))
        .env("XDG_STATE_HOME", root.join("state"))
        .env("XDG_DATA_HOME", root.join("data"))
        .env("XDG_CACHE_HOME", root.join("cache"))
        .env("HOME", &root)
        .env("MIX_ETC", root.join("etc"))
        .env("MIX_VAR", root.join("var"))
        .env("MIX_RUN", root.join("run"))
        .env("MIX_NODED_URL", bus.unwrap_or("ws://127.0.0.1:1"))
        .env("MIX_VIUD_HEADLESS_REFRESH_HZ", if label == "record-privacy-cursor-epoch" { "30" } else { refresh })
        .env("MIX_TEST_VIUD_AGENT_SEAT_ALLOW", if agent { format!("viu,testkit-input-probe,testkit-ssd-client,testkit-dnd-source,ced-headless-probe,{}", std::env::current_exe().unwrap().file_name().unwrap().to_string_lossy()) } else { String::new() })
        .env("MIX_VIUD_FRAME_TRACE", if trace { "1" } else { "0" })
        .env("MIX_VIUD_FRAME_TRACE_FILE", root.join("frames.log"))
        .env("MIX_VIUD_FRAME_TRACE_LIMIT", "0")
        // Fixed built-in 24px arrow: no host theme or cursor animation.
        .env("XCURSOR_PATH", root.join("cursors"))
        .env("XCURSOR_THEME", "headless-test")
        .env("XCURSOR_SIZE", "24")
        // Deliberately request XWayland: --headless must keep it off.
        .env("MIX_VIUD_XWAYLAND", "1")
        .env_remove("NOTIFY_SOCKET")
        .env_remove("MIX_VT")
        .stdout(Stdio::from(stderr.try_clone().unwrap()))
        .stderr(stderr);
    if seat.is_some() {
        command
            .args(["--kms", "--kms-confirm", "--bus-service", "comp-headless"])
            .env("MIX_TEST_VIUD_SEAT_INACTIVE", seat.unwrap_or("1"))
            .env("MIX_VIUD_OFFSCREEN_SIZE", "1280x720")
            .env("MIX_VIUD_XWAYLAND", "0");
    } else {
        command.args(["--headless", "--size", size]);
    }
    if let Some(node) = explicit_node {
        command.args(["--render-node", node]);
    }
    if scale != 1.0 {
        command.args(["--scale", &scale.to_string()]);
    }
    if let Some(service) = service {
        command.args(["--bus-service", service]);
    }
    let mut control = None;
    let mut child_control = None;
    if seat == Some("transitions") {
        use std::os::{fd::AsRawFd, unix::process::CommandExt};
        let (parent, child) = std::os::unix::net::UnixDatagram::pair().unwrap();
        parent
            .set_read_timeout(Some(Duration::from_secs(8)))
            .unwrap();
        let fd = child.as_raw_fd();
        command.env("MIX_TEST_VIUD_SEAT_INACTIVE", "fd:198");
        // SAFETY: transfer only our owned fixture socket into the test child.
        unsafe {
            command.pre_exec(move || {
                if libc::dup2(fd, 198) < 0 || libc::fcntl(198, libc::F_SETFD, 0) < 0 {
                    return Err(std::io::Error::last_os_error());
                }
                Ok(())
            });
        }
        control = Some(parent);
        child_control = Some(child);
    }
    let started = Instant::now();
    let oom_before = oom_evidence();
    let child = command.spawn().expect("start viud");
    drop(child_control);
    let pid = child.id();
    let (stop, stopped) = mpsc::channel();
    let watchdog_fired = Arc::new(AtomicBool::new(false));
    let fired = watchdog_fired.clone();
    let watchdog = thread::spawn(move || {
        if stopped.recv_timeout(Duration::from_secs(45)).is_err() {
            fired.store(true, Ordering::SeqCst);
            // Break a stuck Wayland roundtrip too; never leave the child behind.
            unsafe {
                libc::kill(pid as i32, libc::SIGKILL);
            }
        }
    });
    Process {
        control,
        child,
        root,
        stop,
        watchdog: Some(watchdog),
        watchdog_fired,
        started,
        oom_before,
    }
}

#[allow(dead_code)]
pub fn spawn_denied(label: &str, bus: &str) -> Process {
    spawn_options(Options {
        label,
        bus: Some(bus),
        agent: false,
        ..Options::default()
    })
}

#[allow(dead_code)]
pub fn spawn_inactive(label: &str, bus: &str) -> Process {
    spawn_options(Options {
        label,
        bus: Some(bus),
        agent: false,
        seat: Some("1"),
        ..Options::default()
    })
}

#[allow(dead_code)]
pub fn spawn_transitions(label: &str, bus: &str) -> Process {
    spawn_options(Options {
        label,
        bus: Some(bus),
        agent: false,
        seat: Some("transitions"),
        ..Options::default()
    })
}

#[allow(dead_code)] // Used by capture.rs; this module is shared with other targets.
pub fn spawn_4k(label: &str, bus: &str) -> Process {
    spawn_options(Options {
        label,
        bus: Some(bus),
        size: "3840x2160",
        osd: std::env::var(paths::env::test::TEST_CAPTURE_OSD).as_deref() == Ok("1"),
        ..Options::default()
    })
}

#[allow(dead_code)]
pub fn spawn_size(label: &str, bus: &str, size: &str) -> Process {
    spawn_options(Options {
        label,
        bus: Some(bus),
        size,
        ..Options::default()
    })
}

#[allow(dead_code)] // Used by window motion's isolated lifecycle fixture.
pub fn spawn_preferences(label: &str, bus: &str, preferences: serde_json::Value) -> Process {
    spawn_preferences_size(label, bus, "1280x720", preferences)
}

pub fn spawn_scale_probe(label: &str, bus: &str) -> Process {
    spawn_options(Options {
        label,
        bus: Some(bus),
        scale: 2.5,
        preferences: Some(serde_json::json!({"fractional_invisible":"full", "workspaces_count":2})),
        ..Options::default()
    })
}

#[allow(dead_code)]
pub fn spawn_preferences_size(
    label: &str,
    bus: &str,
    size: &str,
    preferences: serde_json::Value,
) -> Process {
    spawn_options(Options {
        label,
        bus: Some(bus),
        size,
        preferences: Some(preferences),
        ..Options::default()
    })
}

#[allow(dead_code)]
pub fn spawn_osd(label: &str, bus: &str, size: &str) -> Process {
    spawn_options(Options {
        label,
        bus: Some(bus),
        osd: true,
        size,
        ..Options::default()
    })
}
