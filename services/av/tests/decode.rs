use media_webm::{Demuxer, Video, Writer};
use std::io::Cursor;
use std::process::Command;

const FIXTURE: &[u8] = include_bytes!("../../../libraries/media/webm/tests/fixtures/rav1e.webm");
fn repeats(count: u64) -> Vec<u8> {
    repeat_stream(count, false)
}

#[test]
fn framemd5_cli_receipt_budget_and_duration_ceiling() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.webm");
    let hashes = dir.path().join("out.txt");
    let mut bytes = repeats(20);
    std::fs::write(&input, &bytes).unwrap();
    let invoke = || {
        Command::new(env!("CARGO_BIN_EXE_av"))
            .arg("decode")
            .arg(&input)
            .arg("--framemd5")
            .arg(&hashes)
            .args(["--max-output-bytes", "4000"])
            .output()
            .unwrap()
    };
    let result = invoke();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let receipt: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(receipt["frames"], 20);
    assert_eq!(receipt["framemd5"], true);
    assert_eq!(receipt["framemd5_lines"], 20);
    assert_eq!(
        receipt["output_bytes"],
        std::fs::metadata(&hashes).unwrap().len()
    );
    assert_eq!(
        std::fs::read_to_string(&hashes).unwrap().lines().count(),
        21
    );
    std::fs::remove_file(&hashes).unwrap();
    let at = bytes.windows(2).position(|b| b == [0x44, 0x89]).unwrap() + 10;
    bytes[at..at + 8].copy_from_slice(&40f64.to_be_bytes());
    std::fs::write(&input, bytes).unwrap();
    let result = invoke();
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("FRAME_COUNT_LIMIT_EXCEEDED"));
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}
fn repeat_stream(count: u64, vfr: bool) -> Vec<u8> {
    let d = Demuxer::new(FIXTURE).unwrap();
    let t = &d.tracks[0];
    let mut w = Writer::new(
        Cursor::new(Vec::new()),
        Video {
            width: 64,
            height: 48,
            codec_private: t.codec_private.to_vec(),
        },
        None,
    )
    .unwrap();
    for i in 0..count {
        let data = if i < 2 {
            d.packets()[i as usize].data
        } else {
            &[0x12, 0, 0x1a, 1, 0x98]
        };
        let timestamp = i * 40 + if vfr && i > 0 { 120 } else { 0 };
        w.packet(1, timestamp, 40, i == 0, data).unwrap();
    }
    w.finish().unwrap().into_inner()
}

#[test]
fn video_and_audio_share_budget_and_publish_together() {
    let dir = tempfile::tempdir().unwrap();
    let video = Demuxer::new(FIXTURE).unwrap();
    let audio = Demuxer::new(include_bytes!(
        "../../../libraries/media/webm/tests/fixtures/sine.webm"
    ))
    .unwrap();
    let head = audio.tracks[0].codec_private;
    let mut w = Writer::new(
        Cursor::new(Vec::new()),
        Video {
            width: 64,
            height: 48,
            codec_private: video.tracks[0].codec_private.to_vec(),
        },
        Some(media_webm::Opus {
            channels: head[9],
            pre_skip: u16::from_le_bytes([head[10], head[11]]),
            input_sample_rate: 48000,
        }),
    )
    .unwrap();
    w.packet(1, 0, 40, true, video.packets()[0].data).unwrap();
    w.opus_packet(0, audio.packets()[0].data, 0).unwrap();
    w.packet(1, 40, 40, false, video.packets()[1].data).unwrap();
    let input = dir.path().join("input.webm");
    std::fs::write(&input, w.finish().unwrap().into_inner()).unwrap();
    let y4m = dir.path().join("out.y4m");
    let wav = dir.path().join("out.wav");
    let invoke = |limit: &str| {
        Command::new(env!("CARGO_BIN_EXE_av"))
            .arg("decode")
            .arg(&input)
            .arg("--y4m")
            .arg(&y4m)
            .arg("--wav")
            .arg(&wav)
            .arg("--max-output-bytes")
            .arg(limit)
            .output()
            .unwrap()
    };
    let result = invoke("200000");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let receipt: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    let video_bytes = std::fs::metadata(&y4m).unwrap().len();
    assert_eq!(
        receipt["output_bytes"],
        video_bytes + std::fs::metadata(&wav).unwrap().len()
    );
    assert_eq!(
        hound::WavReader::open(&wav).unwrap().spec().sample_rate,
        48000
    );
    std::fs::remove_file(&y4m).unwrap();
    std::fs::remove_file(&wav).unwrap();
    let result = invoke(&video_bytes.to_string());
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("OUTPUT_LIMIT_EXCEEDED"));
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn variable_packet_intervals_do_not_invent_a_frame_rate_ceiling() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("vfr.webm");
    std::fs::write(&input, repeat_stream(20, true)).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("decode")
        .arg(&input)
        .arg("--y4m")
        .arg(dir.path().join("out.y4m"))
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let receipt: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(receipt["frames"], 20);
    let bytes = std::fs::read(dir.path().join("out.y4m")).unwrap();
    let mut at = bytes.iter().position(|b| *b == b'\n').unwrap() + 1;
    let header = std::str::from_utf8(&bytes[..at]).unwrap();
    assert!(header.contains(" F1000000000:40000000 "), "{header}");
    for i in 0..20 {
        let end = at + bytes[at..].iter().position(|b| *b == b'\n').unwrap() + 1;
        let timestamp = (i * 40 + if i > 0 { 120 } else { 0 }) * 1_000_000;
        assert_eq!(&bytes[at..end], format!("FRAME XPTS={timestamp}\n").as_bytes());
        at = end + 64 * 48 * 3 / 2;
    }
    assert_eq!(at, bytes.len());
}

#[test]
fn lexical_output_aliases_are_refused_before_staging() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("input.webm"), FIXTURE).unwrap();
    for flags in [
        vec!["--y4m", "out", "--framemd5", "./out"],
        vec!["--framemd5", "out", "--wav", "missing/../out"],
        vec!["--framemd5", "./input.webm"],
        vec!["--y4m", "missing/../input.webm"],
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_av"))
            .current_dir(dir.path()).args(["decode", "input.webm"])
            .args(flags).output().unwrap();
        assert_eq!(result.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&result.stderr).contains("OUTPUT_PATH_CONFLICT"));
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }
}

#[test]
fn probe_rate_uses_all_timestamps_independently_of_duration_and_decoded_count() {
    let source = Demuxer::new(FIXTURE).unwrap();
    for (timestamps, fps) in [
        (
            (0..31).map(|i| i * 1000 / 30).collect::<Vec<_>>(),
            Some(30.0),
        ),
        (vec![0, 20, 80, 100], None),
        (vec![0], None),
    ] {
        let mut writer = Writer::new(
            Cursor::new(Vec::new()),
            Video {
                width: 64,
                height: 48,
                codec_private: source.tracks[0].codec_private.to_vec(),
            },
            None,
        )
        .unwrap();
        for (i, &timestamp) in timestamps.iter().enumerate() {
            let data = source
                .packets()
                .get(i)
                .map_or(&[0x12, 0, 0x1a, 1, 0x98][..], |p| p.data);
            writer.packet(1, timestamp, 33, i == 0, data).unwrap();
        }
        let mut bytes = writer.finish().unwrap().into_inner();
        // Deliberately false Segment Duration: it must neither establish a
        // frame rate nor replace the full decoded frame count.
        let at = bytes.windows(2).position(|b| b == [0x44, 0x89]).unwrap() + 10;
        bytes[at..at + 8].copy_from_slice(&40f64.to_be_bytes());
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("timing.webm");
        std::fs::write(&input, bytes).unwrap();
        let result = Command::new(env!("CARGO_BIN_EXE_av"))
            .arg("probe")
            .arg(&input)
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let receipt: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert_eq!(receipt["tracks"][0]["fps"], serde_json::json!(fps));
        assert_eq!(receipt["frames"], timestamps.len());
        assert_eq!(receipt["duration"], 0.04);
    }
}

#[test]
fn symlink_destinations_and_late_audio_failure_publish_nothing() {
    for wav_link in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let input = dir.path().join("input.webm");
        std::fs::write(&input, FIXTURE).unwrap();
        let victim = dir.path().join("victim");
        std::fs::write(&victim, b"keep me").unwrap();
        let y4m = dir.path().join("out.y4m");
        let wav = dir.path().join("out.wav");
        std::os::unix::fs::symlink(&victim, if wav_link { &wav } else { &y4m }).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_av"));
        command.arg("decode").arg(&input).arg("--y4m").arg(&y4m);
        if wav_link {
            command.arg("--wav").arg(&wav);
        }
        let result = command.output().unwrap();
        assert_eq!(result.status.code(), Some(2));
        assert_eq!(std::fs::read(&victim).unwrap(), b"keep me");
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 3);
        if wav_link {
            assert!(!y4m.exists());
        }
    }
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.webm");
    std::fs::write(&input, FIXTURE).unwrap();
    let y4m = dir.path().join("out.y4m");
    let wav = dir.path().join("out.wav");
    let result = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("decode")
        .arg(input)
        .arg("--y4m")
        .arg(&y4m)
        .arg("--wav")
        .arg(&wav)
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("no Opus track"));
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn repeat_amplification_and_duration_limits_are_transactional() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("tiny.webm");
    let y4m = dir.path().join("out.y4m");
    let mut bytes = repeats(20);
    assert!(bytes.len() < 10_000);
    std::fs::write(&input, &bytes).unwrap();
    let invoke = |limit: &str| {
        Command::new(env!("CARGO_BIN_EXE_av"))
            .arg("decode")
            .arg(&input)
            .arg("--y4m")
            .arg(&y4m)
            .arg("--max-output-bytes")
            .arg(limit)
            .output()
            .unwrap()
    };
    let result = invoke("20000");
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("OUTPUT_LIMIT_EXCEEDED"));
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    let result = invoke("200000");
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let receipt: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert_eq!(receipt["frames"], 20);
    assert_eq!(
        receipt["output_bytes"],
        std::fs::metadata(&y4m).unwrap().len()
    );
    std::fs::remove_file(&y4m).unwrap();
    // Advertise one frame of duration while retaining twenty valid frame units.
    let at = bytes.windows(2).position(|b| b == [0x44, 0x89]).unwrap() + 10;
    bytes[at..at + 8].copy_from_slice(&40f64.to_be_bytes());
    std::fs::write(&input, bytes).unwrap();
    let result = invoke("200000");
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("FRAME_COUNT_LIMIT_EXCEEDED"));
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn zero_frames_refuse_and_receipt_failure_preserves_published_output() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.webm");
    let d = Demuxer::new(FIXTURE).unwrap();
    let mut w = Writer::new(
        Cursor::new(Vec::new()),
        Video {
            width: 64,
            height: 48,
            codec_private: d.tracks[0].codec_private.to_vec(),
        },
        None,
    )
    .unwrap();
    w.packet(1, 0, 40, true, &[0x12, 0]).unwrap();
    std::fs::write(&input, w.finish().unwrap().into_inner()).unwrap();
    let out = dir.path().join("out.y4m");
    let result = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("decode")
        .arg(&input)
        .arg("--y4m")
        .arg(&out)
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("NO_VIDEO_FRAMES"));
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    std::fs::write(&input, FIXTURE).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("decode")
        .arg(&input)
        .arg("--y4m")
        .arg(&out)
        .stdout(
            std::fs::File::options()
                .write(true)
                .open("/dev/full")
                .unwrap(),
        )
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(4));
    assert!(String::from_utf8_lossy(&result.stderr).contains("published but receipt not written"));
    assert!(std::fs::read(out).unwrap().starts_with(b"YUV4MPEG2"));
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 2);
}

fn element(id: u32, data: &[u8]) -> Vec<u8> {
    let bytes = id.to_be_bytes();
    let mut out = bytes[bytes.iter().position(|b| *b != 0).unwrap()..].to_vec();
    out.extend(((1u64 << 56) | data.len() as u64).to_be_bytes());
    out.extend(data);
    out
}

#[test]
fn minimum_signed_discard_padding_is_a_clean_transactional_error() {
    let dir = tempfile::tempdir().unwrap();
    let v = Demuxer::new(FIXTURE).unwrap();
    let a = Demuxer::new(include_bytes!(
        "../../../libraries/media/webm/tests/fixtures/sine.webm"
    ))
    .unwrap();
    let tracks = element(
        0x1654ae6b,
        &[
            element(
                0xae,
                &[
                    element(0xd7, &[1]),
                    element(0x83, &[1]),
                    element(0x86, b"V_AV1"),
                ]
                .concat(),
            ),
            element(
                0xae,
                &[
                    element(0xd7, &[2]),
                    element(0x83, &[2]),
                    element(0x86, b"A_OPUS"),
                    element(0x63a2, a.tracks[0].codec_private),
                ]
                .concat(),
            ),
        ]
        .concat(),
    );
    let mut block = vec![0x81, 0, 0, 0x80];
    block.extend(v.packets()[0].data);
    let mut opus = vec![0x82, 0, 0, 0x80];
    opus.extend(a.packets()[0].data);
    let cluster = element(
        0x1f43b675,
        &[
            element(0xe7, &[0]),
            element(0xa3, &block),
            element(
                0xa0,
                &[
                    element(0xa1, &opus),
                    element(0x75a2, &i64::MIN.to_be_bytes()),
                ]
                .concat(),
            ),
        ]
        .concat(),
    );
    let bytes = [
        element(0x1a45dfa3, &element(0x4282, b"webm")),
        element(0x18538067, &[tracks, cluster].concat()),
    ]
    .concat();
    let input = dir.path().join("padding.webm");
    std::fs::write(&input, bytes).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("decode")
        .arg(&input)
        .arg("--y4m")
        .arg(dir.path().join("out.y4m"))
        .arg("--wav")
        .arg(dir.path().join("out.wav"))
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&result.stderr).contains("TIMESTAMP_OVERFLOW: discard padding")
    );
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}
#[test]
fn extreme_packet_delta_is_a_clean_error_and_usage_lists_commands() {
    let dir = tempfile::tempdir().unwrap();
    let header = element(0x1a45dfa3, &element(0x4282, b"webm"));
    let tracks = element(
        0x1654ae6b,
        &element(
            0xae,
            &[
                element(0xd7, &[1]),
                element(0x83, &[1]),
                element(0x86, b"V_AV1"),
            ]
            .concat(),
        ),
    );
    let a = element(
        0x1f43b675,
        &[
            element(0xe7, &[0]),
            element(0xa3, &[0x81, 0x80, 0, 0x80, 0x12, 0]),
        ]
        .concat(),
    );
    let b = element(
        0x1f43b675,
        &[
            element(0xe7, &(i64::MAX as u64).to_be_bytes()),
            element(0xa3, &[0x81, 0, 0, 0x80, 0x12, 0]),
        ]
        .concat(),
    );
    let input = dir.path().join("overflow.webm");
    std::fs::write(
        &input,
        [header, element(0x18538067, &[tracks, a, b].concat())].concat(),
    )
    .unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("probe")
        .arg(&input)
        .output()
        .unwrap();
    assert!(result.status.success());
    let receipt: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert!(receipt["tracks"][0]["fps"].is_null());
    let result = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("decode")
        .arg(&input)
        .arg("--y4m")
        .arg(dir.path().join("overflow.y4m"))
        .output()
        .unwrap();
    assert_eq!(result.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&result.stderr).contains("TIMESTAMP_OVERFLOW"));
    let result = Command::new(env!("CARGO_BIN_EXE_av")).output().unwrap();
    let usage = String::from_utf8_lossy(&result.stderr);
    for command in ["av audio", "av decode", "av probe"] {
        assert!(usage.contains(command));
    }
}

#[test]
fn probe_non_monotonic_timing_keeps_receipt() {
    let source = Demuxer::new(FIXTURE).unwrap();
    let tracks = element(
        0x1654ae6b,
        &element(
            0xae,
            &[
                element(0xd7, &[1]),
                element(0x83, &[1]),
                element(0x86, b"V_AV1"),
            ]
            .concat(),
        ),
    );
    let mut segment = tracks;
    for (i, timestamp) in [0u64, 67, 33, 100].iter().enumerate() {
        let block = [&[0x81, 0, 0, 0x80][..], source.packets()[i].data].concat();
        segment.extend(element(
            0x1f43b675,
            &[
                element(0xe7, &timestamp.to_be_bytes()),
                element(0xa3, &block),
            ]
            .concat(),
        ));
    }
    let bytes = [
        element(0x1a45dfa3, &element(0x4282, b"webm")),
        element(0x18538067, &segment),
    ]
    .concat();
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("non-monotonic.webm");
    std::fs::write(&input, bytes).unwrap();
    let result = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("probe")
        .arg(input)
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    let receipt: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
    assert!(receipt["tracks"][0]["fps"].is_null());
    assert_eq!(receipt["frames"], 4);
    assert_eq!(receipt["packets"], 4);
    assert_eq!(receipt["duration"], 0.1);
}

#[test]
fn probe_decode_and_error_conventions() {
    let dir = tempfile::tempdir().unwrap();
    let input = dir.path().join("input.webm");
    std::fs::write(
        &input,
        include_bytes!("../../../libraries/media/webm/tests/fixtures/rav1e.webm"),
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("probe")
        .arg(&input)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(out.stdout.iter().filter(|b| **b == b'\n').count(), 1);
    let j: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(j["frames"], 10);
    assert_eq!(j["tracks"][0]["codec"], "V_AV1");
    assert_eq!(j["tracks"][0]["width"], 64);
    let y4m = dir.path().join("out.y4m");
    let out = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("decode")
        .arg(&input)
        .arg("--y4m")
        .arg(&y4m)
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let pixels = std::fs::read(y4m).unwrap();
    assert!(pixels.starts_with(b"YUV4MPEG2 W64 H48"));
    assert_eq!(pixels.windows(5).filter(|w| *w == b"FRAME").count(), 10);
    for args in [
        vec!["decode"],
        vec!["probe"],
        vec!["decode", "missing.webm", "--unknown"],
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_av"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stderr.starts_with(b"av: "));
    }
}
