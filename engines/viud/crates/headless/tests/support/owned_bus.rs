// SPDX-License-Identifier: MIT OR Apache-2.0
//! A re-exec owner survives the test's SIGKILL long enough to reap noded and
//! drop its TempDir. EOF on the private stdin pipe is the lifetime tie.
#![allow(dead_code)]
use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
    os::unix::process::CommandExt,
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

pub struct OwnedBus {
    owner: Child,
    pub root: PathBuf,
    pub url: String,
}

impl OwnedBus {
    pub fn start(label: &str) -> Self {
        let name = module_path!().split_once("::").unwrap().1;
        let owner = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                &format!("{name}::owner_process"),
                "--ignored",
                "--nocapture",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        // Install the guard before any fallible readiness work.
        let mut bus = Self {
            owner,
            root: PathBuf::new(),
            url: String::new(),
        };
        writeln!(bus.owner.stdin.as_mut().unwrap(), "{label}").unwrap();
        let output = bus.owner.stdout.take().unwrap();
        for line in BufReader::new(output).lines() {
            let line = line.unwrap();
            if let Some(record) = line.strip_prefix("BUS_READY ") {
                let record: serde_json::Value = serde_json::from_str(record).unwrap();
                bus.root = record["root"].as_str().unwrap().into();
                bus.url = record["url"].as_str().unwrap().into();
                return bus;
            }
        }
        panic!("broker owner exited before readiness");
    }
}

impl Drop for OwnedBus {
    fn drop(&mut self) {
        drop(self.owner.stdin.take());
        // The owner kills and waits before removing the directory. Do not kill
        // it here: even panic cleanup must leave it time to drop the TempDir.
        let _ = self.owner.wait();
    }
}

struct Broker(Child);
impl Drop for Broker {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
#[ignore = "private re-exec entry; requires a label on stdin"]
fn owner_process() {
    let mut label = String::new();
    std::io::stdin().read_line(&mut label).unwrap();
    let root = tempfile::Builder::new()
        .prefix("headless-bus-")
        .tempdir()
        .unwrap();
    for dir in ["etc", "var", "run/noded", "home"] {
        fs::create_dir_all(root.path().join(dir)).unwrap();
    }
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    let conf = root.path().join("etc/node.conf.mix");
    fs::write(&conf, serde_json::json!({
        "node":"headless-test", "mesh":"headless-test.invalid", "wg_ip":"127.0.0.1",
        "noded":{"port":port,"admission":"off","unix_socket":root.path().join("run/noded/bus.sock")}
    }).to_string()).unwrap();
    let mesh = root.path().join("etc/mesh.conf.mix");
    fs::write(&mesh, r#"{"node_name":"headless-test","peers":[]}"#).unwrap();
    let bin = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("noded");
    let mut command = Command::new(bin);
    command
        .args([
            "serve",
            "--listen",
            &format!("127.0.0.1:{port}"),
            "--node",
            "headless-test",
            "--no-monitor",
            "--no-log",
            "--mesh-config",
        ])
        .arg(&mesh)
        .env_clear()
        .env("HOME", root.path().join("home"))
        .env("MIX_ETC", root.path().join("etc"))
        .env("MIX_VAR", root.path().join("var"))
        .env("MIX_RUN", root.path().join("run"))
        .env("MIX_NODE_CONFIG", &conf)
        .stdin(Stdio::null())
        .stdout(fs::File::create(root.path().join("noded.log")).unwrap())
        .stderr(Stdio::null());
    let parent = std::process::id() as libc::pid_t;
    // SAFETY: only async-signal-safe syscalls in the fork/exec window.
    unsafe {
        command.pre_exec(move || {
            if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGKILL) == -1 {
                return Err(std::io::Error::last_os_error());
            }
            if libc::getppid() != parent {
                return Err(std::io::Error::from_raw_os_error(libc::ECHILD));
            }
            Ok(())
        });
    }
    drop(listener);
    let mut broker = Broker(command.spawn().unwrap());
    let url = format!("ws://127.0.0.1:{port}/ws");
    let deadline = Instant::now() + Duration::from_secs(10);
    // Wait for this child's Unix socket, not an arbitrary process on the port.
    while std::os::unix::net::UnixStream::connect(root.path().join("run/noded/bus.sock")).is_err() {
        let mut pipe = libc::pollfd {
            fd: libc::STDIN_FILENO,
            events: libc::POLLHUP,
            revents: 0,
        };
        // SAFETY: one valid pollfd, no blocking. Also clean up when the test
        // dies during startup, before the readiness handshake.
        if unsafe { libc::poll(&mut pipe, 1, 0) } > 0 && pipe.revents & libc::POLLHUP != 0 {
            return;
        }
        assert!(
            broker.0.try_wait().unwrap().is_none(),
            "noded exited: {}",
            fs::read_to_string(root.path().join("noded.log")).unwrap()
        );
        assert!(Instant::now() < deadline, "noded readiness timeout");
        std::thread::yield_now();
    }
    fs::write(
        root.path().join("bus.json"),
        serde_json::json!({"pid":broker.0.id(),"url":url}).to_string(),
    )
    .unwrap();
    let release = (label.trim() == "negative-control")
        .then(|| std::os::unix::net::UnixListener::bind(root.path().join("release.sock")).unwrap());
    println!(
        "BUS_READY {}",
        serde_json::json!({"root":root.path(),"url":url})
    );
    std::io::stdout().flush().unwrap();
    // The control removes the pipe tie and waits for an explicit release from
    // the regression controller. No timing sleep stands in for a handshake.
    std::io::stdin().read_to_end(&mut Vec::new()).unwrap();
    if let Some(release) = release {
        release.set_nonblocking(true).unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        while release.accept().is_err() && Instant::now() < deadline {
            std::thread::yield_now();
        }
    }
    drop(broker);
    drop(root);
    // The parent may already be dead, with its stdout pipe closed. All owned
    // resources are gone; skip libtest's final write to that broken pipe.
    std::process::exit(0);
}
