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
        if !done.behaviour_available { self.behaviour_ready = false; }
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
        ui.label("Quoin volume — safe audio fixture");
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
fn main() -> eframe::Result {
    let root = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("apps/quoin/volume"));
    let path = root
        .join("viu.mix")
        .canonicalize()
        .expect("volume behaviour path");
    let program = Program {
        source: std::fs::read_to_string(&path).expect("behaviour"),
        path,
        scene: "volume".into(),
        citizen: "scene-volume".into(),
    };
    let source = std::fs::read_to_string(root.join("scene.mix")).expect("scene");
    eframe::run_native(
        "Viu — Quoin lab",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([440.0, 680.0])
                .with_app_id("mixviu.quoin.lab"),
            ..Default::default()
        },
        Box::new(move |cc| {
            let ctx = cc.egui_ctx.clone();
            let mut runtime = Runtime::new(
                32,
                Arc::new(move || ctx.request_repaint()),
                Some(Box::new(|| fixture::AudioFixture::new())),
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
            }))
        }),
    )
}
