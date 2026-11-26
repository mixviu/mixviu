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
        (0usize..48000 * 30)
            .flat_map(|n| {
                let t = n as f64 / 48000.0;
                let phrase = 10.0_f64.powf(if (n / (48000 * 6)).is_multiple_of(2) {
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
                let plosive = if (5700..5940).contains(&(n % 46828)) {
                    2.2
                } else {
                    1.0
                };
                let x = 0.18 * phrase * (0.15 + 0.85 * syllable) * voiced * plosive;
                [x, x * 0.9]
            })
            .collect(),
    )
    .unwrap()
}

#[test]
fn short_quiet_ceiling_limited_requires_explicit_opt_in() {
    let dir = std::env::temp_dir().join(format!("short-ceiling-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let mut clip = Audio::new(48000, 2, speech().samples()[..120000 * 2].to_vec()).unwrap();
    // A sparse stronger plosive makes this a ceiling-limited specimen rather
    // than relying on duration alone to cause a refusal.
    let samples = clip.samples().iter().enumerate().map(|(i, x)| {
        x * if (5700..5940).contains(&((i / 2) % 46828)) { 2.7 / 2.2 } else { 1.0 }
    }).collect();
    clip = Audio::new(48000, 2, samples).unwrap();
    clip.normalise(-20.5, 0.0).unwrap();
    assert!((clip.measure().unwrap().integrated_lufs + 20.5).abs() < 0.01);
    clip.write_wav(File::create(dir.join("short.wav")).unwrap(), WavFormat::Float32).unwrap();
    let mut t: Timeline = serde_json::from_value(json!({"version":1,"sample_rate":48000,"channels":2,
        "duration_s":2.5,"sources":[{"path":"short.wav","at_s":0}],
        "master":{"integrated_lufs":-14,"true_peak_dbtp":-1}})).unwrap();
    let out = dir.join("short.webm");
    let wav = dir.join("short-master.wav");
    let refused = render(&t, &dir, &out, Some(&wav)).unwrap();
    println!("short default: {}", serde_json::to_string(&refused).unwrap());
    assert!(refused.refused && !refused.target_reached);
    assert!(!out.exists() && !wav.exists());
    t.master.accept_ceiling_limited = true;
    let accepted = render(&t, &dir, &out, Some(&wav)).unwrap();
    assert!(!accepted.refused && !accepted.target_reached);
    assert!(accepted.shortfall_lu > 0.1);
    assert!(accepted.encoded.true_peak_dbtp <= -1.0);
    assert_eq!(accepted.encoded.integrated_lufs, refused.encoded.integrated_lufs);
    assert_eq!(accepted.passes, refused.passes);
    assert_eq!(accepted.codec_attempts, accepted.codec_passes + 1);
    assert_eq!(accepted.mastering_cost.iter().map(|c| c.limiter_passes).sum::<u8>(), accepted.passes);
    assert!(out.exists() && wav.exists());
    std::fs::remove_dir_all(dir).unwrap();
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
    assert!(
        r.codec_passes > 0,
        "fixture must exercise codec corrections"
    );
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
    let repeat_out = dir.join("repeat.webm");
    let repeat = render(&t, &dir, &repeat_out, None).unwrap();
    assert_eq!(
        std::fs::read(&out).unwrap(),
        std::fs::read(&repeat_out).unwrap()
    );
    assert_eq!(r.encoded.integrated_lufs, repeat.encoded.integrated_lufs);
    assert_eq!(r.passes, repeat.passes);
    assert_eq!(r.codec_passes, repeat.codec_passes);
    // Impossible peak/loudness pair must return measured refusal and publish nothing.
    let mut impossible = t;
    impossible.master.true_peak_dbtp = -20.0;
    let refused_out = dir.join("refused.webm");
    let refused_wav = dir.join("refused.wav");
    let mut before_refusal: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    before_refusal.sort_unstable();
    let refused = render(&impossible, &dir, &refused_out, Some(&refused_wav)).unwrap();
    println!(
        "limiter-bound refusal: {}",
        serde_json::to_string(&refused).unwrap()
    );
    assert!(refused.refused && !refused.target_reached);
    assert_eq!(refused.passes, 1);
    assert_eq!(refused.mastering_cost.len(), 1);
    assert_eq!(refused.mastering_cost[0].limiter_passes, 1);
    assert_eq!(refused.mastering_cost[0].limiter_cpu_s.len(), 1);
    assert!(refused.mastering_cost[0].limiter_cpu_s[0] > 0.0);
    assert!(refused.encoded.integrated_lufs < -14.1);
    assert!(!refused_out.exists());
    assert!(!refused_wav.exists());
    let mut after_refusal: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    after_refusal.sort_unstable();
    assert_eq!(
        before_refusal, after_refusal,
        "no staging files left behind"
    );
    if let Ok(artifact) = std::env::var("MIX_TEST_MASTERING_ARTIFACT_DIR") {
        std::fs::create_dir_all(&artifact).unwrap();
        std::fs::copy(&out, std::path::Path::new(&artifact).join("speech.webm")).unwrap();
        std::fs::copy(
            dir.join("speech.wav"),
            std::path::Path::new(&artifact).join("speech.wav"),
        )
        .unwrap();
        std::fs::write(
            std::path::Path::new(&artifact).join("receipt.json"),
            serde_json::to_vec_pretty(&r).unwrap(),
        )
        .unwrap();
    }
    std::fs::remove_dir_all(dir).unwrap();
}
