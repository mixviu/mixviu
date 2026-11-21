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
        key_interval: 5,
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

#[test]
#[ignore = "one-shot generated fixture"]
fn generate_webm_fixture() {
    let path = std::env::var("MEDIA_AV1_FIXTURE").expect("MEDIA_AV1_FIXTURE required");
    let v = video(64, 48, 10);
    let mut w = Writer::new(Cursor::new(Vec::new()), Video { width:64, height:48, codec_private:av1_config(&v[0].data).unwrap() }, None).unwrap();
    for p in v { w.packet(1,p.pts*40,40,p.key_frame,&p.data).unwrap(); }
    std::fs::write(path,w.finish().unwrap().into_inner()).unwrap();
}
