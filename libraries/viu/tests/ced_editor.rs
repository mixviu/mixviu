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
};
mod fixture {
    include!("support/documents.rs");
}
struct JobsAccepted;
impl mix::BusHandler for JobsAccepted {
    fn send<'a>(
        &'a self,
        _: &'a str,
        _: &'a str,
        _: &'a mix::value::Value,
    ) -> mix::evaluator::BusFuture<'a, mix::MixResult<(i32, mix::value::Value)>> {
        Box::pin(async { Ok((0, mix::value::Value::Nil)) })
    }
    fn emit<'a>(
        &'a self,
        _: &'a str,
        _: &'a str,
        _: &'a mix::value::Value,
    ) -> mix::evaluator::BusFuture<'a, mix::MixResult<()>> {
        Box::pin(async { Ok(()) })
    }
    fn port_exists<'a>(
        &'a self,
        _: &'a str,
    ) -> mix::evaluator::BusFuture<'a, mix::MixResult<bool>> {
        Box::pin(async { Ok(false) })
    }
    fn next_incoming<'a>(
        &'a self,
    ) -> mix::evaluator::BusFuture<'a, Option<mix::evaluator::IncomingEvent>> {
        Box::pin(async { None })
    }
}

fn done(runtime: &Runtime, rx: &mpsc::Receiver<()>) -> Completion {
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    runtime.try_completion().unwrap()
}

#[test]
fn ced_retains_independent_tab_drafts_and_preserves_recoverable_text_on_caption_close() {
    let mut bundle = Bundle::load(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/ced"),
        &[],
    )
    .unwrap();
    bundle.program.source.push_str(
        r#"
$state.tabs=[viu_document("state","one",{}),viu_document("state","two",{})]
$state.classified={one:true,two:true}
publish_ced($state)
"#,
    );
    let docs = viu::documents::Documents::default();
    let _one = fixture::seed(&docs, "one", "/tmp/one.rs", "original");
    let _two = fixture::seed(&docs, "two", "/tmp/two.mix", "second");
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::with_services(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        Some(Box::new(|| std::rc::Rc::new(JobsAccepted))),
        viu::runtime::Services {
            documents: Some(docs.clone()),
            ..Default::default()
        },
    );
    runtime.load(bundle.program).unwrap();
    let initial = done(&runtime, &rx);
    assert!(initial.error.is_none(), "{:?}", initial.error);
    let model = initial.model.unwrap();
    assert_eq!(model["values"]["editor"]["document"], "one");
    let mut view = viu::gold_frame::mount(&bundle.scene, model).unwrap();
    view.set_documents(docs.clone());
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
    assert_eq!(
        docs.call("read", "one", json!({})).unwrap()["text"],
        "unsaved λ"
    );
    runtime
        .event("ced.intent".into(), json!({"id":"tabs","value":"1"}))
        .unwrap();
    let second = done(&runtime, &rx);
    assert!(second.error.is_none(), "{:?}", second.error);
    assert_eq!(second.model.unwrap()["values"]["editor"]["document"], "two");
    runtime
        .event("ced.intent".into(), json!({"id":"tabs","value":"0"}))
        .unwrap();
    assert_eq!(
        {
            assert!(done(&runtime, &rx).error.is_none());
            docs.call("read", "one", json!({})).unwrap()["text"].clone()
        },
        "unsaved λ"
    );
    runtime.event("viu.frame.close".into(), json!({})).unwrap();
    let refused = done(&runtime, &rx);
    assert!(refused.error.is_none(), "{refused:?}");
    assert_eq!(docs.call("read", "one", json!({})).unwrap()["text"], "unsaved λ");
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
    assert_eq!(
        docs.call("read", "one", json!({})).unwrap()["text"],
        "unsaved λ"
    );
    let appearance = viu::appearance::resolve(&model["appearance"]).unwrap();
    assert_eq!(appearance.body_weight, 700);
    view.apply_model(model).unwrap();
}
