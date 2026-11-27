// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>

//! Mesh peer discovery and WebSocket connection management.
//!
//! Peers are discovered from a config file listing known nodes with their
//! WireGuard mesh IPs and broker ports. The MeshPeers struct manages
//! connections to remote brokers and provides message relay.

use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, RwLockReadGuard, mpsc, oneshot, watch};
use tokio_tungstenite::tungstenite;

use base64::Engine as _;
use bus::wire::{BusMessage, BusTarget};
use mesh_trust::admission::{AdmissionTranscript, sign_admission_transcript};
pub use mesh_trust::routing::DEFAULT_NODED_PORT;

/// Configuration for a single mesh peer.
///
/// `deny_unknown_fields` makes a stale config loud: any peer entry still
/// using the pre-rename `hub_port` key will fail to load instead of
/// silently falling through to the `noded_port` default and connecting
/// to the wrong port on a node that customized it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PeerConfig {
    /// Human-readable node name (used in Bus addresses like `files.delta.bus`)
    pub name: String,
    /// WireGuard mesh IP. Typed as `IpAddr`, not `String`: the broker
    /// hands this straight to `connect_async`, so a hostname here would
    /// make mesh routing consult DNS. Parsing at deserialize time makes
    /// that unrepresentable rather than merely discouraged.
    pub mesh_ip: IpAddr,
    /// Broker WebSocket port on that node (default 4200)
    #[serde(default = "default_noded_port")]
    pub noded_port: u16,
    /// Temporary until inventory endpoints and DNS-SD land in PR 4.
    #[serde(default)]
    pub tls_port: Option<u16>,
}

fn default_noded_port() -> u16 {
    DEFAULT_NODED_PORT
}

impl PeerConfig {
    /// Typed transport address for cache identity and IPv6-safe URL rendering.
    pub fn noded_addr(&self) -> SocketAddr {
        SocketAddr::new(self.mesh_ip, self.tls_port.unwrap_or(self.noded_port))
    }

    /// WebSocket URL for this peer's broker.
    pub fn noded_url(&self) -> String {
        // Rendered via SocketAddr so an IPv6 address is bracketed
        // (`[::1]:4200`); direct interpolation would emit an invalid URL.
        format!(
            "{}://{}/ws",
            if self.tls_port.is_some() { "wss" } else { "ws" },
            self.noded_addr()
        )
    }
}

/// Mesh configuration loaded from a `mesh.conf.mix` file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshConfig {
    /// This node's name (must match what other peers call us).
    pub node_name: String,
    /// Known peers on the mesh.
    #[serde(default)]
    pub peers: Vec<PeerConfig>,
    /// SPEC 13 §9a D2 admission seed (2-c-1c) — this node's `kind:"d2"` private
    /// seed, used by the bridge prover to sign the admission transcript. NOT in
    /// `mesh.conf.mix`; injected at startup by noded from
    /// `mix:etc/noded/d2.seed`. `None` ⇒ the node cannot prove itself
    /// (prover-incapable; the broker logs `unproven:no-proof`).
    #[serde(skip)]
    pub d2_seed: Option<crate::seed::D2Seed>,
}

impl MeshConfig {
    /// Load mesh config from an explicit `mesh.conf.mix` file path (the
    /// `noded --mesh-config` path).
    ///
    /// A read or parse failure is a hard error.
    pub fn load(path: &str) -> Result<Self> {
        Ok(config::load_conf_mix_path::<Self>(std::path::Path::new(
            path,
        ))?)
    }

    /// Load `mesh.conf.mix` from the default directory
    /// (`~/.config/mix/`), or return an empty config. Best-effort: a
    /// present-but-broken config warns and falls back to empty rather
    /// than silently masking it with a different file.
    pub fn load_default(node_name: &str) -> Self {
        let dir = config::path(config::Dir::Etc);
        let chosen = Some(dir.join("mesh.conf.mix")).filter(|p| p.exists());

        if let Some(path) = chosen {
            match config::load_conf_mix_path::<Self>(&path) {
                Ok(c) => return c,
                Err(e) => tracing::warn!(
                    error = %e,
                    path = %path.display(),
                    "Failed to load mesh config, using empty"
                ),
            }
        }

        Self {
            node_name: node_name.to_string(),
            peers: Vec::new(),
            d2_seed: None,
        }
    }

    /// Find a peer by node name.
    pub fn find_peer(&self, node_name: &str) -> Option<&PeerConfig> {
        self.peers.iter().find(|p| p.name == node_name)
    }
}

/// SPEC 13 §9a (2-c-1c) — build a signed `noded.admit.response` for a received
/// `noded.admit.challenge`. `claimed_source_node` is this node's BARE name
/// (e.g. `delta`), NOT the `bridge-<node>` label (the `from` header keeps the
/// label; `admit()`'s `NameMismatch` binds the bare name). Origin-only:
/// `client_ephemeral`/`channel_binding_hash` are 32 zero bytes, built from the
/// challenge body's DECODED raw bytes / native-`u64` epoch (the byte-for-byte
/// contract the broker's `reconstruct_proof` mirrors). Returns `None` if there
/// is no d2 seed (prover-incapable), the challenge is malformed, or its
/// verifier/mesh differs from the caller's trusted expectation. Never
/// derive these expectations from the received challenge. The caller
/// then registers without proving (the broker logs `unproven:no-proof`).
pub fn build_admit_response(
    challenge: &BusMessage,
    node_name: &str,
    d2_seed: &Option<crate::seed::D2Seed>,
    expected_verifier: &str,
    expected_mesh: &str,
    local_epoch: u64,
) -> Option<String> {
    build_bound_admit_response(
        challenge,
        node_name,
        d2_seed,
        expected_verifier,
        expected_mesh,
        local_epoch,
        &[0; 32],
    )
}

pub fn build_bound_admit_response(
    challenge: &BusMessage,
    node_name: &str,
    d2_seed: &Option<crate::seed::D2Seed>,
    expected_verifier: &str,
    expected_mesh: &str,
    local_epoch: u64,
    binding: &[u8; 32],
) -> Option<String> {
    let seed = d2_seed.as_ref()?;
    let id = challenge.get("id")?.to_string();
    let body: serde_json::Value = serde_json::from_str(&challenge.body).ok()?;
    if body["verifying_broker_node"].as_str() != Some(expected_verifier)
        || body["mesh_fqdn"].as_str() != Some(expected_mesh)
    {
        challenge_diagnostic(true, expected_verifier, expected_mesh, local_epoch, &body);
        return None;
    }
    let challenge_epoch = body.get("inventory_epoch")?.as_u64()?;
    CHALLENGE_EPOCHS
        .lock()
        .expect("challenge epochs poisoned")
        .insert(expected_verifier.to_owned(), challenge_epoch);
    // The verifier owns the nonce and checks the proof against its own
    // authority. A rolling inventory publish must not strand valid members.
    if challenge_epoch != local_epoch {
        challenge_diagnostic(false, expected_verifier, expected_mesh, local_epoch, &body);
    }
    let b64 = base64::engine::general_purpose::STANDARD;
    let transcript = AdmissionTranscript {
        mesh_fqdn: body.get("mesh_fqdn")?.as_str()?.to_string(),
        claimed_source_node: node_name.to_string(),
        verifying_broker_node: body.get("verifying_broker_node")?.as_str()?.to_string(),
        inventory_epoch: body.get("inventory_epoch")?.as_u64()?,
        session_id: b64.decode(body.get("session_id")?.as_str()?).ok()?,
        server_nonce: b64.decode(body.get("server_nonce")?.as_str()?).ok()?,
        client_ephemeral: vec![0u8; 32],
        channel_binding_hash: binding.to_vec(),
    };
    let sig = sign_admission_transcript(seed.bytes(), &transcript).ok()?;
    let mut resp = BusMessage::new()
        .with_header("command", "noded.admit.response")
        .with_header("type", "response")
        .with_header("from", &format!("bridge-{node_name}"))
        .with_header("to", "noded")
        .with_header("id", &id);
    resp.body = serde_json::json!({
        "claimed_source_node": node_name,
        "signed_epoch": transcript.inventory_epoch,
        "signature": b64.encode(sig),
        "client_ephemeral": b64.encode([0u8; 32]),
        "channel_binding_hash": b64.encode(binding),
    })
    .to_string();
    resp.originate_mesh(node_name);
    // Bootstrap carries the bridge registration label, not an application
    // from-address. The receiver consumes it before the all-frame gate.
    resp.set("from", &format!("bridge-{node_name}"));
    Some(resp.to_wire())
}

const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(30);
const ADMISSION_CHALLENGE_TIMEOUT: Duration = Duration::from_secs(1);

// Process-wide state survives reconnects. Keys come from trusted peer config,
// never from received challenge fields.
static CHALLENGE_REFUSALS: std::sync::LazyLock<Mutex<HashMap<String, DiagnosticLog>>> =
    std::sync::LazyLock::new(Default::default);
static CHALLENGE_SKEW: std::sync::LazyLock<Mutex<HashMap<String, DiagnosticLog>>> =
    std::sync::LazyLock::new(Default::default);
static CHALLENGE_EPOCHS: std::sync::LazyLock<Mutex<HashMap<String, u64>>> =
    std::sync::LazyLock::new(Default::default);

/// Last matching admission challenge epoch observed from this configured peer.
/// Diagnostic only: it is not an authenticated inventory or routing authority.
pub fn peer_challenge_epoch(peer: &str) -> Option<u64> {
    CHALLENGE_EPOCHS
        .lock()
        .expect("challenge epochs poisoned")
        .get(peer)
        .copied()
}

/// Lifetime refused-challenge total, including suppressed diagnostics.
pub fn challenge_refusal_total() -> u64 {
    CHALLENGE_REFUSALS
        .lock()
        .expect("challenge log counter poisoned")
        .values()
        .map(|log| log.total)
        .sum()
}

/// Monotonic, reconnect-independent diagnostic coalescing state.
#[derive(Default)]
pub struct DiagnosticLog {
    last: Option<std::time::Instant>,
    total: u64,
    pending: u64,
}

impl DiagnosticLog {
    /// Record every occurrence; emit first, then at most once per minute.
    /// Returns (interval count, lifetime count) when a line is due.
    pub fn record(&mut self, now: std::time::Instant) -> Option<(u64, u64)> {
        self.total += 1;
        self.pending += 1;
        if self
            .last
            .is_some_and(|last| now.duration_since(last) < Duration::from_secs(60))
        {
            return None;
        }
        self.last = Some(now);
        Some((std::mem::take(&mut self.pending), self.total))
    }

    /// Includes a trailing burst that has not produced another log line.
    pub fn total(&self) -> u64 {
        self.total
    }
}

fn challenge_diagnostic(
    refused: bool,
    peer: &str,
    mesh: &str,
    local_epoch: u64,
    body: &serde_json::Value,
) {
    let counter = if refused {
        &CHALLENGE_REFUSALS
    } else {
        &CHALLENGE_SKEW
    };
    let now = std::time::Instant::now();
    let emission = counter
        .lock()
        .expect("challenge log counter poisoned")
        .entry(peer.to_owned())
        .or_default()
        .record(now);
    let Some((count, total)) = emission else {
        return;
    };
    if refused {
        tracing::warn!(
            peer,
            mesh,
            local_epoch,
            received_verifier = body["verifying_broker_node"].as_str(),
            received_mesh = body["mesh_fqdn"].as_str(),
            count,
            total,
            "Refusing admission challenge with unexpected verifier or mesh (coalesced)"
        );
    } else {
        tracing::debug!(
            peer,
            mesh,
            local_epoch,
            challenge_epoch = body["inventory_epoch"].as_u64(),
            count,
            total,
            "Answering admission challenge despite inventory epoch skew (coalesced)"
        );
    }
}

static UNAUTHENTICATED_FRAMES: AtomicU64 = AtomicU64::new(0);

/// Lifetime total, including observe-mode frames delivered for compatibility.
pub fn unauthenticated_frame_total() -> u64 {
    UNAUTHENTICATED_FRAMES.load(Ordering::Relaxed)
}

#[derive(Default)]
// A burst ending inside the window leaves pending counts unflushed. They
// remain in the lifetime counter; connection totals reset on reconnect.
struct UnauthenticatedFrameLog {
    last: Option<std::time::Instant>,
    total: u64,
    pending: u64,
}

impl UnauthenticatedFrameLog {
    fn record(&mut self, now: std::time::Instant, peer: &str, address: SocketAddr, enforce: bool) {
        UNAUTHENTICATED_FRAMES.fetch_add(1, Ordering::Relaxed);
        self.total += 1;
        self.pending += 1;
        if self
            .last
            .is_none_or(|last| now.duration_since(last) >= Duration::from_secs(60))
        {
            let count = std::mem::take(&mut self.pending);
            self.last = Some(now);
            tracing::info!(peer, %address, posture = if enforce { "enforce" } else { "observe" },
                count, total = self.total, "dialled peer frame lacks authenticated from identity (coalesced)");
        }
    }
}

/// The signed-routing revision whose desired endpoints are installed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct AuthorityRevision {
    pub epoch: u64,
    pub recovery_generation: u64,
}

/// Why an established connection stopped being authorised.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionRetireReason {
    Removed,
    KeyRevoked,
    TlsKeyRevoked,
    UnprovenLinkRetired,
    EndpointChanged { replacement: SocketAddr },
}

impl ConnectionRetireReason {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Removed => "removed",
            Self::KeyRevoked => "exact-key-revoked",
            Self::TlsKeyRevoked => "tls-key-revoked",
            Self::UnprovenLinkRetired => "unproven-link-retired",
            Self::EndpointChanged { .. } => "endpoint-changed",
        }
    }
}

/// An established transport retired by endpoint reconciliation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetiredConnection {
    pub peer: String,
    pub endpoint: SocketAddr,
    pub connection_generation: uuid::Uuid,
    pub reason: ConnectionRetireReason,
}

/// An outbound RPC failed by endpoint reconciliation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetiredInflight {
    pub reason: ConnectionRetireReason,
    pub peer: String,
    pub connection_generation: uuid::Uuid,
    pub message_id: String,
    pub class: String,
}

/// The complete, synchronous effect of installing a desired endpoint set.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReconcileReport {
    pub added: usize,
    pub removed: usize,
    pub endpoint_changed: usize,
    pub connections_retired: Vec<RetiredConnection>,
    pub connections_retained: usize,
    pub inflight_failed: Vec<RetiredInflight>,
}

/// An unsolicited frame received from a particular transport generation.
#[derive(Debug)]
pub struct MeshInbound {
    pub context: bus::native_session::PeerContext,
    pub peer: String,
    pub connection_generation: uuid::Uuid,
    pub message: BusMessage,
}

/// Holds the transport read fence through the caller's final local enqueue.
pub struct MeshInboundGuard<'a> {
    _state: RwLockReadGuard<'a, TransportState>,
}

/// A connection to a remote broker with send/receive channels.
struct RemoteBroker {
    context: bus::native_session::PeerContext,
    /// The exact resolved transport target this connection owns. A later
    /// inventory epoch may retain the name while changing its mesh address;
    /// such a call must not reuse this connection.
    target: SocketAddr,
    /// Distinguishes successive connections to the same named target so an old
    /// reader cannot remove a newer cache entry when it finally exits.
    generation: uuid::Uuid,
    tx: mpsc::UnboundedSender<String>,
    connected: Arc<AtomicBool>,
    cancel: watch::Sender<bool>,
}

impl RemoteBroker {
    fn retire(&self) {
        self.connected.store(false, Ordering::Release);
        self.cancel.send_replace(true);
    }
}

struct PendingCall {
    peer: String,
    connection_generation: uuid::Uuid,
    class: String,
    response: oneshot::Sender<std::result::Result<BusMessage, String>>,
}

/// A call future can be dropped at any await. Removing its entry synchronously
/// keeps cancellation bounded, without spawning a second cleanup task.
struct PendingGuard {
    pending: Arc<Mutex<HashMap<String, PendingCall>>>,
    id: String,
    generation: uuid::Uuid,
}

impl Drop for PendingGuard {
    fn drop(&mut self) {
        let mut pending = self
            .pending
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if pending
            .get(&self.id)
            .is_some_and(|call| call.connection_generation == self.generation)
        {
            pending.remove(&self.id);
        }
    }
}

#[derive(Clone)]
enum AttemptOutcome {
    Pending,
    Complete(std::result::Result<Arc<RemoteBroker>, Arc<str>>),
}

struct ConnectAttempt {
    id: uuid::Uuid,
    target: SocketAddr,
    cancel: watch::Sender<bool>,
    outcome: watch::Receiver<AttemptOutcome>,
}

struct TransportState {
    tls: Option<Arc<TlsRuntime>>,
    tls_keys: Arc<mesh_trust::identity::TlsAuthority>,
    frame_enforce: bool,
    keys: Option<(mesh_trust::identity::KeyAuthority, String)>,
    desired: HashMap<String, SocketAddr>,
    revision: AuthorityRevision,
    connections: HashMap<String, Arc<RemoteBroker>>,
    attempts: HashMap<String, Arc<ConnectAttempt>>,
}

impl TransportState {
    fn authenticates(&self, context: &bus::native_session::PeerContext) -> bool {
        context.mesh_peer().is_some_and(|(node, key)| {
            self.keys.as_ref().is_some_and(|(keys, _)| {
                let _ = (node, key);
                mesh_trust::identity::link_refusal(context, keys, &self.tls_keys).is_none()
            })
        })
    }
}

pub struct TlsRuntime {
    pub credential: crate::tls::Credential,
    pub authority: crate::tls::LiveAuthority,
}

/// Manages connections to remote mesh peers.
///
/// Used by noded to bridge messages to remote nodes.
pub struct MeshPeers {
    config: MeshConfig,
    /// Desired endpoints, active connections and shared attempts have one
    /// linearisation point. Network I/O never runs while this lock is held.
    state: Arc<RwLock<TransportState>>,
    /// Pending responses: message_id → oneshot sender for the response
    pending: Arc<Mutex<HashMap<String, PendingCall>>>,
    /// Channel to deliver incoming messages from remote brokers back to local broker
    incoming_tx: mpsc::UnboundedSender<MeshInbound>,
}

impl MeshPeers {
    /// Create a new MeshPeers manager.
    ///
    /// `incoming_tx` receives messages from remote brokers that need to be
    /// delivered to local services.
    pub fn new(config: MeshConfig, incoming_tx: mpsc::UnboundedSender<MeshInbound>) -> Self {
        let desired = config
            .peers
            .iter()
            .map(|peer| (peer.name.clone(), peer.noded_addr()))
            .collect();
        Self {
            config,
            state: Arc::new(RwLock::new(TransportState {
                tls: None,
                tls_keys: Default::default(),
                frame_enforce: false,
                keys: None,
                desired,
                revision: AuthorityRevision::default(),
                connections: HashMap::new(),
                attempts: HashMap::new(),
            })),
            pending: Arc::new(Mutex::new(HashMap::new())),
            incoming_tx,
        }
    }

    /// Get this node's name.
    pub async fn set_frame_enforce(&self, enforce: bool) {
        self.state.write().await.frame_enforce = enforce;
    }

    pub async fn set_tls(&self, tls: Arc<TlsRuntime>) {
        self.state.write().await.tls = Some(tls);
    }

    pub async fn peer_context(&self, peer: &str) -> Option<bus::native_session::PeerContext> {
        self.state
            .read()
            .await
            .connections
            .get(peer)
            .map(|link| link.context.clone())
    }

    pub fn node_name(&self) -> &str {
        &self.config.node_name
    }

    /// Atomically install the route authority's desired endpoint projection.
    /// Established transports and in-progress attempts which no longer match
    /// are fenced before this method returns.
    pub async fn reconcile_identity(
        &self,
        desired: HashMap<String, SocketAddr>,
        revision: AuthorityRevision,
        keys: mesh_trust::identity::KeyAuthority,
        mesh: String,
    ) -> ReconcileReport {
        self.reconcile(desired, revision, Some((keys, mesh)), Default::default())
            .await
    }

    pub async fn reconcile_endpoints(
        &self,
        desired: HashMap<String, SocketAddr>,
        revision: AuthorityRevision,
    ) -> ReconcileReport {
        self.reconcile(desired, revision, None, Default::default())
            .await
    }

    pub async fn reconcile_security(
        &self,
        desired: HashMap<String, SocketAddr>,
        revision: AuthorityRevision,
        keys: mesh_trust::identity::KeyAuthority,
        mesh: String,
        tls_keys: Arc<mesh_trust::identity::TlsAuthority>,
    ) -> ReconcileReport {
        self.reconcile(desired, revision, Some((keys, mesh)), tls_keys)
            .await
    }

    async fn reconcile(
        &self,
        desired: HashMap<String, SocketAddr>,
        revision: AuthorityRevision,
        keys: Option<(mesh_trust::identity::KeyAuthority, String)>,
        tls_keys: Arc<mesh_trust::identity::TlsAuthority>,
    ) -> ReconcileReport {
        let mut state = self.state.write().await;
        let mut report = ReconcileReport::default();
        let mut reasons = HashMap::new();

        for (name, old_endpoint) in &state.desired {
            match desired.get(name) {
                None => {
                    report.removed += 1;
                    reasons.insert(name.clone(), ConnectionRetireReason::Removed);
                }
                Some(new_endpoint) if new_endpoint != old_endpoint => {
                    report.endpoint_changed += 1;
                    reasons.insert(
                        name.clone(),
                        ConnectionRetireReason::EndpointChanged {
                            replacement: *new_endpoint,
                        },
                    );
                }
                Some(_) => {}
            }
        }
        if let Some((authority, _)) = &keys {
            for (name, connection) in &state.connections {
                if let Some((node, key)) = connection.context.mesh_peer() {
                    let _ = (node, key);
                    if let Some(credential) = mesh_trust::identity::link_refusal(
                        &connection.context,
                        authority,
                        &tls_keys,
                    ) {
                        tracing::info!(peer = %name, credential, "outbound link credential revoked");
                        reasons.insert(
                            name.clone(),
                            if credential == "tls-key-revoked" {
                                ConnectionRetireReason::TlsKeyRevoked
                            } else {
                                ConnectionRetireReason::KeyRevoked
                            },
                        );
                    }
                } else if authority.epoch() > 0 {
                    reasons
                        .entry(name.clone())
                        .or_insert(ConnectionRetireReason::UnprovenLinkRetired);
                }
            }
            if state.revision != revision {
                for (_, attempt) in state.attempts.drain() {
                    attempt.cancel.send_replace(true);
                }
            }
        }
        state.keys = keys;
        state.tls_keys = tls_keys;
        report.added = desired
            .keys()
            .filter(|name| !state.desired.contains_key(*name))
            .count();

        for (name, reason) in &reasons {
            if let Some(connection) = state.connections.remove(name) {
                connection.retire();
                report.connections_retired.push(RetiredConnection {
                    peer: name.clone(),
                    endpoint: connection.target,
                    connection_generation: connection.generation,
                    reason: reason.clone(),
                });
            }
            if let Some(attempt) = state.attempts.remove(name) {
                attempt.cancel.send_replace(true);
            }
        }

        state.desired = desired;
        state.revision = revision;
        report.connections_retained = state
            .connections
            .iter()
            .filter(|(name, connection)| {
                state.desired.get(*name) == Some(&connection.target)
                    && connection.connected.load(Ordering::Acquire)
            })
            .count();

        let retired_generations: HashMap<_, _> = report
            .connections_retired
            .iter()
            .map(|retired| (retired.connection_generation, retired.reason.clone()))
            .collect();

        // Sweep pending calls while STILL holding the state write guard: a
        // retired reader's exit path takes the state write lock (in
        // `remove_connection_generation`) before its own pending sweep, so
        // holding the guard here guarantees the reload's "route revoked"
        // verdict — and its `inflight_failed` record — wins that race
        // instead of a generic "connection closed" with no report entry.
        // Lock order state → pending matches every other nesting site.
        if !retired_generations.is_empty() {
            let mut pending = self.pending.lock().expect("pending calls lock poisoned");
            let message_ids: Vec<_> = pending
                .iter()
                .filter(|(_, call)| retired_generations.contains_key(&call.connection_generation))
                .map(|(id, _)| id.clone())
                .collect();
            for message_id in message_ids {
                let Some(call) = pending.remove(&message_id) else {
                    continue;
                };
                report.inflight_failed.push(RetiredInflight {
                    reason: retired_generations[&call.connection_generation].clone(),
                    peer: call.peer.clone(),
                    connection_generation: call.connection_generation,
                    message_id,
                    class: call.class,
                });
                let _ = call.response.send(Err(format!(
                    "route revoked by endpoint reconciliation: {}",
                    retired_generations[&call.connection_generation].as_str()
                )));
            }
        }

        report
    }

    /// Acquire the final-delivery fence for a queued inbound frame. `None`
    /// means the frame's connection has been retired or replaced.
    pub async fn validate_inbound(
        &self,
        peer: &str,
        connection_generation: uuid::Uuid,
    ) -> Option<MeshInboundGuard<'_>> {
        let state = self.state.read().await;
        let valid = state.connections.get(peer).is_some_and(|connection| {
            connection.generation == connection_generation
                && connection.connected.load(Ordering::Acquire)
                && state.desired.get(peer) == Some(&connection.target)
        });
        valid.then_some(MeshInboundGuard { _state: state })
    }

    /// Cheap early rejection before doing local registry work. The final
    /// enqueue must still use `validate_inbound` to close the dequeue race.
    pub async fn inbound_generation_is_current(
        &self,
        peer: &str,
        connection_generation: uuid::Uuid,
    ) -> bool {
        self.validate_inbound(peer, connection_generation)
            .await
            .is_some()
    }

    /// Send a message to a remote node's broker. Returns the response.
    ///
    /// The caller supplies the already-authorised, resolved peer. Membership
    /// policy belongs to noded's routing snapshot; this type owns transport and
    /// connection reuse only.
    pub async fn call(&self, peer: PeerConfig, msg: BusMessage) -> Result<BusMessage> {
        let node_name = peer.name.clone();
        let connection = self.ensure_connected(&peer).await?;

        // Client IDs are scoped to their own connections. Give this hop a
        // fresh correlation ID so two clients both using "1" cannot collide.
        let caller_id = msg.get("id").map(str::to_owned);
        let msg_id = uuid::Uuid::new_v4().to_string();
        let _pending_guard = PendingGuard {
            pending: self.pending.clone(),
            id: msg_id.clone(),
            generation: connection.generation,
        };

        // Send the message with the id
        let mut msg = msg;
        msg.originate_mesh(&self.config.node_name);
        let caller_request_id = msg.get("request_id").map(str::to_owned);
        msg.forward_mesh().map_err(anyhow::Error::msg)?;
        msg.set("id", &msg_id);
        let class = msg.command_name().unwrap_or("unknown").to_string();
        let (resp_tx, resp_rx) = oneshot::channel();
        self.enqueue_call(
            &peer,
            &connection,
            msg.to_wire(),
            msg_id.clone(),
            class,
            resp_tx,
        )
        .await?;

        // Wait for response with timeout
        match tokio::time::timeout(RESPONSE_TIMEOUT, resp_rx).await {
            Ok(Ok(Ok(mut resp))) => {
                if let Some(request_id) = caller_request_id {
                    resp.set("request_id", &request_id);
                }
                if let Some(caller_id) = caller_id {
                    resp.set("id", &caller_id);
                } else {
                    resp.headers.remove("id");
                }
                Ok(resp)
            }
            Ok(Ok(Err(reason))) => anyhow::bail!("Call to {node_name} failed: {reason}"),
            Ok(Err(_)) => {
                self.remove_pending(&msg_id, connection.generation).await;
                anyhow::bail!("Response channel for {node_name} closed")
            }
            Err(_) => {
                self.remove_pending(&msg_id, connection.generation).await;
                anyhow::bail!("Timeout waiting for response from {node_name}")
            }
        }
    }

    /// Send a fire-and-forget message to a remote node.
    pub async fn send(&self, peer: PeerConfig, msg: BusMessage) -> Result<()> {
        let mut msg = msg;
        msg.originate_mesh(&self.config.node_name);
        msg.forward_mesh().map_err(anyhow::Error::msg)?;
        let node_name = peer.name.clone();
        let connection = self.ensure_connected(&peer).await?;
        let state = self.state.read().await;
        self.validate_outbound_locked(&state, &peer, &connection)?;
        connection
            .tx
            .send(msg.to_wire())
            .map_err(|_| anyhow::anyhow!("Send to {node_name} failed"))?;

        Ok(())
    }

    async fn enqueue_call(
        &self,
        peer: &PeerConfig,
        connection: &Arc<RemoteBroker>,
        wire: String,
        message_id: String,
        class: String,
        response: oneshot::Sender<std::result::Result<BusMessage, String>>,
    ) -> Result<()> {
        let state = self.state.read().await;
        self.validate_outbound_locked(&state, peer, connection)?;
        let mut pending = self.pending.lock().expect("pending calls lock poisoned");
        if pending.contains_key(&message_id) {
            anyhow::bail!("Duplicate pending message id {message_id}");
        }
        pending.insert(
            message_id.clone(),
            PendingCall {
                peer: peer.name.clone(),
                connection_generation: connection.generation,
                class,
                response,
            },
        );
        if connection.tx.send(wire).is_err() {
            pending.remove(&message_id);
            anyhow::bail!("Send to {} failed", peer.name);
        }
        Ok(())
    }

    fn validate_outbound_locked(
        &self,
        state: &TransportState,
        peer: &PeerConfig,
        connection: &Arc<RemoteBroker>,
    ) -> Result<()> {
        let target = peer.noded_addr();
        let current = state.connections.get(&peer.name);
        if state.desired.get(&peer.name) != Some(&target)
            || current.is_none_or(|cached| cached.generation != connection.generation)
            || connection.target != target
            || !connection.connected.load(Ordering::Acquire)
        {
            anyhow::bail!("Route to {} is no longer authorised", peer.name);
        }
        Ok(())
    }

    async fn remove_pending(&self, message_id: &str, generation: uuid::Uuid) {
        // Take the state lock (read) before pending: during a reload's
        // critical section this serialises the caller's own timeout cleanup
        // AFTER reconcile's sweep, so a call retired at the fence always
        // gets its `inflight_failed` record even when its RESPONSE_TIMEOUT
        // expires in the same instant (the sweep's send to the caller's
        // already-dropped oneshot is harmless). Lock order state → pending,
        // as at every other nesting site.
        let _state = self.state.read().await;
        let mut pending = self.pending.lock().expect("pending calls lock poisoned");
        if pending
            .get(message_id)
            .is_some_and(|call| call.connection_generation == generation)
        {
            pending.remove(message_id);
        }
    }

    /// Ensure we have an active WebSocket connection to a peer's broker.
    async fn ensure_connected(&self, peer: &PeerConfig) -> Result<Arc<RemoteBroker>> {
        let target = peer.noded_addr();
        if let Some(connection) = self.cached_connection(peer).await {
            return Ok(connection);
        }

        let attempt = {
            let mut state = self.state.write().await;
            if state.desired.get(&peer.name) != Some(&target) {
                anyhow::bail!("Route to {} is no longer authorised", peer.name);
            }
            if let Some(connection) = state.connections.get(&peer.name).filter(|connection| {
                connection.target == target && connection.connected.load(Ordering::Acquire)
            }) {
                return Ok(connection.clone());
            }
            if let Some(attempt) = state
                .attempts
                .get(&peer.name)
                .filter(|attempt| attempt.target == target)
            {
                attempt.clone()
            } else {
                if let Some(stale) = state.attempts.remove(&peer.name) {
                    stale.cancel.send_replace(true);
                }
                let id = uuid::Uuid::new_v4();
                let (cancel, cancel_rx) = watch::channel(false);
                let (outcome_tx, outcome) = watch::channel(AttemptOutcome::Pending);
                let attempt = Arc::new(ConnectAttempt {
                    id,
                    target,
                    cancel,
                    outcome,
                });
                state.attempts.insert(peer.name.clone(), attempt.clone());
                self.spawn_connect_attempt(peer.clone(), id, cancel_rx, outcome_tx);
                attempt
            }
        };
        let result = wait_for_attempt(attempt.outcome.clone()).await;
        if result.is_err() {
            // A driver that died without publishing an outcome (panic/abort)
            // leaves its map entry behind, and every future caller would join
            // the corpse forever — reconcile only clears attempts for
            // removed/changed endpoints. Sweep it (id-guarded, so a fresh
            // replacement attempt is never touched); on a normally-failed
            // attempt this just races the driver's own idempotent cleanup.
            let mut state = self.state.write().await;
            if state
                .attempts
                .get(&peer.name)
                .is_some_and(|current| current.id == attempt.id)
            {
                state.attempts.remove(&peer.name);
            }
        }
        result
    }

    fn spawn_connect_attempt(
        &self,
        peer: PeerConfig,
        attempt_id: uuid::Uuid,
        cancel: watch::Receiver<bool>,
        outcome: watch::Sender<AttemptOutcome>,
    ) {
        let node_name = self.config.node_name.clone();
        let d2_seed = self.config.d2_seed.clone();
        let state = self.state.clone();
        let pending = self.pending.clone();
        let incoming_tx = self.incoming_tx.clone();
        tokio::spawn(async move {
            let result = connect_and_publish(
                peer.clone(),
                attempt_id,
                cancel,
                node_name,
                d2_seed,
                state.clone(),
                pending,
                incoming_tx,
            )
            .await
            .map_err(|error| Arc::<str>::from(error.to_string()));
            outcome.send_replace(AttemptOutcome::Complete(result));
            let mut state = state.write().await;
            if state
                .attempts
                .get(&peer.name)
                .is_some_and(|attempt| attempt.id == attempt_id)
            {
                state.attempts.remove(&peer.name);
            }
        });
    }

    async fn cached_connection(&self, peer: &PeerConfig) -> Option<Arc<RemoteBroker>> {
        let target = peer.noded_addr();
        let state = self.state.read().await;
        if state.desired.get(&peer.name) != Some(&target) {
            return None;
        }
        state
            .connections
            .get(&peer.name)
            .filter(|connection| {
                connection.target == target && connection.connected.load(Ordering::Acquire)
            })
            .cloned()
    }
}

async fn wait_for_attempt(
    mut outcome: watch::Receiver<AttemptOutcome>,
) -> Result<Arc<RemoteBroker>> {
    loop {
        match outcome.borrow().clone() {
            AttemptOutcome::Pending => {}
            AttemptOutcome::Complete(Ok(connection)) => return Ok(connection),
            AttemptOutcome::Complete(Err(error)) => anyhow::bail!(error.to_string()),
        }
        if outcome.changed().await.is_err() {
            anyhow::bail!("Shared connection attempt ended without a result");
        }
    }
}

trait PeerIo: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send {}
impl<T: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send> PeerIo for T {}

async fn dial(
    peer: &PeerConfig,
    tls: Option<Arc<TlsRuntime>>,
) -> Result<(
    tokio_tungstenite::WebSocketStream<Box<dyn PeerIo>>,
    Option<(mesh_trust::identity::TlsMember, [u8; 32])>,
)> {
    let tcp = tokio::net::TcpStream::connect(peer.noded_addr()).await?;
    tcp.set_nodelay(true)?;
    let (io, identity): (Box<dyn PeerIo>, _) = if peer.tls_port.is_some() {
        let tls =
            tls.ok_or_else(|| anyhow::anyhow!("TLS peer configured without TLS credential"))?;
        let config = tls
            .credential
            .client_config(tls.authority.clone(), peer.name.clone())?;
        let stream = tokio_rustls::TlsConnector::from(config)
            .connect(
                rustls::pki_types::ServerName::try_from("bus.example.invalid")?,
                tcp,
            )
            .await?;
        let conn = stream.get_ref().1;
        let member = crate::tls::verified_member(
            &tls.authority,
            conn.peer_certificates().unwrap_or(&[]),
            Some(&peer.name),
        )?;
        let binding = crate::tls::client_channel_binding(conn)?;
        (Box::new(stream), Some((member, binding)))
    } else {
        (Box::new(tcp), None)
    };
    let (ws, _) = tokio_tungstenite::client_async_with_config(peer.noded_url(), io, None).await?;
    Ok((ws, identity))
}

#[allow(clippy::too_many_arguments)]
async fn connect_and_publish(
    peer: PeerConfig,
    attempt_id: uuid::Uuid,
    mut attempt_cancel: watch::Receiver<bool>,
    node_name: String,
    d2_seed: Option<crate::seed::D2Seed>,
    state: Arc<RwLock<TransportState>>,
    pending: Arc<Mutex<HashMap<String, PendingCall>>>,
    incoming_tx: mpsc::UnboundedSender<MeshInbound>,
) -> Result<Arc<RemoteBroker>> {
    let target = peer.noded_addr();
    let url = peer.noded_url();
    tracing::info!(peer = %peer.name, url = %url, "Connecting to remote broker");

    // Nagle off on this end too: TCP_NODELAY is per socket, so the remote
    // broker's accept hook cannot cover the side that dialled.
    let tls = state.read().await.tls.clone();
    let connect = tokio::time::timeout(CONNECT_TIMEOUT, dial(&peer, tls));
    let (ws_stream, tls_peer) = tokio::select! {
        biased;
        changed = attempt_cancel.changed() => {
            let _ = changed;
            anyhow::bail!("Route to {} was revoked during connection", peer.name);
        }
        result = connect => match result {
            Err(_) => {
                tracing::warn!(
                    event = "mesh.connection.failed",
                    cause = "connect-timeout",
                    peer = %peer.name,
                    endpoint = %target,
                    timeout_ms = CONNECT_TIMEOUT.as_millis() as u64,
                );
                anyhow::bail!("Connection to {} timed out after {}ms", peer.name, CONNECT_TIMEOUT.as_millis());
            }
            Ok(Err(error)) => {
                if peer.tls_port.is_some() { crate::tls::DIAL_REFUSALS.record(target, "dial-or-pin"); }
                anyhow::bail!("Failed to connect to {}: {error}", peer.name)
            },
            Ok(Ok(connected)) => connected,
        }
    };

    let (mut ws_sink, mut ws_stream_rx) = ws_stream.split();
    let mut early_inbound = None;

    // SPEC 13 §9a (2-c-1c) — D2 admission prover. A 2-c broker's FIRST frame
    // is a `noded.admit.challenge`; respond with a signed transcript BEFORE
    // registering. A pre-2c/`off` broker sends nothing first → the bounded
    // read times out and we register-first (back-compat). Only an exact
    // challenge command is consumed here; any other first frame is delivered
    // to the normal inbound path so it is not lost. A late challenge (after
    // the timeout) is handled harmlessly by the reader (delivered, ignored)
    // — the session just goes unproven (the broker logs `no-proof`).
    let first_frame = tokio::select! {
        biased;
        changed = attempt_cancel.changed() => {
            let _ = changed;
            anyhow::bail!("Route to {} was revoked during admission", peer.name);
        }
        frame = tokio::time::timeout(ADMISSION_CHALLENGE_TIMEOUT, ws_stream_rx.next()) => frame,
    };
    match first_frame {
        // Timed out — a silent (pre-2c / `off`) broker. The connection is
        // alive; fall through and register-first (back-compat).
        Err(_) => {}
        // Closed / EOF / ws error before any frame — the connection is dead;
        // return WITHOUT storing a dead `RemoteBroker` (the next
        // `ensure_connected` reconnects).
        Ok(None) | Ok(Some(Err(_))) | Ok(Some(Ok(tungstenite::Message::Close(_)))) => {
            anyhow::bail!("Connection to {} closed during admission", peer.name);
        }
        Ok(Some(Ok(tungstenite::Message::Text(t)))) => {
            if let Ok(bus_msg) = bus::parse(&t) {
                if bus_msg.command_name() == Some("noded.admit.challenge") {
                    let authority = state.read().await.keys.clone();
                    if let Some(resp_wire) = authority.as_ref().and_then(|(keys, mesh)| {
                        build_bound_admit_response(
                            &bus_msg,
                            &node_name,
                            &d2_seed,
                            &peer.name,
                            mesh,
                            keys.epoch(),
                            tls_peer.as_ref().map(|(_, hash)| hash).unwrap_or(&[0; 32]),
                        )
                    }) {
                        // A send failure here means the connection died —
                        // bail rather than store it. The write is bounded and
                        // cancellable like every other await on this path: a
                        // zero-window peer must not wedge the shared attempt
                        // (and every joined caller) past the connect budget.
                        let sent = tokio::select! {
                            biased;
                            changed = attempt_cancel.changed() => {
                                let _ = changed;
                                anyhow::bail!("Route to {} was revoked during admission response", peer.name);
                            }
                            result = tokio::time::timeout(
                                CONNECT_TIMEOUT,
                                ws_sink.send(tungstenite::Message::Text(resp_wire.into())),
                            ) => result,
                        };
                        match sent {
                            Err(_) => anyhow::bail!(
                                "Admission response to {} timed out after {}ms",
                                peer.name,
                                CONNECT_TIMEOUT.as_millis()
                            ),
                            Ok(Err(_)) => anyhow::bail!(
                                "Connection to {} closed during admission response",
                                peer.name
                            ),
                            Ok(Ok(())) => {}
                        }
                    }
                } else {
                    // Not a challenge — deliver as a normal inbound frame.
                    early_inbound = Some(bus_msg);
                }
            }
        }
        // A non-text first frame (ping/pong/binary) — not a challenge, not a
        // close; proceed to register-first.
        Ok(Some(Ok(_))) => {}
    }

    let mut context = bus::native_session::PeerContext {
        transport: None,
        identity: Default::default(),
        address: Some(target),
    };
    let authority = state.read().await.keys.clone();
    if let Some((keys, mesh)) = authority {
        let id = uuid::Uuid::new_v4().to_string();
        let b64 = base64::engine::general_purpose::STANDARD;
        let mut nonce = uuid::Uuid::new_v4().as_bytes().to_vec();
        nonce.extend_from_slice(uuid::Uuid::new_v4().as_bytes());
        let body = serde_json::json!({
            "mesh_fqdn": mesh, "inventory_epoch": keys.epoch(),
            "verifying_broker_node": node_name, "session_id": b64.encode(uuid::Uuid::new_v4().as_bytes()),
            "server_nonce": b64.encode(nonce),
        });
        let challenge = BusMessage::new()
            .with_header("command", "noded.identity.challenge")
            .with_header("id", &id)
            .with_body(&body.to_string());
        let exchange = async {
            ws_sink
                .send(tungstenite::Message::Text(challenge.to_wire().into()))
                .await
                .ok()?;
            loop {
                let tungstenite::Message::Text(wire) = ws_stream_rx.next().await?.ok()? else {
                    continue;
                };
                let response = bus::parse(&wire).ok()?;
                if response.command_name() != Some("noded.identity.response")
                    || response.get("id") != Some(id.as_str())
                {
                    continue;
                }
                return keys.authenticate_bound_response(
                    &body,
                    &serde_json::from_str(&response.body).ok()?,
                    &peer.name,
                    tls_peer.as_ref().map(|(_, hash)| hash),
                );
            }
        };
        tokio::select! {
            biased;
            _ = attempt_cancel.changed() => anyhow::bail!("Route revoked during peer proof"),
            result = tokio::time::timeout(ADMISSION_CHALLENGE_TIMEOUT, exchange) => {
                if let Ok(Some(identity)) = result { context.identity = identity; }
            }
        }
    }
    if let Some((member, _)) = tls_peer {
        let Some((node, d2_key)) = context.mesh_peer() else {
            anyhow::bail!("TLS peer did not pass exporter-bound D2 proof");
        };
        anyhow::ensure!(node == member.member, "TLS/D2 member mismatch");
        context.transport = Some(bus::native_session::TransportIdentity::MeshTls {
            member: member.member,
            tls_key: member.key,
            d2_key: *d2_key,
            epoch: member.epoch,
            revision: member.revision,
            source: target,
        });
    }
    // Enforce gates egress before even registration is written. Off/observe
    // retain the bounded legacy fallback for old brokers without reverse proof.
    {
        let locked = state.read().await;
        anyhow::ensure!(
            !locked.frame_enforce || locked.authenticates(&context),
            "Dialled peer did not prove its identity"
        );
    }
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    let connected = Arc::new(AtomicBool::new(true));
    let generation = uuid::Uuid::new_v4();
    let (connection_cancel, mut writer_cancel) = watch::channel(false);
    let mut reader_cancel = connection_cancel.subscribe();
    let connection = Arc::new(RemoteBroker {
        context: context.clone(),
        target,
        generation,
        tx,
        connected: connected.clone(),
        cancel: connection_cancel,
    });

    let mut register_msg = BusMessage::new()
        .with_header("command", "noded.register")
        .with_header("from", &format!("bridge-{node_name}"))
        .with_header("to", "noded");
    register_msg.originate_mesh(&node_name);
    register_msg.set("from", &format!("bridge-{node_name}"));
    tokio::select! {
        biased;
        changed = attempt_cancel.changed() => {
            let _ = changed;
            anyhow::bail!("Route to {} was revoked before registration", peer.name);
        }
        result = tokio::time::timeout(
            CONNECT_TIMEOUT,
            ws_sink.send(tungstenite::Message::Text(register_msg.to_wire().into())),
        ) => {
            match result {
                Err(_) => anyhow::bail!(
                    "Registration to {} timed out after {}ms",
                    peer.name,
                    CONNECT_TIMEOUT.as_millis()
                ),
                Ok(sent) => sent.map_err(|error| anyhow::anyhow!("Connection to {} closed during registration: {error}", peer.name))?,
            }
        }
    }

    {
        let mut locked = state.write().await;
        anyhow::ensure!(
            !locked.frame_enforce || locked.authenticates(&context),
            "Unproven dialled peer cannot be published"
        );
        if let Some((keys, _)) = &locked.keys
            && let Some((node, key)) = context.mesh_peer()
        {
            anyhow::ensure!(
                mesh_trust::identity::link_refusal(&context, keys, &locked.tls_keys).is_none(),
                "Peer key revoked before connection publication"
            );
        }
        let still_current = locked.desired.get(&peer.name) == Some(&target)
            && locked
                .attempts
                .get(&peer.name)
                .is_some_and(|attempt| attempt.id == attempt_id && attempt.target == target)
            && !*attempt_cancel.borrow();
        if !still_current {
            anyhow::bail!(
                "Route to {} changed before connection publication",
                peer.name
            );
        }
        locked
            .connections
            .insert(peer.name.clone(), connection.clone());
    }

    if let Some(message) = early_inbound {
        let _ = incoming_tx.send(MeshInbound {
            context: context.clone(),
            peer: peer.name.clone(),
            connection_generation: generation,
            message,
        });
    }

    let connected_send = connected.clone();
    let writer_node = node_name.clone();
    tokio::spawn(async move {
        let mut cancelled = false;
        loop {
            tokio::select! {
                biased;
                changed = writer_cancel.changed() => {
                    let _ = changed;
                    cancelled = true;
                    break;
                }
                next = rx.recv() => {
                    let Some(message) = next else { break };
                    let message = if let Ok(mut frame) = bus::parse(&message) {
                        frame.originate_mesh(&writer_node);
                        frame.to_wire()
                    } else { message };
                    // The flush itself must also be cancellable: §7.7 says
                    // nothing is written to a retired endpoint after
                    // `routing.reload_applied`, and a backpressured peer can
                    // hold this await open long past the reload. A cancelled
                    // partial write is fine — teardown drops the socket. (A
                    // frame whose send completed in the same poll in which
                    // cancel was not yet observable can still land — the
                    // one-frame slip inherent to cooperative cancellation,
                    // covered by §7.7's kernel-visible allowance.)
                    let flushed = tokio::select! {
                        biased;
                        changed = writer_cancel.changed() => {
                            let _ = changed;
                            cancelled = true;
                            break;
                        }
                        result = ws_sink.send(tungstenite::Message::Text(message.into())) => result,
                    };
                    if flushed.is_err() {
                        break;
                    }
                }
            }
        }
        connected_send.store(false, Ordering::Release);
        // On cancel (retirement), DROP the sink instead of closing it: a
        // graceful WebSocket close first flushes userspace-buffered frames
        // (a §7.7 fence violation on a retired endpoint) and then waits for
        // the peer's close reply — a zero-window peer would park this task
        // and hold the socket forever, leaking one task per wedged
        // retirement. Dropping both split halves (the reader also breaks on
        // cancel) closes the TCP socket without flushing. On a natural exit
        // the polite close is still attempted, but bounded — never parked
        // on a wedged peer.
        if !cancelled {
            let _ = tokio::time::timeout(CONNECT_TIMEOUT, ws_sink.close()).await;
        }
    });

    let peer_name = peer.name.clone();
    let connected_read = connected.clone();
    let state_read = state.clone();
    let link_name = format!("bridge-{node_name}");
    let link_tx = connection.tx.clone();
    tokio::spawn(async move {
        let mut registration_answered = false;
        let mut unauthenticated = UnauthenticatedFrameLog::default();
        loop {
            let next = tokio::select! {
                biased;
                changed = reader_cancel.changed() => {
                    let _ = changed;
                    break;
                }
                next = ws_stream_rx.next() => next,
            };
            let Some(Ok(msg)) = next else { break };
            let text = match msg {
                tungstenite::Message::Text(t) => t.to_string(),
                tungstenite::Message::Close(_) => break,
                _ => continue,
            };

            let Ok(bus_msg) = bus::parse(&text) else {
                continue;
            };

            // This registration is a transport endpoint, not a command
            // service. Broker calls use the node address, never the link's
            // registration name. Do not drop a correlated request into mesh
            // ingress, which has no local recipient for this identity.
            let addressed_to_link = bus_msg.to_addr().is_some_and(|target| {
                target == link_name
                    || matches!(BusTarget::parse(target), Some(BusTarget::Local(address))
                        if address.service.as_deref() == Some(link_name.as_str()))
            });
            if addressed_to_link
                && matches!(bus_msg.get("type"), None | Some("request"))
                && let Some(id) = bus_msg.get("id")
            {
                let command = bus_msg.command_name().unwrap_or("");
                let reply = BusMessage::new()
                    .with_header("type", "response")
                    .with_header("id", id)
                    .with_header("rc", "10")
                    .with_header("command", command)
                    .with_body(&serde_json::json!({
                        "error_code": "UNKNOWN_COMMAND",
                        "message": "mesh links do not serve commands; address the remote node instead",
                        "command": command,
                    }).to_string());
                let _ = link_tx.send(reply.to_wire());
                continue;
            }

            // The answer to this link's own `noded.register` carries no ID
            // (the registration sent none) and no target. It belongs to the
            // link, not to mesh ingress, where it would be refused for want
            // of a recipient. A refused registration is logged, not lost.
            if !registration_answered
                && bus_msg.get("type") == Some("response")
                && bus_msg.get("id").is_none()
                && bus_msg.command_name() == Some("noded.register")
            {
                registration_answered = true;
                if bus_msg.get("rc") != Some("0") {
                    tracing::warn!(
                        peer = %peer_name,
                        rc = bus_msg.get("rc").unwrap_or(""),
                        "Remote broker refused this link's registration"
                    );
                }
                continue;
            }

            if bus_msg.is_empty_message() {
                continue;
            }
            let permitted = match context.mesh_peer() {
                Some((node, _)) => {
                    matches!(bus_msg.from_addr().and_then(BusTarget::parse), Some(BusTarget::Local(addr)) if addr.node == node)
                }
                None => false,
            };
            if !permitted {
                let enforce = state_read.read().await.frame_enforce;
                unauthenticated.record(std::time::Instant::now(), &peer_name, target, enforce);
                if enforce {
                    continue;
                }
            }

            // Only responses can satisfy a call. An event or inbound request
            // may legitimately carry an ID matching an unrelated request.
            if let Some(id) = bus_msg
                .get("id")
                .filter(|_| bus_msg.get("type") == Some("response"))
            {
                // Reconciliation holds the write fence through its pending
                // sweep. A response must acquire that same fence before it
                // can consume a call, so a retired route cannot win the race.
                let transport = state_read.read().await;
                let current = transport
                    .connections
                    .get(&peer_name)
                    .is_some_and(|connection| {
                        connection.generation == generation
                            && connection.connected.load(Ordering::Acquire)
                            && transport.desired.get(&peer_name) == Some(&connection.target)
                    });
                if !current {
                    continue;
                }
                let mut calls = pending.lock().expect("pending calls lock poisoned");
                if calls
                    .get(id)
                    .is_some_and(|call| call.connection_generation == generation)
                {
                    let call = calls.remove(id).expect("pending call checked above");
                    let _ = call.response.send(Ok(bus_msg));
                    continue;
                }
            }

            let _ = incoming_tx.send(MeshInbound {
                context: context.clone(),
                peer: peer_name.clone(),
                connection_generation: generation,
                message: bus_msg,
            });
        }

        connected_read.store(false, Ordering::Release);
        remove_connection_generation(&state_read, &peer_name, generation).await;
        fail_pending_generation(&pending, generation, "connection closed").await;
        tracing::info!(peer = %peer_name, "Remote broker disconnected");
    });

    tracing::info!(peer = %peer.name, target = %target, "Connected to remote broker");
    Ok(connection)
}

async fn remove_connection_generation(
    state: &RwLock<TransportState>,
    peer_name: &str,
    generation: uuid::Uuid,
) {
    let mut state = state.write().await;
    if state
        .connections
        .get(peer_name)
        .is_some_and(|connection| connection.generation == generation)
    {
        state.connections.remove(peer_name);
    }
}

async fn fail_pending_generation(
    pending: &Mutex<HashMap<String, PendingCall>>,
    generation: uuid::Uuid,
    reason: &str,
) {
    let mut pending = pending.lock().expect("pending calls lock poisoned");
    let ids: Vec<_> = pending
        .iter()
        .filter(|(_, call)| call.connection_generation == generation)
        .map(|(id, _)| id.clone())
        .collect();
    for id in ids {
        if let Some(call) = pending.remove(&id) {
            let _ = call.response.send(Err(reason.to_string()));
        }
    }
}

#[cfg(test)]
mod tests {
    // Callsite interest is process-wide; a rebuild can still race another test.
    // Run captures alone, with a fresh cache and diagnostic counters.
    fn log_capture_subprocess(test: &str) -> bool {
        let test = format!("{}::{test}", module_path!().split_once("::").unwrap().1);
        if std::env::var("MIXVIU_LOG_CAPTURE_TEST").as_deref() == Ok(test.as_str()) {
            return false;
        }
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", &test, "--nocapture"])
            .env("MIXVIU_LOG_CAPTURE_TEST", &test)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{test}:\n{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("1 passed"));
        true
    }

    #[test]
    fn diagnostic_window_counts_every_occurrence_and_emits_first_then_each_minute() {
        let start = std::time::Instant::now();
        let mut log = super::DiagnosticLog::default();
        let mut emissions = Vec::new();
        for second in 0..180 {
            if let Some(counts) = log.record(start + Duration::from_secs(second)) {
                emissions.push((second, counts));
            }
        }
        assert_eq!(
            emissions,
            vec![(0, (1, 1)), (60, (60, 61)), (120, (60, 121))]
        );
        assert_eq!(log.total(), 180);
    }

    #[test]
    fn challenge_epochs_are_per_peer_and_do_not_accept_a_wrong_verifier() {
        let mut challenge = BusMessage::new().with_header("id", "epoch-test");
        for (peer, epoch) in [("epoch-alpha", 7), ("epoch-beta", 8)] {
            challenge.body = serde_json::json!({
                "verifying_broker_node": peer, "mesh_fqdn": "example.test",
                "inventory_epoch": epoch,
                "session_id": base64::engine::general_purpose::STANDARD.encode([1; 16]),
                "server_nonce": base64::engine::general_purpose::STANDARD.encode([2; 32]),
            })
            .to_string();
            assert!(
                build_admit_response(
                    &challenge,
                    "local",
                    &Some(crate::seed::D2Seed::new([9; 32])),
                    peer,
                    "example.test",
                    6
                )
                .is_some()
            );
        }
        assert_eq!(super::peer_challenge_epoch("epoch-alpha"), Some(7));
        assert_eq!(super::peer_challenge_epoch("epoch-beta"), Some(8));
        assert!(
            build_admit_response(
                &challenge,
                "local",
                &Some(crate::seed::D2Seed::new([9; 32])),
                "epoch-alpha",
                "example.test",
                6
            )
            .is_none()
        );
        assert_eq!(super::peer_challenge_epoch("epoch-alpha"), Some(7));
        assert_eq!(super::peer_challenge_epoch("epoch-unknown"), None);
    }

    #[test]
    fn challenge_refusals_name_both_peers_inside_one_window() {
        if log_capture_subprocess("challenge_refusals_name_both_peers_inside_one_window") {
            return;
        }
        use std::io::Write;
        #[derive(Clone, Default)]
        struct Capture(Arc<Mutex<Vec<u8>>>);
        impl Write for Capture {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let output = Capture::default();
        let writer = output.clone();
        let subscriber = tracing_subscriber::fmt()
            .without_time()
            .with_ansi(false)
            .with_writer(move || writer.clone())
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            // Other tests may have registered this callsite without a subscriber.
            tracing::callsite::rebuild_interest_cache();
            // Reset only this test's namespace, preserving other tests' counters.
            {
                let mut logs = super::CHALLENGE_REFUSALS.lock().unwrap();
                for peer in ["refusal-alpha", "refusal-beta"] {
                    logs.remove(peer);
                }
            }
            for _ in 0..100 {
                for peer in ["refusal-alpha", "refusal-beta"] {
                    super::challenge_diagnostic(
                        true,
                        peer,
                        "example.test",
                        1,
                        &serde_json::json!({}),
                    );
                }
            }
        });
        let text = String::from_utf8(output.0.lock().unwrap().clone()).unwrap();
        assert_eq!(text.lines().count(), 2, "{text}");
        assert!(text.contains("refusal-alpha") && text.contains("refusal-beta"));
        let logs = super::CHALLENGE_REFUSALS.lock().unwrap();
        assert_eq!(logs["refusal-alpha"].total(), 100);
        assert_eq!(logs["refusal-beta"].total(), 100);
    }
    use super::*;
    use std::sync::atomic::AtomicUsize;
    use tokio::net::TcpListener;

    #[test]
    fn public_default_port_reexport_survives() {
        assert_eq!(crate::DEFAULT_NODED_PORT, 4200);
        assert_eq!(crate::DEFAULT_NODED_PORT, DEFAULT_NODED_PORT);
    }

    /// SPEC 13 §9a (2-c-1c) — the prover. No seed ⇒ `None` (register without
    /// proving). With a seed ⇒ a `noded.admit.response` whose `from` is the
    /// `bridge-<node>` label but whose `claimed_source_node` is the BARE name
    /// (the split `admit()`'s NameMismatch depends on).
    #[test]
    fn build_admit_response_shape_and_missing_seed() {
        let b64 = base64::engine::general_purpose::STANDARD;
        let mut ch = BusMessage::new()
            .with_header("command", "noded.admit.challenge")
            .with_header("id", "adm-1");
        ch.body = serde_json::json!({
            "mesh_fqdn": "bus",
            "verifying_broker_node": "beta",
            "inventory_epoch": 7,
            "session_id": b64.encode([1u8; 16]),
            "server_nonce": b64.encode([2u8; 32]),
        })
        .to_string();

        // B dialled C (gamma), which relays A's (beta's) challenge. B must
        // not supply the signature C could forward to A to impersonate B.
        assert!(
            build_admit_response(
                &ch,
                "delta",
                &Some(crate::seed::D2Seed::new([9; 32])),
                "gamma",
                "bus",
                7
            )
            .is_none()
        );
        assert!(
            build_admit_response(
                &ch,
                "delta",
                &Some(crate::seed::D2Seed::new([9; 32])),
                "beta",
                "other.example",
                7
            )
            .is_none()
        );
        for epoch in [6, 8] {
            assert!(
                build_admit_response(
                    &ch,
                    "delta",
                    &Some(crate::seed::D2Seed::new([9; 32])),
                    "beta",
                    "bus",
                    epoch
                )
                .is_some()
            );
        }

        // No seed → prover-incapable → None.
        assert!(build_admit_response(&ch, "delta", &None, "beta", "bus", 7).is_none());

        // With a seed → a well-shaped response.
        let wire = build_admit_response(
            &ch,
            "delta",
            &Some(crate::seed::D2Seed::new([9u8; 32])),
            "beta",
            "bus",
            7,
        )
        .expect("response");
        let resp = bus::parse(&wire).expect("parse");
        assert_eq!(resp.command_name(), Some("noded.admit.response"));
        assert_eq!(
            resp.get("from"),
            Some("bridge-delta"),
            "label keeps bridge- prefix"
        );
        let body: serde_json::Value = serde_json::from_str(&resp.body).unwrap();
        assert_eq!(
            body["claimed_source_node"], "delta",
            "bare node name, not bridge-"
        );
        assert_eq!(body["signed_epoch"], 7);
        // origin-only zeros + a 64-byte signature.
        assert_eq!(
            b64.decode(body["signature"].as_str().unwrap())
                .unwrap()
                .len(),
            64
        );
        assert_eq!(
            b64.decode(body["client_ephemeral"].as_str().unwrap())
                .unwrap(),
            vec![0u8; 32]
        );
    }

    // `.conf.mix` map literals separate entries with `,` (newlines are
    // suppressed inside `{ }`); the top-level body uses newlines.

    #[test]
    fn conf_mix_parses_peers_and_default_port() {
        let src = r#"
            node_name: "alpha"
            peers: [
              { name: "beta", mesh_ip: "192.0.2.2" },
              { name: "gamma", mesh_ip: "192.0.2.3", noded_port: 4300 }
            ]
        "#;
        let cfg: MeshConfig = config::from_conf_mix_str(src).expect("parse");
        assert_eq!(cfg.node_name, "alpha");
        assert_eq!(cfg.peers.len(), 2);
        // `noded_port` defaults to 4200 when omitted.
        assert_eq!(cfg.find_peer("beta").unwrap().noded_port, 4200);
        assert_eq!(cfg.find_peer("gamma").unwrap().noded_port, 4300);
        assert_eq!(
            cfg.find_peer("gamma").unwrap().noded_url(),
            "ws://192.0.2.3:4300/ws"
        );
    }

    #[test]
    fn deny_unknown_fields_rejects_stale_hub_port() {
        // The exact regression PeerConfig's deny_unknown_fields guards:
        // a peer entry still using the pre-rename `hub_port` key must
        // error, not silently default `noded_port` to 4200 and connect
        // to the wrong port on a node that customized it.
        let src = r#"
            node_name: "alpha"
            peers: [ { name: "beta", mesh_ip: "192.0.2.2", hub_port: 9999 } ]
        "#;
        let err = config::from_conf_mix_str::<MeshConfig>(src).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("unknown field") && msg.contains("hub_port"),
            "expected unknown-field error naming hub_port, got: {msg}"
        );
    }

    fn temp_dir(tag: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("mix-mesh-cfg-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn load_explicit_conf_mix_path() {
        let dir = temp_dir("explicit");
        let path = dir.join("mesh.conf.mix");
        std::fs::write(
            &path,
            "node_name: \"alpha\"\npeers: [ { name: \"beta\", mesh_ip: \"192.0.2.2\" } ]\n",
        )
        .unwrap();
        let cfg = MeshConfig::load(&path.to_string_lossy()).expect("load");
        assert_eq!(cfg.node_name, "alpha");
        assert_eq!(cfg.peers.len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn load_missing_path_is_error() {
        assert!(MeshConfig::load("/nonexistent/mix/mesh.conf.mix").is_err());
    }

    #[test]
    fn hostname_in_mesh_ip_is_rejected() {
        // The whole point of `mesh_ip: IpAddr`. A hostname here would be
        // interpolated into the broker's ws:// URL and resolved by
        // `connect_async`, silently making DNS the mesh routing
        // authority. It must fail at deserialize, not at connect.
        let src = r#"
            node_name: "alpha"
            peers: [ { name: "beta", mesh_ip: "noded.beta.example.internal" } ]
        "#;
        let err = config::from_conf_mix_str::<MeshConfig>(src).unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("invalid IP address syntax"),
            "expected the IpAddr parse to reject a hostname, got: {msg}"
        );
    }

    #[test]
    fn ipv6_peer_url_is_bracketed() {
        // Direct `format!("ws://{}:{}", ip, port)` would emit
        // `ws://::1:4200`, which is not a valid URL. SocketAddr brackets
        // it. The fleet is IPv4-only today, so only a test pins this.
        let src = r#"
            node_name: "alpha"
            peers: [ { name: "beta", mesh_ip: "2001:db8::1", noded_port: 4300 } ]
        "#;
        let cfg: MeshConfig = config::from_conf_mix_str(src).expect("parse");
        assert_eq!(
            cfg.find_peer("beta").unwrap().noded_url(),
            "ws://[2001:db8::1]:4300/ws"
        );
    }

    fn test_mesh() -> MeshPeers {
        let (incoming_tx, _incoming_rx) = mpsc::unbounded_channel();
        MeshPeers::new(
            MeshConfig {
                node_name: "alpha".into(),
                peers: vec![],
                d2_seed: None,
            },
            incoming_tx,
        )
    }

    fn peer(name: &str, endpoint: SocketAddr) -> PeerConfig {
        PeerConfig {
            tls_port: None,
            name: name.into(),
            mesh_ip: endpoint.ip(),
            noded_port: endpoint.port(),
        }
    }

    fn test_mesh_with_peer(peer: PeerConfig) -> (MeshPeers, mpsc::UnboundedReceiver<MeshInbound>) {
        let (incoming_tx, incoming_rx) = mpsc::unbounded_channel();
        (
            MeshPeers::new(
                MeshConfig {
                    node_name: "alpha".into(),
                    peers: vec![peer],
                    d2_seed: None,
                },
                incoming_tx,
            ),
            incoming_rx,
        )
    }

    fn test_connection(target: SocketAddr) -> Arc<RemoteBroker> {
        let (tx, _rx) = mpsc::unbounded_channel();
        let (cancel, _cancel_rx) = watch::channel(false);
        Arc::new(RemoteBroker {
            context: Default::default(),
            target,
            generation: uuid::Uuid::new_v4(),
            tx,
            connected: Arc::new(AtomicBool::new(true)),
            cancel,
        })
    }

    #[test]
    fn sustained_old_peer_logs_one_initial_line_then_counted_summaries() {
        if log_capture_subprocess("sustained_old_peer_logs_one_initial_line_then_counted_summaries")
        {
            return;
        }
        use std::io::Write;
        #[derive(Clone, Default)]
        struct Capture(Arc<Mutex<Vec<u8>>>);
        impl Write for Capture {
            fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
                self.0.lock().unwrap().extend_from_slice(bytes);
                Ok(bytes.len())
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let capture = Capture::default();
        let writer = capture.clone();
        let subscriber = tracing_subscriber::fmt()
            .without_time()
            .with_ansi(false)
            .with_max_level(tracing::Level::INFO)
            .with_writer(move || writer.clone())
            .finish();
        let _guard = tracing::subscriber::set_default(subscriber);
        tracing::callsite::rebuild_interest_cache();
        let now = std::time::Instant::now();
        let endpoint = "192.0.2.2:4200".parse().unwrap();
        let before = unauthenticated_frame_total();
        let mut log = UnauthenticatedFrameLog::default();
        for _ in 0..1127 {
            log.record(now, "beta", endpoint, false);
        }
        assert_eq!(
            String::from_utf8(capture.0.lock().unwrap().clone())
                .unwrap()
                .lines()
                .count(),
            1
        );
        log.record(now + Duration::from_secs(60), "beta", endpoint, false);
        let text = String::from_utf8(capture.0.lock().unwrap().clone()).unwrap();
        assert_eq!(text.lines().count(), 2);
        assert!(text.contains("count=1127"), "{text}");
        assert!(text.contains("total=1128"), "{text}");
        assert!(text.contains("posture=\"observe\""), "{text}");
        assert_eq!(log.total, 1128);
        assert!(unauthenticated_frame_total() >= before + 1128);
    }

    #[tokio::test]
    async fn unchanged_keys_retire_unproven_link_with_honest_inflight_reason() {
        let endpoint = "192.0.2.2:4200".parse().unwrap();
        let (mesh, _rx) = test_mesh_with_peer(peer("beta", endpoint));
        let keys = mesh_trust::identity::KeyAuthority::new(&Default::default(), 2, 0);
        let desired = HashMap::from([("beta".into(), endpoint)]);
        let revision = AuthorityRevision {
            epoch: 2,
            recovery_generation: 0,
        };
        mesh.reconcile_identity(desired.clone(), revision, keys.clone(), "bus".into())
            .await;
        let connection = test_connection(endpoint);
        mesh.state
            .write()
            .await
            .connections
            .insert("beta".into(), connection.clone());
        let (response, receive) = oneshot::channel();
        mesh.pending.lock().unwrap().insert(
            "old-call".into(),
            PendingCall {
                peer: "beta".into(),
                connection_generation: connection.generation,
                class: "echo.slow".into(),
                response,
            },
        );
        let report = mesh
            .reconcile_identity(desired, revision, keys, "bus".into())
            .await;
        assert_eq!(
            report.connections_retired[0].reason.as_str(),
            "unproven-link-retired"
        );
        assert_eq!(
            report.inflight_failed[0].reason.as_str(),
            "unproven-link-retired"
        );
        assert!(
            receive
                .await
                .unwrap()
                .unwrap_err()
                .contains("unproven-link-retired")
        );
    }

    #[tokio::test]
    async fn exact_key_reconcile_retains_overlap_then_drops_only_revoked_generation() {
        use std::collections::BTreeMap;
        let endpoint = "192.0.2.2:4200".parse().unwrap();
        let (mesh, _rx) = test_mesh_with_peer(peer("beta", endpoint));
        let mut old = test_connection(endpoint);
        Arc::get_mut(&mut old).unwrap().context.identity =
            bus::native_session::PeerIdentity::MeshAdmitted {
                node: "beta".into(),
                key: [3; 32],
                fingerprint: String::new(),
                epoch: 1,
                revision: 0,
            };
        mesh.state
            .write()
            .await
            .connections
            .insert("beta".into(), old.clone());
        let keys = |include_old| {
            let mut credentials = vec![
                serde_json::json!({"kind":"d2", "pubkey":base64::engine::general_purpose::STANDARD.encode([4;32]),"from_epoch":1,"until_epoch":null}),
            ];
            if include_old {
                credentials.push(serde_json::json!({"kind":"d2","pubkey":base64::engine::general_purpose::STANDARD.encode([3;32]),"from_epoch":1,"until_epoch":null}));
            }
            mesh_trust::identity::KeyAuthority::new(
                &BTreeMap::from([(
                    "beta".into(),
                    serde_json::json!({"status":"active","bus":true,"credentials":credentials}),
                )]),
                2,
                0,
            )
        };
        let desired = HashMap::from([("beta".into(), endpoint)]);
        let overlap = mesh
            .reconcile_identity(
                desired.clone(),
                AuthorityRevision {
                    epoch: 2,
                    recovery_generation: 0,
                },
                keys(true),
                "example.test".into(),
            )
            .await;
        assert_eq!(overlap.connections_retained, 1);
        let start = std::time::Instant::now();
        let rotated = mesh
            .reconcile_identity(
                desired,
                AuthorityRevision {
                    epoch: 3,
                    recovery_generation: 0,
                },
                keys(false),
                "example.test".into(),
            )
            .await;
        assert_eq!(rotated.connections_retired.len(), 1);
        assert!(!old.connected.load(Ordering::Acquire));
        assert!(start.elapsed() < Duration::from_secs(1));
    }

    #[tokio::test]
    async fn dead_attempt_is_swept_so_the_next_caller_dials_fresh() {
        // A connect driver that dies without publishing an outcome (panic or
        // abort) drops its `outcome` sender with the map entry still in
        // place. Joiners must not inherit that corpse forever: the first
        // caller reports the dead attempt, and the NEXT caller dials fresh.
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = listener.local_addr().unwrap();
        drop(listener); // closed port → a fresh dial fails with refused, fast
        let beta = peer("beta", endpoint);
        let (mesh, _incoming_rx) = test_mesh_with_peer(beta.clone());
        mesh.reconcile_endpoints(
            HashMap::from([("beta".to_string(), endpoint)]),
            AuthorityRevision::default(),
        )
        .await;
        {
            let mut state = mesh.state.write().await;
            let (cancel, _cancel_rx) = watch::channel(false);
            let (outcome_tx, outcome) = watch::channel(AttemptOutcome::Pending);
            drop(outcome_tx); // the driver died before send_replace
            state.attempts.insert(
                "beta".into(),
                Arc::new(ConnectAttempt {
                    id: uuid::Uuid::new_v4(),
                    target: endpoint,
                    cancel,
                    outcome,
                }),
            );
        }

        let first = mesh
            .call(beta.clone(), BusMessage::new())
            .await
            .expect_err("dead attempt must error");
        assert!(
            first.to_string().contains("ended without a result"),
            "first caller reports the dead attempt: {first}"
        );
        assert!(
            !mesh.state.read().await.attempts.contains_key("beta"),
            "the dead attempt entry must be swept"
        );
        let second = mesh
            .call(beta, BusMessage::new())
            .await
            .expect_err("closed port must refuse");
        assert!(
            !second.to_string().contains("ended without a result"),
            "second caller must dial fresh, not join the corpse: {second}"
        );
    }

    #[tokio::test]
    async fn cache_reuses_only_the_same_resolved_endpoint() {
        let mesh = test_mesh();
        let original = PeerConfig {
            tls_port: None,
            name: "beta".into(),
            mesh_ip: "192.0.2.2".parse().unwrap(),
            noded_port: DEFAULT_NODED_PORT,
        };
        let connection = test_connection(original.noded_addr());
        {
            let mut state = mesh.state.write().await;
            state
                .desired
                .insert(original.name.clone(), original.noded_addr());
            state
                .connections
                .insert(original.name.clone(), connection.clone());
        }

        let reused = mesh.cached_connection(&original).await.unwrap();
        assert!(Arc::ptr_eq(&reused, &connection));

        let changed_ip = PeerConfig {
            tls_port: None,
            mesh_ip: "192.0.2.3".parse().unwrap(),
            ..original.clone()
        };
        assert!(mesh.cached_connection(&changed_ip).await.is_none());

        let changed_port = PeerConfig {
            tls_port: None,
            noded_port: 4300,
            ..original.clone()
        };
        assert!(mesh.cached_connection(&changed_port).await.is_none());
        assert!(Arc::ptr_eq(
            &mesh.cached_connection(&original).await.unwrap(),
            &connection
        ));
    }

    #[tokio::test]
    async fn disconnected_cache_entry_is_not_reused() {
        let mesh = test_mesh();
        let peer = PeerConfig {
            tls_port: None,
            name: "beta".into(),
            mesh_ip: "192.0.2.2".parse().unwrap(),
            noded_port: DEFAULT_NODED_PORT,
        };
        let connection = test_connection(peer.noded_addr());
        connection.connected.store(false, Ordering::Release);
        {
            let mut state = mesh.state.write().await;
            state.desired.insert(peer.name.clone(), peer.noded_addr());
            state.connections.insert(peer.name.clone(), connection);
        }

        assert!(mesh.cached_connection(&peer).await.is_none());
    }

    #[tokio::test]
    async fn same_endpoint_survives_revision_change_with_same_generation() {
        let mesh = test_mesh();
        let endpoint: SocketAddr = "192.0.2.2:4200".parse().unwrap();
        let connection = test_connection(endpoint);
        {
            let mut state = mesh.state.write().await;
            state.desired.insert("beta".into(), endpoint);
            state.connections.insert("beta".into(), connection.clone());
        }

        let report = mesh
            .reconcile_endpoints(
                HashMap::from([("beta".into(), endpoint)]),
                AuthorityRevision {
                    epoch: 8,
                    recovery_generation: 2,
                },
            )
            .await;

        assert_eq!(
            report,
            ReconcileReport {
                connections_retained: 1,
                ..ReconcileReport::default()
            }
        );
        assert_eq!(
            mesh.state
                .read()
                .await
                .connections
                .get("beta")
                .unwrap()
                .generation,
            connection.generation
        );
    }

    #[tokio::test]
    async fn removed_and_changed_endpoints_retire_only_their_connections() {
        let mesh = test_mesh();
        let beta_old: SocketAddr = "192.0.2.2:4200".parse().unwrap();
        let beta_new: SocketAddr = "192.0.2.22:4200".parse().unwrap();
        let gamma: SocketAddr = "192.0.2.3:4200".parse().unwrap();
        let delta: SocketAddr = "192.0.2.4:4200".parse().unwrap();
        let beta_connection = test_connection(beta_old);
        let gamma_connection = test_connection(gamma);
        let delta_connection = test_connection(delta);
        {
            let mut state = mesh.state.write().await;
            state.desired = HashMap::from([
                ("beta".into(), beta_old),
                ("gamma".into(), gamma),
                ("delta".into(), delta),
            ]);
            state.connections = HashMap::from([
                ("beta".into(), beta_connection.clone()),
                ("gamma".into(), gamma_connection.clone()),
                ("delta".into(), delta_connection.clone()),
            ]);
        }

        let report = mesh
            .reconcile_endpoints(
                HashMap::from([("beta".into(), beta_new), ("gamma".into(), gamma)]),
                AuthorityRevision {
                    epoch: 9,
                    recovery_generation: 2,
                },
            )
            .await;

        assert_eq!(report.added, 0);
        assert_eq!(report.removed, 1);
        assert_eq!(report.endpoint_changed, 1);
        assert_eq!(report.connections_retired.len(), 2);
        assert_eq!(report.connections_retained, 1);
        assert!(!beta_connection.connected.load(Ordering::Acquire));
        assert!(!delta_connection.connected.load(Ordering::Acquire));
        assert!(gamma_connection.connected.load(Ordering::Acquire));
        let state = mesh.state.read().await;
        assert_eq!(state.connections.len(), 1);
        assert_eq!(
            state.connections.get("gamma").unwrap().generation,
            gamma_connection.generation
        );
    }

    #[tokio::test]
    async fn stale_caller_cannot_enqueue_or_reconnect_after_reconcile() {
        let endpoint: SocketAddr = "192.0.2.2:4200".parse().unwrap();
        let beta = peer("beta", endpoint);
        let (mesh, _incoming) = test_mesh_with_peer(beta.clone());
        let captured = test_connection(endpoint);
        mesh.state
            .write()
            .await
            .connections
            .insert("beta".into(), captured.clone());

        mesh.reconcile_endpoints(HashMap::new(), AuthorityRevision::default())
            .await;

        let state = mesh.state.read().await;
        assert!(
            mesh.validate_outbound_locked(&state, &beta, &captured)
                .is_err()
        );
        drop(state);
        assert!(
            mesh.send(beta, BusMessage::new())
                .await
                .unwrap_err()
                .to_string()
                .contains("no longer authorised")
        );
    }

    #[tokio::test]
    async fn stale_queued_inbound_is_rejected_at_dequeue() {
        let endpoint: SocketAddr = "192.0.2.2:4200".parse().unwrap();
        let beta = peer("beta", endpoint);
        let (mesh, _incoming) = test_mesh_with_peer(beta);
        let connection = test_connection(endpoint);
        mesh.state
            .write()
            .await
            .connections
            .insert("beta".into(), connection.clone());
        let inbound = MeshInbound {
            context: Default::default(),
            peer: "beta".into(),
            connection_generation: connection.generation,
            message: BusMessage::new(),
        };
        assert!(
            mesh.inbound_generation_is_current(&inbound.peer, inbound.connection_generation)
                .await
        );

        mesh.reconcile_endpoints(HashMap::new(), AuthorityRevision::default())
            .await;

        assert!(
            !mesh
                .inbound_generation_is_current(&inbound.peer, inbound.connection_generation)
                .await
        );
        assert!(
            mesh.validate_inbound(&inbound.peer, inbound.connection_generation)
                .await
                .is_none()
        );
    }

    #[tokio::test]
    async fn pending_rpc_is_failed_once_and_reported() {
        let endpoint: SocketAddr = "192.0.2.2:4200".parse().unwrap();
        let beta = peer("beta", endpoint);
        let (mesh, _incoming) = test_mesh_with_peer(beta);
        let connection = test_connection(endpoint);
        mesh.state
            .write()
            .await
            .connections
            .insert("beta".into(), connection.clone());
        let (response, receive) = oneshot::channel();
        mesh.pending
            .lock()
            .expect("pending calls lock poisoned")
            .insert(
                "rpc-1".into(),
                PendingCall {
                    peer: "beta".into(),
                    connection_generation: connection.generation,
                    class: "props.get".into(),
                    response,
                },
            );

        let report = mesh
            .reconcile_endpoints(HashMap::new(), AuthorityRevision::default())
            .await;
        assert_eq!(report.inflight_failed.len(), 1);
        assert_eq!(report.inflight_failed[0].message_id, "rpc-1");
        assert!(receive.await.unwrap().unwrap_err().contains("revoked"));

        let second = mesh
            .reconcile_endpoints(HashMap::new(), AuthorityRevision::default())
            .await;
        assert!(second.inflight_failed.is_empty());
    }

    #[tokio::test]
    async fn empty_desired_set_retires_every_connection() {
        let mesh = test_mesh();
        let beta: SocketAddr = "192.0.2.2:4200".parse().unwrap();
        let gamma: SocketAddr = "192.0.2.3:4200".parse().unwrap();
        {
            let mut state = mesh.state.write().await;
            state.desired = HashMap::from([("beta".into(), beta), ("gamma".into(), gamma)]);
            state.connections = HashMap::from([
                ("beta".into(), test_connection(beta)),
                ("gamma".into(), test_connection(gamma)),
            ]);
        }

        let report = mesh
            .reconcile_endpoints(HashMap::new(), AuthorityRevision::default())
            .await;
        assert_eq!(report.removed, 2);
        assert_eq!(report.connections_retired.len(), 2);
        assert!(mesh.state.read().await.connections.is_empty());
    }

    async fn spawn_open_websocket() -> (
        SocketAddr,
        oneshot::Receiver<()>,
        tokio::task::JoinHandle<()>,
    ) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = listener.local_addr().unwrap();
        let (closed_tx, closed_rx) = oneshot::channel();
        let task = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut websocket = tokio_tungstenite::accept_async(stream).await.unwrap();
            websocket
                .send(tungstenite::Message::Ping(Vec::new().into()))
                .await
                .unwrap();
            assert!(websocket.next().await.is_some(), "registration frame");
            while websocket.next().await.is_some() {}
            let _ = closed_tx.send(());
        });
        (endpoint, closed_rx, task)
    }

    async fn next_bus_frame(
        websocket: &mut tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    ) -> BusMessage {
        loop {
            match websocket.next().await.unwrap().unwrap() {
                tungstenite::Message::Text(text) => return bus::parse(&text).unwrap(),
                tungstenite::Message::Ping(_) | tungstenite::Message::Pong(_) => continue,
                frame => panic!("unexpected frame: {frame:?}"),
            }
        }
    }

    #[tokio::test]
    async fn dialler_refuses_relay_and_wrong_authority_but_answers_normal_challenge() {
        let b64 = base64::engine::general_purpose::STANDARD;
        for (verifier, mesh_name, epoch, answered, trusted) in [
            ("gamma", "bus", 7, false, true),
            ("beta", "other.example", 7, false, true),
            ("beta", "bus", 6, true, true),
            ("beta", "bus", 8, true, true),
            ("beta", "bus", 7, true, true),
            ("beta", "bus", 7, false, false),
        ] {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let beta = peer("beta", listener.local_addr().unwrap());
            let (tx, _rx) = mpsc::unbounded_channel();
            let transport = MeshPeers::new(
                MeshConfig {
                    node_name: "alpha".into(),
                    peers: vec![beta.clone()],
                    d2_seed: Some(crate::seed::D2Seed::new([9; 32])),
                },
                tx,
            );
            if trusted {
                transport
                    .reconcile_identity(
                        HashMap::from([("beta".into(), beta.noded_addr())]),
                        AuthorityRevision {
                            epoch: 7,
                            recovery_generation: 0,
                        },
                        mesh_trust::identity::KeyAuthority::new(&Default::default(), 7, 0),
                        "bus".into(),
                    )
                    .await;
            }
            let challenge = BusMessage::new()
                .with_header("command", "noded.admit.challenge")
                .with_header("id", "relayed-challenge")
                .with_body(
                    &serde_json::json!({
                        "mesh_fqdn": mesh_name, "inventory_epoch": epoch,
                        "verifying_broker_node": verifier, "session_id": b64.encode([1;16]),
                        "server_nonce": b64.encode([2;32]),
                    })
                    .to_string(),
                );
            let server = tokio::spawn(async move {
                let (stream, _) = listener.accept().await.unwrap();
                let mut socket = tokio_tungstenite::accept_async(stream).await.unwrap();
                socket
                    .send(tungstenite::Message::Text(challenge.to_wire().into()))
                    .await
                    .unwrap();
                let first = next_bus_frame(&mut socket).await;
                assert_eq!(
                    first.command_name(),
                    Some(if answered {
                        "noded.admit.response"
                    } else if trusted {
                        "noded.identity.challenge"
                    } else {
                        "noded.register"
                    })
                );
                if answered {
                    assert_eq!(first.get("id"), Some("relayed-challenge"));
                }
            });
            let _ = transport.ensure_connected(&beta).await;
            server.await.unwrap();
        }
    }

    #[tokio::test]
    async fn concurrent_clients_can_use_the_same_id_and_receive_their_own_reply() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let beta = peer("beta", listener.local_addr().unwrap());
        let (mesh, _incoming) = test_mesh_with_peer(beta.clone());
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut websocket = tokio_tungstenite::accept_async(stream).await.unwrap();
            websocket
                .send(tungstenite::Message::Ping(Vec::new().into()))
                .await
                .unwrap();
            let _registration = next_bus_frame(&mut websocket).await;
            let first = next_bus_frame(&mut websocket).await;
            let second = next_bus_frame(&mut websocket).await;
            assert_ne!(first.get("id"), second.get("id"));
            for request in [second, first] {
                assert_eq!(request.get("hop_limit"), Some("0"));
                assert!(request.get("deadline").is_some());
                assert!(request.get("request_id").is_some());
                let response = BusMessage::new()
                    .with_header("type", "response")
                    .with_header("id", request.get("id").unwrap())
                    .with_body(request.command_name().unwrap());
                websocket
                    .send(tungstenite::Message::Text(response.to_wire().into()))
                    .await
                    .unwrap();
            }
        });
        let request = |command| {
            BusMessage::new()
                .with_header("type", "request")
                .with_header("id", "1")
                .with_header("command", command)
                .with_header("request_id", command)
        };
        let calls = async {
            tokio::join!(
                mesh.call(beta.clone(), request("first")),
                mesh.call(beta, request("second"))
            )
        };
        let (first, second) = tokio::time::timeout(Duration::from_secs(2), calls)
            .await
            .unwrap();
        for (reply, expected) in [(first.unwrap(), "first"), (second.unwrap(), "second")] {
            assert_eq!(reply.get("id"), Some("1"));
            assert_eq!(reply.body, expected);
            assert_eq!(reply.get("request_id"), Some(expected));
        }
        server.await.unwrap();
    }

    #[tokio::test]
    async fn link_registration_refuses_commands_immediately() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let beta = peer("beta", listener.local_addr().unwrap());
        let (mesh, mut incoming) = test_mesh_with_peer(beta.clone());
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut websocket = tokio_tungstenite::accept_async(stream).await.unwrap();
            websocket
                .send(tungstenite::Message::Ping(Vec::new().into()))
                .await
                .unwrap();
            let registration = next_bus_frame(&mut websocket).await;
            assert_eq!(registration.command_name(), Some("noded.register"));
            let name = registration.from_addr().unwrap();
            assert_eq!(name, "bridge-alpha");
            // Events with correlation IDs still enter mesh ingress. A link
            // must never manufacture an UNKNOWN_COMMAND response from one.
            let event = BusMessage::new()
                .with_header("to", name)
                .with_header("type", "event")
                .with_header("command", "props.changed")
                .with_header("id", "event-id");
            websocket
                .send(tungstenite::Message::Text(event.to_wire().into()))
                .await
                .unwrap();
            assert!(
                tokio::time::timeout(Duration::from_millis(100), next_bus_frame(&mut websocket))
                    .await
                    .is_err(),
                "link-addressed events must not be answered"
            );
            for command in ["HELP", "x.ping"] {
                let request = BusMessage::new()
                    .with_header("to", name)
                    .with_header("type", "request")
                    .with_header("command", command)
                    .with_header("id", command);
                websocket
                    .send(tungstenite::Message::Text(request.to_wire().into()))
                    .await
                    .unwrap();
                let reply = tokio::time::timeout(
                    Duration::from_millis(250),
                    next_bus_frame(&mut websocket),
                )
                .await
                .expect("link must refuse well under one second");
                assert_eq!(reply.get("type"), Some("response"));
                assert_eq!(reply.get("rc"), Some("10"));
                assert_eq!(reply.get("id"), Some(command));
                let body: serde_json::Value = serde_json::from_str(&reply.body).unwrap();
                assert_eq!(body["error_code"], "UNKNOWN_COMMAND");
                assert_eq!(body["command"], command);
                assert!(body["message"].as_str().unwrap().contains("mesh links"));
            }
        });
        mesh.ensure_connected(&beta).await.unwrap();
        server.await.unwrap();
        let event = tokio::time::timeout(Duration::from_secs(2), incoming.recv())
            .await
            .unwrap()
            .expect("link-addressed event must still reach mesh ingress");
        assert_eq!(event.peer, "beta");
        assert_eq!(event.message.get("type"), Some("event"));
        assert_eq!(event.message.get("id"), Some("event-id"));
        assert_eq!(event.message.to_addr(), Some("bridge-alpha"));
        assert_eq!(event.message.command_name(), Some("props.changed"));
        assert!(
            incoming.try_recv().is_err(),
            "link requests must not enter mesh ingress"
        );
        mesh.reconcile_endpoints(HashMap::new(), AuthorityRevision::default())
            .await;
    }

    #[tokio::test]
    async fn registration_answer_stays_on_the_link() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let beta = peer("beta", listener.local_addr().unwrap());
        let (mesh, mut incoming) = test_mesh_with_peer(beta.clone());
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut websocket = tokio_tungstenite::accept_async(stream).await.unwrap();
            websocket
                .send(tungstenite::Message::Ping(Vec::new().into()))
                .await
                .unwrap();
            let registration = next_bus_frame(&mut websocket).await;
            assert_eq!(registration.command_name(), Some("noded.register"));
            assert_eq!(registration.get("id"), None);
            // What noded answers a registration with: no ID, no target. A
            // second one is not this link's and goes to ingress as before.
            for _ in 0..2 {
                let answer = BusMessage::new()
                    .with_header("type", "response")
                    .with_header("from", "noded")
                    .with_header("rc", "0")
                    .with_header("command", "noded.register");
                websocket
                    .send(tungstenite::Message::Text(answer.to_wire().into()))
                    .await
                    .unwrap();
            }
            websocket
        });
        mesh.ensure_connected(&beta).await.unwrap();
        let _websocket = server.await.unwrap();
        let first = tokio::time::timeout(Duration::from_secs(2), incoming.recv())
            .await
            .unwrap()
            .expect("the second answer reaches mesh ingress");
        assert_eq!(first.message.command_name(), Some("noded.register"));
        assert!(
            tokio::time::timeout(Duration::from_millis(100), incoming.recv())
                .await
                .is_err(),
            "the link's own registration answer must not enter mesh ingress"
        );
        mesh.reconcile_endpoints(HashMap::new(), AuthorityRevision::default())
            .await;
    }

    #[tokio::test]
    async fn an_event_cannot_complete_a_call_and_cancellation_removes_its_entry() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let beta = peer("beta", listener.local_addr().unwrap());
        let (mesh, mut incoming) = test_mesh_with_peer(beta.clone());
        let mesh = Arc::new(mesh);
        let (event_sent, event_received) = oneshot::channel();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut websocket = tokio_tungstenite::accept_async(stream).await.unwrap();
            websocket
                .send(tungstenite::Message::Ping(Vec::new().into()))
                .await
                .unwrap();
            let _registration = next_bus_frame(&mut websocket).await;
            let request = next_bus_frame(&mut websocket).await;
            let event = BusMessage::new()
                .with_header("type", "event")
                .with_header("id", request.get("id").unwrap())
                .with_header("command", "props.changed");
            websocket
                .send(tungstenite::Message::Text(event.to_wire().into()))
                .await
                .unwrap();
            event_sent.send(()).unwrap();
            while websocket.next().await.is_some() {}
        });
        let caller_mesh = mesh.clone();
        let caller = tokio::spawn(async move {
            caller_mesh
                .call(
                    beta,
                    BusMessage::new()
                        .with_header("id", "1")
                        .with_header("command", "pending"),
                )
                .await
        });
        event_received.await.unwrap();
        let event = tokio::time::timeout(Duration::from_secs(2), incoming.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(event.message.get("type"), Some("event"));
        assert!(
            !caller.is_finished(),
            "events do not satisfy response correlation"
        );
        assert_eq!(mesh.pending.lock().unwrap().len(), 1);
        caller.abort();
        assert!(caller.await.unwrap_err().is_cancelled());
        assert!(mesh.pending.lock().unwrap().is_empty());
        mesh.reconcile_endpoints(HashMap::new(), AuthorityRevision::default())
            .await;
        server.await.unwrap();
    }

    #[tokio::test]
    async fn retirement_stops_transport_tasks_and_closes_websocket() {
        let (endpoint, closed, server) = spawn_open_websocket().await;
        let beta = peer("beta", endpoint);
        let (mesh, _incoming) = test_mesh_with_peer(beta.clone());
        mesh.ensure_connected(&beta).await.unwrap();

        let report = mesh
            .reconcile_endpoints(HashMap::new(), AuthorityRevision::default())
            .await;
        assert_eq!(report.connections_retired.len(), 1);
        tokio::time::timeout(Duration::from_secs(1), closed)
            .await
            .expect("server observed transport closure")
            .unwrap();
        server.await.unwrap();
    }

    #[tokio::test]
    async fn late_connect_cannot_publish_after_reconcile() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = listener.local_addr().unwrap();
        let beta = peer("beta", endpoint);
        let (mesh, _incoming) = test_mesh_with_peer(beta.clone());
        let mesh = Arc::new(mesh);
        let accepted = Arc::new(tokio::sync::Notify::new());
        let accepted_server = accepted.clone();
        let server = tokio::spawn(async move {
            let (_stream, _) = listener.accept().await.unwrap();
            accepted_server.notify_one();
            tokio::time::sleep(Duration::from_secs(2)).await;
        });
        let caller_mesh = mesh.clone();
        let caller = tokio::spawn(async move { caller_mesh.ensure_connected(&beta).await });
        accepted.notified().await;

        mesh.reconcile_endpoints(HashMap::new(), AuthorityRevision::default())
            .await;
        assert!(caller.await.unwrap().is_err());
        let state = mesh.state.read().await;
        assert!(state.connections.is_empty());
        assert!(state.attempts.is_empty());
        drop(state);
        server.abort();
    }

    #[tokio::test]
    async fn shared_attempt_timeout_fans_out_once() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = listener.local_addr().unwrap();
        let beta = peer("beta", endpoint);
        let (mesh, _incoming) = test_mesh_with_peer(beta.clone());
        let mesh = Arc::new(mesh);
        let accepted = Arc::new(AtomicUsize::new(0));
        let accepted_server = accepted.clone();
        let server = tokio::spawn(async move {
            let mut held = Vec::new();
            while let Ok(Ok((stream, _))) =
                tokio::time::timeout(Duration::from_millis(5500), listener.accept()).await
            {
                accepted_server.fetch_add(1, Ordering::SeqCst);
                held.push(stream);
            }
            held
        });

        let started = tokio::time::Instant::now();
        let mut callers = Vec::new();
        for _ in 0..8 {
            let mesh = mesh.clone();
            let beta = beta.clone();
            callers.push(tokio::spawn(async move {
                mesh.ensure_connected(&beta)
                    .await
                    .err()
                    .expect("connection attempt must time out")
                    .to_string()
            }));
        }
        let mut errors = Vec::new();
        for caller in callers {
            errors.push(caller.await.unwrap());
        }
        let elapsed = started.elapsed();
        assert!(
            elapsed >= Duration::from_millis(4900),
            "elapsed {elapsed:?}"
        );
        assert!(elapsed < Duration::from_secs(6), "elapsed {elapsed:?}");
        assert!(errors.iter().all(|error| error == &errors[0]));
        assert!(errors[0].contains("timed out"));
        assert_eq!(accepted.load(Ordering::SeqCst), 1);
        server.abort();
    }

    #[tokio::test]
    async fn old_reader_generation_cannot_remove_replacement() {
        let target = SocketAddr::new("192.0.2.2".parse().unwrap(), DEFAULT_NODED_PORT);
        let old = test_connection(target);
        let replacement = test_connection(target);
        let state = RwLock::new(TransportState {
            tls: None,
            tls_keys: Default::default(),
            frame_enforce: false,
            keys: None,
            desired: HashMap::from([("beta".into(), target)]),
            revision: AuthorityRevision::default(),
            connections: HashMap::from([("beta".into(), replacement.clone())]),
            attempts: HashMap::new(),
        });

        remove_connection_generation(&state, "beta", old.generation).await;
        assert_eq!(
            state
                .read()
                .await
                .connections
                .get("beta")
                .unwrap()
                .generation,
            replacement.generation
        );

        remove_connection_generation(&state, "beta", replacement.generation).await;
        assert!(!state.read().await.connections.contains_key("beta"));
    }
}
