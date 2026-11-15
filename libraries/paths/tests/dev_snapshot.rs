// SPDX-License-Identifier: MIT OR Apache-2.0
//! Real, separately compiled gold/dev processes sharing one temporary prefix.
use std::{
    os::unix::fs::symlink,
    path::Path,
    process::{Command, Stdio},
    time::{Duration, Instant},
};

const GOLD: &str = "0123456789abcdef0123456789abcdef01234567";
const DEV: &str = "fedcba9876543210fedcba9876543210fedcba98";

fn rustc(args: &[&str]) {
    let output = Command::new("rustc").args(args).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn command(exe: &Path) -> Command {
    let mut cmd = Command::new(exe);
    cmd.env_clear()
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    cmd
}

// Bound probes that would hang if a marker FIFO were opened. Kill and reap
// on failure so a regression cannot leave a process on the build worker.
fn promptly(mut cmd: Command) -> std::process::Output {
    let mut child = cmd.spawn().unwrap();
    let deadline = Instant::now() + Duration::from_secs(3);
    loop {
        if child.try_wait().unwrap().is_some() {
            return child.wait_with_output().unwrap();
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("strict mismatch probe blocked on a marker");
        }
        std::thread::sleep(Duration::from_millis(10));
    }
}

#[test]
fn two_binaries_one_prefix_and_fail_closed_controls() {
    let scratch_root =
        std::env::temp_dir().join(format!("paths-dev-snapshot-{}", std::process::id()));
    let prefix = scratch_root.join("install");
    std::fs::create_dir_all(prefix.join("bin")).unwrap();
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _scratch = Scratch(scratch_root.clone());
    // Compile the real paths reader once, then two immutable build identities.
    // This test runs on the build worker; no desktop Rust build is involved.
    let library = prefix.join("libpaths.rlib");
    rustc(&[
        "--edition=2024",
        "--crate-name=paths",
        "--crate-type=rlib",
        concat!(env!("CARGO_MANIFEST_DIR"), "/src/lib.rs"),
        "-o",
        library.to_str().unwrap(),
    ]);
    for (name, stamp) in [("mixd", GOLD), ("viu", DEV), ("legacy", DEV)] {
        let registration = if name == "legacy" {
            format!("paths::env::register_snapshot(\"{stamp}\");")
        } else {
            format!("paths::env::register_member_snapshot(\"{name}\", \"{stamp}\");")
        };
        let source = prefix.join(format!("{name}.rs"));
        std::fs::write(
            &source,
            format!(
                r#"
fn main() {{
    {registration}
    for _ in 0..2 {{
        match paths::env::setting(paths::env::NODED_SERVICE) {{
            Ok(value) => assert_eq!(value, "shared-default"),
            Err(e @ paths::env::SettingError::SnapshotMismatch {{ .. }}) => {{
                assert_eq!(e.to_string(), "MIX_NODED_SERVICE: env.mx snapshot {GOLD} differs from binary {DEV}");
                std::process::exit(42);
            }}
            Err(e) => panic!("unexpected error: {{e}}"),
        }}
    }}
}}
"#
            ),
        )
        .unwrap();
        rustc(&[
            "--edition=2024",
            source.to_str().unwrap(),
            "--extern",
            &format!("paths={}", library.display()),
            "-o",
            prefix.join("bin").join(name).to_str().unwrap(),
        ]);
    }
    std::fs::write(
        prefix.join("env.mx"),
        format!("schema: 1\ncommit: '{GOLD}'\nMIX_NODED_SERVICE: 'shared-default'\n"),
    )
    .unwrap();
    let valid = format!(r#"{{"schema":1,"gold":"{GOLD}","members":{{"viu":"{DEV}"}}}}"#);
    let marker = prefix.join("DEV.json");
    std::fs::write(&marker, &valid).unwrap();
    // Start both before waiting: independent stamps read the very same file.
    let gold = command(&prefix.join("bin/mixd")).spawn().unwrap();
    let dev = command(&prefix.join("bin/viu")).spawn().unwrap();
    let gold = gold.wait_with_output().unwrap();
    let dev = dev.wait_with_output().unwrap();
    assert!(gold.status.success());
    assert!(gold.stderr.is_empty());
    assert!(
        dev.status.success(),
        "{}",
        String::from_utf8_lossy(&dev.stderr)
    );
    let log = String::from_utf8(dev.stderr).unwrap();
    assert_eq!(log.matches("dev snapshot exception").count(), 1);
    for expected in [GOLD, DEV, marker.to_str().unwrap()] {
        assert!(log.contains(expected));
    }

    // Whitespace and field order do not change the marker's meaning.
    let formatted = format!(
        "{{\n\t\"members\" : {{ \"viu\" : \"{DEV}\" }},\r\n\"gold\" : \"{GOLD}\", \"schema\" : 1\n}}\n"
    );
    std::fs::write(&marker, &formatted).unwrap();
    assert!(
        command(&prefix.join("bin/viu"))
            .output()
            .unwrap()
            .status
            .success()
    );
    // Pin both sides of the size bound with otherwise valid JSON whitespace.
    let at_limit = format!("{valid}{}", " ".repeat(65536 - valid.len()));
    std::fs::write(&marker, &at_limit).unwrap();
    assert!(
        command(&prefix.join("bin/viu"))
            .output()
            .unwrap()
            .status
            .success()
    );

    for invalid in [
        valid.replace("viu", "mixd"),
        valid.replace(DEV, GOLD),
        valid.replace(GOLD, DEV),
        "{".into(),
        valid.replace("\"schema\":1", "\"schema\":2"),
        valid.replace("\"schema\":1", "\"schema\":1,\"schema\":1"),
        valid.replace("\"members\"", "\"unknown\""),
        valid.replace("}}", ",}}"),
        format!("{valid} trailing"),
        format!("{at_limit} "),
        valid.replace("\"viu\"", "\"v\\u0069u\""),
        valid.replace("\"viu\"", "\"-viu\""),
        valid.replace("\"viu\"", "\"Viu\""),
        valid.replace("}}", &format!(",\"viu\":\"{DEV}\"}}}}")),
        format!("\u{feff}{valid}"),
        format!("/* comment */{valid}"),
    ] {
        std::fs::write(&marker, invalid).unwrap();
        let out = command(&prefix.join("bin/viu")).output().unwrap();
        assert_eq!(out.status.code(), Some(42));
        assert!(String::from_utf8_lossy(&out.stderr).contains(marker.to_str().unwrap()));
        let out = command(&prefix.join("bin/mixd")).output().unwrap();
        assert!(out.status.success());
        assert!(out.stderr.is_empty(), "gold must not read DEV.json");
    }
    let mut non_utf8 = valid.as_bytes().to_vec();
    non_utf8.push(0xff);
    std::fs::write(&marker, non_utf8).unwrap();
    let out = command(&prefix.join("bin/viu")).output().unwrap();
    assert_eq!(out.status.code(), Some(42));
    assert!(String::from_utf8_lossy(&out.stderr).contains(marker.to_str().unwrap()));

    // Identity is compiled in: copying viu under another member's filename
    // cannot make a marker naming that member authorise it.
    std::fs::write(&marker, valid.replace("viu", "inputd")).unwrap();
    std::fs::copy(prefix.join("bin/viu"), prefix.join("bin/inputd")).unwrap();
    let out = command(&prefix.join("bin/inputd")).output().unwrap();
    assert_eq!(out.status.code(), Some(42));
    assert!(String::from_utf8_lossy(&out.stderr).contains("member viu"));

    std::fs::write(&marker, &valid).unwrap();
    let out = command(&prefix.join("bin/legacy")).output().unwrap();
    assert_eq!(out.status.code(), Some(42));
    assert!(
        out.stderr.is_empty(),
        "strict-only registration stays silent"
    );

    // Existing prefix derivation follows the real /proc/self/exe target.
    // Both inside and foreign launch symlinks therefore use the target's
    // marker, not a marker beside the symlink. Do not infer launch-path policy.
    let foreign = scratch_root.join("foreign/bin");
    std::fs::create_dir_all(&foreign).unwrap();
    for link in [prefix.join("bin/viu-link"), foreign.join("viu")] {
        symlink(prefix.join("bin/viu"), &link).unwrap();
        let out = command(&link).output().unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(String::from_utf8_lossy(&out.stderr).contains(marker.to_str().unwrap()));
    }
    std::fs::remove_file(&marker).unwrap();
    let out = command(&prefix.join("bin/viu")).output().unwrap();
    assert_eq!(out.status.code(), Some(42));
    assert!(
        out.stderr.is_empty(),
        "missing marker preserves strict diagnostics"
    );
    // An unreadable marker (directory works even under a privileged test user).
    std::fs::create_dir(&marker).unwrap();
    let out = command(&prefix.join("bin/viu")).output().unwrap();
    assert_eq!(out.status.code(), Some(42));
    assert!(String::from_utf8_lossy(&out.stderr).contains(marker.to_str().unwrap()));
    std::fs::remove_dir(&marker).unwrap();
    // A caller-controlled settings prefix cannot supply marker authority.
    let other = scratch_root.join("other");
    std::fs::create_dir(&other).unwrap();
    std::fs::copy(prefix.join("env.mx"), other.join("env.mx")).unwrap();
    let redirected = other.join("DEV.json");
    for content in [&valid, "{\"caller-chosen-secret\":1}"] {
        std::fs::write(&redirected, content).unwrap();
        let mut cmd = command(&prefix.join("bin/viu"));
        cmd.env("MIX_PREFIX", &other);
        let out = promptly(cmd);
        assert_eq!(out.status.code(), Some(42));
        assert!(
            out.stderr.is_empty(),
            "redirected marker must not be diagnosed"
        );
    }
    std::fs::remove_file(&redirected).unwrap();
    assert!(
        Command::new("mkfifo")
            .arg(&redirected)
            .status()
            .unwrap()
            .success()
    );
    let mut cmd = command(&prefix.join("bin/viu"));
    cmd.env("MIX_PREFIX", &other);
    let out = promptly(cmd);
    assert_eq!(out.status.code(), Some(42));
    assert!(out.stderr.is_empty());

    // Legacy registration must not even open a marker at its own prefix.
    assert!(
        Command::new("mkfifo")
            .arg(&marker)
            .status()
            .unwrap()
            .success()
    );
    let out = promptly(command(&prefix.join("bin/legacy")));
    assert_eq!(out.status.code(), Some(42));
    assert!(out.stderr.is_empty());
}
