// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Duration,
};
use viu::{bundle::Bundle, runtime::Runtime};

#[test]
fn prefs_mounts_the_shared_frame_and_owns_caption_close() {
    let bundle = Bundle::load(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/prefs"),
        &[],
    )
    .unwrap();
    assert_eq!(bundle.manifest.frame.as_deref(), Some("gold"));
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime.load(bundle.program).unwrap();
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let done = runtime.try_completion().unwrap();
    assert!(done.error.is_none(), "{:?}", done.error);
    let model = done.model.unwrap();
    assert_eq!(model["values"]["gold-title"], "Preferences");
    let mut view = viu::gold_frame::mount(&bundle.scene, model).unwrap();
    let ctx = eframe::egui::Context::default();
    viu::icons::install(&ctx);
    ctx.run_ui(
        eframe::egui::RawInput {
            screen_rect: Some(eframe::egui::Rect::from_min_size(
                eframe::egui::Pos2::ZERO,
                eframe::egui::vec2(1040.0, 680.0),
            )),
            ..Default::default()
        },
        |ui| {
            view.show(ui);
        },
    )
    .drop_without_applying_deltas();
    assert!(view.geometry.contains_key("gold-header"));
    assert!(view.geometry.contains_key("table"));
    runtime.event("viu.frame.close".into(), json!({})).unwrap();
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let done = runtime.try_completion().unwrap();
    assert!(done.error.is_none(), "{:?}", done.error);
    assert_eq!(done.model.unwrap()["window_close"], true);
}
