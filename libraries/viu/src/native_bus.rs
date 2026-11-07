// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded in-process adaptation of the existing native supervised Bus client.
//! Applications supply their command descriptors and own all command policy.
use ::bus::{CommandDescriptor, SupervisedClient, native_client::BoundedIncomingEvent};
use mix::{BusHandler, MixError, MixResult, evaluator::BusFuture, value::Value as MixValue};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    rc::Rc,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU8, AtomicU64, Ordering},
        mpsc,
    },
    thread::JoinHandle,
    time::Duration,
};
use tokio::sync::oneshot;

pub struct Request {
    pub from: String,
    pub command: String,
    pub args: Value,
    pub headers: BTreeMap<String, String>,
    pub reply: oneshot::Sender<(u8, Value)>,
    /// An application can request close for this reply. The owner acts only
    /// after the successful response has actually crossed the native transport.
    pub close_after_reply: Arc<AtomicBool>,
    /// 0 pending, 1 reply crossed the transport, 2 delivery failed/cancelled.
    pub delivery: Arc<AtomicU8>,
    /// A Mix owner may retain this request beyond its callback/admission timeout.
    pub deferred: Arc<AtomicBool>,
}
pub struct Notice {
    pub command: String,
    pub args: Value,
    pub headers: BTreeMap<String, String>,
}
type SharedClient = Arc<Mutex<Option<Arc<SupervisedClient>>>>;
struct Hint {
    topic: String,
    body: Value,
    probe: Option<String>,
}
type HintFuture = std::pin::Pin<Box<dyn std::future::Future<Output = (String, bool, bool)> + Send>>;
/// Launch-time activation uses the existing native SDK and compositor path.
pub fn activate_existing(service: &str, command: &str, url: &str) -> Result<bool, String> {
    activate_existing_plan(service, &[json!({"command":command,"body":{}})], url)
}
pub fn activate_existing_plan(
    service: &str,
    operations: &[Value],
    url: &str,
) -> Result<bool, String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| e.to_string())?;
    runtime.block_on(async {
        let probe = format!("viu-launch-{}", std::process::id());
        let client = match tokio::time::timeout(
            Duration::from_secs(2),
            SupervisedClient::connect_options(&probe, url).connect(),
        )
        .await
        {
            Ok(Ok(client)) => client,
            _ => return Ok(false),
        };
        let result = async {
            let services = client.list_services().await.map_err(|e| e.to_string())?;
            if !services.iter().any(|s| s == service) {
                return Ok(false);
            }
            for operation in operations {
                let (rc, body, error) = client
                    .call_with_headers_raw_within(
                        service,
                        operation["command"]
                            .as_str()
                            .ok_or("activation command required")?,
                        &BTreeMap::new(),
                        &operation["body"].to_string(),
                        Duration::from_secs(10),
                    )
                    .await
                    .map_err(|e| e.to_string())?;
                if rc != 0 {
                    return Err(error.unwrap_or(body));
                }
            }
            Ok(true)
        }
        .await;
        client.close().await;
        result
    })
}

async fn send_hint(native: Arc<SupervisedClient>, hint: Hint) -> (String, bool, bool) {
    let mut frame = ::bus::BusMessage::new();
    frame.set("command", &hint.topic);
    frame.body = hint.body.to_string();
    let headers = BTreeMap::from([
        ("name".into(), hint.topic.clone()),
        ("retain".into(), "false".into()),
    ]);
    let wire = frame.to_wire();
    let result = native
        .call_with_headers_raw_within(
            "noded",
            "topic.publish",
            &headers,
            &wire,
            Duration::from_secs(2),
        )
        .await;
    // The existing host's ping wakes its window/decoration registry after an
    // offered bar. Without this, a window mapped after the registry pass can
    // remain absent from the host's initial titlebar subscription set.
    let probe = if let Some(service) = hint.probe {
        matches!(
            native
                .call_with_headers_raw_within(
                    &service,
                    &format!("{service}.ping"),
                    &BTreeMap::new(),
                    "{}",
                    Duration::from_secs(2)
                )
                .await,
            Ok((0, _, _))
        )
    } else {
        true
    };
    (hint.topic, matches!(result, Ok((0, _, _))), probe)
}

pub struct Endpoint {
    name: String,
    pub registration_failed: Arc<AtomicBool>,
    pub requests: mpsc::Receiver<Request>,
    pub notices: mpsc::Receiver<Notice>,
    pub close_requested: Arc<AtomicBool>,
    pub startup_error: Arc<Mutex<Option<String>>>,
    pub dropped_notices: Arc<AtomicU64>,
    client: SharedClient,
    hints: tokio::sync::mpsc::Sender<Hint>,
    stop: Option<oneshot::Sender<()>>,
    thread: Option<JoinHandle<()>>,
}
/// Bundle-authored compatibility for native HELP and ordinary reply deadlines.
pub struct RequestPolicy {
    pub help: Option<Value>,
    pub owner_help: bool,
    pub timeout: Duration,
}
impl Default for RequestPolicy {
    fn default() -> Self {
        Self {
            help: None,
            owner_help: false,
            timeout: Duration::from_secs(180),
        }
    }
}
impl Endpoint {
    pub fn offline() -> Self {
        let (_, requests) = mpsc::sync_channel(1);
        let (_, notices) = mpsc::sync_channel(1);
        let (hints, _) = tokio::sync::mpsc::channel(1);
        Self {
            name: String::new(),
            startup_error: Arc::new(Mutex::new(None)),
            registration_failed: Arc::new(AtomicBool::new(false)),
            requests,
            notices,
            close_requested: Arc::new(AtomicBool::new(false)),
            dropped_notices: Arc::new(AtomicU64::new(0)),
            client: Arc::new(Mutex::new(None)),
            hints,
            stop: None,
            thread: None,
        }
    }
    pub fn start(
        name: String,
        url: String,
        commands: Vec<CommandDescriptor>,
        quit_command: String,
        wake: crate::runtime::Wake,
    ) -> Self {
        Self::start_described(name, url, commands, quit_command, wake, None, false)
    }
    pub fn start_described(
        name: String,
        url: String,
        commands: Vec<CommandDescriptor>,
        quit_command: String,
        wake: crate::runtime::Wake,
        help: Option<Value>,
        owner_help: bool,
    ) -> Self {
        Self::start_policy(
            name,
            url,
            commands,
            quit_command,
            wake,
            RequestPolicy {
                help,
                owner_help,
                ..Default::default()
            },
        )
    }
    pub fn start_policy(
        name: String,
        url: String,
        commands: Vec<CommandDescriptor>,
        quit_command: String,
        wake: crate::runtime::Wake,
        policy: RequestPolicy,
    ) -> Self {
        let RequestPolicy {
            help,
            owner_help,
            timeout,
        } = policy;
        let (tx, requests) = mpsc::sync_channel(32);
        let (notice_tx, notices) = mpsc::sync_channel(32);
        let close_requested = Arc::new(AtomicBool::new(false));
        let registration_failed = Arc::new(AtomicBool::new(false));
        let failed = registration_failed.clone();
        let close = close_requested.clone();
        let startup_error = Arc::new(Mutex::new(None));
        let startup_failed = startup_error.clone();
        let dropped_notices = Arc::new(AtomicU64::new(0));
        let dropped = dropped_notices.clone();
        let client: SharedClient = Arc::new(Mutex::new(None));
        let shared = client.clone();
        let (stop, mut stopped) = oneshot::channel();
        let (hints, mut hint_rx) = tokio::sync::mpsc::channel::<Hint>(1);
        let service_name = name.clone();
        let thread=std::thread::Builder::new().name("viu-native-bus".into()).spawn(move||{
            let rt=tokio::runtime::Builder::new_current_thread().enable_all().build().expect("native Bus runtime");
            rt.block_on(async move{
                let options=SupervisedClient::connect_options(&name,&url).bounded_incoming_rpc(32).fatal_on_registration_rejection(true);
                let options=if help.is_none() && !owner_help{options.with_commands(commands)}else{options};
                let native=tokio::select!{_=&mut stopped=>return,c=options.connect()=>match c{Ok(c)=>Arc::new(c),Err(e)=>{eprintln!("native Bus: {e}");failed.store(true,Ordering::Release);*startup_failed.lock().unwrap_or_else(|e|e.into_inner())=Some(e.to_string());close.store(true,Ordering::Release);wake();return;}}};
                let Some(mut incoming)=native.incoming_bounded()else{return;};
                *shared.lock().unwrap_or_else(|e|e.into_inner())=Some(native.clone());
                let mut state=native.subscribe_state();
                let post=|notice:Notice|{if notice_tx.try_send(notice).is_err(){dropped.fetch_add(1,Ordering::Relaxed);}wake();};
                post(Notice{command:"bus.connected".into(),args:json!({}),headers:BTreeMap::new()});
                let mut responses=tokio::task::JoinSet::new();
                let mut publishing:Option<HintFuture>=None;
                loop{
                    let event=tokio::select!{
                        _=&mut stopped=>break,
                        _=responses.join_next(),if !responses.is_empty()=>continue,
                        hint=hint_rx.recv(),if publishing.is_none()=>{if let Some(hint)=hint {publishing=Some(Box::pin(send_hint(native.clone(),hint)));}continue;},
                        (topic,ok,probe_ok)=async{publishing.as_mut().expect("guarded hint future").await},if publishing.is_some()=>{publishing=None;post(Notice{command:"bus.hint.completed".into(),args:json!({"topic":topic,"ok":ok,"probe_ok":probe_ok}),headers:BTreeMap::new()});continue;},
                        changed=state.changed()=>{if changed.is_err(){break;}let connected=native.is_connected();if !connected {responses.abort_all();}post(Notice{command:if connected{"bus.connected"}else{"bus.disconnected"}.into(),args:json!({"generation":native.connection_generation()}),headers:BTreeMap::new()});continue;},
                        event=incoming.recv()=>match event{Some(event)=>event,None=>break}
                    };
                    let command=match event{
                        BoundedIncomingEvent::Overflow{dropped:count}=>{dropped.fetch_add(count,Ordering::Relaxed);post(Notice{command:"bus.overflow".into(),args:json!({"dropped":count}),headers:BTreeMap::new()});continue;},
                        BoundedIncomingEvent::Command(command)=>command
                    };
                    if command.id.is_none(){let mut headers=command.headers.clone();headers.insert("type".into(),"event".into());headers.insert("from".into(),command.from.clone());post(Notice{command:command.topic().unwrap_or(&command.command).to_owned(),args:command.args,headers});continue;}
                    toolkit::debug::received(&name,&command.command);
                    if command.command=="HELP" && command.args.is_object() {if let Some(help)=&help {let _=native.respond(&command,0,&help.to_string()).await;continue;}}
                    if close.load(Ordering::Acquire){let _=native.respond(&command,10,&json!({"error_code":"CLOSING"}).to_string()).await;continue;}
                    if responses.len()>=32{let _=native.respond(&command,10,&json!({"error_code":"BUSY"}).to_string()).await;continue;}
                    let (reply,response)=oneshot::channel();
                    let close_after_reply=Arc::new(AtomicBool::new(false));
                    let delivery=Arc::new(AtomicU8::new(0));
                    let deferred=Arc::new(AtomicBool::new(false));
                    let request=Request{from:command.from.clone(),command:command.command.clone(),args:if command.body.is_empty(){json!({})}else{command.args.clone()},headers:command.headers.clone(),reply,close_after_reply:close_after_reply.clone(),delivery:delivery.clone(),deferred:deferred.clone()};
                    let admitted=tx.try_send(request).is_ok();
                    if admitted{wake();}
                    let native=native.clone();let wake=wake.clone();let close=close.clone();let quit=command.command==quit_command;let service=name.clone();
                    responses.spawn(async move{
                        let ack=Delivery{state:delivery,wake:wake.clone()};
                        let mut response=response;
                        let answer=if admitted{match tokio::time::timeout(timeout,&mut response).await{Ok(Ok(value))=>value,Err(_) if deferred.load(Ordering::Acquire)=>response.await.unwrap_or((20,json!({"error_code":"OWNER_CLOSED"}))),_=>(20,json!({"error_code":"OWNER_TIMEOUT"}))}}else{(10,json!({"error_code":"BUSY"}))};
                        toolkit::debug::answered(&service,&command.command,answer.0);
                        let sent=matches!(tokio::time::timeout(Duration::from_secs(2),native.respond(&command,answer.0,&answer.1.to_string())).await,Ok(Ok(())));
                        ack.state.store(if sent{1}else{2},Ordering::Release);wake();
                        if (quit||close_after_reply.load(Ordering::Acquire))&&answer.0==0&&sent{close.store(true,Ordering::Release);wake();}
                    });
                }
                responses.abort_all();while responses.join_next().await.is_some(){}
                drop(publishing);
                *shared.lock().unwrap_or_else(|e|e.into_inner())=None;
                native.close().await;
            });
        }).expect("start native Bus owner");
        Self {
            name: service_name,
            requests,
            notices,
            close_requested,
            registration_failed,
            startup_error,
            dropped_notices,
            client,
            hints,
            stop: Some(stop),
            thread: Some(thread),
        }
    }
    /// Construct on the Mix worker. It shares the existing native connection;
    /// it creates no broker, socket, dispatcher or secondary service identity.
    pub fn backend(&self) -> crate::runtime::BackendFactory {
        let client = self.client.clone();
        Box::new(move || Rc::new(Bridge { client }))
    }
    pub fn backend_maker(&self) -> crate::runtime::BackendMaker {
        let shared = self.client.clone();
        Arc::new(move || {
            let client = shared.clone();
            Box::new(move || Rc::new(Bridge { client }))
        })
    }
    pub fn client(&self) -> Option<Arc<SupervisedClient>> {
        self.client
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
    pub fn service(&self) -> &str {
        &self.name
    }
    /// One bounded topic-publication slot on this same native connection.
    /// The owner coalesces data and retries a refused/failed hint; no transport
    /// or service identity is created. Completion is a native owner notice.
    pub fn hint(&self, topic: String, body: Value) -> Result<(), String> {
        self.queue_hint(topic, body, None)
    }
    /// The same one-slot native hint lane, with a bounded existing-host ping.
    /// At most one future runs; normal requests remain independently serviced.
    pub fn hint_with_probe(
        &self,
        topic: String,
        body: Value,
        service: String,
    ) -> Result<(), String> {
        if service.is_empty()
            || service.len() > 64
            || !service
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        {
            return Err("invalid native probe service".into());
        }
        self.queue_hint(topic, body, Some(service))
    }
    fn queue_hint(&self, topic: String, body: Value, probe: Option<String>) -> Result<(), String> {
        if topic.is_empty() || topic.len() > 256 || body.to_string().len() > 4096 {
            return Err("hint exceeds topic/body budget".into());
        }
        self.hints
            .try_send(Hint { topic, body, probe })
            .map_err(|_| "native hint slot is busy or closed".into())
    }
}
struct Delivery {
    state: Arc<AtomicU8>,
    wake: crate::runtime::Wake,
}
impl Drop for Delivery {
    fn drop(&mut self) {
        let _ = self
            .state
            .compare_exchange(0, 2, Ordering::AcqRel, Ordering::Acquire);
        (self.wake)();
    }
}
impl Drop for Endpoint {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn failure(error: impl ToString) -> MixError {
    MixError::RuntimeError {
        msg: error.to_string(),
        span: None,
    }
}
pub struct Bridge {
    client: SharedClient,
}
impl Bridge {
    fn native(&self) -> MixResult<Arc<SupervisedClient>> {
        self.client
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
            .ok_or_else(|| failure("native Bus not connected"))
    }
    fn payload(
        command: &str,
        args: &MixValue,
    ) -> MixResult<(BTreeMap<String, String>, String, bool)> {
        Self::payload_mode(command, args, false)
    }
    fn payload_mode(
        command: &str,
        args: &MixValue,
        all_headers: bool,
    ) -> MixResult<(BTreeMap<String, String>, String, bool)> {
        if let MixValue::Map(map) = args {
            let route = all_headers
                || map.contains_key("body")
                || (command.contains(".props.") && map.contains_key("namespace"));
            if route
                && map
                    .values()
                    .any(|v| matches!(v, MixValue::Bytes(_) | MixValue::Buffer(_)))
            {
                return Err(failure(
                    "native Bus headers/body do not accept bytes; encode them explicitly",
                ));
            }
        }
        let args = crate::runtime::to_json(args)?;
        let mut headers = BTreeMap::new();
        let header_route = (all_headers && args.is_object())
            || args.get("body").is_some()
            || (command.contains(".props.") && args.get("namespace").is_some());
        let body = if header_route {
            if let Some(map) = args.as_object() {
                for (key, value) in map {
                    if key != "body" && key != "timeout" && key != "reply_mode" && !value.is_null()
                    {
                        headers.insert(
                            key.clone(),
                            value
                                .as_str()
                                .map(str::to_owned)
                                .unwrap_or_else(|| value.to_string()),
                        );
                    }
                }
            }
            match args.get("body") {
                None => String::new(),
                Some(Value::String(body)) => body.clone(),
                Some(body) => body.to_string(),
            }
        } else if args.is_null() {
            String::new()
        } else {
            args.to_string()
        };
        Ok((headers, body, header_route))
    }
    async fn call_parts(
        &self,
        target: &str,
        command: &str,
        args: &MixValue,
    ) -> MixResult<(i32, MixValue, MixValue)> {
        let raw = crate::runtime::to_json(args)?["reply_mode"] == "raw";
        let native = match self.native() {
            Ok(native) => native,
            Err(error) if raw => return raw_transport_parts(error.to_string(), false),
            Err(error) => return Err(error),
        };
        let timeout = crate::runtime::to_json(args)?["timeout"]
            .as_f64()
            .unwrap_or(10.0);
        if !timeout.is_finite() || !(0.1..=86_400.0).contains(&timeout) {
            return Err(failure("native timeout must be 0.1..86400 seconds"));
        }
        let (headers, body, header_route) = Self::payload(command, args)?;
        let received = native
            .call_with_headers_raw_within(
                target,
                command,
                &headers,
                &body,
                Duration::from_secs_f64(timeout),
            )
            .await;
        let (rc, body, error) = match received {
            Ok(reply) => reply,
            Err(error) if raw => {
                let unknown = !matches!(
                    error,
                    ::bus::client::SupervisedError::Disconnected
                        | ::bus::client::SupervisedError::ShuttingDown
                );
                return raw_transport_parts(error.to_string(), unknown);
            }
            Err(error) => return Err(failure(error)),
        };
        if raw {
            return raw_reply_parts(rc, body, error);
        }
        reply_parts(rc, body, error, header_route)
    }
}
impl BusHandler for Bridge {
    fn send<'a>(
        &'a self,
        target: &'a str,
        command: &'a str,
        args: &'a MixValue,
    ) -> BusFuture<'a, MixResult<(i32, MixValue)>> {
        Box::pin(async move {
            let (rc, result, _) = self.call_parts(target, command, args).await?;
            Ok((rc, result))
        })
    }
    fn send_with_reply<'a>(
        &'a self,
        target: &'a str,
        command: &'a str,
        args: &'a MixValue,
    ) -> BusFuture<'a, MixResult<(i32, MixValue, MixValue)>> {
        Box::pin(async move { self.call_parts(target, command, args).await })
    }
    fn emit<'a>(
        &'a self,
        target: &'a str,
        command: &'a str,
        args: &'a MixValue,
    ) -> BusFuture<'a, MixResult<()>> {
        Box::pin(async move {
            let native = self.native()?;
            let (mut headers, body, _) = Self::payload_mode(command, args, true)?;
            headers.insert("type".into(), "event".into());
            native
                .send_with_headers(target, command, &headers, &body)
                .await
                .map_err(failure)
        })
    }
    fn port_exists<'a>(&'a self, target: &'a str) -> BusFuture<'a, MixResult<bool>> {
        Box::pin(async move {
            Ok(self
                .native()?
                .list_services()
                .await
                .map_err(failure)?
                .iter()
                .any(|s| s == target))
        })
    }
    fn next_incoming<'a>(&'a self) -> BusFuture<'a, Option<mix::evaluator::IncomingEvent>> {
        Box::pin(async { None })
    }
    fn subscribe_topic<'a>(&'a self, topic: &'a str) -> BusFuture<'a, MixResult<()>> {
        Box::pin(async move { self.native()?.subscribe_topic(topic).await.map_err(failure) })
    }
    fn unsubscribe_topic<'a>(&'a self, topic: &'a str) -> BusFuture<'a, MixResult<()>> {
        Box::pin(async move {
            self.native()?
                .unsubscribe_topic(topic)
                .await
                .map_err(failure)
        })
    }
}

// Keep Mix's established result/reply distinction and exact peer rc. Header
// routing preserves structured refusals; JSON-body RPC preserves its message
// result. Both retain every parsed field in $reply, including JSON strings.
fn raw_reply_parts(
    rc: u8,
    body: String,
    error: Option<String>,
) -> MixResult<(i32, MixValue, MixValue)> {
    let envelope = crate::runtime::from_json(json!({"rc":rc,"body":body,"error":error}))?;
    Ok((i32::from(rc), envelope.clone(), envelope))
}

fn raw_transport_parts(
    message: String,
    outcome_unknown: bool,
) -> MixResult<(i32, MixValue, MixValue)> {
    let envelope = crate::runtime::from_json(
        json!({"rc":-1,"body":"","error":message,"transport_error":message,"outcome_unknown":outcome_unknown,"retried":false}),
    )?;
    Ok((-1, envelope.clone(), envelope))
}

fn reply_parts(
    rc: u8,
    body: String,
    error: Option<String>,
    header_route: bool,
) -> MixResult<(i32, MixValue, MixValue)> {
    let parsed = serde_json::from_str::<Value>(&body).ok();
    let reply = crate::runtime::from_json(parsed.clone().unwrap_or(Value::Null))?;
    if rc < 10 {
        let result = match parsed {
            Some(value) => crate::runtime::from_json(value)?,
            None if body.is_empty() => MixValue::Nil,
            None => MixValue::String(body),
        };
        return Ok((i32::from(rc), result, reply));
    }
    if header_route
        && parsed
            .as_ref()
            .is_some_and(|v| v.get("error_code").is_some())
    {
        return Ok((i32::from(rc), reply.clone(), reply));
    }
    let message = if header_route {
        parsed
            .as_ref()
            .and_then(|v| v.get("message").or_else(|| v.get("error")))
            .and_then(Value::as_str)
            .map(str::to_owned)
            .or(error)
            .unwrap_or_else(|| {
                if body.is_empty() {
                    format!("rc={rc} (no error body)")
                } else {
                    body
                }
            })
    } else {
        error.unwrap_or_else(|| {
            parsed
                .as_ref()
                .and_then(|v| v.get("error"))
                .and_then(Value::as_str)
                .map(str::to_owned)
                .unwrap_or_else(|| {
                    if body.trim().is_empty() {
                        "unknown error".into()
                    } else {
                        body.trim().into()
                    }
                })
        })
    };
    Ok((i32::from(rc), MixValue::String(message), reply))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn raw_transport_failure_keeps_delivery_uncertainty_and_never_claims_retry() {
        for unknown in [false, true] {
            let (rc, _, reply) = raw_transport_parts("transport fault".into(), unknown).unwrap();
            assert_eq!(rc, -1);
            let reply = crate::runtime::to_json(&reply).unwrap();
            assert_eq!(reply["outcome_unknown"], unknown);
            assert_eq!(reply["retried"], false);
            assert_eq!(reply["transport_error"], "transport fault");
        }
    }
    #[test]
    fn raw_reply_envelope_preserves_whitespace_and_error_separately() {
        for body in [" { \"x\" : 1 } \n", "permission denied\n", "", "\"text\""] {
            let (rc, result, reply) =
                raw_reply_parts(17, body.into(), Some("refused".into())).unwrap();
            assert_eq!(rc, 17);
            let value = crate::runtime::to_json(&reply).unwrap();
            assert_eq!(value, json!({"rc":17,"body":body,"error":"refused"}));
            assert_eq!(crate::runtime::to_json(&result).unwrap(), value);
        }
        let args = crate::runtime::from_json(json!({"body":"{}","reply_mode":"raw","timeout":30}))
            .unwrap();
        let (headers, body, route) = Bridge::payload("echo", &args).unwrap();
        assert!(route);
        assert_eq!(body, "{}");
        assert!(!headers.contains_key("reply_mode"));
        assert!(!headers.contains_key("timeout"));
    }
    #[test]
    fn native_reply_routes_preserve_structured_refusals_and_scalar_bodies() {
        let body = json!({"error_code":"INVALID_NAME","message":"try another name","retry":true})
            .to_string();
        let (rc, result, reply) = reply_parts(10, body.clone(), None, true).unwrap();
        assert_eq!(rc, 10);
        assert_eq!(
            crate::runtime::to_json(&result).unwrap(),
            json!({"error_code":"INVALID_NAME","message":"try another name","retry":true})
        );
        assert_eq!(crate::runtime::to_json(&reply).unwrap()["retry"], true);
        let (_, result, reply) = reply_parts(10, body.clone(), None, false).unwrap();
        assert_eq!(crate::runtime::to_json(&result).unwrap(), json!(body));
        assert_eq!(
            crate::runtime::to_json(&reply).unwrap()["error_code"],
            "INVALID_NAME"
        );
        let (rc, result, reply) = reply_parts(5, "warning text".into(), None, false).unwrap();
        assert_eq!(rc, 5);
        assert_eq!(
            crate::runtime::to_json(&result).unwrap(),
            json!("warning text")
        );
        assert!(crate::runtime::to_json(&reply).unwrap().is_null());
        let (_, _, reply) = reply_parts(0, "\"typed string\"".into(), None, false).unwrap();
        assert_eq!(
            crate::runtime::to_json(&reply).unwrap(),
            json!("typed string")
        );
    }
    #[test]
    fn native_props_namespace_routes_headers_and_flat_paths_keep_json() {
        let args = crate::runtime::from_json(json!({"namespace":"sample","path":"test","value":3}))
            .unwrap();
        let (headers, body, route) = Bridge::payload("settings.props.set", &args).unwrap();
        assert!(route);
        assert!(body.is_empty());
        assert_eq!(headers["value"], "3");
        let args = crate::runtime::from_json(json!({"path":"test"})).unwrap();
        let (headers, body, route) = Bridge::payload("settings.props.get", &args).unwrap();
        assert!(!route);
        assert!(headers.is_empty());
        assert_eq!(
            serde_json::from_str::<Value>(&body).unwrap()["path"],
            "test"
        );
    }
}
