// SPDX-License-Identifier: MIT OR Apache-2.0
//! Scene-authored native viewports. One Mix runtime owns all policy and receives
//! local intents; child callbacks hold only render models and native input.
use crate::{
    input::Input,
    native_bus::Request,
    view::{Event, SceneView},
    widgets::Widgets,
    window_frame::WindowFrame,
};
use egui::ViewportId;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{Arc, Mutex, mpsc},
};
type Reply = tokio::sync::oneshot::Sender<(u8, Value)>;
struct Child {
    key: String,
    viewport: ViewportId,
    title: String,
    source: String,
    model: Value,
    view: SceneView,
    frame: WindowFrame,
    overlays: Widgets,
    input: Input,
    input_reply: Option<Reply>,
    requests: Vec<Request>,
    capture: crate::capture::Capture,
    capture_reply: Option<Reply>,
    pending: usize,
    close_pending: bool,
    error: String,
    size: [f32; 2],
    persist: bool,
    events: mpsc::Sender<(String, Event)>,
}
pub(crate) struct Windows {
    children: BTreeMap<String, Arc<Mutex<Child>>>,
    tx: mpsc::Sender<(String, Event)>,
    rx: mpsc::Receiver<(String, Event)>,
    pending: BTreeMap<u64, String>,
}
impl Default for Windows {
    fn default() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            children: BTreeMap::new(),
            tx,
            rx,
            pending: BTreeMap::new(),
        }
    }
}
fn descriptors(model: &Value) -> Result<&[Value], String> {
    if model["native_windows"].is_null() {
        return Ok(&[]);
    }
    model["native_windows"]
        .as_array()
        .filter(|a| a.len() <= 64)
        .map(Vec::as_slice)
        .ok_or("Native window budget exceeded".into())
}
fn child_model(descriptor: &Value) -> Value {
    let mut model = descriptor["model"].clone();
    model["__viu_window_client"] = json!(true);
    model["__viu_header_hosted"] = json!(false);
    model
}
impl Windows {
    pub(crate) fn validate(&self, model: &Value) -> Result<(), String> {
        let mut keys = BTreeSet::new();
        for descriptor in descriptors(model)? {
            let key = descriptor["key"]
                .as_str()
                .filter(|s| !s.is_empty() && s.len() <= 128)
                .ok_or("Native window key required")?;
            if !keys.insert(key) {
                return Err("Duplicate native window key".into());
            }
            let source = descriptor["scene"]
                .as_str()
                .filter(|s| s.len() <= 1024 * 1024)
                .ok_or("Native window scene required")?;
            if self
                .children
                .get(key)
                .is_some_and(|c| c.lock().unwrap().source != source)
            {
                return Err("A live native window cannot change its scene".into());
            }
            if descriptor["title"].as_str().is_none_or(|s| s.len() > 4096) {
                return Err("Native window title required".into());
            }
            let size = descriptor["size"]
                .as_array()
                .filter(|a| a.len() == 2)
                .ok_or("Native window size required")?;
            if size.iter().any(|n| {
                n.as_f64()
                    .is_none_or(|n| !n.is_finite() || !(240.0..=8192.0).contains(&n))
            }) {
                return Err("Native window size exceeds bounds".into());
            }
            let model = child_model(descriptor);
            super::app_host::validate_overlays(&model)?;
            crate::gold_frame::mount(source, model)?;
        }
        Ok(())
    }
    pub(crate) fn sync(
        &mut self,
        ctx: &egui::Context,
        app: &str,
        model: &Value,
        documents: &crate::documents::Documents,
        native: bool,
    ) {
        let descriptors = descriptors(model).expect("validated native windows");
        self.children.retain(|key, child| {
            if descriptors.iter().any(|d| d["key"] == key.as_str()) {
                return true;
            }
            let mut child = child.lock().unwrap();
            ctx.send_viewport_cmd_to(child.viewport, egui::ViewportCommand::Close);
            toolkit::drive::unregister_viewport(ctx, ViewportId::ROOT, key, child.viewport);
            if child.persist {
                toolkit::saved::secondary::finish(ctx, app, key);
            }
            // The owner can consume a native close click by removing this
            // descriptor. Its input is complete even though no later frame
            // will exist to paint an acknowledgement.
            if child.input.now.is_empty() && child.input.later.is_empty() {
                if let Some(reply) = child.input_reply.take() {
                    let _ = reply.send((0, json!({"window":key,"closed":true})));
                }
            }
            child.refuse_waits("Native window closed");
            false
        });
        for descriptor in descriptors {
            let key = descriptor["key"].as_str().unwrap();
            let model = child_model(descriptor);
            if let Some(child) = self.children.get(key) {
                let mut child = child.lock().unwrap();
                child
                    .view
                    .apply_model(model.clone())
                    .expect("validated child model");
                child.model = model;
                child.title = descriptor["title"].as_str().unwrap().into();
            } else {
                let source = descriptor["scene"].as_str().unwrap().to_owned();
                let mut view = crate::gold_frame::mount(&source, model.clone())
                    .expect("validated child scene");
                view.set_documents(documents.clone());
                view.set_native_menus(native);
                let root = std::env::var_os("XDG_RUNTIME_DIR")
                    .map(std::path::PathBuf::from)
                    .unwrap_or_else(std::env::temp_dir)
                    .join("viu-captures")
                    .join(key);
                let wake = ctx.clone();
                self.children.insert(
                    key.into(),
                    Arc::new(Mutex::new(Child {
                        key: key.into(),
                        viewport: ViewportId::from_hash_of((app, key)),
                        title: descriptor["title"].as_str().unwrap().into(),
                        source,
                        model,
                        view,
                        frame: Default::default(),
                        overlays: Default::default(),
                        input: Default::default(),
                        input_reply: None,
                        requests: Vec::new(),
                        capture: crate::capture::Capture::new(
                            root,
                            Arc::new(move || wake.request_repaint()),
                        ),
                        capture_reply: None,
                        pending: 0,
                        close_pending: false,
                        error: String::new(),
                        size: [
                            descriptor["size"][0].as_f64().unwrap() as f32,
                            descriptor["size"][1].as_f64().unwrap() as f32,
                        ],
                        persist: descriptor["persist"] == true,
                        events: self.tx.clone(),
                    })),
                );
            }
        }
    }
    pub(crate) fn take_one(&mut self) -> Option<(String, Event)> {
        self.rx.try_recv().ok()
    }
    pub(crate) fn submitted(&mut self, sequence: u64, key: String) {
        self.pending.insert(sequence, key);
    }
    pub(crate) fn completed(&mut self, sequence: u64) {
        if let Some(key) = self.pending.remove(&sequence) {
            self.rejected(&key);
        }
    }
    pub(crate) fn rejected(&mut self, key: &str) {
        if let Some(child) = self.children.get(key) {
            let mut c = child.lock().unwrap();
            c.pending = c.pending.saturating_sub(1);
        }
    }
    pub(crate) fn request(&mut self, key: &str, request: Request, ctx: &egui::Context) {
        if let Some(child) = self.children.get(key) {
            let mut child = child.lock().unwrap();
            if child.requests.len() < 16 {
                child.requests.push(request);
                ctx.request_repaint_of(child.viewport);
                return;
            }
        }
        let _ = request.reply.send((
            10,
            json!({"error_code":"NOT_FOUND","message":"No such native window"}),
        ));
    }
    pub(crate) fn raw_input(&mut self, raw: &mut egui::RawInput) -> bool {
        for child in self.children.values() {
            let mut child = child.lock().unwrap();
            if child.viewport == raw.viewport_id {
                raw.dropped_files.append(&mut child.input.dropped);
                if !child.input.now.is_empty() {
                    raw.events.append(&mut child.input.now);
                } else if let Some(mut events) = child.input.later.pop_front() {
                    raw.events.append(&mut events);
                }
                return true;
            }
        }
        false
    }
    pub(crate) fn show(&self, ui: &mut egui::Ui, app: &str) {
        let ctx = ui.ctx().clone();
        for child in self.children.values() {
            let c = child.lock().unwrap();
            let viewport = c.viewport;
            let builder = toolkit::titlebar::viewport(&format!("{app}.{}", c.key), &c.title)
                .with_title(c.title.clone())
                .with_decorations(false)
                .with_inner_size(c.size);
            let builder = if c.persist {
                toolkit::saved::secondary::builder(&ctx, app, &c.key, builder)
            } else {
                builder
            };
            drop(c);
            let weak = Arc::downgrade(child);
            let app = app.to_owned();
            ctx.show_viewport_deferred(viewport, builder, move |ui, _| {
                if let Some(child) = weak.upgrade() {
                    child.lock().unwrap().show(ui, &app);
                }
            });
        }
    }
}
impl Child {
    fn state(&self) -> Value {
        let geometry: serde_json::Map<String, Value> = self
            .view
            .geometry
            .iter()
            .map(|(key, r)| {
                (
                    key.clone(),
                    json!({"min":[r.min.x,r.min.y],"max":[r.max.x,r.max.y]}),
                )
            })
            .collect();
        json!({"ready":true,"window":self.key,"model":self.model,"revision":self.view.revision,"geometry":geometry,"error":self.error})
    }
    fn emit(&mut self, command: String, mut body: Value, ctx: &egui::Context) {
        body["window"] = json!(self.key);
        self.pending += 1;
        let _ = self
            .events
            .send((self.key.clone(), Event { command, body }));
        ctx.request_repaint_of(ViewportId::ROOT);
    }
    fn refuse_waits(&mut self, message: &str) {
        for reply in [self.input_reply.take(), self.capture_reply.take()]
            .into_iter()
            .flatten()
        {
            let _ = reply.send((10, json!({"error_code":"NOT_FOUND","message":message})));
        }
        for request in self.requests.drain(..) {
            let _ = request
                .reply
                .send((10, json!({"error_code":"NOT_FOUND","message":message})));
        }
    }
    fn show(&mut self, ui: &mut egui::Ui, app: &str) {
        let ctx = ui.ctx().clone();
        let dropped = ctx.input(|i| {
            i.raw
                .dropped_files
                .iter()
                .map(|f| f.path().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
        });
        if !dropped.is_empty() {
            self.emit("viu.files.dropped".into(), json!({"paths":dropped}), &ctx);
        }
        toolkit::chrome::client_side(&ctx);
        toolkit::drive::register_viewport(&ctx, ViewportId::ROOT, &self.key, self.viewport);
        if self.persist {
            toolkit::saved::secondary::capture(&ctx, app, &self.key);
        }
        for request in std::mem::take(&mut self.requests) {
            match request.command.as_str() {
                "viu.state" | "viu.ui.tree" => {
                    let _ = request.reply.send((0, self.state()));
                }
                "viu.capture" => match self.capture.request(&ctx, request.args["name"].as_str()) {
                    Ok(_) => self.capture_reply = Some(request.reply),
                    Err(error) => {
                        let _ = request.reply.send((10, json!({"error":error})));
                    }
                },
                command if command.starts_with("viu.ui.") => {
                    let result = if self.input_reply.is_some() {
                        Err("BUSY".into())
                    } else {
                        self.input
                            .queue(&command[7..], &request.args, &mut self.view)
                    };
                    match result {
                        Ok(()) => self.input_reply = Some(request.reply),
                        Err(error) => {
                            let _ = request.reply.send((10, json!({"error":error})));
                        }
                    }
                }
                _ => {
                    let _ = request
                        .reply
                        .send((10, json!({"error_code":"UNKNOWN_COMMAND"})));
                }
            }
        }
        if ctx.input(|i| i.events.contains(&egui::Event::WindowFocused(true))) {
            self.emit("viu.window.focused".into(), json!({}), &ctx);
        }
        let settings = self.model["window_settings"].clone();
        crate::rich::configure_menu(&ctx, &settings, None);
        self.frame
            .show_with_chrome(ui, &self.title, &settings, true, true);
        let slots = self
            .frame
            .geometry
            .get("window.header")
            .map(|rect| {
                let key = if self.view.has_slot("window.header.content") {
                    "window.header.content"
                } else {
                    "window.header"
                };
                let rect = self.frame.geometry.get(key).unwrap_or(rect);
                BTreeMap::from([(key.into(), *rect)])
            })
            .unwrap_or_default();
        if let Err(error) = self.view.set_slots(slots) {
            self.error = error;
        }
        for event in self.view.show(ui) {
            self.emit(event.command, event.body, &ctx);
        }
        self.overlays.begin_frame();
        for overlay in self.model["native_overlays"]
            .as_array()
            .into_iter()
            .flatten()
            .cloned()
            .collect::<Vec<_>>()
        {
            let widget = crate::widgets::Widget::decode(&overlay["definition"])
                .expect("validated child overlay");
            for intent in self.overlays.widget(ui, &widget, &self.model) {
                self.emit(
                    overlay["command"].as_str().unwrap().into(),
                    intent.encode(),
                    &ctx,
                );
            }
        }
        self.view.geometry.extend(self.overlays.geometry.clone());
        if let Some(bindings) = self.model["shortcuts"]
            .as_array()
            .filter(|_| !toolkit::menu::is_open(&ctx))
            .cloned()
        {
            for binding in bindings {
                if binding["enabled"] == false {
                    continue;
                }
                if let (Some(chord), Some(id), Some(command)) = (
                    binding["chord"].as_str(),
                    binding["id"].as_str(),
                    self.model["on_shortcut"].as_str(),
                ) {
                    if crate::input::consume_chord(&ctx, chord) {
                        self.emit(command.into(), json!({"id":id}), &ctx);
                    }
                }
            }
        }
        self.frame
            .finish_captions(ui, self.view.allocated_region("gold-captions"));
        self.frame.edges(ui);
        self.view.geometry.extend(self.frame.geometry.clone());
        if self.frame.close_requested || ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.close_pending = true;
        }
        if self.close_pending && !toolkit::drive::pending(&ctx) {
            self.close_pending = false;
            self.emit("viu.window.close".into(), json!({}), &ctx);
        }
        if self.close_pending {
            ctx.request_repaint();
        }
        if let Some((_, result)) = self.capture.poll(&ctx) {
            if let Some(reply) = self.capture_reply.take() {
                let _ = reply.send(match result {
                    Ok(value) => (0, value),
                    Err(error) => (10, json!({"error":error})),
                });
            }
        }
        if self.input.now.is_empty()
            && self.input.later.is_empty()
            && self.pending == 0
            && self.view.current_revision_drawn()
        {
            if let Some(reply) = self.input_reply.take() {
                let _ = reply.send((0, self.state()));
            }
        }
        if !self.input.now.is_empty() || !self.input.later.is_empty() || self.pending > 0 {
            ctx.request_repaint();
        }
        if !self.error.is_empty() {
            ui.colored_label(egui::Color32::RED, &self.error);
        }
    }
}
