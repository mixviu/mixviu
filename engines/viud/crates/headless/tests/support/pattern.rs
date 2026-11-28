// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Shm test client: four opaque quadrants. Cursor tests use the agent arrow.
macro_rules! mapping_trace {
    ($($arg:tt)*) => {
        if std::env::var_os(paths::env::test::TEST_MAPPING_TRACE).is_some() {
            eprintln!($($arg)*);
        }
    };
}
use std::{
    fs::File,
    os::{
        fd::{AsFd, FromRawFd},
        unix::{fs::FileExt, net::UnixStream},
    },
    thread,
    time::{Duration, Instant},
};
use wayland_client::{
    Connection, Dispatch, EventQueue, QueueHandle, WEnum, delegate_noop,
    protocol::{
        wl_buffer::WlBuffer,
        wl_callback::{self, WlCallback},
        wl_compositor::WlCompositor,
        wl_keyboard::{self, WlKeyboard},
        wl_pointer::{self, WlPointer},
        wl_registry::{self, WlRegistry},
        wl_seat::{self, WlSeat},
        wl_shm::{self, WlShm},
        wl_shm_pool::WlShmPool,
        wl_surface::WlSurface,
    },
};
use wayland_protocols::{
    ext::session_lock::v1::client::{
        ext_session_lock_manager_v1::ExtSessionLockManagerV1,
        ext_session_lock_v1::{self, ExtSessionLockV1},
    },
    wp::presentation_time::client::{
        wp_presentation::{self, WpPresentation},
        wp_presentation_feedback::{self, WpPresentationFeedback},
    },
    wp::{
        fractional_scale::v1::client::{
            wp_fractional_scale_manager_v1::WpFractionalScaleManagerV1,
            wp_fractional_scale_v1::{self, WpFractionalScaleV1},
        },
        viewporter::client::{wp_viewport::WpViewport, wp_viewporter::WpViewporter},
    },
    xdg::decoration::zv1::client::{
        zxdg_decoration_manager_v1::ZxdgDecorationManagerV1,
        zxdg_toplevel_decoration_v1::{self, ZxdgToplevelDecorationV1},
    },
    xdg::shell::client::{
        xdg_popup::{self, XdgPopup},
        xdg_positioner::{Anchor, Gravity, XdgPositioner},
        xdg_surface::{self, XdgSurface},
        xdg_toplevel::{self, XdgToplevel},
        xdg_wm_base::{self, XdgWmBase},
    },
};

pub const COLOURS: [u32; 4] = [0xffd02030, 0xff20d040, 0xff3040d0, 0xffe0d020];

#[derive(Default)]
pub struct State {
    pub scale_probe: bool,
    commit_id: u64,
    buffer_info: (i32, i32, u32),
    pub scale120: u32,
    pub events: Vec<(Instant, ScaleEvent)>,
    scale_dirty: bool,
    fractional_received: bool,
    globals: Vec<(u32, String, u32)>,
    configured: bool,
    pointers: Vec<WlPointer>,
    pub size: (i32, i32),
    pub callbacks: u64,
    pub buffer_releases: u64,
    pub pointer_buttons: u64,
    pub pointer_positions: Vec<(f64, f64)>,
    pub pointer_motions: u64,
    pub keys: u64,
    pub keyboard_enters: u64,
    last_press: Option<(u32, WlSeat)>,
    last_enter: Option<(u32, WlPointer)>,
    pub pointer_focus: Option<WlSurface>,
    popup_configured: bool,
    pub locked: bool,
    pub presented: u64,
    pub discarded: u64,
    pub clock: Option<u32>,
    pub times: Vec<u64>,
    pub idled: u64,
    layer_size: Option<(i32, i32)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScaleEvent {
    Preferred(u32),
    BufferScale(i32),
    Configure(i32, i32, Vec<u32>),
    Ack(u32),
    Callback,
    Commit(i32, i32, u32),
    Presented(u64, i32, i32, u32),
}

impl State {
    fn log_scale(&mut self, event: ScaleEvent) {
        if self.scale_probe {
            let at = Instant::now();
            eprintln!("FRACTIONAL_EVENT at={at:?} event={event:?}");
            self.events.push((at, event));
        }
    }
}

pub struct Pattern {
    viewport: Option<WpViewport>,
    _fractional: Option<WpFractionalScaleV1>,
    conn: Connection,
    queue: EventQueue<State>,
    pub state: State,
    pub surface: WlSurface,
    shm: WlShm,
    presentation: WpPresentation,
    buffer: WlBuffer,
    registry: WlRegistry,
    compositor: WlCompositor,
    shell: XdgWmBase,
    xdg: XdgSurface,
    _top: XdgToplevel,
    _decoration: Option<ZxdgToplevelDecorationV1>,
}

impl Pattern {
    /// Send a new buffer and destroy its role in the same Wayland read batch.
    pub fn commit_then_destroy(&mut self) {
        self.surface.attach(Some(&self.buffer), 0, 0);
        self.surface
            .damage_buffer(0, 0, self.state.size.0, self.state.size.1);
        self.surface.commit();
        if let Some(decoration) = self._decoration.take() {
            decoration.destroy();
        }
        self._top.destroy();
        self.xdg.destroy();
        self.surface.destroy();
        self.conn.flush().unwrap();
    }
    pub fn another_window(&self, name: &str) -> Self {
        Self::from_connection(self.conn.clone(), name, false, false, (200, 160))
    }
    pub fn hide_cursor(&mut self) {
        let (serial, pointer) = self.state.last_enter.clone().expect("cursor enter");
        pointer.set_cursor(serial, None, 0, 0);
        self.conn.flush().unwrap();
    }
    pub fn named_cursor(&mut self) {
        use wayland_protocols::wp::cursor_shape::v1::client::{
            wp_cursor_shape_device_v1::Shape, wp_cursor_shape_manager_v1::WpCursorShapeManagerV1,
        };
        let name = self
            .state
            .globals
            .iter()
            .find(|(_, i, _)| i == "wp_cursor_shape_manager_v1")
            .unwrap()
            .0;
        let qh = self.queue.handle();
        let manager: WpCursorShapeManagerV1 = self.registry.bind(name, 1, &qh, ());
        let (serial, pointer) = self.state.last_enter.clone().expect("cursor enter");
        let device = manager.get_pointer(&pointer, &qh, ());
        device.set_shape(serial, Shape::Crosshair);
        device.destroy();
        manager.destroy();
        self.queue.roundtrip(&mut self.state).unwrap();
    }
    pub fn subsurface(
        &mut self,
    ) -> (
        WlSurface,
        wayland_client::protocol::wl_subsurface::WlSubsurface,
    ) {
        use wayland_client::protocol::wl_subcompositor::WlSubcompositor;
        let name = self
            .state
            .globals
            .iter()
            .find(|(_, i, _)| i == "wl_subcompositor")
            .unwrap()
            .0;
        let qh = self.queue.handle();
        let manager: WlSubcompositor = self.registry.bind(name, 1, &qh, ());
        let surface = self.compositor.create_surface(&qh, ());
        let sub = manager.get_subsurface(&surface, &self.surface, &qh, ());
        sub.set_position(40, 40);
        let buffer = buffer(&self.shm, &qh, 40, 40, |_, _| 0xff885522);
        surface.attach(Some(&buffer), 0, 0);
        surface.damage_buffer(0, 0, 40, 40);
        surface.commit();
        self.surface.commit();
        self.queue.roundtrip(&mut self.state).unwrap();
        (surface, sub)
    }
    pub fn custom_cursor(&mut self, colour: u32) -> WlSurface {
        let qh = self.queue.handle();
        let surface = self.compositor.create_surface(&qh, ());
        let buffer = buffer(&self.shm, &qh, 32, 32, |_, _| colour);
        let (serial, pointer) = self.state.last_enter.clone().expect("cursor enter");
        pointer.set_cursor(serial, Some(&surface), 0, 0);
        surface.attach(Some(&buffer), 0, 0);
        surface.damage_buffer(0, 0, 32, 32);
        surface.commit();
        self.conn.flush().unwrap();
        surface
    }
    pub fn drag_icon(
        &mut self,
        colour: u32,
    ) -> (
        WlSurface,
        wayland_client::protocol::wl_data_device::WlDataDevice,
    ) {
        use wayland_client::protocol::wl_data_device_manager::WlDataDeviceManager;
        let name = self
            .state
            .globals
            .iter()
            .find(|(_, i, _)| i == "wl_data_device_manager")
            .unwrap()
            .0;
        let qh = self.queue.handle();
        let manager: WlDataDeviceManager = self.registry.bind(name, 3, &qh, ());
        let (serial, seat) = self
            .state
            .last_press
            .clone()
            .expect("implicit pointer grab");
        let device = manager.get_data_device(&seat, &qh, ());
        let icon = self.compositor.create_surface(&qh, ());
        let buffer = buffer(&self.shm, &qh, 32, 32, |_, _| colour);
        device.start_drag(None, &self.surface, Some(&icon), serial);
        icon.attach(Some(&buffer), 0, 0);
        icon.damage_buffer(0, 0, 32, 32);
        icon.commit();
        self.conn.flush().unwrap();
        (icon, device)
    }
    pub fn layer(
        &mut self,
        namespace: &str,
        layer: wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::Layer,
        colour: u32,
    ) -> (
        WlSurface,
        wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
    ) {
        use wayland_protocols_wlr::layer_shell::v1::client::{
            zwlr_layer_shell_v1::ZwlrLayerShellV1, zwlr_layer_surface_v1::Anchor,
        };
        let name = self
            .state
            .globals
            .iter()
            .find(|(_, i, _)| i == "zwlr_layer_shell_v1")
            .unwrap()
            .0;
        let qh = self.queue.handle();
        let shell: ZwlrLayerShellV1 = self.registry.bind(name, 1, &qh, ());
        let surface = self.compositor.create_surface(&qh, ());
        let role = shell.get_layer_surface(&surface, None, layer, namespace.into(), &qh, ());
        role.set_anchor(Anchor::Top | Anchor::Bottom | Anchor::Left | Anchor::Right);
        role.set_size(0, 0);
        role.set_exclusive_zone(-1);
        self.state.layer_size = None;
        surface.commit();
        while self.state.layer_size.is_none() {
            self.queue.roundtrip(&mut self.state).unwrap();
        }
        let (w, h) = self.state.layer_size.unwrap();
        let buffer = buffer(&self.shm, &qh, w, h, |_, _| colour);
        surface.attach(Some(&buffer), 0, 0);
        surface.damage_buffer(0, 0, w, h);
        surface.commit();
        self.queue.roundtrip(&mut self.state).unwrap();
        (surface, role)
    }
    pub fn current_buffer(&self) -> WlBuffer {
        self.buffer.clone()
    }
    pub fn swap_buffer(&mut self, buffer: &mut WlBuffer) {
        std::mem::swap(&mut self.buffer, buffer);
    }
    pub fn idle_notification(&mut self, timeout_ms: u32) {
        use wayland_protocols::ext::idle_notify::v1::client::ext_idle_notifier_v1::ExtIdleNotifierV1;
        let name = self
            .state
            .globals
            .iter()
            .find(|(_, i, _)| i == "ext_idle_notifier_v1")
            .unwrap()
            .0;
        let qh = self.queue.handle();
        let manager: ExtIdleNotifierV1 = self.registry.bind(name, 1, &qh, ());
        let name = self
            .state
            .globals
            .iter()
            .find(|(_, i, _)| i == "wl_seat")
            .unwrap()
            .0;
        let seat: WlSeat = self.registry.bind(name, 1, &qh, ());
        manager.get_idle_notification(timeout_ms, &seat, &qh, ());
        manager.destroy();
        self.conn.flush().unwrap();
    }
    pub fn inhibit(
        &mut self,
    ) -> wayland_protocols::wp::idle_inhibit::zv1::client::zwp_idle_inhibitor_v1::ZwpIdleInhibitorV1
    {
        use wayland_protocols::wp::idle_inhibit::zv1::client::zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1;
        let name = self
            .state
            .globals
            .iter()
            .find(|(_, i, _)| i == "zwp_idle_inhibit_manager_v1")
            .unwrap()
            .0;
        let qh = self.queue.handle();
        let manager: ZwpIdleInhibitManagerV1 = self.registry.bind(name, 1, &qh, ());
        let inhibitor = manager.create_inhibitor(&self.surface, &qh, ());
        manager.destroy();
        self.conn.flush().unwrap();
        inhibitor
    }

    pub fn unmap(&mut self) {
        self.surface.attach(None, 0, 0);
        self.surface.commit();
        self.conn.flush().unwrap();
    }
    pub fn destroy_window(&mut self) {
        self._top.destroy();
        self.xdg.destroy();
        self.surface.destroy();
        self.queue.roundtrip(&mut self.state).unwrap();
    }
    pub fn new(stream: UnixStream) -> Self {
        Self::named(stream, "headless-pattern", true)
    }

    pub fn named(stream: UnixStream, name: &str, fullscreen: bool) -> Self {
        Self::create(stream, name, fullscreen, false, (200, 160))
    }

    pub fn sized(stream: UnixStream, name: &str, size: (i32, i32), decorated: bool) -> Self {
        Self::create(stream, name, false, decorated, size)
    }

    fn create(
        stream: UnixStream,
        name: &str,
        fullscreen: bool,
        decorated: bool,
        size: (i32, i32),
    ) -> Self {
        mapping_trace!("PATTERN start app_id={name} socket={stream:?}");
        stream.set_nonblocking(true).unwrap();
        let conn = Connection::from_socket(stream).unwrap();
        Self::from_connection(conn, name, fullscreen, decorated, size)
    }

    fn from_connection(
        conn: Connection,
        name: &str,
        fullscreen: bool,
        decorated: bool,
        size: (i32, i32),
    ) -> Self {
        Self::from_connection_requests(conn, name, fullscreen, decorated, size, &[], false)
    }

    pub fn premap_fullscreen(stream: UnixStream, requests: &[bool]) -> Self {
        stream.set_nonblocking(true).unwrap();
        let conn = Connection::from_socket(stream).unwrap();
        Self::from_connection_requests(
            conn,
            "headless-pattern",
            true,
            false,
            (1280, 720),
            requests,
            false,
        )
    }

    /// Opt-in protocol oracle; existing clients keep their original behaviour.
    pub fn scale_probe(stream: UnixStream) -> Self {
        stream.set_nonblocking(true).unwrap();
        Self::from_connection_requests(
            Connection::from_socket(stream).unwrap(),
            "headless-pattern",
            false,
            false,
            (200, 160),
            &[],
            true,
        )
    }

    fn from_connection_requests(
        conn: Connection,
        name: &str,
        fullscreen: bool,
        decorated: bool,
        size: (i32, i32),
        requests: &[bool],
        scale_probe: bool,
    ) -> Self {
        let mut queue = conn.new_event_queue::<State>();
        let qh = queue.handle();
        let registry = conn.display().get_registry(&qh, ());
        let mut state = State::default();
        state.scale_probe = scale_probe;
        state.scale120 = 120;
        queue.roundtrip(&mut state).unwrap();
        let find = |name: &str| {
            state
                .globals
                .iter()
                .find(|(_, interface, _)| interface == name)
                .map(|(n, v, version)| (*n, v.clone(), *version))
                .unwrap()
        };
        let (n, _, v) = find("wl_compositor");
        let compositor: WlCompositor =
            registry.bind(n, v.min(if scale_probe { 6 } else { 4 }), &qh, ());
        let (n, _, _) = find("wl_shm");
        let shm: WlShm = registry.bind(n, 1, &qh, ());
        let (n, _, v) = find("xdg_wm_base");
        let shell: XdgWmBase = registry.bind(n, if scale_probe { v.min(6) } else { 1 }, &qh, ());
        let (n, _, _) = find("wp_presentation");
        let presentation: WpPresentation = registry.bind(n, 1, &qh, ());
        let surface = compositor.create_surface(&qh, ());
        let (viewport, fractional) = if scale_probe {
            let (n, _, _) = find("wp_viewporter");
            let manager: WpViewporter = registry.bind(n, 1, &qh, ());
            let viewport = manager.get_viewport(&surface, &qh, ());
            let (n, _, _) = find("wp_fractional_scale_manager_v1");
            let manager: WpFractionalScaleManagerV1 = registry.bind(n, 1, &qh, ());
            (
                Some(viewport),
                Some(manager.get_fractional_scale(&surface, &qh, ())),
            )
        } else {
            (None, None)
        };
        let xdg = shell.get_xdg_surface(&surface, &qh, ());
        let top = xdg.get_toplevel(&qh, ());
        let decoration = decorated.then(|| {
            let (n, _, _) = find("zxdg_decoration_manager_v1");
            let manager: ZxdgDecorationManagerV1 = registry.bind(n, 1, &qh, ());
            let decoration = manager.get_toplevel_decoration(&top, &qh, ());
            decoration.set_mode(zxdg_toplevel_decoration_v1::Mode::ServerSide);
            decoration
        });
        mapping_trace!("PATTERN get_toplevel app_id={name}");
        top.set_app_id(name.into());
        mapping_trace!("PATTERN set_app_id={name}");
        if !requests.is_empty() {
            for on in requests {
                if *on {
                    top.set_fullscreen(None);
                } else {
                    top.unset_fullscreen();
                }
            }
        } else if fullscreen {
            top.set_fullscreen(None);
        } else {
            top.set_min_size(size.0, size.1);
            top.set_max_size(size.0, size.1);
        }
        for (n, _, v) in state.globals.iter().filter(|(_, i, _)| i == "wl_seat") {
            let _: WlSeat = registry.bind(*n, (*v).min(7), &qh, ());
        }
        surface.commit();
        mapping_trace!("PATTERN initial empty commit app_id={name}");
        let deadline = Instant::now() + Duration::from_secs(10);
        while !state.configured {
            queue.roundtrip(&mut state).unwrap();
            assert!(Instant::now() < deadline, "initial configure");
            thread::sleep(Duration::from_millis(5));
        }
        // xdg-shell may initially offer 0x0 (the client's choice), before
        // viud places the first buffer and sends its decided size.
        if state.size == (0, 0) {
            state.size = if fullscreen { (1280, 720) } else { size };
        }
        let (w, h) = state.size;
        let buffer = buffer(&shm, &qh, w, h, |x, y| {
            COLOURS[usize::from(x >= w / 2) + 2 * usize::from(y >= h / 2)]
        });
        let mut client = Self {
            viewport,
            _fractional: fractional,
            conn,
            queue,
            state,
            surface,
            shm,
            presentation,
            buffer,
            registry,
            compositor,
            shell,
            xdg,
            _top: top,
            _decoration: decoration,
        };
        client.commit(true);
        mapping_trace!(
            "PATTERN initial buffer attached, committed and flushed app_id={name} size={:?}",
            client.state.size
        );
        client
    }

    pub fn pump(&mut self) {
        self.conn.flush().unwrap();
        self.queue.dispatch_pending(&mut self.state).unwrap();
        if let Some(guard) = self.queue.prepare_read() {
            match guard.read() {
                Ok(_) => {}
                Err(wayland_client::backend::WaylandError::Io(e))
                    if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(e) => panic!("pattern read: {e}"),
            }
        }
        self.queue.dispatch_pending(&mut self.state).unwrap();
        if self.state.scale_probe && self.state.scale_dirty {
            self.commit(true);
        }
    }

    pub fn commit(&mut self, damage: bool) {
        if self.state.scale_probe && damage {
            let scale = self.state.scale120;
            let (lw, lh) = self.state.size;
            let w = ((lw as f64 * scale as f64 / 120.0).ceil() as i32).max(1);
            let h = ((lh as f64 * scale as f64 / 120.0).ceil() as i32).max(1);
            self.buffer = buffer(&self.shm, &self.queue.handle(), w, h, |x, y| {
                COLOURS[usize::from(x >= w / 2) + 2 * usize::from(y >= h / 2)]
            });
            self.viewport.as_ref().unwrap().set_destination(lw, lh);
            self.surface.set_buffer_scale(1);
            self.surface.attach(Some(&self.buffer), 0, 0);
            self.surface.damage_buffer(0, 0, w, h);
            self.state.buffer_info = (w, h, scale);
            self.state.log_scale(ScaleEvent::Commit(w, h, scale));
            self.state.scale_dirty = false;
        }
        if damage && !self.state.scale_probe {
            self.surface.attach(Some(&self.buffer), 0, 0);
            self.surface
                .damage_buffer(0, 0, self.state.size.0, self.state.size.1);
        }
        self.surface.frame(&self.queue.handle(), ());
        self.state.commit_id += 1;
        let (w, h, scale) = self.state.buffer_info;
        self.presentation.feedback(&self.surface, &self.queue.handle(), (self.state.commit_id, w, h, scale));
        self.surface.commit();
        self.conn.flush().unwrap();
    }

    pub fn wait_callback(&mut self, previous: u64) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while self.state.callbacks <= previous {
            self.pump();
            assert!(Instant::now() < deadline, "frame callback timed out");
            thread::sleep(Duration::from_millis(2));
        }
    }

    pub fn change_buffer(&mut self) {
        self.solid_buffer(0xffaa3377);
    }

    pub fn solid_buffer(&mut self, colour: u32) {
        let (w, h) = self.state.size;
        self.buffer = buffer(&self.shm, &self.queue.handle(), w, h, |_, _| colour);
    }

    pub fn remap(&mut self) {
        self.surface.attach(None, 0, 0);
        self.surface.commit();
        self.queue.roundtrip(&mut self.state).unwrap();
        self.state.configured = false;
        self.surface.commit();
        while !self.state.configured {
            self.queue.roundtrip(&mut self.state).unwrap();
        }
        self.commit(true);
        self.queue.roundtrip(&mut self.state).unwrap();
    }

    pub fn retake_role(&mut self, name: &str) {
        self.surface.attach(None, 0, 0);
        self.surface.commit();
        self._top.destroy();
        self.xdg.destroy();
        self.queue.roundtrip(&mut self.state).unwrap();
        let qh = self.queue.handle();
        self.xdg = self.shell.get_xdg_surface(&self.surface, &qh, ());
        self._top = self.xdg.get_toplevel(&qh, ());
        self._top.set_app_id(name.into());
        self.state.configured = false;
        self.surface.commit();
        while !self.state.configured {
            self.queue.roundtrip(&mut self.state).unwrap();
        }
        self.commit(true);
        self.queue.roundtrip(&mut self.state).unwrap();
    }

    pub fn popup(&mut self, grab: bool) -> (WlSurface, XdgPopup) {
        let qh = self.queue.handle();
        let surface = self.compositor.create_surface(&qh, ());
        let positioner = self.shell.create_positioner(&qh, ());
        positioner.set_size(80, 60);
        positioner.set_anchor_rect(0, 0, 1, 1);
        positioner.set_anchor(Anchor::TopLeft);
        positioner.set_gravity(Gravity::BottomRight);
        positioner.set_offset(180, 40);
        let xdg = self.shell.get_xdg_surface(&surface, &qh, ());
        let popup = xdg.get_popup(Some(&self.xdg), &positioner, &qh, ());
        if grab {
            let (serial, seat) = self.state.last_press.clone().expect("pressed pointer");
            popup.grab(&seat, serial);
        }
        self.state.popup_configured = false;
        surface.commit();
        let deadline = Instant::now() + Duration::from_secs(5);
        while !self.state.popup_configured {
            self.queue.roundtrip(&mut self.state).unwrap();
            assert!(Instant::now() < deadline, "popup configure timed out");
        }
        let buffer = buffer(&self.shm, &qh, 80, 60, |_, _| 0xffff00ff);
        surface.attach(Some(&buffer), 0, 0);
        surface.damage_buffer(0, 0, 80, 60);
        surface.commit();
        self.queue.roundtrip(&mut self.state).unwrap();
        (surface, popup)
    }

    pub fn lock_session(&mut self) -> ExtSessionLockV1 {
        let (n, _, v) = self
            .state
            .globals
            .iter()
            .find(|(_, name, _)| name == "ext_session_lock_manager_v1")
            .unwrap();
        let manager: ExtSessionLockManagerV1 = self.registry.bind(*n, *v, &self.queue.handle(), ());
        let lock = manager.lock(&self.queue.handle(), ());
        let deadline = Instant::now() + Duration::from_secs(5);
        while !self.state.locked {
            self.pump();
            assert!(Instant::now() < deadline, "session lock not accepted");
            thread::sleep(Duration::from_millis(5));
        }
        lock
    }
}

fn buffer(
    shm: &WlShm,
    qh: &QueueHandle<State>,
    w: i32,
    h: i32,
    colour: impl Fn(i32, i32) -> u32,
) -> WlBuffer {
    let fd = unsafe { libc::memfd_create(c"headless-pattern".as_ptr(), libc::MFD_CLOEXEC) };
    assert!(fd >= 0);
    let file = unsafe { File::from_raw_fd(fd) };
    file.set_len((w * h * 4) as u64).unwrap();
    for y in 0..h {
        let row: Vec<_> = (0..w).flat_map(|x| colour(x, y).to_le_bytes()).collect();
        file.write_all_at(&row, (y * w * 4) as u64).unwrap();
    }
    let pool = shm.create_pool(file.as_fd(), w * h * 4, qh, ());
    let buffer = pool.create_buffer(0, w, h, w * 4, wl_shm::Format::Argb8888, qh, ());
    pool.destroy();
    buffer
}

impl Dispatch<WlRegistry, ()> for State {
    fn event(
        state: &mut Self,
        _: &WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            state.globals.push((name, interface, version));
        }
    }
}
impl Dispatch<XdgWmBase, ()> for State {
    fn event(
        _: &mut Self,
        base: &XdgWmBase,
        event: xdg_wm_base::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_wm_base::Event::Ping { serial } = event {
            base.pong(serial);
        }
    }
}
impl Dispatch<XdgSurface, ()> for State {
    fn event(
        state: &mut Self,
        surface: &XdgSurface,
        event: xdg_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_surface::Event::Configure { serial } = event {
            mapping_trace!("PATTERN xdg_surface.configure serial={serial}");
            surface.ack_configure(serial);
            state.log_scale(ScaleEvent::Ack(serial));
            mapping_trace!("PATTERN ack_configure serial={serial}");
            state.configured = true;
        }
    }
}
impl Dispatch<XdgToplevel, ()> for State {
    fn event(
        state: &mut Self,
        _: &XdgToplevel,
        event: xdg_toplevel::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let xdg_toplevel::Event::Configure {
            width,
            height,
            states,
        } = event
        {
            state.log_scale(ScaleEvent::Configure(
                width,
                height,
                states
                    .chunks_exact(4)
                    .map(|b| u32::from_ne_bytes(b.try_into().unwrap()))
                    .collect(),
            ));
            if state.scale_probe {
                state.scale_dirty = true;
            }
            mapping_trace!("PATTERN xdg_toplevel.configure size={width}x{height}");
            state.size = (width, height);
        }
    }
}
impl Dispatch<WlCallback, ()> for State {
    fn event(
        state: &mut Self,
        _: &WlCallback,
        event: wl_callback::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wl_callback::Event::Done { .. } = event {
            state.log_scale(ScaleEvent::Callback);
            state.callbacks += 1;
        }
    }
}
impl Dispatch<WlSeat, ()> for State {
    fn event(
        state: &mut Self,
        seat: &WlSeat,
        event: wl_seat::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_seat::Event::Capabilities {
            capabilities: WEnum::Value(caps),
        } = event
        {
            if caps.contains(wl_seat::Capability::Pointer) {
                state.pointers.push(seat.get_pointer(qh, seat.clone()));
            }
            if caps.contains(wl_seat::Capability::Keyboard) {
                let _: WlKeyboard = seat.get_keyboard(qh, ());
            }
        }
    }
}
impl Dispatch<WpPresentation, ()> for State {
    fn event(
        state: &mut Self,
        _: &WpPresentation,
        event: wp_presentation::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wp_presentation::Event::ClockId { clk_id } = event {
            state.clock = Some(clk_id);
        }
    }
}
impl Dispatch<WpPresentationFeedback, (u64, i32, i32, u32)> for State {
    fn event(
        state: &mut Self,
        _: &WpPresentationFeedback,
        event: wp_presentation_feedback::Event,
        buffer: &(u64, i32, i32, u32),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wp_presentation_feedback::Event::Presented {
                tv_sec_hi,
                tv_sec_lo,
                tv_nsec,
                flags,
                ..
            } => {
                assert_eq!(
                    flags,
                    wayland_client::WEnum::Value(wp_presentation_feedback::Kind::empty())
                );
                state.presented += 1;
                state.log_scale(ScaleEvent::Presented(buffer.0, buffer.1, buffer.2, buffer.3));
                state.times.push(
                    (((tv_sec_hi as u64) << 32) | tv_sec_lo as u64) * 1_000_000_000
                        + tv_nsec as u64,
                );
            }
            wp_presentation_feedback::Event::Discarded => state.discarded += 1,
            _ => {}
        }
    }
}
delegate_noop!(State: ignore WlCompositor);
delegate_noop!(State: ignore wayland_client::protocol::wl_subcompositor::WlSubcompositor);
delegate_noop!(State: ignore wayland_client::protocol::wl_subsurface::WlSubsurface);
delegate_noop!(State: ignore wayland_protocols::wp::cursor_shape::v1::client::wp_cursor_shape_manager_v1::WpCursorShapeManagerV1);
delegate_noop!(State: ignore wayland_protocols::wp::cursor_shape::v1::client::wp_cursor_shape_device_v1::WpCursorShapeDeviceV1);
delegate_noop!(State: ignore wayland_client::protocol::wl_data_device_manager::WlDataDeviceManager);
delegate_noop!(State: ignore wayland_client::protocol::wl_data_device::WlDataDevice);
delegate_noop!(State: ignore wayland_client::protocol::wl_data_offer::WlDataOffer);
delegate_noop!(State: ignore wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_shell_v1::ZwlrLayerShellV1);
impl
    Dispatch<
        wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        (),
    > for State
{
    fn event(
        state: &mut Self,
        role: &wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::ZwlrLayerSurfaceV1,
        event: wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wayland_protocols_wlr::layer_shell::v1::client::zwlr_layer_surface_v1::Event::Configure {serial,width,height} = event {
            role.ack_configure(serial);
            state.layer_size = Some((width as i32, height as i32));
        }
    }
}
delegate_noop!(State: ignore wayland_protocols::ext::idle_notify::v1::client::ext_idle_notifier_v1::ExtIdleNotifierV1);
impl Dispatch<wayland_protocols::ext::idle_notify::v1::client::ext_idle_notification_v1::ExtIdleNotificationV1, ()> for State {
    fn event(state: &mut Self,
        _: &wayland_protocols::ext::idle_notify::v1::client::ext_idle_notification_v1::ExtIdleNotificationV1,
        event: wayland_protocols::ext::idle_notify::v1::client::ext_idle_notification_v1::Event,
        _: &(), _: &Connection, _: &QueueHandle<Self>) {
        if matches!(event, wayland_protocols::ext::idle_notify::v1::client::ext_idle_notification_v1::Event::Idled) {
            state.idled += 1;
        }
    }
}
delegate_noop!(State: ignore wayland_protocols::wp::idle_inhibit::zv1::client::zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1);
delegate_noop!(State: ignore wayland_protocols::wp::idle_inhibit::zv1::client::zwp_idle_inhibitor_v1::ZwpIdleInhibitorV1);
impl Dispatch<WlSurface, ()> for State {
    fn event(
        state: &mut Self,
        _: &WlSurface,
        event: wayland_client::protocol::wl_surface::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wayland_client::protocol::wl_surface::Event::PreferredBufferScale { factor } = event
        {
            state.log_scale(ScaleEvent::BufferScale(factor));
            if state.scale_probe && !state.fractional_received {
                state.scale120 = factor as u32 * 120;
                state.scale_dirty = true;
            }
        }
    }
}
delegate_noop!(State: ignore WpViewporter);
delegate_noop!(State: ignore WpViewport);
delegate_noop!(State: ignore WpFractionalScaleManagerV1);
impl Dispatch<WpFractionalScaleV1, ()> for State {
    fn event(
        state: &mut Self,
        _: &WpFractionalScaleV1,
        event: wp_fractional_scale_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let wp_fractional_scale_v1::Event::PreferredScale { scale } = event {
            state.log_scale(ScaleEvent::Preferred(scale));
            state.scale120 = scale;
            state.fractional_received = true;
            state.scale_dirty = true;
        }
    }
}
delegate_noop!(State: ignore WlShm);
delegate_noop!(State: WlShmPool);
impl Dispatch<WlBuffer, ()> for State {
    fn event(
        state: &mut Self,
        _: &WlBuffer,
        event: wayland_client::protocol::wl_buffer::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if matches!(event, wayland_client::protocol::wl_buffer::Event::Release) {
            state.buffer_releases += 1;
        }
    }
}
impl Dispatch<WlPointer, WlSeat> for State {
    fn event(
        state: &mut Self,
        pointer: &WlPointer,
        event: wl_pointer::Event,
        seat: &WlSeat,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match &event {
            wl_pointer::Event::Enter {
                surface_x,
                surface_y,
                ..
            }
            | wl_pointer::Event::Motion {
                surface_x,
                surface_y,
                ..
            } => {
                state.pointer_positions.push((*surface_x, *surface_y));
            }
            _ => {}
        }
        if matches!(event, wl_pointer::Event::Motion { .. }) {
            state.pointer_motions += 1;
        }
        if let wl_pointer::Event::Enter {
            serial, surface, ..
        } = &event
        {
            state.last_enter = Some((*serial, pointer.clone()));
            state.pointer_focus = Some(surface.clone());
        }
        if let wl_pointer::Event::Leave { .. } = &event {
            state.pointer_focus = None;
        }
        if let wl_pointer::Event::Button {
            serial,
            state: button_state,
            ..
        } = event
        {
            state.pointer_buttons += 1;
            if button_state == WEnum::Value(wl_pointer::ButtonState::Pressed) {
                state.last_press = Some((serial, seat.clone()));
            }
        }
    }
}

impl Dispatch<WlKeyboard, ()> for State {
    fn event(
        state: &mut Self,
        _: &WlKeyboard,
        event: wl_keyboard::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        match event {
            wl_keyboard::Event::Key { .. } => state.keys += 1,
            wl_keyboard::Event::Enter { .. } => state.keyboard_enters += 1,
            _ => {}
        }
    }
}
impl Dispatch<XdgPopup, ()> for State {
    fn event(
        state: &mut Self,
        _: &XdgPopup,
        event: xdg_popup::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if matches!(event, xdg_popup::Event::Configure { .. }) {
            state.popup_configured = true;
        }
    }
}
delegate_noop!(State: ignore XdgPositioner);
delegate_noop!(State: ignore ZxdgDecorationManagerV1);
delegate_noop!(State: ignore ZxdgToplevelDecorationV1);
delegate_noop!(State: ignore ExtSessionLockManagerV1);
impl Dispatch<ExtSessionLockV1, ()> for State {
    fn event(
        state: &mut Self,
        _: &ExtSessionLockV1,
        event: ext_session_lock_v1::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if matches!(event, ext_session_lock_v1::Event::Locked) {
            state.locked = true;
        }
    }
}
