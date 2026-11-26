use serde_json::{Value, json};
use std::{fs, process::Command};
#[test]
fn black_vfl_still_black_decoded_frame_hashes() {
    use std::hash::{Hash, Hasher};
    let root = tempfile::tempdir().unwrap();
    let mut w = media_vfl::Writer::new(root.path().join("take.vfl"), 32, 32, 30).unwrap();
    w.origin(0).unwrap();
    w.frame(media_vfl::Frame {
        slot: 0,
        pixels: media_vfl::Pixels::Bgra([10, 80, 180, 255].repeat(32 * 32).into()),
    })
    .unwrap();
    w.finish(120, 0, 0, None, true).unwrap();
    let mut e = png::Encoder::new(
        fs::File::create(root.path().join("step.png")).unwrap(),
        16,
        8,
    );
    e.set_color(png::ColorType::Rgb);
    e.write_header()
        .unwrap()
        .write_image_data(&[30, 180, 60].repeat(16 * 8))
        .unwrap();
    media_audio::Audio::new(
        48000,
        1,
        (0..192000)
            .map(|n| 0.1 * (n as f64 * 440.0 * std::f64::consts::TAU / 48000.0).sin())
            .collect(),
    )
    .unwrap()
    .write_wav(
        fs::File::create(root.path().join("tone.wav")).unwrap(),
        media_audio::WavFormat::Float32,
    )
    .unwrap();
    let t = json!({"version":1,"duration_s":4,"video":{"width":32,"height":32,"fps":30,
        "segments":[{"solid":"black","at_s":0,"duration_s":0.6},
            {"path":"take.vfl","from_slot":0,"to_slot":120,"at_s":0.2},
            {"still":"step.png","at_s":1.3,"duration_s":1.2},
            {"solid":[0,0,0],"at_s":2.5,"duration_s":1.5}],
        "crossfades":[{"at_s":0.2,"duration_s":0.6},{"at_s":1.3,"duration_s":0.3},{"at_s":2.5,"duration_s":1.0}]},
        "audio":{"sample_rate":48000,"channels":2,"sources":[{"path":"tone.wav","at_s":0}],"master":{"integrated_lufs":-14,"true_peak_dbtp":-1}},
        "encode":{"speed":10,"qp":100,"tile_cols":1,"tile_rows":1,"threads":1,"chunks":1,"keyint_s":60}});
    let timeline = root.path().join("timeline.json");
    fs::write(&timeline, serde_json::to_vec(&t).unwrap()).unwrap();
    let out = root.path().join("out.webm");
    let render = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("render")
        .arg(&timeline)
        .arg("--out")
        .arg(&out)
        .output()
        .unwrap();
    assert!(
        render.status.success(),
        "{}",
        String::from_utf8_lossy(&render.stderr)
    );
    let receipt: Value = serde_json::from_slice(&render.stdout).unwrap();
    let hash = Command::new("sha256sum").arg(&out).output().unwrap();
    assert!(hash.status.success());
    println!(
        "RENDER_WEBM_SHA256 {}",
        String::from_utf8_lossy(&hash.stdout)
    );
    assert_eq!(
        String::from_utf8_lossy(&hash.stdout)
            .split_whitespace()
            .next(),
        Some("a8e1a7067438aef5fbd4b2e10a23aa653ba924f3ac61e5e8ac3f7a930f610381")
    );
    assert_eq!(receipt["frames_out"], 120);
    assert!(receipt["repeats"].as_u64().unwrap() > 40);
    assert!(receipt["scaling"]["frames"].as_u64().unwrap() > 0);
    let y4m = root.path().join("decoded.y4m");
    let decode = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("decode")
        .arg(&out)
        .arg("--y4m")
        .arg(&y4m)
        .output()
        .unwrap();
    assert!(
        decode.status.success(),
        "{}",
        String::from_utf8_lossy(&decode.stderr)
    );
    let receipt: Value = serde_json::from_slice(&decode.stdout).unwrap();
    assert_eq!(receipt["frames"], 120);
    let bytes = fs::read(y4m).unwrap();
    let header = bytes.iter().position(|b| *b == b'\n').unwrap() + 1;
    assert!(
        std::str::from_utf8(&bytes[..header])
            .unwrap()
            .contains(" F30:1 ")
    );
    let mut frames = Vec::new();
    let mut offset = header;
    while offset < bytes.len() {
        let end = offset + bytes[offset..].iter().position(|b| *b == b'\n').unwrap() + 1;
        assert!(bytes[offset..end].starts_with(b"FRAME XPTS="));
        let mut h = std::collections::hash_map::DefaultHasher::new();
        bytes[end..end + 32 * 32 * 3 / 2].hash(&mut h);
        frames.push(h.finish());
        offset = end + 32 * 32 * 3 / 2;
    }
    assert_eq!(frames.len(), 120);
    for range in [0..6, 24..39, 48..75, 105..120] {
        for k in range.clone() {
            assert_eq!(frames[k], frames[range.start], "held frame {k}");
        }
    }
    assert_eq!(frames[0], frames[119]);
    assert_ne!(frames[0], frames[30]);
    assert_ne!(frames[30], frames[60]);
    for range in [6..24, 39..48, 75..105] {
        assert!(frames[range].windows(2).any(|p| p[0] != p[1]));
    }
    println!(
        "black/VFL/still/black decoded hashes: {:x} {:x} {:x} {:x}",
        frames[0], frames[30], frames[60], frames[119]
    );
}
#[test]
fn repeats_chunks_scaled_fade_and_gap() {
    let root = std::env::temp_dir().join(format!("av-render-{}", std::process::id()));
    fs::create_dir(&root).unwrap();
    for (name, size, colour) in [
        ("a.vfl", 32, [10, 80, 180, 255]),
        ("b.vfl", 16, [180, 90, 10, 255]),
    ] {
        let mut w = media_vfl::Writer::new(root.join(name), size, size, 30).unwrap();
        w.origin(0).unwrap();
        w.frame(media_vfl::Frame {
            slot: 0,
            pixels: media_vfl::Pixels::Bgra(
                vec![colour; (size * size) as usize]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .into(),
            ),
        })
        .unwrap();
        w.frame(media_vfl::Frame {
            slot: 5,
            pixels: media_vfl::Pixels::Bgra(
                vec![[20, 200, 40, 255]; (size * size) as usize]
                    .into_iter()
                    .flatten()
                    .collect::<Vec<_>>()
                    .into(),
            ),
        })
        .unwrap();
        w.finish(90, 0, 0, None, true).unwrap();
    }
    media_audio::Audio::new(
        48000,
        1,
        (0..192000)
            .map(|n| 0.1 * (n as f64 * 440.0 * std::f64::consts::TAU / 48000.0).sin())
            .collect(),
    )
    .unwrap()
    .write_wav(
        fs::File::create(root.join("tone.wav")).unwrap(),
        media_audio::WavFormat::Float32,
    )
    .unwrap();
    let t = json!({"version":1,"duration_s":4,"video":{"width":32,"height":32,"fps":30,"segments":[{"path":"a.vfl","from_slot":0,"to_slot":60,"at_s":0},{"path":"b.vfl","from_slot":0,"to_slot":30,"at_s":1.5}],"crossfades":[{"at_s":1.5,"duration_s":0.5}]},"audio":{"sample_rate":48000,"channels":2,"sources":[{"path":"tone.wav","at_s":0}],"master":{"integrated_lufs":-14,"true_peak_dbtp":-1}},"encode":{"speed":10,"qp":100,"tile_cols":1,"tile_rows":1,"threads":1,"chunks":2,"keyint_s":1}});
    for (pointer, value, name) in [
        ("/version", json!(2), "version"),
        ("/duration_s", json!(0), "duration_s"),
        ("/video/fps", json!(25), "fps"),
        ("/video/width", json!(0), "dimensions"),
        ("/encode/chunks", json!(0), "encode_settings"),
        ("/encode/keyint_s", json!(0), "encode_settings"),
        ("/encode/tile_cols", json!(3), "encode_tiles_threads"),
        (
            "/video/segments/0/path",
            json!("../a.vfl"),
            "segment_path_relative",
        ),
        (
            "/video/segments/0/at_s",
            json!(0.1),
            "gap_before_first_segment",
        ),
        (
            "/video/segments/0/to_slot",
            json!(100),
            "segment_trim_outside_vfl",
        ),
        (
            "/video/crossfades/0/at_s",
            json!(1.0),
            "crossfade_segment_start",
        ),
        (
            "/video/crossfades/0/duration_s",
            json!(3.0),
            "crossfade_range",
        ),
    ] {
        let mut bad = t.clone();
        *bad.pointer_mut(pointer).unwrap() = value;
        fs::write(
            root.join("timeline.json"),
            serde_json::to_vec(&bad).unwrap(),
        )
        .unwrap();
        let o = Command::new(env!("CARGO_BIN_EXE_av"))
            .arg("render")
            .arg(root.join("timeline.json"))
            .arg("--out")
            .arg(root.join("out.webm"))
            .output()
            .unwrap();
        assert_eq!(o.status.code(), Some(2));
        assert!(
            String::from_utf8_lossy(&o.stderr).contains(name),
            "{}",
            String::from_utf8_lossy(&o.stderr)
        );
        assert!(!root.join("out.webm").exists());
    }
    fs::write(root.join("timeline.json"), serde_json::to_vec(&t).unwrap()).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("render")
        .arg(root.join("timeline.json"))
        .arg("--out")
        .arg(root.join("out.webm"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let receipt: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(receipt["frames_out"], 120);
    assert!(receipt["peak_rss_kib"].as_u64().unwrap() > 0);
    assert!(receipt["frames_encoded"].as_u64().unwrap() < 45);
    let bytes = fs::read(root.join("out.webm")).unwrap();
    let demux = media_webm::Demuxer::new(&bytes).unwrap();
    let video: Vec<_> = demux.packets().iter().filter(|p| p.track == 1).collect();
    let audio: Vec<_> = demux.packets().iter().filter(|p| p.track == 2).collect();
    assert_eq!(video.len(), 120);
    assert_eq!(video.iter().map(|p| p.duration.unwrap()).sum::<u64>(), 4000);
    assert_eq!(
        video.iter().filter(|p| p.key_frame).count() as u64,
        receipt["keyframes"].as_u64().unwrap()
    );
    let probe = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("probe")
        .arg(root.join("out.webm"))
        .output()
        .unwrap();
    assert!(
        probe.status.success(),
        "{}",
        String::from_utf8_lossy(&probe.stderr)
    );
    let probe: Value = serde_json::from_slice(&probe.stdout).unwrap();
    assert_eq!(probe["frames"], 120);
    assert_eq!(probe["duration"], 4.0);
    let decoded = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("decode")
        .arg(root.join("out.webm"))
        .arg("--y4m")
        .arg(root.join("own.y4m"))
        .output()
        .unwrap();
    assert!(
        decoded.status.success(),
        "{}",
        String::from_utf8_lossy(&decoded.stderr)
    );
    let decoded: Value = serde_json::from_slice(&decoded.stdout).unwrap();
    assert_eq!(decoded["frames"], 120);
    assert_eq!(video[0].timestamp, 0);
    assert_eq!(audio[0].timestamp, 0);
    assert!((video.last().unwrap().timestamp - audio.last().unwrap().timestamp).abs() <= 34);
    let oracle = Command::new("ffmpeg")
        .args(["-v", "error", "-c:v", "libdav1d", "-i"])
        .arg(root.join("out.webm"))
        .args([
            "-map",
            "0:v:0",
            "-fps_mode",
            "passthrough",
            "-f",
            "framemd5",
            "-",
        ])
        .output();
    match oracle {
        Err(e) if std::env::var_os("MIX_TEST_REQUIRE_FFMPEG").is_none() => {
            eprintln!("ffmpeg unavailable: {e}")
        }
        result => {
            let o = result.unwrap();
            assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
            let text = String::from_utf8(o.stdout).unwrap();
            let hashes: Vec<_> = text
                .lines()
                .filter(|l| !l.starts_with('#') && !l.is_empty())
                .map(|l| l.rsplit(',').next().unwrap().trim())
                .collect();
            assert_eq!(hashes.len(), 120);
            let own = Command::new("ffmpeg")
                .args(["-v", "error", "-i"])
                .arg(root.join("own.y4m"))
                .args(["-fps_mode", "passthrough", "-f", "framemd5", "-"])
                .output()
                .unwrap();
            assert!(own.status.success());
            let own = String::from_utf8(own.stdout).unwrap();
            let own: Vec<_> = own
                .lines()
                .filter(|l| !l.starts_with('#') && !l.is_empty())
                .map(|l| l.rsplit(',').next().unwrap().trim())
                .collect();
            assert_eq!(hashes, own);
            assert_ne!(hashes[4], hashes[5]);
            // All holds must remain identical, including both sides of chunk keys.
            for k in 1..120 {
                if k != 5 && !(45..61).contains(&k) {
                    assert_eq!(hashes[k], hashes[k - 1], "frame {k}");
                }
            }
        }
    }
    let invoke = || {
        let mut c = Command::new(env!("CARGO_BIN_EXE_av"));
        c.arg("render")
            .arg(root.join("timeline.json"))
            .arg("--out")
            .arg(root.join("next.webm"));
        c
    };
    fs::write(root.join("victim"), b"keep").unwrap();
    std::os::unix::fs::symlink(root.join("victim"), root.join("next.webm")).unwrap();
    assert_eq!(invoke().output().unwrap().status.code(), Some(2));
    assert_eq!(fs::read(root.join("victim")).unwrap(), b"keep");
    fs::remove_file(root.join("next.webm")).unwrap();
    let full = || fs::File::options().write(true).open("/dev/full").unwrap();
    let failed = invoke().stdout(full()).output().unwrap();
    assert_eq!(failed.status.code(), Some(4));
    assert!(String::from_utf8_lossy(&failed.stderr).contains("published but receipt not written"));
    assert!(root.join("next.webm").is_file());
    fs::remove_file(root.join("next.webm")).unwrap();
    let mut closed = invoke()
        .stdout(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    drop(closed.stdout.take());
    assert_eq!(closed.wait().unwrap().code(), Some(0));
    fs::remove_file(root.join("next.webm")).unwrap();
    let mut refused = t.clone();
    refused["audio"]["master"]["integrated_lufs"] = json!(-1);
    refused["audio"]["master"]["true_peak_dbtp"] = json!(-20);
    fs::write(
        root.join("timeline.json"),
        serde_json::to_vec(&refused).unwrap(),
    )
    .unwrap();
    let refused = invoke().stdout(full()).stderr(full()).status().unwrap();
    assert_eq!(refused.code(), Some(3));
    assert!(!root.join("next.webm").exists());
    fs::remove_dir_all(root).unwrap();
}
