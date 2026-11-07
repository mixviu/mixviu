// SPDX-License-Identifier: MIT OR Apache-2.0
//! Execute the actual Mix bundle without host tick callbacks masking its first
//! load error. The child process owns an isolated HOME and runtime directory.
use serde_json::json;
use std::{
    sync::{Arc, mpsc},
    time::Duration,
};

fn invoke(
    runtime: &mut viu::runtime::Runtime,
    rx: &mpsc::Receiver<()>,
    command: &str,
    args: serde_json::Value,
) -> viu::runtime::Completion {
    let sequence = runtime.event(command.into(), args).unwrap();
    loop {
        if let Some(done) = runtime.try_completion() {
            assert!(
                done.error.is_none(),
                "{command}: {:?}; stderr={}",
                done.error,
                done.stderr
            );
            if done.sequence == sequence {
                return done;
            }
        } else {
            rx.recv_timeout(Duration::from_secs(25))
                .expect("command completion");
        }
    }
}

#[test]
fn isolated_term_bundle_load() {
    if std::env::var_os("VIU_TERM_BUNDLE_CHILD").is_some() {
        return;
    }
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
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("TERM_BUNDLE_LOADED"));
}

#[test]
fn term_bundle_child() {
    if std::env::var_os("VIU_TERM_BUNDLE_CHILD").is_none() {
        return;
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../apps/term");
    let bundle = viu::bundle::Bundle::load(&root, &[]).unwrap();
    let scene = std::fs::read_to_string(root.join("scene.mix")).unwrap();
    let (tx, rx) = mpsc::channel();
    let sessions = viu::terminal::sessions::Sessions::default();
    let mut runtime = viu::runtime::Runtime::with_services(
        4,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
        viu::runtime::Services {
            terminals: Some(sessions.clone()),
            ..Default::default()
        },
    );
    runtime.load(bundle.program).unwrap();
    rx.recv_timeout(Duration::from_secs(25))
        .expect("initial load completion");
    let done = runtime.try_completion().unwrap();
    assert!(
        done.error.is_none(),
        "initial load error: {:?}; stdout={}; stderr={}",
        done.error,
        done.stdout,
        done.stderr
    );
    let model = done.model.expect("initial model");
    viu::gold_frame::mount(&scene, model).expect("native frame and terminal model validate");
    let split = invoke(
        &mut runtime,
        &rx,
        "term.pane.split",
        json!({"dir":"v","request_id":"split"}),
    );
    let split_reply = split.reply.unwrap();
    assert_eq!(split_reply.rc, 0, "{}", split_reply.body);
    assert!(
        split_reply
            .body
            .as_str()
            .unwrap()
            .starts_with("split id=2 dir=v tab=1 pane=2 revision=2")
    );
    let split_model = split.model.unwrap();
    assert_eq!(split_model["terminal"]["initial"]["first"], "vt-1");
    assert_eq!(split_model["terminal"]["initial"]["second"], "vt-2");
    viu::gold_frame::mount(&scene, split_model).expect("split and tab model validate");
    let replay = invoke(
        &mut runtime,
        &rx,
        "term.pane.split",
        json!({"request_id":"split","dir":"v"}),
    );
    assert_eq!(replay.reply.unwrap().body, split_reply.body);
    let opened = invoke(&mut runtime, &rx, "term.tab.new", json!({"title":"Pinned"}));
    assert_eq!(opened.reply.unwrap().rc, 0);
    let bad = invoke(
        &mut runtime,
        &rx,
        "term.type",
        json!({"text":"do not type","request_id":"retry"}),
    );
    assert_eq!(bad.reply.unwrap().rc, 10);
    let typed = invoke(
        &mut runtime,
        &rx,
        "term.type",
        json!({"pane":1,"text":"","request_id":"retry"}),
    );
    let typed = typed.reply.unwrap();
    assert_eq!(typed.rc, 0, "{}", typed.body);
    assert!(
        typed
            .body
            .as_str()
            .unwrap()
            .ends_with("tab=1 pane=1 revision=3")
    );
    let tabs = invoke(&mut runtime, &rx, "term.tabs", json!({}))
        .reply
        .unwrap();
    assert!(
        tabs.body
            .as_str()
            .unwrap()
            .contains("id=2 active=true title=Pinned")
    );
    let closed = invoke(&mut runtime, &rx, "term.tab.close", json!({"id":1}))
        .reply
        .unwrap();
    assert_eq!(closed.rc, 0);
    assert!(sessions.call("state", "vt-1", json!({})).is_err());
    assert!(sessions.call("state", "vt-2", json!({})).is_err());
    let late = invoke(
        &mut runtime,
        &rx,
        "term.type",
        json!({"pane":1,"text":"","request_id":"retry"}),
    )
    .reply
    .unwrap();
    assert_eq!(
        late.body, typed.body,
        "successful retry retains its original outcome after close"
    );
    let about = invoke(&mut runtime, &rx, "term.dialog", json!({"open":"about"}));
    assert_eq!(about.reply.unwrap().rc, 0);
    let model = about.model.unwrap();
    let dialog = model["native_overlays"].as_array().unwrap().iter()
        .find(|overlay| overlay["definition"]["id"]=="term-dialog").unwrap();
    let body = dialog["definition"]["initial"]["body"].as_str().unwrap();
    assert!(body.contains(&buildinfo::build_info!().line()), "About must carry actual compiled host provenance: {body}");
    assert!(body.ends_with("Settings source: settingsd not connected"));
    invoke(&mut runtime, &rx, "term.dialog", json!({"answer":"done"}));
    let quit = invoke(&mut runtime, &rx, "app.quit", json!({}));
    assert_eq!(quit.reply.unwrap().rc, 0);
    assert!(quit.model.unwrap()["window_close"] == true);
    assert!(sessions.call("state", "vt-3", json!({})).is_err());
    println!("TERM_BUNDLE_LOADED");
}
