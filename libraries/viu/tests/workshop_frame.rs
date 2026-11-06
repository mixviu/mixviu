// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Duration,
};
use viu::{bundle::Bundle, runtime::Runtime};

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
    let mut view = viu::gold_frame::mount(&bundle.scene, model).unwrap();
    let ctx = egui::Context::default();
    viu::appearance::install(&ctx);
    viu::icons::install(&ctx);
    for _ in 0..2 {
        ctx.run_ui(
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
        )
        .drop_without_applying_deltas();
    }
    assert!(view.geometry.contains_key("gold-menu"));
    assert!(view.geometry.contains_key("gold-status-text"));
    assert!(view.geometry.contains_key("services-tree.key.nodes"));
    assert!(view.geometry.contains_key("body"));
    let large = viu::widgets::Widget::new("reply", "editor", "", json!("🦀".repeat(250_000)))
        .with_properties(json!({"read_only":true,"max_bytes":1_000_100}));
    assert!(large.validate(&large.initial).is_ok());
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
fn macro_library_binding_keeps_disk_work_inside_the_owned_session_roots() {
    let dir=tempfile::tempdir().unwrap();
    let roots=macros::Roots {builtin:dir.path().join("builtin"),user:dir.path().join("user"),home:dir.path().join("home"),mix:"/opt/mixos/bin/mix".into()};
    let session=macros::Session::with_roots("workshop",Some(roots.clone()));
    let (tx,rx)=mpsc::channel();
    let mut runtime=Runtime::with_services(8,Arc::new(move || {let _=tx.send(());}),None,viu::runtime::Services {macros:Some(session),chooser:None});
    runtime.load(viu::runtime::Program {
        path:dir.path().join("library-test.mix"),
        source:r#"
$global=viu_macro_storage("global","new",{title:"Global example"})
$app=viu_macro_storage("email","new",{title:"App example"})
$catalogues=viu_macro_library()
reply(0,json_encode({global:$global,app:$app,catalogues:$catalogues}))
"#.into(),scene:"library-test".into(),citizen:"library-test".into(),
    }).unwrap();
    rx.recv_timeout(Duration::from_secs(20)).unwrap();
    let done=runtime.try_completion().unwrap();
    assert!(done.error.is_none(),"{:?}",done.error);
    let reply=done.reply.unwrap();
    assert_eq!(reply.rc,0);
    assert_eq!(reply.body["global"]["rc"],0);
    assert_eq!(reply.body["app"]["rc"],0);
    assert!(std::path::Path::new(reply.body["global"]["body"]["path"].as_str().unwrap()).starts_with(&roots.user));
    assert!(std::path::Path::new(reply.body["app"]["body"]["path"].as_str().unwrap()).starts_with(roots.user.join("email")));
    assert_eq!(reply.body["catalogues"]["email"]["running"],false);
    assert!(reply.body["catalogues"]["global"]["macros"].as_array().unwrap().iter().any(|m|m["id"]=="global-example"));
}
