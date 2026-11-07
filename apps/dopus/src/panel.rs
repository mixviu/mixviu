// SPDX-License-Identifier: MIT OR Apache-2.0
//! Generic owned Mix panel host. The panel's workflow and composition live in Mix.
use super::{Reply, Value, json};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};
use viu::runtime::{Program, Runtime, Services, Wake};
pub struct Panel {
    worker: Runtime,
    pub form: Value,
    pending: BTreeMap<u64, Option<Reply>>,
    epoch: u64,
    dirty: Arc<AtomicBool>,
    _hook: macros::session::ChangeHook,
}
impl Panel {
    pub fn new(path: PathBuf, services: Services, wake: Wake) -> Result<Self, String> {
        let mut worker = Runtime::with_services(32, wake.clone(), None, services.clone());
        worker
            .load(Program {
                source: std::fs::read_to_string(&path).map_err(|e| e.to_string())?,
                path,
                scene: "macros".into(),
                citizen: "dopus-macros-panel".into(),
            })
            .map_err(|e| format!("{e:?}"))?;
        let dirty = Arc::new(AtomicBool::new(false));
        let flag = dirty.clone();
        let hook: macros::session::ChangeHook = Arc::new(move || {
            flag.store(true, Ordering::Release);
            wake();
        });
        if let Some(session) = services.macros {
            session.observe_changes(&hook);
        }
        Ok(Self {
            worker,
            form: json!({"open":false}),
            pending: BTreeMap::new(),
            epoch: 0,
            dirty,
            _hook: hook,
        })
    }
    pub fn active(&self) -> bool {
        self.form["open"] == true || !self.pending.is_empty()
    }
    pub fn pending(&self) -> bool {
        !self.pending.is_empty()
    }
    pub fn call(
        &mut self,
        mut args: Value,
        reply: Option<Reply>,
    ) -> Result<(), (String, Option<Reply>)> {
        if args["command"] == "open" {
            self.epoch += 1;
            args["epoch"] = json!(self.epoch);
        }
        match self.worker.event("macros.call".into(), args) {
            Ok(sequence) => {
                self.pending.insert(sequence, reply);
                Ok(())
            }
            Err(error) => Err((format!("{error:?}"), reply)),
        }
    }
    pub fn poll(&mut self) -> (bool, Option<String>) {
        let mut changed = false;
        let mut failed = None;
        for result in self.worker.drain() {
            let reply = self.pending.remove(&result.sequence).flatten();
            let action_error = reply
                .is_none()
                .then(|| {
                    result
                        .reply
                        .as_ref()
                        .and_then(|reply| (reply.rc >= 10).then(|| reply.body.to_string()))
                })
                .flatten();
            let error = result.error.or_else(|| {
                (result.generation != self.worker.generation()).then(|| "STALE_GENERATION".into())
            });
            if error.is_none() {
                if let Some(model) = result.model {
                    self.form = model["form"].clone();
                    changed = true;
                }
            }
            if let Some(reply) = reply {
                let _ = reply.send(if let Some(error) = &error {
                    (20, json!({"error_code":"PANEL_FAILED","message":error}))
                } else {
                    result
                        .reply
                        .map(|r| (r.rc, r.body))
                        .unwrap_or((20, json!({"error_code":"MISSING_REPLY"})))
                });
            }
            if error.is_some() {
                failed = error;
            } else if action_error.is_some() {
                failed = action_error;
            }
        }
        if self.dirty.load(Ordering::Acquire)
            && self.form["open"] == true
            && self.pending.is_empty()
        {
            if self.call(json!({"command":"refresh"}), None).is_ok() {
                self.dirty.store(false, Ordering::Release);
            }
        }
        (changed, failed)
    }
}
impl Drop for Panel {
    fn drop(&mut self) {
        for (_, reply) in std::mem::take(&mut self.pending) {
            if let Some(reply) = reply {
                let _ = reply.send((20, json!({"error_code":"OWNER_CLOSED"})));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn call(panel: &mut Panel, args: Value) -> (u8, Value) {
        let (tx, mut rx) = tokio::sync::oneshot::channel();
        assert!(panel.call(args, Some(tx)).is_ok());
        let until = std::time::Instant::now() + std::time::Duration::from_secs(8);
        loop {
            let (_, error) = panel.poll();
            assert!(error.is_none(), "{error:?}");
            if let Ok(answer) = rx.try_recv() {
                eprintln!("macro panel answer: {answer:?}");
                return answer;
            }
            assert!(std::time::Instant::now() < until, "panel did not answer");
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
    }
    fn answer(panel: &mut Panel, choice: &str, field: Value) -> (u8, Value) {
        call(
            panel,
            json!({"command":"answer","choice":choice,"field":field,"instance":panel.form["instance"]}),
        )
    }
    #[test]
    fn shared_mix_macro_panel_creates_edits_refuses_stale_and_removes_real_source() {
        let dir = tempfile::tempdir().unwrap();
        let session = macros::Session::with_roots(
            "dopus",
            Some(macros::Roots {
                builtin: dir.path().join("builtin"),
                user: dir.path().join("user"),
                home: dir.path().into(),
                mix: dir.path().join("deliberately-absent-interpreter"),
            }),
        );
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../libraries/viu/mix/macros.mix");
        let mut panel = Panel::new(
            path,
            Services {
                macros: Some(session.clone()),
                ..Default::default()
            },
            Arc::new(|| {}),
        )
        .unwrap();
        assert_eq!(call(&mut panel, json!({"command":"open"})).0, 0);
        assert_eq!(answer(&mut panel, "new", Value::Null).0, 0);
        assert_eq!(answer(&mut panel, "create", json!("Panel sample")).0, 0);
        let entry = session.catalogue().unwrap().entries[0].clone();
        assert_eq!(answer(&mut panel, "edit", Value::Null).0, 0);
        let instance = panel.form["instance"].clone();
        let source = "-- title: Panel sample\n-- target: email\nprint(42)\n";
        assert_eq!(answer(&mut panel, "save", json!(source)).0, 0);
        assert_eq!(std::fs::read_to_string(&entry.path).unwrap(), source);
        assert_eq!(answer(&mut panel, "run-target", Value::Null).0, 0);
        assert_eq!(answer(&mut panel, "run-on", json!("forbidden-app")).0, 10);
        assert_eq!(answer(&mut panel, "back", Value::Null).0, 0);
        assert_eq!(
            answer(&mut panel, &format!("select:{}", entry.id), Value::Null).0,
            0
        );
        assert_eq!(
            call(
                &mut panel,
                json!({"command":"answer","choice":"remove","instance":instance})
            )
            .0,
            10
        );
        assert!(entry.path.exists());
        assert_eq!(answer(&mut panel, "remove", Value::Null).0, 0);
        assert_eq!(answer(&mut panel, "confirm-remove", Value::Null).0, 0);
        assert!(!entry.path.exists());
        assert_eq!(answer(&mut panel, "new", Value::Null).0, 0);
        assert_eq!(answer(&mut panel, "create", json!("Moved sample")).0, 0);
        let moved = session.catalogue().unwrap().entries[0].clone();
        assert_eq!(answer(&mut panel, "move-other", Value::Null).0, 0);
        assert_eq!(answer(&mut panel, "move-to", json!("email")).0, 0);
        assert!(!moved.path.exists());
        assert!(
            dir.path()
                .join("user/email")
                .join(moved.path.file_name().unwrap())
                .is_file()
        );
        assert_eq!(answer(&mut panel, "close", Value::Null).0, 0);
        assert_eq!(panel.form["open"], false);
        assert!(
            panel
                .call(
                    json!({"command":"answer","choice":"remove","instance":"expired"}),
                    None
                )
                .is_ok()
        );
        let until = std::time::Instant::now() + std::time::Duration::from_secs(8);
        loop {
            let (_, error) = panel.poll();
            if let Some(error) = error {
                assert!(error.contains("STALE"), "{error}");
                break;
            }
            assert!(
                std::time::Instant::now() < until,
                "UI refusal was not surfaced"
            );
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        session.shutdown().unwrap();
    }
}
