use media_audio::{Audio, WavFormat, timeline::Timeline};
use serde_json::{Value, json};
fn spec() -> Value {
    json!({"version":1,"sample_rate":48000,"channels":2,"duration_s":1.0,"sources":[],"master":{"integrated_lufs":-14,"true_peak_dbtp":-1}})
}
#[test]
fn ceiling_limited_opt_in_wire_format() {
    let mut v = spec();
    let default: Timeline = serde_json::from_value(v.clone()).unwrap();
    assert!(!default.master.accept_ceiling_limited);
    v["master"]["accept_ceiling_limited"] = json!(true);
    let t: Timeline = serde_json::from_str(&v.to_string()).unwrap();
    assert!(t.master.accept_ceiling_limited);
    t.validate().unwrap();
}
#[test]
fn validation() {
    for (field, value) in [
        ("version", json!(2)),
        ("sample_rate", json!(44100)),
        ("channels", json!(1)),
        ("duration_s", json!(-1)),
        ("duration_s", json!(10801)),
        ("extra", json!(0)),
    ] {
        let mut v = spec();
        v[field] = value;
        assert!(
            serde_json::from_value::<Timeline>(v)
                .map(|t| t.validate().is_err())
                .unwrap_or(true),
            "{field}"
        );
    }
    for (field, value) in [
        ("at_s", json!(-1)),
        ("at_s", json!(2)),
        ("from_s", json!(-1)),
        ("to_s", json!(-1)),
        ("gain_db", json!(61)),
        ("fade_in_s", json!(2)),
        ("fade_out_s", json!(2)),
        ("extra", json!(0)),
    ] {
        let mut v = spec();
        v["sources"] = json!([{"path":"x","at_s":0,"to_s":1}]);
        v["sources"][0][field] = value;
        assert!(
            serde_json::from_value::<Timeline>(v)
                .map(|t| t.validate().is_err())
                .unwrap_or(true),
            "{field}"
        );
    }
    let mut t: Timeline = serde_json::from_value(spec()).unwrap();
    t.duration_s = f64::NAN;
    assert!(t.validate().is_err());
    let mut v = spec();
    v["sources"] = json!(vec![json!({"path":"x","at_s":0}); 10001]);
    assert!(
        serde_json::from_value::<Timeline>(v)
            .unwrap()
            .validate()
            .is_err()
    );
}
#[test]
fn nonfinite_and_nested_validation() {
    // Decimal subtraction must not reject an equal-length, frame-exact fade.
    let mut v = spec();
    v["sources"] = json!([{"path":"x","at_s":0,"from_s":0.1,"to_s":0.3,"fade_in_s":0.2}]);
    assert!(
        serde_json::from_value::<Timeline>(v)
            .unwrap()
            .validate()
            .is_ok()
    );
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY, -0.1] {
        for field in 0..7 {
            let mut v = spec();
            v["sources"] = json!([{"path":"x","at_s":0}]);
            let mut t: Timeline = serde_json::from_value(v).unwrap();
            match field {
                0 => t.sources[0].at_s = bad,
                1 => t.sources[0].from_s = bad,
                2 => t.sources[0].to_s = Some(bad),
                3 => t.sources[0].fade_in_s = bad,
                4 => t.sources[0].fade_out_s = bad,
                5 => t.sources[0].gain_db = if bad.is_finite() { 61.0 } else { bad },
                _ => t.duration_s = bad,
            }
            assert!(t.validate().is_err(), "field {field} value {bad}");
        }
    }
    for (field, value) in [
        ("integrated_lufs", json!(-71)),
        ("true_peak_dbtp", json!(1)),
        ("unknown", json!(0)),
    ] {
        let mut v = spec();
        v["master"][field] = value;
        assert!(
            serde_json::from_value::<Timeline>(v)
                .map(|t| t.validate().is_err())
                .unwrap_or(true)
        );
    }
    let mut v = spec();
    v["sources"] = json!([{"path":"x","at_s":0,"from_s":1,"to_s":0}]);
    assert!(
        serde_json::from_value::<Timeline>(v)
            .unwrap()
            .validate()
            .is_err()
    );
}
#[test]
fn placement_trim_fades_crossfade_and_mono() {
    let dir = std::env::temp_dir().join(format!("timeline-unit-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    Audio::new(48000, 1, vec![0.5; 8])
        .unwrap()
        .write_wav(
            std::fs::File::create(dir.join("x.wav")).unwrap(),
            WavFormat::Float32,
        )
        .unwrap();
    let mut v = spec();
    v["duration_s"] = json!(12.0 / 48000.0);
    v["sources"] = json!([{"path":"x.wav","at_s":2.0/48000.0,"from_s":2.0/48000.0,"to_s":6.0/48000.0,"fade_in_s":4.0/48000.0},{"path":"x.wav","at_s":12.0/48000.0}]);
    let a = serde_json::from_value::<Timeline>(v)
        .unwrap()
        .mix(&dir)
        .unwrap();
    assert_eq!(a.frames(), 12);
    assert_eq!(&a.samples()[..6], &[0.0; 6]);
    for k in 0..4 {
        assert!(
            (a.samples()[(k + 2) * 2] - 0.5 * std::f64::consts::FRAC_1_SQRT_2 * k as f64 / 3.0)
                .abs()
                < 1e-7
        );
    }
    let mut v = spec();
    v["duration_s"] = json!(8.0 / 48000.0);
    v["sources"] = json!([{"path":"x.wav","at_s":0,"fade_in_s":8.0/48000.0},{"path":"x.wav","at_s":0,"fade_out_s":8.0/48000.0}]);
    let a = serde_json::from_value::<Timeline>(v)
        .unwrap()
        .mix(&dir)
        .unwrap();
    assert!(
        a.samples()
            .iter()
            .all(|s| (s - 0.5 * std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-7)
    );
    let samples: Vec<f64> = (0..16).map(|k| (k / 2) as f64 / 16.0).collect();
    Audio::new(48000, 2, samples.clone())
        .unwrap()
        .write_wav(
            std::fs::File::create(dir.join("trim.wav")).unwrap(),
            WavFormat::Float32,
        )
        .unwrap();
    let mut v = spec();
    v["duration_s"] = json!(4.0 / 48000.0);
    v["sources"] = json!([{"path":"trim.wav","at_s":0,"from_s":2.0/48000.0,"to_s":6.0/48000.0}]);
    let a = serde_json::from_value::<Timeline>(v.clone())
        .unwrap()
        .mix(&dir)
        .unwrap();
    assert_eq!(a.samples(), &samples[4..12]);
    v["sources"][0]["to_s"] = json!(1);
    assert!(
        serde_json::from_value::<Timeline>(v)
            .unwrap()
            .mix(&dir)
            .is_err()
    );
    let tone: Vec<f64> = (0..480)
        .flat_map(|k| {
            let s = 0.1 * (std::f64::consts::TAU * 1000.0 * k as f64 / 48000.0).sin();
            [s, s]
        })
        .collect();
    Audio::new(48000, 2, tone.clone())
        .unwrap()
        .write_wav(
            std::fs::File::create(dir.join("cross.wav")).unwrap(),
            WavFormat::Float32,
        )
        .unwrap();
    let mut v = spec();
    v["duration_s"] = json!(0.01);
    v["sources"] = json!([{"path":"cross.wav","at_s":0,"fade_in_s":0.01},{"path":"cross.wav","at_s":0,"fade_out_s":0.01}]);
    let a = serde_json::from_value::<Timeline>(v.clone())
        .unwrap()
        .mix(&dir)
        .unwrap();
    assert!(
        a.samples()
            .iter()
            .zip(tone)
            .all(|(a, b)| (a - b).abs() < 1e-7)
    );
    v["sources"][0]["fade_in_s"] = json!(0.02);
    assert!(
        serde_json::from_value::<Timeline>(v)
            .unwrap()
            .mix(&dir)
            .is_err()
    );
    std::fs::remove_dir_all(dir).unwrap();
}
