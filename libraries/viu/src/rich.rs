// SPDX-License-Identifier: MIT OR Apache-2.0
//! Application-supplied menus, command bars, property fields and modal forms.
//! Command identity, availability and form outcomes remain application data.
use crate::widgets::{Intent, Widget};
use egui::{Rect, Ui};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub const KINDS: &[&str] = &["menubar", "command-bar", "property-grid", "dialog-form"];
pub fn actions(kind: &str) -> Vec<&'static str> {
    match kind {
        "menubar" | "command-bar" => vec!["set", "command"],
        "dialog-form" => vec!["set", "edit", "answer"],
        _ => vec!["set"],
    }
}
fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}
fn entries(value: &Value, depth: usize, ids: &mut BTreeSet<String>) -> Result<(), String> {
    if depth > 8 {
        return Err("menu nesting exceeds eight levels".into());
    }
    let values = value.as_array().ok_or("command items must be an array")?;
    for entry in values {
        if entry["separator"] == true {
            continue;
        }
        if text(entry, "id").is_empty()
            || !ids.insert(text(entry, "id").into())
            || !entry["label"].is_string()
            || !entry["enabled"].is_boolean()
        {
            return Err("command items require unique ID, label and enabled flag".into());
        }
        if let Some(icon) = entry["icon"].as_str() {
            if crate::icons::Icon::from_name(icon).is_none() {
                return Err("unknown command icon".into());
            }
        }
        if entry.get("children").is_some() {
            entries(&entry["children"], depth + 1, ids)?;
        }
    }
    Ok(())
}
pub fn validate(kind: &str, value: &Value) -> Result<(), String> {
    match kind {
        "menubar" | "command-bar" => entries(value, 0, &mut BTreeSet::new()),
        "property-grid" => {
            let fields = value.as_array().ok_or("property fields must be an array")?;
            if fields
                .iter()
                .any(|f| !f["label"].is_string() || !f["value"].is_string())
            {
                Err("property fields require label and value strings".into())
            } else {
                Ok(())
            }
        }
        "dialog-form" => {
            if !value["open"].is_boolean()
                || !value["title"].is_string()
                || !value["body"].is_string()
            {
                return Err("modal form requires open, title and body".into());
            }
            if value
                .get("field")
                .is_some_and(|f| !f.is_null() && !f.is_string())
            {
                return Err("modal form field must be string or null".into());
            }
            if value
                .get("max_field_bytes")
                .is_some_and(|v| v.as_u64().is_none_or(|n| n == 0 || n > 1_048_576))
                || value.get("multiline").is_some_and(|v| !v.is_boolean())
                || value
                    .get("rows")
                    .is_some_and(|v| v.as_u64().is_none_or(|n| !(1..=40).contains(&n)))
                || value.get("width").is_some_and(|v| {
                    v.as_f64()
                        .is_none_or(|n| !n.is_finite() || !(320.0..=1200.0).contains(&n))
                })
            {
                return Err("invalid bounded form editor options".into());
            }
            entries(&value["choices"], 0, &mut BTreeSet::new())
        }
        _ => Err("unknown rich control".into()),
    }
}
fn find<'a>(value: &'a Value, id: &str) -> Option<&'a Value> {
    for item in value.as_array().into_iter().flatten() {
        if item["enabled"] != true {
            continue;
        }
        if text(item, "id") == id {
            return Some(item);
        }
        if let Some(found) = find(&item["children"], id) {
            return Some(found);
        }
    }
    None
}
pub fn admit(kind: &str, current: &Value, action: &str, payload: &Value) -> Result<(), String> {
    validate(kind, current)?;
    match action {
        "command" => {
            if find(current, text(payload, "command"))
                .is_none_or(|item| item["enabled"] != true || item.get("children").is_some())
            {
                return Err("disabled or unknown command".into());
            }
        }
        "answer" => {
            if current["open"] != true
                || current
                    .get("instance")
                    .is_some_and(|instance| payload.get("instance") != Some(instance))
                || find(&current["choices"], text(payload, "choice"))
                    .is_none_or(|item| item["enabled"] != true)
            {
                return Err("disabled or unknown form choice".into());
            }
        }
        "edit" => {
            if current["open"] != true
                || !current["field"].is_string()
                || payload.as_str().is_none_or(|s| {
                    s.len() > current["max_field_bytes"].as_u64().unwrap_or(4096) as usize
                })
            {
                return Err("form is not editable".into());
            }
        }
        _ => {}
    }
    Ok(())
}
fn emit(events: &mut Vec<Intent>, id: &str, action: &str, value: Value) {
    events.push(Intent {
        id: id.into(),
        action: action.into(),
        value,
    });
}
fn label(item: &Value) -> egui::RichText {
    egui::RichText::new(format!(
        "{}{}",
        if item["checked"] == true { "✓  " } else { "" },
        text(item, "label")
    ))
}
fn menu(
    ui: &mut Ui,
    value: &Value,
    id: &str,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) {
    for item in value.as_array().into_iter().flatten() {
        if item["separator"] == true {
            ui.separator();
            continue;
        }
        if item.get("children").is_some() {
            let shown = ui.add_enabled_ui(item["enabled"] == true, |ui| {
                ui.menu_button(label(item), |ui| {
                    menu(ui, &item["children"], id, geometry, events)
                })
            });
            geometry.insert(
                format!("{id}.command.{}", text(item, "id")),
                shown.inner.response.rect,
            );
        } else {
            let mut button = if let Some(icon) = item["icon"]
                .as_str()
                .and_then(crate::icons::Icon::from_name)
            {
                egui::Button::image_and_text(
                    crate::icons::image_of(ui.ctx(), icon, 16.0, ui.visuals().text_color()),
                    label(item),
                )
            } else {
                egui::Button::new(label(item))
            };
            if let Some(shortcut) = item["shortcut"].as_str() {
                button = button.shortcut_text(shortcut);
            }
            let response = ui.add_enabled(item["enabled"] == true, button);
            geometry.insert(format!("{id}.command.{}", text(item, "id")), response.rect);
            if response.clicked() {
                emit(events, id, "command", json!({"command":item["id"]}));
                ui.close();
            }
        }
    }
}
pub(crate) fn draw(
    ui: &mut Ui,
    widget: &Widget,
    value: &Value,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) -> egui::Response {
    let id = &widget.id;
    match widget.kind.as_str() {
        "menubar" => {
            egui::MenuBar::new()
                .ui(ui, |ui| {
                    for item in value.as_array().into_iter().flatten() {
                        let shown = ui.add_enabled_ui(item["enabled"] == true, |ui| {
                            ui.menu_button(text(item, "label"), |ui| {
                                menu(ui, &item["children"], id, geometry, events)
                            })
                        });
                        geometry.insert(
                            format!("{id}.menu.{}", text(item, "id")),
                            shown.inner.response.rect,
                        );
                    }
                })
                .response
        }
        "command-bar" => {
            ui.horizontal(|ui| {
                for item in value.as_array().into_iter().flatten() {
                    if item["separator"] == true {
                        ui.separator();
                        continue;
                    }
                    let response = if let Some(icon) = item["icon"]
                        .as_str()
                        .and_then(crate::icons::Icon::from_name)
                    {
                        crate::icons::button(
                            ui,
                            icon,
                            if item["icon_only"] == true {
                                ""
                            } else {
                                text(item, "label")
                            },
                            16.0,
                            None,
                            item["enabled"] == true,
                            item["checked"] == true,
                        )
                    } else {
                        ui.add_enabled(item["enabled"] == true, egui::Button::new(label(item)))
                    };
                    let response = response.on_hover_text(text(item, "label"));
                    geometry.insert(format!("{id}.command.{}", text(item, "id")), response.rect);
                    if item.get("children").is_some() {
                        egui::Popup::menu(&response)
                            .show(|ui| menu(ui, &item["children"], id, geometry, events));
                    } else if response.clicked() {
                        emit(events, id, "command", json!({"command":item["id"]}));
                    }
                }
            })
            .response
        }
        "property-grid" => {
            ui.vertical(|ui| {
                egui::ScrollArea::vertical()
                    .id_salt((id, "fields"))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for field in value.as_array().into_iter().flatten() {
                            ui.label(egui::RichText::new(text(field, "label")).small().weak());
                            ui.add(egui::Label::new(text(field, "value")).wrap());
                            ui.add_space(4.0);
                        }
                    });
            })
            .response
        }
        "dialog-form" => {
            let mut field = value["field"].as_str().map(str::to_owned);
            let modal = egui::Modal::new(egui::Id::unique((ui.ctx().viewport_id(), id, "modal")))
                .show(ui.ctx(), |ui| {
                    let width=value["width"].as_f64().unwrap_or(520.0) as f32;
                    ui.set_min_width(width.min(ui.ctx().content_rect().width()-32.0).max(280.0));
                    ui.set_max_width(width);
                    ui.heading(text(value, "title"));
                    if !text(value, "body").is_empty() {
                        egui::ScrollArea::vertical()
                            .id_salt((id, "body"))
                            .max_height(350.0)
                            .show(ui, |ui| {
                                ui.add(egui::Label::new(text(value, "body")).wrap());
                            });
                    }
                    if let Some(text) = field.as_mut() {
                        let edit=if value["multiline"]==true{egui::TextEdit::multiline(text).desired_rows(value["rows"].as_u64().unwrap_or(16) as usize).code_editor()}else{egui::TextEdit::singleline(text)};
                        let response = ui.add(
                            edit
                                .id(egui::Id::unique((ui.ctx().viewport_id(), id, "field")))
                                .desired_width(ui.available_width()),
                        );
                        geometry.insert(format!("{id}.field"), response.rect);
                        let marker = egui::Id::unique((
                            ui.ctx().viewport_id(),
                            id,
                            "modal-focused",
                        ));
                        if ui.ctx().data(|d|d.get_temp::<Value>(marker)).as_ref()!=Some(&value["instance"]){
                            response.request_focus();
                            ui.ctx().data_mut(|d| d.insert_temp(marker,value["instance"].clone()));
                        }
                        if response.changed() {
                            emit(events, id, "edit", json!(text));
                        }
                    }
                    if !text(value, "error").is_empty() {
                        ui.colored_label(ui.visuals().error_fg_color, text(value, "error"));
                    }
                    let mut choices = |ui: &mut Ui| {
                        for choice in value["choices"].as_array().into_iter().flatten() {
                            let response = ui.add_enabled(
                                choice["enabled"] == true,
                                egui::Button::new(text(choice, "label")),
                            );
                            geometry.insert(
                                format!("{id}.choice.{}", text(choice, "id")),
                                response.rect,
                            );
                            if response.clicked() {
                                emit(
                                    events,
                                    id,
                                    "answer",
                                    json!({"choice":choice["id"],"field":field,"instance":value["instance"]}),
                                );
                            }
                        }
                    };
                    if value["choice_layout"] == "list" {
                        egui::ScrollArea::vertical()
                            .id_salt((id, "choices"))
                            .max_height(350.0)
                            .show(ui, |ui| {
                                ui.vertical(&mut choices);
                            });
                    } else {
                        ui.horizontal(choices);
                    }
                });
            if modal.should_close() {
                if let Some(cancel) = value["choices"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .find(|c| c["cancel"] == true)
                {
                    emit(
                        events,
                        id,
                        "answer",
                        json!({"choice":cancel["id"],"field":field,"instance":value["instance"]}),
                    );
                }
            } else if ui.input(|i| {
                (value["multiline"] != true && i.key_pressed(egui::Key::Enter))
                    || (value["multiline"] == true
                        && i.modifiers.ctrl
                        && (i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::S)))
            }) {
                if let Some(default) = value["choices"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .find(|c| c["default"] == true && c["enabled"] == true)
                {
                    emit(
                        events,
                        id,
                        "answer",
                        json!({"choice":default["id"],"field":field,"instance":value["instance"]}),
                    );
                }
            }
            modal.response
        }
        _ => ui.colored_label(egui::Color32::RED, "Unsupported rich widget"),
    }
}
