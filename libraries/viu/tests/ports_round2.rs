// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Duration,
};
use viu::{
    bundle::Bundle,
    runtime::{Completion, Program, Runtime},
    view::SceneView,
};
fn done(runtime: &Runtime, rx: &mpsc::Receiver<()>) -> Completion {
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    runtime.try_completion().unwrap()
}
#[test]
fn round_two_bundles_publish_and_draw_through_the_real_embedded_interpreter() {
    let scratch = tempfile::tempdir().unwrap();
    for name in ["screenshot", "prefs", "ced", "workshop"] {
        let args = if name == "screenshot" {
            vec![
                "comp".into(),
                scratch.path().join("saved").to_string_lossy().into(),
                scratch.path().join("raw").to_string_lossy().into(),
            ]
        } else {
            vec!["example".into()]
        };
        let bundle = Bundle::load(
            &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../apps")
                .join(name),
            &args,
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
        let result = done(&runtime, &rx);
        assert!(result.error.is_none(), "{name}: {:?}", result.error);
        let model = result.model.unwrap();
        let mut view = if bundle.manifest.frame.is_some() {
            viu::gold_frame::mount(&bundle.scene, model)
        } else {
            SceneView::with_model(&bundle.scene, model)
        }.unwrap();
        let ctx = eframe::egui::Context::default();
        ctx.run_ui(
            eframe::egui::RawInput {
                screen_rect: Some(eframe::egui::Rect::from_min_size(
                    eframe::egui::Pos2::ZERO,
                    eframe::egui::vec2(980.0, 760.0),
                )),
                ..Default::default()
            },
            |ui| {
                view.show(ui);
            },
        )
        .drop_without_applying_deltas();
        assert!(!view.geometry.is_empty());
    }
}
#[test]
fn editor_is_reusable_and_does_not_relax_existing_text_controls() {
    let editor = viu::widgets::Widget::new("notes", "editor", "", json!(""));
    assert!(editor.validate(&json!("🦀".repeat(16384))).is_ok());
    assert!(editor.validate(&json!("x".repeat(65537))).is_err());
    assert!(editor.validate(&json!(false)).is_err());
    let field = viu::widgets::Widget::new("path", "field", "", json!(""));
    assert!(field.validate(&json!("x".repeat(4097))).is_err());
}
#[test]
fn shared_image_export_is_bounded_and_never_overwrites() {
    let scratch = tempfile::tempdir().unwrap();
    let source = scratch.path().join("source.png");
    let destination = scratch.path().join("saved.png");
    image::RgbImage::from_pixel(4, 3, image::Rgb([12, 34, 56]))
        .save(&source)
        .unwrap();
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        4,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime.load(Program{path:scratch.path().join("test.mix"),source:r#"on image; try reply(0,json_encode(viu_image("export",$event.args.source,$event.args.destination))) catch $message reply(10,json_encode({error:$message})) end end"#.into(),scene:"image".into(),citizen:"image".into()}).unwrap();
    assert!(done(&runtime, &rx).error.is_none());
    let args = json!({"source":source,"destination":destination});
    runtime.event("image".into(), args.clone()).unwrap();
    assert_eq!(done(&runtime, &rx).reply.unwrap().body["width"], 4);
    let before = std::fs::read(&destination).unwrap();
    runtime.event("image".into(), args).unwrap();
    assert_eq!(done(&runtime, &rx).reply.unwrap().rc, 10);
    assert_eq!(std::fs::read(&destination).unwrap(), before);
}
