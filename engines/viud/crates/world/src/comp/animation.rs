// SPDX-License-Identifier: MIT OR Apache-2.0
//! Composition-only window tracks. The frame owner supplies the sample clock.
use std::cell::Cell;
use std::sync::Mutex;
use std::time::Instant;

use comp_model::request::VisualAttributes;
use motion::{Preset, Property, Values};
use smithay::desktop::Window;

thread_local! {
    static FRAME_TIME: Cell<Option<Instant>> = const { Cell::new(None) };
}

/// Pin every scene contributor to the same presentation timestamp. Input uses
/// the most recently drawn timestamp until the next frame is assembled.
pub fn set_frame_time(now: Instant) {
    FRAME_TIME.with(|time| time.set(Some(now)));
}

pub fn frame_time() -> Instant {
    FRAME_TIME
        .with(|time| time.get())
        .unwrap_or_else(Instant::now)
}

#[derive(Clone, Debug)]
pub struct VisualRun {
    pub started: Instant,
    pub preset: Preset,
}

impl VisualRun {
    pub fn elapsed(&self, now: Instant) -> f64 {
        now.saturating_duration_since(self.started).as_secs_f64()
    }

    pub fn sample(&self, now: Instant) -> Values {
        self.preset.sample(self.elapsed(now))
    }

    pub fn active(&self, now: Instant) -> bool {
        self.elapsed(now) < self.preset.timing.start() + self.preset.timing.duration
    }
}

pub fn start(window: &Window, preset: &Preset, started: Instant) {
    replaced_by(window, "run");
    window
        .user_data()
        .insert_if_missing(|| Mutex::new(None::<VisualRun>));
    *window
        .user_data()
        .get::<Mutex<Option<VisualRun>>>()
        .unwrap()
        .lock()
        .unwrap() = Some(VisualRun {
        started,
        preset: preset.clone(),
    });
}

pub fn replaced_by(window: &Window, cause: &'static str) {
    *window
        .user_data()
        .get_or_insert(|| Mutex::new("run"))
        .lock()
        .unwrap() = cause;
}

pub fn replacement_cause(window: &Window) -> &'static str {
    window
        .user_data()
        .get::<Mutex<&'static str>>()
        .map(|v| *v.lock().unwrap())
        .unwrap_or("run")
}

pub fn sampled_values(window: &Window, now: Instant) -> Option<Values> {
    window
        .user_data()
        .get::<Mutex<Option<VisualRun>>>()?
        .lock()
        .unwrap()
        .as_ref()
        .map(|run| run.sample(now))
}

pub fn run(window: &Window) -> Option<VisualRun> {
    window
        .user_data()
        .get::<Mutex<Option<VisualRun>>>()?
        .lock()
        .unwrap()
        .clone()
}

/// Cancel at the last presented sample, without first applying the endpoint.
pub fn cancel(window: &Window) {
    if let Some(cell) = window.user_data().get::<Mutex<Option<VisualRun>>>() {
        cell.lock().unwrap().take();
    }
}

/// Settle an expired run before placement, or hold the last drawn sample on
/// explicit hide. The caller then applies the supplied placement fields.
pub fn prepare_place(window: &Window, base: VisualAttributes, hide: bool, now: Instant) -> VisualAttributes {
    if started(window).is_some() && !active(window, now) {
        finish(window);
        return *window.user_data().get::<Mutex<VisualAttributes>>().unwrap().lock().unwrap();
    }
    if hide {
        let visual = sample(window, base, frame_time());
        cancel(window);
        return visual;
    }
    base
}

/// Placement takes ownership only of the fields it supplies. Remaining tracks
/// keep their original clock and endpoint; finish cannot restore removed ones.
pub fn place(window: &Window, spec: &comp_model::request::PlaceSpec) -> bool {
    let Some(cell) = window.user_data().get::<Mutex<Option<VisualRun>>>() else {
        return false;
    };
    let mut slot = cell.lock().unwrap();
    let Some(run) = slot.as_mut() else {
        return false;
    };
    let before = run.preset.tracks.len();
    run.preset.tracks.retain(|track| !match track.property {
        Property::Opacity => spec.opacity.is_some(),
        Property::Scale => spec.scale.is_some(),
        Property::Tx | Property::Ty => spec.translate.is_some(),
        _ => false,
    });
    let changed = before != run.preset.tracks.len();
    if changed {
        replaced_by(window, "place");
    }
    if run.preset.tracks.is_empty() {
        *slot = None;
    }
    changed
}

/// Identity of the current visual run, including its terminal filled sample.
pub fn started(window: &Window) -> Option<Instant> {
    window
        .user_data()
        .get::<Mutex<Option<VisualRun>>>()?
        .lock()
        .unwrap()
        .as_ref()
        .map(|run| run.started)
}

pub fn active(window: &Window, now: Instant) -> bool {
    window
        .user_data()
        .get::<Mutex<Option<VisualRun>>>()
        .is_some_and(|cell| {
            cell.lock()
                .unwrap()
                .as_ref()
                .is_some_and(|run| run.active(now))
        })
}

pub fn apply(values: &Values, mut visual: VisualAttributes) -> VisualAttributes {
    if values.contains(Property::Opacity) {
        visual.opacity = values.get(Property::Opacity).clamp(0.0, 1.0);
    }
    if values.contains(Property::Scale) {
        visual.scale.factor = values.get(Property::Scale).clamp(0.0, 64.0);
    }
    if values.contains(Property::Tx) {
        visual.translate.x = values
            .get(Property::Tx)
            .clamp(i32::MIN as f64, i32::MAX as f64);
    }
    if values.contains(Property::Ty) {
        visual.translate.y = values
            .get(Property::Ty)
            .clamp(i32::MIN as f64, i32::MAX as f64);
    }
    visual
}

pub fn sample(window: &Window, base: VisualAttributes, now: Instant) -> VisualAttributes {
    sampled_values(window, now).map_or(base, |values| apply(&values, base))
}

/// Persist the sampled endpoint before dropping the track, so removing the
/// final timer cannot make the next frame snap back to the pre-animation state.
pub fn finish(window: &Window) {
    let Some(cell) = window.user_data().get::<Mutex<Option<VisualRun>>>() else {
        return;
    };
    let Some(run) = cell.lock().unwrap().take() else {
        return;
    };
    let values = run
        .preset
        .sample(run.preset.timing.start() + run.preset.timing.duration);
    window
        .user_data()
        .insert_if_missing(|| Mutex::new(VisualAttributes::default()));
    let mut visual = window
        .user_data()
        .get::<Mutex<VisualAttributes>>()
        .unwrap()
        .lock()
        .unwrap();
    *visual = apply(&values, *visual);
}
