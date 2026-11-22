//! Inline deck text through the real scene painter, without a desktop.
use serde_json::json;
use viu::view::SceneView;

macro_rules! page {
    ($name:ident, $file:literal) => {
        #[test]
        fn $name() {
            let ctx = egui::Context::default();
            viu::fonts::install(&ctx, 400, 600);
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
fn font_port_face_table_covers_every_node_and_inline_combination() {
    const P: &str = "Inter-Regular";
    const J: &str = "JetBrainsMono-Regular";
    const S: &str = "LiberationSans";
    const M: &str = "NotoSansMono-Regular";
    // font, mono_font, node mono, inline mono, base face, inline face.
    let cases = [
        (false, false, false, false, P, P),
        (false, false, false, true, P, J),
        (false, false, true, false, J, J),
        (false, false, true, true, J, J),
        (false, true, false, false, P, P),
        (false, true, false, true, P, M),
        (false, true, true, false, M, M),
        (false, true, true, true, M, M),
        (true, false, false, false, S, S),
        (true, false, false, true, S, J),
        (true, false, true, false, S, S),
        (true, false, true, true, S, S),
        (true, true, false, false, S, S),
        (true, true, false, true, S, M),
        (true, true, true, false, M, M),
        (true, true, true, true, M, M),
    ];
    for (font, mono_font, node_mono, inline_mono, base_face, inline_face) in cases {
        let ctx = egui::Context::default();
        viu::fonts::install(&ctx, 400, 600);
        let mut plain = json!({"widget":"text","text":"plain","size":24,"mono":node_mono});
        if font {
            plain["font"] = json!("Liberation Sans");
        }
        if mono_font {
            plain["mono_font"] = json!("Noto Sans Mono");
        }
        let mut runs = plain.clone();
        runs["text"] = json!("base code");
        runs["spans"] = json!([{"text":"base "},{"text":"code","mono":inline_mono}]);
        let nodes = json!({"root":{"widget":"column","children":["plain","runs"]},"plain":plain,"runs":runs});
        let source =
            format!("---\nscene: 1\nname: font-matrix\ncitizen: test\n---\n```mix\n{nodes}\n```\n");
        let mut view = SceneView::new(&source).unwrap();
        let out = ctx.run_ui(egui::RawInput::default(), |ui| {
            view.show(ui);
        });
        let find = |text: &str| {
            out.shapes
                .iter()
                .find_map(|s| match &s.shape {
                    egui::Shape::Text(t) if t.galley.job.text == text => Some(&t.galley),
                    _ => None,
                })
                .expect("scene text painted")
        };
        let glyphs = |g: &egui::Galley| {
            g.rows
                .iter()
                .flat_map(|r| &r.glyphs)
                .map(|g| (g.chr, g.uv_rect, g.advance_width))
                .collect::<Vec<_>>()
        };
        let check = |actual: &egui::Galley, offset: usize, text: &str, face: &str| {
            let (family, reference) = match face {
                P => (
                    egui::FontFamily::Proportional,
                    egui::FontFamily::Proportional,
                ),
                J => (egui::FontFamily::Monospace, egui::FontFamily::Monospace),
                S => (
                    toolkit::fonts::optional::family("Liberation Sans", 400.),
                    egui::FontFamily::Name("LiberationSans-Regular.ttf".into()),
                ),
                M => (
                    toolkit::fonts::optional::family("Noto Sans Mono", 400.),
                    egui::FontFamily::Name("NotoSansMono-Regular.ttf".into()),
                ),
                _ => unreachable!(),
            };
            let section = actual
                .job
                .sections
                .iter()
                .find(|s| s.byte_range.contains(&offset.into()))
                .unwrap();
            assert_eq!(section.format.font_id.family, family);
            let defs = ctx.fonts(|f| f.definitions().clone());
            let primary = &defs.families[&family][0];
            let mut db = fontdb::Database::new();
            db.load_font_data(defs.font_data[primary].bytes().to_vec());
            assert_eq!(db.faces().next().unwrap().post_script_name, face);
            let expected = ctx.fonts_mut(|f| {
                f.layout_no_wrap(
                    text.into(),
                    egui::FontId::new(24., reference),
                    egui::Color32::WHITE,
                )
            });
            let actual_glyphs = glyphs(actual);
            let expected_glyphs = glyphs(&expected);
            assert!(!expected_glyphs.is_empty());
            assert_eq!(
                &actual_glyphs[offset..offset + text.chars().count()],
                expected_glyphs.as_slice(),
                "painted face for font={font}, mono_font={mono_font}, node={node_mono}, inline={inline_mono}"
            );
        };
        check(find("plain"), 0, "plain", base_face);
        check(find("base code"), 0, "base ", base_face);
        check(find("base code"), 5, "code", inline_face);
        println!(
            "FONT_FACE_TABLE font={font} mono_font={mono_font} node={node_mono} inline={inline_mono}: plain/base={base_face}; inline={inline_face}"
        );
        out.drop_without_applying_deltas();
    }
}

#[test]
fn mono_runs_preserve_node_faces_and_use_theme_mono_for_inline_runs() {
    for whole_node in [false, true] {
        for named_mono in [false, true] {
            let ctx = egui::Context::default();
            viu::fonts::install(&ctx, 400, 600);
            let mut node = json!({"widget":"text","text":"mono","font":"Liberation Sans","size":24,"mono":whole_node,"spans":[{"text":"mono","mono":!whole_node}]});
            if whole_node {
                node["font"] = json!("Noto Sans Mono");
            }
            if named_mono {
                node["mono_font"] = json!("Noto Sans Mono");
            }
            let nodes = json!({"root":node});
            let source =
                format!("---\nscene: 1\nname: mono\ncitizen: test\n---\n```mix\n{nodes}\n```\n");
            let mut view = SceneView::new(&source).unwrap();
            let out = ctx.run_ui(egui::RawInput::default(), |ui| {
                view.show(ui);
            });
            let actual = out
                .shapes
                .iter()
                .find_map(|s| match &s.shape {
                    egui::Shape::Text(t) if t.galley.job.text == "mono" => Some(&t.galley),
                    _ => None,
                })
                .expect("SceneView paints the mono text");
            let expected_family = if named_mono || whole_node {
                toolkit::fonts::optional::family("Noto Sans Mono", 400.)
            } else {
                egui::FontFamily::Monospace
            };
            assert_eq!(
                actual.job.sections[0].format.font_id.family,
                expected_family
            );
            let defs = ctx.fonts(|f| f.definitions().clone());
            let primary = &defs.families[&expected_family][0];
            let mut db = fontdb::Database::new();
            db.load_font_data(defs.font_data[primary].bytes().to_vec());
            let face = &db.faces().next().unwrap().post_script_name;
            assert_eq!(
                face,
                if named_mono || whole_node {
                    "NotoSansMono-Regular"
                } else {
                    "JetBrainsMono-Regular"
                }
            );
            let reference_family = if named_mono || whole_node {
                egui::FontFamily::Name("NotoSansMono-Regular.ttf".into())
            } else {
                egui::FontFamily::Monospace
            };
            let expected = ctx.fonts_mut(|f| {
                f.layout_no_wrap(
                    "mono".into(),
                    egui::FontId::new(24., reference_family),
                    egui::Color32::WHITE,
                )
            });
            let sans = ctx.fonts_mut(|f| {
                f.layout_no_wrap(
                    "mono".into(),
                    egui::FontId::new(
                        24.,
                        toolkit::fonts::optional::family("Liberation Sans", 400.),
                    ),
                    egui::Color32::WHITE,
                )
            });
            let glyphs = |galley: &egui::Galley| {
                galley
                    .rows
                    .iter()
                    .flat_map(|r| &r.glyphs)
                    .map(|g| (g.chr, g.uv_rect, g.advance_width))
                    .collect::<Vec<_>>()
            };
            assert!(!glyphs(actual).is_empty());
            assert_eq!(glyphs(actual), glyphs(&expected), "painted raster face");
            assert_ne!(
                glyphs(actual),
                glyphs(&sans),
                "mono must not use the sans face"
            );
            println!(
                "MONO_IDENTITY {} {}: {:?} -> {face}",
                if whole_node { "node" } else { "inline" },
                if named_mono {
                    "with mono_font"
                } else {
                    "without mono_font"
                },
                expected_family
            );
            out.drop_without_applying_deltas();
        }
    }
}
#[test]
fn unregistered_font_family_refuses_before_paint() {
    let source = "---\nscene: 1\nname: font-refusal\ncitizen: test\n---\n```mix\n{root:{widget:\"text\",text:\"hello\",font:\"missing-family\"}}\n```\n";
    let mut view = SceneView::new(source).unwrap();
    let ctx = egui::Context::default();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        assert!(view.bind_fonts(ui.ctx()).unwrap_err().contains("not bound"));
    });
    output.textures_delta.clear();
}

#[test]
fn mono_gallery_rail_chip_uses_named_mono_face() {
    let ctx = egui::Context::default();
    viu::fonts::install(&ctx, 400, 600);
    let mut view = SceneView::new(include_str!(
        "../../../apps/gallery/deck-pages/rail.scene.mix"
    ))
    .unwrap();
    let out = ctx.run_ui(egui::RawInput::default(), |ui| {
        view.show(ui);
    });
    let actual = out
        .shapes
        .iter()
        .find_map(|s| match &s.shape {
            egui::Shape::Text(t) if t.galley.job.text == "now" => Some(&t.galley),
            _ => None,
        })
        .expect("real gallery chip painted");
    let family = toolkit::fonts::optional::family("Noto Sans Mono", 400.);
    assert_eq!(actual.job.sections[0].format.font_id.family, family);
    let defs = ctx.fonts(|f| f.definitions().clone());
    let primary = &defs.families[&family][0];
    let mut db = fontdb::Database::new();
    db.load_font_data(defs.font_data[primary].bytes().to_vec());
    let face = &db.faces().next().unwrap().post_script_name;
    assert_eq!(face, "NotoSansMono-Regular");
    let expected = ctx.fonts_mut(|f| {
        f.layout_no_wrap(
            "now".into(),
            egui::FontId::new(
                24.,
                egui::FontFamily::Name("NotoSansMono-Regular.ttf".into()),
            ),
            egui::Color32::WHITE,
        )
    });
    let glyphs = |g: &egui::Galley| {
        g.rows
            .iter()
            .flat_map(|r| &r.glyphs)
            .map(|g| (g.chr, g.uv_rect, g.advance_width))
            .collect::<Vec<_>>()
    };
    assert!(!glyphs(actual).is_empty());
    assert_eq!(
        glyphs(actual),
        glyphs(&expected),
        "gallery chip raster face"
    );
    println!("MONO_IDENTITY gallery rail chip now: {family:?} -> {face}");
    out.drop_without_applying_deltas();
}

#[test]
fn bound_font_and_mono_font_refuse_unknown_or_absent_families_without_panicking() {
    let absent = tempfile::tempdir().unwrap();
    let ctx = egui::Context::default();
    viu::fonts::install_with_font_directory(&ctx, 400, 600, absent.path());
    for (port, name) in [
        ("font", "unknown"),
        ("font", "Liberation Sans"),
        ("mono_font", "unknown"),
        ("mono_font", "Noto Sans Mono"),
    ] {
        let nodes =
            json!({"root":{"widget":"text","text":"hello",port:format!("= $model.{port}")}});
        let source =
            format!("---\nscene: 1\nname: bound\ncitizen: test\n---\n```mix\n{nodes}\n```\n");
        let mut view = SceneView::with_model(&source, json!({port:"heading"})).unwrap();
        // Rebinding before context admission must also be safe to show.
        view.apply_model(json!({port:name})).unwrap();
        let out = ctx.run_ui(egui::RawInput::default(), |ui| {
            assert!(view.bind_fonts(ui.ctx()).unwrap_err().contains("not bound"));
            view.show(ui);
        });
        assert!(!out.shapes.is_empty());
        out.drop_without_applying_deltas();
    }
}

#[test]
fn bound_fonts_survive_file_removal_and_invalid_font_patches_preserve_the_scene() {
    let temp = tempfile::tempdir().unwrap();
    for file in toolkit::fonts::optional::FILES {
        std::fs::copy(
            toolkit::fonts::optional::directory().join(file),
            temp.path().join(file),
        )
        .unwrap();
    }
    let mut defs = egui::FontDefinitions::default();
    toolkit::fonts::optional::register(&mut defs, temp.path()).unwrap();
    std::fs::remove_dir_all(temp.path()).unwrap();
    let ctx = egui::Context::default();
    ctx.set_fonts(defs);
    let source = "---\nscene: 1\nname: bound\ncitizen: test\n---\n```mix\n{root:{widget:\"text\",text:\"hello\",font:\"= $model.font\"}}\n```\n";
    let model = json!({"font":"Liberation Sans"});
    let mut view = SceneView::with_model(source, model.clone()).unwrap();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        view.bind_fonts(ui.ctx()).unwrap();
        let error = view.apply_model(json!({"font":"absent"})).unwrap_err();
        assert!(error.contains("required font family not bound"));
        assert_eq!(view.model(), &model);
        view.show(ui);
    });
    output.textures_delta.clear();
}
#[test]
fn inline_runs_keep_layout_colours_history_and_highlight() {
    let ctx = egui::Context::default();
    viu::fonts::install(&ctx, 400, 600);
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
fn number_text_samples_a_motion_scalar_track_and_settles_without_repaint() {
    let nodes = json!({"root":{"widget":"text","text":"1,507","number":{"to":1507},"size":76}});
    let source = format!("---\nscene: 1\nname: count\ncitizen: test\n---\n```mix\n{nodes}\n```\n");
    let mut view = SceneView::new(&source).unwrap();
    let ctx = egui::Context::default();
    viu::fonts::install(&ctx, 400, 600);
    let track = motion::Track {
        property: motion::Property::Scalar,
        keyframes: vec![
            motion::Keyframe {
                offset: 0.,
                value: 0.,
                easing: motion::Curve::Bezier {
                    x1: 1. / 3.,
                    y1: 1.,
                    x2: 2. / 3.,
                    y2: 1.,
                },
            },
            motion::Keyframe {
                offset: 1.,
                value: 1507.,
                easing: motion::Curve::Linear,
            },
        ],
    };
    let timing = motion::Timing {
        duration: 1.4,
        delay: 0.2,
        fill: motion::Fill::Both,
        ..Default::default()
    };
    for t in [0., 0.2, 0.9, 1.6, 2., 3., 4., 5.] {
        let mut out = ctx.run_ui(
            egui::RawInput {
                time: Some(t),
                ..Default::default()
            },
            |ui| {
                view.show(ui);
            },
        );
        out.textures_delta.clear();
        let text = out
            .shapes
            .iter()
            .find_map(|s| match &s.shape {
                egui::Shape::Text(t) => Some(t.galley.job.text.as_str()),
                _ => None,
            })
            .unwrap();
        let actual = text.replace(',', "").parse::<f64>().unwrap();
        assert_eq!(
            actual,
            track
                .sample_at(&timing, t)
                .get(motion::Property::Scalar)
                .round()
        );
        if t == 5. {
            assert_eq!(text, "1,507");
            assert!(
                out.viewport_output
                    .values()
                    .all(|v| v.repaint_delay == std::time::Duration::MAX)
            );
        }
    }
    let source = source.replace("\"size\":76", "\"size\":76,\"motion\":false");
    let mut view = SceneView::new(&source).unwrap();
    let mut out = ctx.run_ui(
        egui::RawInput {
            time: Some(6.),
            ..Default::default()
        },
        |ui| {
            view.show(ui);
        },
    );
    out.textures_delta.clear();
    assert!(
        out.shapes
            .iter()
            .any(|s| matches!(&s.shape,egui::Shape::Text(t) if t.galley.job.text=="1,507"))
    );
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
            let rect = ui.ctx().content_rect();
            ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                ui.set_clip_rect(rect);
                view.show(ui);
            });
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
    for _ in 0..8 {
        frame(&mut view, 0.).drop_without_applying_deltas();
    }
    let backdrop_builds = ctx.data(|d| {
        d.get_temp::<usize>(egui::Id::new("viu-backdrop-builds"))
            .unwrap_or(0)
    });
    assert!(
        backdrop_builds > 0,
        "staged specimen must exercise the backdrop"
    );
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
    assert_eq!(
        ctx.data(|d| d
            .get_temp::<usize>(egui::Id::new("viu-backdrop-builds"))
            .unwrap_or(0)),
        backdrop_builds,
        "static backdrop rebuilt during frame loop"
    );
    println!("DECK_BACKDROP_BUILDS={backdrop_builds}; unchanged across 1000 frames and idle");
}
#[test]
fn before_after_cards_paint_the_named_border_colours() {
    let mut view = SceneView::new(include_str!(
        "../../../apps/gallery/deck-pages/cards.scene.mix"
    ))
    .unwrap();
    let ctx = egui::Context::default();
    viu::fonts::install(&ctx, 400, 600);
    let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
        view.show(ui);
    });
    out.textures_delta.clear();
    for colour in [
        egui::Color32::from_rgb(90, 43, 43),
        egui::Color32::from_rgb(36, 81, 58),
    ] {
        assert!(out.shapes.iter().any(|s| matches!(&s.shape,egui::Shape::Rect(r) if r.stroke.color==colour && r.stroke.width==1.)));
    }
}
#[test]
fn a_constant_number_track_never_arms_animation_frames() {
    let nodes = json!({"root":{"widget":"text","text":"0","number":{"to":0,"ms":60000}}});
    let source = format!("---\nscene: 1\nname: count\ncitizen: test\n---\n```mix\n{nodes}\n```\n");
    let mut view = SceneView::new(&source).unwrap();
    let ctx = egui::Context::default();
    viu::fonts::install(&ctx, 400, 600);
    for t in [0., 0.01, 0.02] {
        let mut out = ctx.run_ui(
            egui::RawInput {
                time: Some(t),
                ..Default::default()
            },
            |ui| {
                view.show(ui);
            },
        );
        out.textures_delta.clear();
        if t == 0.02 {
            assert!(
                out.viewport_output
                    .values()
                    .all(|v| v.repaint_delay == std::time::Duration::MAX)
            );
        }
    }
}

#[test]
fn fixed_width_frame_fills_its_entire_interior() {
    let nodes = json!({"root":{"widget":"column","min_width":260,"height":100,"padding":22,"background":"#121821","outline":"#3cc8d8","outline_width":2,"children":["label"]},"label":{"widget":"text","text":"short"}});
    let source = format!("---\nscene: 1\nname: fill\ncitizen: test\n---\n```mix\n{nodes}\n```\n");
    let mut view = SceneView::new(&source).unwrap();
    let rect = std::rc::Rc::new(std::cell::Cell::new(egui::Rect::NOTHING));
    let out = rect.clone();
    let mut harness = egui_kittest::Harness::builder()
        .with_size(egui::vec2(400., 200.))
        .wgpu()
        .build_ui(move |ui| {
            view.show(ui);
            out.set(view.geometry["root"]);
        });
    harness.run();
    let image = harness.render().unwrap();
    let r = rect.get();
    for x in (r.left() as u32 + 4)..(r.right() as u32 - 4) {
        assert_eq!(
            image.get_pixel(x, r.bottom() as u32 - 10).0,
            [18, 24, 33, 255],
            "interior x={x}, rect={r:?}"
        );
    }
}

#[test]
fn centred_labels_and_spans_use_the_heading_face_when_bold() {
    let nodes = json!({"root":{"widget":"column","min_width":400,"children":["title","runs"]},"title":{"widget":"text","text":"Title","bold":true,"size":56,"align":"center","fill":true},"runs":{"widget":"text","text":"runs","spans":[{"text":"runs"}],"align":"center","fill":true}});
    let source =
        format!("---\nscene: 1\nname: typography\ncitizen: test\n---\n```mix\n{nodes}\n```\n");
    let ctx = egui::Context::default();
    viu::fonts::install(&ctx, 400, 600);
    let mut view = SceneView::new(&source).unwrap();
    for _ in 0..2 {
        let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
            view.show(ui);
        });
        out.textures_delta.clear();
        for shape in &out.shapes {
            if let egui::Shape::Text(t) = &shape.shape {
                assert_eq!(t.galley.job.halign, egui::Align::Center);
                if t.galley.job.text == "Title" {
                    assert_eq!(
                        t.galley.job.sections[0].format.font_id.family,
                        egui::FontFamily::Name("heading".into())
                    );
                    assert_eq!(t.galley.job.sections[0].format.font_id.size, 56.);
                }
            }
        }
    }
}

#[test]
fn unequal_growing_boxes_and_arrow_share_the_row_centre() {
    let nodes = json!({"root":{"widget":"row","align":"center","min_width":600,"children":["a","arrow","b"]},"a":{"widget":"column","grow":1,"height":60,"children":[]},"arrow":{"widget":"text","text":"→","min_width":40},"b":{"widget":"column","grow":1,"height":140,"children":[]}});
    let source = format!("---\nscene: 1\nname: row\ncitizen: test\n---\n```mix\n{nodes}\n```\n");
    let ctx = egui::Context::default();
    viu::fonts::install(&ctx, 400, 600);
    let mut view = SceneView::new(&source).unwrap();
    for _ in 0..8 {
        ctx.run_ui(egui::RawInput::default(), |ui| {
            view.show(ui);
        })
        .drop_without_applying_deltas();
    }
    let centre = view.geometry["b"].center().y;
    for id in ["a", "arrow"] {
        assert!(
            (view.geometry[id].center().y - centre).abs() < 1.,
            "{id}: {:?}",
            view.geometry
        );
    }
}
