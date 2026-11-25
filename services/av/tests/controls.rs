use serde_json::{Value, json};
use std::{
    fs,
    process::{Command, Output},
};

fn invoke(root: &std::path::Path, command: &str, flags: &[&str], name: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_av"))
        .arg(command)
        .arg(root.join("timeline.json"))
        .args(flags)
        .arg("--out")
        .arg(root.join(name))
        .output()
        .unwrap()
}
fn lines(output: &Output) -> Vec<Value> {
    String::from_utf8_lossy(&output.stderr)
        .lines()
        .filter_map(|l| l.strip_prefix("AVP "))
        .map(|l| serde_json::from_str(l).unwrap())
        .collect()
}

#[test]
fn progress_stderr_preserves_append_and_supports_sockets() {
    use std::{io::Read, os::unix::net::UnixStream, process::Stdio};
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("stderr.log");
    let existing = b"existing log bytes\n";
    fs::write(&path, existing).unwrap();
    let log = fs::OpenOptions::new().append(true).open(&path).unwrap();
    let command = || {
        let mut c = Command::new(env!("CARGO_BIN_EXE_av"));
        c.args([
            "render",
            "missing.json",
            "--progress",
            "--cpu",
            "0",
            "--out",
            "unused",
        ]);
        c
    };
    let status = command().stderr(Stdio::from(log)).status().unwrap();
    assert_eq!(status.code(), Some(2));
    let bytes = fs::read(path).unwrap();
    assert!(bytes.starts_with(existing));
    let text = String::from_utf8(bytes).unwrap();
    assert!(text.contains("av: cpu_limit\n"));
    assert!(text.contains("AVP "));

    let (mut reader, writer) = UnixStream::pair().unwrap();
    reader
        .set_read_timeout(Some(std::time::Duration::from_secs(5)))
        .unwrap();
    let status = command()
        .stderr(Stdio::from(std::os::fd::OwnedFd::from(writer)))
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(2));
    let mut text = String::new();
    reader.read_to_string(&mut text).unwrap();
    assert!(text.contains("AVP "));
    assert!(text.contains("av: cpu_limit\n"));
}

#[test]
fn literal_progress_filename_is_not_a_flag() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("--progress"), "{}").unwrap();
    // Reaching JSON validation proves the positional path was opened.
    let output = Command::new(env!("CARGO_BIN_EXE_av"))
        .current_dir(root.path())
        .args(["audio", "--progress", "--out", "output.webm"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("usage:"));
    assert!(String::from_utf8_lossy(&output.stderr).contains("missing field"));
    assert!(lines(&output).is_empty());
}
fn blocked_stderr_render(root: &std::path::Path) {
    use std::{
        io::Write,
        os::fd::{AsRawFd, FromRawFd},
        process::Stdio,
        time::{Duration, Instant},
    };
    let mut fds = [-1; 2];
    // SAFETY: valid two-descriptor array; successful descriptors are each
    // transferred exactly once to owned Files below.
    assert_eq!(
        unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_CLOEXEC | libc::O_NONBLOCK) },
        0
    );
    let reader = unsafe { fs::File::from_raw_fd(fds[0]) };
    let mut writer = unsafe { fs::File::from_raw_fd(fds[1]) };
    loop {
        match writer.write(&[b'x'; 4096]) {
            Ok(n) => assert!(n > 0),
            Err(e) => {
                assert_eq!(e.kind(), std::io::ErrorKind::WouldBlock);
                break;
            }
        }
    }
    // SAFETY: owned live pipe descriptor. Restore blocking mode so the AVP
    // writer really blocks; keep the unread read end alive until child exit.
    assert_eq!(
        unsafe { libc::fcntl(writer.as_raw_fd(), libc::F_SETFL, 0) },
        0
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("render")
        .arg(root.join("timeline.json"))
        .arg("--progress")
        .arg("--out")
        .arg(root.join("blocked.webm"))
        .stderr(Stdio::from(writer))
        .stdout(Stdio::from(
            fs::File::create(root.join("blocked-receipt.json")).unwrap(),
        ))
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        if Instant::now() >= deadline {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("render stalled on unread, full stderr pipe");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    drop(reader);
    assert_eq!(
        fs::read(root.join("blocked.webm")).unwrap(),
        fs::read(root.join("plain.webm")).unwrap()
    );
    let receipt: Value =
        serde_json::from_slice(&fs::read(root.join("blocked-receipt.json")).unwrap()).unwrap();
    assert_eq!(receipt["frames_out"], 30);
}
#[test]
fn video_progress_continues_through_overlapping_mux() {
    let root = tempfile::tempdir().unwrap();
    media_audio::Audio::new(48000, 1, vec![0.1; 48000])
        .unwrap()
        .write_wav(
            fs::File::create(root.path().join("tone.wav")).unwrap(),
            media_audio::WavFormat::Float32,
        )
        .unwrap();
    let timeline = json!({"version":1,"duration_s":8,
        "audio":{"sample_rate":48000,"channels":2,"sources":[{"path":"tone.wav","at_s":0}],
            "master":{"integrated_lufs":-14,"true_peak_dbtp":-1}},
        "video":{"width":256,"height":256,"fps":30,
            "segments":[{"solid":"black","at_s":0,"duration_s":8},
                {"solid":[255,255,255],"at_s":1,"duration_s":7}],
            "crossfades":[{"at_s":1,"duration_s":6}]},
        "encode":{"speed":8,"qp":100,"tile_cols":1,"tile_rows":1,
            "threads":1,"chunks":1,"keyint_s":1}});
    fs::write(root.path().join("timeline.json"), timeline.to_string()).unwrap();
    let output = invoke(
        root.path(),
        "render",
        &["--progress", "--progress"],
        "overlap.webm",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let events = lines(&output);
    let mux_start = events
        .iter()
        .position(|e| e["stage"] == "mux" && e["event"] == "start")
        .unwrap();
    let video_end = events
        .iter()
        .position(|e| e["stage"] == "video" && e["event"] == "end" && e.get("chunk").is_none())
        .unwrap();
    assert!(mux_start < video_end);
    assert!(
        events[mux_start + 1..video_end]
            .iter()
            .any(|e| e["stage"] == "video" && e["event"] == "progress"),
        "{events:#?}"
    );
    for stage in ["mux", "done"] {
        let end = events
            .iter()
            .position(|e| e["stage"] == stage && e["event"] == "end")
            .unwrap();
        assert!(
            !events[end + 1..]
                .iter()
                .any(|e| e["stage"] == "video" && e["event"] == "progress")
        );
    }
    assert_eq!(events.last().unwrap()["events_dropped"], 0);
}

#[test]
fn limits_progress_and_default_bytes() {
    let root = tempfile::tempdir().unwrap();
    let root = root.path();
    media_audio::Audio::new(
        48000,
        1,
        (0..48000)
            .map(|n| 0.1 * (n as f64 * 440.0 * std::f64::consts::TAU / 48000.0).sin())
            .collect(),
    )
    .unwrap()
    .write_wav(
        fs::File::create(root.join("tone.wav")).unwrap(),
        media_audio::WavFormat::Float32,
    )
    .unwrap();
    let audio = json!({"sample_rate":48000,"channels":2,"sources":[{"path":"tone.wav","at_s":0}],"master":{"integrated_lufs":-14,"true_peak_dbtp":-1}});
    let mut timeline = json!({"version":1,"duration_s":1,"audio":audio,
        "video":{"width":32,"height":32,"fps":30,"segments":[{"solid":"black","at_s":0,"duration_s":1}]},
        "encode":{"speed":10,"qp":100,"tile_cols":1,"tile_rows":1,"threads":1,"chunks":2,"keyint_s":0.5}});
    fs::write(root.join("timeline.json"), timeline.to_string()).unwrap();
    let plain = invoke(root, "render", &[], "plain.webm");
    assert!(
        plain.status.success(),
        "{}",
        String::from_utf8_lossy(&plain.stderr)
    );
    assert!(lines(&plain).is_empty());
    blocked_stderr_render(root);
    let progress = invoke(root, "render", &["--progress"], "progress.webm");
    assert!(
        progress.status.success(),
        "{}",
        String::from_utf8_lossy(&progress.stderr)
    );
    assert_eq!(
        fs::read(root.join("plain.webm")).unwrap(),
        fs::read(root.join("progress.webm")).unwrap()
    );
    let receipt: Value = serde_json::from_slice(&progress.stdout).unwrap();
    assert_eq!(receipt["parallel_chunks"], 2);
    assert_eq!(receipt["chunks"], 2);
    let events = lines(&progress);
    let mut stages = Vec::new();
    let (mut out, mut encoded, mut t, mut last_progress) = (0, 0, 0.0, None);
    for e in &events {
        assert_eq!(e["v"], 1);
        let stage = e["stage"].as_str().unwrap();
        if !stages.contains(&stage) {
            stages.push(stage);
        }
        let now = e["t"].as_f64().unwrap();
        assert!(now >= t);
        t = now;
        if e["event"] == "progress" {
            if let Some(last) = last_progress {
                assert!(now - last >= 0.49);
            }
            last_progress = Some(now);
        }
        if let Some(n) = e["frames_out"].as_u64() {
            assert!(n >= out);
            out = n;
        }
        if let Some(n) = e["frames_encoded"].as_u64() {
            assert!(n >= encoded);
            encoded = n;
        }
    }
    assert_eq!(stages, ["input", "audio", "video", "mux", "done"]);
    assert_eq!(out, receipt["frames_out"].as_u64().unwrap());
    assert_eq!(encoded, receipt["frames_encoded"].as_u64().unwrap());
    assert_eq!(events.last().unwrap()["ok"], true);
    assert_eq!(events.last().unwrap()["receipt"], "stdout");
    for chunk in 0..2 {
        for event in ["start", "end"] {
            assert!(
                events
                    .iter()
                    .any(|e| e["chunk"] == chunk && e["event"] == event)
            );
        }
    }
    timeline["encode"]["chunks"] = json!(2);
    timeline["encode"]["threads"] = json!(8);
    fs::write(root.join("timeline.json"), timeline.to_string()).unwrap();
    let limited = invoke(
        root,
        "render",
        &["--cpu", "1", "--mem", "1G", "--progress"],
        "limited.webm",
    );
    assert!(
        limited.status.success(),
        "{}",
        String::from_utf8_lossy(&limited.stderr)
    );
    let receipt: Value = serde_json::from_slice(&limited.stdout).unwrap();
    assert_eq!(receipt["encode_requested"]["threads"], 8);
    assert_eq!(receipt["encode_effective"]["chunks"], 1);
    assert_eq!(receipt["encode_effective"]["threads"], 1);
    assert_eq!(receipt["limits"]["cpu"], 1);
    if std::thread::available_parallelism().unwrap().get() >= 16 {
        timeline["encode"]["chunks"] = json!(4);
        fs::write(root.join("timeline.json"), timeline.to_string()).unwrap();
        let o = invoke(root, "render", &["--cpu", "16"], "cpu16.webm");
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        let r: Value = serde_json::from_slice(&o.stdout).unwrap();
        assert_eq!(r["encode_effective"]["chunks"], 2);
        assert_eq!(r["encode_effective"]["threads"], 8);
    }
    timeline["encode"]["chunks"] = json!(2);
    timeline["encode"]["threads"] = json!(1);
    fs::write(root.join("timeline.json"), timeline.to_string()).unwrap();
    let o = invoke(root, "render", &["--mem", "200M"], "memory.webm");
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    let r: Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(r["encode_effective"]["chunks"], 1);
    let above = (std::thread::available_parallelism().unwrap().get() + 1).to_string();
    let o = invoke(
        root,
        "render",
        &["--cpu", &above, "--progress"],
        "failed.webm",
    );
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("cpu_limit"));
    assert_eq!(lines(&o).last().unwrap()["ok"], false);
    for flags in [
        vec!["--cpu", "0"],
        vec!["--cpu", "bad"],
        vec!["--mem", "1"],
        vec!["--mem", "bad"],
    ] {
        let failed = invoke(root, "render", &flags, "failed.webm");
        assert_eq!(failed.status.code(), Some(2));
        assert!(!root.join("failed.webm").exists());
    }
    let mut audio = audio;
    audio["version"] = json!(1);
    audio["duration_s"] = json!(1);
    fs::write(root.join("timeline.json"), audio.to_string()).unwrap();
    let a = invoke(
        root,
        "audio",
        &["--cpu", "1", "--mem", "1G", "--progress"],
        "audio.webm",
    );
    assert!(a.status.success(), "{}", String::from_utf8_lossy(&a.stderr));
    assert_eq!(lines(&a).last().unwrap()["ok"], true);
    let b = invoke(root, "audio", &[], "audio-plain.webm");
    assert!(b.status.success());
    assert!(lines(&b).is_empty());
    for option in ["--out", "--wav"] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_av"));
        command.current_dir(root).args(["audio", "timeline.json"]);
        if option == "--wav" {
            command.args(["--out", "named-path.webm"]);
        }
        let output = command.args([option, "--progress"]).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(lines(&output).is_empty());
        assert!(root.join("--progress").is_file());
        fs::remove_file(root.join("--progress")).unwrap();
    }
    assert_eq!(
        fs::read(root.join("audio.webm")).unwrap(),
        fs::read(root.join("audio-plain.webm")).unwrap()
    );
    let failed = invoke(root, "audio", &["--progress", "--cpu", "0"], "failed.webm");
    assert_eq!(failed.status.code(), Some(2));
    assert_eq!(lines(&failed).last().unwrap()["ok"], false);
    for value in ["bad", "-1", "1g", "18446744073709551615G", ""] {
        let failed = invoke(root, "audio", &["--mem", value], "invalid-mem.webm");
        assert_eq!(failed.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&failed.stderr).contains("av: mem_limit"));
        assert!(!root.join("invalid-mem.webm").exists());
    }
    let zero = invoke(root, "audio", &["--mem", "0"], "audio-zero.webm");
    assert!(zero.status.success());
    assert_eq!(
        fs::read(root.join("audio-zero.webm")).unwrap(),
        fs::read(root.join("audio-plain.webm")).unwrap()
    );
    for command in ["render", "audio"] {
        for (flags, error) in [
            (
                vec!["--cpu", "1", "--cpu", "1"],
                "unknown_or_duplicate_option",
            ),
            (
                vec!["--mem", "1G", "--mem", "1G"],
                "unknown_or_duplicate_option",
            ),
            (vec!["--unknown", "x"], "unknown_or_duplicate_option"),
        ] {
            let failed = invoke(root, command, &flags, "invalid-options.webm");
            assert_eq!(failed.status.code(), Some(2));
            assert_eq!(
                String::from_utf8_lossy(&failed.stderr).trim(),
                format!("av: {error}")
            );
        }
        let failed = Command::new(env!("CARGO_BIN_EXE_av"))
            .arg(command)
            .arg(root.join("timeline.json"))
            .args(["--out", "unused", "--mem"])
            .output()
            .unwrap();
        assert_eq!(failed.status.code(), Some(2));
        assert_eq!(
            String::from_utf8_lossy(&failed.stderr).trim(),
            "av: missing_option_value"
        );
    }
}
