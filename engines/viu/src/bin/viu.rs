// SPDX-License-Identifier: MIT OR Apache-2.0
use eframe::egui;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    time::{Duration, Instant},
};
use viu::{
    bundle::Bundle,
    input::Input,
    native_bus::{Endpoint, Request},
    runtime::Runtime,
    view::SceneView,
};
type Reply = tokio::sync::oneshot::Sender<(u8, Value)>;
struct Host {
    runtime: Runtime,
    view: SceneView,
    bus: Endpoint,
    pending: BTreeMap<u64, Reply>,
    inflight: BTreeSet<u64>,
    input: Input,
    input_reply: Option<Reply>,
    capture: viu::capture::Capture,
    capture_reply: Option<Reply>,
    ready: bool,
    error: String,
    tick: Option<Duration>,
    next_tick: Instant,
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
        json!({"ready":self.ready,"model":self.view.model(),"revision":self.view.revision,"geometry":geometry,"error":self.error})
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
    fn request(&mut self, r: Request, ctx: &egui::Context) {
        match r.command.as_str() {
            "viu.state" | "viu.ui.tree" => {
                let _ = r.reply.send((0, self.state()));
            }
            "viu.quit" => {
                let _ = r.reply.send((0, json!({"ok":true})));
            }
            _ if !self.ready => {
                let _ = r.reply.send((10, json!({"error_code":"NOT_READY"})));
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
            _ => self.dispatch(r.command, r.args, r.headers, Some(r.reply)),
        }
    }
}
impl eframe::App for Host {
    fn raw_input_hook(&mut self, _: &egui::Context, raw: &mut egui::RawInput) {
        if !self.input.now.is_empty() {
            raw.events.append(&mut self.input.now);
        } else if let Some(mut events) = self.input.later.pop_front() {
            raw.events.append(&mut events);
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) {
        if self
            .bus
            .close_requested
            .load(std::sync::atomic::Ordering::Acquire)
        {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
        while let Some(done) = self.runtime.try_completion() {
            self.inflight.remove(&done.sequence);
            self.error = done.error.unwrap_or_default();
            if self.error.is_empty() {
                if let Some(model) = done.model {
                    match self.view.apply_model(model) {
                        Ok(()) => self.ready = true,
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
                let _ = reply.send(answer);
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
                    let _ = self
                        .runtime
                        .notify_with_headers(n.command, n.args, n.headers);
                }
                Err(_) => break,
            }
        }
        if self.ready {
            for event in self.view.show(ui) {
                self.dispatch(event.command, event.body, BTreeMap::new(), None);
            }
        }
        if let Some(error) = self.view.take_focus_error() {
            self.error = error;
        }
        if self.input.now.is_empty() && self.input.later.is_empty() && self.inflight.is_empty() {
            if let Some(reply) = self.input_reply.take() {
                let _ = reply.send((if self.error.is_empty() { 0 } else { 10 }, self.state()));
            }
        } else {
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
            ui.ctx()
                .request_repaint_after(self.next_tick.saturating_duration_since(Instant::now()));
        }
        if !self.error.is_empty() {
            ui.colored_label(egui::Color32::RED, &self.error);
        }
    }
}
fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) != Some("run") || args.len() < 2 {
        eprintln!("usage: viu run APP_DIR [ARGS]");
        std::process::exit(2);
    }
    let bundle = Bundle::load(std::path::Path::new(&args[1]), &args[2..]).unwrap_or_else(|e| {
        eprintln!("viu: {e}");
        std::process::exit(2)
    });
    let title = bundle.manifest.title.clone();
    let size = bundle.manifest.size;
    let service = bundle.manifest.service.clone();
    let mut commands = bundle.manifest.commands.clone();
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
        ]
        .map(String::from),
    );
    eframe::run_native(
        &title,
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size(size)
                .with_app_id(format!("mixviu.{service}")),
            ..Default::default()
        },
        Box::new(move |cc| {
            viu::icons::install(&cc.egui_ctx);
            let ctx = cc.egui_ctx.clone();
            let wake: viu::runtime::Wake = Arc::new(move || ctx.request_repaint());
            let bus = Endpoint::start(
                service,
                std::env::var("MIXOS_NODED_URL")
                    .unwrap_or_else(|_| "ws://127.0.0.1:4200/ws".into()),
                commands
                    .into_iter()
                    .map(|name| {
                        bus::CommandDescriptor::new(&name, &[], "Mix bundle command", false)
                    })
                    .collect(),
                "viu.quit".into(),
                wake.clone(),
            );
            let mut runtime = Runtime::new(32, wake.clone(), Some(bus.backend()));
            runtime.load(bundle.program).expect("load behaviour");
            let capture_root = std::env::var_os("XDG_RUNTIME_DIR")
                .map(std::path::PathBuf::from)
                .unwrap_or_else(std::env::temp_dir)
                .join("viu-captures");
            Ok(Box::new(Host {
                runtime,
                view: SceneView::new(&bundle.scene).expect("scene"),
                bus,
                pending: BTreeMap::new(),
                inflight: BTreeSet::new(),
                input: Input::default(),
                input_reply: None,
                capture: viu::capture::Capture::new(capture_root, wake),
                capture_reply: None,
                ready: false,
                error: String::new(),
                tick: bundle.manifest.tick_ms.map(Duration::from_millis),
                next_tick: Instant::now(),
            }))
        }),
    )
}
