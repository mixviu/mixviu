// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;

fn geometry(source: &str) -> std::collections::BTreeMap<String, egui::Rect> {
    let mut view = SceneView::new(&format!(
        "---\nscene: 1\nname: layout-test\ncitizen: test\n---\n```mix\n{source}\n```\n"
    ))
    .unwrap();
    let ctx = egui::Context::default();
    for _ in 0..8 {
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1000., 600.),
                )),
                ..Default::default()
            },
            |ui| {
                view.show(ui);
            },
        );
        output.textures_delta.clear();
    }
    view.geometry
}

#[test]
fn between_keeps_a_nested_date_at_the_right_edge() {
    let rects = geometry(
        r##"{root:{widget:"row",fill:true,justify:"between",children:["chips","date"]},chips:{widget:"row",children:["chip"]},chip:{widget:"text",text:"#210 #215 #214"},date:{widget:"column",padding_right:9,children:["label"]},label:{widget:"text",text:"21 NOV 2026"}}"##,
    );
    assert!(rects["label"].left() > rects["chips"].right() + 100.);
    assert!(
        (rects["root"].right() - 9. - rects["label"].right()).abs() < 1.,
        "{rects:?}"
    );
}

#[test]
fn grid_intrinsics_cannot_move_the_next_column() {
    let rects = geometry(
        r##"{root:{widget:"grid",columns:2,column_gap:40,children:["left","right"]},left:{widget:"column",fill:true,children:["wide"]},wide:{widget:"text",text:"a_very_long_unbroken_value_that_is_wider_than_the_first_grid_cell_and_must_not_move_the_second_cell",size:30},right:{widget:"column",fill:true,children:["label"]},label:{widget:"text",text:"Right column"}}"##,
    );
    let expected = rects["root"].left() + (rects["root"].width() + 40.) / 2.;
    assert!((rects["right"].left() - expected).abs() < 1., "{rects:?}");
}

#[test]
fn explicit_line_height_sizes_each_text_line() {
    let mut view=SceneView::new("---\nscene: 1\nname: leading\ncitizen: test\n---\n```mix\n{root:{widget:\"text\",text:\"first\\nsecond\",size:20,line_height:60}}\n```\n").unwrap();
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        view.show(ui);
    });
    output.textures_delta.clear();
    assert!((view.geometry["root"].height() - 120.).abs() < 1.);
}

#[test]
fn wrapped_text_obeys_its_maximum_width() {
    let mut view=SceneView::new("---\nscene: 1\nname: bounded\ncitizen: test\n---\n```mix\n{root:{widget:\"text\",text:\"first second third fourth\",size:20,max_width:80,wrap:true}}\n```\n").unwrap();
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        view.show(ui);
    });
    output.textures_delta.clear();
    let rect = view.geometry["root"];
    assert!(rect.width() <= 80. && rect.height() > 40., "{rect:?}");
}

#[test]
fn column_justify_centres_the_content() {
    let mut view = SceneView::new("---\nscene: 1\nname: centre\ncitizen: test\n---\n```mix\n{root:{widget:\"column\",height:400,justify:\"center\",children:[\"a\"]},a:{widget:\"text\",text:\"Centred\",size:30}}\n```\n").unwrap();
    let ctx = egui::Context::default();
    viu::fonts::install(&ctx, 400, 600);
    for _ in 0..4 {
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(600., 600.),
                )),
                ..Default::default()
            },
            |ui| {
                view.show(ui);
            },
        );
        output.textures_delta.clear();
    }
    let root = view.geometry["root"];
    let text = view.geometry["a"];
    assert!(
        (root.center().y - text.center().y).abs() < 1.,
        "{root:?} {text:?}"
    );
}

#[test]
#[ignore = "explicit CBC evidence; DECK_LAYOUT_INPUT and DECK_LAYOUT_OUTPUT required"]
fn render_layout_steps() {
    let path = PathBuf::from(std::env::var_os("DECK_LAYOUT_INPUT").expect("DECK_LAYOUT_INPUT"));
    let out = PathBuf::from(std::env::var_os("DECK_LAYOUT_OUTPUT").expect("DECK_LAYOUT_OUTPUT"));
    std::fs::create_dir_all(&out).unwrap();
    let document: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    for slide in document["slides"].as_array().unwrap() {
        let source = format!(
            "---\nscene: 1\nname: layout-input\ncitizen: deck\n---\n```mix\n{}\n```\n",
            slide["scene"]
        );
        let parsed=scene::parse(&source).unwrap_or_else(|error|panic!("{}: {error:?}",slide["id"]));
        scene::resolve(&parsed).unwrap_or_else(|error|panic!("{}: {error:?}",slide["id"]));
    }
    let mut deck = Deck::new();
    let opened = deck.call("deck.open", json!({"path":path})).reply.unwrap();
    assert_eq!(opened.rc, 0, "{opened:?}");
    for slide in document["slides"].as_array().unwrap() {
        let id = slide["id"].as_str().unwrap();
        for step in 0..slide["steps"].as_u64().unwrap() {
            let model = deck
                .call("deck.goto", json!({"slide":id,"step":step}))
                .model
                .unwrap();
            let mut view = SceneView::with_model(
                model["presentation"]["scene"].as_str().unwrap(),
                model.clone(),
            )
            .unwrap();
            view.set_rasters(deck.rasters.clone());
            let rects = Arc::new(std::sync::Mutex::new(json!({})));
            let captured = rects.clone();
            let mut harness = egui_kittest::Harness::builder()
                .with_size(egui::vec2(1600., 900.))
                .with_pixels_per_point(2.4)
                .with_context_setup(|ctx| {
                    viu::fonts::install(ctx, 400, 600);
                    viu::icons::install(ctx);
                })
                .wgpu()
                .build_ui(move |ui| {
                    let mut canvas = ui.new_child(egui::UiBuilder::new().max_rect(
                        egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1600., 900.)),
                    ));
                    view.show(&mut canvas);
                    *captured.lock().unwrap() = view
                        .geometry
                        .iter()
                        .map(|(k, r)| {
                            (
                                k.clone(),
                                json!([
                                    r.left() * 2.4,
                                    r.top() * 2.4,
                                    r.width() * 2.4,
                                    r.height() * 2.4
                                ]),
                            )
                        })
                        .collect();
                });
            harness.run();
            harness
                .render()
                .unwrap()
                .save(out.join(format!("{id}-{step}.png")))
                .unwrap();
            std::fs::write(
                out.join(format!("{id}-{step}.json")),
                serde_json::to_vec_pretty(&*rects.lock().unwrap()).unwrap(),
            )
            .unwrap();
        }
    }
}
