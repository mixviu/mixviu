//! The one-shot timer owner, independent of Wayland/GPU state.
use std::time::Instant;

use smithay_client_toolkit::reexports::calloop::timer::{TimeoutAction, Timer};
use smithay_client_toolkit::reexports::calloop::{LoopHandle, RegistrationToken};

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
        if let Some(token) = self.token.take() {
            handle.remove(token);
        }
        self.deadline = None;
    }

    /// Returns false if registration failed; the caller can draw immediately.
    pub(super) fn arm<T: 'static>(
        &mut self,
        handle: &LoopHandle<'static, T>,
        at: Option<Instant>,
        mut fire: impl FnMut(&mut T) + 'static,
    ) -> bool {
        if self.deadline == at {
            return true;
        }
        self.cancel(handle);
        let Some(at) = at else { return true };
        match handle.insert_source(Timer::from_deadline(at), move |_, _, state| {
            fire(state);
            TimeoutAction::Drop
        }) {
            Ok(token) => {
                self.token = Some(token);
                self.deadline = Some(at);
                true
            }
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::layer::kinetic::{FRAME, Kinetic};
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
        // Natural completion, output loss, leave, button, key and new scroll.
        for cause in 0..6 {
            let mut event_loop: EventLoop<'static, Harness> = EventLoop::try_new().unwrap();
            let t = Instant::now();
            let mut k = Kinetic::default();
            k.axis(0, t, Some(true), egui::Vec2::splat(10.0), [false; 2]);
            k.axis(16, t + FRAME, None, egui::Vec2::splat(10.0), [false; 2]);
            k.axis(16, t + FRAME, None, egui::Vec2::ZERO, [true; 2]);
            let mut s = Harness {
                timer: RepaintTimer::default(),
                kinetic: k,
                now: t,
                fired: 0,
                handle: event_loop.handle(),
            };
            s.arm();
            match cause {
                0 => {}
                1 => s.kinetic.cancel(), // output loss drops/cancels the timer owner
                2..=4 => {
                    let event = match cause {
                        2 => egui::Event::PointerGone,
                        3 => egui::Event::PointerButton {
                            pos: egui::Pos2::ZERO,
                            button: egui::PointerButton::Primary,
                            pressed: true,
                            modifiers: egui::Modifiers::NONE,
                        },
                        _ => egui::Event::Key {
                            key: egui::Key::A,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers: egui::Modifiers::NONE,
                        },
                    };
                    assert!(Kinetic::interrupts(&event));
                    s.kinetic.cancel();
                }
                _ => s.kinetic.axis(
                    32,
                    t + FRAME * 2,
                    Some(true),
                    egui::Vec2::splat(1.0),
                    [false; 2],
                ),
            }
            s.arm();
            for _ in 0..1000 {
                event_loop.dispatch(Duration::ZERO, &mut s).unwrap();
                if s.kinetic.deadline().is_none() {
                    break;
                }
            }
            assert_eq!(s.kinetic.deadline(), None);
            assert!(s.timer.token.is_none());
            assert!(s.timer.deadline.is_none());
            let fired = s.fired;
            event_loop.dispatch(Duration::ZERO, &mut s).unwrap();
            assert_eq!(s.fired, fired);
            assert_eq!(fired == 0, cause != 0);
        }
    }
}
