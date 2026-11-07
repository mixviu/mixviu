// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native command search over application-owned labels, IDs and availability.
use serde_json::{Value, json};
use std::sync::OnceLock;

pub fn validate(value: &Value) -> Result<(), String> {
    let mut ids = std::collections::BTreeSet::new();
    if value["open"].is_boolean()
        && value["commands"].as_array().is_some_and(|rows| {
            rows.len() <= 256
                && rows.iter().all(|row| {
                    row["id"]
                        .as_str()
                        .is_some_and(|id| !id.is_empty() && id.len() <= 128 && ids.insert(id))
                        && row["label"].as_str().is_some_and(|label| {
                            label.len() <= 4096 && !label.chars().any(char::is_control)
                        })
                        && row["enabled"].is_boolean()
                })
        })
    {
        Ok(())
    } else {
        Err("invalid command palette".into())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn labels_remain_literal_and_duplicate_ids_refuse_before_drawing() {
        let value = serde_json::json!({"open":true,"commands":[{"id":"test","label":"Quotes \" braces { } and \\ — literal","enabled":true}]});
        assert!(super::validate(&value).is_ok());
        let context = egui::Context::default();
        crate::appearance::install(&context);
        context
            .run_ui(egui::RawInput::default(), |ui| {
                super::draw(ui, "palette", &value, &mut Vec::new());
            })
            .drop_without_applying_deltas();
        let mut duplicate = value.clone();
        duplicate["commands"]
            .as_array_mut()
            .unwrap()
            .push(value["commands"][0].clone());
        assert!(super::validate(&duplicate).is_err());
    }
}

pub fn draw(
    ui: &mut egui::Ui,
    id: &str,
    value: &Value,
    events: &mut Vec<crate::widgets::Intent>,
) -> egui::Response {
    if value["open"] != true {
        let _ = toolkit::search::show(
            ui.ctx(),
            &toolkit::Registry::<()>::new(),
            &(),
            &toolkit::Strings::new(""),
            false,
        );
        return ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover());
    }
    let (registry, strings) = catalogue(ui.ctx(), &value["commands"], true);
    if let Some(choice) =
        toolkit::search::show(ui.ctx(), &registry, &(), &strings, value["open"] == true)
    {
        events.push(crate::widgets::Intent {
            id: id.into(),
            action: "choose".into(),
            value: choice
                .and_then(|(slot, _)| crate::native_menu::application_id(ui.ctx(), slot))
                .map_or(Value::Null, |command| json!({"command":command})),
        });
    }
    ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
}

/// The native shortcut grid shares the current, app-owned command catalogue.
pub fn shortcuts(ui: &mut egui::Ui, commands: &Value) {
    let (registry, strings) = catalogue(ui.ctx(), commands, false);
    registry.shortcut_grid(ui, &strings);
}

fn catalogue(
    ctx: &egui::Context,
    commands: &Value,
    enabled_only: bool,
) -> (toolkit::Registry<()>, toolkit::Strings) {
    static LABELS: OnceLock<Vec<&'static str>> = OnceLock::new();
    let labels = LABELS.get_or_init(|| {
        (0..256)
            .map(|i| &*Box::leak(format!("viu-palette-label-{i}").into_boxed_str()))
            .collect()
    });
    let mut registry = toolkit::Registry::<()>::new();
    let mut catalogue = String::new();
    for (i, row) in commands.as_array().unwrap().iter().enumerate() {
        if enabled_only && row["enabled"] != true {
            continue;
        }
        let Some(slot) = crate::native_menu::native_id(ctx, row["id"].as_str().unwrap()) else {
            continue;
        };
        // Quoted Fluent string expressions preserve punctuation without letting
        // application labels become catalogue syntax. Controls were refused.
        catalogue.push_str(&format!(
            "{} = {{ {} }}\n",
            labels[i],
            serde_json::to_string(row["label"].as_str().unwrap()).unwrap()
        ));
        let keys = row["keys"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|key| key.as_str().and_then(toolkit::keymap::parse_chord))
            .collect::<Vec<_>>();
        registry.add(toolkit::Action {
            id: slot,
            label: labels[i],
            menu: None,
            submenu: None,
            group: 0,
            shortcut: keys.first().copied(),
            icon: None,
            enabled: |_| true,
            run: |_| {},
        });
        registry.add_shortcuts(slot, keys.into_iter().skip(1));
    }
    let strings = toolkit::Strings::new(&catalogue);
    (registry, strings)
}
