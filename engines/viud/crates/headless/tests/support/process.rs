// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Child, Command, Stdio},
    sync::mpsc,
    thread,
    time::Duration,
};

#[allow(dead_code)] // Only recording tests need these optional host facilities.
pub fn record_available() -> bool {
    let node = std::env::var("MIX_TEST_VIUD_RENDER_NODE")
        .unwrap_or_else(|_| "/dev/dri/renderD128".into());
    let mut missing = Vec::new();
    if let Err(error) = fs::OpenOptions::new().read(true).write(true).open(&node) {
        missing.push(format!("render node {node}: {error}"));
    }
    if !Command::new("ffmpeg").arg("-version").output().is_ok_and(|output| output.status.success()) {
        missing.push("ffmpeg is absent from PATH or cannot run".into());
    }
    if missing.is_empty() {
        return true;
    }
    let reason = missing.join("; ");
    assert!(std::env::var("MIX_TEST_REQUIRE_RECORD").as_deref() != Ok("1"), "recording tests required by MIX_TEST_REQUIRE_RECORD=1: {reason}");
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
}

impl Drop for Process {
    fn drop(&mut self) {
        if thread::panicking() {
            eprintln!(
                "viud log: {}",
                fs::read_to_string(self.root.join("viud.log")).unwrap_or_default()
            );
        }
        let _ = self.stop.send(());
        if let Some(watchdog) = self.watchdog.take() {
            let _ = watchdog.join();
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = fs::remove_dir_all(&self.root);
    }
}

impl Process {
    #[allow(dead_code)] // Only the long presentation measurement extends it.
    pub fn extend_watchdog(&mut self, duration: Duration) {
        let _ = self.stop.send(());
        if let Some(watchdog) = self.watchdog.take() {
            let _ = watchdog.join();
        }
        let pid = self.child.id();
        let (stop, stopped) = mpsc::channel();
        self.stop = stop;
        self.watchdog = Some(thread::spawn(move || {
            if stopped.recv_timeout(duration).is_err() {
                // SAFETY: killing only our owned test child.
                unsafe {
                    libc::kill(pid as i32, libc::SIGKILL);
                }
            }
        }));
    }
}

#[allow(dead_code)] // The timing target uses spawn_measurement instead.
pub fn spawn(scale: i32, explicit_node: Option<&str>, label: &str, bus: Option<&str>) -> Process {
    spawn_options(scale, explicit_node, label, bus, "60", None, true, true)
}

#[allow(dead_code)]
pub fn spawn_service(label: &str, bus: &str, service: &str) -> Process {
    spawn_options(1, None, label, Some(bus), "60", Some(service), true, true)
}

#[allow(dead_code)] // Used only by boot.rs; each integration target shares this module.
pub fn spawn_at_refresh(refresh: &str) -> Process {
    spawn_options(1, None, "refresh", None, refresh, None, true, true)
}

#[allow(dead_code)] // Only the timing integration target disables tracing.
pub fn spawn_measurement() -> Process {
    spawn_options(1, None, "presentation", None, "60", None, false, true)
}

fn spawn_options(
    scale: i32,
    explicit_node: Option<&str>,
    label: &str,
    bus: Option<&str>,
    refresh: &str,
    service: Option<&str>,
    trace: bool,
    agent: bool,
) -> Process {
    spawn_with_seat(
        scale,
        explicit_node,
        label,
        bus,
        refresh,
        service,
        trace,
        agent,
        None,
    )
}
fn spawn_with_seat(
    scale: i32,
    explicit_node: Option<&str>,
    label: &str,
    bus: Option<&str>,
    refresh: &str,
    service: Option<&str>,
    trace: bool,
    agent: bool,
    seat: Option<&str>,
) -> Process {
    spawn_with_seat_size(scale, explicit_node, label, bus, refresh, service, trace, agent, seat, "1280x720")
}

fn spawn_with_seat_size(
    scale: i32,
    explicit_node: Option<&str>,
    label: &str,
    bus: Option<&str>,
    refresh: &str,
    service: Option<&str>,
    trace: bool,
    agent: bool,
    seat: Option<&str>,
    size: &str,
) -> Process {
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
    let node =
        std::env::var("MIX_TEST_VIUD_RENDER_NODE").unwrap_or_else(|_| "/dev/dri/renderD128".into());
    let mut config = model::environment::config::base::default_settings();
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
    let log = root.join("viud.log");
    let stderr = fs::File::create(&log).unwrap();
    let mut command = Command::new(binary);
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
        .env("MIX_VIUD_HEADLESS_REFRESH_HZ", refresh)
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
    if scale != 1 {
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
    let child = command.spawn().expect("start viud");
    drop(child_control);
    let pid = child.id();
    let (stop, stopped) = mpsc::channel();
    let watchdog = thread::spawn(move || {
        if stopped.recv_timeout(Duration::from_secs(45)).is_err() {
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
    }
}

#[allow(dead_code)]
pub fn spawn_denied(label: &str, bus: &str) -> Process {
    spawn_options(1, None, label, Some(bus), "60", None, true, false)
}

#[allow(dead_code)]
pub fn spawn_inactive(label: &str, bus: &str) -> Process {
    spawn_with_seat(
        1,
        None,
        label,
        Some(bus),
        "60",
        None,
        true,
        false,
        Some("1"),
    )
}

#[allow(dead_code)]
pub fn spawn_transitions(label: &str, bus: &str) -> Process {
    spawn_with_seat(
        1,
        None,
        label,
        Some(bus),
        "60",
        None,
        true,
        false,
        Some("transitions"),
    )
}

#[allow(dead_code)] // Used by capture.rs; this module is shared with other targets.
pub fn spawn_4k(label: &str, bus: &str) -> Process {
    spawn_with_seat_size(1, None, label, Some(bus), "60", None, true, true, None, "3840x2160")
}
