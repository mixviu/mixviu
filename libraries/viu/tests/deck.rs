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
    rasters: viu::raster::Rasters,
    runtime: Runtime,
    wakes: mpsc::Receiver<()>,
    calls: viu::calls::Calls,
}
#[test]
fn authored_reference_nodes_resolve() {
    let doc: Value = serde_json::from_str(include_str!(
        "../../../apps/deck/examples/vfl-self-recording.json"
    ))
    .unwrap();
    for slide in doc["slides"].as_array().unwrap() {
        let source = format!(
            "---\nscene: 1\nname: reference\ncitizen: deck\n---\n```mix\n{}\n```\n",
            slide["scene"]
        );
        let document = scene::parse(&source).unwrap();
        scene::resolve(&document).unwrap_or_else(|e| panic!("{}: {e:?}", slide["id"]));
        assert!(!source.contains("STUB"));
    }
}

#[test]
fn approved_cyclops_asset_paints_a_white_ring_and_red_eye() {
    let source = include_str!("../../../apps/gallery/deck-pages/brand.scene.mix");
    let mut view = SceneView::new(source).unwrap();
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1600., 900.))
        .with_context_setup(|ctx| {
            viu::fonts::install(ctx,400,600);
            viu::icons::install(ctx);
        })
        .wgpu()
        .build_ui(move |ui| {
            view.show(ui);
        });
    harness.run();
    let pixels = harness.render().unwrap();
    let eye = pixels
        .enumerate_pixels()
        .filter(|(x, y, p)| *x < 320 && *y < 300 && p[0] > 200 && p[1] < 20 && p[2] < 20)
        .count();
    let ring = pixels
        .enumerate_pixels()
        .filter(|(x, y, p)| *x < 320 && *y < 300 && p[0] > 240 && p[1] > 240 && p[2] > 240)
        .count();
    assert!(eye > 500, "red eye pixels: {eye}");
    assert!(ring > 5000, "white ring pixels: {ring}");
}

#[test]
fn entrance_uses_the_oracle_beat_plus_delay_without_an_extra_lead() {
    let mut deck = Deck::new();
    deck.open();
    deck.call(
        "deck.play",
        json!({"slides":["before"],"beats":[[0.9,3,5]]}),
    );
    deck.call("viu.presentation", json!({"time":100}));
    for at in [1.0, 1.05, 1.1, 1.5] {
        let model = deck
            .call("viu.presentation", json!({"time":100.0+at}))
            .model
            .unwrap();
        let values = motion::scene::Entrance::Name("left".into())
            .playback(0., 0, false)
            .sample(at - 0.9 - 0.15);
        let actual = model["build"]["root.1"]["opacity"].as_f64().unwrap();
        assert!((actual - values.get(motion::Property::Opacity)).abs() < 1e-6);
        println!(
            "ENTRANCE time={at} beat=0.9 delay=0.15 age={} opacity={actual}",
            at - 1.05
        );
    }
}

#[test]
fn nested_grow_cards_keep_the_right_edge_inside_the_canvas() {
    let source = include_str!("../../../apps/gallery/deck-pages/nested_grow.scene.mix");
    let mut view = SceneView::new(source).unwrap();
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1700., 950.))
        .with_context_setup(|ctx| viu::fonts::install(ctx,400,600))
        .wgpu()
        .build_ui(move |ui| {
            ui.set_width(1600.);
            view.show(ui);
        });
    harness.run();
    let pixels = harness.render().unwrap();
    // The gallery canvas ends at 1600 (+ the harness's panel margin). The
    // overlong code/flow must never paint its red/cyan/green content here.
    for x in 1620..1690 {
        for y in 20..700 {
            let p = pixels.get_pixel(x, y).0;
            assert!(
                p[0] < 40 && p[1] < 40 && p[2] < 40,
                "overflow at {x},{y}: {p:?}"
            );
        }
    }
}

#[test]
fn raster_open_pins_pixels_and_refuses_unsafe_sources_before_navigation() {
    let mut deck = Deck::new();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("deck.json");
    let png = dir.path().join("shot.png");
    image::RgbaImage::from_pixel(16, 16, image::Rgba([255, 0, 0, 255]))
        .save(&png)
        .unwrap();
    let open = |deck: &mut Deck, src: &str| {
        std::fs::write(&path,json!({"schema":"deck.v1","slides":[{"id":"shot","steps":1,"scene":{"root":{"widget":"image","src":src,"w":64,"h":64}}}]}).to_string()).unwrap();
        deck.call("deck.open", json!({"path":path})).reply.unwrap()
    };
    assert_eq!(open(&mut deck, "shot.png").rc, 0);
    std::fs::remove_file(&png).unwrap();
    let done = deck.call("deck.goto", json!({"slide":"shot"}));
    assert_eq!(done.reply.unwrap().rc, 0);
    let model = done.model.unwrap();
    let mut view = SceneView::with_model(
        model["presentation"]["scene"].as_str().unwrap(),
        model.clone(),
    )
    .unwrap();
    view.set_rasters(deck.rasters.clone());
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(100., 100.))
        .wgpu()
        .build_ui(move |ui| {
            view.show(ui);
        });
    harness.run();
    let pixels = harness.render().unwrap();
    assert_eq!(pixels.get_pixel(32, 32).0, [255, 0, 0, 255]);
    let outside = tempfile::NamedTempFile::new().unwrap();
    assert_eq!(open(&mut deck, outside.path().to_str().unwrap()).rc, 10);
    std::os::unix::fs::symlink(outside.path(), &png).unwrap();
    assert_eq!(open(&mut deck, "shot.png").rc, 10);
    std::fs::write(dir.path().join("bad.png"), b"not an image").unwrap();
    assert_eq!(open(&mut deck, "bad.png").rc, 10);
    std::fs::File::create(dir.path().join("large.png"))
        .unwrap()
        .set_len(32 * 1024 * 1024 + 1)
        .unwrap();
    assert_eq!(open(&mut deck, "large.png").rc, 10);
    assert_eq!(
        deck.rasters.bytes(),
        16 * 16 * 4,
        "refused opens retain the active deck only"
    );
}
impl Deck {
    fn new() -> Self {
        let (tx, wakes) = mpsc::channel();
        let calls = viu::calls::Calls::default();
        let rasters = viu::raster::Rasters::default();
        let mut runtime = Runtime::with_services(
            8,
            Arc::new(move || {
                let _ = tx.send(());
            }),
            None,
            viu::runtime::Services {
                calls: Some(calls.clone()),
                rasters: Some(rasters.clone()),
                ..Default::default()
            },
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
        let s = Self {
            rasters,
            runtime,
            wakes,
            calls,
        };
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

#[test]
fn play_replies_immediately_and_survives_caller_cancellation_past_180_seconds() {
    let mut deck = Deck::new();
    deck.open();
    let (tx, _rx) = tokio::sync::oneshot::channel();
    let token = deck
        .calls
        .begin(tx, Arc::new(std::sync::atomic::AtomicBool::new(false)))
        .unwrap();
    deck.runtime
        .event_with_headers(
            "deck.play".into(),
            json!({"slides":["sting"],"beats":[[0.9,3600]]}),
            std::collections::BTreeMap::from([("viu-call".into(), token.clone())]),
        )
        .unwrap();
    let start = deck.next();
    let reply = start.reply.unwrap();
    assert_eq!(reply.rc, 0);
    assert!(reply.body["play_id"].as_u64().is_some());
    assert!(start.deferred_answers.is_empty());
    deck.call("viu.presentation", json!({"time":100}));
    deck.call("viu.call.cancelled", json!({"token":token}));
    deck.call("viu.presentation", json!({"time":281}));
    assert_eq!(
        deck.call("deck.state", json!({})).reply.unwrap().body["playing"],
        true
    );
    let end = deck.call("viu.presentation", json!({"time":3700}));
    assert_eq!(end.model.unwrap()["presentation"]["clock"], false);
    let state = deck.call("deck.state", json!({})).reply.unwrap().body;
    assert_eq!(state["finished"], true);
    assert_eq!(state["play_id"], reply.body["play_id"]);
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
fn native_schema_refusal_names_the_slide_and_preserves_the_open_deck() {
    let mut deck = Deck::new();
    deck.open();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("bad.json");
    std::fs::write(&path,json!({"schema":"deck.v1","slides":[{"id":"broken","steps":1,"scene":{"root":{"widget":"not-a-widget"}}}]}).to_string()).unwrap();
    let refused = deck.call("deck.open", json!({"path":path})).reply.unwrap();
    assert_eq!(refused.rc, 10);
    assert_eq!(refused.body["error_code"], "DECK_SCENE");
    assert!(
        refused.body["message"]
            .as_str()
            .unwrap()
            .contains("slides(broken).scene")
    );
    assert_eq!(
        deck.call("deck.state", json!({})).reply.unwrap().body["slide"],
        "sting"
    );
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
fn until_removes_painted_nodes_at_the_beat_and_goto_uses_the_same_build() {
    let mut deck = Deck::new();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("build.json");
    std::fs::write(&path,json!({"schema":"deck.v1","slides":[{"id":"build","steps":2,"scene":{"root":{"widget":"column","children":["before","after"]},"before":{"widget":"text","text":"Before"},"after":{"widget":"text","text":"After"}},"build":{"before":{"b":0,"until":0},"after":{"b":1}}}]}).to_string()).unwrap();
    assert_eq!(
        deck.call("deck.open", json!({"path":path}))
            .reply
            .unwrap()
            .rc,
        0
    );
    deck.call("deck.play", json!({"beats":[[0,2,4]]}));
    let first = deck
        .call("viu.presentation", json!({"time":100}))
        .model
        .unwrap();
    let (view, mut out) = frame(first);
    assert!(view.geometry.contains_key("before"));
    assert!(
        view.geometry.contains_key("after"),
        "a future build keeps layout space"
    );
    out.textures_delta.clear();
    let next = deck
        .call("viu.presentation", json!({"time":102.8}))
        .model
        .unwrap();
    let (view, mut out) = frame(next);
    assert!(!view.geometry.contains_key("before"));
    assert!(view.geometry.contains_key("after"));
    assert!(
        !out.shapes
            .iter()
            .any(|s| matches!(&s.shape,egui::Shape::Text(t) if t.galley.job.text=="Before"))
    );
    out.textures_delta.clear();
    let manual = deck
        .call("deck.goto", json!({"slide":"build","step":1}))
        .model
        .unwrap();
    let (view, mut out) = frame(manual);
    assert!(!view.geometry.contains_key("before"));
    assert!(
        out.shapes
            .iter()
            .any(|s| matches!(&s.shape,egui::Shape::Text(t) if t.galley.job.text=="After"))
    );
    out.textures_delta.clear();
}
#[test]
#[ignore = "CBC offscreen evidence; PNGs written in temp_dir()/mixviu-deck-app-review"]
fn render_reference_scenes() {
    let mut deck = Deck::new();
    deck.open();
    let dir = std::env::temp_dir().join("mixviu-deck-app-review");
    std::fs::create_dir_all(&dir).unwrap();
    for id in ["before", "recorder"] {
        // Match Chrome's #s=ID&t=10&beats=0,3,15. All supported
        // entrances and the HTML count-up have settled by this scene time.
        deck.call("deck.play", json!({"slides":[id],"beats":[[0,3,15]]}));
        deck.call("viu.presentation", json!({"time":100}));
        let model = deck
            .call("viu.presentation", json!({"time":110}))
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
        println!("DECK_SAMPLE={id} time=10 beats=0,3,15");
    }
}
