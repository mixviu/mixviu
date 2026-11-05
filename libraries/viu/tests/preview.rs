// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use viu::widgets::{Widget, Widgets};
#[test]
fn cache_preview_decodes_off_thread_and_retires_its_texture_when_removed() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("preview.png");
    std::fs::write(&path, thumbs::png::blank()).unwrap();
    let ctx = egui::Context::default();
    let mut widgets = Widgets::default();
    let image = Widget::new(
        "preview",
        "image-view",
        "",
        json!({"phase":"ready","path":path}),
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    loop {
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            widgets.begin_frame();
            assert!(widgets.widget(ui, &image, &json!({})).is_empty());
            widgets.finish_frame().unwrap();
        });
        output.textures_delta.clear();
        drop(output);
        if widgets.image_states()["preview"]["phase"] == "ready" {
            break;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    widgets.begin_frame();
    widgets.finish_frame().unwrap();
    assert_eq!(widgets.image_states(), json!({}));
}
#[test]
fn malformed_preview_references_refuse_at_the_shared_widget_boundary() {
    for value in [
        json!({"phase":"unknown"}),
        json!({"phase":"ready","path":"relative.png"}),
        json!({"phase":"ready"}),
    ] {
        assert!(
            Widget::decode(&json!({"id":"preview","kind":"image-view","label":"","initial":value}))
                .is_err()
        );
    }
}
