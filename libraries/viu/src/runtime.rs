// SPDX-License-Identifier: MIT OR Apache-2.0
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
    mpsc,
};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use mix::evaluator::{BusFuture, Evaluator, IncomingEvent, ReservedOutcome, ServeRuntime};
use mix::{BusHandler, MixError, MixResult, value::Value};
use serde_json::{Value as Json, json};

pub type Wake = Arc<dyn Fn() + Send + Sync>;
pub type BackendFactory = Box<dyn FnOnce() -> Rc<dyn BusHandler> + Send>;

/// Per-request log retention has a fixed ceiling, including a noisy script.
#[derive(Clone, Default)]
struct SharedBuf(Rc<RefCell<(Vec<u8>, bool)>>);
impl SharedBuf {
    fn new() -> Self {
        Self::default()
    }
    fn take(&self) -> Vec<u8> {
        let (mut bytes, truncated) = std::mem::take(&mut *self.0.borrow_mut());
        if truncated {
            bytes.extend_from_slice(b"\n[viu output truncated]\n");
        }
        bytes
    }
}
impl std::io::Write for SharedBuf {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let mut buffer = self.0.borrow_mut();
        let remaining = 65_536usize.saturating_sub(buffer.0.len());
        buffer
            .0
            .extend_from_slice(&bytes[..bytes.len().min(remaining)]);
        buffer.1 |= bytes.len() > remaining;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[derive(Clone)]
pub struct Program {
    pub path: PathBuf,
    pub source: String,
    pub scene: String,
    pub citizen: String,
}

#[derive(Debug)]
pub struct Completion {
    pub generation: u64,
    pub sequence: u64,
    pub model: Option<Json>,
    pub error: Option<String>,
    pub elapsed: Duration,
    pub stdout: String,
    pub stderr: String,
    pub behaviour_available: bool,
}

enum RunFailure {
    Evaluation(String),
    Deadline,
    OwnerDropped,
}
impl RunFailure {
    fn message(self) -> String {
        match self {
            Self::Evaluation(message) => message,
            Self::Deadline => "behaviour deadline exceeded; reload required".into(),
            Self::OwnerDropped => "owner dropped".into(),
        }
    }
}

enum Request {
    Load(u64, Program),
    Event(u64, u64, String, Json, bool),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Refusal {
    Full,
    Closed,
}

/// One bounded, ordered mailbox of semantic operations. No operation is coalesced.
/// `Full` means the caller must keep/retry that operation or show a refusal.
pub struct Runtime {
    tx: Option<mpsc::SyncSender<Request>>,
    rx: Option<mpsc::Receiver<Completion>>,
    thread: Option<JoinHandle<()>>,
    interrupt: Arc<AtomicBool>,
    stop: Arc<AtomicBool>,
    generation: u64,
    sequence: u64,
}

impl Runtime {
    pub fn new(capacity: usize, wake: Wake, backend: Option<BackendFactory>) -> Self {
        assert!(capacity > 0);
        let (tx, commands) = mpsc::sync_channel(capacity);
        // At most capacity queued requests, one in flight and one initial load.
        let (results, rx) = mpsc::sync_channel(capacity + 2);
        let interrupt = Arc::new(AtomicBool::new(false));
        let stop = Arc::new(AtomicBool::new(false));
        let worker_interrupt = interrupt.clone();
        let worker_stop = stop.clone();
        let thread = std::thread::Builder::new().name("viu-mix".into()).spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_time().build().unwrap();
            let backend = backend.map(|make| make());
            let mut current: Option<(u64, Evaluator, Rc<Local>, SharedBuf, SharedBuf)> = None;
            while let Ok(request) = commands.recv() {
                worker_interrupt.store(false, Ordering::Release);
                if worker_stop.load(Ordering::Acquire) { break; }
                let start = Instant::now();
                let (generation, sequence, result) = match request {
                    Request::Load(generation, program) => {
                        if let Some((_, mut old, ..)) = current.take() { old.close_native_events(); }
                        // A fresh evaluator never retains old closures or module state.
                        let local = Rc::new(Local { generation, model: RefCell::new(None), error: RefCell::new(None), backend: backend.clone() });
                        let stdout = SharedBuf::new();
                        let stderr = SharedBuf::new();
                        let mut eval = Evaluator::with_output(Box::new(stdout.clone()), Box::new(stderr.clone()));
                        eval.set_interrupt_flag(worker_interrupt.clone());
                        eval.set_limits(mix::EvalLimits { time_limit: Some(Duration::from_secs(20)), max_string_len: Some(1_048_576), max_list_len: Some(65_536), max_map_len: Some(65_536), ..Default::default() });
                        eval.set_entry_script(&program.path.to_string_lossy());
                        eval.set_file(program.path.to_string_lossy().to_string());
                        eval.set_bus_handler(local.clone());
                        eval.set_serve_runtime(local.clone());
                        eval.register("viu_context", mix::sync_ext(move |_| from_json(json!({
                            "name": program.scene, "generation": generation, "citizen": program.citizen,
                            "loader": "viu-local", "host": "viu-local"
                        }))));
                        let outcome = rt.block_on(bounded(&worker_stop, eval.execute_script_source(&program.source)));
                        current = Some((generation, eval, local, stdout, stderr));
                        (generation, 0, outcome.map(|_| ()))
                    }
                    Request::Event(generation, sequence, command, args, is_request) => {
                        let outcome = match current.as_mut() {
                            Some((active, eval, ..)) if *active == generation => {
                                let event = IncomingEvent { generation, command, headers: BTreeMap::from([
                                    ("from".into(), "viu-client".into()),
                                    ("type".into(), if is_request { "request" } else { "event" }.into()),
                                    ("id".into(), sequence.to_string()),
                                ]), body: args.to_string() };
                                let tasks = tokio::task::LocalSet::new();
                                rt.block_on(bounded(&worker_stop, async {
                                    tasks.run_until(eval.dispatch_event(event)).await?;
                                    tasks.await;
                                    Ok(())
                                }))
                            }
                            Some(_) => Err(RunFailure::Evaluation("stale_generation: superseded behaviour".into())),
                            None => Err(RunFailure::Evaluation("behaviour_unavailable: load failed or callback deadline expired; reload required".into())),
                        };
                        (generation, sequence, outcome)
                    }
                };
                if worker_stop.load(Ordering::Acquire) { break; }
                let (mut model, stdout, stderr, fault) = match current.as_ref() {
                    Some((_, _, local, out, err)) => (local.model.borrow_mut().take(), String::from_utf8_lossy(&out.take()).into_owned(), String::from_utf8_lossy(&err.take()).into_owned(), local.error.borrow_mut().take()),
                    None => (None, String::new(), String::new(), None),
                };
                let result = result.and_then(|_| fault.map_or(Ok(()), |error| Err(RunFailure::Evaluation(error))));
                // Failed callbacks never publish a partial model to the UI.
                if result.is_err() { model = None; }
                if (sequence == 0 && result.is_err()) || matches!(&result, Err(RunFailure::Deadline)) {
                    if let Some((_, mut old, ..)) = current.take() { old.close_native_events(); }
                }
                let completion = Completion { generation, sequence, model, error: result.err().map(RunFailure::message), elapsed: start.elapsed(), stdout, stderr, behaviour_available: current.is_some() };
                if results.send(completion).is_err() { break; }
                wake();
            }
            if let Some((_, mut eval, ..)) = current { eval.close_native_events(); }
        }).expect("start Mix worker");
        Self {
            tx: Some(tx),
            rx: Some(rx),
            thread: Some(thread),
            interrupt,
            stop,
            generation: 0,
            sequence: 0,
        }
    }

    /// Fences visible results immediately after a successful enqueue. A refused
    /// reload leaves the old generation usable. The old callback can finish but
    /// its completion cannot be applied to the new scene.
    pub fn load(&mut self, program: Program) -> Result<u64, Refusal> {
        let next = self.generation.checked_add(1).expect("generation overflow");
        self.enqueue(Request::Load(next, program))?;
        self.generation = next;
        Ok(next)
    }

    pub fn event(&mut self, command: String, args: Json) -> Result<u64, Refusal> {
        self.submit_event(command, args, true)
    }

    /// Native Bus topic delivery has no reply obligation. It uses the same
    /// ordered mailbox and generation fences as UI requests.
    pub fn notify(&mut self, command: String, args: Json) -> Result<u64, Refusal> {
        self.submit_event(command, args, false)
    }

    fn submit_event(
        &mut self,
        command: String,
        args: Json,
        is_request: bool,
    ) -> Result<u64, Refusal> {
        let sequence = self.sequence.checked_add(1).expect("sequence overflow");
        self.enqueue(Request::Event(
            self.generation,
            sequence,
            command,
            args,
            is_request,
        ))?;
        self.sequence = sequence;
        Ok(sequence)
    }

    fn enqueue(&self, request: Request) -> Result<(), Refusal> {
        self.tx
            .as_ref()
            .ok_or(Refusal::Closed)?
            .try_send(request)
            .map_err(|e| match e {
                mpsc::TrySendError::Full(_) => Refusal::Full,
                mpsc::TrySendError::Disconnected(_) => Refusal::Closed,
            })
    }

    pub fn generation(&self) -> u64 {
        self.generation
    }
    pub fn try_completion(&self) -> Option<Completion> {
        self.rx.as_ref()?.try_recv().ok()
    }
    pub fn accepts(&self, completion: &Completion) -> bool {
        completion.generation == self.generation
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        self.interrupt.store(true, Ordering::Release);
        self.tx.take();
        // Unblock a worker held at the bounded completion mailbox before join.
        self.rx.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

async fn bounded<T>(
    stop: &AtomicBool,
    future: impl std::future::Future<Output = MixResult<T>>,
) -> Result<T, RunFailure> {
    tokio::pin!(future);
    // Event-driven callback execution with a deadline; this timer belongs only
    // to active behaviour, and causes no idle client wakes.
    tokio::select! {
        result = &mut future => result.map_err(|e| RunFailure::Evaluation(e.to_string())),
        _ = async { loop { tokio::time::sleep(Duration::from_millis(10)).await; if stop.load(Ordering::Acquire) { break; } } } => Err(RunFailure::OwnerDropped),
        _ = tokio::time::sleep(Duration::from_secs(20)) => Err(RunFailure::Deadline),
    }
}

pub fn from_json(value: Json) -> MixResult<Value> {
    Ok(mix::json::json_to_mix(value))
}
pub fn to_json(value: &Value) -> MixResult<Json> {
    // JSON's null is distinct from the strict-data nil spelling.
    mix::json::mix_to_json(value).map_err(|e| failure(&e))
}

struct Local {
    generation: u64,
    model: RefCell<Option<Json>>,
    error: RefCell<Option<String>>,
    backend: Option<Rc<dyn BusHandler>>,
}

impl ServeRuntime for Local {
    fn handle_reserved(
        &self,
        _: &str,
        _: Option<&str>,
        _: &str,
        _: &[(&str, Option<&str>)],
        _: bool,
    ) -> Option<ReservedOutcome> {
        None
    }
    fn record_handler_fault(&self, summary: &str) {
        *self.error.borrow_mut() = Some(summary.into());
    }
}

fn failure(message: &str) -> MixError {
    MixError::RuntimeError {
        msg: message.into(),
        span: None,
    }
}

impl BusHandler for Local {
    fn send<'a>(
        &'a self,
        target: &'a str,
        command: &'a str,
        args: &'a Value,
    ) -> BusFuture<'a, MixResult<(i32, Value)>> {
        Box::pin(async move {
            if target != "viu-local" {
                return match &self.backend {
                    Some(backend) => backend.send(target, command, args).await,
                    None => Err(failure("no native Bus backend configured")),
                };
            }
            // The native handler contract already carries typed Mix values.
            // Encode only when crossing a wire; retain legacy JSON-string bodies.
            let payload = match args {
                Value::Map(map) => map.get("body").unwrap_or(args),
                _ => args,
            };
            let mut body: Json = match payload {
                Value::String(s) => serde_json::from_str(s).map_err(|_| failure("invalid body"))?,
                value => to_json(value)?,
            };
            let result = match command {
                "shell.scene.describe" => {
                    json!([{"path":"flow", "enum":["horizontal","vertical"]}, {"path":"hidden","type":"bool"}])
                }
                "scenes.ready" | "scenes.model" => {
                    if body["generation"].as_u64() != Some(self.generation) {
                        return Ok((10, from_json(json!({"error_code":"stale_generation"}))?));
                    }
                    if command == "scenes.model" {
                        if !body["value"].is_object() {
                            return Err(failure("model must be a map"));
                        }
                        *self.model.borrow_mut() =
                            body.as_object_mut().and_then(|m| m.remove("value"));
                    }
                    json!({"ok":true})
                }
                _ => return Err(failure("unsupported local scene command")),
            };
            Ok((0, from_json(result)?))
        })
    }
    fn emit<'a>(
        &'a self,
        target: &'a str,
        command: &'a str,
        args: &'a Value,
    ) -> BusFuture<'a, MixResult<()>> {
        Box::pin(async move {
            match &self.backend {
                Some(b) => b.emit(target, command, args).await,
                None => Err(failure("no native event backend configured")),
            }
        })
    }
    fn port_exists<'a>(&'a self, target: &'a str) -> BusFuture<'a, MixResult<bool>> {
        Box::pin(async move {
            if target == "viu-local" {
                Ok(true)
            } else {
                match &self.backend {
                    Some(b) => b.port_exists(target).await,
                    None => Ok(false),
                }
            }
        })
    }
    fn next_incoming<'a>(&'a self) -> BusFuture<'a, Option<IncomingEvent>> {
        Box::pin(async { None })
    }
    fn reply<'a>(
        &'a self,
        _: &'a str,
        _: &'a str,
        _: Option<&'a str>,
        rc: u8,
        body: &'a str,
    ) -> BusFuture<'a, MixResult<()>> {
        Box::pin(async move {
            if rc != 0 && self.error.borrow().is_none() {
                *self.error.borrow_mut() = Some(body.into());
            }
            Ok(())
        })
    }
}
