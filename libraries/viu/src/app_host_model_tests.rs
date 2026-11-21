// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;

#[test]
fn malformed_native_theme_has_no_design_notice() {
    let bundle = review_bundle(r#"
send "viu-local" "native.theme" body={scheme:"spoofed"}
if $rc == 0 or $body.error_code != "ARGUMENT" then raise("TEST","malformed native choice must be ARGUMENT") end
send "viu-local" "scenes.model" body={generation:viu_context().generation,value:{label:"native argument refused"}}
on viu.model.applied
end
"#);
    let mut host = create(bundle, Arc::new(|| {}));
    let ctx = egui::Context::default();
    settle(&mut host, &ctx);
    assert!(host.error.is_empty(), "{}", host.error);
    assert_eq!(host.model["label"], "native argument refused");
    assert!(host.design_validation.notice.is_empty());
    assert_eq!(host.design_validation.info()["errors"], json!({}));
}

#[test]
fn design_refusals_are_typed_and_dismissal_clears_only_the_duplicate() {
    use crate::native_theme::Failure;
    let mut host = create(review_bundle("on test.noop\nend\n"), Arc::new(|| {}));
    host.surface_theme_failure(&Failure::Argument("bad axis".into()));
    assert_eq!(host.design_validation.info()["errors"], json!({}));
    assert!(host.design_validation.notice.is_empty());
    host.surface_theme_failure(&Failure::Adapter("session decode failed".into()));
    assert_eq!(
        host.error, "session decode failed",
        "pre-use base failure must be surfaced"
    );
    assert_eq!(host.design_validation.info()["errors"], json!({}));
    let digest = design::source_digest("failed source");
    let refusal = Failure::Design {
        digest: digest.clone(),
        message: "compile failed".into(),
    };
    assert_eq!(refusal.code(), "DESIGN_VALIDATION");
    host.surface_theme_failure(&refusal);
    assert_eq!(
        host.design_validation.info()["errors"][&digest],
        "compile failed"
    );
    host.error = host.design_validation.notice.clone();
    host.dismiss_design_notice();
    assert!(host.error.is_empty());
    assert!(host.design_validation.notice.is_empty());
    assert_eq!(
        host.design_validation.info()["errors"][&digest],
        "compile failed"
    );
    host.surface_theme_failure(&refusal);
    host.error = "unrelated error".into();
    host.dismiss_design_notice();
    assert_eq!(host.error, "unrelated error");
}

fn review_bundle(source: &str) -> Bundle {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/workshop");
    let mut bundle = Bundle::load(&root, &["--headless".into()]).unwrap();
    bundle.manifest.bus = false;
    bundle.manifest.require_bus = false;
    bundle.manifest.follow_settings = false;
    bundle.manifest.saved_version = None;
    bundle.manifest.frame = None;
    bundle.manifest.tick_ms = None;
    bundle.manifest.native_drive = false;
    bundle.scene="---\nscene: 1\nname: review\ncitizen: test\n---\n```mix\nroot: {widget: \"text\", text: \"= $model.label ?? \\\"data\\\"\"}\n```\n".into();
    bundle.program.source = source.into();
    bundle
}

#[test]
fn mix_differ_publishes_deep_models_and_long_keys_without_refusal() {
    let bundle = review_bundle(
        r#"
$ctx=viu_context();$pub=require("../../libraries/viu/mix/model-publication.mix")
$before=0;$after=1
for $i=1 to 40 $before={child:$before};$after={child:$after} end
$key="";for $i=1 to 150 $key=$key.."é" end
$nested={};$nested[$key]=true
$nested_model={nested:$nested};$changed=$nested_model;$changed.nested[$key]=false
$list_before={id:"leaf",value:0};$list_after={id:"leaf",value:1}
for $i=1 to 40 $list_before={id:to_string($i),rows:[$list_before]};$list_after={id:to_string($i),rows:[$list_after]} end
fn publish_review($old,$next,$base,$full,$depth)
  if $pub.requires_full($old,$next)!=$full then raise("TEST","unexpected full-publication decision") end
  if $full then
    send "viu-local" "scenes.model" body={generation:$ctx.generation,revision:$base+1,value:$next}
  else
    $ops=$pub.difference($old,$next)
    if len($ops)!=1 or len($ops[0].path)!=$depth then raise("TEST","path was not collapsed at the expected parent") end
    send "viu-local" "scenes.model.patch" body={generation:$ctx.generation,base:$base,revision:$base+1,operations:$ops}
  end
  if $rc!=0 then raise("TEST","publication send was refused") end
end
send "viu-local" "scenes.model" body={generation:$ctx.generation,revision:1,value:$before}
on test.deep
  publish_review($before,$after,1,false,8)
end
on test.long
  $next=$after;$next[$key]=true
  publish_review($after,$next,2,true,0)
end
on test.nested.seed
  send "viu-local" "scenes.model" body={generation:$ctx.generation,revision:4,value:$nested_model}
end
on test.nested.change
  publish_review($nested_model,$changed,4,false,1)
end
on test.nested.remove
  publish_review($changed,{nested:{}},5,false,1)
end
on test.nested.add
  publish_review({nested:{}},$nested_model,6,false,1)
end
on test.list.seed
  send "viu-local" "scenes.model" body={generation:$ctx.generation,revision:8,value:$list_before}
end
on test.list.change
  publish_review($list_before,$list_after,8,false,8)
end
on viu.model.applied
end
on viu.model.refused
  raise("TEST","publication was refused by the owner")
end
"#,
    );
    let mut host = create(bundle, Arc::new(|| {}));
    let ctx = egui::Context::default();
    settle(&mut host, &ctx);
    for (command, revision) in [
        ("test.deep", 2),
        ("test.long", 3),
        ("test.nested.seed", 4),
        ("test.nested.change", 5),
        ("test.nested.remove", 6),
        ("test.nested.add", 7),
        ("test.list.seed", 8),
        ("test.list.change", 9),
    ] {
        host.dispatch(command.into(), json!({}), BTreeMap::new(), None);
        settle(&mut host, &ctx);
        assert!(host.error.is_empty(), "{}", host.error);
        assert_eq!(host.model_revision.unwrap().1, revision);
        if revision == 3 {
            assert_eq!(host.model["é".repeat(150)], true);
        }
        if revision == 5 {
            assert_eq!(host.model["nested"]["é".repeat(150)], false);
        }
    }
    let mut leaf = &host.model;
    for _ in 0..40 {
        leaf = &leaf["rows"][0];
    }
    assert_eq!(leaf["value"], 1);
}

#[test]
fn receipt_limit_tracks_configured_runtime_width() {
    use std::sync::{Mutex, mpsc};
    struct Release(Option<mpsc::Sender<()>>);
    impl Drop for Release {
        fn drop(&mut self) {
            if let Some(tx) = self.0.take() {
                let _ = tx.send(());
            }
        }
    }
    let bundle = review_bundle(
        "send \"viu-local\" \"scenes.model\" body={generation:viu_context().generation,value:{label:\"ready\"}}\non test.noop\nend\non viu.model.applied\nend\n",
    );
    let program = bundle.program.clone();
    let mut host = create(bundle, Arc::new(|| {}));
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Mutex::new(Some(release_rx));
    host.runtime = Runtime::new(
        8,
        Arc::new(move || {
            if let Some(rx) = release_rx.lock().unwrap().take() {
                let _ = entered_tx.send(());
                let _ = rx.recv();
            }
        }),
        None,
    );
    let release = Release(Some(release_tx));
    host.runtime.load(program).unwrap();
    entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let generation = host.runtime.generation();
    while host.runtime.notify("test.noop".into(), json!({})).is_ok() {}
    for revision in 1..=8 {
        host.publication_result(generation, revision, Ok(()));
    }
    assert_eq!(host.model_receipts.len(), 8);
    let ctx = egui::Context::default();
    host.pump(&ctx);
    assert!(
        !host.ready,
        "width-eight receipt saturation must pause completion retirement"
    );
    assert_eq!(host.model_receipts.len(), 8);
    drop(release);
    settle(&mut host, &ctx);
    assert!(host.ready);
    assert!(host.model_receipts.is_empty());
}

#[cfg(feature = "test-support")]
const FLUSH_PUBLICATION_BYTE_BOUND: u64 = 1024 * 1024;
#[cfg(feature = "test-support")]
#[test]
fn flush_publication_is_measured_by_the_byte_guard() {
    let bundle = review_bundle(
        r#"
$g=viu_context().generation
send "viu-local" "scenes.model" body={generation:$g,revision:1,value:{label:"initial"}}
on viu.model.applied
  if $event.args.revision==1 then
    send "viu-local" "scenes.model.patch" body={generation:$g,base:1,revision:2,operations:[{op:"set",path:["label"],value:"flushed"}]}
  end
end
"#,
    );
    let mut host = test_support::Harness::new(bundle);
    host.probe_discovery();
    host.discovery_numeric_bounds(u64::MAX, FLUSH_PUBLICATION_BYTE_BOUND);
    let end = Instant::now() + Duration::from_secs(10);
    loop {
        host.pump();
        if host.state()["model"]["label"] == "flushed" {
            break;
        }
        assert!(Instant::now() < end);
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(host.discovery_measurements()["tick_model_bytes"]["n"], 1);
}

#[test]
fn publication_receipt_survives_a_full_evaluator_mailbox() {
    use std::sync::{
        Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    };
    struct Release(Option<mpsc::Sender<()>>);
    impl Drop for Release {
        fn drop(&mut self) {
            if let Some(tx) = self.0.take() {
                let _ = tx.send(());
            }
        }
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/workshop");
    let mut bundle = Bundle::load(&root, &["--headless".into()]).unwrap();
    bundle.manifest.bus = false;
    bundle.manifest.require_bus = false;
    bundle.manifest.follow_settings = false;
    bundle.manifest.saved_version = None;
    bundle.manifest.frame = None;
    bundle.manifest.tick_ms = None;
    bundle.manifest.native_drive = false;
    bundle.scene="---\nscene: 1\nname: publication\ncitizen: test\n---\n```mix\nroot: {widget: \"text\", text: \"= $model.label\"}\n```\n".into();
    bundle.program.source=r#"
$ctx=viu_context()
send "viu-local" "scenes.model" body={generation:$ctx.generation,revision:1,value:{label:"initial"}}
on test.noop
end
on viu.model.applied
  send "viu-local" "scenes.model" body={generation:$ctx.generation,value:{label:to_string($event.args.revision)}}
end
"#.into();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let release_rx = Mutex::new(release_rx);
    let blocked = AtomicBool::new(false);
    let mut host = create(
        bundle,
        Arc::new(move || {
            if !blocked.swap(true, Ordering::SeqCst) {
                let _ = entered_tx.send(());
                let _ = release_rx.lock().unwrap().recv();
            }
        }),
    );
    let release = Release(Some(release_tx));
    entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let ctx = egui::Context::default();
    host.pump(&ctx);
    let generation = host.runtime.generation();
    let mut queued = 0;
    while host.runtime.notify("test.noop".into(), json!({})).is_ok() {
        queued += 1;
    }
    assert!(queued > 0);
    host.publication_result(generation, 2, Ok(()));
    assert_eq!(
        host.model_receipts.len(),
        1,
        "full mailbox must retain its acceptance"
    );
    drop(release);
    settle(&mut host, &ctx);
    assert!(host.model_receipts.is_empty());
    assert_eq!(
        host.view.model()["label"],
        "2",
        "retained acceptance must reach the evaluator"
    );
}

fn settle(host: &mut Host, ctx: &egui::Context) {
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        host.pump(ctx);
        if host.ready && host.inflight.is_empty() && host.model_receipts.is_empty() {
            break;
        }
        assert!(
            Instant::now() < end,
            "host failed to settle: {}",
            host.error
        );
        std::thread::sleep(Duration::from_millis(1));
    }
}

#[test]
fn native_publication_fences_rejection_callback_failure_and_full_recovery() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/workshop");
    let mut bundle = Bundle::load(&root, &["--headless".into()]).unwrap();
    bundle.manifest.bus = false;
    bundle.manifest.require_bus = false;
    bundle.manifest.follow_settings = false;
    bundle.manifest.saved_version = None;
    bundle.manifest.frame = None;
    bundle.manifest.tick_ms = None;
    bundle.manifest.native_drive = false;
    bundle.scene="---\nscene: 1\nname: publication\ncitizen: test\n---\n```mix\nroot: {widget: \"text\", text: \"= $model.label ?? \\\"waiting\\\"\"}\n```\n".into();
    bundle.program.source=r#"
$ctx=viu_context()
send "viu-local" "scenes.model" body={generation:$ctx.generation,revision:1,value:{label:"initial"}}
on test.patch
  $a=$event.args
  send "viu-local" "scenes.model.patch" body={generation:$a.generation ?? $ctx.generation,base:$a.base,revision:$a.revision,operations:[{op:"set",path:["label"],value:$a.label}]}
end
on test.full
  send "viu-local" "scenes.model" body={generation:$ctx.generation,revision:$event.args.revision,value:{label:$event.args.label}}
end
on test.legacy
  send "viu-local" "scenes.model" body={generation:$ctx.generation,value:{label:"legacy"}}
end
on test.fail
  send "viu-local" "scenes.model.patch" body={generation:$ctx.generation,base:$event.args.base,revision:$event.args.revision,operations:[{op:"set",path:["label"],value:"must not escape failed callback"}]}
  raise("TEST","callback failed after publication")
end
on test.twice
  $body={generation:$ctx.generation,base:$event.args.base,revision:$event.args.revision,operations:[{op:"set",path:["label"],value:"must not escape mixed callback"}]}
  send "viu-local" "scenes.model.patch" body=$body
  send "viu-local" "scenes.model.patch" body=$body
end
on viu.model.applied
end
on viu.model.refused
end
"#.into();
    let mut host = create(bundle, Arc::new(|| {}));
    let ctx = egui::Context::default();
    settle(&mut host, &ctx);
    let generation = host.model_revision.unwrap().0;
    assert_eq!(host.model_revision, Some((generation, 1)));
    host.dispatch(
        "test.patch".into(),
        json!({"base":1,"revision":2,"label":"changed"}),
        BTreeMap::new(),
        None,
    );
    settle(&mut host, &ctx);
    assert!(host.error.is_empty(), "{}", host.error);
    assert_eq!(host.model_revision, Some((generation, 2)));
    assert_eq!(host.view.model()["label"], "changed");
    let revision = host.view.revision;
    host.dispatch(
        "test.patch".into(),
        json!({"base":1,"revision":2,"label":"stale"}),
        BTreeMap::new(),
        None,
    );
    settle(&mut host, &ctx);
    assert!(host.error.contains("stale model base"));
    assert_eq!(host.model_revision, Some((generation, 2)));
    assert_eq!(host.view.revision, revision);
    assert_eq!(host.view.model()["label"], "changed");
    host.dispatch(
        "test.patch".into(),
        json!({"base":2,"revision":3,"label":"must wait for resync"}),
        BTreeMap::new(),
        None,
    );
    settle(&mut host, &ctx);
    assert!(host.error.contains("requires full resynchronisation"));
    assert_eq!(host.model_revision, Some((generation, 2)));
    assert_eq!(host.view.model()["label"], "changed");
    host.dispatch(
        "test.full".into(),
        json!({"revision":4,"label":"resynchronised"}),
        BTreeMap::new(),
        None,
    );
    settle(&mut host, &ctx);
    assert_eq!(host.model_revision, Some((generation, 4)));
    assert!(host.error.is_empty());
    for command in ["test.fail", "test.twice"] {
        host.dispatch(
            command.into(),
            json!({"base":4,"revision":5}),
            BTreeMap::new(),
            None,
        );
        settle(&mut host, &ctx);
        assert_eq!(host.model_revision, Some((generation, 4)));
        assert_eq!(host.view.model()["label"], "resynchronised");
    }
    host.dispatch(
        "test.full".into(),
        json!({"revision":6,"label":"recovered"}),
        BTreeMap::new(),
        None,
    );
    settle(&mut host, &ctx);
    assert_eq!(host.model_revision, Some((generation, 6)));
    assert!(host.error.is_empty());
    host.dispatch(
        "test.patch".into(),
        json!({"generation":generation+1,"base":6,"revision":7,"label":"wrong generation"}),
        BTreeMap::new(),
        None,
    );
    settle(&mut host, &ctx);
    assert_eq!(host.model_revision, Some((generation, 6)));
    assert_eq!(host.view.model()["label"], "recovered");
    host.dispatch("test.legacy".into(), json!({}), BTreeMap::new(), None);
    settle(&mut host, &ctx);
    assert_eq!(host.model_revision, None);
    host.dispatch(
        "test.full".into(),
        json!({"revision":1,"label":"must not reuse a retired revision"}),
        BTreeMap::new(),
        None,
    );
    settle(&mut host, &ctx);
    assert!(host.error.contains("revision must advance"));
    assert_eq!(host.view.model()["label"], "legacy");
    assert_eq!(host.model_refused, Some(generation));
    host.dispatch("test.legacy".into(), json!({}), BTreeMap::new(), None);
    settle(&mut host, &ctx);
    assert_eq!(host.model_refused, None);
    assert_eq!(host.model_revision, None);
    host.dispatch(
        "test.full".into(),
        json!({"revision":7,"label":"new base"}),
        BTreeMap::new(),
        None,
    );
    settle(&mut host, &ctx);
    assert!(host.error.is_empty());
    assert_eq!(host.model_revision, Some((generation, 7)));
    let mut style = (*ctx.global_style()).clone();
    style.spacing.item_spacing.x += 1.0;
    ctx.set_global_style(style);
    host.dispatch(
        "test.patch".into(),
        json!({"base":7,"revision":8,"label":"must not retain old host metrics"}),
        BTreeMap::new(),
        None,
    );
    settle(&mut host, &ctx);
    assert!(host.error.contains("host projection changed"));
    assert_eq!(host.model_revision, Some((generation, 7)));
    assert_eq!(host.view.model()["label"], "new base");
    host.dispatch(
        "test.full".into(),
        json!({"revision":9,"label":"new host projection"}),
        BTreeMap::new(),
        None,
    );
    settle(&mut host, &ctx);
    assert!(host.error.is_empty());
    assert_eq!(host.model_revision, Some((generation, 9)));
    host.dispatch(
        "test.full".into(),
        json!({"revision":crate::model_patch::MAX_REVISION,"label":"terminal"}),
        BTreeMap::new(),
        None,
    );
    settle(&mut host, &ctx);
    host.dispatch(
        "test.full".into(),
        json!({"revision":crate::model_patch::MAX_REVISION,"label":"refused"}),
        BTreeMap::new(),
        None,
    );
    settle(&mut host, &ctx);
    assert!(host.error.contains("revision must advance"));
    assert_eq!(host.view.model()["label"], "terminal");
    host.dispatch("test.legacy".into(), json!({}), BTreeMap::new(), None);
    settle(&mut host, &ctx);
    assert!(host.error.is_empty());
    assert_eq!(host.model_refused, None);
    assert_eq!(
        host.view.model()["__viu_metrics"]["item_spacing_x"],
        json!(ctx.global_style().spacing.item_spacing.x)
    );
}
