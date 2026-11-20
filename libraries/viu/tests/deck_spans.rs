//! Inline deck text through the real scene painter, without a desktop.
use serde_json::json;
use viu::view::SceneView;

macro_rules! page {
    ($name:ident, $file:literal) => {
        #[test]
        fn $name() {
            let ctx = egui::Context::default();
            let mut view = SceneView::new(include_str!(concat!(
                "../../../apps/gallery/deck-pages/",
                $file,
                ".scene.mix"
            )))
            .unwrap();
            let raw = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1600., 900.),
                )),
                ..Default::default()
            };
            let mut output = ctx.run_ui(raw, |ui| {
                view.show(ui);
            });
            output.textures_delta.clear();
            assert!(!output.shapes.is_empty());
            let root = view.geometry["root"];
            for (id, rect) in &view.geometry {
                assert!(rect.is_finite(), "{id}");
                assert!(rect.width() <= root.width() + 1., "{id}: {rect:?}");
                if id != "root" {
                    assert!(
                        rect.bottom() <= 800.,
                        "{id}: content extends into rail: {rect:?}"
                    );
                }
            }
            output.textures_delta.clear();
        }
    };
}
page!(typography, "typography");
page!(cards, "cards");
page!(flow, "flow");
page!(code, "code");
page!(terminal, "terminal");
page!(tree, "tree");
page!(chips, "chips");
page!(stat, "stat");
page!(mock, "mock");
page!(brand, "brand");
page!(topic, "topic");
page!(rail, "rail");

fn source() -> String {
    let nodes = json!({"root":{"widget":"text","text":"fallback","size":25,"mono":true,"spans":[
        {"text":"fn ","role":"keyword","color":"#ff8fa3"},
        {"text":"hello","role":"function","color":"#7fd4ff"},
        {"text":" old","strike":true,"color":"#5d6876"},
        {"text":"\nnew","highlight":true,"color":"#3cc8d8"}
    ]}});
    format!("---\nscene: 1\nname: deck\ncitizen: test\n---\n```mix\n{nodes}\n```\n")
}
#[test]
fn inline_runs_keep_layout_colours_history_and_highlight() {
    let ctx = egui::Context::default();
    let mut view = SceneView::new(&source()).unwrap();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        view.show(ui);
    });
    let job = output
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::Text(t) if t.galley.job.text == "fn hello old\nnew" => Some(&t.galley.job),
            _ => None,
        })
        .expect("one inline layout, preserving newline and spaces");
    assert_eq!(job.sections.len(), 4);
    assert_eq!(
        job.sections[0].format.color,
        egui::Color32::from_rgb(255, 143, 163)
    );
    assert_eq!(
        job.sections[1].format.color,
        egui::Color32::from_rgb(127, 212, 255)
    );
    assert!(job.sections[2].format.strikethrough.width > 0.);
    assert!(job.sections[3].format.background.a() > 0);
    assert!(view.geometry["root"].height() > 25.);
    output.textures_delta.clear();
}
#[test]
fn malformed_bound_runs_refuse_before_rendering() {
    let nodes = json!({"root":{"widget":"text","text":"","spans":"= $model.runs"}});
    let source = format!("---\nscene: 1\nname: deck\ncitizen: test\n---\n```mix\n{nodes}\n```\n");
    let model = json!({"runs":[{"text":"x","role":"typo"}]});
    let mut document = scene::parse(&source).unwrap();
    document.model = Some(model.clone());
    assert!(
        scene::lint(&document)
            .iter()
            .any(|d| d.code == "binding-type")
    );
    let resolved = scene::resolve(&document).unwrap();
    assert!(!resolved.nodes["root"].ports.contains_key("spans"));
    // Bad bound values follow the shared binding fallback policy; they do
    // not make an otherwise usable scene fail to open.
    assert!(SceneView::with_model(&source, model).is_ok());
}

#[test]
#[ignore = "CBC offscreen review: stage slide.scene.mix in temp_dir()/mixviu-deck-review"]
fn render_review_slide() {
    let dir = std::env::temp_dir().join("mixviu-deck-review");
    let source = std::fs::read_to_string(dir.join("slide.scene.mix")).unwrap();
    let mut view = SceneView::new(&source).unwrap();
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1600., 900.))
        .with_pixels_per_point(2.4)
        .with_context_setup(|ctx| viu::fonts::install(ctx, 400, 600))
        .wgpu()
        .build_ui(move |ui| {
            view.show(ui);
        });
    harness.run();
    let image = harness.render().expect("native offscreen render");
    assert_eq!(image.dimensions(), (3840, 2160));
    let out = dir.join("viu-slide.png");
    image.save(&out).unwrap();
    println!("DECK_PNG={}", out.display());
}

#[test]
#[ignore = "CBC release measurement; uses the same staged private review scene"]
fn review_slide_cost_and_idle() {
    let source =
        std::fs::read_to_string(std::env::temp_dir().join("mixviu-deck-review/slide.scene.mix"))
            .unwrap();
    assert!(source.contains("\"root\":{"));
    let source = source.replacen("\"root\":{", "\"root\":{\"enter\":\"up\",", 1);
    let mut view = SceneView::new(&source).unwrap();
    let ctx = egui::Context::default();
    viu::fonts::install(&ctx, 400, 600);
    ctx.set_pixels_per_point(2.4);
    let frame = |view: &mut SceneView, t| {
        ctx.run_ui(
            egui::RawInput {
                time: Some(t),
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1600., 900.),
                )),
                ..Default::default()
            },
            |ui| {
                view.show(ui);
            },
        )
    };
    frame(&mut view, 0.).drop_without_applying_deltas();
    let mut us = Vec::new();
    for i in 1..=1000 {
        let start = std::time::Instant::now();
        let mut out = frame(&mut view, i as f64 / 2000.);
        let _ = ctx.tessellate(std::mem::take(&mut out.shapes), out.pixels_per_point);
        out.textures_delta.clear();
        us.push(start.elapsed().as_secs_f64() * 1e6);
    }
    us.sort_by(f64::total_cmp);
    println!(
        "DECK_CPU_US median={:.3} p95={:.3} frames=1000; layout+paint+tessellation, no GPU",
        us[500], us[950]
    );
    for t in [2., 3., 4.] {
        frame(&mut view, t).drop_without_applying_deltas();
    }
    let mut settled = frame(&mut view, 5.);
    assert!(
        settled
            .viewport_output
            .values()
            .all(|v| v.repaint_delay == std::time::Duration::MAX),
        "idle viewport scheduled a repaint"
    );
    settled.textures_delta.clear();
    println!("DECK_IDLE_REPAINT=none");
}
