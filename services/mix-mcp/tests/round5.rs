// SPDX-License-Identifier: MIT OR Apache-2.0
use rmcp::{ServiceExt, model::CallToolRequestParams, transport::TokioChildProcess};
use tokio::process::Command;

#[tokio::test]
async fn bare_startup_junk_deadline_sibling_and_worker_surface() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    std::fs::create_dir(dir.path().join("bin"))?;
    std::fs::write(dir.path().join("bin/mix"), "planted interpreter must never run")?;
    let mut command = Command::new(env!("CARGO_BIN_EXE_mix-mcp"));
    command.env_clear().env("HOME", dir.path()).env("PATH", "/usr/bin:/bin")
        .env(paths::env::PREFIX, dir.path()).env(paths::env::VAR, dir.path())
        .env(paths::env::ETC, dir.path())
        .env(paths::env::MCP_TOOL_DEADLINE_SECS, "junk");
    let service = ().serve(TokioChildProcess::new(command)?).await?;
    let tools = service.list_all_tools().await?;
    let mut names: Vec<_> = tools.iter().map(|t| t.name.as_ref()).collect();
    names.sort_unstable();
    assert_eq!(names.join("\n"), include_str!("tools.txt").trim());
    for (script, expected) in [
        ("print(17)", "17"),
        ("tar_list(\"absent.tar\")", "outside the frozen MCP builtin surface"),
    ] {
        let reply = service.call_tool(CallToolRequestParams::new("mix_execute")
            .with_arguments(serde_json::json!({"script":script}).as_object().cloned().ok_or_else(|| anyhow::anyhow!("arguments"))?)).await?;
        assert!(serde_json::to_string(&reply)?.contains(expected), "{reply:?}");
    }
    let reply = service.call_tool(CallToolRequestParams::new("context_search")
        .with_arguments(serde_json::json!({"query":"fixture"}).as_object().cloned().ok_or_else(|| anyhow::anyhow!("arguments"))?)).await?;
    let text = serde_json::to_string(&reply)?;
    assert!(text.contains("MIX_MCP_KNOWLEDGE") && text.contains("env.mx"), "{text}");
    assert_eq!(service.list_all_tools().await?.len(), 29);
    service.cancel().await?;
    Ok(())
}
