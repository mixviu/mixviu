// SPDX-License-Identifier: MIT OR Apache-2.0
//! Optional lifecycle policy, using the same tracks as explicit window runs.
use crate::state::Loop;
use crate::window::interface::record::window::LoopWindow;
use dispatcher::wire::trait_::wire_trait::WireTrait;
use model::environment::preference::base::{WindowMotion, WindowTransition};
use motion::{Direction, Fill, Preset, Property};
use smithay::desktop::Window;
use std::cell::RefCell;
use std::collections::HashMap;
use std::time::Instant;
use uuid::Uuid;

#[derive(Clone, Copy, PartialEq)]
struct Facts {
    mapped: bool,
    minimised: bool,
    workspace: Option<u32>,
    shown: bool,
    visible: bool,
}

#[derive(Default)]
struct State {
    previous: HashMap<Uuid, Facts>,
    pub exits: HashMap<Uuid, super::animation::VisualRun>,
    settings: WindowMotion,
    active: bool,
    incoming: HashMap<Uuid, Instant>,
    workspace: Option<u32>,
}

thread_local! { static STATE: RefCell<State> = RefCell::new(State::default()); }

pub fn preset(config: &WindowTransition, reverse: bool) -> Option<Preset> {
    if config.duration_ms > 10_000 {
        return None;
    }
    let mut preset = motion::presets()
        .into_iter()
        .find(|p| p.name == config.preset)?;
    preset.tracks.retain(|track| {
        matches!(
            track.property,
            Property::Opacity | Property::Scale | Property::Tx | Property::Ty
        )
    });
    if preset.tracks.is_empty() {
        return None;
    }
    preset.timing.duration = config.duration_ms as f64 / 1000.0;
    preset.timing.fill = Fill::Both;
    preset.timing.delay = 0.0;
    preset.timing.iterations = motion::Iterations::Finite(1.0);
    preset.timing.direction = if reverse {
        Direction::Reverse
    } else {
        Direction::Normal
    };
    Some(preset)
}

fn relative(mut preset: Preset, base: comp_model::request::VisualAttributes) -> Preset {
    for track in &mut preset.tracks {
        for frame in &mut track.keyframes {
            frame.value = match track.property {
                Property::Opacity => frame.value * base.opacity,
                Property::Scale => frame.value * base.scale.factor,
                Property::Tx => frame.value + base.translate.x,
                Property::Ty => frame.value + base.translate.y,
                _ => frame.value,
            };
        }
    }
    preset
}

/// Observe after protocol/policy changes. A close removes input immediately;
/// its outgoing image lives separately, with no registry or Window reference.
pub fn service(lp: &mut Loop) -> bool {
    let now = lp.inner.comp.clock.now();
    let settings = lp.inner.preference.window_motion.clone();
    if settings.map.is_none()
        && settings.unmap.is_none()
        && settings.minimise.is_none()
        && settings.restore.is_none()
        && settings.workspace.is_none()
    {
        let stopped = STATE.with(|state| {
            let mut state = state.borrow_mut();
            if state.incoming.is_empty()
                && state.exits.is_empty()
                && !state.active
                && state.settings == WindowMotion::default()
            {
                return None;
            }
            let incoming = std::mem::take(&mut state.incoming);
            *state = State::default();
            Some(incoming)
        });
        if let Some(incoming) = stopped {
            for space in lp.inner.all_world_spaces() {
                for window in space.state.elements() {
                    if window
                        .uuid()
                        .and_then(|uuid| incoming.get(&uuid).copied())
                        .is_some_and(|started| super::animation::started(window) == Some(started))
                    {
                        super::animation::finish(window);
                    }
                }
            }
            lp.state
                .schedule_redraw(dispatcher::state::RedrawReason::WindowState);
        }
        return false;
    }
    let windows: HashMap<Uuid, Window> = lp
        .inner
        .all_world_spaces()
        .into_iter()
        .flat_map(|space| space.state.elements())
        .filter_map(|window| Some((window.uuid()?, window.clone())))
        .collect();
    let current: HashMap<Uuid, Facts> = lp
        .inner
        .comp
        .registry
        .surface_rows()
        .filter(|r| r.role().managed_toplevel())
        .filter_map(|r| {
            Some((
                r.uuid()?,
                Facts {
                    mapped: r.mapped(),
                    minimised: r.minimized(),
                    workspace: r.workspace(),
                    shown: r.mapped() && !lp.inner.comp.hidden_id(r.id()),
                    visible: lp.inner.comp.visual(r.id()).visible,
                },
            ))
        })
        .collect();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let workspace = lp.inner.comp.current_workspace();
        let workspace_changed = state.workspace != Some(workspace);
        if workspace_changed {
            state.exits.clear();
            crate::window::draw::ghost::clear();
            state.workspace = Some(workspace);
        }
        state.incoming.retain(|uuid, started| {
            let Some(window) = windows.get(uuid) else { return false; };
            if super::animation::started(window) != Some(*started) { return false; }
            if !super::animation::active(window, now) {
                super::animation::finish(window);
                return false;
            }
            true
        });
        if settings != state.settings {
            for (event, config) in [
                ("map", &settings.map),
                ("unmap", &settings.unmap),
                ("minimise", &settings.minimise),
                ("restore", &settings.restore),
                ("workspace", &settings.workspace),
            ] {
                if let Some(config) = config
                    && preset(config, false).is_none()
                {
                    warn!(
                        "unsupported window lifecycle transition {event} preset {} ignored",
                        config.preset
                    );
                }
            }
        }
        for (uuid, fact) in &current {
            let old = state.previous.get(uuid);
            let appearing = fact.shown && old.is_none_or(|old| !old.shown);
            let enter = if fact.mapped && old.is_none_or(|old| !old.mapped) {
                settings.map.as_ref()
            } else if fact.shown && old.is_some_and(|old| old.minimised) {
                settings.restore.as_ref()
            } else if fact.shown && old.is_some_and(|old| !old.shown && (workspace_changed || old.workspace != fact.workspace)) {
                settings.workspace.as_ref()
            } else {
                None
            };
            if appearing {
                state.exits.remove(uuid);
            }
            if let Some(config) = enter
                && let Some(preset) = preset(config, false)
                && let Some(window) = windows.get(uuid)
            {
                let base = crate::window::draw::frame::scene::visual_of(window);
                super::animation::start(window, &relative(preset, base), now);
                if super::animation::active(window, now) {
                    state.incoming.insert(*uuid, now);
                } else { super::animation::finish(window); }
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
            } else {
                None
            };
            if new.is_some_and(|facts| !facts.visible) {
                state.exits.remove(uuid);
                continue;
            }
            if old.workspace != Some(workspace) { continue; }
            if let Some(config) = exit
                && let Some(preset) = preset(config, true)
            {
                crate::window::draw::ghost::capture_exit(*uuid);
                state.exits.insert(
                    *uuid,
                    super::animation::VisualRun {
                        started: now,
                        preset,
                    },
                );
            }
        }
        state.previous = current;
        state.settings = settings;
        state.exits.retain(|_, run| run.active(now));
        let active = !state.exits.is_empty()
            || windows
                .values()
                .any(|window| super::animation::active(window, now));
        if state.active && !active {
            lp.state
                .schedule_redraw(dispatcher::state::RedrawReason::WindowState);
        }
        state.active = active;
        active
    })
}

pub fn exits(now: Instant) -> HashMap<Uuid, motion::Values> {
    STATE.with(|state| {
        state
            .borrow()
            .exits
            .iter()
            .filter(|(_, run)| run.active(now))
            .map(|(uuid, run)| (*uuid, run.sample(now)))
            .collect()
    })
}

pub fn retain_snapshots() -> bool {
    STATE.with(|state| {
        let state = state.borrow();
        state.settings.unmap.is_some()
            || state.settings.minimise.is_some()
    })
}

pub fn retained_windows() -> std::collections::HashSet<Uuid> {
    STATE.with(|state| {
        let state = state.borrow();
        state
            .previous
            .iter()
            .filter(|(_, facts)| facts.shown)
            .map(|(uuid, _)| *uuid)
            .chain(state.exits.keys().copied())
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn disabled_defaults_and_shared_reverse_exit() {
        let settings: WindowMotion = serde_json::from_str("{}").unwrap();
        assert_eq!(settings, WindowMotion::default());
        let config = WindowTransition {
            preset: "fade".into(),
            duration_ms: 300,
        };
        let exit = preset(&config, true).unwrap();
        assert_eq!(exit.sample(0.0).get(Property::Opacity), 1.0);
        assert_eq!(exit.sample(0.3).get(Property::Opacity), 0.0);
        assert!(
            preset(
                &WindowTransition {
                    preset: "draw".into(),
                    duration_ms: 300
                },
                false
            )
            .is_none()
        );
    }

    #[test]
    fn entrances_restore_requested_visual_attributes() {
        let config = WindowTransition {
            preset: "pop".into(),
            duration_ms: 300,
        };
        let mut base = comp_model::request::VisualAttributes::default();
        base.opacity = 0.5;
        base.scale.factor = 0.75;
        base.translate.x = 12.0;
        let incoming = relative(preset(&config, false).unwrap(), base);
        let final_value = super::super::animation::apply(&incoming.sample(0.3), base);
        assert_eq!(final_value.opacity, base.opacity);
        assert_eq!(final_value.scale, base.scale);
        assert_eq!(final_value.translate, base.translate);
    }
}
