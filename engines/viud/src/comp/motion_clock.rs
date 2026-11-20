// SPDX-License-Identifier: MIT OR Apache-2.0
//! Animation invalidations follow each output's refresh clock, not loop wakes.
use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

#[derive(Default)]
pub(super) struct MotionClock {
    next: BTreeMap<String, Instant>,
}

impl MotionClock {
    pub fn due(&self, now: Instant) -> bool {
        self.next.values().any(|at| now >= *at) || self.next.is_empty()
    }
    /// Coalesce all runs into one request per output interval. Unrelated wakes
    /// keep the existing deadline. A missed interval is skipped, never replayed.
    pub fn step(
        &mut self,
        now: Instant,
        active: bool,
        outputs: impl IntoIterator<Item = (String, Duration)>,
        mut request: impl FnMut(&str),
    ) -> Option<Instant> {
        if !active {
            self.next.clear();
            return None;
        }
        let outputs: BTreeMap<_, _> = outputs.into_iter().collect();
        self.next.retain(|key, _| outputs.contains_key(key));
        for (key, interval) in outputs {
            let at = self.next.entry(key.clone()).or_insert(now);
            if now >= *at {
                request(&key);
                // Preserve the output's phase even after a late dispatch.
                let interval = interval.max(Duration::from_nanos(1));
                let missed = now.duration_since(*at).as_nanos() / interval.as_nanos();
                *at += interval.mul_f64((missed + 1) as f64);
            }
        }
        self.next.values().copied().min()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mixed_refresh_outputs_bound_requests_and_keep_deadlines_across_busy_wakes() {
        let start = Instant::now();
        let mut clock = MotionClock::default();
        let mut counts = BTreeMap::<String, usize>::new();
        for us in 0..1_000_000 {
            let next = clock.step(
                start + Duration::from_micros(us),
                true,
                [
                    ("fast".into(), Duration::from_millis(10)),
                    ("slow".into(), Duration::from_millis(20)),
                ],
                |key| *counts.entry(key.into()).or_default() += 1,
            );
            let expected = start + Duration::from_millis((us / 10_000 + 1) * 10);
            assert_eq!(
                next,
                Some(expected),
                "unrelated wakes cannot postpone a tick"
            );
        }
        assert_eq!(counts["fast"], 100);
        assert_eq!(counts["slow"], 50);
        let end = start + Duration::from_secs(1);
        assert_eq!(
            clock.step(end, false, [], |_| panic!(
                "settled scene requested a frame"
            )),
            None
        );
        assert!(clock.next.is_empty(), "no idle deadline");
        println!("MOTION_CLOCK requests fast=100/100 slow=50/50 idle=0");
    }
}
