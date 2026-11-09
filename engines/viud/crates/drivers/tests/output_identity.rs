// SPDX-License-Identifier: MIT OR Apache-2.0
use drivers::output::base::build_teleport;
use model::environment::preference::{base::*, identity};

#[test]
fn qualified_layout_routes_to_connectors_and_survives_hotplug() {
    let a = identity::key("Example Panel 42", "DP-1");
    let b = identity::key("Example Panel 42", "DP-2");
    let mut prefs = Preference::default();
    prefs.outputs_layout = vec![
        LayoutPlacement {
            id: 1,
            identity: a.clone(),
            x: 0.0,
            y: 0.0,
            w: 100.0,
            h: 100.0,
        },
        LayoutPlacement {
            id: 2,
            identity: b.clone(),
            x: 100.0,
            y: 0.0,
            w: 100.0,
            h: 100.0,
        },
    ];
    let both = build_teleport(&prefs, &[a.clone(), b.clone()]);
    assert_eq!(both.placements.len(), 2);
    assert_eq!(both.placements[0].key, "DP-1");
    assert_eq!(both.placements[1].key, "DP-2");
    let survivor = build_teleport(&prefs, &[b.clone()]);
    assert_eq!(survivor.placements.len(), 1);
    assert_eq!(survivor.placements[0], both.placements[1]);
    let restored = build_teleport(&prefs, &[b, a]);
    assert_eq!(restored.placements, both.placements);
}

#[test]
fn legacy_layout_is_compatible_and_explicit_placement_wins() {
    let key = identity::key("Example Panel 42", "DP-1");
    let mut prefs = Preference::default();
    let old = LayoutPlacement {
        id: 7,
        identity: "Example Panel 42".into(),
        x: 1.0,
        y: 2.0,
        w: 100.0,
        h: 100.0,
    };
    prefs.outputs_layout.push(old.clone());
    let migrated = build_teleport(&prefs, &[key.clone()]);
    assert_eq!(migrated.placements[0].key, "DP-1");
    assert_eq!(migrated.placements[0].x, 1.0);
    prefs.outputs_layout.push(LayoutPlacement {
        id: 8,
        identity: key.clone(),
        x: 50.0,
        ..old
    });
    let explicit = build_teleport(&prefs, &[key]);
    assert_eq!(explicit.placements.len(), 1);
    assert_eq!(explicit.placements[0].x, 50.0);
}

#[test]
fn legacy_placement_uses_smallest_active_matching_connector() {
    let a = identity::key("Example Panel 42", "DP-1");
    let b = identity::key("Example Panel 42", "DP-2");
    let mut prefs = Preference::default();
    prefs.outputs_layout.push(LayoutPlacement {
        id: 7,
        identity: "Example Panel 42".into(),
        x: 1.0,
        y: 2.0,
        w: 100.0,
        h: 100.0,
    });
    model::environment::preference::base::set_active(
        &mut prefs.outputs,
        &a,
        false,
        ConnectedOutputs::from_complete_snapshot(&[a.clone(), b.clone()]),
    );
    let layout = build_teleport(&prefs, &[a, b]);
    assert_eq!(layout.placements.len(), 1);
    assert_eq!(layout.placements[0].key, "DP-2");
    assert_eq!(layout.placements[0].id, 7);
}
