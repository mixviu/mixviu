// SPDX-License-Identifier: MIT OR Apache-2.0
use kms::{
    connector::select::{ConnectorInfo, connector_name},
    edid::{identity, parse::ParsedEdid},
    output::physical,
};
use model::environment::preference::base::{LayoutPlacement, Preference};
use smithay::reexports::drm::control::connector::Interface;

struct Connector(Interface, u32);
impl ConnectorInfo for Connector {
    fn interface(&self) -> Interface {
        self.0
    }
    fn interface_id(&self) -> u32 {
        self.1
    }
    fn size(&self) -> Option<(u32, u32)> {
        Some((600, 340))
    }
}

#[test]
fn physical_output_and_connector_selection_share_names() {
    for interface in [
        Interface::DisplayPort,
        Interface::HDMIA,
        Interface::EmbeddedDisplayPort,
    ] {
        let info = Connector(interface, 1);
        let id = identity::for_connector(None, &info);
        assert_eq!(physical::create(&info, &id).name(), connector_name(&info));
    }
}

#[test]
fn enumeration_key_matches_output_even_with_debug_spelled_fallback_serial() {
    let info = Connector(Interface::DisplayPort, 1);
    let id = identity::for_connector(None, &info);
    assert_eq!(id.serial, "DisplayPort-1");
    assert_eq!(connector_name(&info), "DP-1");
    let enumerated = identity::connector_key(None, &info);
    assert_eq!(enumerated, "Native Monitor DisplayPort-1 @connector:DP-1");
    assert_eq!(
        enumerated,
        world::state::output_preference_key(&physical::create(&info, &id))
    );
}

#[test]
fn reconcile_connector_key_preserves_edid_placement() {
    let info = Connector(Interface::DisplayPort, 2);
    let parsed = ParsedEdid {
        manufacturer: "Example".into(),
        product_code: 1,
        serial: 42,
        display_name: Some("Panel".into()),
    };
    // connector_key is the function used by reconcile::identity_key and enumerate.
    let connected = identity::connector_key(Some(&parsed), &info);
    let output = physical::create(&info, &identity::for_connector(Some(&parsed), &info));
    assert_eq!(connected, world::state::output_preference_key(&output));
    let mut prefs = Preference::default();
    prefs.outputs_layout.push(LayoutPlacement {
        id: 7,
        identity: connected.clone(),
        x: 10.0,
        y: 20.0,
        w: 100.0,
        h: 100.0,
    });
    let layout = drivers::output::base::build_teleport(&prefs, &[connected]);
    assert_eq!(layout.placements.len(), 1);
    assert_eq!(layout.placements[0].key, "DP-2");
    assert_eq!(layout.placements[0].x, 10.0);
}
