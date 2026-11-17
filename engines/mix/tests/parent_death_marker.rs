// SPDX-License-Identifier: MIT OR Apache-2.0
#![cfg(target_os = "linux")]

use std::os::unix::process::CommandExt;
use std::process::{Command, Output};

const MARKER: &str = "MIX_EXPECTED_PARENT_PID";

fn probe(marker: &str, signal: i32, source: &str) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_mix"));
    command
        .args(["--no-prelude", "-c", source])
        .env(MARKER, marker)
        .env("MIX_STATS", "off");
    // SAFETY: pre_exec calls only prctl, with integer arguments.
    unsafe {
        command.pre_exec(move || {
            if libc::prctl(libc::PR_SET_PDEATHSIG, signal, 0, 0, 0) == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    command.output().expect("run real mix")
}

#[test]
fn ambient_marker_cannot_disable_mix_and_is_scrubbed() {
    for marker in ["1", "not-a-pid"] {
        let result = probe(marker, 0, "print(env(\"MIX_EXPECTED_PARENT_PID\") == \"\")");
        assert!(result.status.success(), "{:?}", result);
        assert_eq!(result.stdout, b"true\n");
        assert!(result.stderr.is_empty());
    }
}

#[test]
fn armed_mix_scrubs_marker_before_starting_grandchild() {
    let exe = serde_json::to_string(env!("CARGO_BIN_EXE_mix")).unwrap();
    let source = format!(
        "print(env(\"MIX_EXPECTED_PARENT_PID\") == \"\"); print(run_argv_must([{exe}, \"--no-prelude\", \"-c\", \"print(env(\\\"MIX_EXPECTED_PARENT_PID\\\") == \\\"\\\")\"]))"
    );
    let result = probe(&std::process::id().to_string(), libc::SIGKILL, &source);
    assert!(result.status.success(), "{:?}", result);
    assert_eq!(String::from_utf8_lossy(&result.stdout).trim(), "true\ntrue");
}

#[test]
fn armed_parent_mismatch_exits_with_one_diagnostic() {
    let result = probe("1", libc::SIGKILL, "print(\"must not run\")");
    assert_eq!(result.status.code(), Some(126));
    assert!(result.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&result.stderr);
    assert_eq!(stderr.lines().count(), 1);
    assert!(stderr.contains("expected parent 1"), "{stderr}");
    assert!(stderr.contains("parent died"), "{stderr}");
}
