// SPDX-License-Identifier: MIT OR Apache-2.0
use smithay::output::{Output, PhysicalProperties, Subpixel};
use world::state::{output_key, output_preference_key};

fn panel(connector: &str, serial: &str) -> Output {
    Output::new(connector.into(), PhysicalProperties {
        size: (600, 340).into(), subpixel: Subpixel::Unknown,
        make: "Example".into(), model: "Panel".into(), serial_number: serial.into(),
    })
}

#[test]
fn duplicated_nonzero_edid_serial_never_aliases_runtime_or_preferences() {
    let a = panel("DP-1", "42");
    let b = panel("DP-2", "42");
    assert_eq!(output_key(&a), "DP-1");
    assert_ne!(output_key(&a), output_key(&b));
    assert_ne!(output_preference_key(&a), output_preference_key(&b));
    assert_eq!(output_preference_key(&a), "Example Panel 42 @connector:DP-1");

    // A hotplug must neither rename the survivor nor steal its independent view.
    let mut views = world::viewport::state::OutputViews::default();
    views.ensure(&output_key(&a));
    views.ensure(&output_key(&b));
    views.views_mut(&output_key(&b)).next_id = 42;
    let before = output_preference_key(&b);
    let mut space = smithay::desktop::Space::<smithay::desktop::Window>::default();
    space.map_output(&a, (0, 0));
    space.map_output(&b, (1920, 0));
    space.unmap_output(&a);
    assert_eq!(space.outputs().map(output_key).collect::<Vec<_>>(), ["DP-2"]);
    drop(a);
    assert_eq!(output_preference_key(&b), before);
    let reconnected = panel("DP-1", "42");
    space.map_output(&reconnected, (0, 0));
    assert_eq!(space.outputs().count(), 2);
    views.ensure(&output_key(&reconnected));
    assert_eq!(views.views(&output_key(&b)).next_id, 42);
    assert_eq!(output_preference_key(&reconnected), "Example Panel 42 @connector:DP-1");
}

#[test]
fn serialless_and_virtual_outputs_are_distinct() {
    for serial in ["0", "Unknown", "headless-0"] {
        let a = panel("headless-0", serial);
        let b = panel("headless-1", serial);
        assert_ne!(output_key(&a), output_key(&b));
        assert_ne!(output_preference_key(&a), output_preference_key(&b));
    }
}
