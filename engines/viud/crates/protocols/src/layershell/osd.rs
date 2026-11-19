// SPDX-License-Identifier: MIT OR Apache-2.0
//! Compositor-owned OSD identity. A namespace is a label, never authority.
use std::sync::Mutex;

use smithay::reexports::wayland_server::{Client, DisplayHandle};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Identity {
    pid: i32,
    start: u64,
}

impl Identity {
    fn matches(self, peer: i32, live_start: Option<u64>) -> bool {
        self.pid == peer && live_start == Some(self.start)
    }
}

static HOST: Mutex<Option<Identity>> = Mutex::new(None);

fn start_time(pid: i32) -> Option<u64> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    stat[stat.rfind(')')? + 1..].split_whitespace().nth(19)?.parse().ok()
}

/// Pin a direct child launched by the compositor. Unreadable identity fails closed.
pub fn bind(pid: i32) -> bool {
    let identity = start_time(pid).map(|start| Identity { pid, start });
    *HOST.lock().unwrap() = identity;
    identity.is_some()
}

pub fn unbind() {
    *HOST.lock().unwrap() = None;
}

pub fn authorized(client: &Client, display: &DisplayHandle) -> bool {
    let Some(host) = *HOST.lock().unwrap() else { return false };
    client.get_credentials(display).is_ok_and(|credentials| {
        host.matches(credentials.pid, start_time(host.pid))
    })
}

/// Attached by viud to the Wayland resource after peer authentication.
#[derive(Debug)]
pub struct Mark {
    pub badge: smithay::backend::renderer::element::Id,
}

impl Default for Mark {
    fn default() -> Self {
        Self { badge: smithay::backend::renderer::element::Id::new() }
    }
}

pub fn marked(surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface) -> bool {
    let mut surface = surface.clone();
    loop {
        if smithay::wayland::compositor::with_states(&surface, |state| state.data_map.get::<Mark>().is_some()) {
            return true;
        }
        let Some(parent) = smithay::wayland::compositor::get_parent(&surface) else { return false };
        surface = parent;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_is_live_and_unreadable_identity_fails_closed() {
        assert!(start_time(std::process::id() as i32).is_some());
        assert_eq!(start_time(-1), None);
    }

    #[test]
    fn identity_rejects_pid_reuse_dead_leader_and_other_clients() {
        let host = Identity { pid: 42, start: 1234 };
        assert!(host.matches(42, Some(1234)));
        assert!(!host.matches(43, Some(1234)));
        assert!(!host.matches(42, Some(1235)));
        assert!(!host.matches(42, None));
    }
}
