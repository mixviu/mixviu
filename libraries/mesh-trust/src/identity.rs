// SPDX-License-Identifier: MIT OR Apache-2.0
//! Immutable key authority, built once for each accepted inventory revision.
use std::collections::{BTreeMap, HashMap};

use bus::native_session::PeerIdentity;
use ed25519_dalek::{Signature, VerifyingKey};
use serde_json::Value;

use crate::admission::{AdmissionTranscript, select_d2_pubkeys};

#[derive(Debug, Clone, Default)]
pub struct KeyAuthority {
    epoch: u64,
    revision: u64,
    // None records an ambiguous key, including after a third occurrence.
    keys: HashMap<[u8; 32], Option<String>>,
    members: HashMap<String, Vec<[u8; 32]>>,
}

impl KeyAuthority {
    pub fn new(members: &BTreeMap<String, Value>, epoch: u64, revision: u64) -> Self {
        let mut authority = Self { epoch, revision, ..Self::default() };
        for (node, member) in members {
            if member["status"] != "active" || member["bus"] != true {
                continue;
            }
            for key in select_d2_pubkeys(member, epoch) {
                let Ok(key) = <[u8; 32]>::try_from(key) else { continue };
                authority.keys.entry(key).and_modify(|owner| {
                    if owner.as_deref() != Some(node) { *owner = None; }
                }).or_insert_with(|| Some(node.clone()));
                authority.members.entry(node.clone()).or_default().push(key);
            }
        }
        authority
    }

    pub fn contains(&self, node: &str, key: &[u8; 32]) -> bool {
        self.keys.get(key).and_then(Option::as_deref) == Some(node)
    }

    /// Verify only the claimed member's current keys. No per-frame inventory scan.
    pub fn prove(&self, transcript: &AdmissionTranscript, signature: &[u8]) -> Option<PeerIdentity> {
        if transcript.inventory_epoch != self.epoch { return None; }
        let node = &transcript.claimed_source_node;
        let signature = Signature::from_slice(signature).ok()?;
        let bytes = transcript.canonical_bytes();
        self.members.get(node)?.iter().find_map(|key| {
            if !self.contains(node, key) { return None; }
            VerifyingKey::from_bytes(key).ok()?.verify_strict(&bytes, &signature).ok()?;
            Some(PeerIdentity::MeshAdmitted {
                node: node.clone(), key: *key,
                fingerprint: blake3::hash(key).to_hex().to_string(),
                epoch: self.epoch, revision: self.revision,
            })
        })
    }
}
