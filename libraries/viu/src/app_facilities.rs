// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native window, macro and diagnostic facilities shared by Mix applications.
use crate::{
    bundle::Manifest,
    native_bus::{Endpoint, Request},
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{Arc, mpsc},
};
type Reply = tokio::sync::oneshot::Sender<(u8, Value)>;
pub struct Facilities {
    app: String,
    ui: bool,
    debug: bool,
    installed: bool,
    context: Option<egui::Context>,
    pub surface: Option<Value>,
    pub macros: Option<macros::Session>,
    next: u64,
    pending: BTreeMap<u64, Reply>,
    completed: mpsc::Receiver<(Reply, macros::commands::Answer)>,
    complete: mpsc::Sender<(Reply, macros::commands::Answer)>,
    finished: mpsc::Receiver<Value>,
    subscriber: Option<u64>,
    hints: VecDeque<Value>,
    wake: crate::runtime::Wake,
}
impl Facilities {
    pub fn native_ui(&self) -> bool {
        self.ui
    }
    pub fn new(manifest: &Manifest, wake: crate::runtime::Wake) -> Self {
        let app = manifest.service.clone();
        let mut surface = manifest.bus_surface.clone();
        if let Some(surface) = surface.as_mut() {
            let commands = surface["commands"]
                .as_array_mut()
                .expect("validated Bus surface");
            if manifest.native_ui {
                commands.extend(toolkit::drive::describe(&app));
            }
            if manifest.native_macros {
                commands.extend(macros::commands::describe(&app));
            }
            if manifest.native_debug {
                commands.extend(toolkit::debug::describe(&app));
            }
        }
        let macros = manifest
            .native_macros
            .then(|| macros::Session::for_app(&app));
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
            ui: manifest.native_ui,
            debug: manifest.native_debug,
            installed: false,
            context: None,
            surface,
            macros,
            next: 0,
            pending: BTreeMap::new(),
            completed,
            complete,
            finished,
            subscriber,
            hints: VecDeque::new(),
            wake,
        }
    }
    pub fn prepare(&mut self, ctx: &egui::Context, bus: &Endpoint) {
        if self.ui && !self.installed {
            toolkit::drive::install(ctx, &self.app);
            self.installed = true;
            self.context = Some(ctx.clone());
        }
        if self.ui {
            toolkit::drive::set_service(ctx, bus.client().as_ref().map(|_| self.app.as_str()));
        }
        while let Ok((reply, answer)) = self.completed.try_recv() {
            let _ = reply.send(macro_answer(answer));
        }
        while let Ok(body) = self.finished.try_recv() {
            self.hints.push_back(body);
        }
        while let Some(body) = self.hints.front() {
            if bus
                .hint(format!("{}.macros.finished", self.app), body.clone())
                .is_err()
            {
                break;
            }
            self.hints.pop_front();
        }
        self.answers(ctx, toolkit::drive::logic(ctx));
    }
    fn answers(&mut self, ctx: &egui::Context, answers: Vec<toolkit::drive::Answer>) {
        for mut answer in answers {
            if let Some(reply) = self.pending.remove(&answer.id) {
                remap(ctx, &mut answer.body);
                let _ = reply.send((answer.rc, answer.body));
            }
        }
    }
    pub fn request(&mut self, r: Request, ctx: &egui::Context) -> Option<Request> {
        if r.command == "app.describe" {
            if let Some(surface) = &self.surface {
                let _ = r.reply.send((0, surface.clone()));
                return None;
            }
        }
        let prefix = format!("{}.", self.app);
        if let Some(suffix) = r.command.strip_prefix(&prefix) {
            if self.ui && toolkit::drive::COMMANDS.contains(&suffix) {
                self.next += 1;
                let id = self.next;
                self.pending.insert(id, r.reply);
                if let Some(answer) = toolkit::drive::request(ctx, id, suffix, &r.args) {
                    self.answers(ctx, vec![answer]);
                }
                return None;
            }
            if let Some(suffix) = suffix.strip_prefix("macros.") {
                if let Some(session) = &self.macros {
                    match macros::commands::handle(session, suffix, &r.args) {
                        macros::commands::Reply::Immediate(answer) => {
                            let _ = r.reply.send(macro_answer(answer));
                        }
                        macros::commands::Reply::Deferred(rx) => {
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
            if let Some(answer) =
                toolkit::debug::request(&self.app, &r.command, &r.args.to_string())
            {
                let _ = r.reply.send(answer);
                return None;
            }
        }
        Some(r)
    }
    pub fn finish(&mut self, ctx: &egui::Context) {
        self.answers(ctx, toolkit::drive::finish(ctx));
    }
}
fn macro_answer(answer: macros::commands::Answer) -> (u8, Value) {
    match answer {
        Ok(body) => (0, body),
        Err(e) => (10, json!({"error_code":e.code,"message":e.message})),
    }
}
fn remap(ctx: &egui::Context, value: &mut Value) {
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
