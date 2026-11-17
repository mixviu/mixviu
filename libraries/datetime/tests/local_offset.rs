// SPDX-License-Identifier: MIT OR Apache-2.0
// Process isolation preserves Rust's environment safety and avoids TZ caches.
use datetime::{DateTime, Local};

#[test]
fn local_offset_child() {
    let Ok(expected) = std::env::var("DATETIME_OFFSET_PROBE") else { return };
    let t = DateTime::parse_from_rfc3339("2023-11-14T22:13:20Z").unwrap().with_timezone(&Local);
    assert_eq!(t.format("%Y-%m-%d %H:%M:%S %z %:z %Z").to_string(), expected);
}

#[test]
fn local_offsets_are_process_scoped() {
    for (zone, expected) in [
        ("UTC0", "2023-11-14 22:13:20 +0000 +00:00 +00:00"),
        ("AEST-10", "2023-11-15 08:13:20 +1000 +10:00 +10:00"),
        ("EST5EDT,M3.2.0/2,M11.1.0/2", "2023-11-14 17:13:20 -0500 -05:00 -05:00"),
    ] {
        assert!(std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "local_offset_child"])
            .env("TZ", zone).env("DATETIME_OFFSET_PROBE", expected)
            .status().unwrap().success(), "zone {zone}");
    }
}
