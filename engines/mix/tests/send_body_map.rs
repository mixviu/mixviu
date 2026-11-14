// SPDX-License-Identifier: MIT OR Apache-2.0
//! Exercise the built shell against a real embedded Bus broker.
use std::process::{Command, Stdio};

use bus::native_client::NodedClient;
use serde_json::{Value, json};
use test_broker::Broker;

#[tokio::test]
async fn body_collections_round_trip_from_scripts_and_shell_input() {
    let broker = Broker::start();
    let dir = tempfile::tempdir().unwrap();
    let service = NodedClient::connect_anonymous(&broker.url).await.unwrap();
    service.register_as("honest-probe").await.unwrap();
    let mut incoming = service.incoming_async().await.unwrap();
    let receiver = tokio::spawn(async move {
        for _ in 0..10 {
            let command = tokio::time::timeout(std::time::Duration::from_secs(10), incoming.recv())
                .await
                .unwrap()
                .unwrap();
            let body = json!({"args": serde_json::from_str::<Value>(&command.body).ok(),
                "body": command.body});
            service
                .respond(&command, 0, &body.to_string())
                .await
                .unwrap();
        }
        service.close().await;
    });
    let expected = json!({"args": {"params": {}, "services": ["term"]},
        "body": r#"{"params":{},"services":["term"]}"#});
    let source = r#"$body = {params: {}, services: ["term"]}
send honest-probe probe.args body={params: {}, services: ["term"]}
print(json_encode($result))
send honest-probe probe.args body=$body
print(json_encode($result))
send honest-probe probe.args body=merge($body, {})
print(json_encode($result))
send honest-probe probe.args body=[{}, "term"]
print(json_encode($result))
send honest-probe probe.args body=" {\"raw\":true} "
print(json_encode($result))
"#;
    let sender = dir.path().join("sender.mix");
    std::fs::write(&sender, source).unwrap();
    for shell_input in [false, true] {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_mix"));
        cmd.arg("--no-prelude");
        if shell_input {
            cmd.args(["-c", source]);
        } else {
            cmd.arg(&sender);
        }
        cmd.env("HOME", dir.path())
            .env("MIX_NODE_CONFIG", dir.path().join("missing.conf.mix"))
            .env("MIX_NODED_URL", &broker.url)
            .env("MIX_STATS", "off")
            .env_remove("MIX_SESSION_FD")
            .env_remove("COSMIX_SESSION_FD")
            .stdin(Stdio::null());
        let out = tokio::task::spawn_blocking(move || cmd.output().unwrap())
            .await
            .unwrap();
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            out.status.success(),
            "{stdout}\n{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let values: Vec<Value> = stdout
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        assert_eq!(
            values,
            vec![
                expected.clone(),
                expected.clone(),
                expected.clone(),
                json!({"args": [{}, "term"], "body": r#"[{},"term"]"#}),
                json!({"args": {"raw": true}, "body": " {\"raw\":true} "})
            ]
        );
    }
    receiver.await.unwrap();
}
