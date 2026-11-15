//! Finger velocity and fling deadlines; all time comes from the caller.
use std::time::{Duration, Instant};

use egui::Vec2;

// Match egui 0.37 KineticScrollStyle's 0.5 s exponential coast and 0.5
// point remaining-distance cutoff. 60 Hz bounds host wakeups; the 100 ms
// sample window rejects fingers resting before release, not a measured tune.
const DECAY_SECONDS: f32 = 0.5;
const STOP_SPEED: f32 = 0.5 / DECAY_SECONDS;
pub(super) const FRAME: Duration = Duration::from_nanos(16_666_667);
const SAMPLE_WINDOW: Duration = Duration::from_millis(100);

#[derive(Clone, Default)]
struct Axis {
    sample: Option<(Instant, u32)>,
    velocity: f32,
    fling: bool,
    last_step: Option<Instant>,
}

#[derive(Clone, Default)]
pub(super) struct Kinetic {
    finger: bool,
    axes: [Axis; 2],
    deadline: Option<Instant>,
}

impl Kinetic {
    pub(super) fn restarts(finger: Option<bool>, delta: Vec2, stop: [bool; 2]) -> bool {
        // A fling belongs to a lifted finger. Movement on either axis means
        // the finger is down again, so no axis may keep coasting.
        finger == Some(false)
            || delta != Vec2::ZERO
            || (finger.is_some() && !stop.iter().any(|s| *s))
    }
    pub(super) fn interrupts(event: &egui::Event) -> bool {
        matches!(
            event,
            egui::Event::PointerButton { .. } | egui::Event::Key { .. } | egui::Event::PointerGone
        )
    }
    pub(super) fn cancel(&mut self) {
        *self = Self::default();
    }

    /// Retain only samples belonging to axes still driven by the finger.
    pub(super) fn live_drag(&self) -> Self {
        let mut live = self.clone();
        for axis in &mut live.axes {
            if axis.fling {
                *axis = Axis::default();
            }
        }
        live.deadline = None;
        live
    }

    pub(super) fn deadline(&self) -> Option<Instant> {
        self.deadline
    }

    pub(super) fn axis(
        &mut self,
        time_ms: u32,
        now: Instant,
        finger: Option<bool>,
        delta: Vec2,
        stop: [bool; 2],
    ) {
        if finger == Some(false) {
            self.cancel();
            return;
        }
        if Self::restarts(finger, delta, stop) {
            // New movement cancels every coasting axis, retaining only live
            // finger samples. A fresh gesture never inherits fling velocity.
            for axis in &mut self.axes {
                if axis.fling {
                    *axis = Axis::default();
                }
            }
            self.deadline = None;
        }
        self.finger = finger.unwrap_or(self.finger);
        if !self.finger {
            return;
        }
        for (i, axis) in self.axes.iter_mut().enumerate() {
            if delta[i] != 0.0 {
                axis.velocity = axis.sample.map_or(0.0, |(_, last_ms)| {
                    // Wayland timestamps, not dispatch intervals: queued frames
                    // can be delivered microseconds apart. Wrap is protocol-defined.
                    let dt = Duration::from_millis(u64::from(time_ms.wrapping_sub(last_ms)));
                    if dt.is_zero() || dt > SAMPLE_WINDOW {
                        0.0
                    } else {
                        delta[i] / dt.as_secs_f32()
                    }
                });
                axis.sample = Some((now, time_ms));
            }
            if stop[i] && axis.sample.is_some() {
                axis.fling = axis.sample.is_some_and(|(last, last_ms)| {
                    // Also reject dispatch stalls: starting a fling late is
                    // worse than omitting it, even with fresh protocol times.
                    now.saturating_duration_since(last) <= SAMPLE_WINDOW
                        && Duration::from_millis(u64::from(time_ms.wrapping_sub(last_ms)))
                            <= SAMPLE_WINDOW
                }) && axis.velocity.is_finite()
                    && axis.velocity.abs() >= STOP_SPEED;
                axis.sample = None;
                axis.last_step = axis.fling.then_some(now);
            }
        }
        if self.axes.iter().any(|a| a.fling) && self.deadline.is_none() {
            self.deadline = Some(now + FRAME);
        }
        if stop.iter().any(|s| *s) && self.axes.iter().all(|a| a.sample.is_none()) {
            self.finger = false;
        }
    }

    pub(super) fn step(&mut self, now: Instant) -> Vec2 {
        if self.deadline.is_none_or(|at| at > now) {
            return Vec2::ZERO;
        }
        let mut delta = Vec2::ZERO;
        for (i, axis) in self.axes.iter_mut().enumerate() {
            if axis.fling {
                let dt = now
                    .saturating_duration_since(axis.last_step.unwrap())
                    .as_secs_f32();
                let decay = (-dt / DECAY_SECONDS).exp();
                let next = axis.velocity * decay;
                delta[i] = DECAY_SECONDS * (axis.velocity - next);
                axis.velocity = next;
                axis.fling = next.abs() >= STOP_SPEED;
                axis.last_step = axis.fling.then_some(now);
            }
        }
        self.deadline = self.axes.iter().any(|a| a.fling).then_some(now + FRAME);
        delta
    }
}

/// Both egui and a fling share the surface's one-shot repaint timer.
pub(super) fn deadline(repaint: Option<Instant>, fling: Option<Instant>) -> Option<Instant> {
    repaint.into_iter().chain(fling).min()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fling(t: Instant) -> Kinetic {
        let mut k = Kinetic::default();
        k.axis(0, t, Some(true), Vec2::splat(10.0), [false; 2]);
        k.axis(16, t + FRAME, Some(true), Vec2::splat(10.0), [false; 2]);
        k.axis(16, t + FRAME, None, Vec2::ZERO, [true; 2]);
        k
    }

    #[test]
    fn decays_monotonically_and_disarms_the_shared_timer() {
        let t = Instant::now();
        let mut k = fling(t);
        let mut previous = f32::INFINITY;
        let mut n = 0;
        while let Some(at) = deadline(None, k.deadline()) {
            let d = k.step(at).x;
            assert!(d > 0.0 && d < previous);
            previous = d;
            n += 1;
            assert!(n < 1000);
        }
        assert!(n > 10);
        assert_eq!(k.step(t + Duration::from_secs(60)), Vec2::ZERO);
        assert_eq!(deadline(None, k.deadline()), None);
    }

    #[test]
    fn cancel_removes_samples_velocity_and_deadline() {
        let t = Instant::now();
        let mut k = fling(t);
        k.cancel();
        assert_eq!(deadline(None, k.deadline()), None);
        assert_eq!(k.step(t + Duration::from_secs(1)), Vec2::ZERO);
        k.axis(16, t + FRAME, Some(true), Vec2::ZERO, [true; 2]);
        assert_eq!(k.deadline(), None);
    }

    #[test]
    fn each_input_cancel_cause_stops_before_the_next_tick() {
        let t = Instant::now();
        for event in [
            egui::Event::PointerGone,
            egui::Event::PointerButton {
                pos: egui::Pos2::ZERO,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
            egui::Event::Key {
                key: egui::Key::A,
                physical_key: None,
                pressed: true,
                repeat: false,
                modifiers: egui::Modifiers::NONE,
            },
        ] {
            let mut k = fling(t);
            assert!(Kinetic::interrupts(&event));
            k.cancel();
            assert_eq!(k.step(t + FRAME * 2), Vec2::ZERO);
            assert_eq!(deadline(None, k.deadline()), None);
        }
    }

    #[test]
    fn wheel_continuous_and_missing_stop_never_fling() {
        let t = Instant::now();
        for finger in [None, Some(false), Some(true)] {
            let mut k = Kinetic::default();
            k.axis(0, t, finger, Vec2::splat(10.0), [false; 2]);
            k.axis(16, t + FRAME, finger, Vec2::splat(10.0), [false; 2]);
            assert_eq!(k.deadline(), None);
            if finger != Some(true) {
                k.axis(16, t + FRAME, finger, Vec2::ZERO, [true; 2]);
                assert_eq!(k.deadline(), None);
            }
        }
    }

    #[test]
    fn new_scroll_and_stale_release_do_not_reuse_velocity() {
        let t = Instant::now();
        let mut k = fling(t);
        k.axis(33, t + FRAME * 2, Some(true), Vec2::splat(-2.0), [false; 2]);
        assert_eq!(k.deadline(), None);
        k.axis(
            1000,
            t + Duration::from_secs(1),
            None,
            Vec2::ZERO,
            [true; 2],
        );
        assert_eq!(k.deadline(), None);
    }

    #[test]
    fn axis_stops_are_independent_and_egui_deadline_survives() {
        let t = Instant::now();
        let mut k = Kinetic::default();
        k.axis(0, t, Some(true), Vec2::splat(10.0), [false; 2]);
        k.axis(16, t + FRAME, None, Vec2::splat(10.0), [true, false]);
        let at = k.deadline().unwrap();
        assert!(k.step(at).x > 0.0);
        assert_eq!(k.step(at).y, 0.0);
        assert_eq!(deadline(Some(t), k.deadline()), Some(t));
        k.cancel();
        assert_eq!(deadline(Some(t), k.deadline()), Some(t));
    }

    #[test]
    fn buffered_events_use_wrapping_protocol_time_not_dispatch_time() {
        let t = Instant::now();
        let mut k = Kinetic::default();
        k.axis(u32::MAX - 7, t, Some(true), Vec2::splat(10.0), [false; 2]);
        k.axis(8, t, None, Vec2::splat(10.0), [false; 2]);
        k.axis(8, t, None, Vec2::ZERO, [true; 2]);
        let d = k.step(t + FRAME);
        assert!(d.x > 9.0 && d.x < 11.0, "{d:?}");
    }

    #[test]
    fn source_only_restart_cancels_fling() {
        let t = Instant::now();
        let mut k = fling(t);
        k.axis(0, t + FRAME, Some(true), Vec2::ZERO, [false; 2]);
        assert_eq!(k.deadline(), None);
    }

    #[test]
    fn staggered_axis_stops_decay_from_each_release() {
        let t = Instant::now();
        let mut k = Kinetic::default();
        k.axis(0, t, Some(true), Vec2::splat(10.0), [false; 2]);
        k.axis(
            16,
            t + Duration::from_millis(16),
            None,
            Vec2::splat(10.0),
            [false; 2],
        );
        let x_release = t + Duration::from_millis(16);
        let y_release = t + Duration::from_millis(32);
        k.axis(16, x_release, None, Vec2::ZERO, [true, false]);
        k.axis(32, y_release, None, Vec2::ZERO, [false, true]);
        let at = k.deadline().unwrap();
        let delta = k.step(at);
        let distance = |since: Instant, until: Instant| {
            DECAY_SECONDS * 625.0 * (1.0 - (-(until - since).as_secs_f32() / DECAY_SECONDS).exp())
        };
        assert!((delta.x - distance(x_release, at)).abs() < 0.0001);
        assert!((delta.y - distance(y_release, at)).abs() < 0.0001);
        assert!(delta.y < delta.x);
        let next = k.deadline().unwrap();
        let total = delta + k.step(next);
        assert!((total.x - distance(x_release, next)).abs() < 0.0001);
        assert!((total.y - distance(y_release, next)).abs() < 0.0001);
    }

    #[test]
    fn duplicate_or_unsampled_stops_leave_a_coast_unchanged() {
        let t = Instant::now();
        let mut k = Kinetic::default();
        k.axis(0, t, Some(true), Vec2::splat(10.0), [false; 2]);
        // Keep y live so another x stop reaches the per-axis stop logic.
        k.axis(16, t + FRAME, None, Vec2::splat(10.0), [true, false]);
        let mut control = k.clone();
        let at = k.deadline().unwrap();
        k.axis(
            24,
            t + Duration::from_millis(24),
            None,
            Vec2::ZERO,
            [true, false],
        );
        assert_eq!(k.deadline(), Some(at));
        assert_eq!(k.step(at), control.step(at));
        let mut empty = Kinetic::default();
        empty.axis(0, t, Some(true), Vec2::ZERO, [true; 2]);
        assert_eq!(empty.deadline(), None);
    }

    #[test]
    fn fresh_protocol_release_after_dispatch_stall_does_not_fling() {
        let t = Instant::now();
        let mut k = Kinetic::default();
        k.axis(0, t, Some(true), Vec2::splat(10.0), [false; 2]);
        k.axis(16, t + FRAME, None, Vec2::splat(10.0), [false; 2]);
        k.axis(
            17,
            t + Duration::from_millis(200),
            None,
            Vec2::ZERO,
            [true; 2],
        );
        assert_eq!(k.deadline(), None);
    }
}
