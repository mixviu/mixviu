// SPDX-License-Identifier: MIT OR Apache-2.0
//! Lab listener: every accepted stream is TLS, including loopback.
use axum::{extract::connect_info::Connected, serve::{IncomingStream, Listener}};
use bus::native_session::TransportIdentity;
use mesh::tls::{Material, public_key};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::{TlsAcceptor, server::TlsStream};

pub struct TlsListener {
    tcp: TcpListener,
    acceptor: TlsAcceptor,
    material: Material,
}
impl TlsListener {
    pub fn new(tcp: TcpListener, material: Material) -> std::io::Result<Self> {
        Ok(Self { tcp, acceptor: TlsAcceptor::from(material.server().map_err(std::io::Error::other)?), material })
    }
}
impl Listener for TlsListener {
    type Io = TlsStream<TcpStream>;
    type Addr = TransportIdentity;
    async fn accept(&mut self) -> (Self::Io, Self::Addr) {
        loop {
            let Ok((tcp, _)) = self.tcp.accept().await else { continue; };
            let _ = tcp.set_nodelay(true);
            let Ok(Ok(tls)) = tokio::time::timeout(std::time::Duration::from_secs(3), self.acceptor.accept(tcp)).await else { continue; };
            let Some(cert) = tls.get_ref().1.peer_certificates().and_then(|c| c.first()) else { continue; };
            let Some(key) = public_key(cert) else { continue; };
            let Some(node) = (self.material.lookup)(&key) else { continue; };
            return (tls, TransportIdentity::MeshTls { node, key });
        }
    }
    fn local_addr(&self) -> std::io::Result<Self::Addr> {
        Ok(TransportIdentity::LegacyTcp { source_ip: self.tcp.local_addr()?.ip() })
    }
}
#[derive(Clone)]
pub struct TlsPeer(pub TransportIdentity);
impl Connected<IncomingStream<'_, TlsListener>> for TlsPeer {
    fn connect_info(stream: IncomingStream<'_, TlsListener>) -> Self { Self(stream.remote_addr().clone()) }
}
