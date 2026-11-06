// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Duration,
};
use viu::runtime::{Completion, Program, Runtime};

fn next(r: &Runtime, wakes: &mpsc::Receiver<()>) -> Completion {
    wakes.recv_timeout(Duration::from_secs(5)).unwrap();
    r.try_completion().unwrap()
}
fn program() -> Program {
    Program {
        path: PathBuf::from("/tmp/deferred.mix"),
        scene: "deferred".into(),
        citizen: "deferred".into(),
        source: r#"
$handles=[]
$ctx=viu_context()
on keep
  $h=viu_reply("keep")
  push($handles,$h)
  send "viu-local" "scenes.model" body={generation:$ctx.generation,value:{handle:$h,broker_from:$event.headers.from}}
end
on keep-and-reply
  viu_reply("keep")
  reply(0,"{}")
end
on complete
  $h=$handles[$event.args.index]
  $h.caller=$event.args.caller ?? $h.caller
  viu_reply("send",$h,10,{error_code:"CANCELLED",request:$h.request})
  if $event.args.fail==true then raise("FAULT","after staging") end
  reply(0,"{}")
end
on ping
  reply(0,"{\"alive\":true}")
end
on invalid
  try
    viu_reply("send",$event.args.handle,0,{})
    reply(0,"{}")
  catch $m
    reply(10,json_encode({error:$m}))
  end
end
"#
        .into(),
    }
}
fn setup() -> (Runtime, mpsc::Receiver<()>) {
    let (tx, rx) = mpsc::channel();
    let mut r = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    r.load(program()).unwrap();
    assert!(next(&r, &rx).error.is_none());
    (r, rx)
}

#[test]
fn requests_survive_callbacks_and_reply_to_the_original_caller_once() {
    let (mut r, wakes) = setup();
    let mut sequences = Vec::new();
    for caller in ["first", "second"] {
        sequences.push(
            r.event_with_headers(
                "keep".into(),
                json!({}),
                BTreeMap::from([("from".into(), caller.into())]),
            )
            .unwrap(),
        );
        let done = next(&r, &wakes);
        assert!(done.error.is_none(), "{:?}", done.error);
        assert!(done.deferred);
        assert!(done.reply.is_none());
    }
    r.event("ping".into(), json!({})).unwrap();
    assert_eq!(next(&r, &wakes).reply.unwrap().body, json!({"alive":true}));
    for index in [1, 0] {
        r.event("complete".into(), json!({"index":index})).unwrap();
        let done = next(&r, &wakes);
        assert!(done.error.is_none(), "{:?}", done.error);
        assert_eq!(done.answers.len(), 1);
        assert_eq!(done.answers[0].0, sequences[index]);
        assert_eq!(done.answers[0].1.rc, 10);
        assert_eq!(done.answers[0].1.body["error_code"], "CANCELLED");
    }
    r.event("complete".into(), json!({"index":0})).unwrap();
    let done = next(&r, &wakes);
    assert!(done.error.is_some());
    assert!(done.answers.is_empty());
}

#[test]
fn reply_receipts_preserve_broker_caller_headers_for_application_lanes() {
    let (mut r, wakes) = setup();
    let sequence = r.event_with_headers("keep".into(), json!({}), BTreeMap::from([
        ("from".into(), "stable-editor-agent".into()),
        ("viu-request-caller".into(), "wire-request-59".into()),
    ])).unwrap();
    let done = next(&r, &wakes);
    assert!(done.error.is_none(), "{:?}", done.error);
    let model = done.model.unwrap();
    assert_eq!(model["broker_from"], "stable-editor-agent");
    assert_eq!(model["handle"]["caller"], "wire-request-59");
    r.event("complete".into(), json!({"index":0})).unwrap();
    let done = next(&r, &wakes);
    assert!(done.error.is_none(), "{:?}", done.error);
    assert_eq!(done.answers[0].0, sequence);
}

#[test]
fn identity_faults_and_caller_loss_do_not_deliver_a_wrong_or_partial_reply() {
    let (mut r, wakes) = setup();
    let sequence = r.event("keep".into(), json!({})).unwrap();
    let kept = next(&r, &wakes);
    assert!(kept.deferred);
    let old_handle = kept.model.unwrap()["handle"].clone();
    r.event("complete".into(), json!({"index":0,"caller":"other"}))
        .unwrap();
    assert!(next(&r, &wakes).error.is_some());
    r.event("complete".into(), json!({"index":0,"fail":true}))
        .unwrap();
    let done = next(&r, &wakes);
    assert!(done.error.is_some());
    assert!(done.answers.is_empty());
    r.event("complete".into(), json!({"index":0})).unwrap();
    let retried = next(&r, &wakes);
    assert!(retried.error.is_none(), "{:?}", retried.error);
    assert_eq!(retried.answers[0].0, sequence);
    let forgotten = r.event("keep".into(), json!({})).unwrap();
    assert!(next(&r, &wakes).deferred);
    r.forget_reply(forgotten).unwrap();
    r.event("complete".into(), json!({"index":1})).unwrap();
    assert!(next(&r, &wakes).error.is_some());
    r.notify("keep".into(), json!({})).unwrap();
    assert!(
        next(&r, &wakes).error.is_some(),
        "notifications have no caller to retain"
    );
    r.load(program()).unwrap();
    assert!(next(&r, &wakes).error.is_none());
    r.event("invalid".into(), json!({"handle":old_handle}))
        .unwrap();
    assert_eq!(next(&r, &wakes).reply.unwrap().rc, 10);
    r.event("keep-and-reply".into(), json!({})).unwrap();
    let fault = next(&r, &wakes);
    assert!(fault.error.is_some());
    assert!(!fault.deferred);
    assert!(fault.reply.is_none());
}

#[test]
fn retained_requests_are_bounded_and_reload_releases_capacity() {
    let (mut r, wakes) = setup();
    for _ in 0..32 {
        r.event("keep".into(), json!({})).unwrap();
        assert!(next(&r, &wakes).deferred);
    }
    r.event("keep".into(), json!({})).unwrap();
    let done = next(&r, &wakes);
    assert!(done.error.is_some());
    assert!(!done.deferred);
    r.load(program()).unwrap();
    assert!(next(&r, &wakes).error.is_none());
    r.event("keep".into(), json!({})).unwrap();
    assert!(next(&r, &wakes).deferred);
}
