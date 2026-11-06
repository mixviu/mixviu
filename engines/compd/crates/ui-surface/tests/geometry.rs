// SPDX-License-Identifier: MIT OR Apache-2.0
use ui_surface::{HandleId, Transform, UiItemView, UiSpace};
use smithay::utils::{Physical, Point, Size};

fn item(space: UiSpace) -> UiItemView {
    UiItemView { id: HandleId(7), space, location: (140, -30).into(),
        size: (160, 80).into(), scale_factor: 2.0, zoom_lock: None, pending_resize: None,
        output: Some("example-output".into()), layer: 4, draw_order: 2,
        visible: true, passthrough: false, keyboard_transparent: false }
}

#[test]
fn world_hit_coordinates_follow_the_pane_camera_and_instance_scale() {
    let item = item(UiSpace::World);
    let camera = Transform { position: (100.0, -50.0).into(), zoom: 0.5 };
    let output: Size<f64, Physical> = (800.0, 600.0).into();
    let bounds = item.screen_rect(&camera, output);
    assert_eq!(bounds.loc, Point::from((420, 310)));
    assert_eq!(bounds.size, Size::from((80, 40)));
    assert_eq!(item.local_coords((440.0, 320.0).into(), &camera, output), Some((20.0, 10.0)));
    assert_eq!(item.local_coords((500.0, 320.0).into(), &camera, output), None);
    assert_eq!(item.local_coords_unclamped((510.0, 320.0).into(), &camera, output), (90.0, 10.0));
}

#[test]
fn supersampling_pins_size_but_keeps_the_world_anchor_and_screen_items_ignore_zoom() {
    let mut world = item(UiSpace::World);
    world.zoom_lock = Some(2.0);
    let output = (800.0, 600.0).into();
    let before = Transform { position: (100.0, -50.0).into(), zoom: 0.5 };
    let after = Transform { zoom: 3.0, ..before };
    assert_eq!(world.screen_size(&before), world.screen_size(&after));
    assert_ne!(world.screen_location(&before, output), world.screen_location(&after, output));
    let screen = item(UiSpace::Screen);
    assert_eq!(screen.screen_rect(&before, output), screen.screen_rect(&after, output));
    assert_eq!(screen.output(), Some("example-output"));
}

#[test]
fn the_surface_trait_is_object_safe() {
    fn accepts_object(_: Option<Box<dyn ui_surface::UiSurfaces>>) {}
    accepts_object(None);
}
