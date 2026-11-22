// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;

pub(super) fn geometry(source: &str) -> std::collections::BTreeMap<String, egui::Rect> {
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
        let mut found = std::collections::BTreeSet::new();
        for s in o.shapes {
            if let egui::Shape::Text(t) = s.shape {
                let wanted = match t.galley.job.text.as_str() {
                    "White" => Color32::WHITE,
                    "Bold" => strong,
                    "Design" => design,
                    _ => continue,
                };
                found.insert(t.galley.job.text.clone());
                assert!(
                    t.galley
                        .job
                        .sections
                        .iter()
                        .all(|s| s.format.color == wanted)
                );
            }
        }
        assert_eq!(found.len(), 3);
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

#[test]
fn grid_scroll_rows_shrink_with_viewport_and_reserve_prior_rows() {
    let nodes = json!({"root":{"widget":"grid","columns":1,"row_gap":12,"children":["fixed","scroll"]},"fixed":{"widget":"column","height":80,"children":[]},"scroll":{"widget":"column","scroll":true,"children":["long"]},"long":{"widget":"text","text":"line\n".repeat(200)}});
    let mut view = SceneView::new(&format!(
        "---\nscene: 1\nname: resize\ncitizen: test\n---\n```mix\n{nodes}\n```\n"
    ))
    .unwrap();
    let ctx = egui::Context::default();
    let mut previous = 0.;
    for height in [800., 400.] {
        for _ in 0..8 {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(800., height),
                    )),
                    ..Default::default()
                },
                |ui| {
                    view.show(ui);
                },
            )
            .drop_without_applying_deltas();
        }
        let scroll = view.geometry["scroll"];
        assert!(scroll.bottom() <= height + 1., "{scroll:?}");
        assert!(scroll.top() >= view.geometry["fixed"].bottom() + 11.);
        if previous > 0. {
            assert!(scroll.height() < previous - 200., "{scroll:?}");
        }
        previous = scroll.height();
    }
}

#[test]
fn default_rows_keep_main_cross_alignment_with_and_without_height() {
    for height in ["", ",height:80"] {
        let r = geometry(&format!(
            r#"{{root:{{widget:"row"{height},children:["label","button"]}},label:{{widget:"text",text:"Short",size:12}},button:{{widget:"button",label:"Button"}}}}"#
        ));
        assert!(
            (r["label"].center().y - r["button"].center().y).abs() < 1.,
            "{r:?}"
        );
        assert!(
            (r["label"].center().y - r["root"].center().y).abs() < 1.,
            "{r:?}"
        );
    }
}

#[test]
fn real_workshop_and_dopus_rows_match_main_centred_cross_positions() {
    let model = json!({"macro_confirm":true,"mesh_ping":{"id":"mesh-ping","kind":"button","label":"Ping","initial":"","properties":{"text":"Ping"}},"location_controls":{"left":{"id":"left-location","kind":"field","label":"","initial":"","properties":{"accessible_name":"Path","fill":true}},"right":{"id":"right-location","kind":"field","label":"","initial":"","properties":{"accessible_name":"Path","fill":true}}},"__viu_metrics":{"height":600,"control_row_height":60,"item_spacing_x":8}});
    for (source, ids) in [
        (
            include_str!("../../../../apps/workshop/scene.mix"),
            vec!["macro-confirmation", "mesh-header"],
        ),
        (
            include_str!("../../../../apps/dopus/scene.mix"),
            vec![
                "left-location-row",
                "right-location-row",
                "left-summary-row",
                "right-summary-row",
            ],
        ),
        (
            include_str!("../../../../apps/dopus/chooser.scene.mix"),
            vec!["location-row", "summary-row"],
        ),
    ] {
        let doc = scene::parse(source).unwrap();
        for id in ids {
            for height in [None, Some(80)] {
                let row = &doc.nodes[id];
                let mut ports = row.ports.clone();
                ports.insert("widget".into(), json!("row"));
                ports.shift_remove("hidden");
                if let Some(h) = height {
                    ports.insert("height".into(), json!(h));
                }
                let children = row.ports["children"].as_array().unwrap();
                let mut nodes = serde_json::Map::new();
                nodes.insert("root".into(), serde_json::to_value(ports).unwrap());
                for c in children {
                    let name = c.as_str().unwrap();
                    let raw = &doc.nodes[name];
                    let mut p = raw.ports.clone();
                    p.insert("widget".into(), json!(raw.widget));
                    nodes.insert(name.into(), serde_json::to_value(p).unwrap());
                }
                let mut geometries = vec![];
                for reference in [false, true] {
                    let mut nodes = nodes.clone();
                    if reference {
                        nodes["root"]["align"] = json!("center");
                    }
                    let source = format!(
                        "---\nscene: 1\nname: row-parity\ncitizen: test\n---\n```mix\n{}\n```\n",
                        Value::Object(nodes)
                    );
                    let mut v = SceneView::with_model(&source, model.clone()).unwrap();
                    let ctx = egui::Context::default();
                    for _ in 0..6 {
                        ctx.run_ui(egui::RawInput::default(), |ui| {
                            v.show(ui);
                        })
                        .drop_without_applying_deltas();
                    }
                    geometries.push(v.geometry);
                }
                for child in children {
                    let child = child.as_str().unwrap();
                    let a = geometries[0][child];
                    let b = geometries[1][child];
                    let delta = (a.center().y - geometries[0]["root"].top())
                        - (b.center().y - geometries[1]["root"].top());
                    assert!(delta.abs() < 0.01, "{id}/{child}: {delta}");
                }
                println!("ROW_MAIN_PARITY {id} height={height:?}: cross-position delta 0");
            }
        }
    }
}

#[test]
fn translucent_plain_node_and_theme_colours_are_not_premultiplied_twice() {
    let ctx = egui::Context::default();
    viu::appearance::install_theme(&ctx, &toolkit::Theme::embedded()).unwrap();
    let theme = Color32::from_rgba_unmultiplied(60, 120, 220, 128);
    let normal = Color32::from_rgba_unmultiplied(150, 200, 250, 96);
    ctx.data_mut(|d| {
        let key = egui::Id::unique("viu-design-colours");
        let mut palette = (**d
            .get_temp::<Arc<std::collections::BTreeMap<String, Color32>>>(key)
            .as_ref()
            .unwrap())
        .clone();
        palette.insert("translucent".into(), theme);
        d.insert_temp(key, Arc::new(palette));
    });
    ctx.global_style_mut(|s| s.visuals.override_text_color = Some(normal));
    let mut v=SceneView::new("---\nscene: 1\nname: alpha\ncitizen: test\n---\n```mix\n{root:{widget:\"column\",children:[\"a\",\"b\",\"c\"]},a:{widget:\"text\",text:\"Node\",color:\"#ff804080\"},b:{widget:\"text\",text:\"Theme\",color:\"design.translucent\"},c:{widget:\"text\",text:\"Normal\"}}\n```\n").unwrap();
    let mut o = ctx.run_ui(egui::RawInput::default(), |ui| {
        v.show(ui);
    });
    o.textures_delta.clear();
    let mut found = 0;
    for s in o.shapes {
        if let egui::Shape::Text(t) = s.shape {
            let expected = match t.galley.job.text.as_str() {
                "Node" => Color32::from_rgba_unmultiplied(255, 128, 64, 128),
                "Theme" => theme,
                "Normal" => normal,
                _ => continue,
            };
            found += 1;
            assert!(
                t.galley
                    .job
                    .sections
                    .iter()
                    .all(|s| s.format.color == expected)
            );
        }
    }
    assert_eq!(found, 3);
}

#[test]
fn stretched_justified_columns_settle_and_shrink_with_padding() {
    for justify in ["center", "end", "between"] {
        let source = format!(
            r#"---
scene: 1
name: stretch
citizen: test
---
```mix
{{root:{{widget:"row",fill:true,align:"stretch",children:["a","b"]}},a:{{widget:"column",grow:1,padding_top:12,padding_bottom:14,justify:"{justify}",children:["one","two"]}},one:{{widget:"text",text:"One"}},two:{{widget:"text",text:"Two"}},b:{{widget:"column",grow:1,height:"= $model.height",children:[]}}}}
```
"#
        );
        let mut v = SceneView::with_model(&source, json!({"height":220})).unwrap();
        let ctx = egui::Context::default();
        for target in [220, 100] {
            v.apply_model(json!({"height":target})).unwrap();
            for frame in 0..24 {
                let mut o = ctx.run_ui(egui::RawInput::default(), |ui| {
                    v.show(ui);
                });
                o.textures_delta.clear();
                if frame == 23 {
                    assert!(
                        !o.platform_output.requested_discard(),
                        "{justify}: {:?}",
                        o.platform_output.request_discard_reasons
                    );
                }
            }
            assert!(
                v.geometry["root"].height() <= target as f32 + 1.,
                "{justify}: {:?}",
                v.geometry
            );
        }
    }
}

#[test]
fn cross_axis_is_independent_on_plain_and_flex_rows() {
    for flex in [false, true] {
        let grow = if flex { ",grow:1" } else { "" };
        let mut tops = Vec::new();
        for cross in ["start", "center", "end"] {
            let g = geometry(&format!(
                r#"{{root:{{widget:"row",cross_align:"{cross}",children:["a","b"]}},a:{{widget:"column",min_width:100,height:80,children:[]}},b:{{widget:"column",height:20,children:[]{grow}}}}}"#
            ));
            assert!(g["a"].left() < g["b"].left(), "{flex} {cross}");
            let delta = g["b"].top() - g["a"].top();
            let expected = match cross {
                "start" => 0.,
                "center" => 30.,
                _ => 60.,
            };
            assert!((delta - expected).abs() < 1., "{flex} {cross} {delta}");
            tops.push(delta);
        }
        assert!(tops[0] < tops[1] && tops[1] < tops[2]);
    }
    let g = geometry(
        r#"{root:{widget:"row",children:["a","b"]},a:{widget:"column",min_width:100,height:80,children:[]},b:{widget:"column",height:20,grow:1,children:[]}}"#,
    );
    assert!(
        (g["a"].top() - g["b"].top()).abs() < 1.,
        "main default grow rows are top aligned"
    );
}

#[test]
fn explicit_cross_alignment_overrides_legacy_stretch() {
    let g = geometry(
        r#"{root:{widget:"row",align:"stretch",cross_align:"end",children:["a","b"]},a:{widget:"column",height:80,min_width:100,children:[]},b:{widget:"column",grow:1,children:["t"]},t:{widget:"text",text:"short"}}"#,
    );
    assert!((g["a"].bottom() - g["b"].bottom()).abs() < 1.);
    assert!(g["b"].height() < 30.);
}

#[test]
fn auto_height_justify_keeps_natural_packing() {
    let mut expected = None;
    for mode in ["start", "center", "end", "between", "around", "evenly"] {
        let g = geometry(&format!(
            r#"{{root:{{widget:"column",justify:"{mode}",children:["a","b"]}},a:{{widget:"text",text:"one"}},b:{{widget:"text",text:"two"}}}}"#
        ));
        let packing = (g["a"], g["b"], g["root"]);
        if let Some(expected) = expected {
            assert_eq!(packing, expected, "{mode}");
        } else {
            expected = Some(packing);
        }
    }
}

#[test]
fn explicit_cross_works_with_native_spacing_wrap_and_between() {
    for extra in ["wrap:true", "justify: \"between\""] {
        let source = format!(
            r#"{{root:{{widget:"row",cross_align:"end",native_spacing:true,{extra},children:["a","b"]}},a:{{widget:"column",height:80,min_width:100,children:[]}},b:{{widget:"column",height:20,min_width:100,children:[]}}}}"#
        );
        let g = geometry(&source);
        assert!((g["a"].bottom() - g["b"].bottom()).abs() < 1., "{extra}");
    }
}

#[test]
fn default_wrap_and_between_match_native_main_placement() {
    fn positions(source: &str) -> std::collections::BTreeMap<String, egui::Pos2> {
        let mut view = SceneView::new(&format!(
            "---\nscene: 1\nname: main-ink\ncitizen: test\n---\n```mix\n{source}\n```\n"
        ))
        .unwrap();
        let ctx = egui::Context::default();
        let mut found = std::collections::BTreeMap::new();
        for _ in 0..8 {
            let mut out = ctx.run_ui(
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
            out.textures_delta.clear();
            found.clear();
            for shape in &out.shapes {
                if let egui::Shape::Text(t) = &shape.shape {
                    found.insert(t.galley.job.text.clone(), t.pos);
                }
            }
        }
        found
    }
    for mode in ["wrap:true", "justify:\"between\""] {
        let scene = |native: bool| {
            format!(
                r#"{{root:{{widget:"row",{mode},native_spacing:{native},gap:8,children:["a","b","c"]}},a:{{widget:"text",text:"Large",size:40}},b:{{widget:"text",text:"small",size:12}},c:{{widget:"text",text:"third",size:20}}}}"#
            )
        };
        let a = positions(&scene(false));
        let b = positions(&scene(true));
        if mode.starts_with("justify") {
            let c = positions(&scene(false).replace(
                "native_spacing:false",
                "native_spacing:false,cross_align:\"end\"",
            ));
            for id in ["Large", "small", "third"] {
                assert!(
                    (c[id].x - b[id].x).abs() < 1.,
                    "explicit cross moved x: {c:?} {b:?}"
                );
            }
        }
        assert_eq!(a.len(), 3);
        assert_eq!(b.len(), 3);
        for id in ["Large", "small", "third"] {
            assert!((a[id].y - b[id].y).abs() < 1., "{mode} {id}: {a:?} {b:?}");
            if mode.starts_with("justify") {
                assert!((a[id].x - b[id].x).abs() < 1.);
            }
        }
    }
}

#[test]
fn authored_height_does_not_change_legacy_center_grow_cross_positions() {
    let scene = |height: &str| {
        format!(
            r#"{{root:{{widget:"row",align:"center",{height}children:["a","b"]}},a:{{widget:"column",height:80,min_width:100,children:[]}},b:{{widget:"column",height:20,grow:1,children:[]}}}}"#
        )
    };
    let a = geometry(&scene(""));
    let b = geometry(&scene("height:300,"));
    for id in ["a", "b"] {
        assert!((a[id].top() - b[id].top()).abs() < 1.);
    }
}
#[test]
fn explicit_cross_preserves_end_packing_and_plain_row_width() {
    for flex in [false, true] {
        for cross in ["start", "center", "end"] {
            let grow = if flex { ",grow:1,max_width:100" } else { "" };
            let scene = |cross: &str| {
                format!(
                    r#"{{root:{{widget:"row",align:"end",fill:true,{cross}children:["a","b"]}},a:{{widget:"column",min_width:100,height:80,children:[]}},b:{{widget:"column",min_width:100,height:20{grow},children:[]}}}}"#
                )
            };
            let a = geometry(&scene(""));
            let b = geometry(&scene(&format!("cross_align:\"{cross}\",")));
            for id in ["a", "b"] {
                assert!((a[id].left() - b[id].left()).abs() < 1., "{flex} {cross}");
            }
            assert!((b["root"].right() - b["b"].right()).abs() < 1.);
        }
    }
    let a = geometry(
        r#"{root:{widget:"column",fill:true,align:"center",children:["r"]},r:{widget:"row",cross_align:"center",background:"design.panel",children:["a","b"]},a:{widget:"text",text:"one"},b:{widget:"text",text:"two"}}"#,
    );
    assert!(a["r"].width() < 200., "{:?}", a["r"]);
    assert!((a["root"].center().x - a["r"].center().x).abs() < 1.);
}
#[test]
fn unknown_leaves_use_native_row_measurement_without_clipping() {
    for extra in ["", "wrap:true,", "justify:\"between\","] {
        let scene = |native: bool| {
            format!(
                r#"{{root:{{widget:"row",{extra}native_spacing:{native},children:["a","b"]}},a:{{widget:"toggle",value:true,label:"Full checkbox label"}},b:{{widget:"button",label:"Next"}}}}"#
            )
        };
        let a = geometry(&scene(false));
        let b = geometry(&scene(true));
        assert!(a["a"].width() > 100.);
        assert!((a["a"].width() - b["a"].width()).abs() < 1.);
        assert!(a["b"].left() >= a["a"].right());
    }
}
#[test]
fn non_text_column_distribution_falls_back_to_main_packing() {
    let scene = |mode: &str| {
        format!(
            r#"{{root:{{widget:"column",height:300,justify:"{mode}",children:["a","b"]}},a:{{widget:"button",label:"Button"}},b:{{widget:"image",src:"brand:cyclops",w:32,h:32}}}}"#
        )
    };
    let main = geometry(&scene("start"));
    for mode in ["between", "around", "evenly"] {
        let g = geometry(&scene(mode));
        for id in ["a", "b"] {
            assert_eq!(g[id], main[id], "{mode}");
        }
    }
}
