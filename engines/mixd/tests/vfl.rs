use media_vfl::{Frame, Pixels, Writer};
use std::{
    io::{BufRead, BufReader, Read, Write},
    process::{Command, Stdio},
};

#[test]
fn verify_schema_usage_codes_and_completion_policy() {
    let p = pipe_take("verify-policy", 4);
    let invoke = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_mixd"))
            .args(args)
            .output()
            .unwrap()
    };
    let path = p.to_str().unwrap();
    for args in [
        vec!["vfl"],
        vec!["vfl", "verify", path, "--bogus"],
        vec!["vfl", "export", path, "--from", "bad"],
        vec!["vfl", "export", path, "--fps", "0"],
        vec!["vfl", "verify", "--require-complete", "--require-complete"],
        vec!["vfl", "verify", "--bogus"],
        vec!["vfl", "markers", "--require-complete", path],
        vec!["vfl", "markers", "--require-complete"],
        vec!["vfl", "export", "--require-complete", path],
        vec!["vfl", "export", "--require-complete"],
    ] {
        let o = invoke(&args);
        assert_eq!(o.status.code(), Some(2), "{o:?}");
        assert!(String::from_utf8_lossy(&o.stderr).contains("Exit codes:"));
    }
    let o = invoke(&["vfl", "verify", path, "--require-complete"]);
    assert_eq!(o.status.code(), Some(0));
    assert_eq!(
        invoke(&["vfl", "verify", "--require-complete", path])
            .status
            .code(),
        Some(0)
    );
    std::fs::remove_file(p.join("complete.json")).unwrap();
    assert_eq!(invoke(&["vfl", "verify", path]).status.code(), Some(0));
    let o = invoke(&["vfl", "verify", path, "--require-complete"]);
    assert_eq!(o.status.code(), Some(1));
    let v: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
    assert_eq!(v["valid_frames"], 128);
    let e = &v["errors"][0];
    assert_eq!(e["kind"], "incomplete");
    assert!(e["file"].as_str().unwrap().ends_with("complete.json"));
    assert!(e["chunk"].is_null());
    assert!(e["offset"].is_u64());
    assert!(e["message"].is_string());
    let o = invoke(&["--personalities"]);
    assert!(
        !String::from_utf8_lossy(&o.stdout)
            .lines()
            .any(|l| l.starts_with("vfl "))
    );
    std::fs::remove_dir_all(p).unwrap();
}

#[test]
fn export_recovers_only_valid_frames_before_torn_record() {
    let p = pipe_take("torn-export", 4);
    std::fs::remove_file(p.join("complete.json")).unwrap();
    let chunk = p.join("chunk-000000.bin");
    let mut b = std::fs::read(&chunk).unwrap();
    b.truncate(b.len() - 1);
    std::fs::write(chunk, b).unwrap();
    let damaged = Command::new(env!("CARGO_BIN_EXE_mixd"))
        .args(["vfl", "export"])
        .arg(&p)
        .output()
        .unwrap();
    assert_eq!(damaged.status.code(), Some(1));
    let error = String::from_utf8_lossy(&damaged.stderr);
    assert!(error.contains("chunk 0 offset"));
    assert!(!error.contains("Some("));
    std::fs::remove_file(p.join("chunk-000000.json")).unwrap();
    let rows = std::fs::read_to_string(p.join("frames.jsonl")).unwrap();
    std::fs::write(
        p.join("frames.jsonl"),
        rows.lines().take(127).collect::<Vec<_>>().join("\n") + "\n",
    )
    .unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_mixd"))
        .args(["vfl", "export"])
        .arg(&p)
        .output()
        .unwrap();
    assert_eq!(o.status.code(), Some(0), "{o:?}");
    assert_eq!(o.stdout.len(), 127 * 4 * 4 * 4);
    std::fs::remove_dir_all(p).unwrap();
}

// More output than a pipe can hold, so closing the reader necessarily reaches
// the writer while it still has output left (without sleeps or timing races).
fn pipe_take(name: &str, side: u32) -> std::path::PathBuf {
    let p = std::env::temp_dir().join(format!("mixd-vfl-{name}-{}", std::process::id()));
    let mut w = Writer::new(p.clone(), side, side, 30).unwrap();
    w.origin(0).unwrap();
    w.frame(Frame {
        slot: 0,
        pixels: Pixels::Bgra(vec![0; (side * side * 4) as usize].into()),
    })
    .unwrap();
    w.finish(128, 0, 0, None, true).unwrap();
    // Keep marker generation independent of the recorder's marker policy.
    let mut markers = std::fs::File::create(p.join("markers.jsonl")).unwrap();
    for seq in 0..16384 {
        writeln!(
            markers,
            "{{\"seq\":{seq},\"t_us\":0,\"frame\":0,\"label\":\"test\",\"data\":null}}"
        )
        .unwrap();
    }
    p
}

#[test]
fn streaming_stdout_closed_early_is_quiet() {
    let p = pipe_take("closed-pipe", 128);
    for args in [
        vec!["export", "--format", "bgra"],
        vec!["export", "--format", "y4m"],
        vec!["markers"],
    ] {
        let mut child = Command::new(env!("CARGO_BIN_EXE_mixd"))
            .args(["vfl", args[0]])
            .arg(&p)
            .args(&args[1..])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        let mut reader = BufReader::new(child.stdout.take().unwrap());
        if args[0] == "markers" {
            let mut line = String::new();
            reader.read_line(&mut line).unwrap();
            assert!(line.contains("test"));
        } else {
            let size = if args.last() == Some(&"y4m") {
                let mut header = String::new();
                reader.read_line(&mut header).unwrap();
                assert!(header.starts_with("YUV4MPEG2"));
                reader.read_line(&mut String::new()).unwrap();
                128 * 128 * 3 / 2
            } else {
                128 * 128 * 4
            };
            reader.read_exact(&mut vec![0; size]).unwrap();
        }
        drop(reader);
        let o = child.wait_with_output().unwrap();
        assert_eq!(o.status.code(), Some(0), "{args:?}: {o:?}");
        assert!(o.stderr.is_empty(), "{args:?}: {o:?}");
    }
    std::fs::remove_dir_all(p).unwrap();
}

#[cfg(target_os = "linux")]
#[test]
fn streaming_stdout_other_errors_still_fail() {
    // A small frame also exercises errors from the final buffered flush.
    let p = pipe_take("full-output", 4);
    for command in ["export", "markers"] {
        let full = std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .unwrap();
        let o = Command::new(env!("CARGO_BIN_EXE_mixd"))
            .args(["vfl", command])
            .arg(&p)
            .stdout(full)
            .output()
            .unwrap();
        assert!(!o.status.success(), "{command}: {o:?}");
        assert!(!o.stderr.is_empty(), "{command}: {o:?}");
    }
    std::fs::remove_dir_all(p).unwrap();
}
fn ffmpeg(input: &[u8], args: &[&str]) -> Vec<u8> {
    let mut c = Command::new("ffmpeg")
        .args(["-v", "error"])
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    c.stdin.take().unwrap().write_all(input).unwrap();
    let o = c.wait_with_output().unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    o.stdout
}
#[test]
fn export_bgra_y4m_and_ranges() {
    check_export(false);
}

#[test]
fn export_ffmpeg_framemd5_y4m_oracle() {
    match Command::new("ffmpeg").arg("-version").output() {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            assert_ne!(
                std::env::var("MIX_TEST_REQUIRE_FFMPEG").as_deref(),
                Ok("1"),
                "ffmpeg required: framemd5 oracle cannot run"
            );
            eprintln!("ffmpeg absent: framemd5 oracle not run");
            return;
        }
        Err(e) => panic!("cannot run ffmpeg: {e}"),
        Ok(o) => assert!(o.status.success(), "ffmpeg probe failed: {o:?}"),
    }
    check_export(true);
}

fn check_export(oracle: bool) {
    let p = std::env::temp_dir().join(format!("mixd-vfl-test-{oracle}-{}", std::process::id()));
    let mut w = Writer::new(p.clone(), 4, 4, 30).unwrap();
    w.origin(0).unwrap();
    let a: Vec<u8> = (0..64).map(|i| i * 3).collect();
    let b: Vec<u8> = (0..64).map(|i| 255 - i * 3).collect();
    w.frame(Frame {
        slot: 0,
        pixels: Pixels::Bgra(a.clone().into()),
    })
    .unwrap();
    w.frame(Frame {
        slot: 2,
        pixels: Pixels::Bgra(b.clone().into()),
    })
    .unwrap();
    w.finish(3, 1, 0, None, true).unwrap();
    let export = |args: &[&str]| {
        let o = Command::new(env!("CARGO_BIN_EXE_mixd"))
            .args(["vfl", "export"])
            .arg(&p)
            .args(args)
            .output()
            .unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        o.stdout
    };
    let raw = export(&[]);
    let expected = [a.clone(), a.clone(), b.clone()].concat();
    assert_eq!(raw, expected);
    let mut reader = media_vfl::reader::Reader::open(&p).unwrap();
    assert_eq!(reader.frame_count().unwrap(), 3);
    let mut frames = Vec::new();
    while let Some(frame) = reader.read().unwrap() {
        frames.push(frame.pixels.to_vec());
    }
    let hash = |bytes: &[u8]| {
        use std::hash::{Hash, Hasher};
        let mut h = std::collections::hash_map::DefaultHasher::new();
        bytes.hash(&mut h);
        h.finish()
    };
    assert_eq!(raw.len(), 3 * 64);
    assert_eq!(
        raw.chunks_exact(64).map(hash).collect::<Vec<_>>(),
        frames.iter().map(|f| hash(f)).collect::<Vec<_>>()
    );
    let md5args = [
        "-f",
        "rawvideo",
        "-pixel_format",
        "bgra",
        "-video_size",
        "4x4",
        "-framerate",
        "30",
        "-i",
        "pipe:0",
        "-f",
        "framemd5",
        "pipe:1",
    ];
    if oracle {
        assert_eq!(ffmpeg(&raw, &md5args), ffmpeg(&expected, &md5args));
    }
    assert_eq!(export(&["--from", "1", "--to", "1"]), a);
    let y4m = export(&["--format", "y4m"]);
    let decoded = if oracle {
        ffmpeg(
            &y4m,
            &[
                "-i", "pipe:0", "-pix_fmt", "yuv420p", "-f", "rawvideo", "pipe:1",
            ],
        )
    } else {
        Vec::new()
    };
    let mut yuv = Vec::new();
    for pixels in &frames {
        let y = media_frame::Bgra::new(4, 4, pixels.clone())
            .unwrap()
            .to_yuv420();
        yuv.extend(y.y);
        yuv.extend(y.u);
        yuv.extend(y.v);
    }
    let header = b"YUV4MPEG2 W4 H4 F30:1 Ip A1:1 C420jpeg XCOLORRANGE=LIMITED\n";
    assert!(y4m.starts_with(header));
    assert_eq!(y4m.len(), header.len() + 3 * (6 + 24));
    for (record, pixels) in y4m[header.len()..]
        .chunks_exact(30)
        .zip(yuv.chunks_exact(24))
    {
        assert_eq!(&record[..6], b"FRAME\n");
        assert_eq!(hash(&record[6..]), hash(pixels));
    }
    if oracle {
        assert_eq!(decoded, yuv);
    }
    let o = Command::new(env!("CARGO_BIN_EXE_mixd"))
        .args(["vfl", "verify"])
        .arg(&p)
        .output()
        .unwrap();
    assert!(o.status.success());
    assert!(String::from_utf8(o.stdout).unwrap().contains("\"ok\":true"));
    for fps in ["0", "61"] {
        let o = Command::new(env!("CARGO_BIN_EXE_mixd"))
            .args(["vfl", "export"])
            .arg(&p)
            .args(["--fps", fps])
            .output()
            .unwrap();
        assert!(!o.status.success());
        assert!(String::from_utf8(o.stderr).unwrap().contains("1-60"));
    }
    std::fs::remove_file(p.join("complete.json")).unwrap();
    std::fs::remove_file(p.join("markers.jsonl")).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_mixd"))
        .args(["vfl", "markers"])
        .arg(&p)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    assert!(o.stdout.is_empty());
    std::fs::remove_dir_all(p).unwrap();
    let o = Command::new(env!("CARGO_BIN_EXE_mixd"))
        .args(["vfl", "verify", "/nonexistent-vfl-demo-take"])
        .output()
        .unwrap();
    assert!(!o.status.success());
    assert!(
        String::from_utf8(o.stdout)
            .unwrap()
            .contains("\"ok\":false")
    );
}
