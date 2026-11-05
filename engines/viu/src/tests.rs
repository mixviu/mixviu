// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;
use serde_json::json;
use std::{sync::mpsc, time::Duration};
use viu::{runtime::Completion, view::Event};

fn ready(runtime: &Runtime, wakes: &mpsc::Receiver<()>) -> Completion {
    wakes.recv_timeout(Duration::from_secs(5)).unwrap();
    let done = runtime.try_completion().unwrap();
    assert!(
        done.error.is_none(),
        "{:?} stdout={} stderr={}",
        done.error,
        done.stdout,
        done.stderr
    );
    done
}
fn input(ctx: &egui::Context, view: &mut SceneView, events: Vec<egui::Event>) -> Vec<Event> {
    let mut result = Vec::new();
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(1600.0, 1000.0),
            )),
            events,
            ..Default::default()
        },
        |ui| {
            result = view.show(ui);
        },
    );
    output.textures_delta.clear();
    result
}
fn click(ctx: &egui::Context, view: &mut SceneView, id: &str) -> Vec<Event> {
    input(ctx, view, vec![]);
    let pos = view.geometry[id].center();
    input(ctx, view, vec![egui::Event::PointerMoved(pos)]);
    input(
        ctx,
        view,
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        }],
    );
    input(
        ctx,
        view,
        vec![egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        }],
    )
}

#[test]
fn native_widget_input_runs_existing_mix_callbacks_and_readback() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/quoin/volume");
    let path = root.join("viu.mix").canonicalize().unwrap();
    let (tx, wakes) = mpsc::channel();
    let mut runtime = Runtime::new(
        16,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        Some(Box::new(|| fixture::AudioFixture::new())),
    );
    runtime
        .load(Program {
            source: std::fs::read_to_string(&path).unwrap(),
            path,
            scene: "volume".into(),
            citizen: "scene-volume".into(),
        })
        .unwrap();
    let mut view =
        SceneView::new(&std::fs::read_to_string(root.join("scene.mix")).unwrap()).unwrap();
    view.apply_model(
        ready(&runtime, &wakes)
            .model
            .expect("initial snapshot model"),
    )
    .unwrap();
    assert_eq!(view.model()["out"]["level_text"], "50%");
    let ctx = egui::Context::default();
    let events = click(&ctx, &mut view, "vo_up");
    assert_eq!(events.len(), 1, "one native click gives one semantic event");
    assert_eq!(events[0].command, "vol_out_up");
    for event in events {
        runtime.event(event.command, event.body).unwrap();
    }
    view.apply_model(ready(&runtime, &wakes).model.unwrap())
        .unwrap();
    assert_eq!(view.model()["out"]["level_text"], "55%");
    // Input/mute and stream selection remain separate operations.
    runtime.event("vol_in_mute".into(), json!({})).unwrap();
    view.apply_model(ready(&runtime, &wakes).model.unwrap())
        .unwrap();
    assert_eq!(view.model()["inp"]["mute_label"], "Unmute");
    runtime
        .event("vol_stream_pick".into(), json!({"item":{"id":"si_6"}}))
        .unwrap();
    view.apply_model(ready(&runtime, &wakes).model.unwrap())
        .unwrap();
    assert_eq!(view.model()["stream"]["name"], "Example call");
    runtime
        .event("vol_output_pick".into(), json!({"item":{"id":"sink_4"}}))
        .unwrap();
    view.apply_model(ready(&runtime, &wakes).model.unwrap())
        .unwrap();
    assert_eq!(view.model()["out"]["name"], "Example headphones");
    assert_eq!(view.model()["out"]["level_text"], "65%");
    let events = click(&ctx, &mut view, "sink_1#vo_row");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].command, "vol_output_pick");
    assert_eq!(events[0].body["node"], "v_outs");
    assert_eq!(events[0].body["item"]["id"], "sink_1");
    for event in events {
        runtime.event(event.command, event.body).unwrap();
    }
    view.apply_model(ready(&runtime, &wakes).model.unwrap())
        .unwrap();
    let events = click(&ctx, &mut view, "v75#vo_seg");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].command, "vol_out_set");
    assert_eq!(events[0].body["node"], "vo_bar");
    assert_eq!(events[0].body["item"]["pct"].as_f64(), Some(75.0));
    for event in events {
        runtime.event(event.command, event.body).unwrap();
    }
    view.apply_model(ready(&runtime, &wakes).model.unwrap())
        .unwrap();
    assert_eq!(view.model()["out"]["level_text"], "75%");
    for open in [true, false, true] {
        runtime
            .notify(
                "scenes.changed".into(),
                json!({"scenes":[{"name":"volume","open":open}]}),
            )
            .unwrap();
        let done = ready(&runtime, &wakes);
        if open {
            assert!(
                done.model.is_some(),
                "every popup reopen must refresh, not just the first"
            );
        }
    }
}

#[test]
fn reload_commits_scene_with_its_fresh_model_and_retains_last_good_on_error() {
    let old_scene = "---\nscene: 1\nname: test\ncitizen: test\nmodel: {\"old\":\"previous\"}\n---\n```mix\nroot: {widget:\"text\",text:\"= $model.old\"}\n```\n";
    let new_scene = old_scene.replace("old", "new");
    let (tx, wakes) = mpsc::channel();
    let runtime = Runtime::new(
        4,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    let mut client = Client {
        runtime,
        view: SceneView::new(old_scene).unwrap(),
        error: String::new(),
        program: Program {
            path: PathBuf::from("/tmp/viu-reload.mix"),
            source: String::new(),
            scene: "test".into(),
            citizen: "test".into(),
        },
        witness: None,
        last_sequence: 0,
        pending_view: None,
        behaviour_ready: true,
        lifecycle_note: String::new(),
        caption: "Test".into(),
    };
    let source = "$ctx=viu_context(); send \"viu-local\" \"scenes.model\" body=json_encode({generation:$ctx.generation,value:{new:\"fresh\"}});";
    client.reload(&new_scene, source.into()).unwrap();
    assert!(!client.behaviour_ready);
    assert_eq!(
        client.view.model()["old"],
        "previous",
        "keep committed view while load is pending"
    );
    client.complete(ready(&client.runtime, &wakes));
    assert!(client.behaviour_ready);
    assert_eq!(client.view.model()["new"], "fresh");
    assert!(client.view.model().get("old").is_none());
    let revision = client.view.revision;
    client.reload(old_scene, "$broken = [".into()).unwrap();
    wakes.recv_timeout(Duration::from_secs(5)).unwrap();
    client.complete(client.runtime.try_completion().unwrap());
    assert!(!client.behaviour_ready);
    assert!(!client.error.is_empty());
    assert_eq!(client.view.model()["new"], "fresh");
    assert_eq!(client.view.revision, revision);
    client.reload(&new_scene, source.into()).unwrap();
    client.complete(ready(&client.runtime, &wakes));
    assert!(client.behaviour_ready);
    assert!(client.error.is_empty());
}

#[test]
fn fields_route_native_text_and_enter_and_keep_edits_across_model_updates() {
    let source = "---\nscene: 1\nname: fields\ncitizen: fields\nmodel: {\"path\":\"\",\"status\":\"ready\"}\n---\n```mix\nroot: {widget:\"column\",children:[\"path\"]}\npath: {widget:\"field\",value:\"= $model.path\",width:300,on_change:\"edit\",on_submit:\"open\"}\n```\n";
    let mut view = SceneView::new(source).unwrap();
    let ctx = egui::Context::default();
    click(&ctx, &mut view, "path");
    let events = input(&ctx, &mut view, vec![egui::Event::Text("folder".into())]);
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].command, "edit");
    assert_eq!(events[0].body["value"], "folder");
    view.apply_model(json!({"path":"", "status":"unrelated update"}))
        .unwrap();
    let events = input(&ctx, &mut view, vec![egui::Event::Text("2".into())]);
    assert_eq!(events[0].body["value"], "folder2");
    let events = input(
        &ctx,
        &mut view,
        vec![egui::Event::Key {
            key: egui::Key::Enter,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    assert!(
        events
            .iter()
            .any(|e| e.command == "open" && e.body["value"] == "folder2")
    );
    view.apply_model(json!({"path":"authoritative", "status":"navigated"}))
        .unwrap();
    click(&ctx, &mut view, "path");
    input(
        &ctx,
        &mut view,
        vec![egui::Event::Key {
            key: egui::Key::Home,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: Default::default(),
        }],
    );
    let events = input(&ctx, &mut view, vec![egui::Event::Text("x".into())]);
    assert!(
        events
            .iter()
            .any(|e| e.body["value"].as_str().unwrap().contains("authoritative"))
    );
}

#[test]
fn application_config_separates_dopus_from_the_audio_fixture() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/dopus");
    let source = std::fs::read_to_string(root.join("client.json")).unwrap();
    let config = ClientConfig::parse(Some(&source)).unwrap();
    assert_eq!(config.behaviour, "dopus.mix");
    assert_eq!(config.scene, "dopus-mix");
    assert_eq!(config.app_id, "mixviu.dopus.lab");
    assert!(!config.audio_fixture);
    assert!(ClientConfig::parse(None).unwrap().audio_fixture);
    assert!(ClientConfig::parse(Some("{}")).is_err());
    assert!(ClientConfig::parse(Some(&source.replace("\"none\"", "\"unknown\""))).is_err());
    assert!(ClientConfig::parse(Some(&source.replace("dopus.mix", "../outside.mix"))).is_err());
}

#[test]
fn embedded_dopus_native_fields_and_rows_drive_real_file_operations() {
    let app = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/dopus");
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("viu-dopus-{}-{unique}", std::process::id()));
    std::fs::create_dir_all(root.join("left")).unwrap();
    std::fs::create_dir_all(root.join("right")).unwrap();
    std::fs::write(root.join("left/a.txt"), "alpha").unwrap();
    let path = app.join("dopus.mix").canonicalize().unwrap();
    let source = std::fs::read_to_string(&path).unwrap().replace(
        "$demo.prepare_demo($root)",
        &format!(
            "$root = {}; $demo.prepare_demo($root)",
            json!(root.to_str().unwrap())
        ),
    );
    let (tx, wakes) = mpsc::channel();
    let mut runtime = Runtime::new(
        16,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime
        .load(Program {
            path,
            source,
            scene: "dopus-mix".into(),
            citizen: "dopus-mix".into(),
        })
        .unwrap();
    let mut view =
        SceneView::new(&std::fs::read_to_string(app.join("scene.mix")).unwrap()).unwrap();
    view.apply_model(ready(&runtime, &wakes).model.unwrap())
        .unwrap();
    let ctx = egui::Context::default();
    let row = format!("{}#file_row", root.join("left/a.txt").display());
    for event in click(&ctx, &mut view, &row) {
        runtime.event(event.command, event.body).unwrap();
        view.apply_model(ready(&runtime, &wakes).model.unwrap())
            .unwrap();
    }
    assert!(view.geometry["list_left"].height() >= 280.0);
    assert!(view.geometry["list_right"].height() >= 280.0);
    assert!(
        view.model()["left"]["heading"]
            .as_str()
            .unwrap()
            .contains("1 selected")
    );
    click(&ctx, &mut view, "name");
    let events = input(
        &ctx,
        &mut view,
        vec![egui::Event::Text("renamed.txt".into())],
    );
    assert_eq!(events.len(), 1);
    for event in events {
        runtime.event(event.command, event.body).unwrap();
        assert!(ready(&runtime, &wakes).model.is_none());
    }
    for command in ["rename", "copy"] {
        let events = click(&ctx, &mut view, command);
        assert_eq!(events.len(), 1, "native {command} button");
        for event in events {
            runtime.event(event.command, event.body).unwrap();
        }
        view.apply_model(ready(&runtime, &wakes).model.unwrap())
            .unwrap();
    }
    assert!(!root.join("left/a.txt").exists());
    assert_eq!(
        std::fs::read_to_string(root.join("right/renamed.txt")).unwrap(),
        "alpha"
    );
    runtime.event("copy".into(), json!({})).unwrap();
    let refusal = ready(&runtime, &wakes).model.unwrap();
    assert!(
        refusal["status"]
            .as_str()
            .unwrap()
            .starts_with("DEST_EXISTS")
    );
    drop(runtime);
    std::fs::remove_dir_all(root).unwrap();
}
