// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native window, macro and diagnostic facilities shared by Mix applications.
use crate::{
    bundle::Manifest,
    native_bus::{Endpoint, Request},
};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{Arc, mpsc},
};
type Reply = tokio::sync::oneshot::Sender<(u8, Value)>;
pub struct Facilities {
    app: String,
    service: String,
    app_commands: std::collections::BTreeSet<String>,
    ui: bool,
    debug: bool,
    installed: bool,
    context: Option<egui::Context>,
    pub surface: Option<Value>,
    pub macros: Option<macros::Session>,
    next: u64,
    pending: BTreeMap<u64, Reply>,
    ready_answers: Vec<(Reply, u8, Value)>,
    input_headers: BTreeMap<u64, BTreeMap<String, String>>,
    completed: mpsc::Receiver<(Reply, macros::commands::Answer)>,
    complete: mpsc::Sender<(Reply, macros::commands::Answer)>,
    finished: mpsc::Receiver<Value>,
    subscriber: Option<u64>,
    wake: crate::runtime::Wake,
    journal: toolkit::executed::Journal,
    events: crate::native_events::Events,
    publishing: Option<(String, Value)>,
}
impl Facilities {
    pub(crate) fn event(&mut self, topic: String, body: Value) {
        self.events.push(topic, body, 0);
    }
    pub fn native_ui(&self) -> bool {
        self.ui
    }
    pub fn new(manifest: &Manifest, wake: crate::runtime::Wake) -> Self {
        let app = manifest.app().to_owned();
        let mut surface = manifest.bus_surface.clone();
        if let Some(surface) = surface.as_mut() {
            let commands = surface["commands"]
                .as_array_mut()
                .expect("validated Bus surface");
            let mut native = Vec::new();
            if manifest.native_ui {
                native.extend(toolkit::drive::describe(&app));
            }
            if manifest.native_debug {
                for descriptor in toolkit::debug::describe(&app) {
                    if !commands
                        .iter()
                        .any(|existing| existing["name"] == descriptor["name"])
                    {
                        commands.push(descriptor);
                    }
                }
            }
            if manifest.native_macros {
                native.extend(macros::commands::describe(&app));
            }
            let overrides: std::collections::BTreeMap<_, _> = commands
                .iter()
                .filter(|c| native.iter().any(|n| n["name"] == c["name"]))
                .map(|c| (c["name"].as_str().unwrap().to_owned(), c.clone()))
                .collect();
            commands.retain(|c| !overrides.contains_key(c["name"].as_str().unwrap()));
            commands.extend(native.into_iter().map(|c| {
                overrides
                    .get(c["name"].as_str().unwrap())
                    .cloned()
                    .unwrap_or(c)
            }));
            let mut names = std::collections::BTreeSet::new();
            commands.retain(|c| names.insert(c["name"].as_str().unwrap_or_default().to_owned()));
        }
        let macros = manifest
            .native_macros
            .then(|| macros::Session::for_app(manifest.drive.as_deref().unwrap_or(&app)));
        let (tx, finished) = mpsc::channel();
        let wake_done = wake.clone();
        let subscriber = macros.as_ref().map(|s| {
            s.add_finish_hook(Arc::new(move |body| {
                let _ = tx.send(body);
                wake_done();
            }))
        });
        let (complete, completed) = mpsc::channel();
        Self {
            app,
            service: std::env::var("VIU_SERVICE").unwrap_or_else(|_| manifest.service.clone()),
            ui: manifest.native_ui || manifest.native_drive,
            debug: manifest.native_debug || manifest.native_drive,
            app_commands: manifest.commands.iter().cloned().collect(),
            installed: false,
            context: None,
            surface,
            macros,
            next: 0,
            pending: BTreeMap::new(),
            ready_answers: Vec::new(),
            input_headers: BTreeMap::new(),
            completed,
            complete,
            finished,
            subscriber,
            wake,
            journal: Default::default(),
            events: Default::default(),
            publishing: None,
        }
    }
    pub fn prepare(&mut self, ctx: &egui::Context, bus: &Endpoint) {
        toolkit::drive::publish_titlebar(ctx, |topic, bodies, lost| {
            self.events.lost_batch(lost);
            for body in bodies {
                self.events.push(topic.into(), body, 0);
            }
        });
        let (records, lost) = self.journal.take();
        self.events.lost_batch(lost);
        for record in records {
            self.events
                .push(toolkit::executed::topic(&self.app), record.body(), 0);
        }
        if self.ui && !self.installed {
            toolkit::drive::install(ctx, &self.app);
            self.installed = true;
            self.context = Some(ctx.clone());
        }
        if self.ui {
            toolkit::drive::set_service(ctx, bus.client().as_ref().map(|_| bus.service()));
        }
        while let Ok((reply, answer)) = self.completed.try_recv() {
            let _ = reply.send(macro_answer(answer));
        }
        while let Ok(body) = self.finished.try_recv() {
            self.events
                .push(format!("{}.macros.finished", self.service), body, 0);
        }
        if self.publishing.is_none() {
            if let Some((topic, body)) = self.events.front() {
                if bus.hint(topic.clone(), body.clone()).is_ok() {
                    self.events.accepted();
                    self.publishing = Some((topic, body));
                }
            }
        }
        if self.ui {
            self.answers(ctx, toolkit::drive::logic(ctx));
        }
    }
    pub fn record(&mut self, args: &Value) -> Result<Value, String> {
        use toolkit::executed::Origin;
        let origin = match args["origin"].as_str() {
            Some("menu") => Origin::Menu,
            Some("shortcut") => Origin::Shortcut,
            Some("palette") => Origin::Palette,
            Some("ui") => Origin::Ui,
            Some("bus") => Origin::Bus,
            Some("macro") => Origin::Macro,
            _ => return Err("unknown action origin".into()),
        };
        let id = args["id"]
            .as_str()
            .filter(|s| !s.is_empty() && s.len() <= 128)
            .ok_or("invalid command id")?;
        self.journal
            .record(id, origin, args["params"].clone(), args["result"].clone());
        Ok(json!({"recorded":true}))
    }
    pub fn input_headers(&self, ctx: &egui::Context) -> BTreeMap<String, String> {
        if let Some(id) = toolkit::drive::input_request(ctx) {
            let mut headers = self.input_headers.get(&id).cloned().unwrap_or_default();
            headers.insert("viu_input".into(), "bus".into());
            headers
        } else {
            BTreeMap::from([("viu_input".into(), "physical".into())])
        }
    }
    pub fn notice(&mut self, command: &str, args: &Value) {
        if command == "bus.hint.completed"
            && self
                .publishing
                .as_ref()
                .is_some_and(|(topic, _)| args["topic"] == topic.as_str())
        {
            let (_, body) = self.publishing.take().unwrap();
            if args["ok"] != true {
                self.events.failed(&body);
            }
            (self.wake)();
        }
    }
    fn answers(&mut self, ctx: &egui::Context, answers: Vec<toolkit::drive::Answer>) {
        for mut answer in answers {
            let input = self.input_headers.remove(&answer.id).is_some();
            if let Some(reply) = self.pending.remove(&answer.id) {
                remap(ctx, &mut answer.body);
                if input && answer.rc == 0 {
                    self.ready_answers.push((reply, answer.rc, answer.body));
                } else {
                    let _ = reply.send((answer.rc, answer.body));
                }
            }
        }
    }
    /// A completed gesture also waits for its in-process application callback
    /// and the resulting scene revision, just like the native Rust shells.
    pub fn settle_answers(&mut self) {
        for (reply, rc, body) in self.ready_answers.drain(..) {
            let _ = reply.send((rc, body));
        }
    }
    pub fn request(&mut self, mut r: Request, ctx: &egui::Context) -> Option<Request> {
        if self
            .surface
            .as_ref()
            .is_some_and(|s| s["arguments_any"] != true)
            && !r.command.starts_with(&format!("{}.macros.", self.app))
            && !r.args.is_object()
        {
            let _ = r.reply.send((
                10,
                json!({"error_code":"ARGUMENT","message":"arguments must be a JSON object"}),
            ));
            return None;
        }
        // Empty native bodies are normalised by the transport adapter.
        if r.command == "app.describe" || r.command == "HELP" {
            if let Some(surface) = &self.surface {
                let _ = r.reply.send((
                    0,
                    if r.command == "HELP" {
                        surface["commands"].clone()
                    } else {
                        surface.clone()
                    },
                ));
                return None;
            }
        }
        if self.surface.is_some()
            && !r.command.starts_with(&format!("{}.macros.", self.app))
            && !r.args.is_object()
        {
            let _ = r.reply.send((
                10,
                json!({"error_code":"ARGUMENT","message":"arguments must be a JSON object"}),
            ));
            return None;
        }
        if r.command == format!("{}.execute", self.app) {
            if let Some(id) = r.args["id"]
                .as_str()
                .and_then(|id| crate::native_menu::application_id(ctx, id))
            {
                r.args["id"] = json!(id);
            }
        }
        if r.command == "app.describe" || r.command == "HELP" {
            if let Some(surface) = &self.surface {
                let _ = r.reply.send((
                    0,
                    if r.command == "HELP" || surface["describe_list"] == true {
                        surface["commands"].clone()
                    } else {
                        surface.clone()
                    },
                ));
                return None;
            }
        }
        let prefix = format!("{}.", self.app);
        if let Some(suffix) = r.command.strip_prefix(&prefix) {
            if self.ui && toolkit::drive::COMMANDS.contains(&suffix) {
                self.next += 1;
                let id = self.next;
                self.pending.insert(id, r.reply);
                self.input_headers.insert(id, r.headers.clone());
                if let Some(answer) = toolkit::drive::request(ctx, id, suffix, &r.args) {
                    self.answers(ctx, vec![answer]);
                }
                return None;
            }
            if let Some(suffix) = suffix.strip_prefix("macros.") {
                // An explicitly declared application workflow owns its command;
                // otherwise the reusable native macro surface handles it.
                if self.app_commands.contains(&r.command) {
                    return Some(r);
                }
                if let Some(session) = &self.macros {
                    match macros::commands::handle(session, suffix, &r.args) {
                        macros::commands::Reply::Immediate(answer) => {
                            let _ = r.reply.send(macro_answer(answer));
                        }
                        macros::commands::Reply::Deferred(rx) => {
                            r.deferred.store(true, std::sync::atomic::Ordering::Release);
                            let tx = self.complete.clone();
                            let wake = self.wake.clone();
                            std::thread::spawn(move || {
                                let answer = rx.recv().unwrap_or_else(|e| {
                                    Err(macros::commands::Refusal::new("INTERNAL", e))
                                });
                                let _ = tx.send((r.reply, answer));
                                wake();
                            });
                        }
                    }
                    return None;
                }
            }
        }
        if self.debug {
            if let Some(answer) = toolkit::debug::request_for(
                &self.service,
                &self.app,
                &r.command,
                &r.args.to_string(),
            ) {
                let _ = r.reply.send(answer);
                return None;
            }
        }
        Some(r)
    }
    pub fn finish(&mut self, ctx: &egui::Context) {
        if self.ui {
            self.answers(ctx, toolkit::drive::finish(ctx));
        }
    }
}
fn macro_answer(answer: macros::commands::Answer) -> (u8, Value) {
    match answer {
        Ok(body) => (0, body),
        Err(e) => (10, json!({"error_code":e.code,"message":e.message})),
    }
}
pub(crate) fn remap(ctx: &egui::Context, value: &mut Value) {
    match value {
        Value::Array(values) => {
            for value in values {
                remap(ctx, value);
            }
        }
        Value::Object(values) => {
            for (key, value) in values {
                if ["id", "command"].contains(&key.as_str()) {
                    if let Some(id) = value
                        .as_str()
                        .and_then(|s| crate::native_menu::application_id(ctx, s))
                    {
                        *value = json!(id);
                    }
                } else {
                    remap(ctx, value);
                }
            }
        }
        _ => {}
    }
}
impl Drop for Facilities {
    fn drop(&mut self) {
        if let Some(ctx) = self.context.clone() {
            self.finish(&ctx);
            if self.macros.is_some() {
                toolkit::macros::close_app(&ctx, &self.app);
            }
        }
        if let Some(session) = &self.macros {
            if let Some(id) = self.subscriber.take() {
                session.remove_finish_hook(id);
            }
            let _ = session.shutdown();
        }
    }
}
