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
struct Host {
    settings_reload: u64,
    facilities: crate::app_facilities::Facilities,
    platform: crate::platform::Platform,
    paste_waits: std::collections::VecDeque<String>,
    platform_results: std::collections::VecDeque<Value>,
    service: String,
    native_drive: bool,
    drive_next: u64,
    drive_pending: BTreeMap<u64, Reply>,
    ui_state: crate::ui_state::UiState,
    measurements: crate::ui_state::Measurements,
    input_at: Option<Instant>,
    calls: crate::calls::Calls,
    timers: crate::timers::Timers,
    timer_results: std::collections::VecDeque<Value>,
    documents: crate::documents::Documents,
    framed: bool,
    title: String,
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
    inflight: BTreeSet<u64>,
    input: Input,
    input_reply: Option<Reply>,
    capture: crate::capture::Capture,
    capture_reply: Option<Reply>,
    ready: bool,
    error: String,
    tick: Option<Duration>,
    next_tick: Instant,
    session: Option<crate::session::Session>,
    session_appearance: bool,
    file_theme: toolkit::Theme,
    last_ui: settings::model::CommonUi,
    session_update: Option<Value>,
    noticed_drops: u64,
}
impl Host {
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
        json!({"ready":self.ready,"model":self.view.model(),"revision":self.view.revision,"geometry":geometry,"outputs":self.outputs,"error":self.error})
    }
    fn dispatch(
        &mut self,
        command: String,
        body: Value,
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
        match self.runtime.event_with_headers(command, body, headers) {
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
        if !self.commands.contains(&r.command) {
            let _ = r.reply.send((
                10,
                json!({"error_code":"UNKNOWN_COMMAND","command":r.command}),
            ));
            return;
        }
        if self.native_drive {
            if let Some((rc, body)) =
                crate::debug::request(&self.service, &r.command, &r.args.to_string())
            {
                let _ = r.reply.send((rc, body));
                return;
            }
            let prefix = format!("{}.", self.service);
            if let Some(suffix) = r.command.strip_prefix(&prefix) {
                if toolkit::drive::COMMANDS.contains(&suffix) {
                    if suffix == "window" && r.args["action"] == "close" {
                        r.command = "app.quit".into();
                    } else {
                        self.drive_next += 1;
                        let id = self.drive_next;
                        let answer = toolkit::drive::request(ctx, id, suffix, &r.args);
                        if let Some(answer) = answer {
                            let _ = r.reply.send((answer.rc, answer.body));
                        } else {
                            self.drive_pending.insert(id, r.reply);
                            ctx.request_repaint();
                        }
                        return;
                    }
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
            _ => match self.calls.begin(r.reply, r.close_after_reply) {
                Ok(token) => {
                    let mut headers = r.headers;
                    headers.insert("viu-call".into(), token);
                    self.dispatch(r.command, r.args, headers, None);
                }
                Err(reply) => {
                    let _ = reply.send((10, json!({"error_code":"BUSY"})));
                }
            },
        }
    }
}
impl eframe::App for Host {
    fn raw_input_hook(&mut self, _: &egui::Context, raw: &mut egui::RawInput) {
        if !self.paste_waits.is_empty() {
            raw.events.retain(|event| {
                if let egui::Event::Paste(text) = event {
                    if let Some(id) = self.paste_waits.pop_front() {
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
    fn frame(&mut self, ui: &mut egui::Ui) {
        self.facilities.prepare(ui.ctx(), &self.bus);
        for request in self.platform.take() {
            if request["kind"] == "copy" {
                ui.ctx()
                    .copy_text(request["body"]["text"].as_str().unwrap_or("").into());
            } else {
                self.paste_waits
                    .push_back(request["id"].as_str().unwrap_or("").into());
                ui.ctx()
                    .send_viewport_cmd(egui::ViewportCommand::RequestPaste);
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
        if self.native_drive {
            toolkit::drive::set_service(
                ui.ctx(),
                self.bus.client().as_ref().map(|_| self.service.as_str()),
            );
            for answer in toolkit::drive::logic(ui.ctx()) {
                if let Some(reply) = self.drive_pending.remove(&answer.id) {
                    let _ = reply.send((answer.rc, answer.body));
                }
            }
        }
        for token in self.calls.cancelled() {
            let _ = self
                .runtime
                .notify("viu.call.cancelled".into(), json!({"token":token}));
        }
        let (due, deadline) = self.timers.poll();
        self.timer_results.extend(due);
        if let Some(at) = deadline {
            ui.ctx()
                .request_repaint_after(at.saturating_duration_since(Instant::now()));
        }
        while let Some(body) = self.timer_results.front() {
            if self
                .runtime
                .notify("viu.timer".into(), body.clone())
                .is_err()
            {
                ui.ctx().request_repaint();
                break;
            }
            self.timer_results.pop_front();
        }
        if let Some(session) = &mut self.session {
            let drops = self
                .bus
                .dropped_notices
                .load(std::sync::atomic::Ordering::Relaxed);
            if drops != self.noticed_drops {
                session.hint("bus.connected", &json!({}));
                self.noticed_drops = drops;
            }
            if let Some(mut update) = session.poll() {
                match crate::appearance::install_session(
                    ui.ctx(),
                    &update,
                    &self.service,
                    &self.file_theme,
                    &self.last_ui,
                ) {
                    Ok(value) => {
                        self.last_ui = value;
                        self.session_appearance = true;
                        update["appearance"]["mode"]=json!(if ui.visuals().dark_mode {"dark"}else{"light"});
                    }
                    Err(error) => self.error = error,
                }
                self.session_update = Some(update);
            }
            if let Some(at) = session.deadline() {
                ui.ctx()
                    .request_repaint_after(at.saturating_duration_since(Instant::now()));
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
                ui.ctx().request_repaint();
            }
        }
        for (_, image) in self.clipboard.drain() {
            if let Some(reply) = self.clipboard_reply.take() {
                let result = if let Some(image) = image {
                    ui.ctx().copy_image(image);
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
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
        while let Some(done) = self.runtime.try_completion() {
            self.measurements
                .model(done.elapsed.as_micros().min(u64::MAX as u128) as u64);
            self.inflight.remove(&done.sequence);
            self.error = done.error.unwrap_or_default();
            if self.error.is_empty() {
                if let Some(model) = done.model {
                    let projected = self
                        .output
                        .as_ref()
                        .and_then(|output| model["surfaces"].get(output))
                        .unwrap_or(&model)
                        .clone();
                    let appearance = if self.framed
                        && !self.session_appearance
                        && (!self.loaded || self.model["appearance"] != model["appearance"])
                    {
                        crate::appearance::resolve(&model["appearance"]).map(Some)
                    } else {
                        Ok(None)
                    };
                    let result = appearance.as_ref().map_err(Clone::clone).and_then(|_| {
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
                                self.view.set_native_menus(self.native_drive);
                                self.loaded = true;
                            })
                        }
                    });
                    match result {
                        Ok(()) => {
                            if let Some(style) = appearance.expect("validated appearance") {
                                crate::appearance::install_style(
                                    ui.ctx(),
                                    &model["appearance"],
                                    style,
                                );
                            }
                            self.ready = true;
                            self.model = model;
                        }
                        Err(e) => self.error = e,
                    }
                }
            }
            if let Some(token) = self.pending.remove(&done.sequence) {
                let answer = if !self.error.is_empty() {
                    (10, json!({"error":self.error}))
                } else if let Some(r) = done.reply {
                    (r.rc, r.body)
                } else {
                    (0, self.state())
                };
                self.calls.complete(&token, answer.0, answer.1);
            }
        }
        for _ in 0..32 {
            match self.bus.requests.try_recv() {
                Ok(r) => self.request(r, ui.ctx()),
                Err(_) => break,
            }
        }
        for _ in 0..32 {
            match self.bus.notices.try_recv() {
                Ok(n) => {
                    if n.command == "theme.changed" {
                        self.file_theme = toolkit::Theme::load();
                    }
                    if let Some(session) = &mut self.session {
                        session.hint(&n.command, &n.args);
                        ui.ctx().request_repaint();
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
                match if self.framed {
                    crate::gold_frame::mount(&self.scene_source, projected.clone())
                } else {
                    SceneView::with_model(&self.scene_source, projected.clone())
                } {
                    Ok(view) => {
                        self.view = view;
                        self.view.set_documents(self.documents.clone());
                        self.view.set_native_menus(self.native_drive);
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
            if let Some(mode)=self.model["local_mode"].as_str() {if let Err(error)=crate::appearance::local_mode(ui.ctx(),mode) {self.error=error;}}
            let reload=self.model["settings_reload"].as_u64().unwrap_or(0);
            if reload!=self.settings_reload {self.settings_reload=reload;self.file_theme=toolkit::Theme::load();if let Some(session)=&mut self.session {session.hint("theme.changed",&json!({}));}ui.ctx().request_repaint();}
            if let Some(bindings) = self.model["shortcuts"]
                .as_array()
                .filter(|_| !toolkit::menu::is_open(ui.ctx()))
            {
                let bindings = bindings.clone();
                for binding in bindings {
                    if binding["enabled"] == false {
                        continue;
                    }
                    if let (Some(chord), Some(id), Some(command)) = (
                        binding["chord"].as_str(),
                        binding["id"].as_str(),
                        self.model["on_shortcut"].as_str(),
                    ) {
                        if crate::input::consume_chord(ui.ctx(), chord) {
                            self.dispatch(command.into(), json!({"id":id}), BTreeMap::new(), None);
                        }
                    }
                }
            }
            if self.framed {
                let settings = self.model["window_settings"].clone();
                crate::rich::configure_menu(ui.ctx(), &settings, None);
                if self.native_drive && !self.session_appearance {
                    let mut chrome = toolkit::chrome::Chrome::of(ui.ctx());
                    let style = ui.ctx().global_style();
                    chrome.palette = toolkit::chrome::Palette::from_visuals(&style.visuals);
                    chrome.mode = if style.visuals.dark_mode {
                        design::Mode::Dark
                    } else {
                        design::Mode::Light
                    };
                    chrome.metrics = toolkit::chrome::Metrics::from_egui(&style, &chrome.style);
                    chrome.menu_style = design::MenuStyle::resolve(
                        settings["window.menu_style"].as_str().unwrap_or("bar"),
                    );
                    chrome.menu_side = design::MenuSide::resolve(
                        settings["window.menu_side"].as_str().unwrap_or("left"),
                    );
                    toolkit::chrome::install(ui.ctx(), &chrome);
                    toolkit::chrome::client_side(ui.ctx());
                }
                self.window_frame
                    .show_with_chrome(ui, &self.title, &settings, true, true);
                let slots = self
                    .window_frame
                    .geometry
                    .get("window.header")
                    .map(|rect| BTreeMap::from([("window.header".into(), *rect)]))
                    .unwrap_or_default();
                if let Err(e) = self.view.set_slots(slots) {
                    self.error = e;
                }
            }
            if self.framed {
                ui.painter().rect_filled(
                    ui.max_rect(),
                    0.0,
                    toolkit::chrome::Chrome::of(ui.ctx()).palette.dock,
                );
            }
            let started = Instant::now();
            for event in self.view.show(ui) {
                self.dispatch(event.command, event.body, BTreeMap::new(), None);
            }
            self.measurements.view(started.elapsed().as_micros() as u64);
            if let Some(at) = self.input_at.take() {
                self.measurements.next(at.elapsed().as_micros() as u64);
            }
            if self.framed {
                self.window_frame
                    .finish_captions(ui, self.view.allocated_region("gold-captions"));
                self.window_frame.edges(ui);
                self.view
                    .geometry
                    .extend(self.window_frame.geometry.clone());
                if self.window_frame.close_requested {
                    self.dispatch("viu.frame.close".into(), json!({}), BTreeMap::new(), None);
                }
                if self.model["window_close"] == true {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                }
            }
        }
        if let Some(error) = self.view.take_focus_error() {
            self.error = error;
        }
        if self.input.now.is_empty() && self.input.later.is_empty() && self.inflight.is_empty() {
            if let Some(reply) = self.input_reply.take() {
                let _ = reply.send((if self.error.is_empty() { 0 } else { 10 }, self.state()));
            }
        } else if !self.input.now.is_empty() || !self.input.later.is_empty() {
            ui.ctx().request_repaint();
        }
        if let Some((_, result)) = self.capture.poll(ui.ctx()) {
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
            ui.ctx()
                .request_repaint_after(if wait.is_zero() { tick } else { wait });
        }
        if !self.error.is_empty() {
            ui.colored_label(egui::Color32::RED, &self.error);
        }
        self.ui_state.set(
            ui.ctx().content_rect(),
            &self.view.geometry,
            self.measurements.snapshot(),
        );
        self.facilities.finish(ui.ctx());
        if self.native_drive {
            for answer in toolkit::drive::finish(ui.ctx()) {
                if let Some(reply) = self.drive_pending.remove(&answer.id) {
                    let _ = reply.send((answer.rc, answer.body));
                }
            }
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
                .into_iter()
                .flatten()
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
    let declared = commands.iter().cloned().collect();
    let bus = Endpoint::start(
        service,
        std::env::var("MIXOS_NODED_URL").unwrap_or_else(|_| "ws://127.0.0.1:4200/ws".into()),
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
    );
    let documents = crate::documents::Documents::default();
    let timers = crate::timers::Timers::default();
    let calls = crate::calls::Calls::default();
    let ui_state = crate::ui_state::UiState::default();
    let platform = crate::platform::Platform::default();
    let services = crate::runtime::Services {
        max_string_bytes: bundle.manifest.max_string_bytes,
        platform: Some(platform.clone()),
        ui_state: Some(ui_state.clone()),
        macros: bundle
            .manifest
            .macros
            .then(|| macros::Session::for_app(&bundle.manifest.service)),
        calls: Some(calls.clone()),
        timers: Some(timers.clone()),
        documents: Some(documents.clone()),
        ..Default::default()
    };
    let mut jobs = crate::jobs::Jobs::new(4, wake.clone());
    jobs.set_services(services.clone());
    let session = bundle.manifest.follow_settings.then(|| {
        crate::session::Session::new(
            bundle
                .root
                .join("../../libraries/viu/mix/session-runtime.mix"),
            &bundle.manifest.service,
            std::fs::read_to_string("/proc/sys/kernel/hostname")
                .unwrap_or_else(|_| "host".into())
                .trim(),
            wake.clone(),
            bus.backend(),
        )
        .expect("start settings follower")
    });
    jobs.set_native_backend(bus.backend_maker());
    let mut runtime = Runtime::with_services(
        32,
        wake.clone(),
        Some(jobs.backend(Some(bus.backend()))),
        services,
    );
    runtime.load(bundle.program).expect("load behaviour");
    let capture_root = std::env::var_os("XDG_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join("viu-captures");
    Host {
        settings_reload: 0,
        facilities,
        platform,
        paste_waits: Default::default(),
        platform_results: Default::default(),
        service: bundle.manifest.service.clone(),
        native_drive: bundle.manifest.native_drive,
        drive_next: 0,
        drive_pending: BTreeMap::new(),
        ui_state,
        measurements: Default::default(),
        input_at: None,
        calls,
        timers,
        timer_results: Default::default(),
        documents,
        framed: bundle.manifest.frame.is_some(),
        title: bundle.manifest.title.clone(),
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
        inflight: BTreeSet::new(),
        input: Input::default(),
        input_reply: None,
        capture: crate::capture::Capture::new(capture_root, wake),
        capture_reply: None,
        ready: false,
        error: String::new(),
        tick: bundle.manifest.tick_ms.map(Duration::from_millis),
        next_tick: Instant::now(),
        session,
        session_update: None,
        session_appearance: false,
        file_theme: toolkit::Theme::load(),
        last_ui: settings::model::CommonUi::default(),
        noticed_drops: 0,
    }
}

pub fn run(mut args: Vec<String>) -> eframe::Result {
    crate::debug::init("info", &mut args).unwrap_or_else(|e| {
        eprintln!("viu: {e}");
        std::process::exit(2)
    });
    if args.first().is_some_and(|p| p.ends_with(".mix")) {
        std::process::exit(crate::macro_host::run(
            args[0].clone().into(),
            args[1..].to_vec(),
        ));
    }
    if args.first().map(String::as_str) != Some("run") || args.len() < 2 {
        eprintln!("usage: viu run APP_DIR [ARGS]");
        std::process::exit(2);
    }
    let bundle = Bundle::load(std::path::Path::new(&args[1]), &args[2..]).unwrap_or_else(|e| {
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
    let title = bundle.manifest.title.clone();
    let size = bundle.manifest.size;
    let service = bundle.manifest.service.clone();
    let native_drive = bundle.manifest.native_drive;
    eframe::run_native(
        &title,
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size(size)
                .with_app_id(format!("mixviu.{service}")),
            ..Default::default()
        },
        Box::new(move |cc| {
            crate::appearance::install(&cc.egui_ctx);
            if native_drive {
                toolkit::drive::install(&cc.egui_ctx, &service);
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
