// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Panel blank policy. Rendering, captures and client pacing never read this state.

use crate::state::Loop;
use smithay::backend::input::{
    ButtonState, InputBackend, InputEvent, KeyState, KeyboardKeyEvent, PointerButtonEvent,
    TabletToolButtonEvent, TabletToolTipEvent, TabletToolTipState, TouchEvent,
};
use smithay::reexports::calloop::{
    RegistrationToken,
    timer::{TimeoutAction, Timer},
};
use std::{
    cell::Cell,
    collections::BTreeSet,
    rc::Rc,
    time::{Duration, Instant},
};

pub type Backend = Rc<dyn Fn(bool) -> Result<(), String>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Hold {
    Key(u32),
    Button(u32),
    Touch(i32),
    TabletButton(u32),
    TabletTip,
}

#[derive(Clone, Copy, Debug)]
pub enum Edge {
    Press(Hold),
    Release(Hold),
    Motion(Hold),
    Other,
    CancelTouch,
}

impl Edge {
    pub fn from_key(code: u32, pressed: bool) -> Self {
        if pressed {
            Self::Press(Hold::Key(code))
        } else {
            Self::Release(Hold::Key(code))
        }
    }
    pub fn from_button(code: u32, pressed: bool) -> Self {
        if pressed {
            Self::Press(Hold::Button(code))
        } else {
            Self::Release(Hold::Button(code))
        }
    }
}

pub struct BlankPolicy {
    pub blanked: bool,
    pub supported: bool,
    pub backend: Option<Backend>,
    timeout: Option<u64>,
    last_human: Instant,
    expired: bool,
    swallowed: BTreeSet<Hold>,
    armed: Option<(RegistrationToken, Instant, Rc<Cell<bool>>)>,
}

impl Default for BlankPolicy {
    fn default() -> Self {
        Self {
            blanked: false,
            supported: false,
            backend: None,
            timeout: None,
            last_human: Instant::now(),
            expired: false,
            swallowed: BTreeSet::new(),
            armed: None,
        }
    }
}

impl BlankPolicy {
    pub fn timeout(&self) -> Option<u64> {
        self.timeout
    }
    pub fn blanked(&self) -> bool {
        self.blanked
    }
    pub fn supported(&self) -> bool {
        self.supported
    }

    fn deadline(&self) -> Option<Instant> {
        if self.blanked || self.expired {
            return None;
        }
        self.last_human
            .checked_add(Duration::from_secs(self.timeout?))
    }

    fn configure(&mut self, timeout: Option<u64>) {
        self.timeout = timeout;
        self.expired = false;
    }

    fn activity(&mut self, now: Instant) {
        self.last_human = now;
        self.expired = false;
    }

    /// Keep ownership of a waking press through its matching release, even
    /// after a Bus unblank or another blank. Touch motion follows its down.
    /// Policy transition only; backend wake and timer rearming belong to the caller.
    pub fn consume_edge(&mut self, edge: Edge) -> bool {
        match edge {
            Edge::Press(hold) => {
                if self.blanked {
                    self.swallowed.insert(hold);
                }
                self.blanked || self.swallowed.contains(&hold)
            }
            Edge::Release(hold) => self.swallowed.remove(&hold) || self.blanked,
            Edge::Motion(hold) => self.blanked || self.swallowed.contains(&hold),
            Edge::CancelTouch => {
                let had = self.swallowed.iter().any(|h| matches!(h, Hold::Touch(_)));
                self.swallowed.retain(|h| !matches!(h, Hold::Touch(_)));
                had || self.blanked
            }
            Edge::Other => self.blanked,
        }
    }
}

pub fn set_supported(lp: &mut Loop, supported: bool) {
    if lp.inner.comp.blank.supported != supported {
        lp.inner.comp.blank.supported = supported;
        lp.inner
            .comp
            .settings_changed("display.blank_supported", "backend");
    }
}

/// Synchronous native commit, then logical publication. Backends without a
/// CRTC have no callback: they retain policy state and leave pixels alone.
pub fn set(lp: &mut Loop, on: bool, cause: &'static str) -> Result<(), String> {
    if let Some(backend) = lp.inner.comp.blank.backend.clone() {
        if let Err(error) = backend(on) {
            set_supported(lp, false);
            return Err(error);
        }
        if on {
            set_supported(lp, true);
        }
    }
    if lp.inner.comp.blank.blanked != on {
        lp.inner.comp.blank.blanked = on;
        lp.inner.comp.settings_changed("display.blanked", cause);
    }
    service(lp);
    Ok(())
}

pub fn configure(lp: &mut Loop, timeout: Option<u64>) {
    lp.inner.comp.blank.configure(timeout);
    lp.inner.comp.settings_changed("power", "props.set");
    service(lp);
}

/// Called only by the human funnels. Agent activity cannot move this clock.
pub fn activity(lp: &mut Loop) {
    lp.inner.comp.blank.activity(lp.inner.comp.clock.now());
    service(lp);
}

/// Before regions, bindings, focus changes and client delivery.
pub fn consume(lp: &mut Loop, edge: Edge) -> bool {
    let wake = lp.inner.comp.blank.blanked;
    let consumed = lp.inner.comp.blank.consume_edge(edge);
    if wake && let Err(error) = set(lp, false, "input") {
        warn!("panel unblank: {error}");
    }
    consumed
}

pub fn device_input<I: InputBackend>(lp: &mut Loop, event: &InputEvent<I>) -> bool {
    let edge = match event {
        InputEvent::Keyboard { event } => {
            let hold = Hold::Key(event.key_code().raw());
            if event.state() == KeyState::Pressed {
                Edge::Press(hold)
            } else {
                Edge::Release(hold)
            }
        }
        InputEvent::PointerButton { event } => {
            let hold = Hold::Button(event.button_code());
            if event.state() == ButtonState::Pressed {
                Edge::Press(hold)
            } else {
                Edge::Release(hold)
            }
        }
        InputEvent::TouchDown { event } => Edge::Press(Hold::Touch(event.slot().into())),
        InputEvent::TouchUp { event } => Edge::Release(Hold::Touch(event.slot().into())),
        InputEvent::TouchMotion { event } => Edge::Motion(Hold::Touch(event.slot().into())),
        InputEvent::TouchCancel { .. } => Edge::CancelTouch,
        InputEvent::TabletToolButton { event } => {
            let hold = Hold::TabletButton(event.button());
            if event.button_state() == ButtonState::Pressed {
                Edge::Press(hold)
            } else {
                Edge::Release(hold)
            }
        }
        InputEvent::TabletToolTip { event } => {
            if event.tip_state() == TabletToolTipState::Down {
                Edge::Press(Hold::TabletTip)
            } else {
                Edge::Release(Hold::TabletTip)
            }
        }
        InputEvent::DeviceAdded { .. }
        | InputEvent::DeviceRemoved { .. }
        | InputEvent::Special(_) => return false,
        _ => Edge::Other,
    };
    consume(lp, edge)
}

/// The compositor's own one-shot, not an idle-protocol client or a polling
/// timer. Unchanged deadlines do no work. A refused idle commit is attempted
/// once, until human activity or a setting change arms another deadline.
pub fn service(lp: &mut Loop) {
    let deadline = lp.inner.comp.blank.deadline();
    let policy = &mut lp.inner.comp.blank;
    if policy
        .armed
        .as_ref()
        .is_some_and(|(_, at, fired)| !fired.get() && Some(*at) == deadline)
    {
        return;
    }
    if let Some((token, _, fired)) = policy.armed.take()
        && !fired.get()
    {
        lp.loop_handle.remove(token);
    }
    let Some(at) = deadline else {
        return;
    };
    let fired = Rc::new(Cell::new(false));
    let flag = fired.clone();
    match lp
        .loop_handle
        .insert_source(Timer::from_deadline(at), move |_, _, lp| {
            flag.set(true);
            lp.inner.comp.blank.expired = true;
            if let Err(error) = set(lp, true, "idle") {
                warn!("idle panel blank: {error}");
            }
            TimeoutAction::Drop
        }) {
        Ok(token) => lp.inner.comp.blank.armed = Some((token, at, fired)),
        Err(error) => warn!("panel blank timer: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arm_rearm_fire_never_and_setting_changes() {
        let mut p = BlankPolicy::default();
        let now = p.last_human;
        assert_eq!(p.deadline(), None);
        p.configure(Some(10));
        assert_eq!(p.deadline(), Some(now + Duration::from_secs(10)));
        p.activity(now + Duration::from_secs(4));
        assert_eq!(p.deadline(), Some(now + Duration::from_secs(14)));
        p.configure(Some(3));
        assert_eq!(p.deadline(), Some(now + Duration::from_secs(7)));
        p.expired = true;
        assert_eq!(p.deadline(), None); // refused fire never loops
        p.blanked = true;
        p.configure(Some(2));
        assert_eq!(p.deadline(), None); // settings do not wake
        p.blanked = false;
        p.activity(now + Duration::from_secs(20));
        assert_eq!(p.deadline(), Some(now + Duration::from_secs(22)));
        p.configure(None);
        assert_eq!(p.deadline(), None);
    }
    #[test]
    fn waking_press_repeat_release_and_touch_are_owned() {
        for hold in [
            Hold::Key(30),
            Hold::Button(272),
            Hold::Touch(1),
            Hold::TabletTip,
            Hold::TabletButton(1),
        ] {
            let mut p = BlankPolicy {
                blanked: true,
                ..Default::default()
            };
            assert!(p.consume_edge(Edge::Press(hold)));
            p.blanked = false;
            assert!(p.consume_edge(Edge::Press(hold)));
            assert!(p.consume_edge(Edge::Motion(hold)));
            assert!(p.consume_edge(Edge::Release(hold)));
            assert!(!p.consume_edge(Edge::Press(hold)));
        }
    }
}
