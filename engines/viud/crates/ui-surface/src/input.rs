// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Input for compositor-owned UIs.
//!
//! The compositor speaks in what it has (surface-local points, Linux button
//! codes, xkb keysyms with their composed text, discrete or continuous axis
//! deltas) through [`Input`]. Each runtime turns that into egui events with
//! its [`Translator`], which remembers the pointer position and the modifier
//! state egui wants on every button, key and wheel event. Nothing outside this
//! crate touches egui's event types.

/// Held modifier keys.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub logo: bool,
}

impl Modifiers {
    pub const NONE: Self = Self {
        shift: false,
        ctrl: false,
        alt: false,
        logo: false,
    };
    pub const SHIFT: Self = Self {
        shift: true,
        ..Self::NONE
    };
    pub const CTRL: Self = Self {
        ctrl: true,
        ..Self::NONE
    };
    pub const ALT: Self = Self {
        alt: true,
        ..Self::NONE
    };
    pub const LOGO: Self = Self {
        logo: true,
        ..Self::NONE
    };

    pub fn is_empty(self) -> bool {
        self == Self::NONE
    }

    /// These modifiers with `other` held (`pressed`) or released.
    pub fn with(self, other: Self, pressed: bool) -> Self {
        let pick = |held: bool, change: bool| if change { pressed } else { held };
        Self {
            shift: pick(self.shift, other.shift),
            ctrl: pick(self.ctrl, other.ctrl),
            alt: pick(self.alt, other.alt),
            logo: pick(self.logo, other.logo),
        }
    }
}

/// A scroll amount: wheel ticks or continuous pixels, Wayland's sign
/// (positive is down and right).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScrollDelta {
    Lines { x: f32, y: f32 },
    Pixels { x: f32, y: f32 },
}

/// The backend's axis source, independent of the UI toolkit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollSource {
    Unknown,
    Wheel,
    Finger,
}

/// One key press or release.
#[derive(Clone, Debug, PartialEq)]
pub struct KeyInput {
    /// The physical key (evdev code + 8, as xkb and smithay count): what
    /// matches a release to its press while modifiers change the keysym.
    pub code: u32,
    /// The raw xkb keysym.
    pub keysym: u32,
    /// The composed text of a press, if it types anything.
    pub text: Option<String>,
    pub pressed: bool,
    pub repeat: bool,
}

/// Input for one UI instance. Positions are surface-local, in points
/// (physical pixels divided by the instance's scale).
#[derive(Clone, Debug, PartialEq)]
pub enum Input {
    PointerMoved {
        x: f32,
        y: f32,
    },
    PointerLeft,
    /// A Linux input-event code (`BTN_LEFT` is 0x110).
    Button {
        code: u32,
        pressed: bool,
    },
    Scroll {
        delta: ScrollDelta,
        source: ScrollSource,
    },
    Key(KeyInput),
    Modifiers(Modifiers),
    Focused(bool),
}

// xkb keysyms (stable: X11 protocol, keysymdef.h).
pub const XK_SHIFT_L: u32 = 0xffe1;
pub const XK_SHIFT_R: u32 = 0xffe2;
pub const XK_CONTROL_L: u32 = 0xffe3;
pub const XK_CONTROL_R: u32 = 0xffe4;
pub const XK_ALT_L: u32 = 0xffe9;
pub const XK_ALT_R: u32 = 0xffea;
pub const XK_SUPER_L: u32 = 0xffeb;
pub const XK_SUPER_R: u32 = 0xffec;

/// The modifier a keysym is, if it is one.
pub fn keysym_modifier(keysym: u32) -> Option<Modifiers> {
    match keysym {
        XK_SHIFT_L | XK_SHIFT_R => Some(Modifiers::SHIFT),
        XK_CONTROL_L | XK_CONTROL_R => Some(Modifiers::CTRL),
        XK_ALT_L | XK_ALT_R => Some(Modifiers::ALT),
        XK_SUPER_L | XK_SUPER_R => Some(Modifiers::LOGO),
        _ => None,
    }
}

/// A key event from smithay's keyboard handler.
pub fn key(code: u32, keysym: u32, utf8: Option<&str>, pressed: bool, repeat: bool) -> Input {
    let text = utf8
        .filter(|text| pressed && !text.is_empty())
        .map(str::to_owned);
    Input::Key(KeyInput {
        code,
        keysym,
        text,
        pressed,
        repeat,
    })
}

/// A scroll event: `discrete_*` are wheel ticks, `pixel_*` continuous
/// deltas (touchpad). Ticks win when present; `None` when both are zero.
pub fn wheel(
    discrete_x: i32,
    discrete_y: i32,
    pixel_x: f64,
    pixel_y: f64,
    source: ScrollSource,
) -> Option<Input> {
    let delta = if discrete_x != 0 || discrete_y != 0 {
        ScrollDelta::Lines {
            x: discrete_x as f32,
            y: discrete_y as f32,
        }
    } else if pixel_x != 0.0 || pixel_y != 0.0 {
        ScrollDelta::Pixels {
            x: pixel_x as f32,
            y: pixel_y as f32,
        }
    } else {
        return None;
    };
    Some(Input::Scroll { delta, source })
}

use smithay::backend::input::KeyState as SmithayKeyState;
pub fn translate_keyboard(
    code: u32,
    keysym_raw: u32,
    utf8: Option<&str>,
    key_state: SmithayKeyState,
    is_repeat: bool,
) -> Input {
    let pressed = matches!(key_state, SmithayKeyState::Pressed);
    key(code, keysym_raw, utf8, pressed, is_repeat)
}
