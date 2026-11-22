// SPDX-License-Identifier: MIT OR Apache-2.0
use super::layout::geometry;
use super::*;

#[test]
fn authored_deck_builds_settle_in_ci() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../apps/deck/examples/vfl-self-recording.json");
    let doc: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
    let mut deck = Deck::new();
    assert_eq!(
        deck.call("deck.open", json!({"path":path}))
            .reply
            .unwrap()
            .rc,
        0
    );
    let mut count = 0;
    for slide in doc["slides"].as_array().unwrap() {
        for step in 0..slide["steps"].as_u64().unwrap() {
            let model = deck
                .call("deck.goto", json!({"slide":slide["id"],"step":step}))
                .model
                .unwrap();
            let mut view = SceneView::with_model(
                model["presentation"]["scene"].as_str().unwrap(),
                model.clone(),
            )
            .unwrap();
            view.set_rasters(deck.rasters.clone());
            let ctx = egui::Context::default();
            viu::fonts::install(&ctx, 400, 600);
            viu::icons::install(&ctx);
            for frame in 0..120 {
                let mut out = ctx.run_ui(
                    egui::RawInput {
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
                out.textures_delta.clear();
                if frame == 119 {
                    assert!(
                        !out.platform_output.requested_discard(),
                        "{}-{step}: {:?}",
                        slide["id"],
                        out.platform_output.request_discard_reasons
                    );
                }
            }
            count += 1;
        }
    }
    assert_eq!(count, 21);
}
#[test]
fn comparison_row_aligns_actual_glyph_baselines() {
    let source = "---\nscene: 1\nname: baseline\ncitizen: test\n---\n```mix\n{root:{widget:\"row\",align:\"baseline\",children:[\"big\",\"small\"]},big:{widget:\"text\",text:\"6.3 s\",size:44,font:\"Liberation Sans\",weight:700},small:{widget:\"text\",text:\"median\",size:20,font:\"Liberation Sans\"}}\n```\n";
    let mut view = SceneView::new(source).unwrap();
    let ctx = egui::Context::default();
    viu::fonts::install(&ctx, 400, 600);
    let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
        view.show(ui);
    });
    out.textures_delta.clear();
    let baselines: Vec<_> = out
        .shapes
        .iter()
        .filter_map(|s| {
            if let egui::Shape::Text(t) = &s.shape {
                let r = &t.galley.rows[0];
                Some(t.pos.y + r.pos.y + r.glyphs[0].pos.y)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(baselines.len(), 2);
    assert!((baselines[0] - baselines[1]).abs() < 0.01, "{baselines:?}");
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
        let parsed =
            scene::parse(&source).unwrap_or_else(|error| panic!("{}: {error:?}", slide["id"]));
        scene::resolve(&parsed).unwrap_or_else(|error| panic!("{}: {error:?}", slide["id"]));
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
            let mut settle_view = SceneView::with_model(
                model["presentation"]["scene"].as_str().unwrap(),
                model.clone(),
            )
            .unwrap();
            settle_view.set_rasters(deck.rasters.clone());
            let settle = egui::Context::default();
            viu::fonts::install(&settle, 400, 600);
            for frame in 0..24 {
                let mut output = settle.run_ui(
                    egui::RawInput {
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1600., 900.),
                        )),
                        ..Default::default()
                    },
                    |ui| {
                        settle_view.show(ui);
                    },
                );
                output.textures_delta.clear();
                if frame == 23 {
                    assert!(
                        !output.platform_output.requested_discard(),
                        "{id}-{step}: {:?}",
                        output.platform_output.request_discard_reasons
                    );
                }
            }
            println!("SETTLED {id}-{step}: no discard reasons after 24 frames");
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

#[test]
fn gallery_and_workshop_layouts_settle() {
    let mut sources = vec![];
    for entry in std::fs::read_dir(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../apps/gallery/deck-pages"
    ))
    .unwrap()
    {
        let p = entry.unwrap().path();
        if p.extension().is_some_and(|x| x == "mix") {
            sources.push(std::fs::read_to_string(p).unwrap());
        }
    }
    // Workshop's actual container definitions, with deterministic control data.
    let d = scene::parse(include_str!("../../../../apps/workshop/scene.mix")).unwrap();
    for root in ["body-actions", "mesh-top-lists"] {
        let mut nodes = serde_json::Map::new();
        let raw = &d.nodes[root];
        let mut ports = raw.ports.clone();
        ports.shift_remove("hidden");
        ports.insert("widget".into(), json!(raw.widget));
        nodes.insert("root".into(), serde_json::to_value(ports).unwrap());
        for child in raw.ports["children"].as_array().unwrap() {
            let id = child.as_str().unwrap();
            nodes.insert(id.into(), json!({"widget":"button","label":id}));
        }
        sources.push(format!(
            "---\nscene: 1\nname: workshop-layout\ncitizen: test\n---\n```mix\n{}\n```\n",
            Value::Object(nodes)
        ));
    }
    for source in sources {
        let mut v = SceneView::new(&source).unwrap();
        let ctx = egui::Context::default();
        for frame in 0..24 {
            let mut o = ctx.run_ui(egui::RawInput::default(), |ui| {
                v.show(ui);
            });
            o.textures_delta.clear();
            if frame == 23 {
                assert!(
                    !o.platform_output.requested_discard(),
                    "{}: {:?}",
                    source,
                    o.platform_output.request_discard_reasons
                );
            }
        }
    }
}

#[test]
fn between_keeps_a_nested_date_at_the_right_edge() {
    let rects = geometry(
        r##"{root:{widget:"row",fill:true,justify:"between",cross_align:"center",children:["chips","date"]},chips:{widget:"row",children:["chip"]},chip:{widget:"text",text:"#210 #215 #214"},date:{widget:"column",padding_right:9,children:["label"]},label:{widget:"text",text:"21 NOV 2026",wrap:true}}"##,
    );
    assert!(rects["label"].left() > rects["chips"].right() + 100.);
    assert!(
        (rects["root"].right() - 9. - rects["label"].right()).abs() < 1.,
        "{rects:?}"
    );
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
