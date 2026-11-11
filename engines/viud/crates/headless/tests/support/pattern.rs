// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Shm test client: four opaque quadrants. Cursor tests use the agent arrow.
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
    globals: Vec<(u32, String, u32)>,
    configured: bool,
    pointers: Vec<WlPointer>,
    pub size: (i32, i32),
    pub callbacks: u64,
    pub pointer_buttons: u64,
    pub keys: u64,
    pub keyboard_enters: u64,
    last_press: Option<(u32, WlSeat)>,
    popup_configured: bool,
    pub locked: bool,
    pub presented: u64,
    pub discarded: u64,
    pub clock: Option<u32>,
    pub times: Vec<u64>,
}

pub struct Pattern {
    conn: Connection,
    queue: EventQueue<State>,
    pub state: State,
    surface: WlSurface,
    shm: WlShm,
    presentation: WpPresentation,
    buffer: WlBuffer,
    registry: WlRegistry,
    compositor: WlCompositor,
    shell: XdgWmBase,
    xdg: XdgSurface,
    _top: XdgToplevel,
}

impl Pattern {
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
    pub fn new(stream: UnixStream) -> Self {
        Self::named(stream, "headless-pattern", true)
    }

    pub fn named(stream: UnixStream, name: &str, fullscreen: bool) -> Self {
        stream.set_nonblocking(true).unwrap();
        let conn = Connection::from_socket(stream).unwrap();
        let mut queue = conn.new_event_queue::<State>();
        let qh = queue.handle();
        let registry = conn.display().get_registry(&qh, ());
        let mut state = State::default();
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
        let compositor: WlCompositor = registry.bind(n, v.min(4), &qh, ());
        let (n, _, _) = find("wl_shm");
        let shm: WlShm = registry.bind(n, 1, &qh, ());
        let (n, _, _) = find("xdg_wm_base");
        let shell: XdgWmBase = registry.bind(n, 1, &qh, ());
        let (n, _, _) = find("wp_presentation");
        let presentation: WpPresentation = registry.bind(n, 1, &qh, ());
        let surface = compositor.create_surface(&qh, ());
        let xdg = shell.get_xdg_surface(&surface, &qh, ());
        let top = xdg.get_toplevel(&qh, ());
        top.set_app_id(name.into());
        if fullscreen {
            top.set_fullscreen(None);
        } else {
            top.set_min_size(200, 160);
            top.set_max_size(200, 160);
        }
        for (n, _, v) in state.globals.iter().filter(|(_, i, _)| i == "wl_seat") {
            let _: WlSeat = registry.bind(*n, (*v).min(7), &qh, ());
        }
        surface.commit();
        let deadline = Instant::now() + Duration::from_secs(10);
        while !state.configured {
            queue.roundtrip(&mut state).unwrap();
            assert!(Instant::now() < deadline, "initial configure");
            thread::sleep(Duration::from_millis(5));
        }
        // xdg-shell may initially offer 0x0 (the client's choice), before
        // viud places the first buffer and sends its decided size.
        if state.size == (0, 0) {
            state.size = if fullscreen { (1280, 720) } else { (200, 160) };
        }
        let (w, h) = state.size;
        let buffer = buffer(&shm, &qh, w, h, |x, y| {
            COLOURS[usize::from(x >= w / 2) + 2 * usize::from(y >= h / 2)]
        });
        let mut client = Self {
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
        };
        client.commit(true);
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
    }

    pub fn commit(&mut self, damage: bool) {
        if damage {
            self.surface.attach(Some(&self.buffer), 0, 0);
            self.surface
                .damage_buffer(0, 0, self.state.size.0, self.state.size.1);
        }
        self.surface.frame(&self.queue.handle(), ());
        self.presentation
            .feedback(&self.surface, &self.queue.handle(), ());
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
        let (w, h) = self.state.size;
        self.buffer = buffer(&self.shm, &self.queue.handle(), w, h, |_, _| 0xffaa3377);
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
            surface.ack_configure(serial);
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
        if let xdg_toplevel::Event::Configure { width, height, .. } = event {
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
impl Dispatch<WpPresentationFeedback, ()> for State {
    fn event(
        state: &mut Self,
        _: &WpPresentationFeedback,
        event: wp_presentation_feedback::Event,
        _: &(),
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
delegate_noop!(State: ignore wayland_protocols::wp::idle_inhibit::zv1::client::zwp_idle_inhibit_manager_v1::ZwpIdleInhibitManagerV1);
delegate_noop!(State: ignore wayland_protocols::wp::idle_inhibit::zv1::client::zwp_idle_inhibitor_v1::ZwpIdleInhibitorV1);
delegate_noop!(State: ignore WlSurface);
delegate_noop!(State: ignore WlShm);
delegate_noop!(State: WlShmPool);
delegate_noop!(State: ignore WlBuffer);
impl Dispatch<WlPointer, WlSeat> for State {
    fn event(
        state: &mut Self,
        _: &WlPointer,
        event: wl_pointer::Event,
        seat: &WlSeat,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
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
