// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded TLS handshake ingress. Queued sockets own no handshake permit.
use std::collections::VecDeque;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::connect_info::Connected;
use axum::serve::{IncomingStream, Listener};
use mesh::tls::{Credential, LiveAuthority, verified_member};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio::task::JoinSet;
use tokio_rustls::{TlsAcceptor, server::TlsStream};
pub use mesh::tls::Refusals;

pub(crate) fn prepare(
    posture: &crate::authority::Posture, node: &str, settings: &config::node::NodedTls,
) -> anyhow::Result<mesh::TlsRuntime> {
    let crate::authority::Posture::Verified(a) = posture else {
        anyhow::bail!("TLS requires a verified accepted inventory");
    };
    let keys = Arc::new(mesh_trust::identity::TlsAuthority::new(&a.members_full, a.epoch, a.recovery_generation));
    let path = paths::resolve(&settings.key).map_err(anyhow::Error::msg)?;
    let credential = Credential::new(mesh::tls::load_seed(&path)?)?;
    let own_key = mesh::tls::public_key(&credential.certificate)?;
    anyhow::ensure!(keys.member_for(&own_key).is_some_and(|m| m.member == node), "TLS requires a current own bus-tls key");
    Ok(mesh::TlsRuntime { credential, authority: Arc::new(std::sync::RwLock::new(keys)) })
}

#[derive(Clone, Debug)]
pub(crate) struct TlsPeer {
    pub address: SocketAddr,
    pub member: mesh_trust::identity::TlsMember,
    pub binding: [u8; 32],
}

type Accepted = (TlsStream<TcpStream>, TlsPeer);

pub(crate) struct TlsListener {
    ready: mpsc::Receiver<Accepted>,
    driver: tokio::task::JoinHandle<()>,
}
impl TlsListener {
    pub fn new(
        tcp: TcpListener,
        credential: &Credential,
        authority: LiveAuthority,
        limits: &config::node::NodedTls,
        refusals: Arc<Refusals>,
    ) -> anyhow::Result<Self> {
        anyhow::ensure!(limits.handshake_timeout_s > 0, "TLS handshake timeout must be positive");
        anyhow::ensure!(limits.max_handshakes > 0, "TLS max_handshakes must be positive");
        let acceptor = TlsAcceptor::from(credential.server_config(authority.clone())?);
        let (tx, ready) = mpsc::channel(limits.max_handshakes);
        let deadline = Duration::from_secs(limits.handshake_timeout_s);
        let max_handshakes = limits.max_handshakes;
        let max_queue = limits.max_queue;
        let driver = tokio::spawn(async move {
            let mut running = JoinSet::new();
            let mut queue = VecDeque::new();
            loop {
                // JoinSet length is the concurrency permit. A queued socket
                // does not enter it until a completed handshake is reaped.
                while running.len() < max_handshakes {
                    let Some((tcp, peer)) = queue.pop_front() else { break };
                    let acceptor = acceptor.clone();
                    let authority = authority.clone();
                    let tx = tx.clone();
                    let refusals = refusals.clone();
                    running.spawn(async move {
                        let result = tokio::time::timeout(deadline, async {
                            let tls = acceptor.accept(tcp).await.map_err(|_| "protocol-or-key")?;
                            let conn = tls.get_ref().1;
                            let member = verified_member(
                                &authority, conn.peer_certificates().unwrap_or(&[]), None,
                            ).map_err(|_| "membership")?;
                            let binding = mesh::tls::server_channel_binding(conn).map_err(|_| "exporter")?;
                            Ok::<_, &'static str>((tls, TlsPeer { address: peer, member, binding }))
                        }).await;
                        match result {
                            Err(_) => refusals.record(peer, "timeout"),
                            Ok(Err(class)) => refusals.record(peer, class),
                            Ok(Ok(accepted)) => {
                                // A slow HTTP accept consumer cannot retain a
                                // handshake slot waiting for delivery capacity.
                                if tx.try_send(accepted).is_err() {
                                    refusals.record(peer, "ready-queue-full");
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
                                let _ = tcp.set_nodelay(true);
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
    fn drop(&mut self) { self.driver.abort(); }
}
impl Listener for TlsListener {
    type Io = TlsStream<TcpStream>;
    type Addr = Option<TlsPeer>;
    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        let (stream, peer) = self.ready.recv().await.expect("TLS driver stopped");
        (stream, Some(peer))
    }
    fn local_addr(&self) -> std::io::Result<Self::Addr> { Ok(None) }
}
impl Connected<IncomingStream<'_, TlsListener>> for TlsPeer {
    fn connect_info(stream: IncomingStream<'_, TlsListener>) -> Self {
        stream.remote_addr().as_ref().expect("accepted TLS peer").clone()
    }
}
