// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Gold's bounded retained-operation store. The control lane serialises
//! admission/check/commit; a retained result is checked before pane liveness.
use super::protocol::{Reply, Target, mismatch, retired};
use bus::native_session::DecimalU64;
use serde_json::{Value, json};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

const RETENTION: Duration = Duration::from_secs(900);
const PER_ACTOR: usize = 1024;
const TOTAL: usize = 4096;
const ACTORS: usize = 256;
struct Entry {
    target: Target,
    outcome: Option<Reply>,
    sequence: u64,
    operation: u64,
    child_operation: Option<DecimalU64>,
    digest: [u8; 32],
    at: Instant,
    reply: Reply,
}

#[cfg(test)]
mod tests {
    use super::*;
    use bus::native_session::HexBytes;

    fn target() -> Target {
        Target {
            instance_id: HexBytes([1; 16]),
            incarnation: HexBytes([2; 16]),
            pane_id: DecimalU64(u64::MAX),
            pane_generation: DecimalU64(1),
        }
    }
    fn value(reply: &Reply) -> Value {
        serde_json::from_str(&reply.body).unwrap()
    }
    fn record(store: &mut Replay, actor: &str, sequence: u64, digest: [u8; 32]) -> Reply {
        store
            .reserve_request_id(actor, sequence, digest)
            .ok()
            .unwrap();
        let mut reply = Reply::ok(json!({"accepted":sequence}));
        store.record_outcome(actor, &target(), sequence, digest, &mut reply);
        reply
    }

    #[test]
    fn exact_retry_conflict_and_lost_outcome_never_reexecute() {
        let mut store = Replay::default();
        let original = record(&mut store, "caller-a", u64::MAX - 1, [1; 32]);
        assert_eq!(
            store
                .lookup("caller-a", u64::MAX - 1, [1; 32])
                .unwrap()
                .body,
            original.body
        );
        let conflict = store.lookup("caller-a", u64::MAX - 1, [2; 32]).unwrap();
        assert_eq!(
            value(&conflict),
            json!({"error_code":"CONFLICT","details":{"reason":"request_mismatch","retry_requires":"byte_identical_body"}})
        );
        assert!(store.lookup("caller-b", u64::MAX - 1, [1; 32]).is_none());
        store
            .reserve_request_id("caller-a", u64::MAX, [3; 32])
            .ok()
            .unwrap();
        // No record_outcome: the mutation committed but its answer was lost.
        let lost = store.lookup("caller-a", u64::MAX, [3; 32]).unwrap();
        assert_eq!(
            value(&lost),
            json!({"error_code":"UNKNOWN_OUTCOME","details":{"reason":"retired_request_id","request_high_water":u64::MAX.to_string()}})
        );
        assert!(
            store
                .reserve_request_id("caller-a", u64::MAX, [3; 32])
                .is_err()
        );
    }

    #[test]
    fn expiry_preserves_high_water_and_operation_survives_target_removal() {
        let mut store = Replay::default();
        let original = record(&mut store, "caller", 1, [1; 32]);
        assert_eq!(store.operation(&target(), 1).body, original.body);
        let mut other = target();
        other.pane_generation.0 += 1;
        assert_eq!(
            value(&store.operation(&other, 1))["error_code"],
            "UNKNOWN_OUTCOME"
        );
        store.state.history.get_mut("caller").unwrap().entries[0].at = Instant::now() - RETENTION;
        assert_eq!(
            value(&store.operation(&target(), 1))["error_code"],
            "UNKNOWN_OUTCOME"
        );
        assert_eq!(
            value(&store.lookup("caller", 1, [1; 32]).unwrap())["error_code"],
            "UNKNOWN_OUTCOME"
        );
        assert_eq!(store.high_water("caller"), 1);
        assert!(store.state.history["caller"].entries.is_empty());
        assert!(store.reserve_request_id("caller", 1, [1; 32]).is_err());
        assert!(store.reserve_request_id("caller", 2, [2; 32]).is_ok());
    }

    #[test]
    fn retained_child_operation_is_remapped_and_notice_outcome_is_separate() {
        let mut store = Replay::default();
        store.reserve_request_id("a", 1, [1; 32]).ok().unwrap();
        let mut reply = Reply::ok(json!({"operation_id":u64::MAX.to_string(),"status":"queued"}));
        store.record_outcome("a", &target(), 1, [1; 32], &mut reply);
        assert_eq!(value(&reply)["operation_id"], "1");
        let entry = &mut store.state.history.get_mut("a").unwrap().entries[0];
        assert_eq!(entry.child_operation, Some(DecimalU64(u64::MAX)));
        entry.outcome = Some(Reply::ok(json!({"status":"revoked","written":"0"})));
        assert_eq!(value(&store.operation(&target(), 1))["status"], "revoked");
        assert_eq!(
            value(&store.lookup("a", 1, [1; 32]).unwrap())["status"],
            "queued"
        );
    }

    #[test]
    fn actor_and_instance_limits_keep_retired_floors() {
        let mut store = Replay::default();
        for sequence in 1..=PER_ACTOR as u64 + 1 {
            record(&mut store, "a", sequence, [1; 32]);
        }
        assert_eq!(store.state.history["a"].entries.len(), PER_ACTOR);
        assert_eq!(
            value(&store.lookup("a", 1, [1; 32]).unwrap())["error_code"],
            "UNKNOWN_OUTCOME"
        );
        assert_eq!(store.high_water("a"), PER_ACTOR as u64 + 1);
        let mut full = Replay::default();
        for actor in 0..4 {
            for sequence in 1..=PER_ACTOR as u64 {
                record(&mut full, &format!("actor-{actor}"), sequence, [1; 32]);
            }
        }
        let refusal = full.reserve_request_id("new", 1, [2; 32]).err().unwrap();
        assert_eq!(value(&refusal)["error_code"], "RESOURCE_LIMIT");
        assert_eq!(full.high_water("new"), 0);
        assert_eq!(
            full.state
                .history
                .values()
                .map(|h| h.entries.len())
                .sum::<usize>(),
            TOTAL
        );
    }

    #[test]
    fn cold_actor_eviction_allows_new_connections_after_churn() {
        let mut store = Replay::default();
        let now = Instant::now();
        for actor in 0..ACTORS {
            let id = format!("actor-{actor}");
            record(&mut store, &id, 1, [1; 32]);
            let history = store.state.history.get_mut(&id).unwrap();
            history.entries[0].at = now - RETENTION;
            history.last = now - Duration::from_secs(actor as u64 + 1);
        }
        assert!(store.reserve_request_id("new", 1, [2; 32]).is_ok());
        assert_eq!(store.state.history.len(), ACTORS);
        assert!(
            !store
                .state
                .history
                .contains_key(&format!("actor-{}", ACTORS - 1))
        );
        assert_eq!(store.high_water("actor-0"), 1);
        assert!(store.reserve_request_id("actor-0", 1, [1; 32]).is_err());
    }

    #[test]
    fn forwarding_ids_separate_callers_and_capacity_evicts_one_mapping() {
        let mut store = Replay::default();
        assert_eq!(store.forwarded_id("a", 1, [1; 32]), Forward::Fresh(1));
        assert_eq!(store.forwarded_id("b", 1, [1; 32]), Forward::Fresh(2));
        assert_eq!(store.forwarded_id("a", 1, [1; 32]), Forward::Retry(1));
        assert_eq!(store.forwarded_id("a", 1, [2; 32]), Forward::Conflict);
        for sequence in 2..TOTAL as u64 {
            store.forwarded_id("a", sequence, [1; 32]);
        }
        assert_eq!(store.state.forwarded.len(), TOTAL);
        let before = store.state.forwarded.clone();
        assert_eq!(
            store.forwarded_id("new", 1, [1; 32]),
            Forward::Fresh(TOTAL as u64 + 1)
        );
        assert_eq!(store.state.forwarded.len(), TOTAL);
        assert_eq!(
            before
                .iter()
                .filter(|(key, value)| store.state.forwarded.get(*key) == Some(*value))
                .count(),
            TOTAL - 1
        );
    }
}
struct History {
    high_water: u64,
    entries: VecDeque<Entry>,
    /// Last accepted mutation. Only used to pick the coldest key to evict at
    /// the actor cap; it never affects whether a retry is answered.
    last: Instant,
}
impl Default for History {
    fn default() -> Self {
        Self {
            high_water: 0,
            entries: VecDeque::new(),
            last: Instant::now(),
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Forward {
    Fresh(u64),
    /// Already forwarded under this id. Re-forwarding reaches the child's own
    /// dedupe, which is the only place that can say what actually happened.
    Retry(u64),
    Conflict,
}

#[derive(Default)]
struct State {
    next_operation: u64,
    history: HashMap<String, History>,
    forwarded: HashMap<String, (u64, [u8; 32])>,
    next_forward: u64,
}

#[derive(Default)]
pub(crate) struct Replay {
    state: State,
}

impl Replay {
    /// Direct pane mutations allocate an instance operation even on refusal;
    /// successful replies carry it. Forwarded child replies use record_outcome.
    pub fn record_local_outcome(&mut self, identity: &str, target: &Target, sequence: u64, digest: [u8; 32], reply: &mut Reply) {
        self.state.next_operation = self.state.next_operation.checked_add(1).expect("operation id exhausted");
        let operation = self.state.next_operation;
        if reply.rc == 0 {
            let mut body: Value = serde_json::from_str(&reply.body).expect("owned reply JSON");
            body["operation_id"] = json!(DecimalU64(operation));
            reply.body = body.to_string();
        }
        let history = self.state.history.get_mut(identity).unwrap();
        history.last = Instant::now();
        history.entries.push_back(Entry { target: target.clone(), outcome: None, sequence, operation, child_operation: None, digest, at: Instant::now(), reply: reply.clone() });
        if history.entries.len() > PER_ACTOR {history.entries.pop_front();}
    }

    pub fn settle_input(&mut self, identity: &str, sequence: Option<DecimalU64>, expected: usize, written: u64) {
        if let Some(entry) = self.state.history.get_mut(identity).and_then(|h| h.entries.iter_mut().find(|e| Some(DecimalU64(e.sequence)) == sequence)) {
            entry.outcome = Some(Reply::ok(json!({"operation_id":DecimalU64(entry.operation),"status":if written >= expected as u64 {"completed"} else {"unknown"},"boundary":"pty_write","reason":"input_lease_ended","delivered_bytes_lower_bound":written})));
        }
    }
    /// Preserve high-water marks when entries expire, exactly as gold. A lost
    /// outcome must not turn a committed mutation into a fresh request.
    pub fn lookup(&mut self, identity: &str, sequence: u64, digest: [u8; 32]) -> Option<Reply> {
        for history in self.state.history.values_mut() {
            while history
                .entries
                .front()
                .is_some_and(|e| e.at.elapsed() >= RETENTION)
            {
                history.entries.pop_front();
            }
        }
        let history = self.state.history.get(identity)?;
        if let Some(entry) = history.entries.iter().find(|e| e.sequence == sequence) {
            return Some(if entry.digest == digest {
                entry.reply.clone()
            } else {
                Reply::refuse("CONFLICT", Some(mismatch()))
            });
        }
        (sequence <= history.high_water)
            .then(|| Reply::refuse("UNKNOWN_OUTCOME", Some(retired(history.high_water))))
    }

    /// Outcomes survive pane removal. Gold scopes this query by target and
    /// operation identity, rather than by the currently selected pane/caller.
    pub fn operation(&self, target: &Target, operation: u64) -> Reply {
        match self
            .state
            .history
            .values()
            .flat_map(|h| &h.entries)
            .find(|entry| {
                entry.operation == operation
                    && entry.target == *target
                    && entry.at.elapsed() < RETENTION
            }) {
            Some(entry) => entry.outcome.as_ref().unwrap_or(&entry.reply).clone(),
            None => Reply::error("UNKNOWN_OUTCOME"),
        }
    }

    pub fn high_water(&self, identity: &str) -> u64 {
        self.state.history.get(identity).map_or(0, |h| h.high_water)
    }
    pub fn reserve_request_id(
        &mut self,
        identity: &str,
        sequence: u64,
        digest: [u8; 32],
    ) -> Result<(), Reply> {
        let state = &mut self.state;
        for history in state.history.values_mut() {
            while history
                .entries
                .front()
                .is_some_and(|e| e.at.elapsed() >= RETENTION)
            {
                history.entries.pop_front();
            }
        }
        let total = state
            .history
            .values()
            .map(|h| h.entries.len())
            .sum::<usize>();
        // A key keeps its high-water mark even after every entry expires,
        // so a late retry answers UNKNOWN_OUTCOME instead of re-executing;
        // ageing alone therefore never drops one. The cap still has to
        // evict rather than refuse. An ambient key names one connection and
        // a connection id never returns, so a full table is overwhelmingly
        // keys that can never be addressed again — refusing at the cap
        // would brick every mutation on the instance permanently, which is
        // strictly worse than losing the coldest actor's dedupe. Keys with
        // no live entry go first, since losing one costs only a high-water
        // mark; then the coldest overall. BROKER-023 allows earlier
        // eviction at the instance cap, and TOTAL still bounds memory.
        if !state.history.contains_key(identity) && state.history.len() >= ACTORS {
            let victim = state
                .history
                .iter()
                .min_by_key(|(_, history)| (!history.entries.is_empty(), history.last))
                .map(|(key, _)| key.clone());
            if let Some(key) = victim {
                state.history.remove(&key);
            }
        }
        let history = state.history.entry(identity.to_owned()).or_default();
        if let Some(entry) = history.entries.iter().find(|e| e.sequence == sequence) {
            return Err(if entry.digest == digest {
                entry.reply.clone()
            } else {
                Reply::refuse("CONFLICT", Some(mismatch()))
            });
        }
        if sequence <= history.high_water {
            return Err(Reply::refuse(
                "UNKNOWN_OUTCOME",
                Some(retired(history.high_water)),
            ));
        }
        if total >= TOTAL {
            return Err(Reply::error("RESOURCE_LIMIT"));
        }
        history.high_water = sequence;
        Ok(())
    }

    /// The id Term forwards for one caller request, minted once and remembered
    /// so a byte-identical retry forwards the SAME id and reaches the child's
    /// own dedupe rather than becoming a second submission.
    pub fn forwarded_id(&mut self, identity: &str, sequence: u64, digest: [u8; 32]) -> Forward {
        let state = &mut self.state;
        let key = format!("{identity}\0{sequence}");
        if let Some((existing, recorded)) = state.forwarded.get(&key) {
            return if *recorded == digest {
                Forward::Retry(*existing)
            } else {
                Forward::Conflict
            };
        }
        state.next_forward += 1;
        let minted = state.next_forward;
        // Bounded alongside the history it belongs to. Evict ONE key, not
        // the whole table: clearing it would strip every live caller of its
        // retry path at once, turning a capacity event into a fleet-wide
        // outage of exactly the property the mapping exists to provide.
        if state.forwarded.len() >= TOTAL
            && let Some(victim) = state.forwarded.keys().next().cloned()
        {
            state.forwarded.remove(&victim);
        }
        state.forwarded.insert(key, (minted, digest));
        Forward::Fresh(minted)
    }

    /// Retain a completed mutation's outcome so a retry replays it.
    pub fn record_outcome(
        &mut self,
        identity: &str,
        target: &Target,
        sequence: u64,
        digest: [u8; 32],
        reply: &mut Reply,
    ) {
        let state = &mut self.state;
        state.next_operation = state
            .next_operation
            .checked_add(1)
            .expect("operation id exhausted");
        let operation = state.next_operation;
        let mut body: Value = serde_json::from_str(&reply.body).expect("owned reply JSON");
        let child_operation = body
            .get("operation_id")
            .cloned()
            .and_then(|id| serde_json::from_value(id).ok());
        if child_operation.is_some() {
            body["operation_id"] = json!(DecimalU64(operation));
            reply.body = body.to_string();
        }
        let Some(history) = state.history.get_mut(identity) else {
            return;
        };
        history.last = Instant::now();
        history.entries.push_back(Entry {
            target: target.clone(),
            outcome: None,
            sequence,
            operation,
            child_operation,
            digest,
            at: Instant::now(),
            reply: reply.clone(),
        });
        if history.entries.len() > PER_ACTOR {
            history.entries.pop_front();
        }
    }
}
