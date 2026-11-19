use motion::{Property, presets};
use serde_json::Value;

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
    let expected_names: std::collections::BTreeSet<_> = table.iter().map(|p| p.name.as_str()).collect();
    let captured_names: std::collections::BTreeSet<_> = captured.iter().map(|p| p["name"].as_str().unwrap()).collect();
    assert_eq!(expected_names.len(), table.len(), "duplicate table names");
    assert_eq!(captured_names.len(), captured.len(), "duplicate fixture names");
    assert_eq!(captured_names, expected_names, "fixture must cover every preset exactly once");
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
