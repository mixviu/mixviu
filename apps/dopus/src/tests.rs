// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;
use std::time::Duration;
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
}

#[test]
fn native_queued_confirmations_keep_selection_and_core_tokens_across_other_forms(){
    let dir=tempfile::tempdir().unwrap();
    for name in ["a.txt","b.txt"]{std::fs::write(dir.path().join(name),name).unwrap();}
    std::fs::create_dir(dir.path().join("profile")).unwrap();
    let profile=dir.path().join("profile");
    std::fs::write(profile.join("config.conf.mix"),json!({"left":{"path":dir.path(),"previews":false},"right":{"path":dir.path(),"previews":false}}).to_string()).unwrap();
    let contract:Value=serde_json::from_str(&std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("contract.json")).unwrap()).unwrap();
    let mut native=native::Native::new(&profile,contract,Arc::new(||{}));
    let ctx=egui::Context::default();
    let end=Instant::now()+Duration::from_secs(5);
    while native.model()["ready"]!=true{native.poll(&ctx);assert!(Instant::now()<end);std::thread::sleep(Duration::from_millis(2));}
    native.action("help.about",&json!({})).unwrap();
    for name in ["a.txt","b.txt"]{
        native.intent(&json!({"id":"left-tree","action":"select","value":{"key":dir.path().join(name)}})).unwrap();
        native.action("file.delete",&json!({})).unwrap();
    }
    assert_eq!(native.model()["values"]["form"]["title"],"About Dopus");
    native.answer("ok",None).unwrap();
    let first=native.model()["values"]["form"].clone();
    assert!(first["body"].as_str().unwrap().contains("a.txt"));
    native.answer("cancel",None).unwrap();
    let second=native.model()["values"]["form"].clone();
    assert!(second["body"].as_str().unwrap().contains("b.txt"));
    assert_ne!(first["instance"],second["instance"]);
    native.answer("ok",None).unwrap();
    let end=Instant::now()+Duration::from_secs(5);
    while native.model()["busy"]==true{native.poll(&ctx);assert!(Instant::now()<end);std::thread::sleep(Duration::from_millis(2));}
    assert!(dir.path().join("a.txt").exists());
    assert!(!dir.path().join("b.txt").exists());
    assert!(native.answer("ok",None).is_err());
}
