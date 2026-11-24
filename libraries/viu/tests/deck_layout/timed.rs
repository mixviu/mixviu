// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;

#[test]
fn held_equal_cards_during_parallel_animation() {
    let source = r##"---
scene: 1
name: held-cards
citizen: test
---
```mix
{root:{widget:"column",children:["cards","animation"]},
cards:{widget:"grid",columns:2,equal_height:true,children:["a","b"]},
a:{widget:"column",stretch_last:true,children:["heading","last"]},
heading:{widget:"text",text:"Held heading"},
last:{widget:"column",background:"#203040",children:["short"]},short:{widget:"text",text:"Short"},
b:{widget:"column",background:"#203040",children:["tall"]},tall:{widget:"spacer",size:100},
animation:{widget:"text",text:"Parallel animation",paint_effect:"= $model.effect"}}
```
"##;
    let mut view =
        SceneView::with_model(source, json!({"effect":{"preset":"spin","time":0.2}})).unwrap();
    let geometry = Arc::new(std::sync::Mutex::new(view.geometry.clone()));
    let captured = geometry.clone();
    let clock = Arc::new(std::sync::Mutex::new(0.2_f64));
    let time = clock.clone();
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(320., 240.))
        .with_pixels_per_point(2.4)
        .wgpu()
        .build_ui(move |ui| {
            view.apply_model(json!({"effect":{"preset":"spin","time":*time.lock().unwrap()}}))
                .unwrap();
            view.show(ui);
            *captured.lock().unwrap() = view.geometry.clone();
        });
    for _ in 0..4 {
        harness.step();
    }
    let baseline = geometry.lock().unwrap().clone();
    assert!((baseline["a"].bottom() - baseline["b"].bottom()).abs() * 2.4 <= 0.5);
    assert!((baseline["last"].bottom() - baseline["a"].bottom()).abs() * 2.4 <= 0.5);
    let image = harness.render().unwrap();
    let region = image::imageops::crop_imm(&image, 30, 150, 600, 120).to_image();
    for frame in 0..24 {
        *clock.lock().unwrap() = 0.2 + frame as f64 / 120.;
        harness.step();
        let current = geometry.lock().unwrap();
        for id in ["a", "b", "heading", "last"] {
            assert_eq!(current[id], baseline[id], "{id} held frame {frame}");
        }
        let image = harness.render().unwrap();
        assert_eq!(
            image::imageops::crop_imm(&image, 30, 150, 600, 120).to_image(),
            region,
            "source region frame {frame}"
        );
    }
}

#[test]
fn scale_entrances_paint_the_stretched_frame() {
    for anim in ["pop", "zoom"] {
        let source = r##"---
scene: 1
name: scaled-frame
citizen: test
---
```mix
{root:{widget:"grid",columns:2,equal_height:true,children:["a","b"]},
a:{widget:"column",background:"#ff0000",stretch_last:true,paint_effect:"= $model.effect",children:["last"]},
last:{widget:"column",children:["short"]},short:{widget:"spacer",size:10},
b:{widget:"column",children:["tall"]},tall:{widget:"spacer",size:80}}
```
"##;
        let mut view = SceneView::new(source).unwrap();
        let ctx = egui::Context::default();
        ctx.set_pixels_per_point(2.4);
        for frame in 0..24 {
            let time = 0.2 + frame as f64 * 0.001;
            view.apply_model(json!({"effect":{"preset":anim,"time":time}}))
                .unwrap();
            let out = ctx.run_ui(egui::RawInput::default(), |ui| {
                view.show(ui);
            });
            let scale = motion::scene::paint_preset(anim)
                .unwrap()
                .sample(time)
                .get(motion::Property::Scale) as f32;
            let painted = out
                .shapes
                .iter()
                .find_map(|s| match &s.shape {
                    egui::Shape::Rect(r) if r.fill == Color32::RED => Some(r.rect),
                    _ => None,
                })
                .unwrap();
            if frame >= 4 {
                assert!(
                    (view.geometry["a"].bottom() - view.geometry["b"].bottom()).abs() * 2.4 <= 0.5
                );
                assert!(
                    (view.geometry["last"].bottom() - view.geometry["a"].bottom()).abs() * 2.4
                        <= 0.5
                );
                assert!(
                    (painted.height() - view.geometry["b"].height() * scale).abs() * 2.4 <= 0.5
                );
                assert!(!out.platform_output.requested_discard());
            }
            out.drop_without_applying_deltas();
        }
    }
}

fn timed_models(path: &std::path::Path, slide: &str) -> Vec<Value> {
    let mut deck = Deck::new();
    let reply = deck.call("deck.open", json!({"path":path})).reply.unwrap();
    assert_eq!(reply.rc, 0, "{reply:?}");
    let document: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    let steps = document["slides"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == slide)
        .unwrap()["steps"]
        .as_u64()
        .unwrap();
    let beats: Vec<_> = (0..=steps)
        .map(|i| {
            if i == 0 {
                0.
            } else if i == 1 {
                0.1
            } else {
                10. * i as f64
            }
        })
        .collect();
    deck.call("deck.play", json!({"slides":[slide],"beats":[beats]}));
    deck.call("viu.presentation", json!({"time":100}));
    (1..=720)
        .map(|f| {
            deck.call("viu.presentation", json!({"time":100.+f as f64/60.}))
                .model
                .unwrap()
        })
        .collect()
}

fn play(
    models: Vec<Value>,
    output: Option<&std::path::Path>,
) -> std::collections::BTreeMap<String, egui::Rect> {
    let mut view = SceneView::with_model(
        models[0]["presentation"]["scene"].as_str().unwrap(),
        models[0].clone(),
    )
    .unwrap();
    let captured = Arc::new(std::sync::Mutex::new(std::collections::BTreeMap::new()));
    let rects = captured.clone();
    // The live host pumps publications between egui passes too. A static
    // model for the discarded pass would conceal perpetual height rekeying.
    let mut publications = models.into_iter();
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(1600., 900.))
        .with_pixels_per_point(2.4)
        .with_context_setup(|ctx| {
            viu::fonts::install(ctx, 400, 600);
            viu::icons::install(ctx);
        })
        .wgpu()
        .build_ui(move |ui| {
            view.apply_model(publications.next().expect("timed publication per pass"))
                .unwrap();
            let mut canvas = ui.new_child(egui::UiBuilder::new().max_rect(
                egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1600., 900.)),
            ));
            view.show(&mut canvas);
            *rects.lock().unwrap() = view.geometry.clone();
        });
    for _ in 0..180 {
        harness.step();
    }
    // The host keeps publishing elapsed time after every entrance has settled.
    let out = harness.output();
    {
        assert!(
            !out.platform_output.requested_discard(),
            "{:?}",
            out.platform_output.request_discard_reasons
        );
        assert!(
            out.viewport_output
                .values()
                .all(|v| v.repaint_delay == Duration::MAX)
        );
    }
    if let Some(path) = output {
        harness.render().unwrap().save(path).unwrap();
    }
    captured.lock().unwrap().clone()
}

#[test]
fn timed_equal_height_entrances() {
    for anim in ["fade", "up", "down", "left", "right", "pop", "zoom", "wipe"] {
        for stretch_last in [false, true] {
            let mut nodes = json!({"root":{"widget":"column","children":["row0","row1"]}});
            let mut build = json!({});
            for row in 0..2 {
                let ids: Vec<_> = (0..3).map(|i| format!("card{row}{i}")).collect();
                nodes[format!("row{row}")] = json!({"widget":"grid","columns":3,"column_gap":14,"equal_height":true,"children":ids});
                for (i, id) in ids.iter().enumerate() {
                    let kids: Vec<_> = (0..i + 2).map(|j| format!("{id}-{j}")).collect();
                    nodes[id] = json!({"widget":"column","padding":12,"background":"#203040","stretch_last":stretch_last,"children":kids});
                    for kid in kids {
                        nodes[&kid] = json!({"widget":"column","children":[format!("{kid}-text")]});
                        nodes[format!("{kid}-text")] = json!({"widget":"text","text":"Chip"});
                    }
                    build[id] = json!({"anim":anim,"b":1,"delay":(row*3+i) as f64*0.3});
                }
            }
            let dir = tempfile::tempdir().unwrap();
            let path = dir.path().join("deck.json");
            std::fs::write(&path,json!({"schema":"deck.v1","slides":[{"id":"intro","steps":3,"scene":nodes,"build":build}]}).to_string()).unwrap();
            let g = play(timed_models(&path, "intro"), None);
            for row in 0..2 {
                for i in 1..3 {
                    assert!(
                        (g[&format!("card{row}0")].bottom() - g[&format!("card{row}{i}")].bottom())
                            .abs()
                            * 2.4
                            <= 0.5,
                        "{anim} stretch_last={stretch_last}: {g:?}"
                    );
                }
            }
        }
    }
}

#[test]
#[ignore = "explicit CBC evidence; DECK_LAYOUT_INPUT and DECK_LAYOUT_OUTPUT required"]
fn render_timed_intro() {
    let path = PathBuf::from(std::env::var_os("DECK_LAYOUT_INPUT").unwrap());
    let out = PathBuf::from(std::env::var_os("DECK_LAYOUT_OUTPUT").unwrap());
    std::fs::create_dir_all(&out).unwrap();
    let g = play(
        timed_models(&path, "intro"),
        Some(&out.join("intro-timed.png")),
    );
    std::fs::write(
        out.join("intro-timed.json"),
        serde_json::to_vec_pretty(
            &g.iter()
                .map(|(id, r)| {
                    (
                        id.clone(),
                        json!([
                            r.left() * 2.4,
                            r.top() * 2.4,
                            r.width() * 2.4,
                            r.height() * 2.4
                        ]),
                    )
                })
                .collect::<serde_json::Map<String, Value>>(),
        )
        .unwrap(),
    )
    .unwrap();
    println!("TIMED_INTRO {g:?}");
}

#[test]
#[ignore = "explicit CBC held-scene evidence; DECK_LAYOUT_INPUT required"]
fn held_source_cards() {
    let path = PathBuf::from(std::env::var_os("DECK_LAYOUT_INPUT").unwrap());
    for slide in ["s220", "s222", "s243"] {
        let beats = match slide {
            "s220" => [0.9, 12.060666666666668, 27.541333333333338, 42.5],
            "s222" => [
                0.9,
                12.180666666666667,
                29.221333333333334,
                50.06666666666667,
            ],
            _ => [
                0.9,
                14.940666666666667,
                30.781333333333333,
                47.13333333333333,
            ],
        };
        let mut deck = Deck::new();
        let opened = deck.call("deck.open", json!({"path":path})).reply.unwrap();
        assert_eq!(opened.rc, 0, "{opened:?}");
        deck.call("deck.play", json!({"slides":[slide],"beats":[beats]}));
        let model = deck
            .call("viu.presentation", json!({"time":100}))
            .model
            .unwrap();
        let mut view = SceneView::with_model(
            model["presentation"]["scene"].as_str().unwrap(),
            model.clone(),
        )
        .unwrap();
        let rects = Arc::new(std::sync::Mutex::new(view.geometry.clone()));
        let captured = rects.clone();
        let clock = Arc::new(std::sync::Mutex::new(100_f64));
        let time = clock.clone();
        let mut harness = egui_kittest::Harness::builder()
            .with_size(egui::vec2(1600., 900.))
            .with_pixels_per_point(2.)
            .with_context_setup(|ctx| {
                viu::fonts::install(ctx, 400, 600);
                viu::icons::install(ctx);
            })
            .wgpu()
            .build_ui(move |ui| {
                if let Some(model) = deck
                    .call("viu.presentation", json!({"time":*time.lock().unwrap()}))
                    .model
                {
                    view.apply_model(model).unwrap();
                }
                view.show(ui);
                *captured.lock().unwrap() = view.geometry.clone();
            });
        let warm = ((beats[2] + 4.) * 60.) as usize;
        for tick in 1..=warm {
            *clock.lock().unwrap() = 100. + tick as f64 / 60.;
            harness.step();
        }
        let baseline = rects.lock().unwrap().clone();
        let image = harness.render().unwrap();
        // Same flat source patch as the VFL investigation, before encoding.
        let patch = image::imageops::crop_imm(&image, 334, 1208, 212, 26).to_image();
        let frames = ((beats[3] - beats[2] - 4.5) * 60.) as usize;
        for frame in 0..frames {
            *clock.lock().unwrap() = 100. + (warm + frame + 1) as f64 / 60.;
            harness.step();
            let current = rects.lock().unwrap().clone();
            if current != baseline {
                let image = harness.render().unwrap();
                image
                    .save(std::env::temp_dir().join(format!("held-{slide}-{frame}.png")))
                    .unwrap();
                panic!(
                    "{slide} held frame {frame}: geometry changed: {current:?}; baseline: {baseline:?}"
                );
            }
            if frame % 60 == 0 {
                let image = harness.render().unwrap();
                assert_eq!(
                    image::imageops::crop_imm(&image, 334, 1208, 212, 26).to_image(),
                    patch,
                    "{slide} source patch frame {frame}"
                );
            }
        }
        println!(
            "HELD_SOURCE {slide}: {frames} held frames, stable rects each frame and source patch sampled every 60 frames"
        );
    }
}
