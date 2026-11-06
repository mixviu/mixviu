// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Duration,
};
use viu::{
    bundle::Bundle,
    runtime::{Completion, Runtime},
    view::SceneView,
};

fn done(runtime: &Runtime, rx: &mpsc::Receiver<()>) -> Completion {
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    runtime.try_completion().unwrap()
}

#[test]
fn ced_retains_independent_tab_drafts_and_refuses_dirty_caption_close() {
    let mut bundle = Bundle::load(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/ced"),
        &[],
    )
    .unwrap();
    bundle.program.source.push_str(r#"
$state.tabs=[{buffer:"one",path:"/tmp/one.rs",text:"original",draft:"original",rev:1,language:"rust"},{buffer:"two",path:"/tmp/two.mix",text:"second",draft:"second",rev:2,language:"mix"}]
publish_ced($state)
"#);
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime.load(bundle.program).unwrap();
    let initial = done(&runtime, &rx);
    assert!(initial.error.is_none(), "{:?}", initial.error);
    let model = initial.model.unwrap();
    assert_eq!(model["editor"]["properties"]["document"], "one");
    let mut view = SceneView::with_model(&bundle.scene, model).unwrap();
    let ctx = egui::Context::default();
    viu::appearance::install(&ctx);
    ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(980.0, 800.0),
            )),
            ..Default::default()
        },
        |ui| {
            view.show(ui);
        },
    )
    .drop_without_applying_deltas();
    assert!(view.geometry["editor"].top() >= view.geometry["gold-header"].bottom());
    assert!(view.geometry["editor"].bottom() <= view.geometry["gold-status"].top());
    runtime
        .event("ced.draft".into(), json!({"text":"unsaved λ"}))
        .unwrap();
    assert!(done(&runtime, &rx).error.is_none());
    runtime
        .event(
            "ced.command".into(),
            json!({"op":"open","path":"/tmp/one.rs"}),
        )
        .unwrap();
    let reopened = done(&runtime, &rx);
    assert!(reopened.error.is_none(), "{:?}", reopened.error);
    assert_eq!(reopened.model.unwrap()["values"]["editor"], "unsaved λ");
    runtime
        .event("ced.intent".into(), json!({"id":"tabs","value":"1"}))
        .unwrap();
    let second = done(&runtime, &rx);
    assert!(second.error.is_none(), "{:?}", second.error);
    assert_eq!(
        second.model.unwrap()["editor"]["properties"]["document"],
        "two"
    );
    runtime
        .event("ced.intent".into(), json!({"id":"tabs","value":"0"}))
        .unwrap();
    assert_eq!(
        done(&runtime, &rx).model.unwrap()["values"]["editor"],
        "unsaved λ"
    );
    runtime.event("viu.frame.close".into(), json!({})).unwrap();
    let refused = done(&runtime, &rx);
    assert!(refused.error.is_some(), "{refused:?}");
    runtime
        .event(
            "viu.frame.intent".into(),
            json!({"value":{"command":"wrap"}}),
        )
        .unwrap();
    let wrapped = done(&runtime, &rx);
    assert!(wrapped.error.is_none(), "{:?}", wrapped.error);
    assert_eq!(
        wrapped.model.unwrap()["editor"]["properties"]["soft_wrap"],
        true
    );
    runtime
        .event(
            "viu.frame.intent".into(),
            json!({"value":{"command":"open"}}),
        )
        .unwrap();
    assert_eq!(
        done(&runtime, &rx).model.unwrap()["values"]["open-dialog"]["open"],
        true
    );
    runtime
        .event(
            "ced.intent".into(),
            json!({"id":"open-dialog","action":"edit","value":"/tmp/other.rs"}),
        )
        .unwrap();
    let edited = done(&runtime, &rx);
    assert!(edited.error.is_none(), "{:?}", edited.error);
    assert_eq!(
        edited.model.unwrap()["values"]["open-dialog"]["field"],
        "/tmp/other.rs"
    );
    runtime.event("ced.intent".into(), json!({"id":"open-dialog","action":"answer","value":{"field":"/tmp/other.rs","choice":"cancel"}})).unwrap();
    assert_eq!(
        done(&runtime, &rx).model.unwrap()["values"]["open-dialog"]["open"],
        false
    );
    runtime.event("viu.session".into(), json!({"appearance":{"scheme":"ocean","mode":"light","weight":700.0,"density":"compact","roundness":"square","text_scale":1.5},"state":{"revision":"3","lost":false,"values":{"ced.editor.font.size":18,"ced.editor.padding":16,"ced.editor.light":true}}})).unwrap();
    let changed = done(&runtime, &rx);
    assert!(changed.error.is_none(), "{:?}", changed.error);
    let model = changed.model.unwrap();
    assert_eq!(model["editor"]["properties"]["font_size"], 27.0);
    assert_eq!(model["editor"]["properties"]["padding"], 14.0);
    assert_eq!(model["editor"]["properties"]["light"], true);
    assert_eq!(model["values"]["editor"], "unsaved λ");
    let appearance = viu::appearance::resolve(&model["appearance"]).unwrap();
    assert_eq!(appearance.body_weight, 700);
    view.apply_model(model).unwrap();
}
