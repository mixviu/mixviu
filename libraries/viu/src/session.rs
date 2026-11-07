// SPDX-License-Identifier: MIT OR Apache-2.0
//! Read-only desktop profile follower on one owned Mix worker, sharing the
//! application's native Bus identity. Hints coalesce; idle has no timer.
use crate::runtime::{BackendFactory, Program, Runtime, Wake};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

pub struct Session {
    worker: Runtime,
    pending: Option<u64>,
    dirty: bool,
    retry: Option<Instant>,
    pub snapshot: Value,
    pub error: String,
}
impl Session {
    pub fn embedded(
        app: &str,
        instance: &str,
        wake: Wake,
        backend: BackendFactory,
    ) -> Result<Self, String> {
        let source = format!(
            "{}\n{}",
            include_str!("../mix/session.mix"),
            include_str!("../mix/session-runtime.mix")
                .replace("$session=require(\"session.mix\")", "")
                .replace("$session.session_", "session_")
        );
        Self::start(
            PathBuf::from("viu-session.mix"),
            source,
            app,
            instance,
            wake,
            backend,
        )
    }
    pub fn new(
        path: PathBuf,
        app: &str,
        instance: &str,
        wake: Wake,
        backend: BackendFactory,
    ) -> Result<Self, String> {
        Self::start(
            path,
            include_str!("../mix/session-runtime.mix").into(),
            app,
            instance,
            wake,
            backend,
        )
    }
    fn start(
        path: PathBuf,
        source: String,
        app: &str,
        instance: &str,
        wake: Wake,
        backend: BackendFactory,
    ) -> Result<Self, String> {
        let mut worker = Runtime::new(2, wake, Some(backend));
        worker
            .load(Program {
                path,
                source,
                scene: "session".into(),
                citizen: app.into(),
            })
            .map_err(|e| format!("{e:?}"))?;
        let pending = worker
            .event(
                "session.start".into(),
                json!({"instance":instance,"app":app}),
            )
            .map_err(|e| format!("{e:?}"))?;
        Ok(Self {
            worker,
            pending: Some(pending),
            dirty: false,
            retry: None,
            snapshot: json!({}),
            error: String::new(),
        })
    }
    pub fn hint(&mut self, topic: &str, body: &Value) {
        if topic == "bus.connected"
            || topic == "theme.changed"
            || topic == "settingsd.desktop.look.default"
            || topic == "settingsd.desktop.keys.default.keymaps"
            || topic == "settingsd.desktop.keys.default.terminal"
            || topic == "settingsd.desktop.keys.default.email"
            || topic == "settingsd.desktop.keys.default.ced"
            || (topic == "noded.props.changed"
                && (body["path"].is_null() || body["path"] == "services.registered"))
        {
            self.dirty = true;
        }
    }
    pub fn poll(&mut self) -> Option<Value> {
        let mut changed = None;
        for result in self.worker.drain() {
            if self.pending != Some(result.sequence) {
                continue;
            }
            self.pending = None;
            if let Some(error) = result.error {
                self.error = error;
            }
            if let Some(snapshot) = result.model {
                self.error = snapshot["state"]["error"].as_str().unwrap_or("").into();
                self.retry = snapshot["state"]["retry"]
                    .as_f64()
                    .map(|s| Instant::now() + Duration::from_secs_f64(s.clamp(0.0, 30.0)));
                if snapshot["state"]["changed"] == true {
                    changed = Some(snapshot.clone());
                }
                self.snapshot = snapshot;
            }
        }
        if self.retry.is_some_and(|at| at <= Instant::now()) {
            self.retry = None;
            self.dirty = true;
        }
        if self.pending.is_none() && self.dirty {
            match self.worker.event("session.read".into(), json!({})) {
                Ok(seq) => {
                    self.pending = Some(seq);
                    self.dirty = false;
                }
                Err(error) => self.error = format!("{error:?}"),
            }
        }
        changed
    }
    pub fn deadline(&self) -> Option<Instant> {
        self.retry
    }
}
