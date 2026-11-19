// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};
use viu::{bundle::Bundle, runtime::Runtime, view::SceneView};

fn draw(view: &mut SceneView, ctx: &egui::Context, width: f32, height: f32) {
    draw_capture(view, ctx, width, height, true);
}
fn draw_capture(view: &mut SceneView, ctx: &egui::Context, width: f32, height: f32, capture: bool) {
    if capture {
        view.request_widget_rectangles();
    }
    ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(width, height),
            )),
            ..Default::default()
        },
        |ui| {
            view.show(ui);
        },
    )
    .drop_without_applying_deltas();
}

#[test]
fn two_app_scenes_report_scene_ids_and_resize_geometry() {
    for app in ["gallery", "prefs"] {
        let bundle = Bundle::load(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../apps")
                .join(app),
            &[],
        )
        .unwrap();
        let (tx, rx) = mpsc::channel();
        let mut runtime = Runtime::new(
            8,
            Arc::new(move || {
                let _ = tx.send(());
            }),
            None,
        );
        runtime.load(bundle.program).unwrap();
        rx.recv_timeout(Duration::from_secs(15)).unwrap();
        let done = runtime.try_completion().unwrap();
        assert!(done.error.is_none(), "{:?}", done.error);
        let mut view = viu::gold_frame::mount(&bundle.scene, done.model.unwrap()).unwrap();
        let ctx = egui::Context::default();
        viu::icons::install(&ctx);
        assert_eq!(view.widget_rectangles(&json!({})).unwrap(), json!([]));
        let mut widths = Vec::new();
        for width in [1040., 1400.] {
            for _ in 0..3 {
                draw(&mut view, &ctx, width, 680.);
            }
            let rows = view.widget_rectangles(&json!({})).unwrap();
            let rows = rows.as_array().unwrap();
            assert!(!rows.is_empty(), "{app}");
            for row in rows {
                let id = row["id"].as_str().unwrap();
                let rect = view.geometry[id];
                assert_eq!(
                    row["rect"],
                    json!({"x":rect.left(),"y":rect.top(),"w":rect.width(),"h":rect.height()}),
                    "{app}/{id}"
                );
            }
            widths.push(view.geometry["gold-header"].width());
            let filtered = view
                .widget_rectangles(&json!({"ids":["gold-header"]}))
                .unwrap();
            assert_eq!(filtered.as_array().unwrap().len(), 1);
        }
        assert!(widths[1] > widths[0], "{app}: {widths:?}");
    }
}

#[test]
fn clipping_hidden_nodes_and_filter_validation() {
    let source = format!(
        "---\nscene: 1\nname: test\ncitizen: test\n---\n```mix\n{}\n```\n",
        json!({
            "root":{"widget":"column","children":["on","space","off","hidden"],"scroll":true},
            "on":{"widget":"button","label":"Initially visible"},
            "space":{"widget":"spacer","size":800},
            "off":{"widget":"button","label":"Off screen"},
            "hidden":{"widget":"button","label":"Hidden","hidden":true}
        })
    );
    let mut view = SceneView::new(&source).unwrap();
    let ctx = egui::Context::default();
    for _ in 0..3 {
        draw(&mut view, &ctx, 400., 200.);
    }
    assert_eq!(
        view.widget_rectangles(&json!({"ids":["off"]}))
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        view.widget_rectangles(&json!({"ids":["off"],"visible_only":true}))
            .unwrap(),
        json!([])
    );
    assert_eq!(
        view.widget_rectangles(&json!({"ids":["hidden"]})).unwrap(),
        json!([])
    );
    for args in [
        json!([]),
        json!({"ids":false}),
        json!({"kinds":[2]}),
        json!({"visible_only":1}),
    ] {
        assert!(view.widget_rectangles(&args).is_err());
    }
    assert_eq!(
        view.widget_rectangles(&json!({"ids":["on"],"visible_only":true}))
            .unwrap()
            .as_array()
            .unwrap()
            .len(),
        1
    );
    ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400., 200.),
            )),
            events: vec![
                egui::Event::PointerMoved(egui::pos2(30., 40.)),
                egui::Event::MouseWheel {
                    source: egui::MouseWheelSource::Unknown,
                    phase: egui::TouchPhase::Move,
                    unit: egui::MouseWheelUnit::Point,
                    delta: egui::vec2(0., -900.),
                    modifiers: Default::default(),
                },
            ],
            ..Default::default()
        },
        |ui| {
            view.show(ui);
        },
    )
    .drop_without_applying_deltas();
    for _ in 0..30 {
        draw(&mut view, &ctx, 400., 200.);
    }
    assert_eq!(
        view.widget_rectangles(&json!({"ids":["on"],"visible_only":true}))
            .unwrap(),
        json!([])
    );
}

#[test]
fn rectangle_frame_cost() {
    let mut nodes = serde_json::Map::new();
    let ids: Vec<_> = (0..100).map(|i| format!("button-{i}")).collect();
    nodes.insert("root".into(), json!({"widget":"column","children":ids}));
    for id in ids {
        nodes.insert(id, json!({"widget":"button","label":"Bench"}));
    }
    let source = format!(
        "---\nscene: 1\nname: bench\ncitizen: bench\n---\n```mix\n{}\n```\n",
        json!(nodes)
    );
    let mut view = SceneView::new(&source).unwrap();
    let ctx = egui::Context::default();
    for _ in 0..20 {
        draw(&mut view, &ctx, 800., 600.);
    }
    for queried in [false, true] {
        let start = Instant::now();
        for _ in 0..500 {
            draw_capture(&mut view, &ctx, 800., 600., queried);
            if queried {
                std::hint::black_box(view.widget_rectangles(&json!({})).unwrap());
            }
        }
        println!(
            "widget_rectangles 100 buttons pending={queried} us/frame={:.3}",
            start.elapsed().as_secs_f64() * 1e6 / 500.
        );
    }
}

#[test]
fn control_identity_comes_from_the_scene_and_enabled_from_the_control() {
    let source = format!(
        "---\nscene: 1\nname: test\ncitizen: test\n---\n```mix\n{}\n```\n",
        json!({
            "root":{"widget":"column","children":["scene-button"]},
            "scene-button":{"widget":"control","definition":{"id":"internal-button","kind":"button","label":"Save","initial":"","properties":{"enabled":false}}}
        })
    );
    assert!(
        SceneView::new(&source).is_err(),
        "control ids must match their scene node"
    );
    let source = source.replace("internal-button", "scene-button");
    let mut view = SceneView::new(&source).unwrap();
    let ctx = egui::Context::default();
    draw(&mut view, &ctx, 400., 200.);
    let rows = view
        .widget_rectangles(&json!({"kinds":["button"]}))
        .unwrap();
    assert_eq!(rows.as_array().unwrap().len(), 1);
    assert_eq!(rows[0]["id"], "scene-button");
    assert_eq!(rows[0]["enabled"], false);
    assert_eq!(
        view.widget_rectangles(&json!({"ids":["internal-button"]}))
            .unwrap(),
        json!([])
    );
}

#[test]
fn focus_readback_uses_the_scene_id() {
    let source = format!(
        "---\nscene: 1\nname: test\ncitizen: test\n---\n```mix\n{}\n```\n",
        json!({
            "root":{"widget":"column","children":["entry","other"]},
            "entry":{"widget":"control","definition":{"id":"entry","kind":"field","label":"Entry","initial":""}},
            "other":{"widget":"control","definition":{"id":"other","kind":"field","label":"Other","initial":""}}
        })
    );
    let mut view = SceneView::new(&source).unwrap();
    let ctx = egui::Context::default();
    draw(&mut view, &ctx, 400., 200.);
    for target in ["entry", "other", "entry"] {
        view.request_focus(target).unwrap();
        draw(&mut view, &ctx, 400., 200.);
        let rows = view.widget_rectangles(&json!({})).unwrap();
        let rows = rows.as_array().unwrap();
        assert_eq!(rows.iter().filter(|r| r["id"] == target).count(), 1);
        for row in rows {
            assert_eq!(
                row["focused"],
                row["id"] == target,
                "focus on {target}: {row}"
            );
        }
    }
}

#[test]
fn app_zoom_is_converted_to_surface_logical_coordinates() {
    let source = format!(
        "---\nscene: 1\nname: test\ncitizen: test\n---\n```mix\n{}\n```\n",
        json!({
            "root":{"widget":"column","children":["button"]},
            "button":{"widget":"button","label":"Zoom"}
        })
    );
    let mut view = SceneView::new(&source).unwrap();
    let ctx = egui::Context::default();
    ctx.set_zoom_factor(1.5);
    for _ in 0..3 {
        draw(&mut view, &ctx, 600., 300.);
    }
    let row = view.widget_rectangles(&json!({"ids":["button"]})).unwrap();
    let rect = view.geometry["button"];
    assert_eq!(row[0]["rect"]["x"], json!(rect.left() * 1.5));
    assert_eq!(row[0]["rect"]["w"], json!(rect.width() * 1.5));
}

#[cfg(feature = "test-support")]
#[test]
fn headless_bus_sweep() {
    use viu::app_host::test_support::Harness;
    let broker = test_broker::Broker::start();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let client = Arc::new(
        rt.block_on(bus::SupervisedClient::connect_options("rect-sweep", &broker.url).connect())
            .unwrap(),
    );
    for app in ["gallery", "prefs", "term", "dopus"] {
        let mut bundle = viu::launch::load(
            &std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../apps")
                .join(app),
            &[
                "--headless".into(),
                "--noded-url".into(),
                broker.url.clone(),
            ],
        )
        .unwrap();
        bundle.manifest.bus_url = Some(broker.url.clone());
        bundle.launch = json!({"headless":true});
        let mut host = Harness::new(bundle);
        let c = client.clone();
        let task = rt.spawn(async move {
            let deadline = Instant::now() + Duration::from_secs(20);
            loop {
                if let Ok((0, body, _)) = c
                    .call_with_headers_raw_within(
                        app,
                        &format!("{app}.widgets"),
                        &Default::default(),
                        "{}",
                        Duration::from_secs(2),
                    )
                    .await
                {
                    return serde_json::from_str::<serde_json::Value>(&body).unwrap();
                }
                assert!(Instant::now() < deadline, "{app} unavailable");
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        });
        let deadline = Instant::now() + Duration::from_secs(25);
        while !task.is_finished() {
            host.pump();
            assert!(Instant::now() < deadline, "{app} Bus timeout");
            std::thread::sleep(Duration::from_millis(2));
        }
        assert_eq!(
            rt.block_on(task).unwrap(),
            json!([]),
            "{app}: no layout in headless mode"
        );
        println!("{app}.widgets: [] (headless, no laid-out frame)");
    }
}

#[cfg(feature = "test-support")]
fn bus_widget_query(
    catalogue: Option<(u8, serde_json::Value)>,
    args: serde_json::Value,
) -> (u8, serde_json::Value) {
    use viu::app_host::test_support::Harness;
    let broker = test_broker::Broker::start();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();
    let comp = catalogue.as_ref().map(|_| {
        viu::native_bus::Endpoint::start(
            "comp".into(),
            broker.url.clone(),
            vec![bus::CommandDescriptor::new(
                "comp.windows.list",
                &[],
                "Test catalogue",
                true,
            )],
            "comp.quit".into(),
            Arc::new(|| {}),
        )
    });
    let client = Arc::new(
        rt.block_on(
            bus::SupervisedClient::connect_options("rect-populated", &broker.url).connect(),
        )
        .unwrap(),
    );
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/gallery");
    let bundle = viu::launch::load(
        &root,
        &[
            "--headless".into(),
            "--noded-url".into(),
            broker.url.clone(),
        ],
    )
    .unwrap();
    let mut host = Harness::new(bundle);
    let deadline = Instant::now() + Duration::from_secs(20);
    while !host.ready() || comp.as_ref().is_some_and(|c| c.client().is_none()) {
        host.pump();
        assert!(
            Instant::now() < deadline,
            "host readiness: {}",
            host.error()
        );
        std::thread::sleep(Duration::from_millis(2));
    }
    host.windowed();
    for _ in 0..4 {
        host.draw();
    }
    let task = rt.spawn(async move {
        let (rc, body, _) = client
            .call_with_headers_raw_within(
                "gallery",
                "gallery.widgets",
                &Default::default(),
                &args.to_string(),
                Duration::from_secs(10),
            )
            .await
            .unwrap();
        (
            rc,
            serde_json::from_str::<serde_json::Value>(&body).unwrap(),
        )
    });
    while !task.is_finished() {
        host.draw();
        host.pump();
        if let Some(comp) = &comp {
            while let Ok(request) = comp.requests.try_recv() {
                assert_eq!(request.command, "comp.windows.list");
                let _ = request.reply.send(catalogue.clone().unwrap());
            }
        }
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(2));
    }
    rt.block_on(task).unwrap()
}

#[cfg(feature = "test-support")]
#[test]
fn populated_bus_reply_has_a_compositor_window_fence() {
    let (rc, body) = bus_widget_query(
        Some((
            0,
            json!({"windows":[{"id":7,"generation":2,"pid":std::process::id()}]}),
        )),
        json!({"ids":["gold-header"],"visible_only":true}),
    );
    assert_eq!(rc, 0, "{body}");
    assert_eq!(body.as_array().unwrap().len(), 1);
    assert_eq!(body[0]["id"], "gold-header");
    assert_eq!(body[0]["window"], json!({"id":7,"generation":2}));
    assert!(body[0]["rect"]["w"].as_f64().unwrap() > 0.);
}

#[cfg(feature = "test-support")]
#[test]
fn bus_refuses_missing_ambiguous_and_refused_compositor_and_bad_arguments() {
    let pid = std::process::id();
    for catalogue in [
        None,
        Some((10, json!({"error_code":"REFUSED"}))),
        Some((
            0,
            json!({"windows":[{"id":1,"generation":1,"pid":pid},{"id":2,"generation":1,"pid":pid}]}),
        )),
    ] {
        let (rc, body) = bus_widget_query(catalogue, json!({"ids":["gold-header"]}));
        assert_eq!(rc, 10, "{body}");
        assert_eq!(body["error_code"], "WINDOW_UNAVAILABLE");
    }
    for args in [json!({"id":"typo"}), json!({"ids":false}), json!([])] {
        let (rc, body) = bus_widget_query(None, args);
        assert_eq!(rc, 10, "{body}");
        assert_eq!(body["error_code"], "ARGUMENT");
    }
}

fn scene(nodes: serde_json::Value) -> String {
    format!("---\nscene: 1\nname: test\ncitizen: test\n---\n```mix\n{nodes}\n```\n")
}

#[test]
fn frame_app_id_collisions_are_refused_at_mount() {
    let app = scene(
        json!({"root":{"widget":"column","children":["shared"]},"shared":{"widget":"text","text":"App"}}),
    );
    let frame = scene(
        json!({"root":{"widget":"column","children":["gold-content","shared"]},"gold-content":{"widget":"column","children":[]},"shared":{"widget":"text","text":"Frame"}}),
    );
    assert!(
        SceneView::with_frame(&app, json!({}), &frame)
            .err()
            .unwrap()
            .contains("identities overlap")
    );
}

#[test]
fn repeated_template_instances_are_laid_out_but_omitted_from_reply() {
    let source = scene(json!({
        "root":{"widget":"column","children":["list"]},
        "list":{"widget":"list","row":"template","row_height":24,"rows":[{"id":"a","cells":["A"]},{"id":"b","cells":["B"]}]},
        "template":{"widget":"row","children":["cell"]},
        "cell":{"widget":"text","text":"{cells[0]}"}
    }));
    let mut view = SceneView::new(&source).unwrap();
    let ctx = egui::Context::default();
    draw(&mut view, &ctx, 400., 200.);
    assert!(view.geometry.contains_key("a#cell"));
    assert!(view.geometry.contains_key("b#cell"));
    let rows = view.widget_rectangles(&json!({})).unwrap();
    assert!(
        rows.as_array()
            .unwrap()
            .iter()
            .all(|r| r["id"] == "root" || r["id"] == "list")
    );
}

#[test]
fn capture_latch_is_consumed_and_metadata_refreshes_each_query() {
    let source = scene(
        json!({"root":{"widget":"column","children":["label"]},"label":{"widget":"text","text":"= $model.caption"}}),
    );
    let mut view = SceneView::with_model(&source, json!({"caption":"Before"})).unwrap();
    let ctx = egui::Context::default();
    draw_capture(&mut view, &ctx, 400., 200., false);
    assert_eq!(view.widget_rectangles(&json!({})).unwrap(), json!([]));
    draw(&mut view, &ctx, 400., 200.);
    let before = view.widget_rectangles(&json!({"ids":["label"]})).unwrap();
    assert_eq!(before[0]["label"], "Before");
    view.apply_model(json!({"caption":"After"})).unwrap();
    draw_capture(&mut view, &ctx, 600., 200., false);
    assert_eq!(
        view.widget_rectangles(&json!({"ids":["label"]})).unwrap(),
        before
    );
    draw(&mut view, &ctx, 600., 200.);
    assert_eq!(
        view.widget_rectangles(&json!({"ids":["label"]})).unwrap()[0]["label"],
        "After"
    );
}

#[test]
fn authored_padding_has_a_hand_computed_surface_origin() {
    let source = scene(
        json!({"root":{"widget":"column","padding":12,"children":["gap"]},"gap":{"widget":"spacer","size":20}}),
    );
    let mut view = SceneView::new(&source).unwrap();
    let ctx = egui::Context::default();
    draw(&mut view, &ctx, 400., 200.);
    let row = view.widget_rectangles(&json!({"ids":["gap"]})).unwrap();
    assert_eq!(row.as_array().unwrap().len(), 1);
    assert_eq!(row[0]["rect"]["x"], json!(12.));
    assert_eq!(row[0]["rect"]["y"], json!(12.));
    assert_eq!(row[0]["rect"]["h"], json!(20.));
}
