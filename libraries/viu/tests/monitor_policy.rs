// SPDX-License-Identifier: MIT OR Apache-2.0
use mix::{
    BusHandler, MixResult,
    evaluator::{BusFuture, IncomingEvent},
    value::Value as MixValue,
};
use serde_json::{Value, json};
use std::{
    path::PathBuf,
    rc::Rc,
    sync::{Arc, Mutex, mpsc},
    time::Duration,
};
use viu::{
    bundle::Bundle,
    runtime::{Completion, Program, Runtime, from_json, to_json},
};

fn done(runtime: &Runtime, rx: &mpsc::Receiver<()>) -> Completion {
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let result = runtime.try_completion().unwrap();
    assert!(
        result.error.is_none(),
        "{:?}\n{}\n{}",
        result.error,
        result.stdout,
        result.stderr
    );
    result
}

#[test]
fn monitor_mix_contracts_and_disabled_signal_paths_send_no_signals() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/monitor");
    for name in ["core", "wire", "history", "fetch", "policy"] {
        let (tx, rx) = mpsc::channel();
        let mut runtime = Runtime::new(
            8,
            Arc::new(move || {
                let _ = tx.send(());
            }),
            None,
        );
        let path = root.join(format!("{name}-test.mix"));
        runtime
            .load(Program {
                source: std::fs::read_to_string(&path).unwrap().replace(
                    "$fixture.create()",
                    &format!("$fixture.create({})", serde_json::to_string(&root).unwrap()),
                ),
                path,
                scene: "probe".into(),
                citizen: "probe".into(),
            })
            .unwrap();
        done(&runtime, &rx);
    }
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    // Build a copy with fake procfs using the same fixture helper as Mix tests.
    let fixture_guard = tempfile::tempdir().unwrap();
    runtime
        .load(Program {
            source: format!(
                "$fixture=require({});print($fixture.create({},{}))",
                serde_json::to_string(&root.join("../../libraries/viu/tests/monitor_procfs.mix"))
                    .unwrap(),
                serde_json::to_string(&root).unwrap(),
                serde_json::to_string(fixture_guard.path()).unwrap()
            ),
            path: root.join("fixture-generator.mix"),
            scene: "probe".into(),
            citizen: "probe".into(),
        })
        .unwrap();
    let fixture = PathBuf::from(done(&runtime, &rx).stdout.trim());
    let mut bundle = Bundle::load(&fixture.join("monitor"), &[]).unwrap();
    bundle.program.source.push_str(r#"
on fixture
  $state.selected=$event.args.pid
  $state.processes=[{pid:$event.args.pid,uid:$event.args.owner,name:"fixture",user:"fixture",cpu:0,memory:0}]
  publish_monitor($state)
end
"#);
    runtime.load(bundle.program).unwrap();
    done(&runtime, &rx);
    for column in ["name", "pid", "user", "cpu", "memory", "io"] {
        for tree in [false, true] {
            runtime
                .event("monitor.tree".into(), json!({"on":tree}))
                .unwrap();
            done(&runtime, &rx);
            for descending in [false, true] {
                runtime
                    .event(
                        "monitor.sort".into(),
                        json!({"column":column,"reverse":descending}),
                    )
                    .unwrap();
                let result = done(&runtime, &rx);
                assert_eq!(result.reply.unwrap().body["reverse"], descending);
                let table = result.model.unwrap()["table"]["initial"].clone();
                assert_eq!(table["ascending"], !descending);
                // Striped tables retain gold's reverse=true => up-arrow flag.
                assert_eq!(table["reverse"], !descending);
                runtime
                    .event(
                        "monitor.intent".into(),
                        json!({"id":"processes","action":"sort","value":{"column":column}}),
                    )
                    .unwrap();
                let table = done(&runtime, &rx).model.unwrap()["table"]["initial"].clone();
                assert_eq!(table["ascending"], descending);
                assert_eq!(table["reverse"], descending);
            }
        }
    }
    // Pure fixture ids only. Every path must refuse before the native primitive.
    for (pid, uid, owner) in [(1, 0, 0), (90, 0, 0), (80, 1000, 1000), (42, 1000, 1001)] {
        std::fs::write(
            fixture.join("proc/self/status"),
            format!("Uid: {uid} {uid}"),
        )
        .unwrap();
        std::fs::write(
            fixture.join(format!("proc/{pid}/status")),
            format!("Uid: {owner} {owner}"),
        )
        .unwrap();
        runtime
            .event("fixture".into(), json!({"pid":pid,"uid":uid,"owner":owner}))
            .unwrap();
        let model = done(&runtime, &rx).model.unwrap();
        assert_eq!(model["end_button"]["properties"]["enabled"], false);
        for (name, args) in [
            ("monitor.signal", json!({"pid":pid,"signal":"SIGTERM"})),
            ("monitor.signal", json!({"pid":pid,"signal":"SIGKILL"})),
            ("monitor.execute", json!({"id":"process.end"})),
            ("monitor.execute", json!({"id":"process.kill"})),
        ] {
            runtime.event(name.into(), args).unwrap();
            let reply = done(&runtime, &rx).reply.unwrap();
            assert_eq!(reply.rc, 10);
            assert_eq!(reply.body["error_code"], "DISABLED");
        }
    }
    // Names and advertised row uid are irrelevant: the unit suffix and ppid
    // come from fake procfs. No route reaches the native signal primitive.
    for unit in ["viud", "noded", "mixd", "shell-host"] {
        std::fs::write(
            fixture.join("proc/45/cgroup"),
            format!("0::/system.slice/{unit}@4.service\n"),
        )
        .unwrap();
        runtime
            .event("fixture".into(), json!({"pid":45,"uid":1000,"owner":1000}))
            .unwrap();
        let model = done(&runtime, &rx).model.unwrap();
        assert_eq!(model["end_button"]["properties"]["enabled"], false);
        assert!(
            model["end_button"]["properties"]["tooltip"]
                .as_str()
                .unwrap()
                .contains("desk.restart")
        );
        for (name, args) in [
            ("monitor.signal", json!({"pid":45,"signal":"SIGTERM"})),
            ("monitor.signal", json!({"pid":45,"signal":"SIGKILL"})),
            ("monitor.execute", json!({"id":"process.end"})),
            ("monitor.execute", json!({"id":"process.kill"})),
            ("monitor.intent", json!({"id":"end-task","origin":"ui"})),
            (
                "viu.frame.intent",
                json!({"value":{"command":"process.end","origin":"ui"}}),
            ),
            (
                "viu.frame.intent",
                json!({"value":{"command":"process.kill","origin":"bus"}}),
            ),
        ] {
            runtime.event(name.into(), args).unwrap();
            let reply = done(&runtime, &rx).reply.unwrap();
            assert_eq!(reply.rc, 10);
            assert_eq!(reply.body["error_code"], "PROTECTED");
            assert!(
                reply.body["recovery"]
                    .as_str()
                    .unwrap()
                    .contains("desk.restart")
            );
        }
    }
}

struct SuccessBus {
    calls: Arc<Mutex<Vec<(String, Value)>>>,
}
impl BusHandler for SuccessBus {
    fn send<'a>(
        &'a self,
        _: &'a str,
        command: &'a str,
        args: &'a MixValue,
    ) -> BusFuture<'a, MixResult<(i32, MixValue)>> {
        Box::pin(async move {
            let envelope = to_json(args)?;
            self.calls
                .lock()
                .unwrap()
                .push((command.into(), envelope["body"].clone()));
            let body = match command {
                "monitord.snapshot" => {
                    json!({"host":"fixture","cpu":{"busy":0,"cores":1},"memory":{"used":0,"total":1},"swap":{"used":0,"total":0}})
                }
                "monitord.processes" => {
                    json!({"total":1,"rows":[{"pid":42,"uid":1000,"name":"app","user":"user","cpu":0,"memory":0}]})
                }
                "monitord.history" => json!({"series":{}}),
                "native.executed" => json!({}),
                _ => panic!("unexpected effect {command}"),
            };
            Ok((0, from_json(body)?))
        })
    }
    fn emit<'a>(&'a self, _: &'a str, _: &'a str, _: &'a MixValue) -> BusFuture<'a, MixResult<()>> {
        Box::pin(async { Ok(()) })
    }
    fn port_exists<'a>(&'a self, _: &'a str) -> BusFuture<'a, MixResult<bool>> {
        Box::pin(async { Ok(true) })
    }
    fn next_incoming<'a>(&'a self) -> BusFuture<'a, Option<IncomingEvent>> {
        Box::pin(async { None })
    }
}

#[test]
fn monitor_success_routes_refresh_state_and_record_execution_without_real_signals() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/monitor");
    let fixture_guard = tempfile::tempdir().unwrap();
    let calls = Arc::new(Mutex::new(Vec::new()));
    let backend_calls = calls.clone();
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        Some(Box::new(move || {
            Rc::new(SuccessBus {
                calls: backend_calls,
            })
        })),
    );
    runtime
        .load(Program {
            source: format!(
                "$fixture=require({});print($fixture.create({},{}))",
                serde_json::to_string(&root.join("../../libraries/viu/tests/monitor_procfs.mix"))
                    .unwrap(),
                serde_json::to_string(&root).unwrap(),
                serde_json::to_string(fixture_guard.path()).unwrap()
            ),
            path: root.join("fixture-generator.mix"),
            scene: "probe".into(),
            citizen: "probe".into(),
        })
        .unwrap();
    done(&runtime, &rx);
    let mut bundle = Bundle::load(&fixture_guard.path().join("monitor"), &[]).unwrap();
    // Replace both primitive call sites in the test copy. The fake verifies
    // the token supplied by signal_monitor; it cannot send a kernel signal.
    assert_eq!(bundle.program.source.matches("viu_process(").count(), 2);
    bundle.program.source = bundle
        .program
        .source
        .replace("viu_process(", "fixture_process(");
    bundle.program.source.push_str(r#"
$fake_signals=[]
fn fixture_process($op,$args)
  global $fake_signals
  if $op=="snapshot" then return {"42":"fixture-token"} end
  if $op!="signal" or $args.pid!=42 or $args.start_time!="fixture-token" then raise("FAKE_SIGNAL","Unexpected signal request") end
  push($fake_signals,$args)
  return {ok:true}
end
on fixture
  $state.selected=42
  $state.processes=[{pid:42,uid:1000,name:"app",user:"user",cpu:0,memory:0}]
  $state.start_times={"42":"fixture-token"}
  $state.inflight={};$state.history_seeded=false;$state.serial=0
  $state.error="previous error";$fake_signals=[]
end
on fixture.signals
  reply(0,json_encode($fake_signals))
end
"#);
    runtime.load(bundle.program).unwrap();
    done(&runtime, &rx);
    for (name, args, command, signal, origin) in [
        (
            "monitor.signal",
            json!({"pid":42,"signal":"SIGTERM"}),
            None,
            "SIGTERM",
            "bus",
        ),
        (
            "monitor.execute",
            json!({"id":"process.end"}),
            Some("process.end"),
            "SIGTERM",
            "bus",
        ),
        (
            "monitor.execute",
            json!({"id":"process.kill"}),
            Some("process.kill"),
            "SIGKILL",
            "bus",
        ),
        (
            "monitor.intent",
            json!({"id":"end-task"}),
            Some("process.end"),
            "SIGTERM",
            "ui",
        ),
        (
            "viu.frame.intent",
            json!({"value":{"command":"process.kill","origin":"menu"}}),
            Some("process.kill"),
            "SIGKILL",
            "menu",
        ),
    ] {
        runtime.event("fixture".into(), json!({})).unwrap();
        done(&runtime, &rx);
        calls.lock().unwrap().clear();
        runtime.event(name.into(), args).unwrap();
        let result = done(&runtime, &rx);
        assert_eq!(result.reply.unwrap().rc, 0);
        assert_eq!(result.model.unwrap()["serial"], 1);
        runtime
            .event("monitor.debug.state".into(), json!({}))
            .unwrap();
        let state = done(&runtime, &rx).reply.unwrap().body;
        assert_eq!(
            state["status"],
            if signal == "SIGTERM" {
                "Asked to end"
            } else {
                "Killed"
            }
        );
        assert_eq!(state["serial"], 1);
        assert!(state["error"].is_null());
        assert_eq!(state["total"], 1);
        assert_eq!(state["start_times"]["42"], "fixture-token");
        assert!(state["history_seeded"].as_bool().unwrap());
        runtime.event("fixture.signals".into(), json!({})).unwrap();
        let signals = done(&runtime, &rx).reply.unwrap().body;
        assert_eq!(signals.as_array().unwrap().len(), 1);
        assert_eq!(signals[0]["signal"], signal);
        let effects = calls.lock().unwrap();
        for expected in [
            "monitord.snapshot",
            "monitord.processes",
            "monitord.history",
        ] {
            assert_eq!(
                effects.iter().filter(|(name, _)| name == expected).count(),
                1
            );
        }
        let recorded: Vec<_> = effects
            .iter()
            .filter(|(name, _)| name == "native.executed")
            .collect();
        if let Some(command) = command {
            assert_eq!(recorded.len(), 1);
            assert_eq!(recorded[0].1["id"], command);
            assert_eq!(recorded[0].1["origin"], origin);
            assert_eq!(recorded[0].1["params"], json!({"pid":42,"signal":signal}));
        } else {
            assert!(recorded.is_empty());
        }
    }
}
