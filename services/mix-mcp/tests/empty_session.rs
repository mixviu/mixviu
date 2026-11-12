// SPDX-License-Identifier: MIT OR Apache-2.0
use rmcp::{ServiceExt, model::CallToolRequestParams, transport::TokioChildProcess};
use tokio::process::Command;

#[tokio::test]
async fn empty_session_override_refuses_bus() {
    let dir = tempfile::tempdir().unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_mix-mcp"));
    command
        .env(paths::env::AGENT_SESSION, "")
        .env(paths::env::ETC, dir.path())
        .env(paths::env::VAR, dir.path())
        .env(paths::env::PREFIX, dir.path())
        .env("HOME", dir.path());
    let service = ().serve(TokioChildProcess::new(command).unwrap()).await.unwrap();
    let reply = service
        .call_tool(CallToolRequestParams::new("bus_status"))
        .await
        .unwrap();
    let text = serde_json::to_string(&reply).unwrap();
    assert!(text.contains("refusing to connect"), "{text}");
    assert!(text.contains(paths::env::AGENT_SESSION), "{text}");
    service.cancel().await.unwrap();
}
