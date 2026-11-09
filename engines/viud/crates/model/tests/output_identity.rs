// SPDX-License-Identifier: MIT OR Apache-2.0
use model::environment::preference::{base::*, identity};

#[test]
fn headless_legacy_maps_to_zero_only_and_exact_wins() {
    let old = OutputProfile { identity: Some("MixOS Headless Unknown".into()), active: false, ..OutputProfile::default() };
    let zero = identity::key("MixOS Headless headless-0", "headless-0");
    let one = identity::key("MixOS Headless headless-1", "headless-1");
    let mut profiles = vec![old];
    assert!(!output_active(&profiles, &zero));
    assert!(output_active(&profiles, &one));
    set_active(&mut profiles, &zero, true, &[]);
    assert!(output_active(&profiles, &zero));
    assert!(!profiles[0].active);
}

#[test]
fn qualified_selection_beats_earlier_legacy_but_keeps_priority_within_pass() {
    let a = identity::key("Example Panel 42", "DP-1");
    let b = identity::key("Example Panel 42", "DP-2");
    let keys = vec![a.clone(), b.clone()];
    assert_eq!(identity::preferred_index(&keys, &["Example Panel 42", &b]), Some(1));
    assert_eq!(identity::preferred_index(&keys, &[&b, &a]), Some(1));
    assert_eq!(identity::preferred_index(&keys, &["Example Panel 42"]), Some(0));
}

#[test]
fn legacy_touch_uses_association_only_within_matching_outputs() {
    let a = identity::key("Example Panel 42", "DP-1");
    let b = identity::key("Example Panel 42", "DP-2");
    let keys = vec![a.clone(), b];
    assert_eq!(identity::touch_index(&keys, "Example Panel 42", Some("DP-2")), Some(1));
    assert_eq!(identity::touch_index(&keys, "Example Panel 42", None), Some(0));
    assert_eq!(identity::touch_index(&keys, "Example Panel 42", Some("missing")), Some(0));
    assert_eq!(identity::touch_index(&keys, &a, Some("DP-2")), Some(0));
}

#[test]
fn single_panel_writes_refresh_legacy_for_moves_but_duplicates_do_not() {
    let a = identity::key("Example Panel 42", "DP-1");
    let moved = identity::key("Example Panel 42", "DP-2");
    let mut profiles = vec![OutputProfile { identity: Some("Example Panel 42".into()), ..OutputProfile::default() }];
    set_active(&mut profiles, &a, false, &[a.clone()]);
    assert!(!output_active(&profiles, &moved));
    upsert_output(&mut profiles, &a, ModeRequest::Advertised { width: 1920, height: 1080, refresh_mhz: 60000 }, &[a.clone()]);
    set_touch_device(&mut profiles, &a, Some("touch".into()), &[a.clone()]);
    set_default(&mut profiles, &a, &[a.clone()]);
    assert_eq!(touch_device_of(&profiles, &moved), Some("touch"));
    assert!(identity::find(&profiles, &moved, |p| p.identity.as_deref()).unwrap().mode.is_some());
    set_active(&mut profiles, &a, true, &[a.clone(), moved.clone()]);
    assert!(output_active(&profiles, &a));
    assert!(!output_active(&profiles, &moved));
    set_active(&mut profiles, &a, true, &[]); // unknown topology cannot change legacy
    assert!(!output_active(&profiles, &moved));
}

#[test]
fn stable_splits_only_the_last_connector_suffix() {
    let base = "Example @connector:in-model Panel 42";
    let key = identity::key(base, "DP-1");
    assert_eq!(identity::stable(&key), base);
    assert_eq!(identity::connector(&key), Some("DP-1"));
}

#[test]
fn legacy_profile_is_read_and_copied_without_losing_other_fields() {
    let mut profiles: Vec<OutputProfile> = serde_json::from_str(r#"[
        {"identity":"Example Panel 42","mode":{"Advertised":{"width":1920,"height":1080,"refresh_mhz":60000}},"active":false,"touch_device":"touch"}
    ]"#).unwrap();
    let a = identity::key("Example Panel 42", "DP-1");
    let b = identity::key("Example Panel 42", "DP-2");
    assert!(!output_active(&profiles, &a));
    set_active(&mut profiles, &a, true, &[a.clone(), b.clone()]);
    assert!(output_active(&profiles, &a));
    assert!(!output_active(&profiles, &b));
    assert_eq!(touch_device_of(&profiles, &a), Some("touch"));
    assert!(
        identity::find(&profiles, &a, |p| p.identity.as_deref())
            .unwrap()
            .mode
            .is_some()
    );
    let saved = serde_json::to_string(&profiles).unwrap();
    let loaded: Vec<OutputProfile> = serde_json::from_str(&saved).unwrap();
    assert!(output_active(&loaded, &a));
    assert!(!output_active(&loaded, &b));
    assert_eq!(loaded[0].identity.as_deref(), Some("Example Panel 42"));
}
