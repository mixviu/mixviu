// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Client furniture host. No compositor Loop, registry or input hooks.
mod toast;
use crate::{
    appearance,
    command::SceneCommand,
    mount,
    store::{Mounted, SceneMount, SceneStore},
    view::{self, Content, SceneMessage, SceneUi},
};
use edges::{PanelConfig, PanelInput, PanelMode, PanelStateMachine, PanelWake};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use toolkit::layer::{
    self, Anchor, KeyboardInteractivity, Layer, LayerOptions, LayerPresentation, LayerProgram,
    Outputs, SurfaceInfo,
};

type Shared = Arc<Mutex<State>>;
type Key = (String, String);
fn shell_edge(edge: &str) -> Option<edges::Edge> {
    Some(match edge {
        "left" => edges::Edge::Left,
        "right" => edges::Edge::Right,
        "top" => edges::Edge::Top,
        "bottom" => edges::Edge::Bottom,
        _ => return None,
    })
}
struct State {
    store: SceneStore,
    panels: BTreeMap<Key, PanelStateMachine>,
    contexts: BTreeMap<Key, egui::Context>,
    wakes: Vec<Arc<dyn Fn() + Send + Sync>>,
    started: Instant,
    look: Option<Arc<appearance::Look>>,
    look_revision: u64,
    corner_events: u64,
    corner_phases: Vec<String>,
    comp_state: Value,
    outputs: Value,
    publish: Vec<String>,
    hover: Value,
    tips: BTreeMap<String, (String, view::tip::TipAt)>,
    output_size: BTreeMap<String, (f32, f32)>,
    preferences: crate::preferences::Preferences,
    selected: BTreeMap<Key, String>,
}
impl State {
    fn new() -> Self {
        use settings::view::View;
        Self {
            store: SceneStore::default(),
            panels: BTreeMap::new(),
            contexts: BTreeMap::new(),
            wakes: Vec::new(),
            started: Instant::now(),
            look: None,
            look_revision: 0,
            corner_events: 0,
            corner_phases: Vec::new(),
            comp_state: Value::Null,
            outputs: Value::Null,
            publish: Vec::new(),
            hover: Value::Null,
            tips: BTreeMap::new(),
            output_size: BTreeMap::new(),
            selected: BTreeMap::new(),
            preferences: crate::preferences::Preferences::prepare(&settings::view::Panels::of(
                &Default::default(),
            ))
            .expect("embedded panel defaults"),
        }
    }
    fn wake(&self) {
        for wake in &self.wakes {
            wake();
        }
        for ctx in self.contexts.values() {
            ctx.request_repaint();
        }
    }
    fn panel(&mut self, output: &str, edge: &str) -> &mut PanelStateMachine {
        let pref = shell_edge(edge)
            .and_then(|edge| self.preferences.record(edge))
            .map(|r| r.preference);
        self.panels
            .entry((output.into(), edge.into()))
            .or_insert_with(|| {
                let thickness = pref.map(|p| p.thickness()).unwrap_or_else(|| {
                    if edge == "left" || edge == "right" {
                        440.0
                    } else {
                        40.0
                    }
                });
                let mut model = PanelStateMachine::new(
                    PanelConfig::new(
                        thickness,
                        Duration::from_millis(800),
                        Duration::from_millis(200),
                    )
                    .unwrap(),
                    Duration::ZERO,
                )
                .unwrap();
                model.set_holder_plane(true, Duration::ZERO).unwrap();
                model
                    .apply(
                        Duration::ZERO,
                        PanelInput::SetMode(pref.map_or(PanelMode::Hidden, |p| p.mode())),
                    )
                    .unwrap();
                model
            })
    }
    fn input(&mut self, output: &str, edge: &str, input: PanelInput) {
        let now = self.started.elapsed();
        let _ = self.panel(output, edge).apply(now, input);
        self.wake();
    }
    fn content(&self, output: &str, edge: &str) -> Option<(String, Arc<Content>)> {
        self.store
            .scenes()
            .filter_map(|(name, entry)| {
                let tree = entry.tree();
                if mount::is_dialog(tree)
                    || mount::is_toast(tree)
                    || mount::scene_edge(tree).as_str() != edge
                {
                    return None;
                }
                let page = mount::page_id(tree);
                if self
                    .selected
                    .get(&(output.into(), edge.into()))
                    .is_some_and(|selected| selected != &page)
                {
                    return None;
                }
                let seat = self.store.pages().seat(&page)?;
                if seat.output != output && seat.output != comp_model::snapshot::output_key(output)
                {
                    return None;
                }
                Some((
                    seat.accepted_at,
                    name.to_owned(),
                    Arc::new(Content {
                        tree: tree.clone(),
                        lists: entry.prepared().clone(),
                        revision: entry.revision(),
                        frame: None,
                        dialog: false,
                    }),
                ))
            })
            .min_by_key(|(accepted, name, _)| (*accepted, name.to_string()))
            .map(|(_, name, content)| (name, content))
    }
}
enum Outgoing {
    Fire(String, String, Value),
    Register(String, String),
    Mode(String, String, String),
    Presented(String, String, u64),
    Frame(String, u32),
}
struct Program {
    edge: String,
    state: Shared,
    send: tokio::sync::mpsc::UnboundedSender<Outgoing>,
    views: BTreeMap<String, (String, SceneUi, u64)>,
    last: BTreeMap<String, (String, u64)>,
    cold: bool,
    cold_frame: bool,
}
impl LayerProgram for Program {
    fn set_waker(&mut self, wake: Arc<dyn Fn() + Send + Sync>) {
        self.state.lock().unwrap().wakes.push(wake);
    }
    fn setup(&mut self, ctx: &egui::Context) {
        toolkit::install(ctx, &toolkit::Theme::embedded());
    }
    fn frame_interval(&self) -> Duration {
        Duration::from_millis(16)
    }
    fn strict_frame_callbacks(&self) -> bool {
        true
    }
    fn presentation(&mut self, ctx: &egui::Context, info: &SurfaceInfo) -> LayerPresentation {
        let output = info.output.as_deref().unwrap_or("default");
        let mut state = self.state.lock().unwrap();
        let key = (output.into(), self.edge.clone());
        if !state.contexts.contains_key(&key) {
            state.contexts.insert(key, ctx.clone());
            let _ = self
                .send
                .send(Outgoing::Register(output.into(), self.edge.clone()));
        }
        let content = state.content(output, &self.edge);
        let has_content = content.is_some();
        let minimum = content
            .as_ref()
            .and_then(|(_, c)| mount::mount_config(&c.tree))
            .and_then(|c| {
                c[if self.edge == "left" || self.edge == "right" {
                    "w"
                } else {
                    "h"
                }]
                .as_f64()
            })
            .unwrap_or(0.0) as f32;
        let size = state.output_size.entry(output.into()).or_default();
        if self.edge == "top" || self.edge == "bottom" {
            size.0 = info.width as f32;
        } else {
            size.1 = info.height as f32;
        }
        let now = state.started.elapsed();
        let model = state.panel(output, &self.edge);
        if model.snapshot().thickness_px < minimum {
            let _ = model.restore_thickness(minimum);
        }
        let _ = model.tick(now);
        let snap = model.snapshot();
        if model.wake() == PanelWake::Animate {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
        let offset = ((1.0 - snap.visible_fraction) * snap.thickness_px).round() as i32;
        if !has_content || !snap.mapped {
            if state
                .tips
                .get(output)
                .is_some_and(|(edge, _)| edge == &self.edge)
            {
                state.tips.remove(output);
                state.wake();
            }
        }
        let margin = match self.edge.as_str() {
            "left" => (0, 0, 0, -offset),
            "right" => (0, -offset, 0, 0),
            "top" => (-offset, 0, 0, 0),
            _ => (0, 0, -offset, 0),
        };
        LayerPresentation {
            input_passthrough: false,
            visible: has_content && snap.mapped,
            exclusive_zone: Some(snap.exclusive_zone_px.round() as i32),
            size: Some(if self.edge == "left" || self.edge == "right" {
                (snap.thickness_px as u32, 0)
            } else {
                (0, snap.thickness_px as u32)
            }),
            layer: Some(if snap.mode == PanelMode::Docked {
                Layer::Top
            } else {
                Layer::Overlay
            }),
            margin: Some(margin),
        }
    }
    fn show(&mut self, ui: &mut egui::Ui, info: &SurfaceInfo) {
        let output = info.output.as_deref().unwrap_or("default").to_owned();
        let (content, look, look_revision) = {
            let state = self.state.lock().unwrap();
            (
                state.content(&output, &self.edge),
                state.look.clone(),
                state.look_revision,
            )
        };
        let Some((name, content)) = content else {
            return;
        };
        let entry = self.views.entry(output.clone()).or_insert_with(|| {
            let mut view = SceneUi::new(content.clone());
            view.setup(ui.ctx());
            (name.clone(), view, 0)
        });
        if entry.0 != name {
            let mut view = SceneUi::new(content.clone());
            view.setup(ui.ctx());
            *entry = (name.clone(), view, 0);
        }
        if entry.1.revision() != content.revision {
            entry.1.update(SceneMessage::Replace(content.clone()));
        }
        if entry.2 != look_revision {
            if let Some(look) = look {
                entry.1.update(SceneMessage::Appearance(look));
            }
            entry.2 = look_revision;
        }
        let mut messages = Vec::new();
        entry.1.show(ui, &mut messages);
        for message in messages {
            if let Some((service, command, body)) = view::event_of(&message) {
                let _ = self
                    .send
                    .send(Outgoing::Fire(service.into(), command.into(), body));
            }
            if matches!(message, SceneMessage::EscapeEdge) {
                let _ = self.send.send(Outgoing::Mode(
                    output.clone(),
                    self.edge.clone(),
                    "hidden".into(),
                ));
            }
            if let SceneMessage::Tooltip(tip) = &message {
                let mut state = self.state.lock().unwrap();
                if let Some(tip) = tip {
                    state
                        .tips
                        .insert(output.clone(), (self.edge.clone(), tip.clone()));
                } else {
                    state.tips.remove(&output);
                }
                state.wake();
            }
            entry.1.update(message);
        }
        self.last.insert(output, (name, content.revision));
    }
    fn presented(&mut self, info: &SurfaceInfo) {
        let output = info.output.as_deref().unwrap_or("default");
        if let Some((name, revision)) = self.last.get(output) {
            let _ = self
                .send
                .send(Outgoing::Presented(name.clone(), output.into(), *revision));
            if !self.cold {
                println!(
                    "VIU_PANEL_FIRST_COMMIT edge={} output={} cold_ms={:.3}",
                    self.edge,
                    output,
                    self.state.lock().unwrap().started.elapsed().as_secs_f64() * 1000.0
                );
                self.cold = true;
            }
        }
    }
    fn frame(&mut self, info: &SurfaceInfo, time_ms: u32) {
        let output = info.output.as_deref().unwrap_or("default");
        if let Some((name, _)) = self.last.get(output) {
            let _ = self.send.send(Outgoing::Frame(name.clone(), time_ms));
            if !self.cold_frame {
                println!(
                    "VIU_PANEL_FIRST_FRAME edge={} output={} cold_ms={:.3}",
                    self.edge,
                    output,
                    self.state.lock().unwrap().started.elapsed().as_secs_f64() * 1000.0
                );
                self.cold_frame = true;
            }
        }
    }
    fn removed(&mut self, _: &egui::Context, info: &SurfaceInfo) {
        let output = info.output.as_deref().unwrap_or("default");
        self.state
            .lock()
            .unwrap()
            .contexts
            .remove(&(output.into(), self.edge.clone()));
        self.views.remove(output);
        self.last.remove(output);
    }
}

struct CardState {
    view: crate::tooltip::TooltipUi,
    metrics: egui::Context,
    hover: view::tip::Hover,
    shown: Option<view::tip::TipAt>,
    revision: u64,
}
struct TooltipProgram {
    state: Shared,
    cards: BTreeMap<String, CardState>,
}
impl LayerProgram for TooltipProgram {
    fn set_waker(&mut self, wake: Arc<dyn Fn() + Send + Sync>) {
        self.state.lock().unwrap().wakes.push(wake);
    }
    fn setup(&mut self, ctx: &egui::Context) {
        toolkit::install(ctx, &toolkit::Theme::embedded());
    }
    fn presentation(&mut self, ctx: &egui::Context, info: &SurfaceInfo) -> LayerPresentation {
        use view::tip::{self, Candidate, Outcome, Tip};
        let output = info.output.as_deref().unwrap_or("default");
        let mut shared = self.state.lock().unwrap();
        shared
            .contexts
            .insert((output.into(), "tooltip".into()), ctx.clone());
        let card = self.cards.entry(output.into()).or_insert_with(|| {
            let metrics = egui::Context::default();
            toolkit::install(&metrics, &toolkit::Theme::embedded());
            let mut view = crate::tooltip::TooltipUi {
                tip: Tip::default(),
                look: shared.look.clone(),
                installed: None,
            };
            view.setup(ctx);
            CardState {
                view,
                metrics,
                hover: tip::Hover::default(),
                shown: None,
                revision: u64::MAX,
            }
        });
        if card.revision != shared.look_revision {
            if let Some(look) = &shared.look {
                card.view
                    .update(crate::tooltip::Message::Appearance(look.clone()));
                appearance::install(&card.metrics, &look.theme, look.text_scale, false);
            }
            card.metrics
                .run_ui(egui::RawInput::default(), |_| {})
                .drop_without_applying_deltas();
            card.revision = shared.look_revision;
        }
        let hover = &shared.hover;
        let active = hover["output"]
            .as_str()
            .is_some_and(|key| key == output || key == comp_model::snapshot::output_key(output));
        let anchor: Vec<f32> = hover["anchor"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|v| v.as_f64().map(|v| v as f32))
                    .collect()
            })
            .unwrap_or_default();
        let candidate = (active && anchor.len() == 4).then(|| Candidate {
            key: hover["key"].as_str().unwrap_or_default().into(),
            rect: egui::Rect::from_min_size(
                egui::pos2(anchor[0], anchor[1]),
                egui::vec2(anchor[2], anchor[3]),
            ),
            tip: Tip {
                title: hover["title"].as_str().unwrap_or_default().into(),
                ..Tip::default()
            },
        });
        let metrics = &card.metrics;
        let ui_size = shared
            .look
            .as_ref()
            .map_or(13.0, |l| appearance::ui_size(&l.theme, l.text_scale));
        let measure = |tip: &Tip| {
            tip::card(tip, ui_size, &|text, font| {
                let size = metrics.fonts_mut(|f| {
                    f.layout_no_wrap(text.into(), font.clone(), egui::Color32::WHITE)
                        .size()
                });
                (size.x, size.y)
            })
        };
        match card.hover.pass(
            Instant::now(),
            candidate,
            active && hover["pressed"] == true,
            &measure,
        ) {
            Outcome::Show(at) => card.shown = Some(at),
            Outcome::Hide => card.shown = None,
            Outcome::Wait(delay) => ctx.request_repaint_after(delay),
            Outcome::Same => {}
        }
        let mut size = shared
            .output_size
            .get(output)
            .copied()
            .unwrap_or((0.0, 0.0));
        let want = if let Some(at) = &card.shown {
            if let Some(values) = hover["output_size"].as_array() {
                size = (
                    values[0].as_f64().unwrap_or(0.0) as f32,
                    values[1].as_f64().unwrap_or(0.0) as f32,
                );
            }
            let (x, y, w, h) = at.anchor;
            let mut at = at.clone();
            at.anchor = (0.0, 0.0, w, h);
            Some(((x, y, w, h), at))
        } else {
            shared.tips.get(output).map(|(edge, at)| {
                let mut at = at.clone();
                if at.tip.thumbnail.take().is_some() {
                    at.tip
                        .lines
                        .push("Window previews unavailable until B7".into());
                    let laid = measure(&at.tip);
                    at.size = laid.size;
                    at.thumb = None;
                }
                let thickness = shared
                    .panels
                    .get(&(output.into(), edge.clone()))
                    .map_or(40.0, |p| p.snapshot().thickness_px as f32);
                let scene = match edge.as_str() {
                    "left" => (0.0, 0.0, thickness, size.1),
                    "right" => (size.0 - thickness, 0.0, thickness, size.1),
                    "top" => (0.0, 0.0, size.0, thickness),
                    _ => (0.0, size.1 - thickness, size.0, thickness),
                };
                (scene, at)
            })
        };
        let Some((scene, at)) = want.filter(|_| size.0 > 0.0 && size.1 > 0.0) else {
            return LayerPresentation {
                visible: false,
                input_passthrough: true,
                ..Default::default()
            };
        };
        let (x, y, w, h) = crate::tooltip::place(scene, &at, size);
        if card.view.tip != at.tip {
            card.view.update(crate::tooltip::Message::Replace(at.tip));
            ctx.request_repaint();
        }
        let margin = crate::tooltip::MARGIN;
        LayerPresentation {
            visible: true,
            input_passthrough: true,
            exclusive_zone: Some(-1),
            size: Some((
                (w + 2.0 * margin).ceil() as u32,
                (h + 2.0 * margin).ceil() as u32,
            )),
            margin: Some((
                (y - margin).round() as i32,
                0,
                0,
                (x - margin).round() as i32,
            )),
            ..Default::default()
        }
    }
    fn show(&mut self, ui: &mut egui::Ui, info: &SurfaceInfo) {
        if let Some(card) = self
            .cards
            .get_mut(info.output.as_deref().unwrap_or("default"))
        {
            card.view.show(ui, &mut Vec::new());
        }
    }
    fn removed(&mut self, _: &egui::Context, info: &SurfaceInfo) {
        let output = info.output.as_deref().unwrap_or("default");
        self.cards.remove(output);
        self.state
            .lock()
            .unwrap()
            .contexts
            .remove(&(output.into(), "tooltip".into()));
    }
}

pub fn run(service: String, comp: String) -> Result<(), String> {
    let state = Arc::new(Mutex::new(State::new()));
    let (send, receive) = tokio::sync::mpsc::unbounded_channel();
    let worker_state = state.clone();
    std::thread::Builder::new()
        .name("viu-bus".into())
        .spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            if let Err(error) = runtime.block_on(worker(service, comp, worker_state, receive)) {
                eprintln!("viu Bus: {error}");
                std::process::exit(1);
            }
        })
        .map_err(|e| e.to_string())?;
    let mut threads = Vec::new();
    threads.push(toast::start(state.clone(), send.clone()));
    let tip_state = state.clone();
    threads.push(std::thread::spawn(move || {
        let options = LayerOptions {
            namespace: "viu-tooltip".into(),
            layer: Layer::Overlay,
            anchor: Anchor::TOP | Anchor::LEFT,
            keyboard: KeyboardInteractivity::None,
            exclusive_zone: -1,
            size: (320, 100),
            outputs: Outputs::Each,
        };
        if let Err(error) = layer::run(
            options,
            TooltipProgram {
                state: tip_state,
                cards: BTreeMap::new(),
            },
        ) {
            eprintln!("viu tooltip: {error}");
            std::process::exit(1);
        }
    }));
    for edge in ["left", "right", "top", "bottom"] {
        let anchor = match edge {
            "left" => Anchor::LEFT | Anchor::TOP | Anchor::BOTTOM,
            "right" => Anchor::RIGHT | Anchor::TOP | Anchor::BOTTOM,
            "top" => Anchor::TOP | Anchor::LEFT | Anchor::RIGHT,
            _ => Anchor::BOTTOM | Anchor::LEFT | Anchor::RIGHT,
        };
        let options = LayerOptions {
            namespace: format!("viu-panel-{edge}"),
            layer: Layer::Overlay,
            anchor,
            keyboard: KeyboardInteractivity::OnDemand,
            exclusive_zone: 0,
            size: if edge == "left" || edge == "right" {
                (440, 0)
            } else {
                (0, 40)
            },
            outputs: Outputs::Each,
        };
        let program = Program {
            edge: edge.into(),
            state: state.clone(),
            send: send.clone(),
            views: BTreeMap::new(),
            last: BTreeMap::new(),
            cold: false,
            cold_frame: false,
        };
        threads.push(std::thread::spawn(move || {
            if let Err(error) = layer::run(options, program) {
                eprintln!("viu layer: {error}");
                std::process::exit(1);
            }
        }));
    }
    for thread in threads {
        thread.join().map_err(|_| "viu layer thread panicked")?;
    }
    Ok(())
}

async fn report(client: &bus::SupervisedClient, comp: &str, output: &str, edge: &str, mode: &str) {
    let args = json!({"output":output,"edge":edge,"surface":format!("viu-panel-{edge}"),"mode":mode,"generation":client.connection_generation()});
    let _ = tokio::time::timeout(
        Duration::from_secs(1),
        client.call(comp, "comp.panel.mode", args),
    )
    .await;
}
async fn worker(
    service: String,
    comp: String,
    state: Shared,
    mut outgoing: tokio::sync::mpsc::UnboundedReceiver<Outgoing>,
) -> Result<(), String> {
    let client = bus::SupervisedClient::connect_options(&service, &bus::noded_url())
        .connect()
        .await
        .map_err(|e| e.to_string())?;
    let mut incoming = client.incoming().ok_or("Bus incoming already taken")?;
    for suffix in [
        "corner.input",
        "corner.clicked.v2",
        "corner.entered",
        "corner.left",
        "panel.command",
        "props.changed",
        "chrome.action.hover",
    ] {
        client
            .subscribe_topic(&format!("{comp}.{suffix}"))
            .await
            .map_err(|e| e.to_string())?;
    }
    let binding = settings::session::binding().map_err(|e| format!("settings binding: {e:?}"))?;
    let mut consumer = settings::consumer::Consumer::for_shell(binding)
        .map_err(|e| format!("settings consumer: {e:?}"))?;
    let mut work = None;
    let mut generation = None;
    let mut comp_generation = None;
    let mut states = client.subscribe_state();
    let mut retry = tokio::time::interval(Duration::from_secs(1));
    retry.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        if let Some(live) = settings::native::live_generation(&client) {
            if comp_generation != Some(live) {
                if refresh_comp(&client, &comp, &state).await {
                    comp_generation = Some(live);
                }
            }
            if generation != Some(live) {
                generation = Some(live);
                work = consumer.connected(live);
            }
        } else {
            comp_generation = None;
            generation = None;
            consumer.disconnected();
        }
        while let Some(action) = work.take() {
            let result = settings::native::execute(&client, &action).await;
            work = consumer.complete(&action, result);
        }
        if let Some(update) = consumer.pending().cloned() {
            if let Ok(look) = appearance::build(update.snapshot()) {
                use settings::view::View;
                let preferences = crate::preferences::Preferences::prepare(
                    &settings::view::Panels::of(&update.snapshot().values),
                );
                let mut shared = state.lock().unwrap();
                if let Ok(preferences) = preferences {
                    let now = shared.started.elapsed();
                    for ((_, edge), panel) in &mut shared.panels {
                        if let Some(edge) = shell_edge(edge) {
                            if let Some(record) = preferences.record(edge) {
                                let _ = panel.restore_thickness(record.preference.thickness());
                                // Appearance-only updates never undo a user's live mode.
                                // Settings changes remain the owner's accepted request.
                            }
                        }
                    }
                    if shared.preferences.values() != preferences.values() {
                        for ((_, edge), panel) in &mut shared.panels {
                            if let Some(record) =
                                shell_edge(edge).and_then(|e| preferences.record(e))
                            {
                                let _ =
                                    panel.apply(now, PanelInput::SetMode(record.preference.mode()));
                            }
                        }
                    }
                    shared.preferences = preferences;
                }
                shared.look = Some(Arc::new(look));
                shared.look_revision += 1;
                shared.wake();
                consumer.acknowledge(&update);
            }
        }
        tokio::select! {
            changed=states.changed()=>{if changed.is_err(){break;} states.borrow_and_update();}
            _=retry.tick()=>{
                work=consumer.refresh();
                let reports:Vec<_>={let shared=state.lock().unwrap();shared.panels.iter().map(|((o,e),p)|(o.clone(),e.clone(),p.snapshot().mode.as_str().to_owned())).collect()};
                for (output,edge,mode) in reports {report(&client,&comp,&output,&edge,&mode).await;}
            }
            action=outgoing.recv()=>{
                let Some(action)=action else {break;};
                match action {
                    Outgoing::Fire(to,command,body)=>{let _=client.send(&to,&command,body).await;}
                    Outgoing::Register(output,edge)=>{let mode={let mut shared=state.lock().unwrap();shared.panel(&output,&edge).snapshot().mode.as_str().to_owned()};report(&client,&comp,&output,&edge,&mode).await;}
                    Outgoing::Mode(output,edge,mode)=>{if let Some(mode)=PanelMode::parse(&mode) {state.lock().unwrap().input(&output,&edge,PanelInput::SetMode(mode));report(&client,&comp,&output,&edge,mode.as_str()).await;}}
                    Outgoing::Presented(name,_output,revision)=>{state.lock().unwrap().store.set_mounted(&name,Some(Mounted {handle:revision,revision}));}
                    Outgoing::Frame(name,time)=>{
                        let event=state.lock().unwrap().store.frame_event(&name,true,f64::from(time)/1000.0,1.0/60.0);
                        if let Some((citizen,body))=event {let _=client.send(&citizen,"frame",body).await;}
                    }
                }
            }
            command=incoming.recv()=>{
                let Some(command)=command else {break;};
                if command.is_topic_delivery() {
                    if command.header("broker_service")==Some(comp.as_str()) && command.header("broker_origin")==Some("local") {
                        if command.topic()==Some(format!("{comp}.props.changed").as_str()) {
                            if !refresh_comp(&client,&comp,&state).await {comp_generation=None;}
                        }
                        if let Ok(body)=serde_json::from_str::<Value>(&command.body) {topic(&state,&comp,command.topic().unwrap_or(""),&body);}
                    }
                    if let Some(next)=consumer.native_delivery(&command) {work=Some(next);}
                    continue;
                }
                let args=serde_json::from_str::<Value>(&command.body).unwrap_or_else(|_|command.args.clone());
                let (rc,body)=dispatch(&state,&service,&command,&args);
                let publications=std::mem::take(&mut state.lock().unwrap().publish);
                for wire in publications {
                    let mut headers=std::collections::BTreeMap::new();
                    headers.insert("name".into(),format!("{service}.scene.changed"));
                    headers.insert("retain".into(),"false".into());
                    let _=client.call_with_headers_raw("noded","topic.publish",&headers,&wire).await;
                }
                if command.id.is_some() {let _=client.respond_parts(command.generation,&command.from,&command.command,command.id.as_deref(),rc,&body.to_string()).await;}
            }
        }
    }
    Ok(())
}
async fn refresh_comp(client: &bus::SupervisedClient, comp: &str, state: &Shared) -> bool {
    let result = tokio::time::timeout(Duration::from_secs(2), async {
        client.call(comp, "comp.props.watch", json!({})).await?;
        client.call(comp, "comp.props.get", json!({})).await
    })
    .await;
    if let Ok(Ok(props)) = result {
        let mut shared = state.lock().unwrap();
        shared.outputs = props["outputs"].clone();
        shared.comp_state = props;
        shared.wake();
        true
    } else {
        false
    }
}

fn topic(state: &Shared, comp: &str, topic: &str, body: &Value) {
    let mut shared = state.lock().unwrap();
    if topic == format!("{comp}.chrome.action.hover") {
        shared.hover = body["hover"].clone();
        shared.wake();
        return;
    }
    if topic == format!("{comp}.props.changed") {
        return;
    }
    if topic == format!("{comp}.corner.input") {
        shared.corner_events += 1;
        if let Some(phase) = body["phase"].as_str() {
            if shared.corner_phases.len() == 16 {
                shared.corner_phases.remove(0);
            }
            shared.corner_phases.push(phase.into());
        }
        return;
    }
    let key = body["output"].as_str().unwrap_or_default();
    let output = shared
        .contexts
        .keys()
        .map(|(o, _)| o)
        .find(|o| o.as_str() == key || comp_model::snapshot::output_key(o) == key)
        .cloned();
    let Some(output) = output else {
        return;
    };
    if topic == format!("{comp}.panel.command") {
        if let Some(edge) = body["edge"].as_str() {
            shared.input(
                &output,
                edge,
                if body["action"] == "reveal" {
                    PanelInput::HolderReveal
                } else {
                    PanelInput::HolderConceal
                },
            );
        }
    } else if topic == format!("{comp}.corner.clicked.v2") {
        let edge = match body["corner"].as_str() {
            Some("tl") => "left",
            Some("bl") => "bottom",
            Some("br") => "right",
            Some("tr") => "top",
            _ => return,
        };
        if body["button"] == "left" {
            let shift = body["modifiers"]
                .as_array()
                .is_some_and(|m| m.iter().any(|v| v == "shift"));
            shared.input(
                &output,
                edge,
                if shift {
                    PanelInput::DockToggle
                } else {
                    PanelInput::PinToggle
                },
            );
        }
    }
}
fn dispatch(
    state: &Shared,
    service: &str,
    command: &bus::IncomingCommand,
    args: &Value,
) -> (u8, Value) {
    if matches!(command.command.as_str(), "HELP" | "app.describe") {
        return (0, json!({"commands":SceneCommand::manifest(service)}));
    }
    if command.header("broker_origin") == Some("mesh") {
        return (
            10,
            json!({"error_code":"FORBIDDEN","message":"Furniture authority requires a local broker caller"}),
        );
    }
    let mut shared = state.lock().unwrap();
    let Some(kind) = SceneCommand::parse(service, &command.command) else {
        return (10, json!({"error_code":"UNKNOWN_COMMAND"}));
    };
    let output = args["output"]
        .as_str()
        .map(str::to_owned)
        .or_else(|| shared.contexts.keys().next().map(|(o, _)| o.clone()))
        .or_else(|| {
            shared
                .outputs
                .as_object()
                .and_then(|rows| {
                    rows.values()
                        .find(|row| row["default"] == true)
                        .or_else(|| rows.values().next())
                })
                .and_then(|row| row["name"].as_str())
                .map(str::to_owned)
        })
        .unwrap_or_default();
    // Bus callers may use the compositor's stable output key; layer clients
    // identify that same output by its Wayland name.
    let output = shared
        .outputs
        .get(&output)
        .and_then(|row| row["name"].as_str())
        .unwrap_or(&output)
        .to_owned();
    let edge = if matches!(kind, SceneCommand::Panel { corner: true, .. }) {
        match args["corner"].as_str() {
            Some("tl") => "left",
            Some("bl") => "bottom",
            Some("br") => "right",
            Some("tr") => "top",
            _ => {
                return (
                    10,
                    json!({"error_code":"BAD_ARGUMENT","message":"unknown corner"}),
                );
            }
        }
    } else {
        args["edge"].as_str().unwrap_or("bottom")
    };
    if shell_edge(edge).is_none() {
        return (
            10,
            json!({"error_code":"BAD_ARGUMENT","message":"unknown edge"}),
        );
    }
    let result = match kind {
        SceneCommand::Ping => json!({"service":service,"status":"ok","engine":"viu"}),
        SceneCommand::Info => {
            json!({"service":service,"engine":"viu","commands":SceneCommand::names(service),"pending":["dialogs","corner menus"],"window_previews":false})
        }
        SceneCommand::List => shared
            .store
            .list(&comp_model::snapshot::output_key(&output)),
        SceneCommand::PropsGet => {
            json!({"panels":shared.panels.iter().map(|((o,e),p)|json!({"output":o,"edge":e,"mode":p.snapshot().mode.as_str(),"mapped":p.snapshot().mapped,"settings_owner":shell_edge(e).and_then(|edge|shared.preferences.record(edge)).map(|record|record.id.as_str())})).collect::<Vec<_>>(),"corner_events":shared.corner_events,"corner_phases":shared.corner_phases,"window_previews":false})
        }
        SceneCommand::PanelState => {
            let p = shared.panel(&output, edge).snapshot();
            json!({"edge":edge,"mode":p.mode.as_str(),"mapped":p.mapped,"fraction":p.visible_fraction})
        }
        SceneCommand::PanelPageSet => {
            let Some(id) = args["id"].as_str() else {
                return (10, json!({"error_code":"BAD_ARGUMENT"}));
            };
            let known = shared.store.pages().seat(id).is_some_and(|seat| {
                (seat.output == output || seat.output == comp_model::snapshot::output_key(&output))
                    && seat.edge.as_str() == edge
            });
            if !known {
                return (10, json!({"error_code":"UNKNOWN_PAGE"}));
            }
            shared
                .selected
                .insert((output.clone(), edge.into()), id.into());
            shared.wake();
            json!({"accepted":true,"applied":true,"page":id})
        }
        SceneCommand::PanelResize => {
            let Some(px) = args["thickness_px"].as_f64().filter(|v| v.is_finite()) else {
                return (10, json!({"error_code":"BAD_ARGUMENT"}));
            };
            if shared
                .panel(&output, edge)
                .resize_thickness(
                    px as f32,
                    edges::resize_thickness_range(shell_edge(edge).unwrap()),
                )
                .is_err()
            {
                return (10, json!({"error_code":"PANEL_RESIZE_REJECTED"}));
            }
            shared.wake();
            json!({"accepted":true,"applied":true})
        }
        SceneCommand::PanelMode | SceneCommand::Panel { .. } => {
            let input = match kind {
                SceneCommand::PanelMode => match args["mode"].as_str().and_then(PanelMode::parse) {
                    Some(mode) => PanelInput::SetMode(mode),
                    None => return (10, json!({"error_code":"INVALID_ARGS","field":"mode"})),
                },
                SceneCommand::Panel { kind, .. } => match kind {
                    crate::panels::PanelCommand::Show => PanelInput::Reveal,
                    crate::panels::PanelCommand::Hide => PanelInput::Hide,
                    crate::panels::PanelCommand::Toggle => PanelInput::ToggleShown,
                    crate::panels::PanelCommand::Pin => PanelInput::Pin,
                    crate::panels::PanelCommand::Unpin => PanelInput::Release,
                    crate::panels::PanelCommand::Dock => PanelInput::Dock,
                },
                _ => unreachable!(),
            };
            shared.input(&output, edge, input);
            json!({"edge":edge,"mode":shared.panel(&output,edge).snapshot().mode.as_str()})
        }
        SceneCommand::Frame => match shared
            .store
            .arm_frame(args["scene"].as_str().unwrap_or_default(), &command.from)
        {
            Ok(()) => json!({"armed":true}),
            Err(error) => return (10, error),
        },
        SceneCommand::Layout => {
            return (
                10,
                json!({"error_code":"SCENE_NOT_RENDERED","message":"Client layout readback is pending"}),
            );
        }
        SceneCommand::ToastShow | SceneCommand::ToastHide => {
            let name = args["scene"].as_str().unwrap_or_default();
            let visible = kind == SceneCommand::ToastShow;
            let height = if visible && args.get("h").is_some() {
                let Some(h) = args["h"]
                    .as_f64()
                    .or_else(|| args["h"].as_str().and_then(|h| h.parse::<f64>().ok()))
                    .filter(|h| h.is_finite())
                else {
                    return (
                        10,
                        json!({"error_code":"INVALID_HEIGHT","message":"h must be a finite number"}),
                    );
                };
                Some(h.clamp(60.0, 2048.0) as f32)
            } else {
                None
            };
            if output.is_empty() {
                return (10, json!({"error_code":"OUTPUT_NOT_READY"}));
            }
            if let Err(error) = shared.store.set_toast_visible(
                name,
                visible,
                &comp_model::snapshot::output_key(&output),
                height,
            ) {
                return (10, error);
            }
            shared.wake();
            json!({"scene":name,"visible":visible,"applied":true})
        }
        SceneCommand::Load
        | SceneCommand::Validate
        | SceneCommand::Patch
        | SceneCommand::Get
        | SceneCommand::Describe
        | SceneCommand::Unload
        | SceneCommand::Watch => {
            if output.is_empty() {
                return (10, json!({"error_code":"OUTPUT_NOT_READY"}));
            }
            let output = comp_model::snapshot::output_key(&output);
            let mut mount = SceneMount {
                output: &output,
                owner: &command.from,
                accepted_at: command.generation,
            };
            let result = shared.store.dispatch(kind, &command.body, args, &mut mount);
            shared.publish.extend(result.publish);
            shared.wake();
            return (
                result.rc,
                serde_json::from_str(&result.body).unwrap_or(Value::Null),
            );
        }
        _ => {
            return (
                10,
                json!({"error_code":"NOT_IMPLEMENTED","message":"Dialog and menu presentation follows the furniture client"}),
            );
        }
    };
    (0, result)
}
