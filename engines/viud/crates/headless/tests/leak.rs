// SPDX-License-Identifier: MIT OR Apache-2.0
//! Real noded lifecycle checks, independent of render-node availability.
#[path = "support/owned_bus.rs"]
mod owned_bus;
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::process::CommandExt,
    path::Path,
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};

struct Parent(Child);
impl Drop for Parent {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
#[ignore = "private re-exec entry; requires a mode on stdin"]
fn fixture_parent() {
    let mut mode = String::new();
    std::io::stdin().read_line(&mut mode).unwrap();
    let bus = owned_bus::OwnedBus::start(mode.trim());
    let record: serde_json::Value =
        serde_json::from_slice(&fs::read(bus.root.join("bus.json")).unwrap()).unwrap();
    println!("CHILD_READY {} {}", record["pid"], bus.root.display());
    std::io::stdout().flush().unwrap();
    std::io::stdin().read_line(&mut String::new()).unwrap();
    if mode.trim() == "panic" {
        panic!("injected panic with owned broker");
    }
    drop(bus);
}

fn gone(pid: u32, root: &Path) -> bool {
    let running = fs::read(format!("/proc/{pid}/cmdline")).is_ok_and(|cmd| !cmd.is_empty());
    !running && !root.exists()
}

fn within(pid: u32, root: &Path, bound: Duration) -> bool {
    let deadline = Instant::now() + bound;
    while !gone(pid, root) {
        if Instant::now() >= deadline {
            return false;
        }
        std::thread::yield_now();
    }
    true
}

fn check(mode: &str) {
    let mut parent = Parent(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "fixture_parent", "--ignored", "--nocapture"])
            .process_group(0)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    writeln!(parent.0.stdin.as_mut().unwrap(), "{mode}").unwrap();
    let mut output = BufReader::new(parent.0.stdout.take().unwrap());
    let mut line = String::new();
    loop {
        line.clear();
        assert_ne!(
            output.read_line(&mut line).unwrap(),
            0,
            "fixture exited before handshake"
        );
        if line.starts_with("CHILD_READY ") {
            break;
        }
    }
    let parts: Vec<_> = line.split_whitespace().collect();
    let pid: u32 = parts[1].parse().unwrap();
    let root = Path::new(parts[2]);
    assert!(!gone(pid, root), "handshake must name a live broker");
    if mode == "group-kill" {
        // SAFETY: the fixture has its own process group, created above.
        assert_eq!(unsafe { libc::kill(-(parent.0.id() as i32), libc::SIGKILL) }, 0);
    } else if mode != "panic" && mode != "normal" {
        parent.0.kill().unwrap();
    } else if mode == "panic" {
        writeln!(parent.0.stdin.as_mut().unwrap(), "panic now").unwrap();
    } else if mode == "normal" {
        drop(parent.0.stdin.take());
    }
    let status = parent.0.wait().unwrap();
    assert_eq!(status.success(), mode == "normal");
    let cleaned = within(pid, root, Duration::from_secs(2));
    if mode == "negative-control" {
        assert!(
            !cleaned,
            "negative control unexpectedly passed the leak oracle"
        );
        std::os::unix::net::UnixStream::connect(root.join("release.sock")).unwrap();
        assert!(
            within(pid, root, Duration::from_secs(2)),
            "control cleanup failed"
        );
    } else {
        assert!(
            cleaned,
            "{mode}: noded {pid} or {} survived 2 s",
            root.display()
        );
    }
    eprintln!("LIFETIME mode={mode} noded={pid} cleaned={cleaned}");
}

#[test]
fn sigkill_reaps_broker_and_directory() {
    check("kill");
}
#[test]
fn cancelled_process_group_reaps_broker_and_directory() {
    check("group-kill");
}
#[test]
fn panic_reaps_broker_and_directory() {
    check("panic");
}
#[test]
fn normal_exit_reaps_broker_and_directory() {
    check("normal");
}
#[test]
fn missing_lifetime_tie_fails_oracle() {
    check("negative-control");
}
