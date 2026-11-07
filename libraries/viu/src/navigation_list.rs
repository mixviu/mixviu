// SPDX-License-Identifier: MIT OR Apache-2.0
//! A bounded navigation rail. The Mix owner supplies selection and destinations.
use serde_json::{Value, json};

pub fn validate(value: &Value) -> Result<(), String> {
    if value["rows"].as_array().is_some_and(|rows| {
        rows.len() <= 4096
            && rows.iter().all(|row| {
                row["key"].is_string()
                    && row["cells"][0].is_string()
                    && row["selected"].is_boolean()
                    && row["icon"]
                        .as_str()
                        .and_then(crate::icons::Icon::from_name)
                        .is_some()
            })
    }) {
        Ok(())
    } else {
        Err("navigation-list requires bounded keyed icon/text rows".into())
    }
}

pub fn draw(
    ui: &mut egui::Ui,
    id: &str,
    label: &str,
    value: &Value,
    events: &mut Vec<crate::widgets::Intent>,
) -> egui::Response {
    ui.vertical(|ui| {
        toolkit::panel::section_label(ui, label);
        let font = egui::TextStyle::Body.resolve(ui.style());
        egui::ScrollArea::vertical()
            .auto_shrink(false)
            .show(ui, |ui| {
                for row in value["rows"].as_array().unwrap() {
                    ui.push_id(row["key"].as_str().unwrap(), |ui| {
                        ui.horizontal(|ui| {
                            let icon = crate::icons::Icon::from_name(row["icon"].as_str().unwrap())
                                .unwrap();
                            ui.add(crate::icons::image_of(
                                ui.ctx(),
                                icon,
                                16.0,
                                ui.visuals().weak_text_color(),
                            ));
                            let width = (ui.available_width()
                                - 2.0 * ui.spacing().button_padding.x)
                                .max(0.0);
                            let text = crate::elide::fit(
                                ui,
                                row["cells"][0].as_str().unwrap(),
                                &font,
                                width,
                            );
                            let response = toolkit::selectable(ui, row["selected"] == true, text)
                                .on_hover_text(row["path"].as_str().unwrap_or(""));
                            if response.clicked() {
                                events.push(crate::widgets::Intent {
                                    id: id.into(),
                                    action: "select".into(),
                                    value: json!({"key":row["key"]}),
                                });
                            }
                        });
                    });
                }
            });
    })
    .response
}
