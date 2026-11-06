// SPDX-License-Identifier: MIT OR Apache-2.0
mod fixture {
    include!("support/documents.rs");
}
use fixture::{seed, settle};
use serde_json::json;
use viu::documents::Documents;

#[test]
fn large_mirror_stays_out_of_scene_models_and_edits_are_in_process_until_mix_drains() {
    let docs = Documents::default();
    let text = "// Unicode λ long line\n".repeat(400_000);
    let mut server = seed(&docs, "large", "/tmp/large.rs", &text);
    let before = docs.call("state", "large", json!({})).unwrap();
    assert_eq!(before["bytes"], text.len());
    assert!(serde_json::to_vec(&before).unwrap().len() < 4096);
    docs.call("select", "large", json!({"anchor":0,"head":0}))
        .unwrap();
    docs.call(
        "command",
        "large",
        json!({"name":"insert","text":"local λ\n"}),
    )
    .unwrap();
    assert_eq!(
        server.rev(before["buffer"].as_str().unwrap()),
        0,
        "no implicit transport"
    );
    let page = docs
        .call("read", "large", json!({"start":0,"max_bytes":64}))
        .unwrap();
    assert!(page["text"].as_str().unwrap().starts_with("local λ\n"));
    assert!(page["next"].is_number());
    let ctx = egui::Context::default();
    viu::appearance::install(&ctx);
    let mut widgets = viu::widgets::Widgets::default();
    widgets.set_documents(docs.clone());
    let widget = viu::widgets::Widget::new("editor", "editor", "", json!({"document":"large"}))
        .with_properties(json!({"native_document":true}));
    let output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1000.0, 720.0),
            )),
            ..Default::default()
        },
        |ui| {
            let events = widgets.widget(ui, &widget, &json!({}));
            assert!(
                serde_json::to_vec(&events.iter().map(|e| e.encode()).collect::<Vec<_>>())
                    .unwrap()
                    .len()
                    < 8192
            );
        },
    );
    assert!(output.shapes.len() < 2000, "only visible rows are painted");
    output.drop_without_applying_deltas();
    settle(&docs, "large", &mut server);
    assert!(
        server
            .text(before["buffer"].as_str().unwrap())
            .starts_with("local λ\n")
    );
    docs.call("command", "large", json!({"name":"undo"}))
        .unwrap();
    settle(&docs, "large", &mut server);
    assert_eq!(server.text(before["buffer"].as_str().unwrap()), text);
}
