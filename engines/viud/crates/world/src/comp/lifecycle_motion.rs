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

#[derive(Clone, PartialEq)]
struct Facts {
    mapped: bool,
    minimised: bool,
    workspace: Option<u32>,
    shown: bool,
    visible: bool,
    output: String,
}

impl Facts {
    fn on_current(&self, workspaces: &HashMap<String, u32>) -> bool {
        self.workspace == workspaces.get(&self.output).copied()
    }
}

#[derive(Default)]
struct State {
    previous: HashMap<Uuid, Facts>,
    exits: HashMap<Uuid, (String, super::animation::VisualRun)>,
    settings: WindowMotion,
    active: bool,
    incoming: HashMap<Uuid, Instant>,
    workspaces: HashMap<String, u32>,
    presented: HashMap<(Uuid, String), u32>,
}

thread_local! { static STATE: RefCell<State> = RefCell::new(State::default()); }

pub fn presented(output: String, shown: &[(Uuid, u32)]) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.presented.retain(|(_, scope), _| scope != &output);
        for &(uuid, workspace) in shown {
            state.presented.insert((uuid, output.clone()), workspace);
        }
    });
}

fn stable_output(mut areas: Vec<(i64, String)>) -> Option<String> {
    areas.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(&b.1)));
    areas.into_iter().next().map(|(_, name)| name)
}

/// Record explicit policy at application time, before destruction can erase
/// the registry row and before the next lifecycle observation.
pub fn visibility(uuid: Uuid, visible: bool) {
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        if let Some(old) = state.previous.get_mut(&uuid) {
            old.visible = visible;
            if !visible {
                old.shown = false;
            }
        }
        if !visible {
            state.exits.remove(&uuid);
        }
    });
}

impl State {
    fn sync_workspaces(&mut self, current: HashMap<String, u32>) {
        for (output, old) in &self.workspaces {
            if current.get(output) != Some(old) {
                self.exits.retain(|_, (scope, _)| scope != output);
                crate::window::draw::ghost::clear_output(output);
            }
        }
        self.workspaces = current;
    }
}

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
    let default_output = lp
        .inner
        .comp
        .default_output
        .as_ref()
        .map(|o| o.key.clone())
        .unwrap_or_default();
    let outputs: HashMap<Uuid, String> = lp
        .inner
        .all_world_spaces()
        .into_iter()
        .flat_map(|space| {
            space
                .state
                .elements()
                .filter_map(|window| {
                    Some((
                        window.uuid()?,
                        stable_output(
                            space
                                .state
                                .outputs_for_element(window)
                                .iter()
                                .map(|output| {
                                    let area = space
                                        .state
                                        .element_bbox(window)
                                        .and_then(|bbox| {
                                            bbox.intersection(space.state.output_geometry(output)?)
                                        })
                                        .map_or(0, |r| i64::from(r.size.w) * i64::from(r.size.h));
                                    (area, output.name())
                                })
                                .collect(),
                        )
                        .map(|name| comp_model::snapshot::output_key(&name))
                        .unwrap_or_else(|| default_output.clone()),
                    ))
                })
                .collect::<Vec<_>>()
        })
        .collect();
    let mut workspaces: HashMap<String, u32> = lp
        .inner
        .comp
        .workspaces
        .current
        .iter()
        .map(|(output, workspace)| (output.clone(), *workspace))
        .collect();
    for space in lp.inner.all_world_spaces() {
        for output in space.state.outputs() {
            let key = comp_model::snapshot::output_key(&output.name());
            workspaces
                .entry(key.clone())
                .or_insert_with(|| lp.inner.comp.workspaces.current_for(Some(&key)));
        }
    }
    for output in outputs.values().chain(std::iter::once(&default_output)) {
        workspaces
            .entry(output.clone())
            .or_insert_with(|| lp.inner.comp.workspaces.current_for(Some(output)));
    }
    let current: HashMap<Uuid, Facts> = lp
        .inner
        .comp
        .registry
        .surface_rows()
        .filter(|r| r.role().managed_toplevel())
        .filter_map(|r| {
            let uuid = r.uuid()?;
            let output = outputs.get(&uuid).unwrap_or(&default_output).clone();
            let visible = lp.inner.comp.visual(r.id()).visible;
            Some((
                uuid,
                Facts {
                    mapped: r.mapped(),
                    minimised: r.minimized(),
                    workspace: r.workspace(),
                    shown: r.mapped()
                        && visible
                        && !r.minimized()
                        && r.workspace()
                            == Some(lp.inner.comp.workspaces.current_for(Some(&output))),
                    visible,
                    output,
                },
            ))
        })
        .collect();
    STATE.with(|state| {
        let mut state = state.borrow_mut();
        let previous_workspaces = state.workspaces.clone();
        state.sync_workspaces(workspaces);
        state.incoming.retain(|uuid, started| {
            let Some(window) = windows.get(uuid) else {
                return false;
            };
            if super::animation::started(window) != Some(*started) {
                return false;
            }
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
            } else if fact.shown
                && old.is_some_and(|old| {
                    !old.shown
                        && (previous_workspaces.get(&fact.output)
                            != state.workspaces.get(&fact.output)
                            || old.workspace != fact.workspace)
                })
            {
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
                {
                    let cell = window.user_data().get_or_insert(|| {
                        std::sync::Mutex::new(comp_model::request::VisualAttributes::default())
                    });
                    cell.lock().unwrap().scale.center = true;
                }
                let base = crate::window::draw::frame::scene::visual_of(window);
                super::animation::start(window, &relative(preset, base), now);
                super::animation::replaced_by(window, "lifecycle");
                if super::animation::active(window, now) {
                    state.incoming.insert(*uuid, now);
                } else {
                    super::animation::finish(window);
                }
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
            if !old.visible
                || new.is_some_and(|facts| !facts.visible)
                || (!old.shown && old.mapped && new.is_none_or(|facts| !facts.mapped))
            {
                state.exits.remove(uuid);
                continue;
            }
            if !old.shown
                || !old.on_current(&state.workspaces)
                || !state
                    .presented
                    .get(&(*uuid, old.output.clone()))
                    .is_some_and(|workspace| Some(*workspace) == old.workspace)
            {
                continue;
            }
            if let Some(config) = exit
                && let Some(preset) = preset(config, true)
            {
                crate::window::draw::ghost::capture_exit(*uuid);
                state.exits.insert(
                    *uuid,
                    (
                        old.output.clone(),
                        super::animation::VisualRun {
                            started: now,
                            preset,
                        },
                    ),
                );
            }
        }
        state.previous = current;
        state.settings = settings;
        state.exits.retain(|_, (_, run)| run.active(now));
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
            .filter(|(_, (_, run))| run.active(now))
            .map(|(uuid, (_, run))| (*uuid, run.sample(now)))
            .collect()
    })
}

pub fn retain_snapshots() -> bool {
    STATE.with(|state| {
        let state = state.borrow();
        state.settings.unmap.is_some() || state.settings.minimise.is_some()
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
    fn output_scope_uses_largest_overlap_then_connector_name() {
        for candidates in [
            vec![(50, "b".into()), (50, "a".into())],
            vec![(50, "a".into()), (50, "b".into())],
        ] {
            assert_eq!(stable_output(candidates), Some("a".into()));
        }
        assert_eq!(
            stable_output(vec![(100, "b".into()), (50, "a".into())]),
            Some("b".into())
        );
    }
    #[test]
    fn workspace_switch_keeps_the_other_outputs_exit_and_uses_its_workspace() {
        let a = Uuid::from_u128(1);
        let b = Uuid::from_u128(2);
        let started = Instant::now();
        let run = super::super::animation::VisualRun {
            started,
            preset: preset(
                &WindowTransition {
                    preset: "fade".into(),
                    duration_ms: 1000,
                },
                true,
            )
            .unwrap(),
        };
        let mut state = State::default();
        state.workspaces = HashMap::from([("a".into(), 1), ("b".into(), 2)]);
        state.exits.insert(a, ("a".into(), run.clone()));
        state.exits.insert(b, ("b".into(), run.clone()));
        let other = Facts {
            mapped: true,
            minimised: false,
            workspace: Some(2),
            shown: true,
            visible: true,
            output: "b".into(),
        };
        assert!(other.on_current(&state.workspaces));
        state.sync_workspaces(HashMap::from([("a".into(), 3), ("b".into(), 2)]));
        assert!(!state.exits.contains_key(&a));
        let (_, remaining) = state
            .exits
            .get(&b)
            .expect("unaffected output retains its exit");
        let later = started + std::time::Duration::from_millis(500);
        assert!(remaining.active(later));
        assert_eq!(remaining.started, started);
        assert_eq!(remaining.sample(later), run.sample(later));
        assert!(
            remaining.sample(later).get(Property::Opacity)
                > remaining
                    .sample(started + std::time::Duration::from_millis(750))
                    .get(Property::Opacity)
        );
        assert!(other.on_current(&state.workspaces));
    }

    #[test]
    fn explicit_hide_updates_stale_facts_before_the_next_observation() {
        let uuid = Uuid::from_u128(11);
        STATE.with(|state| {
            state.borrow_mut().previous.insert(
                uuid,
                Facts {
                    mapped: true,
                    minimised: false,
                    workspace: Some(1),
                    shown: true,
                    visible: true,
                    output: "a".into(),
                },
            )
        });
        visibility(uuid, false);
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let old = state.previous.remove(&uuid).unwrap();
            assert!(
                !old.visible && !old.shown,
                "destroy cannot turn stale shown facts into an exit"
            );
        });
    }
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
