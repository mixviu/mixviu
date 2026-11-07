// SPDX-License-Identifier: MIT OR Apache-2.0
//! Gold terminal key, pointer and wheel encoders; app commands remain Mix.
use super::model::Key as TerminalKey;
use egui::{Key, Modifiers, MouseWheelUnit};

pub fn meta(keys: Vec<TerminalKey>) -> Vec<TerminalKey> {
    if keys.is_empty() {
        return keys;
    }
    let mut out = Vec::with_capacity(keys.len() + 1);
    out.push(TerminalKey::Escape);
    out.extend(keys);
    out
}

pub fn pointer_cell(
    position: egui::Pos2,
    border: f32,
    cell: (f32, f32),
    grid: (u16, u16),
) -> (u16, u16) {
    let at = |offset: f32, size: f32, count: u16| {
        let index = ((offset - border) / size.max(f32::EPSILON)).floor();
        if index.is_finite() {
            index.clamp(0.0, f32::from(count.max(1) - 1)) as u16
        } else {
            0
        }
    };
    (
        at(position.x, cell.0, grid.0),
        at(position.y, cell.1, grid.1),
    )
}

/// Points of smooth (touchpad) scrolling that make one zoom step. A
/// notched wheel reports whole lines and steps once per notch.
pub const PIXELS_PER_STEP: f32 = 40.0;

/// How far one wheel event moves, in lines (`Line` and `Page` units) or
/// points.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Wheel {
    Lines(f32),
    Points(f32),
}

impl Wheel {
    /// The vertical travel of an egui wheel event; positive is away from
    /// the person (up).
    pub fn of(unit: MouseWheelUnit, delta: egui::Vec2) -> Self {
        match unit {
            MouseWheelUnit::Point => Self::Points(delta.y),
            MouseWheelUnit::Line | MouseWheelUnit::Page => Self::Lines(delta.y),
        }
    }
}

/// Ctrl+wheel into font steps: positive grows, negative shrinks. Positive
/// is wheel-up, the direction foot calls `BTN_WHEEL_BACK`, and foot binds
/// `font-increase=Control+BTN_WHEEL_BACK`, so the two agree.
///
/// Fractions accumulate in `pending`, so a touchpad's stream of small
/// deltas adds up to steps instead of rounding each to zero; turning round
/// discards what was accumulated the other way.
pub fn wheel_steps(pending: &mut f32, wheel: Wheel) -> i32 {
    steps(
        pending,
        match wheel {
            Wheel::Lines(y) => y,
            Wheel::Points(y) => y / PIXELS_PER_STEP,
        },
    )
}

/// `terminal.scroll.speed`'s default: history moves a line per wheel line,
/// and a line per cell height of touchpad travel.
pub const DEFAULT_SCROLL_SPEED: f32 = 40.0;

/// Scrollback travel: touchpad points count in cell heights, and both
/// scale by `speed` against [`DEFAULT_SCROLL_SPEED`].
pub fn scroll_steps(pending: &mut f32, wheel: Wheel, cell_height: f32, speed: f32) -> i32 {
    let rate = speed / DEFAULT_SCROLL_SPEED;
    steps(
        pending,
        match wheel {
            Wheel::Lines(y) => y * rate,
            Wheel::Points(y) => y / cell_height * rate,
        },
    )
}

/// Whole steps of `amount` travel, the fraction kept in `pending` for the
/// next event in the same direction.
fn steps(pending: &mut f32, amount: f32) -> i32 {
    if !amount.is_finite() || amount == 0.0 {
        return 0;
    }
    if amount.signum() != pending.signum() {
        *pending = 0.0;
    }
    *pending += amount;
    let steps = pending.trunc();
    *pending -= steps;
    steps as i32
}

/// The PTY keys one key press sends. Empty means the press types text,
/// which arrives as its own event ([`text_keys`], ESC-prefixed under Alt).
///
/// Alt is Meta, as in xterm with `metaSendsEscape`: ESC before the key's
/// own bytes (Alt+Enter is ESC CR, Alt+Backspace ESC DEL, Ctrl+Alt+x
/// ESC ^X), except the cursor keys, which carry the xterm modifier
/// (Alt+Left is CSI 1;3D).
pub fn keys_for(key: Key, modifiers: Modifiers) -> Vec<TerminalKey> {
    if modifiers.alt {
        if let Some(code) = cursor_code(key) {
            return vec![TerminalKey::Modified { code, modifier: 3 }];
        }
        let plain = Modifiers {
            alt: false,
            ..modifiers
        };
        return meta(keys_for(key, plain));
    }
    if modifiers.ctrl {
        // Ctrl is a chord, never text: the control code comes from the
        // letter or punctuation key, whatever text the seat would have
        // reported (egui reports none under Ctrl).
        return letter(key)
            .or_else(|| control_punctuation(key, modifiers.shift))
            .map(TerminalKey::Control)
            .into_iter()
            .collect();
    }
    named_key(key).into_iter().collect()
}

/// The character whose control code a Ctrl+punctuation key sends, as
/// xterm and foot do: Ctrl+[ is ESC (Vim's escape), Ctrl+\ FS, Ctrl+] GS,
/// Ctrl+Space and Ctrl+@ (Shift+2) NUL, Ctrl+^ (Shift+6) RS, Ctrl+_
/// (Shift+minus) and Ctrl+/ US. Shifted keys arrive as their US position
/// when egui has no name for the shifted character.
fn control_punctuation(key: Key, shift: bool) -> Option<char> {
    Some(match key {
        Key::OpenBracket => '[',
        Key::Backslash => '\\',
        Key::CloseBracket => ']',
        Key::Space => ' ',
        Key::Slash => '_',
        Key::Num2 if shift => '@',
        Key::Num6 if shift => '^',
        Key::Minus if shift => '_',
        _ => return None,
    })
}

/// The final byte of a cursor key's xterm sequence.
fn cursor_code(key: Key) -> Option<u8> {
    Some(match key {
        Key::ArrowUp => b'A',
        Key::ArrowDown => b'B',
        Key::ArrowRight => b'C',
        Key::ArrowLeft => b'D',
        Key::Home => b'H',
        Key::End => b'F',
        _ => return None,
    })
}

/// Typed text into PTY keys. Shared by key text and IME commits; format
/// characters (ZWJ, variation selectors) are text, controls are not.
pub fn text_keys(text: &str) -> Vec<TerminalKey> {
    text.chars()
        .filter(|c| !c.is_control())
        .map(TerminalKey::Char)
        .collect()
}

/// The keys typed text sends under `modifiers`: itself, ESC-prefixed
/// under Alt (egui reports the text of Alt chords, the layout's character),
/// nothing under Ctrl (the key press carries the chord).
pub fn typed(text: &str, modifiers: Modifiers) -> Vec<TerminalKey> {
    if modifiers.ctrl {
        return Vec::new();
    }
    let keys = text_keys(text);
    if modifiers.alt { meta(keys) } else { keys }
}

/// The lowercase ASCII letter a letter key stands for.
fn letter(key: Key) -> Option<char> {
    let mut chars = key.name().chars();
    let c = chars.next()?;
    (c.is_ascii_alphabetic() && chars.next().is_none()).then(|| c.to_ascii_lowercase())
}

fn named_key(key: Key) -> Option<TerminalKey> {
    Some(match key {
        Key::Enter => TerminalKey::Enter,
        Key::Backspace => TerminalKey::Backspace,
        Key::Tab => TerminalKey::Tab,
        Key::Escape => TerminalKey::Escape,
        Key::ArrowUp => TerminalKey::Up,
        Key::ArrowDown => TerminalKey::Down,
        Key::ArrowLeft => TerminalKey::Left,
        Key::ArrowRight => TerminalKey::Right,
        Key::Home => TerminalKey::Home,
        Key::End => TerminalKey::End,
        Key::Delete => TerminalKey::Delete,
        Key::PageUp => TerminalKey::PageUp,
        Key::PageDown => TerminalKey::PageDown,
        // Space carries its own text; anything else here is not a VT key.
        _ => return None,
    })
}
