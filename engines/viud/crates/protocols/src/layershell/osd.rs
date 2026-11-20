// SPDX-License-Identifier: MIT OR Apache-2.0
//! Compositor-owned OSD identity. A namespace is a label, never authority.
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use smithay::reexports::wayland_server::{Client, DisplayHandle};
use smithay::reexports::wayland_server::backend::{ClientData, ClientId, DisconnectReason};
use smithay::wayland::compositor::CompositorClientState;

// Only insert() can construct this token; no shared-socket credential grants it.
struct Connection {
    state: CompositorClientState,
    live: Arc<AtomicBool>,
}
impl ClientData for Connection {
    fn initialized(&self, _: ClientId) {}
    fn disconnected(&self, _: ClientId, _: DisconnectReason) {}
}
pub struct Lease(Arc<AtomicBool>);
impl Lease {
    pub fn revoke(&self) { self.0.store(false, Ordering::Release); }
}
impl Drop for Lease {
    fn drop(&mut self) { self.revoke(); }
}
/// Register the private server end before spawning the child. Dropping the
/// lease revokes authority even if another process retained the old socket.
pub fn insert(display: &DisplayHandle, stream: std::os::unix::net::UnixStream) -> std::io::Result<Lease> {
    let live = Arc::new(AtomicBool::new(true));
    display.clone().insert_client(stream, Arc::new(Connection {
        state: CompositorClientState::default(), live: live.clone(),
    }))?;
    Ok(Lease(live))
}
pub fn compositor_state(client: &Client) -> Option<&CompositorClientState> {
    client.get_data::<Connection>().map(|data| &data.state)
}
pub fn authorized(client: &Client, _: &DisplayHandle) -> bool {
    client.get_data::<Connection>().is_some_and(|data| data.live.load(Ordering::Acquire))
}

/// Attached by viud to the Wayland resource after peer authentication.
#[derive(Debug)]
pub struct Mark {
    pub badge: smithay::backend::renderer::element::Id,
}

impl Default for Mark {
    fn default() -> Self {
        Self {
            badge: smithay::backend::renderer::element::Id::new(),
        }
    }
}

pub fn marked(
    surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
) -> bool {
    let mut surface = surface.clone();
    loop {
        if smithay::wayland::compositor::with_states(&surface, |state| {
            state.data_map.get::<Mark>().is_some()
        }) {
            return true;
        }
        let Some(parent) = smithay::wayland::compositor::get_parent(&surface) else {
            return false;
        };
        surface = parent;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn same_pid_shared_client_is_refused_and_restart_revokes_old_connection() {
        let display = smithay::reexports::wayland_server::Display::<()>::new().unwrap();
        let mut handle = display.handle();
        let (shared, _peer) = std::os::unix::net::UnixStream::pair().unwrap();
        let ordinary = handle.insert_client(shared, Arc::new(())).unwrap();
        let (private, _child) = std::os::unix::net::UnixStream::pair().unwrap();
        let lease = insert(&handle, private).unwrap();
        // Both socketpairs were created by this process: identical peer PIDs.
        assert_eq!(ordinary.get_credentials(&handle).unwrap().pid, std::process::id() as i32);
        assert!(!authorized(&ordinary, &handle));
        // Find the private client through a second insertion with retained handle.
        let live = Arc::new(AtomicBool::new(true));
        let (server, _client) = std::os::unix::net::UnixStream::pair().unwrap();
        let host = handle.insert_client(server, Arc::new(Connection {
            state: CompositorClientState::default(), live: live.clone(),
        })).unwrap();
        let old = Lease(live);
        assert!(authorized(&host, &handle));
        drop(old);
        assert!(!authorized(&host, &handle));
        let (server, _replacement) = std::os::unix::net::UnixStream::pair().unwrap();
        let live = Arc::new(AtomicBool::new(true));
        let replacement = handle.insert_client(server, Arc::new(Connection {
            state: CompositorClientState::default(), live,
        })).unwrap();
        assert!(authorized(&replacement, &handle));
        assert!(!authorized(&host, &handle));
        drop(lease);
    }
}

