// SPDX-License-Identifier: MIT OR Apache-2.0
mod chooser;
mod input;
mod native;
mod panel;
#[cfg(test)]
mod raw_names_test;
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
    session: viu::session::Session,
    session_pending: Option<Value>,
    model: Value,
    base_model: Value,
    panel: panel::Panel,
    view: Option<SceneView>,
    scene: String,
    contract: Value,
    default_contract: Value,
    app_dir: PathBuf,
    effect_next: u64,
    effect_admissions: BTreeMap<String, tokio::sync::oneshot::Receiver<(i32, Value)>>,
    macro_session: macros::Session,
    job_replies: BTreeMap<String, Reply>,
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
    title: String,
    window_frame: viu::window_frame::WindowFrame,
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
    fn publish(&mut self, mut model: Value, ctx: &egui::Context) -> Result<(), String> {
        self.base_model = model.clone();
        if self.panel.form["open"] == true {
            model["values"]["form"] = self.panel.form.clone();
        }
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
        if let Some(path) = self.model["panes"][self.model["active"].as_u64().unwrap_or(0) as usize]
            ["path"]
            .as_str()
        {
            let path = path
                .chars()
                .map(|c| if c.is_control() { '�' } else { c })
                .collect::<String>();
            let title = format!(
                "{path} — Dopus {} / Viu",
                if self.mode == "mix" { "Mix" } else { "Rust" }
            );
            if title != self.title {
                ctx.send_viewport_cmd(egui::ViewportCommand::Title(title.clone()));
                self.title = title;
            }
        }
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
        let open_macros = (command == "dopus.execute" && args["id"] == "help.macros")
            || (command == "dopus.intent"
                && ["menu", "toolbar"].iter().any(|id| args["id"] == *id)
                && args["value"]["command"] == "help.macros");
        let panel_args = if open_macros {
            if self.model["values"]["form"]["open"] == true || self.panel.active() {
                if let Some(reply) = reply {
                    let _ = reply.send((
                        10,
                        json!({"error_code":"BUSY","message":"Answer the current dialog first"}),
                    ));
                }
                return;
            }
            Some(json!({"command":"open"}))
        } else if self.panel.active() && command == "dopus.intent" && args["id"] == "form" {
            Some(if args["action"] == "edit" {
                json!({"command":"edit","value":args["value"]})
            } else {
                json!({"command":"answer","choice":args["value"]["choice"],"field":args["value"]["field"],"instance":args["value"]["instance"]})
            })
        } else if self.panel.active() && command == "dopus.dialog.answer" {
            Some(
                json!({"command":"answer","choice":args["choice"],"field":args["text"],"instance":args["instance"]}),
            )
        } else {
            None
        };
        if let Some(args) = panel_args {
            if let Err((error, reply)) = self.panel.call(args, reply) {
                if let Some(reply) = reply {
                    let _ = reply.send((10, json!({"error_code":"BUSY","message":error})));
                } else {
                    self.input_error = Some(error);
                }
            }
            ctx.request_repaint();
            return;
        }
        let quit = command == "dopus.quit"
            || (command == "dopus.execute" && args["id"] == "app.quit")
            || (command == "dopus.intent"
                && ["menu", "toolbar"].iter().any(|id| args["id"] == *id)
                && args["value"]["command"] == "app.quit");
        let ui_call = reply.is_none() && self.input_wait && command != "dopus.persist";
        let human_close = reply.is_none() && quit;
        if !matches!(
            command.as_str(),
            "dopus.persist" | "dopus.quit" | "dopus.session"
        ) {
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
            // Synchronous core commands accumulate prompt/status events.
            // Drain them before publishing or acknowledging their UI result.
            self.native.as_mut().unwrap().poll(ctx);
            self.last_service_us = start.elapsed().as_micros();
            let model = self.native.as_ref().unwrap().model();
            let applied = self.publish(model, ctx);
            let result = result.and_then(|body| applied.map(|_| body));
            if result.as_ref().is_ok_and(|body| body["panel"] == "macros") {
                if let Err((error, reply)) = self.panel.call(json!({"command":"open"}), reply) {
                    if let Some(reply) = reply {
                        let _ = reply.send((10, json!({"error_code":"BUSY","message":error})));
                    } else {
                        self.error = error;
                    }
                }
                ctx.request_repaint();
                return;
            }
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
            ..
        } = r;
        tracing::debug!(command, "Bus command received");
        if let Some(answer) = viu::debug::request("dopus", &command, &args.to_string()) {
            let _ = reply.send(answer);
            return;
        }
        if let Some(suffix) = command.strip_prefix("dopus.macros.") {
            self.effect_next += 1;
            let id = format!("macro-rpc-{}", self.effect_next);
            let path = self.app_dir.join("macro-job.mix");
            let admission=std::fs::read_to_string(&path).map_err(|e|e.to_string()).and_then(|source|self.jobs.submit(json!({"id":id,"owner":0,"path":path,"source":source,"command":"job.run","args":{"command":suffix,"body":args}})));
            match admission {
                Ok(receipt) => {
                    self.effect_admissions.insert(id.clone(), receipt);
                    self.job_replies.insert(id, reply);
                }
                Err(error) => {
                    let _ = reply.send((10, json!({"error_code":"BUSY","message":error})));
                }
            }
            return;
        }
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
            "dopus.session.state" => {
                let _ = reply.send((
                    0,
                    json!({"snapshot":self.session.snapshot,"error":self.session.error}),
                ));
            }
            "dopus.state" | "dopus.ui.tree" => {
                // Optional observation avoids copying the entire rendered row
                // model over the Bus when checking a controller outcome.
                let state = if command == "dopus.state" && args["compact"] == true {
                    json!({"ready":self.ready(),"mode":self.mode,"model":{"panes":self.model["panes"],"active":self.model["active"],"busy":self.model["busy"],"serial":self.model["serial"],"appearance":self.model["appearance"]},"error":self.error,"last_service_us":self.last_service_us,"endpoint":"compact applied-model observation; physical presentation unmeasured"})
                } else {
                    self.state()
                };
                let _ = reply.send((0, state));
            }
            "dopus.describe" | "app.describe" => {
                let _=reply.send((0,json!({"schema":"viu.dopus.v1","mode":self.mode,"actions":self.contract["actions"],"commands":commands(),"shared_scene":true,"shared_widgets":true,"original_core":self.mode=="rust","limits":{"bus":32,"behaviour":32,"jobs":4,"job_admission":8},"features":{"macros":"native session and shared Mix panel","chooser":"--choose-service provides the native files.choose contract with shared Viu controls","external_effects":"native open process admission and mail Bus delivery"},"boundaries":["external application completion is not measured","physical presentation is not measured"]})));
            }
            "dopus.actions" => {
                let _ = reply.send((0, json!({"actions":self.model["actions"]})));
            }
            "dopus.ui.menu.list" => {
                let _ = reply.send((0, json!({"menus":self.model["values"]["menu"]})));
            }
            "dopus.ui.menu" => {
                let snapshot = self
                    .view
                    .as_ref()
                    .map(|view| {
                        viu::rich::menu_snapshot(
                            &self.model["values"]["menu"],
                            "menu",
                            &view.geometry,
                        )
                    })
                    .unwrap_or_else(|| json!({"open":false,"path":[]}));
                let _ = reply.send((0, snapshot));
            }
            "dopus.ui.menu.close" => {
                egui::Popup::close_all(ctx);
                self.replies_after_frame.push(reply);
                ctx.request_repaint();
            }
            "dopus.ui.menu.open" => {
                let result = self
                    .view
                    .as_mut()
                    .ok_or("NOT_READY".into())
                    .and_then(|view| {
                        viu::rich::menu_targets(&self.model["values"]["menu"], "menu", &args)?;
                        if self.input_wait {
                            return Err("BUSY: native input gesture in progress".into());
                        }
                        egui::Popup::close_all(ctx);
                        self.input.queue("menu.open", &args, view)
                    });
                match result {
                    Ok(()) => {
                        self.input_replies.push(reply);
                        self.input_wait = true;
                        self.input_error = None;
                        ctx.request_repaint();
                    }
                    Err(error) => {
                        let _ =
                            reply.send((10, json!({"error_code":"INPUT_REFUSED","message":error})));
                    }
                }
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
                self.effect_next += 1;
                let id = format!("window-rpc-{}", self.effect_next);
                let path = self.app_dir.join("../../libraries/viu/mix/window.mix");
                let admission=std::fs::read_to_string(&path).map_err(|e|e.to_string()).and_then(|source|self.jobs.submit(json!({"id":id,"owner":0,"path":path,"source":source,"command":"job.run","args":{"app_id":format!("mixviu.dopus.{}",self.mode),"pid":std::process::id()}})));
                match admission {
                    Ok(receipt) => {
                        self.effect_admissions.insert(id.clone(), receipt);
                        self.job_replies.insert(id, reply);
                    }
                    Err(error) => {
                        let _ = reply.send((10, json!({"error_code":"BUSY","message":error})));
                    }
                }
            }
            "dopus.window.state" => {
                let state = ctx.input(|i| i.viewport().clone());
                let _=reply.send((0,json!({"inner_rect":state.inner_rect.map(|r|json!([r.min.x,r.min.y,r.width(),r.height()])),"focused":state.focused,"maximized":state.maximized,"minimized":state.minimized})));
            }
            "dopus.window" => {
                if let Some(action) = args["action"].as_str() {
                    match action {
                        "close" => {
                            close_after_reply.store(true, Ordering::Release);
                            self.call("dopus.quit".into(), json!({}), headers, Some(reply), ctx);
                            return;
                        }
                        "minimize" => {
                            if self.input_wait || self.capture_reply.is_some() {
                                let _=reply.send((10,json!({"error_code":"BUSY","message":"Native input or capture is still pending"})));
                                return;
                            }
                            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
                            let _ = reply.send((0, json!({"requested":"minimize"})));
                            return;
                        }
                        "maximize" => ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true)),
                        "restore" => {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(false));
                            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(false));
                        }
                        "focus" => {
                            let _=reply.send((10,json!({"error_code":"UNSUPPORTED","message":"Use dopus.show through compositor authority"})));
                            return;
                        }
                        _ => {
                            let _ = reply.send((10, json!({"error_code":"INVALID_ARGUMENT"})));
                            return;
                        }
                    }
                    self.replies_after_frame.push(reply);
                    ctx.request_repaint();
                    return;
                }
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
        let (changed, error) = self.panel.poll();
        if changed {
            if let Err(error) = self.publish(self.base_model.clone(), ctx) {
                self.error = error;
            }
        }
        if let Some(error) = error {
            self.error = error.clone();
            if self.input_wait {
                self.input_error = Some(error);
            }
        }
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
            if error.is_none()
                && result
                    .reply
                    .as_ref()
                    .is_some_and(|r| r.body["panel"] == "macros")
            {
                if let Err((error, reply)) = self.panel.call(json!({"command":"open"}), reply) {
                    if let Some(reply) = reply {
                        let _ = reply.send((10, json!({"error_code":"BUSY","message":error})));
                    } else {
                        self.error = error;
                    }
                }
                ctx.request_repaint();
                continue;
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
            if let Some(reply) = done.tag["id"]
                .as_str()
                .and_then(|id| self.job_replies.remove(id))
            {
                let _ = reply.send((done.rc, done.body));
            } else if self.runtime.is_some() {
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
                        if let Some(reply) = self.job_replies.remove(id) {
                            let _ = reply.send((rc as u8, body));
                        } else if let Some(native) = self.native.as_mut() {
                            native.effect_result(rc as u8, &body);
                        }
                    }
                    retired.push(id.clone());
                }
                Err(tokio::sync::oneshot::error::TryRecvError::Closed) => {
                    let body =
                        json!({"error_code":"OWNER_CLOSED","message":"Job admission closed"});
                    if let Some(reply) = self.job_replies.remove(id) {
                        let _ = reply.send((20, body));
                    } else if let Some(native) = self.native.as_mut() {
                        native.effect_result(20, &body);
                    }
                    retired.push(id.clone());
                }
                Err(tokio::sync::oneshot::error::TryRecvError::Empty) => {}
            }
        }
        let effect_changed = effect_changed || !retired.is_empty();
        for id in retired {
            self.effect_admissions.remove(&id);
        }
        if effect_changed && self.native.is_some() {
            let model = self.native.as_ref().unwrap().model();
            if let Err(error) = self.publish(model, ctx) {
                self.error = error;
            }
        }
        while self.notices.len() < 64 {
            let Ok(notice) = self.bus.notices.try_recv() else {
                break;
            };
            self.session.hint(&notice.command, &notice.args);
            self.notices.push_back(notice);
        }
        if let Some(snapshot) = self.session.poll() {
            self.session_pending = Some(snapshot);
        }
        if self.ready() {
            if let Some(mut snapshot) = self.session_pending.take() {
                if viu::appearance::resolve(&snapshot["appearance"]).is_err() {
                    for key in ["scheme", "mode", "style", "contrast", "accent"] {
                        snapshot["appearance"].as_object_mut().unwrap().remove(key);
                    }
                }
                if viu::appearance::resolve(&snapshot["appearance"]).is_ok() {
                    self.contract = self.default_contract.clone();
                    for action in self.contract["actions"].as_array_mut().unwrap() {
                        if let Some(keys) = snapshot["keymap"].get(action["id"].as_str().unwrap()) {
                            action["keys"] = keys.clone();
                        }
                    }
                    self.call(
                        "dopus.session".into(),
                        json!({"appearance":snapshot["appearance"],"keymap":snapshot["keymap"]}),
                        BTreeMap::new(),
                        None,
                        ctx,
                    );
                } else {
                    self.error = "Session appearance refused; previous look retained".into();
                }
            }
        }
        if let Some(at) = self.session.deadline() {
            ctx.request_repaint_after(at.saturating_duration_since(Instant::now()));
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
    fn logic(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
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
    }
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        let context = ui.ctx().clone();
        let ctx = &context;
        if self.bus.close_requested.load(Ordering::Acquire)
            || (self.close_after_idle && self.inflight.is_empty())
        {
            return;
        }
        ui.painter().rect_filled(
            ui.max_rect(),
            0.0,
            viu::appearance::colour(ctx, "dock").unwrap_or(ui.visuals().panel_fill),
        );
        self.window_frame.show_with_header(
            ui,
            &self.title,
            &self.session.snapshot["state"]["values"],
            self.view
                .as_ref()
                .is_some_and(|view| view.has_slot("window.header")),
        );
        if let Some(view) = self.view.as_mut() {
            let slots = self
                .window_frame
                .geometry
                .get("window.header")
                .map(|rect| BTreeMap::from([("window.header".into(), *rect)]))
                .unwrap_or_default();
            if let Err(error) = view.set_slots(slots) {
                self.error = error;
            }
        }
        let events = if let Some(view) = &mut self.view {
            view.show(ui)
        } else {
            ui.label("Loading Dopus…");
            Vec::new()
        };
        self.window_frame.edges(ui);
        if let Some(view) = self.view.as_mut() {
            view.geometry.extend(self.window_frame.geometry.clone());
        }
        if let Some(error) = self.view.as_mut().and_then(SceneView::take_focus_error) {
            if self.input_wait {
                self.input_error = Some(error);
            } else {
                self.error = error;
            }
        }
        if let Some(view) = &mut self.view {
            if !self.model["focus"].is_null() {
                let serial = self.model["serial"].as_u64().unwrap_or(0);
                let mark = (serial, self.model["focus"].to_string());
                if self.focus_serial.as_ref() != Some(&mark) {
                    if view.request_focus_value(&self.model["focus"]).is_ok() {
                        self.focus_serial = Some(mark);
                        ctx.request_repaint();
                    }
                }
            }
        }
        // A synchronous controller can close a modal while consuming its
        // Enter/Escape intent. Keep that frame's key inside the drawn modal.
        let shortcuts_blocked = self.model["values"]["form"]["open"] == true
            || self.view.as_ref().is_some_and(|v| v.text_input_active());
        for event in events {
            self.call(event.command, event.body, BTreeMap::new(), None, ctx);
        }
        if self.window_frame.close_requested {
            self.call("dopus.quit".into(), json!({}), BTreeMap::new(), None, ctx);
        }
        for action in input::shortcuts(ctx, &self.contract, shortcuts_blocked) {
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
            } else if let Some(result) = self
                .view
                .as_mut()
                .and_then(|view| self.input.advance_menu(view))
            {
                if let Err(error) = result {
                    self.input_error = Some(error);
                }
                ctx.request_repaint();
            } else if self.input_wait
                && self.inflight.is_empty()
                && !self.panel.pending()
                && !self
                    .view
                    .as_ref()
                    .is_some_and(|v| v.focus_pending() || !v.current_revision_drawn())
            {
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
            } else if self.input_wait
                && self
                    .view
                    .as_ref()
                    .is_some_and(|v| !v.current_revision_drawn())
            {
                ctx.request_repaint();
            }
        }
        let answer = self.state();
        for reply in self.replies_after_frame.drain(..) {
            let _ = reply.send((0, answer.clone()));
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
        if let Err(error) = self.macro_session.shutdown() {
            eprintln!("macro shutdown: {error}");
        }
        for (_, reply) in std::mem::take(&mut self.job_replies) {
            let _ = reply.send((20, json!({"error_code":"OWNER_CLOSED"})));
        }
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
        "dopus.session.state",
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
        "dopus.ui.menu",
        "dopus.ui.menu.list",
        "dopus.ui.menu.open",
        "dopus.ui.menu.close",
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
        "dopus.debug",
        "dopus.debug.set",
        "dopus.debug.recent",
    ]
    .into_iter()
    .map(str::to_owned)
    .chain(
        macros::commands::SUFFIXES
            .into_iter()
            .map(|suffix| format!("dopus.macros.{suffix}")),
    )
    .collect()
}
fn main() -> eframe::Result {
    let mut args = std::env::args().collect::<Vec<_>>();
    if let Err(error) = viu::debug::init("warn", &mut args) {
        eprintln!("viu-dopus: {error}");
        std::process::exit(2);
    }
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
    if let Some(path) = args
        .get(1)
        .filter(|p| p.ends_with(".mix") && std::path::Path::new(p).is_file())
    {
        std::process::exit(viu::macro_host::run(
            PathBuf::from(path),
            args[2..].to_vec(),
        ));
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
    let profile = if profile.is_absolute() {
        profile
    } else {
        std::env::current_dir()
            .expect("working directory")
            .join(profile)
    };
    let scene = std::fs::read_to_string(app_dir.join("scene.mix")).expect("Dopus scene");
    let contract: Value = serde_json::from_str(
        &std::fs::read_to_string(app_dir.join("contract.json")).expect("Dopus contract"),
    )
    .expect("Dopus contract data");
    if args.iter().any(|a| a == "--choose-service") {
        let service = args
            .iter()
            .position(|a| a == "--service")
            .and_then(|i| args.get(i + 1))
            .cloned()
            .unwrap_or_else(|| "files".into());
        return chooser::run(app_dir, profile, service);
    }
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
            let session = viu::session::Session::new(
                app_dir.join("../../libraries/viu/mix/session-runtime.mix"),
                "dopus",
                std::fs::read_to_string("/proc/sys/kernel/hostname")
                    .unwrap_or_else(|_| "host".into())
                    .trim(),
                wake.clone(),
                bus.backend(),
            )
            .map_err(std::io::Error::other)?;
            jobs.set_native_backend(bus.backend_maker());
            let macro_session = macros::Session::with_roots(
                "dopus",
                Some(macros::Roots {
                    builtin: app_dir.join("macros"),
                    user: profile.join("macros"),
                    home: std::env::var_os("HOME")
                        .map(PathBuf::from)
                        .unwrap_or_else(|| profile.clone()),
                    mix: std::env::current_exe().expect("own macro interpreter artifact"),
                }),
            );
            let services = viu::runtime::Services {
                macros: Some(macro_session.clone()),
                ..Default::default()
            };
            let panel = panel::Panel::new(
                app_dir.join("../../libraries/viu/mix/macros.mix"),
                services.clone(),
                wake.clone(),
            )
            .map_err(std::io::Error::other)?;
            jobs.set_services(services.clone());
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
                let mut worker = Runtime::with_services(
                    32,
                    wake.clone(),
                    Some(jobs.backend(Some(bus.backend()))),
                    services,
                );
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
                session,
                session_pending: None,
                model: json!({}),
                base_model: json!({}),
                panel,
                view: None,
                scene,
                default_contract: contract.clone(),
                contract,
                app_dir,
                effect_next: 0,
                effect_admissions: BTreeMap::new(),
                macro_session,
                job_replies: BTreeMap::new(),
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
                title: String::new(),
                window_frame: viu::window_frame::WindowFrame::default(),
            };
            if let Some(native) = &app.native {
                app.publish(native.model(), &cc.egui_ctx)
                    .map_err(|e| std::io::Error::other(e))?;
            }
            Ok(Box::new(app))
        }),
    )
}
