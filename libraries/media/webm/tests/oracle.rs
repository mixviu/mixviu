use media_webm::{Demuxer, Opus, Video, Writer, av1_config};
use std::{io::Cursor, process::Command};

fn run(program: &str, args: &[&str]) -> Vec<u8> {
    let o = Command::new(program)
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("required oracle {program}: {e}"));
    assert!(
        o.status.success(),
        "{program} {args:?}: {} (required decoder/encoder unavailable or oracle failed)",
        String::from_utf8_lossy(&o.stderr)
    );
    assert!(
        o.stderr.is_empty(),
        "oracle diagnostics: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    o.stdout
}
fn video(width: usize, height: usize, n: usize) -> Vec<media_av1::Packet> {
    let mut e = media_av1::Encoder::new(media_av1::Settings {
        width,
        height,
        fps_num: 25,
        fps_den: 1,
        speed: 10,
        target: media_av1::Target::Quality(120),
        key_interval: 25,
        tile_cols: 1,
        tile_rows: 1,
        threads: 2,
    })
    .unwrap();
    let f = e
        .prepare(
            &media_frame::Bgra::solid(width, height, [50, 100, 150, 255])
                .unwrap()
                .to_yuv420(),
        )
        .unwrap();
    let mut packets = Vec::new();
    for _ in 0..n {
        packets.extend(e.submit(&f).unwrap());
    }
    packets.extend(e.finish().unwrap());
    packets
}
fn mux(packets: &[media_av1::Packet], width: u32, height: u32, audio: &Demuxer<'_>) -> Vec<u8> {
    let at = audio.tracks.iter().find(|t| t.kind == 2).unwrap();
    let head = at.codec_private;
    let a = Opus {
        channels: head[9],
        pre_skip: u16::from_le_bytes([head[10], head[11]]),
        input_sample_rate: 48000,
    };
    let mut w = Writer::new(
        Cursor::new(Vec::new()),
        Video {
            width,
            height,
            codec_private: av1_config(&packets[0].data).unwrap(),
        },
        Some(a),
    )
    .unwrap();
    // Fixture's negative first timestamp is rebased to zero, preserving packet deltas.
    let first = audio.packets()[0].timestamp;
    let mut events: Vec<_> = packets
        .iter()
        .map(|p| (p.pts * 40, 1, p.key_frame, p.data.as_slice(), 40))
        .collect();
    events.extend(
        audio
            .packets()
            .iter()
            .map(|p| ((p.timestamp - first) as u64, 2, true, p.data, 20)),
    );
    events.sort_by_key(|e| e.0);
    for (t, track, key, b, d) in events {
        w.packet(track, t, d, key, b).unwrap();
    }
    w.finish().unwrap().into_inner()
}
#[test]
fn roundtrip_and_oracles() {
    let version = run(
        "ffprobe",
        &["-v", "error", "-show_program_version", "-of", "json"],
    );
    let version: serde_json::Value = serde_json::from_slice(&version).unwrap();
    assert!(
        version["program_version"]["version"]
            .as_str()
            .unwrap()
            .trim_start_matches('n')
            .starts_with("9."),
        "ffmpeg 9 oracle required"
    );
    let audio = Demuxer::new(include_bytes!("fixtures/sine.webm")).unwrap();
    let v = video(64, 48, 10);
    let bytes = mux(&v, 64, 48, &audio);
    let mut d = Demuxer::new(&bytes).unwrap();
    let vp: Vec<_> = d.packets().iter().filter(|p| p.track == 1).collect();
    assert_eq!(vp.len(), v.len());
    for (a, b) in vp.iter().zip(&v) {
        assert_eq!(a.data, b.data);
        assert_eq!(a.timestamp, b.pts as i64 * 40);
        assert_eq!(a.key_frame, b.key_frame);
    }
    let ap: Vec<_> = d.packets().iter().filter(|p| p.track == 2).collect();
    assert_eq!(ap.len(), audio.packets().len());
    for (a, b) in ap.iter().zip(audio.packets()) {
        assert_eq!(a.data, b.data);
        assert_eq!(a.timestamp, b.timestamp - audio.packets()[0].timestamp);
    }
    assert_eq!(d.seek(399), Some(0));
    assert!(d.next_packet().unwrap().key_frame);
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("ours.webm");
    std::fs::write(&path, bytes).unwrap();
    let path = path.to_str().unwrap();
    let probe = run(
        "ffprobe",
        &[
            "-v",
            "error",
            "-count_frames",
            "-show_streams",
            "-show_format",
            "-of",
            "json",
            path,
        ],
    );
    let j: serde_json::Value = serde_json::from_slice(&probe).unwrap();
    let s = &j["streams"][0];
    assert_eq!(s["codec_name"], "av1");
    assert_eq!(j["streams"][1]["codec_name"], "opus");
    assert_eq!(s["nb_read_frames"], "10");
    assert_eq!(s["color_space"], "bt709");
    assert_eq!(s["color_transfer"], "bt709");
    assert_eq!(s["color_primaries"], "bt709");
    assert_eq!(s["color_range"], "tv");
    assert_eq!(j["format"]["duration"], "0.400000");
    run(
        "ffmpeg",
        &["-v", "error", "-xerror", "-i", path, "-f", "null", "-"],
    );
    let remux = temp.path().join("ffmpeg.webm");
    let remux = remux.to_str().unwrap();
    run("ffmpeg", &["-v", "error", "-i", path, "-c", "copy", remux]);
    let b = std::fs::read(remux).unwrap();
    let mut r = Demuxer::new(&b).unwrap();
    let probe = run(
        "ffprobe",
        &["-v", "error", "-show_packets", "-of", "json", remux],
    );
    let j: serde_json::Value = serde_json::from_slice(&probe).unwrap();
    let oracle = j["packets"].as_array().unwrap();
    assert_eq!(r.packets().len(), oracle.len());
    for (p, o) in r.packets().iter().zip(oracle) {
        assert_eq!(r.presentation_timestamp(p).unwrap(), o["pts"].as_i64().unwrap());
    }
    assert_eq!(r.seek(390), Some(7));
    assert!(r.next_packet().unwrap().key_frame);
    let independent = temp.path().join("independent.webm");
    let independent = independent.to_str().unwrap();
    run(
        "ffmpeg",
        &[
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            "testsrc2=size=64x48:rate=25:duration=0.4",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=0.3",
            "-c:v",
            "libaom-av1",
            "-cpu-used",
            "8",
            "-g",
            "5",
            "-c:a",
            "libopus",
            independent,
        ],
    );
    let bytes = std::fs::read(independent).unwrap();
    let mut d = Demuxer::new(&bytes).unwrap();
    let probe = run(
        "ffprobe",
        &["-v", "error", "-show_packets", "-of", "json", independent],
    );
    let j: serde_json::Value = serde_json::from_slice(&probe).unwrap();
    let oracle = j["packets"].as_array().unwrap();
    assert_eq!(d.packets().len(), oracle.len());
    for (p, o) in d.packets().iter().zip(oracle) {
        assert_eq!(d.presentation_timestamp(p).unwrap(), o["pts"].as_i64().unwrap());
        assert_eq!(p.key_frame, o["flags"].as_str().unwrap().starts_with('K'));
    }
    let want = d
        .packets()
        .iter()
        .filter(|p| p.track == 1 && p.key_frame && p.timestamp <= 300)
        .map(|p| p.timestamp)
        .max()
        .unwrap() as u64;
    assert_eq!(d.seek(300), Some(want));
    assert_eq!(d.next_packet().unwrap().timestamp, want as i64);
}
#[test]
fn negative_inputs() {
    let audio = Demuxer::new(include_bytes!("fixtures/sine.webm")).unwrap();
    let v = video(32, 32, 2);
    let bytes = mux(&v, 32, 32, &audio);
    for n in 0..bytes.len() {
        assert!(
            Demuxer::new(&bytes[..n]).is_err(),
            "accepted truncation {n}"
        );
    }
    for n in 0..bytes.len() {
        let mut b = bytes.clone();
        b[n] = 0xff;
        let _ = Demuxer::new(&b);
    }
    // Small, structurally valid file with one hostile SimpleBlock.
    let mut w = Writer::new(
        Cursor::new(Vec::new()),
        Video {
            width: 32,
            height: 32,
            codec_private: av1_config(&v[0].data).unwrap(),
        },
        None,
    )
    .unwrap();
    w.packet(1, 0, 40, true, &[0x11]).unwrap();
    let mut b = w.finish().unwrap().into_inner();
    let at = b
        .windows(5)
        .position(|x| x == [0x81, 0, 0, 0x80, 0x11])
        .unwrap();
    b[at + 3] |= 2;
    assert!(matches!(
        Demuxer::new(&b),
        Err(media_webm::Error::UnsupportedLacing)
    ));
    assert!(matches!(
        Demuxer::new(&[0; 16]),
        Err(media_webm::Error::SizeOverflow)
    ));
}

/// Opt-in supervisor sample; run through CBC with MEDIA_WEBM_SAMPLE pointing
/// at a temporary output file, then copy that artifact to the bench directory.
#[test]
#[ignore = "10 second 4K sample artifact"]
fn sample_4k() {
    let path = std::env::var("MEDIA_WEBM_SAMPLE").expect("MEDIA_WEBM_SAMPLE required");
    let audio_bytes = run(
        "ffmpeg",
        &[
            "-v",
            "error",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=440:sample_rate=48000:duration=10",
            "-c:a",
            "libopus",
            "-f",
            "webm",
            "pipe:1",
        ],
    );
    let audio = Demuxer::new(&audio_bytes).unwrap();
    let v = video(3840, 2160, 250);
    std::fs::write(&path, mux(&v, 3840, 2160, &audio)).unwrap();
    run(
        "ffmpeg",
        &["-v", "error", "-xerror", "-i", &path, "-f", "null", "-"],
    );
    let probe = run(
        "ffprobe",
        &[
            "-v",
            "error",
            "-count_frames",
            "-show_streams",
            "-show_format",
            "-of",
            "json",
            &path,
        ],
    );
    std::fs::write(format!("{path}.ffprobe.json"), probe).unwrap();
}
