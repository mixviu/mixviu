// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded, owned in-process Mix work using the same Runtime mailbox. This is
//! an evaluator facility, not a Bus service or a second transport. Applications
//! supply the program and decide how its tagged result changes their model.
use crate::runtime::{BackendFactory, Program, Runtime, Wake, from_json, to_json};
use mix::{BusHandler, MixError, MixResult, evaluator::BusFuture, value::Value as MixValue};
use serde_json::{Value, json};
use std::{collections::BTreeMap, path::PathBuf, rc::Rc, sync::mpsc};
use tokio::sync::oneshot;

struct Start {
    spec: Value,
    reply: oneshot::Sender<(i32, Value)>,
}
struct Running {
    runtime: Runtime,
    sequence: u64,
    tag: Value,
}
pub struct ResultMessage {
    pub tag: Value,
    pub rc: u8,
    pub body: Value,
    pub elapsed_us: u128,
}
pub struct Jobs {
    tx: mpsc::SyncSender<Start>,
    rx: mpsc::Receiver<Start>,
    running: BTreeMap<String, Running>,
    limit: usize,
    wake: Wake,
    native_backend: Option<crate::runtime::BackendMaker>,
    services:crate::runtime::Services,
}
impl Jobs {
    pub fn new(limit: usize, wake: Wake) -> Self {
        assert!(limit > 0);
        let (tx, rx) = mpsc::sync_channel(8);
        Self {
            tx,
            rx,
            running: BTreeMap::new(),
            limit,
            wake,
            native_backend: None,
            services:Default::default(),
        }
    }
    pub fn set_native_backend(&mut self, maker: crate::runtime::BackendMaker) {
        self.native_backend = Some(maker);
    }
    pub fn set_services(&mut self,services:crate::runtime::Services){self.services=services;}
    pub fn backend(&self, base: Option<BackendFactory>) -> BackendFactory {
        let tx = self.tx.clone();
        let wake = self.wake.clone();
        Box::new(move || {
            Rc::new(Backend {
                base: base.map(|f| f()),
                tx,
                wake,
            })
        })
    }
    pub fn pending(&self) -> usize {
        self.running.len()
    }
    /// Native clients enter exactly the same bounded admission path as Mix.
    /// The returned receipt resolves when the owner has admitted or refused it.
    pub fn submit(&self, spec: Value) -> Result<oneshot::Receiver<(i32, Value)>, String> {
        let (reply, response) = oneshot::channel();
        self.tx
            .try_send(Start { spec, reply })
            .map_err(|_| "job admission queue is full or closed".to_owned())?;
        (self.wake)();
        Ok(response)
    }
    pub fn poll(&mut self) -> Vec<ResultMessage> {
        while let Ok(start) = self.rx.try_recv() {
            let id = start.spec["id"].as_str().unwrap_or("");
            if start.spec["cancel"] == true {
                let removed = self.running.remove(id).is_some();
                let _ = start.reply.send((0, json!({"cancelled":removed,"id":id})));
                continue;
            }
            let path = start.spec["path"].as_str();
            let source = start.spec["source"].as_str();
            if id.is_empty()
                || id.len() > 128
                || path.is_none()
                || source.is_none_or(|s| s.len() > 1_048_576)
                || !start.spec["command"].is_string()
            {
                let _ = start.reply.send((10, json!({"error_code":"INVALID_JOB"})));
                continue;
            }
            if self.running.len() >= self.limit || self.running.contains_key(id) {
                let _ = start.reply.send((10, json!({"error_code":"BUSY"})));
                continue;
            }
            let mut runtime = Runtime::with_services(
                2,
                self.wake.clone(),
                self.native_backend.as_ref().map(|f| f()),
                self.services.clone(),
            );
            let program = Program {
                path: PathBuf::from(path.unwrap()),
                source: source.unwrap().into(),
                scene: id.into(),
                citizen: "viu-job".into(),
            };
            let accepted = runtime.load(program).and_then(|_| {
                runtime.event(
                    start.spec["command"].as_str().unwrap().into(),
                    start.spec["args"].clone(),
                )
            });
            match accepted {
                Ok(sequence) => {
                    self.running.insert(
                        id.into(),
                        Running {
                            runtime,
                            sequence,
                            tag: json!({"id":id,"owner":start.spec["owner"]}),
                        },
                    );
                    let _ = start.reply.send((0, json!({"accepted":true,"id":id})));
                }
                Err(_) => {
                    let _ = start.reply.send((10, json!({"error_code":"JOB_CLOSED"})));
                }
            }
        }
        let mut done = Vec::new();
        let mut remove = Vec::new();
        for (id, job) in &mut self.running {
            for result in job.runtime.drain() {
                if result.sequence == job.sequence || result.error.is_some() {
                    let (rc, body) = if let Some(error) = result.error {
                        (20, json!({"error_code":"JOB_FAILED","message":error}))
                    } else if let Some(reply) = result.reply {
                        (reply.rc, reply.body)
                    } else {
                        (20, json!({"error_code":"JOB_MISSING_REPLY"}))
                    };
                    done.push(ResultMessage {
                        tag: job.tag.clone(),
                        rc,
                        body,
                        elapsed_us: result.elapsed.as_micros(),
                    });
                    remove.push(id.clone());
                    break;
                }
            }
        }
        for id in remove {
            self.running.remove(&id);
        }
        done
    }
    /// Cancels asynchronous work and joins every owned evaluator. As with
    /// Runtime, a synchronous syscall already entered must return first.
    pub fn cancel_all(&mut self) {
        self.running.clear();
    }
}
fn error(message: &str) -> MixError {
    MixError::RuntimeError {
        msg: message.into(),
        span: None,
    }
}
struct Backend {
    base: Option<Rc<dyn BusHandler>>,
    tx: mpsc::SyncSender<Start>,
    wake: Wake,
}
impl BusHandler for Backend {
    fn send<'a>(
        &'a self,
        target: &'a str,
        command: &'a str,
        args: &'a MixValue,
    ) -> BusFuture<'a, MixResult<(i32, MixValue)>> {
        Box::pin(async move {
            let (rc, result, _) = self.send_with_reply(target, command, args).await?;
            Ok((rc, result))
        })
    }
    fn send_with_reply<'a>(
        &'a self,
        target: &'a str,
        command: &'a str,
        args: &'a MixValue,
    ) -> BusFuture<'a, MixResult<(i32, MixValue, MixValue)>> {
        Box::pin(async move {
            if target == "viu-local" && ["jobs.start", "jobs.cancel"].contains(&command) {
                let mut spec = to_json(args)?;
                if !spec.is_object() {
                    return Err(error("job arguments must be an object"));
                }
                if command == "jobs.cancel" {
                    spec["cancel"] = json!(true);
                }
                let (reply, response) = oneshot::channel();
                if self.tx.try_send(Start { spec, reply }).is_err() {
                    let result = from_json(json!({"error_code":"BUSY"}))?;
                    return Ok((10, result.clone(), result));
                }
                (self.wake)();
                let (rc, result) =
                    tokio::time::timeout(std::time::Duration::from_secs(5), response)
                        .await
                        .map_err(|_| error("job owner timeout"))?
                        .map_err(|_| error("job owner closed"))?;
                let result = from_json(result)?;
                return Ok((rc, result.clone(), result));
            }
            self.base
                .as_ref()
                .ok_or_else(|| error("no native backend"))?
                .send_with_reply(target, command, args)
                .await
        })
    }
    fn emit<'a>(
        &'a self,
        target: &'a str,
        command: &'a str,
        args: &'a MixValue,
    ) -> BusFuture<'a, MixResult<()>> {
        Box::pin(async move {
            self.base
                .as_ref()
                .ok_or_else(|| error("no native backend"))?
                .emit(target, command, args)
                .await
        })
    }
    fn port_exists<'a>(&'a self, target: &'a str) -> BusFuture<'a, MixResult<bool>> {
        Box::pin(async move {
            if target == "viu-local" {
                Ok(true)
            } else {
                self.base
                    .as_ref()
                    .ok_or_else(|| error("no native backend"))?
                    .port_exists(target)
                    .await
            }
        })
    }
    fn next_incoming<'a>(&'a self) -> BusFuture<'a, Option<mix::evaluator::IncomingEvent>> {
        Box::pin(async { None })
    }
    fn subscribe_topic<'a>(&'a self, topic: &'a str) -> BusFuture<'a, MixResult<()>> {
        Box::pin(async move {
            self.base
                .as_ref()
                .ok_or_else(|| error("no native backend"))?
                .subscribe_topic(topic)
                .await
        })
    }
    fn unsubscribe_topic<'a>(&'a self, topic: &'a str) -> BusFuture<'a, MixResult<()>> {
        Box::pin(async move {
            self.base
                .as_ref()
                .ok_or_else(|| error("no native backend"))?
                .unsubscribe_topic(topic)
                .await
        })
    }
}
