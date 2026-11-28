// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Duration,
};
use viu::{bundle::Bundle, runtime::Runtime};

#[test]
fn traffic_redaction_job_returns_decoded_map_rows_through_native_jobs() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/workshop");
    let wake: viu::runtime::Wake = Arc::new(|| {});
    let mut jobs = viu::jobs::Jobs::new(2, wake.clone());
    let mut runtime = Runtime::new(8, wake, Some(jobs.backend(None)));
    runtime.load(viu::runtime::Program {
        path: root.join("app.mix"),
        scene: "traffic-job-contract".into(),
        citizen: "traffic-job-contract".into(),
        source: r#"
$traffic=require("traffic.mix");$model=require("model.mix");$controller=require("controller.mix");$work=require("work.mix")
$state=$model.initial()
for $seq=1 to 2
  $state=$traffic.defer_row($state,{seq:$seq,payload:{token:"hidden",nested:[{password:"hidden",public:$seq}]}},false)
end
$state=$work.launch($traffic.launch($state))
on viu.job.completed
  $state=$controller.received($state,$event.args)
  send "viu-local" "scenes.model" body={generation:viu_context().generation,value:{rows:$state.traffic}}
end
"#.into(),
    }).unwrap();
    let until = std::time::Instant::now() + Duration::from_secs(60);
    loop {
        for result in jobs.poll() {
            assert!(
                result.tag["owner"].is_string(),
                "Traffic Job must carry reset generation in native owner tag"
            );
            runtime
                .notify(
                    "viu.job.completed".into(),
                    json!({"tag":result.tag,"rc":result.rc,"body":result.body}),
                )
                .unwrap();
        }
        if let Some(done) = runtime.try_completion() {
            assert!(done.error.is_none(), "{:?}", done.error);
            if let Some(model) = done.model {
                let rows = model["rows"].as_array().expect("Traffic ring is a list");
                assert_eq!(
                    rows.len(),
                    2,
                    "real redaction Job must deliver decoded map rows"
                );
                for (i, row) in rows.iter().enumerate() {
                    assert!(row.is_object());
                    assert_eq!(
                        row["seq"],
                        i + 1,
                        "native redaction batch must preserve FIFO"
                    );
                    assert_eq!(row["payload"]["token"], "[redacted]");
                    assert_eq!(row["payload"]["nested"][0]["password"], "[redacted]");
                    assert_eq!(row["payload"]["nested"][0]["public"], i + 1);
                }
                break;
            }
        }
        assert!(
            std::time::Instant::now() < until,
            "native redaction Job stalled"
        );
        std::thread::sleep(Duration::from_millis(2));
    }
}

#[test]
fn accepted_publication_immediately_flushes_coalesced_input() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/workshop");
    let bundle = Bundle::load(&root, &[]).unwrap();
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::with_services(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
        viu::runtime::Services {
            incremental_models: true,
            ..Default::default()
        },
    );
    runtime.load(bundle.program).unwrap();
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let first = runtime.try_completion().unwrap();
    assert!(first.error.is_none(), "{:?}", first.error);
    assert_eq!(first.model_revision, Some(1));
    runtime
        .event("workshop.filter".into(), json!({"text":"queued input"}))
        .unwrap();
    rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let pending = runtime.try_completion().unwrap();
    assert!(pending.error.is_none(), "{:?}", pending.error);
    assert_eq!(pending.model_revision, None);
    runtime
        .notify(
            "viu.model.applied".into(),
            json!({"generation":first.generation,"revision":1}),
        )
        .unwrap();
    rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let flushed = runtime.try_completion().unwrap();
    assert!(flushed.error.is_none(), "{:?}", flushed.error);
    assert_eq!(
        flushed.model_revision,
        Some(2),
        "coalesced input must not wait for another tick"
    );
    runtime
        .notify(
            "viu.model.applied".into(),
            json!({"generation":first.generation,"revision":2}),
        )
        .unwrap();
    rx.recv_timeout(Duration::from_secs(5)).unwrap();
    let quiet = runtime.try_completion().unwrap();
    assert!(quiet.error.is_none(), "{:?}", quiet.error);
    assert_eq!(
        quiet.model_revision, None,
        "acceptance must not start an idle publication loop"
    );
}

#[test]
fn workshop_topic_bursts_project_once_and_idle_ticks_do_not_project() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/workshop");
    let bundle = Bundle::load(&root, &[]).unwrap();
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime.load(bundle.program).unwrap();
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    assert!(runtime.try_completion().unwrap().model.is_some());
    for command in ["noded.props.changed", "agent.session.updated"]
        .into_iter()
        .cycle()
        .take(20)
    {
        runtime
            .notify(
                "workshop.notice".into(),
                json!({"command":command,"args":{}}),
            )
            .unwrap();
        rx.recv_timeout(Duration::from_secs(5)).unwrap();
        let done = runtime.try_completion().unwrap();
        assert!(done.error.is_none(), "{:?}", done.error);
        assert!(
            done.model.is_none(),
            "topic {command} projected before tick"
        );
    }
    for dirty in [true, false, false] {
        runtime.notify("viu.tick".into(), json!({})).unwrap();
        rx.recv_timeout(Duration::from_secs(5)).unwrap();
        let done = runtime.try_completion().unwrap();
        assert!(done.error.is_none(), "{:?}", done.error);
        assert_eq!(done.model.is_some(), dirty, "idle ticks must stay idle");
    }
}

#[test]
fn workshop_mounts_gold_frame_and_routes_dialogs_without_bus_transport() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/workshop");
    let bundle = Bundle::load(&root, &[]).unwrap();
    assert_eq!(bundle.manifest.frame.as_deref(), Some("gold"));
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime.load(bundle.program).unwrap();
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let done = runtime.try_completion().unwrap();
    assert!(done.error.is_none(), "{:?}", done.error);
    let model = done.model.unwrap();
    assert_eq!(model["values"]["gold-title"], "Workshop");
    assert_eq!(model["values"]["gold-icon"], "server");
    assert!(model["theme_choice"]["mode"].is_null());
    viu::appearance::resolve(&model["appearance"]).unwrap();
    let mut view = viu::gold_frame::mount(&bundle.scene, model).unwrap();
    let ctx = egui::Context::default();
    viu::appearance::install(&ctx);
    viu::icons::install(&ctx);
    for frame in 0..24 {
        let output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1040., 680.),
                )),
                ..Default::default()
            },
            |ui| {
                view.show(ui);
            },
        );
        if frame == 23 {
            assert!(
                !output.platform_output.requested_discard(),
                "{:?}",
                output.platform_output.request_discard_reasons
            );
        }
        output.drop_without_applying_deltas();
    }
    assert!(view.geometry.contains_key("gold-menu"));
    assert!(view.geometry.contains_key("gold-status-text"));
    assert!(view.geometry.contains_key("services-tree.key.nodes"));
    assert!(view.geometry.contains_key("body"));
    let large = viu::widgets::Widget::new("reply", "editor", "", json!("🦀".repeat(250_000)))
        .with_properties(json!({"read_only":true,"max_bytes":1_000_100}));
    assert!(large.validate(&large.initial).is_ok());
    assert!(large.validate(&json!("🦀".repeat(250_026))).is_err());
    let editable = large
        .clone()
        .with_properties(json!({"read_only":false,"max_bytes":1_000_100}));
    assert!(editable.validate(&editable.initial).is_err());
    runtime
        .event("workshop.dialog".into(), json!({"open":"about"}))
        .unwrap();
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let done = runtime.try_completion().unwrap();
    assert!(done.error.is_none(), "{:?}", done.error);
    assert_eq!(done.reply.unwrap().rc, 0);
    let model = done.model.unwrap();
    assert_eq!(
        model["native_overlays"][0]["definition"]["initial"]["title"],
        "About Workshop"
    );
    runtime.event("viu.frame.close".into(), json!({})).unwrap();
    rx.recv_timeout(Duration::from_secs(10)).unwrap();
    let done = runtime.try_completion().unwrap();
    assert!(done.error.is_none(), "{:?}", done.error);
    assert_eq!(done.model.unwrap()["window_close"], true);
}

#[test]
fn workshop_embedded_text_budget_retains_gold_sized_utf8_replies() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/workshop");
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime
        .load(viu::runtime::Program {
            path: root.join("text-test.mix"),
            source: std::fs::read_to_string(root.join("text-test.mix")).unwrap(),
            scene: "workshop-text".into(),
            citizen: "workshop-text".into(),
        })
        .unwrap();
    rx.recv_timeout(Duration::from_secs(20)).unwrap();
    let done = runtime.try_completion().unwrap();
    assert!(done.error.is_none(), "{:?}", done.error);
    assert!(done.stdout.contains("PASS"));
}

#[test]
fn workshop_policy_contracts_run_inside_the_embedded_runtime() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/workshop");
    for name in [
        "tree",
        "agents",
        "traffic",
        "transcript",
        "counter",
        "mesh",
        "view",
        "saved",
        "discovery",
        "publication",
    ] {
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
            .load(viu::runtime::Program {
                source: std::fs::read_to_string(&path).unwrap(),
                path,
                scene: format!("workshop-{name}"),
                citizen: format!("workshop-{name}"),
            })
            .unwrap();
        rx.recv_timeout(Duration::from_secs(20)).unwrap();
        let done = runtime.try_completion().unwrap();
        assert!(done.error.is_none(), "{name}: {:?}", done.error);
        assert!(done.stdout.contains("PASS"), "{name}: {}", done.stdout);
    }
}

#[test]
fn macro_library_binding_keeps_disk_work_inside_the_owned_session_roots() {
    let dir = tempfile::tempdir().unwrap();
    let roots = macros::Roots {
        builtin: dir.path().join("builtin"),
        user: dir.path().join("user"),
        home: dir.path().join("home"),
        mix: "/opt/mix/bin/mix".into(),
    };
    let session = macros::Session::with_roots("workshop", Some(roots.clone()));
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::with_services(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
        viu::runtime::Services {
            macros: Some(session),
            chooser: None,
            ..Default::default()
        },
    );
    runtime.load(viu::runtime::Program {
        path:dir.path().join("library-test.mix"),
        source:r#"
$global=viu_macro_storage("global","new",{title:"Global example"})
$app=viu_macro_storage("email","new",{title:"App example"})
$own=viu_macros("new",{title:"Own example"})
$catalogues=viu_macro_library()
send "viu-local" "scenes.model" body={generation:viu_context().generation,value:{global:$global,app:$app,own:$own,catalogues:$catalogues}}
"#.into(),scene:"library-test".into(),citizen:"library-test".into(),
    }).unwrap();
    rx.recv_timeout(Duration::from_secs(20)).unwrap();
    let done = runtime.try_completion().unwrap();
    assert!(done.error.is_none(), "{:?}", done.error);
    let model = done.model.unwrap();
    assert_eq!(model["global"]["rc"], 0);
    assert_eq!(model["app"]["rc"], 0);
    assert_eq!(model["own"]["rc"], 0);
    // Gold reserves Workshop's own endpoint for the global catalogue.
    assert_eq!(
        model["own"]["body"]["path"],
        json!(roots.user.join("own-example.mix"))
    );
    assert!(!roots.user.join("workshop").exists());
    assert!(
        std::path::Path::new(model["global"]["body"]["path"].as_str().unwrap())
            .starts_with(&roots.user)
    );
    assert!(
        std::path::Path::new(model["app"]["body"]["path"].as_str().unwrap())
            .starts_with(roots.user.join("email"))
    );
    assert_eq!(model["catalogues"]["email"]["running"], false);
    assert!(
        model["catalogues"]["global"]["macros"]
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["id"] == "global-example")
    );
}
