//! ffmpeg oracle. Hosted CI has no ffmpeg: the oracle is skipped there, and
//! required (a missing ffmpeg fails) when MIX_TEST_REQUIRE_FFMPEG=1, which
//! `tools/ci.mix all` sets on CBC. Same convention as engines/mixd/tests/vfl.rs.
use media_audio::{Audio, WavFormat};
use serde_json::Value;
use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
    process::Command,
};
struct TempDir(PathBuf);
impl TempDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "audio-oracle-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn ff(args: &[&str]) -> String {
    let o = Command::new("ffmpeg")
        .args(["-hide_banner", "-nostats", "-y"])
        .args(args)
        .output()
        .expect("ffmpeg required on oracle worker");
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8(o.stderr).unwrap()
}
/// True when the ffmpeg oracle can run. Absent ffmpeg skips the oracle unless
/// MIX_TEST_REQUIRE_FFMPEG=1, which turns absence into a failure.
fn oracle_available() -> bool {
    match Command::new("ffmpeg").arg("-version").output() {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            assert_ne!(
                std::env::var("MIX_TEST_REQUIRE_FFMPEG").as_deref(),
                Ok("1"),
                "ffmpeg required: loudness oracle cannot run"
            );
            eprintln!("SKIP: ffmpeg absent, loudness oracle not run");
            false
        }
        Err(e) => panic!("cannot run ffmpeg: {e}"),
        Ok(o) => {
            assert!(o.status.success(), "ffmpeg probe failed: {o:?}");
            true
        }
    }
}
fn number(s: &str) -> f64 {
    s.parse().unwrap()
}
#[test]
fn generated_tone_pink_noise_and_linear_loudnorm() {
    if !oracle_available() {
        return;
    }
    let temp = TempDir::new();
    let dir = &temp.0;
    for pink in [false, true] {
        let mut seed = 17u32;
        let mut rows = [0.0; 12];
        let samples = (0..48000 * 20)
            .flat_map(|n| {
                let s = if pink {
                    // Voss-McCartney: octave-rate independent random rows; fixed seed.
                    seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                    let idx = (n as u32 + 1).trailing_zeros() as usize % 12;
                    rows[idx] = (seed as f64 / u32::MAX as f64 - 0.5) * 0.025;
                    rows.iter().sum::<f64>()
                } else {
                    10.0_f64.powf((if n < 48000 * 10 { -23.0 } else { -24.0 }) / 20.0)
                        * (std::f64::consts::TAU * 1000.0 * n as f64 / 48000.0).sin()
                };
                [s, s]
            })
            .collect();
        let mut a = Audio::new(48000, 2, samples).unwrap();
        let src = dir.join("source.wav");
        let dst = dir.join("linear.wav");
        a.write_wav(
            BufWriter::new(File::create(&src).unwrap()),
            WavFormat::Float32,
        )
        .unwrap();
        let src = src.to_str().unwrap();
        let dst = dst.to_str().unwrap();
        let log = ff(&["-i", src, "-af", "ebur128=peak=true", "-f", "null", "-"]);
        let summary = log.rsplit("Summary:").next().unwrap();
        let get = |key: &str| {
            let t = summary.split(key).nth(1).expect("ebur128 summary field");
            number(t.split_whitespace().next().unwrap())
        };
        let m = a.measure().unwrap();
        let di = m.integrated_lufs - get("I:");
        let dt = m.true_peak_dbtp - get("Peak:");
        println!(
            "pink={pink} rust={m:?} ffmpeg I={} TP={} delta_i={di} delta_tp={dt}",
            get("I:"),
            get("Peak:")
        );
        assert!(di.abs() <= 0.5);
        assert!(dt.abs() <= 0.3);
        let log = ff(&[
            "-i",
            src,
            "-af",
            "loudnorm=I=-14:TP=-1:LRA=50:print_format=json",
            "-f",
            "null",
            "-",
        ]);
        let start = log.rfind('{').expect("loudnorm JSON required");
        let v: Value = serde_json::from_str(&log[start..=log.rfind('}').unwrap()]).unwrap();
        let filter = format!(
            "loudnorm=I=-14:TP=-1:LRA=50:measured_I={}:measured_TP={}:measured_LRA={}:measured_thresh={}:offset=0:linear=true:print_format=json,aresample=48000",
            v["input_i"].as_str().unwrap(),
            v["input_tp"].as_str().unwrap(),
            v["input_lra"].as_str().unwrap(),
            v["input_thresh"].as_str().unwrap()
        );
        let second = ff(&["-i", src, "-af", &filter, "-c:a", "pcm_f32le", dst]);
        assert!(
            second.contains("\"normalization_type\" : \"linear\""),
            "linear comparison must not silently become dynamic: {second}"
        );
        let fm = Audio::read_wav(BufReader::new(File::open(dst).unwrap()))
            .unwrap()
            .measure()
            .unwrap();
        let r = a.normalise(-14.0, -1.0).unwrap();
        println!("pink={pink} linear rust={:?} ffmpeg={fm:?}", r.after);
        assert!((r.after.integrated_lufs - fm.integrated_lufs).abs() <= 0.5);
        assert!((r.after.true_peak_dbtp - fm.true_peak_dbtp).abs() <= 0.3);
    }
}

#[test]
fn temporary_directory_is_removed_on_panic() {
    let mut path = None;
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let dir = TempDir::new();
        path = Some(dir.0.clone());
        std::fs::write(dir.0.join("partial.wav"), b"partial").unwrap();
        panic!("exercise unwind cleanup");
    }));
    assert!(result.is_err());
    assert!(!path.unwrap().exists());
}

#[test]
fn limited_burst_true_peak_oracle() {
    if !oracle_available() {
        return;
    }
    let temp = TempDir::new();
    // 6 kHz burst, 1% duty: analytic crest 23.01 dB; discrete sample crest
    // is 22.35 dB at this phase (sample peaks do not equal sine amplitude).
    let samples: Vec<f64> = (0..48000 * 4)
        .map(|n| {
            let level = if n % 48000 < 480 { 1.6 } else { 0.0 };
            level * (std::f64::consts::TAU * 6000.0 * n as f64 / 48000.0 + 0.4).sin()
        })
        .collect();
    let rms = (samples.iter().map(|x| x*x).sum::<f64>() / samples.len() as f64).sqrt();
    let peak = samples.iter().map(|x| x.abs()).fold(0.0_f64, f64::max);
    assert!((20.0*(peak/rms).log10() - 22.35).abs() < 0.01);
    let mut a = Audio::new(48000, 1, samples).unwrap();
    let r = a.limit_true_peak(-1.0, Default::default()).unwrap();
    assert!(r.after.true_peak_dbtp <= -1.0);
    let path = temp.0.join("limited.wav");
    a.write_wav(
        BufWriter::new(File::create(&path).unwrap()),
        WavFormat::Float32,
    )
    .unwrap();
    let log = ff(&[
        "-i",
        path.to_str().unwrap(),
        "-af",
        "ebur128=peak=true",
        "-f",
        "null",
        "-",
    ]);
    let summary = log.rsplit("Summary:").next().unwrap();
    let peak = number(
        summary
            .split("Peak:")
            .nth(1)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap(),
    );
    println!("limited burst Rust={:?} ffmpeg TP={peak}", r.after);
    assert!(peak <= -0.9);
}
