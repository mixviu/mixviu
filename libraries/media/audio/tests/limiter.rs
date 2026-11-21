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
    let mut b = a.clone();
    b.normalise(-14.0, -1.0).unwrap();
    let r = a
        .normalise_to_target(-14.0, -1.0, LimiterConfig::default())
        .unwrap();
    assert!(r.refused);
    assert!(r.max_gain_reduction_db > 6.0);
    assert!(r.shortfall_lu > 0.5);
    assert_eq!(a, b);
}
#[test]
fn impulse_alignment_and_edges() {
    for position in [0, 2400, 4799] {
        let mut samples = vec![0.0; 4800];
        samples[position] = 2.0;
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
    let mut a = Audio::new(48000, 2, base.samples().iter().enumerate().map(|(i, &x)| {
        if i / 2 % 48000 < 480 { x * 0.375 } else { x }
    }).collect()).unwrap();
    let r = a
        .normalise_to_target(-14.0, -1.0, LimiterConfig::default())
        .unwrap();
    assert!(!r.refused, "{r:?}");
    assert!((r.after.integrated_lufs + 14.0).abs() <= 0.5, "{r:?}");
    assert!(r.after.true_peak_dbtp <= -1.0);
    assert!((1..=3).contains(&r.passes));
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
