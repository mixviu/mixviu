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
    invoke_raw(runtime, rx, command, &args.to_string())
}
fn invoke_raw(
    runtime: &mut viu::runtime::Runtime,
    rx: &mpsc::Receiver<()>,
    command: &str,
    body: &str,
) -> viu::runtime::Completion {
    let sequence = runtime
        .event_with_body(command.into(), body.into(), Default::default())
        .unwrap();
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
    let help = invoke(&mut runtime, &rx, "HELP", json!({})).reply.unwrap();
    assert_eq!(help.rc, 0);
    let help = help.body;
    assert!(
        help.as_array()
            .unwrap()
            .iter()
            .any(|entry| { entry["name"] == "term.show" && entry["read_only"] == false })
    );
    let described = invoke(&mut runtime, &rx, "app.describe", json!({}))
        .reply
        .unwrap();
    assert_eq!(described.rc, 0);
    let described = described.body;
    assert_eq!(described["schema"], "term.v1");
    assert_eq!(described["app_id"], "dev.mixos.term");
    let info = invoke(&mut runtime, &rx, "INFO", json!({})).reply.unwrap();
    assert_eq!(info.rc, 0);
    assert!(
        info.body
            .as_str()
            .unwrap()
            .ends_with(&format!(" pid={}", std::process::id()))
    );
    for (command, body, expected) in [
        (
            "term.tab.select",
            r#"{"id":1.0}"#,
            "id must be a non-negative integer (u64)",
        ),
        (
            "term.tab.select",
            r#"{"id":1e0}"#,
            "id must be a non-negative integer (u64)",
        ),
        (
            "term.tab.select",
            r#"{"id":18446744073709551615}"#,
            "not-found: tab id=18446744073709551615",
        ),
        (
            "term.snapshot",
            r#"{"pane":18446744073709551615,"contents":false}"#,
            "not-found: pane id=18446744073709551615",
        ),
        (
            "term.type",
            r#"{"pane":null,"text":3}"#,
            "text must be a string",
        ),
        (
            "term.type",
            r#"{"pane":null,"text":""}"#,
            "invalid-argument: pane must be a non-negative integer (u64)",
        ),
    ] {
        let answer = invoke_raw(&mut runtime, &rx, command, body).reply.unwrap();
        assert_eq!(answer.rc, 10, "{command} {body}");
        assert_eq!(answer.body, json!(expected), "{command} {body}");
    }
    let moved = invoke_raw(
        &mut runtime,
        &rx,
        "term.tab.move",
        r#"{"id":1,"index":18446744073709551615}"#,
    )
    .reply
    .unwrap();
    assert_eq!(moved.rc, 0, "{}", moved.body);
    assert_eq!(
        moved.body,
        json!("moved id=1 index=0 tab=1 pane=1 revision=1")
    );
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
    // The real document-tabs widget emits term.intent, not a Bus command
    // envelope. Its translated id must reach the reducer; strict wire parsing
    // of the original UI body rejects action/value and silently loses clicks.
    let new_tab = invoke(&mut runtime, &rx, "term.tab.new", json!({"title":"UI tab"}));
    assert_eq!(new_tab.reply.unwrap().rc, 0);
    let selected = invoke(
        &mut runtime,
        &rx,
        "term.intent",
        json!({"id":"tabs","action":"select","value":"2"}),
    );
    assert_eq!(selected.reply.unwrap().rc, 0);
    let tabs = invoke(&mut runtime, &rx, "term.tabs", json!({}))
        .reply
        .unwrap();
    assert!(
        tabs.body
            .as_str()
            .unwrap()
            .contains("id=2 active=true title=Pinned")
    );
    let closed = invoke(
        &mut runtime,
        &rx,
        "term.intent",
        json!({"id":"tabs","action":"close","value":"3"}),
    );
    assert_eq!(closed.reply.unwrap().rc, 0);
    let tabs = invoke(&mut runtime, &rx, "term.tabs", json!({}))
        .reply
        .unwrap();
    assert_eq!(tabs.body.as_str().unwrap().lines().count(), 1);
    assert!(
        tabs.body
            .as_str()
            .unwrap()
            .contains("id=2 active=true title=Pinned")
    );
    assert!(
        sessions.call("state", "vt-4", json!({})).is_err(),
        "UI close must release its owned PTY"
    );
    let about = invoke(&mut runtime, &rx, "term.dialog", json!({"open":"about"}));
    assert_eq!(about.reply.unwrap().rc, 0);
    let model = about.model.unwrap();
    let dialog = model["native_overlays"]
        .as_array()
        .unwrap()
        .iter()
        .find(|overlay| overlay["definition"]["id"] == "term-dialog")
        .unwrap();
    let body = dialog["definition"]["initial"]["body"].as_str().unwrap();
    assert!(
        body.contains(&buildinfo::build_info!().line()),
        "About must carry actual compiled host provenance: {body}"
    );
    assert!(body.ends_with("Settings source: settingsd not connected"));
    invoke(&mut runtime, &rx, "term.dialog", json!({"answer":"done"}));
    // Standalone Mix tests use a different evaluator budget. Drive the real
    // persistent app through the deepest supported layout, including its
    // metadata, publication, focus and removal paths.
    for id in 5..=35 {
        let split = invoke(&mut runtime, &rx, "term.pane.split", json!({"dir":"v"}));
        let reply = split.reply.unwrap();
        assert_eq!(reply.rc, 0, "pane {id}: {}", reply.body);
        assert!(
            reply
                .body
                .as_str()
                .unwrap()
                .starts_with(&format!("split id={id} "))
        );
        if id == 35 {
            viu::gold_frame::mount(&scene, split.model.unwrap())
                .expect("32-pane published frame validates at native depth");
        }
    }
    let panes = invoke(&mut runtime, &rx, "term.panes", json!({}))
        .reply
        .unwrap();
    assert_eq!(panes.body.as_str().unwrap().lines().count(), 32);
    let overflow = invoke(&mut runtime, &rx, "term.pane.split", json!({"dir":"v"}))
        .reply
        .unwrap();
    assert_eq!(overflow.rc, 10);
    assert_eq!(overflow.body, json!("tab limit (32) reached"));
    let deepest = invoke(&mut runtime, &rx, "term.pane.close", json!({}))
        .reply
        .unwrap();
    assert_eq!(deepest.rc, 0, "{}", deepest.body);
    assert!(deepest.body.as_str().unwrap().starts_with("closed id=35 "));
    let selected = invoke(&mut runtime, &rx, "term.pane.select", json!({"id":3}))
        .reply
        .unwrap();
    assert_eq!(selected.rc, 0, "{}", selected.body);
    let first = invoke(&mut runtime, &rx, "term.pane.close", json!({}))
        .reply
        .unwrap();
    assert_eq!(first.rc, 0, "{}", first.body);
    assert!(first.body.as_str().unwrap().starts_with("closed id=3 "));
    let panes = invoke(&mut runtime, &rx, "term.panes", json!({}))
        .reply
        .unwrap();
    assert_eq!(panes.body.as_str().unwrap().lines().count(), 30);
    let quit = invoke(&mut runtime, &rx, "app.quit", json!({}));
    assert_eq!(quit.reply.unwrap().rc, 0);
    assert!(quit.model.unwrap()["window_close"] == true);
    assert!(sessions.call("state", "vt-3", json!({})).is_err());
    assert!(sessions.call("state", "vt-35", json!({})).is_err());
    println!("TERM_BUNDLE_LOADED");
}
