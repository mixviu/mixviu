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
fn wrapped_fill_label_retains_natural_glyph_advances() {
    let ctx = egui::Context::default();
    viu::fonts::install(&ctx, 400, 600);
    let mut view=SceneView::new("---\nscene: 1\nname: glyphs\ncitizen: test\n---\n```mix\n{root:{widget:\"text\",text:\"RAW NOTICES\",font:\"Liberation Sans\",size:20,weight:500,letter_spacing:1.6,max_width:100,wrap:true,fill:true,align:\"center\"}}\n```\n").unwrap();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        view.show(ui);
    });
    output.textures_delta.clear();
    let galleys: Vec<_> = output
        .shapes
        .iter()
        .filter_map(|s| {
            if let egui::Shape::Text(t) = &s.shape {
                Some(t.galley.clone())
            } else {
                None
            }
        })
        .collect();
    let galley = galleys
        .iter()
        .find(|g| g.job.text == "RAW NOTICES")
        .unwrap();
    assert!(galley.rows.len() >= 2);
    assert!(!galley.job.justify);
    assert!(galley.rows[0].glyphs.last().unwrap().max_x() - galley.rows[0].glyphs[0].pos.x < 80.);
    for row in &galley.rows {
        let word = row.text().trim().to_owned();
        let mut job = egui::text::LayoutJob::simple(
            word.clone(),
            FontId::new(
                20.,
                toolkit::fonts::optional::family("Liberation Sans", 500.),
            ),
            Color32::WHITE,
            f32::INFINITY,
        );
        job.sections[0].format.extra_letter_spacing = 1.6;
        let natural = ctx.fonts_mut(|f| f.layout_job(job));
        for (a, b) in row.glyphs.windows(2).zip(natural.rows[0].glyphs.windows(2)) {
            let error = ((a[1].pos.x - a[0].pos.x) - (b[1].pos.x - b[0].pos.x)).abs();
            assert!(error <= 1.01, "{word}: advance delta {error}");
        }
        println!("GLYPH_ADVANCE {word}: natural advances preserved");
    }
    output.textures_delta.clear();
}

#[test]
fn first_frame_between_and_wrap_have_visible_text() {
    for ports in [
        "justify:\"between\"",
        "wrap:true",
        "justify:\"between\",native_spacing:true",
    ] {
        let source = format!(
            "---\nscene: 1\nname: initial\ncitizen: test\n---\n```mix\n{{root:{{widget:\"row\",fill:true,{ports},children:[\"a\",\"b\"]}},a:{{widget:\"column\",children:[\"one\"]}},b:{{widget:\"column\",children:[\"two\"]}},one:{{widget:\"text\",text:\"First\"}},two:{{widget:\"text\",text:\"Second\"}}}}\n```\n"
        );
        let mut view = SceneView::new(&source).unwrap();
        let ctx = egui::Context::default();
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            view.show(ui);
        });
        out.textures_delta.clear();
        let text: Vec<_> = out
            .shapes
            .iter()
            .filter(|s| matches!(&s.shape, egui::Shape::Text(_)))
            .collect();
        assert!(text.len() >= 2, "{ports}");
        assert!(
            text.iter()
                .all(|s| s.clip_rect.width() > 10. && s.clip_rect.height() > 10.),
            "{ports}"
        );
        assert!(view.geometry["root"].height() > 0.);
    }
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
fn shrink_row_preserves_images_buttons_spans_and_grow() {
    let rects = geometry(
        r##"{root:{widget:"row",fill:true,gap:8,children:["photo","button","spans","grow"]},photo:{widget:"image",src:"brand:cyclops",w:80,h:80},button:{widget:"button",label:"Ready"},spans:{widget:"text",text:"ignored",size:24,shrink:1,wrap:true,spans:[{text:"Wide bold ",bold:true},{text:"mono identifier",mono:true}]},grow:{widget:"column",grow:1,children:["tail"]},tail:{widget:"text",text:"Tail"}}"##,
    );
    assert!(rects["photo"].width() >= 80., "{rects:?}");
    assert!(rects["button"].width() > 30., "{rects:?}");
    assert!(rects["spans"].width() > 100., "{rects:?}");
    assert!(rects["grow"].width() > 100., "{rects:?}");
    assert!(
        rects["button"].left() >= rects["photo"].right() + 7.,
        "{rects:?}"
    );
    assert!(
        rects["spans"].left() >= rects["button"].right() + 7.,
        "{rects:?}"
    );
}

#[test]
fn start_row_reserves_height_and_hidden_grid_cell_keeps_its_slot() {
    let rects = geometry(
        r##"{root:{widget:"column",children:["row","grid"]},row:{widget:"row",height:80,align:"start",children:["label"]},label:{widget:"text",text:"Top"},grid:{widget:"grid",columns:2,column_gap:20,children:["hidden","visible"]},hidden:{widget:"column",hidden:true,children:[]},visible:{widget:"text",text:"Right"}}"##,
    );
    assert!(rects["row"].height() >= 80., "{rects:?}");
    assert!(
        rects["visible"].left() > rects["grid"].left() + 400.,
        "{rects:?}"
    );
}

#[test]
fn bounded_columns_keep_center_and_end_placement() {
    let rects = geometry(
        r##"{root:{widget:"column",fill:true,align:"center",children:["center","row"]},center:{widget:"column",grow:1,min_width:200,max_width:200,children:["middle"]},middle:{widget:"text",text:"Middle"},row:{widget:"row",fill:true,align:"end",children:["right"]},right:{widget:"column",grow:1,min_width:200,max_width:200,children:["last"]},last:{widget:"text",text:"Right"}}"##,
    );
    assert!(
        (rects["root"].center().x - rects["center"].center().x).abs() < 1.,
        "{rects:?}"
    );
    assert!(
        (rects["row"].right() - rects["right"].right()).abs() < 1.,
        "{rects:?}"
    );
}

#[test]
fn columns_distribute_free_space_and_scroll_grid_has_first_frame_height() {
    for mode in ["end", "between", "around", "evenly"] {
        let source = format!(
            r##"{{root:{{widget:"column",height:400,justify:"{mode}",children:["a","b"]}},a:{{widget:"text",text:"One"}},b:{{widget:"text",text:"Two"}}}}"##
        );
        let r = geometry(&source);
        assert!(r["b"].top() > r["a"].top(), "{mode}: {r:?}");
        if mode == "end" {
            assert!((r["root"].bottom() - r["b"].bottom()).abs() < 1., "{r:?}");
        }
        if mode == "between" {
            assert!((r["root"].bottom() - r["b"].bottom()).abs() < 1., "{r:?}");
            assert!((r["root"].top() - r["a"].top()).abs() < 1.);
        }
    }
    let r = geometry(
        r##"{root:{widget:"grid",columns:1,children:["scroll"]},scroll:{widget:"column",fill:true,scroll:true,children:["line"]},line:{widget:"text",text:"Visible"}}"##,
    );
    assert!(r["scroll"].height() > 10., "{r:?}");
}

#[test]
fn all_hidden_flex_children_leave_a_finite_empty_row() {
    let r = geometry(
        r##"{root:{widget:"row",children:["hidden"]},hidden:{widget:"column",shrink:1,hidden:true,children:[]}}"##,
    );
    assert!(r["root"].is_finite());
}

#[test]
fn between_keeps_a_nested_date_at_the_right_edge() {
    let rects = geometry(
        r##"{root:{widget:"row",fill:true,justify:"between",children:["chips","date"]},chips:{widget:"row",children:["chip"]},chip:{widget:"text",text:"#210 #215 #214"},date:{widget:"column",padding_right:9,children:["label"]},label:{widget:"text",text:"21 NOV 2026",wrap:true}}"##,
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
    assert!(
        rects["left"].right() <= rects["right"].left() - 39.,
        "{rects:?}"
    );
}

#[test]
fn a_minimum_width_does_not_cap_natural_content() {
    let rects = geometry(
        r##"{root:{widget:"column",min_width:10,children:["label"]},label:{widget:"text",text:"A wider natural label",size:30}}"##,
    );
    assert!(
        rects["root"].width() >= rects["label"].width() && rects["root"].width() > 100.,
        "{rects:?}"
    );
}

#[test]
fn shrinking_compounds_wrap_inside_their_shared_row() {
    let rects = geometry(
        r##"{root:{widget:"row",fill:true,gap:8,children:["a","arrow","b"]},a:{widget:"column",shrink:1,padding_left:14,padding_right:14,children:["one"]},b:{widget:"column",shrink:1,padding_left:14,padding_right:14,children:["two"]},arrow:{widget:"text",text:"→",size:40},one:{widget:"text",text:"A long heading with several separate words",size:40,wrap:true},two:{widget:"text",text:"Another heading that must wrap within its column",size:40,wrap:true}}"##,
    );
    assert!(rects["one"].height() > 50., "{rects:?}");
    assert!(rects["two"].height() > 50., "{rects:?}");
    assert!(
        (rects["root"].right() - rects["b"].right()).abs() < 1.,
        "{rects:?}"
    );
    assert!(rects["a"].right() + 7. < rects["arrow"].left(), "{rects:?}");
}

#[test]
fn wrapped_compound_tags_move_as_whole_cells() {
    let rects = geometry(
        r##"{root:{widget:"row",fill:true,wrap:true,gap:8,children:["a","b","c"]},a:{widget:"column",min_width:600,children:["one"]},b:{widget:"column",min_width:600,children:["two"]},c:{widget:"column",min_width:600,children:["three"]},one:{widget:"text",text:"First tag"},two:{widget:"text",text:"Second tag"},three:{widget:"text",text:"Third tag"}}"##,
    );
    assert!(rects["b"].top() >= rects["a"].bottom() + 7., "{rects:?}");
    assert!(rects["c"].top() >= rects["b"].bottom() + 7., "{rects:?}");
    assert!(rects["c"].right() <= rects["root"].right(), "{rects:?}");
}

#[test]
fn flex_basis_and_stretch_keep_weighted_cards_in_one_row() {
    let large = geometry(
        r##"{root:{widget:"row",fill:true,children:["a","b"]},a:{widget:"column",grow:1,basis:1000000,children:[]},b:{widget:"column",grow:1,basis:1000000,children:[]}}"##,
    );
    assert!(
        large["a"].is_finite() && large["b"].right() <= large["root"].right() + 0.01,
        "{large:?}"
    );
    let rects = geometry(
        r##"{root:{widget:"row",fill:true,gap:40,align:"stretch",children:["a","b"]},a:{widget:"column",grow:1,basis:60,children:["one"]},b:{widget:"column",grow:2,basis:60,children:["three"]},one:{widget:"text",text:"short"},three:{widget:"text",text:"one\ntwo\nthree"}}"##,
    );
    let share = (rects["root"].width() - 40. - 120.) / 3.;
    assert!((rects["a"].width() - share - 60.).abs() < 1., "{rects:?}");
    assert!(
        (rects["a"].height() - rects["b"].height()).abs() < 1.,
        "{rects:?}"
    );
    assert!(
        (rects["root"].right() - rects["b"].right()).abs() < 1.,
        "{rects:?}"
    );
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
                        view.show(ui);
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
fn themed_plain_colours_and_bold_are_painted() {
    for dark in [false, true] {
        let ctx = egui::Context::default();
        viu::appearance::install_theme(&ctx, &toolkit::Theme::embedded()).unwrap();
        ctx.set_visuals(if dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        });
        let mut v=SceneView::new("---\nscene: 1\nname: colours\ncitizen: test\n---\n```mix\n{root:{widget:\"column\",children:[\"a\",\"b\",\"c\"]},a:{widget:\"text\",text:\"White\",color:\"#ffffff\"},b:{widget:\"text\",text:\"Bold\",bold:true},c:{widget:\"text\",text:\"Design\",color:\"design.text\"}}\n```\n").unwrap();
        let strong = ctx.global_style().visuals.strong_text_color();
        let design = viu::appearance::colour(&ctx, "text").unwrap();
        let mut o = ctx.run_ui(egui::RawInput::default(), |ui| {
            v.show(ui);
        });
        o.textures_delta.clear();
        for s in o.shapes {
            if let egui::Shape::Text(t) = s.shape {
                let wanted = match t.galley.job.text.as_str() {
                    "White" => Color32::WHITE,
                    "Bold" => strong,
                    "Design" => design,
                    _ => continue,
                };
                assert!(
                    t.galley
                        .job
                        .sections
                        .iter()
                        .all(|s| s.format.color == wanted)
                );
            }
        }
    }
}

#[test]
fn scroll_justify_stays_finite_and_max_width_keeps_parent_centre() {
    for mode in ["center", "between", "evenly"] {
        let r = geometry(&format!(
            r#"{{root:{{widget:"column",scroll:true,children:["j"]}},j:{{widget:"column",justify:"{mode}",children:["a","b"]}},a:{{widget:"text",text:"One"}},b:{{widget:"text",text:"Two"}}}}"#
        ));
        assert!(r.values().all(|r| r.is_finite()));
    }
    let r = geometry(
        r#"{root:{widget:"column",fill:true,align:"center",children:["a","b"]},a:{widget:"text",text:"A wrapped centred subtitle with enough words to wrap",max_width:140,wrap:true,align:"center"},b:{widget:"text",text:"Following"}}"#,
    );
    assert!((r["a"].center().x - r["root"].center().x).abs() < 1.);
    assert!((r["b"].center().x - r["root"].center().x).abs() < 1.);
}

#[test]
fn grid_clips_painted_text_and_native_between_skips_hidden() {
    let source = "---\nscene: 1\nname: grid\ncitizen: test\n---\n```mix\n{root:{widget:\"grid\",columns:2,children:[\"a\",\"b\"]},a:{widget:\"text\",text:\"AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\",size:40},b:{widget:\"text\",text:\"Right\"}}\n```\n";
    let mut v = SceneView::new(source).unwrap();
    let ctx = egui::Context::default();
    let mut o = ctx.run_ui(egui::RawInput::default(), |ui| {
        v.show(ui);
    });
    o.textures_delta.clear();
    let right = v.geometry["b"].left();
    assert!(o.shapes.iter().any(|s|matches!(&s.shape,egui::Shape::Text(t) if t.galley.job.text.starts_with("AAAA") && s.clip_rect.right()<right)));
    let r = geometry(
        r#"{root:{widget:"row",fill:true,justify:"between",native_spacing:true,children:["hidden","a","b"]},hidden:{widget:"text",text:"Hidden",hidden:true},a:{widget:"text",text:"Left"},b:{widget:"button",label:"Right"}}"#,
    );
    assert!((r["a"].left() - r["root"].left()).abs() < 1.);
    assert!((r["b"].right() - r["root"].right()).abs() < 1.);
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
fn measured_grid_sums_columns_and_min_width_is_only_a_floor() {
    let r = geometry(
        r#"{root:{widget:"row",justify:"between",fill:true,children:["g","tail"]},g:{widget:"grid",columns:2,column_gap:20,children:["a","b"]},a:{widget:"text",text:"A long first column",size:24},b:{widget:"text",text:"Second column",size:24},tail:{widget:"text",text:"Tail"}}"#,
    );
    assert!(r["g"].width() > r["a"].width() + r["b"].width() + 18.);
    let r = geometry(
        r#"{root:{widget:"row",children:["c","field","spacer","tail"]},c:{widget:"column",min_width:30,shrink:1,children:["text"]},text:{widget:"text",text:"Long intrinsic content"},field:{widget:"field",value:"",placeholder:"Input"},spacer:{widget:"spacer",size:20},tail:{widget:"column",basis:100,children:[]}}"#,
    );
    assert!(r["c"].width() > 40.);
    assert!(r["field"].width() > 20.);
    assert!(r["spacer"].width() >= 20.);
    assert!((r["tail"].width() - 100.).abs() < 1.);
}
