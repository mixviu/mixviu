// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! One supervisor for VT notifications, watcher recovery and seat-open backoff.
use super::Handles;
use smithay::output::Output;
use smithay::reexports::calloop::{
    LoopHandle, RegistrationToken,
    channel::{Event, Sender, channel},
    timer::{TimeoutAction, Timer},
};
use std::{
    cell::RefCell,
    ffi::OsString,
    rc::Rc,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use world::state::Loop;

static WATCH_FAILED: AtomicBool = AtomicBool::new(false);
static WATCH_RUNNING: AtomicBool = AtomicBool::new(false);
pub fn watcher_running() -> bool {
    WATCH_RUNNING.load(Ordering::Relaxed)
}
pub fn watcher_failed() -> bool {
    WATCH_FAILED.load(Ordering::Relaxed)
}
pub(super) fn fixture_watcher_failed(failed: bool) {
    WATCH_FAILED.store(failed, Ordering::Relaxed);
}

enum Notice {
    Ready,
    Active,
    Stopped(String),
}

#[derive(Default)]
pub(super) struct Backoff(Duration);
impl Backoff {
    fn reset(&mut self) {
        self.0 = Duration::ZERO;
    }
    pub(super) fn next(&mut self) -> Duration {
        self.0 = (self.0 * 2).clamp(Duration::from_millis(250), Duration::from_secs(5));
        self.0
    }
}

struct Activation {
    handles: Handles,
    output: Output,
    socket: OsString,
    handle: LoopHandle<'static, Loop>,
    tx: Sender<Notice>,
    watcher_alive: bool,
    watcher_warned: bool,
    retry: Backoff,
    timer: Option<RegistrationToken>,
    stop: std::sync::Arc<AtomicBool>,
}

fn spawn_watcher(
    stop: &AtomicBool,
    run: impl FnOnce() + Send + 'static,
) -> std::io::Result<std::thread::JoinHandle<()>> {
    stop.store(false, Ordering::Relaxed);
    std::thread::Builder::new()
        .name("viud-vt-wait".into())
        .spawn(run)
}

impl Activation {
    fn start_watcher(&mut self) {
        self.watcher_alive = true;
        let tx = self.tx.clone();
        let stop = self.stop.clone();
        let started = spawn_watcher(&self.stop, move || {
            WATCH_RUNNING.store(true, Ordering::Relaxed);
            let result = std::panic::catch_unwind(|| {
                kms::seat::session::vt_wait::watch_own_vt(
                    || {
                        let _ = tx.send(Notice::Active);
                    },
                    || {
                        let _ = tx.send(Notice::Ready);
                    },
                    &stop,
                )
            });
            let error = match result {
                Ok(Err(error)) => error.to_string(),
                Ok(Ok(())) => "VT source ended".into(),
                Err(_) => "VT source panicked".into(),
            };
            WATCH_RUNNING.store(false, Ordering::Relaxed);
            if stop.load(Ordering::Relaxed) {
                info!("VT watcher exited after native attachment; libseat owns VT notifications");
            }
            let _ = tx.send(Notice::Stopped(error));
        });
        if let Err(error) = started {
            self.watcher_alive = false;
            WATCH_FAILED.store(true, Ordering::Relaxed);
            warn!("VT watcher start failed; retrying: {error}");
        }
    }
}

fn notice(
    shared: &Rc<RefCell<Activation>>,
    notice: Notice,
    failed: &AtomicBool,
    attempt: impl FnOnce(),
) -> bool {
    if shared.borrow().handles.borrow().is_some() {
        return false;
    }
    match notice {
        // Readiness proves the watcher recovered, but must not reset seat or
        // watcher backoff: the source may still fail immediately afterwards.
        Notice::Ready => failed.store(false, Ordering::Relaxed),
        Notice::Active => attempt(),
        Notice::Stopped(error) => {
            failed.store(true, Ordering::Relaxed);
            let warn = {
                let mut activation = shared.borrow_mut();
                activation.watcher_alive = false;
                !std::mem::replace(&mut activation.watcher_warned, true)
            };
            if warn {
                warn!("VT watcher stopped; offscreen desk will re-arm it: {error}");
            }
            arm(shared);
            return warn;
        }
    }
    false
}

fn arm(shared: &Rc<RefCell<Activation>>) {
    let mut activation = shared.borrow_mut();
    if activation.timer.is_some() || activation.handles.borrow().is_some() {
        return;
    }
    let delay = activation.retry.next();
    let shared = shared.clone();
    let token = activation
        .handle
        .insert_source(Timer::from_duration(delay), move |_, _, state| {
            shared.borrow_mut().timer = None;
            attempt(&shared, state);
            TimeoutAction::Drop
        })
        .expect("activation retry timer registration");
    activation.timer = Some(token);
}

fn attempt(shared: &Rc<RefCell<Activation>>, state: &mut Loop) {
    attempt_with(shared, &WATCH_FAILED, |activation| {
        if !kms::seat::session::vt_wait::own_vt_active() {
            return None;
        }
        let Some(session) = kms::seat::session::factory::try_create() else {
            return Some(false);
        };
        let native = native::wire::entry::wire(
            state,
            activation.socket.clone(),
            &activation.handle,
            Some(activation.output.clone()),
            session,
        );
        native::wire::session::activate(state, &native.ctx);
        *activation.handles.borrow_mut() = Some(native);
        Some(true)
    });
}

// The seat transaction is injectable; retry, stop and status publication are
// the production path even when the test has no VT, DRM device or libseat.
fn attempt_with(
    shared: &Rc<RefCell<Activation>>,
    failed: &AtomicBool,
    attach: impl FnOnce(&Activation) -> Option<bool>,
) {
    let mut activation = shared.borrow_mut();
    if activation.handles.borrow().is_some() {
        return;
    }
    if !activation.watcher_alive {
        activation.start_watcher();
    }
    if !activation.watcher_alive {
        drop(activation);
        arm(shared);
        return;
    }
    let Some(attached) = attach(&activation) else {
        return;
    };
    if !attached {
        drop(activation);
        arm(shared);
        return;
    }
    activation.stop.store(true, Ordering::Relaxed);
    if let Some(timer) = activation.timer.take() {
        activation.handle.remove(timer);
    }
    failed.store(false, Ordering::Relaxed);
    activation.retry.reset();
}

pub fn install(
    state: &mut Loop,
    socket: OsString,
    output: Output,
) -> Result<Handles, Box<dyn std::error::Error>> {
    if !std::path::Path::new("/dev/tty0").exists() {
        warn!(
            "/dev/tty0 is not exposed: the VT monitor can use the session tty, but seatd console access may fail; check comp.session.get activation_error when the VT activates"
        );
    }
    let handles = Rc::new(RefCell::new(None));
    let handle = state.loop_handle.clone();
    let (tx, source) = channel();
    let shared = Rc::new(RefCell::new(Activation {
        handles: handles.clone(),
        socket,
        output,
        handle: handle.clone(),
        tx,
        watcher_alive: false,
        watcher_warned: false,
        retry: Backoff::default(),
        timer: None,
        stop: Default::default(),
    }));
    let callback = shared.clone();
    handle.insert_source(source, move |event, _, state| {
        let Event::Msg(notice) = event else { return };
        self::notice(&callback, notice, &WATCH_FAILED, || {
            attempt(&callback, state)
        });
    })?;
    shared.borrow_mut().start_watcher();
    if !shared.borrow().watcher_alive {
        arm(&shared);
    }
    Ok(handles)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn watcher_spawn_clears_stop_before_running() {
        let stop = std::sync::Arc::new(AtomicBool::new(true));
        let observed = stop.clone();
        spawn_watcher(&stop, move || assert!(!observed.load(Ordering::Relaxed)))
            .unwrap()
            .join()
            .unwrap();
    }
    #[test]
    fn ready_then_seat_failure_or_watcher_failure_grows_retry_interval() {
        // Exercise the production notice handler without opening a VT/seat.
        for seat_failure in [true, false] {
            let failed = AtomicBool::new(true);
            let event_loop = smithay::reexports::calloop::EventLoop::<Loop>::try_new().unwrap();
            let (tx, _) = channel();
            let activation = Rc::new(RefCell::new(Activation {
                handles: Rc::new(RefCell::new(None)),
                output: Output::new(
                    "test".into(),
                    smithay::output::PhysicalProperties {
                        size: (0, 0).into(),
                        subpixel: smithay::output::Subpixel::Unknown,
                        make: "test".into(),
                        model: "test".into(),
                        serial_number: "test".into(),
                    },
                ),
                socket: "test".into(),
                handle: event_loop.handle(),
                tx,
                watcher_alive: true,
                watcher_warned: false,
                retry: Backoff::default(),
                timer: None,
                stop: Default::default(),
            }));
            let mut warnings = 0;
            for expected in [250, 500, 1000, 2000, 4000, 5000, 5000, 5000] {
                notice(&activation, Notice::Ready, &failed, || panic!("not active"));
                assert!(!failed.load(Ordering::Relaxed));
                if seat_failure {
                    notice(&activation, Notice::Active, &failed, || arm(&activation));
                }
                warnings += usize::from(notice(
                    &activation,
                    Notice::Stopped("test failure".into()),
                    &failed,
                    || panic!("not active"),
                ));
                assert_eq!(activation.borrow().retry.0, Duration::from_millis(expected));
                let mut activation = activation.borrow_mut();
                let token = activation.timer.take().unwrap();
                activation.handle.remove(token);
            }
            assert_eq!(
                warnings, 1,
                "readiness clears status, not the warning latch"
            );
            activation.borrow_mut().watcher_alive = true;
            arm(&activation);
            // Run the actual attach attempt with only the hardware transaction
            // replaced. A failure keeps status/backoff; success retires retry.
            attempt_with(&activation, &failed, |_| Some(false));
            assert!(failed.load(Ordering::Relaxed));
            assert!(activation.borrow().timer.is_some());
            attempt_with(&activation, &failed, |_| Some(true));
            assert!(!failed.load(Ordering::Relaxed));
            let activation = activation.borrow();
            assert!(activation.stop.load(Ordering::Relaxed));
            assert!(activation.timer.is_none());
            assert_eq!(activation.retry.0, Duration::ZERO);
        }
    }

    #[test]
    fn seat_and_watcher_retries_are_bounded_and_do_not_require_another_edge() {
        let mut retry = Backoff::default();
        assert_eq!(retry.next(), Duration::from_millis(250));
        assert_eq!(retry.next(), Duration::from_millis(500));
        for _ in 0..32 {
            assert!(retry.next() <= Duration::from_secs(5));
        }
        assert_eq!(retry.next(), Duration::from_secs(5));
        retry.reset();
        assert_eq!(retry.next(), Duration::from_millis(250));
    }
}
