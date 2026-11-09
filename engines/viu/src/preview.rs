// SPDX-License-Identifier: MIT OR Apache-2.0
//! A passive scene specimen; no behaviours or device actions are executed.
use crate::{command::SceneCommand, store::SceneStore, view::{Content, SceneMessage, SceneUi}};
use serde_json::json;
use std::{path::PathBuf, sync::Arc, time::{Duration, Instant}};

struct Preview {
    dir: PathBuf,
    model: Option<PathBuf>,
    fingerprint: Vec<Option<Vec<u8>>>,
    next: Instant,
    store: SceneStore,
    view: Option<SceneUi>,
    error: Option<String>,
}
impl Preview {
    fn refresh(&mut self, ctx: &egui::Context) {
        let files = [Some(self.dir.join("scene.mix")), Some(self.dir.join("template.conf.mix")), Some(self.dir.join("behaviour.mix")), self.model.clone()];
        let fingerprint: Vec<_> = files.iter().map(|p| p.as_ref().and_then(|p| std::fs::read(p).ok())).collect();
        if fingerprint == self.fingerprint { return; }
        self.fingerprint = fingerprint;
        let result = (|| -> Result<_, String> {
            let script = std::env::var_os("VIU_PREVIEW_VALIDATOR").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../viud/scripts/scene_preview.mix")));
            let mut command = std::process::Command::new("/opt/mixos/bin/mix");
            command.arg(script).arg(&self.dir);
            if let Some(model) = &self.model { command.arg(model); }
            let output = command.output().map_err(|e| e.to_string())?;
            if !output.status.success() { return Err(String::from_utf8_lossy(&output.stderr).into_owned()); }
            let source = String::from_utf8(output.stdout).map_err(|e| e.to_string())?;
            self.store.request(SceneCommand::Load, &source, &json!({})).map_err(|e| e.to_string())?;
            let (_, entry) = self.store.scenes().next().ok_or("missing scene")?;
            Ok(Arc::new(Content { tree:entry.tree().clone(), lists:entry.prepared().clone(), revision:entry.revision(), frame:None, dialog:false }))
        })();
        match result {
            Ok(content) => {
                if let Some(view) = &mut self.view { view.update(SceneMessage::Replace(content)); }
                else { let mut view = SceneUi::new(content); view.setup(ctx); self.view=Some(view); }
                self.error=None;
            }
            Err(error) => self.error=Some(error),
        }
    }
}
impl eframe::App for Preview {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if Instant::now() >= self.next { self.refresh(ctx); self.next=Instant::now()+Duration::from_millis(200); }
        ctx.request_repaint_after(Duration::from_millis(200));
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(view) = &mut self.view { view.show(ui, &mut Vec::new()); }
            if let Some(error)=&self.error { crate::dev_loop::error_strip(ui,error); }
        });
    }
}
pub fn run(args: &[String]) -> eframe::Result {
    let valid = args.len()==1 || (args.len()==3 && args[1]=="--model");
    if !valid { eprintln!("usage: viu --preview DIR [--model FILE]"); std::process::exit(2); }
    let dir=PathBuf::from(&args[0]);
    let model=args.get(2).map(PathBuf::from);
    eframe::run_native("Scene Preview", eframe::NativeOptions::default(), Box::new(move |_| Ok(Box::new(Preview {dir,model,fingerprint:Vec::new(),next:Instant::now(),store:SceneStore::default(),view:None,error:None}))))
}
