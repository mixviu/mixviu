use std::process::Command;
#[test]
fn fifo_timeline_returns_promptly() {
    let dir = std::env::temp_dir().join(format!("av-fifo-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let input = dir.join("timeline.json");
    let name = std::ffi::CString::new(input.as_os_str().as_encoded_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
    let mut child = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("audio")
        .arg(&input)
        .arg("--out")
        .arg(dir.join("out.webm"))
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
    while child.try_wait().unwrap().is_none() {
        if std::time::Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("FIFO timeline open blocked");
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let result = child.wait_with_output().unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("not a regular file"));
    std::fs::remove_dir_all(dir).unwrap();
}
#[test]
fn av_errors_are_clean() {
    for args in [
        vec![],
        vec!["audio"],
        vec!["unknown", "x", "--out", "y"],
        vec!["audio", "missing.json", "--out", "x", "--out", "y"],
    ] {
        let o = Command::new(env!("CARGO_BIN_EXE_av"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(o.status.code(), Some(2));
        assert!(o.stdout.is_empty());
        let error = String::from_utf8(o.stderr).unwrap();
        assert!(error.starts_with("av: "), "{error}");
        assert!(!error.contains("panicked"), "{error}");
    }
}

#[test]
fn refusal_paths_size_and_atomic_outputs() {
    use media_audio::{Audio, WavFormat};
    use serde_json::{Value, json};
    let dir = std::env::temp_dir().join(format!("av-review-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
    let input = dir.join("timeline.json");
    let out = dir.join("out.webm");
    let wav = dir.join("out.wav");
    let invoke = || {
        Command::new(env!("CARGO_BIN_EXE_av"))
            .arg("audio")
            .arg(&input)
            .arg("--out")
            .arg(&out)
            .arg("--wav")
            .arg(&wav)
            .output()
            .unwrap()
    };
    let mut timeline = json!({"version":1,"sample_rate":48000,"channels":2,"duration_s":4,"sources":[{"path":"source.wav","at_s":0}],"master":{"integrated_lufs":-14,"true_peak_dbtp":-1}});
    for path in ["/tmp/av-absolute.wav", "../escape.wav", "sub/../escape.wav"] {
        timeline["sources"] = json!([{"path":"missing.wav","at_s":0},{"path":path,"at_s":0}]);
        std::fs::write(&input, serde_json::to_vec(&timeline).unwrap()).unwrap();
        let o = invoke();
        assert_eq!(o.status.code(), Some(2));
        assert!(
            String::from_utf8(o.stderr)
                .unwrap()
                .contains("sources[1].path")
        );
        assert!(!out.exists() && !wav.exists());
    }
    let file = std::fs::File::create(&input).unwrap();
    file.set_len(16 * 1024 * 1024 + 1).unwrap();
    drop(file);
    let o = invoke();
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8(o.stderr).unwrap().contains("16 MiB"));
    let samples = (0..48000 * 4)
        .map(|n| {
            if n % 48000 == 0 {
                0.9
            } else {
                0.01 * (std::f64::consts::TAU * 440.0 * n as f64 / 48000.0).sin()
            }
        })
        .collect();
    Audio::new(48000, 1, samples)
        .unwrap()
        .write_wav(
            std::fs::File::create(dir.join("source.wav")).unwrap(),
            WavFormat::Float32,
        )
        .unwrap();
    timeline["sources"] = json!([{"path":"source.wav","at_s":0}]);
    std::fs::write(&input, serde_json::to_vec(&timeline).unwrap()).unwrap();
    let o = invoke();
    assert_eq!(
        o.status.code(),
        Some(3),
        "{}",
        String::from_utf8_lossy(&o.stderr)
    );
    let r: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert!(r["refused"] == true || r["target_reached"] == false);
    assert!(
        String::from_utf8(o.stderr)
            .unwrap()
            .starts_with("av: mastering refused:")
    );
    assert!(!out.exists() && !wav.exists());
    // Receipt delivery cannot turn an unpublished refusal into success.
    let mut child = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("audio")
        .arg(&input)
        .arg("--out")
        .arg(&out)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    let closed = child.wait_with_output().unwrap();
    assert_eq!(
        closed.status.code(),
        Some(3),
        "{}",
        String::from_utf8_lossy(&closed.stderr)
    );
    assert!(String::from_utf8_lossy(&closed.stderr).starts_with("av: mastering refused:"));
    assert!(!out.exists());
    // Neither stdout nor stderr ENOSPC may override the refusal outcome.
    let full = || std::fs::File::options().write(true).open("/dev/full").unwrap();
    let undelivered = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("audio").arg(&input).arg("--out").arg(&out)
        .stdout(full()).stderr(full()).status().unwrap();
    assert_eq!(undelivered.code(), Some(3));
    assert!(!out.exists());
    assert_eq!(
        std::fs::read_dir(&dir).unwrap().count(),
        2,
        "temporary files must be removed"
    );
    // Error during staging must also remove a previously staged WAV and must
    // preserve the pre-existing destination.
    timeline["master"]["integrated_lufs"] = json!(-40);
    std::fs::write(&input, serde_json::to_vec(&timeline).unwrap()).unwrap();
    std::fs::write(&out, b"keep").unwrap();
    let o = invoke();
    assert_eq!(o.status.code(), Some(2));
    assert_eq!(std::fs::read(&out).unwrap(), b"keep");
    assert!(!wav.exists());
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 3);
    std::fs::remove_file(&out).unwrap();
    let lost = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("audio")
        .arg(&input)
        .arg("--out")
        .arg(&out)
        .stdout(
            std::fs::File::options()
                .write(true)
                .open("/dev/full")
                .unwrap(),
        )
        .output()
        .unwrap();
    assert_eq!(
        lost.status.code(),
        Some(4),
        "{}",
        String::from_utf8_lossy(&lost.stderr)
    );
    assert!(
        String::from_utf8_lossy(&lost.stderr).starts_with("av: published but receipt not written:")
    );
    assert!(out.is_file());
    std::fs::remove_file(&out).unwrap();
    let mut published = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("audio").arg(&input).arg("--out").arg(&out)
        .stdout(std::process::Stdio::piped()).stderr(std::process::Stdio::piped())
        .spawn().unwrap();
    drop(published.stdout.take());
    let published = published.wait_with_output().unwrap();
    assert_eq!(published.status.code(), Some(0));
    assert!(published.stderr.is_empty());
    assert!(out.is_file());
    Audio::new(48000, 1, vec![f32::MAX as f64; 8])
        .unwrap()
        .write_wav(
            std::fs::File::create(dir.join("source.wav")).unwrap(),
            WavFormat::Float32,
        )
        .unwrap();
    timeline["sources"][0]["gain_db"] = json!(60);
    std::fs::write(&input, serde_json::to_vec(&timeline).unwrap()).unwrap();
    let o = invoke();
    assert_eq!(o.status.code(), Some(2));
    assert!(
        String::from_utf8(o.stderr)
            .unwrap()
            .contains("sources[0].mix: non-finite sum")
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn timeline_symlink_uses_opened_files_real_directory() {
    use media_audio::{Audio, WavFormat};
    let dir = std::env::temp_dir().join(format!("av-timeline-link-{}", std::process::id()));
    let real = dir.join("real");
    let alias = dir.join("alias");
    std::fs::create_dir_all(&real).unwrap();
    std::fs::create_dir(&alias).unwrap();
    let timeline = serde_json::json!({"version":1,"sample_rate":48000,"channels":2,"duration_s":1,"sources":[{"path":"source.wav","at_s":0}],"master":{"integrated_lufs":-40,"true_peak_dbtp":-1}});
    std::fs::write(real.join("timeline.json"),serde_json::to_vec(&timeline).unwrap()).unwrap();
    Audio::new(48000,1,(0..48000).map(|n|0.1*(std::f64::consts::TAU*440.0*n as f64/48000.0).sin()).collect()).unwrap()
        .write_wav(std::fs::File::create(real.join("source.wav")).unwrap(),WavFormat::Float32).unwrap();
    std::fs::write(alias.join("source.wav"), b"wrong lexical directory").unwrap();
    std::os::unix::fs::symlink("../real/timeline.json",alias.join("link.json")).unwrap();
    let out = dir.join("out.webm");
    let result = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("audio").arg(alias.join("link.json")).arg("--out").arg(&out).output().unwrap();
    assert_eq!(result.status.code(),Some(0),"{}",String::from_utf8_lossy(&result.stderr));
    assert!(out.is_file());
    std::fs::remove_dir_all(dir).unwrap();
}
