use media_audio::{
    Audio, WavFormat,
    timeline::{Timeline, render},
};
use serde_json::json;
use std::{fs::File, process::Command};

fn speech() -> Audio {
    // Voiced harmonics, syllables, word gaps and alternating six-second phrases.
    // Entirely deterministic; phrase-level contrast produces roughly 4 LU LRA.
    Audio::new(
        48000,
        2,
        (0..48000 * 30)
            .flat_map(|n| {
                let t = n as f64 / 48000.0;
                let phrase = 10.0_f64.powf(if (n / (48000 * 6)) % 2 == 0 {
                    -2.3 / 20.0
                } else {
                    2.3 / 20.0
                });
                let syllable = (std::f64::consts::PI * (t * 4.1).fract()).sin().powi(2);
                let voiced = (1..=12)
                    .map(|h| (std::f64::consts::TAU * 137.0 * h as f64 * t).sin() / h as f64)
                    .sum::<f64>();
                // Five-ms plosives every fourth syllable demand about 4 dB
                // limiting at -14 LUFS without breaching the 6 dB cap.
                let plosive = if (5700..5940).contains(&(n % 46828)) { 2.2 } else { 1.0 };
                let x = 0.18 * phrase * (0.15 + 0.85 * syllable) * voiced * plosive;
                [x, x * 0.9]
            })
            .collect(),
    )
    .unwrap()
}

#[test]
fn speech_encoded_target_and_oracle() {
    let dir = std::env::temp_dir().join(format!("mastering-speech-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let source = speech();
    let before = source.measure().unwrap();
    println!("speech input: {before:?}");
    assert!((3.0..=5.0).contains(&before.range_lu), "{before:?}");
    source
        .write_wav(
            File::create(dir.join("speech.wav")).unwrap(),
            WavFormat::Float32,
        )
        .unwrap();
    let t: Timeline = serde_json::from_value(json!({"version":1,"sample_rate":48000,"channels":2,"duration_s":30,"sources":[{"path":"speech.wav","at_s":0}],"master":{"integrated_lufs":-14,"true_peak_dbtp":-1}})).unwrap();
    let out = dir.join("speech.webm");
    let r = render(&t, &dir, &out, None).unwrap();
    println!("speech receipt: {}", serde_json::to_string(&r).unwrap());
    assert!(r.target_reached && !r.refused);
    assert!(r.passes > 1, "fixture must exercise limiter corrections");
    assert!(r.codec_passes > 0, "fixture must exercise codec corrections");
    assert!((-14.1..=-13.9).contains(&r.encoded.integrated_lufs));
    // Serialize to inspect the same peak field supplied to callers.
    assert!(
        serde_json::to_value(&r).unwrap()["encoded"]["true_peak_dbtp"]
            .as_f64()
            .unwrap()
            <= -1.0
    );
    if Command::new("ffmpeg").arg("-version").output().is_ok() {
        let oracle = Command::new("ffmpeg")
            .args(["-hide_banner", "-nostats", "-i"])
            .arg(&out)
            .args([
                "-af",
                "ebur128=peak=true:framelog=verbose",
                "-f",
                "null",
                "-",
            ])
            .output()
            .unwrap();
        assert!(oracle.status.success());
        let log = String::from_utf8(oracle.stderr).unwrap();
        let summary = log.rsplit("Summary:").next().unwrap();
        println!("speech ffmpeg ebur128: {summary}");
        let field = |key| {
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
        assert!((-14.1..=-13.9).contains(&field("I:")), "{summary}");
        assert!(field("Peak:") <= -1.0, "{summary}");
    } else {
        assert_ne!(std::env::var("MIX_TEST_REQUIRE_FFMPEG").as_deref(), Ok("1"));
    }
    // Impossible peak/loudness pair must return measured refusal and publish nothing.
    let mut impossible = t;
    impossible.master.true_peak_dbtp = -20.0;
    let refused_out = dir.join("refused.webm");
    let refused = render(&impossible, &dir, &refused_out, None).unwrap();
    assert!(refused.refused && !refused.target_reached);
    assert!(refused.encoded.integrated_lufs < -14.1);
    assert!(!refused_out.exists());
    std::fs::remove_dir_all(dir).unwrap();
}
