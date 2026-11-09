// SPDX-License-Identifier: MIT OR Apache-2.0
//! Preference identities are separate from live connector routing. Always retain
//! the connector qualifier, even while a duplicate panel is unplugged. Hardware
//! with duplicated EDIDs cannot be distinguished independently of its connector.

const SEPARATOR: &str = " @connector:";

pub fn key(stable: &str, connector: &str) -> String {
    format!("{stable}{SEPARATOR}{connector}")
}

pub fn stable(key: &str) -> &str {
    // Split at the last qualifier: an EDID make/model can itself contain the
    // separator text. Only the final suffix was added by key().
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
        .or_else(|| items.iter().find(|p| id(p).is_some_and(|s| headless_legacy(s, requested))))
}

/// Used only at preference boundaries, never for live output equality.
pub fn matches(saved: &str, requested: &str) -> bool {
    saved == requested || saved == stable(requested) || headless_legacy(saved, requested)
}

fn headless_legacy(saved: &str, requested: &str) -> bool {
    saved == "MixOS Headless Unknown"
        && stable(requested) == "MixOS Headless headless-0"
        && connector(requested) == Some("headless-0")
}

/// Preserve profile priority within each pass, but never let an earlier legacy
/// match shadow an explicit connected output.
pub fn preferred_index(keys: &[String], saved: &[&str]) -> Option<usize> {
    saved.iter().find_map(|s| keys.iter().position(|k| k == s))
        .or_else(|| saved.iter().find_map(|s| keys.iter().position(|k| matches(s, k))))
}

/// Resolve a saved touch claim; use udev association to disambiguate a legacy
/// shared EDID. Without an association retain the original output order.
pub fn touch_index(keys: &[String], saved: &str, associated: Option<&str>) -> Option<usize> {
    keys.iter().position(|k| k == saved)
        .or_else(|| associated.and_then(|name| keys.iter().position(|k| matches(saved, k) && connector(k) == Some(name))))
        .or_else(|| keys.iter().position(|k| matches(saved, k)))
}
