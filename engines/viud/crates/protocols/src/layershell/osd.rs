// SPDX-License-Identifier: MIT OR Apache-2.0
//! Compositor-owned OSD identity. A namespace is a label, never authority.
use smithay::reexports::wayland_server::backend::{ClientData, ClientId, DisconnectReason};
use smithay::reexports::wayland_server::{Client, DisplayHandle};
use smithay::wayland::compositor::CompositorClientState;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

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
    pub fn revoke(&self) {
        self.0.store(false, Ordering::Release);
    }
}
impl Drop for Lease {
    fn drop(&mut self) {
        self.revoke();
    }
}
/// Register the private server end before spawning the child. Dropping the
/// lease revokes authority even if another process retained the old socket.
pub fn insert(
    display: &DisplayHandle,
    stream: std::os::unix::net::UnixStream,
) -> std::io::Result<Lease> {
    insert_connection(display, stream).map(|(_, lease)| lease)
}
fn insert_connection(
    display: &DisplayHandle,
    stream: std::os::unix::net::UnixStream,
) -> std::io::Result<(Client, Lease)> {
    let live = Arc::new(AtomicBool::new(true));
    let client = display.clone().insert_client(
        stream,
        Arc::new(Connection {
            state: CompositorClientState::default(),
            live: live.clone(),
        }),
    )?;
    Ok((client, Lease(live)))
}
pub fn compositor_state(client: &Client) -> Option<&CompositorClientState> {
    client.get_data::<Connection>().map(|data| &data.state)
}
pub fn authorized(client: &Client, _: &DisplayHandle) -> bool {
    client
        .get_data::<Connection>()
        .is_some_and(|data| data.live.load(Ordering::Acquire))
}

/// Attached by viud to the Wayland resource after peer authentication.
#[derive(Debug)]
pub struct Mark {
    pub badge: smithay::backend::renderer::element::Id,
    live: Arc<AtomicBool>,
}

impl Mark {
    pub fn for_client(client: &Client) -> Option<Self> {
        let connection = client.get_data::<Connection>()?;
        Some(Self {
            badge: smithay::backend::renderer::element::Id::new(),
            live: connection.live.clone(),
        })
    }
    fn is_live(&self) -> bool {
        self.live.load(Ordering::Acquire)
    }
}

pub fn marked(
    surface: &smithay::reexports::wayland_server::protocol::wl_surface::WlSurface,
) -> bool {
    let mut surface = surface.clone();
    loop {
        if smithay::wayland::compositor::with_states(&surface, |state| {
            state.data_map.get::<Mark>().is_some_and(Mark::is_live)
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
        let path = std::env::temp_dir().join(format!(
            "osd-shared-{}-{}.sock",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let listener = std::os::unix::net::UnixListener::bind(&path).unwrap();
        let _peer = std::os::unix::net::UnixStream::connect(&path).unwrap();
        let (shared, _) = listener.accept().unwrap();
        std::fs::remove_file(path).unwrap();
        let ordinary = handle
            .insert_client(
                shared,
                Arc::new(crate::wayland::connection::record::WaylandClientSession {
                    compositor_state: CompositorClientState::default(),
                    proprietary: false,
                    agent_clients: Default::default(),
                }),
            )
            .unwrap();
        let (private, _child) = std::os::unix::net::UnixStream::pair().unwrap();
        let (host, lease) = insert_connection(&handle, private).unwrap();
        // The shared connection and private socketpair have identical peer PIDs.
        assert_eq!(
            ordinary.get_credentials(&handle).unwrap().pid,
            std::process::id() as i32
        );
        assert!(!authorized(&ordinary, &handle));
        assert_eq!(
            host.get_credentials(&handle).unwrap().pid,
            ordinary.get_credentials(&handle).unwrap().pid
        );
        assert!(authorized(&host, &handle));
        let mark = Mark::for_client(&host).unwrap();
        assert!(mark.is_live());
        drop(lease);
        assert!(
            !mark.is_live(),
            "existing marks lose band and hit authority on revoke"
        );
        assert!(!authorized(&host, &handle));
        let (server, _replacement) = std::os::unix::net::UnixStream::pair().unwrap();
        let (replacement, _lease) = insert_connection(&handle, server).unwrap();
        assert!(authorized(&replacement, &handle));
        assert!(!authorized(&host, &handle));
    }
}
