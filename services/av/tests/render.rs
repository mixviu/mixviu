use serde_json::{Value, json};
use std::{fs, process::Command};
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
        ("/video/segments/0/path", json!("../a.vfl"), "segment_path_relative"),
        ("/video/segments/0/at_s", json!(0.1), "gap_before_first_segment"),
        ("/video/segments/0/to_slot", json!(100), "segment_trim_outside_vfl"),
        ("/video/crossfades/0/at_s", json!(1.0), "crossfade_segment_start"),
        ("/video/crossfades/0/duration_s", json!(1.0), "crossfade_requires_overlap"),
    ] {
        let mut bad = t.clone(); *bad.pointer_mut(pointer).unwrap() = value;
        fs::write(root.join("timeline.json"),serde_json::to_vec(&bad).unwrap()).unwrap();
        let o = Command::new(env!("CARGO_BIN_EXE_av")).arg("render").arg(root.join("timeline.json")).arg("--out").arg(root.join("out.webm")).output().unwrap();
        assert_eq!(o.status.code(),Some(2));
        assert!(String::from_utf8_lossy(&o.stderr).contains(name), "{}",String::from_utf8_lossy(&o.stderr));
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
    assert!(receipt["frames_encoded"].as_u64().unwrap() < 45);
    let bytes = fs::read(root.join("out.webm")).unwrap();
    let demux = media_webm::Demuxer::new(&bytes).unwrap();
    let video: Vec<_> = demux.packets().iter().filter(|p| p.track == 1).collect();
    let audio: Vec<_> = demux.packets().iter().filter(|p| p.track == 2).collect();
    assert_eq!(video.len(),120);
    assert_eq!(video[0].timestamp,0);
    assert_eq!(audio[0].timestamp,0);
    assert!((video.last().unwrap().timestamp-audio.last().unwrap().timestamp).abs() <= 34);
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
            // All holds must remain identical, including both sides of chunk keys.
            for k in 1..120 {
                if !(45..61).contains(&k) {
                    assert_eq!(hashes[k], hashes[k - 1], "frame {k}");
                }
            }
        }
    }
    fs::remove_dir_all(root).unwrap();
}
