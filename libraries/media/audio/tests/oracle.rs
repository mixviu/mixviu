//! Requires the worker's ffmpeg; missing executable/filter is a failure.
use media_audio::{Audio, WavFormat};
use serde_json::Value;
use std::{fs::File, process::Command};
fn ff(args: &[&str]) -> String {
    let o = Command::new("ffmpeg")
        .args(["-hide_banner", "-nostats", "-y"])
        .args(args)
        .output()
        .expect("ffmpeg required on oracle worker");
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8(o.stderr).unwrap()
}
fn number(s: &str) -> f64 {
    s.parse().unwrap()
}
#[test]
fn generated_tone_pink_noise_and_linear_loudnorm() {
    let dir = std::env::temp_dir().join(format!("audio-oracle-{}", std::process::id()));
    std::fs::create_dir(&dir).unwrap();
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
        a.write_wav(File::create(&src).unwrap(), WavFormat::Float32)
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
        let fm = Audio::read_wav(File::open(dst).unwrap())
            .unwrap()
            .measure()
            .unwrap();
        let r = a.normalise(-14.0, -1.0).unwrap();
        println!("pink={pink} linear rust={:?} ffmpeg={fm:?}", r.after);
        assert!((r.after.integrated_lufs - fm.integrated_lufs).abs() <= 0.5);
        assert!((r.after.true_peak_dbtp - fm.true_peak_dbtp).abs() <= 0.3);
    }
    std::fs::remove_dir_all(dir).unwrap();
}
