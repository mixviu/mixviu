// SPDX-License-Identifier: MIT OR Apache-2.0
//! Preference identities are separate from live connector routing. Always retain
//! the connector qualifier, even while a duplicate panel is unplugged. Hardware
//! with duplicated EDIDs cannot be distinguished independently of its connector.

const SEPARATOR: &str = " @connector:";

pub fn key(stable: &str, connector: &str) -> String {
    format!("{stable}{SEPARATOR}{connector}")
}

pub fn stable(key: &str) -> &str {
    key.rsplit_once(SEPARATOR).map_or(key, |(base, _)| base)
}

pub fn connector(key: &str) -> Option<&str> {
    key.rsplit_once(SEPARATOR).map(|(_, connector)| connector)
}

/// Exact qualified settings win over legacy shared settings, regardless of order.
pub fn find<'a, T>(
    items: &'a [T],
    requested: &str,
    id: impl Fn(&T) -> Option<&str>,
) -> Option<&'a T> {
    items
        .iter()
        .find(|p| id(p) == Some(requested))
        .or_else(|| items.iter().find(|p| id(p) == Some(stable(requested))))
}

/// Used only at preference boundaries, never for live output equality.
pub fn matches(saved: &str, requested: &str) -> bool {
    saved == requested || saved == stable(requested)
}
