// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Isolated agent execution, derived from the frozen MCP worker. This private
//! entry point speaks no MCP and skips ordinary shell hooks. The caller owns
//! the process group; Bus identity and transport policy stay with agent_client.
use ::bus::agent_client as connection;
use serde::Deserialize;
use std::io::{Read, Write};

pub const ARG: &str = "--internal-mix-worker";
const SCRIPT_LIMIT: usize = 256 * 1024;
const INPUT_LIMIT: usize = SCRIPT_LIMIT * 6 + 4096;
const OUTPUT_LIMIT: usize = 1024 * 1024;

// Frozen 8779f33 builtin registries, excluding feature-gated arms outside
// json/regex/toml/datetime/url/crypto. Cargo's workspace feature union must
// never widen the evaluator used by the MCP adapter.
struct McpBuiltins;
#[test]
fn frozen_worker_inventory_excludes_shell_extensions() {
    use mix::evaluator::CapabilityPolicy;
    assert_eq!(include_str!("mcp_builtins.txt").lines().count(), 332);
    for name in [
        "json_parse",
        "re_match",
        "toml_parse",
        "date_parse",
        "run_argv",
        "write_file",
    ] {
        assert!(McpBuiltins.check_builtin(name).is_ok(), "{name}");
    }
    for name in [
        "tar_list",
        "http_get",
        "http_stream",
        "sqlopen",
        "ws_connect",
        "ws_on",
        "future_builtin",
    ] {
        assert!(McpBuiltins.check_builtin(name).is_err(), "{name}");
    }
}
impl mix::evaluator::CapabilityPolicy for McpBuiltins {
    fn check_builtin(&self, name: &str) -> Result<(), String> {
        if include_str!("mcp_builtins.txt").lines().any(|n| n == name) {
            Ok(())
        } else {
            Err(format!("{name} is outside the frozen MCP builtin surface"))
        }
    }
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    script: String,
    cwd: Option<String>,
}

fn worker_provenance() -> ::bus::RegisterProvenance {
    let build = buildinfo::build_info!();
    ::bus::RegisterProvenance {
        binary: Some("mix".into()),
        version: Some(build.version.into()),
        git_sha: Some(build.git_sha.into()),
        git_dirty: Some(build.git_dirty),
        build_time: Some(build.build_time.into()),
        pid: Some(std::process::id()),
        started_at: Some(buildinfo::now_rfc3339()),
        ..Default::default()
    }
}

#[derive(Clone, Default)]
pub(crate) struct CaptureBuffer(std::sync::Arc<std::sync::Mutex<(Vec<u8>, bool)>>);

impl CaptureBuffer {
    pub(crate) fn to_string_lossy(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().unwrap_or_else(|e| e.into_inner()).0).into_owned()
    }

    pub(crate) fn exceeded(&self) -> bool {
        self.0.lock().unwrap_or_else(|e| e.into_inner()).1
    }
}

impl Write for CaptureBuffer {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        if bytes.len() > OUTPUT_LIMIT.saturating_sub(state.0.len()) {
            state.1 = true;
            return Err(std::io::Error::other("Mix captured output exceeded 1 MiB"));
        }
        state.0.extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

struct AgentBusHandler {
    noded: connection::BrokerConnection,
}

impl mix::evaluator::BusHandler for AgentBusHandler {
    fn send<'a>(
        &'a self,
        target: &'a str,
        command: &'a str,
        args: &'a mix::value::Value,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = mix::error::MixResult<(i32, mix::value::Value)>> + 'a>,
    > {
        Box::pin(async move {
            // Fallible since mix 0.21.1 (non-finite numbers have no
            // JSON representation) — surface as an rc=10 error like any other
            // call failure, never a silently-mangled payload.
            let json_args = match mix::json::mix_to_json(args) {
                Ok(j) => j,
                Err(e) => {
                    return Ok((
                        10,
                        mix::value::Value::String(format!("send args not JSON-encodable: {e}")),
                    ));
                }
            };
            // call_typed keeps TRANSPORT failures as `Err` (→ `$rc = -1`,
            // RC_TRANSPORT) while a peer `rc >= 10` reply preserves its exact
            // status (→ `$rc = rc`) and a success carries its rc (0 or a
            // warning 5) — the Mix rc-band contract (was every Err → rc=10,
            // conflating transport with application error).
            let noded = self
                .noded
                .get()
                .await
                .map_err(|e| mix::error::MixError::RuntimeError {
                    span: None,
                    msg: format!("mcp send broker unavailable: {e}"),
                })?;
            match noded.call_typed(target, command, json_args).await {
                Ok(bus::native_client::PortReply::Ok { rc, value }) => {
                    Ok((i32::from(rc), mix::json::json_to_mix(value)))
                }
                Ok(bus::native_client::PortReply::AppError { rc, message }) => {
                    Ok((i32::from(rc), mix::value::Value::String(message)))
                }
                Err(e) => Err(mix::error::MixError::RuntimeError {
                    span: None,
                    msg: format!("mcp send transport failure: {e}"),
                }),
            }
        })
    }

    fn emit<'a>(
        &'a self,
        target: &'a str,
        command: &'a str,
        args: &'a mix::value::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = mix::error::MixResult<()>> + 'a>> {
        Box::pin(async move {
            // emit is fire-and-forget but has an error channel — a
            // non-JSON-encodable payload (non-finite number) raises rather
            // than emitting a mangled event.
            let json_args =
                mix::json::mix_to_json(args).map_err(|e| mix::error::MixError::RuntimeError {
                    span: None,
                    msg: format!("emit args not JSON-encodable: {e}"),
                })?;
            let noded = self
                .noded
                .get()
                .await
                .map_err(|e| mix::error::MixError::RuntimeError {
                    span: None,
                    msg: format!("mcp emit broker unavailable: {e}"),
                })?;
            noded.send(target, command, json_args).await.map_err(|e| {
                mix::error::MixError::RuntimeError {
                    span: None,
                    msg: format!("mcp emit transport failure: {e}; delivery outcome unknown"),
                }
            })?;
            Ok(())
        })
    }

    fn port_exists<'a>(
        &'a self,
        target: &'a str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = mix::error::MixResult<bool>> + 'a>>
    {
        Box::pin(async move {
            let noded = match self.noded.get().await {
                Ok(noded) => noded,
                Err(_) => return Ok(false),
            };
            match noded.list_services().await {
                Ok(services) => Ok(services.iter().any(|s| s == target)),
                Err(_) => Ok(false),
            }
        })
    }

    fn next_incoming<'a>(
        &'a self,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Option<mix::evaluator::IncomingEvent>> + 'a>,
    > {
        // MCP runs scripts synchronously for tool calls; it does not host
        // long-lived `on` handlers, so incoming delivery is not supported.
        Box::pin(async move { None })
    }
}

/// Run a Mix script in the internal isolated worker process only.
async fn run_mix_script(script: &str, cwd: Option<&str>) -> Result<(String, String, i32), String> {
    // Set working directory if requested
    if let Some(cwd) = cwd {
        let expanded = if cwd.starts_with("~/") {
            if let Some(mut home) = paths::env::os("HOME") {
                // Concatenate as the source did, including empty HOME and
                // its leading slash, while retaining non-UTF-8 OS bytes.
                home.push(&cwd[1..]);
                std::path::PathBuf::from(home)
            } else {
                std::path::PathBuf::from(cwd)
            }
        } else {
            std::path::PathBuf::from(cwd)
        };
        std::env::set_current_dir(&expanded)
            .map_err(|e| format!("ERROR: failed to set cwd to {}: {e}", expanded.display()))?;
    }

    // Parse
    let mut lexer = mix::lexer::Lexer::new(script);
    let tokens = lexer
        .tokenize()
        .map_err(|e| format!("ERROR: parse error: {e}"))?;
    let mut parser = mix::parser::Parser::new(tokens, script);
    let stmts = parser
        .parse_program()
        .map_err(|e| format!("ERROR: parse error: {e}"))?;

    // Create evaluator
    let stdout = CaptureBuffer::default();
    let stderr = CaptureBuffer::default();
    let mut eval =
        mix::evaluator::Evaluator::with_output(Box::new(stdout.clone()), Box::new(stderr.clone()));
    eval.set_capability_policy(std::rc::Rc::new(McpBuiltins));

    // Native connection is lazy even inside the worker. Pure local scripts
    // need no broker; the first send/emit/port_exists acquires an owned client.
    eval.set_bus_handler(std::rc::Rc::new(AgentBusHandler {
        noded: connection::BrokerConnection::for_current_agent(
            Some(worker_provenance()),
            Some(::paths::env::MCP_ALLOW_TCP),
        ),
    }));

    // Execute
    let execution = eval.execute(&stmts).await;
    if stdout.exceeded() || stderr.exceeded() {
        return Err("Mix captured output exceeded 1 MiB; effects may already have occurred".into());
    }
    match execution {
        Ok(_) => {
            let out = stdout.to_string_lossy();
            let err = stderr.to_string_lossy();
            Ok((out, err, 0))
        }
        Err(e) => {
            let out = stdout.to_string_lossy();
            let err = stderr.to_string_lossy();
            Ok((out, format!("{err}\nERROR: {e}"), 1))
        }
    }
}

/// Internal execution entrypoint, after the side-effect-free --version gate
/// and before logging/MCP startup. Worker stdout is tool output, never JSON-RPC.
pub fn run() -> i32 {
    crate::register_build_snapshot();
    let mut input = Vec::new();
    if let Err(e) = std::io::stdin()
        .take((INPUT_LIMIT + 1) as u64)
        .read_to_end(&mut input)
    {
        eprintln!("read Mix request: {e}");
        return 1;
    }
    if input.len() > INPUT_LIMIT {
        eprintln!("Mix request exceeds input limit");
        return 1;
    }
    let request: Request = match serde_json::from_slice(&input) {
        Ok(request) => request,
        Err(e) => {
            eprintln!("invalid Mix worker request: {e}");
            return 1;
        }
    };
    if request.script.len() > SCRIPT_LIMIT {
        eprintln!("Mix script exceeds input limit");
        return 1;
    }
    let runtime = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(e) => {
            eprintln!("start Mix runtime: {e}");
            return 1;
        }
    };
    let result = runtime.block_on(run_mix_script(&request.script, request.cwd.as_deref()));
    match result {
        Ok((stdout, stderr, code)) => {
            if let Err(e) = std::io::stdout().write_all(stdout.as_bytes()) {
                eprintln!("write Mix output: {e}");
                return 1;
            }
            if let Err(e) = std::io::stderr().write_all(stderr.as_bytes()) {
                eprintln!("write Mix stderr: {e}");
                return 1;
            }
            code
        }
        Err(error) => {
            eprintln!("{error}");
            1
        }
    }
}
