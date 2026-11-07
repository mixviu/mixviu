// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Recipient admission extracted from gold control dispatch. This is the
//! recipient freshness check, not pane/layout admission or a mutation permit.
use super::protocol::{self, Reply, Request};
use bus::native_client::session::{Deadline, Hello};
use bus::native_client::{VerifiedCommand, VerifiedConnection};
use bus::native_session::*;

pub(crate) struct Admission {
    pub request: Request,
    pub operation: String,
    pub capability: Capability,
    pub identity: String,
    pub epoch: HexBytes<16>,
    pub label: String,
    pub digest: [u8; 32],
    grantless: bool,
}

impl Admission {
    /// Run before replay lookup, pane access, or application callbacks, in the
    /// same order as gold. Own lease evidence comes from the actor's renew
    /// cadence; this path never establishes freshness with a new broker RPC.
    pub fn check(
        connection: &VerifiedConnection,
        parent: &SessionRecord,
        own: &(Hello, Deadline),
        event: &VerifiedCommand,
    ) -> Result<Self, Reply> {
        if !connection.client().is_connected() {
            return Err(Reply::error("DISCONNECTED"));
        }
        let command = event.command();
        if !protocol::stamped(command) {
            return Err(Reply::error("INVALID_ARGUMENT"));
        }
        let mut request = Request::decode(&command.body)?;
        let operation = request.operation(&command.command)?.to_owned();
        if (request.request_id.is_some() || request.operation_id.is_some())
            && (request.target.instance_id != parent.instance_id
                || request.target.incarnation != parent.incarnation)
        {
            return Err(Reply::error("UNKNOWN_OUTCOME"));
        }
        if !protocol::target_matches(parent, &request.target) {
            return Err(Reply::error("FORBIDDEN"));
        }
        let capability = protocol::capability_of(&operation, request.contents)
            .ok_or_else(|| Reply::error("UNSUPPORTED"))?;
        if own.1.target() != &parent.reference()
            || !own.1.is_live(&own.0).unwrap_or(false)
        {
            return Err(Reply::error("FORBIDDEN"));
        }
        Ok(Self {
            request,
            operation,
            capability,
            identity: protocol::caller_key(event),
            epoch: protocol::caller_epoch(event, parent.broker_epoch),
            label: protocol::caller_label(event),
            digest: protocol::request_digest(&command.command, &command.body),
            grantless: event.trusted_context().is_none_or(|actor| actor.session.is_none()),
        })
    }

    /// Gold checks mutation epoch after retained outcomes and affected-pane
    /// validation. Keep this separate so replay can survive pane removal.
    pub fn mutation_epoch(&mut self) -> Result<(), Reply> {
        if self.grantless && self.request.request_epoch.is_none() {
            self.request.request_epoch = Some(self.epoch);
        }
        match self.request.request_epoch {
            None => Err(Reply::error("INVALID_ARGUMENT")),
            Some(epoch) if epoch != self.epoch => Err(Reply::error("UNKNOWN_OUTCOME")),
            Some(_) => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal::session_actor::{ControlLane, LaneFuture, Supervisor};
    use serde_json::{Value, json};
    use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

    struct Probe {
        wake: Arc<tokio::sync::Notify>,
        stale_parent: AtomicBool,
    }
    impl ControlLane for Probe {
        fn wake(&self) -> Arc<tokio::sync::Notify> {self.wake.clone()}
        fn invalidate(&self, _: Option<&RecordRef>) {}
        fn flush_notices<'a>(&'a self, _: &'a VerifiedConnection) -> LaneFuture<'a, ()> {
            Box::pin(async {})
        }
        fn dispatch<'a>(&'a self, connection: &'a VerifiedConnection, parent: &'a SessionRecord, own: &'a (Hello, Deadline), event: &'a VerifiedCommand) -> LaneFuture<'a, Reply> {
            Box::pin(async move {
                let mut parent = parent.clone();
                if self.stale_parent.load(Ordering::Acquire) {parent.binding_generation.0 += 1;}
                match Admission::check(connection, &parent, own, event) {
                    Err(reply) => reply,
                    Ok(mut admitted) => {
                        if event.command().command == "term.type" {
                            if let Err(reply) = admitted.mutation_epoch() {return reply;}
                        }
                        Reply::ok(json!({"operation":admitted.operation,"epoch":admitted.epoch,"identity":admitted.identity,"label":admitted.label}))
                    }
                }
            })
        }
    }

    #[test]
    fn real_broker_stamps_scope_epochs_and_recipient_lease_are_required() {
        use bus::native_client::{NodedClient, UnixConnectOutcome};
        let broker = test_broker::Broker::start();
        let supervisor = Supervisor::with_options(broker.options(), broker.url.clone()).unwrap();
        crate::terminal::session_actor::tests::wait_ready(&supervisor.handle, 1);
        let probe = Arc::new(Probe {wake: Arc::default(), stale_parent: AtomicBool::new(false)});
        let lane: Arc<dyn ControlLane> = probe.clone();
        supervisor.handle.install_control(&lane);
        let runtime = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        runtime.block_on(async {
            let UnixConnectOutcome::VerifiedUnix(caller) = NodedClient::connect_unix("", &broker.url, &broker.options(), None).await.unwrap() else {panic!("verified observer required")};
            let parent = caller.session_list().await.unwrap().records.into_iter().find(|r| r.role == Role::Term && r.state == BindingState::Attached).unwrap();
            let target = json!({"instance_id":parent.instance_id,"incarnation":parent.incarnation,"pane_id":"1","pane_generation":"1"});
            let invoke = async |command: &str, body: Value| {
                let (rc, body, _) = tokio::time::timeout(std::time::Duration::from_secs(6), caller.client().call_with_headers_raw(&parent.name, command, &Default::default(), &body.to_string())).await.unwrap().unwrap();
                (rc, serde_json::from_str::<Value>(&body).unwrap())
            };
            let (rc, state) = invoke("term.session", json!({"target":target})).await;
            assert_eq!(rc, 0, "{state}");
            assert_eq!(state["operation"], "term.session");
            let (rc, mutation) = invoke("term.type", json!({"target":target,"request_id":"1","text":"probe only"})).await;
            assert_eq!(rc, 0, "{mutation}");
            assert_eq!(state["epoch"], mutation["epoch"]);
            let wrong_epoch = invoke("term.type", json!({"target":target,"request_id":"2","request_epoch":"ff".repeat(16)})).await;
            assert_eq!(wrong_epoch, (10,json!({"error_code":"UNKNOWN_OUTCOME"})));
            let mut wrong = target.clone();
            wrong["instance_id"] = json!("ee".repeat(16));
            assert_eq!(invoke("term.session",json!({"target":wrong})).await, (10,json!({"error_code":"FORBIDDEN"})));
            assert_eq!(invoke("term.type",json!({"target":wrong,"request_id":"3"})).await, (10,json!({"error_code":"UNKNOWN_OUTCOME"})));
            assert_eq!(invoke("term.session",json!({"target":target,"actor":"forged"})).await, (10,json!({"error_code":"INVALID_ARGUMENT"})));
            probe.stale_parent.store(true, Ordering::Release);
            assert_eq!(invoke("term.session",json!({"target":target})).await, (10,json!({"error_code":"FORBIDDEN"})));
            caller.client().close().await;
        });
    }
}
