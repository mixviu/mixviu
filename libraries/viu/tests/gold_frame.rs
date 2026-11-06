// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Duration,
};
use viu::{
    runtime::{Program, Runtime},
    view::SceneView,
};

#[test]
fn mix_frame_preserves_application_content_and_intent_identity() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime.load(Program {
        path: root.join("mix/frame-test.mix"),
        source: "$frame=require(\"gold-frame.mix\");$model=$frame.apply({values:{content:42}},{icon:\"monitor\",title:\"Monitor\",menus:[],toolbar:[],status:\"Live\"});send \"viu-local\" \"scenes.model\" body={generation:viu_context().generation,value:$model}".into(),
        scene: "test".into(), citizen: "test".into(),
    }).unwrap();
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let done = runtime.try_completion().unwrap();
    assert!(done.error.is_none(), "{:?}", done.error);
    let model = done.model.unwrap();
    assert_eq!(model["values"]["content"], 42);
    let source = "---\nscene: 1\nname: test\ncitizen: test\n---\n```mix\n{root:{widget:\"column\",children:[\"content\"]},content:{widget:\"control\",definition:{id:\"content\",kind:\"slider\",label:\"Volume\",initial:0,properties:{min:0,max:100}},on_intent:\"app.intent\"}}\n```\n";
    let composed = viu::gold_frame::compose(source).unwrap();
    let mut view = SceneView::with_model(&composed, model).unwrap();
    let ctx = egui::Context::default();
    viu::appearance::install(&ctx);
    for _ in 0..2 {
        ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1040.0, 680.0),
                )),
                ..Default::default()
            },
            |ui| {
                view.show(ui);
            },
        )
        .drop_without_applying_deltas();
    }
    assert!(view.geometry["content"].top() >= view.geometry["gold-header"].bottom());
    assert!(view.geometry["content"].bottom() <= view.geometry["gold-status"].top());
    assert!(view.admit_intent("content", "set", &json!(50)).is_ok());
    assert!(composed.contains("app.intent"));
    assert!(viu::gold_frame::compose(&source.replace("content", "gold-title")).is_err());
    let control_root = source.replace(
        "root:{widget:\"column\",children:[\"content\"]},content:",
        "root:",
    );
    assert!(viu::gold_frame::compose(&control_root).is_err());
    let event_root = source.replace(
        "widget:\"column\",children:",
        "widget:\"column\",on_change:\"app.layout\",children:",
    );
    assert!(viu::gold_frame::compose(&event_root).is_err());
}
