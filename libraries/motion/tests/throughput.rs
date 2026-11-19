use motion::{Curve, Fill, Keyframe, Preset, Property, Timing, Track, sample};
use std::{hint::black_box, time::Instant};

/// Explicitly invoked in an optimised test build; never part of normal tests.
#[test]
#[ignore = "micro-benchmark; run with --release -- --ignored --nocapture"]
fn scene_throughput() {
    // Ten independently timed objects with three properties each.
    let scene: Vec<_> = (0..10)
        .map(|i| Preset {
            name: format!("object-{i}"),
            timing: Timing {
                duration: 0.7,
                delay: i as f64 * 0.01,
                fill: Fill::Both,
                ..Timing::default()
            },
            tracks: [Property::Opacity, Property::Tx, Property::Scale]
                .into_iter()
                .map(|property| Track {
                    property,
                    keyframes: vec![
                        Keyframe {
                            offset: 0.0,
                            value: 0.0,
                            easing: Curve::EaseOut,
                        },
                        Keyframe {
                            offset: 1.0,
                            value: 1.0,
                            easing: Curve::Linear,
                        },
                    ],
                })
                .collect(),
        })
        .collect();
    let frames = 1_000_000;
    let start = Instant::now();
    for frame in 0..frames {
        let t = black_box((frame % 1000) as f64 / 1000.0);
        for object in &scene {
            black_box(sample(black_box(object), black_box(&object.timing), t));
        }
    }
    let elapsed = start.elapsed().as_secs_f64();
    println!(
        "MOTION_BENCH scenes_per_second={:.0} property_samples_per_second={:.0} ns_per_30_property_scene={:.0}",
        frames as f64 / elapsed,
        frames as f64 * 30.0 / elapsed,
        elapsed * 1e9 / frames as f64
    );
}
