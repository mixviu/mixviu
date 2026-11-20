// SPDX-License-Identifier: MIT OR Apache-2.0
//! Geometry tracks are committed on the same clock as visual composition.
use crate::state::Loop;
use dispatcher::wire::trait_::wire_trait::WireTrait;
use motion::{Easing, Timeline};
use smithay::desktop::Window;
use smithay::utils::{Logical, Point, Size};
use std::sync::Mutex;
use std::time::Instant;

struct Run {
    started: Instant,
    from: [f64; 4],
    to: [Option<f64>; 4],
    timeline: Timeline,
}

pub fn start(
    window: &Window,
    started: Instant,
    from: [f64; 4],
    to: [Option<f64>; 4],
    duration: f64,
    easing: Easing,
) {
    window
        .user_data()
        .insert_if_missing(|| Mutex::new(None::<Run>));
    *window
        .user_data()
        .get::<Mutex<Option<Run>>>()
        .unwrap()
        .lock()
        .unwrap() = Some(Run {
        started,
        from,
        to,
        timeline: Timeline::new(0.0, duration, easing),
    });
}

pub fn sample_window(lp: &mut Loop, window: &Window, now: Instant) {
    let Some(cell) = window.user_data().get::<Mutex<Option<Run>>>() else {
        return;
    };
    let mut guard = cell.lock().unwrap();
    let Some(run) = guard.as_mut() else { return };
    let elapsed = now.saturating_duration_since(run.started).as_secs_f64();
    let progress = run.timeline.progress(elapsed);
    let rect: [f64; 4] = std::array::from_fn(|i| {
        run.from[i] + (run.to[i].unwrap_or(run.from[i]) - run.from[i]) * progress
    });
    if let Some(owner) = lp.inner.world_of_window(window) {
        let location =
            Point::<i32, Logical>::from((rect[0].round() as i32, rect[1].round() as i32));
        let wanted = Size::<i32, Logical>::from((
            (rect[2].round() as i32).max(1),
            (rect[3].round() as i32).max(1),
        ));
        if (run.to[2].is_some() || run.to[3].is_some())
            && crate::camera::transform::translate::slot::expected_size(window) != Some(wanted)
        {
            protocols::window::shell::stage(window, wanted, false);
            protocols::window::shell::send(window);
            crate::camera::transform::translate::slot::set_expected_size(window, wanted);
        }
        lp.inner
            .space_of_mut(owner)
            .state
            .map_element(window.clone(), location, false);
    }
    if elapsed >= run.timeline.end() {
        *guard = None;
    }
}

pub fn sample_all(lp: &mut Loop, now: Instant) {
    let windows: Vec<_> = lp
        .inner
        .all_world_spaces()
        .into_iter()
        .flat_map(|space| space.state.elements().cloned())
        .collect();
    for window in windows {
        sample_window(lp, &window, now);
    }
}

pub fn cancel(window: &Window) {
    if let Some(cell) = window.user_data().get::<Mutex<Option<Run>>>() {
        *cell.lock().unwrap() = None;
    }
}
