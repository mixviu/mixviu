use media_audio::{Audio, Error, LimiterConfig};

fn bed_with_isp() -> Audio {
    Audio::new(
        48000,
        2,
        (0..48000 * 4)
            .flat_map(|n| {
                let right = if (96000..96960).contains(&n) {
                    let edge = ((n - 96000).min(96959 - n) as f64 / 128.0).min(1.0);
                    1.05 * edge
                        * (std::f64::consts::TAU * 0.25 * n as f64 + std::f64::consts::FRAC_PI_4)
                            .sin()
                } else {
                    0.01
                };
                [0.01, right]
            })
            .collect(),
    )
    .unwrap()
}
fn burst_mix() -> Audio {
    Audio::new(
        48000,
        2,
        (0..48000 * 4)
            .flat_map(|n| {
                let s = (if n % 48000 < 480 { 1.6 } else { 0.12 })
                    * (std::f64::consts::TAU * 6000.0 * n as f64 / 48000.0 + 0.4).sin();
                [s, s * 0.3]
            })
            .collect(),
    )
    .unwrap()
}
fn gain(a: &Audio, n: usize) -> f64 {
    a.samples()[n * 2] / 0.01
}
#[test]
fn target_error_is_transactional() {
    let mut a = burst_mix();
    let original = a.clone();
    assert!(
        a.normalise_to_target(-14.0, -10000.0, Default::default())
            .is_err()
    );
    assert!(a == original, "input was modified");
}
#[test]
fn unmeasurable_target_preserves_input() {
    for mut a in [
        Audio::silence(48000, 2, 48000).unwrap(),
        burst_mix(),
        Audio::new(48000, 1, vec![1e-12; 48000]).unwrap(),
    ] {
        let original = a.clone();
        assert!(matches!(
            a.normalise_to_target(-80.0, -1.0, Default::default()),
            Err(Error::Unmeasurable)
        ));
        assert!(a == original, "input was modified");
    }
}
#[test]
fn limiter_locality_preserves_distant_bed_exactly() {
    let mut a = bed_with_isp();
    a.limit_true_peak(-1.0, Default::default()).unwrap();
    for n in [48000, 144960] {
        assert_eq!(a.samples()[n * 2], 0.01, "frame {n}");
    }
}
#[test]
fn lookahead_precedes_peak_with_bounded_slope() {
    // Abrupt onset, with a nonzero pilot so pre-burst gain is observable.
    let mut a = Audio::new(48000, 2, (0..192000).flat_map(|n| {
        [0.01, if (96000..96960).contains(&n) { 1.3 } else { 0.01 }]
    }).collect()).unwrap();
    a.limit_true_peak(-1.0, Default::default()).unwrap();
    assert!(gain(&a, 95990) < gain(&a, 95800) - 0.001);
    let step = 1.0 / (48000.0 * 0.005);
    for n in 95800..96000 {
        assert!(gain(&a, n) - gain(&a, n + 1) <= step + 1e-12);
    }
}
#[test]
fn release_has_one_hundred_ms_time_constant() {
    let mut a = bed_with_isp();
    a.limit_true_peak(-1.0, LimiterConfig::default()).unwrap();
    let maximum = (96000..96960)
        .map(|n| 1.0 - gain(&a, n))
        .fold(0.0_f64, f64::max);
    let remaining = 1.0 - gain(&a, 96960 + 4800);
    println!("release ratio={} expected={} tolerance=0.025", remaining/maximum, (-1.0_f64).exp());
    // The 128-frame fade-out advances recovery by at most 2.67 ms.
    assert!(
        (remaining / maximum - (-1.0_f64).exp()).abs() < 0.025,
        "ratio={}",
        remaining / maximum
    );
}
#[test]
fn four_x_detection_is_local_not_a_global_rescue() {
    let mut a = bed_with_isp();
    assert!(a.measure().unwrap().sample_peak_dbfs < -1.0);
    assert!(a.measure().unwrap().true_peak_dbtp > -1.0);
    a.limit_true_peak(-1.0, Default::default()).unwrap();
    assert!(a.measure().unwrap().true_peak_dbtp <= -1.0);
    assert!(gain(&a, 96400) < 0.9);
    assert_eq!(gain(&a, 48000), 1.0);
}
#[test]
fn corrective_refusal_keeps_best_in_bound() {
    let mut a = burst_mix();
    let r = a
        .normalise_to_target(-14.0, -1.0, Default::default())
        .unwrap();
    assert!(r.refused && r.passes > 1, "{r:?}");
    assert!(
        r.applied_gain_reduction_db > 0.0 && r.applied_gain_reduction_db <= 6.0,
        "{r:?}"
    );
    assert!(r.after.integrated_lufs > -18.0, "{r:?}");
}
#[test]
fn corrective_pass_uses_point_two_lu_acceptance() {
    let base = burst_mix();
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
        .normalise_to_target(-14.0, -1.0, Default::default())
        .unwrap();
    println!("point-two report {r:?}");
    assert!((r.after.integrated_lufs + 14.0).abs() <= 0.2, "{r:?}");
}
#[test]
fn limiter_docs_describe_slope_and_discrete_crest() {
    assert!(include_str!("../README.md").contains("unity-to-zero slope"));
    assert!(include_str!("oracle.rs").contains("discrete sample crest"));
}
