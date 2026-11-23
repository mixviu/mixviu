use std::process::Command;
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
