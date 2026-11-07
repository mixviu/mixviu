// SPDX-License-Identifier: MIT OR Apache-2.0
//! Gold cursor phase: 530 ms halves, steady without keyboard focus.
use std::time::{Duration, Instant};
const HALF: Duration = Duration::from_millis(530);

pub(super) struct Blink {
    epoch: Instant,
    focused: bool,
}
impl Default for Blink {
    fn default() -> Self {
        Self {
            epoch: Instant::now(),
            focused: false,
        }
    }
}
impl Blink {
    pub fn activity(&mut self, now: Instant) {
        self.epoch = now;
    }
    pub fn frame(
        &mut self,
        now: Instant,
        focused: bool,
        enabled: bool,
        cursor: bool,
    ) -> (bool, Option<Duration>) {
        if focused && !self.focused {
            self.activity(now);
        }
        self.focused = focused;
        if !focused || !enabled || !cursor {
            return (true, None);
        }
        let elapsed = now.saturating_duration_since(self.epoch);
        let phase = elapsed.as_nanos() / HALF.as_nanos();
        (
            phase.is_multiple_of(2),
            Some(Duration::from_nanos(
                (HALF.as_nanos() - elapsed.as_nanos() % HALF.as_nanos()) as u64,
            )),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn phases_focus_activity_and_hidden_cursor_do_not_busy_repaint() {
        let start = Instant::now();
        let mut blink = Blink::default();
        assert_eq!(blink.frame(start, true, true, true), (true, Some(HALF)));
        assert_eq!(
            blink.frame(start + HALF, true, true, true),
            (false, Some(HALF))
        );
        assert_eq!(
            blink.frame(start + HALF * 2, true, true, true),
            (true, Some(HALF))
        );
        let now = start + HALF * 3;
        assert_eq!(blink.frame(now, false, true, true), (true, None));
        assert_eq!(blink.frame(now, true, true, true), (true, Some(HALF)));
        assert_eq!(blink.frame(now + HALF, true, true, false), (true, None));
        assert_eq!(blink.frame(now + HALF, true, false, true), (true, None));
        blink.activity(now + HALF);
        assert_eq!(
            blink.frame(now + HALF, true, true, true),
            (true, Some(HALF))
        );
        assert_eq!(
            blink.frame(now + HALF + Duration::from_millis(529), true, true, true),
            (true, Some(Duration::from_millis(1)))
        );
    }
}
