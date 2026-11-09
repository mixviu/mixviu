// SPDX-License-Identifier: MIT OR Apache-2.0
//! P3 lab evidence only: documents a production identity collision, not a fix.
use kms::edid::{identity::identity, parse::ParsedEdid};

#[test]
fn missing_serials_use_distinct_connector_fallbacks() {
    assert_ne!(identity(None,"DP-1").key(),identity(None,"DP-2").key());
    let edid = ParsedEdid { manufacturer:"Example".into(),product_code:1,serial:0,display_name:Some("Panel".into()) };
    assert_ne!(identity(Some(&edid),"DP-1").key(),identity(Some(&edid),"DP-2").key());
}

#[test]
fn duplicate_nonzero_serial_reproduces_main_identity_alias() {
    let edid = ParsedEdid { manufacturer:"Example".into(),product_code:1,serial:42,display_name:Some("Panel".into()) };
    let a = identity(Some(&edid),"DP-1").key();
    let b = identity(Some(&edid),"DP-2").key();
    // Existing production behaviour: distinct connectors collapse. Runtime
    // output identity needs separation from the stable preference identity.
    assert_eq!(a,b);
    println!("P3_MAIN_IDENTITY_ALIAS connectors=DP-1,DP-2 key={a:?}");
}
