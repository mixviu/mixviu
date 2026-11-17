// SPDX-License-Identifier: MIT OR Apache-2.0
//! Standalone, single-threaded regression probe; run on CBC with cargo run.
use datetime::{DateTime, Local};
fn main() {
    let instant = DateTime::parse_from_rfc3339("2023-11-14T22:13:20Z").unwrap();
    for (tz, expected) in [("UTC0", "+0000"), ("AEST-10", "+1000"), ("UTC0", "+0000")] {
        // SAFETY: this executable never creates any other thread or invokes
        // third-party code that creates threads. Environment reads are serial.
        unsafe { std::env::set_var("TZ", tz); }
        assert_eq!(instant.with_timezone(&Local).format("%z").to_string(), expected);
    }
    println!("timezone_refresh_within_single_threaded_process: passed");
}
