// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{path::PathBuf,sync::{Arc,mpsc},time::Duration};
use viu::runtime::{Runtime,Program};
#[test]
fn enumeration_and_depth_refusals_leave_the_entire_previous_pane() {
    let tmp=tempfile::tempdir().unwrap();let folder=tmp.path().join("root");
    std::fs::create_dir_all(folder.join("child")).unwrap();
    std::fs::write(folder.join("child/leaf"),"fixture").unwrap();
    let source=format!(r#"
$dp=require("core.mix")
$pane=$dp.dp_pane({{path:{folder}}})
on probe
  $before=$pane
  $candidate=$pane
  $candidate.scan_budget=$event.args.budget
  if $event.args.expand then $candidate.expanded[{child}]=true end
  try
    $pane=$dp.dp_refresh($candidate)
    reply(0,json_encode({{changed:true}}))
  catch $message,$error
    reply(0,json_encode({{code:$error.code,unchanged:deep_eq($before,$pane)}}))
  end
end
"#,folder=json!(folder).to_string(),child=json!(folder.join("child")).to_string());
    let (tx,rx)=mpsc::channel();let mut r=Runtime::new(4,Arc::new(move ||{let _=tx.send(());}),None);
    r.load(Program {source,path:PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/dopus/budget-probe.mix"),scene:"budget".into(),citizen:"budget".into()}).unwrap();
    rx.recv_timeout(Duration::from_secs(20)).unwrap();assert!(r.try_completion().unwrap().error.is_none());
    for body in [json!({"budget":{"rows":1,"depth":64},"expand":false}),json!({"budget":{"rows":100,"depth":0},"expand":true})] {
        r.event("probe".into(),body).unwrap();rx.recv_timeout(Duration::from_secs(20)).unwrap();
        let c=r.try_completion().unwrap();assert!(c.error.is_none(),"{:?}",c.error);assert!(c.model.is_none());
        let answer=c.reply.unwrap();assert_eq!(answer.body["code"],"SCAN_BUDGET");assert_eq!(answer.body["unchanged"],true);
    }
    r.event("probe".into(),json!({"budget":{"rows":2,"depth":0},"expand":false})).unwrap();
    rx.recv_timeout(Duration::from_secs(20)).unwrap();let c=r.try_completion().unwrap();assert!(c.error.is_none());assert_eq!(c.reply.unwrap().body["changed"],true);
}
