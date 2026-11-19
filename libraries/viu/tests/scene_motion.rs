//! Scene motion through the application headless painter, no window or GPU.
use serde_json::json;
use viu::view::SceneView;
#[test]
fn static_offset_and_container_opacity_survive_disabled_motion() {
    for reduced in [false, true] {
        let source = |offset, opacity| format!("---\nscene: 1\nname: motion\ncitizen: test\n---\n```mix\n{}\n```\n", json!({"root":{"widget":"column","opacity":opacity,"motion":false,"offset":[offset,0],"children":["b"]},"b":{"widget":"button","label":"Offset","on_click":"test.click"}}));
        let ctx = egui::Context::default();
        if reduced { ctx.global_style_mut(|s| s.animation_time=0.); }
        let mut view = SceneView::new(&source(20,0.5)).unwrap();
        let mut plain = SceneView::new(&source(0,1.)).unwrap();
        let reference_ctx = egui::Context::default();
        frame(&reference_ctx,&mut plain,0.).drop_without_applying_deltas();
        let mut out=frame(&ctx,&mut view,0.);
        assert!((view.geometry["b"].left()-plain.geometry["b"].left()-20.).abs()<0.01);
        assert!(out.shapes.iter().any(|s| matches!(&s.shape,egui::Shape::Text(t) if t.galley.job.text=="Offset" && t.fallback_color.a() <= 128)));
        out.textures_delta.clear();
        let target=view.geometry["b"].center();
        let mut commands=Vec::new();
        for pressed in [true,false] {
            let mut raw=egui::RawInput {time:Some(0.1), ..Default::default()};
            raw.events=vec![egui::Event::PointerMoved(target),egui::Event::PointerButton{pos:target,button:egui::PointerButton::Primary,pressed,modifiers:Default::default()}];
            ctx.run_ui(raw,|ui| commands.extend(view.show(ui))).drop_without_applying_deltas();
        }
        assert!(commands.iter().any(|e| e.command=="test.click"));
    }
}

#[test]
fn duplicate_native_list_labels_have_independent_exits() {
    let nodes=json!({"root":{"widget":"control","definition":"= $model.widget"}});
    let source=format!("---\nscene: 1\nname: motion\ncitizen: test\n---\n```mix\n{nodes}\n```\n");
    let model=|rows| json!({"datasets":{"rows":rows},"widget":{"id":"root","kind":"list","label":"List","initial":0,"properties":{"dataset":"rows","row_count":2,"columns":["Name","Size"]}}});
    let ctx=egui::Context::default();
    let mut view=SceneView::with_model(&source,model(json!([["duplicate","1"],["duplicate","2"]]))).unwrap();
    for t in [0.,1.] {frame(&ctx,&mut view,t).drop_without_applying_deltas();}
    view.apply_model(model(json!([["duplicate","1"]]))).unwrap();
    let mut out=frame(&ctx,&mut view,1.1);
    assert_eq!(out.shapes.iter().filter(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.job.text=="duplicate")).count(),2);
    out.textures_delta.clear();
    let mut out=frame(&ctx,&mut view,3.);
    assert_eq!(out.shapes.iter().filter(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.job.text=="duplicate")).count(),1);
    out.textures_delta.clear();
}
fn contains_text(shapes: &[egui::epaint::ClippedShape], needle: &str) -> bool {
    fn has(s: &egui::Shape, n: &str) -> bool {
        match s {
            egui::Shape::Text(t) => t.galley.job.text.contains(n),
            egui::Shape::Vec(ss) => ss.iter().any(|s| has(s, n)),
            _ => false,
        }
    }
    shapes.iter().any(|s| has(&s.shape, needle))
}
#[test]
fn scaled_input_and_per_node_reduced_motion_use_the_drawn_rectangle() {
    for enabled in [false, true] {
        let nodes = json!({"root":{"widget":"column","motion":enabled,"children":["button"]},"button":{"widget":"button","label":"Scaled","enter":"pop","on_click":"test.click"}});
        let source =
            format!("---\nscene: 1\nname: motion\ncitizen: test\n---\n```mix\n{nodes}\n```\n");
        let ctx = egui::Context::default();
        let mut view = SceneView::new(&source).unwrap();
        for t in [0., 0.03] {
            frame(&ctx, &mut view, t).drop_without_applying_deltas();
        }
        let painted = view.geometry["button"];
        let sample = motion::presets()
            .into_iter()
            .find(|p| p.name == "pop")
            .unwrap()
            .sample(0.03);
        let reference_ctx = egui::Context::default();
        let mut reference =
            SceneView::new(&source.replace("\"motion\":true", "\"motion\":false")).unwrap();
        frame(&reference_ctx, &mut reference, 0.03).drop_without_applying_deltas();
        let scale = if enabled {
            sample.get(motion::Property::Scale) as f32
        } else {
            1.
        };
        assert!((painted.width() - reference.geometry["button"].width() * scale).abs() < 0.1);
        // Preserve the toolkit's existing hit slop; test beyond that margin.
        for (target, expected) in [
            (painted.center(), true),
            (egui::pos2(painted.left() - 8., painted.center().y), false),
        ] {
            let mut commands = Vec::new();
            for pressed in [true, false] {
                ctx.run_ui(
                    egui::RawInput {
                        time: Some(0.03),
                        screen_rect: Some(egui::Rect::from_min_size(
                            egui::Pos2::ZERO,
                            egui::vec2(1000., 4000.),
                        )),
                        events: vec![
                            egui::Event::PointerMoved(target),
                            egui::Event::PointerButton {
                                pos: target,
                                button: egui::PointerButton::Primary,
                                pressed,
                                modifiers: Default::default(),
                            },
                        ],
                        ..Default::default()
                    },
                    |ui| commands.extend(view.show(ui)),
                )
                .drop_without_applying_deltas();
            }
            assert_eq!(
                commands.iter().any(|e| e.command == "test.click"),
                expected,
                "enabled={enabled}, target={target:?}, rect={painted:?}"
            );
        }
    }
}
#[test]
fn settled_application_scene_has_no_repaint_request() {
    let ctx = egui::Context::default();
    let mut view = SceneView::new(&source(true)).unwrap();
    for t in [0., 1., 2., 3., 4.] {
        frame(&ctx, &mut view, t).drop_without_applying_deltas();
    }
    assert!(!ctx.has_requested_repaint());
}
#[test]
fn list_insertions_and_removals_use_fade_and_reduced_motion_snaps() {
    for native in [false, true] {
        for reduced in [false, true] {
            let model = |updated: bool| {
                if native {
                    let rows = if updated {
                        json!([["remaining", "2"], ["inserted row", "3"]])
                    } else {
                        json!([["removed row", "1"], ["remaining", "2"]])
                    };
                    json!({"datasets":{"rows":rows},"widget":{"id":"root","kind":"list","label":"List","initial":0,"properties":{"dataset":"rows","row_count":2,"columns":["Name","Size"]}}})
                } else {
                    json!({"rows":if updated {json!([{"id":"b","cells":["remaining"]},{"id":"c","cells":["inserted row"]}])} else {json!([{"id":"a","cells":["removed row"]},{"id":"b","cells":["remaining"]}])}})
                }
            };
            let nodes = if native {
                json!({"root":{"widget":"control","definition":"= $model.widget"}})
            } else {
                json!({"root":{"widget":"list","rows":"= $model.rows","row":"item","row_height":24},"item":{"widget":"row","children":["label"]},"label":{"widget":"text","text":"{cells[0]}"}})
            };
            let source =
                format!("---\nscene: 1\nname: motion\ncitizen: test\n---\n```mix\n{nodes}\n```\n");
            let ctx = egui::Context::default();
            if reduced {
                ctx.global_style_mut(|s| s.animation_time = 0.);
            }
            let mut view = SceneView::with_model(&source, model(false)).unwrap();
            for t in [0., 1.] {
                frame(&ctx, &mut view, t).drop_without_applying_deltas();
            }
            view.apply_model(model(true)).unwrap();
            let mut changed = frame(&ctx, &mut view, 1.1);
            assert_eq!(
                contains_text(&changed.shapes, "removed row"),
                !reduced,
                "native={native}, reduced={reduced}"
            );
            assert!(contains_text(&changed.shapes, "inserted row"));
            changed.textures_delta.clear();
            let mut settled = frame(&ctx, &mut view, 3.);
            assert!(!contains_text(&settled.shapes, "removed row"));
            settled.textures_delta.clear();
        }
    }
}
#[test]
fn a_button_is_clicked_where_up_paints_it_and_exit_has_no_input() {
    let nodes = json!({"root":{"widget":"column","children":["button"]},"button":{"widget":"button","label":"Moving","on_click":"test.click","enter":"up","exit":"fade","hidden":"= $model.hidden"}});
    let source = format!("---\nscene: 1\nname: motion\ncitizen: test\n---\n```mix\n{nodes}\n```\n");
    let mut view = SceneView::with_model(&source, json!({"hidden":false})).unwrap();
    let ctx = egui::Context::default();
    frame(&ctx, &mut view, 0.).drop_without_applying_deltas();
    frame(&ctx, &mut view, 0.03).drop_without_applying_deltas();
    let target = view.geometry["button"].center();
    let click = |view: &mut SceneView, t: f64| {
        let mut commands = Vec::new();
        for pressed in [true, false] {
            let mut raw = egui::RawInput {
                time: Some(t),
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1000., 4000.),
                )),
                ..Default::default()
            };
            raw.events = vec![
                egui::Event::PointerMoved(target),
                egui::Event::PointerButton {
                    pos: target,
                    button: egui::PointerButton::Primary,
                    pressed,
                    modifiers: Default::default(),
                },
            ];
            ctx.run_ui(raw, |ui| commands.extend(view.show(ui)))
                .drop_without_applying_deltas();
        }
        commands
    };
    assert!(
        click(&mut view, 0.03)
            .iter()
            .any(|e| e.command == "test.click")
    );
    view.apply_model(json!({"hidden":true})).unwrap();
    frame(&ctx, &mut view, 0.04).drop_without_applying_deltas();
    assert!(click(&mut view, 0.05).is_empty());
}
fn source(animate: bool) -> String {
    let mut nodes = json!({"root":{"widget":"column","children":(0..100).map(|i|format!("n{i}")).collect::<Vec<_>>()}});
    for i in 0..100 {
        let mut node = json!({"widget":"text","text":format!("Motion {i}"),"size":13});
        if animate {
            node["enter"] = json!("up");
        }
        nodes[format!("n{i}")] = node;
    }
    format!("---\nscene: 1\nname: motion\ncitizen: test\n---\n```mix\n{nodes}\n```\n")
}
fn frame(ctx: &egui::Context, view: &mut SceneView, t: f64) -> egui::FullOutput {
    ctx.run_ui(
        egui::RawInput {
            time: Some(t),
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1000., 4000.),
            )),
            ..Default::default()
        },
        |ui| {
            view.show(ui);
        },
    )
}
#[test]
fn application_painter_samples_up_at_presentation_time() {
    let ctx = egui::Context::default();
    let mut view = SceneView::new(&source(true)).unwrap();
    frame(&ctx, &mut view, 0.).drop_without_applying_deltas();
    let mut moving = frame(&ctx, &mut view, 0.35);
    let at = moving
        .shapes
        .iter()
        .find_map(|s| {
            if let egui::Shape::Text(t) = &s.shape {
                if t.galley.job.text == "Motion 0" {
                    Some(t.pos.y)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap();
    moving.textures_delta.clear();
    let mut settled = frame(&ctx, &mut view, 1.);
    let end = settled
        .shapes
        .iter()
        .find_map(|s| {
            if let egui::Shape::Text(t) = &s.shape {
                if t.galley.job.text == "Motion 0" {
                    Some(t.pos.y)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .unwrap();
    settled.textures_delta.clear();
    let preset = motion::presets()
        .into_iter()
        .find(|p| p.name == "up")
        .unwrap();
    let expected = preset.sample(0.35).get(motion::Property::Ty) as f32;
    assert!(
        (at - end - expected).abs() < 0.001,
        "offset: {} vs {expected}",
        at - end
    );
}
#[test]
#[ignore = "release measurement; run explicitly with --release --ignored --nocapture"]
fn hundred_node_cost() {
    for animate in [false, true] {
        let ctx = egui::Context::default();
        let mut view = SceneView::new(&source(animate)).unwrap();
        frame(&ctx, &mut view, 0.).drop_without_applying_deltas();
        let mut us = Vec::new();
        for i in 1..1001 {
            let start = std::time::Instant::now();
            let mut out = frame(&ctx, &mut view, i as f64 / 2000.);
            let _ = ctx.tessellate(std::mem::take(&mut out.shapes), out.pixels_per_point);
            out.textures_delta.clear();
            us.push(start.elapsed().as_secs_f64() * 1e6);
        }
        us.sort_by(f64::total_cmp);
        println!(
            "100 nodes animated={animate}: median_us={:.3} p95_us={:.3} frames=1000 (headless layout, paint, tessellation; no GPU)",
            us[500], us[950]
        );
    }
}
