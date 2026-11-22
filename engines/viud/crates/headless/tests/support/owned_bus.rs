// SPDX-License-Identifier: MIT OR Apache-2.0
//! The test owns noded with Drop and PDEATHSIG. A separate TempDir owner
//! survives test cancellation and removes the directory on pipe EOF.
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
    broker: Option<Broker>,
    owner: Child,
    pub root: PathBuf,
    pub url: String,
}

impl OwnedBus {
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
        };
        writeln!(bus.owner.stdin.as_mut().unwrap(), "{label}").unwrap();
        let output = bus.owner.stdout.take().unwrap();
        for line in BufReader::new(output).lines() {
            let line = line.unwrap();
            if let Some(record) = line.strip_prefix("BUS_READY ") {
                let record: serde_json::Value = serde_json::from_str(record).unwrap();
                bus.root = record["root"].as_str().unwrap().into();
                bus.url = record["url"].as_str().unwrap().into();
                bus.spawn_broker(death_signal, record["port"].as_u64().unwrap());
                return bus;
            }
        }
        panic!("broker owner exited before readiness");
    }

    fn spawn_broker(&mut self, death_signal: bool, port: u64) {
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
                &format!("127.0.0.1:{port}"),
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
            std::thread::yield_now();
        }
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
    drop(listener);
    println!(
        "BUS_READY {}",
        serde_json::json!({"root":root.path(),"url":format!("ws://127.0.0.1:{port}/ws"),"port":port})
    );
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_to_end(&mut Vec::new()).unwrap();
    drop(root);
    // All owned resources are gone; skip libtest writing to a dead parent.
    std::process::exit(0);
}
