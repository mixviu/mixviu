use serde_json::{Value, json};
use std::{fs, io::Cursor, process::Command};

#[test]
fn pcm24_float32_and_extensible_probe() {
    let root = tempfile::tempdir().unwrap();
    for (bits, float, extensible, codec) in [
        (24u16, false, false, "PCM_S24LE"),
        (32, true, false, "PCM_F32LE"),
        (24, false, true, "PCM_S24LE"),
    ] {
        let channels = 2u16;
        let align = channels * bits / 8;
        let mut fmt = Vec::new();
        fmt.extend_from_slice(
            &(if extensible {
                0xfffeu16
            } else if float {
                3
            } else {
                1
            })
            .to_le_bytes(),
        );
        fmt.extend_from_slice(&channels.to_le_bytes());
        fmt.extend_from_slice(&48000u32.to_le_bytes());
        fmt.extend_from_slice(&(48000u32 * u32::from(align)).to_le_bytes());
        fmt.extend_from_slice(&align.to_le_bytes());
        fmt.extend_from_slice(&bits.to_le_bytes());
        if extensible {
            fmt.extend_from_slice(&22u16.to_le_bytes());
            fmt.extend_from_slice(&bits.to_le_bytes());
            fmt.extend_from_slice(&3u32.to_le_bytes()); // stereo channel mask
            fmt.extend_from_slice(&[1, 0, 0, 0, 0, 0, 16, 0, 128, 0, 0, 170, 0, 56, 155, 113]); // PCM GUID
        }
        let data = vec![0u8; usize::from(align) * 48];
        let mut bytes = b"RIFF".to_vec();
        bytes.extend_from_slice(&(20 + fmt.len() as u32 + data.len() as u32).to_le_bytes());
        bytes.extend_from_slice(b"WAVEfmt ");
        bytes.extend_from_slice(&(fmt.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&fmt);
        bytes.extend_from_slice(b"data");
        bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&data);
        let path = root.path().join("format.wav");
        fs::write(&path, bytes).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_av"))
            .arg("probe")
            .arg(path)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let receipt: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(receipt["tracks"][0]["codec"], codec);
        assert_eq!(receipt["tracks"][0]["bits_per_sample"], bits);
        assert_eq!(receipt["tracks"][0]["channels"], 2);
        assert_eq!(receipt["tracks"][0]["sample_rate"], 48000);
        assert_eq!(receipt["frames"], 48);
        assert_eq!(receipt["duration"], 0.001);
    }
}

#[test]
fn oversized_wav_probe_refused_before_decoding() {
    use std::io::Write;
    let root = tempfile::tempdir().unwrap();
    let path = root.path().join("large.wav");
    let mut file = fs::File::create(&path).unwrap();
    file.write_all(b"RIFF\0\0\0\0WAVE").unwrap();
    file.set_len(256 * 1024 * 1024 + 1).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_av"))
        .arg("probe")
        .arg(path)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("PROBE_INPUT_LIMIT"));
    assert!(output.stdout.is_empty());
}

#[test]
fn tts_shaped_pcm16_and_list_chunk_probe_and_render() {
    let root = tempfile::tempdir().unwrap();
    for channels in [1, 2] {
        let audio = media_audio::Audio::new(
            48000,
            channels,
            (0..287552)
                .flat_map(|i| vec![0.1 * (i as f64 * 0.0576).sin(); channels as usize])
                .collect(),
        )
        .unwrap();
        let mut wav = Cursor::new(Vec::new());
        audio
            .write_wav(&mut wav, media_audio::WavFormat::Pcm16)
            .unwrap();
        let canonical = wav.into_inner();
        for list in [false, true] {
            let mut bytes = canonical.clone();
            if list {
                bytes.splice(36..36, *b"LIST\x04\0\0\0INFO");
                let size = (bytes.len() - 8) as u32;
                bytes[4..8].copy_from_slice(&size.to_le_bytes());
            }
            let path = root.path().join("voice.wav");
            fs::write(&path, &bytes).unwrap();
            let probe = Command::new(env!("CARGO_BIN_EXE_av"))
                .arg("probe")
                .arg(&path)
                .output()
                .unwrap();
            assert!(
                probe.status.success(),
                "{}",
                String::from_utf8_lossy(&probe.stderr)
            );
            let receipt: Value = serde_json::from_slice(&probe.stdout).unwrap();
            assert_eq!(receipt["format"], "RIFF WAV");
            assert_eq!(receipt["tracks"][0]["codec"], "PCM_S16LE");
            assert_eq!(receipt["tracks"][0]["channels"], channels);
            assert_eq!(receipt["tracks"][0]["sample_rate"], 48000);
            assert_eq!(receipt["duration"], 287552.0 / 48000.0);
            let timeline = json!({"version":1,"duration_s":1,"video":{"width":32,"height":32,"fps":30,
                "segments":[{"solid":"black","at_s":0,"duration_s":1}]},
                "audio":{"sample_rate":48000,"channels":2,"sources":[{"path":"voice.wav","at_s":0}],
                    "master":{"integrated_lufs":-14,"true_peak_dbtp":-1}},
                "encode":{"speed":10,"qp":100,"tile_cols":1,"tile_rows":1,"threads":1,"chunks":1,"keyint_s":1}});
            fs::write(
                root.path().join("timeline.json"),
                serde_json::to_vec(&timeline).unwrap(),
            )
            .unwrap();
            let out = root.path().join(format!("{channels}-{list}.webm"));
            let render = Command::new(env!("CARGO_BIN_EXE_av"))
                .arg("render")
                .arg(root.path().join("timeline.json"))
                .arg("--out")
                .arg(out)
                .output()
                .unwrap();
            assert!(
                render.status.success(),
                "{} {}",
                String::from_utf8_lossy(&render.stdout),
                String::from_utf8_lossy(&render.stderr)
            );
        }
        // Unbounded streaming data is not a shape accepted by render's reader.
        let mut streaming = canonical;
        streaming[40..44].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(media_audio::Audio::read_wav(Cursor::new(&streaming)).is_err());
        fs::write(root.path().join("streaming.wav"), streaming).unwrap();
        let probe = Command::new(env!("CARGO_BIN_EXE_av"))
            .arg("probe")
            .arg(root.path().join("streaming.wav"))
            .output()
            .unwrap();
        assert!(!probe.status.success());
        assert!(!String::from_utf8_lossy(&probe.stderr).contains("SizeOverflow"));
    }
}
