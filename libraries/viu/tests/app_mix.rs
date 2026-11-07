// SPDX-License-Identifier: MIT OR Apache-2.0
use std::{path::PathBuf, sync::{Arc, mpsc}, time::Duration};
use viu::runtime::{Program, Runtime};

#[test]
fn every_app_mix_test_runs_with_real_host_bindings() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps");
    let temporary = tempfile::tempdir().unwrap();
    // Dopus's filesystem test explicitly requires a fresh, absent directory.
    mix::builtins::set_script_argv(vec![temporary.path().join("dopus").to_string_lossy().into_owned()]);
    let mut scripts = Vec::new();
    for app in std::fs::read_dir(&root).unwrap() {
        let app = app.unwrap().path();
        if !app.is_dir() { continue; }
        for entry in std::fs::read_dir(app).unwrap() {
            let path = entry.unwrap().path();
            if path.file_name().unwrap().to_string_lossy().ends_with("-test.mix") {
                scripts.push(path);
            }
        }
    }
    scripts.sort();
    assert!(!scripts.is_empty());
    for path in &scripts {
        let (tx, rx) = mpsc::channel();
        let mut runtime = Runtime::new(8, Arc::new(move || { let _ = tx.send(()); }), None);
        runtime.load(Program {
            path: path.clone(),
            source: std::fs::read_to_string(path).unwrap(),
            scene: "app-test".into(),
            citizen: "app-test".into(),
        }).unwrap();
        rx.recv_timeout(Duration::from_secs(30)).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let done = runtime.try_completion().unwrap();
        assert!(done.error.is_none(), "{}: {:?}\n{}\n{}", path.display(), done.error, done.stdout, done.stderr);
        println!("{}: {}", path.display(), done.stdout);
    }
    println!("APP_MIX_TESTS={}", scripts.len());
}
