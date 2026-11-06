// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
#[test]
fn native_receipts_are_bounded_once_only_and_close_after_successful_reply() {
    let calls = viu::calls::Calls::default();
    let close = Arc::new(AtomicBool::new(false));
    let (tx, mut rx) = tokio::sync::oneshot::channel();
    let key = calls.begin(tx, close.clone()).unwrap();
    let binding = calls.binding();
    // The public methods prove one completion; the interpreter test below
    // drives hold and deferred completion through the actual binding.
    assert!(calls.finish(&key, 0, json!({"answer":42}), true));
    assert_eq!(rx.try_recv().unwrap(), (0, json!({"answer":42})));
    assert!(close.load(Ordering::Acquire));
    assert!(!calls.finish(&key, 0, json!({}), false));
    drop(binding);
    let mut held = Vec::new();
    for _ in 0..32 {
        let (tx, rx) = tokio::sync::oneshot::channel();
        calls.begin(tx, close.clone()).unwrap();
        held.push(rx);
    }
    let (tx, _rx) = tokio::sync::oneshot::channel();
    assert!(calls.begin(tx, close).is_err());
    drop(held);
    assert_eq!(calls.cancelled().len(), 32);
}
#[test]
fn mix_holds_a_reply_across_callbacks_without_blocking_the_evaluator() {
    use std::{sync::mpsc, time::Duration};
    use viu::runtime::{Program, Runtime, Services};
    let calls = viu::calls::Calls::default();
    let (tx, mut answer) = tokio::sync::oneshot::channel();
    let token = calls.begin(tx, Arc::new(AtomicBool::new(false))).unwrap();
    let (wake, ready) = mpsc::channel();
    let mut runtime = Runtime::with_services(
        4,
        Arc::new(move || {
            let _ = wake.send(());
        }),
        None,
        Services {
            calls: Some(calls.clone()),
            ..Default::default()
        },
    );
    runtime.load(Program{path:"/tmp/replies.mix".into(),scene:"test".into(),citizen:"test".into(),source:"on wait\nviu_reply(\"hold\",$event.args.token,{})\nreply(0,\"{}\")\nend\non finish\nviu_reply(\"finish\",$event.args.token,{rc:0,body:{finished:true}})\nend".into()}).unwrap();
    ready.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(runtime.try_completion().unwrap().error.is_none());
    runtime
        .event("wait".into(), json!({"token":token}))
        .unwrap();
    ready.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(runtime.try_completion().unwrap().error.is_none());
    calls.complete(&token, 0, json!({"too_early":true}));
    assert!(answer.try_recv().is_err());
    runtime
        .event("finish".into(), json!({"token":token}))
        .unwrap();
    ready.recv_timeout(Duration::from_secs(5)).unwrap();
    assert!(runtime.try_completion().unwrap().error.is_none());
    assert_eq!(answer.try_recv().unwrap(), (0, json!({"finished":true})));
}
