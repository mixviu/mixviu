// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};
use viu::runtime::{Completion, Program, Refusal, Runtime};

fn program(source: &str) -> Program {
    Program {
        path: PathBuf::from("/tmp/viu-test.mix"),
        source: source.into(),
        scene: "test".into(),
        citizen: "scene-test".into(),
    }
}
fn runtime(capacity: usize) -> (Runtime, mpsc::Receiver<()>) {
    let (tx, rx) = mpsc::channel();
    (
        Runtime::new(
            capacity,
            Arc::new(move || {
                let _ = tx.send(());
            }),
            None,
        ),
        rx,
    )
}
fn next(runtime: &Runtime, wakes: &mpsc::Receiver<()>) -> Completion {
    wakes
        .recv_timeout(Duration::from_secs(5))
        .expect("completion wakes idle host");
    runtime
        .try_completion()
        .expect("wake follows result publication")
}
const INIT: &str = "$ctx = viu_context(); $n = 0; on bump; $n = $n + 1; send \"viu-local\" \"scenes.model\" body=json_encode({generation: $ctx.generation, value: {n:$n}}); reply(0, \"{}\"); end";

#[test]
fn embedded_workers_refuse_global_environment_and_directory_mutation_but_keep_argv() {
    let (mut runtime, wakes) = runtime(4);
    runtime.load(program(r#"
on export-host
  try export VIU_TEST_HOST_ENV="bad";reply(0,"{}") catch $message reply(10,json_encode({message:$message})) end
end
on chdir-host
  try chdir("/");reply(0,"{}") catch $message reply(10,json_encode({message:$message})) end
end
on native-argv
  $child=run_argv(["true"],{timeout:1})
  reply(0,json_encode({ok:$child.ok}))
end
"#)).unwrap();
    assert!(next(&runtime, &wakes).error.is_none());
    let cwd = std::env::current_dir().unwrap();
    let before = std::env::var_os("VIU_TEST_HOST_ENV");
    for event in ["export-host", "chdir-host"] {
        runtime.event(event.into(), json!({})).unwrap();
        let done = next(&runtime, &wakes);
        assert!(done.error.is_none(), "{:?}", done.error);
        assert_eq!(done.reply.unwrap().rc, 10);
    }
    assert_eq!(std::env::current_dir().unwrap(), cwd);
    assert_eq!(std::env::var_os("VIU_TEST_HOST_ENV"), before);
    runtime.event("native-argv".into(), json!({})).unwrap();
    assert_eq!(
        next(&runtime, &wakes).reply.unwrap().body,
        json!({"ok":true})
    );
}

#[test]
fn caller_owned_macro_service_is_shared_without_an_installed_interpreter() {
    let dir = tempfile::tempdir().unwrap();
    let session = macros::Session::with_roots(
        "dopus",
        Some(macros::Roots {
            builtin: dir.path().join("builtin"),
            user: dir.path().join("user"),
            home: dir.path().into(),
            mix: dir.path().join("deliberately-no-executable"),
        }),
    );
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::with_services(
        4,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
        viu::runtime::Services {
            macros: Some(session.clone()),
            ..Default::default()
        },
    );
    runtime
        .load(program(
            r#"
on manage
  $reply=viu_macros($event.args.command,$event.args.body)
  reply($reply.rc,json_encode($reply.body))
end
"#,
        ))
        .unwrap();
    assert!(next(&runtime, &rx).error.is_none());
    runtime
        .event(
            "manage".into(),
            json!({"command":"new","body":{"title":"Worker created"}}),
        )
        .unwrap();
    let created = next(&runtime, &rx).reply.unwrap();
    assert_eq!(created.rc, 0);
    let id = created.body["id"].clone();
    assert!(
        session
            .catalogue()
            .unwrap()
            .entries
            .iter()
            .any(|e| json!(e.id) == id)
    );
    runtime
        .event(
            "manage".into(),
            json!({"command":"put","body":{"id":id,"text":"print(42)\n"}}),
        )
        .unwrap();
    assert_eq!(next(&runtime, &rx).reply.unwrap().rc, 0);
    runtime
        .event("manage".into(), json!({"command":"get","body":{"id":id}}))
        .unwrap();
    assert_eq!(
        next(&runtime, &rx).reply.unwrap().body["text"],
        "print(42)\n"
    );
    runtime
        .event(
            "manage".into(),
            json!({"command":"get","body":{"id":"missing"}}),
        )
        .unwrap();
    let refused = next(&runtime, &rx);
    assert!(refused.error.is_none());
    assert_eq!(refused.reply.unwrap().rc, 10);
    session.shutdown().unwrap();
}

#[test]
fn optional_read_and_atomic_replace_preserve_absence_and_refuse_linked_files() {
    let dir = tempfile::tempdir().unwrap();
    let outside = dir.path().join("outside");
    std::fs::write(&outside, "keep").unwrap();
    std::os::unix::fs::symlink(&outside, dir.path().join("linked")).unwrap();
    let (mut runtime, wakes) = runtime(4);
    runtime
        .load(program(
            r#"
on read
  try reply(0,json_encode(viu_read_optional($event.args.path)))
  catch $message reply(10,json_encode({error:$message})) end
end
on write
  try viu_atomic_replace($event.args.path,$event.args.text);reply(0,"{}")
  catch $message reply(10,json_encode({error:$message})) end
end
"#,
        ))
        .unwrap();
    assert!(next(&runtime, &wakes).error.is_none());
    runtime
        .event("read".into(), json!({"path":dir.path().join("missing")}))
        .unwrap();
    assert_eq!(
        next(&runtime, &wakes).reply.unwrap().body,
        json!({"found":false})
    );
    runtime
        .event(
            "write".into(),
            json!({"path":dir.path().join("new"),"text":"whole"}),
        )
        .unwrap();
    assert_eq!(next(&runtime, &wakes).reply.unwrap().rc, 0);
    runtime
        .event("read".into(), json!({"path":dir.path().join("new")}))
        .unwrap();
    assert_eq!(
        next(&runtime, &wakes).reply.unwrap().body,
        json!({"found":true,"text":"whole"})
    );
    for command in ["read", "write"] {
        runtime
            .event(
                command.into(),
                json!({"path":dir.path().join("linked"),"text":"bad"}),
            )
            .unwrap();
        let result = next(&runtime, &wakes);
        assert!(result.error.is_none(), "{:?}", result.error);
        assert_eq!(result.reply.unwrap().rc, 10);
        assert_eq!(std::fs::read_to_string(&outside).unwrap(), "keep");
    }
}

#[test]
fn application_replies_preserve_refusal_body_without_poisoning_next_request() {
    let (mut runtime, wakes) = runtime(4);
    runtime
        .load(program(
            r#"
$ctx=viu_context()
on refuse
  send "viu-local" "scenes.model" body={generation:$ctx.generation,value:{status:"Invalid name"}}
  reply(10,json_encode({error_code:"INVALID_NAME",details:{kept:true}}))
end
on inspect
  reply(0,json_encode({rows:["one","two"],ok:true,caller:$event.headers.from}))
end
on fault
  reply(0,"{}")
  raise("FAULT","after staged reply")
end
"#,
        ))
        .unwrap();
    assert!(next(&runtime, &wakes).error.is_none());
    runtime.event("refuse".into(), json!({})).unwrap();
    let refused = next(&runtime, &wakes);
    assert!(refused.error.is_none());
    assert!(refused.behaviour_available);
    assert_eq!(refused.model.unwrap(), json!({"status":"Invalid name"}));
    let reply = refused.reply.unwrap();
    assert_eq!(reply.rc, 10);
    assert_eq!(reply.body["error_code"], "INVALID_NAME");
    runtime
        .event_with_headers(
            "inspect".into(),
            json!({}),
            std::collections::BTreeMap::from([("from".into(), "caller-test".into())]),
        )
        .unwrap();
    let reply = next(&runtime, &wakes).reply.unwrap();
    assert_eq!(reply.rc, 0);
    assert_eq!(
        reply.body,
        json!({"rows":["one","two"],"ok":true,"caller":"caller-test"})
    );
    runtime.event("fault".into(), json!({})).unwrap();
    let fault = next(&runtime, &wakes);
    assert!(fault.error.is_some());
    assert!(fault.reply.is_none());
}

#[test]
fn real_evaluator_distinct_callbacks_wake_and_order() {
    let (mut runtime, wakes) = runtime(4);
    runtime.load(program(INIT)).unwrap();
    assert!(next(&runtime, &wakes).error.is_none());
    for _ in 0..4 {
        runtime.event("bump".into(), json!({})).unwrap();
    }
    for n in 1..=4 {
        let done = next(&runtime, &wakes);
        assert!(done.error.is_none(), "{:?}", done.error);
        assert_eq!(done.model, Some(json!({"n":n})));
    }
    assert!(
        wakes.recv_timeout(Duration::from_millis(40)).is_err(),
        "idle has no periodic UI wakes"
    );
}

#[test]
fn delayed_async_callback_bounds_mailbox_and_stale_generation() {
    let (mut runtime, wakes) = runtime(1);
    runtime
        .load(program("on slow async; sleep(0.15); reply(0, \"{}\"); end"))
        .unwrap();
    assert!(next(&runtime, &wakes).error.is_none());
    runtime.event("slow".into(), json!({})).unwrap();
    // Wait for the worker to receive this request before filling its one slot.
    std::thread::sleep(Duration::from_millis(30));
    let t = Instant::now();
    runtime.load(program(INIT)).unwrap();
    assert_eq!(runtime.event("bump".into(), json!({})), Err(Refusal::Full));
    assert!(
        !runtime.has_capacity(),
        "do not dequeue native requests into a full mailbox"
    );
    assert!(
        t.elapsed() < Duration::from_millis(40),
        "enqueue must not wait for behaviour"
    );
    let old = next(&runtime, &wakes);
    assert!(!runtime.accepts(&old));
    assert!(runtime.accepts(&next(&runtime, &wakes)));
    assert!(
        runtime.has_capacity(),
        "worker receive restores admission credit"
    );
}

#[test]
fn callback_fault_discards_partial_publish_and_reload_recovers() {
    let (mut runtime, wakes) = runtime(4);
    runtime.load(program("$ctx=viu_context(); on fail; send \"viu-local\" \"scenes.model\" body=json_encode({generation:$ctx.generation,value:{bad:true}}); raise(\"LAB\",\"expected\"); end")).unwrap();
    assert!(next(&runtime, &wakes).error.is_none());
    runtime.event("fail".into(), json!({})).unwrap();
    let done = next(&runtime, &wakes);
    assert!(done.error.is_some());
    assert!(done.model.is_none());
    runtime.load(program("$x = [")).unwrap();
    assert!(next(&runtime, &wakes).error.is_some());
    runtime.event("fail".into(), json!({})).unwrap();
    assert!(
        next(&runtime, &wakes)
            .error
            .unwrap()
            .starts_with("behaviour_unavailable:")
    );
    runtime.load(program(INIT)).unwrap();
    assert!(next(&runtime, &wakes).error.is_none());
    runtime.event("bump".into(), json!({})).unwrap();
    assert_eq!(next(&runtime, &wakes).model, Some(json!({"n":1})));
}

#[test]
fn drop_cancels_sleeping_behaviour_and_reaps_worker() {
    let (mut runtime, wakes) = runtime(1);
    runtime
        .load(program("on wait async; sleep(10); end"))
        .unwrap();
    assert!(next(&runtime, &wakes).error.is_none());
    runtime.event("wait".into(), json!({})).unwrap();
    std::thread::sleep(Duration::from_millis(20));
    let start = Instant::now();
    drop(runtime);
    assert!(start.elapsed() < Duration::from_secs(1));
}

#[test]
fn typed_and_legacy_bodies_preserve_nested_model_and_publication_snapshot() {
    for body in ["$payload", "json_encode($payload)"] {
        let source = format!(
            r#"$ctx=viu_context()
$payload={{generation:$ctx.generation,value:{{items:[{{id:"a",cells:["line\nnext","é"]}}], flag:true, empty:nil}}}}
send "viu-local" "scenes.model" body={body}
$payload.value.items[0].id="mutated after publication"
"#
        );
        let (mut runtime, wakes) = runtime(2);
        runtime.load(program(&source)).unwrap();
        let done = next(&runtime, &wakes);
        assert!(done.error.is_none(), "{:?}", done.error);
        assert_eq!(
            done.model,
            Some(json!({"items":[{"id":"a","cells":["line\nnext","é"]}],"flag":true,"empty":null}))
        );
    }
}

#[test]
fn application_error_text_cannot_retire_a_live_generation() {
    let (mut runtime, wakes) = runtime(2);
    runtime
        .load(program(
            &(INIT.to_owned() + "\non named; raise(\"APP\",\"upstream deadline exceeded\"); end"),
        ))
        .unwrap();
    assert!(next(&runtime, &wakes).error.is_none());
    runtime.event("named".into(), json!({})).unwrap();
    let failed = next(&runtime, &wakes);
    assert!(failed.error.unwrap().contains("deadline exceeded"));
    assert!(failed.behaviour_available);
    runtime.event("bump".into(), json!({})).unwrap();
    assert_eq!(next(&runtime, &wakes).model, Some(json!({"n":1})));
}
