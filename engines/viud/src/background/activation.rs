// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! One supervisor for VT notifications, watcher recovery and seat-open backoff.
use super::Handles;
use smithay::output::Output;
use smithay::reexports::calloop::{LoopHandle, RegistrationToken, channel::{channel, Event, Sender}, timer::{Timer, TimeoutAction}};
use std::{cell::RefCell, ffi::OsString, rc::Rc, sync::atomic::{AtomicBool, Ordering}, time::Duration};
use world::state::Loop;

static WATCH_FAILED: AtomicBool = AtomicBool::new(false);
pub fn watcher_failed() -> bool { WATCH_FAILED.load(Ordering::Relaxed) }
pub(super) fn fixture_watcher_failed(failed: bool) { WATCH_FAILED.store(failed, Ordering::Relaxed); }

enum Notice { Ready, Active, Stopped(String) }

#[derive(Default)]
pub(super) struct Backoff(Duration);
impl Backoff {
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
    retry: Backoff,
    timer: Option<RegistrationToken>,
}

impl Activation {
    fn start_watcher(&mut self) {
        self.watcher_alive = true;
        let tx = self.tx.clone();
        let started = std::thread::Builder::new().name("viud-vt-wait".into()).spawn(move || {
            let result = std::panic::catch_unwind(|| {
                kms::seat::session::vt_wait::watch_own_vt(
                    || { let _ = tx.send(Notice::Active); },
                    || { let _ = tx.send(Notice::Ready); },
                )
            });
            let error = match result {
                Ok(Err(error)) => error.to_string(),
                Ok(Ok(())) => "VT source ended".into(),
                Err(_) => "VT source panicked".into(),
            };
            let _ = tx.send(Notice::Stopped(error));
        });
        if let Err(error) = started {
            self.watcher_alive = false;
            WATCH_FAILED.store(true, Ordering::Relaxed);
            warn!("VT watcher start failed; retrying: {error}");
        }
    }
}

fn arm(shared: &Rc<RefCell<Activation>>) {
    let mut activation = shared.borrow_mut();
    if activation.timer.is_some() || activation.handles.borrow().is_some() { return; }
    let delay = activation.retry.next();
    let shared = shared.clone();
    let token = activation.handle.insert_source(Timer::from_duration(delay), move |_, _, state| {
        shared.borrow_mut().timer = None;
        attempt(&shared, state);
        TimeoutAction::Drop
    }).expect("activation retry timer registration");
    activation.timer = Some(token);
}

fn attempt(shared: &Rc<RefCell<Activation>>, state: &mut Loop) {
    let mut activation = shared.borrow_mut();
    if activation.handles.borrow().is_some() { return; }
    if !activation.watcher_alive { activation.start_watcher(); }
    if !activation.watcher_alive {
        drop(activation);
        arm(shared);
        return;
    }
    if !kms::seat::session::vt_wait::own_vt_active() { return; }
    let Some(session) = kms::seat::session::factory::try_create() else {
        drop(activation);
        arm(shared);
        return;
    };
    let native = native::wire::entry::wire(state, activation.socket.clone(), &activation.handle, Some(activation.output.clone()), session);
    native::wire::session::activate(state, &native.ctx);
    *activation.handles.borrow_mut() = Some(native);
    if let Some(timer) = activation.timer.take() { activation.handle.remove(timer); }
    WATCH_FAILED.store(false, Ordering::Relaxed);
}

pub fn install(state: &mut Loop, socket: OsString, output: Output) -> Result<Handles, Box<dyn std::error::Error>> {
    let handles = Rc::new(RefCell::new(None));
    let handle = state.loop_handle.clone();
    let (tx, source) = channel();
    let shared = Rc::new(RefCell::new(Activation { handles: handles.clone(), socket, output, handle: handle.clone(), tx, watcher_alive: false, retry: Backoff::default(), timer: None }));
    let callback = shared.clone();
    handle.insert_source(source, move |event, _, state| {
        let Event::Msg(notice) = event else { return };
        if callback.borrow().handles.borrow().is_some() { return; }
        match notice {
            Notice::Ready => { WATCH_FAILED.store(false, Ordering::Relaxed); }
            Notice::Active => { attempt(&callback, state); }
            Notice::Stopped(error) => {
                callback.borrow_mut().watcher_alive = false;
                WATCH_FAILED.store(true, Ordering::Relaxed);
                warn!("VT watcher stopped; offscreen desk will re-arm it: {error}");
                arm(&callback);
            }
        }
    })?;
    shared.borrow_mut().start_watcher();
    if !shared.borrow().watcher_alive { arm(&shared); }
    Ok(handles)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn seat_and_watcher_retries_are_bounded_and_do_not_require_another_edge() {
        let mut retry = Backoff::default();
        assert_eq!(retry.next(), Duration::from_millis(250));
        assert_eq!(retry.next(), Duration::from_millis(500));
        for _ in 0..32 { assert!(retry.next() <= Duration::from_secs(5)); }
        assert_eq!(retry.next(), Duration::from_secs(5));
    }
}
