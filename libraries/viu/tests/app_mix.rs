// SPDX-License-Identifier: MIT OR Apache-2.0
use std::{
    path::PathBuf,
    sync::{Arc, mpsc},
    time::Duration,
};
use viu::runtime::{Program, Runtime};

#[test]
fn every_app_mix_test_runs_with_real_host_bindings() {
    if let Ok(script) = std::env::var("MIX_TEST_VIU_APP_SCRIPT") {
        // set_script_argv is intentionally write-once. Each script owns a
        // fresh process, so its argv is discarded even on a panic.
        run_script(&PathBuf::from(script));
        return;
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../apps");
    let mut scripts = Vec::new();
    for app in std::fs::read_dir(&root).unwrap() {
        let app = app.unwrap().path();
        if !app.is_dir() {
            continue;
        }
        for entry in std::fs::read_dir(app).unwrap() {
            let path = entry.unwrap().path();
            if path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .ends_with("-test.mix")
            {
                scripts.push(path);
            }
        }
    }
    scripts.sort();
    assert!(!scripts.is_empty());
    for path in &scripts {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "every_app_mix_test_runs_with_real_host_bindings",
                "--nocapture",
            ])
            .env("MIX_TEST_VIU_APP_SCRIPT", path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}:\n{}\n{}",
            path.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        print!("{}", String::from_utf8_lossy(&output.stdout));
    }
    println!("APP_MIX_TESTS={}", scripts.len());
}

fn run_script(path: &std::path::Path) {
    let started = std::time::Instant::now();
    let temporary = tempfile::tempdir().unwrap();
    let argv = if path.ends_with("dopus/core-test.mix") {
        vec![
            temporary
                .path()
                .join("filesystem")
                .to_string_lossy()
                .into_owned(),
        ]
    } else {
        Vec::new()
    };
    mix::builtins::set_script_argv(argv);
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::with_services(
        8,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
        viu::runtime::Services {
            #[cfg(feature = "test-support")]
            test_cpu_probe: std::env::var_os("MIX_TEST_SECTION_TIMING").is_some(),
            // Exercise the actual 8 MiB queue cap, rather than lowering it.
            // Other app scripts retain their ordinary 1 MiB string limit.
            max_string_bytes: path
                .ends_with("workshop/redaction-budget-test.mix")
                .then_some(16 * 1024 * 1024),
            ..Default::default()
        },
    );
    runtime
        .load(Program {
            path: path.to_path_buf(),
            source: std::fs::read_to_string(path).unwrap(),
            scene: "app-test".into(),
            citizen: "app-test".into(),
        })
        .unwrap();
    rx.recv_timeout(Duration::from_secs(30))
        .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let done = runtime.try_completion().unwrap();
    assert!(
        done.error.is_none(),
        "{}: {:?}\n{}\n{}",
        path.display(),
        done.error,
        done.stdout,
        done.stderr
    );
    println!("{}: {}", path.display(), done.stdout);
    println!("APP_SCRIPT_TIME {} {:.6}", path.display(), started.elapsed().as_secs_f64());
}
