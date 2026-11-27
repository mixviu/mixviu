// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded source buckets and member session leases. Addresses are never identity.
use std::collections::{BTreeMap, HashMap};
use std::net::{IpAddr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// A full source table refuses new networks rather than evicting live buckets
/// and allowing source churn to replenish an attacker's tokens.
pub struct SourceRate {
    rate: u32,
    burst: u32,
    buckets: HashMap<IpAddr, (f64, Instant)>,
}
impl SourceRate {
    pub fn new(rate: u32, burst: u32) -> Self {
        Self { rate, burst, buckets: HashMap::new() }
    }
    pub fn allow(&mut self, peer: SocketAddr, now: Instant) -> bool {
        let network = super::tls::refusal_network(peer);
        if !self.buckets.contains_key(&network) && self.buckets.len() >= 4096 {
            // Only fully replenished buckets can be reclaimed safely.
            let rate = self.rate as f64;
            let burst = self.burst as f64;
            self.buckets.retain(|_, (tokens, at)| {
                *tokens + now.duration_since(*at).as_secs_f64() * rate < burst
            });
            if self.buckets.len() >= 4096 { return false; }
        }
        let (tokens, at) = self.buckets.entry(network).or_insert((self.burst as f64, now));
        *tokens = (*tokens + now.duration_since(*at).as_secs_f64() * self.rate as f64)
            .min(self.burst as f64);
        *at = now;
        if *tokens < 1.0 { return false; }
        *tokens -= 1.0;
        true
    }
}

#[derive(Debug)]
pub struct Sessions {
    cap: usize,
    members: Mutex<BTreeMap<String, usize>>,
}
impl Sessions {
    pub fn new(cap: usize) -> Arc<Self> {
        Arc::new(Self { cap, members: Mutex::new(BTreeMap::new()) })
    }
    pub fn acquire(self: &Arc<Self>, member: &str) -> Option<Session> {
        let mut members = self.members.lock().expect("TLS sessions poisoned");
        let count = members.entry(member.to_owned()).or_default();
        if *count >= self.cap { return None; }
        *count += 1;
        Some(Session { owner: self.clone(), member: member.to_owned() })
    }
    pub fn snapshot(&self) -> BTreeMap<String, usize> {
        self.members.lock().expect("TLS sessions poisoned").clone()
    }
}
#[derive(Debug)]
pub struct Session {
    owner: Arc<Sessions>,
    member: String,
}
impl Drop for Session {
    fn drop(&mut self) {
        let mut members = self.owner.members.lock().expect("TLS sessions poisoned");
        let count = members.get_mut(&self.member).expect("live TLS session");
        *count -= 1;
        if *count == 0 { members.remove(&self.member); }
    }
}

/// Hold the lease until the actual transport drops, including HTTP upgrade and
/// failure paths. No connection-info clone can keep a dead session counted.
pub struct LeasedIo<T> {
    pub io: T,
    pub session: Session,
}
impl<T: tokio::io::AsyncRead + Unpin> tokio::io::AsyncRead for LeasedIo<T> {
    fn poll_read(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>, buf: &mut tokio::io::ReadBuf<'_>) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.io).poll_read(cx, buf)
    }
}
impl<T: tokio::io::AsyncWrite + Unpin> tokio::io::AsyncWrite for LeasedIo<T> {
    fn poll_write(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>, buf: &[u8]) -> std::task::Poll<std::io::Result<usize>> {
        std::pin::Pin::new(&mut self.io).poll_write(cx, buf)
    }
    fn poll_flush(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.io).poll_flush(cx)
    }
    fn poll_shutdown(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<std::io::Result<()>> {
        std::pin::Pin::new(&mut self.io).poll_shutdown(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_burst_isolated_refills_and_groups_ipv6() {
        let now = Instant::now();
        let mut rate = SourceRate::new(2, 2);
        let a = "192.0.2.1:1".parse().unwrap();
        assert!(rate.allow(a, now));
        assert!(rate.allow(a, now));
        assert!(!rate.allow(a, now));
        assert!(rate.allow("192.0.2.2:1".parse().unwrap(), now));
        assert!(rate.allow(a, now + std::time::Duration::from_millis(500)));
        assert!(!rate.allow("[::ffff:192.0.2.1]:2".parse().unwrap(), now + std::time::Duration::from_millis(500)));
        assert!(rate.allow("[2001:db8::1]:1".parse().unwrap(), now));
        assert!(rate.allow("[2001:db8::2]:1".parse().unwrap(), now));
        assert!(!rate.allow("[2001:db8::3]:1".parse().unwrap(), now));
        assert!(rate.allow("[2001:db8:0:1::1]:1".parse().unwrap(), now));
    }
    #[test]
    fn cap_is_per_member_and_drop_releases() {
        let sessions = Sessions::new(2);
        let a = sessions.acquire("alpha").unwrap();
        let _b = sessions.acquire("alpha").unwrap();
        assert!(sessions.acquire("alpha").is_none());
        let c = sessions.acquire("beta").unwrap();
        assert_eq!(sessions.snapshot()["alpha"], 2);
        drop(a);
        assert!(sessions.acquire("alpha").is_some());
        drop(c);
        assert!(!sessions.snapshot().contains_key("beta"));
    }
}
