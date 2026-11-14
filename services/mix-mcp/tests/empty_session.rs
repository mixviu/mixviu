// SPDX-License-Identifier: MIT OR Apache-2.0
use rmcp::{ServiceExt, model::CallToolRequestParams, transport::TokioChildProcess};
use tokio::process::Command;

#[tokio::test]
async fn empty_session_override_refuses_bus() {
    let dir = tempfile::tempdir().unwrap();
    let registry = mix::parse_data(include_str!("../../../etc/env.conf.mix")).unwrap();
    let registry = mix::json::mix_to_json(&registry).unwrap();
    let mut defaults = serde_json::json!({
        "schema": 1,
        "commit": paths::stamp!(GIT_SHA_FULL).unwrap_or("fixture"),
    });
    for entry in registry["vars"].as_array().unwrap() {
        if let Some(value) = entry.get("default") {
            defaults[entry["name"].as_str().unwrap()] = value.clone();
        }
    }
    std::fs::write(dir.path().join("env.mx"), defaults.to_string()).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_mix-mcp"));
    command
        .env(paths::env::AGENT_SESSION, "")
        .env(paths::env::ETC, dir.path())
        .env(paths::env::VAR, dir.path())
        .env(paths::env::PREFIX, dir.path())
        .env("HOME", dir.path());
    let service = ().serve(TokioChildProcess::new(command).unwrap()).await.unwrap();
    let reply = service
        .call_tool(CallToolRequestParams::new("bus_list_services"))
        .await
        .unwrap();
    let text = serde_json::to_string(&reply).unwrap();
    assert!(text.contains("refusing to connect"), "{text}");
    assert!(text.contains(paths::env::AGENT_SESSION), "{text}");
    service.cancel().await.unwrap();
}
