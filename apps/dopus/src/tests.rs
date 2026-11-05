// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;
use std::time::Duration;
pub(super) fn wait(
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
fn embedded_dopus_initialises_reduces_refuses_and_renders_the_shared_scene() {
    let dir = tempfile::tempdir().unwrap();
    let left = dir.path().join("left");
    let right = dir.path().join("right");
    std::fs::create_dir(&left).unwrap();
    std::fs::create_dir(&right).unwrap();
    std::fs::write(left.join("one.txt"), "one").unwrap();
    std::fs::create_dir(left.join("Folder")).unwrap();
    let config = json!({"schema_version":2,"left":{"path":left,"previews":false},"right":{"path":right,"previews":false},"bookmarks":[],"places":{"open":true,"width":0.15},"properties":{"open":true,"width":0.22},"split_ratio":0.5});
    std::fs::write(dir.path().join("config.conf.mix"), config.to_string()).unwrap();
    let app_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let path = app_dir.join("dopus.mix");
    let wake: viu::runtime::Wake = Arc::new(|| {});
    let mut jobs = Jobs::new(2, wake.clone());
    let mut runtime = Runtime::new(8, wake, Some(jobs.backend(None)));
    runtime
        .load(Program {
            path: path.clone(),
            source: std::fs::read_to_string(path).unwrap(),
            scene: "dopus".into(),
            citizen: "dopus-test".into(),
        })
        .unwrap();
    let seq = runtime
        .event(
            "dopus.initialise".into(),
            json!({"profile":dir.path(),"app_dir":app_dir}),
        )
        .unwrap();
    let first = wait(&runtime, &mut jobs, seq).0;
    let model = first.model.unwrap();
    assert_eq!(model["panes"][0]["rows"], 2);
    assert_eq!(model["panes"][0]["summary"], "1 folder, 1 file (3 B)");
    let seq = runtime.event("dopus.call".into(), json!({"command":"dopus.intent","args":{"id":"left-tree","action":"activate","value":{"key":left.join("Folder")}}})).unwrap();
    let expanded = wait(&runtime, &mut jobs, seq).0;
    assert_eq!(expanded.reply.unwrap().rc, 0);
    let expanded = expanded.model.unwrap();
    assert_eq!(expanded["panes"][0]["path"], json!(left));
    assert_eq!(
        expanded["panes"][0]["expanded"],
        json!([left.join("Folder")])
    );
    for pane in [json!(1), json!(0)] {
        let seq = runtime.event("dopus.call".into(), json!({"command":"dopus.execute","args":{"id":"location.focus","params":{"pane":pane}}})).unwrap();
        let changed = wait(&runtime, &mut jobs, seq).0;
        assert_eq!(changed.reply.unwrap().rc, 0);
        assert_eq!(changed.model.unwrap()["active"], pane);
    }
    for pane in [json!(2), json!(true), json!(-1)] {
        let seq = runtime.event("dopus.call".into(), json!({"command":"dopus.execute","args":{"id":"location.focus","params":{"pane":pane}}})).unwrap();
        let refused = wait(&runtime, &mut jobs, seq).0;
        assert_eq!(refused.reply.unwrap().rc, 10);
        assert!(refused.model.is_none());
    }
    let mut view = SceneView::with_model(
        &std::fs::read_to_string(app_dir.join("scene.mix")).unwrap(),
        model,
    )
    .unwrap();
    let ctx = egui::Context::default();
    viu::appearance::install(&ctx);
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        assert!(view.show(ui).is_empty());
    });
    output.textures_delta.clear();
    drop(output);
    assert!(view.geometry.contains_key("left-tree"));
    let seq = runtime
        .event(
            "dopus.call".into(),
            json!({"command":"dopus.execute","args":{"id":"selection.last"}}),
        )
        .unwrap();
    let selected = wait(&runtime, &mut jobs, seq).0;
    let selected = selected.model.unwrap();
    assert_eq!(
        selected["panes"][0]["selected"],
        json!(left.join("one.txt"))
    );
    let rows = selected["values"]["left-tree"]["rows"].clone();
    assert_eq!(
        selected["values"]["left-tree"]["selected_keys"],
        json!([left.join("one.txt")])
    );
    let seq = runtime
        .event(
            "dopus.call".into(),
            json!({"command":"dopus.execute","args":{"id":"selection.first"}}),
        )
        .unwrap();
    let first_selected = wait(&runtime, &mut jobs, seq).0.model.unwrap();
    assert_eq!(first_selected["values"]["left-tree"]["rows"], rows);
    assert_eq!(
        first_selected["values"]["left-tree"]["focused_key"],
        json!(left.join("Folder"))
    );
    std::fs::write(left.join("new.txt"), "new listing").unwrap();
    let seq = runtime
        .event(
            "dopus.call".into(),
            json!({"command":"dopus.execute","args":{"id":"view.refresh"}}),
        )
        .unwrap();
    let refreshed = wait(&runtime, &mut jobs, seq).0.model.unwrap();
    assert_eq!(refreshed["panes"][0]["rows"], 3);
    assert_ne!(refreshed["values"]["left-tree"]["rows"], rows);
    assert_eq!(refreshed["values"]["left-tree"]["selected_keys"], json!([]));
    let seq = runtime
        .event(
            "dopus.call".into(),
            json!({"command":"dopus.execute","args":{"id":"selection.last"}}),
        )
        .unwrap();
    wait(&runtime, &mut jobs, seq);
    let seq=runtime.event("dopus.call".into(),json!({"command":"dopus.intent","args":{"id":"left-tree","action":"select","value":{"key":"/not-a-row"}}})).unwrap();
    let refused = wait(&runtime, &mut jobs, seq).0;
    assert_eq!(refused.reply.unwrap().rc, 10);
    assert!(refused.model.is_none());
    let seq = runtime
        .event(
            "dopus.call".into(),
            json!({"command":"dopus.execute","args":{"id":"file.copy-other-pane"}}),
        )
        .unwrap();
    let started = wait(&runtime, &mut jobs, seq);
    assert_eq!(
        started.0.reply.as_ref().map(|r| r.rc),
        Some(0),
        "{:?}",
        started.0.reply
    );
    assert_eq!(started.0.model.unwrap()["busy"], true);
    let mut done = started.1;
    let end = Instant::now() + Duration::from_secs(8);
    loop {
        done.extend(jobs.poll());
        if !done.is_empty() {
            let done = done.into_iter().next().unwrap();
            assert_eq!(done.rc, 0, "{}", done.body);
            assert_eq!(
                std::fs::read_to_string(right.join("one.txt")).unwrap(),
                "one"
            );
            let seq = runtime
                .notify(
                    "dopus.job.completed".into(),
                    json!({"tag":done.tag,"rc":done.rc,"body":done.body}),
                )
                .unwrap();
            let settled = wait(&runtime, &mut jobs, seq).0;
            assert_eq!(settled.model.unwrap()["busy"], false);
            break;
        }
        assert!(Instant::now() < end);
        std::thread::sleep(Duration::from_millis(2));
    }
    let seq=runtime.event("dopus.call".into(),json!({"command":"dopus.theme.set","args":{"mode":"light","scheme":"forest","style":"plain"}})).unwrap();
    let changed = wait(&runtime, &mut jobs, seq).0;
    assert_eq!(changed.reply.unwrap().rc, 0);
    let model = changed.model.unwrap();
    assert_eq!(
        model["appearance"],
        json!({"mode":"light","scheme":"forest","style":"plain"})
    );
    assert!(
        model["actions"]
            .as_array()
            .unwrap()
            .iter()
            .any(|a| a["id"] == "theme.style-plain" && a["checked"] == true)
    );
    let seq = runtime
        .event(
            "dopus.call".into(),
            json!({"command":"dopus.theme.set","args":{"mode":"dark","style":"invalid"}}),
        )
        .unwrap();
    let refused = wait(&runtime, &mut jobs, seq).0;
    assert_eq!(refused.reply.unwrap().rc, 10);
    assert!(refused.model.is_none());
    let seq = runtime
        .event(
            "dopus.call".into(),
            json!({"command":"dopus.theme.set","args":{}}),
        )
        .unwrap();
    assert_eq!(
        wait(&runtime, &mut jobs, seq).0.model.unwrap()["appearance"],
        model["appearance"]
    );
    let seq=runtime.event("dopus.call".into(),json!({"command":"dopus.session","args":{"appearance":{"scheme":"ocean","mode":"dark","density":"compact"},"keymap":{"nav.parent":["Ctrl+U"]}}})).unwrap();
    let followed = wait(&runtime, &mut jobs, seq).0.model.unwrap();
    assert_eq!(followed["appearance"]["scheme"], "forest");
    assert_eq!(followed["appearance"]["mode"], "light");
    assert_eq!(followed["appearance"]["density"], "compact");
    assert_eq!(
        followed["actions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|a| a["id"] == "nav.parent")
            .unwrap()["keys"],
        json!(["Ctrl+U"])
    );
    for (key,expected) in [("scheme","ocean"),("mode","dark"),("style","default")] {
        let seq=runtime.event("dopus.call".into(),json!({"command":"dopus.execute","args":{"id":format!("theme.{key}-session")}})).unwrap();
        let reset=wait(&runtime,&mut jobs,seq).0.model.unwrap();
        assert_eq!(reset["appearance"][key],expected);
        assert!(reset["actions"].as_array().unwrap().iter().any(|a|a["id"]==format!("theme.{key}-session") && a["checked"]==true));
    }
    let seq=runtime.event("dopus.call".into(),json!({"command":"dopus.session","args":{"appearance":{"scheme":"sunset","mode":"light","style":"plain"},"keymap":{}}})).unwrap();
    let reset_followed=wait(&runtime,&mut jobs,seq).0.model.unwrap();
    assert_eq!(reset_followed["appearance"],json!({"scheme":"sunset","mode":"light","style":"plain"}));
}

#[test]
fn native_queued_confirmations_keep_selection_and_core_tokens_across_other_forms() {
    let dir = tempfile::tempdir().unwrap();
    for name in ["a.txt", "b.txt"] {
        std::fs::write(dir.path().join(name), name).unwrap();
    }
    std::fs::create_dir(dir.path().join("profile")).unwrap();
    let profile = dir.path().join("profile");
    std::fs::write(profile.join("config.conf.mix"),json!({"left":{"path":dir.path(),"previews":false},"right":{"path":dir.path(),"previews":false}}).to_string()).unwrap();
    let contract: Value = serde_json::from_str(
        &std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("contract.json"))
            .unwrap(),
    )
    .unwrap();
    let mut native = native::Native::new(&profile, contract, Arc::new(|| {}));
    let ctx = egui::Context::default();
    let end = Instant::now() + Duration::from_secs(5);
    while native.model()["ready"] != true {
        native.poll(&ctx);
        assert!(Instant::now() < end);
        std::thread::sleep(Duration::from_millis(2));
    }
    native.call("dopus.theme.set",&json!({"scheme":"forest","mode":"light","style":"plain"})).unwrap();
    native.call("dopus.session",&json!({"appearance":{"scheme":"ocean","mode":"dark","style":"default"},"keymap":{}})).unwrap();
    assert_eq!(native.model()["appearance"]["scheme"],"forest");
    for (key,expected) in [("scheme","ocean"),("mode","dark"),("style","default")] {
        native.action(&format!("theme.{key}-session"),&json!({})).unwrap();
        assert_eq!(native.model()["appearance"][key],expected);
        assert!(native.model()["actions"].as_array().unwrap().iter().any(|a|a["id"]==format!("theme.{key}-session") && a["checked"]==true));
    }
    native.call("dopus.session",&json!({"appearance":{"scheme":"sunset","mode":"light","style":"plain"},"keymap":{}})).unwrap();
    assert_eq!(native.model()["appearance"],json!({"scheme":"sunset","mode":"light","style":"plain"}));
    for pane in [json!(1), json!(0)] {
        native
            .action("location.focus", &json!({"pane":pane}))
            .unwrap();
        assert_eq!(native.model()["active"], pane);
    }
    for pane in [json!(2), json!(true), json!(-1)] {
        assert!(
            native
                .action("location.focus", &json!({"pane":pane}))
                .is_err()
        );
        assert_eq!(native.model()["active"], 0);
    }
    native
        .intent(&json!({"id":"left-tree","action":"activate","value":{"key":profile}}))
        .unwrap();
    assert_eq!(native.model()["panes"][0]["path"], json!(dir.path()));
    assert_eq!(native.model()["panes"][0]["expanded"], json!([profile]));
    native.action("help.about", &json!({})).unwrap();
    for name in ["a.txt", "b.txt"] {
        native
            .intent(
                &json!({"id":"left-tree","action":"select","value":{"key":dir.path().join(name)}}),
            )
            .unwrap();
        native.action("file.delete", &json!({})).unwrap();
    }
    assert_eq!(native.model()["values"]["form"]["title"], "About Dopus");
    native.answer("ok", None).unwrap();
    let first = native.model()["values"]["form"].clone();
    assert!(first["body"].as_str().unwrap().contains("a.txt"));
    native.answer("cancel", None).unwrap();
    let second = native.model()["values"]["form"].clone();
    assert!(second["body"].as_str().unwrap().contains("b.txt"));
    assert_ne!(first["instance"], second["instance"]);
    native.answer("ok", None).unwrap();
    let end = Instant::now() + Duration::from_secs(5);
    while native.model()["busy"] == true {
        native.poll(&ctx);
        assert!(Instant::now() < end);
        std::thread::sleep(Duration::from_millis(2));
    }
    assert!(dir.path().join("a.txt").exists());
    assert!(!dir.path().join("b.txt").exists());
    assert!(native.answer("ok", None).is_err());
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
    let app_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
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
