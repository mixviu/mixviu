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

#[derive(Default)]
struct Axis {
    sample: Option<Instant>,
    velocity: f32,
    fling: bool,
}

#[derive(Default)]
pub(super) struct Kinetic {
    finger: bool,
    axes: [Axis; 2],
    last_step: Option<Instant>,
    deadline: Option<Instant>,
}

impl Kinetic {
    pub(super) fn cancel(&mut self) {
        *self = Self::default();
    }

    pub(super) fn deadline(&self) -> Option<Instant> {
        self.deadline
    }

    pub(super) fn axis(
        &mut self,
        now: Instant,
        finger: Option<bool>,
        delta: Vec2,
        stop: [bool; 2],
    ) {
        if finger == Some(false) {
            self.cancel();
            return;
        }
        if delta != Vec2::ZERO {
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
                axis.velocity = axis.sample.map_or(0.0, |last| {
                    let dt = now.saturating_duration_since(last);
                    if dt.is_zero() || dt > SAMPLE_WINDOW {
                        0.0
                    } else {
                        delta[i] / dt.as_secs_f32()
                    }
                });
                axis.sample = Some(now);
            }
            if stop[i] {
                axis.fling = axis.sample.is_some_and(|last| {
                    now.saturating_duration_since(last) <= SAMPLE_WINDOW
                }) && axis.velocity.is_finite()
                    && axis.velocity.abs() >= STOP_SPEED;
                axis.sample = None;
            }
        }
        if self.axes.iter().any(|a| a.fling) && self.deadline.is_none() {
            self.last_step = Some(now);
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
        let dt = now.saturating_duration_since(self.last_step.unwrap()).as_secs_f32();
        let decay = (-dt / DECAY_SECONDS).exp();
        let mut delta = Vec2::ZERO;
        for (i, axis) in self.axes.iter_mut().enumerate() {
            if axis.fling {
                let next = axis.velocity * decay;
                delta[i] = DECAY_SECONDS * (axis.velocity - next);
                axis.velocity = next;
                axis.fling = next.abs() >= STOP_SPEED;
            }
        }
        self.last_step = Some(now);
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
        k.axis(t, Some(true), Vec2::splat(10.0), [false; 2]);
        k.axis(t + FRAME, Some(true), Vec2::splat(10.0), [false; 2]);
        k.axis(t + FRAME, None, Vec2::ZERO, [true; 2]);
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
        k.axis(t + FRAME, Some(true), Vec2::ZERO, [true; 2]);
        assert_eq!(k.deadline(), None);
    }

    #[test]
    fn wheel_continuous_and_missing_stop_never_fling() {
        let t = Instant::now();
        for finger in [None, Some(false), Some(true)] {
            let mut k = Kinetic::default();
            k.axis(t, finger, Vec2::splat(10.0), [false; 2]);
            k.axis(t + FRAME, finger, Vec2::splat(10.0), [false; 2]);
            assert_eq!(k.deadline(), None);
            if finger != Some(true) {
                k.axis(t + FRAME, finger, Vec2::ZERO, [true; 2]);
                assert_eq!(k.deadline(), None);
            }
        }
    }

    #[test]
    fn new_scroll_and_stale_release_do_not_reuse_velocity() {
        let t = Instant::now();
        let mut k = fling(t);
        k.axis(t + FRAME * 2, Some(true), Vec2::splat(-2.0), [false; 2]);
        assert_eq!(k.deadline(), None);
        k.axis(t + Duration::from_secs(1), None, Vec2::ZERO, [true; 2]);
        assert_eq!(k.deadline(), None);
    }

    #[test]
    fn axis_stops_are_independent_and_egui_deadline_survives() {
        let t = Instant::now();
        let mut k = Kinetic::default();
        k.axis(t, Some(true), Vec2::splat(10.0), [false; 2]);
        k.axis(t + FRAME, None, Vec2::splat(10.0), [true, false]);
        let at = k.deadline().unwrap();
        assert!(k.step(at).x > 0.0);
        assert_eq!(k.step(at).y, 0.0);
        assert_eq!(deadline(Some(t), k.deadline()), Some(t));
        k.cancel();
        assert_eq!(deadline(Some(t), k.deadline()), Some(t));
    }
}
