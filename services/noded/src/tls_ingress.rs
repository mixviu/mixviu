// SPDX-License-Identifier: MIT OR Apache-2.0
//! Lab listener: every accepted stream is TLS, including loopback.
use axum::{extract::connect_info::Connected, serve::{IncomingStream, Listener}};
use bus::native_session::TransportIdentity;
use mesh::tls::{Material, public_key};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{TlsAcceptor, server::TlsStream};
type Accepted = (TlsStream<TcpStream>, TransportIdentity);
pub struct TlsListener {
    ready: tokio::sync::mpsc::Receiver<Accepted>,
    address: std::net::SocketAddr,
    driver: tokio::task::JoinHandle<()>,
}
impl TlsListener {
    pub fn new(tcp: TcpListener, material: Material) -> std::io::Result<Self> {
        let address=tcp.local_addr()?;
        let acceptor=TlsAcceptor::from(material.server().map_err(std::io::Error::other)?);
        let (tx,ready)=tokio::sync::mpsc::channel(64);
        let driver=tokio::spawn(async move {
            let slots=std::sync::Arc::new(tokio::sync::Semaphore::new(64));
            let mut handshakes=tokio::task::JoinSet::new();
            loop {
                tokio::select! {
                    _ = handshakes.join_next(), if !handshakes.is_empty() => {},
                    accepted = tcp.accept() => {
                        let Ok((tcp,_))=accepted else {tokio::task::yield_now().await;continue;};
                        let Ok(permit)=slots.clone().try_acquire_owned() else {continue;};
                        let _=tcp.set_nodelay(true);
                        let acceptor=acceptor.clone();let tx=tx.clone();let lookup=material.lookup.clone();
                        handshakes.spawn(async move {
                            let _permit=permit;
                            let Ok(Ok(tls))=tokio::time::timeout(std::time::Duration::from_secs(3),acceptor.accept(tcp)).await else {return;};
                            let Some(cert)=tls.get_ref().1.peer_certificates().and_then(|c|c.first()) else {return;};
                            let Some(key)=public_key(cert) else {return;};
                            let Some(node)=lookup(&key) else {return;};
                            let _=tx.send((tls,TransportIdentity::MeshTls {node,key})).await;
                        });
                    }
                }
            }
        });
        Ok(Self {ready,address,driver})
    }
}
impl Drop for TlsListener {fn drop(&mut self) {self.driver.abort();}}
impl Listener for TlsListener {
    type Io = TlsStream<TcpStream>;
    type Addr = TransportIdentity;
    async fn accept(&mut self) -> Accepted {self.ready.recv().await.expect("TLS accept driver stopped")}
    fn local_addr(&self) -> std::io::Result<Self::Addr> {Ok(TransportIdentity::LegacyTcp {source_ip:self.address.ip()})}
}
#[derive(Clone)]
pub struct TlsPeer(pub TransportIdentity);
impl Connected<IncomingStream<'_, TlsListener>> for TlsPeer {
    fn connect_info(stream: IncomingStream<'_, TlsListener>) -> Self {Self(stream.remote_addr().clone())}
}
