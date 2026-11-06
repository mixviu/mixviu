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
fn completion(runtime: &Runtime, wakes: &mpsc::Receiver<()>) -> Completion {
    wakes.recv_timeout(Duration::from_secs(10)).unwrap();
    let done = runtime.try_completion().unwrap();
    assert!(
        done.error.is_none(),
        "{:?}\n{}\n{}",
        done.error,
        done.stdout,
        done.stderr
    );
    done
}
fn app(name: &str) -> (Bundle, Runtime, mpsc::Receiver<()>) {
    let bundle = Bundle::load(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../apps")
            .join(name),
        &[],
    )
    .unwrap();
    let (tx, rx) = mpsc::channel();
    let runtime = Runtime::with_services(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
        viu::runtime::Services { rasters: Some(Default::default()), timers: Some(Default::default()), ui_state: Some(Default::default()), ..Default::default() },
    );
    (bundle, runtime, rx)
}
#[test]
fn all_mix_bundles_publish_models_the_actual_scene_adapter_can_draw() {
    for name in ["gallery", "backdrop", "monitor"] {
        let (bundle, mut runtime, wakes) = app(name);
        runtime.load(bundle.program).unwrap();
        let model = completion(&runtime, &wakes).model.unwrap();
        let mut view = if bundle.manifest.frame.is_some() {
            viu::gold_frame::mount(&bundle.scene, model)
        } else {
            SceneView::with_model(&bundle.scene, model)
        }
        .unwrap();
        let ctx = eframe::egui::Context::default();
        viu::icons::install(&ctx);
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
        assert!(!view.geometry.is_empty(), "{name}");
    }
}
#[test]
fn bundle_arguments_are_literal_data_including_mix_interpolation() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("scene.mix"), "unused").unwrap();
    std::fs::write(dir.path().join("app.mix"),r#"$ctx=viu_context(); send "viu-local" "scenes.model" body={generation:$ctx.generation,value:{args:$app_args}}"#).unwrap();
    let args = vec![
        "${raise(\"NO\",\"bad\")}".into(),
        "~/literal".into(),
        "a\"b\\c\n".into(),
    ];
    let bundle = Bundle::load(dir.path(), &args).unwrap();
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        2,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime.load(bundle.program).unwrap();
    assert_eq!(
        completion(&runtime, &rx).model.unwrap()["args"],
        json!(args)
    );
}
