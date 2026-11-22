// SPDX-License-Identifier: MIT OR Apache-2.0
//! The test owns noded with Drop and PDEATHSIG. A separate TempDir owner
//! survives test cancellation and removes the directory on pipe EOF.
#![allow(dead_code)]
#[path = "handshake.rs"]
mod handshake;
pub use handshake::read_ready;
use std::{
    fs,
    io::{Read, Write},
    os::unix::process::CommandExt,
    path::PathBuf,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

pub struct OwnedBus {
    broker: Option<Broker>,
    owner: Child,
    pub root: PathBuf,
    pub url: String,
    spawning_thread: std::thread::ThreadId,
}

impl OwnedBus {
    /// Construct and drop on the same thread, which must outlive the broker:
    /// Linux PDEATHSIG follows the forking thread, not just its process.
    pub fn start(label: &str) -> Self {
        Self::start_inner(label, true)
    }

    pub fn without_death_signal() -> Self {
        Self::start_inner("negative-control", false)
    }

    fn start_inner(label: &str, death_signal: bool) -> Self {
        let name = module_path!().split_once("::").unwrap().1;
        let owner = Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                &format!("{name}::owner_process"),
                "--ignored",
                "--nocapture",
            ])
            // Stay alive when the test pool cancels the test's process group:
            // this owner must remove its directory on pipe EOF.
            .process_group(0)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap();
        // Install the guard before any fallible readiness work.
        let mut bus = Self {
            broker: None,
            owner,
            root: PathBuf::new(),
            url: String::new(),
            spawning_thread: std::thread::current().id(),
        };
        writeln!(bus.owner.stdin.as_mut().unwrap(), "{label}").unwrap();
        let mut output = bus.owner.stdout.take().unwrap();
        let record = read_ready(
            &mut output,
            "BUS_READY ",
            Instant::now() + Duration::from_secs(10),
        )
        .expect("directory owner handshake (10 s deadline)");
        let record: serde_json::Value = serde_json::from_str(&record).unwrap();
        bus.root = record["root"].as_str().unwrap().into();
        bus.spawn_broker(death_signal);
        bus
    }

    fn spawn_broker(&mut self, death_signal: bool) {
        let bin = std::env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("noded");
        let log = fs::File::create(self.root.join("noded.log")).unwrap();
        let mut command = Command::new(bin);
        command
            .args([
                "serve",
                "--listen",
                "127.0.0.1:0",
                "--node",
                "headless-test",
                "--no-monitor",
                "--no-log",
                "--mesh-config",
            ])
            .arg(self.root.join("etc/mesh.conf.mix"))
            .env_clear()
            .env("HOME", self.root.join("home"))
            .env("MIX_ETC", self.root.join("etc"))
            .env("MIX_VAR", self.root.join("var"))
            .env("MIX_RUN", self.root.join("run"))
            .env("MIX_NODE_CONFIG", self.root.join("etc/node.conf.mix"))
            .stdin(Stdio::null())
            .stdout(log.try_clone().unwrap())
            .stderr(log);
        let parent = std::process::id() as libc::pid_t;
        if death_signal {
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
        }
        self.broker = Some(Broker(command.spawn().unwrap()));
        let broker = self.broker.as_mut().unwrap();
        let deadline = Instant::now() + Duration::from_secs(10);
        // This unique socket can only be created by the child we just spawned.
        while std::os::unix::net::UnixStream::connect(self.root.join("run/noded/bus.sock")).is_err()
        {
            assert!(
                broker.0.try_wait().unwrap().is_none(),
                "noded exited: {}",
                fs::read_to_string(self.root.join("noded.log")).unwrap()
            );
            assert!(Instant::now() < deadline, "noded readiness timeout");
            std::thread::sleep(Duration::from_millis(5));
        }
        // Noded owns the ephemeral TCP bind from the outset. Its existing
        // noded.peers diagnostic reports listener.local_addr().port(), reached
        // through the unique Unix endpoint without guessing a TCP address.
        let mut query = Broker(
            Command::new(
                std::env::var_os("MIX_TEST_MIX_EXE").unwrap_or_else(|| "/opt/mix/bin/mix".into()),
            )
            .arg(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/support/bus.mix"
            ))
            .arg("discover")
            .arg(&self.root)
            .env(
                "MIX_NODED_URL",
                format!("unix://{}", self.root.join("run/noded/bus.sock").display()),
            )
            .env("MIX_RUN", self.root.join("run"))
            .env("MIX_ETC", self.root.join("etc"))
            .env("MIX_VAR", self.root.join("var"))
            .env("MIX_NODE_CONFIG", self.root.join("etc/node.conf.mix"))
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap(),
        );
        let record = read_ready(
            &mut query.0.stdout.take().unwrap(),
            "BUS_ADDRESS ",
            deadline,
        )
        .unwrap_or_else(|error| {
            panic!(
                "bound address handshake: {error}; {}",
                fs::read_to_string(self.root.join("noded.log")).unwrap()
            )
        });
        let record: serde_json::Value = serde_json::from_str(&record).unwrap();
        let port = record["port"]
            .as_u64()
            .filter(|port| (1..=65535).contains(port))
            .expect("noded bound TCP port");
        assert_eq!(record["node"], "headless-test");
        self.url = format!("ws://127.0.0.1:{port}/ws");
        fs::write(
            self.root.join("bus.json"),
            serde_json::json!({"pid":broker.0.id(),"url":self.url}).to_string(),
        )
        .unwrap();
    }
}

impl Drop for OwnedBus {
    fn drop(&mut self) {
        drop(self.broker.take());
        drop(self.owner.stdin.take());
        // Do not kill the directory owner: it needs time to drop its TempDir.
        let deadline = Instant::now() + Duration::from_secs(1);
        while matches!(self.owner.try_wait(), Ok(None)) {
            if Instant::now() >= deadline {
                let _ = self.owner.kill();
                break;
            }
            std::thread::sleep(Duration::from_millis(5));
        }
        let _ = self.owner.wait();
        // The owner normally removes this itself. If its handshake hung and
        // the bounded fallback killed it, retire our known directory here.
        if !self.root.as_os_str().is_empty() {
            let _ = fs::remove_dir_all(&self.root);
        }
        debug_assert_eq!(
            self.spawning_thread,
            std::thread::current().id(),
            "OwnedBus must be constructed and dropped on the thread that outlives its broker"
        );
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
    let conf = root.path().join("etc/node.conf.mix");
    fs::write(&conf, serde_json::json!({
        "node":"headless-test", "mesh":"headless-test.invalid", "wg_ip":"127.0.0.1",
        "noded":{"port":0,"admission":"off","unix_socket":root.path().join("run/noded/bus.sock")}
    }).to_string()).unwrap();
    let mesh = root.path().join("etc/mesh.conf.mix");
    fs::write(&mesh, r#"{"node_name":"headless-test","peers":[]}"#).unwrap();
    println!("BUS_READY {}", serde_json::json!({"root":root.path()}));
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_to_end(&mut Vec::new()).unwrap();
    drop(root);
    // All owned resources are gone; skip libtest writing to a dead parent.
    std::process::exit(0);
}
