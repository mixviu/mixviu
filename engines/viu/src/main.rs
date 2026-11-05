// SPDX-License-Identifier: MIT OR Apache-2.0
mod fixture;
#[cfg(test)]
mod tests;
use std::{path::PathBuf, sync::Arc};
use viu::{runtime::{Program, Runtime}, view::SceneView};

struct Client { runtime: Runtime, view: SceneView, error: String, program: Program }
impl eframe::App for Client {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        while let Some(done) = self.runtime.try_completion() {
            if !self.runtime.accepts(&done) { continue; }
            if let Some(error) = done.error { self.error = error; }
            if let Some(model) = done.model { if let Err(error) = self.view.apply_model(model) { self.error = error; } }
        }
        ui.label("Quoin volume — safe audio fixture");
        if ui.button("Reload behaviour").clicked() {
            match std::fs::read_to_string(&self.program.path) {
                Ok(source) => { self.program.source = source; if let Err(e) = self.runtime.load(self.program.clone()) {self.error = format!("{e:?}");} }
                Err(e) => self.error = e.to_string(),
            }
        }
        if !self.error.is_empty() { ui.colored_label(egui::Color32::RED, &self.error); }
        for event in self.view.show(ui) {
            if let Err(refusal) = self.runtime.event(event.command, event.body) { self.error = format!("Input refused: {refusal:?}"); }
        }
    }
}

use eframe::egui;
fn main() -> eframe::Result {
    let root = std::env::args_os().nth(1).map(PathBuf::from).unwrap_or_else(|| PathBuf::from("apps/quoin/volume"));
    let path = root.join("viu.mix").canonicalize().expect("volume behaviour path");
    let program = Program { source: std::fs::read_to_string(&path).expect("behaviour"), path, scene: "volume".into(), citizen: "scene-volume".into() };
    let source = std::fs::read_to_string(root.join("scene.mix")).expect("scene");
    eframe::run_native("Viu — Quoin lab", eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([440.0, 680.0]).with_app_id("mixviu.quoin.lab"),
        ..Default::default()
    }, Box::new(move |cc| {
        let ctx = cc.egui_ctx.clone();
        let mut runtime = Runtime::new(32, Arc::new(move || ctx.request_repaint()), Some(Box::new(|| fixture::AudioFixture::new())));
        runtime.load(program.clone()).expect("initial load");
        Ok(Box::new(Client { runtime, view: SceneView::new(&source).expect("scene"), error: String::new(), program }))
    }))
}
