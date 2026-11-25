use serde_json::{Value, json};
use std::{fs, process::{Command, Output}};

fn invoke(root: &std::path::Path, command: &str, flags: &[&str], name: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_av"))
        .arg(command).arg(root.join("timeline.json")).args(flags)
        .arg("--out").arg(root.join(name)).output().unwrap()
}
fn lines(output: &Output) -> Vec<Value> {
    String::from_utf8_lossy(&output.stderr).lines()
        .filter_map(|l| l.strip_prefix("AVP "))
        .map(|l| serde_json::from_str(l).unwrap()).collect()
}
#[test]
fn limits_progress_and_default_bytes() {
    let root = tempfile::tempdir().unwrap();
    let root = root.path();
    media_audio::Audio::new(48000, 1, (0..48000)
        .map(|n| 0.1 * (n as f64 * 440.0 * std::f64::consts::TAU / 48000.0).sin()).collect())
        .unwrap().write_wav(fs::File::create(root.join("tone.wav")).unwrap(), media_audio::WavFormat::Float32).unwrap();
    let audio = json!({"sample_rate":48000,"channels":2,"sources":[{"path":"tone.wav","at_s":0}],"master":{"integrated_lufs":-14,"true_peak_dbtp":-1}});
    let mut timeline = json!({"version":1,"duration_s":1,"audio":audio,
        "video":{"width":32,"height":32,"fps":30,"segments":[{"solid":"black","at_s":0,"duration_s":1}]},
        "encode":{"speed":10,"qp":100,"tile_cols":1,"tile_rows":1,"threads":1,"chunks":1,"keyint_s":0.5}});
    fs::write(root.join("timeline.json"), timeline.to_string()).unwrap();
    let plain = invoke(root, "render", &[], "plain.webm");
    assert!(plain.status.success(), "{}", String::from_utf8_lossy(&plain.stderr));
    assert!(lines(&plain).is_empty());
    let progress = invoke(root, "render", &["--progress"], "progress.webm");
    assert!(progress.status.success(), "{}", String::from_utf8_lossy(&progress.stderr));
    assert_eq!(fs::read(root.join("plain.webm")).unwrap(), fs::read(root.join("progress.webm")).unwrap());
    let receipt: Value = serde_json::from_slice(&progress.stdout).unwrap();
    let events = lines(&progress);
    let mut stages = Vec::new();
    let (mut out, mut encoded, mut t, mut last_progress) = (0, 0, 0.0, None);
    for e in &events {
        assert_eq!(e["v"], 1);
        let stage = e["stage"].as_str().unwrap();
        if !stages.contains(&stage) { stages.push(stage); }
        let now = e["t"].as_f64().unwrap();
        assert!(now >= t); t = now;
        if e["event"] == "progress" {
            if let Some(last) = last_progress { assert!(now - last >= 0.49); }
            last_progress = Some(now);
        }
        if let Some(n) = e["frames_out"].as_u64() { assert!(n >= out); out = n; }
        if let Some(n) = e["frames_encoded"].as_u64() { assert!(n >= encoded); encoded = n; }
    }
    assert_eq!(stages, ["input", "audio", "video", "mux", "done"]);
    assert_eq!(out, receipt["frames_out"].as_u64().unwrap());
    assert_eq!(encoded, receipt["frames_encoded"].as_u64().unwrap());
    assert_eq!(events.last().unwrap()["ok"], true);
    assert_eq!(events.last().unwrap()["receipt"], "stdout");
    for chunk in 0..2 {
        for event in ["start", "end"] {
            assert!(events.iter().any(|e| e["chunk"] == chunk && e["event"] == event));
        }
    }
    timeline["encode"]["chunks"] = json!(2);
    timeline["encode"]["threads"] = json!(8);
    fs::write(root.join("timeline.json"), timeline.to_string()).unwrap();
    let limited = invoke(root, "render", &["--cpu", "1", "--mem", "1G", "--progress"], "limited.webm");
    assert!(limited.status.success(), "{}", String::from_utf8_lossy(&limited.stderr));
    let receipt: Value = serde_json::from_slice(&limited.stdout).unwrap();
    assert_eq!(receipt["encode_requested"]["threads"], 8);
    assert_eq!(receipt["encode_effective"]["chunks"], 1);
    assert_eq!(receipt["encode_effective"]["threads"], 1);
    assert_eq!(receipt["limits"]["cpu"], 1);
    for flags in [vec!["--cpu", "0"], vec!["--cpu", "bad"], vec!["--mem", "1"], vec!["--mem", "bad"]] {
        let failed = invoke(root, "render", &flags, "failed.webm");
        assert_eq!(failed.status.code(), Some(2));
        assert!(!root.join("failed.webm").exists());
    }
    let mut audio = audio;
    audio["version"] = json!(1); audio["duration_s"] = json!(1);
    fs::write(root.join("timeline.json"), audio.to_string()).unwrap();
    let a = invoke(root, "audio", &["--cpu", "1", "--progress"], "audio.webm");
    assert!(a.status.success(), "{}", String::from_utf8_lossy(&a.stderr));
    assert_eq!(lines(&a).last().unwrap()["ok"], true);
    let b = invoke(root, "audio", &[], "audio-plain.webm");
    assert!(b.status.success()); assert!(lines(&b).is_empty());
    assert_eq!(fs::read(root.join("audio.webm")).unwrap(), fs::read(root.join("audio-plain.webm")).unwrap());
    let failed = invoke(root, "audio", &["--progress", "--cpu", "0"], "failed.webm");
    assert_eq!(failed.status.code(), Some(2));
    assert_eq!(lines(&failed).last().unwrap()["ok"], false);
}
