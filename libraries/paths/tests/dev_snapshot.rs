// SPDX-License-Identifier: MIT OR Apache-2.0
//! Real, separately compiled gold/dev processes sharing one temporary prefix.
use std::{
    path::Path,
    process::{Command, Stdio},
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

#[test]
fn two_binaries_one_prefix_and_fail_closed_controls() {
    let prefix = std::env::temp_dir().join(format!("paths-dev-snapshot-{}", std::process::id()));
    std::fs::create_dir_all(prefix.join("bin")).unwrap();
    struct Scratch(std::path::PathBuf);
    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _scratch = Scratch(prefix.clone());
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
    for (name, stamp) in [("mixd", GOLD), ("viu", DEV)] {
        let source = prefix.join(format!("{name}.rs"));
        std::fs::write(
            &source,
            format!(
                r#"
fn main() {{
    paths::env::register_member_snapshot("{name}", "{stamp}");
    for _ in 0..2 {{
        match paths::env::setting(paths::env::NODED_SERVICE) {{
            Ok(value) => assert_eq!(value, "shared-default"),
            Err(paths::env::SettingError::SnapshotMismatch {{ .. }}) => std::process::exit(42),
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
        "x".repeat(65537),
    ] {
        std::fs::write(&marker, invalid).unwrap();
        let out = command(&prefix.join("bin/viu")).output().unwrap();
        assert_eq!(out.status.code(), Some(42));
        assert!(String::from_utf8_lossy(&out.stderr).contains(marker.to_str().unwrap()));
        let out = command(&prefix.join("bin/mixd")).output().unwrap();
        assert!(out.status.success());
        assert!(out.stderr.is_empty(), "gold must not read DEV.json");
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
    let other = prefix.join("other");
    std::fs::create_dir(&other).unwrap();
    std::fs::copy(prefix.join("env.mx"), other.join("env.mx")).unwrap();
    std::fs::write(other.join("DEV.json"), valid).unwrap();
    let out = command(&prefix.join("bin/viu"))
        .env("MIX_PREFIX", &other)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(42));
    assert!(String::from_utf8_lossy(&out.stderr).contains("outside executable prefix"));
}
