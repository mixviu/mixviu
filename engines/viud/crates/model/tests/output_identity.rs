// SPDX-License-Identifier: MIT OR Apache-2.0
use model::environment::preference::{base::*, identity};

#[test]
fn legacy_profile_is_read_and_copied_without_losing_other_fields() {
    let mut profiles: Vec<OutputProfile> = serde_json::from_str(r#"[
        {"identity":"Example Panel 42","mode":{"Advertised":{"width":1920,"height":1080,"refresh_mhz":60000}},"active":false,"touch_device":"touch"}
    ]"#).unwrap();
    let a = identity::key("Example Panel 42", "DP-1");
    let b = identity::key("Example Panel 42", "DP-2");
    assert!(!output_active(&profiles, &a));
    set_active(&mut profiles, &a, true);
    assert!(output_active(&profiles, &a));
    assert!(!output_active(&profiles, &b));
    assert_eq!(touch_device_of(&profiles, &a), Some("touch"));
    assert!(identity::find(&profiles, &a, |p| p.identity.as_deref()).unwrap().mode.is_some());
    let saved = serde_json::to_string(&profiles).unwrap();
    let loaded: Vec<OutputProfile> = serde_json::from_str(&saved).unwrap();
    assert!(output_active(&loaded, &a));
    assert!(!output_active(&loaded, &b));
    assert_eq!(loaded[0].identity.as_deref(), Some("Example Panel 42"));
}
