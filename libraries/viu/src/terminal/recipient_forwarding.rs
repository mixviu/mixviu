// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Gold's child-shell execution/task relay; no layout or prompt policy here.
use super::*;
use crate::terminal::protocol::{Request, mismatch, refusal_code, shell_refusal};
use crate::terminal::replay::Forward;

impl Recipient {
    #[expect(
        clippy::too_many_arguments,
        reason = "forwarding retains verified request identity and replay digest"
    )]
    pub(super) async fn forward_execute(
        &self,
        connection: &VerifiedConnection,
        request_epoch: HexBytes<16>,
        caller_label: &str,
        command_name: &str,
        request: &Request,
        identity: &str,
        digest: [u8; 32],
    ) -> Reply {
        let Some(child) = self
            .native
            .child_binding(request.target.pane_id.0, request.target.pane_generation.0)
        else {
            return Reply::error("FORBIDDEN");
        };
        // The child's own Source, not this pane's Target. They name the same
        // pane through different identities, and the child will refuse anything
        // that is not exactly its own.
        let child_target = json!({
            "broker_epoch": child.broker_epoch,
            "record": child.reference(),
            "instance_id": child.instance_id,
            "pane_id": child.pane_id,
            "pane_generation": child.pane_generation,
        });
        let (shell_command, body, sequence) = match command_name {
            "term.execute" => {
                let (Some(source), Some(generation)) =
                    (request.source.as_deref(), request.prompt_generation)
                else {
                    return Reply::error("INVALID_ARGUMENT");
                };
                let Some(sequence) = request.request_id.map(|id| id.0).filter(|id| *id > 0) else {
                    return Reply::error("INVALID_ARGUMENT");
                };
                // A submission is a mutation: it must carry the connection that
                // is making it, so a reconnected caller cannot inherit an
                // in-flight id.
                match request.request_epoch {
                    None => return Reply::error("INVALID_ARGUMENT"),
                    Some(epoch) if epoch != request_epoch => {
                        return Reply::error("UNKNOWN_OUTCOME");
                    }
                    Some(_) => {}
                }
                // Order matters. A caller request Term has ALREADY forwarded
                // maps to a stable child id, so re-forwarding it cannot execute
                // twice — the child's dedupe owns that decision. Consulting the
                // mapping first is what lets a retry get past Term's own
                // high-water mark, which is otherwise the thing that turns
                // "your answer was lost" into "unknown, forever".
                let forwarding = self
                    .replay
                    .lock()
                    .unwrap()
                    .forwarded_id(identity, sequence, digest);
                let forwarded = match forwarding {
                    Forward::Conflict => return Reply::refuse("CONFLICT", Some(mismatch())),
                    Forward::Retry(id) => id,
                    Forward::Fresh(id) => {
                        if let Err(reply) = self
                            .replay
                            .lock()
                            .unwrap()
                            .reserve_request_id(identity, sequence, digest)
                        {
                            return reply;
                        }
                        id
                    }
                };
                (
                    "shell.execute",
                    json!({
                        "version": 1,
                        "target": child_target,
                        "request_id": DecimalU64(forwarded),
                        "prompt_generation": generation,
                        "source": source,
                        // The shell authenticated Term, not this actor, so the
                        // shell renders it as relayed. Taken from Term's own
                        // trusted context — a caller cannot put a name here.
                        "on_behalf_of": caller_label,
                    }),
                    Some(sequence),
                )
            }
            // A task submission mirrors the evaluation one mechanically: same
            // epoch rule, same forwarded-id mapping, same retry discipline.
            // What it does NOT carry is `on_behalf_of` — a task never renders
            // into the pane, so there is no announcement for a principal to
            // appear in, and sending the field would be refused by the child's
            // `deny_unknown_fields`.
            "term.task.submit" => {
                let Some(sequence) = request.request_id.map(|id| id.0).filter(|id| *id > 0) else {
                    return Reply::error("INVALID_ARGUMENT");
                };
                match request.request_epoch {
                    None => return Reply::error("INVALID_ARGUMENT"),
                    Some(epoch) if epoch != request_epoch => {
                        return Reply::error("UNKNOWN_OUTCOME");
                    }
                    Some(_) => {}
                }
                // The SAME forwarded map as evaluations, keyed (identity,
                // sequence) and not split by kind: splitting it would let one
                // caller request id mint a fresh forwarded id on each surface
                // and BOTH execute.
                let forwarding = self
                    .replay
                    .lock()
                    .unwrap()
                    .forwarded_id(identity, sequence, digest);
                let forwarded = match forwarding {
                    Forward::Conflict => return Reply::refuse("CONFLICT", Some(mismatch())),
                    Forward::Retry(id) => id,
                    Forward::Fresh(id) => {
                        if let Err(reply) = self
                            .replay
                            .lock()
                            .unwrap()
                            .reserve_request_id(identity, sequence, digest)
                        {
                            return reply;
                        }
                        id
                    }
                };
                let mut body = json!({
                    "version": 1,
                    "target": child_target,
                    "request_id": DecimalU64(forwarded),
                    "env": request.env,
                });
                // Absent fields are OMITTED, never sent as null. A relayed null
                // is a present field of the wrong type, so the child answers
                // INVALID_REQUEST — a malformed-body complaint — where it
                // should be applying its own default or answering
                // INVALID_ARGUMENT about a value the caller actually chose.
                if let Some(cwd) = &request.cwd {
                    body["cwd"] = json!(cwd);
                }
                if let Some(timeout_ms) = request.timeout_ms {
                    body["timeout_ms"] = json!(timeout_ms);
                }
                // The discriminated union is relayed as the caller sent it;
                // Term does not choose a side, so "both" and "neither" reach
                // the child's own refusal rather than being resolved here.
                if let Some(source) = &request.source {
                    body["source"] = json!(source);
                }
                if let Some(argv) = &request.argv {
                    body["argv"] = json!(argv);
                }
                ("shell.task.submit", body, Some(sequence))
            }
            // Reading a result and cancelling are idempotent against one
            // immutable identity, so neither spends a request id.
            _ => {
                let Some(operation) = request.operation_id else {
                    return Reply::error("INVALID_ARGUMENT");
                };
                let child_operation = self
                    .replay
                    .lock()
                    .unwrap()
                    .child_operation(&request.target, operation.0);
                let Some(child_operation) = child_operation else {
                    return Reply::error("UNKNOWN_OUTCOME");
                };
                // The pane shell resolves operation IDs and command families.
                let shell_command = match command_name {
                    "term.exec.cancel" => "shell.execute.cancel",
                    "term.task.cancel" => "shell.task.cancel",
                    "term.task.result" => "shell.task.result",
                    _ => "shell.execute.result",
                };
                (
                    shell_command,
                    json!({
                        "version": 1,
                        "target": child_target,
                        "operation_id": child_operation,
                    }),
                    None,
                )
            }
        };
        let answer = tokio::time::timeout(
            Duration::from_secs(5),
            connection.client().call(&child.name, shell_command, body),
        )
        .await;
        // `retain` is the load-bearing distinction. A reply that came FROM the
        // child is that request's settled outcome and is retained so a retry
        // replays it. A local placeholder — Term's own timeout, or a reply too
        // large to deliver — is not an outcome at all: retaining it would make
        // every byte-identical retry replay the placeholder forever, when
        // re-forwarding would reach the child's own dedupe and get the real
        // answer. So placeholders are returned and NOT recorded.
        let (mut reply, mut retain) = match answer {
            Ok(Ok(value)) => {
                let mut value = value;
                value["target"] = json!(request.target);
                if let Some(sequence) = sequence {
                    value["operation_id_request"] = json!(DecimalU64(sequence));
                } else {
                    value["operation_id"] = json!(request.operation_id);
                }
                (Reply::ok(value), true)
            }
            // The child's refusal is ITS answer about ITS prompt. Term relays
            // it rather than replacing it with a guess, because BUSY and
            // STALE_GENERATION tell the caller two different things to do next.
            Ok(Err(error)) => {
                let body = error.to_string();
                // A refusal about the child's own LOAD settles nothing about
                // this request. Retaining one replays "too many tasks" for that
                // id forever, while the documented move for both codes is to
                // back off and retry the same submission — which only reaches
                // the child's dedupe if Term did not record the refusal.
                let transient = matches!(refusal_code(&body).as_deref(), Some("RESOURCE_LIMIT"));
                (shell_refusal(&body, sequence.is_some()), !transient)
            }
            // A submission whose answer never arrived may or may not have been
            // admitted. The caller's route forward is `term.exec.result`, or a
            // byte-identical retry that re-forwards to the child's dedupe —
            // which is only possible because this is not recorded.
            //
            // The REASON is logged because it cannot be relayed: the caller is
            // told "unknown", which is all Term honestly knows about its
            // request, but an operator holding the logs should not have to
            // guess whether the child was slow, gone, or never asked.
            Err(error) if sequence.is_some() => {
                eprintln!(
                    "term control: {command_name} -> {shell_command} child call failed ({error}); reporting unknown outcome"
                );
                (Reply::error("UNKNOWN_OUTCOME"), false)
            }
            Err(error) => {
                eprintln!(
                    "term control: {command_name} -> {shell_command} child call failed ({error})"
                );
                (Reply::error("DISCONNECTED"), false)
            }
        };
        if reply.body.len() > 256 * 1024 {
            reply = Reply::error(if sequence.is_some() {
                "UNKNOWN_OUTCOME"
            } else {
                "RESOURCE_LIMIT"
            });
            retain = false;
        }
        if let Some(sequence) = sequence
            && retain
        {
            self.replay.lock().unwrap().record_outcome(
                identity,
                &request.target,
                sequence,
                digest,
                &mut reply,
            );
        }
        reply
    }
}
