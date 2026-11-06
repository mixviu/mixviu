// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{
    path::Path,
    sync::{Arc, mpsc},
    time::Duration,
};
use viu::runtime::{Completion, Runtime};

fn next(runtime: &Runtime, wakes: &mpsc::Receiver<()>) -> Completion {
    wakes.recv_timeout(Duration::from_secs(10)).unwrap();
    runtime.try_completion().unwrap()
}

#[test]
fn screenshot_mix_controller_mounts_and_keeps_read_surfaces_available() {
    let dir = tempfile::tempdir().unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/screenshot");
    let bundle = viu::bundle::Bundle::load(
        &root,
        &[
            "comp-test".into(),
            dir.path().join("saved").to_string_lossy().into(),
            dir.path().join("raw").to_string_lossy().into(),
        ],
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
    let loaded = next(&runtime, &rx);
    assert!(loaded.error.is_none(), "{:?}", loaded.error);
    let model = loaded.model.expect("initial model");
    viu::gold_frame::mount(&bundle.scene, model).unwrap();
    for (command, args) in [
        ("screenshot.ping", json!({})),
        ("screenshot.info", json!({})),
        ("screenshot.actions", json!({})),
        ("screenshot.select", json!({"mode":"screen"})),
        ("screenshot.theme", json!({"mode":"light"})),
        ("screenshot.dialog", json!({"open":"about"})),
        ("screenshot.dialog", json!({"answer":"done"})),
    ] {
        runtime.event(command.into(), args).unwrap();
        let done = next(&runtime, &rx);
        assert!(done.error.is_none(), "{command}: {:?}", done.error);
        let reply = done.reply.expect(command);
        assert_eq!(reply.rc, 0, "{command}: {}", reply.body);
    }
    runtime
        .event(
            "screenshot.theme".into(),
            json!({"scheme":"invalid","mode":"dark"}),
        )
        .unwrap();
    let refused = next(&runtime, &rx).reply.unwrap();
    assert_eq!(refused.rc, 10);
    runtime.event("screenshot.theme".into(), json!({})).unwrap();
    assert_eq!(next(&runtime, &rx).reply.unwrap().body["mode"], "light");
}
