use std::io::Read;
use std::os::fd::AsRawFd;
use std::os::unix::process::CommandExt;
use std::process::Command;

fn refuses(command: &str) {
    let root = tempfile::tempdir().unwrap();
    // An unusable PATH proves these shortcuts need neither git nor cargo.
    for flags in [vec![], vec!["--help"], vec!["--release"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_mix"))
            .arg(command)
            .args(flags)
            .env("PATH", root.path())
            .env("MIX_SRC", root.path())
            .env("MIX_BIN", root.path().join("bin"))
            .current_dir(root.path())
            .output()
            .unwrap();
        assert!(!output.status.success());
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("refused"), "{stderr}");
        assert!(stderr.contains("mix tools/ci.mix <stage>"), "{stderr}");
        assert!(stderr.contains("cbc-build.mix"), "{stderr}");
        assert_eq!(std::fs::read_dir(root.path()).unwrap().count(), 0);
    }
}

#[test]
fn build_refuses_local_cargo() {
    refuses("build");
}
#[test]
fn clean_refuses_local_cargo() {
    refuses("clean");
}
#[test]
fn test_refuses_local_cargo() {
    refuses("test");
}
#[test]
fn update_refuses_local_cargo() {
    refuses("update");
}

#[test]
fn help_describes_the_disabled_shortcuts() {
    let output = Command::new(env!("CARGO_BIN_EXE_mix"))
        .arg("help")
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    for command in ["build", "clean", "update", "test"] {
        let line = text
            .lines()
            .find(|line| line.trim_start().starts_with(&format!("mix {command} ")))
            .unwrap_or_else(|| panic!("missing {command}: {text}"));
        assert!(line.contains("Disabled"), "{line}");
    }
}

#[test]
fn result_fault_hook_follows_the_binary_feature() {
    let (mut reader, writer) = std::os::unix::net::UnixStream::pair().unwrap();
    let fd = writer.as_raw_fd();
    let mut command = Command::new(env!("CARGO_BIN_EXE_mix"));
    command
        .args(["--result-fd", &fd.to_string(), "-c", "42"])
        .env("MIX_TEST_RESULT_TORN", "1")
        .env("MIX_TEST_ADMIT_DELAY_MS", "10000")
        .env("MIX_TEST_CLAIM_DELAY_MS", "10000")
        .env("MIX_TEST_RESERVE_HOLD_MS", "10000");
    // Clear CLOEXEC only in the forked child; the parent keeps ownership.
    unsafe {
        command.pre_exec(move || {
            if libc::fcntl(fd, libc::F_SETFD, 0) == -1 {
                return Err(std::io::Error::last_os_error());
            }
            Ok(())
        });
    }
    let output = command.output().unwrap();
    assert!(output.status.success(), "{output:?}");
    drop(writer);
    let mut frame = Vec::new();
    reader.read_to_end(&mut frame).unwrap();
    let declared = u32::from_be_bytes(frame[..4].try_into().unwrap()) as usize;
    assert_eq!(
        frame.len() - 4,
        if cfg!(feature = "test-hooks") {
            declared / 2
        } else {
            declared
        }
    );
}
