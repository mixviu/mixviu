use media_audio::{Audio, LimiterConfig};

fn tone(burst: bool) -> Audio {
    Audio::new(
        48000,
        2,
        (0..48000 * 4)
            .flat_map(|n| {
                let amplitude = if burst && n % 48000 < 480 { 1.6 } else { 0.12 };
                let s =
                    amplitude * (std::f64::consts::TAU * 6000.0 * n as f64 / 48000.0 + 0.4).sin();
                [s, s * 0.3]
            })
            .collect(),
    )
    .unwrap()
}
#[test]
fn burst_ceiling_linking_and_determinism() {
    let mut a = tone(true);
    let mut b = a.clone();
    let r = a.limit_true_peak(-1.0, LimiterConfig::default()).unwrap();
    b.limit_true_peak(-1.0, LimiterConfig::default()).unwrap();
    assert_eq!(a, b);
    assert!(r.max_gain_reduction_db > 4.0);
    assert!(r.after.true_peak_dbtp <= -1.0);
    for f in a.samples().chunks_exact(2) {
        assert!((f[0] * 0.3 - f[1]).abs() < 1e-14);
    }
    assert_eq!(a.frames(), 48000 * 4);
}
#[test]
fn static_path_is_bit_identical() {
    let mut a = tone(false);
    let mut b = a.clone();
    let old = a.normalise(-14.0, -1.0).unwrap();
    let new = b
        .normalise_to_target(-14.0, -1.0, LimiterConfig::default())
        .unwrap();
    assert_eq!(a, b);
    assert_eq!(old.after.integrated_lufs, new.after.integrated_lufs);
    assert_eq!(new.passes, 0);
}
#[test]
fn refusal_returns_legacy_static_result() {
    let mut a = tone(false);
    let mut samples = a.samples().to_vec();
    samples[48000] = 30.0;
    a = Audio::new(48000, 2, samples).unwrap();
    let mut direct = a.clone();
    assert!(matches!(
        direct.limit_true_peak(-1.0, Default::default()),
        Err(media_audio::Error::ReductionLimit { .. })
    ));
    assert_eq!(direct, a);
    let mut b = a.clone();
    b.normalise(-14.0, -1.0).unwrap();
    let r = a
        .normalise_to_target(-14.0, -1.0, LimiterConfig::default())
        .unwrap();
    assert!(r.refused);
    assert_eq!(
        r.passes, 1,
        "first refused render is still an attempted pass"
    );
    assert!(r.pass_cpu_s[0] > 0.0);
    assert!(r.pass_cpu_s[1..].iter().all(|&seconds| seconds == 0.0));
    assert!(r.max_gain_reduction_db > 6.0);
    assert!(r.shortfall_lu > 0.5);
    assert_eq!(a, b);
}

#[test]
fn tiny_buffers_preserve_samples() {
    for samples in [vec![], vec![0.0], vec![0.4], vec![0.2, 0.3]] {
        let mut audio = Audio::new(48000, 1, samples.clone()).unwrap();
        audio.limit_true_peak(-1.0, Default::default()).unwrap();
        assert_eq!(audio.samples(), samples);
    }
}
#[test]
fn impulse_alignment_and_edges() {
    for position in [0, 2400, 4799] {
        let mut samples = vec![0.0; 4800];
        samples[position] = 1.5;
        let mut a = Audio::new(48000, 1, samples).unwrap();
        a.limit_true_peak(-1.0, LimiterConfig::default()).unwrap();
        assert_eq!(a.frames(), 4800);
        assert!(a.samples()[position] > 0.0);
        assert_eq!(a.samples().iter().filter(|&&x| x != 0.0).count(), 1);
        assert!(a.measure().unwrap().true_peak_dbtp <= -1.0);
    }
}
#[test]
fn bounded_target_and_invalid_settings() {
    let base = tone(true);
    let mut a = Audio::new(
        48000,
        2,
        base.samples()
            .iter()
            .enumerate()
            .map(|(i, &x)| if i / 2 % 48000 < 480 { x * 0.5 } else { x })
            .collect(),
    )
    .unwrap();
    let r = a
        .normalise_to_target(-14.0, -1.0, LimiterConfig::default())
        .unwrap();
    assert!(!r.refused, "{r:?}");
    assert!(r.target_reached);
    assert!((r.after.integrated_lufs + 14.0).abs() <= 0.1, "{r:?}");
    assert!(r.after.true_peak_dbtp <= -1.0);
    assert!((1..=8).contains(&r.passes));
    assert!(
        a.limit_true_peak(
            -1.0,
            LimiterConfig {
                lookahead_ms: f64::NAN,
                release_ms: 100.0
            }
        )
        .is_err()
    );
}

#[test]
fn intersample_peak_nondefault_timing_and_right_channel_trigger() {
    for rate in [44100, 48000, 96000] {
        let source: Vec<_> = (0..rate)
            .flat_map(|n| {
                let s =
                    (std::f64::consts::TAU * 0.25 * n as f64 + std::f64::consts::FRAC_PI_4).sin();
                [s * 0.05, s * 1.05]
            })
            .collect();
        let mut a = Audio::new(rate, 2, source.clone()).unwrap();
        let before = a.measure().unwrap();
        assert!(before.true_peak_dbtp > before.sample_peak_dbfs + 1.0);
        let r = a
            .limit_true_peak(
                -1.0,
                LimiterConfig {
                    lookahead_ms: 8.0,
                    release_ms: 75.0,
                },
            )
            .unwrap();
        assert!(r.after.true_peak_dbtp <= -1.0);
        assert!(r.max_gain_reduction_db > 0.5);
        for (input, output) in source.chunks_exact(2).zip(a.samples().chunks_exact(2)) {
            assert!((output[0] / input[0] - output[1] / input[1]).abs() < 1e-12);
        }
    }
}
