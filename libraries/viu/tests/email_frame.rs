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

fn complete(runtime: &Runtime, wakes: &mpsc::Receiver<()>) -> Completion {
    wakes.recv_timeout(Duration::from_secs(10)).unwrap();
    let done = runtime.try_completion().unwrap();
    assert!(done.error.is_none(), "{:?}", done.error);
    done
}

#[test]
fn email_empty_frame_draws_and_primary_button_keeps_native_intent() {
    let bundle = Bundle::load(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/email"),
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
    let model = complete(&runtime, &rx).model.unwrap();
    assert_eq!(model["no_accounts"], true);
    let mut view = viu::gold_frame::mount(&bundle.scene, model).unwrap();
    let ctx = egui::Context::default();
    viu::appearance::install(&ctx);
    let mut input = viu::input::Input::default();
    let mut intents = Vec::new();
    for step in 0..5 {
        if step == 2 {
            input
                .queue("click", &json!({"id":"open-settings"}), &mut view)
                .unwrap();
        }
        let events = if !input.now.is_empty() {
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
                events,
                ..Default::default()
            },
            |ui| {
                intents.extend(view.show(ui));
            },
        )
        .drop_without_applying_deltas();
    }
    assert!(view.geometry["empty-title"].top() >= view.geometry["gold-header"].bottom());
    assert!(view.geometry["open-settings"].bottom() < view.geometry["gold-status"].top());
    assert!(
        intents
            .iter()
            .any(|e| e.command == "email.intent" && e.body["id"] == "open-settings")
    );
    runtime
        .event("email.execute".into(), json!({"id":"mail.compose"}))
        .unwrap();
    let reply = complete(&runtime, &rx).reply.unwrap();
    assert_eq!(reply.rc, 10);
    assert_eq!(reply.body["error_code"], "DISABLED");
    runtime
        .event("email.dialog".into(), json!({"open":"about"}))
        .unwrap();
    let model = complete(&runtime, &rx).model.unwrap();
    assert_eq!(
        model["native_overlays"][0]["definition"]["initial"]["title"],
        "About Email"
    );
    runtime.event("email.dialog".into(), json!({"open":"shortcuts"})).unwrap();
    let model=complete(&runtime,&rx).model.unwrap();
    let widget=viu::widgets::Widget::decode(&model["native_overlays"][0]["definition"]).unwrap();
    assert!(widget.initial["grid"].as_array().unwrap().len()>12);
    let mut overlays=viu::widgets::Widgets::default();
    for _ in 0..3 {
        ctx.run_ui(egui::RawInput {
            screen_rect:Some(egui::Rect::from_min_size(egui::Pos2::ZERO,egui::vec2(1040.0,680.0))),
            ..Default::default()
        },|ui| { overlays.widget(ui,&widget,&model); }).drop_without_applying_deltas();
    }
    assert!(overlays.geometry["email-help"].height()>400.0,"shortcut rows were not drawn");
}
