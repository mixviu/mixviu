// SPDX-License-Identifier: MIT OR Apache-2.0
//! Dopus uses the generic runtime and frame, with no file-manager Rust host.
use serde_json::json;
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Duration,
};
use viu::runtime::{Completion, Runtime};

fn done(runtime: &Runtime, wakes: &mpsc::Receiver<()>) -> Completion {
    wakes.recv_timeout(Duration::from_secs(30)).unwrap();
    let result = runtime.try_completion().unwrap();
    assert!(result.error.is_none(), "{:?}", result.error);
    result
}

#[test]
fn chooser_mix_mounts_shared_frame_and_keeps_native_save_focus_and_rows() {
    let fixture = tempfile::tempdir().unwrap();
    std::fs::write(fixture.path().join("notes.txt"), "fixture").unwrap();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/dopus");
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
            source: std::fs::read_to_string(root.join("chooser.mix")).unwrap(),
            path: root.join("chooser.mix"),
            scene: "chooser".into(),
            citizen: "files-window".into(),
        })
        .unwrap();
    done(&runtime, &rx);
    runtime.event("chooser.open".into(),json!({"id":"request-test","caller":"local:test","home":fixture.path(),"profile":fixture.path(),"request":{"mode":"save","title":"Save file","accept_label":"Save","current_folder":fixture.path(),"current_name":"draft.txt","choices":[]}})).unwrap();
    let completion = done(&runtime, &rx);
    assert_eq!(completion.reply.unwrap().rc, 0);
    let model = completion.model.unwrap();
    assert_eq!(
        model["values"]["files"]["rows"].as_array().unwrap().len(),
        1
    );
    assert_eq!(model["focus"], json!({"id":"name","selection":[0,5]}));
    assert_eq!(model["viewport"]["size"], json!([820, 600]));
    assert_eq!(model["frame"]["status_hidden"], true);
    assert_eq!(model["values"]["identity"], "Requested by local:test");
    assert_eq!(model["form_rows"], 1);
    let mut doc =
        scene::parse(&std::fs::read_to_string(root.join("chooser.scene.mix")).unwrap()).unwrap();
    doc.model = Some(model.clone());
    let resolved = scene::resolve(&doc).unwrap();
    assert!(
        resolved.nodes["root"]
            .ports
            .get("placements")
            .is_some_and(|v| v.is_object()),
        "{:?}",
        resolved
    );
    let mut view = viu::gold_frame::mount(
        &std::fs::read_to_string(root.join("chooser.scene.mix")).unwrap(),
        model,
    )
    .unwrap();
    let ctx = egui::Context::default();
    viu::appearance::install(&ctx);
    ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(820., 600.),
            )),
            ..Default::default()
        },
        |ui| {
            view.show(ui);
        },
    )
    .drop_without_applying_deltas();
    for id in [
        "gold-header",
        "places",
        "files",
        "name",
        "footer.command.accept",
    ] {
        assert!(view.geometry.contains_key(id), "{id}");
    }
    assert!(!view.geometry.contains_key("gold-status"));
}

#[test]
fn generic_bundle_mounts_frame_and_preserves_both_panes_on_scan_refusal() {
    let fixture = tempfile::tempdir().unwrap();
    let left = fixture.path().join("left");
    let right = fixture.path().join("right");
    std::fs::create_dir(&left).unwrap();
    std::fs::create_dir(&right).unwrap();
    std::fs::write(left.join("selected.txt"), "fixture").unwrap();
    std::fs::write(
        fixture.path().join("config.conf.mix"),
        json!({
            "schema_version":2,"left":{"path":left,"previews":false},
            "right":{"path":right,"previews":false}
        })
        .to_string(),
    )
    .unwrap();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/dopus");
    let mut bundle = viu::launch::load(
        &root,
        &[
            "--profile".into(),
            fixture.path().to_string_lossy().into_owned(),
        ],
    )
    .unwrap();
    assert_eq!(bundle.manifest.frame.as_deref(), Some("gold"));
    bundle.program.source.push_str(
        r#"
on budget-test
  $state.panes[1].scan_budget={rows:1,depth:0}
  reply(0,"{}")
end
on operation-test
  $state.job="test-operation";$state.busy=true
  dp_job_completed({tag:{id:"test-operation",owner:viu_context().generation},rc:0,body:{}})
  reply(0,"{}")
end
"#,
    );
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime.load(bundle.program).unwrap();
    let model = done(&runtime, &rx).model.unwrap();
    assert_eq!(model["panes"][0]["rows"], 1);
    assert_eq!(model["values"]["gold-title"], "MixViu DOpus");
    assert_eq!(model["values"]["caption-title"], "MixViu DOpus");
    assert_eq!(model["window_title"], "MixViu DOpus");
    let mut view = viu::gold_frame::mount(&bundle.scene, model.clone()).unwrap();
    assert!(view.has_control_kind("navigation-list"));
    let ctx = egui::Context::default();
    viu::appearance::install(&ctx);
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
    assert!(view.geometry.contains_key("gold-header"));
    assert!(view.geometry.contains_key("left-tree"));
    assert!(!view.geometry.contains_key("topbar"));
    runtime.event("dopus.actions".into(), json!({})).unwrap();
    let actions = done(&runtime, &rx).reply.unwrap().body;
    for id in ["nav.pane-left", "nav.pane-right"] {
        assert!(
            actions["actions"]
                .as_array()
                .unwrap()
                .iter()
                .any(|a| { a["id"] == id && a["enabled"] == true && a["keys"] == json!([]) })
        );
    }
    // Repeated explicit actions choose the same pane, including an empty one.
    // Exercise the real Bus handlers and verify the next action's target.
    for (id, active, rows) in [
        ("nav.pane-right", 1, 0),
        ("nav.pane-right", 1, 0),
        ("nav.pane-left", 0, 1),
        ("nav.pane-left", 0, 1),
    ] {
        runtime
            .event("dopus.execute".into(), json!({"id":id}))
            .unwrap();
        let completion = done(&runtime, &rx);
        assert_eq!(completion.reply.unwrap().rc, 0);
        let selected = completion.model.unwrap();
        assert_eq!(selected["active"], active);
        assert_eq!(selected["panes"][active]["rows"], rows);
        runtime
            .event("dopus.execute".into(), json!({"id":"selection.first"}))
            .unwrap();
        let selected = done(&runtime, &rx).model.unwrap();
        assert_eq!(selected["active"], active);
        if active == 0 {
            assert_eq!(
                selected["panes"][0]["selected"],
                json!(left.join("selected.txt"))
            );
        } else {
            assert_eq!(selected["panes"][1]["selected"], json!(null));
        }
        assert_eq!(selected["values"]["gold-title"], "MixViu DOpus");
        assert_eq!(selected["info"], if active == 0 {
            "selected.txt — TXT file — 7 B"
        } else { "Select a file or folder" });
        assert_eq!(selected["panes"][0]["summary"], "0 folders, 1 file (7 B)");
        runtime
            .event("dopus.execute".into(), json!({"id":"file.new-folder"}))
            .unwrap();
        let prompt = done(&runtime, &rx).model.unwrap();
        assert_eq!(prompt["active"], active);
        let instance = prompt["values"]["form"]["instance"].clone();
        runtime
            .event(
                "dopus.dialog.answer".into(),
                json!({"instance":instance,"choice":"cancel"}),
            )
            .unwrap();
        done(&runtime, &rx);
    }
    // A second file in each pane exercises pointer toggles, keyboard ranges,
    // active-pane changes and deselection through the actual bundle handlers.
    std::fs::write(left.join("second.txt"), "123").unwrap();
    std::fs::write(right.join("photo.jpg"), "12345").unwrap();
    let steps = [
        ("dopus.execute", json!({"id":"view.refresh"}), "Select a file or folder"),
        ("dopus.intent", json!({"id":"left-tree","action":"select","value":{"key":left.join("selected.txt")}}), "selected.txt — TXT file — 7 B"),
        ("dopus.intent", json!({"id":"left-tree","action":"select","value":{"key":left.join("second.txt"),"ctrl":true}}), "2 selected — 10 B"),
        ("dopus.intent", json!({"id":"left-tree","action":"select","value":{"key":left.join("second.txt"),"ctrl":true}}), "selected.txt — TXT file — 7 B"),
        ("dopus.intent", json!({"id":"left-tree","action":"select","value":{"key":left.join("selected.txt"),"ctrl":true}}), "Select a file or folder"),
        ("dopus.execute", json!({"id":"selection.first"}), "second.txt — TXT file — 3 B"),
        ("dopus.intent", json!({"id":"left-tree","action":"navigate","value":{"key":"End","shift":true}}), "2 selected — 10 B"),
        ("dopus.execute", json!({"id":"nav.pane-right"}), "Select a file or folder"),
        ("dopus.execute", json!({"id":"view.refresh"}), "Select a file or folder"),
        ("dopus.execute", json!({"id":"selection.first"}), "photo.jpg — JPG file — 5 B"),
        ("dopus.execute", json!({"id":"nav.pane-left"}), "2 selected — 10 B"),
        ("dopus.execute", json!({"id":"nav.pane-right"}), "photo.jpg — JPG file — 5 B"),
        ("operation-test", json!({}), "Select a file or folder"),
    ];
    for (command, args, info) in steps {
        runtime.event(command.into(), args).unwrap();
        let model = done(&runtime, &rx).model.unwrap();
        assert_eq!(model["info"], info, "{command}");
        assert_eq!(model["values"]["status"], info);
        assert_eq!(model["values"]["gold-status-text"]["left"], info);
        assert_eq!(model["values"]["gold-title"], "MixViu DOpus");
        assert_eq!(model["window_title"], "MixViu DOpus");
    }
    runtime.event("budget-test".into(), json!({})).unwrap();
    done(&runtime, &rx);
    // First navigation succeeds in the candidate; second exhausts its budget.
    // Neither pane nor the published model may commit the partial request.
    let broad = fixture.path().join("broad");
    std::fs::create_dir(&broad).unwrap();
    for name in ["a", "b"] {
        std::fs::write(broad.join(name), "fixture").unwrap();
    }
    runtime
        .event("dopus.open".into(), json!({"paths":[right,broad]}))
        .unwrap();
    let refusal = done(&runtime, &rx);
    assert!(refusal.model.is_none());
    assert_eq!(refusal.reply.unwrap().body["error_code"], "SCAN_BUDGET");
    runtime.event("dopus.state".into(), json!({})).unwrap();
    let state = done(&runtime, &rx).reply.unwrap().body;
    assert_eq!(state["panes"][0]["path"], json!(left));
    assert_eq!(state["panes"][1]["path"], json!(right));
    // The same live-session notice must update the published surface and Bus
    // state through authority loss and recovery, without replacing the app.
    for appearance in [
        json!({"mode":"light","scheme":"ocean"}),
        json!({}),
        json!({"mode":"dark","scheme":"pro"}),
    ] {
        runtime
            .event(
                "viu.session".into(),
                json!({"state":{"values":{}},"appearance":appearance}),
            )
            .unwrap();
        let model = done(&runtime, &rx).model.unwrap();
        let expected = if appearance.as_object().unwrap().is_empty() {
            json!({"mode":"dark","scheme":"studio"})
        } else {
            appearance
        };
        assert_eq!(model["appearance"]["mode"], expected["mode"]);
        assert_eq!(model["appearance"]["scheme"], expected["scheme"]);
        runtime.event("dopus.state".into(), json!({})).unwrap();
        let state = done(&runtime, &rx).reply.unwrap().body;
        assert_eq!(state["theme_mode"], expected["mode"]);
        assert_eq!(state["theme_scheme"], expected["scheme"]);
    }
}

#[test]
fn mix_launch_plan_preserves_literal_paths_and_print_config_stays_before_the_host() {
    let fixture = tempfile::tempdir().unwrap();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/dopus");
    let profile = fixture.path().to_string_lossy().into_owned();
    let bundle = viu::launch::load(
        &root,
        &[
            "--profile".into(),
            profile.clone(),
            "--headless".into(),
            "--".into(),
            "--literal-path".into(),
        ],
    )
    .unwrap();
    assert_eq!(bundle.launch_plan["headless"], true);
    assert_eq!(
        bundle.launch_plan["activation"][0]["body"]["paths"][0],
        std::env::current_dir()
            .unwrap()
            .join("--literal-path")
            .to_string_lossy()
            .as_ref()
    );
    let printed = viu::launch::load(
        &root,
        &["--profile".into(), profile, "--print-config".into()],
    )
    .unwrap();
    assert_eq!(printed.launch_plan["exit"], true);
    let config: serde_json::Value =
        serde_json::from_str(printed.launch_plan["stdout"].as_str().unwrap()).unwrap();
    assert_eq!(config["schema_version"], 2);
    assert!(viu::launch::load(&root, &["--unknown".into()]).is_err());
    // Mix expands tildes in double-quoted strings. Matching a literal tilde
    // must never mistake an absolute child of HOME for the shorthand.
    let home_child =
        std::path::PathBuf::from(std::env::var_os("HOME").unwrap()).join("activation-fixture");
    let absolute = viu::launch::load(&root, &[home_child.to_string_lossy().into_owned()]).unwrap();
    assert_eq!(
        absolute.launch_plan["activation"][0]["body"]["paths"][0],
        json!(home_child)
    );
}
