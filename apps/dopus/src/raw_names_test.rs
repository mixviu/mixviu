// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;
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
    let app_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
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
    let initial = super::tests::wait(&runtime, &mut jobs, seq)
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
    let selected = super::tests::wait(&runtime, &mut jobs, seq)
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
    let (_, mut done) = super::tests::wait(&runtime, &mut jobs, seq);
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

#[test]
fn rust_comparator_uses_the_same_opaque_keys_and_persists_raw_folder_paths() {
    let dir = tempfile::tempdir().unwrap();
    let raw_folder = dir.path().join(std::ffi::OsStr::from_bytes(b"folder\xff"));
    std::fs::create_dir(&raw_folder).unwrap();
    let raw_file = raw_folder.join(std::ffi::OsStr::from_bytes(b"file\xfe.txt"));
    std::fs::write(&raw_file, b"raw comparator").unwrap();
    let profile = dir.path().join("profile");
    std::fs::create_dir(&profile).unwrap();
    std::fs::write(profile.join("config.conf.mix"),json!({"left":{"path":viu::file_data::encode_path(&raw_folder).unwrap(),"previews":false},"right":{"path":dir.path(),"previews":false}}).to_string()).unwrap();
    let contract: Value = serde_json::from_str(
        &std::fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("contract.json"))
            .unwrap(),
    )
    .unwrap();
    let mut native = native::Native::new(&profile, contract, Arc::new(|| {}));
    let ctx = egui::Context::default();
    let until = Instant::now() + std::time::Duration::from_secs(5);
    while native.model()["ready"] != true {
        native.poll(&ctx);
        assert!(Instant::now() < until);
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    let model = native.model();
    let key = viu::file_data::encode_path(&raw_file).unwrap();
    assert_eq!(model["values"]["left-tree"]["rows"][0]["key"], key);
    native
        .intent(&json!({"id":"left-tree","action":"select","value":{"key":key}}))
        .unwrap();
    assert_eq!(native.model()["panes"][0]["selected"], key);
    native.call("dopus.persist", &json!({})).unwrap();
    let saved: Value =
        serde_json::from_str(&std::fs::read_to_string(profile.join("config.conf.mix")).unwrap())
            .unwrap();
    assert_eq!(
        saved["left"]["path"],
        viu::file_data::encode_path(&raw_folder).unwrap()
    );
    native.action("file.copy-other-pane", &json!({})).unwrap();
    while !dir.path().join(raw_file.file_name().unwrap()).exists() {
        native.poll(&ctx);
        assert!(Instant::now() < until);
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert_eq!(
        std::fs::read(dir.path().join(raw_file.file_name().unwrap())).unwrap(),
        b"raw comparator"
    );
}
