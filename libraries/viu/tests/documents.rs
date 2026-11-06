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

#[test]
fn epoch_reattach_preserves_a_differing_local_draft_until_keep_or_take() {
    for keep in [false, true] {
        let docs = Documents::default();
        let mut old = seed(&docs, "draft", "/tmp/recovery.rs", "old\n");
        docs.call("select", "draft", json!({"anchor":0,"head":0})).unwrap();
        docs.call("command", "draft", json!({"name":"insert","text":"mine λ\n"})).unwrap();
        // No reply or event for this edit: the process died with a local draft.
        assert_eq!(old.text(docs.call("state","draft",json!({})).unwrap()["buffer"].as_str().unwrap()), "old\n");
        docs.call("epoch", "draft", json!({})).unwrap();
        let detached = docs.call("state", "draft", json!({})).unwrap();
        assert_eq!(detached["reattach_needed"], true);
        assert_eq!(docs.call("read", "draft", json!({})).unwrap()["text"], "mine λ\nold\n");
        let mut server = editor_model::fake::FakeEditd::new("87654321");
        server.create(Some("/tmp/recovery.rs"), "theirs\n");
        let open = server.handle("ced", "edit.open", &json!({"path":"/tmp/recovery.rs"}));
        docs.call("reattach", "draft", open.body).unwrap();
        settle(&docs, "draft", &mut server);
        assert_eq!(docs.call("state", "draft", json!({})).unwrap()["detached_copy"], true);
        assert_eq!(docs.call("read", "draft", json!({})).unwrap()["text"], "theirs\n");
        docs.call(if keep { "keep-mine" } else { "take-theirs" }, "draft", json!({})).unwrap();
        settle(&docs, "draft", &mut server);
        assert_eq!(docs.call("read", "draft", json!({})).unwrap()["text"], if keep { "mine λ\nold\n" } else { "theirs\n" });
        assert_eq!(docs.call("state", "draft", json!({})).unwrap()["detached_copy"], false);
        docs.call("restore-view", "draft", json!({"caret":999999,"first_line":999999})).unwrap();
        let restored = docs.call("state", "draft", json!({})).unwrap();
        assert_eq!(restored["selection"]["head"]["offset"], restored["bytes"]);
        assert_eq!(restored["first_line"], restored["lines"]);
    }
}
