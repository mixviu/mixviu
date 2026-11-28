// SPDX-License-Identifier: MIT OR Apache-2.0
//! History-only names. No routing decision consumes this projection.
use super::*;
use std::ops::{Deref, DerefMut};

#[derive(Default)]
pub(super) struct Names {
    services: HashMap<String, (mpsc::Sender<String>, bool)>,
    commands: HashMap<String, Vec<(String, mpsc::Sender<String>)>>,
}

impl Names {
    pub fn vouched_command(&self, name: &str, allowlist: &std::collections::HashSet<String>) -> bool {
        self.commands.get(name).is_some_and(|owners| owners.iter().any(|(service, tx)| allowlist.contains(service) && !tx.is_closed()))
    }
    fn registered(&self, name: &str) -> bool {
        self.services.get(name).is_some_and(|(tx, _)| !tx.is_closed())
    }
}

/// All registry mutation paths (including session expiry) publish on guard
/// drop, before unlocking. A closed channel stops vouching without a sweep.
pub(crate) struct HistoryRegistry {
    registry: RwLock<HashMap<String, ServiceEntry>>,
    pub(super) names: ArcSwap<Names>,
}
impl HistoryRegistry {
    pub fn new(registry: HashMap<String, ServiceEntry>) -> Self {
        let names = Self::snapshot(&registry);
        Self { registry: RwLock::new(registry), names: ArcSwap::from_pointee(names) }
    }
    fn snapshot(registry: &HashMap<String, ServiceEntry>) -> Names {
        let mut names = Names::default();
        for (service, entry) in registry {
            if entry.tx.is_closed() { continue; }
            names.services.insert(service.clone(), (entry.tx.clone(), !entry.commands.names.is_empty()));
            for name in &entry.commands.names {
                names.commands.entry(name.clone()).or_default().push((service.clone(), entry.tx.clone()));
            }
        }
        names
    }
    pub async fn read(&self) -> tokio::sync::RwLockReadGuard<'_, HashMap<String, ServiceEntry>> { self.registry.read().await }
    pub fn try_read(&self) -> Result<tokio::sync::RwLockReadGuard<'_, HashMap<String, ServiceEntry>>, tokio::sync::TryLockError> { self.registry.try_read() }
    pub async fn write(&self) -> Write<'_> { Write { guard: self.registry.write().await, owner: self } }
    pub fn try_write(&self) -> Result<Write<'_>, tokio::sync::TryLockError> { Ok(Write { guard: self.registry.try_write()?, owner: self }) }
}
pub(crate) struct Write<'a> {
    guard: tokio::sync::RwLockWriteGuard<'a, HashMap<String, ServiceEntry>>,
    owner: &'a HistoryRegistry,
}
impl Deref for Write<'_> { type Target = HashMap<String, ServiceEntry>; fn deref(&self) -> &Self::Target { &self.guard } }
impl DerefMut for Write<'_> { fn deref_mut(&mut self) -> &mut Self::Target { &mut self.guard } }
impl Drop for Write<'_> {
    fn drop(&mut self) {
        self.owner.names.store(Arc::new(HistoryRegistry::snapshot(&self.guard)));
    }
}

pub(crate) struct Projector {
    pub registry: Registry,
    pub authority: Arc<ArcSwap<crate::routing::RoutingAuthority>>,
    pub node: String,
    pub allowlist: std::collections::HashSet<String>,
}

#[derive(Default)]
pub(crate) struct Projected {
    pub from: Option<String>,
    pub to: Option<String>,
    pub command: Option<String>,
    pub reason: Option<&'static str>,
    pub name_len: Option<usize>,
    pub to_sub_len: Option<usize>,
}

impl Projector {
    /// Single component builder for destinations, replies and mesh sources.
    /// PR B replaces the local registered-name arm here with opaque handles.
    fn address(&self, raw: &str, peer: Option<&str>, names: &Names) -> (String, Option<(&'static str, usize)>, Option<usize>) {
        let invalid = |reason| ("invalid-target".into(), Some((reason, raw.len())), None);
        if raw.len() > 253 { return invalid("oversize-name"); }
        let parsed = BusTarget::parse(raw);
        if matches!(parsed, Some(BusTarget::CrossMesh { .. })) { return invalid("refused-cross-mesh"); }
        let (service, node, sub) = match parsed.as_ref() {
            Some(BusTarget::Local(addr)) => (addr.service.as_deref().unwrap_or("noded"), peer.unwrap_or(&addr.node), addr.sub.as_ref().map(String::len)),
            None if bus::wire::is_valid_label(raw) || reserved_session_name(raw) => (raw, peer.unwrap_or(&self.node), None),
            _ => return invalid("malformed-target"),
        };
        if node != self.node {
            let authority = self.authority.load();
            let known = match &authority.posture {
                crate::authority::Posture::Verified(a) => a.members_full.contains_key(node),
                _ => false,
            };
            if !known { return ("unknown-target".into(), Some(("no-such-node", node.len())), sub); }
            if service == "noded" { return (format!("{node}.bus"), None, sub); }
            if self.allowlist.contains(service) || (peer.is_some() && reserved_session_name(service)) {
                return (format!("{service}.{node}.bus"), None, sub);
            }
            return (format!("unknown-service.{node}.bus"), Some(("remote-unvouched", service.len())), sub);
        }
        if service == "noded" { return (format!("{node}.bus"), None, sub); }
        if names.registered(service) { return (service.into(), None, sub); }
        ("unknown-target".into(), Some(("no-such-service", service.len())), sub)
    }

    pub fn project(&self, observation: &Observation<'_>) -> Projected {
        let names = self.registry.names.load();
        let mut result = Projected::default();
        // Canonical local source provenance is already enforced by routing.
        // D2 replaces it in this same projection function in PR B.
        result.from = observation.message.from_addr().map(str::to_owned);
        if observation.direction == ObserveDirection::MeshIn {
            result.from = observation.message.from_addr().map(|raw| {
                let (value, issue, _) = self.address(raw, observation.peer, &names);
                if let Some((reason, len)) = issue { result.reason = Some(reason); result.name_len = Some(len); }
                value
            });
        }
        if let Some(raw) = observation.message.to_addr() {
            let (value, issue, sub) = self.address(raw, None, &names);
            result.to = Some(value);
            result.to_sub_len = sub;
            if let Some((reason, len)) = issue { result.reason = Some(reason); result.name_len = Some(len); }
        }
        if let Some(command) = observation.message.command_name() {
            if commands::BROKER_CONTROL_VERBS.contains(&command) || names.vouched_command(command, &self.allowlist) {
                result.command = Some(command.into());
            } else {
                result.command = Some("unknown".into());
                // Address diagnostics take precedence when both slots project.
                if result.reason.is_none() {
                    let service = if observation.message.get("type") == Some("response") {
                        observation.message.from_addr()
                    } else { observation.message.to_addr() };
                    let service = service.map(|s| BusTarget::parse(s).map(|a| a.local().service.clone().unwrap_or("noded".into())).unwrap_or_else(|| s.into()));
                    let reason = if command.len() > bus::MAX_MANIFEST_NAME_BYTES { "oversize-name" }
                    else if observation.direction != ObserveDirection::Local { "remote-unvouched" }
                    else if service.as_deref() == Some("noded") { "unknown-verb" }
                    else if let Some(service) = service.as_deref().filter(|s| names.registered(s)) {
                        if !self.allowlist.contains(service) { "unlisted-service" }
                        else if !names.services[service].1 { "no-manifest" } else { "unknown-verb" }
                    } else { "unknown-verb" };
                    result.reason = Some(reason);
                    result.name_len = Some(command.len());
                }
            }
        }
        result
    }
}
