// SPDX-License-Identifier: MIT OR Apache-2.0
mod fixture;
#[cfg(test)]
mod tests;
use std::{path::PathBuf, sync::Arc};
use viu::{
    runtime::{Program, Runtime},
    view::SceneView,
};

struct Client {
    runtime: Runtime,
    view: SceneView,
    error: String,
    program: Program,
    witness: Option<PathBuf>,
    last_sequence: u64,
    pending_view: Option<(u64, SceneView)>,
    behaviour_ready: bool,
    lifecycle_note: String,
    caption: String,
}
impl Client {
    fn reload(&mut self, scene: &str, source: String) -> Result<(), String> {
        let view = SceneView::new(scene)?;
        let mut program = self.program.clone();
        program.source = source;
        let generation = self
            .runtime
            .load(program.clone())
            .map_err(|e| format!("{e:?}"))?;
        self.program = program;
        self.pending_view = Some((generation, view));
        self.behaviour_ready = false;
        self.lifecycle_note =
            "Reload pending: earlier input results are superseded; controls wait for the new model"
                .into();
        Ok(())
    }

    fn complete(&mut self, done: viu::runtime::Completion) {
        if !self.runtime.accepts(&done) {
            eprintln!(
                "VIU superseded generation={} sequence={}",
                done.generation, done.sequence
            );
            self.lifecycle_note = "An earlier generation's result was superseded by reload".into();
            return;
        }
        self.last_sequence = done.sequence;
        if !done.behaviour_available {
            self.behaviour_ready = false;
        }
        eprintln!(
            "VIU completion generation={} sequence={} callback_us={} error={:?}",
            done.generation,
            done.sequence,
            done.elapsed.as_micros(),
            done.error
        );
        if let Some(error) = done.error {
            self.error = error;
            if done.sequence == 0 {
                self.pending_view = None;
                self.behaviour_ready = false;
                self.lifecycle_note =
                    "Reload failed; last good scene retained, reload to recover".into();
            }
            return;
        }
        if let Some((generation, mut candidate)) = self.pending_view.take() {
            if generation != done.generation || done.sequence != 0 {
                self.error = "reload completion does not match pending scene".into();
                return;
            }
            let result = done
                .model
                .ok_or_else(|| "reloaded behaviour did not publish an initial model".to_owned())
                .and_then(|model| candidate.apply_model(model));
            match result {
                Ok(()) => {
                    self.view = candidate;
                    self.behaviour_ready = true;
                    self.error.clear();
                    self.lifecycle_note =
                        "Reload committed; earlier input results were superseded".into();
                }
                Err(error) => self.error = error,
            }
        } else if let Some(model) = done.model {
            match self.view.apply_model(model) {
                Ok(()) => {
                    self.behaviour_ready = true;
                    self.error.clear();
                }
                Err(error) => self.error = error,
            }
        }
    }
}
impl eframe::App for Client {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        while let Some(done) = self.runtime.try_completion() {
            self.complete(done);
        }
        ui.label(&self.caption);
        if ui.button("Reload scene and behaviour").clicked() {
            let scene = self.program.path.parent().unwrap().join("scene.mix");
            let files = std::fs::read_to_string(scene).and_then(|scene| {
                std::fs::read_to_string(&self.program.path).map(|source| (scene, source))
            });
            match files {
                Ok((scene, source)) => {
                    if let Err(error) = self.reload(&scene, source) {
                        self.error = error;
                    }
                }
                Err(error) => self.error = error.to_string(),
            }
        }
        if !self.error.is_empty() {
            ui.colored_label(egui::Color32::RED, &self.error);
        }
        if !self.lifecycle_note.is_empty() {
            ui.label(&self.lifecycle_note);
        }
        let events = ui
            .add_enabled_ui(self.behaviour_ready, |ui| {
                egui::ScrollArea::vertical()
                    .id_salt("scene-root")
                    .show(ui, |ui| self.view.show(ui))
                    .inner
            })
            .inner;
        for event in events {
            eprintln!("VIU input command={} body={}", event.command, event.body);
            if let Err(refusal) = self.runtime.event(event.command, event.body) {
                self.error = format!("Input refused: {refusal:?}");
            }
        }
        if let Some(path) = &self.witness {
            let geometry: serde_json::Map<String,serde_json::Value> = self.view.geometry.iter().map(|(key,rect)| (key.clone(),serde_json::json!({"min":[rect.min.x,rect.min.y],"max":[rect.max.x,rect.max.y]}))).collect();
            let receipt = serde_json::json!({"generation":self.runtime.generation(),"sequence":self.last_sequence,"revision":self.view.revision,"model":self.view.model(),"geometry":geometry,"error":self.error});
            let temporary = path.with_extension("tmp");
            if std::fs::write(&temporary, receipt.to_string()).is_ok() {
                let _ = std::fs::rename(temporary, path);
            }
        }
    }
}

use eframe::egui;

struct ClientConfig {
    behaviour: String,
    scene: String,
    citizen: String,
    title: String,
    app_id: String,
    caption: String,
    size: [f32; 2],
    audio_fixture: bool,
}
impl ClientConfig {
    fn parse(source: Option<&str>) -> Result<Self, String> {
        let Some(source) = source else {
            return Ok(Self { behaviour: "viu.mix".into(), scene: "volume".into(),
                citizen: "scene-volume".into(), title: "Viu — Quoin lab".into(),
                app_id: "mixviu.quoin.lab".into(), caption: "Quoin volume — safe audio fixture".into(),
                size: [440.0, 680.0], audio_fixture: true });
        };
        let value: serde_json::Value = serde_json::from_str(source).map_err(|e| e.to_string())?;
        let string = |key: &str| -> Result<String, String> {
            value[key].as_str().filter(|s| !s.is_empty()).map(str::to_owned)
                .ok_or_else(|| format!("client.json requires nonempty string {key}"))
        };
        let backend = string("backend")?;
        if backend != "none" && backend != "audio-fixture" {
            return Err("unsupported client backend".into());
        }
        let behaviour = string("behaviour")?;
        if !std::path::Path::new(&behaviour).components().all(|c| matches!(c, std::path::Component::Normal(_))) {
            return Err("behaviour must be a relative application path".into());
        }
        let size = value["size"].as_array().filter(|s| s.len() == 2)
            .ok_or("client.json size must contain width and height")?;
        let dimension = |i: usize| -> Result<f32, String> {
            let n = size[i].as_f64().ok_or("window dimensions must be numbers")? as f32;
            if n.is_finite() && n > 0.0 { Ok(n) } else { Err("invalid window dimension".into()) }
        };
        Ok(Self { behaviour, scene: string("scene")?, citizen: string("citizen")?,
            title: string("title")?, app_id: string("app_id")?, caption: string("caption")?,
            size: [dimension(0)?, dimension(1)?], audio_fixture: backend == "audio-fixture" })
    }
}
fn main() -> eframe::Result {
    let root = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("apps/quoin/volume"));
    let config_source = match std::fs::read_to_string(root.join("client.json")) {
        Ok(source) => Some(source),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(error) => panic!("read client configuration: {error}"),
    };
    let config = ClientConfig::parse(config_source.as_deref()).expect("client configuration");
    let path = root
        .join(&config.behaviour)
        .canonicalize()
        .expect("application behaviour path");
    let program = Program {
        source: std::fs::read_to_string(&path).expect("behaviour"),
        path,
        scene: config.scene.clone(),
        citizen: config.citizen.clone(),
    };
    let source = std::fs::read_to_string(root.join("scene.mix")).expect("scene");
    eframe::run_native(
        &config.title,
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size(config.size)
                .with_app_id(&config.app_id),
            ..Default::default()
        },
        Box::new(move |cc| {
            let ctx = cc.egui_ctx.clone();
            let mut runtime = Runtime::new(
                32,
                Arc::new(move || ctx.request_repaint()),
                if config.audio_fixture { Some(Box::new(|| fixture::AudioFixture::new())) } else { None },
            );
            runtime.load(program.clone()).expect("initial load");
            Ok(Box::new(Client {
                runtime,
                view: SceneView::new(&source).expect("scene"),
                error: String::new(),
                program,
                witness: std::env::var_os("VIU_LAB_WITNESS").map(PathBuf::from),
                last_sequence: 0,
                pending_view: None,
                behaviour_ready: false,
                lifecycle_note: String::new(),
                caption: config.caption.clone(),
            }))
        }),
    )
}
