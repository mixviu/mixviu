//! Inline deck text through the real scene painter, without a desktop.
use serde_json::json;
use viu::view::SceneView;

macro_rules! page {
    ($name:ident, $file:literal) => {
        #[test]
        fn $name() {
            let ctx = egui::Context::default();
            let mut view = SceneView::new(include_str!(concat!("../../../apps/gallery/deck-pages/", $file, ".scene.mix"))).unwrap();
            let raw = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1600.,900.))),
                ..Default::default()
            };
            let mut output = ctx.run_ui(raw, |ui| { view.show(ui); });
            assert!(!output.shapes.is_empty());
            let root = view.geometry["root"];
            for (id, rect) in &view.geometry {
                assert!(rect.is_finite(), "{id}");
                assert!(rect.width() <= root.width()+1., "{id}: {rect:?}");
                if id != "root" {
                    assert!(rect.bottom() <= 800., "{id}: content extends into rail: {rect:?}");
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
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| { view.show(ui); });
    let job = output.shapes.iter().find_map(|s| match &s.shape {
        egui::Shape::Text(t) if t.galley.job.text == "fn hello old\nnew" => Some(&t.galley.job),
        _ => None,
    }).expect("one inline layout, preserving newline and spaces");
    assert_eq!(job.sections.len(), 4);
    assert_eq!(job.sections[0].format.color, egui::Color32::from_rgb(255,143,163));
    assert_eq!(job.sections[1].format.color, egui::Color32::from_rgb(127,212,255));
    assert!(job.sections[2].format.strikethrough.width > 0.);
    assert!(job.sections[3].format.background.a() > 0);
    assert!(view.geometry["root"].height() > 25.);
    output.textures_delta.clear();
}
#[test]
fn malformed_bound_runs_refuse_before_rendering() {
    let nodes = json!({"root":{"widget":"text","text":"","spans":"= $model.runs"}});
    let source = format!("---\nscene: 1\nname: deck\ncitizen: test\n---\n```mix\n{nodes}\n```\n");
    assert!(SceneView::with_model(&source,json!({"runs":[{"text":"x","role":"typo"}]})).is_err());
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
        .wgpu()
        .build_ui(move |ui| { view.show(ui); });
    harness.run();
    let image = harness.render().expect("native offscreen render");
    assert_eq!(image.dimensions(), (3840,2160));
    let out = dir.join("viu-slide.png");
    image.save(&out).unwrap();
    println!("DECK_PNG={}", out.display());
}
