// SPDX-License-Identifier: MIT OR Apache-2.0
//! Labelled header rows, optional row actions and attachment chips. The Mix
//! owner supplies visibility, values and commands; this control owns no mail.
use crate::widgets::Intent;
use egui::{Id, Ui};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

pub fn validate(v: &Value) -> bool {
    v["enabled"].is_boolean()
        && v["fields"].as_array().is_some_and(|rows| rows.len() <= 32 && rows.iter().all(|r| {
            r["id"].is_string() && r["label"].is_string()
                && r["value"].as_str().is_some_and(|s| s.len() <= 65536)
                && r["visible"].is_boolean()
                && r["actions"].as_array().is_some_and(|a| a.len() <= 8 && a.iter().all(|a| {
                    a["id"].is_string() && a["label"].is_string()
                        && a["icon"].as_str().and_then(crate::icons::Icon::from_name).is_some()
                }))
        }))
        && v["attachments"].as_array().is_some_and(|a| a.len() <= 1024 && a.iter().all(|p| p.as_str().is_some_and(|s| s.len() <= 65536)))
        && v["attachment_label"].is_string() && v["attach_label"].is_string()
        && v["remove_label"].is_string()
        && v["chooser"]["open"].is_boolean()
        && v["chooser"]["title"].is_string() && v["chooser"]["directory"].is_string()
}
pub fn admit(current: &Value, action: &str, payload: &Value) -> Result<(), String> {
    let valid = current["enabled"] == true && match action {
        "edit" => payload.as_object().is_some_and(|m| !m.is_empty() && m.iter().all(|(key, v)| {
            v.as_str().is_some_and(|s| s.len() <= 65536) && current["fields"].as_array().unwrap().iter().any(|r| r["id"] == *key && r["read_only"] != true && r["visible"] == true)
        })),
        "commit" => payload.is_null() || payload.as_str().is_some_and(|key| current["fields"].as_array().unwrap().iter().any(|r| r["id"] == key)),
        "command" => payload == "attach" || payload.as_str().is_some_and(|key| current["fields"].as_array().unwrap().iter().flat_map(|r| r["actions"].as_array().unwrap()).any(|a| a["id"] == key)),
        "remove" => payload.as_u64().is_some_and(|i| i < current["attachments"].as_array().unwrap().len() as u64),
        "picked" => current["chooser"]["open"] == true && payload.as_array().is_some_and(|a| a.len() <= 1024 && a.iter().all(|p| p.as_str().is_some_and(|s| s.len() <= 65536))),
        "cancel" => current["chooser"]["open"] == true && payload.is_null(),
        _ => false,
    };
    if valid { Ok(()) } else { Err("invalid header form intent".into()) }
}
pub fn draw(ui: &mut Ui, id: &str, v: &Value, events: &mut Vec<Intent>) -> egui::Response {
    use toolkit::header_form as form;
    let emit = |events: &mut Vec<Intent>, action: &str, value| events.push(Intent { id: id.into(), action: action.into(), value });
    let response = ui.scope(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;
        let column = form::column(ui, v["fields"].as_array().unwrap().iter().map(|r| r["label"].as_str().unwrap().to_owned()).chain([v["attachment_label"].as_str().unwrap().to_owned()]));
        let mut edits = serde_json::Map::new();
        let mut commits = Vec::new();
        ui.add_enabled_ui(v["enabled"] == true, |ui| {
            for r in v["fields"].as_array().unwrap().iter().filter(|r| r["visible"] == true) {
                let key = r["id"].as_str().unwrap();
                let mut text = r["value"].as_str().unwrap().to_owned();
                form::row(ui, r["label"].as_str().unwrap(), column, |ui, labelled| {
                    let mut field = |ui: &mut Ui| {
                        let response = ui.add_enabled_ui(r["read_only"] != true, |ui| form::text(ui, Id::unique((id, &v["identity"], key)), labelled, &mut text, ui.available_width())).inner;
                        let marker = Id::unique((id, key, "focus"));
                        if r["focus"] == true && ui.ctx().data(|d| d.get_temp::<Value>(marker)).as_ref() != Some(&v["focus_instance"]) {
                            response.request_focus();
                            ui.ctx().data_mut(|d| d.insert_temp(marker, v["focus_instance"].clone()));
                        }
                        if response.has_focus() { ui.ctx().data_mut(|d| d.insert_temp(Id::unique("viu-focused-widget"), id.to_owned())); }
                        if response.changed() { edits.insert(key.to_owned(), json!(text)); }
                        if response.lost_focus() { commits.push(json!(key)); }
                    };
                    if r["actions"].as_array().unwrap().is_empty() { field(ui); } else {
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            for a in r["actions"].as_array().unwrap().iter().rev() {
                                let icon = crate::icons::Icon::from_name(a["icon"].as_str().unwrap()).unwrap();
                                if ui.add(toolkit::button::IconButton::new(icon).size(toolkit::field::height_of(ui.ctx())).selected(a["selected"] == true).tooltip(a["label"].as_str().unwrap())).clicked() {
                                    emit(events, "command", a["id"].clone());
                                }
                            }
                            field(ui);
                        });
                    }
                });
            }
            form::row(ui, v["attachment_label"].as_str().unwrap(), column, |ui, _| {
                ui.horizontal_wrapped(|ui| {
                    if ui.button(v["attach_label"].as_str().unwrap()).clicked() { emit(events, "command", json!("attach")); }
                    for (i, p) in v["attachments"].as_array().unwrap().iter().enumerate() {
                        let p = p.as_str().unwrap();
                        if form::attachment_chip(ui, i, std::path::Path::new(p), p, false, toolkit::field::height_of(ui.ctx()), v["remove_label"].as_str().unwrap()) { emit(events, "remove", json!(i)); }
                    }
                });
            });
        });
        // Preview values always precede commands collected in this frame.
        if !edits.is_empty() { events.insert(0, Intent { id: id.into(), action: "edit".into(), value: Value::Object(edits) }); }
        for key in commits { emit(events, "commit", key); }
        if let Some(notice) = v["notice"].as_str().filter(|s| !s.is_empty()) { ui.weak(notice); }
    }).response;
    let chooser_id = Id::unique((id, "chooser", &v["identity"], &v["chooser"]["instance"]));
    if v["chooser"]["open"] == true {
        let chooser = ui.ctx().data_mut(|d| d.get_temp_mut_or_insert_with(chooser_id, || Arc::new(Mutex::new(toolkit::file_chooser::FileChooser::new(v["chooser"]["directory"].as_str().unwrap(), v["chooser"]["title"].as_str().unwrap(), v["attach_label"].as_str().unwrap())))).clone());
        let outcome = chooser.lock().unwrap().show(ui.ctx());
        if let Some(outcome) = outcome {
            match outcome {
                toolkit::file_chooser::Outcome::Picked(paths) => emit(events, "picked", json!(paths)),
                _ => emit(events, "cancel", Value::Null),
            }
            ui.ctx().data_mut(|d| d.remove::<Arc<Mutex<toolkit::file_chooser::FileChooser>>>(chooser_id));
        }
    } else { ui.ctx().data_mut(|d| d.remove::<Arc<Mutex<toolkit::file_chooser::FileChooser>>>(chooser_id)); }
    response
}
