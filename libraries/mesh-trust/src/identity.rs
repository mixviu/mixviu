// SPDX-License-Identifier: MIT OR Apache-2.0
//! Immutable key authority, built once for each accepted inventory revision.
use std::collections::{BTreeMap, HashMap};

use bus::native_session::PeerIdentity;
use ed25519_dalek::{Signature, VerifyingKey};
use serde_json::Value;

use crate::admission::{AdmissionTranscript, select_d2_pubkeys};

/// Validate purpose separation and irreversible closed windows in one snapshot.
/// Missing credential lists remain valid for inventories predating admission.
pub fn validate_member_keys(members: &Value, epoch: u64) -> Result<(), String> {
    let records = key_windows(members)?;
    for (i, a) in records.iter().enumerate() {
        for b in records.iter().skip(i + 1) {
            if a.key != b.key {
                continue;
            }
            if (a.kind == "d2" && b.kind == "bus-tls") || (a.kind == "bus-tls" && b.kind == "d2") {
                return Err("pubkey is shared by d2 and bus-tls credentials".into());
            }
            if a.until.is_some_and(|end| b.from >= end)
                || b.until.is_some_and(|end| a.from >= end)
                || (a.tombstoned && b.current(epoch))
                || (b.tombstoned && a.current(epoch))
            {
                return Err("pubkey reopens a closed credential window".into());
            }
        }
    }
    Ok(())
}

/// Only the accepted snapshot supplies history; nothing is persisted or pruned.
pub fn validate_key_reload(
    current: &Value,
    current_epoch: u64,
    candidate: &Value,
    candidate_epoch: u64,
) -> Result<(), String> {
    validate_member_keys(candidate, candidate_epoch)?;
    let old = key_windows(current)?;
    let next = key_windows(candidate)?;
    for retained in old
        .iter()
        .filter(|key| matches!(key.kind.as_str(), "d2" | "bus-tls"))
    {
        let same = |key: &&KeyWindow| key.kind == retained.kind && key.key == retained.key;
        if !next.iter().any(|key| same(&key)) {
            return Err(format!(
                "member '{}' candidate drops retained {} key",
                retained.member, retained.kind
            ));
        }
        let closed = old
            .iter()
            .filter(same)
            .all(|key| key.tombstoned || key.until.is_some_and(|end| end <= current_epoch));
        if closed {
            for key in next.iter().filter(same) {
                if !key.tombstoned && !key.until.is_some_and(|end| end <= candidate_epoch) {
                    return Err(format!(
                        "member '{}' candidate reopens closed {} key",
                        key.member, key.kind
                    ));
                }
            }
        }
    }
    for retained in old.iter().filter(|key| key.kind == "wg") {
        let closed = old
            .iter()
            .filter(|key| key.kind == "wg" && key.key == retained.key)
            .all(|key| key.tombstoned || key.until.is_some_and(|end| end <= current_epoch));
        if closed && next
            .iter()
            .any(|key| key.key == retained.key && key.current(candidate_epoch))
        {
            return Err(
                "candidate makes a key current that is closed in the accepted inventory".into(),
            );
        }
    }
    Ok(())
}

struct KeyWindow {
    member: String,
    key: Vec<u8>,
    kind: String,
    from: u64,
    until: Option<u64>,
    tombstoned: bool,
    eligible: bool,
}
impl KeyWindow {
    fn current(&self, epoch: u64) -> bool {
        self.eligible
            && !self.tombstoned
            && self.from <= epoch
            && self.until.is_none_or(|end| epoch < end)
    }
}
fn key_windows(members: &Value) -> Result<Vec<KeyWindow>, String> {
    use base64::Engine as _;
    let mut out = Vec::new();
    for member in members.as_array().ok_or("members must be an array")? {
        let Some(credentials) = member.get("credentials") else {
            continue;
        };
        for c in credentials
            .as_array()
            .ok_or("credentials must be an array")?
        {
            let kind = c["kind"]
                .as_str()
                .ok_or("credential kind must be a string")?;
            if !matches!(kind, "wg" | "d2" | "bus-tls") {
                return Err("unknown credential kind".into());
            }
            // Ineligible owners still poison shared TLS keys in the reverse
            // index. Authoring refuses them; snapshot verification preserves
            // this defensive coverage rather than silently deleting records.
            let key = base64::engine::general_purpose::STANDARD
                .decode(c["pubkey"].as_str().ok_or("missing credential pubkey")?)
                .map_err(|_| "invalid credential pubkey")?;
            // Historical member JSON is opaque, including old WG fixtures.
            // The admission selectors enforce usable key length separately.
            let from = c["from_epoch"]
                .as_u64()
                .ok_or("invalid credential from_epoch")?;
            let until = match c.get("until_epoch") {
                None | Some(Value::Null) => None,
                Some(value) => Some(value.as_u64().ok_or("invalid credential until_epoch")?),
            };
            // Empty historical windows are inert in the selectors. Retain
            // them as closed-key evidence without adding a new schema gate.
            out.push(KeyWindow {
                member: member["name"].as_str().unwrap_or("<unnamed>").into(),
                key,
                kind: kind.into(),
                from,
                until,
                tombstoned: member["status"] == "tombstoned",
                eligible: kind == "wg" || member["bus"] == true,
            });
        }
    }
    Ok(out)
}

/// Shared inbound/outbound revocation policy. Both caches must come from the
/// same accepted inventory. Legacy transports retain the D2-only check.
pub fn link_refusal(
    context: &bus::native_session::PeerContext,
    d2: &KeyAuthority,
    tls: &TlsAuthority,
) -> Option<&'static str> {
    let Some((node, key)) = context.mesh_peer() else {
        return Some("unproven-link");
    };
    if !d2.contains(node, key) {
        return Some("d2-key-revoked");
    }
    if let Some(bus::native_session::TransportIdentity::MeshTls {
        member,
        tls_key,
        d2_key,
        ..
    }) = &context.transport
    {
        if member != node || d2_key != key {
            return Some("credential-member-mismatch");
        }
        if !tls
            .member_for(tls_key)
            .is_some_and(|identity| identity.member == node)
        {
            return Some("tls-key-revoked");
        }
    }
    None
}

/// Identity authenticated by the TLS CertificateVerify proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TlsMember {
    pub member: String,
    pub epoch: u64,
    pub revision: u64,
    pub key: [u8; 32],
}

/// Immutable reverse index built once per accepted revision. Ineligible owners
/// poison shared keys too, so a duplicate never identifies an eligible owner.
#[derive(Debug, Default)]
pub struct TlsAuthority {
    epoch: u64,
    revision: u64,
    keys: HashMap<[u8; 32], Option<String>>,
}

impl TlsAuthority {
    pub fn new(members: &BTreeMap<String, Value>, epoch: u64, revision: u64) -> Self {
        let mut keys = HashMap::new();
        for (name, member) in members {
            for key in crate::admission::select_bus_tls_pubkeys(member, epoch) {
                let key: [u8; 32] = key.try_into().expect("selector checks length");
                let owner =
                    (member["status"] == "active" && member["bus"] == true).then(|| name.clone());
                keys.entry(key)
                    .and_modify(|old| {
                        if old != &owner {
                            *old = None;
                        }
                    })
                    .or_insert(owner);
            }
        }
        Self {
            epoch,
            revision,
            keys,
        }
    }

    pub fn member_for(&self, key: &[u8; 32]) -> Option<TlsMember> {
        Some(TlsMember {
            member: self.keys.get(key)?.as_ref()?.clone(),
            epoch: self.epoch,
            revision: self.revision,
            key: *key,
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct KeyAuthority {
    epoch: u64,
    revision: u64,
    // None records an ambiguous key, including after a third occurrence.
    keys: HashMap<[u8; 32], Option<String>>,
    members: HashMap<String, Vec<[u8; 32]>>,
}

impl KeyAuthority {
    pub fn epoch(&self) -> u64 {
        self.epoch
    }

    pub fn authenticate_response(
        &self,
        challenge: &Value,
        response: &Value,
        expected: &str,
    ) -> Option<PeerIdentity> {
        self.authenticate_bound_response(challenge, response, expected, None)
    }

    pub fn authenticate_bound_response(
        &self,
        challenge: &Value,
        response: &Value,
        expected: &str,
        binding: Option<&[u8; 32]>,
    ) -> Option<PeerIdentity> {
        use base64::Engine as _;
        let b64 = base64::engine::general_purpose::STANDARD;
        if response["claimed_source_node"].as_str()? != expected {
            return None;
        }
        let transcript = AdmissionTranscript {
            mesh_fqdn: challenge["mesh_fqdn"].as_str()?.to_owned(),
            claimed_source_node: expected.to_owned(),
            verifying_broker_node: challenge["verifying_broker_node"].as_str()?.to_owned(),
            inventory_epoch: challenge["inventory_epoch"].as_u64()?,
            session_id: b64.decode(challenge["session_id"].as_str()?).ok()?,
            server_nonce: b64.decode(challenge["server_nonce"].as_str()?).ok()?,
            client_ephemeral: vec![0; 32],
            channel_binding_hash: binding.unwrap_or(&[0; 32]).to_vec(),
        };
        if b64.decode(response["client_ephemeral"].as_str()?).ok()? != [0; 32]
            || b64
                .decode(response["channel_binding_hash"].as_str()?)
                .ok()?
                != binding.unwrap_or(&[0; 32])
            || binding.is_some_and(|hash| *hash == [0; 32])
        {
            return None;
        }
        self.prove(
            &transcript,
            &b64.decode(response["signature"].as_str()?).ok()?,
        )
    }
    pub fn new(members: &BTreeMap<String, Value>, epoch: u64, revision: u64) -> Self {
        let mut authority = Self {
            epoch,
            revision,
            ..Self::default()
        };
        for (node, member) in members {
            let eligible = member["status"] == "active" && member["bus"] == true;
            for key in select_d2_pubkeys(member, epoch) {
                let Ok(key) = <[u8; 32]>::try_from(key) else {
                    continue;
                };
                authority
                    .keys
                    .entry(key)
                    .and_modify(|owner| {
                        if owner.as_deref() != Some(node) {
                            *owner = None;
                        }
                    })
                    .or_insert_with(|| eligible.then(|| node.clone()));
                if eligible {
                    authority.members.entry(node.clone()).or_default().push(key);
                }
            }
        }
        authority
    }

    pub fn contains(&self, node: &str, key: &[u8; 32]) -> bool {
        self.keys.get(key).and_then(Option::as_deref) == Some(node)
    }

    /// Verify only the claimed member's current keys. No per-frame inventory scan.
    pub fn prove(
        &self,
        transcript: &AdmissionTranscript,
        signature: &[u8],
    ) -> Option<PeerIdentity> {
        if transcript.inventory_epoch != self.epoch {
            return None;
        }
        let node = &transcript.claimed_source_node;
        let signature = Signature::from_slice(signature).ok()?;
        let bytes = transcript.canonical_bytes();
        self.members.get(node)?.iter().find_map(|key| {
            if !self.contains(node, key) {
                return None;
            }
            VerifyingKey::from_bytes(key)
                .ok()?
                .verify_strict(&bytes, &signature)
                .ok()?;
            Some(PeerIdentity::MeshAdmitted {
                node: node.clone(),
                key: *key,
                fingerprint: blake3::hash(key).to_hex().to_string(),
                epoch: self.epoch,
                revision: self.revision,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::admission::sign_admission_transcript;
    use base64::Engine as _;

    #[test]
    fn wg_overlapping_records_keep_key_open_independent_of_order() {
        let mut accepted = member(&[[7; 32], [7; 32]]);
        accepted["credentials"][0]["kind"] = serde_json::json!("wg");
        accepted["credentials"][0]["until_epoch"] = serde_json::json!(9);
        accepted["credentials"][1]["kind"] = serde_json::json!("wg");
        accepted["credentials"][1]["from_epoch"] = serde_json::json!(8);
        for reverse_old in [false, true] {
            for reverse_next in [false, true] {
                let mut old = accepted.clone();
                let mut next = accepted.clone();
                if reverse_old {
                    old["credentials"].as_array_mut().unwrap().reverse();
                }
                if reverse_next {
                    next["credentials"].as_array_mut().unwrap().reverse();
                }
                assert!(validate_member_keys(&serde_json::json!([old.clone()]), 10).is_ok());
                assert!(validate_key_reload(
                    &serde_json::json!([old]), 10, &serde_json::json!([next]), 11
                ).is_ok());
            }
        }
    }

    #[test]
    fn wg_closed_key_cannot_reopen_with_overlapping_record_independent_of_order() {
        let mut accepted = member(&[[7; 32]]);
        accepted["credentials"][0]["kind"] = serde_json::json!("wg");
        accepted["credentials"][0]["until_epoch"] = serde_json::json!(9);
        let mut next = accepted.clone();
        let mut reopened = next["credentials"][0].clone();
        reopened["from_epoch"] = serde_json::json!(8);
        reopened["until_epoch"] = Value::Null;
        next["credentials"].as_array_mut().unwrap().push(reopened);
        for reverse in [false, true] {
            let mut candidate = next.clone();
            if reverse {
                candidate["credentials"].as_array_mut().unwrap().reverse();
            }
            let candidate = serde_json::json!([candidate]);
            assert!(validate_member_keys(&candidate, 11).is_ok());
            assert!(validate_key_reload(&serde_json::json!([accepted.clone()]), 10, &candidate, 11)
                .unwrap_err().contains("closed in the accepted inventory"));
        }
    }

    #[test]
    fn live_d2_key_allows_signed_suspension() {
        let live = member(&[[7; 32]]);
        let mut future = live.clone();
        future["credentials"][0]["from_epoch"] = serde_json::json!(12);
        let mut no_bus = live.clone();
        no_bus["bus"] = serde_json::json!(false);
        for suspended in [future, no_bus] {
            assert!(validate_key_reload(
                &serde_json::json!([live.clone()]), 10, &serde_json::json!([suspended]), 11
            ).is_ok());
        }
    }

    #[test]
    fn reload_retention_staging_and_rotation_table() {
        for kind in ["d2", "bus-tls"] {
            let mut live = member(&[[7; 32]]);
            live["name"] = serde_json::json!("alpha");
            live["credentials"][0]["kind"] = serde_json::json!(kind);
            let mut closed = live.clone();
            closed["credentials"][0]["until_epoch"] = serde_json::json!(2);
            let mut tombstone = live.clone();
            tombstone["status"] = serde_json::json!("tombstoned");
            let mut dropped = live.clone();
            dropped["credentials"] = serde_json::json!([]);
            let mut future = live.clone();
            future["credentials"][0]["from_epoch"] = serde_json::json!(5);
            let mut staged = live.clone();
            staged["bus"] = serde_json::json!(false);
            let mut relabel = live.clone();
            relabel["credentials"][0]["kind"] =
                serde_json::json!(if kind == "d2" { "bus-tls" } else { "d2" });
            let mut rotated = closed.clone();
            let mut fresh = live["credentials"][0].clone();
            fresh["pubkey"] =
                serde_json::json!(base64::engine::general_purpose::STANDARD.encode([8; 32]));
            fresh["from_epoch"] = serde_json::json!(3);
            rotated["credentials"]
                .as_array_mut()
                .unwrap()
                .push(fresh.clone());
            let mut overlap = live.clone();
            overlap["credentials"].as_array_mut().unwrap().push(fresh);
            for (label, old, next, ok) in [
                (
                    "drop record",
                    live.clone(),
                    serde_json::json!([dropped]),
                    false,
                ),
                (
                    "drop tombstoned member",
                    tombstone.clone(),
                    serde_json::json!([]),
                    false,
                ),
                (
                    "future staging",
                    closed.clone(),
                    serde_json::json!([future]),
                    false,
                ),
                (
                    "bus:false staging",
                    closed.clone(),
                    serde_json::json!([staged]),
                    false,
                ),
                ("relabel", live.clone(), serde_json::json!([relabel]), false),
                (
                    "un-tombstone",
                    tombstone,
                    serde_json::json!([live.clone()]),
                    false,
                ),
                ("overlap", live.clone(), serde_json::json!([overlap]), true),
                (
                    "readmission",
                    closed,
                    serde_json::json!([rotated.clone()]),
                    true,
                ),
                ("rotation", live, serde_json::json!([rotated]), true),
            ] {
                assert_eq!(
                    validate_key_reload(&serde_json::json!([old]), 2, &next, 3).is_ok(),
                    ok,
                    "{kind}: {label}"
                );
            }
        }
    }

    #[test]
    fn two_eligible_tls_owners_identify_neither() {
        let mut tls = member(&[[7; 32]]);
        tls["credentials"][0]["kind"] = serde_json::json!("bus-tls");
        let authority = TlsAuthority::new(
            &BTreeMap::from([("alpha".into(), tls.clone()), ("beta".into(), tls)]),
            1,
            1,
        );
        assert!(authority.member_for(&[7; 32]).is_none());
    }

    #[test]
    fn purpose_and_closed_windows_reject_independent_of_order() {
        let mut old = member(&[[7; 32]]);
        old["credentials"][0]["until_epoch"] = serde_json::json!(2);
        let mut next = member(&[[7; 32]]);
        next["credentials"][0]["kind"] = serde_json::json!("bus-tls");
        for records in [
            serde_json::json!([old.clone(), next.clone()]),
            serde_json::json!([next.clone(), old.clone()]),
        ] {
            assert!(
                validate_member_keys(&records, 3)
                    .unwrap_err()
                    .contains("d2 and bus-tls")
            );
        }
        next["credentials"][0]["kind"] = serde_json::json!("d2");
        next["credentials"][0]["from_epoch"] = serde_json::json!(2);
        for records in [
            serde_json::json!([old.clone(), next.clone()]),
            serde_json::json!([next.clone(), old.clone()]),
        ] {
            assert!(
                validate_member_keys(&records, 3)
                    .unwrap_err()
                    .contains("closed")
            );
        }
        assert!(
            validate_key_reload(
                &serde_json::json!([old.clone()]),
                2,
                &serde_json::json!([next]),
                3
            )
            .is_err()
        );
        assert!(
            validate_key_reload(
                &serde_json::json!([old.clone()]),
                2,
                &serde_json::json!([old, member(&[[8; 32]])]),
                3
            )
            .is_ok()
        );
    }

    #[test]
    fn tls_reverse_index_is_purpose_separated_and_fail_closed() {
        use crate::admission::select_bus_tls_pubkeys;
        let key = [7; 32];
        let mut tls = member(&[key]);
        assert!(select_bus_tls_pubkeys(&tls, 1).is_empty());
        tls["credentials"][0]["kind"] = serde_json::json!("bus-tls");
        let index = |members| TlsAuthority::new(&members, 1, 42);
        let records = BTreeMap::from([("alpha".into(), tls.clone())]);
        let identity = index(records).member_for(&key).unwrap();
        assert_eq!(identity.member, "alpha");
        assert_eq!((identity.epoch, identity.revision), (1, 42));
        for field in ["from_epoch", "until_epoch", "pubkey"] {
            let mut bad = tls.clone();
            bad["credentials"][0][field] = serde_json::json!(false);
            assert!(select_bus_tls_pubkeys(&bad, 1).is_empty());
        }
        let mut missing = tls.clone();
        missing["credentials"][0]
            .as_object_mut()
            .unwrap()
            .remove("from_epoch");
        assert!(select_bus_tls_pubkeys(&missing, 1).is_empty());
        let mut closed = tls.clone();
        closed["credentials"][0]["until_epoch"] = serde_json::json!(2);
        assert_eq!(select_bus_tls_pubkeys(&closed, 1).len(), 1);
        assert!(select_bus_tls_pubkeys(&closed, 2).is_empty());
    }

    fn assert_tls_ineligible_poison(ineligible: &str) {
        let key = [7; 32];
        let mut tls = member(&[key]);
        tls["credentials"][0]["kind"] = serde_json::json!("bus-tls");
        let mut no_bus = tls.clone();
        no_bus["bus"] = serde_json::json!(false);
        let authority = TlsAuthority::new(
            &BTreeMap::from([("alpha".into(), tls), (ineligible.into(), no_bus)]),
            1,
            42,
        );
        assert!(
            authority.member_for(&key).is_none(),
            "ineligible {ineligible}"
        );
    }

    #[test]
    fn tls_ineligible_owner_after_eligible_poisons_key() {
        assert_tls_ineligible_poison("beta");
    }

    #[test]
    fn tls_ineligible_owner_before_eligible_poisons_key() {
        assert_tls_ineligible_poison("aardvark");
    }

    fn member(keys: &[[u8; 32]]) -> Value {
        serde_json::json!({"status":"active", "bus":true, "credentials": keys.iter().map(|key|
            serde_json::json!({"kind":"d2", "pubkey":base64::engine::general_purpose::STANDARD.encode(key), "from_epoch":1, "until_epoch":null})
        ).collect::<Vec<_>>()})
    }

    #[test]
    fn rotation_keeps_exact_overlap_and_refuses_session_reuse_after_old_key_closes() {
        let old = ed25519_dalek::SigningKey::from_bytes(&[7; 32]);
        let new = ed25519_dalek::SigningKey::from_bytes(&[8; 32]);
        let old_key = old.verifying_key().to_bytes();
        let new_key = new.verifying_key().to_bytes();
        let transcript = AdmissionTranscript {
            mesh_fqdn: "example.test".into(),
            claimed_source_node: "alpha".into(),
            verifying_broker_node: "beta".into(),
            inventory_epoch: 1,
            session_id: vec![1; 16],
            server_nonce: vec![2; 32],
            client_ephemeral: vec![0; 32],
            channel_binding_hash: vec![0; 32],
        };
        let proof = sign_admission_transcript(&old.to_bytes(), &transcript).unwrap();
        let overlap = KeyAuthority::new(
            &BTreeMap::from([("alpha".into(), member(&[old_key, new_key]))]),
            1,
            0,
        );
        let identity = overlap.prove(&transcript, &proof).unwrap();
        assert!(matches!(identity, PeerIdentity::MeshAdmitted { key, .. } if key == old_key));
        let rotated = KeyAuthority::new(
            &BTreeMap::from([("alpha".into(), member(&[new_key]))]),
            2,
            0,
        );
        assert!(!rotated.contains("alpha", &old_key));
        assert!(rotated.contains("alpha", &new_key));
        assert!(rotated.prove(&transcript, &proof).is_none());
        assert!(overlap.contains("alpha", &old_key)); // immutable in-flight snapshot
    }

    #[test]
    fn shared_key_identifies_neither_member_and_bus_false_cannot_admit() {
        let key = ed25519_dalek::SigningKey::from_bytes(&[7; 32])
            .verifying_key()
            .to_bytes();
        let authority = KeyAuthority::new(
            &BTreeMap::from([
                ("alpha".into(), member(&[key])),
                ("beta".into(), member(&[key])),
                ("gamma".into(), member(&[key])),
            ]),
            1,
            0,
        );
        for node in ["alpha", "beta", "gamma"] {
            assert!(!authority.contains(node, &key));
        }
        let mut transport = member(&[key]);
        transport["bus"] = Value::Bool(false);
        assert!(
            !KeyAuthority::new(&BTreeMap::from([("alpha".into(), transport)]), 1, 0)
                .contains("alpha", &key)
        );
    }
}
