use motion::{Property, presets};
use serde_json::Value;

/// Requires the real generated fixture; absence is a failure, never a skip.
#[test]
#[ignore = "blocked until an authorised Chrome bench captures oracle/chrome.json"]
fn chrome_oracle() {
    let fixture: Value = serde_json::from_str(
        &std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/oracle/chrome.json"))
            .expect("capture Chrome fixture through native bench queue first"),
    )
    .unwrap();
    let table = presets();
    let captured = fixture["presets"].as_array().unwrap();
    assert_eq!(captured.len(), 19);
    for entry in captured {
        let name = entry["name"].as_str().unwrap();
        let preset = table.iter().find(|p| p.name == name).unwrap();
        let samples = entry["samples"].as_array().unwrap();
        assert_eq!(samples.len(), 21);
        let mut maxima = [0.0_f64; motion::PROPERTY_COUNT];
        for point in samples {
            let result = preset.sample(point["t"].as_f64().unwrap());
            for (key, expected) in point["values"].as_object().unwrap() {
                let property: Property =
                    serde_json::from_value(Value::String(key.clone())).unwrap();
                let error = (result.get(property) - expected.as_f64().unwrap()).abs();
                maxima[property as usize] = maxima[property as usize].max(error);
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
                assert!(
                    error <= tolerance,
                    "{name} {key} t={} error={error}",
                    point["t"]
                );
            }
        }
        println!("MOTION_ORACLE {name} max_error={maxima:?}");
    }
}
