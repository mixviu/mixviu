//! The one-shot timer owner, independent of Wayland/GPU state.
use std::time::Instant;

use smithay_client_toolkit::reexports::calloop::{LoopHandle, RegistrationToken};
use smithay_client_toolkit::reexports::calloop::timer::{Timer, TimeoutAction};

#[derive(Default)]
pub(super) struct RepaintTimer {
    deadline: Option<Instant>,
    token: Option<RegistrationToken>,
}

impl RepaintTimer {
    pub(super) fn fired(&mut self) {
        self.deadline = None;
        self.token = None;
    }

    pub(super) fn cancel<T>(&mut self, handle: &LoopHandle<'_, T>) {
        if let Some(token) = self.token.take() { handle.remove(token); }
        self.deadline = None;
    }

    /// Returns false if registration failed; the caller can draw immediately.
    pub(super) fn arm<T: 'static>(
        &mut self,
        handle: &LoopHandle<'static, T>,
        at: Option<Instant>,
        mut fire: impl FnMut(&mut T) + 'static,
    ) -> bool {
        if self.deadline == at { return true; }
        self.cancel(handle);
        let Some(at) = at else { return true };
        match handle.insert_source(Timer::from_deadline(at), move |_, _, state| {
            fire(state);
            TimeoutAction::Drop
        }) {
            Ok(token) => { self.token = Some(token); self.deadline = Some(at); true }
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::kinetic::{Kinetic, FRAME};
    use smithay_client_toolkit::reexports::calloop::EventLoop;
    use std::time::Duration;

    struct Harness {
        timer: RepaintTimer,
        kinetic: Kinetic,
        now: Instant,
        fired: usize,
        handle: LoopHandle<'static, Harness>,
    }
    impl Harness {
        fn arm(&mut self) {
            // The simulation clock advances to the fling deadline in the
            // callback. Make the real timer immediately ready; no sleeping.
            let at = self.kinetic.deadline().map(|_| Instant::now());
            assert!(self.timer.arm(&self.handle, at, |s| {
                s.timer.fired();
                s.now = s.kinetic.deadline().unwrap();
                s.kinetic.step(s.now);
                s.fired += 1;
                s.arm();
            }));
        }
    }

    #[test]
    fn real_registration_is_removed_after_settle_or_cancel() {
        for cancel in [false, true] {
            let mut event_loop: EventLoop<'static, Harness> = EventLoop::try_new().unwrap();
            let t = Instant::now();
            let mut k = Kinetic::default();
            k.axis(0, t, Some(true), egui::Vec2::splat(10.0), [false; 2]);
            k.axis(16, t + FRAME, None, egui::Vec2::splat(10.0), [false; 2]);
            k.axis(16, t + FRAME, None, egui::Vec2::ZERO, [true; 2]);
            let mut s = Harness { timer: RepaintTimer::default(), kinetic: k,
                now: t, fired: 0, handle: event_loop.handle() };
            s.arm();
            if cancel { s.kinetic.cancel(); s.arm(); }
            for _ in 0..1000 {
                event_loop.dispatch(Duration::ZERO, &mut s).unwrap();
                if s.kinetic.deadline().is_none() { break; }
            }
            assert_eq!(s.kinetic.deadline(), None);
            assert!(s.timer.token.is_none());
            assert!(s.timer.deadline.is_none());
            let fired = s.fired;
            event_loop.dispatch(Duration::ZERO, &mut s).unwrap();
            assert_eq!(s.fired, fired);
            assert_eq!(fired == 0, cancel);
        }
    }
}
