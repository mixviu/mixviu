// SPDX-License-Identifier: MIT OR Apache-2.0
use eframe::egui;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    time::{Duration, Instant},
};
use crate::{
    bundle::Bundle,
    input::Input,
    native_bus::{Endpoint, Request},
    runtime::Runtime,
    view::SceneView,
};
type Reply = tokio::sync::oneshot::Sender<(u8, Value)>;
struct Host {
    model: Value,
    output: Option<String>,
    runtime: Runtime,
    view: SceneView,
    bus: Endpoint,
    pending: BTreeMap<u64, Reply>,
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
    fn ui(&mut self, ui: &mut egui::Ui, _: &mut eframe::Frame) { self.frame(ui); }
}
impl Host {
    fn frame(&mut self, ui: &mut egui::Ui) {
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
                    let projected=self.output.as_ref().and_then(|output|model["surfaces"].get(output)).unwrap_or(&model).clone();
                    match self.view.apply_model(projected) {
                        Ok(()) => {self.ready = true;self.model=model;},
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
        if let Some(projected)=self.output.as_ref().and_then(|output|self.model["surfaces"].get(output)).or_else(||self.output.as_ref().map(|_|&self.model)) {
            if self.view.model()!=projected {if let Err(e)=self.view.apply_model(projected.clone()){self.error=e;}}
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

fn create(bundle: Bundle, wake: crate::runtime::Wake) -> Host {
 let service=bundle.manifest.service.clone();
 let mut commands=bundle.manifest.commands.clone();
 commands.extend(["viu.state","viu.ui.tree","viu.ui.click","viu.ui.focus","viu.ui.text","viu.ui.key","viu.ui.hover","viu.ui.scroll","viu.ui.drag","viu.capture","viu.quit"].map(String::from));
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
            Host {
                model: json!({}), output: None,
                runtime,
                view: SceneView::new(&bundle.scene).expect("scene"),
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
            }

}

pub fn run(args: Vec<String>) -> eframe::Result {
    if args.first().map(String::as_str)!=Some("run") || args.len()<2 {eprintln!("usage: viu run APP_DIR [ARGS]");std::process::exit(2);}
    let bundle=Bundle::load(std::path::Path::new(&args[1]),&args[2..]).unwrap_or_else(|e|{eprintln!("viu: {e}");std::process::exit(2)});
    if bundle.manifest.surface=="background" {
        let mut options=crate::layer::LayerOptions::background(&bundle.manifest.service);
        options.exclusive_zone=0;
        let scene=bundle.scene.clone();
        let layer=LayerHost{bundle:Some(bundle),host:None,scene,views:BTreeMap::new(),contexts:Arc::new(std::sync::Mutex::new(Vec::new()))};
        if let Err(e)=crate::layer::run(options,layer){eprintln!("viu layer: {e}");std::process::exit(1);}
        return Ok(());
    }
    let title=bundle.manifest.title.clone();let size=bundle.manifest.size;let service=bundle.manifest.service.clone();
    eframe::run_native(&title,eframe::NativeOptions{viewport:egui::ViewportBuilder::default().with_inner_size(size).with_app_id(format!("mixviu.{service}")),..Default::default()},Box::new(move|cc|{
        crate::icons::install(&cc.egui_ctx);
        let ctx=cc.egui_ctx.clone();
        Ok(Box::new(create(bundle,Arc::new(move||ctx.request_repaint()))))
    }))
}
struct LayerHost {
    bundle: Option<Bundle>, host: Option<Host>, scene: String,
    views: BTreeMap<String,SceneView>,
    contexts: Arc<std::sync::Mutex<Vec<egui::Context>>>,
}
impl crate::layer::LayerProgram for LayerHost {
    fn should_close(&self)->bool {self.host.as_ref().is_some_and(|h|h.bus.close_requested.load(std::sync::atomic::Ordering::Acquire))}
    fn setup(&mut self,ctx:&egui::Context){
        crate::icons::install(ctx);
        self.contexts.lock().unwrap().push(ctx.clone());
        if let Some(bundle)=self.bundle.take(){
            let contexts=self.contexts.clone();
            self.host=Some(create(bundle,Arc::new(move||{for ctx in contexts.lock().unwrap().iter(){ctx.request_repaint();}})));
        }
    }
    fn show(&mut self,ui:&mut egui::Ui,surface:&crate::layer::SurfaceInfo){
        let host=self.host.as_mut().unwrap();
        let output=surface.output.clone().unwrap_or_default();
        let view=self.views.entry(output.clone()).or_insert_with(||SceneView::new(&self.scene).expect("layer scene"));
        std::mem::swap(&mut host.view,view);
        host.output=Some(output);
        // Layer hosts use the same owner loop; eframe's Frame is not needed.
        host.frame(ui);
        std::mem::swap(&mut host.view,view);
    }
    fn removed(&mut self,ctx:&egui::Context,surface:&crate::layer::SurfaceInfo){
        self.views.remove(&surface.output.clone().unwrap_or_default());
        self.contexts.lock().unwrap().retain(|c|c!=ctx);
    }
}
