// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>

use std::os::unix::fs::PermissionsExt;

#[test]
fn fixture_ignores_private_tmpdir() {
    const CHILD: &str = "TEST_BROKER_TMPDIR_CHILD";
    if let Some(mask) = std::env::var_os(CHILD) {
        let mask = u32::from_str_radix(mask.to_str().unwrap(), 8).unwrap();
        // This subprocess has one test; never change the parent runner's umask.
        unsafe { libc::umask(mask) };
        let mut broker = test_broker::Broker::start();
        assert!(!broker.endpoint.starts_with(std::env::temp_dir()));
        let endpoint = broker.endpoint.clone();
        let directory_mode = || {
            std::fs::metadata(endpoint.parent().unwrap())
                .unwrap()
                .permissions()
                .mode()
                & 0o777
        };
        assert_eq!(directory_mode(), 0o755);
        assert!(endpoint.exists());
        broker.bounce(); // Each boot must negotiate verified native ingress.
        assert_eq!(directory_mode(), 0o755);
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
    for mask in ["077", "000"] {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "fixture_ignores_private_tmpdir", "--nocapture"])
            .env(CHILD, mask)
            .env("TMPDIR", &scratch)
            .output()
            .unwrap();
        let mode = std::fs::metadata(&scratch).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o700, "the job's permissions must remain private");
        assert!(
            output.status.success(),
            "native fixture failed under private long TMPDIR and umask {mask}: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    std::fs::remove_dir(&scratch).unwrap();
}
