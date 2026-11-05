// SPDX-License-Identifier: MIT OR Apache-2.0
mod input;
mod native;
#[cfg(test)]
mod tests;
use eframe::egui;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    path::PathBuf,
    sync::{Arc, atomic::Ordering},
    time::Instant,
};
use viu::{
    jobs::Jobs,
    native_bus::{Endpoint, Request},
    runtime::{Program, Runtime},
    view::SceneView,
};
type Reply = tokio::sync::oneshot::Sender<(u8, Value)>;
struct App {
    runtime: Option<Runtime>,
    native: Option<native::Native>,
    jobs: Jobs,
    bus: Endpoint,
    model: Value,
    view: Option<SceneView>,
    scene: String,
    contract: Value,
    app_dir: PathBuf,
    effect_next: u64,
    effect_admissions: BTreeMap<String, tokio::sync::oneshot::Receiver<(i32, Value)>>,
    capture: viu::capture::Capture,
    capture_reply: Option<(u64, u64, u64, Reply)>,
    close_after_idle: bool,
    input_close: Option<Arc<std::sync::atomic::AtomicBool>>,
    input_error: Option<String>,
    ui_sequences: BTreeSet<u64>,
    mode: String,
    pending: BTreeMap<u64, Reply>,
    inflight: BTreeSet<u64>,
    input: input::Input,
    input_replies: Vec<Reply>,
    input_wait: bool,
    replies_after_frame: Vec<Reply>,
    notices: VecDeque<viu::native_bus::Notice>,
    error: String,
    last_service_us: u128,
    last_model_at: Instant,
    save_at: Option<Instant>,
    focus_serial: Option<(u64, String)>,
}
impl App {
    fn ready(&self) -> bool {
        self.view.is_some() && self.model["ready"] == true
    }
    fn state(&self) -> Value {
        let geometry: BTreeMap<_, _> = self
            .view
            .as_ref()
            .into_iter()
            .flat_map(|v| &v.geometry)
            .map(|(k, r)| {
                (
                    k.clone(),
                    json!({"min":[r.min.x,r.min.y],"max":[r.max.x,r.max.y]}),
                )
            })
            .collect();
        json!({"ready":self.ready(),"mode":self.mode,"model":self.model,"geometry":geometry,"error":self.error,"pending":self.pending.len(),"jobs":self.jobs.pending(),"last_service_us":self.last_service_us,"model_age_us":self.last_model_at.elapsed().as_micros(),"endpoint":"model applied/native input consumed; physical presentation is not measured"})
    }
    fn publish(&mut self, model: Value, ctx: &egui::Context) -> Result<(), String> {
        let style = (self.model["appearance"] != model["appearance"])
            .then(|| viu::appearance::resolve(&model["appearance"]))
            .transpose()?;
        if let Some(view) = &mut self.view {
            view.apply_model(model.clone())?;
        } else {
            self.view = Some(SceneView::with_model(&self.scene, model.clone())?);
        }
        if let Some(style) = style {
            viu::appearance::install_style(ctx, &model["appearance"], style);
        }
        self.model = model;
        self.last_model_at = Instant::now();
        Ok(())
    }
    fn call(
        &mut self,
        command: String,
        args: Value,
        headers: BTreeMap<String, String>,
        reply: Option<Reply>,
        ctx: &egui::Context,
    ) {
        if command == "dopus.intent" {
            let admission = self
                .view
                .as_ref()
                .ok_or("scene is not ready".into())
                .and_then(|v| {
                    v.admit_intent(
                        args["id"].as_str().unwrap_or(""),
                        args["action"].as_str().unwrap_or(""),
                        &args["value"],
                    )
                });
            if let Err(error) = admission {
                if let Some(reply) = reply {
                    let _ =
                        reply.send((10, json!({"error_code":"INVALID_INTENT","message":error})));
                } else if self.input_wait {
                    self.input_error = Some(error);
                } else {
                    self.error = error;
                }
                return;
            }
        }
        let quit = command == "dopus.quit"
            || (command == "dopus.execute" && args["id"] == "app.quit")
            || (command == "dopus.intent"
                && ["menu", "toolbar"].iter().any(|id| args["id"] == *id)
                && args["value"]["command"] == "app.quit");
        let ui_call = reply.is_none() && self.input_wait && command != "dopus.persist";
        let human_close = reply.is_none() && quit;
        if !matches!(command.as_str(), "dopus.persist" | "dopus.quit") {
            self.save_at = Some(Instant::now() + std::time::Duration::from_millis(350));
        }
        if let Some(runtime) = &mut self.runtime {
            match runtime.event_with_headers(
                "dopus.call".into(),
                json!({"command":command,"args":args}),
                headers,
            ) {
                Ok(seq) => {
                    self.inflight.insert(seq);
                    if ui_call {
                        self.ui_sequences.insert(seq);
                    }
                    if human_close {
                        if let Some(flag) = &self.input_close {
                            flag.store(true, Ordering::Release);
                        } else {
                            self.close_after_idle = true;
                        }
                    }
                    if let Some(reply) = reply {
                        self.pending.insert(seq, reply);
                    }
                }
                Err(e) => {
                    if let Some(reply) = reply {
                        let _ = reply
                            .send((10, json!({"error_code":"BUSY","message":format!("{e:?}")})));
                    } else {
                        self.error = format!("{e:?}");
                        if ui_call {
                            self.input_error = Some(self.error.clone());
                        }
                    }
                }
            }
        } else {
            let start = Instant::now();
            let result = self.native.as_mut().unwrap().call(&command, &args);
            self.last_service_us = start.elapsed().as_micros();
            let model = self.native.as_ref().unwrap().model();
            let applied = self.publish(model, ctx);
            let result = result.and_then(|body| applied.map(|_| body));
            if human_close && result.is_ok() {
                if let Some(flag) = &self.input_close {
                    flag.store(true, Ordering::Release);
                } else {
                    self.close_after_idle = true;
                }
                ctx.request_repaint();
            }
            if let Some(reply) = reply {
                let _ = reply.send(match result {
                    Ok(body) => (0, body),
                    Err(error) => (10, json!({"error_code":"REFUSED","message":error})),
                });
            } else if let Err(error) = result {
                if ui_call {
                    self.input_error = Some(error.clone());
                }
                self.error = error;
            }
        }
    }
    fn request(&mut self, r: Request, ctx: &egui::Context) {
        let Request {
            command,
            args,
            headers,
            reply,
            close_after_reply,
        } = r;
        if (command == "dopus.execute" && args["id"] == "app.quit")
            || (command == "dopus.intent"
                && ["menu", "toolbar"].iter().any(|id| args["id"] == *id)
                && args["value"]["command"] == "app.quit")
        {
            close_after_reply.store(true, Ordering::Release);
        }
        if !self.ready()
            && !matches!(
                command.as_str(),
                "HELP"
                    | "dopus.ping"
                    | "dopus.state"
                    | "dopus.describe"
                    | "app.describe"
                    | "dopus.quit"
            )
        {
            let _ = reply.send((10, json!({"error_code":"NOT_READY"})));
            return;
        }
        match command.as_str() {
            "HELP" => {
                let _ = reply.send((0, json!({"commands":commands()})));
            }
            "dopus.ping" => {
                let _ = reply.send((0, json!({"ok":true,"mode":self.mode,"ready":self.ready()})));
            }
            "dopus.state" | "dopus.ui.tree" => {
                let _ = reply.send((0, self.state()));
            }
            "dopus.describe" | "app.describe" => {
                let _=reply.send((0,json!({"schema":"viu.dopus.v1","mode":self.mode,"actions":self.contract["actions"],"commands":commands(),"shared_scene":true,"shared_widgets":true,"original_core":self.mode=="rust","limits":{"bus":32,"behaviour":32,"jobs":4,"job_admission":8},"unsupported":["macros","chooser service","external open/mail completion"]})));
            }
            "dopus.actions" => {
                let actions = if let Some(native) = &self.native {
                    native.actions()
                } else {
                    let menus = self.model["values"]["menu"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .flat_map(|m| m["children"].as_array().into_iter().flatten())
                        .cloned()
                        .collect::<Vec<_>>();
                    json!({"actions":self.contract["actions"].as_array().unwrap().iter().map(|a|menus.iter().find(|c|c["id"]==a["id"]).cloned().unwrap_or_else(||a.clone())).collect::<Vec<_>>()})
                };
                let _ = reply.send((0, actions));
            }
            "dopus.info" => {
                let _=reply.send((0,json!({"info":self.model["info"],"busy":self.model["busy"],"panes":self.model["panes"]})));
            }
            "dopus.dialog" => {
                let _ = reply.send((0, self.model["values"]["form"].clone()));
            }
            "dopus.ui.capture" => {
                if args
                    .as_object()
                    .is_none_or(|m| m.keys().any(|k| k != "name"))
                    || args.get("name").is_some_and(|v| !v.is_string())
                {
                    let _=reply.send((10,json!({"error_code":"INVALID_ARGUMENT","message":"Only an optional plain PNG name is accepted"})));
                    return;
                }
                match self.capture.request(ctx, args["name"].as_str()) {
                    Ok(token) => {
                        self.capture_reply = Some((
                            token,
                            self.model["serial"].as_u64().unwrap_or(0),
                            self.view.as_ref().map(|v| v.revision).unwrap_or(0),
                            reply,
                        ))
                    }
                    Err(error) => {
                        let _ = reply
                            .send((10, json!({"error_code":"CAPTURE_REFUSED","message":error})));
                    }
                }
            }
            "dopus.show" => {
                ctx.send_viewport_cmd(egui::ViewportCommand::Visible(true));
                ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
                self.replies_after_frame.push(reply);
            }
            "dopus.window.state" => {
                let state = ctx.input(|i| i.viewport().clone());
                let _=reply.send((0,json!({"inner_rect":state.inner_rect.map(|r|json!([r.min.x,r.min.y,r.width(),r.height()])),"focused":state.focused,"maximized":state.maximized,"minimized":state.minimized})));
            }
            "dopus.window" => {
                let mut valid = false;
                if let Some(size) = args["size"].as_array().filter(|a| a.len() == 2) {
                    if let (Some(w), Some(h)) = (size[0].as_f64(), size[1].as_f64()) {
                        if w.is_finite()
                            && h.is_finite()
                            && (300.0..=8000.0).contains(&w)
                            && (200.0..=8000.0).contains(&h)
                        {
                            ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(egui::vec2(
                                w as f32, h as f32,
                            )));
                            valid = true;
                        }
                    }
                }
                if let Some(pos) = args["position"].as_array().filter(|a| a.len() == 2) {
                    if let (Some(x), Some(y)) = (pos[0].as_f64(), pos[1].as_f64()) {
                        if x.is_finite() && y.is_finite() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::OuterPosition(
                                egui::pos2(x as f32, y as f32),
                            ));
                            valid = true;
                        }
                    }
                }
                if let Some(max) = args["maximized"].as_bool() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(max));
                    valid = true;
                }
                if valid {
                    self.replies_after_frame.push(reply);
                } else {
                    let _ = reply.send((10, json!({"error_code":"INVALID_ARGUMENT"})));
                }
            }
            command if command.starts_with("dopus.ui.") => {
                let suffix = &command[9..];
                let result = self
                    .view
                    .as_mut()
                    .ok_or("NOT_READY".into())
                    .and_then(|view| self.input.queue(suffix, &args, view));
                match result {
                    Ok(()) => {
                        self.input_replies.push(reply);
                        self.input_wait = true;
                        self.input_close = Some(close_after_reply.clone());
                        self.input_error = None;
                        ctx.request_repaint();
                    }
                    Err(error) => {
                        let _ =
                            reply.send((10, json!({"error_code":"INPUT_REFUSED","message":error})));
                    }
                }
            }
            _ => self.call(command, args, headers, Some(reply), ctx),
        }
    }
    fn completions(&mut self, ctx: &egui::Context) {
        if let Some((token, result)) = self.capture.poll(ctx) {
            if let Some((expected, serial, revision, reply)) = self.capture_reply.take() {
                if expected == token {
                    let answer = match result {
                        Ok(mut body) => {
                            body["serial_at_request"] = json!(serial);
                            body["serial_at_completion"] = self.model["serial"].clone();
                            body["revision_at_request"] = json!(revision);
                            body["revision_at_completion"] =
                                json!(self.view.as_ref().map(|v| v.revision).unwrap_or(0));
                            (0, body)
                        }
                        Err(error) => (20, json!({"error_code":"CAPTURE_FAILED","message":error})),
                    };
                    let _ = reply.send(answer);
                } else {
                    let _ = reply.send((20, json!({"error_code":"STALE_CAPTURE"})));
                }
            }
        }
        let results = self.runtime.as_mut().map(|r| r.drain()).unwrap_or_default();
        for result in results {
            let reply = self.pending.remove(&result.sequence);
            self.inflight.remove(&result.sequence);
            let ui_request = self.ui_sequences.remove(&result.sequence);
            if self
                .runtime
                .as_ref()
                .is_some_and(|r| r.generation() != result.generation)
            {
                if ui_request {
                    self.input_error = Some("STALE_GENERATION".into());
                }
                if let Some(reply) = reply {
                    let _ = reply.send((10, json!({"error_code":"STALE_GENERATION"})));
                }
                continue;
            }
            self.last_service_us = result.elapsed.as_micros();
            if !result.stdout.is_empty() {
                eprint!("{}", result.stdout);
            }
            if !result.stderr.is_empty() {
                eprint!("{}", result.stderr);
            }
            let mut error = result.error;
            if ui_request {
                if let Some(error) = &error {
                    self.input_error = Some(error.clone());
                } else if let Some(reply) = &result.reply {
                    if reply.rc >= 10 {
                        self.input_error = Some(reply.body.to_string());
                    }
                }
            }
            if let Some(model) = result.model {
                if let Err(e) = self.publish(model, ctx) {
                    error = Some(e);
                }
            }
            if ui_request {
                if let Some(error) = &error {
                    self.input_error = Some(error.clone());
                }
            }
            if let Some(reply) = reply {
                let answer = if let Some(error) = &error {
                    (20, json!({"error_code":"BEHAVIOUR_FAILED","message":error}))
                } else {
                    result
                        .reply
                        .map(|r| (r.rc, r.body))
                        .unwrap_or((20, json!({"error_code":"MISSING_REPLY"})))
                };
                let _ = reply.send(answer);
            }
            if let Some(error) = error {
                self.error = error;
            }
        }
        if self.native.as_mut().is_some_and(|n| n.poll(ctx)) {
            let model = self.native.as_ref().unwrap().model();
            if let Err(e) = self.publish(model, ctx) {
                self.error = e;
            }
        }
        let effects: Vec<_> = self
            .native
            .as_mut()
            .map(|n| n.effects.drain(..).collect())
            .unwrap_or_default();
        let effect_changed = !effects.is_empty();
        for effect in effects {
            self.effect_next += 1;
            let id = format!("native-effect-{}", self.effect_next);
            let path = self.app_dir.join("effect-job.mix");
            let admission=std::fs::read_to_string(&path).map_err(|e|e.to_string()).and_then(|source|self.jobs.submit(json!({"id":id,"owner":0,"path":path,"source":source,"command":"job.run","args":effect})));
            match admission {
                Ok(receipt) => {
                    self.effect_admissions.insert(id, receipt);
                }
                Err(error) => self
                    .native
                    .as_mut()
                    .unwrap()
                    .effect_result(10, &json!({"message":error})),
            }
        }
        for done in self.jobs.poll() {
            if self.runtime.is_some() {
                self.notices.push_back(viu::native_bus::Notice{command:"dopus.job.completed".into(),args:json!({"tag":done.tag,"rc":done.rc,"body":done.body,"elapsed_us":done.elapsed_us}),headers:BTreeMap::new()});
            } else if let Some(native) = self.native.as_mut() {
                native.effect_result(done.rc, &done.body);
                let model = native.model();
                if let Err(error) = self.publish(model, ctx) {
                    self.error = error;
                }
            }
        }
        let mut retired = Vec::new();
        for (id, receipt) in &mut self.effect_admissions {
            match receipt.try_recv() {
                Ok((rc, body)) => {
                    if rc >= 10 {
                        self.native.as_mut().unwrap().effect_result(rc as u8, &body);
                    }
                    retired.push(id.clone());
                }
                Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
                    self.native
                        .as_mut()
                        .unwrap()
                        .effect_result(20, &json!({"message":"Effect admission closed"}));
                    retired.push(id.clone());
                }
                Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {}
            }
        }
        let effect_changed = effect_changed || !retired.is_empty();
        for id in retired {
            self.effect_admissions.remove(&id);
        }
        if effect_changed {
            let model = self.native.as_ref().unwrap().model();
            if let Err(error) = self.publish(model, ctx) {
                self.error = error;
            }
        }
        while self.notices.len() < 64 {
            let Ok(notice) = self.bus.notices.try_recv() else {
                break;
            };
            self.notices.push_back(notice);
        }
        if let Some(runtime) = &mut self.runtime {
            while let Some(notice) = self.notices.front() {
                if runtime
                    .notify_with_headers(
                        notice.command.clone(),
                        notice.args.clone(),
                        notice.headers.clone(),
                    )
                    .is_err()
                {
                    break;
                }
                self.notices.pop_front();
            }
        } else {
            self.notices.clear();
        }
        let dropped = self.bus.dropped_notices.swap(0, Ordering::AcqRel);
        if dropped > 0 {
            self.error =
                format!("Native Bus overflow: {dropped} notices lost; state refresh required");
        }
    }
}
impl eframe::App for App {
    fn raw_input_hook(&mut self, _: &egui::Context, input: &mut egui::RawInput) {
        input.events.append(&mut self.input.now);
    }
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        let context = ui.ctx().clone();
        let ctx = &context;
        self.completions(ctx);
        if ctx.input(|i| i.viewport().close_requested())
            && !self.close_after_idle
            && !self.bus.close_requested.load(Ordering::Acquire)
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.call("dopus.quit".into(), json!({}), BTreeMap::new(), None, ctx);
        }
        while let Ok(request) = self.bus.requests.try_recv() {
            self.request(request, ctx);
        }
        if self.bus.close_requested.load(Ordering::Acquire)
            || (self.close_after_idle && self.inflight.is_empty())
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            return;
        }
        ui.painter().rect_filled(
            ui.max_rect(),
            0.0,
            viu::appearance::colour(ctx, "dock").unwrap_or(ui.visuals().panel_fill),
        );
        let events = if let Some(view) = &mut self.view {
            view.show(ui)
        } else {
            ui.label("Loading Dopus…");
            Vec::new()
        };
        if let Some(error) = self.view.as_mut().and_then(SceneView::take_focus_error) {
            if self.input_wait {
                self.input_error = Some(error);
            } else {
                self.error = error;
            }
        }
        if let Some(view) = &mut self.view {
            if let Some(id) = self.model["focus"].as_str() {
                let serial = self.model["serial"].as_u64().unwrap_or(0);
                let mark = (serial, id.to_owned());
                if self.focus_serial.as_ref() != Some(&mark) {
                    if view.request_focus(id).is_ok() {
                        self.focus_serial = Some(mark);
                        ctx.request_repaint();
                    }
                }
            }
        }
        for event in events {
            self.call(event.command, event.body, BTreeMap::new(), None, ctx);
        }
        for action in input::Input::shortcuts(
            ctx,
            &self.contract,
            self.model["values"]["form"]["open"] == true
                || self.view.as_ref().is_some_and(|v| v.text_input_active()),
        ) {
            if let Some(key) = action["tree_navigation"].as_str() {
                let side = if self.model["active"] == 0 {
                    "left"
                } else {
                    "right"
                };
                self.call("dopus.intent".into(),json!({"id":format!("{side}-tree"),"action":"navigate","value":{"key":key,"ctrl":action["ctrl"],"shift":action["shift"]}}),BTreeMap::new(),None,ctx);
            } else {
                self.call("dopus.execute".into(), action, BTreeMap::new(), None, ctx);
            }
        }
        if self.input.now.is_empty() {
            if let Some(next) = self.input.later.pop_front() {
                self.input.now = next;
                ctx.request_repaint();
            } else if self.input_wait && self.inflight.is_empty() {
                self.input_wait = false;
                let mut answer = self.state();
                let rc = if let Some(error) = self.input_error.take() {
                    answer["error_code"] = json!("INPUT_ACTION_REFUSED");
                    answer["message"] = json!(error);
                    10
                } else {
                    0
                };
                for reply in self.input_replies.drain(..) {
                    let _ = reply.send((rc, answer.clone()));
                }
                self.input_close = None;
            }
        }
        let answer = self.state();
        for reply in self.replies_after_frame.drain(..) {
            let _ = reply.send((0, answer.clone()));
        }
        if self.save_at.is_some_and(|t| t <= Instant::now()) {
            self.save_at = None;
            self.call(
                "dopus.persist".into(),
                json!({}),
                BTreeMap::new(),
                None,
                ctx,
            );
        } else if let Some(at) = self.save_at {
            ctx.request_repaint_after(at.saturating_duration_since(Instant::now()));
        }
        if let Some(path) = std::env::var_os("VIU_DOPUS_WITNESS") {
            let _ = std::fs::write(path, self.state().to_string());
        }
    }
}
impl Drop for App {
    fn drop(&mut self) {
        self.runtime.take();
        self.jobs.cancel_all();
        self.native.take();
        for (_, reply) in std::mem::take(&mut self.pending) {
            let _ = reply.send((20, json!({"error_code":"OWNER_CLOSED"})));
        }
    }
}
fn commands() -> Vec<String> {
    [
        "HELP",
        "app.describe",
        "dopus.ping",
        "dopus.describe",
        "dopus.info",
        "dopus.state",
        "dopus.dialog",
        "dopus.execute",
        "dopus.intent",
        "dopus.actions",
        "dopus.theme.set",
        "dopus.open",
        "dopus.show",
        "dopus.quit",
        "dopus.dialog.answer",
        "dopus.ui.tree",
        "dopus.ui.focus",
        "dopus.ui.click",
        "dopus.ui.pointer",
        "dopus.ui.scroll",
        "dopus.ui.key",
        "dopus.ui.type",
        "dopus.ui.text",
        "dopus.ui.hover",
        "dopus.ui.drag",
        "dopus.ui.capture",
        "dopus.window",
        "dopus.window.state",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}
fn main() -> eframe::Result {
    let args = std::env::args().collect::<Vec<_>>();
    if args.get(1).is_some_and(|a| a == "--macro") {
        let Some(path) = args.get(2) else {
            eprintln!("usage: viu-dopus --macro SCRIPT [ARG...]");
            std::process::exit(2);
        };
        std::process::exit(viu::macro_host::run(
            PathBuf::from(path),
            args[3..].to_vec(),
        ));
    }
    // Keep the native runner's nested-interpreter argv form. It enters this
    // artifact's own Mix library, just like the explicit --macro flag.
    if let Some(path)=args.get(1).filter(|p|p.ends_with(".mix")&&std::path::Path::new(p).is_file()){
        std::process::exit(viu::macro_host::run(PathBuf::from(path),args[2..].to_vec()));
    }
    let mode = args
        .get(1)
        .filter(|s| ["rust", "mix"].contains(&s.as_str()))
        .expect("usage: viu-dopus rust|mix APP_DIR PROFILE_DIR")
        .clone();
    let app_dir = PathBuf::from(args.get(2).expect("APP_DIR"))
        .canonicalize()
        .expect("app directory");
    let profile = PathBuf::from(args.get(3).expect("PROFILE_DIR"));
    let scene = std::fs::read_to_string(app_dir.join("scene.mix")).expect("Dopus scene");
    let contract: Value = serde_json::from_str(
        &std::fs::read_to_string(app_dir.join("contract.json")).expect("Dopus contract"),
    )
    .expect("Dopus contract data");
    let title = format!(
        "Dopus — {} / Viu",
        if mode == "rust" { "Rust" } else { "Mix" }
    );
    eframe::run_native(
        &title,
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([980.0, 760.0])
                .with_min_inner_size([480.0, 300.0])
                .with_app_id(format!("mixviu.dopus.{mode}")),
            ..Default::default()
        },
        Box::new(move |cc| {
            viu::appearance::install(&cc.egui_ctx);
            let context = cc.egui_ctx.clone();
            let wake: viu::runtime::Wake = Arc::new(move || context.request_repaint());
            let descriptors = commands()
                .into_iter()
                .map(|name| {
                    bus::CommandDescriptor::new(
                        &name,
                        &[],
                        "See dopus.describe for the comparison contract",
                        name.ends_with("state") || name.ends_with("describe") || name == "HELP",
                    )
                })
                .collect();
            let bus = Endpoint::start(
                format!("dopus-{mode}"),
                std::env::var("MIXOS_NODED_URL")
                    .unwrap_or_else(|_| "ws://127.0.0.1:4200/ws".into()),
                descriptors,
                "dopus.quit".into(),
                wake.clone(),
            );
            let mut jobs = Jobs::new(4, wake.clone());
            jobs.set_native_backend(bus.backend_maker());
            let capture = viu::capture::Capture::new(
                std::env::var_os("XDG_RUNTIME_DIR")
                    .map(PathBuf::from)
                    .map(|p| p.join(format!("viu-dopus-{mode}/captures")))
                    .unwrap_or_default(),
                wake.clone(),
            );
            let mut runtime = None;
            let mut native = None;
            if mode == "mix" {
                let path = app_dir.join("dopus.mix");
                let mut worker =
                    Runtime::new(32, wake.clone(), Some(jobs.backend(Some(bus.backend()))));
                worker
                    .load(Program {
                        source: std::fs::read_to_string(&path).expect("Dopus Mix source"),
                        path,
                        scene: "dopus".into(),
                        citizen: "dopus-mix".into(),
                    })
                    .expect("load Dopus");
                worker
                    .event(
                        "dopus.initialise".into(),
                        json!({"app_dir":app_dir,"profile":profile}),
                    )
                    .expect("initialise Dopus");
                runtime = Some(worker);
            } else {
                native = Some(native::Native::new(&profile, contract.clone(), wake));
            }
            let mut app = App {
                runtime,
                native,
                jobs,
                bus,
                model: json!({}),
                view: None,
                scene,
                contract,
                app_dir,
                effect_next: 0,
                effect_admissions: BTreeMap::new(),
                capture,
                capture_reply: None,
                close_after_idle: false,
                input_close: None,
                input_error: None,
                ui_sequences: BTreeSet::new(),
                mode,
                pending: BTreeMap::new(),
                inflight: BTreeSet::new(),
                input: input::Input::default(),
                input_replies: Vec::new(),
                input_wait: false,
                replies_after_frame: Vec::new(),
                notices: VecDeque::new(),
                error: String::new(),
                last_service_us: 0,
                last_model_at: Instant::now(),
                save_at: None,
                focus_serial: None,
            };
            if let Some(native) = &app.native {
                app.publish(native.model(), &cc.egui_ctx)
                    .map_err(|e| std::io::Error::other(e))?;
            }
            Ok(Box::new(app))
        }),
    )
}
