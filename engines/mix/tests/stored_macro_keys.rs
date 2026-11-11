// SPDX-License-Identifier: MIT OR Apache-2.0
//! A real stored Mix macro keeps running after the app refuses a retired key.
use bus::native_client::NodedClient;
use std::time::Duration;

#[tokio::test]
async fn stored_macro_skips_only_reserved_key_refusals_and_preserves_source() {
    let broker = test_broker::Broker::start();
    let service = NodedClient::connect_anonymous(&broker.url).await.unwrap();
    service.register_as("fixture-macro").await.unwrap();
    let mut incoming = service.incoming_async().await.unwrap();
    let dir = tempfile::tempdir().unwrap();
    let port = broker
        .url
        .trim_start_matches("ws://127.0.0.1:")
        .trim_end_matches("/ws");
    let config = dir.path().join("node.conf.mix");
    std::fs::write(
        &config,
        format!("wg_ip: \"127.0.0.1\"\nnoded: {{ port: {port} }}\n"),
    )
    .unwrap();
    for replay in [false, true] {
        let expected = if replay { 0 } else { 10 };
        let source = format!(
            r#"-- title: Stored obsolete keys
send fixture-macro fixture.ui.key key="F1" timeout=2
if $rc!={expected} then raise("TEST","F1 result") end
send fixture-macro fixture.ui.key body={{key:"ArrowLeft",ctrl:true}} timeout=2
if $rc!={expected} then raise("TEST","Ctrl arrow result") end
send fixture-macro fixture.ui.key body={{key:"ArrowUp",alt:true}} timeout=2
if $rc!={expected} then raise("TEST","Alt arrow result") end
send fixture-macro fixture.valid timeout=2
if $rc!=0 then raise("TEST","valid step lost") end
print("VALID_STEP_SURVIVED")
"#
        );
        let script = dir.path().join("stored.mix");
        std::fs::write(&script, &source).unwrap();
        let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_mix"));
        child
            .arg("--no-prelude")
            .arg(&script)
            .env("MIX_NODE_CONFIG", &config)
            .env("MIX_BUS_SOCKET", &broker.endpoint)
            .env("MIX_STATS", "off")
            .env_remove("MIX_SESSION_FD")
            .env_remove("MIX_MACRO_APP")
            .env_remove("MIX_MACRO_ID");
        if replay {
            child
                .env("MIX_MACRO_APP", "ced")
                .env("MIX_MACRO_ID", "stored");
        }
        let child = tokio::task::spawn_blocking(move || child.output().unwrap());
        for _ in 0..4 {
            let command = tokio::time::timeout(Duration::from_secs(10), incoming.recv())
                .await
                .unwrap()
                .unwrap();
            let (rc, body) = if command.command == "fixture.ui.key" {
                (
                    10,
                    r#"{"error_code":"INVALID_ARGUMENT","message":"F1–F12 and Ctrl/Alt+arrow are reserved for the desk and are refused"}"#,
                )
            } else {
                assert_eq!(command.command, "fixture.valid");
                (0, "{}")
            };
            service.respond(&command, rc, body).await.unwrap();
        }
        let output = child.await.unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("VALID_STEP_SURVIVED"));
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        assert_eq!(
            diagnostic
                .lines()
                .filter(|l| l.contains("skipped stored reserved key"))
                .count(),
            if replay { 3 } else { 0 },
            "{diagnostic}"
        );
        if replay {
            for name in ["F1", "ArrowLeft", "ArrowUp"] {
                assert!(diagnostic.contains(name), "{diagnostic}");
            }
        }
        assert_eq!(std::fs::read_to_string(&script).unwrap(), source);
    }
}
