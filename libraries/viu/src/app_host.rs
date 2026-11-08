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
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};
type Reply = tokio::sync::oneshot::Sender<(u8, Value)>;
enum SelectionWait {
    Platform(String),
    Osc52(crate::terminal::sessions::PaneClipboardReply),
}
pub(crate) fn validate_overlays(model: &Value) -> Result<(), String> {
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
    require_bus: bool,
    forward_output: bool,
    quit_reply_closes: bool,
    windows: crate::child_windows::Windows,

    no_window: bool,
    deliveries: Vec<Arc<std::sync::atomic::AtomicU8>>,
    platform: crate::platform::Platform,
    native_selection: bool,
    paste_waits: std::collections::VecDeque<String>,
    paste_deadline: Option<Instant>,
    selection: crate::terminal::clipboard::Clipboard,
    selection_sequence: u64,
    selection_waits: BTreeMap<u64, SelectionWait>,
    platform_results: std::collections::VecDeque<Value>,
    calls: crate::calls::Calls,
    close_replies: BTreeMap<String, Value>,
    timers: crate::timers::Timers,
    timer_results: std::collections::VecDeque<Value>,
    documents: crate::documents::Documents,
    rasters: crate::raster::Rasters,
    terminals: crate::terminal::sessions::Sessions,
    ui_state: crate::ui_state::UiState,
    measurements: crate::ui_state::Measurements,
    input_at: Option<Instant>,
    native_drive: bool,
    host_macro_popup: bool,
    preserve_request_body: bool,
    service: String,
    settings_reload: u64,
    session_update: Option<Value>,
    deferred_replies: BTreeMap<String, (u64, Instant, String)>,
    drive_replies: BTreeMap<u64, Reply>,
    drive_answers: std::collections::VecDeque<toolkit::drive::Answer>,
    drive_serial: u64,
    drive_geometry: Option<BTreeMap<String, egui::Rect>>,
    drive_revision: Option<u64>,
    drive_geometry_stable: bool,
    macros: macros::Session,
    macro_ui: toolkit::macros::State,
    macro_pending: Vec<(std::sync::mpsc::Receiver<macros::commands::Answer>, Reply)>,
    macro_finished: std::sync::mpsc::Receiver<Value>,
    headless: bool,
    drive: Option<String>,
    wire_deliveries: Vec<Arc<std::sync::atomic::AtomicU8>>,
    retained: BTreeMap<u64, Arc<std::sync::atomic::AtomicBool>>,
    session_notice: Option<Value>,
    local_ui: crate::local_ui::LocalUi,
    native_choice: Value,
    overlays: crate::widgets::Widgets,
    focus_serial: Value,
    facilities: crate::app_facilities::Facilities,
    session: Option<crate::session::Session>,
    session_appearance: Option<Value>,
    session_window: Option<Value>,
    retain_window_on_loss: bool,
    app: String,
    file_theme: toolkit::Theme,
    last_ui: settings::model::CommonUi,
    session_dirty: bool,
    session_key: Value,
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
    local_clipboard: std::sync::mpsc::Receiver<crate::clipboard::Request>,
    commands: BTreeSet<String>,
    scene_source: String,
    loaded: bool,
    active_scene: String,
    owner_close_pending: bool,
    model: Value,
    output: Option<String>,
    outputs: BTreeMap<String, Value>,
    runtime: Runtime,
    view: SceneView,
    bus: Endpoint,
    pending: BTreeMap<u64, String>,
    deferred: BTreeMap<u64, String>,
    held_requests: std::collections::VecDeque<Request>,
    inflight: BTreeSet<u64>,
    ui_inflight: BTreeSet<u64>,
    dispatching_ui: bool,
    input: Input,
    input_reply: Option<Reply>,
    capture: crate::capture::Capture,
    capture_reply: Option<Reply>,
    ready: bool,
    error: String,
    tick: Option<Duration>,
    next_tick: Instant,
    last_viewport: Option<[f32; 3]>,
    notice_command: Option<String>,
    saved: Option<crate::saved_state::Saved>,
}
impl Host {
    fn model_close_ready(&self) -> bool {
        self.model["window_close"] == true
            && self.wire_deliveries.is_empty()
            && self.pending.is_empty()
            && self.deferred.is_empty()
            && self.deferred_replies.is_empty()
            && self.drive_replies.is_empty()
            && self.inflight.is_empty()
            && self.held_requests.is_empty()
    }
    fn queued_click(&self) -> bool {
        self.held_requests.front().is_some_and(|request| {
            request
                .command
                .strip_prefix(&format!("{}.", self.app))
                .is_some_and(|command| command == "ui.click")
        })
    }
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
        if model["native_root"].is_object() {
            projected = model["native_root"]["model"].clone();
            projected["native_windows"] = model["native_windows"].clone();
            projected["native_root"] = model["native_root"].clone();
            projected["window_close"] = model["window_close"].clone();
            projected["__viu_root_key"] = model["native_root"]["key"].clone();
        }
        let style = ctx.global_style();
        projected["__viu_metrics"] = json!({
            "item_spacing_x":style.spacing.item_spacing.x,
            "item_spacing_y":style.spacing.item_spacing.y,
            "control_row_height":toolkit::button::icon_button_of(ctx).max(style.spacing.interact_size.y + 4.0)
        });

        if self.framed {
            projected["__viu_window_client"] = json!(crate::window_frame::is_client(
                &self.window_settings(&projected)
            ));
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
        mut body: Value,
        raw: Option<String>,
        mut headers: BTreeMap<String, String>,
        reply: Option<Reply>,
    ) {
        if reply.is_none() && !headers.contains_key("viu-call") && body.is_object() {
            if let Some(key) = self.model["__viu_root_key"].as_str() {
                body["window"] = json!(key);
            }
        }
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
                if self.dispatching_ui {
                    self.ui_inflight.insert(seq);
                }
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
        if self.no_window {
            self.deliveries
                .retain(|d| d.load(std::sync::atomic::Ordering::Acquire) == 0);
            self.deliveries.push(r.delivery.clone());
            if !self.commands.contains(&r.command) {
                if let Some(handler) = self.unknown_handler.clone() {
                    if !self.ready {
                        let _ = r.reply.send((10, json!({"error_code":"NOT_READY"})));
                        return;
                    }
                    // Preserve authored admission policy without running any
                    // window facility for an undeclared headless command.
                    r.headers.insert("viu-unknown-command".into(), r.command);
                    let raw = self.preserve_request_body.then_some(r.body);
                    self.dispatch_body(handler, r.args, raw, r.headers, Some(r.reply));
                    return;
                }
                let _ = r.reply.send((
                    10,
                    self.unknown_reply
                        .clone()
                        .unwrap_or_else(|| json!({"error_code":"UNKNOWN_COMMAND"})),
                ));
                return;
            }
        }
        self.wire_deliveries.push(r.delivery.clone());
        r.headers.insert("viu_input".into(), "bus".into());
        r.headers.insert("from".into(), r.from.clone());
        let mut close_reply = None;

        if self.native_drive && self.commands.contains(&format!("{}.quit", self.app)) {
            if r.command == format!("{}.window", self.app) && r.args["action"] == "close" {
                close_reply =
                    toolkit::drive::request(ctx, 0, "window.state", &json!({})).map(|answer| {
                        let mut body = answer.body;
                        body["requested"] = json!("close");
                        body
                    });
                r.command = format!("{}.quit", self.app);
            }
        }
        if r.command.starts_with("viu.") {
            if let Some(key) = r.args["window"].as_str().map(str::to_owned) {
                if self.model["__viu_root_key"] == key {
                    r.args.as_object_mut().unwrap().remove("window");
                } else {
                    self.windows.request(&key, r, ctx);
                    return;
                }
            }
        }
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
        if self.quit_reply_closes
            && (r.command == format!("{}.quit", self.app)
                || (r.command == format!("{}.execute", self.app)
                    && matches!(r.args["id"].as_str(), Some("file.quit" | "app.quit"))))
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
                            |message| if message.contains("{command}") { json!({"error_code":"UNKNOWN_COMMAND","message":message.replace("{command}",&r.command),"command":r.command}) } else { json!({"error_code":"UNKNOWN_COMMAND","message":message}) },
                        )
                    }),
                ));
                return;
            }
        }
        if let Some((rc, body)) = crate::debug::request(&self.app, &r.command, &r.args.to_string())
        {
            let _ = r.reply.send((rc, body));
            return;
        }
        let prefix = format!("{}.", self.app);
        if let Some(suffix) = r.command.strip_prefix(&prefix) {
            if toolkit::drive::COMMANDS.contains(&suffix) {
                if self.no_window {
                    let _=r.reply.send((10,json!({"error_code":"HEADLESS","message":"a headless instance has no window"})));
                    return;
                }
                if suffix == "window"
                    && r.args["action"] == "close"
                    && r.args.get("viewport").is_none()
                {
                    self.dispatch("viu.frame.close".into(), json!({}), BTreeMap::new(), None);
                    if self.drive.is_some() {
                        if let Some(mut a) =
                            toolkit::drive::request(ctx, 0, "window.state", &json!({}))
                        {
                            a.body["requested"] = json!("close");
                            let _ = r.reply.send((a.rc, a.body));
                        }
                    } else {
                        let _ = r.reply.send((0, json!({"action":"close"})));
                    }
                    return;
                }
                self.drive_serial += 1;
                let id = self.drive_serial;
                if let Some(mut answer) = toolkit::drive::request(ctx, id, suffix, &r.args) {
                    crate::app_facilities::remap(ctx, &mut answer.body);
                    let _ = r.reply.send((answer.rc, answer.body));
                } else {
                    self.drive_replies.insert(id, r.reply);
                }
                return;
            }
            if let Some(command) = suffix
                .strip_prefix("macros.")
                .filter(|_| !self.facilities.owns_macro_command(&r.command))
            {
                match macros::commands::handle(&self.macros, command, &r.args) {
                    macros::commands::Reply::Immediate(answer) => {
                        let _ = r.reply.send(macro_answer(answer));
                    }
                    macros::commands::Reply::Deferred(answer) => {
                        self.macro_pending.push((answer, r.reply))
                    }
                }
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
            _ => match self.calls.begin_full(
                r.reply,
                r.close_after_reply,
                r.deferred.clone(),
                success_body,
            ) {
                Ok(token) => {
                    if let Some(body) = close_reply {
                        self.close_replies.insert(token.clone(), body);
                    }
                    let mut headers = r.headers;
                    headers.insert("viu-request-caller".into(), r.from);
                    let before = self.runtime.sequence();
                    headers.insert("viu-call".into(), token);
                    let raw = self.preserve_request_body.then_some(r.body);
                    self.dispatch_body(r.command, r.args, raw, headers, None);
                    let after = self.runtime.sequence();
                    if after != before {
                        self.retained.insert(after, r.deferred);
                    }
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
        if self.framed
            && self.model["window_close"] != true
            && ctx.input(|i| i.viewport().close_requested())
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.owner_close_pending = true;
        }
        self.pump(ctx);
        if self.owner_close_pending && !toolkit::drive::pending(ctx) {
            self.owner_close_pending = false;
            self.dispatch("viu.frame.close".into(), json!({}), BTreeMap::new(), None);
        }
    }
    fn raw_input_hook(&mut self, _: &egui::Context, raw: &mut egui::RawInput) {
        if self.windows.raw_input(raw) {
            return;
        }
        raw.dropped_files.append(&mut self.input.dropped);
        if !raw.dropped_files.is_empty() {
            let paths = raw
                .dropped_files
                .iter()
                .map(|f| f.path().to_string_lossy().into_owned())
                .collect::<Vec<_>>();
            if !paths.is_empty() {
                self.dispatch(
                    "viu.files.dropped".into(),
                    json!({"paths":paths}),
                    BTreeMap::new(),
                    None,
                );
            }
        }
        if raw.events.contains(&egui::Event::WindowFocused(true)) {
            self.dispatch(
                "viu.window.focused".into(),
                json!({}),
                BTreeMap::new(),
                None,
            );
        }
        if self.paste_deadline.is_some() {
            raw.events.retain(|event| {
                if let egui::Event::Paste(text) = event {
                    if self.paste_deadline.is_some()
                        && let Some(id) = self.paste_waits.pop_front()
                    {
                        self.paste_deadline = None;
                        self.platform_results
                            .push_back(json!({"id":id,"text":text}));
                        return false;
                    }
                }
                true
            });
        }
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
        self.wire_deliveries
            .retain(|s| s.load(std::sync::atomic::Ordering::Acquire) == 0);
        while let Ok(request) = self.local_clipboard.try_recv() {
            if self.clipboard_reply.is_none()
                && self
                    .clipboard
                    .replace(&request.path, std::path::Path::new(&request.path), ctx)
            {
                self.clipboard_reply = Some(request.reply)
            } else {
                let _ = request.reply.send((10, json!({"error_code":"BUSY"})));
            }
        }
        if !self.facilities.native_ui() {
            self.drive_answers.extend(toolkit::drive::logic(ctx));
        }
        if self.inflight.is_empty() && self.view.current_revision_drawn() {
            while let Some(mut answer) = self.drive_answers.pop_front() {
                if let Some(reply) = self.drive_replies.remove(&answer.id) {
                    crate::app_facilities::remap(ctx, &mut answer.body);
                    let _ = reply.send((answer.rc, answer.body));
                }
            }
        }
        let mut waiting = Vec::new();
        for (receiver, reply) in self.macro_pending.drain(..) {
            match receiver.try_recv() {
                Ok(answer) => {
                    let _ = reply.send(macro_answer(answer));
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => waiting.push((receiver, reply)),
                Err(_) => {
                    let _ = reply.send((
                        10,
                        json!({"error_code":"INTERNAL","message":"macro worker closed"}),
                    ));
                }
            }
        }
        self.macro_pending = waiting;
        while let Ok(body) = self.macro_finished.try_recv() {
            if self.facilities.macros.is_none() {
                self.facilities
                    .event(format!("{}.macros.finished", self.app), body);
            }
        }
        while self.runtime.has_capacity() {
            let Some((key, event)) = self.windows.take_one() else {
                break;
            };
            match self.runtime.event(event.command, event.body) {
                Ok(sequence) => {
                    self.inflight.insert(sequence);
                    self.windows.submitted(sequence, key);
                }
                Err(_) => self.windows.rejected(&key),
            }
        }
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
            if !self.native_selection {
                if request["kind"] == "copy" { ctx.copy_text(request["body"]["text"].as_str().unwrap_or("").into()); }
                else { self.paste_waits.push_back(request["id"].as_str().unwrap_or("").into()); }
                continue;
            }
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
        if self.paste_deadline.is_some_and(|at| at <= Instant::now()) {
            self.paste_deadline = None;
            if let Some(id) = self.paste_waits.pop_front() {
                self.platform_results
                    .push_back(json!({"id":id,"text":"","reason":"timeout"}));
            }
        }
        if self.paste_deadline.is_none() && !self.paste_waits.is_empty() {
            self.paste_deadline = Some(Instant::now() + Duration::from_secs(2));
            ctx.send_viewport_cmd(egui::ViewportCommand::RequestPaste);
        }
        if let Some(at) = self.paste_deadline {
            ctx.request_repaint_after(at.saturating_duration_since(Instant::now()));
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
            let seqs = self
                .pending
                .iter()
                .filter_map(|(seq, t)| (t == &token).then_some(*seq))
                .collect::<Vec<_>>();
            for seq in seqs {
                let _ = self.runtime.forget_reply(seq);
                self.pending.remove(&seq);
                self.retained.remove(&seq);
            }
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

        self.deferred.retain(|_, token| self.calls.contains(token));
        while let Ok(request) = self.local_ui.requests.try_recv() {
            let answer = match request.command.as_str() {
                "native.defer" => request.args["id"]
                    .as_u64()
                    .ok_or("request id must be an integer".into())
                    .and_then(|id| {
                        let reply = self.pending.remove(&id).ok_or("request is not pending")?;
                        self.calls.hold(&reply);
                        if let Some(flag) = self.retained.get(&id) {
                            flag.store(true, std::sync::atomic::Ordering::Release);
                        }
                        self.deferred.insert(id, reply);
                        Ok(json!({"deferred":id}))
                    }),
                "native.reply" => request.args["id"]
                    .as_u64()
                    .zip(request.args["rc"].as_u64().filter(|rc| *rc <= 255))
                    .ok_or("request id and byte rc required".into())
                    .and_then(|(id, rc)| {
                        let value = match request.args.get("value_json") {
                            Some(Value::String(text)) if text.len() <= 16 * 1024 * 1024 => {
                                serde_json::from_str(text).map_err(|e| e.to_string())?
                            }
                            Some(_) => return Err("value_json must be bounded JSON text".into()),
                            None => request.args["value"].clone(),
                        };
                        let reply = self.deferred.remove(&id).ok_or("request is not deferred")?;
                        if !self.calls.finish(&reply, rc as u8, value, false) {
                            return Err("request was cancelled".to_owned());
                        }
                        Ok(json!({"replied":id}))
                    }),
                "native.executed" => self.facilities.record(&request.args),
                "native.saved" => self
                    .saved
                    .as_mut()
                    .ok_or("saved state is not enabled".into())
                    .map(|saved| {
                        saved.apply(&request.args);
                        saved.tick(ctx, &self.app, false);
                        saved.store.describe()
                    }),
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
            if let Some((code, message)) = self.ui_state.take_exit() {
                if !message.is_empty() {
                    eprintln!("{}: {message}", self.app);
                }
                std::process::exit(i32::from(code));
            }
            if self.forward_output {
                use std::io::Write;
                print!("{}", done.stdout);
                let _ = std::io::stdout().flush();
                eprint!("{}", done.stderr);
            }
            self.measurements
                .model(done.elapsed.as_micros().min(u64::MAX as u128) as u64);
            self.inflight.remove(&done.sequence);
            self.ui_inflight.remove(&done.sequence);
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
                    if self.no_window {
                        self.ready = true;
                        self.model = model;
                    } else {
                        self.session_dirty |= self.model["theme_choice"] != model["theme_choice"];
                        let projected = self.projection(&model, ctx);
                        let source = model["native_root"]["scene"]
                            .as_str()
                            .unwrap_or(&self.scene_source)
                            .to_owned();
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
                            if model["native_root"].is_object() {
                                self.windows.validate(
                                    &json!({"native_windows":[model["native_root"].clone()]}),
                                )?;
                            }
                            self.windows.validate(&model)?;
                            if self.loaded && self.active_scene == source {
                                self.view.apply_model(projected)
                            } else {
                                (if self.framed {
                                    crate::gold_frame::mount(&source, projected)
                                } else {
                                    SceneView::with_model(&source, projected)
                                })
                                .map(|view| {
                                    self.view = view;
                                    self.view.set_documents(self.documents.clone());
                                    self.view.set_rasters(self.rasters.clone());
                                    self.view.set_terminals(self.terminals.clone());
                                    self.view.set_native_menus(self.native_drive);
                                    self.loaded = true;
                                    self.active_scene = source;
                                })
                            }
                        });
                        match result {
                            Ok(()) => {
                                if self.model["theme_choice"] != model["theme_choice"] {
                                    self.session_dirty = true;
                                }
                                if let Some(style) = appearance.expect("validated appearance") {
                                    crate::appearance::install_style(
                                        ctx,
                                        &model["appearance"],
                                        style,
                                    );
                                }
                                self.ready = true;
                                self.windows.sync(
                                    ctx,
                                    &self.app,
                                    &model,
                                    &self.documents,
                                    self.native_drive,
                                );
                                if let Some(root) =
                                    model.get("native_root").filter(|v| v.is_object())
                                {
                                    let key = root["key"].as_str().unwrap();
                                    toolkit::drive::register_viewport(
                                        ctx,
                                        egui::ViewportId::ROOT,
                                        key,
                                        egui::ViewportId::ROOT,
                                    );
                                    let title = root["title"].as_str().unwrap();
                                    if self.title != title {
                                        self.title = title.into();
                                        ctx.send_viewport_cmd(egui::ViewportCommand::Title(
                                            title.into(),
                                        ));
                                        ctx.send_viewport_cmd(egui::ViewportCommand::InnerSize(
                                            egui::vec2(
                                                root["size"][0].as_f64().unwrap() as f32,
                                                root["size"][1].as_f64().unwrap() as f32,
                                            ),
                                        ));
                                    }
                                    self.model = self.projection(&model, ctx);
                                } else {
                                    self.model = model;
                                }
                                ctx.request_repaint();
                                if let Some(saved) = &mut self.saved {
                                    if saved.announced {
                                        saved.apply(&self.model);
                                    }
                                }
                            }
                            Err(e) => self.error = e,
                        }
                    }
                }
            }
            self.windows.completed(done.sequence);
            for (seq, answer) in done.answers {
                if let Some(token) = self.pending.remove(&seq) {
                    self.calls.finish(&token, answer.rc, answer.body, false);
                    self.retained.remove(&seq);
                }
            }
            for answer in done.deferred_answers {
                if let Some((generation, _, reply)) = self.deferred_replies.remove(&answer.token) {
                    if generation == done.generation {
                        self.calls
                            .finish(&reply, answer.reply.rc, answer.reply.body, false);
                    } else {
                        self.calls.finish(&reply,10,json!({"error_code":"STALE_GENERATION","message":"deferred request owner changed"}),false);
                    }
                }
            }
            if done.deferred && self.error.is_empty() {
                if let Some(flag) = self.retained.get(&done.sequence) {
                    flag.store(true, std::sync::atomic::Ordering::Release);
                }
                continue;
            }
            self.retained.remove(&done.sequence);
            if !self.error.is_empty() {
                if let Some(token) = self.deferred.remove(&done.sequence) {
                    self.calls
                        .finish(&token, 10, json!({"error":self.error}), false);
                }
            }

            if let Some(reply) = self.pending.remove(&done.sequence) {
                if self.error.is_empty()
                    && let Some(token) = done.deferred_token
                {
                    if self.deferred_replies.len() >= 32
                        || self.deferred_replies.contains_key(&token)
                    {
                        self.calls.finish(&reply,10,json!({"error_code":"BUSY","message":"deferred request budget or token collision"}),false);
                    } else {
                        self.deferred_replies.insert(
                            token,
                            (
                                done.generation,
                                Instant::now() + Duration::from_secs(90),
                                reply,
                            ),
                        );
                    }
                    continue;
                }
                let answer = if !self.error.is_empty() {
                    (10, json!({"error":self.error}))
                } else if let Some(r) = done.reply {
                    (r.rc, r.body)
                } else {
                    (0, self.state())
                };
                let body = self
                    .close_replies
                    .remove(&reply)
                    .filter(|_| answer.0 == 0)
                    .unwrap_or(answer.1);
                self.calls.complete(&reply, answer.0, body);
            }
            if !done.behaviour_available {
                self.retained.clear();
                for (_, reply) in std::mem::take(&mut self.pending) {
                    self.calls
                        .finish(&reply, 20, json!({"error_code":"OWNER_CLOSED"}), false);
                }
            }
        }
        let expired: Vec<_> = self
            .deferred_replies
            .iter()
            .filter(|(_, (_, at, _))| *at <= Instant::now())
            .map(|(key, _)| key.clone())
            .collect();
        for key in expired {
            if let Some((_, _, reply)) = self.deferred_replies.remove(&key) {
                self.calls.finish(
                    &reply,
                    20,
                    json!({"error_code":"TIMEOUT","message":"deferred request expired"}),
                    false,
                );
            }
        }
        if let Some(deadline) = self.deferred_replies.values().map(|(_, at, _)| *at).min() {
            ctx.request_repaint_after(deadline.saturating_duration_since(Instant::now()));
        }
        if let Some(saved) = &mut self.saved {
            if self.ready && !saved.announced {
                saved.store.widgets.install(ctx);
                saved.store.install_windows(ctx, &self.app);
                if self
                    .runtime
                    .notify("viu.saved.loaded".into(), saved.notice())
                    .is_ok()
                {
                    saved.announced = true;
                }
            }
            if saved.announced {
                saved.restore(self.bus.client());
                if self.model["window_close"] == true {
                    saved.capture_exit(self.bus.client());
                }
                saved.tick(ctx, &self.app, self.model["window_close"] == true);
            }
        }
        for _ in self.held_requests.len()..32 {
            match self.bus.requests.try_recv() {
                Ok(r) => self.held_requests.push_back(r),
                Err(_) => break,
            }
        }
        while !self.held_requests.is_empty() {
            if !self.runtime.has_capacity() {
                break;
            }
            if self.ready && !self.headless && !self.view.current_revision_drawn() {
                ctx.request_repaint();
                break;
            }
            if toolkit::drive::pending(ctx) || !self.drive_answers.is_empty() {
                break;
            }
            if self.queued_click() && (!self.inflight.is_empty() || !self.drive_geometry_stable) {
                ctx.request_repaint();
                break;
            }
            let request = self.held_requests.pop_front().unwrap();
            self.request(request, ctx);
        }
        for _ in 0..32 {
            if !self.runtime.has_capacity() {
                break;
            }
            match self.bus.notices.try_recv() {
                Ok(n) => {
                    if let Some(saved) = &self.saved {
                        saved.notice_topic(&n.command);
                    }
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
                    let (command, args) = match &self.notice_command {
                        Some(command) => {
                            (command.clone(), json!({"command":n.command,"args":n.args}))
                        }
                        None => (n.command, n.args),
                    };
                    let _ = self.runtime.notify_with_headers(command, args, n.headers);
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
                        self.view.set_rasters(self.rasters.clone());
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
                let refresh_key = self.model.get("theme_refresh_key");
                let key_changed = refresh_key.is_some_and(|key| key != &self.session_key);
                if let Some(snapshot) = session.poll().or_else(|| {
                    (self.session_dirty || key_changed).then(|| session.snapshot.clone())
                }) {
                    let appearance = snapshot["appearance"].clone();
                    let mut authored = snapshot.clone();
                    if self.model["theme_values"].is_object() {
                        authored["state"]["values"] = self.model["theme_values"].clone();
                        authored["state"]["lost"] = Value::Bool(false);
                    }
                    let installed = if refresh_key.is_none() || key_changed {
                        crate::appearance::install_session_choice(
                            ctx,
                            &authored,
                            &self.app,
                            &self.file_theme,
                            &self.last_ui,
                            &self.model["theme_choice"],
                        )
                    } else {
                        Ok(self.last_ui.clone())
                    };
                    match installed {
                        Ok(common_ui) => {
                            self.session_key = refresh_key.cloned().unwrap_or(Value::Null);
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
                            self.session_update =
                                Some(crate::appearance::session_update(ctx, &snapshot));
                            self.session_notice = Some(snapshot.clone());
                            self.session_dirty = false;
                            self.session_appearance = Some(appearance);
                            let mut window = if snapshot["state"]["lost"] == true
                                && self.retain_window_on_loss
                            {
                                self.session_window
                                    .clone()
                                    .unwrap_or_else(|| self.model["window_settings"].clone())
                            } else {
                                snapshot["state"]["values"].clone()
                            };
                            for (axis, key) in [
                                ("decorations", "window.decorations"),
                                ("caption_side", "window.caption_side"),
                            ] {
                                if self.model["theme_choice"][axis].is_string() {
                                    window[key] = self.model["theme_choice"][axis].clone();
                                }
                            }
                            self.session_window = Some(window);
                            if (refresh_key.is_none() || key_changed)
                                && self
                                    .native_choice
                                    .as_object()
                                    .is_some_and(|c| !c.is_empty())
                                && let Ok(choice) = crate::native_theme::choice(&self.native_choice)
                            {
                                if let Ok((base, _)) = crate::appearance::session_base(
                                    &snapshot,
                                    &self.app,
                                    &self.file_theme,
                                    &self.last_ui,
                                ) {
                                    if let Ok(theme) = base.with_choice(&choice) {
                                        let _ = crate::appearance::install_theme(ctx, &theme);
                                    }
                                }
                            }
                            if refresh_key.is_none() || key_changed {
                                // Followers need the effective theme even when
                                // no local override is selected (for example,
                                // to switch the light/dark toolbar glyph).
                                let _ = self.runtime.notify(
                                    "viu.theme.changed".into(),
                                    crate::native_theme::describe(
                                        &crate::appearance::session_theme(ctx),
                                    ),
                                );
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
        if let Some(snapshot) = &self.session_notice {
            if self
                .runtime
                .notify("viu.session.changed".into(), snapshot.clone())
                .is_ok()
            {
                self.session_notice = None;
            } else {
                ctx.request_repaint();
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
                } else {
                    ctx.request_repaint();
                }
            }
        }
        self.settle(ctx);
    }
    fn settle(&mut self, ctx: &egui::Context) {
        if let Some(error) = self.view.take_focus_error() {
            self.error = error;
        }
        if self.inflight.is_empty()
            && self.ui_inflight.is_empty()
            && self.view.current_revision_drawn()
        {
            self.facilities.settle_answers();
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
        let rect = ui.ctx().content_rect();
        let viewport = [rect.width(), rect.height(), ui.ctx().pixels_per_point()];
        if self.last_viewport != Some(viewport)
            && self
                .runtime
                .notify(
                    "viu.viewport.changed".into(),
                    json!({"width":viewport[0],"height":viewport[1],"scale":viewport[2]}),
                )
                .is_ok()
        {
            self.last_viewport = Some(viewport);
        }
        if self.ready {
            if self.facilities.macros.is_none() {
                self.macro_ui.open = self.model["macros_open"] == true;
                let was_open = self.macro_ui.open;
                self.macro_ui.show(ui.ctx());
                if self.macro_ui.open != was_open {
                    self.dispatch("viu.macros.closed".into(), json!({}), BTreeMap::new(), None);
                }
            }

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
                    .show_scene_frame(ui, &self.title, &settings);
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
            let mut input_headers = self.facilities.input_headers(ui.ctx());
            if self.input_reply.is_some() {
                input_headers.insert("viu_input".into(), "bus".into());
            }
            self.dispatching_ui = true;
            for event in self.view.show(ui) {
                self.dispatch(event.command, event.body, input_headers.clone(), None);
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
                    self.dispatch(command.into(), intent.encode(), input_headers.clone(), None);
                }
            }
            self.view.geometry.extend(self.overlays.geometry.clone());
            if self.model["keyboard_events"] == true {
                let keys=ui.input(|i|i.events.iter().filter_map(|e|match e{egui::Event::Key{key,pressed:true,repeat:false,modifiers,..}=>Some(json!({"key":crate::input::key_name(*key),"ctrl":modifiers.ctrl||modifiers.command,"alt":modifiers.alt,"shift":modifiers.shift,"typing":self.view.text_input_active()})),_=>None}).collect::<Vec<_>>());
                for key in keys {
                    self.dispatch("viu.key".into(), key, BTreeMap::new(), None);
                }
            }
            if self.host_macro_popup
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
                    let mut target = json!({"id":id});
                    if self.model["native_focus"]["selection"].is_array() {
                        target["selection"] = self.model["native_focus"]["selection"].clone();
                    }
                    let _ = self.view.request_focus_value(&target);
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
                        if (self.view.text_input_active()
                            || ui.ctx().memory(|m| m.focused().is_some()))
                            && !editor_focus
                            && !allowed
                        {
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
                        input_headers.clone(),
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
                    self.owner_close_pending = true;
                    ui.ctx().request_repaint();
                }
                if self.model["window_close"] == true
                    && self.wire_deliveries.is_empty()
                    && self.drive_replies.is_empty()
                {
                    self.facilities.finish(ui.ctx());
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
            self.dispatching_ui = false;
        }
        self.windows.show(ui, &self.app);
        self.ui_state.set(
            ui.ctx().content_rect(),
            &self.view.geometry,
            self.measurements.snapshot(),
        );
        // A model's first layout can still use egui's previous grid sizes.
        // Resolve native input only after two matching control passes, so its
        // AccessKit target cannot move between resolution and the press pass.
        let unchanged = self.drive_geometry.as_ref() == Some(&self.view.geometry);
        self.drive_geometry_stable = self.drive_revision == Some(self.view.revision) && unchanged;
        self.drive_revision = Some(self.view.revision);
        if !unchanged {
            self.drive_geometry = Some(self.view.geometry.clone());
        }
        if self.queued_click() && !self.drive_geometry_stable {
            ui.ctx().request_repaint();
        }
        self.settle(ui.ctx());
        if !self.error.is_empty() {
            ui.colored_label(egui::Color32::RED, &self.error);
        }
    }
}

fn macro_answer(answer: macros::commands::Answer) -> (u8, Value) {
    match answer {
        Ok(body) => (0, body),
        Err(error) => (10, json!({"error_code":error.code,"message":error.message})),
    }
}
impl Drop for Host {
    fn drop(&mut self) {
        self.runtime.shutdown();
        self.terminals.shutdown();
        for (_, (_, _, reply)) in std::mem::take(&mut self.deferred_replies) {
            self.calls.finish(
                &reply,
                20,
                json!({"error_code":"APP_CLOSED","message":"request owner closed"}),
                false,
            );
        }
        if let Err(error) = self.macros.shutdown() {
            tracing::warn!(%error, "macro shutdown");
        }
    }
}

fn create(bundle: Bundle, wake: crate::runtime::Wake) -> Host {
    let headless = bundle.manifest.surface == "headless" && bundle.startup.is_some();
    let service = bundle.manifest.service.clone();
    let saved = bundle.manifest.saved_version.map(|v| {
        let mut saved = crate::saved_state::Saved::load(
            bundle.manifest.app(),
            v,
            bundle.launch["headless"] == true,
        );
        if bundle.launch["headless"] != true {
            saved.configure(
                bundle
                    .manifest
                    .bus_surface
                    .as_ref()
                    .and_then(|s| s["app_id"].as_str())
                    .map(str::to_owned)
                    .unwrap_or_else(|| format!("mixviu.{service}")),
                bundle.launch["comp"].as_str().unwrap_or("comp").to_owned(),
            );
        }
        saved
    });
    let native_choice = saved
        .as_ref()
        .map(|saved| {
            let mut v = serde_json::to_value(&saved.store.window.theme).unwrap();
            v["caption_side"] = v["captions"].take();
            v
        })
        .unwrap_or_else(|| json!({}));
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
    let app = bundle.manifest.app();
    commands.extend(toolkit::drive::COMMANDS.map(|suffix| format!("{app}.{suffix}")));
    commands.extend(macros::commands::SUFFIXES.map(|suffix| format!("{app}.macros.{suffix}")));
    commands.extend(["debug", "debug.set", "debug.recent"].map(|suffix| format!("{app}.{suffix}")));
    if !headless {
        commands.extend(
            [
                "viu.state",
                "viu.ui.tree",
                "viu.ui.click",
                "viu.ui.focus",
                "viu.ui.text",
                "viu.ui.key",
                "viu.ui.ime",
                "viu.ui.hover",
                "viu.ui.scroll",
                "viu.ui.drag",
                "viu.ui.drop",
                "viu.ui.clipboard",
                "viu.capture",
                "viu.quit",
                "viu.clipboard.image",
            ]
            .map(String::from),
        );
    }
    let mut command_names = BTreeSet::new();
    commands.retain(|name| command_names.insert(name.clone()));
    let declared = commands.iter().cloned().collect();
    let bus = if !bundle.manifest.bus {
        Endpoint::offline()
    } else {
        Endpoint::start_policy(
            std::env::var("MIX_VIU_SERVICE").unwrap_or(service),
            bundle.manifest.bus_url(),
            {
                let mut described = bundle.manifest.command_descriptors.clone();
                for entry in &bundle.manifest.command_metadata {
                    described.push(bus::CommandDescriptor {
                        name: entry["name"].as_str().unwrap().into(),
                        args: entry["args"]
                            .as_object()
                            .map(|a| a.keys().cloned().collect())
                            .unwrap_or_default(),
                        description: entry["description"].as_str().unwrap().into(),
                        read_only: entry["read_only"] == true,
                    });
                }
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
            crate::native_bus::RequestPolicy {
                help: facilities
                    .surface
                    .as_ref()
                    .map(|s| s["commands"].clone())
                    .or_else(|| bundle.manifest.authored_discovery.then(|| json!([]))),
                owner_help: bundle.manifest.owner_help,
                timeout: Duration::from_secs(bundle.manifest.reply_timeout_secs),
                transport: crate::native_bus::Policy {
                    pid_fallback: bundle.manifest.registration_pid_fallback,
                    namespace: bundle.manifest.command_namespace.clone(),
                    text_replies: bundle.manifest.text_replies,
                    unknown_reply: bundle.manifest.unknown_reply.clone(),
                },
            },
        )
    };
    let documents = crate::documents::Documents::default();
    let rasters = crate::raster::Rasters::default();
    let terminals = crate::terminal::sessions::Sessions::with_native(wake.clone());
    let timers = crate::timers::Timers::default();
    let calls = crate::calls::Calls::default();
    let ui_state = crate::ui_state::UiState::default();
    let platform = crate::platform::Platform::default();
    let roots = macros::Roots::resolve().map(|mut roots| {
        roots.mix = std::env::current_exe().expect("own macro interpreter artifact");
        roots
    });
    let macros = facilities
        .macros
        .clone()
        .unwrap_or_else(|| macros::Session::with_roots(&bundle.manifest.service, roots));
    let (macro_tx, macro_finished) = std::sync::mpsc::channel();
    let macro_wake = wake.clone();
    macros.add_finish_hook(Arc::new(move |body| {
        let _ = macro_tx.send(body);
        macro_wake();
    }));
    let (clipboard, local_clipboard) = crate::clipboard::Clipboard::channel(wake.clone());
    let services = crate::runtime::Services {
        clipboard: Some(clipboard),
        options: json!({"title":bundle.manifest.title,"headless":bundle.manifest.surface=="headless" || bundle.launch["headless"]==true}),
        max_string_bytes: bundle.manifest.max_string_bytes,
        platform: Some(platform.clone()),
        ui_state: Some(ui_state.clone()),
        macros: Some(macros.clone()),
        calls: Some(calls.clone()),
        timers: Some(timers.clone()),
        documents: Some(documents.clone()),
        rasters: Some(rasters.clone()),
        terminals: Some(terminals.clone()),
        ..Default::default()
    };
    let mut jobs = crate::jobs::Jobs::new(bundle.manifest.job_limit, wake.clone());
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
        require_bus: bundle.manifest.require_bus,
        forward_output: bundle.manifest.forward_output,
        quit_reply_closes: bundle.manifest.quit_reply_closes,
        windows: Default::default(),

        no_window: bool,
    deliveries: Vec::new(),
        native_selection: bundle.manifest.native_selection,
        paste_waits: Default::default(),
        paste_deadline: None,
        platform,
        selection: crate::terminal::clipboard::Clipboard::none(),
        selection_sequence: 0,
        selection_waits: BTreeMap::new(),
        platform_results: Default::default(),
        calls,
        close_replies: Default::default(),
        timers,
        timer_results: Default::default(),
        documents,
        rasters,
        terminals,
        ui_state,
        measurements: Default::default(),
        input_at: None,
        native_drive: bundle.manifest.native_drive,
        preserve_request_body: bundle.manifest.preserve_request_body,
        host_macro_popup: !bundle.manifest.native_drive || bundle.manifest.host_macro_popup,
        service: std::env::var("MIX_VIU_SERVICE")
            .unwrap_or_else(|_| bundle.manifest.service.clone()),
        settings_reload: 0,
        session_update: None,
        deferred_replies: BTreeMap::new(),
        drive_replies: BTreeMap::new(),
        drive_answers: Default::default(),
        drive_serial: 0,
        drive_geometry: None,
        drive_revision: None,
        drive_geometry_stable: false,
        macro_ui: toolkit::macros::State::for_session(&bundle.manifest.service, macros.clone()),
        macros,
        macro_pending: Vec::new(),
        macro_finished,
        headless: bundle.manifest.surface == "headless" || bundle.launch["headless"] == true,
        drive: bundle.manifest.drive.clone(),

        wire_deliveries: Vec::new(),
        session_notice: None,
        local_ui,
        native_choice,
        overlays: Default::default(),
        focus_serial: Value::Null,
        facilities,
        app: bundle.manifest.app().to_owned(),
        file_theme: toolkit::Theme::load(),
        last_ui: settings::model::CommonUi::default(),
        session_dirty: false,
        session_key: Value::Null,
        session: (bundle.manifest.frame.is_some() || bundle.manifest.follow_settings).then(|| {
            crate::session::Session::embedded(
                bundle.manifest.app(),
                &std::env::var("MIX_INSTANCE").unwrap_or_else(|_| "default".into()),
                wake.clone(),
                bus.backend(),
            )
            .expect("embedded session worker")
            .with_appearance_retention(bundle.manifest.retain_appearance_on_loss)
        }),
        session_appearance: None,
        session_window: None,
        retain_window_on_loss: bundle.manifest.retain_window_on_loss,
        framed: bundle.manifest.frame.is_some(),
        title: bundle.manifest.title.clone(),
        unknown_command: bundle.manifest.unknown_command.clone(),
        unknown_reply: bundle.manifest.unknown_reply.clone(),
        unknown_handler: bundle.manifest.unknown_handler.clone(),
        window_frame: crate::window_frame::WindowFrame::default(),
        clipboard: crate::thumbnail::Decoder::with_decoder(
            "viu-clipboard-decode",
            crate::image_view::decode_document,
        ),
        clipboard_reply: None,
        local_clipboard,
        jobs,
        job_results: std::collections::VecDeque::new(),
        commands: declared,
        scene_source: bundle.scene,
        loaded: false,
        active_scene: String::new(),
        owner_close_pending: false,
        model: json!({}),
        output: None,
        outputs: BTreeMap::new(),
        runtime,
        view: loading_view(),
        bus,
        pending: BTreeMap::new(),
        deferred: BTreeMap::new(),
        held_requests: std::collections::VecDeque::new(),
        retained: BTreeMap::new(),
        inflight: BTreeSet::new(),
        ui_inflight: BTreeSet::new(),
        dispatching_ui: false,
        input: Input::default(),
        input_reply: None,
        capture: crate::capture::Capture::new(capture_root, wake),
        capture_reply: None,
        ready: false,
        error: String::new(),
        tick: bundle.manifest.tick_ms.map(Duration::from_millis),
        next_tick: Instant::now(),
        last_viewport: None,
        notice_command: bundle.manifest.notice_command.clone(),
        saved,
    }
}

pub fn run(mut args: Vec<String>) -> eframe::Result {
    if args.first().map(String::as_str) != Some("run") {
        if let Err(error) = crate::debug::init("warn", &mut args) {
            eprintln!("viu: {error}");
            std::process::exit(2);
        }
    }
    if args.first().is_some_and(|arg| arg == "--macro") {
        let Some(path) = args.get(1) else {
            eprintln!("usage: viu --macro SCRIPT [ARG...]");
            std::process::exit(2);
        };
        std::process::exit(crate::macro_host::run(path.into(), args[2..].to_vec()));
    }
    if let Some(path) = args
        .first()
        .filter(|path| path.ends_with(".mix") && std::path::Path::new(path).is_file())
    {
        std::process::exit(crate::macro_host::run(path.into(), args[1..].to_vec()));
    }
    if args.first().map(String::as_str) != Some("run") || args.len() < 2 {
        eprintln!("usage: viu run APP_DIR [ARGS]");
        std::process::exit(2);
    }
    let root = std::path::Path::new(&args[1]);
    if args[2..].iter().any(|arg| arg == "--choose-service") {
        let chooser_args = args[2..]
            .iter()
            .filter(|arg| arg.as_str() != "--choose-service")
            .cloned()
            .collect::<Vec<_>>();
        let options = crate::launch::parse(&chooser_args)
            .map_err(|error| eframe::Error::AppCreation(Box::new(std::io::Error::other(error))))?;
        let profile = options.args.first().map(PathBuf::from).unwrap_or_else(|| {
            std::env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .unwrap_or_else(|| {
                    PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config")
                })
                .join("files")
        });
        let root = root
            .canonicalize()
            .map_err(|error| eframe::Error::AppCreation(Box::new(error)))?;
        return crate::chooser_host::run(
            root,
            profile,
            options.service.unwrap_or_else(|| "files".into()),
            options
                .url
                .unwrap_or_else(bus::client_helpers::resolve_noded_url),
        );
    }
    let preview = Bundle::load(root, &[]).unwrap_or_else(|e| {
        eprintln!("viu: {e}");
        std::process::exit(2)
    });
    let mut logging_args = std::iter::once("viu".to_owned())
        .chain(args.iter().cloned())
        .collect::<Vec<_>>();
    let _logging = toolkit::debug::init(
        preview.manifest.app(),
        preview.manifest.log_filter.as_deref().unwrap_or("warn"),
        &mut logging_args,
    )
    .unwrap_or_else(|e| {
        eprintln!("viu logging: {e}");
        std::process::exit(2)
    });
    if preview.manifest.startup.is_none()
        && logging_args
            .iter()
            .skip(3)
            .any(|a| matches!(a.as_str(), "--help" | "-h"))
    {
        if let Some(help) = &preview.manifest.help {
            print!("{help}");
            return Ok(());
        }

        println!(
            "Usage: viu run APP_DIR [--service NAME] [--comp NAME] [--noded-url URL] [APP_ARGS]"
        );
        println!("{}", toolkit::debug::HELP);
        return Ok(());
    }
    if preview.manifest.startup.is_none()
        && logging_args.get(3).map(String::as_str) == Some("--version")
    {
        println!(
            "{} {} (generic viu)",
            preview.manifest.app(),
            env!("CARGO_PKG_VERSION")
        );
        return Ok(());
    }
    let mut bundle = crate::launch::load(root, &logging_args[3..]).unwrap_or_else(|e| {
        eprintln!("viu: {e}");
        std::process::exit(i32::from(preview.manifest.argument_error_exit))
    });
    if let Some(program) = &bundle.startup {
        let choice = crate::startup::evaluate(program).unwrap_or_else(|e| {
            eprintln!("viu startup: {e}");
            std::process::exit(2)
        });
        if let Some((code, stdout, stderr)) = crate::startup::prepare(&mut bundle, choice)
            .unwrap_or_else(|e| {
                eprintln!("viu startup: {e}");
                std::process::exit(2)
            })
        {
            use std::io::Write;
            let out = std::io::stdout().write_all(stdout.as_bytes());
            let err = std::io::stderr().write_all(stderr.as_bytes());
            std::process::exit(if out.is_err() || err.is_err() {
                1
            } else {
                code.into()
            });
        }
    }
    // Authored early exits precede this native lifetime boundary. A descriptor
    // remains available to its intended owner but cannot leak to new children.
    crate::terminal::session_fd::quarantine_inherited();
    if let Some(output) = bundle.launch_plan["stdout"].as_str() {
        print!("{output}");
    }
    if bundle.launch_plan["exit"] == true {
        return Ok(());
    }
    if bundle.launch["headless"] == true {
        return run_headless(bundle);
    }
    if bundle.manifest.surface == "headless" && bundle.startup.is_some() {
        return run_terminal_headless(bundle)
            .map_err(|e| eframe::Error::AppCreation(Box::new(std::io::Error::other(e))));
    }
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
            reported: BTreeMap::new(),
        };
        if let Err(e) = crate::layer::run(options, layer) {
            eprintln!("viu layer: {e}");
            std::process::exit(1);
        }
        return Ok(());
    }
    let activation = bundle.launch_plan["activation"]
        .as_array()
        .map(|rows| {
            rows.iter()
                .map(|row| (row["command"].as_str().unwrap(), row["body"].clone()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| {
            bundle
                .manifest
                .activation
                .as_ref()
                .map(|command| vec![(command.as_str(), json!({}))])
                .unwrap_or_default()
        });
    if !activation.is_empty() && bundle.launch_plan["headless"] != true {
        match crate::native_bus::activate_existing_commands(
            &std::env::var("MIX_VIU_SERVICE").unwrap_or_else(|_| bundle.manifest.service.clone()),
            &activation,
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
    if bundle.launch_plan["headless"] == true {
        bundle.manifest.native_ui = false;
        bundle.manifest.native_drive = false;
        bundle.manifest.frame = None;
        let prefix = format!("{}.", bundle.manifest.app());
        if let Some(commands) = bundle
            .manifest
            .bus_surface
            .as_mut()
            .and_then(|surface| surface["commands"].as_array_mut())
        {
            commands.retain(|row| {
                row["name"]
                    .as_str()
                    .and_then(|name| name.strip_prefix(&prefix))
                    .is_none_or(|suffix| !toolkit::drive::COMMANDS.contains(&suffix))
            });
        }
        let thread = std::thread::current();
        let mut host = create(bundle, Arc::new(move || thread.unpark()));
        let context = egui::Context::default();
        crate::appearance::install(&context);
        loop {
            host.pump(&context);
            if let Some(error) = host
                .bus
                .startup_error
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .clone()
            {
                return Err(eframe::Error::AppCreation(Box::new(std::io::Error::other(
                    error,
                ))));
            }
            if !host.error.is_empty() {
                return Err(eframe::Error::AppCreation(Box::new(std::io::Error::other(
                    host.error.clone(),
                ))));
            }
            if host.model["window_close"] == true
                || host
                    .bus
                    .close_requested
                    .load(std::sync::atomic::Ordering::Acquire)
            {
                break;
            }
            std::thread::park_timeout(Duration::from_millis(20));
        }
        return Ok(());
    }
    let title = bundle.manifest.title.clone();
    let size = bundle.manifest.size;
    let service = bundle.manifest.app().to_owned();
    let live_service =
        std::env::var("MIX_VIU_SERVICE").unwrap_or_else(|_| bundle.manifest.service.clone());
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
    let mut viewport = egui::ViewportBuilder::default()
        .with_transparent(bundle.manifest.frame.is_some())
        .with_decorations(bundle.manifest.frame.is_none())
        .with_inner_size(size)
        .with_app_id(app_id);
    if let Some(version) = bundle.manifest.saved_version {
        viewport = crate::saved_state::Saved::load(bundle.manifest.app(), version, false)
            .store
            .window
            .viewport(viewport);
    }
    eframe::run_native(
        &title,
        eframe::NativeOptions {
            viewport,
            ..Default::default()
        },
        Box::new(move |cc| {
            toolkit::decorations::initialise(cc);
            toolkit::decorations::initialise_window(&cc.egui_ctx, cc.winit_window().cloned());
            crate::appearance::install(&cc.egui_ctx);
            toolkit::drive::install(&cc.egui_ctx, &service);
            toolkit::drive::set_service(&cc.egui_ctx, Some(&live_service));
            if let Some(state) = cc.wgpu_render_state.as_ref() {
                state
                    .renderer
                    .write()
                    .callback_resources
                    .insert(state.target_format);
            }
            let wake = owner_wake(cc.egui_ctx.clone());
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
fn run_headless(bundle: Bundle) -> eframe::Result {
    let (tx, rx) = std::sync::mpsc::channel();
    let wake: crate::runtime::Wake = Arc::new(move || {
        let _ = tx.send(());
    });
    let mut host = create(bundle, wake);
    let ctx = egui::Context::default();
    crate::appearance::install(&ctx);
    loop {
        let output = ctx.run_ui(egui::RawInput::default(), |ui| host.pump(ui.ctx()));
        if host
            .bus
            .close_requested
            .load(std::sync::atomic::Ordering::Acquire)
            || host.model_close_ready()
        {
            output.drop_without_applying_deltas();
            break;
        }
        let repaint = output
            .viewport_output
            .values()
            .map(|o| o.repaint_delay)
            .min()
            .unwrap_or(Duration::MAX);
        let delay = host
            .tick
            .map(|_| host.next_tick.saturating_duration_since(Instant::now()))
            .unwrap_or(Duration::MAX)
            .min(repaint);
        output.drop_without_applying_deltas();
        if delay == Duration::MAX {
            if rx.recv().is_err() {
                break;
            }
        } else {
            let _ = rx.recv_timeout(delay);
        }
    }
    let failed = host
        .bus
        .startup_error
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .clone();
    if let Some(error) = failed {
        return Err(eframe::Error::AppCreation(Box::new(std::io::Error::other(
            error,
        ))));
    }
    Ok(())
}

fn run_terminal_headless(bundle: Bundle) -> Result<(), String> {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let wake: crate::runtime::Wake = Arc::new(move || {
        let _ = tx.try_send(());
    });
    let mut host = create(bundle, wake);
    let ctx = egui::Context::default();
    loop {
        host.pump(&ctx);
        if !host.error.is_empty() {
            return Err(host.error.clone());
        }
        host.deliveries
            .retain(|d| d.load(std::sync::atomic::Ordering::Acquire) == 0);
        if host
            .bus
            .close_requested
            .load(std::sync::atomic::Ordering::Acquire)
            || (host.model["window_close"] == true
                && host.pending.is_empty()
                && host.inflight.is_empty()
                && host.deliveries.is_empty())
        {
            return Ok(());
        }
        let deadline = host
            .timers
            .deadline()
            .into_iter()
            .chain(
                host.tick
                    .filter(|_| host.ready && host.inflight.is_empty())
                    .map(|_| host.next_tick),
            )
            .min();
        match deadline {
            Some(at) => {
                let _ = rx.recv_timeout(at.saturating_duration_since(Instant::now()));
            }
            None => rx.recv().map_err(|e| e.to_string())?,
        }
    }
}
// Bus, runtime and clipboard completions are consumed by Host::logic on the
// root window. A deferred viewport may be current when a worker posts work.
// Ambient request_repaint() would then wake only that child, leaving the owner
// queue asleep until unrelated root input arrives.
fn owner_wake(ctx: egui::Context) -> crate::runtime::Wake {
    Arc::new(move || ctx.request_repaint_of(egui::ViewportId::ROOT))
}

struct LayerHost {
    bundle: Option<Bundle>,
    host: Option<Host>,
    views: BTreeMap<String, (SceneView, bool)>,
    contexts: Arc<std::sync::Mutex<Vec<egui::Context>>>,
    reported: BTreeMap<String, crate::layer::SurfaceInfo>,
}
impl crate::layer::LayerProgram for LayerHost {
    fn strict_frame_callbacks(&self) -> bool {
        self.host
            .as_ref()
            .is_some_and(|h| h.model["layer"]["strict_callbacks"] == true)
    }
    fn frame_interval(&self) -> Duration {
        Duration::from_millis(
            self.host
                .as_ref()
                .and_then(|h| h.model["layer"]["frame_ms"].as_u64())
                .unwrap_or(0),
        )
    }
    fn visibility(&mut self, ctx: &egui::Context, visible: bool) {
        if let (Some(host), Some(output)) = (
            &mut self.host,
            ctx.data(|d| d.get_temp::<String>(egui::Id::unique("viu-layer-output"))),
        ) {
            let changed = host
                .outputs
                .get(&output)
                .is_none_or(|state| state["visible"] != visible);
            if let Some(state) = host.outputs.get_mut(&output) {
                state["visible"] = json!(visible);
            }
            host.ui_state.set_outputs(&host.outputs);
            if changed {
                let _ = host.runtime.notify(
                    "viu.layer.visibility".into(),
                    json!({"output":output,"visible":visible}),
                );
            }
        }
    }
    fn should_close(&self) -> bool {
        if self.host.as_ref().is_some_and(|h| {
            h.require_bus
                && h.bus
                    .registration_failed
                    .load(std::sync::atomic::Ordering::Acquire)
        }) {
            std::process::exit(1);
        }
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
        if let Some(host) = &self.host {
            let _ = crate::appearance::install_theme(ctx, &host.file_theme);
        }
    }
    fn show(&mut self, ui: &mut egui::Ui, surface: &crate::layer::SurfaceInfo) {
        let host = self.host.as_mut().unwrap();
        let output = surface.output.clone().unwrap_or_default();
        ui.ctx()
            .data_mut(|d| d.insert_temp(egui::Id::unique("viu-layer-output"), output.clone()));
        let frames = host
            .outputs
            .get(&output)
            .and_then(|o| o["presented_frames"].as_u64())
            .unwrap_or(0);
        let state = host
            .outputs
            .entry(output.clone())
            .or_insert_with(|| json!({}));
        let changed = state["width"] != surface.width
            || state["height"] != surface.height
            || state["scale"] != surface.scale;
        state["width"] = json!(surface.width);
        state["height"] = json!(surface.height);
        state["scale"] = json!(surface.scale);
        state["presented_frames"] = json!(frames);
        host.ui_state.set_outputs(&host.outputs);
        if changed {
            let _=host.runtime.notify("viu.layer.configure".into(),json!({"output":output,"width":surface.width,"height":surface.height,"scale":surface.scale}));
        }
        let (view, loaded) = self
            .views
            .entry(output.clone())
            .or_insert_with(|| (loading_view(), false));
        std::mem::swap(&mut host.view, view);
        std::mem::swap(&mut host.loaded, loaded);
        host.output = Some(output);
        // Layer hosts use the same owner loop; eframe's Frame is not needed.
        host.pump(ui.ctx());
        if host.model["layer"]["observe_input"] == true {
            if let Some(pos) = ui.input(|i| {
                i.pointer
                    .button_pressed(egui::PointerButton::Primary)
                    .then(|| i.pointer.interact_pos())
                    .flatten()
            }) {
                let _ = host.runtime.notify(
                    "viu.layer.input".into(),
                    json!({"output":host.output,"x":pos.x,"y":pos.y}),
                );
            }
        }
        if let Some(session) = &host.session {
            if session.snapshot["state"].is_object() {
                let stamp = json!([
                    session.snapshot["state"]["incarnation"],
                    session.snapshot["state"]["revision"],
                    session.snapshot["state"]["lost"]
                ]);
                let key = egui::Id::unique("viu-layer-look-revision");
                if ui.ctx().data(|d| d.get_temp::<Value>(key)).as_ref() != Some(&stamp) {
                    if let Ok(common) = crate::appearance::install_session(
                        ui.ctx(),
                        &session.snapshot,
                        &host.app,
                        &host.file_theme,
                        &host.last_ui,
                    ) {
                        host.last_ui = common;
                        ui.ctx().data_mut(|d| d.insert_temp(key, stamp));
                    }
                }
            }
        }
        host.frame(ui);
        if let Some(state) = host.outputs.get_mut(host.output.as_ref().unwrap()) {
            let raster = host.view.raster_states();
            if let Some(shows) = host.model["layer"]["content"].as_str() {
                state["shows"] = json!(shows);
                state["error"] = Value::Null;
            } else if raster["root"].is_object() {
                for key in ["shows", "error", "fit"] {
                    state[key] = raster["root"][key].clone();
                }
            }
        }
        std::mem::swap(&mut host.view, view);
        std::mem::swap(&mut host.loaded, loaded);
    }
    fn removed(&mut self, ctx: &egui::Context, surface: &crate::layer::SurfaceInfo) {
        let output = surface.output.clone().unwrap_or_default();
        self.views.remove(&output);
        if let Some(host) = &mut self.host {
            host.outputs.remove(&output);
            let _ = host
                .runtime
                .notify("viu.layer.removed".into(), json!({"output":output}));
            host.ui_state.set_outputs(&host.outputs);
        }
        self.contexts.lock().unwrap().retain(|c| c != ctx);
    }
    fn presented(&mut self, surface: &crate::layer::SurfaceInfo) {
        let output = surface.output.clone().unwrap_or_default();
        if let Some(host) = &mut self.host {
            if host.model["layer"]["observe_presentation"] == true
                && self.reported.get(&output) != Some(surface)
            {
                self.reported.insert(output.clone(), surface.clone());
                let _=host.runtime.notify("viu.layer.presented".into(),json!({"output":output,"width":surface.width,"height":surface.height,"scale":surface.scale}));
            }
            if let Some(state) = host.outputs.get_mut(&output) {
                state["presented_view_frames"] =
                    json!(if state["_presented_shows"] == state["shows"] {
                        state["presented_view_frames"].as_u64().unwrap_or(0) + 1
                    } else {
                        1
                    });
                state["_presented_shows"] = state["shows"].clone();
                state["presented_frames"] =
                    json!(state["presented_frames"].as_u64().unwrap_or(0) + 1);
                state["presented_revision"] = json!(
                    self.views
                        .get(&output)
                        .map(|(v, _)| v.model()["serial"].as_u64().unwrap_or(v.revision))
                );
            }
            host.ui_state.set_outputs(&host.outputs);
        }
    }
}
fn loading_view() -> SceneView {
    SceneView::new("---\nscene: 1\nname: loading\ncitizen: loading\n---\n```mix\n{root:{widget:\"text\",text:\"Loading…\"}}\n```\n").expect("static loading scene")
}

#[cfg(test)]
mod tests {
    #[test]
    fn worker_completion_during_child_pass_wakes_root_owner() {
        let ctx = egui::Context::default();
        for _ in 0..3 {
            ctx.run_ui(egui::RawInput::default(), |_| {})
                .drop_without_applying_deltas();
        }
        let wake = super::owner_wake(ctx.clone());
        let requests = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let recorded = requests.clone();
        ctx.set_request_repaint_callback(move |request| {
            recorded.lock().unwrap().push(request.viewport_id);
        });
        let child = egui::ViewportId::from_hash_of("completion-child");
        let mut input = egui::RawInput {
            viewport_id: child,
            ..Default::default()
        };
        input.viewports.insert(child, Default::default());
        ctx.run_ui(input, |ui| {
            assert_eq!(ui.ctx().viewport_id(), child);
            requests.lock().unwrap().clear();
            let wake = wake.clone();
            std::thread::spawn(move || wake()).join().unwrap();
            assert_eq!(*requests.lock().unwrap(), vec![egui::ViewportId::ROOT]);
        })
        .drop_without_applying_deltas();
    }
}
