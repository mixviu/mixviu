// SPDX-License-Identifier: MIT OR Apache-2.0
use crate::{
    bundle::Bundle,
    input::Input,
    native_bus::{Endpoint, Request},
    runtime::Runtime,
    view::SceneView,
};
use eframe::egui;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    time::{Duration, Instant},
};
type Reply = tokio::sync::oneshot::Sender<(u8, Value)>;
fn validate_overlays(model: &Value) -> Result<(), String> {
    if let Some(overlays) = model.get("native_overlays") {
        let overlays = overlays
            .as_array()
            .filter(|a| a.len() <= 16)
            .ok_or("native overlays exceed budget")?;
        let mut ids = BTreeSet::new();
        for overlay in overlays {
            if !overlay["command"].is_string() {
                return Err("overlay requires an intent command".into());
            }
            let widget = crate::widgets::Widget::decode(&overlay["definition"])?;
            if !ids.insert(widget.id.clone()) {
                return Err("duplicate overlay identity".into());
            }
            widget.validate_model(model)?;
        }
    }
    Ok(())
}
struct Host {
    headless: bool,
    local_ui: crate::local_ui::LocalUi,
    native_choice: Value,
    overlays: crate::widgets::Widgets,
    focus_serial: Value,
    facilities: crate::app_facilities::Facilities,
    session: Option<crate::session::Session>,
    session_appearance: Option<Value>,
    session_window: Option<Value>,
    service: String,
    drive: Option<String>,
    drive_pending: BTreeMap<u64, Reply>,
    drive_sequence: u64,
    wire_deliveries: Vec<Arc<std::sync::atomic::AtomicU8>>,
    file_theme: toolkit::Theme,
    last_ui: settings::model::CommonUi,
    session_dirty: bool,
    session_notice: Option<Value>,
    framed: bool,
    title: String,
    window_frame: crate::window_frame::WindowFrame,
    jobs: crate::jobs::Jobs,
    job_results: std::collections::VecDeque<Value>,
    clipboard: crate::thumbnail::Decoder<String>,
    clipboard_reply: Option<Reply>,
    local_clipboard: std::sync::mpsc::Receiver<crate::clipboard::Request>,
    macros: macros::Session,
    commands: BTreeSet<String>,
    scene_source: String,
    loaded: bool,
    model: Value,
    output: Option<String>,
    outputs: BTreeMap<String, Value>,
    runtime: Runtime,
    view: SceneView,
    bus: Endpoint,
    pending: BTreeMap<u64, Reply>,
    retained: BTreeMap<u64, Arc<std::sync::atomic::AtomicBool>>,
    inflight: BTreeSet<u64>,
    input: Input,
    input_reply: Option<Reply>,
    capture: crate::capture::Capture,
    capture_reply: Option<Reply>,
    ready: bool,
    error: String,
    tick: Option<Duration>,
    next_tick: Instant,
}
impl Host {
    fn window_settings(&self, model: &Value) -> Value {
        let mut settings = self
            .session_window
            .clone()
            .unwrap_or_else(|| model["window_settings"].clone());
        for (axis, key) in [
            ("decorations", "window.decorations"),
            ("caption_side", "window.caption_side"),
        ] {
            if !self.native_choice[axis].is_null() {
                settings[key] = self.native_choice[axis].clone();
            }
        }
        settings
    }
    fn projection(&self, model: &Value, ctx: &egui::Context) -> Value {
        let mut projected = self
            .output
            .as_ref()
            .and_then(|output| model["surfaces"].get(output))
            .unwrap_or(model)
            .clone();
        if self.framed {
            projected["__viu_window_client"] =
                json!(crate::window_frame::is_client(&self.window_settings(model)));
            projected["__viu_header_hosted"] = json!(
                projected["__viu_window_client"] == false
                    && toolkit::menu::form(ctx)["hosted"] == true
                    && toolkit::titlebar::hosting::hosted(ctx)
            );
        }
        projected
    }
    fn state(&self) -> Value {
        let geometry: serde_json::Map<String, Value> = self
            .view
            .geometry
            .iter()
            .map(|(id, r)| {
                (
                    id.clone(),
                    json!({"min":[r.min.x,r.min.y],"max":[r.max.x,r.max.y]}),
                )
            })
            .collect();
        json!({"ready":self.ready,"model":self.view.model(),"revision":self.view.revision,"geometry":geometry,"outputs":self.outputs,"error":self.error,"session":self.session.as_ref().map(|session| &session.snapshot)})
    }
    fn dispatch(
        &mut self,
        command: String,
        body: Value,
        headers: BTreeMap<String, String>,
        reply: Option<Reply>,
    ) {
        match self.runtime.event_with_headers(command, body, headers) {
            Ok(seq) => {
                self.inflight.insert(seq);
                if let Some(reply) = reply {
                    self.pending.insert(seq, reply);
                }
            }
            Err(e) => {
                if let Some(reply) = reply {
                    let _ = reply.send((10, json!({"error_code":"BUSY","error":format!("{e:?}")})));
                }
            }
        }
    }
    fn request(&mut self, mut r: Request, ctx: &egui::Context) {
        self.wire_deliveries.push(r.delivery.clone());
        let Some(owned) = self.facilities.request(r, ctx) else {
            return;
        };
        r = owned;
        if self.model["window_close"] == true {
            let _ = r.reply.send((10, json!({"error_code":"CLOSING"})));
            return;
        }
        if !self.commands.contains(&r.command) {
            let _ = r.reply.send((
                10,
                json!({"error_code":"UNKNOWN_COMMAND","command":r.command}),
            ));
            return;
        }
        if let Some(prefix) = &self.drive {
            if let Some(answer) =
                toolkit::debug::request_for(&self.service, prefix, &r.command, &r.args.to_string())
            {
                let _ = r.reply.send(answer);
                return;
            }
        }
        if let Some(prefix) = &self.drive {
            if let Some(command) = r.command.strip_prefix(&format!("{prefix}.")) {
                if toolkit::drive::COMMANDS.contains(&command) {
                    if self.headless {
                        let _ = r.reply.send((10,json!({"error_code":"HEADLESS","message":"a headless instance has no window"})));
                        return;
                    }
                    self.drive_sequence += 1;
                    let id = self.drive_sequence;
                    if command == "window" && r.args["action"] == "close" {
                        if let Some(mut answer) =
                            toolkit::drive::request(ctx, id, "window.state", &json!({}))
                        {
                            answer.body["requested"] = json!("close");
                            let _ = r.reply.send((answer.rc, answer.body));
                        }
                        self.dispatch("viu.frame.close".into(), json!({}), BTreeMap::new(), None);
                    } else if let Some(mut answer) =
                        toolkit::drive::request(ctx, id, command, &r.args)
                    {
                        crate::app_facilities::remap(ctx, &mut answer.body);
                        let _ = r.reply.send((answer.rc, answer.body));
                    } else {
                        self.drive_pending.insert(id, r.reply);
                    }
                    return;
                }
            }
        }
        match r.command.as_str() {
            "viu.state" | "viu.ui.tree" => {
                let _ = r.reply.send((0, self.state()));
            }
            "viu.quit" => {
                let _ = r.reply.send((0, json!({"ok":true})));
            }
            "viu.clipboard.image" => {
                let path = r.args["path"].as_str().unwrap_or("");
                if self.clipboard_reply.is_some()
                    || !std::path::Path::new(path).is_absolute()
                    || !self
                        .clipboard
                        .replace(&path.to_owned(), std::path::Path::new(path), ctx)
                {
                    let _ = r
                        .reply
                        .send((10, json!({"error":"clipboard busy or invalid image path"})));
                } else {
                    self.clipboard_reply = Some(r.reply);
                }
            }
            _ if !self.ready => {
                let _ = r.reply.send((10, json!({"error_code":"NOT_READY"})));
            }
            "viu.capture" if self.output.is_some() => {
                let _=r.reply.send((10,json!({"error_code":"LAYER_CAPTURE_UNAVAILABLE","message":"Use the compositor capture endpoint for layer surfaces"})));
            }
            "viu.capture" => match self.capture.request(ctx, r.args["name"].as_str()) {
                Ok(_) => self.capture_reply = Some(r.reply),
                Err(e) => {
                    let _ = r.reply.send((10, json!({"error":e})));
                }
            },
            command if command.starts_with("viu.ui.") => {
                let result = if self.input_reply.is_some() {
                    Err("BUSY".into())
                } else {
                    self.input.queue(&command[7..], &r.args, &mut self.view)
                };
                match result {
                    Ok(()) => {
                        self.input_reply = Some(r.reply);
                        ctx.request_repaint();
                    }
                    Err(e) => {
                        let _ = r.reply.send((10, json!({"error":e})));
                    }
                }
            }
            _ => {
                r.headers.insert("from".into(), r.from);
                let before = self.runtime.sequence();
                self.dispatch(r.command, r.args, r.headers, Some(r.reply));
                let after = self.runtime.sequence();
                if after != before {
                    self.retained.insert(after, r.deferred);
                }
            }
        }
    }
}
impl eframe::App for Host {
    fn logic(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        self.pump(ctx);
    }
    fn raw_input_hook(&mut self, _: &egui::Context, raw: &mut egui::RawInput) {
        if !self.input.now.is_empty() {
            raw.events.append(&mut self.input.now);
        } else if let Some(mut events) = self.input.later.pop_front() {
            raw.events.append(&mut events);
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        self.frame(ui);
    }
}
impl Drop for Host {
    fn drop(&mut self) {
        let _ = self.macros.shutdown();
    }
}
impl Host {
    fn pump(&mut self, ctx: &egui::Context) {
        self.wire_deliveries
            .retain(|state| state.load(std::sync::atomic::Ordering::Acquire) == 0);
        // A native input pass may dispatch a Mix reducer. Advance the next
        // drive step only after its model has been accepted and drawn, so a
        // following key/type command sees the resulting dialog and focus.
        if self.drive.is_some() && self.inflight.is_empty() && self.view.current_revision_drawn() {
            for mut answer in toolkit::drive::logic(ctx) {
                if let Some(reply) = self.drive_pending.remove(&answer.id) {
                    crate::app_facilities::remap(ctx, &mut answer.body);
                    let _ = reply.send((answer.rc, answer.body));
                }
            }
        }
        while let Ok(request) = self.local_clipboard.try_recv() {
            if self.clipboard_reply.is_none()
                && self
                    .clipboard
                    .replace(&request.path, std::path::Path::new(&request.path), ctx)
            {
                self.clipboard_reply = Some(request.reply);
            } else {
                let _ = request.reply.send((10, json!({"error_code":"BUSY"})));
            }
        }
        let gone: Vec<_> = self
            .pending
            .iter()
            .filter_map(|(seq, reply)| reply.is_closed().then_some(*seq))
            .collect();
        for seq in gone {
            if self.runtime.forget_reply(seq).is_ok() {
                self.pending.remove(&seq);
                self.retained.remove(&seq);
            }
        }
        self.facilities.prepare(ctx, &self.bus);
        while let Ok(request) = self.local_ui.requests.try_recv() {
            let answer = match request.command.as_str() {
                "native.executed" => self.facilities.record(&request.args),
                "native.bindings" => crate::shortcuts::resolve(&request.args),
                "native.theme.file" => Ok(crate::native_theme::describe(&toolkit::Theme::load())),
                "native.copy" => match request.args["text"]
                    .as_str()
                    .filter(|s| s.len() <= 1_048_576)
                {
                    Some(text) => {
                        ctx.copy_text(text.into());
                        Ok(json!({"copied":true}))
                    }
                    None => Err("text must be bounded text".into()),
                },
                "native.focus" => request.args["id"]
                    .as_str()
                    .ok_or("id required".into())
                    .and_then(|id| self.view.request_focus(id))
                    .map(|_| json!({"focused":true})),
                "native.theme" => {
                    let snapshot = self.session.as_ref().map(|s| s.snapshot.clone());
                    crate::native_theme::choice(&request.args).and_then(|choice| {
                        let base = if let Some(snapshot) = snapshot {
                            crate::appearance::session_base(
                                &snapshot,
                                self.drive.as_deref().unwrap_or(&self.service),
                                &self.file_theme,
                                &self.last_ui,
                            )?
                            .0
                        } else {
                            self.file_theme.clone()
                        };
                        let theme = base.with_choice(&choice).map_err(|e| e.to_string())?;
                        crate::appearance::install_theme(ctx, &theme)?;
                        self.native_choice = request.args.clone();
                        Ok(crate::native_theme::describe(&theme))
                    })
                }
                _ => Err("unknown native operation".into()),
            };
            let _ = request.reply.send(match answer {
                Ok(body) => (0, body),
                Err(message) => (10, json!({"error_code":"ARGUMENT","message":message})),
            });
        }
        for (_, image) in self.clipboard.drain() {
            if let Some(reply) = self.clipboard_reply.take() {
                let result = if let Some(image) = image {
                    ctx.copy_image(image);
                    (
                        0,
                        json!({"submitted":true,"endpoint":"native clipboard submission; readback is separate"}),
                    )
                } else {
                    (10, json!({"error":"image decode refused"}))
                };
                let _ = reply.send(result);
            }
        }
        for done in self.jobs.poll() {
            self.job_results.push_back(
                json!({"tag":done.tag,"rc":done.rc,"body":done.body,"elapsed_us":done.elapsed_us}),
            );
        }
        while let Some(body) = self.job_results.front() {
            if self
                .runtime
                .notify("viu.job.completed".into(), body.clone())
                .is_err()
            {
                break;
            }
            self.job_results.pop_front();
        }
        if self
            .bus
            .close_requested
            .load(std::sync::atomic::Ordering::Acquire)
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
        }
        while let Some(done) = self.runtime.try_completion() {
            self.inflight.remove(&done.sequence);
            if !self.runtime.accepts(&done) {
                continue;
            }
            if !done.stderr.is_empty() {
                tracing::warn!(callback = done.sequence, message = %done.stderr, "Mix callback diagnostic");
            }
            if !done.stdout.is_empty() {
                tracing::info!(callback = done.sequence, message = %done.stdout, "Mix callback output");
            }
            if let Some(error) = done.error.as_ref() {
                eprintln!("viu callback {}: {error}", done.sequence);
                self.error = error.clone();
            } else if done.model.is_some() {
                self.error.clear();
            }
            if self.error.is_empty() {
                if let Some(model) = done.model {
                    self.session_dirty |= self.model["theme_choice"] != model["theme_choice"];
                    let projected = self.projection(&model, ctx);
                    let appearance = if self.framed
                        && self.session_appearance.is_none()
                        && (!self.loaded || self.model["appearance"] != model["appearance"])
                    {
                        crate::appearance::resolve(&model["appearance"]).map(Some)
                    } else {
                        Ok(None)
                    };
                    let result = appearance.as_ref().map_err(Clone::clone).and_then(|_| {
                        crate::appearance::theme_choice(&model["theme_choice"])?;
                        validate_overlays(&projected)?;
                        if self.loaded {
                            self.view.apply_model(projected)
                        } else {
                            (if self.framed {
                                crate::gold_frame::mount(&self.scene_source, projected)
                            } else {
                                SceneView::with_model(&self.scene_source, projected)
                            })
                            .map(|view| {
                                self.view = view;
                                self.loaded = true;
                            })
                        }
                    });
                    match result {
                        Ok(()) => {
                            if let Some(style) = appearance.expect("validated appearance") {
                                crate::appearance::install_style(ctx, &model["appearance"], style);
                            }
                            self.ready = true;
                            self.model = model;
                        }
                        Err(e) => self.error = e,
                    }
                }
            }
            if done.deferred && self.error.is_empty() {
                if let Some(kept) = self.retained.get(&done.sequence) {
                    kept.store(true, std::sync::atomic::Ordering::Release);
                }
            } else if let Some(reply) = self.pending.remove(&done.sequence) {
                self.retained.remove(&done.sequence);
                let answer = if !self.error.is_empty() {
                    (10, json!({"error":self.error}))
                } else if let Some(r) = done.reply {
                    (r.rc, r.body)
                } else {
                    (0, self.state())
                };
                let _ = reply.send(answer);
            }
            for (sequence, answer) in done.answers {
                self.retained.remove(&sequence);
                if let Some(reply) = self.pending.remove(&sequence) {
                    let _ = reply.send(if self.error.is_empty() {
                        (answer.rc, answer.body)
                    } else {
                        (10, json!({"error":self.error}))
                    });
                }
            }
            if !done.behaviour_available {
                self.retained.clear();
                for (_, reply) in std::mem::take(&mut self.pending) {
                    let _ = reply.send((20, json!({"error_code":"OWNER_CLOSED"})));
                }
            }
        }
        for _ in 0..32 {
            match self.bus.requests.try_recv() {
                Ok(r) => self.request(r, ctx),
                Err(_) => break,
            }
        }
        for _ in 0..32 {
            match self.bus.notices.try_recv() {
                Ok(n) => {
                    self.facilities.notice(&n.command, &n.args);
                    if self.drive.is_some() && n.command == "bus.connected" {
                        toolkit::drive::set_service(ctx, Some(&self.service));
                    }
                    if self.drive.is_some() && n.command == "bus.disconnected" {
                        toolkit::drive::set_service(ctx, None);
                    }
                    if n.command == "theme.changed" {
                        self.file_theme = toolkit::Theme::load();
                        self.session_dirty = true;
                    }
                    if let Some(session) = &mut self.session {
                        session.hint(&n.command, &n.args);
                    }
                    let _ = self
                        .runtime
                        .notify_with_headers(n.command, n.args, n.headers);
                }
                Err(_) => break,
            }
        }
        if let Some(projected) = self
            .output
            .as_ref()
            .and_then(|output| self.model["surfaces"].get(output))
            .or_else(|| self.output.as_ref().map(|_| &self.model))
        {
            if self.ready && !self.loaded {
                match SceneView::with_model(&self.scene_source, projected.clone()) {
                    Ok(view) => {
                        self.view = view;
                        self.loaded = true;
                    }
                    Err(e) => self.error = e,
                }
            } else if self.loaded && self.view.model() != projected {
                if let Err(e) = self.view.apply_model(projected.clone()) {
                    self.error = e;
                }
            }
        }
        if self.ready {
            if let Some(session) = &mut self.session {
                if let Some(snapshot) = session
                    .poll()
                    .or_else(|| self.session_dirty.then(|| session.snapshot.clone()))
                {
                    let appearance = snapshot["appearance"].clone();
                    match crate::appearance::install_session_choice(
                        ctx,
                        &snapshot,
                        self.drive.as_deref().unwrap_or(&self.service),
                        &self.file_theme,
                        &self.last_ui,
                        &self.model["theme_choice"],
                    ) {
                        Ok(common_ui) => {
                            use settings::view::View;
                            if snapshot["state"]["lost"] != true {
                                if let Ok(values) =
                                    serde_json::from_value::<settings::registry::Values>(
                                        snapshot["state"]["values"].clone(),
                                    )
                                {
                                    let keymap = settings::view::Keymaps::of(&values)
                                        .app(self.drive.as_deref().unwrap_or(&self.service));
                                    let _ = self
                                        .runtime
                                        .notify("viu.keymap.changed".into(), json!(keymap));
                                }
                            }
                            self.last_ui = common_ui;
                            self.session_dirty = false;
                            self.session_appearance = Some(appearance);
                            self.session_window = Some(snapshot["state"]["values"].clone());
                            self.session_notice = Some(snapshot.clone());
                            if !self.native_choice.as_object().is_some_and(|m| m.is_empty()) {
                                if let Ok(choice) = crate::native_theme::choice(&self.native_choice)
                                {
                                    if let Ok((base, _)) = crate::appearance::session_base(
                                        &snapshot,
                                        self.drive.as_deref().unwrap_or(&self.service),
                                        &self.file_theme,
                                        &self.last_ui,
                                    ) {
                                        if let Ok(theme) = base.with_choice(&choice) {
                                            let _ = crate::appearance::install_theme(ctx, &theme);
                                            let _ = self.runtime.notify(
                                                "viu.theme.changed".into(),
                                                crate::native_theme::describe(&theme),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        Err(error) => self.error = error,
                    }
                }
                if let Some(at) = session.deadline() {
                    ctx.request_repaint_after(at.saturating_duration_since(Instant::now()));
                }
            }
        }
        if let Some(snapshot) = self.session_notice.as_ref() {
            if self
                .runtime
                .notify("viu.session.changed".into(), snapshot.clone())
                .is_ok()
            {
                self.session_notice = None;
            }
        }
        if self.ready && self.framed {
            let projected = self.projection(&self.model, ctx);
            if self.view.model() != &projected {
                if let Err(error) = self.view.apply_model(projected) {
                    self.error = error;
                }
            }
        }
        self.settle(ctx);
    }
    fn settle(&mut self, ctx: &egui::Context) {
        if let Some(error) = self.view.take_focus_error() {
            self.error = error;
        }
        if self.input.now.is_empty()
            && self.input.later.is_empty()
            && self.inflight.is_empty()
            && self.view.current_revision_drawn()
        {
            if let Some(reply) = self.input_reply.take() {
                let _ = reply.send((if self.error.is_empty() { 0 } else { 10 }, self.state()));
            }
        } else if !self.input.now.is_empty() || !self.input.later.is_empty() {
            ctx.request_repaint();
        }
        if let Some((_, result)) = self.capture.poll(ctx) {
            if let Some(reply) = self.capture_reply.take() {
                let _ = reply.send(match result {
                    Ok(v) => (0, v),
                    Err(e) => (10, json!({"error":e})),
                });
            }
        }
        if let Some(tick) = self.tick {
            if self.ready && Instant::now() >= self.next_tick && self.inflight.is_empty() {
                self.dispatch("viu.tick".into(), json!({}), BTreeMap::new(), None);
                self.next_tick = Instant::now() + tick;
            }
            let wait = self.next_tick.saturating_duration_since(Instant::now());
            ctx.request_repaint_after(if wait.is_zero() { tick } else { wait });
        }
    }
    fn frame(&mut self, ui: &mut egui::Ui) {
        if self.ready {
            if self.framed {
                ui.painter().rect_filled(
                    ui.max_rect(),
                    0.0,
                    crate::appearance::colour(ui.ctx(), "dock").unwrap_or(ui.visuals().panel_fill),
                );
                let settings = self.window_settings(&self.model);
                if let Some(title) = self.model["window_title"].as_str() {
                    ui.ctx()
                        .send_viewport_cmd(egui::ViewportCommand::Title(title.into()));
                }
                crate::rich::configure_menu(ui.ctx(), &settings, None);
                self.window_frame
                    .show_with_chrome(ui, &self.title, &settings, true, true);
                let slots = self
                    .window_frame
                    .geometry
                    .get("window.header")
                    .map(|rect| {
                        let key = if self.view.has_slot("window.header.content") {
                            "window.header.content"
                        } else {
                            "window.header"
                        };
                        let rect = self.window_frame.geometry.get(key).unwrap_or(rect);
                        BTreeMap::from([(key.into(), *rect)])
                    })
                    .unwrap_or_default();
                if let Err(e) = self.view.set_slots(slots) {
                    self.error = e;
                }
            }
            for event in self.view.show(ui) {
                self.dispatch(event.command, event.body, BTreeMap::new(), None);
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
                    .expect("validated overlay");
                let command = overlay["command"].as_str().unwrap();
                for intent in self.overlays.widget(ui, &widget, &self.model) {
                    self.dispatch(command.into(), intent.encode(), BTreeMap::new(), None);
                }
            }
            self.view.geometry.extend(self.overlays.geometry.clone());
            if self.model["keyboard_events"] == true {
                let keys=ui.input(|input|input.events.iter().filter_map(|event|match event{egui::Event::Key{key,pressed:true,repeat:false,modifiers,..}=>Some(json!({"key":crate::input::key_name(*key),"ctrl":modifiers.ctrl||modifiers.command,"alt":modifiers.alt,"shift":modifiers.shift,"typing":self.view.text_input_active()})),_=>None}).collect::<Vec<_>>());
                for key in keys {
                    self.dispatch("viu.key".into(), key, BTreeMap::new(), None);
                }
            }
            if self.facilities.macros.is_some()
                && toolkit::macros::show_app(
                    ui.ctx(),
                    self.drive.as_deref().unwrap_or(&self.service),
                    self.model["macros_open"] == true,
                )
            {
                self.dispatch(
                    "viu.macros.changed".into(),
                    json!({"open":false}),
                    BTreeMap::new(),
                    None,
                );
            }
            if self.model["native_focus"]["serial"] != self.focus_serial
                && self.model["native_focus"]["id"].is_string()
            {
                if let Some(id) = self.model["native_focus"]["id"].as_str() {
                    let _ = self.view.request_focus(id);
                }
                self.focus_serial = self.model["native_focus"]["serial"].clone();
                ui.ctx().request_repaint();
            }
            if !toolkit::menu::is_open(ui.ctx()) {
                for command in crate::shortcuts::pressed(ui.ctx(), &self.model["shortcuts"]) {
                    self.dispatch(
                        "viu.frame.intent".into(),
                        json!({"id":"shortcut","value":{"command":command,"origin":"shortcut"}}),
                        BTreeMap::new(),
                        None,
                    );
                }
            }
            if self.framed {
                let reserved = self.view.allocated_region("gold-captions");
                self.window_frame.finish_captions(ui, reserved);
                self.window_frame.edges(ui);
                self.view
                    .geometry
                    .extend(self.window_frame.geometry.clone());
                if self.window_frame.close_requested {
                    self.dispatch("viu.frame.close".into(), json!({}), BTreeMap::new(), None);
                }
                if self.model["window_close"] == true
                    && self.wire_deliveries.is_empty()
                    && self.drive_pending.is_empty()
                {
                    self.facilities.finish(ui.ctx());
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
        self.settle(ui.ctx());
        if !self.error.is_empty() {
            ui.colored_label(egui::Color32::RED, &self.error);
        }
    }
}

fn create(bundle: Bundle, wake: crate::runtime::Wake) -> Host {
    let service = bundle.manifest.service.clone();
    let facilities = crate::app_facilities::Facilities::new(&bundle.manifest, wake.clone());
    let mut commands = bundle.manifest.commands.clone();
    if let Some(surface) = &facilities.surface {
        commands.extend(
            surface["commands"]
                .as_array()
                .unwrap()
                .iter()
                .filter_map(|c| c["name"].as_str().map(str::to_owned)),
        );
    }
    commands.extend(
        [
            "viu.state",
            "viu.ui.tree",
            "viu.ui.click",
            "viu.ui.focus",
            "viu.ui.text",
            "viu.ui.key",
            "viu.ui.hover",
            "viu.ui.scroll",
            "viu.ui.drag",
            "viu.capture",
            "viu.quit",
            "viu.clipboard.image",
        ]
        .map(String::from),
    );
    let declared = commands.iter().cloned().collect();
    let bus = Endpoint::start_described(
        service,
        bundle.manifest.url.clone().unwrap_or_else(|| {
            std::env::var("MIXOS_NODED_URL").unwrap_or_else(|_| "ws://127.0.0.1:4200/ws".into())
        }),
        commands
            .into_iter()
            .map(|name| bus::CommandDescriptor::new(&name, &[], "Mix bundle command", false))
            .collect(),
        "viu.quit".into(),
        wake.clone(),
        facilities.surface.as_ref().map(|s| s["commands"].clone()),
    );
    let mut jobs = crate::jobs::Jobs::new(4, wake.clone());
    let local_ui = crate::local_ui::LocalUi::new(wake.clone());
    jobs.set_native_backend(bus.backend_maker());
    let (clipboard, local_clipboard) = crate::clipboard::Clipboard::channel(wake.clone());
    let macros = facilities.macros.clone().unwrap_or_else(|| {
        macros::Session::for_app(
            bundle
                .manifest
                .drive
                .as_deref()
                .unwrap_or(&bundle.manifest.service),
        )
    });
    let services = crate::runtime::Services {
        clipboard: Some(clipboard),
        options: json!({"headless":bundle.manifest.surface=="headless"}),
        macros: Some(macros.clone()),
        ..Default::default()
    };
    jobs.set_services(services.clone());
    let mut runtime = Runtime::with_services(
        32,
        wake.clone(),
        Some(jobs.backend(Some(local_ui.backend(bus.backend())))),
        services,
    );
    runtime.load(bundle.program).expect("load behaviour");
    let capture_root = std::env::var_os("XDG_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("viu-captures");
    Host {
        headless: bundle.manifest.surface == "headless",
        drive: bundle.manifest.drive.clone(),
        drive_pending: BTreeMap::new(),
        drive_sequence: 0,
        wire_deliveries: Vec::new(),
        local_ui,
        native_choice: json!({}),
        overlays: Default::default(),
        focus_serial: Value::Null,
        facilities,
        service: bundle.manifest.service.clone(),
        file_theme: toolkit::Theme::load(),
        last_ui: settings::model::CommonUi::default(),
        session_dirty: false,
        session_notice: None,
        session: bundle.manifest.frame.as_ref().map(|_| {
            crate::session::Session::embedded(
                bundle
                    .manifest
                    .drive
                    .as_deref()
                    .unwrap_or(&bundle.manifest.service),
                &std::env::var("MIXOS_INSTANCE").unwrap_or_else(|_| "default".into()),
                wake.clone(),
                bus.backend(),
            )
            .expect("embedded session worker")
        }),
        session_appearance: None,
        session_window: None,
        framed: bundle.manifest.frame.is_some(),
        title: bundle.manifest.title.clone(),
        window_frame: crate::window_frame::WindowFrame::default(),
        clipboard: crate::thumbnail::Decoder::with_decoder(
            "viu-clipboard-decode",
            crate::image_view::decode_document,
        ),
        clipboard_reply: None,
        local_clipboard,
        macros,
        jobs,
        job_results: std::collections::VecDeque::new(),
        commands: declared,
        scene_source: bundle.scene,
        loaded: false,
        model: json!({}),
        output: None,
        outputs: BTreeMap::new(),
        runtime,
        view: loading_view(),
        bus,
        pending: BTreeMap::new(),
        retained: BTreeMap::new(),
        inflight: BTreeSet::new(),
        input: Input::default(),
        input_reply: None,
        capture: crate::capture::Capture::new(capture_root, wake),
        capture_reply: None,
        ready: false,
        error: String::new(),
        tick: bundle.manifest.tick_ms.map(Duration::from_millis),
        next_tick: Instant::now(),
    }
}

pub fn run(args: Vec<String>) -> eframe::Result {
    if args.first().map(String::as_str) != Some("run") || args.len() < 2 {
        eprintln!("usage: viu run APP_DIR [ARGS]");
        std::process::exit(2);
    }
    let bundle = Bundle::load(std::path::Path::new(&args[1]), &args[2..]).unwrap_or_else(|e| {
        eprintln!("viu: {e}");
        std::process::exit(2)
    });
    let mut logging_args: Vec<String> = std::env::args().collect();
    let _logging = toolkit::debug::init(&bundle.manifest.service, "warn", &mut logging_args)
        .unwrap_or_else(|error| {
            eprintln!("viu logging: {error}");
            std::process::exit(2)
        });
    if bundle.manifest.surface == "headless" {
        let ctx = egui::Context::default();
        crate::appearance::install(&ctx);
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        let wake: crate::runtime::Wake = Arc::new(move || {
            let _ = tx.try_send(());
        });
        let repaint = wake.clone();
        ctx.set_request_repaint_callback(move |_| repaint());
        let size = bundle.manifest.size;
        let mut host = create(bundle, wake);
        loop {
            host.pump(&ctx);
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(size[0], size[1]),
                    )),
                    ..Default::default()
                },
                |ui| host.frame(ui),
            )
            .drop_without_applying_deltas();
            if host
                .bus
                .close_requested
                .load(std::sync::atomic::Ordering::Acquire)
                || host.model["window_close"] == true
                    && host.wire_deliveries.is_empty()
                    && host.pending.is_empty()
            {
                break;
            }
            let _ = rx.recv_timeout(Duration::from_millis(100));
        }
        return Ok(());
    }
    if bundle.manifest.surface == "background" {
        let mut options = crate::layer::LayerOptions::background(&bundle.manifest.service);
        options.exclusive_zone = 0;
        let layer = LayerHost {
            bundle: Some(bundle),
            host: None,
            views: BTreeMap::new(),
            contexts: Arc::new(std::sync::Mutex::new(Vec::new())),
        };
        if let Err(e) = crate::layer::run(options, layer) {
            eprintln!("viu layer: {e}");
            std::process::exit(1);
        }
        return Ok(());
    }
    if let Some(command) = &bundle.manifest.activation {
        match crate::native_bus::activate_existing(
            &bundle.manifest.service,
            command,
            &std::env::var("MIXOS_NODED_URL").unwrap_or_else(|_| "ws://127.0.0.1:4200/ws".into()),
        ) {
            Ok(true) => return Ok(()),
            Ok(false) => {}
            Err(error) => {
                eprintln!("viu activation: {error}");
                std::process::exit(1);
            }
        }
    }
    let title = bundle.manifest.title.clone();
    let size = bundle.manifest.size;
    let service = bundle.manifest.service.clone();
    let app_id = bundle
        .manifest
        .app_id
        .clone()
        .or_else(|| {
            bundle
                .manifest
                .bus_surface
                .as_ref()
                .and_then(|s| s["app_id"].as_str())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| format!("mixviu.{service}"));
    eframe::run_native(
        &title,
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_transparent(bundle.manifest.frame.is_some())
                .with_decorations(bundle.manifest.frame.is_none())
                .with_inner_size(size)
                .with_app_id(app_id),
            ..Default::default()
        },
        Box::new(move |cc| {
            toolkit::decorations::initialise(cc);
            toolkit::decorations::initialise_window(&cc.egui_ctx, cc.winit_window().cloned());
            crate::appearance::install(&cc.egui_ctx);
            if let Some(prefix) = &bundle.manifest.drive {
                toolkit::drive::install(&cc.egui_ctx, prefix);
            }
            let ctx = cc.egui_ctx.clone();
            Ok(Box::new(create(
                bundle,
                Arc::new(move || ctx.request_repaint()),
            )))
        }),
    )
}
struct LayerHost {
    bundle: Option<Bundle>,
    host: Option<Host>,
    views: BTreeMap<String, (SceneView, bool)>,
    contexts: Arc<std::sync::Mutex<Vec<egui::Context>>>,
}
impl crate::layer::LayerProgram for LayerHost {
    fn should_close(&self) -> bool {
        self.host.as_ref().is_some_and(|h| {
            h.bus
                .close_requested
                .load(std::sync::atomic::Ordering::Acquire)
        })
    }
    fn setup(&mut self, ctx: &egui::Context) {
        crate::icons::install(ctx);
        self.contexts.lock().unwrap().push(ctx.clone());
        if let Some(bundle) = self.bundle.take() {
            let contexts = self.contexts.clone();
            self.host = Some(create(
                bundle,
                Arc::new(move || {
                    for ctx in contexts.lock().unwrap().iter() {
                        ctx.request_repaint();
                    }
                }),
            ));
        }
    }
    fn show(&mut self, ui: &mut egui::Ui, surface: &crate::layer::SurfaceInfo) {
        let host = self.host.as_mut().unwrap();
        let output = surface.output.clone().unwrap_or_default();
        let frames = host
            .outputs
            .get(&output)
            .and_then(|o| o["presented_frames"].as_u64())
            .unwrap_or(0);
        host.outputs.insert(output.clone(),json!({"width":surface.width,"height":surface.height,"scale":surface.scale,"presented_frames":frames}));
        let (view, loaded) = self
            .views
            .entry(output.clone())
            .or_insert_with(|| (loading_view(), false));
        std::mem::swap(&mut host.view, view);
        std::mem::swap(&mut host.loaded, loaded);
        host.output = Some(output);
        // Layer hosts use the same owner loop; eframe's Frame is not needed.
        host.pump(ui.ctx());
        host.frame(ui);
        std::mem::swap(&mut host.view, view);
        std::mem::swap(&mut host.loaded, loaded);
    }
    fn removed(&mut self, ctx: &egui::Context, surface: &crate::layer::SurfaceInfo) {
        let output = surface.output.clone().unwrap_or_default();
        self.views.remove(&output);
        if let Some(host) = &mut self.host {
            host.outputs.remove(&output);
        }
        self.contexts.lock().unwrap().retain(|c| c != ctx);
    }
    fn presented(&mut self, surface: &crate::layer::SurfaceInfo) {
        let output = surface.output.clone().unwrap_or_default();
        if let Some(host) = &mut self.host {
            if let Some(state) = host.outputs.get_mut(&output) {
                state["presented_frames"] =
                    json!(state["presented_frames"].as_u64().unwrap_or(0) + 1);
                state["presented_revision"] =
                    json!(self.views.get(&output).map(|(v, _)| v.revision));
            }
        }
    }
}
fn loading_view() -> SceneView {
    SceneView::new("---\nscene: 1\nname: loading\ncitizen: loading\n---\n```mix\n{root:{widget:\"text\",text:\"Loading…\"}}\n```\n").expect("static loading scene")
}
