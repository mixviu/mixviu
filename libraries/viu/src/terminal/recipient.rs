// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Verified native pane control. Mix owns the layout adapter; this recipient
//! owns admission, replay and the final PTY write permit.
use super::{
    Terminal,
    admission::Admission,
    control::Permit,
    protocol::{Reply, Target},
    replay::Replay,
    session_actor::{ControlLane, LaneFuture, NativeSession},
};
use bus::native_client::{
    VerifiedCommand, VerifiedConnection,
    session::{Deadline, Hello},
};
use bus::native_session::*;
use serde_json::json;
use std::sync::{
    Arc, Mutex, Weak,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::time::{Duration, Instant};
use super::metadata::Metadata;

#[path = "recipient_forwarding.rs"]
mod forwarding;
#[path = "recipient_metadata.rs"]
mod metadata_reader;

type PaneLookup = Arc<dyn Fn(u64) -> Option<Arc<Mutex<Terminal>>> + Send + Sync>;
type MetadataLookup = Arc<dyn Fn() -> Option<Metadata> + Send + Sync>;
struct InputNotice {
    actor: String,
    expected: usize,
    to: String,
    actor_epoch: HexBytes<16>,
    actor_connection: HexBytes<16>,
    target: Target,
    request_id: Option<DecimalU64>,
    written: u64,
}

pub(crate) struct Recipient {
    native: NativeSession,
    pane: PaneLookup,
    metadata: MetadataLookup,
    replay: Mutex<Replay>,
    permits: Mutex<Vec<Weak<Permit>>>,
    notice_tx: tokio::sync::mpsc::Sender<InputNotice>,
    notice_rx: Mutex<tokio::sync::mpsc::Receiver<InputNotice>>,
    wake: Arc<tokio::sync::Notify>,
    serving: AtomicBool,
}

struct SingleFlight<'a>(&'a AtomicBool);
impl Drop for SingleFlight<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

impl Recipient {
    pub fn new(native: NativeSession, pane: PaneLookup, metadata: MetadataLookup) -> Self {
        let (notice_tx, notice_rx) = tokio::sync::mpsc::channel(256);
        Self {
            native,
            pane,
            metadata,
            replay: Mutex::default(),
            permits: Mutex::default(),
            notice_tx,
            notice_rx: Mutex::new(notice_rx),
            wake: Arc::default(),
            serving: AtomicBool::new(false),
        }
    }

    async fn serve(
        &self,
        connection: &VerifiedConnection,
        parent: &SessionRecord,
        own: &(Hello, Deadline),
        event: &VerifiedCommand,
    ) -> Reply {
        assert!(
            !self.serving.swap(true, Ordering::AcqRel),
            "terminal control dispatch is not re-entrant"
        );
        let _single = SingleFlight(&self.serving);
        if !connection.client().is_connected() {
            self.invalidate(None);
        }
        let mut admitted = match Admission::check(connection, parent, own, event) {
            Ok(admitted) => admitted,
            Err(reply) => return reply,
        };
        {
            let mut replay = self.replay.lock().unwrap();
            if admitted.operation == "term.operation" {
                return admitted.request.operation_id.map_or_else(
                    || Reply::error("INVALID_ARGUMENT"),
                    |id| replay.operation(&admitted.request.target, id.0),
                );
            }
            if let Some(sequence) = admitted.request.request_id {
                if let Some(reply) = replay.lookup(&admitted.identity, sequence.0, admitted.digest)
                {
                    return reply;
                }
            }
        }
        if matches!(
            admitted.operation.as_str(),
            "term.execute"
                | "term.exec.result"
                | "term.exec.cancel"
                | "term.task.submit"
                | "term.task.result"
                | "term.task.cancel"
        ) {
            return self
                .forward_execute(
                    connection,
                    admitted.epoch,
                    &admitted.label,
                    &admitted.operation,
                    &admitted.request,
                    &admitted.identity,
                    admitted.digest,
                )
                .await;
        }
        let metadata_read=matches!(admitted.operation.as_str(), "term.session" | "term.list" | "term.tabs" | "term.panes");
        if !metadata_read && !matches!(admitted.operation.as_str(), "term.snapshot" | "term.type") {
            return Reply::error("UNSUPPORTED");
        }
        // Do not hold retained-state locks while waiting for the pane: close
        // owns that pane during bounded actor cleanup. Dispatch remains serial.
        let target = admitted.request.target.clone();
        let permit = Arc::new(Permit {
            connection: Arc::new(connection.client().connection_liveness()),
            live: AtomicBool::new(true),
            parent: own.1.clone(),
            hello: own.0.clone(),
            until: Instant::now() + Duration::from_secs(2),
            pane: self
                .native
                .pane_guard(target.pane_id.0, target.pane_generation.0),
            written: AtomicU64::new(0),
            notice: Mutex::new(None),
        });
        let Some(pane) = (self.pane)(target.pane_id.0) else {
            return Reply::error("FORBIDDEN");
        };
        let pane = pane.lock().unwrap();
        if !permit.valid() {
            return Reply::error("FORBIDDEN");
        }
        if metadata_read {
            let foreground=pane.listener.foreground_generation();
            drop(pane);
            return self.describe(&admitted,parent,&permit,foreground);
        }
        let mut replay = self.replay.lock().unwrap();
        let mutation = admitted.capability == Capability::Input;
        if mutation {
            if let Err(reply) = admitted.mutation_epoch() {
                return reply;
            }
            let Some(sequence) = admitted
                .request
                .request_id
                .map(|id| id.0)
                .filter(|id| *id > 0)
            else {
                return Reply::error("INVALID_ARGUMENT");
            };
            if let Err(reply) =
                replay.reserve_request_id(&admitted.identity, sequence, admitted.digest)
            {
                return reply;
            }
        }
        if !permit.valid() {
            return Reply::error("FORBIDDEN");
        }
        let mut reply = match admitted.operation.as_str() {
            "term.snapshot" if admitted.request.contents => {
                Reply::ok(json!({"target":target,"text":pane.snapshot()}))
            }
            "term.snapshot" => Reply::ok(
                json!({"target":target,"foreground_generation":pane.listener.foreground_generation()}),
            ),
            "term.type" => match (
                admitted.request.text.as_deref(),
                admitted.request.foreground_generation,
            ) {
                (Some(text), Some(generation)) => {
                    let mut notice = InputNotice {
                        actor: admitted.identity.clone(),
                        expected: text.len(),
                        to: event.command().from.clone(),
                        actor_epoch: parent.broker_epoch,
                        actor_connection: admitted.epoch,
                        target: target.clone(),
                        request_id: admitted.request.request_id,
                        written: 0,
                    };
                    let tx = self.notice_tx.clone();
                    let wake = self.wake.clone();
                    *permit.notice.lock().unwrap() = Some(Box::new(move |written| {
                        notice.written = written;
                        if tx.try_send(notice).is_ok() {
                            wake.notify_one();
                        }
                    }));
                    match pane
                        .listener
                        .control_text(text, generation.0, permit.clone())
                    {
                        Ok(()) => {
                            let mut permits = self.permits.lock().unwrap();
                            permits.retain(|p| p.strong_count() > 0);
                            permits.push(Arc::downgrade(&permit));
                            Reply::ok(
                                json!({"status":"accepted","outcome":"unknown","request_id":admitted.request.request_id}),
                            )
                        }
                        Err(code) => Reply::error(code),
                    }
                }
                _ => Reply::error("INVALID_ARGUMENT"),
            },
            _ => unreachable!(),
        };
        if reply.body.len() > 256 * 1024 {
            if !mutation {
                return Reply::error("RESOURCE_LIMIT");
            }
            reply = Reply::error("UNKNOWN_OUTCOME");
        }
        if mutation {
            replay.record_local_outcome(
                &admitted.identity,
                &target,
                admitted.request.request_id.unwrap().0,
                admitted.digest,
                &mut reply,
            );
        }
        reply
    }
}

impl ControlLane for Recipient {
    fn wake(&self) -> Arc<tokio::sync::Notify> {
        self.wake.clone()
    }
    fn invalidate(&self, target: Option<&RecordRef>) {
        self.permits.lock().unwrap().retain(|weak| {
            let Some(permit) = weak.upgrade() else {
                return false;
            };
            if target.is_none_or(|t| {
                let r = permit.parent.target();
                r.record_id == t.record_id
                    && r.incarnation == t.incarnation
                    && r.binding_generation.0 <= t.binding_generation.0
            }) {
                permit.revoke();
            }
            permit.valid()
        });
    }
    fn dispatch<'a>(
        &'a self,
        connection: &'a VerifiedConnection,
        parent: &'a SessionRecord,
        own: &'a (Hello, Deadline),
        event: &'a VerifiedCommand,
    ) -> LaneFuture<'a, Reply> {
        Box::pin(async move { self.serve(connection, parent, own, event).await })
    }
    fn flush_notices<'a>(&'a self, connection: &'a VerifiedConnection) -> LaneFuture<'a, ()> {
        Box::pin(async move {
            self.permits.lock().unwrap().retain(|weak| {
                let Some(permit) = weak.upgrade() else {
                    return false;
                };
                if !permit.valid() {
                    permit.revoke();
                    return false;
                }
                true
            });
            let budget = Instant::now() + Duration::from_millis(200);
            for _ in 0..16 {
                if Instant::now() >= budget {
                    break;
                }
                let Some(notice) = self.notice_rx.lock().unwrap().try_recv().ok() else {
                    break;
                };
                let completed = notice.written >= notice.expected as u64;
                self.replay.lock().unwrap().settle_input(
                    &notice.actor,
                    notice.request_id,
                    notice.expected,
                    notice.written,
                );
                let message=bus::bus::BusMessage::new()
                    .with_header("from",&connection.client().name()).with_header("to",&notice.to)
                    .with_header("type","event").with_header("command","term.input.revoked")
                    .with_header("recipient_connection",&json!({"broker_epoch":notice.actor_epoch,"connection_id":notice.actor_connection}).to_string())
                    .with_body(&json!({"target":&notice.target,"request_id":notice.request_id,"status":"revoked","outcome":if completed {"complete"} else {"partial_or_unknown"},"delivered_bytes_lower_bound":notice.written}).to_string());
                if tokio::time::timeout(
                    Duration::from_millis(100),
                    connection.client().send_raw(&message),
                )
                .await
                .is_err()
                {
                    let _ = self.notice_tx.try_send(notice);
                    break;
                }
            }
        })
    }
}
