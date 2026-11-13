// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{path::PathBuf, sync::{Arc, mpsc}, time::Duration};
use viu::{bundle::Bundle, runtime::{Completion, Program, Runtime}};

fn done(runtime: &Runtime, rx: &mpsc::Receiver<()>) -> Completion {
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let result = runtime.try_completion().unwrap();
    assert!(result.error.is_none(), "{:?}\n{}\n{}", result.error, result.stdout, result.stderr);
    result
}

#[test]
fn monitor_mix_contracts_and_disabled_signal_paths_send_no_signals() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/monitor");
    for name in ["core", "wire", "history", "fetch", "policy"] {
        let (tx, rx) = mpsc::channel();
        let mut runtime = Runtime::new(8, Arc::new(move || { let _ = tx.send(()); }), None);
        let path = root.join(format!("{name}-test.mix"));
        runtime.load(Program { source: std::fs::read_to_string(&path).unwrap(), path, scene: "probe".into(), citizen: "probe".into() }).unwrap();
        done(&runtime, &rx);
    }
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(8, Arc::new(move || { let _ = tx.send(()); }), None);
    let mut bundle = Bundle::load(&root, &[]).unwrap();
    bundle.program.source.push_str(r#"
on fixture
  $state.signal_context={uid:$event.args.uid,ancestors:[90,80,1]}
  $state.selected=$event.args.pid
  $state.processes=[{pid:$event.args.pid,uid:$event.args.owner,name:"fixture",user:"fixture",cpu:0,memory:0}]
  publish_monitor($state)
end
"#);
    runtime.load(bundle.program).unwrap();
    done(&runtime, &rx);
    // Pure fixture ids only. Every path must refuse before the native primitive.
    for (pid, uid, owner) in [(1,0,0), (90,0,0), (80,1000,1000), (42,1000,1001)] {
        runtime.event("fixture".into(), json!({"pid":pid,"uid":uid,"owner":owner})).unwrap();
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
}
