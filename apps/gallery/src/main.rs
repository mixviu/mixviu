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
    widgets::{Widget, Widgets},
};

struct Gallery {
    model: Value,
    widgets: Widgets,
    runtime: Option<Runtime>,
    bus: NativeBus,
    pending: BTreeMap<u64, tokio::sync::oneshot::Sender<(u8, Value)>>,
    inflight: BTreeSet<u64>,
    input: Vec<egui::Event>,
    later_input:std::collections::VecDeque<Vec<egui::Event>>,
    input_replies: Vec<tokio::sync::oneshot::Sender<(u8, Value)>>,
    witness: Option<PathBuf>,
    error: String,
    mode: String,
}
impl Gallery {
    fn definitions(&self) -> Vec<Widget> {
        self.model["widgets"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|v| Widget::decode(v).expect("validated composition"))
            .collect()
    }
    fn state(&self) -> Value {
        let geometry: serde_json::Map<String, Value> = self
            .widgets
            .geometry
            .iter()
            .map(|(id, r)| {
                (
                    id.clone(),
                    json!({"min":[r.min.x,r.min.y],"max":[r.max.x,r.max.y]}),
                )
            })
            .collect();
        json!({"model":self.model,"geometry":geometry,"error":self.error,"pending":self.pending.len()})
    }
    fn execute(&mut self, body: Value, reply: Option<tokio::sync::oneshot::Sender<(u8, Value)>>) {
        if let Err(error) = controller::validate(&self.definitions(), &body) {
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
            controller::apply(&mut self.model, &body);
            if let Some(reply) = reply {
                let _ = reply.send((0, self.state()));
            }
        }
    }
    fn request(&mut self, request: Request, ctx: &egui::Context) {
        let Request {
            command,
            body,
            reply,
        } = request;
        if let Some(suffix)=command.strip_prefix("gallery.widget.") {
            if let Some((id,action))=suffix.split_once('.') {self.execute(json!({"id":id,"action":action,"value":body["value"]}),Some(reply));return;}
        }
        match command.as_str() {
            "gallery.state" | "gallery.ui.tree" => {
                let _ = reply.send((0, self.state()));
            }
            "gallery.describe" => {
                let widgets=self.definitions().iter().map(|w|json!({"definition":w.encode(),"actions":controller::actions(w),"ui_actions":["focus","hover","click","key","text","scroll"]})).collect::<Vec<_>>();
                let _=reply.send((0,json!({"schema":"viu.gallery.v1","mode":self.mode,"widgets":widgets,"execute":{"id":"widget ID","action":"set|activate|submit|accept|cancel|reset","value":"typed widget value"},"ui":{"focus":"{id}","hover":"{id}","click":"{id,button?,double?}","key":"{key,ctrl?,shift?,alt?}","text":"{text}","scroll":"{id,delta}"},"endpoint":"state/model applied; not physical presentation"})));
            }
            "gallery.execute" => self.execute(body, Some(reply)),
            "gallery.quit" => {
                let _ = reply.send((0, json!({"ok":true})));
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
            _ => match self.ui_command(&command, &body, ctx) {
                Ok(()) => {
                    self.input_replies.push(reply);
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
            "gallery.ui.focus" | "gallery.ui.hover" | "gallery.ui.click" | "gallery.ui.scroll" | "gallery.ui.drag"=> {
                let id = body["id"].as_str().ok_or("id required")?;
                let rect = *self
                    .widgets
                    .geometry
                    .get(id)
                    .ok_or("widget not currently visible; open its parent first")?;
                if command == "gallery.ui.focus" {
                    self.widgets.focus = Some(id.into());
                } else {
                    self.input.push(egui::Event::PointerMoved(rect.center()));
                    if command == "gallery.ui.click" {
                        let button = match body["button"].as_str().unwrap_or("left") {
                            "left" => egui::PointerButton::Primary,
                            "right" => egui::PointerButton::Secondary,
                            _ => return Err("button must be left or right".into()),
                        };
                        for _ in 0..if body["double"] == true { 2 } else { 1 } {
                            for pressed in [true, false] {
                                self.input.push(egui::Event::PointerButton {
                                    pos: rect.center(),
                                    button,
                                    pressed,
                                    modifiers,
                                });
                            }
                        }
                    } else if command == "gallery.ui.drag" {
                        let fraction=body["fraction"].as_f64().filter(|v|v.is_finite()&&(0.0..=1.0).contains(v)).ok_or("drag requires fraction 0..1")?;
                        let start=rect.center();let end=egui::pos2(rect.left()+rect.width()*fraction as f32,rect.center().y);
                        self.input.push(egui::Event::PointerButton{pos:start,button:egui::PointerButton::Primary,pressed:true,modifiers});
                        self.later_input.push_back(vec![egui::Event::PointerMoved(end)]);
                        self.later_input.push_back(vec![egui::Event::PointerButton{pos:end,button:egui::PointerButton::Primary,pressed:false,modifiers}]);
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
            if let Some(error) = done.error {
                self.error = error;
            } else if let Some(model) = done.model {
                self.model = model;
                self.error.clear();
            }
            if let Some(reply) = self.pending.remove(&done.sequence) {
                let _ = reply.send((if self.error.is_empty() { 0 } else { 10 }, self.state()));
            }
        }
    }
}
impl eframe::App for Gallery {
    fn raw_input_hook(&mut self, _ctx: &egui::Context, input: &mut egui::RawInput) {
        if !self.input.is_empty(){input.events.append(&mut self.input);}else if let Some(mut later)=self.later_input.pop_front(){input.events.append(&mut later);}
    }
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
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
        ui.separator();
        if !self.error.is_empty() {
            ui.colored_label(egui::Color32::RED, &self.error);
        }
        let definitions = self.definitions();
        let events = egui::ScrollArea::vertical()
            .id_salt("gallery-page")
            .auto_shrink([false, false])
            .show(ui, |ui| self.widgets.show(ui, &definitions, &self.model))
            .inner;
        for event in events {
            self.execute(event.encode(), None);
            ui.ctx().request_repaint();
        }
        if self.input.is_empty() && self.later_input.is_empty() && self.inflight.is_empty() && self.widgets.focus.is_none() {
            let state = self.state();
            for reply in self.input_replies.drain(..) {
                let _ = reply.send((if self.error.is_empty() { 0 } else { 10 }, state.clone()));
            }
        }
        if !self.later_input.is_empty(){ui.ctx().request_repaint();}
        // The witness is explicit diagnostic I/O, enabled only by the caller.
        if let Some(path) = &self.witness {
            let temporary = path.with_extension("tmp");
            let _ = std::fs::write(&temporary, self.state().to_string())
                .and_then(|_| std::fs::rename(temporary, path));
        }
    }
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
            Ok(Box::new(Gallery {
                model: if mode == "rust" {
                    controller::initial()
                } else {
                    json!({})
                },
                widgets: Widgets::default(),
                runtime,
                bus,
                pending: BTreeMap::new(),
                inflight: BTreeSet::new(),
                input: Vec::new(),
                later_input:std::collections::VecDeque::new(),
                input_replies: Vec::new(),
                witness: std::env::var_os("VIU_GALLERY_WITNESS").map(PathBuf::from),
                error: String::new(),
                mode,
            }))
        }),
    )
}
