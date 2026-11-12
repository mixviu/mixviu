// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Contract 1.0.0's reads and topics (key space v2 §5.1, §5.2), answered
//! the same way by the authority and by a mirror from its cache: the
//! mirror computes nothing the authority does not.
use crate::authority::Authority;
use crate::publication;
use serde_json::{Value, json};
use settings::topics::{self, KeysBody, KeysContent, LookBody, LookContent, Resolved};
use settings::{ReadRequest, Revision, View};
use std::collections::BTreeMap;

impl Authority {
    /// What the registry resolves beside the authored choices.
    pub fn resolved(&self) -> Resolved {
        Resolved {
            mode: topics::resolved_mode(&self.accepted.values),
            accent: self.accent_hex(),
        }
    }

    /// The schema 3 snapshot.
    pub fn snapshot_v3(&self) -> Result<settings::compact::ValuesSnapshot, Value> {
        settings::compact::encode_values(&self.snapshot, &self.accepted.values)
            .map_err(|e| json!({"status":"validation_failed","message":e.to_string()}))
    }

    fn look(&self) -> LookContent {
        topics::look_content(
            &self.accepted.values,
            &self.resolved(),
            &self.snapshot.source_digest,
        )
    }

    /// Every topic's content, by its board key.
    fn contents(&self) -> BTreeMap<String, Value> {
        let mut out: BTreeMap<String, Value> =
            topics::contents(&self.accepted.values, &self.resolved())
                .into_iter()
                .map(|(prefix, content)| {
                    (
                        prefix.to_owned(),
                        serde_json::to_value(content).unwrap_or(Value::Null),
                    )
                })
                .collect();
        out.insert(
            publication::LOOK.into(),
            serde_json::to_value(self.look()).unwrap_or(Value::Null),
        );
        out.insert(
            publication::SNAPSHOT.into(),
            self.snapshot_v3()
                .ok()
                .and_then(|s| serde_json::to_value(s).ok())
                .unwrap_or(Value::Null),
        );
        out
    }

    /// Stage the current record on the board (after a commit or a load).
    pub fn stage(&mut self) {
        self.stage_with(None);
    }

    /// Stage with the revision that last changed each topic given (a mirror
    /// adopting its authority's stamps). Returns the topics that moved.
    pub fn stage_with(&mut self, changed_at: Option<&BTreeMap<String, Revision>>) -> Vec<String> {
        self.palettes.clear();
        let contents = self.contents();
        let revision = self.accepted.revision;
        self.board.stage(revision, contents, changed_at)
    }

    /// One pending topic's retained name and body.
    pub fn topic(&self, key: &str) -> Option<(String, String)> {
        let profile = &self.accepted.binding.profile;
        // A content that could not be built (null) is no body: publication
        // fails and the topic stays pending.
        let content = self.board.content(key)?.clone();
        if content.is_null() {
            return None;
        }
        let changed_at = self.board.changed_at_of(key)?;
        let identity = (
            self.accepted.binding.clone(),
            self.accepted.incarnation.clone(),
            self.accepted.revision,
        );
        let (name, body) = match key {
            publication::SNAPSHOT => (topics::snapshot_topic(profile), content),
            publication::LOOK => {
                let look: LookContent = serde_json::from_value(content).ok()?;
                let body = LookBody {
                    schema: topics::TOPIC_SCHEMA,
                    binding: identity.0,
                    incarnation: identity.1,
                    revision: identity.2,
                    changed_at,
                    values: look.values,
                    resolved: look.resolved,
                    source_digest: look.source_digest,
                };
                (
                    topics::look_topic(profile),
                    serde_json::to_value(body).ok()?,
                )
            }
            prefix => {
                let keys: KeysContent = serde_json::from_value(content).ok()?;
                let body = KeysBody {
                    schema: topics::TOPIC_SCHEMA,
                    binding: identity.0,
                    incarnation: identity.1,
                    revision: identity.2,
                    changed_at,
                    values: keys.values,
                    resolved: keys.resolved,
                };
                (
                    topics::keys_topic(profile, prefix),
                    serde_json::to_value(body).ok()?,
                )
            }
        };
        Some((name, serde_json::to_string(&body).ok()?))
    }

    /// `settings.get` in a 1.0.0 view.
    pub fn read_v1(&self, request: &ReadRequest) -> Result<Value, Value> {
        let identity = json!({
            "binding": self.accepted.binding,
            "incarnation": self.accepted.incarnation,
            "revision": self.accepted.revision,
        });
        let mut answer = match request.view {
            View::Values => {
                let values = topics::view_values(
                    &self.accepted.values,
                    request.prefix.as_deref(),
                    request.keys.as_deref(),
                )
                .map_err(|d| json!({"status":"validation_failed","diagnostics":d}))?;
                // A prefix's stamp, for Prefs' local conflict check (§5.2).
                let changed_at = request
                    .prefix
                    .as_deref()
                    .and_then(topics::prefix_of)
                    .and_then(|p| self.board.changed_at_of(p));
                let mut answer = json!({"values": values, "resolved": self.resolved()});
                if let Some(at) = changed_at {
                    answer["changed_at"] = json!(at);
                }
                answer
            }
            View::Look => json!({"look": self.look()}),
            View::Snapshot => json!({
                "snapshot": self.snapshot_v3()?,
                "changed_at": self.board.changed_at(),
            }),
        };
        answer["status"] = "current".into();
        if let (Some(answer), Some(identity)) = (answer.as_object_mut(), identity.as_object()) {
            answer.extend(identity.clone());
        }
        Ok(answer)
    }

    /// `settings.palette` (§5.2, ruling 1): the desktop look's palette per
    /// mode from `design::palette`, tagged with the record's identity. A
    /// mirror answers from its cache with the same code, so no palette
    /// crosses the wire between them.
    pub fn palette(&mut self, request: &settings::PaletteRequest) -> Result<Value, Value> {
        self.palette_with(request, settings::resolve::values::palette)
    }

    fn palette_with(
        &mut self,
        request: &settings::PaletteRequest,
        mut resolve: impl FnMut(
            &settings::registry::Values,
            &str,
            Option<settings::resolve::values::Mode>,
        )
            -> Result<serde_json::Map<String, Value>, Vec<settings::Diagnostic>>,
    ) -> Result<Value, Value> {
        if request.binding != self.accepted.binding {
            return Err(json!({"status":"wrong_target","binding":self.accepted.binding}));
        }
        let mode = match request.mode.as_deref() {
            None => None,
            Some(name) => Some(settings::resolve::values::Mode::from_name(name).ok_or_else(
                || json!({"status":"validation_failed","path":"mode","message":"Unknown mode"}),
            )?),
        };
        let mut palette = serde_json::Map::new();
        for each in [
            settings::resolve::values::Mode::Dark,
            settings::resolve::values::Mode::Light,
        ] {
            if mode.is_some_and(|mode| mode != each) {
                continue;
            }
            if !self.palettes.contains_key(each.name()) {
                let mut resolved = resolve(
                    &self.accepted.values,
                    &self.accepted.embedded_source,
                    Some(each),
                )
                .map_err(|d| json!({"status":"validation_failed","diagnostics":d}))?;
                let value = resolved.remove(each.name()).ok_or_else(|| {
                    json!({"status":"validation_failed","message":"Resolved palette mode missing"})
                })?;
                // Keep the completed work before the service sends the reply.
                // Client-side timeouts do not cancel this blocking dispatch.
                self.palettes.insert(each.name().into(), value);
            }
            palette.insert(each.name().into(), self.palettes[each.name()].clone());
        }
        Ok(json!({
            "status": "current",
            "binding": self.accepted.binding,
            "incarnation": self.accepted.incarnation,
            "revision": self.accepted.revision,
            "palette": palette,
        }))
    }

    /// `settings.source` (ADR R14a, App. §K): the pinned source the look
    /// compiles, iff `digest` names it; `stale_digest` with the current one
    /// otherwise, so the follower rereads once. A mirror answers from its
    /// cache, which pins the same source.
    pub fn source(&self, request: &settings::SourceRequest) -> Result<Value, Value> {
        if request.binding != self.accepted.binding {
            return Err(json!({"status":"wrong_target","binding":self.accepted.binding}));
        }
        let current = &self.snapshot.source_digest;
        if &request.digest != current {
            return Err(json!({"status":"stale_digest","digest":current}));
        }
        let source = self
            .accepted
            .values
            .get("appearance.source")
            .and_then(Value::as_str)
            .unwrap_or(&self.accepted.embedded_source);
        Ok(json!({"status":"current","digest":current,"source":source}))
    }

    /// The record a mirror caches, sealed under this authority's `node`.
    pub fn cache(&self, node: &str) -> anyhow::Result<crate::mirror::Cache> {
        crate::mirror::Cache::of(&self.accepted, node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[test]
    fn palette_reuse_is_per_mode_and_cleared_by_a_new_record() {
        let dir = tempfile::tempdir().unwrap();
        let binding = settings::Binding {
            instance: "fixture".into(),
            profile: "default".into(),
        };
        let (store, accepted) =
            Store::create(dir.path(), binding.clone(), Default::default()).unwrap();
        let mut authority = Authority::new(store, accepted).unwrap();
        let calls = std::cell::Cell::new(0);
        let mut resolve =
            |values: &settings::registry::Values,
             _: &str,
             mode: Option<settings::resolve::values::Mode>| {
                calls.set(calls.get() + 1);
                let mode = mode.unwrap().name();
                Ok(serde_json::Map::from_iter([(
                    mode.into(),
                    json!({"contrast":values.get("appearance.contrast").cloned().unwrap_or(json!("normal"))}),
                )]))
            };
        let mut request = settings::PaletteRequest {
            binding: binding.clone(),
            mode: None,
        };
        let before = authority.palette_with(&request, &mut resolve).unwrap();
        assert_eq!(calls.get(), 2, "both modes compiled once");
        request.mode = Some("light".into());
        assert_eq!(
            authority.palette_with(&request, &mut resolve).unwrap()["palette"]["light"],
            before["palette"]["light"]
        );
        request.mode = Some("auto".into());
        assert!(authority.palette_with(&request, &mut resolve).is_err());
        request.mode = Some("dark".into());
        request.binding.profile = "other".into();
        assert!(authority.palette_with(&request, &mut resolve).is_err());
        assert_eq!(
            calls.get(),
            2,
            "cached data never bypasses request validation"
        );
        request.binding = binding.clone();
        authority
            .apply(settings::ApplyRequest {
                binding,
                expected_incarnation: authority.accepted.incarnation.clone(),
                expected_revision: authority.accepted.revision,
                operation_id: "contrast-change".into(),
                changes: BTreeMap::from([("appearance.contrast".into(), json!("high"))]),
                reset: vec![],
                request_digest: None,
            })
            .unwrap();
        let after = authority.palette_with(&request, &mut resolve).unwrap();
        assert_ne!(before["revision"], after["revision"]);
        assert_eq!(after["palette"]["dark"]["contrast"], "high");
        assert_eq!(calls.get(), 3);
        request.mode = None;
        let both = authority.palette_with(&request, &mut resolve).unwrap();
        assert_eq!(both["palette"]["light"]["contrast"], "high");
        assert_eq!(calls.get(), 4, "only the other mode remained to compile");
    }

    #[test]
    fn a_palette_compile_failure_is_not_retained() {
        let dir = tempfile::tempdir().unwrap();
        let binding = settings::Binding {
            instance: "fixture".into(),
            profile: "default".into(),
        };
        let (store, accepted) =
            Store::create(dir.path(), binding.clone(), Default::default()).unwrap();
        let mut authority = Authority::new(store, accepted).unwrap();
        let request = settings::PaletteRequest {
            binding,
            mode: Some("dark".into()),
        };
        assert!(
            authority
                .palette_with(&request, |_, _, _| Err(vec![settings::Diagnostic::new(
                    "fixture", "palette", "failed"
                )]))
                .is_err()
        );
        let answer = authority
            .palette_with(&request, |_, _, _| {
                Ok(serde_json::Map::from_iter([(
                    "dark".into(),
                    json!({"accent":"#123456ff"}),
                )]))
            })
            .unwrap();
        assert_eq!(answer["palette"]["dark"]["accent"], "#123456ff");
    }

    // Force the first requester to leave while compilation is in progress.
    // The authority still finishes that dispatch, as service::handle does.
    // A fresh read of the same accepted record must reuse the completed work,
    // rather than starting another compile that can miss the same deadline.
    #[tokio::test]
    async fn a_completed_palette_survives_an_abandoned_reply() {
        let dir = tempfile::tempdir().unwrap();
        let binding = settings::Binding {
            instance: "fixture".into(),
            profile: "default".into(),
        };
        let (store, accepted) =
            Store::create(dir.path(), binding.clone(), Default::default()).unwrap();
        let mut authority = Authority::new(store, accepted).unwrap();
        let request = settings::PaletteRequest {
            binding,
            mode: Some("dark".into()),
        };
        let calls = Arc::new(AtomicUsize::new(0));
        let counted = calls.clone();
        let first = request.clone();
        let (entered, started) = tokio::sync::oneshot::channel();
        let (release, held) = std::sync::mpsc::channel();
        let (reply, caller) = tokio::sync::oneshot::channel();
        let task = tokio::task::spawn_blocking(move || {
            let mut entered = Some(entered);
            let answer = authority.palette_with(&first, |_, _, _| {
                counted.fetch_add(1, Ordering::SeqCst);
                entered.take().unwrap().send(()).unwrap();
                held.recv().unwrap();
                Ok(serde_json::Map::from_iter([(
                    "dark".into(),
                    json!({"accent":"#123456ff"}),
                )]))
            });
            assert!(reply.send(answer).is_err(), "the first caller has left");
            authority
        });
        started.await.unwrap();
        drop(caller);
        release.send(()).unwrap();
        let mut authority = task.await.unwrap();
        let answer = authority
            .palette_with(&request, |_, _, _| {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(serde_json::Map::from_iter([(
                    "dark".into(),
                    json!({"accent":"#123456ff"}),
                )]))
            })
            .unwrap();
        assert_eq!(answer["palette"]["dark"]["accent"], "#123456ff");
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "a retry must not compile the same palette again"
        );
    }
}
