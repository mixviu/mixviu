// SPDX-License-Identifier: MIT OR Apache-2.0
use std::os::unix::fs::MetadataExt;
use std::process::Command;

#[test]
fn executable_logging_needs_no_environment_or_installed_defaults() {
    for bad in [false, true] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_inputd"));
        child.env_clear().arg("--help");
        if bad {
            child.env(paths::env::INPUTD_LOG, "[bad");
        }
        let output = child.output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        assert!(String::from_utf8_lossy(&output.stdout).contains("Input broker"));
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(stderr.lines().filter(|line| line.contains("ignored invalid")).count(), usize::from(bad));
    }
}

#[test]
fn unit_directory_is_private_owned_and_idempotent() {
    let unit = include_str!("../units/inputd.service");
    assert!(unit.contains("ReadWritePaths=@{etc}/inputd\n"));
    assert!(!unit.contains("ReadWritePaths=@{etc}\n"));
    assert!(unit.contains("ExecStartPre=+/usr/bin/systemd-tmpfiles --create"));
    assert!(unit.contains("--keymap @{etc}/inputd/keymap.json"));
    let template = include_str!("../units/inputd.tmpfiles.conf");
    assert!(template.contains("d @{etc}/inputd 0700 root root -"));
    let exe = std::env::current_exe().unwrap();
    let dir = tempfile::tempdir_in(exe.parent().unwrap()).unwrap();
    let root = dir.path().join("root");
    std::fs::create_dir(&root).unwrap();
    let conf = dir.path().join("inputd.conf");
    // A non-root test worker cannot chown root. Use its own numeric owner in
    // that case; root workers exercise the shipped rule unchanged.
    let uid = unsafe { libc::geteuid() };
    let gid = unsafe { libc::getegid() };
    let rendered = template.replace("@{etc}", "/etc").replace("root root", &format!("{uid} {gid}"));
    std::fs::write(&conf, rendered).unwrap();
    for _ in 0..2 {
        let output = Command::new("/usr/bin/systemd-tmpfiles")
            .arg("--create").arg("--root").arg(&root).arg(&conf).output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let meta = std::fs::metadata(root.join("etc/inputd")).unwrap();
        assert_eq!(meta.mode() & 0o777, 0o700);
        assert_eq!((meta.uid(), meta.gid()), (uid, gid));
    }
}
