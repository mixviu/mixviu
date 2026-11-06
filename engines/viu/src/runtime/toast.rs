// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native presentation of the existing Mix-authored toast seat.
use super::*;

fn content(state: &State, output: &str) -> Option<(String, Arc<Content>, u32, u32)> {
    let seat = state.store.toast_seat()?;
    if seat.output != output && seat.output != comp_model::snapshot::output_key(output) {
        return None;
    }
    let entry = state.store.scene(&seat.scene)?;
    Some((seat.scene.clone(), Arc::new(Content {
        tree: entry.tree().clone(), lists: entry.prepared().clone(),
        revision: entry.revision(), frame: None, dialog: false,
    }), seat.w as u32, seat.h as u32))
}

struct ToastProgram {
    state: Shared,
    send: tokio::sync::mpsc::UnboundedSender<Outgoing>,
    views: BTreeMap<String, (String, SceneUi, u64)>,
}
impl LayerProgram for ToastProgram {
    fn set_waker(&mut self, wake: Arc<dyn Fn() + Send + Sync>) {
        self.state.lock().unwrap().wakes.push(wake);
    }
    fn setup(&mut self, ctx: &egui::Context) {
        toolkit::install(ctx, &toolkit::Theme::embedded());
    }
    fn strict_frame_callbacks(&self) -> bool { true }
    fn frame_interval(&self) -> Duration { Duration::from_millis(16) }
    fn presentation(&mut self, ctx: &egui::Context, info: &SurfaceInfo) -> LayerPresentation {
        let output = info.output.as_deref().unwrap_or("default");
        let mut state = self.state.lock().unwrap();
        state.contexts.insert((output.into(), "toast".into()), ctx.clone());
        let candidate = content(&state, output);
        LayerPresentation {
            visible: candidate.is_some(),
            exclusive_zone: Some(-1),
            size: candidate.map(|(_, _, w, h)| (w, h)),
            margin: Some((16,16,16,16)),
            ..Default::default()
        }
    }
    fn show(&mut self, ui: &mut egui::Ui, info: &SurfaceInfo) {
        let output = info.output.as_deref().unwrap_or("default").to_owned();
        let (candidate, look, look_revision) = {
            let state = self.state.lock().unwrap();
            (content(&state, &output), state.look.clone(), state.look_revision)
        };
        let Some((name, content, _, _)) = candidate else { return; };
        let entry = self.views.entry(output).or_insert_with(|| {
            let mut view = SceneUi::new(content.clone());
            view.setup(ui.ctx());
            (name.clone(), view, u64::MAX)
        });
        if entry.0 != name {
            let mut view = SceneUi::new(content.clone());
            view.setup(ui.ctx());
            *entry = (name, view, u64::MAX);
        }
        if entry.1.revision() != content.revision {
            entry.1.update(SceneMessage::Replace(content));
        }
        if entry.2 != look_revision {
            if let Some(look) = look { entry.1.update(SceneMessage::Appearance(look)); }
            entry.2 = look_revision;
        }
        let mut messages = Vec::new();
        entry.1.show(ui, &mut messages);
        for message in messages {
            if let Some((to, command, body)) = view::event_of(&message) {
                let _ = self.send.send(Outgoing::Fire(to.into(), command.into(), body));
            }
            entry.1.update(message);
        }
    }
    fn presented(&mut self, info: &SurfaceInfo) {
        if let Some((name, view, _)) = self.views.get(info.output.as_deref().unwrap_or("default")) {
            let _ = self.send.send(Outgoing::Presented(name.clone(), info.output.clone().unwrap_or_default(), view.revision()));
        }
    }
    fn frame(&mut self, info: &SurfaceInfo, time: u32) {
        if let Some((name, _, _)) = self.views.get(info.output.as_deref().unwrap_or("default")) {
            let _ = self.send.send(Outgoing::Frame(name.clone(), time));
        }
    }
    fn removed(&mut self, _: &egui::Context, info: &SurfaceInfo) {
        let output = info.output.as_deref().unwrap_or("default");
        self.state.lock().unwrap().contexts.remove(&(output.into(), "toast".into()));
        self.views.remove(output);
    }
}

pub(super) fn start(state: Shared, send: tokio::sync::mpsc::UnboundedSender<Outgoing>) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || {
        let options = LayerOptions { namespace: "viu-toast".into(), layer: Layer::Overlay,
            anchor: Anchor::TOP | Anchor::RIGHT, keyboard: KeyboardInteractivity::None,
            exclusive_zone: -1, size: (360,132), outputs: Outputs::Each };
        if let Err(error) = layer::run(options, ToastProgram { state, send, views: BTreeMap::new() }) {
            eprintln!("viu toast: {error}");
            std::process::exit(1);
        }
    })
}
