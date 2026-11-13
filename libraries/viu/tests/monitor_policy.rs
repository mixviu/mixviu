// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Duration,
};
use viu::{
    bundle::Bundle,
    runtime::{Completion, Program, Runtime},
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
                source: std::fs::read_to_string(&path).unwrap(),
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
    runtime.load(Program {
        source: format!("$fixture=require({});print($fixture.create())",
            serde_json::to_string(&root.join("test-procfs.mix")).unwrap()),
        path: root.join("fixture-generator.mix"),
        scene: "probe".into(), citizen: "probe".into(),
    }).unwrap();
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
        std::fs::write(fixture.join("proc/self/status"), format!("Uid: {uid} {uid}")).unwrap();
        std::fs::write(fixture.join(format!("proc/{pid}/status")), format!("Uid: {owner} {owner}")).unwrap();
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
        std::fs::write(fixture.join("proc/45/cgroup"),
            format!("0::/system.slice/{unit}@4.service\n")).unwrap();
        runtime.event("fixture".into(), json!({"pid":45,"uid":1000,"owner":1000})).unwrap();
        let model = done(&runtime, &rx).model.unwrap();
        assert_eq!(model["end_button"]["properties"]["enabled"], false);
        assert!(model["end_button"]["properties"]["tooltip"].as_str().unwrap().contains("desk.restart"));
        for (name, args) in [
            ("monitor.signal", json!({"pid":45,"signal":"SIGTERM"})),
            ("monitor.signal", json!({"pid":45,"signal":"SIGKILL"})),
            ("monitor.execute", json!({"id":"process.end"})),
            ("monitor.execute", json!({"id":"process.kill"})),
            ("monitor.intent", json!({"id":"end-task","origin":"ui"})),
            ("viu.frame.intent", json!({"value":{"command":"process.end","origin":"ui"}})),
            ("viu.frame.intent", json!({"value":{"command":"process.kill","origin":"bus"}})),
        ] {
            runtime.event(name.into(), args).unwrap();
            let reply = done(&runtime, &rx).reply.unwrap();
            assert_eq!(reply.rc, 10);
            assert_eq!(reply.body["error_code"], "PROTECTED");
            assert!(reply.body["recovery"].as_str().unwrap().contains("desk.restart"));
        }
    }
    std::fs::remove_dir_all(fixture).unwrap();
}
