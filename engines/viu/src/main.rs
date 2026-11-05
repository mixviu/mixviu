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
}
impl eframe::App for Client {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        while let Some(done) = self.runtime.try_completion() {
            if !self.runtime.accepts(&done) {
                continue;
            }
            self.last_sequence = done.sequence;
            eprintln!(
                "VIU completion generation={} sequence={} callback_us={} error={:?}",
                done.generation,
                done.sequence,
                done.elapsed.as_micros(),
                done.error
            );
            if done.error.is_none() {
                self.error.clear();
            }
            if let Some(error) = done.error {
                self.error = error;
            }
            if let Some(model) = done.model {
                if let Err(error) = self.view.apply_model(model) {
                    self.error = error;
                }
            }
        }
        ui.label("Quoin volume — safe audio fixture");
        if ui.button("Reload scene and behaviour").clicked() {
            let scene = self.program.path.parent().unwrap().join("scene.mix");
            match std::fs::read_to_string(scene)
                .map_err(|e| e.to_string())
                .and_then(|source| SceneView::new(&source))
            {
                Ok(mut view) => {
                    if let Err(error) = view.apply_model(self.view.model().clone()) {
                        self.error = error;
                    } else {
                        self.view = view;
                    }
                }
                Err(error) => self.error = error,
            }
            match std::fs::read_to_string(&self.program.path) {
                Ok(source) => {
                    self.program.source = source;
                    if let Err(e) = self.runtime.load(self.program.clone()) {
                        self.error = format!("{e:?}");
                    }
                }
                Err(e) => self.error = e.to_string(),
            }
        }
        if !self.error.is_empty() {
            ui.colored_label(egui::Color32::RED, &self.error);
        }
        let events = egui::ScrollArea::vertical().id_salt("scene-root").show(ui, |ui| self.view.show(ui)).inner;
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
            }))
        }),
    )
}
