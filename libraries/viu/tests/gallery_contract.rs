// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::{Value, json};
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
    wakes.recv_timeout(Duration::from_secs(10)).unwrap();
    let result = runtime.try_completion().unwrap();
    assert!(result.error.is_none(), "{:?}", result.error);
    result
}
fn gallery() -> (Runtime, mpsc::Receiver<()>) {
    let bundle = Bundle::load(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/gallery"),
        &[],
    )
    .unwrap();
    assert_eq!(bundle.manifest.service, "gallery");
    assert_eq!(bundle.manifest.app(), "gallery");
    assert_eq!(bundle.manifest.app_id.as_deref(), Some("dev.mixos.gallery"));
    assert_eq!(bundle.manifest.window_app_id, bundle.manifest.app_id);
    for command in [
        "app.describe",
        "gallery.describe",
        "gallery.actions",
        "gallery.execute",
    ] {
        assert!(bundle.manifest.commands.iter().any(|c| c == command));
    }
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime.load(bundle.program).unwrap();
    assert_eq!(done(&runtime, &rx).model.unwrap()["serial"], 0);
    (runtime, rx)
}
fn call(
    runtime: &mut Runtime,
    wakes: &mpsc::Receiver<()>,
    command: &str,
    args: Value,
) -> Completion {
    runtime.event(command.into(), args).unwrap();
    done(runtime, wakes)
}
#[test]
fn gallery_discovery_uses_the_admission_table_and_shared_action_ids() {
    let (mut runtime, wakes) = gallery();
    let describe = call(&mut runtime, &wakes, "app.describe", json!({}))
        .reply
        .unwrap();
    assert_eq!(describe.rc, 0);
    assert_eq!(describe.body["contract"], "ctk-app-control.v0");
    assert_eq!(describe.body["app"], "gallery");
    assert_eq!(
        call(&mut runtime, &wakes, "gallery.describe", json!({}))
            .reply
            .unwrap()
            .body,
        describe.body
    );
    let actions = call(&mut runtime, &wakes, "gallery.actions", json!({}))
        .reply
        .unwrap()
        .body;
    assert_eq!(describe.body["widgets"].as_array().unwrap().len(), 26);
    for definition in describe.body["widgets"].as_array().unwrap() {
        let widget = viu::widgets::Widget::decode(definition).unwrap();
        assert_eq!(definition["actions"], json!(widget.actions()));
        for action in widget.actions() {
            let id = format!("{}.{}", widget.id, action);
            let entry = actions["actions"]
                .as_array()
                .unwrap()
                .iter()
                .find(|a| a["id"] == id)
                .unwrap();
            assert_eq!(entry["enabled"], true);
            assert!(entry["label"].is_string());
            assert!(entry["keys"].is_array());
        }
    }
    let changed = call(
        &mut runtime,
        &wakes,
        "gallery.execute",
        json!({"id":"slider.set","value":72}),
    );
    assert_eq!(changed.reply.unwrap().rc, 0);
    assert_eq!(changed.model.unwrap()["values"]["slider"], 72);
    let reset = call(
        &mut runtime,
        &wakes,
        "gallery.execute",
        json!({"id":"reset"}),
    );
    assert_eq!(reset.reply.unwrap().rc, 0);
    assert_eq!(reset.model.unwrap()["values"]["slider"], 35);
}
#[test]
fn gallery_admission_refusals_are_typed_and_leave_state_unchanged() {
    let (mut runtime, wakes) = gallery();
    let before = call(&mut runtime, &wakes, "gallery.state", json!({}))
        .reply
        .unwrap()
        .body;
    for (args, code) in [
        (json!({"id":"collapse","action":"expand"}), "UNKNOWN_ACTION"),
        (
            json!({"id":"missing","action":"set","value":true}),
            "UNKNOWN_WIDGET",
        ),
        (
            json!({"id":"slider","action":"set","value":"bad"}),
            "ARGUMENT",
        ),
        (
            json!({"id":"tree","action":"expand","value":"bad"}),
            "ARGUMENT",
        ),
        (json!({"id":"missing"}), "UNKNOWN_ACTION"),
        (json!({}), "UNKNOWN_ACTION"),
        (json!(false), "ARGUMENT"),
    ] {
        let result = call(&mut runtime, &wakes, "gallery.execute", args);
        assert!(result.model.is_none());
        let reply = result.reply.unwrap();
        assert_ne!(reply.rc, 0);
        assert_eq!(reply.body["error_code"], code);
        assert!(
            reply.body["message"]
                .as_str()
                .is_some_and(|s| !s.is_empty())
        );
        assert_eq!(
            call(&mut runtime, &wakes, "gallery.state", json!({}))
                .reply
                .unwrap()
                .body,
            before
        );
    }
    let changed = call(
        &mut runtime,
        &wakes,
        "gallery.execute",
        json!({"id":"collapse","action":"set","value":true}),
    );
    assert_eq!(changed.reply.unwrap().rc, 0);
    assert_eq!(changed.model.unwrap()["values"]["collapse"], true);
}
