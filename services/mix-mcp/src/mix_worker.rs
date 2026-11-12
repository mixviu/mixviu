// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>

//! Isolate the !Send Mix evaluator, process cwd and inherited stdio in an
//! owned sibling Mix worker. Only the parent speaks MCP. Cancellation terminates the
//! worker process group; it cannot undo effects already committed through Bus.

use std::process::Stdio;

use rmcp::model::CallToolResult;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};

const SCRIPT_LIMIT: usize = 256 * 1024;

const OUTPUT_LIMIT: usize = 1024 * 1024;
pub(crate) const WORKER_ARG: &str = "--internal-mix-worker";

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    script: String,
    cwd: Option<String>,
}

/// Own descendants as well as the worker. A dropped MCP future must not leave
/// evaluator threads changing the server's cwd or holding its transport fds.
struct ProcessGroup(std::sync::atomic::AtomicU32);

impl ProcessGroup {
    fn terminate(&self) {
        let pid = self.0.swap(0, std::sync::atomic::Ordering::Relaxed);
        if pid == 0 {
            return;
        }
        #[cfg(unix)]
        // SAFETY: only the child group created by process_group(0) is addressed.
        unsafe {
            libc::kill(-(pid as i32), libc::SIGKILL);
        }
    }
}

impl Drop for ProcessGroup {
    fn drop(&mut self) {
        self.terminate();
    }
}

async fn bounded_read(mut stream: impl AsyncRead + Unpin) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    (&mut stream)
        .take((OUTPUT_LIMIT + 1) as u64)
        .read_to_end(&mut bytes)
        .await
        .map_err(|e| format!("worker output read failed: {e}"))?;
    if bytes.len() > OUTPUT_LIMIT {
        return Err(format!(
            "Mix output exceeded {OUTPUT_LIMIT} bytes; execution stopped, effects may already have occurred"
        ));
    }
    Ok(bytes)
}

pub(crate) async fn execute(script: String, cwd: Option<String>) -> CallToolResult {
    match execute_inner(script, cwd).await {
        Ok((stdout, stderr, exit_code)) => {
            let text = if stdout.is_empty() && stderr.is_empty() {
                "(no output)".into()
            } else if stderr.is_empty() {
                stdout.clone()
            } else {
                format!("{stdout}\n--- stderr ---\n{stderr}")
            };
            let mut result = CallToolResult::structured(serde_json::json!(
                super::tool_output::MixExecutionObservation {
                    result: text,
                    stdout,
                    stderr,
                    exit_code,
                }
            ));
            result.is_error = Some(exit_code != 0);
            result
        }
        Err(message) => CallToolResult::error(vec![rmcp::model::ContentBlock::text(message)]),
    }
}

async fn execute_inner(
    script: String,
    cwd: Option<String>,
) -> Result<(String, String, i32), String> {
    if script.len() > SCRIPT_LIMIT {
        return Err(format!(
            "script exceeds {SCRIPT_LIMIT} bytes; nothing was started"
        ));
    }
    let input = serde_json::to_vec(&Request { script, cwd })
        .map_err(|e| format!("worker request serialization failed: {e}"))?;
    let mut command = tokio::process::Command::new(
        std::env::current_exe()
            .map_err(|e| format!("resolve adapter executable: {e}"))?
            .with_file_name("mix"),
    );
    command
        .arg(WORKER_ARG)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.as_std_mut().process_group(0);
    }
    let mut child = command
        .spawn()
        .map_err(|e| format!("start Mix worker: {e}"))?;
    let group = ProcessGroup(std::sync::atomic::AtomicU32::new(
        child.id().ok_or("Mix worker has no pid")?,
    ));
    let mut stdin = child.stdin.take().ok_or("Mix worker has no stdin")?;
    let stdout = child.stdout.take().ok_or("Mix worker has no stdout")?;
    let stderr = child.stderr.take().ok_or("Mix worker has no stderr")?;
    stdin
        .write_all(&input)
        .await
        .map_err(|e| format!("write Mix request: {e}"))?;
    stdin
        .shutdown()
        .await
        .map_err(|e| format!("close Mix request: {e}"))?;
    drop(stdin);
    // Read both pipes while waiting, otherwise a child filling either pipe
    // deadlocks. A read bound failure drops the child and its group guard.
    let (stdout, stderr, status) =
        tokio::try_join!(bounded_read(stdout), bounded_read(stderr), async {
            let status = child
                .wait()
                .await
                .map_err(|e| format!("wait Mix worker: {e}"))?;
            // A completed script does not leave descendants holding the output
            // pipes open indefinitely. Persistent applications belong to their
            // owning native service lifecycle, not this transient worker.
            group.terminate();
            Ok(status)
        })?;
    Ok((
        String::from_utf8_lossy(&stdout).into_owned(),
        String::from_utf8_lossy(&stderr).into_owned(),
        status.code().unwrap_or(1),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn oversized_script_is_rejected_before_starting_the_interpreter() {
        let error = execute_inner("x".repeat(SCRIPT_LIMIT + 1), None)
            .await
            .unwrap_err();
        assert!(error.contains("nothing was started"));
    }

    #[tokio::test]
    async fn output_bounds_accept_the_limit_and_reject_one_extra_byte() {
        let exact = vec![b'x'; OUTPUT_LIMIT];
        assert_eq!(
            bounded_read(exact.as_slice()).await.unwrap().len(),
            OUTPUT_LIMIT
        );
        let extra = vec![b'x'; OUTPUT_LIMIT + 1];
        let error = bounded_read(extra.as_slice()).await.unwrap_err();
        assert!(error.contains("effects may already have occurred"));
    }
}
