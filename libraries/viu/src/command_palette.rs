// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native command search over application-owned labels, IDs and availability.
use serde_json::{Value, json};
use std::sync::OnceLock;

pub fn validate(value: &Value) -> Result<(), String> {
    if value["open"].is_boolean() && value["commands"].as_array().is_some_and(|rows| {
        rows.len() <= 256 && rows.iter().all(|row| row["id"].is_string()
            && row["label"].as_str().is_some_and(|label| label.len() <= 4096 && !label.chars().any(char::is_control))
            && row["enabled"].is_boolean())
    }) { Ok(()) } else { Err("invalid command palette".into()) }
}

pub fn draw(ui: &mut egui::Ui, id: &str, value: &Value, events: &mut Vec<crate::widgets::Intent>) -> egui::Response {
    if value["open"] != true {
        let _ = toolkit::search::show(ui.ctx(), &toolkit::Registry::<()>::new(), &(), &toolkit::Strings::new(""), false);
        return ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover());
    }
    static LABELS: OnceLock<Vec<&'static str>> = OnceLock::new();
    let labels = LABELS.get_or_init(|| (0..256).map(|i| &*Box::leak(format!("viu-palette-label-{i}").into_boxed_str())).collect());
    let mut registry = toolkit::Registry::<()>::new();
    let mut catalogue = String::new();
    for (i, row) in value["commands"].as_array().unwrap().iter().enumerate() {
        if row["enabled"] != true { continue; }
        let Some(slot) = crate::native_menu::native_id(ui.ctx(), row["id"].as_str().unwrap()) else { continue; };
        // Quoted Fluent string expressions preserve punctuation without letting
        // application labels become catalogue syntax. Controls were refused.
        catalogue.push_str(&format!("{} = {{ {} }}\n", labels[i], serde_json::to_string(row["label"].as_str().unwrap()).unwrap()));
        registry.add(toolkit::Action { id:slot, label:labels[i], menu:None, submenu:None, group:0, shortcut:None, icon:None, enabled: |_| true, run: |_| {} });
    }
    let strings = toolkit::Strings::new(&catalogue);
    if let Some(choice) = toolkit::search::show(ui.ctx(), &registry, &(), &strings, value["open"] == true) {
        events.push(crate::widgets::Intent { id:id.into(), action:"choose".into(), value:choice.and_then(|(slot, _)| crate::native_menu::application_id(ui.ctx(), slot)).map_or(Value::Null, |command| json!({"command":command})) });
    }
    ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
}
