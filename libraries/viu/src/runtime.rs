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
pub type BackendMaker = Arc<dyn Fn() -> BackendFactory + Send + Sync>;

/// Caller-owned native facilities shared by its root and job evaluators.
/// Supplying one does not create a Bus citizen, broker or global registry.
#[derive(Clone, Default)]
pub struct Services {
    pub max_string_bytes: Option<usize>,
    pub platform: Option<crate::platform::Platform>,
    pub ui_state: Option<crate::ui_state::UiState>,
    pub calls: Option<crate::calls::Calls>,
    pub timers: Option<crate::timers::Timers>,
    pub documents: Option<crate::documents::Documents>,
    pub macros: Option<macros::Session>,
    pub chooser: Option<crate::chooser::Service>,
}

// Match the native Mix CLI's explicit evaluation-thread stack. Async evaluator
// frames in debug can overflow the Rust default before Mix's depth guard runs.
const EVALUATOR_STACK: usize = 64 * 1024 * 1024;
struct WorkerPolicy;
impl mix::evaluator::CapabilityPolicy for WorkerPolicy {
    fn check_builtin(&self, name: &str) -> Result<(), String> {
        if ["chdir", "exit"].contains(&name) {
            Err(format!(
                "{name} mutates the embedded host; use explicit process cwd/lifetime"
            ))
        } else {
            Ok(())
        }
    }
    fn check_class(&self, class: mix::builtins::CapabilityClass) -> Result<(), String> {
        if class == mix::builtins::CapabilityClass::Process {
            Err("embedded workers refuse export and shell syntax; use native argv functions with explicit options".into())
        } else {
            Ok(())
        }
    }
}

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
pub struct Reply {
    pub rc: u8,
    pub body: Json,
}
#[derive(Debug)]
pub struct DeferredAnswer {
    pub token: String,
    pub reply: Reply,
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
    /// The callback's explicit application reply. A refusal is a normal
    /// response, distinct from an evaluator/handler failure in `error`.
    pub reply: Option<Reply>,
    pub deferred: Option<String>,
    pub answers: Vec<DeferredAnswer>,
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
    Event(u64, u64, String, Json, bool, BTreeMap<String, String>),
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
        Self::with_services(capacity, wake, backend, Services::default())
    }
    pub fn with_services(
        capacity: usize,
        wake: Wake,
        backend: Option<BackendFactory>,
        services: Services,
    ) -> Self {
        Self::with_deadline(capacity, wake, backend, services, Duration::from_secs(20))
    }
    /// Dedicated work lanes may wait on a human or native IO. Root UI callbacks
    /// retain their existing twenty-second ceiling.
    pub(crate) fn with_deadline(
        capacity: usize,
        wake: Wake,
        backend: Option<BackendFactory>,
        services: Services,
        deadline: Duration,
    ) -> Self {
        assert!(capacity > 0);
        let (tx, commands) = mpsc::sync_channel(capacity);
        // At most capacity queued requests, one in flight and one initial load.
        let (results, rx) = mpsc::sync_channel(capacity + 2);
        let interrupt = Arc::new(AtomicBool::new(false));
        let stop = Arc::new(AtomicBool::new(false));
        let worker_interrupt = interrupt.clone();
        let worker_stop = stop.clone();
        let thread = std::thread::Builder::new().name("viu-mix".into()).stack_size(EVALUATOR_STACK).spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
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
                        let local = Rc::new(Local { generation, model: RefCell::new(None), error: RefCell::new(None), reply: RefCell::new(None), deferred: RefCell::new(None), answers: RefCell::new(Vec::new()), is_request: std::cell::Cell::new(false), backend: backend.clone() });
                        let stdout = SharedBuf::new();
                        let stderr = SharedBuf::new();
                        let mut eval = Evaluator::with_output(Box::new(stdout.clone()), Box::new(stderr.clone()));
                        eval.set_capability_policy(Rc::new(WorkerPolicy));
                        eval.set_interrupt_flag(worker_interrupt.clone());
                        eval.set_limits(mix::EvalLimits { time_limit: Some(deadline), max_string_len: Some(services.max_string_bytes.unwrap_or(1_048_576).clamp(1_048_576,32*1024*1024)), max_list_len: Some(65_536), max_map_len: Some(65_536), ..Default::default() });
                        eval.set_entry_script(&program.path.to_string_lossy());
                        eval.set_file(program.path.to_string_lossy().to_string());
                        eval.set_bus_handler(local.clone());
                        eval.set_serve_runtime(local.clone());
                        eval.register("viu_thumbnail",crate::native_primitives::thumbnail(worker_stop.clone()));
                        eval.register("viu_atomic_replace",crate::native_primitives::atomic_replace());
                        eval.register("viu_read_optional",crate::native_primitives::read_optional());
                        eval.register("viu_app_dirs",crate::native_primitives::app_dirs());
                        eval.register("viu_maildoc",crate::native_primitives::mail_document());
                        eval.register("viu_editor_write",crate::native_primitives::editor_write());
                        eval.register("viu_digest",crate::native_primitives::digest());
                        eval.register("viu_action_event",crate::native_primitives::action_event());
                        eval.register("viu_files",crate::file_data::binding());
                        eval.register("viu_process",crate::process::binding());
                        eval.register("viu_widget_admit",crate::native_primitives::widget_admit());
                        eval.register("viu_settings",crate::native_primitives::settings_values());
                        eval.register("viu_token",crate::native_primitives::token());
                        eval.register("viu_fonts",crate::native_primitives::installed_fonts());
                        eval.register("viu_image",crate::native_primitives::image_file());
                        eval.register("viu_http",crate::http::binding());
                        if let Some(session)=services.macros.clone(){eval.register("viu_macros",crate::native_primitives::macro_commands(session));}
                        if let Some(chooser)=services.chooser.clone(){eval.register("viu_chooser",chooser.binding());}
                        if let Some(documents)=services.documents.clone(){eval.register("viu_document",documents.binding());}
                        if let Some(timers)=services.timers.clone(){eval.register("viu_timer",timers.binding());}
                        if let Some(calls)=services.calls.clone(){eval.register("viu_reply",calls.binding());}
                        if let Some(state)=services.ui_state.clone(){eval.register("viu_ui_state",state.binding());}
                        if let Some(platform)=services.platform.clone(){eval.register("viu_platform",platform.binding());}
                        eval.register("viu_context", mix::sync_ext(move |_| from_json(json!({
                            "name": program.scene, "generation": generation, "citizen": program.citizen,
                            "version":env!("CARGO_PKG_VERSION"),"git_sha":option_env!("GIT_SHA").unwrap_or("unknown"),"build_time":option_env!("BUILD_TIME").unwrap_or("unknown"),
                            "pid":std::process::id(), "version":env!("CARGO_PKG_VERSION"),
                            "loader": "viu-local", "host": "viu-local"
                        }))));
                        let outcome = rt.block_on(bounded(&worker_stop, deadline, eval.execute_script_source(&program.source)));
                        current = Some((generation, eval, local, stdout, stderr));
                        (generation, 0, outcome.map(|_| ()))
                    }
                    Request::Event(generation, sequence, command, args, is_request, mut headers) => {
                        let outcome = match current.as_mut() {
                            Some((active, eval, local, ..)) if *active == generation => {
                                local.is_request.set(is_request);
                                headers.entry("from".into()).or_insert_with(||"viu-client".into());
                                headers.insert("type".into(),if is_request{"request"}else{"event"}.into());
                                headers.insert("id".into(),sequence.to_string());
                                let event = IncomingEvent { generation, command, headers, body: args.to_string() };
                                let tasks = tokio::task::LocalSet::new();
                                rt.block_on(bounded(&worker_stop, deadline, async {
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
                let (mut model, stdout, stderr, fault, mut reply) = match current.as_ref() {
                    Some((_, _, local, out, err)) => (local.model.borrow_mut().take(), String::from_utf8_lossy(&out.take()).into_owned(), String::from_utf8_lossy(&err.take()).into_owned(), local.error.borrow_mut().take(),local.reply.borrow_mut().take()),
                    None => (None, String::new(), String::new(), None,None),
                };
                let result = result.and_then(|_| fault.map_or(Ok(()), |error| Err(RunFailure::Evaluation(error))));
                let (mut deferred, mut answers) = current.as_ref().map(|(_,_,local,_,_)| (local.deferred.borrow_mut().take(), std::mem::take(&mut *local.answers.borrow_mut()))).unwrap_or_default();
                let result = if deferred.is_some() && reply.is_some() { Err(RunFailure::Evaluation("request cannot both reply and defer".into())) } else { result };
                // Failed callbacks never publish a partial model to the UI.
                if result.is_err() { model = None; reply=None; deferred=None; answers.clear(); }
                if (sequence == 0 && result.is_err()) || matches!(&result, Err(RunFailure::Deadline)) {
                    if let Some((_, mut old, ..)) = current.take() { old.close_native_events(); }
                }
                let completion = Completion { generation, sequence, model, error: result.err().map(RunFailure::message), elapsed: start.elapsed(), stdout, stderr, behaviour_available: current.is_some(),reply,deferred,answers };
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
        self.submit_event(command, args, true, BTreeMap::new())
    }
    /// Preserve broker-stamped caller and topic metadata. Request correlation
    /// remains local to the mailbox; the native owner retains the wire reply.
    pub fn event_with_headers(
        &mut self,
        command: String,
        args: Json,
        headers: BTreeMap<String, String>,
    ) -> Result<u64, Refusal> {
        self.submit_event(command, args, true, headers)
    }

    /// Native Bus topic delivery has no reply obligation. It uses the same
    /// ordered mailbox and generation fences as UI requests.
    pub fn notify(&mut self, command: String, args: Json) -> Result<u64, Refusal> {
        self.submit_event(command, args, false, BTreeMap::new())
    }
    pub fn notify_with_headers(
        &mut self,
        command: String,
        args: Json,
        headers: BTreeMap<String, String>,
    ) -> Result<u64, Refusal> {
        self.submit_event(command, args, false, headers)
    }

    fn submit_event(
        &mut self,
        command: String,
        args: Json,
        is_request: bool,
        headers: BTreeMap<String, String>,
    ) -> Result<u64, Refusal> {
        let sequence = self.sequence.checked_add(1).expect("sequence overflow");
        self.enqueue(Request::Event(
            self.generation,
            sequence,
            command,
            args,
            is_request,
            headers,
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
    pub fn drain(&self) -> Vec<Completion> {
        let mut results = Vec::new();
        while let Some(result) = self.try_completion() {
            results.push(result);
        }
        results
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
    deadline: Duration,
    future: impl std::future::Future<Output = MixResult<T>>,
) -> Result<T, RunFailure> {
    tokio::pin!(future);
    // Event-driven callback execution with a deadline; this timer belongs only
    // to active behaviour, and causes no idle client wakes.
    tokio::select! {
        result = &mut future => result.map_err(|e| RunFailure::Evaluation(e.to_string())),
        _ = async { loop { tokio::time::sleep(Duration::from_millis(10)).await; if stop.load(Ordering::Acquire) { break; } } } => Err(RunFailure::OwnerDropped),
        _ = tokio::time::sleep(deadline) => Err(RunFailure::Deadline),
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
    reply: RefCell<Option<Reply>>,
    deferred: RefCell<Option<String>>,
    answers: RefCell<Vec<DeferredAnswer>>,
    is_request: std::cell::Cell<bool>,
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
    fn send_with_reply<'a>(
        &'a self,
        target: &'a str,
        command: &'a str,
        args: &'a Value,
    ) -> BusFuture<'a, MixResult<(i32, Value, Value)>> {
        Box::pin(async move {
            if target != "viu-local"
                || command.starts_with("native.")
                || ["jobs.start", "jobs.cancel"].contains(&command)
            {
                return match &self.backend {
                    Some(backend) => backend.send_with_reply(target, command, args).await,
                    None => Err(failure("no native Bus backend configured")),
                };
            }
            let (rc, result) = self.send(target, command, args).await?;
            Ok((rc, result.clone(), result))
        })
    }
    fn send<'a>(
        &'a self,
        target: &'a str,
        command: &'a str,
        args: &'a Value,
    ) -> BusFuture<'a, MixResult<(i32, Value)>> {
        Box::pin(async move {
            if target != "viu-local"
                || command.starts_with("native.")
                || ["jobs.start", "jobs.cancel"].contains(&command)
            {
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
                "requests.defer" => {
                    let token = body["token"]
                        .as_str()
                        .filter(|s| !s.is_empty() && s.len() <= 128)
                        .ok_or_else(|| failure("invalid deferred request token"))?;
                    if !self.is_request.get() || self.deferred.borrow().is_some() {
                        return Err(failure(
                            "only one deferral is allowed on a request callback",
                        ));
                    }
                    *self.deferred.borrow_mut() = Some(token.into());
                    json!({"deferred":true})
                }
                "requests.reply" => {
                    let token = body["token"]
                        .as_str()
                        .filter(|s| !s.is_empty() && s.len() <= 128)
                        .ok_or_else(|| failure("invalid deferred request token"))?;
                    let rc = body["rc"]
                        .as_u64()
                        .filter(|rc| *rc <= 255)
                        .ok_or_else(|| failure("invalid deferred reply code"))?
                        as u8;
                    if self.answers.borrow().len() >= 32
                        || self.answers.borrow().iter().any(|a| a.token == token)
                    {
                        return Err(failure("duplicate or over-budget deferred reply"));
                    }
                    self.answers.borrow_mut().push(DeferredAnswer {
                        token: token.into(),
                        reply: Reply {
                            rc,
                            body: body["value"].clone(),
                        },
                    });
                    json!({"queued":true})
                }
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
    fn subscribe_topic<'a>(&'a self, topic: &'a str) -> BusFuture<'a, MixResult<()>> {
        Box::pin(async move {
            match &self.backend {
                Some(backend) => backend.subscribe_topic(topic).await,
                None => Err(failure("no native Bus backend configured")),
            }
        })
    }
    fn unsubscribe_topic<'a>(&'a self, topic: &'a str) -> BusFuture<'a, MixResult<()>> {
        Box::pin(async move {
            match &self.backend {
                Some(backend) => backend.unsubscribe_topic(topic).await,
                None => Err(failure("no native Bus backend configured")),
            }
        })
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
            // Mix synthesises NO_REPLY when a clean handler returns without
            // reply(). The host now owns that obligation after local deferral.
            // Explicit replies and fault replies still follow normal admission.
            if self.deferred.borrow().is_some()
                && rc == mix::evaluator::NO_REPLY_RC
                && body == mix::evaluator::NO_REPLY_BODY
            {
                return Ok(());
            }
            if self.reply.borrow().is_some() {
                return Err(failure("callback already replied"));
            }
            let body = serde_json::from_str(body).unwrap_or_else(|_| Json::String(body.into()));
            *self.reply.borrow_mut() = Some(Reply { rc, body });
            Ok(())
        })
    }
}
