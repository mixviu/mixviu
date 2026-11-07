// SPDX-License-Identifier: MIT OR Apache-2.0
// Existing Mix chooser/raw-name tests moved off the retired app host.
use serde_json::json;
use std::{path::PathBuf, sync::Arc, time::{Duration, Instant}};
use viu::{jobs::Jobs, runtime::{Program, Runtime}, view::SceneView};
fn wait(
    runtime: &Runtime,
    jobs: &mut Jobs,
    sequence: u64,
) -> (viu::runtime::Completion, Vec<viu::jobs::ResultMessage>) {
    let mut jobs_done = Vec::new();
    let end = Instant::now() + Duration::from_secs(8);
    loop {
        jobs_done.extend(jobs.poll());
        if let Some(completed) = runtime.try_completion() {
            if completed.sequence == sequence {
                assert!(
                    completed.error.is_none(),
                    "{:?}\n{}",
                    completed.error,
                    completed.stderr
                );
                return (completed, jobs_done);
            }
        }
        assert!(Instant::now() < end, "Mix did not complete");
        std::thread::sleep(Duration::from_millis(2));
    }
}
#[test]
fn mix_chooser_renders_filters_choices_and_pins_replace_acceptance() {
    let dir = tempfile::tempdir().unwrap();
    let existing = dir.path().join("existing.txt");
    std::fs::write(&existing, "retained").unwrap();
    std::fs::write(dir.path().join("ignore.png"), "png").unwrap();
    std::fs::create_dir(dir.path().join("folder")).unwrap();
    let wake: viu::runtime::Wake = Arc::new(|| {});
    let service = viu::chooser::Service::new(
        7,
        dir.path().as_os_str().as_encoded_bytes().to_vec(),
        wake.clone(),
    )
    .unwrap();
    let opened=service.command("choose",&json!({"mode":"save","current_folder":dir.path(),"current_name":"existing.txt","filters":[{"name":"Text","patterns":["*.txt"],"mime_types":[]}],"choices":[{"id":"encoding","label":"Encoding","options":[{"id":"utf8","label":"UTF-8"},{"id":"ascii","label":"ASCII"}],"default":"utf8"}]}),"test");
    let id = opened["body"]["request"].as_str().unwrap();
    let request = service.prepared()[id].open.request.body().unwrap();
    service.command("ready", &json!({"request":id}), "host");
    let app_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/dopus");
    let path = app_dir.join("chooser.mix");
    let mut worker = Runtime::new(8, wake.clone(), None);
    let mut jobs = Jobs::new(1, wake);
    worker
        .load(Program {
            source: std::fs::read_to_string(&path).unwrap(),
            path,
            scene: "chooser".into(),
            citizen: "chooser-test".into(),
        })
        .unwrap();
    let seq = worker
        .event(
            "chooser.open".into(),
            json!({"id":id,"request":request,"home":dir.path()}),
        )
        .unwrap();
    let model = wait(&worker, &mut jobs, seq).0.model.unwrap();
    assert_eq!(
        model["values"]["files"]["rows"].as_array().unwrap().len(),
        2
    );
    let mut view = SceneView::with_model(
        &std::fs::read_to_string(app_dir.join("chooser.scene.mix")).unwrap(),
        model,
    )
    .unwrap();
    let ctx = egui::Context::default();
    viu::appearance::install(&ctx);
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        view.show(ui);
    });
    output.textures_delta.clear();
    drop(output);
    assert!(view.geometry.contains_key("files"));
    let seq=worker.event("chooser.call".into(),json!({"command":"intent","id":"options","action":"command","value":{"command":"option:0:1"}})).unwrap();
    let model = wait(&worker, &mut jobs, seq).0.model.unwrap();
    assert_eq!(model["choices"]["encoding"], "ascii");
    let seq=worker.event("chooser.call".into(),json!({"command":"intent","id":"footer","action":"command","value":{"command":"accept"}})).unwrap();
    let model = wait(&worker, &mut jobs, seq).0.model.unwrap();
    let refusal = service.command("accept", &model["effect"]["body"], "test");
    assert_eq!(refusal["body"]["error_code"], "EXISTS");
    let seq = worker
        .event(
            "chooser.call".into(),
            json!({"command":"result","rc":10,"body":refusal["body"]}),
        )
        .unwrap();
    let model = wait(&worker, &mut jobs, seq).0.model.unwrap();
    let form = &model["values"]["form"];
    assert_eq!(form["open"], true);
    let seq=worker.event("chooser.call".into(),json!({"command":"intent","id":"form","action":"answer","value":{"choice":"replace","instance":form["instance"]}})).unwrap();
    let model = wait(&worker, &mut jobs, seq).0.model.unwrap();
    let accepted = service.command("accept", &model["effect"]["body"], "test");
    assert_eq!(accepted["rc"], 0, "{accepted}");
    assert_eq!(accepted["outcome"]["choices"]["encoding"], "ascii");
    assert_eq!(
        std::fs::read_to_string(existing).unwrap(),
        "retained",
        "A chooser authorises a path; it never writes the file"
    );
}

#[test]
fn mix_chooser_preserves_raw_save_name_until_the_field_is_edited() {
    let dir = tempfile::tempdir().unwrap();
    let wake: viu::runtime::Wake = Arc::new(|| {});
    let app_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/dopus");
    for edited in [false, true] {
        let service = viu::chooser::Service::new(
            7,
            dir.path().as_os_str().as_encoded_bytes().to_vec(),
            wake.clone(),
        )
        .unwrap();
        let opened = service.command("choose", &json!({"mode":"save","current_folder":dir.path(),"current_name":{"bytes":"new%FF.txt"}}), "test");
        assert_eq!(opened["rc"], 0, "{opened}");
        let id = opened["body"]["request"].as_str().unwrap();
        let request = service.prepared()[id].open.request.body().unwrap();
        service.command("ready", &json!({"request":id}), "host");
        let path = app_dir.join("chooser.mix");
        let mut worker = Runtime::new(8, wake.clone(), None);
        let mut jobs = Jobs::new(1, wake.clone());
        worker
            .load(Program {
                source: std::fs::read_to_string(&path).unwrap(),
                path,
                scene: "chooser".into(),
                citizen: "chooser-test".into(),
            })
            .unwrap();
        let seq = worker
            .event(
                "chooser.open".into(),
                json!({"id":id,"request":request,"home":dir.path()}),
            )
            .unwrap();
        let mut model = wait(&worker, &mut jobs, seq).0.model.unwrap();
        assert_eq!(model["values"]["name"], "new\u{fffd}.txt");
        if edited {
            let seq = worker.event("chooser.call".into(), json!({"command":"intent","id":"name","action":"set","value":model["values"]["name"]})).unwrap();
            model = wait(&worker, &mut jobs, seq).0.model.unwrap();
        }
        let seq = worker.event("chooser.call".into(), json!({"command":"intent","id":"name","action":"submit","value":model["values"]["name"]})).unwrap();
        let model = wait(&worker, &mut jobs, seq).0.model.unwrap();
        let accepted = service.command("accept", &model["effect"]["body"], "test");
        assert_eq!(accepted["rc"], 0, "{accepted}");
        let uri = accepted["outcome"]["uris"][0].as_str().unwrap();
        assert!(
            uri.ends_with(if edited {
                "/new%EF%BF%BD.txt"
            } else {
                "/new%FF.txt"
            }),
            "{accepted}"
        );
        assert_eq!(
            std::fs::read_dir(dir.path()).unwrap().count(),
            0,
            "Choosing a save path must not write it"
        );
    }
}

use std::os::unix::ffi::OsStrExt;

#[test]
fn mix_controller_selects_and_copies_a_non_utf8_name_without_replacing_its_bytes() {
    let dir = tempfile::tempdir().unwrap();
    let left = dir.path().join("left");
    let right = dir.path().join("right");
    std::fs::create_dir(&left).unwrap();
    std::fs::create_dir(&right).unwrap();
    let name = std::ffi::OsStr::from_bytes(b"raw\xff.txt");
    std::fs::write(left.join(name), b"exact raw-name file").unwrap();
    std::fs::write(
        dir.path().join("config.conf.mix"),
        json!({"left":{"path":left,"previews":false},"right":{"path":right,"previews":false}})
            .to_string(),
    )
    .unwrap();
    let app_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps/dopus");
    let wake: viu::runtime::Wake = Arc::new(|| {});
    let mut jobs = Jobs::new(1, wake.clone());
    let mut runtime = Runtime::new(4, wake, Some(jobs.backend(None)));
    let path = app_dir.join("dopus.mix");
    runtime
        .load(Program {
            source: std::fs::read_to_string(&path).unwrap(),
            path,
            scene: "raw-name".into(),
            citizen: "raw-name".into(),
        })
        .unwrap();
    let seq = runtime
        .event(
            "dopus.initialise".into(),
            json!({"profile":dir.path(),"app_dir":app_dir}),
        )
        .unwrap();
    let initial = wait(&runtime, &mut jobs, seq)
        .0
        .model
        .unwrap();
    let key = initial["values"]["left-tree"]["rows"][0]["key"]
        .as_str()
        .unwrap();
    assert!(key.contains("%FF"), "{initial}");
    assert_eq!(initial["panes"][0]["rows"], 1);
    let seq = runtime
        .event(
            "dopus.call".into(),
            json!({"command":"dopus.execute","args":{"id":"selection.last"}}),
        )
        .unwrap();
    let selected = wait(&runtime, &mut jobs, seq)
        .0
        .model
        .unwrap();
    assert_eq!(selected["panes"][0]["selected"], key);
    let seq = runtime
        .event(
            "dopus.call".into(),
            json!({"command":"dopus.execute","args":{"id":"file.copy-other-pane"}}),
        )
        .unwrap();
    let (_, mut done) = wait(&runtime, &mut jobs, seq);
    let until = Instant::now() + std::time::Duration::from_secs(5);
    while done.is_empty() {
        done.extend(jobs.poll());
        assert!(Instant::now() < until);
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert_eq!(done[0].rc, 0, "{:?}", done[0].body);
    assert_eq!(
        std::fs::read(right.join(name)).unwrap(),
        b"exact raw-name file"
    );
    assert_eq!(
        std::fs::read(left.join(name)).unwrap(),
        b"exact raw-name file"
    );
    assert!(!right.join("raw�.txt").exists());
}

