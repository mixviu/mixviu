// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Duration,
};
use viu::runtime::{Program, Runtime};

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
    let mut view = viu::gold_frame::mount(source, model.clone()).unwrap();
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
    assert!(view.current_revision_drawn());
    view.set_slots(std::collections::BTreeMap::from([(
        "window.header.content".into(),
        egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(900.0, 38.0)),
    )]))
    .unwrap();
    assert!(!view.current_revision_drawn());

    assert!(
        viu::gold_frame::mount(&source.replace("content", "gold-title"), model.clone()).is_err()
    );
    let control_root = source.replace(
        "root:{widget:\"column\",children:[\"content\"]},content:",
        "root:",
    );
    let control_root = control_root.replace("id:\"content\"", "id:\"root\"");
    let root_view = viu::gold_frame::mount(&control_root, model.clone()).unwrap();
    assert!(root_view.admit_intent("root", "set", &json!(50)).is_ok());
    let event_root = source.replace(
        "widget:\"column\",children:",
        "widget:\"dock\",on_change:\"app.layout\",placements:{content:{edge:\"fill\"}},children:",
    );
    assert!(viu::gold_frame::mount(&event_root, model).is_ok());
}

#[test]
fn mounted_root_control_delivers_its_original_native_intent() {
    let model = json!({"frame":{"icon":"icon:monitor","groups":{
        "gold-icon":{"preferred":28,"minimum":28,"align":"left","order":0,"shrink":null},
        "gold-menu":{"preferred":200,"minimum":100,"align":"left","order":1,"shrink":1},
        "gold-title":{"preferred":100,"minimum":0,"align":"center","order":2,"shrink":1},
        "gold-toolbar":{"preferred":100,"minimum":0,"align":"right","order":3,"shrink":1},
        "gold-captions":{"preferred":"window.captions","minimum":"window.captions","align":"window.captions","order":4,"shrink":null}
    }}});
    let source = "---\nscene: 1\nname: app\ncitizen: app\n---\n```mix\n{root:{widget:\"control\",definition:{id:\"root\",kind:\"button\",label:\"Original root\",initial:\"Press\"},on_intent:\"app.intent\"}}\n```\n";
    let mut view = viu::gold_frame::mount(source, model).unwrap();
    let ctx = egui::Context::default();
    let mut input = viu::input::Input::default();
    let mut events = Vec::new();
    for step in 0..5 {
        if step == 2 {
            input
                .queue("click", &json!({"id":"root"}), &mut view)
                .unwrap();
        }
        let native = if !input.now.is_empty() {
            std::mem::take(&mut input.now)
        } else {
            input.later.pop_front().unwrap_or_default()
        };
        ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1040.0, 680.0),
                )),
                events: native,
                ..Default::default()
            },
            |ui| {
                events.extend(view.show(ui));
            },
        )
        .drop_without_applying_deltas();
    }
    assert!(events.iter().any(|event| event.command == "app.intent"
        && event.body["id"] == "root"
        && event.body["action"] == "activate"));
}
