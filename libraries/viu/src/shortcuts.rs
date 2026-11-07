// SPDX-License-Identifier: MIT OR Apache-2.0
//! Exact chord admission shared by data-authored application frames.
/// Resolve data-authored defaults through gold's binding conflict policy.
pub fn resolve(args: &serde_json::Value) -> Result<serde_json::Value, String> {
    use serde_json::{Value, json};
    let defaults = args["defaults"]
        .as_array()
        .filter(|a| a.len() <= 256)
        .ok_or("bounded defaults required")?;
    let mut defaults = defaults.iter().collect::<Vec<_>>();
    defaults.sort_by_key(|item| item["id"].as_str().unwrap_or(""));
    let overrides: toolkit::keymap::Overrides =
        serde_json::from_value(args["overrides"].clone()).map_err(|e| e.to_string())?;
    let mut registry = toolkit::Registry::<()>::new();
    let mut ids = std::collections::BTreeMap::new();
    let mut chords = Vec::new();
    for (index, item) in defaults.iter().enumerate() {
        let id = item["id"].as_str().ok_or("command ID required")?;
        if ids.contains_key(id) {
            return Err("duplicate command ID".into());
        }
        let slot = crate::native_menu::command_slot(index).unwrap();
        let shortcut = item["shortcut"]
            .as_str()
            .map(|s| toolkit::keymap::parse_chord(s).ok_or("invalid default chord"))
            .transpose()?;
        if let Some(chord) = shortcut {
            if toolkit::keymap::user_only(chord.logical_key) || chords.contains(&chord) {
                return Err("invalid or duplicate default chord".into());
            }
            chords.push(chord);
        }
        registry.add(toolkit::Action {
            id: slot,
            label: slot,
            menu: None,
            submenu: None,
            group: 0,
            shortcut,
            icon: None,
            enabled: |_| true,
            run: |_| {},
        });
        ids.insert(id.to_owned(), slot);
    }
    for item in &defaults {
        if let Some(keys) = item["keys"].as_array() {
            if keys.len() > 16 {
                return Err("too many default chords".into());
            }
            let id = item["id"].as_str().unwrap();
            let extra = keys
                .iter()
                .skip(1)
                .map(|key| {
                    key.as_str()
                        .and_then(toolkit::keymap::parse_chord)
                        .ok_or("invalid default chord")
                })
                .collect::<Result<Vec<_>, _>>()?;
            registry.add_shortcuts(ids[id], extra);
        }
    }
    let mapped = overrides
        .iter()
        .filter_map(|(id, keys)| ids.get(id).map(|slot| (slot.to_string(), keys.clone())))
        .collect();
    let notes = registry.rebind(&mapped);
    let bindings = ids
        .into_iter()
        .map(|(id, slot)| {
            let keys = registry
                .shortcuts_of(slot)
                .iter()
                .map(toolkit::keymap::chord_text)
                .collect::<Vec<_>>();
            (id, json!({"shortcut":keys.first(),"keys":keys}))
        })
        .collect::<serde_json::Map<String, Value>>();
    Ok(json!({"bindings":bindings,"notes":notes}))
}
pub fn pressed(ctx: &egui::Context, actions: &serde_json::Value) -> Vec<serde_json::Value> {
    let typing = ctx.egui_wants_keyboard_input();
    let mut fired = Vec::new();
    ctx.input_mut(|input| {
        let held = input.modifiers;
        input.events.retain(|event| {
            let (key, modifiers) = match event {
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => (*key, *modifiers),
                egui::Event::Copy if !typing => (egui::Key::C, held),
                egui::Event::Cut if !typing => (egui::Key::X, held),
                egui::Event::Paste(_) | egui::Event::PasteImage(_) if !typing => {
                    (egui::Key::V, held)
                }
                _ => return true,
            };
            if let Some(action) = actions.as_array().into_iter().flatten().find(|action| {
                action["shortcut"]
                    .as_str()
                    .and_then(toolkit::keymap::parse_chord)
                    .is_some_and(|chord| {
                        chord.logical_key == key && modifiers.matches_exact(chord.modifiers)
                    })
            }) {
                if action["enabled"] == true && !(typing && action["scope"] == "browsing") {
                    fired.push(action["id"].clone());
                    return false;
                }
            }
            true
        });
    });
    fired
}
#[cfg(test)]
fn take(ctx: &egui::Context, chord: &egui::KeyboardShortcut) -> bool {
    let clipboard = !ctx.egui_wants_keyboard_input();
    ctx.input_mut(|input| {
        let found = input.events.iter().position(|event| {
            let (key, modifiers) = match event {
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => (*key, *modifiers),
                egui::Event::Copy if clipboard => (egui::Key::C, input.modifiers),
                egui::Event::Cut if clipboard => (egui::Key::X, input.modifiers),
                egui::Event::Paste(_) | egui::Event::PasteImage(_) if clipboard => {
                    (egui::Key::V, input.modifiers)
                }
                _ => return false,
            };
            key == chord.logical_key && modifiers.matches_exact(chord.modifiers)
        });
        if let Some(index) = found {
            input.events.remove(index);
            true
        } else {
            false
        }
    })
}
#[cfg(test)]
mod tests {
    #[test]
    fn secondary_defaults_survive_profile_changes_and_restore() {
        let defaults = serde_json::json!([{"id":"parent","shortcut":"Backspace","keys":["Backspace","Alt+ArrowUp"]}]);
        let original =
            super::resolve(&serde_json::json!({"defaults":defaults,"overrides":{}})).unwrap();
        assert_eq!(
            original["bindings"]["parent"]["keys"],
            serde_json::json!(["Backspace", "Alt+ArrowUp"])
        );
        let changed = super::resolve(
            &serde_json::json!({"defaults":defaults,"overrides":{"parent":["Ctrl+U"]}}),
        )
        .unwrap();
        assert_eq!(
            changed["bindings"]["parent"]["keys"],
            serde_json::json!(["Ctrl+U"])
        );
        let restored =
            super::resolve(&serde_json::json!({"defaults":defaults,"overrides":{}})).unwrap();
        assert_eq!(restored["bindings"], original["bindings"]);
    }
    #[test]
    fn user_bindings_claim_defaults_and_restore_without_losing_extra_keys() {
        let defaults = serde_json::json!([{"id":"view.b","shortcut":"Ctrl+B"},{"id":"view.a","shortcut":"Ctrl+A"}]);
        let result = super::resolve(&serde_json::json!({"defaults":defaults,"overrides":{"view.b":["Ctrl+A","F2"],"view.a":[]}})).unwrap();
        assert_eq!(result["bindings"]["view.a"]["keys"], serde_json::json!([]));
        assert_eq!(
            result["bindings"]["view.b"]["keys"],
            serde_json::json!(["Ctrl+A", "F2"])
        );
        let restored =
            super::resolve(&serde_json::json!({"defaults":defaults,"overrides":{}})).unwrap();
        assert_eq!(restored["bindings"]["view.a"]["shortcut"], "Ctrl+A");
        assert_eq!(restored["bindings"]["view.b"]["shortcut"], "Ctrl+B");
    }
    #[test]
    fn commands_follow_native_press_order() {
        let ctx = egui::Context::default();
        let event = |key| egui::Event::Key {
            key,
            physical_key: None,
            pressed: true,
            repeat: false,
            modifiers: egui::Modifiers::NONE,
        };
        ctx.run_ui(egui::RawInput { events:vec![event(egui::Key::B),event(egui::Key::A)], ..Default::default() },|ui| {
            assert_eq!(super::pressed(ui.ctx(),&serde_json::json!([{"id":"a","shortcut":"A","enabled":true},{"id":"b","shortcut":"B","enabled":true}])),vec![serde_json::json!("b"),serde_json::json!("a")]);
        }).drop_without_applying_deltas();
    }
    #[test]
    fn shift_delete_never_falls_through_to_delete() {
        let ctx = egui::Context::default();
        ctx.run_ui(
            egui::RawInput {
                events: vec![egui::Event::Key {
                    key: egui::Key::Delete,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::SHIFT,
                }],
                ..Default::default()
            },
            |ui| {
                assert!(!super::take(
                    ui.ctx(),
                    &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Delete)
                ));
                assert!(super::take(
                    ui.ctx(),
                    &egui::KeyboardShortcut::new(egui::Modifiers::SHIFT, egui::Key::Delete)
                ));
            },
        )
        .drop_without_applying_deltas();
    }
}
