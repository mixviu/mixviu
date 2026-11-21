// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;

#[test]
fn publication_receipt_survives_a_full_evaluator_mailbox() {
    use std::sync::{Mutex, atomic::{AtomicBool, Ordering}, mpsc};
    struct Release(Option<mpsc::Sender<()>>);
    impl Drop for Release {
        fn drop(&mut self) { if let Some(tx) = self.0.take() { let _ = tx.send(()); } }
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
    let release = Release(Some(release_tx));
    let mut host = create(bundle, Arc::new(move || {
        if !blocked.swap(true, Ordering::SeqCst) {
            let _ = entered_tx.send(());
            let _ = release_rx.lock().unwrap().recv();
        }
    }));
    entered_rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let ctx = egui::Context::default();
    host.pump(&ctx);
    let generation = host.runtime.generation();
    let mut queued = 0;
    while host.runtime.notify("test.noop".into(), json!({})).is_ok() { queued += 1; }
    assert!(queued > 0);
    host.publication_result(generation, 2, Ok(()));
    assert_eq!(host.model_receipts.len(), 1, "full mailbox must retain its acceptance");
    drop(release);
    settle(&mut host, &ctx);
    assert!(host.model_receipts.is_empty());
    assert_eq!(host.view.model()["label"], "2", "retained acceptance must reach the evaluator");
}

fn settle(host: &mut Host, ctx: &egui::Context) {
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        host.pump(ctx);
        if host.ready && host.inflight.is_empty() {
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
    assert_eq!(
        host.view.model()["__viu_metrics"]["item_spacing_x"],
        json!(ctx.global_style().spacing.item_spacing.x)
    );
}
