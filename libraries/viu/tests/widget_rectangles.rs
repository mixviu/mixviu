// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};
use viu::{bundle::Bundle, runtime::Runtime, view::SceneView};

fn draw(view: &mut SceneView, ctx: &egui::Context, width: f32, height: f32) {
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
            "root":{"widget":"column","children":["space","off","hidden"],"scroll":true},
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
    for (collect, queried) in [(false, false), (true, false), (true, true)] {
        #[cfg(feature = "test-support")]
        view.set_widget_rectangle_collection(collect);
        let start = Instant::now();
        for _ in 0..500 {
            draw(&mut view, &ctx, 800., 600.);
            if queried {
                std::hint::black_box(view.widget_rectangles(&json!({})).unwrap());
            }
        }
        println!(
            "widget_rectangles 100 buttons collect={collect} queried={queried} us/frame={:.3}",
            start.elapsed().as_secs_f64() * 1e6 / 500.
        );
    }
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
            &["--headless".into(), "--noded-url".into(), broker.url.clone()],
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
