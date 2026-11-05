// SPDX-License-Identifier: MIT OR Apache-2.0
mod controller;
mod native_bus;
#[cfg(test)]
mod tests;
use eframe::egui;
use native_bus::{NativeBus, Request};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
    sync::Arc,
};
use viu::{
    runtime::{Program, Runtime},
    view::SceneView,
    widgets::Widget,
};

struct Gallery {
    model: Value,
    view: Option<SceneView>,
    definitions: Vec<Widget>,
    scene_source: String,
    runtime: Option<Runtime>,
    bus: NativeBus,
    pending: BTreeMap<u64, tokio::sync::oneshot::Sender<(u8, Value)>>,
    inflight: BTreeSet<u64>,
    input: Vec<egui::Event>,
    later_input: std::collections::VecDeque<Vec<egui::Event>>,
    input_replies: Vec<(bool, tokio::sync::oneshot::Sender<(u8, Value)>)>,
    witness: Option<PathBuf>,
    error: String,
    mode: String,
}
impl Gallery {
    fn definitions(&self) -> Vec<Widget> {
        self.definitions.clone()
    }
    fn state(&self) -> Value {
        let geometry: serde_json::Map<String, Value> = self
            .view
            .as_ref()
            .into_iter()
            .flat_map(|v| v.geometry.iter())
            .filter(|(id, _)| {
                self.definitions
                    .iter()
                    .any(|w| id.as_str() == w.id || id.starts_with(&format!("{}.", w.id)))
            })
            .map(|(id, r)| {
                (
                    id.clone(),
                    json!({"min":[r.min.x,r.min.y],"max":[r.max.x,r.max.y]}),
                )
            })
            .collect();
        json!({"ready":self.ready(),"model":self.model,"geometry":geometry,"error":self.error,"pending":self.pending.len()})
    }
    fn ready(&self) -> bool {
        self.model["widgets"]
            .as_array()
            .is_some_and(|w| !w.is_empty())
    }
    fn publish(&mut self, model: Value) -> Result<(), String> {
        controller::validate_model(&model)?;
        let definitions = model["widgets"]
            .as_array()
            .ok_or("missing widgets")?
            .iter()
            .map(Widget::decode)
            .collect::<Result<Vec<_>, _>>()?;
        match &mut self.view {
            Some(view) => view.apply_model(model.clone())?,
            None => {
                self.view = Some(SceneView::with_model(
                    &self.scene_source,
                    model.clone(),
                )?)
            }
        }
        self.definitions = definitions;
        self.model = model;
        Ok(())
    }
    fn execute(&mut self, body: Value, reply: Option<tokio::sync::oneshot::Sender<(u8, Value)>>) {
        let admission = controller::validate(&self.definitions, &body).and_then(|()| {
            if body["action"] == "reset" {
                return Ok(());
            }
            let widget = self
                .definitions
                .iter()
                .find(|w| Some(w.id.as_str()) == body["id"].as_str())
                .ok_or("unknown widget")?;
            widget.admit(
                &self.model,
                body["action"].as_str().unwrap_or(""),
                &body["value"],
            )
        });
        if let Err(error) = admission {
            if let Some(reply) = reply {
                let _ = reply.send((10, json!({"error":error})));
            } else {
                self.error = error;
            }
            return;
        }
        if let Some(runtime) = &mut self.runtime {
            match runtime.event("gallery.execute".into(), body) {
                Ok(sequence) => {
                    self.inflight.insert(sequence);
                    if let Some(reply) = reply {
                        self.pending.insert(sequence, reply);
                    }
                }
                Err(e) => {
                    self.error = format!("{e:?}");
                    if let Some(reply) = reply {
                        let _ = reply.send((10, json!({"error_code":"BUSY"})));
                    }
                }
            }
        } else {
            let mut candidate = self.model.clone();
            controller::apply(&mut candidate, &body);
            let result = self.publish(candidate);
            if let Err(error) = &result {
                self.error = error.clone();
            } else {
                self.error.clear();
            }
            if let Some(reply) = reply {
                let _ = reply.send((if result.is_ok() { 0 } else { 10 }, self.state()));
            }
        }
    }
    fn request(&mut self, request: Request, ctx: &egui::Context) {
        let Request {
            command,
            body,
            reply,
        } = request;
        if !request_ready(self.ready(), &command) {
            let _ = reply.send((10, json!({"error_code":"NOT_READY"})));
            return;
        }
        if let Some(suffix) = command.strip_prefix("gallery.widget.") {
            if let Some((id, action)) = suffix.split_once('.') {
                self.execute(
                    json!({"id":id,"action":action,"value":body["value"]}),
                    Some(reply),
                );
                return;
            }
        }
        match command.as_str() {
            "gallery.state" | "gallery.ui.tree" => {
                let _ = reply.send((0, self.state()));
            }
            "gallery.describe" => {
                let widgets=self.definitions().iter().map(|w|json!({"definition":w.encode(),"actions":controller::actions(w),"commands":controller::actions(w).iter().map(|action|format!("gallery.widget.{}.{action}",w.id)).collect::<Vec<_>>(),"ui_actions":["focus","hover","click","key","text","scroll","drag"]})).collect::<Vec<_>>();
                let _=reply.send((0,json!({"schema":"viu.gallery.v1","mode":self.mode,"icons":viu::icons::Icon::ALL.iter().map(|i|i.name()).collect::<Vec<_>>(),"widgets":widgets,"execute":{"id":"widget ID","action":"set|activate|submit|accept|cancel|expand|sort|reset","value":"typed widget value"},"ui":{"focus":"{id}","hover":"{id}","click":"{id,button?,double?}","key":"{key,ctrl?,shift?,alt?}","text":"{text}","scroll":"{id,delta}","drag":"{id,fraction:0..1}"},"endpoint":"state/model applied; not physical presentation"})));
            }
            "gallery.execute" => self.execute(body, Some(reply)),
            "gallery.quit" => {
                let _ = reply.send((0, json!({"ok":true})));
            }
            _ => match self.ui_command(&command, &body, ctx) {
                Ok(()) => {
                    self.input_replies
                        .push((command == "gallery.ui.focus", reply));
                    ctx.request_repaint();
                }
                Err(error) => {
                    let _ = reply.send((10, json!({"error":error})));
                }
            },
        }
    }
    fn ui_command(
        &mut self,
        command: &str,
        body: &Value,
        ctx: &egui::Context,
    ) -> Result<(), String> {
        let modifiers = egui::Modifiers {
            ctrl: body["ctrl"].as_bool().unwrap_or(false),
            command: body["ctrl"].as_bool().unwrap_or(false),
            shift: body["shift"].as_bool().unwrap_or(false),
            alt: body["alt"].as_bool().unwrap_or(false),
            ..Default::default()
        };
        match command {
            "gallery.ui.key" => {
                let key = body["key"]
                    .as_str()
                    .and_then(egui::Key::from_name)
                    .ok_or("unknown key")?;
                for pressed in [true, false] {
                    self.input.push(egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers,
                    });
                }
            }
            "gallery.ui.text" => {
                let text = body["text"]
                    .as_str()
                    .filter(|t| t.len() <= 4096)
                    .ok_or("text limit is 4096 bytes")?;
                self.input.push(egui::Event::Text(text.into()));
            }
            "gallery.ui.focus" | "gallery.ui.hover" | "gallery.ui.click" | "gallery.ui.scroll"
            | "gallery.ui.drag" => {
                let id = body["id"].as_str().ok_or("id required")?;
                let rect = *self
                    .view
                    .as_ref()
                    .and_then(|v| v.geometry.get(id))
                    .ok_or("widget not currently visible; open its parent first")?;
                if command == "gallery.ui.focus" {
                    self.view
                        .as_mut()
                        .ok_or("scene not ready")?
                        .request_focus(id)?;
                } else {
                    self.input.push(egui::Event::PointerMoved(rect.center()));
                    if command == "gallery.ui.click" {
                        let button = match body["button"].as_str().unwrap_or("left") {
                            "left" => egui::PointerButton::Primary,
                            "right" => egui::PointerButton::Secondary,
                            _ => return Err("button must be left or right".into()),
                        };
                        let mut frames =
                            click_frames(rect.center(), button, modifiers, body["double"] == true);
                        self.input.extend(frames.pop_front().unwrap());
                        self.later_input.extend(frames);
                    } else if command == "gallery.ui.drag" {
                        let fraction = body["fraction"]
                            .as_f64()
                            .filter(|v| v.is_finite() && (0.0..=1.0).contains(v))
                            .ok_or("drag requires fraction 0..1")?;
                        let start = rect.center();
                        let end = egui::pos2(
                            rect.left() + rect.width() * fraction as f32,
                            rect.center().y,
                        );
                        self.input.push(egui::Event::PointerButton {
                            pos: start,
                            button: egui::PointerButton::Primary,
                            pressed: true,
                            modifiers,
                        });
                        self.later_input
                            .push_back(vec![egui::Event::PointerMoved(end)]);
                        self.later_input.push_back(vec![egui::Event::PointerButton {
                            pos: end,
                            button: egui::PointerButton::Primary,
                            pressed: false,
                            modifiers,
                        }]);
                    } else if command == "gallery.ui.scroll" {
                        let delta = body["delta"]
                            .as_f64()
                            .filter(|v| v.is_finite() && v.abs() <= 10000.0)
                            .ok_or("delta must be finite, within 10000")?;
                        self.input.push(egui::Event::MouseWheel {
                            unit: egui::MouseWheelUnit::Point,
                            delta: egui::vec2(0.0, delta as f32),
                            modifiers,
                            phase: egui::TouchPhase::Move,
                            source: egui::MouseWheelSource::Unknown,
                        });
                    }
                }
            }
            _ => return Err(format!("unknown command {command}")),
        }
        ctx.request_repaint();
        Ok(())
    }
    fn completions(&mut self) {
        loop {
            let done = self.runtime.as_ref().and_then(Runtime::try_completion);
            let Some(done) = done else { break };
            self.inflight.remove(&done.sequence);
            if !self.runtime.as_ref().is_some_and(|r| r.accepts(&done)) {
                if let Some(reply) = self.pending.remove(&done.sequence) {
                    let _=reply.send((10,json!({"error_code":"SUPERSEDED","generation":done.generation,"sequence":done.sequence})));
                }
                continue;
            }
            self.error.clear();
            if let Some(error) = done.error {
                self.error = error;
            } else if let Some(model) = done.model {
                match self.publish(model) {
                    Ok(()) => self.error.clear(),
                    Err(error) => self.error = error,
                }
            }
            if let Some(reply) = self.pending.remove(&done.sequence) {
                let _ = reply.send((if self.error.is_empty() { 0 } else { 10 }, self.state()));
            }
        }
    }
}
impl eframe::App for Gallery {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, input: &mut egui::RawInput) {
        if !self.input.is_empty() {
            input.events.append(&mut self.input);
        } else if let Some(mut later) = self.later_input.pop_front() {
            input.events.append(&mut later);
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if self
            .bus
            .close_requested
            .load(std::sync::atomic::Ordering::Acquire)
        {
            ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
        }
        self.completions();
        // Requests admitted this frame inject raw input at the next frame boundary.
        // Reply only after that boundary and after resulting Mix callbacks settle.
        for _ in 0..32 {
            match self.bus.requests.try_recv() {
                Ok(r) => self.request(r, ui.ctx()),
                Err(_) => break,
            }
        }
        ui.heading("Viu Widget Gallery");
        ui.small("Same widgets · same inputs · independent Rust / Mix controllers");
        ui.small(format!(
            "{} actions · {} activations · {}",
            self.model["serial"],
            self.model["activations"],
            self.model["last"].as_str().unwrap_or("Loading")
        ));
        ui.separator();
        if !self.error.is_empty() {
            ui.colored_label(egui::Color32::RED, &self.error);
        }
        let events = egui::ScrollArea::vertical()
            .id_salt("gallery-page")
            .auto_shrink([false, false])
            .show(ui, |ui| {
                self.view.as_mut().map(|v| v.show(ui)).unwrap_or_default()
            })
            .inner;
        if let Some(error) = self.view.as_mut().and_then(SceneView::take_focus_error) {
            let mut unaffected = Vec::new();
            for (focus, reply) in self.input_replies.drain(..) {
                if focus {
                    let _ = reply.send((
                        10,
                        json!({"error_code":"FOCUS_TARGET_UNAVAILABLE","error":error}),
                    ));
                } else {
                    unaffected.push((focus, reply));
                }
            }
            self.input_replies = unaffected;
        }
        for event in events {
            if event.command == "gallery.execute" {
                self.execute(event.body, None);
            }
            ui.ctx().request_repaint();
        }
        if self.input.is_empty() && self.later_input.is_empty() && self.inflight.is_empty() {
            let state = self.state();
            for (_, reply) in self.input_replies.drain(..) {
                let _ = reply.send((if self.error.is_empty() { 0 } else { 10 }, state.clone()));
            }
        }
        if !self.later_input.is_empty() {
            ui.ctx().request_repaint();
        }
        // The witness is explicit diagnostic I/O, enabled only by the caller.
        if let Some(path) = &self.witness {
            let temporary = path.with_extension("tmp");
            let _ = std::fs::write(&temporary, self.state().to_string())
                .and_then(|_| std::fs::rename(temporary, path));
        }
    }
}
fn click_frames(
    pos: egui::Pos2,
    button: egui::PointerButton,
    modifiers: egui::Modifiers,
    double: bool,
) -> std::collections::VecDeque<Vec<egui::Event>> {
    let mut frames: std::collections::VecDeque<_> = (0..if double { 2 } else { 1 })
        .map(|_| {
            [true, false]
                .into_iter()
                .map(|pressed| egui::Event::PointerButton {
                    pos,
                    button,
                    pressed,
                    modifiers,
                })
                .collect()
        })
        .collect();
    // Egui resolves the hovered widget from the preceding frame. A double
    // click arriving from another control needs that hit-test frame first.
    if double {
        frames.push_front(vec![egui::Event::PointerMoved(pos)]);
    }
    frames
}
fn request_ready(ready: bool, command: &str) -> bool {
    ready || ["gallery.state", "gallery.quit"].contains(&command)
}
fn main() -> eframe::Result {
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).cloned().unwrap_or_else(|| "rust".into());
    assert!(
        ["rust", "mix"].contains(&mode.as_str()),
        "usage: viu-gallery rust|mix [gallery.mix]"
    );
    let path = args
        .get(2)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("apps/gallery/gallery.mix"));
    let service = format!("gallery-{mode}");
    let scene_source=std::fs::read_to_string(path.with_file_name("scene.mix")).expect("gallery scene layout");
    let title = format!(
        "Viu Gallery — {}",
        if mode == "rust" { "Rust" } else { "Mix" }
    );
    eframe::run_native(
        &title,
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([740.0, 820.0])
                .with_app_id(format!("mixviu.gallery.{mode}")),
            ..Default::default()
        },
        Box::new(move |cc| {
            cc.egui_ctx.set_visuals(egui::Visuals::dark());
            viu::icons::install(&cc.egui_ctx);
            let ctx = cc.egui_ctx.clone();
            let wake: Arc<dyn Fn() + Send + Sync> = Arc::new(move || ctx.request_repaint());
            let bus = NativeBus::start(
                service,
                std::env::var("MIXOS_NODED_URL")
                    .unwrap_or_else(|_| "ws://127.0.0.1:4200/ws".into()),
                wake.clone(),
            );
            let runtime = if mode == "mix" {
                let path = path.canonicalize().expect("gallery Mix path");
                let mut runtime = Runtime::new(32, wake, None);
                runtime
                    .load(Program {
                        source: std::fs::read_to_string(&path).expect("gallery Mix source"),
                        path,
                        scene: "gallery".into(),
                        citizen: "gallery-mix".into(),
                    })
                    .expect("gallery load");
                Some(runtime)
            } else {
                None
            };
            let model = if mode == "rust" {
                controller::initial()
            } else {
                json!({})
            };
            let view = if mode == "rust" {
                Some(
                    SceneView::with_model(&scene_source, model.clone())
                        .expect("gallery scene"),
                )
            } else {
                None
            };
            let definitions = if mode == "rust" {
                controller::composition()
            } else {
                Vec::new()
            };
            Ok(Box::new(Gallery {
                model,
                view,
                definitions,
                scene_source,
                runtime,
                bus,
                pending: BTreeMap::new(),
                inflight: BTreeSet::new(),
                input: Vec::new(),
                later_input: std::collections::VecDeque::new(),
                input_replies: Vec::new(),
                witness: std::env::var_os("VIU_GALLERY_WITNESS").map(PathBuf::from),
                error: String::new(),
                mode,
            }))
        }),
    )
}
