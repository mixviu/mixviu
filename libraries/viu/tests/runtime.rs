// SPDX-License-Identifier: MIT OR Apache-2.0
use std::{path::PathBuf, sync::{Arc, mpsc}, time::{Duration, Instant}};
use serde_json::json;
use viu::runtime::{Completion, Program, Refusal, Runtime};

fn program(source: &str) -> Program {
    Program { path: PathBuf::from("/tmp/viu-test.mix"), source: source.into(), scene: "test".into(), citizen: "scene-test".into() }
}
fn runtime(capacity: usize) -> (Runtime, mpsc::Receiver<()>) {
    let (tx, rx) = mpsc::channel();
    (Runtime::new(capacity, Arc::new(move || { let _ = tx.send(()); }), None), rx)
}
fn next(runtime: &Runtime, wakes: &mpsc::Receiver<()>) -> Completion {
    wakes.recv_timeout(Duration::from_secs(5)).expect("completion wakes idle host");
    runtime.try_completion().expect("wake follows result publication")
}
const INIT: &str = "$ctx = viu_context(); $n = 0; on bump; $n = $n + 1; send \"viu-local\" \"scenes.model\" body=json_encode({generation: $ctx.generation, value: {n:$n}}); reply(0, \"{}\"); end";

#[test]
fn real_evaluator_distinct_callbacks_wake_and_order() {
    let (mut runtime, wakes) = runtime(4);
    runtime.load(program(INIT)).unwrap();
    assert!(next(&runtime, &wakes).error.is_none());
    for _ in 0..4 { runtime.event("bump".into(), json!({})).unwrap(); }
    for n in 1..=4 {
        let done = next(&runtime, &wakes);
        assert!(done.error.is_none(), "{:?}", done.error);
        assert_eq!(done.model, Some(json!({"n":n})));
    }
    assert!(wakes.recv_timeout(Duration::from_millis(40)).is_err(), "idle has no periodic UI wakes");
}

#[test]
fn delayed_async_callback_bounds_mailbox_and_stale_generation() {
    let (mut runtime, wakes) = runtime(1);
    runtime.load(program("on slow async; sleep(0.15); reply(0, \"{}\"); end")).unwrap();
    assert!(next(&runtime, &wakes).error.is_none());
    runtime.event("slow".into(), json!({})).unwrap();
    // Wait for the worker to receive this request before filling its one slot.
    std::thread::sleep(Duration::from_millis(30));
    let t = Instant::now();
    runtime.load(program(INIT)).unwrap();
    assert_eq!(runtime.event("bump".into(), json!({})), Err(Refusal::Full));
    assert!(t.elapsed() < Duration::from_millis(40), "enqueue must not wait for behaviour");
    let old = next(&runtime, &wakes);
    assert!(!runtime.accepts(&old));
    assert!(runtime.accepts(&next(&runtime, &wakes)));
}

#[test]
fn callback_fault_discards_partial_publish_and_reload_recovers() {
    let (mut runtime, wakes) = runtime(4);
    runtime.load(program("$ctx=viu_context(); on fail; send \"viu-local\" \"scenes.model\" body=json_encode({generation:$ctx.generation,value:{bad:true}}); raise(\"LAB\",\"expected\"); end")).unwrap();
    assert!(next(&runtime, &wakes).error.is_none());
    runtime.event("fail".into(),json!({})).unwrap();
    let done=next(&runtime,&wakes);
    assert!(done.error.is_some());
    assert!(done.model.is_none());
    runtime.load(program("$x = [")).unwrap();
    assert!(next(&runtime,&wakes).error.is_some());
    runtime.event("fail".into(),json!({})).unwrap();
    assert!(next(&runtime,&wakes).error.is_some());
    runtime.load(program(INIT)).unwrap();
    assert!(next(&runtime,&wakes).error.is_none());
    runtime.event("bump".into(),json!({})).unwrap();
    assert_eq!(next(&runtime,&wakes).model,Some(json!({"n":1})));
}

#[test]
fn drop_cancels_sleeping_behaviour_and_reaps_worker() {
    let (mut runtime, wakes) = runtime(1);
    runtime.load(program("on wait async; sleep(10); end")).unwrap();
    assert!(next(&runtime,&wakes).error.is_none());
    runtime.event("wait".into(),json!({})).unwrap();
    std::thread::sleep(Duration::from_millis(20));
    let start=Instant::now();
    drop(runtime);
    assert!(start.elapsed()<Duration::from_secs(1));
}
