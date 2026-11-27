// SPDX-License-Identifier: MIT OR Apache-2.0
use std::os::unix::{fs::PermissionsExt, process::CommandExt};
use std::process::Command;
use base64::Engine as _;

#[test]
fn guest_generator_emits_only_public_material_and_stages_rotation() {
    let dir = tempfile::tempdir().unwrap();
    // Also test the unprivileged node contract when CBC runs as root, without
    // depending on a provisioned service account/group on a build worker.
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o777)).unwrap();
    let path = dir.path().join("bus-tls.seed");
    let invoke = |rotate| {
        let mut command = Command::new(env!("CARGO_BIN_EXE_mix"));
        command.args(["mesh", "bus-tls-gen", "--path"]).arg(&path);
        if rotate { command.arg("--rotate"); }
        // SAFETY: geteuid has no preconditions.
        if unsafe { libc::geteuid() } == 0 { command.gid(65534).uid(65534); }
        command.output().unwrap()
    };
    let first = invoke(false);
    assert!(first.status.success(), "{}", String::from_utf8_lossy(&first.stderr));
    assert!(first.stderr.is_empty());
    let public: serde_json::Value = serde_json::from_slice(&first.stdout).unwrap();
    let seed = std::fs::read(&path).unwrap();
    let secret: [u8;32] = seed.as_slice().try_into().unwrap();
    let expected = ed25519_dalek::SigningKey::from_bytes(&secret).verifying_key().to_bytes();
    assert_eq!(public["pubkey"], base64::engine::general_purpose::STANDARD.encode(expected));
    assert_eq!(std::fs::metadata(&path).unwrap().permissions().mode() & 0o777, 0o600);
    let refused = invoke(false);
    assert!(!refused.status.success());
    let rotated = invoke(true);
    assert!(rotated.status.success(), "{}", String::from_utf8_lossy(&rotated.stderr));
    let next: serde_json::Value = serde_json::from_slice(&rotated.stdout).unwrap();
    let next_path = std::path::Path::new(next["path"].as_str().unwrap());
    assert_ne!(next_path, path);
    assert!(next_path.starts_with(dir.path()));
    assert_ne!(std::fs::read(next_path).unwrap(), seed);
    assert_eq!(std::fs::read(&path).unwrap(), seed);
    for output in [&first,&refused,&rotated] {
        for bytes in [&output.stdout,&output.stderr] {
            assert!(!bytes.windows(32).any(|window| window == seed));
            assert!(!String::from_utf8_lossy(bytes).contains(&base64::engine::general_purpose::STANDARD.encode(&seed)));
        }
    }
}
