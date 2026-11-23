use motion::{Property, presets};
use serde_json::Value;

#[test]
fn paint_effect_resolves_the_entire_current_and_internal_legacy_tables() {
    let old=motion::load_presets(include_str!("../legacy-presets.json")).unwrap();
    for p in motion::presets() {
        let resolved=motion::scene::paint_preset(&p.name).unwrap();
        let legacy=motion::scene::paint_preset(&format!("legacy-{}",p.name)).unwrap();
        let frozen=old.iter().find(|o|o.name==p.name).unwrap();
        for t in [0.,0.1,0.4,0.7,1.,3.] {
            assert_eq!(resolved.sample(t),p.sample(t),"{}",p.name);
            assert_eq!(legacy.sample(t),frozen.sample(t),"legacy-{}",p.name);
        }
    }
    assert!(motion::scene::paint_preset("legacy-unknown").is_err());
}

#[test]
fn title_samples_match_scene_entrances_and_legacy_sting_is_separate() {
    let fixture: Value = serde_json::from_str(include_str!("../oracle/chrome.json")).unwrap();
    let entrance = motion::scene::Entrance::Name("left".into()).playback(0., 0, false);
    for sample in fixture["title_samples"].as_array().unwrap() {
        let values = entrance.sample(sample["t"].as_f64().unwrap());
        assert!((values.get(Property::Opacity) - sample["opacity"].as_f64().unwrap()).abs() < 1e-6);
        assert!((values.get(Property::Tx) - sample["tx"].as_f64().unwrap()).abs() < 0.001);
    }
    let legacy = motion::load_presets(include_str!("../legacy-presets.json")).unwrap();
    for name in ["fade", "zoom", "wipe"] {
        let frozen = motion::scene::paint_preset(&format!("legacy-{name}")).unwrap();
        let old = legacy.iter().find(|p| p.name == name).unwrap();
        for i in 0..40 {
            assert_eq!(frozen.sample(i as f64 / 30.), old.sample(i as f64 / 30.));
        }
        assert_ne!(
            frozen.timing.duration,
            motion::scene::paint_preset(name).unwrap().timing.duration
        );
    }
}

/// Requires the real generated fixture; absence is a failure, never a skip.
#[test]
fn chrome_oracle() {
    let fixture: Value = serde_json::from_str(
        &std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/oracle/chrome.json"))
            .expect("committed Chrome oracle fixture must exist"),
    )
    .unwrap();
    let table = presets();
    let captured = fixture["presets"].as_array().unwrap();
    assert_eq!(table.len(), 19);
    assert_eq!(captured.len(), 19);
    let expected_names: std::collections::BTreeSet<_> =
        table.iter().map(|p| p.name.as_str()).collect();
    let captured_names: std::collections::BTreeSet<_> = captured
        .iter()
        .map(|p| p["name"].as_str().unwrap())
        .collect();
    assert_eq!(expected_names.len(), table.len(), "duplicate table names");
    assert_eq!(
        captured_names.len(),
        captured.len(),
        "duplicate fixture names"
    );
    assert_eq!(
        captured_names, expected_names,
        "fixture must cover every preset exactly once"
    );
    let mut failures = Vec::new();
    for entry in captured {
        let name = entry["name"].as_str().unwrap();
        let preset = table.iter().find(|p| p.name == name).unwrap();
        let samples = entry["samples"].as_array().unwrap();
        assert_eq!(samples.len(), 21);
        let mut maxima = std::collections::BTreeMap::<String, f64>::new();
        for point in samples {
            let result = preset.sample(point["t"].as_f64().unwrap());
            for (key, expected) in point["values"].as_object().unwrap() {
                let property: Property =
                    serde_json::from_value(Value::String(key.clone())).unwrap();
                let error = (result.get(property) - expected.as_f64().unwrap()).abs();
                let maximum = maxima.entry(key.clone()).or_default();
                *maximum = maximum.max(error);
                let tolerance = match property {
                    Property::Opacity | Property::Scale | Property::ScaleX | Property::ScaleY => {
                        1e-4
                    }
                    Property::Tx
                    | Property::Ty
                    | Property::Rotate
                    | Property::Blur
                    | Property::DashOffset => 0.05,
                    Property::Scalar if name == "flip" => 0.05,
                    _ => 1e-4,
                };
                if !error.is_finite() || error > tolerance {
                    failures.push(format!("{name} {key} t={} error={error}", point["t"]));
                }
            }
        }
        println!("MOTION_ORACLE {name} max_error={maxima:?}");
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
