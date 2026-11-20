// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Duration,
};
use viu::{
    runtime::{Completion, Program, Runtime},
    view::SceneView,
};

struct Deck {
    runtime: Runtime,
    wakes: mpsc::Receiver<()>,
}
impl Deck {
    fn new() -> Self {
        let (tx, wakes) = mpsc::channel();
        let mut runtime = Runtime::new(
            8,
            Arc::new(move || {
                let _ = tx.send(());
            }),
            None,
        );
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/deck/app.mix");
        runtime
            .load(Program {
                source: std::fs::read_to_string(&path).unwrap(),
                path,
                scene: "deck".into(),
                citizen: "deck".into(),
            })
            .unwrap();
        let s = Self { runtime, wakes };
        assert!(
            s.next().model.is_some(),
            "startup must admit deck.open on the real host"
        );
        s
    }
    fn next(&self) -> Completion {
        self.wakes.recv_timeout(Duration::from_secs(30)).unwrap();
        let done = self.runtime.try_completion().unwrap();
        assert!(done.error.is_none(), "{:?}\n{}", done.error, done.stderr);
        done
    }
    fn call(&mut self, command: &str, body: Value) -> Completion {
        self.runtime.event(command.into(), body).unwrap();
        self.next()
    }
    fn open(&mut self) {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../apps/deck/examples/vfl-self-recording.json");
        let done = self.call("deck.open", json!({"path":path}));
        assert_eq!(done.reply.as_ref().unwrap().rc, 0, "{:?}", done.reply);
    }
}
fn frame(model: Value) -> (SceneView, egui::FullOutput) {
    let ctx = egui::Context::default();
    let mut view = SceneView::with_model(
        model["presentation"]["scene"].as_str().unwrap(),
        model.clone(),
    )
    .unwrap();
    let output = ctx.run_ui(
        egui::RawInput {
            time: Some(0.),
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1600., 900.),
            )),
            ..Default::default()
        },
        |ui| {
            view.show(ui);
        },
    );
    (view, output)
}
#[test]
fn reference_opens_every_scene_and_goto_renders_exact_step() {
    let mut deck = Deck::new();
    deck.open();
    let doc: Value = serde_json::from_str(include_str!(
        "../../../apps/deck/examples/vfl-self-recording.json"
    ))
    .unwrap();
    for s in doc["slides"].as_array().unwrap() {
        for step in 0..s["steps"].as_u64().unwrap() {
            let done = deck.call("deck.goto", json!({"slide":s["id"],"step":step}));
            assert_eq!(done.reply.unwrap().rc, 0);
            let model = done.model.unwrap();
            assert_eq!(model["presentation"]["clock"], false);
            let (_, mut output) = frame(model);
            assert!(!output.shapes.is_empty());
            output.textures_delta.clear();
        }
    }
}
#[test]
fn play_samples_delays_and_freezes_the_last_sample() {
    let mut deck = Deck::new();
    deck.open();
    let done = deck.call(
        "deck.play",
        json!({"slides":["before","recorder"],"beats":[[0,2,4],[0,2,4]]}),
    );
    assert_eq!(done.reply.unwrap().rc, 0);
    let first = done.model.unwrap();
    assert_eq!(first["build"]["root.2"]["opacity"], 0);
    for (at, step, clock) in [
        (100., 0, true),
        (102., 1, true),
        (104., 0, true),
        (106., 1, true),
        (108., 1, false),
    ] {
        let done = deck.call("viu.presentation", json!({"time":at}));
        let model = done.model.unwrap();
        assert_eq!(model["presentation"]["clock"], clock);
        let mut out = frame(model).1;
        out.textures_delta.clear();
        let state = deck.call("deck.state", json!({})).reply.unwrap().body;
        assert_eq!(state["step"], step);
    }
    let state = deck.call("deck.state", json!({})).reply.unwrap().body;
    assert_eq!(state["finished"], true);
    assert!(
        deck.call("viu.presentation", json!({"time":1000}))
            .model
            .is_none()
    );
}
#[test]
fn invalid_goto_and_actions_have_typed_replies() {
    let mut deck = Deck::new();
    deck.open();
    let done = deck.call("deck.goto", json!({"slide":"before","step":99}));
    assert_eq!(done.reply.unwrap().body["error_code"], "DECK_RANGE");
    let done = deck.call("deck.actions", json!({}));
    assert_eq!(
        done.reply.unwrap().body["actions"]
            .as_array()
            .unwrap()
            .len(),
        6
    );
    assert_eq!(
        deck.call("deck.execute", json!({"id":"deck.next"}))
            .reply
            .unwrap()
            .rc,
        0
    );
    assert_eq!(deck.call("deck.prev", json!({})).reply.unwrap().rc, 0);
}

#[test]
fn settled_manual_slide_arms_no_repaint() {
    let mut deck = Deck::new();
    deck.open();
    let model = deck
        .call("deck.goto", json!({"slide":"before","step":1}))
        .model
        .unwrap();
    assert_eq!(model["presentation"]["clock"], false);
    let mut view = SceneView::with_model(
        model["presentation"]["scene"].as_str().unwrap(),
        model.clone(),
    )
    .unwrap();
    let ctx = egui::Context::default();
    for i in 0..6 {
        let mut out = ctx.run_ui(
            egui::RawInput {
                time: Some(i as f64),
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1600., 900.),
                )),
                ..Default::default()
            },
            |ui| {
                view.show(ui);
            },
        );
        if i == 5 {
            assert!(
                out.viewport_output
                    .values()
                    .all(|v| v.repaint_delay == Duration::MAX)
            );
        }
        out.textures_delta.clear();
    }
}
#[test]
#[ignore = "CBC offscreen evidence; PNGs written in temp_dir()/mixviu-deck-app-review"]
fn render_reference_scenes() {
    let mut deck = Deck::new();
    deck.open();
    let dir = std::env::temp_dir().join("mixviu-deck-app-review");
    std::fs::create_dir_all(&dir).unwrap();
    for id in ["before", "recorder"] {
        let model = deck
            .call("deck.goto", json!({"slide":id,"step":1}))
            .model
            .unwrap();
        let mut view = SceneView::with_model(
            model["presentation"]["scene"].as_str().unwrap(),
            model.clone(),
        )
        .unwrap();
        let mut harness = egui_kittest::Harness::builder()
            .with_size(egui::vec2(1600., 900.))
            .with_pixels_per_point(2.4)
            .with_context_setup(|ctx| viu::fonts::install(ctx, 400, 600))
            .wgpu()
            .build_ui(move |ui| {
                view.show(ui);
            });
        harness.run();
        let image = harness.render().unwrap();
        assert_eq!(image.dimensions(), (3840, 2160));
        let path = dir.join(format!("viu-{id}.png"));
        image.save(&path).unwrap();
        println!("DECK_PNG={}", path.display());
    }
}
