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
enum SelectionWait {
    Platform(String),
    Osc52(crate::terminal::sessions::PaneClipboardReply),
}
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
    platform: crate::platform::Platform,
    selection: crate::terminal::clipboard::Clipboard,
    selection_sequence: u64,
    selection_waits: BTreeMap<u64, SelectionWait>,
    platform_results: std::collections::VecDeque<Value>,
    calls: crate::calls::Calls,
    timers: crate::timers::Timers,
    timer_results: std::collections::VecDeque<Value>,
    documents: crate::documents::Documents,
    terminals: crate::terminal::sessions::Sessions,
    ui_state: crate::ui_state::UiState,
    measurements: crate::ui_state::Measurements,
    input_at: Option<Instant>,
    native_drive: bool,
    preserve_request_body: bool,
    service: String,
    settings_reload: u64,
    session_update: Option<Value>,
    local_ui: crate::local_ui::LocalUi,
    native_choice: Value,
    overlays: crate::widgets::Widgets,
    focus_serial: Value,
    facilities: crate::app_facilities::Facilities,
    session: Option<crate::session::Session>,
    session_appearance: Option<Value>,
    session_window: Option<Value>,
    app: String,
    file_theme: toolkit::Theme,
    last_ui: settings::model::CommonUi,
    session_dirty: bool,
    framed: bool,
    title: String,
    unknown_command: Option<String>,
    unknown_reply: Option<Value>,
    unknown_handler: Option<String>,
    window_frame: crate::window_frame::WindowFrame,
    jobs: crate::jobs::Jobs,
    job_results: std::collections::VecDeque<Value>,
    clipboard: crate::thumbnail::Decoder<String>,
    clipboard_reply: Option<Reply>,
    commands: BTreeSet<String>,
    scene_source: String,
    loaded: bool,
    model: Value,
    output: Option<String>,
    outputs: BTreeMap<String, Value>,
    runtime: Runtime,
    view: SceneView,
    bus: Endpoint,
    pending: BTreeMap<u64, String>,
    held_requests: std::collections::VecDeque<Request>,
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
        let mut settings = if self.native_drive && model["window_settings"].is_object() {
            model["window_settings"].clone()
        } else {
            self.session_window
                .clone()
                .unwrap_or_else(|| model["window_settings"].clone())
        };
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
        json!({"ready":self.ready,"model":self.view.model(),"revision":self.view.revision,"geometry":geometry,"outputs":self.outputs,"stats":self.ui_state.stats(),"error":self.error,"session":self.session.as_ref().map(|session| &session.snapshot)})
    }
    fn dispatch(
        &mut self,
        command: String,
        body: Value,
        headers: BTreeMap<String, String>,
        reply: Option<Reply>,
    ) {
        self.dispatch_body(command, body, None, headers, reply);
    }
    fn dispatch_body(
        &mut self,
        command: String,
        body: Value,
        raw: Option<String>,
        mut headers: BTreeMap<String, String>,
        reply: Option<Reply>,
    ) {
        // Only the host supplies this receipt. A Bus caller cannot forge it.
        let token = if let Some(reply) = reply {
            match self
                .calls
                .begin(reply, Arc::new(std::sync::atomic::AtomicBool::new(false)))
            {
                Ok(token) => Some(token),
                Err(reply) => {
                    let _ = reply.send((10, json!({"error_code":"BUSY"})));
                    return;
                }
            }
        } else {
            headers.get("viu-call").cloned()
        };
        if let Some(token) = &token {
            headers.insert("viu-call".into(), token.clone());
        }
        let submitted = match raw {
            Some(raw) => self.runtime.event_with_body(command, raw, headers),
            None => self.runtime.event_with_headers(command, body, headers),
        };
        match submitted {
            Ok(seq) => {
                self.inflight.insert(seq);
                if let Some(token) = token {
                    self.pending.insert(seq, token);
                }
            }
            Err(e) => {
                if let Some(token) = token {
                    self.calls.finish(
                        &token,
                        10,
                        json!({"error_code":"BUSY","error":format!("{e:?}")}),
                        false,
                    );
                }
            }
        }
    }
    fn request(&mut self, mut r: Request, ctx: &egui::Context) {
        let success_body = if self.native_drive {
            crate::window_frame::root_close_reply(ctx, &self.app, &r.command, &r.args)
        } else {
            None
        };
        if success_body.is_some() {
            r.command = "app.quit".into();
        }
        let Some(mut r) = self.facilities.request(r, ctx) else {
            return;
        };
        if r.command == format!("{}.quit", self.app)
            || (r.command == format!("{}.execute", self.app) && r.args["id"] == "file.quit")
        {
            r.close_after_reply
                .store(true, std::sync::atomic::Ordering::Release);
        }
        if !self.commands.contains(&r.command) {
            if let Some(handler) = &self.unknown_handler {
                // Application-owned refusal policy. The original canonical
                // command is host supplied, never a caller-selected header.
                r.headers
                    .insert("viu-unknown-command".into(), r.command.clone());
                r.command = handler.clone();
            } else {
                let _ = r.reply.send((
                    10,
                    self.unknown_reply.clone().unwrap_or_else(|| {
                        self.unknown_command.as_ref().map_or_else(
                            || json!({"error_code":"UNKNOWN_COMMAND","command":r.command}),
                            |message| json!({"error_code":"UNKNOWN_COMMAND","message":message}),
                        )
                    }),
                ));
                return;
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
            _ => match self.calls.begin_with_success_body(r.reply, r.close_after_reply, success_body) {
                Ok(token) => {
                    let mut headers = r.headers;
                    headers.insert("viu-call".into(), token);
                    let raw = self.preserve_request_body.then_some(r.body);
                    self.dispatch_body(r.command, r.args, raw, headers, None);
                }
                Err(reply) => {
                    let _ = reply.send((10, json!({"error_code":"BUSY"})));
                }
            },
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
        let keys = raw
            .events
            .iter()
            .filter(|e| matches!(e, egui::Event::Key { pressed: true, .. }))
            .count();
        self.measurements.keys += keys as u64;
        if !raw.events.is_empty() {
            self.input_at = Some(Instant::now());
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        self.frame(ui);
    }
}
impl Host {
    fn pump(&mut self, ctx: &egui::Context) {
        if self.ready {
            while let Some(job) = self.terminals.layout_bridge().take() {
                if self
                    .runtime
                    .native_event("viu.terminal.control".into(), json!({}), job.context())
                    .is_err()
                {
                    job.complete(crate::terminal::protocol::Reply::error("RESOURCE_LIMIT"));
                }
            }
            let handles = self.terminals.take_changes();
            if !handles.is_empty()
                && self
                    .runtime
                    .notify("viu.terminal.changed".into(), json!({"handles":handles}))
                    .is_err()
            {
                self.terminals.retry_changes(handles);
            }
        }
        self.facilities.prepare(ctx, &self.bus);
        for request in self.platform.take() {
            if request["kind"] == "copy" {
                self.selection.write(
                    ctx,
                    request["body"]["text"].as_str().unwrap_or("").into(),
                    request["body"]["primary"] == true,
                );
            } else {
                self.selection_sequence = self
                    .selection_sequence
                    .checked_add(1)
                    .expect("selection id exhausted");
                self.selection_waits.insert(
                    self.selection_sequence,
                    SelectionWait::Platform(request["id"].as_str().unwrap_or("").into()),
                );
                self.selection
                    .read(self.selection_sequence, request["body"]["primary"] == true);
            }
        }
        for request in self.terminals.take_clipboard_requests() {
            use crate::terminal::sessions::WindowClipboardRequest;
            match request {
                WindowClipboardRequest::Write { primary, text } => {
                    self.selection.write(ctx, text, primary)
                }
                WindowClipboardRequest::Read { primary, reply } => {
                    self.selection_sequence = self
                        .selection_sequence
                        .checked_add(1)
                        .expect("selection id exhausted");
                    self.selection_waits
                        .insert(self.selection_sequence, SelectionWait::Osc52(reply));
                    self.selection.read(self.selection_sequence, primary);
                }
            }
        }
        for (sequence, result) in self.selection.take() {
            if let Some(owner) = self.selection_waits.remove(&sequence) {
                match owner {
                    SelectionWait::Platform(id) => self.platform_results.push_back(match result {
                        Ok(text) => json!({"id":id,"text":text}),
                        Err(reason) => json!({"id":id,"text":"","reason":format!("{reason:?}")}),
                    }),
                    SelectionWait::Osc52(reply) => match result {
                        Ok(text) => {
                            if let Err(error) = reply.answer(&text) {
                                eprintln!("viu: OSC 52 reply: {error}");
                            }
                        }
                        Err(reason) => eprintln!("viu: OSC 52 read refused: {reason:?}"),
                    },
                }
            } else if sequence == crate::terminal::WRITE_ANSWER
                && let Err(reason) = result
            {
                self.platform_results
                    .push_back(json!({"kind":"copy","reason":format!("{reason:?}")}));
            }
        }
        while let Some(result) = self.platform_results.front() {
            if self
                .runtime
                .notify("viu.platform.completed".into(), result.clone())
                .is_err()
            {
                break;
            }
            self.platform_results.pop_front();
        }
        for token in self.calls.cancelled() {
            let _ = self
                .runtime
                .notify("viu.call.cancelled".into(), json!({"token":token}));
        }
        let (due, deadline) = self.timers.poll();
        self.timer_results.extend(due);
        if let Some(at) = deadline {
            ctx.request_repaint_after(at.saturating_duration_since(Instant::now()));
        }
        while let Some(body) = self.timer_results.front() {
            if self
                .runtime
                .notify("viu.timer".into(), body.clone())
                .is_err()
            {
                ctx.request_repaint();
                break;
            }
            self.timer_results.pop_front();
        }

        while let Ok(request) = self.local_ui.requests.try_recv() {
            let answer = match request.command.as_str() {
                "native.executed" => self.facilities.record(&request.args),
                "native.bindings" => crate::shortcuts::resolve(&request.args),
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
                                &self.app,
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
            self.measurements
                .model(done.elapsed.as_micros().min(u64::MAX as u128) as u64);
            self.inflight.remove(&done.sequence);
            self.error = done.error.unwrap_or_default();
            if self.error.is_empty() {
                if let Some(model) = done.model {
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
                                self.view.set_documents(self.documents.clone());
                                self.view.set_terminals(self.terminals.clone());
                                self.view.set_native_menus(self.native_drive);
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
            if let Some(reply) = self.pending.remove(&done.sequence) {
                let answer = if !self.error.is_empty() {
                    (10, json!({"error":self.error}))
                } else if let Some(r) = done.reply {
                    (r.rc, r.body)
                } else {
                    (0, self.state())
                };
                self.calls.complete(&reply, answer.0, answer.1);
            }
        }
        for _ in self.held_requests.len()..32 {
            match self.bus.requests.try_recv() {
                Ok(r) => self.held_requests.push_back(r),
                Err(_) => break,
            }
        }
        while let Some(request) = self.held_requests.front() {
            let queues = request
                .command
                .strip_prefix(&format!("{}.", self.app))
                .is_some_and(toolkit::drive::queues);
            if toolkit::drive::pending(ctx) && !queues {
                break;
            }
            let request = self.held_requests.pop_front().unwrap();
            self.request(request, ctx);
        }
        for _ in 0..32 {
            match self.bus.notices.try_recv() {
                Ok(n) => {
                    self.facilities.notice(&n.command, &n.args);
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
                    match crate::appearance::install_session(
                        ctx,
                        &snapshot,
                        &self.app,
                        &self.file_theme,
                        &self.last_ui,
                    ) {
                        Ok(common_ui) => {
                            use settings::view::View;
                            if snapshot["state"]["lost"] != true {
                                if let Ok(values) =
                                    serde_json::from_value::<settings::registry::Values>(
                                        snapshot["state"]["values"].clone(),
                                    )
                                {
                                    let keymap =
                                        settings::view::Keymaps::of(&values).app(&self.app);
                                    let _ = self
                                        .runtime
                                        .notify("viu.keymap.changed".into(), json!(keymap));
                                }
                            }
                            self.last_ui = common_ui;
                            let mut update = snapshot.clone();
                            update["appearance"]["mode"] =
                                json!(if ctx.global_style().visuals.dark_mode {
                                    "dark"
                                } else {
                                    "light"
                                });
                            self.session_update = Some(update);
                            self.session_dirty = false;
                            self.session_appearance = Some(appearance);
                            self.session_window = Some(snapshot["state"]["values"].clone());
                            if let Ok(choice) = crate::native_theme::choice(&self.native_choice) {
                                if let Ok((base, _)) = crate::appearance::session_base(
                                    &snapshot,
                                    &self.app,
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
                        Err(error) => self.error = error,
                    }
                }
                if let Some(at) = session.deadline() {
                    ctx.request_repaint_after(at.saturating_duration_since(Instant::now()));
                }
            }
        }
        if let Some(update) = &self.session_update {
            if self
                .runtime
                .notify("viu.session".into(), update.clone())
                .is_ok()
            {
                self.session_update = None;
            } else {
                ctx.request_repaint();
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
                    crate::window_frame::set_title(ui.ctx(), title);
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
            let started = Instant::now();
            for event in self.view.show(ui) {
                self.dispatch(event.command, event.body, BTreeMap::new(), None);
            }
            self.measurements.view(started.elapsed().as_micros() as u64);
            if let Some(at) = self.input_at.take() {
                self.measurements.next(at.elapsed().as_micros() as u64);
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
            if !self.native_drive
                && self.facilities.macros.is_some()
                && toolkit::macros::show_app(ui.ctx(), &self.app, self.model["macros_open"] == true)
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
            if self.native_drive {
                if self.model["local_theme"].is_object() {
                    if let Err(error) =
                        crate::appearance::local_choice(ui.ctx(), &self.model["local_theme"])
                    {
                        self.error = error;
                    }
                } else if let Some(mode) = self.model["local_mode"].as_str() {
                    if let Err(error) = crate::appearance::local_mode(ui.ctx(), mode) {
                        self.error = error;
                    }
                }
                let reload = self.model["settings_reload"].as_u64().unwrap_or(0);
                if reload != self.settings_reload {
                    self.settings_reload = reload;
                    self.file_theme = toolkit::Theme::load();
                    if let Some(session) = &mut self.session {
                        session.hint("theme.changed", &json!({}));
                    }
                    ui.ctx().request_repaint();
                }
                if let Some(bindings) = self.model["shortcuts"]
                    .as_array()
                    .filter(|_| !toolkit::menu::is_open(ui.ctx()))
                {
                    let bindings = bindings.clone();
                    for binding in bindings {
                        let editor_focus = ui
                            .ctx()
                            .data(|d| d.get_temp::<bool>(egui::Id::unique("viu-editor-focused")))
                            .unwrap_or(false);
                        let focused = ui
                            .ctx()
                            .data(|d| d.get_temp::<String>(egui::Id::unique("viu-focused-widget")));
                        let allowed = binding["text_scopes"].as_array().is_some_and(|a| {
                            focused
                                .as_deref()
                                .is_some_and(|id| a.iter().any(|v| v == id))
                        });
                        if ui.ctx().memory(|m| m.focused().is_some()) && !editor_focus && !allowed {
                            continue;
                        }
                        if binding["enabled"] == false {
                            continue;
                        }
                        if let (Some(chord), Some(id), Some(command)) = (
                            binding["chord"].as_str(),
                            binding["id"].as_str(),
                            self.model["on_shortcut"].as_str(),
                        ) {
                            if crate::input::consume_chord(ui.ctx(), chord) {
                                self.dispatch(
                                    command.into(),
                                    json!({"id":id}),
                                    BTreeMap::new(),
                                    None,
                                );
                            }
                        }
                    }
                }
            } else {
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
                if self.model["window_close"] == true {
                    self.facilities.finish(ui.ctx());
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
        let mut stats = self.measurements.snapshot();
        stats["repaint_causes"] = json!(
            ui.ctx()
                .repaint_causes()
                .into_iter()
                .take(16)
                .map(|cause| json!({"file":cause.file,"line":cause.line,"reason":cause.reason}))
                .collect::<Vec<_>>()
        );
        self.ui_state
            .set(ui.ctx().content_rect(), &self.view.geometry, stats);
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
    if bundle.manifest.native_drive {
        commands.extend(
            toolkit::drive::COMMANDS
                .iter()
                .map(|suffix| format!("{service}.{suffix}")),
        );
        commands.push(format!("{service}.debug"));
        commands.extend(["set", "recent"].map(|suffix| format!("{service}.debug.{suffix}")));
    }
    commands.extend(
        [
            "viu.state",
            "viu.ui.tree",
            "viu.ui.click",
            "viu.ui.focus",
            "viu.ui.text",
            "viu.ui.key",
            "viu.ui.clipboard",
            "viu.ui.ime",
            "viu.ui.hover",
            "viu.ui.scroll",
            "viu.ui.drag",
            "viu.capture",
            "viu.quit",
            "viu.clipboard.image",
        ]
        .map(String::from),
    );
    let mut command_names = BTreeSet::new();
    commands.retain(|name| command_names.insert(name.clone()));
    let declared = commands.iter().cloned().collect();
    let bus = Endpoint::start_described_policy(
        service,
        bundle.manifest.bus_url(),
        {
            let mut described = bundle.manifest.command_descriptors.clone();
            let mut native = Vec::new();
            if bundle.manifest.native_drive {
                native.extend(toolkit::drive::describe(&bundle.manifest.service));
            }
            if bundle.manifest.macros {
                native.extend(macros::commands::describe(&bundle.manifest.service));
            }
            for value in native {
                described.push(bus::CommandDescriptor {
                    name: value["name"].as_str().unwrap().into(),
                    args: value["args"]
                        .as_object()
                        .map(|a| a.keys().cloned().collect())
                        .unwrap_or_default(),
                    description: value["description"].as_str().unwrap_or("").into(),
                    read_only: value["read_only"] == true,
                });
            }
            commands
                .into_iter()
                .map(|name| {
                    described
                        .iter()
                        .find(|d| d.name == name)
                        .cloned()
                        .unwrap_or_else(|| {
                            bus::CommandDescriptor::new(&name, &[], "Mix bundle command", false)
                        })
                })
                .collect()
        },
        "viu.quit".into(),
        wake.clone(),
        facilities
            .surface
            .as_ref()
            .map(|s| s["commands"].clone())
            .or_else(|| bundle.manifest.authored_discovery.then(|| json!([]))),
        crate::native_bus::Policy {
            pid_fallback: bundle.manifest.registration_pid_fallback,
            namespace: bundle.manifest.command_namespace.clone(),
            text_replies: bundle.manifest.text_replies,
            unknown_reply: bundle.manifest.unknown_reply.clone(),
        },
    );
    let documents = crate::documents::Documents::default();
    let terminals = crate::terminal::sessions::Sessions::with_native(wake.clone());
    let timers = crate::timers::Timers::default();
    let calls = crate::calls::Calls::default();
    let ui_state = crate::ui_state::UiState::default();
    let platform = crate::platform::Platform::default();
    let services = crate::runtime::Services {
        max_string_bytes: bundle.manifest.max_string_bytes,
        platform: Some(platform.clone()),
        ui_state: Some(ui_state.clone()),
        macros: facilities.macros.clone(),
        calls: Some(calls.clone()),
        timers: Some(timers.clone()),
        documents: Some(documents.clone()),
        terminals: Some(terminals.clone()),
        ..Default::default()
    };
    let mut jobs = crate::jobs::Jobs::new(4, wake.clone());
    let local_ui = crate::local_ui::LocalUi::new(wake.clone());
    jobs.set_native_backend(bus.backend_maker());
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
        platform,
        selection: crate::terminal::clipboard::Clipboard::none(),
        selection_sequence: 0,
        selection_waits: BTreeMap::new(),
        platform_results: Default::default(),
        calls,
        timers,
        timer_results: Default::default(),
        documents,
        terminals,
        ui_state,
        measurements: Default::default(),
        input_at: None,
        native_drive: bundle.manifest.native_drive,
        preserve_request_body: bundle.manifest.preserve_request_body,
        service: bundle.manifest.service.clone(),
        settings_reload: 0,
        session_update: None,
        local_ui,
        native_choice: json!({}),
        overlays: Default::default(),
        focus_serial: Value::Null,
        facilities,
        app: bundle.manifest.app().to_owned(),
        file_theme: toolkit::Theme::load(),
        last_ui: settings::model::CommonUi::default(),
        session_dirty: false,
        session: bundle.manifest.frame.as_ref().map(|_| {
            crate::session::Session::embedded(
                bundle.manifest.app(),
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
        unknown_command: bundle.manifest.unknown_command.clone(),
        unknown_reply: bundle.manifest.unknown_reply.clone(),
        unknown_handler: bundle.manifest.unknown_handler.clone(),
        window_frame: crate::window_frame::WindowFrame::default(),
        clipboard: crate::thumbnail::Decoder::with_decoder(
            "viu-clipboard-decode",
            crate::image_view::decode_surface,
        ),
        clipboard_reply: None,
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
        held_requests: std::collections::VecDeque::new(),
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
    let root = std::path::Path::new(&args[1]);
    let preview = Bundle::load(root, &[]).unwrap_or_else(|e| {
        eprintln!("viu: {e}");
        std::process::exit(2)
    });
    let mut logging_args = std::iter::once("viu".to_owned())
        .chain(args.iter().cloned())
        .collect::<Vec<_>>();
    let _logging = toolkit::debug::init(preview.manifest.app(), "warn", &mut logging_args)
        .unwrap_or_else(|e| {
            eprintln!("viu logging: {e}");
            std::process::exit(2)
        });
    if matches!(
        logging_args.get(3).map(String::as_str),
        Some("--help" | "-h")
    ) {
        println!(
            "Usage: viu run APP_DIR [--service NAME] [--comp NAME] [--noded-url URL] [APP_ARGS]"
        );
        println!("{}", toolkit::debug::HELP);
        return Ok(());
    }
    if logging_args.get(3).map(String::as_str) == Some("--version") {
        println!(
            "{} {} (generic viu)",
            preview.manifest.app(),
            env!("CARGO_PKG_VERSION")
        );
        return Ok(());
    }
    let bundle = crate::launch::load(root, &logging_args[3..]).unwrap_or_else(|e| {
        eprintln!("viu: {e}");
        std::process::exit(2)
    });
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
            &bundle.manifest.bus_url(),
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
    let app_id = crate::launch::app_id(&bundle);
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
            if let Some(state) = cc.wgpu_render_state.as_ref() {
                state
                    .renderer
                    .write()
                    .callback_resources
                    .insert(state.target_format);
            }
            let ctx = cc.egui_ctx.clone();
            let wake: crate::runtime::Wake = Arc::new(move || ctx.request_repaint());
            let mut host = create(bundle, wake.clone());
            use raw_window_handle::HasDisplayHandle;
            // SAFETY: eframe keeps its display alive until after Host drops,
            // matching the pinned gold window/selection-worker ownership.
            host.selection = unsafe {
                crate::terminal::clipboard::Clipboard::new(
                    cc.display_handle().ok().map(|display| display.as_raw()),
                    wake,
                )
            };
            Ok(Box::new(host))
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
