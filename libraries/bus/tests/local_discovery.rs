// SPDX-License-Identifier: MIT OR Apache-2.0
//! Isolate discovery's process environment rather than mutating test threads.
#![cfg(feature = "client")]
#[test]
fn local_discovery_rejects_relative_socket_paths() {
    if std::env::var_os("DISCOVERY_CHILD").is_some() {
        assert_eq!(
            bus::client::noded_url(),
            "unix:///run/example/noded/bus.sock"
        );
        return;
    }
    let path = std::env::temp_dir().join(format!("bus-discovery-{}.mix", std::process::id()));
    std::fs::write(&path, "noded: { unix_socket: 'relative.sock' }").unwrap();
    for socket in [None, Some("relative-env.sock")] {
        let mut child = std::process::Command::new(std::env::current_exe().unwrap());
        child
            .args(["--exact", "local_discovery_rejects_relative_socket_paths"])
            .env("DISCOVERY_CHILD", "1")
            .env_remove("MIX_NODED_URL")
            .env_remove("MIX_BUS_SOCKET")
            .env("MIX_NODE_CONFIG", &path)
            .env("MIX_RUN", "/run/example");
        if let Some(socket) = socket {
            child.env("MIX_BUS_SOCKET", socket);
        }
        assert!(child.status().unwrap().success());
    }
    std::fs::remove_file(path).unwrap();
}
