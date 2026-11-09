// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
#![cfg(feature = "sqlite")]

#[test]
fn bundled_sqlite_has_wal_page_cache_fix() {
    println!("bundled SQLite: {}", rusqlite::version());
    // RUSTSEC-2026-0459 / CVE-2026-119375: do not regress to an affected bundle.
    assert!(rusqlite::version_number() >= 3_054_000);
    let connection = rusqlite::Connection::open_in_memory().unwrap();
    let version: String = connection
        .query_row("SELECT sqlite_version()", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, rusqlite::version());
}
