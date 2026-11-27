// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded TLS handshake ingress. Queued sockets own no handshake permit.
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::connect_info::Connected;
use axum::serve::{IncomingStream, Listener};
pub use mesh::tls::Refusals;
use mesh::tls::{Credential, verified_member};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio_rustls::{TlsAcceptor, server::TlsStream};

fn validate_shared_limits(limits: &config::node::NodedTls) -> anyhow::Result<()> {
    anyhow::ensure!(
        limits.per_source_rate > 0
            && limits.per_source_burst > 0
            && limits.max_sessions_per_member > 0,
        "TLS rate, burst and session cap must be positive"
    );
    Ok(())
}

pub(crate) fn prepare(
    posture: &crate::authority::Posture,
    node: &str,
    settings: &config::node::NodedTls,
) -> anyhow::Result<mesh::TlsRuntime> {
    validate_shared_limits(settings)?;
    let crate::authority::Posture::Verified(a) = posture else {
        anyhow::bail!("TLS requires a verified accepted inventory");
    };
    let keys = Arc::new(mesh_trust::identity::TlsAuthority::new(
        &a.members_full,
        a.epoch,
        a.recovery_generation,
    ));
    let path = paths::resolve(&settings.key).map_err(anyhow::Error::msg)?;
    let credential = Credential::new(mesh::tls::load_seed(&path)?)?;
    let own_key = mesh::tls::public_key(&credential.certificate)?;
    anyhow::ensure!(
        keys.member_for(&own_key).is_some_and(|m| m.member == node),
        "TLS requires a current own bus-tls key"
    );
    Ok(mesh::TlsRuntime {
        credential,
        authority: Arc::new(std::sync::RwLock::new(keys)),
        sessions: mesh::tls_limits::Sessions::new(settings.max_sessions_per_member),
        source_rate: Arc::new(std::sync::Mutex::new(mesh::tls_limits::SourceRate::new(
            settings.per_source_rate,
            settings.per_source_burst,
        ))),
    })
}

#[derive(Clone, Debug)]
pub(crate) struct TlsPeer {
    pub address: SocketAddr,
    pub member: mesh_trust::identity::TlsMember,
    pub binding: [u8; 32],
}

type Accepted = (mesh::tls_limits::LeasedIo<TlsStream<TcpStream>>, TlsPeer);

pub(crate) struct TlsListener {
    ready: mpsc::Receiver<Accepted>,
    driver: tokio::task::JoinHandle<()>,
}
impl TlsListener {
    pub fn new(
        tcp: TcpListener,
        runtime: &mesh::TlsRuntime,
        limits: &config::node::NodedTls,
        refusals: Arc<Refusals>,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(
            limits.handshake_timeout_s > 0,
            "TLS handshake timeout must be positive"
        );
        anyhow::ensure!(
            limits.max_handshakes > 0,
            "TLS max_handshakes must be positive"
        );
        validate_shared_limits(limits)?;
        let acceptor = TlsAcceptor::from(
            runtime
                .credential
                .server_config(runtime.authority.clone())?,
        );
        let (tx, ready) = mpsc::channel(limits.max_handshakes);
        let deadline = Duration::from_secs(limits.handshake_timeout_s);
        let max_handshakes = limits.max_handshakes;
        let max_queue = limits.max_queue;
        let rate = runtime.source_rate.clone();
        let sessions = runtime.sessions.clone();
        let authority = runtime.authority.clone();
        let driver = tokio::spawn(async move {
            let mut running = JoinSet::new();
            let mut queue = VecDeque::new();
            loop {
                // JoinSet length is the concurrency permit. A queued socket
                // does not enter it until a completed handshake is reaped.
                while running.len() < max_handshakes {
                    let Some((tcp, peer)) = queue.pop_front() else {
                        break;
                    };
                    let acceptor = acceptor.clone();
                    let authority = authority.clone();
                    let tx = tx.clone();
                    let refusals = refusals.clone();
                    let sessions = sessions.clone();
                    running.spawn(async move {
                        let result = tokio::time::timeout(deadline, async {
                            let tls = acceptor.accept(tcp).await.map_err(|_| "protocol-or-key")?;
                            let conn = tls.get_ref().1;
                            let member = verified_member(
                                &authority,
                                conn.peer_certificates().unwrap_or(&[]),
                                None,
                            )
                            .map_err(|_| "membership")?;
                            let binding =
                                mesh::tls::server_channel_binding(conn).map_err(|_| "exporter")?;
                            let session = sessions.acquire(&member.member).ok_or("session-cap")?;
                            Ok::<_, &'static str>((
                                mesh::tls_limits::LeasedIo { io: tls, session },
                                TlsPeer {
                                    address: peer,
                                    member,
                                    binding,
                                },
                            ))
                        })
                        .await;
                        match result {
                            Err(_) => refusals.record(peer, "timeout"),
                            Ok(Err(class)) => refusals.record(peer, class),
                            Ok(Ok(accepted)) => {
                                // A slow HTTP accept consumer cannot retain a
                                // handshake slot waiting for delivery capacity.
                                match tx.try_send(accepted) {
                                    Ok(()) => refusals.admitted(),
                                    Err(mpsc::error::TrySendError::Full(_)) => {
                                        refusals.record(peer, "ready-queue-full");
                                    }
                                    Err(mpsc::error::TrySendError::Closed(_)) => {
                                        refusals.record(peer, "ready-queue-closed");
                                    }
                                }
                            }
                        }
                    });
                }
                tokio::select! {
                    _ = tx.closed() => break,
                    _ = running.join_next(), if !running.is_empty() => {},
                    accepted = tcp.accept() => {
                        match accepted {
                            Ok((tcp, peer)) => {
                                if !rate.lock().expect("TLS rate poisoned").allow(peer, std::time::Instant::now()) {
                                    refusals.record(peer, "rate-limited");
                                    continue;
                                }
                                if let Err(error) = tcp.set_nodelay(true) {
                                    tracing::debug!(%error, %peer, "TCP_NODELAY failed on TLS accept");
                                }
                                if running.len() < max_handshakes || queue.len() < max_queue {
                                    queue.push_back((tcp, peer));
                                } else {
                                    refusals.record(peer, "queue-full");
                                }
                            }
                            Err(error) => {
                                tracing::warn!(%error, "TLS TCP accept failed");
                                tokio::time::sleep(Duration::from_millis(100)).await;
                            }
                        }
                    }
                }
            }
            // Dropping JoinSet aborts active handshakes, releasing sockets.
        });
        Ok(Self { ready, driver })
    }
}
impl Drop for TlsListener {
    fn drop(&mut self) {
        self.driver.abort();
    }
}
impl Listener for TlsListener {
    type Io = mesh::tls_limits::LeasedIo<TlsStream<TcpStream>>;
    type Addr = Option<TlsPeer>;
    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        let (stream, peer) = self.ready.recv().await.expect("TLS driver stopped");
        (stream, Some(peer))
    }
    fn local_addr(&self) -> std::io::Result<Self::Addr> {
        Ok(None)
    }
}
impl Connected<IncomingStream<'_, TlsListener>> for TlsPeer {
    fn connect_info(stream: IncomingStream<'_, TlsListener>) -> Self {
        stream
            .remote_addr()
            .as_ref()
            .expect("accepted TLS peer")
            .clone()
    }
}
