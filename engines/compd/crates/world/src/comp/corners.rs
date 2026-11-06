// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Hot corners on the compd host.
//!
//! [`Corners`] is the engine-free half: policy's detector and press
//! ownership on one output at a time, in output-local logical coordinates,
//! with the topics it earned queued for the Bus. [`sample`] and [`button`]
//! are the two call-outs seat makes from the pointer path, BEFORE
//! anything is forwarded to a client or to the canvas edge pan. The dwell timer is
//! the host's: it arms a one-shot at [`Corners::next_deadline`] and calls
//! [`Corners::tick`], so nothing polls.
//!
//! Outputs are named by their smithay name here; the Bus host turns the name
//! into the `o_<slug>` key when it publishes.
//!
//! Corner supremacy over pointer constraints: when a corner engages, the
//! human pointer's active
//! constraint is deactivated so a confined game cannot trap the cursor out of
//! the corner; a constraint broken (or, in the engine's `new_constraint`,
//! declined) while a corner is engaged is re-activated by the first real motion
//! sample outside every corner, never by a detector reset. A LOCKED pointer
//! never moves and so never reaches a corner (a known limit).
//!
//! Not here yet: the hover/flash affordance, with the furniture drawing work.

use std::time::Instant;

pub use policy::corner::{CORNER_CLICK_SLOP_PX, CORNER_DEFAULT_DEADZONE_PX};
use policy::corner::{
    Corner, CornerConfig, CornerDetector, CornerEvent, CornerEvents, CornerPresses, CornerTopic,
    PressFacts,
};
use smithay::utils::{Physical, Point};
use smithay::wayland::pointer_constraints::with_pointer_constraint;

use crate::state::Loop;
use smithay::reexports::wayland_server::backend::ClientId;
use std::collections::BTreeMap;
type PanelClaim = (
    String,
    String,
    Option<ClientId>,
    Option<u64>,
    String,
    String,
);
static PANEL_INPUT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
pub fn enable_panel_input() {
    PANEL_INPUT.store(true, std::sync::atomic::Ordering::Release);
}
pub fn panel_input_enabled() -> bool {
    PANEL_INPUT.load(std::sync::atomic::Ordering::Acquire)
}

/// The corner state of the compd host.
#[derive(Debug)]
pub struct Corners {
    config: CornerConfig,
    detector: CornerDetector,
    presses: CornerPresses,
    /// The output the detector samples (smithay name) and its logical size.
    region: Option<(String, (f64, f64))>,
    /// The last sample, output-local logical: the dwell timer and a
    /// release's re-sample sample here again.
    last: Option<(f64, f64)>,
    clock: Instant,
    topics: Vec<CornerTopic>,
    /// Bumped when the configuration changes (a set, or the first entry
    /// ending discovery): the edge pass and `compd.truth` must see it.
    changes: u64,
    /// An injected move with `corners: false` is running: sampling resets
    /// instead (an engaged or dwelling corner is still left).
    suppressed: bool,
    panel_presses: BTreeMap<u32, (PanelClaim, Corner)>,
}

impl Default for Corners {
    fn default() -> Self {
        let config = CornerConfig::default();
        Self {
            config,
            detector: CornerDetector::new(config, (0.0, 0.0)),
            presses: CornerPresses::default(),
            region: None,
            last: None,
            clock: Instant::now(),
            topics: Vec::new(),
            changes: 0,
            suppressed: false,
            panel_presses: BTreeMap::new(),
        }
    }
}

impl Corners {
    pub fn config(&self) -> CornerConfig {
        self.config
    }

    /// `comp.input.pointer.move {corners: false}` (comp
    /// `InjectionState::suppress_corners`), for the duration of one move.
    pub fn set_suppressed(&mut self, suppressed: bool) {
        self.suppressed = suppressed;
    }

    /// Configuration changes so far (see the field).
    pub fn changes(&self) -> u64 {
        self.changes
    }

    /// The output the detector is on (smithay name).
    pub fn output(&self) -> Option<&str> {
        self.region.as_ref().map(|(name, _)| name.as_str())
    }

    pub fn engaged(&self) -> Option<Corner> {
        self.detector.engaged_corner()
    }

    /// The hotspot the pointer is in, dwelled or not.
    pub fn contact(&self) -> Option<Corner> {
        self.detector.contact_corner()
    }

    /// Buttons the corners own (pressed on an engaged corner, not yet released).
    pub fn owned_buttons(&self) -> Vec<u32> {
        self.presses.owned().collect()
    }

    fn now_ms(&self) -> u64 {
        u64::try_from(self.clock.elapsed().as_millis()).unwrap_or(u64::MAX)
    }

    /// When the pending dwell completes; `None` while nothing is pending.
    pub fn next_deadline(&self) -> Option<Instant> {
        self.detector
            .next_deadline_ms()
            .map(|ms| self.clock + std::time::Duration::from_millis(ms))
    }

    /// The topics earned since the last call, in order.
    pub fn take_topics(&mut self) -> Vec<CornerTopic> {
        std::mem::take(&mut self.topics)
    }

    /// One pointer sample on `output` (logical `size`), at `local` (output
    /// logical) after an attempted motion of `attempted` (the outward push
    /// a pinned cursor still makes). Returns whether the pointer is in a
    /// hotspot, dwelled or not: corner bands win over the canvas edge pan.
    pub fn sample(
        &mut self,
        output: &str,
        size: (f64, f64),
        local: (f64, f64),
        attempted: (f64, f64),
        session_lock: bool,
    ) -> bool {
        if session_lock {
            self.reset();
            return false;
        }
        if self
            .region
            .as_ref()
            .is_none_or(|(name, region_size)| name != output || *region_size != size)
        {
            self.reset();
            self.region = Some((output.to_string(), size));
            let events = self.detector.reconfigure(self.config, size);
            self.emit(events);
        }
        let before = self.detector.contact_corner();
        let events = self.detector.sample(self.now_ms(), local, attempted);
        if PANEL_INPUT.load(std::sync::atomic::Ordering::Acquire) {
            let after = self.detector.contact_corner();
            if after != before {
                if let Some(corner) = after {
                    self.topics.push(CornerTopic::Input {
                        output: output.into(),
                        corner,
                        phase: "engaged",
                        button: None,
                        consumed: false,
                    });
                }
            }
            for event in events.iter().flatten() {
                if let CornerEvent::Entered { corner, .. } = event {
                    self.topics.push(CornerTopic::Input {
                        output: output.into(),
                        corner: *corner,
                        phase: "dwelled",
                        button: None,
                        consumed: false,
                    });
                }
            }
        }
        self.emit(events);
        self.presses.pointer_moved(local, CORNER_CLICK_SLOP_PX);
        self.last = Some(local);
        self.detector.contact_corner().is_some()
    }

    /// The dwell timer fired: sample the resting pointer again.
    pub fn tick(&mut self) {
        let (Some((output, size)), Some(local)) = (self.region.clone(), self.last) else {
            return;
        };
        self.sample(&output, size, local, (0.0, 0.0), false);
    }

    /// The pointer left every output (or corners were suppressed): leave
    /// any engaged corner and cancel owed clicks; ownership stays.
    pub fn reset(&mut self) {
        self.presses.cancel_all();
        let events = self.detector.reset();
        self.emit(events);
        self.region = None;
        self.last = None;
    }

    /// A button went down. Returns whether the corners consume it (the press,
    /// and later its release, never reach a client).
    pub fn press(&mut self, button: u32, modifiers: Vec<&'static str>, session_lock: bool) -> bool {
        self.press_with_contact(button, modifiers, session_lock, false)
    }
    fn press_with_contact(
        &mut self,
        button: u32,
        modifiers: Vec<&'static str>,
        session_lock: bool,
        panel: bool,
    ) -> bool {
        let engaged = self.region.as_ref().and_then(|(output, _)| {
            Some((
                output.clone(),
                if panel {
                    self.detector.contact_corner()?
                } else {
                    self.detector.engaged_corner()?
                },
                self.detector.engaged_dwell_ms().unwrap_or(0),
            ))
        });
        self.presses.consume_press(
            button,
            PressFacts {
                session_lock,
                engaged,
                position: self.last.unwrap_or_default(),
                modifiers,
            },
        )
    }

    /// A button came up. Returns whether the corners consume it; a click still
    /// owed is re-checked against the corner at the pointer first.
    pub fn release(&mut self, button: u32) -> bool {
        if !self.presses.owns(button) {
            return false;
        }
        if self.presses.owes_click(button) {
            self.tick();
        }
        if let Some(topics) = self.presses.consume_release(button) {
            self.topics.extend(topics);
        }
        true
    }

    /// `props.set input.corners.*`: a change
    /// to a detection leaf ends any engagement.
    pub fn set_config(&mut self, config: CornerConfig) {
        if config == self.config {
            return;
        }
        self.config = config;
        self.changes = self.changes.wrapping_add(1);
        let size = self.region.as_ref().map_or((0.0, 0.0), |(_, size)| *size);
        let events = self.detector.reconfigure(config, size);
        self.emit(events);
    }

    fn emit(&mut self, events: CornerEvents) {
        let Some((output, _)) = self.region.clone() else {
            return;
        };
        for event in events.into_iter().flatten() {
            match event {
                CornerEvent::Left { .. } => self.presses.cancel_all(),
                // The first reveal of any kind ends the discovery flash.
                CornerEvent::Entered { .. } if self.config.discovery => {
                    self.config.discovery = false;
                    self.changes = self.changes.wrapping_add(1);
                }
                CornerEvent::Entered { .. } => {}
            }
        }
        self.topics
            .extend(CornerTopic::from_events(&output, events));
    }
}

/// The seat motion call-out: `position` is the cursor in the physical
/// screen space of the output it is on (`screen`, that output's physical
/// size), `attempted` the physical motion it tried to make. Returns whether
/// the pointer is in a corner hotspot.
pub fn sample(
    lp: &mut Loop,
    position: Point<f64, Physical>,
    screen: (f64, f64),
    attempted: (f64, f64),
) -> bool {
    if lp.inner.comp.corners.suppressed {
        lp.inner.comp.corners.reset();
        return false;
    }
    let Some((name, scale)) = cursor_output(lp) else {
        lp.inner.comp.corners.reset();
        return false;
    };
    let scale = if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    };
    let locked = super::session_lock::active(lp);
    let was_engaged = lp.inner.comp.corners.engaged().is_some();
    let hot = lp.inner.comp.corners.sample(
        &name,
        (screen.0 / scale, screen.1 / scale),
        (position.x / scale, position.y / scale),
        (attempted.0 / scale, attempted.1 / scale),
        locked,
    );
    let engaged = lp.inner.comp.corners.engaged().is_some();
    lp.state.seat.corner_engaged = engaged;
    if engaged && !was_engaged {
        break_constraint(lp);
        lp.state.seat.constraint_deferred = true;
    } else if !engaged && lp.state.seat.constraint_deferred {
        lp.state.seat.constraint_deferred = false;
        activate_pending_constraint(lp);
    }
    hot
}

/// The human pointer's focused
/// surface's ACTIVE constraint is deactivated (the corner wins).
fn break_constraint(lp: &mut Loop) {
    let Some(pointer) = lp.state.seat.seat.get_pointer() else {
        return;
    };
    let Some(surface) = pointer.current_focus() else {
        return;
    };
    // The engine's own deactivate (vendored smithay's takes the state, the
    // surface and the pointer), which also routes the unlock hint as every
    // other deactivation does. A no-op without an active constraint.
    lp.state.deactivate_constraint_for(&surface, &pointer);
}

/// The focused surface's
/// pending constraint is activated, under the same conditions the engine's
/// `new_constraint` activates one (the surface has the keyboard, the hand
/// tool is not suspending constraints).
fn activate_pending_constraint(lp: &mut Loop) {
    if lp.state.seat.constraints_suspended {
        return;
    }
    let Some(pointer) = lp.state.seat.seat.get_pointer() else {
        return;
    };
    let Some(surface) = pointer.current_focus() else {
        return;
    };
    if !lp.state.seat.is_keyboard_focused(&surface) {
        return;
    }
    with_pointer_constraint(&surface, &pointer, |constraint| {
        if let Some(constraint) = constraint
            && !constraint.is_active()
        {
            constraint.activate();
        }
    });
}

fn panel_claim(lp: &Loop, output: &str, corner: Corner) -> Option<PanelClaim> {
    let edge = corner.summoned_edge();
    let holder = lp
        .inner
        .comp
        .panels
        .holders
        .get(&(output.into(), edge.into()))?;
    if holder.stalled || holder.reporter.is_none() {
        return None;
    }
    Some((
        output.into(),
        edge.into(),
        holder.owner.clone(),
        holder.generation,
        holder.reporter.clone()?,
        holder.surface.clone(),
    ))
}

/// Early ownership is compositor authority, bound to a live panel incarnation.
/// Every owned release is consumed even after departure, disconnect or lock;
/// losing the claim cancels the action, never the ownership of the release.
pub fn panel_button(lp: &mut Loop, button: u32, pressed: bool) -> bool {
    if !PANEL_INPUT.load(std::sync::atomic::Ordering::Acquire) {
        return false;
    }
    use crate::state::CoordinateTrait as _;
    let point = lp.inner.pointer_mut().motion;
    let screen = lp.size_ctx_all().screen_size_physical;
    sample(lp, Point::from((point.x, point.y)), screen, (0.0, 0.0));
    if !pressed {
        let Some((claim, corner)) = lp.inner.comp.corners.panel_presses.remove(&button) else {
            return false;
        };
        if !panel_claim(lp, &claim.0, corner).is_some_and(|now| {
            now.0 == claim.0
                && now.1 == claim.1
                && now.3 == claim.3
                && now.4 == claim.4
                && now.5 == claim.5
                && (claim.2.is_none() || now.2 == claim.2)
        }) || super::session_lock::active(lp)
        {
            lp.inner.comp.corners.presses.cancel(button);
        }
        let corners = &mut lp.inner.comp.corners;
        let clicked = corners.presses.owes_click(button);
        corners.release(button);
        if clicked {
            corners.topics.push(CornerTopic::Input {
                output: claim.0.clone(),
                corner,
                phase: "clicked",
                button: Some(button),
                consumed: true,
            });
        }
        corners.topics.push(CornerTopic::Input {
            output: claim.0,
            corner,
            phase: "released",
            button: Some(button),
            consumed: true,
        });
        return true;
    }
    if lp.inner.comp.corners.panel_presses.contains_key(&button) {
        return true;
    }
    let Some(corner) = lp.inner.comp.corners.contact() else {
        return false;
    };
    let Some(output) = lp.inner.comp.corners.output().map(str::to_owned) else {
        return false;
    };
    // The global hotspot is viud's even while its renderer is starting or
    // disconnected. A missing live claim cancels the resulting action on
    // release; it must never turn the early press into an application click.
    let claim = panel_claim(lp, &output, corner).unwrap_or_else(|| {
        (
            output.clone(),
            corner.summoned_edge().into(),
            None,
            None,
            String::new(),
            String::new(),
        )
    });
    let locked = super::session_lock::active(lp);
    let modifiers = lp
        .state
        .seat
        .seat
        .get_keyboard()
        .map(|keyboard| {
            let state = keyboard.modifier_state();
            [
                (state.shift, "shift"),
                (state.ctrl, "ctrl"),
                (state.alt, "alt"),
                (state.logo, "super"),
            ]
            .into_iter()
            .filter_map(|(on, name)| on.then_some(name))
            .collect()
        })
        .unwrap_or_default();
    let corners = &mut lp.inner.comp.corners;
    if !corners.press_with_contact(button, modifiers, locked, true) {
        return false;
    }
    corners.panel_presses.insert(button, (claim, corner));
    corners.topics.push(CornerTopic::Input {
        output,
        corner,
        phase: "consumed",
        button: Some(button),
        consumed: true,
    });
    true
}

/// The seat button call-out. Returns whether the corners consumed the button.
pub fn button(lp: &mut Loop, button: u32, pressed: bool) -> bool {
    if !pressed {
        return lp.inner.comp.corners.release(button);
    }
    // Read at the press; the release may hold different modifiers.
    let modifiers = lp
        .state
        .seat
        .seat
        .get_keyboard()
        .map(|keyboard| {
            let state = keyboard.modifier_state();
            [
                (state.shift, "shift"),
                (state.ctrl, "ctrl"),
                (state.alt, "alt"),
                (state.logo, "super"),
            ]
            .into_iter()
            .filter_map(|(active, name)| active.then_some(name))
            .collect()
        })
        .unwrap_or_default();
    let locked = super::session_lock::active(lp);
    lp.inner.comp.corners.press(button, modifiers, locked)
}

/// The output the cursor is on: its smithay name and scale.
fn cursor_output(lp: &Loop) -> Option<(String, f64)> {
    let space = &lp.inner.space_state().state;
    let key = lp.inner.cursor_output.clone();
    let output = space
        .outputs()
        .find(|output| key.as_ref() == Some(&crate::state::output_key(output)))
        .or_else(|| space.outputs().next())?;
    Some((output.name(), output.current_scale().fractional_scale()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn panel_authority_owns_a_fast_click_before_dwell_and_its_release() {
        let mut corners = Corners::default();
        corners.sample("test", (1000.0, 800.0), (2.0, 2.0), (0.0, 0.0), false);
        assert!(corners.contact().is_some());
        assert!(corners.engaged().is_none());
        assert!(
            !corners.press(0x110, Vec::new(), false),
            "legacy dwell-only path remains unchanged"
        );
        assert!(corners.press_with_contact(0x110, Vec::new(), false, true));
        assert!(corners.release(0x110));
        assert!(
            corners
                .take_topics()
                .iter()
                .any(|topic| matches!(topic, CornerTopic::Clicked { dwell_ms: 0, .. }))
        );
    }
    #[test]
    fn lost_panel_claim_cancels_action_but_keeps_release_owned() {
        let mut corners = Corners::default();
        corners.sample("test", (1000.0, 800.0), (2.0, 2.0), (0.0, 0.0), false);
        assert!(corners.press_with_contact(0x110, Vec::new(), false, true));
        corners.presses.cancel(0x110);
        assert!(corners.release(0x110));
        assert!(
            !corners
                .take_topics()
                .iter()
                .any(|topic| matches!(topic, CornerTopic::Clicked { .. }))
        );
        assert!(!corners.release(0x110));
    }

    const SIZE: (f64, f64) = (1_000.0, 800.0);

    fn entered(topics: &[CornerTopic]) -> Vec<(String, Corner)> {
        topics
            .iter()
            .filter_map(|topic| match topic {
                CornerTopic::Entered { output, corner, .. } => Some((output.clone(), *corner)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn outward_push_engages_and_a_click_is_owned() {
        let mut corners = Corners::default();
        assert!(
            corners.sample("DP-1", SIZE, (2.0, 2.0), (0.0, 0.0), false),
            "in the hotspot"
        );
        assert!(corners.engaged().is_none(), "not dwelled yet");
        assert!(corners.next_deadline().is_some(), "a dwell is pending");
        // Same position, a later sample with an outward push: early engage.
        std::thread::sleep(std::time::Duration::from_millis(2));
        corners.sample("DP-1", SIZE, (2.0, 2.0), (0.0, 0.0), false);
        std::thread::sleep(std::time::Duration::from_millis(2));
        corners.sample("DP-1", SIZE, (2.0, 2.0), (-3.0, 0.0), false);
        assert_eq!(corners.engaged(), Some(Corner::TopLeft));
        assert_eq!(
            entered(&corners.take_topics()),
            [("DP-1".to_string(), Corner::TopLeft)]
        );
        assert!(
            corners.press(0x110, Vec::new(), false),
            "an engaged press is consumed"
        );
        assert!(corners.release(0x110));
        let topics = corners.take_topics();
        assert!(matches!(
            topics.as_slice(),
            [CornerTopic::Clicked { .. }, CornerTopic::ClickedV2 { .. }]
        ));
        assert!(!corners.release(0x110), "not owned any more");
    }

    #[test]
    fn leaving_the_hotspot_leaves_and_cancels_the_owed_click() {
        let mut corners = Corners::default();
        corners.sample("DP-1", SIZE, (998.0, 798.0), (0.0, 0.0), false);
        std::thread::sleep(std::time::Duration::from_millis(2));
        corners.sample("DP-1", SIZE, (998.0, 798.0), (0.0, 0.0), false);
        std::thread::sleep(std::time::Duration::from_millis(2));
        corners.sample("DP-1", SIZE, (998.0, 798.0), (2.0, 2.0), false);
        assert_eq!(corners.engaged(), Some(Corner::BottomRight));
        assert!(corners.press(0x110, Vec::new(), false));
        assert!(!corners.sample("DP-1", SIZE, (500.0, 400.0), (0.0, 0.0), false));
        let topics = corners.take_topics();
        assert!(topics.iter().any(|topic| matches!(
            topic,
            CornerTopic::Left {
                corner: Corner::BottomRight,
                ..
            }
        )));
        assert!(corners.release(0x110), "still owned");
        assert!(
            corners.take_topics().is_empty(),
            "but the click was cancelled"
        );
    }

    #[test]
    fn config_changes_count_and_the_first_entry_ends_discovery() {
        let mut corners = Corners::default();
        let config = CornerConfig {
            discovery: true,
            ..CornerConfig::default()
        };
        corners.set_config(config);
        corners.set_config(config);
        assert_eq!(corners.changes(), 1, "an unchanged set is not a change");
        corners.sample("DP-1", SIZE, (2.0, 2.0), (0.0, 0.0), false);
        std::thread::sleep(std::time::Duration::from_millis(2));
        corners.sample("DP-1", SIZE, (2.0, 2.0), (0.0, 0.0), false);
        std::thread::sleep(std::time::Duration::from_millis(2));
        corners.sample("DP-1", SIZE, (2.0, 2.0), (-1.0, 0.0), false);
        assert_eq!(corners.engaged(), Some(Corner::TopLeft));
        assert!(!corners.config().discovery);
        assert_eq!(corners.changes(), 2);
    }

    #[test]
    fn an_unengaged_press_is_not_consumed_and_another_output_resets() {
        let mut corners = Corners::default();
        assert!(!corners.press(0x110, Vec::new(), false));
        assert!(!corners.release(0x110));
        corners.sample("DP-1", SIZE, (2.0, 2.0), (0.0, 0.0), false);
        assert_eq!(corners.output(), Some("DP-1"));
        corners.sample("HDMI-A-1", SIZE, (500.0, 400.0), (0.0, 0.0), false);
        assert_eq!(corners.output(), Some("HDMI-A-1"));
        assert!(corners.contact().is_none());
    }
}
