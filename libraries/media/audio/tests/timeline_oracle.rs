use media_audio::{
    Audio, WavFormat,
    timeline::{Timeline, render},
};
use serde_json::json;
use std::{fs::File, process::Command};
fn ff(args: &[&str]) -> std::process::Output {
    let o = Command::new("ffmpeg")
        .args(["-hide_banner", "-nostats", "-y"])
        .args(args)
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    o
}
#[test]
fn mix_decode_length_and_mastering_oracles() {
    if Command::new("ffmpeg").arg("-version").output().is_err() {
        assert_ne!(std::env::var("MIX_TEST_REQUIRE_FFMPEG").as_deref(), Ok("1"));
        return;
    }
    let dir = std::env::temp_dir().join(format!("timeline-oracle-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let src = dir.join("tone.wav");
    Audio::new(
        48000,
        1,
        (0..480000)
            .map(|n| 0.1 * (std::f64::consts::TAU * 440.0 * n as f64 / 48000.0).sin())
            .collect(),
    )
    .unwrap()
    .write_wav(File::create(&src).unwrap(), WavFormat::Float32)
    .unwrap();
    let t:Timeline=serde_json::from_value(json!({"version":1,"sample_rate":48000,"channels":2,"duration_s":10.000354166666667,"sources":[{"path":"tone.wav","at_s":0},{"path":"tone.wav","at_s":17.0/48000.0,"gain_db":-6}],"master":{"integrated_lufs":-14,"true_peak_dbtp":-1}})).unwrap();
    let mixed = t.mix(&dir).unwrap();
    let o = ff(&[
        "-i",
        src.to_str().unwrap(),
        "-filter_complex",
        "[0:a]aformat=channel_layouts=stereo,asplit[a][b];[b]volume=-6dB,adelay=17S:all=1[b1];[a][b1]amix=inputs=2:normalize=0,apad,atrim=end_sample=480017[out]",
        "-map",
        "[out]",
        "-f",
        "f32le",
        "-",
    ]);
    let oracle: Vec<f64> = o
        .stdout
        .chunks_exact(4)
        .map(|b| f32::from_le_bytes(b.try_into().unwrap()) as f64)
        .collect();
    assert_eq!(oracle.len(), mixed.samples().len());
    let rms = (oracle
        .iter()
        .zip(mixed.samples())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f64>()
        / oracle.len() as f64)
        .sqrt();
    assert!(rms < 0.001, "mix RMS {rms}");
    for length in [480000, 480017, 480800] {
        let t:Timeline=serde_json::from_value(json!({"version":1,"sample_rate":48000,"channels":2,"duration_s":length as f64/48000.0,"sources":[{"path":"tone.wav","at_s":0}],"master":{"integrated_lufs":-14,"true_peak_dbtp":-1}})).unwrap();
        let out = dir.join("out.webm");
        let wav = dir.join("master.wav");
        let r = render(&t, &dir, &out, Some(&wav)).unwrap();
        assert!(r.target_reached);
        let o = ff(&[
            "-c:a",
            "libopus",
            "-i",
            out.to_str().unwrap(),
            "-f",
            "f32le",
            "-",
        ]);
        assert_eq!(o.stdout.len() / 8, length, "decoded length");
        let o = ff(&[
            "-i",
            out.to_str().unwrap(),
            "-af",
            "ebur128=peak=true",
            "-f",
            "null",
            "-",
        ]);
        let log = String::from_utf8(o.stderr).unwrap();
        let summary = log.rsplit("Summary:").next().unwrap();
        let get = |key: &str| {
            summary
                .split(key)
                .nth(1)
                .unwrap()
                .split_whitespace()
                .next()
                .unwrap()
                .parse::<f64>()
                .unwrap()
        };
        assert!((get("I:") + 14.0).abs() <= 0.1, "{summary}");
        assert!(get("Peak:") <= -1.0, "{summary}");
        std::fs::remove_file(out).unwrap();
        std::fs::remove_file(wav).unwrap();
    }
    // Speech-like crest and sharp envelope changes exercise codec peak safety,
    // independently measured after libopus decoding, not just mastered PCM.
    let src = dir.join("burst.wav");
    let samples = (0..48000 * 12)
        .map(|n| {
            let level = if n % 48000 < 4800 { 0.8 } else { 0.04 };
            level * (std::f64::consts::TAU * 440.0 * n as f64 / 48000.0).sin()
        })
        .collect();
    Audio::new(48000, 1, samples)
        .unwrap()
        .write_wav(File::create(&src).unwrap(), WavFormat::Float32)
        .unwrap();
    let t:Timeline=serde_json::from_value(json!({"version":1,"sample_rate":48000,"channels":2,"duration_s":12,"sources":[{"path":"burst.wav","at_s":0}],"master":{"integrated_lufs":-14,"true_peak_dbtp":-1}})).unwrap();
    let out = dir.join("burst.webm");
    let r = render(&t, &dir, &out, None).unwrap();
    let o = ff(&[
        "-c:a",
        "libopus",
        "-i",
        out.to_str().unwrap(),
        "-af",
        "ebur128=peak=true:framelog=verbose",
        "-f",
        "null",
        "-",
    ]);
    let log = String::from_utf8(o.stderr).unwrap();
    let summary = log.rsplit("Summary:").next().unwrap();
    let peak = summary
        .split("Peak:")
        .nth(1)
        .unwrap()
        .split_whitespace()
        .next()
        .unwrap()
        .parse::<f64>()
        .unwrap();
    assert!(peak <= -1.0, "{summary}");
    assert!(r.codec_passes < 6);
    std::fs::remove_dir_all(dir).unwrap();
}
