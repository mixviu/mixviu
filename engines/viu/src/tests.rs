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
                egui::vec2(440.0, 1000.0),
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
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/volume-lab/volume");
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
