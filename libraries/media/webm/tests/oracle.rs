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
fn required_capabilities() {
    for (option, names) in [("-decoders", & ["av1", "opus"][..]), ("-encoders", &["libaom-av1", "libopus"][..]), ("-muxers", &["webm", "null"][..])] {
        let output = run("ffmpeg", &["-v", "error", option]);
        let output = String::from_utf8(output).unwrap();
        for name in names {
            assert!(output.lines().any(|l| l.split_whitespace().nth(1).is_some_and(|n| n.split(',').any(|n| n == *name))), "required ffmpeg capability {option} {name} unavailable");
        }
    }
}
struct VideoPacket { data:Vec<u8>, pts:u64, key_frame:bool }
fn video() -> Vec<VideoPacket> {
    let d=Demuxer::new(include_bytes!("fixtures/rav1e.webm")).unwrap();
    assert_eq!(d.packets().iter().filter(|p|p.key_frame).count(),2);
    d.packets().iter().map(|p|VideoPacket { data:p.data.to_vec(), pts:p.timestamp as u64/40, key_frame:p.key_frame }).collect()
}
fn mux(packets: &[VideoPacket], width: u32, height: u32, audio: &Demuxer<'_>) -> Vec<u8> {
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
    // Fixture container ticks are rebased to zero, preserving packet deltas.
    let first = audio.packets()[0].timestamp;
    let video_end = (packets.last().unwrap().pts + 1) * 40;
    let mut events: Vec<_> = packets
        .iter()
        .map(|p| (p.pts * 40, 1, p.key_frame, p.data.as_slice(), 40))
        .collect();
    events.extend(
        audio
            .packets()
            .iter()
            .map(|p| ((p.timestamp - first) as u64, 2, true, p.data, 20))
            .filter(|e| e.0 < video_end)
            .map(|e| (e.0, e.1, e.2, e.3, e.4.min(video_end - e.0))),
    );
    events.sort_by_key(|e| e.0);
    for (t, track, key, b, d) in events {
        w.packet(track, t, d, key, b).unwrap();
    }
    w.finish().unwrap().into_inner()
}
#[test]
fn roundtrip_and_oracles() {
    required_capabilities();
    let audio = Demuxer::new(include_bytes!("fixtures/sine.webm")).unwrap();
    let v = video();
    let bytes = mux(&v, 64, 48, &audio);
    let mut d = Demuxer::new(&bytes).unwrap();
    let at = d.tracks.iter().find(|t| t.kind == 2).unwrap();
    assert_eq!(at.codec_delay_ns, 6_500_000);
    assert_eq!(at.seek_pre_roll_ns, 80_000_000);
    assert_eq!(at.codec_private, audio.tracks.iter().find(|t| t.kind == 2).unwrap().codec_private);
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
    assert_eq!(d.seek(399), Some(200));
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
    let pts = run("ffprobe", &["-v", "error", "-select_streams", "a", "-show_packets", "-of", "json", path]);
    let pts: serde_json::Value = serde_json::from_slice(&pts).unwrap();
    assert_eq!(pts["packets"][0]["pts"], -7);
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
        assert_eq!(
            r.presentation_timestamp(p).unwrap(),
            o["pts"].as_i64().unwrap()
        );
    }
    assert_eq!(r.seek(390), Some(200));
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
    let first = d.packets().iter().find(|p| p.track == 1).unwrap();
    let config = av1_config(first.data).unwrap();
    let private = d.tracks.iter().find(|t| t.kind == 1).unwrap().codec_private;
    assert_eq!(config, private, "av1C must match ffmpeg/libaom");
    let probe = run(
        "ffprobe",
        &["-v", "error", "-show_packets", "-of", "json", independent],
    );
    let j: serde_json::Value = serde_json::from_slice(&probe).unwrap();
    let oracle = j["packets"].as_array().unwrap();
    assert_eq!(d.packets().len(), oracle.len());
    for (p, o) in d.packets().iter().zip(oracle) {
        assert_eq!(
            d.presentation_timestamp(p).unwrap(),
            o["pts"].as_i64().unwrap()
        );
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
