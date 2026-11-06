// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Duration,
};
use viu::runtime::{Completion, Program, Runtime};

fn completion(runtime: &Runtime, rx: &mpsc::Receiver<()>) -> Completion {
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    runtime.try_completion().unwrap()
}

#[test]
fn workshop_core_preserves_discovery_shapes_and_refuses_unsafe_requests() {
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime.load(Program {
        path: PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/workshop/probe.mix"),
        source: r#"
$c=require("core.mix")
on probe
  try
    $s=$c.initial();$s.services=$c.membership([{name:"echo"},{name:"echo"}])
    $s.commands=$c.catalogue($event.args.catalogue);$s.service="echo";$s.command=$event.args.command;$s.body=$event.args.text
    $s.busy=$event.args.busy ?? false
    $p=$c.call_payload($s)
    reply(0,json_encode({payload:$p,services:$s.services,commands:$s.commands,model:$c.project($s)}))
  catch $message, $error
    reply(10,json_encode({code:$error.code,message:$message}))
  end
end
"#.into(), scene:"workshop".into(), citizen:"workshop".into(),
    }).unwrap();
    assert!(completion(&runtime, &rx).error.is_none());
    let catalogue = json!(["echo", {"name":"echo"}, {"name":"fail","args":{"text":"string"},"description":"Refuse"}]);
    runtime
        .event(
            "probe".into(),
            json!({"catalogue":catalogue,"command":"echo","text":"{\"text\":\"π\"}"}),
        )
        .unwrap();
    let reply = completion(&runtime, &rx).reply.unwrap();
    assert_eq!(reply.rc, 0);
    assert_eq!(reply.body["services"], json!(["echo", "noded"]));
    assert_eq!(reply.body["commands"].as_array().unwrap().len(), 2);
    assert_eq!(reply.body["payload"]["body"], "{\"text\":\"π\"}");
    for (command, text, busy) in [
        ("unadvertised", "{}".to_owned(), false),
        ("echo", "{broken".to_owned(), false),
        ("echo", "🦀".repeat(17000), false),
        ("echo", "{}".to_owned(), true),
    ] {
        runtime.event("probe".into(), json!({"catalogue":{"commands":[{"name":"echo"}]},"command":command,"text":text,"busy":busy})).unwrap();
        assert_eq!(completion(&runtime, &rx).reply.unwrap().rc, 10);
    }
}

#[test]
fn editor_rows_are_bounded_without_changing_text_admission() {
    let editor = viu::widgets::Widget::new("body", "editor", "", json!(""));
    assert!(
        editor
            .clone()
            .with_properties(json!({"rows":8}))
            .validate(&json!("🦀".repeat(16384)))
            .is_ok()
    );
    for rows in [json!(0), json!(101), json!(1.5), json!("8")] {
        assert!(
            editor
                .clone()
                .with_properties(json!({"rows":rows}))
                .validate(&json!(""))
                .is_err()
        );
    }
}
