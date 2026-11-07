// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Duration,
};
use viu::runtime::{Program, Runtime};
#[test]
fn dopus_bulk_projection_keeps_the_interpreted_display_policy() {
    let (tx, rx) = mpsc::channel();
    let mut r = Runtime::new(
        4,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    r.load(Program {
        source: r#"
$p=require("presentation.mix")
on probe
  $pane={entries:$event.args.rows,expanded:$event.args.expanded}
  $bulk=$p.dp_rows($pane)
  $reference=$p.dp_rows_interpreted($pane)
  reply(0,json_encode({same:deep_eq($bulk,$reference)}))
end
"#
        .into(),
        path: PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../apps/dopus/projection-probe.mix"),
        scene: "rows".into(),
        citizen: "rows".into(),
    })
    .unwrap();
    rx.recv_timeout(Duration::from_secs(20)).unwrap();
    assert!(r.try_completion().unwrap().error.is_none());
    let mut rows = vec![];
    for (i, name) in [
        "folder",
        "plain",
        "image.PNG",
        "a.tar.gz",
        "code.mix",
        "sound.flac",
        "video.webm",
        "report.pdf",
        ".txt",
    ]
    .iter()
    .enumerate()
    {
        for size in [0, 1023, 1024, 1048576, 1073741824] {
            rows.push(json!({"path":format!("/home/user/{name}"),"name":name,"dir":i==0,"file":i!=0,"symlink":false,"child_count":size,"size":size,"modified":1700000000,"depth":0}));
        }
    }
    for modified in [json!(1700000000), json!(null), json!(1e18), json!("bad")] {
        for row in &mut rows {
            row["modified"] = modified.clone();
        }
        r.event(
            "probe".into(),
            json!({"rows":rows,"expanded":{"/home/user/folder":true}}),
        )
        .unwrap();
        rx.recv_timeout(Duration::from_secs(20)).unwrap();
        let c = r.try_completion().unwrap();
        assert!(c.error.is_none(), "{:?}", c.error);
        assert_eq!(c.reply.unwrap().body["same"], true, "{modified}");
    }
    rows[0]["modified"] = json!(null);
    rows[1]["modified"] = json!("nil");
    r.event("probe".into(), json!({"rows":rows,"expanded":{}}))
        .unwrap();
    rx.recv_timeout(Duration::from_secs(20)).unwrap();
    let c = r.try_completion().unwrap();
    assert!(c.error.is_none(), "{:?}", c.error);
    assert_eq!(c.reply.unwrap().body["same"], true);
}
