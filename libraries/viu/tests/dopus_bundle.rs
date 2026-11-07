// SPDX-License-Identifier: MIT OR Apache-2.0
//! Dopus uses the generic runtime and frame, with no file-manager Rust host.
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

fn done(runtime: &Runtime, wakes: &mpsc::Receiver<()>) -> Completion {
    wakes.recv_timeout(Duration::from_secs(30)).unwrap();
    let result = runtime.try_completion().unwrap();
    assert!(result.error.is_none(), "{:?}", result.error);
    result
}

#[test]
fn generic_bundle_mounts_frame_and_preserves_both_panes_on_scan_refusal() {
    let fixture = tempfile::tempdir().unwrap();
    let left = fixture.path().join("left");
    let right = fixture.path().join("right");
    std::fs::create_dir(&left).unwrap();
    std::fs::create_dir(&right).unwrap();
    std::fs::write(left.join("selected.txt"), "fixture").unwrap();
    std::fs::write(
        fixture.path().join("config.conf.mix"),
        json!({
            "schema_version":2,"left":{"path":left,"previews":false},
            "right":{"path":right,"previews":false}
        })
        .to_string(),
    )
    .unwrap();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/dopus");
    let mut bundle = Bundle::load(&root, &[fixture.path().to_string_lossy().into_owned()]).unwrap();
    assert_eq!(bundle.manifest.frame.as_deref(), Some("gold"));
    bundle.program.source.push_str(
        r#"
on budget-test
  $state.panes[1].scan_budget={rows:1,depth:0}
  reply(0,"{}")
end
"#,
    );
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime.load(bundle.program).unwrap();
    let model = done(&runtime, &rx).model.unwrap();
    assert_eq!(model["panes"][0]["rows"], 1);
    assert_eq!(
        model["values"]["gold-title"],
        format!("{} — MixOS DOpus", left.display())
    );
    let mut view = viu::gold_frame::mount(&bundle.scene, model.clone()).unwrap();
    assert!(view.has_control_kind("navigation-list"));
    let ctx = egui::Context::default();
    viu::appearance::install(&ctx);
    ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1040., 680.),
            )),
            ..Default::default()
        },
        |ui| {
            view.show(ui);
        },
    )
    .drop_without_applying_deltas();
    assert!(view.geometry.contains_key("gold-header"));
    assert!(view.geometry.contains_key("left-tree"));
    assert!(!view.geometry.contains_key("topbar"));
    runtime.event("budget-test".into(), json!({})).unwrap();
    done(&runtime, &rx);
    // First navigation succeeds in the candidate; second exhausts its budget.
    // Neither pane nor the published model may commit the partial request.
    let broad = fixture.path().join("broad");
    std::fs::create_dir(&broad).unwrap();
    for name in ["a", "b"] {
        std::fs::write(broad.join(name), "fixture").unwrap();
    }
    runtime
        .event("dopus.open".into(), json!({"paths":[right,broad]}))
        .unwrap();
    let refusal = done(&runtime, &rx);
    assert!(refusal.model.is_none());
    assert_eq!(refusal.reply.unwrap().body["error_code"], "SCAN_BUDGET");
    runtime.event("dopus.state".into(), json!({})).unwrap();
    let state = done(&runtime, &rx).reply.unwrap().body;
    assert_eq!(state["panes"][0]["path"], json!(left));
    assert_eq!(state["panes"][1]["path"], json!(right));
}
