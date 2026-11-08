// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Term-owned identity actor and verified recipient lane. Protected controls
//! share its attachment, ordered lifecycle stream and reconnect lifetime.
//! Socket candidates, broker URL and account resolve together through libs/bus.
//! Child launch handoffs retain the verified issuing endpoint; supervisor discovery stays enabled.
use super::session_fd::{LaunchFd, fresh_key};
use ::bus::native_client::session::{Deadline, ExpectedScope, GrantResult, Hello, SessionFailure};
use ::bus::native_client::{
    NodedClient, UnixConnectOptions, UnixConnectOutcome, VerifiedConnection,
};
use ::bus::native_session::*;
use ed25519_dalek::SigningKey;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicU64, Ordering},
    mpsc,
};
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

const RPC_BUDGET: Duration = Duration::from_secs(2);
const RENEW: Duration = Duration::from_secs(5);
/// Verified deliveries the broker may hold for this recipient before the reader
/// starts refusing them. Bounded so stamped authority never accumulates
/// unserved.
const VERIFIED_LANE: usize = 256;
/// Protected requests waiting on the server task. Bounded so a flood sheds with
/// a uniform refusal instead of growing a backlog of stamped authority.
const DISPATCH_QUEUE: usize = 64;

/// One protected request, with the authority that was current when it arrived.
/// A reconnect between arrival and service leaves this connection disconnected,
/// which dispatch refuses on its first check, so a stale job cannot be served
/// against a newer attachment.
struct Dispatch {
    control: Arc<dyn ControlLane>,
    connection: Arc<VerifiedConnection>,
    parent: SessionRecord,
    own: (Hello, Deadline),
    event: ::bus::native_client::VerifiedCommand,
}
const GRANT_LIFETIME: Duration = Duration::from_secs(30);
const RETRY_FLOOR: Duration = Duration::from_secs(120);
const RETRY_CAP: u32 = 3;
const STOP_BUDGET: Duration = Duration::from_secs(8);
const STARTUP_BUDGET: Duration = Duration::from_millis(900);
const PRESENCE_MS: u64 = 5 * 60 * 1000;
const MINT_FLOOR: Duration = Duration::from_secs(60);
// Five two-second budgets (connect/hello/challenge/list/prove), plus spawn slack.
const LAUNCH_MARGIN_MS: u64 = 12_000;

fn clock_ms() -> Option<u64> {
    let mut time = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // SAFETY: valid writable timespec. Unlike Instant, BOOTTIME includes suspend.
    (unsafe { libc::clock_gettime(libc::CLOCK_BOOTTIME, &mut time) } == 0).then(|| {
        (time.tv_sec as u64)
            .saturating_mul(1000)
            .saturating_add(time.tv_nsec as u64 / 1_000_000)
    })
}

async fn close_connection(connection: &VerifiedConnection) {
    if tokio::time::timeout(RPC_BUDGET, connection.client().close())
        .await
        .is_err()
    {
        eprintln!(
            "term abandoned transport close; unreconciled records remain subject to 15s lease / 30s resumption window expiry"
        );
    }
}

fn capabilities() -> Vec<Capability> {
    vec![
        Capability::ReadState,
        Capability::ReadContents,
        Capability::Input,
        Capability::Execute,
        Capability::ManageLayout,
        Capability::Terminate,
    ]
}

#[derive(Clone)]
pub struct NativeSession(
    UnboundedSender<Request>,
    Arc<std::sync::Mutex<Shared>>,
    Arc<AtomicU64>,
);

struct Shared {
    child_capabilities: Vec<Capability>,
    control: Option<std::sync::Weak<dyn ControlLane>>,
    next_id: u64,
    ready: Option<Ready>,
    panes: HashMap<u64, std::sync::Weak<PaneState>>,
    status: HashMap<u64, String>,
    diagnostic: String,
    last_diagnostic: HashMap<Option<u64>, String>,
}

impl Default for Shared {
    fn default() -> Self {
        Self {
            child_capabilities: capabilities(),
            control: None,
            next_id: 1,
            ready: None,
            panes: HashMap::new(),
            status: HashMap::new(),
            diagnostic: "native session starting; no ready launch grant".into(),
            last_diagnostic: HashMap::new(),
        }
    }
}

impl Shared {
    fn log_diagnostic(&mut self, id: Option<u64>, message: &str) -> bool {
        // Suppress repeats, but always report a changed cause, including the
        // eventual connect failure after a provisional startup timeout.
        if self
            .last_diagnostic
            .get(&id)
            .is_some_and(|last| last == message)
        {
            return false;
        }
        match id {
            Some(id) => eprintln!("term native session pane {id}: {message}"),
            None => eprintln!("term native session: {message}"),
        }
        self.last_diagnostic.insert(id, message.into());
        true
    }

    fn available(&mut self) {
        self.last_diagnostic.remove(&None);
    }
}

struct Ready {
    pane: Arc<PaneState>,
    fd: LaunchFd,
    // Only an unconsumed bundle retains its seed for descriptor replacement.
    // prepare() drops this key before handing the memfd to the one spawn.
    key: SigningKey,
    deadline: u64,
}

impl Ready {
    fn usable(&self, id: u64, now: u64) -> bool {
        self.pane.id == id
            && self.pane.live.load(Ordering::Acquire)
            && self.deadline.saturating_sub(now) >= LAUNCH_MARGIN_MS
    }
}

pub struct Supervisor {
    pub handle: NativeSession,
    worker: Option<std::thread::JoinHandle<()>>,
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    done: mpsc::Receiver<()>,
    startup: Option<mpsc::Receiver<()>>,
}

struct PaneState {
    control_ready: AtomicBool,
    id: u64,
    generation: AtomicU64,
    live: AtomicBool,
    launched: AtomicBool,
    public_key: HexBytes<32>,
    /// The child's own enrolled record, published by the actor once the
    /// binding is Attached. A request that Term forwards TO the child has to
    /// name the child's Bus name and its exact generation tuple, and neither
    /// is derivable from the pane id.
    binding: std::sync::Mutex<Option<SessionRecord>>,
}

pub struct PaneSession {
    state: Arc<PaneState>,
    handle: NativeSession,
}

impl PaneSession {
    pub(crate) fn reject_launch(self, message: &str) {
        let id = self.state.id;
        let handle = self.handle.clone();
        drop(self);
        let reason = format!("unbound (graphics-only): {message}");
        let mut shared = handle.1.lock().unwrap();
        shared.status.insert(id, reason.clone());
        shared.log_diagnostic(Some(id), &reason);
    }

    /// Called under the pane model's mutation lock, before cleanup is queued.
    pub fn revoke(&self) {
        self.state.live.store(false, Ordering::Release);
        self.handle.revoke_pane(self.state.id);
    }

    pub fn revoke_before_cleanup(&self) {
        self.state.live.store(false, Ordering::Release);
        let (tx, rx) = mpsc::sync_channel(1);
        if self
            .handle
            .0
            .send(Request::Close(self.state.id, Some(tx)))
            .is_ok()
        {
            // Cleanup worker only. Three RPCs take up to six seconds, plus
            // queued work: this is a best-effort three-second wait, NOT an
            // acknowledgement guarantee. Local invalidation already happened.
            if rx.recv_timeout(Duration::from_secs(3)).is_err() {
                eprintln!(
                    "term pane {}: abandoned revoke acknowledgement at cleanup deadline; remote records may remain until 15s lease / 30s window expiry",
                    self.state.id
                );
            }
        }
    }

    pub fn exit_notifier(&self) -> impl Fn() + Send + Sync + 'static {
        let state = self.state.clone();
        let handle = self.handle.clone();
        move || {
            state.live.store(false, Ordering::Release);
            handle.revoke_pane(state.id);
        }
    }
}

impl Drop for PaneSession {
    fn drop(&mut self) {
        self.revoke();
    }
}

enum Request {
    Provision,
    Close(u64, Option<mpsc::SyncSender<()>>),
}

impl NativeSession {
    pub fn pane_generation(&self, id: u64) -> Option<u64> {
        self.1
            .lock()
            .unwrap()
            .panes
            .get(&id)?
            .upgrade()
            .filter(|p| p.control_ready.load(Ordering::Acquire) && p.live.load(Ordering::Acquire))
            .map(|p| p.generation.load(Ordering::Acquire))
    }
    /// True once `prepare` handed pane `id` a launch grant (enrolment may
    /// still be pending); false for a graphics-only pane that got none.
    pub fn launched(&self, id: u64) -> bool {
        self.1
            .lock()
            .unwrap()
            .panes
            .get(&id)
            .and_then(|p| p.upgrade())
            .is_some_and(|p| p.live.load(Ordering::Acquire) && p.launched.load(Ordering::Acquire))
    }
    /// Installs the owner's in-process protected-command lane. The actor
    /// retains no tabs or application policy and never accepts JSON authority.
    pub fn install_control(&self, control: &Arc<dyn ControlLane>) {
        self.1.lock().unwrap().control = Some(Arc::downgrade(control));
    }
    /// The child's live enrolled record for `id` at exactly `generation`, or
    /// `None`. Every liveness condition `pane_guard` enforces is enforced here
    /// too: a forwarded request must not reach a child whose pane has been
    /// closed, whose generation has moved on, or which has not enrolled yet.
    pub fn child_binding(&self, id: u64, generation: u64) -> Option<SessionRecord> {
        let shared = self.1.lock().unwrap();
        let pane = shared.panes.get(&id)?.upgrade()?;
        if !pane.live.load(Ordering::Acquire)
            || !pane.control_ready.load(Ordering::Acquire)
            || !pane.launched.load(Ordering::Acquire)
            || pane.generation.load(Ordering::Acquire) != generation
        {
            return None;
        }
        let binding = pane.binding.lock().unwrap().clone()?;
        (binding.state == BindingState::Attached
            && binding.pane_generation == Some(DecimalU64(generation))
            && binding.pane_id == Some(DecimalU64(id)))
        .then_some(binding)
    }
    pub fn pane_guard(&self, id: u64, generation: u64) -> Arc<dyn Fn() -> bool + Send + Sync> {
        let pane = self
            .1
            .lock()
            .unwrap()
            .panes
            .get(&id)
            .cloned()
            .unwrap_or_default();
        Arc::new(move || {
            pane.upgrade().is_some_and(|p| {
                p.live.load(Ordering::Acquire)
                    && p.control_ready.load(Ordering::Acquire)
                    && p.launched.load(Ordering::Acquire)
                    && p.generation.load(Ordering::Acquire) == generation
            })
        })
    }
    /// Only physical keyboard/pointer focus paths call this, not Bus mutations.
    pub fn activity(&self) {
        if let Some(now) = clock_ms() {
            self.2.store(now / 1000 * 1000, Ordering::Release);
        }
    }
    /// Invalidates locally under a short lock, then queues remote revocation.
    /// Remote completion is asynchronous; this does not acquire the PTY mutex.
    pub fn revoke_pane(&self, id: u64) {
        let mut shared = self.1.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(state) = shared.panes.remove(&id).and_then(|p| p.upgrade()) {
            state.live.store(false, Ordering::Release);
            *state.binding.lock().unwrap() = None;
        }
        shared.status.remove(&id);
        shared.last_diagnostic.remove(&Some(id));
        drop(shared);
        let _ = self.0.send(Request::Close(id, None));
    }

    /// Consume a pre-provisioned bundle. No random generation, memfd I/O, RPC
    /// or channel wait runs on the GUI / synchronous diagnostic Bus thread.
    pub fn prepare(&self, id: u64) -> Option<(PaneSession, LaunchFd)> {
        let mut shared = self.1.lock().unwrap();
        shared.next_id = id.saturating_add(1);
        let ready = shared.ready.take();
        let ready = ready.filter(|ready| {
            let usable = clock_ms().is_some_and(|now| ready.usable(id, now));
            if !usable {
                ready.pane.live.store(false, Ordering::Release);
            }
            usable
        });
        let result = if let Some(ready) = ready {
            ready.pane.launched.store(true, Ordering::Release);
            shared
                .status
                .insert(id, "grant delivered; awaiting child enrolment".into());
            shared.diagnostic = "preparing next pane launch grant".into();
            shared.available();
            drop(ready.key); // zeroize-on-drop; no actor copy survives handoff
            Some((
                PaneSession {
                    state: ready.pane,
                    handle: self.clone(),
                },
                ready.fd,
            ))
        } else {
            if let Some(pane) = shared.panes.get(&id).and_then(|p| p.upgrade())
                && !pane.launched.load(Ordering::Acquire)
            {
                pane.live.store(false, Ordering::Release);
            }
            let reason = format!(
                "unbound (graphics-only): no usable ready grant; {}",
                shared.diagnostic
            );
            // Share the actor's outage latch, including slow startup. A pane
            // opened while it connects must not emit a second fallback line.
            if !shared.last_diagnostic.contains_key(&None) {
                shared.log_diagnostic(None, "no usable launch grant; pane is graphics-only");
            }
            shared.status.insert(id, reason);
            None
        };
        drop(shared);
        let _ = self.0.send(Request::Provision);
        result
    }

    pub fn status(&self) -> serde_json::Value {
        let shared = self.1.lock().unwrap();
        serde_json::json!({"diagnostic": shared.diagnostic, "panes": shared.status})
    }
}

impl Supervisor {
    /// One bounded wait before the FIRST TabSet open only. Taking the receiver
    /// makes repeated calls no-ops; later pane opens never wait for the actor.
    pub fn wait_startup(&mut self) {
        if let Some(startup) = self.startup.take()
            && startup.recv_timeout(STARTUP_BUDGET).is_err()
        {
            let mut shared = self.handle.1.lock().unwrap();
            if shared.ready.is_none() {
                shared.log_diagnostic(
                    None,
                    "native startup deadline elapsed; no ready launch grant",
                );
            }
        }
    }
    pub fn start() -> Result<Self, String> {
        // Account lookup is trusted system configuration, not the current UID
        // or the ownership of an attacker-selected socket. No numeric default.
        let (mut options, url) = ::bus::client_helpers::BrokerEnvironment::capture()
            .resolve()
            .map_err(|error| format!("{error:#}"))?;
        options.require_native_session = true;
        // Opt in to the bounded verified lane. An unbounded one would let a
        // flood of stamped requests accumulate faster than they can be served,
        // which is authority held in memory that nothing has decided on yet.
        // Bounded, the reader refuses the overflow and reports dropped id-less
        // notices as a gap, both of which the actor settles above.
        options.incoming_capacity = Some(VERIFIED_LANE);
        Self::with_options(options, url)
    }

    /// Term's pane sessions are always default-open: every caller on the Bus
    /// may read and drive them (open-Bus decision, 2026-10-26). There is no
    /// strict mode; integrity checks on stamped provenance still apply.
    pub fn with_options(options: UnixConnectOptions, url: String) -> Result<Self, String> {
        Self::with_capabilities(options, url, capabilities())
    }

    fn with_capabilities(
        options: UnixConnectOptions,
        url: String,
        child_capabilities: Vec<Capability>,
    ) -> Result<Self, String> {
        let key = fresh_key().map_err(|e| e.to_string())?;
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let shared = Arc::new(std::sync::Mutex::new(Shared::default()));
        shared.lock().unwrap().child_capabilities = child_capabilities;
        let actor_shared = shared.clone();
        let activity = Arc::new(AtomicU64::new(clock_ms().unwrap_or(0)));
        let actor_activity = activity.clone();
        let (startup_tx, startup) = mpsc::sync_channel(1);
        let (stop, stopped) = tokio::sync::oneshot::channel();
        let (done_tx, done) = mpsc::sync_channel(1);
        let worker = std::thread::Builder::new()
            .name("term-native-session".into())
            .spawn(move || {
                let runtime = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .expect("session runtime");
                runtime.block_on(async move {
                    let mut actor = Actor {
                        options,
                        url,
                        key,
                        connection: None,
                        parent: None,
                        own_lease: None,
                        children: HashMap::new(),
                        shared: actor_shared,
                        provisioned: None,
                        pool_key: None,
                        activity: actor_activity,
                        startup: Some(startup_tx),
                        #[cfg(test)]
                        grant_creates: 0,
                        #[cfg(test)]
                        faults: TestFaults::default(),
                    };
                    // Cancellation interrupts even a long reconciliation batch.
                    tokio::select! {
                        _ = stopped => {},
                        _ = async { actor.connect().await; actor.run(rx).await; } => {},
                    }
                    actor.shutdown().await;
                });
                runtime.shutdown_timeout(Duration::from_millis(100));
                let _ = done_tx.send(());
            })
            .map_err(|e| e.to_string())?;
        Ok(Self {
            handle: NativeSession(tx, shared, activity),
            worker: Some(worker),
            stop: Some(stop),
            done,
            startup: Some(startup),
        })
    }
}

impl super::native_session::SessionIssuer for NativeSession {
    fn prepare(&self, id: u64) -> Option<(super::native_session::PaneSession, LaunchFd)> {
        let (pane, fd) = NativeSession::prepare(self, id)?;
        let pane = Arc::new(pane);
        let exited = Arc::new(pane.exit_notifier());
        let revoke_owner = pane.clone();
        let cleanup_owner = pane.clone();
        let rejected_owner = self.clone();
        let lease = super::native_session::PaneSession::new(
            exited,
            Arc::new(move || revoke_owner.revoke()),
            Arc::new(move || cleanup_owner.revoke_before_cleanup()),
            Arc::new(move |message| {
                let reason = format!("unbound (graphics-only): {message}");
                let mut shared = rejected_owner.1.lock().unwrap();
                shared.status.insert(id, reason.clone());
                shared.log_diagnostic(Some(id), &reason);
            }),
        );
        Some((lease, fd))
    }
}

impl Drop for Supervisor {
    fn drop(&mut self) {
        if let Some(stop) = self.stop.take() {
            let _ = stop.send(());
        }
        if let Some(worker) = self.worker.take() {
            if self.done.recv_timeout(STOP_BUDGET).is_ok() {
                if worker.is_finished() {
                    let _ = worker.join();
                }
                // The completion signal precedes thread return. If still
                // finishing, dropping the handle detaches instead of joining.
            } else {
                // Dropping a JoinHandle detaches it. A stuck OS/runtime thread
                // cannot hold Term exit beyond this deadline.
                eprintln!(
                    "term native-session shutdown deadline exceeded; abandoned revokes leave records to the 15s lease / 30s resumption window expiry"
                );
            }
        }
    }
}

struct Child {
    pane: Arc<PaneState>,
    record: Option<SessionRecord>,
    retry: Retry,
    pending: Option<PendingGrant>,
}

impl Child {
    /// The pane's published binding is exactly "the current record, while it is
    /// Attached" — the one thing a forwarded request needs and the one thing
    /// `pane_generation` cannot answer, since a name and a generation tuple are
    /// not derivable from a pane id.
    ///
    /// Both the grant fetch and the lifecycle notice change that record, and
    /// BOTH must republish. The notice is the fast path to Attached; publishing
    /// only from the fetch left the binding empty for the whole life of a child
    /// whose fetch never ran again, which reads at the far end as "this pane
    /// has no shell".
    fn publish_binding(&self) {
        *self.pane.binding.lock().unwrap() = self
            .record
            .as_ref()
            .filter(|record| record.state == BindingState::Attached)
            .cloned();
    }
}

#[derive(Default)]
struct Retry {
    enrolled: bool,
    external: bool,
    attempts: u32,
    due: Option<Instant>,
    // Not reset by reconnect/gap rearm, or by broker epoch replacement.
    mint_after: Option<Instant>,
}
impl Retry {
    fn rearm(&mut self) {
        self.external = true;
        self.attempts = 0;
        self.due = None;
    }
    fn expired(&mut self, now: Instant) {
        if self.enrolled && self.attempts < RETRY_CAP {
            self.due
                .get_or_insert(now + RETRY_FLOOR * (1 << self.attempts));
        }
    }
    fn take(&mut self, now: Instant) -> bool {
        if self.external {
            self.external = false;
            self.due = None;
            return true;
        }
        if self.enrolled && self.attempts < RETRY_CAP && self.due.is_some_and(|due| now >= due) {
            self.attempts += 1;
            self.due = None;
            return true;
        }
        false
    }
}

struct PendingGrant {
    deadline: u64,
    expires_ms: Option<DecimalU64>,
}
impl PendingGrant {
    fn usable(&self, grant: &SessionGrant, now: u64) -> bool {
        // Broker timestamps are opaque across time namespaces (BUS-016).
        // Check the grant's own state and exact expiry value, with a conservative
        // local 30s deadline starting BEFORE create, including lost-ACK recovery.
        grant.state == GrantState::Pending
            && grant.expires_ms.0 != 0
            && self
                .expires_ms
                .is_none_or(|expires| expires == grant.expires_ms)
            && now < self.deadline
    }
}

struct Actor {
    #[cfg(test)]
    grant_creates: usize,
    #[cfg(test)]
    faults: TestFaults,
    options: UnixConnectOptions,
    url: String,
    key: SigningKey,
    connection: Option<Arc<VerifiedConnection>>,
    parent: Option<SessionRecord>,
    /// The recipient's own conservative deadline and the context it is bound
    /// to, refreshed by the renew cadence so no protected request ever
    /// establishes it inside its own resolution (PROP-024).
    own_lease: Option<(Hello, Deadline)>,
    children: HashMap<u64, Child>,
    shared: Arc<std::sync::Mutex<Shared>>,
    provisioned: Option<u64>,
    // Withdrawn/unpublished pool keys only. A consumed launch retains no copy.
    pool_key: Option<(u64, SigningKey)>,
    activity: Arc<AtomicU64>,
    startup: Option<mpsc::SyncSender<()>>,
}

#[cfg(test)]
#[derive(Default)]
struct TestFaults {
    memfd: bool,
    create_ack: bool,
    fetch: bool,
}

type ResultSession<T> = Result<T, SessionFailure>;
async fn bounded<T>(
    future: impl std::future::Future<Output = ResultSession<T>>,
) -> ResultSession<T> {
    tokio::time::timeout(RPC_BUDGET, future)
        .await
        .unwrap_or(Err(SessionFailure::InvalidResponse))
}
fn forbidden(error: &SessionFailure) -> bool {
    matches!(error, SessionFailure::Refused { error, .. } if error.error_code == ErrorCode::Forbidden)
}

impl Actor {
    fn control(&self) -> Option<Arc<dyn ControlLane>> {
        self.shared
            .lock()
            .unwrap()
            .control
            .as_ref()
            .and_then(std::sync::Weak::upgrade)
    }
    /// The one place a protected request is refused without being served.
    /// Reached from three saturation points — the lane dropped it, Term's own
    /// queue is full, or there is no attachment to serve it against — and they
    /// share this so the refusals cannot drift apart. Detached so a saturated
    /// or slow peer never blocks the identity loop, which is the whole reason
    /// serving moved off it.
    fn refuse(&self, event: ::bus::native_client::VerifiedCommand, code: &'static str) {
        let Some(connection) = self.connection.clone() else {
            return;
        };
        tokio::spawn(async move {
            let reply = ControlReply::error(code);
            let _ = tokio::time::timeout(
                RPC_BUDGET,
                connection
                    .client()
                    .respond(event.command(), reply.rc, &reply.body),
            )
            .await;
        });
    }

    /// Discard cached lifecycle authority and resynchronise. Reached from a
    /// broker-signalled gap and from a local inbox overflow, which are the same
    /// event: either way a lifecycle notice may have been missed, and nothing
    /// may be resolved against the cached authority until it is re-earned.
    async fn session_gap(&mut self) {
        self.own_lease = None;
        if let Some(control) = self.control() {
            control.invalidate(None);
        }
        self.reconcile().await;
        self.provision().await;
    }
    /// Refresh this attachment and the conservative local deadline it
    /// establishes. Every renew site goes through here so the deadline the
    /// control lane reads can never be older than the lease that authorises
    /// it; a failed renew drops it rather than leaving stale authority behind.
    async fn renew_parent(&mut self) -> bool {
        let refreshed = if let (Some(connection), Some(parent)) = (&self.connection, &self.parent) {
            bounded(connection.session_renew_lease(parent.reference()))
                .await
                .ok()
        } else {
            None
        };
        match refreshed {
            Some((result, hello, deadline)) => {
                self.parent = Some(result.record);
                self.own_lease = Some((hello, deadline));
                true
            }
            None => {
                self.own_lease = None;
                false
            }
        }
    }
    fn launch_fd(&mut self, grant: &GrantResult) -> std::io::Result<LaunchFd> {
        #[cfg(test)]
        if std::mem::take(&mut self.faults.memfd) {
            return Err(std::io::Error::other("injected memfd allocation failure"));
        }
        let connection = self
            .connection
            .as_ref()
            .ok_or_else(|| std::io::Error::other("launch grant has no issuing connection"))?;
        Ok(LaunchFd::new(grant, &self.pool_key.as_ref().unwrap().1)?
            .with_endpoint(connection.endpoint()))
    }
    fn diagnostic(&self, id: Option<u64>, message: impl Into<String>) {
        let message = message.into();
        let mut shared = self.shared.lock().unwrap();
        // Reconnect retries must not flood stderr on a graphics-only desktop.
        shared.log_diagnostic(id, &message);
        if let Some(id) = id {
            if let Some(status) = shared.status.get_mut(&id) {
                *status = message;
            } else if shared
                .ready
                .as_ref()
                .is_some_and(|ready| ready.pane.id == id)
            {
                shared.diagnostic = message;
            }
        } else {
            shared.diagnostic = message;
        }
    }

    fn present(&self, now_ms: u64) -> bool {
        let last = self.activity.load(Ordering::Acquire);
        last != 0 && now_ms >= last && now_ms - last < PRESENCE_MS
    }

    /// Withdraw under the same lock used by prepare, BEFORE any reconciliation
    /// await. Old latches stay invalid; republishing uses a new unlaunched state.
    fn withdraw_ready(&mut self) {
        let mut shared = self.shared.lock().unwrap();
        if let Some(ready) = shared.ready.take() {
            ready.pane.live.store(false, Ordering::Release);
            let pane = Arc::new(PaneState {
                id: ready.pane.id,
                generation: AtomicU64::new(ready.pane.generation.load(Ordering::Acquire)),
                live: AtomicBool::new(true),
                launched: AtomicBool::new(false),
                control_ready: AtomicBool::new(false),
                public_key: ready.pane.public_key,
                binding: std::sync::Mutex::new(None),
            });
            if let Some(child) = self.children.get_mut(&pane.id) {
                child.pane = pane.clone();
            }
            shared.panes.insert(pane.id, Arc::downgrade(&pane));
            self.pool_key = Some((pane.id, ready.key));
            // Dropping ready.fd closes the old descriptor. Its offset, scope
            // and generation can never escape through prepare after withdrawal.
        }
    }

    async fn provision(&mut self) {
        if let Some(now_ms) = clock_ms() {
            self.provision_at(Instant::now(), now_ms).await;
        }
    }

    async fn provision_at(&mut self, now: Instant, now_ms: u64) {
        let id = self.shared.lock().unwrap().next_id;
        if self.connection.is_none() {
            return;
        }
        if self
            .shared
            .lock()
            .unwrap()
            .ready
            .as_ref()
            .is_some_and(|r| clock_ms().is_some_and(|clock| r.usable(id, clock)))
        {
            return;
        }
        self.withdraw_ready();
        if let Some((old_id, _)) = &self.pool_key
            && (*old_id != id
                || self
                    .children
                    .get(old_id)
                    .is_none_or(|child| !child.pane.live.load(Ordering::Acquire)))
        {
            let old_id = *old_id;
            self.pool_key = None;
            self.close_child(old_id).await;
        }
        if !self.present(now_ms) {
            self.diagnostic(None, "launch pool held: waiting for user presence");
            return;
        }
        if self.pool_key.is_none() {
            if self.children.len() >= 128 {
                self.provisioned = None;
                self.diagnostic(None, "revocation backlog full; opening panes unbound");
                return;
            }
            // MAX_TABS coincides with the default pending quota (32); this
            // look-ahead slot also spends quota, and operators may lower it.
            let key = match fresh_key() {
                Ok(key) => key,
                Err(error) => {
                    self.provisioned = None;
                    self.diagnostic(None, format!("launch key unavailable: {error}"));
                    return;
                }
            };
            let pane = Arc::new(PaneState {
                id,
                generation: AtomicU64::new(1),
                live: AtomicBool::new(true),
                launched: AtomicBool::new(false),
                control_ready: AtomicBool::new(false),
                public_key: HexBytes(key.verifying_key().to_bytes()),
                binding: std::sync::Mutex::new(None),
            });
            self.children.insert(
                id,
                Child {
                    pane: pane.clone(),
                    record: None,
                    retry: Retry::default(),
                    pending: None,
                },
            );
            self.shared
                .lock()
                .unwrap()
                .panes
                .insert(id, Arc::downgrade(&pane));
            self.pool_key = Some((id, key));
            self.provisioned = None;
        }
        let child = &self.children[&id];
        let cached_window = child
            .record
            .as_ref()
            .is_some_and(|r| r.state == BindingState::Pending)
            && child.pending.as_ref().is_some_and(|p| {
                p.expires_ms.is_some()
                    && clock_ms()
                        .is_some_and(|clock| p.deadline.saturating_sub(clock) >= LAUNCH_MARGIN_MS)
            });
        if self.provisioned == Some(id)
            && !cached_window
            && child.retry.mint_after.is_some_and(|after| now < after)
        {
            self.diagnostic(
                None,
                "launch pool deliberately waiting for the 60s mint floor",
            );
            return;
        }
        let previous_mint = child.retry.mint_after;
        self.children.get_mut(&id).unwrap().retry.rearm();
        self.provisioned = Some(id);
        let result = self.grant_at(id, now, false).await;
        match result {
            Ok(grant) => {
                let deadline = self.children[&id]
                    .pending
                    .as_ref()
                    .expect("pending pool grant")
                    .deadline;
                if clock_ms().is_none_or(|clock| deadline.saturating_sub(clock) < LAUNCH_MARGIN_MS)
                {
                    self.diagnostic(
                        None,
                        "launch pool deliberately waiting for replacement of a short-window grant",
                    );
                    return;
                }
                let fd = match self.launch_fd(&grant) {
                    Ok(fd) => fd,
                    Err(error) => {
                        // Keep the known grant/key: the next tick retries only
                        // memfd delivery, not a new mint. No failed-fd dead latch.
                        self.provisioned = None;
                        self.diagnostic(
                            None,
                            format!("launch memfd unavailable; delivery will retry: {error}"),
                        );
                        return;
                    }
                };
                let pane = self.children[&id].pane.clone();
                let mut shared = self.shared.lock().unwrap();
                if shared.next_id == id && pane.live.load(Ordering::Acquire) {
                    let (_, key) = self.pool_key.take().unwrap();
                    shared.ready = Some(Ready {
                        pane,
                        key,
                        fd,
                        deadline,
                    });
                    shared.diagnostic = "next pane launch grant ready".into();
                    shared.available();
                    if let Some(startup) = self.startup.take() {
                        let _ = startup.send(());
                    }
                    return;
                }
            }
            Err(error) => {
                // The mutation path reserves the floor only when a create was
                // sent, and rolls it back on a definitive broker refusal.
                // Fetch/transport failures before create are free to retry.
                if self.children[&id].retry.mint_after == previous_mint {
                    self.provisioned = None;
                }
                let message = match error {
                    SessionFailure::Refused { error, .. }
                        if error.error_code == ErrorCode::ResourceLimit =>
                    {
                        let kind = if error.details.get("reason").and_then(|v| v.as_str())
                            == Some("grant_limit")
                        {
                            "pending-grant quota"
                        } else {
                            "resource quota"
                        };
                        format!("broker {kind} exhausted; delivery will retry under presence")
                    }
                    SessionFailure::LeaseExpired => {
                        "launch pool deliberately waiting for grant expiry / the 60s mint floor"
                            .into()
                    }
                    error => format!("launch grant unavailable: {error}"),
                };
                self.diagnostic(None, message);
                return;
            }
        }
        // A concurrent open consumed this slot unbound while it was withdrawn.
        self.pool_key = None;
        self.close_child(id).await;
    }

    async fn connect(&mut self) {
        // No deadline survives a reconnect: it is bound to the old connection.
        self.own_lease = None;
        for child in self.children.values() {
            child.pane.control_ready.store(false, Ordering::Release);
        }
        if let Some(control) = self.control() {
            control.invalidate(None);
        }
        self.withdraw_ready();
        if let Some(old) = self.connection.take() {
            close_connection(&old).await;
        }
        let connection = tokio::time::timeout(
            RPC_BUDGET,
            NodedClient::connect_unix("", &self.url, &self.options, None),
        )
        .await;
        let Ok(Ok(UnixConnectOutcome::VerifiedUnix(connection))) = connection else {
            self.diagnostic(
                None,
                "broker/profile unavailable; opening panes unbound (graphics-only)",
            );
            if let Some(startup) = self.startup.take() {
                let _ = startup.send(());
            }
            return;
        };
        connection.client().set_commands(command_manifest());
        let public_key = HexBytes(self.key.verifying_key().to_bytes());
        let hello = match bounded(connection.session_hello()).await {
            Ok(hello) => hello,
            Err(error) => {
                self.diagnostic(None, format!("session hello unavailable: {error}"));
                if let Some(startup) = self.startup.take() {
                    let _ = startup.send(());
                }
                close_connection(&connection).await;
                return;
            }
        };
        // Always reconcile by retained key first, including uncertain allocate
        // ACKs. Only an authenticated, definitive absence permits allocation.
        let result = match bounded(connection.session_challenge_key(public_key)).await {
            Ok(challenge) => {
                if challenge.wake_error.is_some() {
                    eprintln!("term session wake registration unavailable");
                }
                let expected = ExpectedScope {
                    broker_epoch: hello.broker_epoch,
                    purpose: Purpose::Resume,
                    unix_uid: unsafe { libc::geteuid() },
                    parent_key_hash: None,
                    pane_id: None,
                    pane_high_water: None,
                    role: Role::Term,
                    public_key_hash: HexBytes(Sha256::digest(public_key.0).into()),
                    capabilities_hash: HexBytes(
                        Sha256::digest(
                            encode_capabilities(&capabilities()).expect("fixed capabilities"),
                        )
                        .into(),
                    ),
                };
                match challenge.sign(&self.key, &expected) {
                    Ok(proof) => bounded(connection.session_prove(&proof)).await,
                    Err(error) => Err(error),
                }
            }
            Err(error) if forbidden(&error) => {
                bounded(connection.session_allocate(&self.key, Policy::DefaultOpen)).await
            }
            Err(error) => Err(error),
        };
        match result {
            Ok(result) => {
                let replacement = self.parent.as_ref().is_none_or(|old| {
                    old.record_id != result.record.record_id
                        || old.broker_epoch != result.record.broker_epoch
                });
                self.parent = Some(result.record);
                self.connection = Some(Arc::new(connection));
                if replacement {
                    if self.pool_key.as_ref().is_some_and(|(id, _)| {
                        self.children
                            .get(id)
                            .is_none_or(|child| !child.pane.live.load(Ordering::Acquire))
                    }) {
                        self.pool_key = None;
                    }
                    self.children
                        .retain(|_, child| child.pane.live.load(Ordering::Acquire));
                    for child in self.children.values_mut() {
                        child.record = None;
                        child.pending = None;
                        child.pane.generation.store(1, Ordering::Release);
                    }
                }
                self.reconcile().await;
                // Re-earn the deadline now rather than at the next renew tick.
                // Without this a reconnect denies every protected request for
                // up to five seconds even though the attachment is already
                // live; reconcile only renews when it had children to visit.
                self.renew_parent().await;
            }
            Err(error) => {
                self.diagnostic(None, format!("identity recovery unavailable: {error}"));
                if let Some(startup) = self.startup.take() {
                    let _ = startup.send(());
                }
                close_connection(&connection).await;
            }
        }
    }

    async fn grant(&mut self, id: u64) -> ResultSession<GrantResult> {
        self.grant_at(id, Instant::now(), false).await
    }

    async fn grant_at(
        &mut self,
        id: u64,
        retry_now: Instant,
        prepaid: bool,
    ) -> ResultSession<GrantResult> {
        #[cfg(test)]
        if std::mem::take(&mut self.faults.fetch) {
            return Err(SessionFailure::InvalidResponse);
        }
        let connection = self
            .connection
            .as_ref()
            .ok_or(SessionFailure::InvalidResponse)?;
        let child = self
            .children
            .get_mut(&id)
            .ok_or(SessionFailure::InvalidResponse)?;
        if !child.pane.live.load(Ordering::Acquire) {
            return Err(SessionFailure::ScopeMismatch);
        }
        // Fetch precedes every (re-)mint: a lost create ACK must never lead to
        // a second mutation before the original outcome has been reconciled.
        match bounded(connection.session_grant_fetch(child.pane.public_key)).await {
            Ok(found)
                if found.grant.state == GrantState::Consumed
                    && matches!(
                        found.record.state,
                        BindingState::Attached | BindingState::Suspended
                    )
                    || found.record.state == BindingState::Pending
                        && child.pending.as_ref().is_some_and(|p| {
                            clock_ms().is_some_and(|now| p.usable(&found.grant, now))
                        }) =>
            {
                child.retry.enrolled |= found.grant.state == GrantState::Consumed;
                child.retry.due = None;
                if let Some(pending) = &mut child.pending {
                    pending.expires_ms = Some(found.grant.expires_ms);
                }
                child.pane.generation.store(
                    found
                        .record
                        .pane_generation
                        .ok_or(SessionFailure::InvalidResponse)?
                        .0,
                    Ordering::Release,
                );
                child.pane.control_ready.store(
                    found.record.state == BindingState::Attached,
                    Ordering::Release,
                );
                child.record = Some(found.record.clone());
                // Only an ATTACHED record is published. A pending or suspended
                // one names a child that cannot answer, and forwarding to it
                // would turn a known refusal into a timeout.
                child.publish_binding();
                return Ok(found);
            }
            Ok(found) => {
                child.retry.enrolled |= found.grant.state == GrantState::Consumed;
                // A locally exhausted conservative window must not cause a
                // duplicate mint while the broker still owns a pending grant.
                if found.grant.state == GrantState::Pending {
                    return Err(SessionFailure::LeaseExpired);
                }
                let generation = found
                    .record
                    .pane_generation
                    .ok_or(SessionFailure::InvalidResponse)?
                    .0
                    .checked_add(1)
                    .ok_or(SessionFailure::InvalidResponse)?;
                child
                    .pane
                    .generation
                    .fetch_max(generation, Ordering::AcqRel);
            }
            Err(error) if forbidden(&error) => {
                if let Some(record) = &child.record {
                    let next = record
                        .pane_generation
                        .ok_or(SessionFailure::InvalidResponse)?
                        .0
                        .checked_add(1)
                        .ok_or(SessionFailure::InvalidResponse)?;
                    child.pane.generation.fetch_max(next, Ordering::AcqRel);
                }
            }
            Err(error) => return Err(error),
        }
        if !child.pane.live.load(Ordering::Acquire) {
            return Err(SessionFailure::ScopeMismatch);
        }
        // Charge BEFORE sending: uncertain ACKs and quota failures spend an
        // attempt too. A fetch of an existing grant never spends a name.
        if let Some(after) = child.retry.mint_after
            && retry_now < after
        {
            // An external event may re-arm credit but cannot reset this floor.
            child.retry.due = Some(after);
            return Err(SessionFailure::LeaseExpired);
        }
        if !prepaid && !child.retry.take(retry_now) {
            return Err(SessionFailure::ScopeMismatch);
        }
        let args = GrantCreateArgs {
            parent: self
                .parent
                .as_ref()
                .ok_or(SessionFailure::InvalidResponse)?
                .reference(),
            pane_id: DecimalU64(id),
            pane_generation: DecimalU64(child.pane.generation.load(Ordering::Acquire)),
            public_key: child.pane.public_key,
            role: Role::PaneShell,
            capabilities: self.shared.lock().unwrap().child_capabilities.clone(),
        };
        let previous_pending = child.pending.take();
        child.pending = Some(PendingGrant {
            deadline: clock_ms()
                .ok_or(SessionFailure::InvalidResponse)?
                .saturating_add(GRANT_LIFETIME.as_millis() as u64),
            expires_ms: None,
        });
        #[cfg(test)]
        {
            self.grant_creates += 1;
        }
        let previous_mint = child.retry.mint_after;
        child.retry.mint_after = Some(retry_now.max(Instant::now()) + MINT_FLOOR);
        let result = match bounded(connection.session_grant_create(&args)).await {
            Ok(result) => result,
            Err(error @ SessionFailure::Refused { .. }) => {
                child.retry.mint_after = previous_mint;
                child.pending = previous_pending;
                return Err(error);
            }
            Err(error) => {
                child.retry.mint_after = Some(retry_now.max(Instant::now()) + MINT_FLOOR);
                return Err(error); // uncertain ACK retains the floor
            }
        };
        // Start the floor at receipt, not request start: RPC latency must not
        // shorten the interval between actual broker mints below sixty seconds.
        child.retry.mint_after = Some(retry_now.max(Instant::now()) + MINT_FLOOR);
        #[cfg(test)]
        if std::mem::take(&mut self.faults.create_ack) {
            return Err(SessionFailure::InvalidResponse);
        }
        if result.record.state != BindingState::Pending
            || !clock_ms()
                .is_some_and(|now| child.pending.as_ref().unwrap().usable(&result.grant, now))
        {
            return Err(SessionFailure::InvalidResponse);
        }
        child.pending.as_mut().unwrap().expires_ms = Some(result.grant.expires_ms);
        child.pane.control_ready.store(false, Ordering::Release);
        // A fresh grant supersedes whatever was bound before it; the old name
        // must stop being addressable the moment the generation moves.
        *child.pane.binding.lock().unwrap() = None;
        child.record = Some(result.record.clone());
        Ok(result)
    }

    async fn reconcile(&mut self) {
        self.withdraw_ready();
        let next = self.shared.lock().unwrap().next_id;
        if !self.children.contains_key(&next) {
            self.provisioned = None;
        }
        let ids: Vec<_> = self.children.keys().copied().collect();
        for id in ids {
            if !self.children[&id].pane.live.load(Ordering::Acquire) {
                self.close_child(id).await;
            } else if !self.children[&id].pane.launched.load(Ordering::Acquire) {
                // Only the presence-gated pool path may refresh an unconsumed bundle.
                continue;
            } else {
                // Exactly one mint opportunity per successful reconnect/gap.
                self.children.get_mut(&id).unwrap().retry.rearm();
                match self.grant(id).await {
                    Ok(_) => {},
                    Err(SessionFailure::LeaseExpired) => self.diagnostic(Some(id), "deliberately waiting for pending grant expiry / the 60s mint floor before replacement"),
                    Err(error) => {
                        self.diagnostic(Some(id), format!("child grant reconciliation: {error}"))
                    }
                }
            }
            // A slow reconciliation batch cannot starve the parent's lease.
            self.renew_parent().await;
        }
    }

    async fn close_child(&mut self, id: u64) {
        let Some(child) = self.children.get_mut(&id) else {
            return;
        };
        child.pane.live.store(false, Ordering::Release);
        // A forbidden fetch can also mean a suspended parent. Confirm this
        // attachment is live before treating absence as successful cleanup.
        // This also keeps large close batches from starving the parent lease.
        if !self.renew_parent().await {
            return;
        }
        let (Some(connection), Some(child)) =
            (self.connection.as_ref(), self.children.get_mut(&id))
        else {
            return;
        };
        // Attachment generation may have advanced since the initial grant.
        // Fetch the current owned key's reference before parent-initiated revoke.
        match bounded(connection.session_grant_fetch(child.pane.public_key)).await {
            Ok(found) => {
                child.pane.control_ready.store(
                    found.record.state == BindingState::Attached,
                    Ordering::Release,
                );
                child.record = Some(found.record.clone());
                if bounded(connection.session_revoke(found.record.reference()))
                    .await
                    .is_ok()
                {
                    self.children.remove(&id);
                }
            }
            Err(error) if forbidden(&error) => {
                self.children.remove(&id);
            }
            Err(_) => {}
        }
    }

    async fn retry_due(&mut self, now: Instant) {
        let ids: Vec<_> = self
            .children
            .iter()
            .filter(|(_, c)| {
                c.pane.live.load(Ordering::Acquire)
                    && c.pane.launched.load(Ordering::Acquire)
                    && c.retry.due.is_some_and(|due| now >= due)
            })
            .map(|(id, _)| *id)
            .collect();
        for id in ids {
            // Scheduled retries spend their budget before even fetching, so a
            // transport failure cannot become a five-second fetch/re-mint loop.
            if !self.children.get_mut(&id).unwrap().retry.take(now) {
                continue;
            }
            match self.grant_at(id, now, true).await {
                Ok(_) => {}
                Err(error) => {
                    self.children.get_mut(&id).unwrap().retry.expired(now);
                    let held = self.children[&id].retry.due.is_none();
                    self.diagnostic(
                        Some(id),
                        format!(
                            "bounded child re-grant failed: {error}; {}",
                            if held {
                                "held until reconnect/gap (attempt cap reached)"
                            } else {
                                "next backoff scheduled"
                            }
                        ),
                    );
                }
            }
            // As in reconciliation, a slow batch must not starve the parent.
            self.renew_parent().await;
        }
    }

    async fn run(&mut self, mut requests: UnboundedReceiver<Request>) {
        // Protected requests are served by a sibling task, not on this loop.
        // A bound caller's request can cost a lease-check round trip plus a
        // reply, and this loop is the one that renews the attachment and drains
        // revocation notices; serving inline let one slow caller push renew past
        // its cadence and leave revocations undelivered. The queue is bounded so
        // a flood sheds instead of growing, and serving is serial because
        // dispatch already serialises on the model and policy locks.
        let (dispatch_tx, mut dispatch_rx) = tokio::sync::mpsc::channel::<Dispatch>(DISPATCH_QUEUE);
        let server = tokio::spawn(async move {
            while let Some(job) = dispatch_rx.recv().await {
                let reply = job
                    .control
                    .dispatch(&job.connection, &job.parent, &job.own, &job.event)
                    .await;
                let _ = tokio::time::timeout(
                    RPC_BUDGET,
                    job.connection
                        .client()
                        .respond(job.event.command(), reply.rc, &reply.body),
                )
                .await;
            }
        });
        let mut renew = tokio::time::interval(RENEW);
        renew.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        // Revocation wakes this lane; it is never woken by a clock. The wake
        // appears once the render thread installs the control surface, so it
        // is picked up lazily rather than required at actor startup.
        let mut control_wake: Option<Arc<tokio::sync::Notify>> = None;
        loop {
            if control_wake.is_none() {
                control_wake = self.control().map(|control| control.wake());
            }
            tokio::select! {
                biased;
                _ = renew.tick() => {
                    if !self.renew_parent().await { self.connect().await; }
                    // Notices schedule retries; this tick does not poll grants.
                    self.retry_due(Instant::now()).await;
                    let closing: Vec<_> = self.children.iter().filter(|(_, c)| !c.pane.live.load(Ordering::Acquire)).map(|(id, _)| *id).collect();
                    for id in closing { self.close_child(id).await; }
                    self.provision().await;
                    // Backstop only, on a cadence BROKER-020 already requires:
                    // a permit that merely aged out queues its notice here
                    // rather than waiting for the next revocation to wake us.
                    if let (Some(control), Some(connection)) = (self.control(), &self.connection) {
                        control.flush_notices(connection).await;
                    }
                }
                _ = async { match &control_wake { Some(wake) => wake.notified().await, None => std::future::pending().await } } => {
                    if let (Some(control), Some(connection)) = (self.control(), &self.connection) {
                        control.flush_notices(connection).await;
                    }
                }
                request = requests.recv() => match request {
                    Some(Request::Provision) => self.provision().await,
                    Some(Request::Close(id, ack)) => {
                        self.close_child(id).await;
                        if let Some(ack) = ack { let _ = ack.send(()); }
                    }
                    None => break,
                },
                event = async { self.connection.as_ref().expect("guarded connection").recv_shared().await }, if self.connection.is_some() => {
                    match event {
                        Some(event) => {
                            // The lane says what this delivery is before the command
                            // does. A Gap is the absence of a delivery — the
                            // dropped envelope may have been a lifecycle notice,
                            // so it takes the identical path to a broker-signalled
                            // gap. A Refuse carries correlation and nothing
                            // admissible; the OWNER answers it, because the reader
                            // task must never write.
                            if event.delivery() == ::bus::native_client::Delivery::Gap {
                                self.session_gap().await;
                                continue;
                            }
                            if event.delivery() == ::bus::native_client::Delivery::Refuse {
                                self.refuse(event, "RESOURCE_LIMIT");
                                continue;
                            }
                            let command = event.command();
                            if command.command == "noded.session.lifecycle.gap" {
                                self.session_gap().await;
                            } else if command.command == "noded.session.lifecycle" {
                                self.notice(&command.body).await;
                            } else if let (Some(control), Some(connection), Some(parent), Some(own)) = (self.control(), &self.connection, &self.parent, &self.own_lease) {
                                // Hand the request to the server task and go
                                // straight back to the select. Serving it here
                                // would hold this loop for a lease check plus a
                                // reply, and this loop is what renews the lease
                                // and drains revocation notices.
                                let job = Dispatch {
                                    control,
                                    connection: connection.clone(),
                                    parent: parent.clone(),
                                    own: own.clone(),
                                    event,
                                };
                                if let Err(tokio::sync::mpsc::error::TrySendError::Full(job)) = dispatch_tx.try_send(job) {
                                    // Term's own queue is full. Same shape as the
                                    // lane's refusal above and written the same
                                    // way, so there is one refusal mechanism
                                    // rather than two that could drift apart.
                                    self.refuse(job.event, "RESOURCE_LIMIT");
                                }
                            } else {
                                self.refuse(event, "FORBIDDEN");
                            }
                        }
                        None => { self.connect().await; }
                    }
                }
            }
        }
        // Nothing queued may be served after the identity loop stops: shutdown
        // is about to revoke the records those jobs would be answered against.
        server.abort();
    }

    async fn shutdown(&mut self) {
        self.own_lease = None;
        if let Some(control) = self.control() {
            control.invalidate(None);
        }
        // No unconsumed launch key or descriptor survives shutdown.
        self.shared.lock().unwrap().ready = None;
        self.pool_key = None;
        // Children first, then the parent (which recursively revokes anything
        // whose individual revoke raced a child resumption). Bound total shutdown.
        let children_closed = tokio::time::timeout(Duration::from_secs(3), async {
            let ids: Vec<_> = self.children.keys().copied().collect();
            for id in ids {
                self.close_child(id).await;
            }
        })
        .await;
        if children_closed.is_err() {
            eprintln!(
                "term abandoned child revoke batch; remaining records rely on parent revoke or 15s lease / 30s window expiry"
            );
        }
        if let (Some(connection), Some(parent)) = (&self.connection, &self.parent) {
            if bounded(connection.session_revoke(parent.reference()))
                .await
                .is_err()
            {
                eprintln!(
                    "term parent revoke unconfirmed; remaining records rely on 15s lease / 30s window expiry"
                );
            }
            close_connection(connection).await;
        } else {
            eprintln!(
                "term stopped without a current parent connection (possibly during connect); skipped parent revoke, leaving any allocated records to 15s lease / 30s window expiry"
            );
        }
    }

    async fn notice(&mut self, body: &str) {
        self.notice_at(body, Instant::now()).await;
    }

    async fn notice_at(&mut self, body: &str, now: Instant) {
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Notice {
            target: RecordRef,
            state: BindingState,
            broker_epoch: HexBytes<16>,
        }
        let notice = match serde_json::from_str::<Notice>(body) {
            Ok(notice) => notice,
            Err(error) => {
                // Do not log the payload; exact schema failures are observable.
                eprintln!("term dropped native-session lifecycle notice: {error}");
                return;
            }
        };
        if self
            .parent
            .as_ref()
            .is_none_or(|p| p.broker_epoch != notice.broker_epoch)
        {
            return;
        }
        if let Some(control) = self.control() {
            let mut target = notice.target.clone();
            if notice.state == BindingState::Attached {
                target.binding_generation.0 = target.binding_generation.0.saturating_sub(1);
            }
            control.invalidate(Some(&target));
        }
        // A notice about this attachment retires the deadline the control lane
        // reads, rather than leaving it live until the next renew would have
        // noticed. Nothing protected resolves again until a renew re-earns it.
        if self
            .parent
            .as_ref()
            .is_some_and(|p| p.record_id == notice.target.record_id)
            && notice.state != BindingState::Attached
        {
            self.own_lease = None;
        }
        let id = self.children.iter().find_map(|(id, child)| {
            child
                .record
                .as_ref()
                .filter(|record| {
                    record.record_id == notice.target.record_id
                        && record.incarnation == notice.target.incarnation
                        && notice.target.binding_generation.0 >= record.binding_generation.0
                })
                .map(|_| *id)
        });
        if let Some(id) = id {
            if notice.state == BindingState::Revoked
                && !self.children[&id].retry.enrolled
                && self.children[&id].pane.live.load(Ordering::Acquire)
                && let Some(connection) = &self.connection
                && let Ok(found) =
                    bounded(connection.session_grant_fetch(self.children[&id].pane.public_key))
                        .await
            {
                // A coalesced lifecycle stream may skip Attached. A consumed
                // grant records enrolment even after attachment revocation.
                self.children.get_mut(&id).unwrap().retry.enrolled =
                    found.grant.state == GrantState::Consumed;
            }
            let child = self.children.get_mut(&id).unwrap();
            child.retry.enrolled |= notice.state == BindingState::Attached;
            child
                .pane
                .control_ready
                .store(notice.state == BindingState::Attached, Ordering::Release);
            if let Some(record) = &mut child.record {
                record.binding_generation = notice.target.binding_generation;
                record.state = notice.state;
            }
            // Republished from the record the two lines above just corrected:
            // this is the path a child normally reaches Attached by.
            child.publish_binding();
            if notice.state == BindingState::Revoked && child.pane.live.load(Ordering::Acquire) {
                if !child.pane.launched.load(Ordering::Acquire) {
                    self.diagnostic(Some(id), "unconsumed bundle expired; refresh waits for user presence and the mint floor");
                    return;
                }
                // Never enrol => hold until an external event. Enrolled once
                // => at most three attempts, at 120/240/480s after expiry.
                child.retry.expired(now);
                let message = if child.retry.due.is_some() {
                    "grant expired/revoked; bounded re-grant backoff scheduled"
                } else {
                    "grant expired/revoked; held until reconnect, lifecycle gap or pane restart"
                };
                self.diagnostic(Some(id), message);
            } else {
                self.diagnostic(Some(id), format!("child binding {:?}", notice.state));
            }
        }
    }
}

/// A verified command and its attachment stay typed across the owner boundary.
/// Implementations own policy and must recheck this authority before mutation.
pub type LaneFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;

pub trait ControlLane: Send + Sync {
    fn wake(&self) -> Arc<tokio::sync::Notify>;
    fn invalidate(&self, target: Option<&RecordRef>);
    fn flush_notices<'a>(&'a self, connection: &'a VerifiedConnection) -> LaneFuture<'a, ()>;
    fn dispatch<'a>(
        &'a self,
        connection: &'a VerifiedConnection,
        parent: &'a SessionRecord,
        own: &'a (Hello, Deadline),
        event: &'a ::bus::native_client::VerifiedCommand,
    ) -> LaneFuture<'a, ControlReply>;
}

pub use super::protocol::Reply as ControlReply;
use super::protocol::command_manifest;

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use test_broker::Broker;

    include!("session_actor_tests.rs");

    #[test]
    fn issued_descriptor_and_lease_use_the_native_terminal_seam() {
        let broker = Broker::start();
        let supervisor = Supervisor::with_options(broker.options(), broker.url.clone()).unwrap();
        wait_ready(&supervisor.handle, 1);
        let (lease, fd) =
            super::super::native_session::SessionIssuer::prepare(&supervisor.handle, 1).unwrap();
        assert_eq!(fd.endpoint(), Some(broker.endpoint.as_path()));
        assert!(supervisor.handle.launched(1));
        drop(fd);
        lease.revoke_before_cleanup();
        assert!(!supervisor.handle.launched(1));
        assert!(supervisor.handle.pane_generation(1).is_none());
    }

    #[test]
    fn diagnostic_transitions_are_per_outage_and_per_pane() {
        let mut shared = Shared::default();
        assert!(shared.log_diagnostic(None, "startup timed out"));
        assert!(shared.log_diagnostic(None, "broker/profile unavailable"));
        assert!(!shared.log_diagnostic(None, "broker/profile unavailable"));
        assert!(shared.log_diagnostic(None, "launch pool held: waiting for user presence"));
        assert!(shared.log_diagnostic(None, "broker/profile unavailable"));
        assert!(shared.log_diagnostic(Some(2), "detached"));
        assert!(shared.log_diagnostic(Some(3), "detached"));
        assert!(!shared.log_diagnostic(Some(2), "detached"));
        assert!(shared.log_diagnostic(Some(2), "attached"));
        assert!(shared.log_diagnostic(Some(2), "detached"));
        shared.available();
        assert!(shared.log_diagnostic(None, "broker/profile unavailable"));
        assert!(!shared.log_diagnostic(None, "broker/profile unavailable"));
    }

    fn runtime() -> tokio::runtime::Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
    }

    // Tests may await arbitrary slots; production waits only once at startup.
    pub(crate) fn wait_ready(handle: &NativeSession, id: u64) {
        handle.1.lock().unwrap().next_id = id;
        let _ = handle.0.send(Request::Provision);
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if handle
                .1
                .lock()
                .unwrap()
                .ready
                .as_ref()
                .is_some_and(|r| r.pane.id == id)
            {
                return;
            }
            assert!(
                Instant::now() < deadline,
                "ready bundle: {}",
                handle.status()
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    fn prepare_ready(handle: &NativeSession, id: u64) -> (PaneSession, LaunchFd) {
        wait_ready(handle, id);
        handle.prepare(id).expect("ready launch")
    }

    #[test]
    fn supervisor_discovers_but_handoff_names_the_issuing_endpoint() {
        let broker = Broker::start();
        let mut options = broker.options();
        options.endpoint = Some(broker.endpoint.with_file_name("missing.sock"));
        options.configured_endpoint = Some(broker.endpoint.clone());
        assert!(!options.endpoint_only);
        let supervisor = Supervisor::with_options(options, broker.url.clone()).unwrap();
        let (_pane, fd) = prepare_ready(&supervisor.handle, 1);
        assert_eq!(fd.endpoint(), Some(broker.endpoint.as_path()));
    }

    fn test_actor(broker: &Broker) -> Actor {
        Actor {
            grant_creates: 0,
            faults: TestFaults::default(),
            options: broker.options(),
            url: broker.url.clone(),
            key: fresh_key().unwrap(),
            connection: None,
            parent: None,
            own_lease: None,
            children: HashMap::new(),
            shared: Arc::new(std::sync::Mutex::new(Shared::default())),
            provisioned: None,
            pool_key: None,
            activity: Arc::new(AtomicU64::new(clock_ms().unwrap())),
            startup: None,
        }
    }

    fn test_handle(actor: &Actor) -> NativeSession {
        let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
        NativeSession(tx, actor.shared.clone(), actor.activity.clone())
    }

    #[test]
    fn short_window_bundle_cannot_be_consumed() {
        let broker = Broker::start();
        runtime().block_on(async {
            let mut actor = test_actor(&broker);
            actor.connect().await;
            actor.provision().await;
            let old = actor
                .shared
                .lock()
                .unwrap()
                .ready
                .as_ref()
                .unwrap()
                .pane
                .clone();
            actor
                .shared
                .lock()
                .unwrap()
                .ready
                .as_mut()
                .unwrap()
                .deadline = clock_ms().unwrap() + LAUNCH_MARGIN_MS - 1;
            assert!(test_handle(&actor).prepare(1).is_none());
            assert!(!old.live.load(Ordering::Acquire));
            assert!(!old.launched.load(Ordering::Acquire));
            actor.shutdown().await;
        });
    }

    #[test]
    fn ready_consumption_and_missing_bundle_never_wait_for_broker() {
        let broker = Broker::start();
        let supervisor = Supervisor::with_options(broker.options(), broker.url.clone()).unwrap();
        wait_ready(&supervisor.handle, 1);
        let paused = broker.pause();
        let start = Instant::now();
        let (pane, fd) = supervisor.handle.prepare(1).unwrap();
        assert!(supervisor.handle.prepare(2).is_none());
        assert!(start.elapsed() < Duration::from_millis(100));
        assert!(
            supervisor.handle.status()["panes"]["2"]
                .as_str()
                .unwrap()
                .contains("graphics-only")
        );
        drop(fd);
        drop(paused);
        drop(pane);
    }
}
