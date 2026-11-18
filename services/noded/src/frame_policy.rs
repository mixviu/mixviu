// SPDX-License-Identifier: MIT OR Apache-2.0
//! Inter-node policy, independent of the current transport identity mechanism.
use bus::wire::{BusMessage, BusTarget};

/// SPEC 01 grammar: node.bus, service.node[.bus], or
/// sub.service.node[.bus]. Cross-mesh addresses and bare names are excluded.
pub(super) fn from_peer(msg: &BusMessage, peer: &str) -> bool {
    matches!(msg.from_addr().and_then(BusTarget::parse),
        Some(BusTarget::Local(addr)) if addr.node == peer)
}

pub(super) fn session_refusal(
    msg: &BusMessage,
    peer: Option<&str>,
    registered: Option<&str>,
) -> Option<&'static str> {
    if msg.is_empty_message() {
        return None;
    }
    let Some(peer) = peer else {
        return Some("peer-not-admitted");
    };
    let bridge = format!("bridge-{peer}");
    // Registration is the bootstrap exception; the existing register gate
    // still verifies proof/bind/authority and installs revocation tracking.
    if msg.command_name() == Some("noded.register")
        && (matches!(msg.to_addr(), None | Some("noded"))
            || matches!(msg.to_addr().and_then(BusTarget::parse), Some(BusTarget::Local(addr)) if addr.service.as_deref().unwrap_or("noded") == "noded"))
        && msg.from_addr() == Some(bridge.as_str())
    {
        return None;
    }
    if registered != Some(bridge.as_str()) {
        return Some("bridge-not-registered");
    }
    if !from_peer(msg, peer) {
        return Some("from-node-mismatch");
    }
    None
}

pub(super) fn wrong_node(msg: &BusMessage, own: &str) -> bool {
    matches!(msg.to_addr().and_then(BusTarget::parse),
        Some(BusTarget::Local(addr)) if addr.node != own)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn registration_and_every_frame_from_binding() {
        let call = BusMessage::new().with_header("from", "hostd.alpha.bus");
        assert_eq!(
            session_refusal(&call, None, None),
            Some("peer-not-admitted")
        );
        assert_eq!(
            session_refusal(&call, Some("alpha"), None),
            Some("bridge-not-registered")
        );
        assert_eq!(
            session_refusal(&call, Some("alpha"), Some("bridge-beta")),
            Some("bridge-not-registered")
        );
        for from in [
            "alpha.bus",
            "hostd.alpha",
            "hostd.alpha.bus",
            "one.hostd.alpha.bus",
        ] {
            assert!(from_peer(&call.clone().with_header("from", from), "alpha"));
        }
        for from in [
            "hostd",
            "hostd.beta.bus",
            "hostd.alpha.bus@example.test",
            "",
        ] {
            assert_eq!(
                session_refusal(
                    &call.clone().with_header("from", from),
                    Some("alpha"),
                    Some("bridge-alpha")
                ),
                Some("from-node-mismatch")
            );
        }
        assert_eq!(
            session_refusal(&call, Some("alpha"), Some("bridge-alpha")),
            None
        );
        let register = BusMessage::new()
            .with_header("to", "noded")
            .with_header("command", "noded.register")
            .with_header("from", "bridge-alpha");
        assert_eq!(session_refusal(&register, Some("alpha"), None), None);
        assert!(session_refusal(&register, Some("beta"), None).is_some());
    }
    #[test]
    fn ingress_node_is_exact() {
        assert!(wrong_node(
            &BusMessage::new().with_header("to", "hostd.beta.bus"),
            "alpha"
        ));
        assert!(!wrong_node(
            &BusMessage::new().with_header("to", "hostd.alpha.bus"),
            "alpha"
        ));
    }
}
