// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>

use std::os::unix::fs::PermissionsExt;

#[test]
fn fixture_ignores_private_tmpdir() {
    const CHILD: &str = "TEST_BROKER_TMPDIR_CHILD";
    if std::env::var_os(CHILD).is_some() {
        let mut broker = test_broker::Broker::start();
        assert!(!broker.endpoint.starts_with(std::env::temp_dir()));
        let endpoint = broker.endpoint.clone();
        assert!(endpoint.exists());
        broker.bounce(); // Each boot must negotiate verified native ingress.
        assert!(endpoint.exists());
        drop(broker);
        assert!(!endpoint.exists());
        assert!(!endpoint.parent().unwrap().exists());
        return;
    }

    // Isolate the environment in a subprocess. This path is both private and
    // longer than sockaddr_un, so merely changing the child's mode is no fix.
    let scratch = std::path::PathBuf::from("/tmp").join(format!(
        "test-broker-tmpdir-{:032x}-{}",
        rand::random::<u128>(),
        "x".repeat(100)
    ));
    std::fs::create_dir(&scratch).unwrap();
    std::fs::set_permissions(&scratch, std::fs::Permissions::from_mode(0o700)).unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "fixture_ignores_private_tmpdir", "--nocapture"])
        .env(CHILD, "1")
        .env("TMPDIR", &scratch)
        .output()
        .unwrap();
    let mode = std::fs::metadata(&scratch).unwrap().permissions().mode() & 0o777;
    std::fs::remove_dir(&scratch).unwrap();
    assert_eq!(mode, 0o700, "the job's permissions must remain private");
    assert!(
        output.status.success(),
        "native fixture failed under private long TMPDIR: {}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
