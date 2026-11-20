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
        let visual = world::comp::animation::sample(
            &window,
            control::window_visual(lp, id, generation)?,
            started,
        );
        let mut current = motion::Values::default();
        let previous = world::comp::animation::sampled_values(&window, started).unwrap_or_default();
        for property in motion::Property::ALL {
            if !previous.contains(property) && !preset.tracks.iter().any(|t| t.property == property)
            {
                continue;
            }
            let value = match property {
                motion::Property::Opacity => visual.opacity,
                motion::Property::Scale => visual.scale.factor,
                motion::Property::Tx => visual.translate.x,
                motion::Property::Ty => visual.translate.y,
                _ => continue,
            };
            current.set(property, value);
        }
        let old = world::comp::animation::run(&window).unwrap();
        *preset = continuation(&current, preset.clone(), &old, started);
    }
    world::comp::animation::start(&window, preset, started);
    ledger::frame_trace::event("window_motion_start", || (id, 0, 0));
    lp.state
        .schedule_redraw(dispatcher::state::RedrawReason::WindowState);
    Ok(())
}

/// Persist the final visual sample before releasing a completed run.
pub fn finish_visual(lp: &mut Loop, id: u64, generation: u64) {
    if let Ok((_, window)) = control::motion_window(lp, id, generation) {
        world::comp::animation::finish(&window);
        let Ok(visual) = control::window_visual(lp, id, generation) else {
            return;
        };
        let _ = control::window_visual_step(lp, id, generation, visual);
    }
}

pub fn cancel_visual(lp: &mut Loop, id: u64, generation: u64) {
    if let Ok((_, window)) = control::motion_window(lp, id, generation) {
        world::comp::animation_geometry::cancel(&window);
    }
    if let Ok(visual) = control::window_visual(lp, id, generation) {
        if let Ok((_, window)) = control::motion_window(lp, id, generation) {
            world::comp::animation::cancel(&window);
        }
        let _ = control::window_visual_step(lp, id, generation, visual);
    }
}

pub fn visual_replaced(
    lp: &mut Loop,
    id: u64,
    generation: u64,
    started: std::time::Instant,
) -> bool {
    control::motion_window(lp, id, generation)
        .is_ok_and(|(_, window)| world::comp::animation::started(&window) != Some(started))
}

pub fn replacement_cause(lp: &Loop, id: u64, generation: u64) -> &'static str {
    control::motion_window(lp, id, generation)
        .map(|(_, window)| world::comp::animation::replacement_cause(&window))
        .unwrap_or("lifecycle")
}

pub fn cancel_geometry(lp: &mut Loop, id: u64, generation: u64) {
    if let Ok((_, window)) = control::motion_window(lp, id, generation) {
        world::comp::animation_geometry::cancel(&window);
    }
}

pub fn start_geometry(
    lp: &mut Loop,
    target: (u64, u64),
    from: [f64; 4],
    to: [Option<f64>; 4],
    started: std::time::Instant,
    duration: f64,
    easing: motion::Easing,
) {
    let (id, generation) = target;
    if let Ok((_, window)) = control::motion_window(lp, id, generation) {
        world::comp::animation_geometry::cancel(&window);
        if to.iter().any(Option::is_some) {
            world::comp::animation_geometry::start(&window, started, from, to, duration, easing);
        }
    }
}

pub fn geometry_frame_step(
    lp: &mut Loop,
    id: u64,
    generation: u64,
    timeline: &Timeline,
    now: f64,
) -> Step {
    let (_, window) = match control::motion_window(lp, id, generation) {
        Ok(window) => window,
        Err(refusal) => return Step::Refused(refusal),
    };
    // Timer wakes request frames; only frame preparation samples in-flight values.
    if timeline.status_at(now) == Status::Done {
        world::comp::animation_geometry::sample_window(lp, &window, lp.inner.comp.clock.now());
    }
    lp.state
        .schedule_redraw(dispatcher::state::RedrawReason::WindowState);
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

fn continuation(
    current: &motion::Values,
    mut preset: motion::Preset,
    old: &world::comp::animation::VisualRun,
    started: std::time::Instant,
) -> motion::Preset {
    let requested = preset.sample(preset.timing.duration);
    let remaining = (old.preset.timing.duration - old.elapsed(started)).max(0.0);
    let requested_duration = preset.timing.duration;
    let mut target = motion::Values::default();
    for property in motion::Property::ALL {
        if requested.contains(property) {
            target.set(property, requested.get(property));
        }
    }
    let mut requested_current = motion::Values::default();
    for track in &preset.tracks {
        requested_current.set(track.property, current.get(track.property));
    }
    preset = motion::retarget(&requested_current, &target, preset.timing);
    let duration = requested_duration.max(remaining);
    if duration > 0.0 {
        for track in &mut preset.tracks {
            if requested_duration == 0.0 {
                track.keyframes[0].value = track.keyframes.last().unwrap().value;
            }
            for frame in &mut track.keyframes {
                frame.offset *= requested_duration / duration;
            }
            if requested_duration == 0.0 {
                track.keyframes.remove(1);
            }
            if track.keyframes.last().unwrap().offset < 1.0 {
                let mut endpoint = track.keyframes.last().unwrap().clone();
                endpoint.offset = 1.0;
                track.keyframes.push(endpoint);
            }
        }
        let endpoint = old.preset.sample(old.preset.timing.duration);
        for old_track in &old.preset.tracks {
            if preset
                .tracks
                .iter()
                .any(|track| track.property == old_track.property)
            {
                continue;
            }
            let property = old_track.property;
            let mut frames = vec![motion::Keyframe {
                offset: 0.0,
                value: current.get(property),
                easing: motion::Curve::Linear,
            }];
            if remaining > 0.0 && remaining < duration {
                frames.push(motion::Keyframe {
                    offset: remaining / duration,
                    value: endpoint.get(property),
                    easing: motion::Curve::Linear,
                });
            }
            frames.push(motion::Keyframe {
                offset: 1.0,
                value: endpoint.get(property),
                easing: motion::Curve::Linear,
            });
            preset.tracks.push(motion::Track {
                property,
                keyframes: frames,
            });
        }
        preset.timing.duration = duration;
    }

    preset
}

#[cfg(test)]
mod review_tests {
    use super::*;
    #[test]
    fn omitted_scale_keeps_its_original_deadline() {
        let now = std::time::Instant::now();
        let mut old = motion::presets()
            .into_iter()
            .find(|p| p.name == "fade")
            .unwrap();
        old.timing.duration = 0.8;
        old.tracks = vec![motion::Track {
            property: motion::Property::Scale,
            keyframes: vec![
                motion::Keyframe {
                    offset: 0.0,
                    value: 0.4,
                    easing: motion::Curve::Linear,
                },
                motion::Keyframe {
                    offset: 1.0,
                    value: 0.8,
                    easing: motion::Curve::Linear,
                },
            ],
        }];
        let old = world::comp::animation::VisualRun {
            preset: old,
            started: now - std::time::Duration::from_millis(250),
        };
        let mut replacement = motion::presets()
            .into_iter()
            .find(|p| p.name == "fade")
            .unwrap();
        replacement.timing.duration = 0.5;
        let result = continuation(&old.sample(now), replacement, &old, now);
        assert!(result.validate().is_ok());
        for (time, value) in [(0.0, 0.525), (0.275, 0.6625), (0.55, 0.8), (1.0, 0.8)] {
            assert!((result.sample(time).get(motion::Property::Scale) - value).abs() < 1e-6);
        }
        assert_eq!(result.sample(0.5).get(motion::Property::Opacity), 1.0);
    }
}
