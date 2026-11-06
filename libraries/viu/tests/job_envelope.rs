// SPDX-License-Identifier: MIT OR Apache-2.0
use std::{
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};
#[test]
fn explicit_body_job_envelopes_keep_the_typed_payload_and_owner() {
    let dir = tempfile::tempdir().unwrap();
    let (tx, rx) = mpsc::channel();
    let wake: viu::runtime::Wake = Arc::new(move || {
        let _ = tx.send(());
    });
    let mut jobs = viu::jobs::Jobs::new(2, wake.clone());
    let mut runtime = viu::runtime::Runtime::new(4, wake, Some(jobs.backend(None)));
    let spec = serde_json::json!({"id":"example","owner":4,"path":dir.path().join("worker.mix"),"source":"on job.run;reply(0,json_encode($event.args));end","command":"job.run","args":{"value":17},"timeout":180});
    runtime.load(viu::runtime::Program{path:dir.path().join("root.mix"),scene:"jobs".into(),citizen:"jobs".into(),source:format!("$spec=json_parse({});send \"viu-local\" \"jobs.start\" body=$spec;if $rc!=0 then raise(\"ADMISSION\",json_encode($reply)) end",serde_json::to_string(&spec.to_string()).unwrap())}).unwrap();
    let until = Instant::now() + Duration::from_secs(5);
    let mut admitted = false;
    let mut completed = false;
    while Instant::now() < until && !(admitted && completed) {
        for result in jobs.poll() {
            assert_eq!(result.tag, serde_json::json!({"id":"example","owner":4}));
            assert_eq!(result.rc, 0);
            assert_eq!(result.body, serde_json::json!({"value":17}));
            completed = true;
        }
        if let Some(result) = runtime.try_completion() {
            assert!(result.error.is_none(), "{:?}", result.error);
            admitted = true;
        }
        let _ = rx.recv_timeout(Duration::from_millis(10));
    }
    assert!(admitted && completed);
}
