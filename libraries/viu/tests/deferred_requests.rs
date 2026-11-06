// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{
    sync::{Arc, mpsc},
    time::Duration,
};
use viu::runtime::{Program, Runtime};

#[test]
fn deferred_callbacks_keep_the_runtime_responsive_and_faults_publish_no_partial_answers() {
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime.load(Program {path:"/tmp/deferred.mix".into(),scene:"test".into(),citizen:"test".into(),source:r#"
on begin
  send "viu-local" "requests.defer" token="owned"
  if $rc!=0 then raise("DEFER","deferral admission refused") end
end
on ping
  reply(0,"{\"alive\":true}")
end
on finish
  send "viu-local" "requests.reply" token="owned" rc=10 value={error_code:"REFUSED",message:"backend refusal"}
end
on fault
  send "viu-local" "requests.reply" token="owned" rc=0 value={}
  raise("FAIL","callback failed")
end
on invalid
  reply(0,"{}")
  send "viu-local" "requests.defer" token="double"
end
"#.into()}).unwrap();
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    assert!(runtime.try_completion().unwrap().error.is_none());
    for command in ["begin", "ping", "fault", "finish", "invalid"] {
        runtime.event(command.into(), json!({})).unwrap();
        rx.recv_timeout(Duration::from_secs(10)).unwrap();
        let done = runtime.try_completion().unwrap();
        match command {
            "begin" => {
                assert!(
                    done.error.is_none(),
                    "{:?} {} {}",
                    done.error,
                    done.stdout,
                    done.stderr
                );
                assert_eq!(done.deferred_token.as_deref(), Some("owned"));
                assert!(done.reply.is_none());
            }
            "ping" => assert_eq!(done.reply.unwrap().body["alive"], true),
            "fault" => {
                assert!(done.error.is_some());
                assert!(done.deferred_answers.is_empty());
            }
            "finish" => {
                assert!(done.error.is_none());
                assert_eq!(done.deferred_answers.len(), 1);
                assert_eq!(done.deferred_answers[0].reply.rc, 10);
                assert_eq!(done.deferred_answers[0].reply.body["error_code"], "REFUSED");
            }
            "invalid" => {
                assert!(done.error.is_some());
                assert!(done.reply.is_none());
                assert!(done.deferred_token.is_none());
            }
            _ => unreachable!(),
        }
    }
    runtime.notify("begin".into(), json!({})).unwrap();
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    assert!(runtime.try_completion().unwrap().error.is_some());
}
