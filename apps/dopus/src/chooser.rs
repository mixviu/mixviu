// SPDX-License-Identifier: MIT OR Apache-2.0
//! Thin native viewport/Bus host for the Mix-authored chooser scene. The
//! imported files engine runs on an existing Viu evaluator worker.
use super::{Reply, input::Input};
use eframe::egui;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU8, Ordering},
    },
};
use viu::{
    native_bus::{Endpoint, Request},
    runtime::{Program, Runtime, Services, Wake},
    view::SceneView,
};
fn viewport(id: &str) -> egui::ViewportId {
    egui::ViewportId::from_hash_of(("viu-files", id))
}
type Windows = Arc<Mutex<BTreeMap<String, Arc<Mutex<Window>>>>>;
struct Window {
    id: String,
    title: String,
    ready: bool,
    worker: Runtime,
    view: Option<SceneView>,
    scene: String,
    model: Value,
    error: String,
    pending: BTreeSet<u64>,
    input: Input,
    requests: VecDeque<Request>,
    input_replies: Vec<Reply>,
    input_wait: bool,
    capture: viu::capture::Capture,
    capture_reply: Option<Reply>,
    effect: Option<Value>,
    seen_effect: u64,
    closed: bool,
    engine_pending: bool,
    input_error: Option<String>,
    created: std::time::Instant,
    mapped: bool,
    applied_focus: Option<Value>,
    fatal: Option<String>,
}
impl Window {
    fn new(
        open: &viu::chooser::Prepared,
        app_dir: &std::path::Path,
        profile: &std::path::Path,
        home: &Value,
        ctx: &egui::Context,
    ) -> Result<Self, String> {
        let id = open.open.id.clone();
        let context = ctx.clone();
        let vid = viewport(&id);
        let wake: Wake = Arc::new(move || {
            context.request_repaint_of(vid);
            context.request_repaint_of(egui::ViewportId::ROOT);
        });
        let mut worker = Runtime::new(32, wake.clone(), None);
        let path = app_dir.join("chooser.mix");
        worker
            .load(Program {
                source: std::fs::read_to_string(&path).map_err(|e| e.to_string())?,
                path,
                scene: "chooser".into(),
                citizen: "files-window".into(),
            })
            .map_err(|e| format!("{e:?}"))?;
        let sequence=worker.event("chooser.open".into(),json!({"id":id,"request":open.open.request.body().map_err(|e|e.message)?,"home":home,"profile":profile})).map_err(|e|format!("{e:?}"))?;
        let captures = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_default()
            .join("viu-chooser-captures")
            .join(&id);
        Ok(Self {
            id,
            title: open.open.request.title.clone(),
            ready: open.ready,
            worker,
            view: None,
            scene: std::fs::read_to_string(app_dir.join("chooser.scene.mix"))
                .map_err(|e| e.to_string())?,
            model: json!({}),
            error: String::new(),
            pending: BTreeSet::from([sequence]),
            input: Input::default(),
            requests: VecDeque::new(),
            input_replies: Vec::new(),
            input_wait: false,
            capture: viu::capture::Capture::new(captures, wake),
            capture_reply: None,
            effect: None,
            seen_effect: 0,
            closed: false,
            engine_pending: false,
            input_error: None,
            created: std::time::Instant::now(),
            mapped: false,
            applied_focus: None,
            fatal: None,
        })
    }
    fn state(&self) -> Value {
        let geometry: BTreeMap<_, _> = self
            .view
            .as_ref()
            .into_iter()
            .flat_map(|v| &v.geometry)
            .map(|(id, r)| {
                (
                    id.clone(),
                    json!({"min":[r.min.x,r.min.y],"max":[r.max.x,r.max.y]}),
                )
            })
            .collect();
        json!({"ready":self.ready&&self.view.is_some(),"request":self.id,"model":self.model,"geometry":geometry,"error":self.error,"pending":self.pending.len(),"endpoint":"model applied/native input consumed; physical presentation unmeasured"})
    }
    fn call(&mut self, args: Value) {
        match self.worker.event("chooser.call".into(), args) {
            Ok(seq) => {
                self.pending.insert(seq);
            }
            Err(error) => self.error = format!("BUSY: {error:?}"),
        }
    }
    fn poll(&mut self, ctx: &egui::Context) -> Result<(), String> {
        if let Some(error) = &self.fatal {
            return Err(error.clone());
        }
        let result = self.poll_inner(ctx);
        if let Err(error) = &result {
            self.fatal = Some(error.clone());
        }
        result
    }
    fn poll_inner(&mut self, ctx: &egui::Context) -> Result<(), String> {
        for result in self.worker.drain() {
            self.pending.remove(&result.sequence);
            if let Some(error) = result.error {
                return Err(error);
            }
            if let Some(reply) = result.reply {
                if reply.rc >= 10 {
                    self.error = reply.body.to_string();
                    if self.input_wait {
                        self.input_error = Some(self.error.clone());
                    }
                }
            }
            if let Some(model) = result.model {
                let style = (self.model["appearance"] != model["appearance"])
                    .then(|| viu::appearance::resolve(&model["appearance"]))
                    .transpose()?;
                if let Some(view) = &mut self.view {
                    view.apply_model(model.clone())?;
                } else {
                    self.view = Some(SceneView::with_model(&self.scene, model.clone())?);
                }
                let serial = model["effect"]["serial"].as_u64().unwrap_or(0);
                if serial > self.seen_effect {
                    self.seen_effect = serial;
                    self.effect = Some(model["effect"].clone());
                }
                self.model = model;
                if let Some(style) = style {
                    viu::appearance::install_style(ctx, &self.model["appearance"], style);
                }
            }
        }
        Ok(())
    }
    fn show(&mut self, ui: &mut egui::Ui) {
        self.mapped = true;
        let ctx = ui.ctx().clone();
        ui.painter().rect_filled(
            ui.max_rect(),
            0.0,
            viu::appearance::colour(&ctx, "dock").unwrap_or(ui.visuals().panel_fill),
        );
        if let Some((_, result)) = self.capture.poll(&ctx) {
            if let Some(reply) = self.capture_reply.take() {
                let _ = reply.send(match result {
                    Ok(body) => (0, body),
                    Err(error) => (20, json!({"error_code":"CAPTURE_FAILED","message":error})),
                });
            }
        }
        if ctx.input(|i| i.viewport().close_requested()) && !self.closed {
            self.closed = true;
            self.effect = Some(json!({"command":"cancel","body":{"request":self.id}}));
        }
        while let Some(request) = self.requests.pop_front() {
            let Request {
                command,
                args,
                reply,
                ..
            } = request;
            let suffix = command.strip_prefix("files.ui.").unwrap_or("");
            if suffix == "tree" {
                let _ = reply.send((0, self.state()));
                continue;
            }
            if !self.ready || self.view.is_none() {
                let _ = reply.send((10, json!({"error_code":"NOT_READY"})));
                continue;
            }
            if suffix == "capture" {
                match self.capture.request(&ctx, args["name"].as_str()) {
                    Ok(_) => self.capture_reply = Some(reply),
                    Err(error) => {
                        let _ = reply
                            .send((10, json!({"error_code":"CAPTURE_REFUSED","message":error})));
                    }
                }
                continue;
            }
            if command == "files.window.state" {
                let v = ctx.input(|i| i.viewport().clone());
                let _=reply.send((0,json!({"inner_rect":v.inner_rect.map(|r|[r.min.x,r.min.y,r.width(),r.height()]),"focused":v.focused})));
                continue;
            }
            if command == "files.window" {
                if let Some(size) = args["size"].as_array().filter(|a| a.len() == 2) {
                    if let (Some(w), Some(h)) = (size[0].as_f64(), size[1].as_f64()) {
                        if w.is_finite()
                            && h.is_finite()
                            && (320.0..=8000.0).contains(&w)
                            && (240.0..=8000.0).contains(&h)
                        {
                            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                                w as f32, h as f32,
                            )));
                            let _ = reply.send((0, json!({"accepted":true})));
                            continue;
                        }
                    }
                }
                let _ = reply.send((10, json!({"error_code":"INVALID_ARGUMENT"})));
                continue;
            }
            match self.input.queue(suffix, &args, self.view.as_mut().unwrap()) {
                Ok(()) => {
                    self.input_replies.push(reply);
                    self.input_wait = true;
                    self.input_error = None;
                    ctx.request_repaint();
                }
                Err(error) => {
                    let _ = reply.send((10, json!({"error_code":"INPUT_REFUSED","message":error})));
                }
            }
        }
        if let Some(view) = &mut self.view {
            let events = ui.add_enabled_ui(self.ready, |ui| view.show(ui)).inner;
            for event in events {
                if let Err(error) = self.view.as_ref().unwrap().admit_intent(
                    event.body["id"].as_str().unwrap_or(""),
                    event.body["action"].as_str().unwrap_or(""),
                    &event.body["value"],
                ) {
                    self.error = error;
                    continue;
                }
                let mut args = event.body;
                args["command"] = json!("intent");
                self.call(args);
            }
        } else {
            ui.label("Loading chooser…");
        }
        if let Some(view) = &mut self.view {
            if let Some(error) = view.take_focus_error() {
                self.error = error.clone();
                if self.input_wait {
                    self.input_error = Some(error);
                }
            }
            let focus = &self.model["focus"];
            if focus.is_null() {
                self.applied_focus = None;
            } else if self.applied_focus.as_ref() != Some(focus)
                && view.request_focus_value(focus).is_ok()
            {
                self.applied_focus = Some(focus.clone());
                ctx.request_repaint();
            }
        }
        if self.ready
            && self.model["values"]["form"]["open"] != true
            && !self.view.as_ref().is_some_and(|v| v.text_input_active())
        {
            let keys:Vec<_>=ctx.input(|input|input.events.iter().filter_map(|event|{
                if let egui::Event::Key{key,pressed:true,repeat:false,modifiers,..}=event{Some(json!({"command":"key","key":viu::input::key_name(*key),"ctrl":modifiers.ctrl,"shift":modifiers.shift,"alt":modifiers.alt}))}else{None}
            }).collect());
            for key in keys {
                self.call(key);
            }
        }
        if self.input.now.is_empty() {
            if let Some(next) = self.input.later.pop_front() {
                self.input.now = next;
                ctx.request_repaint();
            } else if self.input_wait
                && self.pending.is_empty()
                && self.effect.is_none()
                && !self.engine_pending
                && !self
                    .view
                    .as_ref()
                    .is_some_and(|v| v.focus_pending() || !v.current_revision_drawn())
            {
                self.input_wait = false;
                let mut state = self.state();
                let rc = if let Some(error) = self.input_error.take() {
                    state["message"] = json!(error);
                    10
                } else {
                    0
                };
                for reply in self.input_replies.drain(..) {
                    let _ = reply.send((rc, state.clone()));
                }
            }
        }
    }
}
impl Drop for Window {
    fn drop(&mut self) {
        for request in self.requests.drain(..) {
            let _ = request
                .reply
                .send((20, json!({"error_code":"OWNER_CLOSED"})));
        }
        for reply in self.input_replies.drain(..) {
            let _ = reply.send((20, json!({"error_code":"CHOOSER_CLOSED"})));
        }
        if let Some(reply) = self.capture_reply.take() {
            let _ = reply.send((20, json!({"error_code":"CHOOSER_CLOSED"})));
        }
    }
}
enum Pending {
    External {
        reply: Reply,
        delivery: Arc<AtomicU8>,
        choose: bool,
    },
    Window(String),
    Internal,
}
struct Shell {
    app_dir: PathBuf,
    profile: PathBuf,
    topic: String,
    home: Value,
    bus: Endpoint,
    service: viu::chooser::Service,
    engine: Runtime,
    pending: BTreeMap<u64, Pending>,
    delivery: BTreeMap<String, Arc<AtomicU8>>,
    windows: Windows,
    quitting: Option<Reply>,
    error: String,
}
impl Shell {
    fn engine_call(
        &mut self,
        command: &str,
        body: Value,
        caller: String,
        pending: Pending,
    ) -> Result<(), Pending> {
        match self.engine.event(
            "engine.call".into(),
            json!({"command":command,"body":body,"caller":caller,"topic":self.topic}),
        ) {
            Ok(seq) => {
                self.pending.insert(seq, pending);
                Ok(())
            }
            Err(_) => Err(pending),
        }
    }
    fn request(&mut self, request: Request, ctx: &egui::Context) {
        let command = request.command.clone();
        match command.as_str() {
            "HELP" | "app.describe" => {
                let _=request.reply.send((0,json!({"commands":commands(),"contract":"files.choose","topic":self.topic,"view":"shared Viu widgets / Mix behaviour"})));
            }
            "files.quit" => {
                if self.quitting.is_some() {
                    let _ = request.reply.send((10, json!({"error_code":"CLOSING"})));
                    return;
                }
                let ids: Vec<_> = self.service.prepared().keys().cloned().collect();
                for id in ids {
                    if self
                        .engine_call(
                            "cancel",
                            json!({"request":id}),
                            "host".into(),
                            Pending::Internal,
                        )
                        .is_err()
                    {
                        self.error = "Shutdown admission full".into();
                    }
                }
                self.quitting = Some(request.reply);
            }
            command if command.starts_with("files.choose") => {
                let suffix = command.strip_prefix("files.choose.").unwrap_or("choose");
                let caller = caller_key(&request);
                let pending = Pending::External {
                    reply: request.reply,
                    delivery: request.delivery,
                    choose: suffix == "choose",
                };
                if let Err(Pending::External { reply, .. }) =
                    self.engine_call(suffix, request.args, caller, pending)
                {
                    let _ = reply.send((10, json!({"error_code":"BUSY"})));
                }
            }
            _ => {
                let mut windows = self.windows.lock().unwrap();
                let id = request.args["viewport"]
                    .as_str()
                    .map(str::to_owned)
                    .or_else(|| {
                        (windows.len() == 1).then(|| windows.keys().next().unwrap().clone())
                    });
                if let Some(window) = id.and_then(|id| windows.get_mut(&id)) {
                    let mut window = window.lock().unwrap();
                    if command == "files.ui.tree" {
                        let _ = request.reply.send((0, window.state()));
                        return;
                    }
                    if window.requests.len() < 32 {
                        window.requests.push_back(request);
                        ctx.request_repaint_of(viewport(&window.id));
                        return;
                    }
                }
                let _=request.reply.send((10,json!({"error_code":"INVALID_VIEWPORT_OR_BUSY","message":"Specify an open viewport unless exactly one chooser exists"})));
            }
        }
    }
}
// Same broker-origin identity rule used by the original native chooser. Caller
// claims never replace the broker's connection/service facts for rate limits.
fn caller_key(request: &Request) -> String {
    let h = &request.headers;
    match h.get("broker_origin").map(String::as_str) {
        Some("local") if !request.from.is_empty() => format!("local:{}", request.from),
        Some("local") => h
            .get("broker_connection")
            .map(|c| format!("local:connection:{c}"))
            .unwrap_or("anon".into()),
        Some("mesh") => format!(
            "mesh:{}@{}",
            h.get("broker_service")
                .map(String::as_str)
                .unwrap_or("unknown"),
            h.get("broker_peer")
                .map(String::as_str)
                .unwrap_or("unknown")
        ),
        _ => "anon".into(),
    }
}
impl eframe::App for Shell {
    fn raw_input_hook(&mut self, _: &egui::Context, input: &mut egui::RawInput) {
        for (id, window) in self.windows.lock().unwrap().iter() {
            // The Context still names the preceding pass here. RawInput is
            // authoritative before egui enters the addressed child viewport.
            if viewport(id) == input.viewport_id {
                input.events.append(&mut window.lock().unwrap().input.now);
                break;
            }
        }
    }
    fn logic(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        while let Ok(request) = self.bus.requests.try_recv() {
            self.request(request, ctx);
        }
        for result in self.engine.drain() {
            if !result.stdout.is_empty() {
                eprint!("{}", result.stdout);
            }
            if !result.stderr.is_empty() {
                eprint!("{}", result.stderr);
            }
            let Some(pending) = self.pending.remove(&result.sequence) else {
                if let Some(error) = result.error {
                    self.error = error;
                }
                continue;
            };
            let mut answer = if let Some(error) = result.error {
                (20, json!({"error_code":"ENGINE_FAILED","message":error}))
            } else {
                result
                    .reply
                    .map(|r| (r.rc, r.body))
                    .unwrap_or((20, json!({"error_code":"MISSING_REPLY"})))
            };
            match pending {
                Pending::External {
                    reply,
                    delivery,
                    choose,
                } => {
                    if choose && answer.0 == 0 {
                        // A renamed comparison citizen advertises its own
                        // native topic; the canonical provider remains files.
                        answer.1["topic"] = json!(self.topic);
                        if let Some(id) = answer.1["request"].as_str() {
                            self.delivery.insert(id.into(), delivery);
                        }
                    }
                    let _ = reply.send(answer);
                }
                Pending::Window(id) => {
                    if let Some(window) = self.windows.lock().unwrap().get(&id) {
                        let mut window = window.lock().unwrap();
                        window.engine_pending = false;
                        if answer.0 == 0 && !self.service.prepared().contains_key(&id) {
                            let mut state = window.state();
                            state["outcome"] = answer.1;
                            for reply in window.input_replies.drain(..) {
                                let _ = reply.send((0, state.clone()));
                            }
                        } else {
                            window.call(json!({"command":"result","rc":answer.0,"body":answer.1}));
                        }
                    }
                }
                Pending::Internal => {
                    if answer.0 != 0 {
                        self.error = answer.1.to_string();
                    }
                }
            }
        }
        let deliveries: Vec<_> = self
            .delivery
            .iter()
            .filter_map(|(id, ack)| {
                let state = ack.load(Ordering::Acquire);
                (state != 0).then(|| (id.clone(), state))
            })
            .collect();
        for (id, state) in deliveries {
            if self
                .engine_call(
                    if state == 1 { "ready" } else { "lost" },
                    json!({"request":id}),
                    "host".into(),
                    Pending::Internal,
                )
                .is_ok()
            {
                self.delivery.remove(&id);
            }
        }
        let prepared = self.service.prepared();
        let existing: Vec<_> = self.windows.lock().unwrap().keys().cloned().collect();
        for id in existing {
            if !prepared.contains_key(&id) {
                self.windows.lock().unwrap().remove(&id);
                ctx.send_viewport_cmd_to(viewport(&id), egui::ViewportCommand::Close);
            }
        }
        for (id, open) in prepared {
            // The engine accepts failure only after the choose reply has been
            // delivered. Do not start a presenter which can fail before then.
            if !open.ready {
                continue;
            }
            if !self.windows.lock().unwrap().contains_key(&id) {
                match Window::new(&open, &self.app_dir, &self.profile, &self.home, ctx) {
                    Ok(window) => {
                        self.windows
                            .lock()
                            .unwrap()
                            .insert(id.clone(), Arc::new(Mutex::new(window)));
                        ctx.request_repaint_after(std::time::Duration::from_secs(5));
                    }
                    Err(error) => {
                        let _ = self.engine_call(
                            "failed",
                            json!({"request":id,"message":error}),
                            "host".into(),
                            Pending::Internal,
                        );
                        continue;
                    }
                }
            }
            let window = self.windows.lock().unwrap().get(&id).cloned().unwrap();
            let mut window = window.lock().unwrap();
            if window.ready != open.ready {
                window.ready = open.ready;
                ctx.request_repaint_of(viewport(&id));
            }
            if open.ready
                && !window.mapped
                && window.created.elapsed() > std::time::Duration::from_secs(5)
            {
                let _=self.engine_call("failed",json!({"request":id,"message":"Native chooser window did not map within five seconds"}),"host".into(),Pending::Internal);
                continue;
            }
            if let Err(error) = window.poll(ctx) {
                let _ = self.engine_call(
                    "failed",
                    json!({"request":id,"message":error}),
                    "host".into(),
                    Pending::Internal,
                );
            }
            if let Some(effect) = window.effect.clone() {
                if self
                    .engine_call(
                        effect["command"].as_str().unwrap_or(""),
                        effect["body"].clone(),
                        "host".into(),
                        Pending::Window(id.clone()),
                    )
                    .is_ok()
                {
                    window.effect = None;
                    window.engine_pending = true;
                }
            }
        }
        if self.quitting.is_some() && self.pending.is_empty() && self.service.prepared().is_empty()
        {
            let _ = self
                .quitting
                .take()
                .unwrap()
                .send((0, json!({"quitting":true})));
        }
        if self.bus.close_requested.load(Ordering::Acquire) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        let windows: Vec<_> = self
            .windows
            .lock()
            .unwrap()
            .iter()
            .map(|(id, w)| (id.clone(), w.clone()))
            .collect();
        for (id, window) in windows {
            let title = window.lock().unwrap().title.clone();
            ui.ctx().show_viewport_deferred(
                viewport(&id),
                egui::ViewportBuilder::default()
                    .with_title(title)
                    .with_inner_size([760.0, 560.0])
                    .with_min_inner_size([480.0, 320.0])
                    .with_app_id("mixviu.files"),
                move |ui, _| window.lock().unwrap().show(ui),
            );
        }
    }
}
fn commands() -> Vec<String> {
    [
        "HELP",
        "app.describe",
        "files.quit",
        "files.choose",
        "files.choose.result",
        "files.choose.cancel",
        "files.choose.list",
        "files.choose.accept",
        "files.ui.tree",
        "files.ui.focus",
        "files.ui.click",
        "files.ui.pointer",
        "files.ui.scroll",
        "files.ui.key",
        "files.ui.type",
        "files.ui.hover",
        "files.ui.drag",
        "files.ui.capture",
        "files.window",
        "files.window.state",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}
pub fn run(app_dir: PathBuf, profile: PathBuf, service_name: String) -> eframe::Result {
    let topic = format!("{service_name}.chosen");
    eframe::run_native(
        "Viu file chooser",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_visible(false)
                .with_app_id(&service_name),
            ..Default::default()
        },
        Box::new(move |cc| {
            viu::decorations::initialise(cc);
            viu::appearance::install(&cc.egui_ctx);
            cc.egui_ctx.set_embed_viewports(false);
            cc.egui_ctx
                .send_viewport_cmd(egui::ViewportCommand::RequestPaintWhileHidden);
            let context = cc.egui_ctx.clone();
            let wake: Wake = Arc::new(move || context.request_repaint_of(egui::ViewportId::ROOT));
            let bus = Endpoint::start(
                service_name,
                std::env::var("MIXOS_NODED_URL")
                    .unwrap_or_else(|_| "ws://127.0.0.1:4200/ws".into()),
                commands()
                    .into_iter()
                    .map(|name| {
                        bus::CommandDescriptor::new(
                            &name,
                            &[],
                            "Native files.choose contract / shared Viu chooser",
                            name.ends_with("result")
                                || name.ends_with("list")
                                || name.ends_with("tree"),
                        )
                    })
                    .collect(),
                "files.quit".into(),
                wake.clone(),
            );
            let home = std::env::var_os("HOME")
                .map(PathBuf::from)
                .filter(|p| p.is_absolute())
                .unwrap_or_else(|| PathBuf::from("/"));
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64
                ^ std::process::id() as u64;
            let service = viu::chooser::Service::new(
                nonce,
                home.as_os_str().as_encoded_bytes().to_vec(),
                wake.clone(),
            )
            .map_err(std::io::Error::other)?;
            let mut engine = Runtime::with_services(
                32,
                wake,
                Some(bus.backend()),
                Services {
                    chooser: Some(service.clone()),
                    ..Default::default()
                },
            );
            let path = app_dir.join("chooser-engine.mix");
            engine
                .load(Program {
                    source: std::fs::read_to_string(&path)?,
                    path,
                    scene: "files-engine".into(),
                    citizen: "files-engine".into(),
                })
                .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
            Ok(Box::new(Shell {
                app_dir,
                profile,
                topic,
                home: json!(home),
                bus,
                service,
                engine,
                pending: BTreeMap::new(),
                delivery: BTreeMap::new(),
                windows: Arc::default(),
                quitting: None,
                error: String::new(),
            }))
        }),
    )
}
