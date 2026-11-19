// SPDX-License-Identifier: MIT OR Apache-2.0
//! Optional lifecycle policy, using the same tracks as explicit window runs.
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Instant;
use model::environment::preference::base::{WindowMotion, WindowTransition};
use motion::{Direction, Fill, Preset, Property};
use smithay::desktop::Window;
use uuid::Uuid;
use crate::state::Loop;
use dispatcher::wire::trait_::wire_trait::WireTrait;
use crate::window::interface::record::window::LoopWindow;

#[derive(Clone, Copy, PartialEq)]
struct Facts { mapped: bool, minimised: bool, workspace: Option<u32>, shown: bool }

#[derive(Default)]
struct State {
    previous: HashMap<Uuid, Facts>,
    pub exits: HashMap<Uuid, super::animation::VisualRun>,
    settings: WindowMotion,
    active: bool,
}

thread_local! { static STATE: RefCell<State> = RefCell::new(State::default()); }

pub fn preset(config: &WindowTransition, reverse: bool) -> Option<Preset> {
    if config.duration_ms > 10_000 { return None; }
    let mut preset = motion::presets().into_iter().find(|p| p.name == config.preset)?;
    preset.tracks.retain(|track| matches!(track.property,
        Property::Opacity | Property::Scale | Property::Tx | Property::Ty));
    if preset.tracks.is_empty() { return None; }
    preset.timing.duration = config.duration_ms as f64 / 1000.0;
    preset.timing.fill = Fill::Both;
    preset.timing.delay = 0.0;
    preset.timing.iterations = motion::Iterations::Finite(1.0);
    preset.timing.direction = if reverse { Direction::Reverse } else { Direction::Normal };
    Some(preset)
}

/// Observe after protocol/policy changes. A close removes input immediately;
/// its outgoing image lives separately, with no registry or Window reference.
pub fn service(lp: &mut Loop) -> bool {
    let now = lp.inner.comp.clock.now();
    let settings = lp.inner.preference.window_motion.clone();
    let windows: HashMap<Uuid, Window> = lp.inner.all_world_spaces().into_iter()
        .flat_map(|space| space.state.elements())
        .filter_map(|window| Some((window.uuid()?, window.clone())))
        .collect();
    let current: HashMap<Uuid, Facts> = lp.inner.comp.registry.surface_rows()
        .filter(|r| r.role().managed_toplevel())
        .filter_map(|r| Some((r.uuid()?, Facts {
            mapped: r.mapped(), minimised: r.minimized(), workspace: r.workspace(),
            shown: r.mapped() && !lp.inner.comp.hidden_id(r.id()),
        })))
        .collect();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        for (uuid, fact) in &current {
            let old = state.previous.get(uuid);
            let enter = if fact.mapped && old.is_none_or(|old| !old.mapped) {
                settings.map.as_ref()
            } else if fact.shown && old.is_some_and(|old| old.minimised) {
                settings.restore.as_ref()
            } else if fact.shown && old.is_some_and(|old| !old.shown) {
                settings.workspace.as_ref()
            } else { None };
            if let Some(config) = enter
                && let Some(preset) = preset(config, false)
                && let Some(window) = windows.get(uuid) {
                super::animation::start(window, &preset, now);
                state.exits.remove(uuid);
            }
        }
        let previous = state.previous.clone();
        for (uuid, old) in &previous {
            let new = current.get(uuid);
            let exit = if old.mapped && new.is_none_or(|new| !new.mapped) {
                settings.unmap.as_ref()
            } else if old.shown && new.is_some_and(|new| new.minimised) {
                settings.minimise.as_ref()
            } else if old.shown && new.is_some_and(|new| !new.shown) {
                settings.workspace.as_ref()
            } else { None };
            if let Some(config) = exit && let Some(preset) = preset(config, true) {
                state.exits.insert(*uuid, super::animation::VisualRun { started: now, preset });
            }
        }
        state.previous = current;
        state.settings = settings;
        state.exits.retain(|_, run| run.active(now));
        let active = !state.exits.is_empty() || windows.values().any(|window| super::animation::active(window, now));
        if state.active && !active {
            lp.state.schedule_redraw(dispatcher::state::RedrawReason::WindowState);
        }
        state.active = active;
        active
    })
}

pub fn exits(now: Instant) -> HashMap<Uuid, motion::Values> {
    STATE.with(|state| state.borrow().exits.iter().filter(|(_, run)| run.active(now))
        .map(|(uuid, run)| (*uuid, run.sample(now))).collect())
}

pub fn retain_snapshots() -> bool {
    STATE.with(|state| {
        let state = state.borrow();
        state.settings.unmap.is_some() || state.settings.minimise.is_some() || state.settings.workspace.is_some()
    })
}

pub fn retained_windows() -> std::collections::HashSet<Uuid> {
    STATE.with(|state| {
        let state = state.borrow();
        state.previous.iter().filter(|(_, facts)| facts.shown).map(|(uuid, _)| *uuid)
            .chain(state.exits.keys().copied()).collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disabled_defaults_and_shared_reverse_exit() {
        let settings: WindowMotion = serde_json::from_str("{}").unwrap();
        assert_eq!(settings, WindowMotion::default());
        let config = WindowTransition { preset: "fade".into(), duration_ms: 300 };
        let exit = preset(&config, true).unwrap();
        assert_eq!(exit.sample(0.0).get(Property::Opacity), 1.0);
        assert_eq!(exit.sample(0.3).get(Property::Opacity), 0.0);
        assert!(preset(&WindowTransition { preset: "draw".into(), duration_ms: 300 }, false).is_none());
    }
}
