// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! `comp.motion.run`: one tick of a motion. The timing lives in the `motion`
//! crate; this module moves the agent pointer or the window to where the
//! motion is, and reports whether it has arrived. The engine
//! (`engines/viud/src/comp.rs`) owns the runs and their replies, and calls
//! these once per tick.

use comp_model::ControlReply;
use motion::{Glide, Status, Timeline};
use world::state::Loop;

use crate::{control, input};

/// Install visual tracks without configuring the client. Replacing a live run
/// uses the shared position-continuous retarget helper.
pub fn start_visual(
    lp: &mut Loop,
    id: u64,
    generation: u64,
    preset: &mut motion::Preset,
    started: std::time::Instant,
) -> Result<(), ControlReply> {
    let (_, window) = control::motion_window(lp, id, generation)?;
    if world::comp::animation::active(&window, started) {
        let visual = world::comp::animation::sample(&window, control::window_visual(lp, id, generation)?, started);
        let mut current = motion::Values::default();
        for track in &preset.tracks {
            let value = match track.property {
                motion::Property::Opacity => visual.opacity,
                motion::Property::Scale => visual.scale.factor,
                motion::Property::Tx => visual.translate.x,
                motion::Property::Ty => visual.translate.y,
                _ => continue,
            };
            current.set(track.property, value);
        }
        let target = preset.sample(preset.timing.duration);
        *preset = motion::retarget(&current, &target, preset.timing);
    }
    world::comp::animation::start(&window, preset, started);
    ledger::frame_trace::event("window_motion_start", || (id, 0, 0));
    lp.state.schedule_redraw(dispatcher::state::RedrawReason::WindowState);
    Ok(())
}

/// Persist the final visual sample before releasing a completed run.
pub fn finish_visual(lp: &mut Loop, id: u64, generation: u64) {
    if let Ok((_, window)) = control::motion_window(lp, id, generation) {
        world::comp::animation::finish(&window);
        let Ok(visual) = control::window_visual(lp, id, generation) else { return };
        let _ = control::window_visual_step(lp, id, generation, visual);
    }
}

pub fn cancel_visual(lp: &mut Loop, id: u64, generation: u64) {
    if let Ok((_, window)) = control::motion_window(lp,id,generation) {
        world::comp::animation_geometry::cancel(&window);
    }
    if let Ok(visual) = control::window_visual(lp, id, generation) {
        if let Ok((_, window)) = control::motion_window(lp, id, generation) {
            world::comp::animation::finish(&window);
        }
        let _ = control::window_visual_step(lp, id, generation, visual);
    }
}

pub fn start_geometry(lp: &mut Loop, id: u64, generation: u64, from: [f64;4], to: [Option<f64>;4], started: std::time::Instant, duration: f64, easing: motion::Easing) {
    if let Ok((_, window)) = control::motion_window(lp,id,generation) {
        world::comp::animation_geometry::cancel(&window);
        if to.iter().any(Option::is_some) {
            world::comp::animation_geometry::start(&window,started,from,to,duration,easing);
        }
    }
}

pub fn geometry_frame_step(lp: &mut Loop, id: u64, generation: u64, timeline: &Timeline, now: f64) -> Step {
    let (_, window) = match control::motion_window(lp,id,generation) { Ok(window) => window, Err(refusal) => return Step::Refused(refusal) };
    // Timer wakes request frames; only frame preparation samples in-flight values.
    if timeline.status_at(now) == Status::Done {
        world::comp::animation_geometry::sample_window(lp,&window,std::time::Instant::now());
    }
    lp.state.schedule_redraw(dispatcher::state::RedrawReason::WindowState);
    arrived_or_moving(timeline.status_at(now))
}

/// What one tick of a motion found.
#[derive(Debug, PartialEq)]
pub enum Step {
    /// The motion is under way and has further to go.
    Moving,
    /// The motion is at its last point; the run is finished.
    Arrived,
    /// The seat or the window refused the move. The run ends and its caller
    /// gets this refusal.
    Refused(ControlReply),
}

/// Move the agent pointer to where `glide` is at `now` seconds, on `output`
/// (or the default output when `None`).
pub fn pointer_step(lp: &mut Loop, glide: &mut Glide, output: Option<&str>, now: f64) -> Step {
    let (point, status) = glide.sample(now);
    if let Err(refusal) = input::glide_step(lp, output, point.x, point.y) {
        return Step::Refused(refusal);
    }
    arrived_or_moving(status)
}

/// Move window `id` (at `generation`) to where its motion is at `now`
/// seconds. `from` is where it started, `to` the target with `None` fields
/// unchanged, and `timeline` gives the eased progress.
pub fn window_step(
    lp: &mut Loop,
    id: u64,
    generation: u64,
    from: [f64; 4],
    to: [Option<f64>; 4],
    timeline: &mut Timeline,
    now: f64,
) -> Step {
    let progress = timeline.progress(now);
    let mut rect = [0.0; 4];
    for (i, slot) in rect.iter_mut().enumerate() {
        let end = to[i].unwrap_or(from[i]);
        *slot = from[i] + (end - from[i]) * progress;
    }
    if to.iter().any(Option::is_some) {
        if let Err(refusal) = control::window_glide_step(lp, id, generation, rect) {
            return Step::Refused(refusal);
        }
    } else if let Err(refusal) = control::window_rect(lp, id, generation) {
        return Step::Refused(refusal);
    }
    arrived_or_moving(timeline.status_at(now))
}

fn arrived_or_moving(status: Status) -> Step {
    match status {
        Status::Running => Step::Moving,
        Status::Done => Step::Arrived,
    }
}
