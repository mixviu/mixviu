// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};
use viu::{
    bundle::Bundle,
    runtime::{Completion, Runtime},
    view::SceneView,
};
fn done(r: &Runtime, rx: &mpsc::Receiver<()>) -> Completion {
    rx.recv_timeout(Duration::from_secs(20)).unwrap();
    r.try_completion().unwrap()
}
fn stats(mut v: Vec<f64>) -> Value {
    v.sort_by(f64::total_cmp);
    json!({"median":v[v.len()/2],"p95":v[(v.len()*95).div_ceil(100)-1],"p99":v[(v.len()*99).div_ceil(100)-1],"raw_sorted":v})
}
#[test]
fn email_list_fixture_equivalence_bounds_and_2000_row_cost() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/email");
    let bundle = Bundle::load(&path, &["--fixture".into()]).unwrap();
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime.load(bundle.program).unwrap();
    assert!(done(&runtime, &rx).error.is_none());
    let messages: Vec<Value>=(0..2000).map(|i| json!({"id":format!("m{i:04}"),"subject":format!("Message {i:04} π"),"from":[{"email":"sender@example.test"}],"receivedAt":"2026-11-06T12:00:00Z","preview":"A bounded plain-text preview"})).collect();
    let mut reference = None;
    for bulk in [false, true] {
        let mut callback = Vec::new();
        let mut ack = Vec::new();
        let mut adapter = Vec::new();
        let mut draw = Vec::new();
        for sample in 0..24 {
            let start = Instant::now();
            runtime
                .event(
                    "email.fixture".into(),
                    json!({"messages":messages,"bulk":bulk}),
                )
                .unwrap();
            let result = done(&runtime, &rx);
            assert!(result.error.is_none(), "{:?}", result.error);
            let ack_ms = start.elapsed().as_secs_f64() * 1000.0;
            if std::env::var_os("MIX_VIU_BULK_PROFILE").is_some() {
                print!("{}", result.stdout);
            }
            let model = result.model.unwrap();
            assert_eq!(
                model["table"]["initial"]["rows"].as_array().unwrap().len(),
                2000
            );
            let rows = model["table"]["initial"]["rows"].clone();
            if let Some(expected) = &reference {
                assert_eq!(&rows, expected);
            } else {
                reference = Some(rows);
            }
            let start = Instant::now();
            let mut view = SceneView::with_model(&bundle.scene, model).unwrap();
            let adapter_ms = start.elapsed().as_secs_f64() * 1000.0;
            let ctx = egui::Context::default();
            let start = Instant::now();
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1100.0, 900.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    view.show(ui);
                },
            )
            .drop_without_applying_deltas();
            let draw_ms = start.elapsed().as_secs_f64() * 1000.0;
            assert!(view.geometry.contains_key("messages"));
            if sample >= 3 {
                callback.push(result.elapsed.as_secs_f64() * 1000.0);
                ack.push(ack_ms);
                adapter.push(adapter_ms);
                draw.push(draw_ms);
            }
        }
        println!(
            "EMAIL_2000 {}",
            json!({"bulk_jq":bulk,"samples":21,"callback_ms":stats(callback),"mailbox_ack_ms":stats(ack),"fresh_adapter_ms":stats(adapter),"cold_context_draw_ms":stats(draw)})
        );
    }
    for bad in [
        json!([{"id":"same"},{"id":"same"}]),
        json!(vec![json!({"id":"x"}); 2001]),
    ] {
        runtime
            .event("email.fixture".into(), json!({"messages":bad,"bulk":true}))
            .unwrap();
        let failed = done(&runtime, &rx);
        assert!(failed.error.is_some());
        assert!(failed.model.is_none());
    }
}
