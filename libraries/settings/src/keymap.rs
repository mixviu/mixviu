// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! The person's key bindings: per application, per action, the chords they
//! chose. An application's defaults ship with the application; only what
//! the person changed is stored here. An empty list unbinds the action.
//!
//! A chord is `Ctrl`, `Alt`, `Shift` and `Super` (any order) then one key, joined by
//! `+`. This module checks that grammar and the limits; whether the key is
//! one the application knows is the application's check, since the key
//! names belong to its toolkit.
use crate::model::Diagnostic;
use std::collections::BTreeMap;

/// One application's changes: action id to chords.
pub type AppKeymap = BTreeMap<String, Vec<String>>;
/// Every application's changes, by application id.
pub type Keymaps = BTreeMap<String, AppKeymap>;

pub const MAX_APPS: usize = 16;
pub const MAX_ACTIONS: usize = 256;
pub const MAX_CHORDS: usize = 4;
pub const MAX_CHORD_BYTES: usize = 32;
pub const MAX_ACTION_BYTES: usize = 64;

/// The modifiers a chord may name.
pub const MODIFIERS: [&str; 4] = ["Ctrl", "Alt", "Shift", "Super"];

fn error(path: &str, message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("validation_failed", path, message)
}

/// Whether `text` is a chord: modifiers, each once, then a key that is a
/// name of ASCII letters and digits or one of `=`, `-`, `/`.
pub fn valid_chord(text: &str) -> bool {
    if text.is_empty() || text.len() > MAX_CHORD_BYTES {
        return false;
    }
    let Some(canonical) = action::keymap::parse_chord(text) else {
        return false;
    };
    let parts: Vec<&str> = canonical.split('+').collect();
    let Some((key, mods)) = parts.split_last() else {
        return false;
    };
    let mut seen = Vec::new();
    for m in mods {
        if !MODIFIERS.contains(m) || seen.contains(m) {
            return false;
        }
        seen.push(m);
    }
    ["=", "-", "/"].contains(key)
        || (!key.is_empty() && key.bytes().all(|b| b.is_ascii_alphanumeric()))
}

/// An action id: 1–64 ASCII letters, digits, `.`, `-` or `_`.
pub fn valid_action(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= MAX_ACTION_BYTES
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
}

/// Check every application's changes against the grammar and the limits.
/// `app_id` checks an application id.
pub fn validate(
    keymaps: &Keymaps,
    app_id: impl Fn(&str, &str) -> Result<(), Diagnostic>,
) -> Result<(), Diagnostic> {
    if keymaps.len() > MAX_APPS {
        return Err(error("keymaps", "Collection limit exceeded"));
    }
    for (app, actions) in keymaps {
        app_id(app, "keymaps")?;
        if actions.len() > MAX_ACTIONS {
            return Err(error(
                &format!("keymaps.{app}"),
                "Collection limit exceeded",
            ));
        }
        for (action, chords) in actions {
            let path = format!("keymaps.{app}.{action}");
            if !valid_action(action) {
                return Err(error(&path, "Invalid action id"));
            }
            if chords.len() > MAX_CHORDS {
                return Err(error(&path, format!("At most {MAX_CHORDS} chords")));
            }
            for (n, chord) in chords.iter().enumerate() {
                if !valid_chord(chord) {
                    return Err(error(&path, format!("{chord:?} is not a chord")));
                }
                if action::keymap::reserved(chord) {
                    return Err(error(
                        &path,
                        "F1–F12 and Ctrl/Alt+arrow are reserved for the desk and are refused",
                    ));
                }
                if !valid_chord(chord) {
                    return Err(error(
                        &path,
                        format!("{chord:?} is not a chord (Ctrl, Alt, Shift, Super, then one key)"),
                    ));
                }
                if chords[..n].contains(chord) {
                    return Err(error(&path, format!("{chord:?} is listed twice")));
                }
            }
        }
    }
    Ok(())
}

/// Compatibility at stored-data boundaries only; never call on new writes.
pub fn scrub_stored(keymaps: &mut Keymaps) -> Vec<String> {
    let mut notes = Vec::new();
    keymaps.retain(|app, overrides| {
        if app != "comp" {
            notes.extend(
                action::keymap::scrub_stored(overrides)
                    .into_iter()
                    .map(|n| format!("keymaps.{app}.{n}")),
            );
        }
        !overrides.is_empty()
    });
    notes
}

/// Format-5 flat registry storage. Compositor bindings are desk policy.
pub fn scrub_values(values: &mut crate::registry::Values) -> Vec<String> {
    let mut notes = Vec::new();
    values.retain(|path, value| {
        if !path.starts_with("keymaps.") || path.starts_with("keymaps.comp.") { return true; }
        let Some(chords) = value.as_array_mut() else { return true; };
        let before = chords.len();
        chords.retain(|chord| {
            let Some(chord) = chord.as_str() else { return true; };
            if action::keymap::reserved(chord) {
                notes.push(format!("{path}: dropped stored reserved chord {chord:?}; command default applies when no valid chord remains"));
                false
            } else { true }
        });
        before == 0 || !chords.is_empty()
    });
    notes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn any_app(_: &str, _: &str) -> Result<(), Diagnostic> {
        Ok(())
    }

    #[test]
    fn chords_follow_the_grammar() {
        for good in [
            "F2",
            "Ctrl+B",
            "Alt+ArrowLeft",
            "Ctrl+Shift+N",
            "Shift+Ctrl+N",
            "Ctrl+=",
            "Ctrl+/",
            "Super+T",
            "Super+Shift+ArrowLeft",
            "Num1",
            "a",
        ] {
            assert!(valid_chord(good), "{good}");
        }
        for bad in [
            "",
            "Ctrl+",
            "+B",
            "Hyper+X",
            "Ctrl+Ctrl+B",
            "Super+Super+T",
            "RightCtrl+ArrowLeft",
            "Ctrl B",
            "Ctrl+B+C",
            "Ctrl+é",
            "Ctrl+ArrowLeftArrowLeftArrowLeftArrowLeft",
        ] {
            assert!(!valid_chord(bad), "{bad}");
        }
    }

    #[test]
    fn limits_and_duplicates_are_refused() {
        let one = |action: &str, chords: &[&str]| {
            Keymaps::from([(
                "dopus".to_owned(),
                AppKeymap::from([(
                    action.to_owned(),
                    chords.iter().map(|c| (*c).to_owned()).collect(),
                )]),
            )])
        };
        assert!(validate(&one("nav.back", &["Alt+ArrowLeft"]), any_app).is_err());
        assert!(validate(&one("nav.back", &["Ctrl+Shift+ArrowLeft"]), any_app).is_ok());
        assert!(validate(&one("nav.back", &[]), any_app).is_ok(), "unbound");
        assert!(validate(&one("nav back", &["F1"]), any_app).is_err());
        assert!(validate(&one("nav.back", &["F1", "F1"]), any_app).is_err());
        assert!(validate(&one("nav.back", &["F1", "F2", "F3", "F4", "F5"]), any_app).is_err());
        let err = validate(&one("nav.back", &["Hyper+X"]), any_app).unwrap_err();
        assert_eq!(err.path, "keymaps.dopus.nav.back");
    }
}
