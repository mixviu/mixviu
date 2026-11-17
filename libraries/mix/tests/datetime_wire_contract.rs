// SPDX-License-Identifier: MIT OR Apache-2.0
// Golden wire fixtures captured before replacing chrono. These spellings are
// shared by this crate’s timestamps; do not adopt a dependency’s default serde.
use chrono::{DateTime, SecondsFormat, Utc};

#[test]
fn datetime_wire_contract() {
    let t = DateTime::<Utc>::from_timestamp(1_700_000_000, 123_456_789).unwrap();
    assert_eq!(t.to_rfc3339(), "2023-11-14T22:13:20.123456789+00:00");
    assert_eq!(t.to_rfc3339_opts(SecondsFormat::Secs, true), "2023-11-14T22:13:20Z");
    assert_eq!(t.to_rfc3339_opts(SecondsFormat::Millis, true), "2023-11-14T22:13:20.123Z");
    assert_eq!(t.to_rfc3339_opts(SecondsFormat::Micros, true), "2023-11-14T22:13:20.123456Z");
    assert_eq!(t.to_rfc2822(), "Tue, 14 Nov 2023 22:13:20 +0000");
    assert_eq!(t.format("%Y%m%dT%H%M%SZ").to_string(), "20231114T221320Z");
    assert_eq!(t.format("%e-%b-%Y %H:%M:%S +0000").to_string(), "14-Nov-2023 22:13:20 +0000");
    assert_eq!(t.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string(), "2023-11-14T22:13:20.123Z");
    assert_eq!(DateTime::<Utc>::from_timestamp_millis(-1).unwrap().to_rfc3339_opts(SecondsFormat::Millis, true), "1969-12-31T23:59:59.999Z");
    assert_eq!(DateTime::parse_from_rfc3339("2023-11-15T08:13:20.123+10:00").unwrap().timestamp_millis(), 1_700_000_000_123);
    assert!(DateTime::<Utc>::from_timestamp_millis(i64::MAX).is_none());
}
