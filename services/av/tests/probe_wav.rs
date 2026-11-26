use serde_json::{Value, json};
use std::{fs, io::Cursor, process::Command};

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
