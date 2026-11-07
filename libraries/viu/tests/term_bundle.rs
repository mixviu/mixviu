// SPDX-License-Identifier: MIT OR Apache-2.0
//! Execute the actual Mix bundle without host tick callbacks masking its first
//! load error. The child process owns an isolated HOME and runtime directory.
use std::{sync::{Arc, mpsc}, time::Duration};

#[test]
fn isolated_term_bundle_load() {
    if std::env::var_os("VIU_TERM_BUNDLE_CHILD").is_some() { return; }
    let root = tempfile::tempdir().unwrap();
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .args(["--exact", "term_bundle_child", "--nocapture"])
        .env("VIU_TERM_BUNDLE_CHILD", "1")
        .env("HOME", root.path())
        .env("XDG_RUNTIME_DIR", root.path())
        .env("XDG_CONFIG_HOME", root.path())
        .env("MIXOS_RUN", root.path())
        .env("MIXOS_BUS_SOCKET", root.path().join("absent.sock"))
        .env_remove("MIXOS_SESSION_FD")
        .output().unwrap();
    assert!(output.status.success(), "stdout={} stderr={}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert!(String::from_utf8_lossy(&output.stdout).contains("TERM_BUNDLE_LOADED"));
}

#[test]
fn term_bundle_child() {
    if std::env::var_os("VIU_TERM_BUNDLE_CHILD").is_none() { return; }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/term");
    let bundle = viu::bundle::Bundle::load(&root, &[]).unwrap();
    let scene = std::fs::read_to_string(root.join("scene.mix")).unwrap();
    let (tx, rx) = mpsc::channel();
    let sessions = viu::terminal::sessions::Sessions::default();
    let mut runtime = viu::runtime::Runtime::with_services(4, Arc::new(move || { let _ = tx.send(()); }), None,
        viu::runtime::Services { terminals: Some(sessions.clone()), ..Default::default() });
    runtime.load(bundle.program).unwrap();
    rx.recv_timeout(Duration::from_secs(25)).expect("initial load completion");
    let done = runtime.try_completion().unwrap();
    assert!(done.error.is_none(), "initial load error: {:?}; stdout={}; stderr={}", done.error, done.stdout, done.stderr);
    let model = done.model.expect("initial model");
    let handle = model["terminal"]["initial"].as_str().unwrap().to_owned();
    viu::gold_frame::mount(&scene, model).expect("native frame and terminal model validate");
    sessions.call("close", &handle, serde_json::json!({})).unwrap();
    println!("TERM_BUNDLE_LOADED");
}
