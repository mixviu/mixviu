// SPDX-License-Identifier: MIT OR Apache-2.0
use rmcp::{ServiceExt, model::CallToolRequestParams, transport::TokioChildProcess};
use tokio::process::Command;

#[tokio::test]
async fn invalid_layout_keeps_discovery_and_returns_log_errors() -> anyhow::Result<()> {
    for malformed in [false, true] {
        let dir = tempfile::tempdir()?;
        if malformed {
            std::fs::write(
                dir.path().join("layout.mx"),
                "this is not valid layout data {",
            )?;
        }
        let stderr = dir.path().join("stderr");
        let mut command = Command::new(env!("CARGO_BIN_EXE_mix-mcp"));
        command
            .env_clear()
            .env("HOME", dir.path())
            .env(
                paths::env::PREFIX,
                if malformed {
                    dir.path().as_os_str()
                } else {
                    std::ffi::OsStr::new("relative")
                },
            )
;
        let (transport, _) = TokioChildProcess::builder(command).stderr(std::fs::File::create(&stderr)?).spawn()?;
        let service = ().serve(transport).await?;
        let tools = service.list_all_tools().await?;
        let mut names: Vec<_> = tools.iter().map(|t| t.name.as_ref()).collect();
        names.sort_unstable();
        assert_eq!(names.join("\n"), include_str!("tools.txt").trim());
        for (name, arguments) in [
            ("log_tail", serde_json::json!({"file":"bus"})),
            (
                "log_search",
                serde_json::json!({"file":"bus", "pattern":"fixture"}),
            ),
            ("context_search", serde_json::json!({"query":"fixture"})),
        ] {
            let reply = service
                .call_tool(
                    CallToolRequestParams::new(name)
                        .with_arguments(arguments.as_object().cloned().unwrap()),
                )
                .await?;
            let text = serde_json::to_string(&reply)?;
            assert!(
                text.contains(if malformed { "layout.mx" } else { "relative" }),
                "{text}"
            );
        }
        assert_eq!(service.list_all_tools().await?.len(), 29);
        service.cancel().await?;
        let diagnostic = std::fs::read_to_string(stderr)?;
        assert_eq!(
            diagnostic
                .matches("mix-mcp: file logging disabled:")
                .count(),
            1,
            "{diagnostic}"
        );
        assert!(!diagnostic.contains("panicked"), "{diagnostic}");
    }
    Ok(())
}

#[tokio::test]
async fn nested_and_indirect_calls_keep_the_worker_policy() -> anyhow::Result<()> {
    let dir = tempfile::tempdir()?;
    let mut command = Command::new(env!("CARGO_BIN_EXE_mix-mcp"));
    command
        .env_clear()
        .env("HOME", dir.path())
        .env(paths::env::PREFIX, dir.path())
        .env(paths::env::VAR, dir.path());
    let service = ().serve(TokioChildProcess::new(command)?).await?;
    let mut scripts = vec![
        "map([1], fn($x) = tar_list(\"absent.tar\"))".to_string(),
        "if has_builtin(\"tar_list\") or has_builtin(\"http_get\") then die \"widened\" end; print(71)"
            .to_string(),
    ];
    for builtin in ["tar_list", "http_get"] {
        let path = dir.path().join(format!("{builtin}.mix"));
        std::fs::write(&path, format!("{builtin}(\"absent\")"))?;
        let literal = serde_json::to_string(&path)?;
        scripts.push(format!("require({literal})"));
        scripts.push(format!("source {literal}"));
        scripts.push(format!("include {literal}"));
    }
    for script in scripts {
        let expected = if script.contains("has_builtin") {
            "71"
        } else {
            "outside the frozen MCP builtin surface"
        };
        let reply = service
            .call_tool(
                CallToolRequestParams::new("mix_execute").with_arguments(
                    serde_json::json!({"script":script})
                        .as_object()
                        .cloned()
                        .unwrap(),
                ),
            )
            .await?;
        assert!(
            serde_json::to_string(&reply)?.contains(expected),
            "{reply:?}"
        );
    }
    service.cancel().await?;
    Ok(())
}
