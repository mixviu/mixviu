// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>

//! Broker-owned observation classification. Never deserialised from a header.
use bus::wire::BusMessage;
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum TrafficClass {
    #[default]
    Legacy,
    NativeSession,
}

impl TrafficClass {
    pub(crate) fn merge(self, other: Self) -> Self {
        if self == Self::NativeSession || other == Self::NativeSession {
            Self::NativeSession
        } else {
            Self::Legacy
        }
    }
    pub(crate) fn protected(self) -> bool {
        self == Self::NativeSession
    }

    pub(crate) fn command(message: &BusMessage) -> Self {
        if message
            .headers
            .iter()
            .any(|(k, v)| k.eq_ignore_ascii_case("command") && v.starts_with("noded.session."))
        {
            return Self::NativeSession;
        }
        // Check the inner command before topic/property canonicalisation.
        if message.command_name() == Some("topic.publish")
            && message.body.starts_with("---\n")
            && bus::parse(&message.body).is_ok_and(|inner| {
                inner
                    .command_name()
                    .is_some_and(|c| c.starts_with("noded.session."))
            })
        {
            return Self::NativeSession;
        }
        Self::Legacy
    }
}

/// Correlation precision cache, not a confidentiality deadline. The responder's
/// connection-lifetime sticky bit is the unconditional backstop after expiry.
/// Overflow conservatively protects unknown responses during this cache horizon.
#[derive(Default)]
pub(crate) struct ResponseProtection {
    entries: std::collections::HashMap<Arc<str>, Instant>,
    /// `entries` in deadline order, one element for each. A full cache drops
    /// its expired entries from the front instead of scanning all of them on
    /// every completion, which cost a pass over 65,536 entries per protected
    /// reply once a broker had completed that many inside the horizon.
    expiry: std::collections::BTreeSet<(Instant, Arc<str>)>,
    overflow_until: Option<Instant>,
}
impl ResponseProtection {
    #[cfg(test)]
    pub(crate) fn expire_for_test(&mut self) {
        let past = Instant::now() - Self::HORIZON;
        let ids: Vec<Arc<str>> = self.entries.keys().cloned().collect();
        for id in ids {
            self.insert(&id, past);
        }
        self.overflow_until = None;
    }
    const HORIZON: std::time::Duration = std::time::Duration::from_secs(15 * 60);
    const LIMIT: usize = 65_536;
    pub(crate) fn retain(&mut self, id: &str, class: TrafficClass) {
        self.retain_at(id, class, Instant::now());
    }
    pub(crate) fn class(&self, id: &str) -> TrafficClass {
        self.class_at(id, Instant::now())
    }

    fn retain_at(&mut self, id: &str, class: TrafficClass, now: Instant) {
        if !class.protected() {
            return;
        }
        if self.entries.len() >= Self::LIMIT {
            self.drop_expired(now);
        }
        let deadline = now + Self::HORIZON;
        if self.entries.len() < Self::LIMIT || self.entries.contains_key(id) {
            self.insert(id, deadline);
        } else {
            self.overflow_until = Some(deadline);
        }
    }

    fn class_at(&self, id: &str, now: Instant) -> TrafficClass {
        if self.entries.get(id).is_some_and(|until| *until > now)
            || self.overflow_until.is_some_and(|until| until > now)
        {
            TrafficClass::NativeSession
        } else {
            TrafficClass::Legacy
        }
    }

    /// Add `id`, or move it to a new deadline. Keeps `expiry` exact.
    fn insert(&mut self, id: &str, deadline: Instant) {
        let id: Arc<str> = Arc::from(id);
        if let Some(previous) = self.entries.insert(id.clone(), deadline) {
            self.expiry.remove(&(previous, id.clone()));
        }
        self.expiry.insert((deadline, id));
    }

    /// Remove every entry whose deadline has passed: exactly the entries a
    /// scan for `deadline > now` would have discarded.
    fn drop_expired(&mut self, now: Instant) {
        while self
            .expiry
            .first()
            .is_some_and(|(deadline, _)| *deadline <= now)
        {
            let (_, id) = self.expiry.pop_first().expect("first exists");
            self.entries.remove(&id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tombstones_are_bounded_and_overflow_never_exposes_payloads() {
        let mut history = ResponseProtection::default();
        let future = std::time::Instant::now() + ResponseProtection::HORIZON;
        for id in 0..ResponseProtection::LIMIT {
            history.insert(&id.to_string(), future);
        }
        history.retain("overflow", TrafficClass::NativeSession);
        assert_eq!(history.entries.len(), ResponseProtection::LIMIT);
        assert!(history.class("overflow").protected());
        assert!(history.class("unknown").protected());
        history.overflow_until = Some(std::time::Instant::now());
        assert_eq!(history.class("unknown"), TrafficClass::Legacy);
        assert!(history.class("0").protected());
        history.insert("0", std::time::Instant::now());
        assert_eq!(history.class("0"), TrafficClass::Legacy);
    }

    /// The scan this cache used to make, kept as the oracle: at the limit it
    /// visited every entry on every protected completion.
    #[derive(Default)]
    struct Scanning {
        entries: std::collections::HashMap<String, Instant>,
        overflow_until: Option<Instant>,
    }
    impl Scanning {
        fn retain_at(&mut self, id: &str, class: TrafficClass, now: Instant) {
            if !class.protected() {
                return;
            }
            if self.entries.len() >= ResponseProtection::LIMIT {
                self.entries.retain(|_, deadline| *deadline > now);
            }
            let deadline = now + ResponseProtection::HORIZON;
            if self.entries.len() < ResponseProtection::LIMIT || self.entries.contains_key(id) {
                self.entries.insert(id.to_owned(), deadline);
            } else {
                self.overflow_until = Some(deadline);
            }
        }
        fn class_at(&self, id: &str, now: Instant) -> TrafficClass {
            if self.entries.get(id).is_some_and(|until| *until > now)
                || self.overflow_until.is_some_and(|until| until > now)
            {
                TrafficClass::NativeSession
            } else {
                TrafficClass::Legacy
            }
        }
    }

    /// Same classification, same contents and same overflow as the scan, for
    /// a run that fills the cache, overflows it, repeats ids, lets entries
    /// expire a few at a time at the limit, and then lets everything expire.
    #[test]
    fn ordered_expiry_matches_the_scan_it_replaces() {
        let mut indexed = ResponseProtection::default();
        let mut scanning = Scanning::default();
        let mut now = Instant::now();
        let mut seed = 0x9e37_79b9_7f4a_7c15_u64;
        let mut random = move |below: u64| {
            seed = seed
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            (seed >> 33) % below
        };
        let limit = ResponseProtection::LIMIT as u64;
        let check = |indexed: &ResponseProtection, scanning: &Scanning, now, probe: &str| {
            assert_eq!(indexed.class_at(probe, now), scanning.class_at(probe, now));
            assert_eq!(indexed.overflow_until, scanning.overflow_until);
            assert_eq!(indexed.entries.len(), scanning.entries.len());
            assert_eq!(indexed.expiry.len(), indexed.entries.len());
        };
        let started = now;
        // Phase 1: fill past the limit 10 ms apart (874 s in all, so nothing
        // expires yet), with some repeated ids and unprotected completions.
        // About 75,000 distinct protected ids: the last 9,000 overflow.
        for step in 0..limit + limit / 3 {
            now += std::time::Duration::from_millis(10);
            let id = if random(10) == 0 {
                format!("noded-{}", random(step + 1))
            } else {
                format!("noded-{step}")
            };
            let class = if random(20) == 0 {
                TrafficClass::Legacy
            } else {
                TrafficClass::NativeSession
            };
            indexed.retain_at(&id, class, now);
            scanning.retain_at(&id, class, now);
            if step % 97 == 0 {
                check(&indexed, &scanning, now, &format!("noded-{}", random(step + 1)));
            }
        }
        assert_eq!(indexed.entries.len(), ResponseProtection::LIMIT);
        assert!(indexed.overflow_until.is_some());
        // Phase 2: stay at the limit while the oldest entries expire one or
        // two at a time, slower than new completions arrive.
        now = started + ResponseProtection::HORIZON + std::time::Duration::from_millis(200);
        for step in 0..1_500 {
            now += std::time::Duration::from_millis(random(8));
            let id = if random(4) == 0 {
                format!("late-{}", random(step + 1))
            } else {
                format!("late-{step}")
            };
            indexed.retain_at(&id, TrafficClass::NativeSession, now);
            scanning.retain_at(&id, TrafficClass::NativeSession, now);
            check(&indexed, &scanning, now, &format!("noded-{}", random(limit)));
            check(&indexed, &scanning, now, &format!("late-{step}"));
        }
        // Phase 3: everything is past its deadline; one completion clears it.
        now += ResponseProtection::HORIZON * 2;
        check(&indexed, &scanning, now, "noded-1");
        indexed.retain_at("fresh", TrafficClass::NativeSession, now);
        scanning.retain_at("fresh", TrafficClass::NativeSession, now);
        check(&indexed, &scanning, now, "fresh");
        assert_eq!(indexed.entries.len(), 1);
        assert_eq!(indexed.class_at("noded-1", now), TrafficClass::Legacy);
        assert!(indexed.class_at("fresh", now).protected());
    }
    #[test]
    fn classification_is_broker_owned_and_monotonic() {
        let forged = BusMessage::new()
            .with_header("native-session", "1")
            .with_header("broker_principal", "forged");
        assert_eq!(TrafficClass::command(&forged), TrafficClass::Legacy);
        let command = BusMessage::new().with_header("Command", "noded.session.prove");
        assert_eq!(TrafficClass::command(&command), TrafficClass::NativeSession);
        let outer = BusMessage::new()
            .with_header("command", "topic.publish")
            .with_body(
                &BusMessage::new()
                    .with_header("command", "noded.session.prove")
                    .to_wire(),
            );
        assert!(TrafficClass::command(&outer).protected());
        assert!(
            TrafficClass::NativeSession
                .merge(TrafficClass::Legacy)
                .protected()
        );
        assert!(
            TrafficClass::Legacy
                .merge(TrafficClass::NativeSession)
                .protected()
        );
    }
}
