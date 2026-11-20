// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;

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
    host.dispatch("test.legacy".into(),json!({}),BTreeMap::new(),None);
    settle(&mut host,&ctx);assert_eq!(host.model_revision,None);
    host.dispatch("test.full".into(),json!({"revision":1,"label":"must not reuse a retired revision"}),BTreeMap::new(),None);
    settle(&mut host,&ctx);assert!(host.error.contains("revision must advance"));assert_eq!(host.view.model()["label"],"legacy");
    host.dispatch("test.full".into(),json!({"revision":7,"label":"new base"}),BTreeMap::new(),None);
    settle(&mut host,&ctx);assert!(host.error.is_empty());assert_eq!(host.model_revision,Some((generation,7)));
}
