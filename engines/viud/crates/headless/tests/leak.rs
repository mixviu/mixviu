// SPDX-License-Identifier: MIT OR Apache-2.0
//! Temporary baseline reproduction, replaced by lifecycle regression tests.
#[path = "support/bus.rs"]
mod bus;

#[test]
fn legacy_child() {
    let Ok(mode) = std::env::var("LEAK_MODE") else { return };
    let root = std::env::temp_dir().join(format!("headless-bus-{}-repro", std::process::id()));
    let output = std::process::Command::new("/opt/mix/bin/mix")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/support/bus.mix"))
        .arg("start").arg(&root).arg(bus::binary("noded"))
        .output().unwrap();
    assert!(output.status.success(), "{output:?}");
    let record: serde_json::Value = serde_json::from_slice(&std::fs::read(root.join("bus.json")).unwrap()).unwrap();
    println!("LEAK_READY {} {}", record["pid"], root.display());
    use std::io::Write;
    std::io::stdout().flush().unwrap();
    if mode == "panic" { panic!("injected panic after broker start, before Bus owns it"); }
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).unwrap();
}

#[test]
fn reproduce_legacy_leaks() {
    use std::io::{BufRead, BufReader};
    for mode in ["kill", "panic"] {
        let mut parent = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "legacy_child", "--nocapture"])
            .env("LEAK_MODE", mode)
            .stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).spawn().unwrap();
        let mut lines = BufReader::new(parent.stdout.take().unwrap()).lines();
        let line = lines.find_map(|line| { let line = line.unwrap(); line.starts_with("LEAK_READY ").then_some(line) }).unwrap();
        let parts: Vec<_> = line.split_whitespace().collect();
        let pid: i32 = parts[1].parse().unwrap();
        if mode == "kill" { parent.kill().unwrap(); }
        let status = parent.wait().unwrap();
        assert!(std::path::Path::new(&format!("/proc/{pid}/cmdline")).exists());
        eprintln!("REPRO mode={mode} parent={status} surviving_noded={pid} root={}", parts[2]);
        unsafe { libc::kill(pid, libc::SIGKILL); }
        std::fs::remove_dir_all(parts[2]).unwrap();
    }
}
