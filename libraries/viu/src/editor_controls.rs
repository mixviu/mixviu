// SPDX-License-Identifier: MIT OR Apache-2.0
//! Data-driven document tabs and search controls. Mix owns document policy.
use crate::widgets::Intent;
use egui::{Id, Ui};
use serde_json::{Value, json};

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn command_search_refuses_disabled_forged_and_closed_commands() {
        let open = json!({"open":true,"items":[{"id":"copy","label":"Copy","enabled":true},{"id":"close","label":"Close","enabled":false}]});
        assert!(validate("command-search", &open).is_ok());
        assert!(
            admit(
                "command-search",
                &open,
                "command",
                &json!({"command":"copy"})
            )
            .is_ok()
        );
        for command in ["close", "forged"] {
            assert!(
                admit(
                    "command-search",
                    &open,
                    "command",
                    &json!({"command":command})
                )
                .is_err()
            );
        }
        let mut closed = open;
        closed["open"] = json!(false);
        assert!(
            admit(
                "command-search",
                &closed,
                "command",
                &json!({"command":"copy"})
            )
            .is_err()
        );
    }
}

pub fn validate(kind: &str, v: &Value) -> Result<(), String> {
    let valid = match kind {
        "command-search" => {
            v["open"].is_boolean()
                && v["items"].as_array().is_some_and(|items| {
                    items.len() <= 256
                        && items.iter().all(|i| {
                            i["id"].is_string()
                                && i["label"].is_string()
                                && i["enabled"].is_boolean()
                        })
                })
        }
        "notice-bar" => v.as_array().is_some_and(|rows| {
            rows.len() <= 128
                && rows.iter().all(|row| {
                    row["label"].as_str().is_some_and(|s| s.len() <= 65536)
                        && row["choices"].as_array().is_some_and(|choices| {
                            choices.len() <= 8
                                && choices.iter().all(|choice| {
                                    choice["id"].is_string()
                                        && choice["label"].is_string()
                                        && choice["enabled"].is_boolean()
                                })
                        })
                })
        }),
        "header-form" => crate::header_controls::validate(v),
        "diagnostic-list" => v.as_array().is_some_and(|rows| {
            rows.len() <= 500
                && rows.iter().all(|r| {
                    crate::widgets::whole(&r["line"]).is_some_and(|n| n > 0)
                        && r["message"].is_string()
                        && r["code"].is_string()
                })
        }),
        "macro-menu" => {
            v["open"].is_boolean()
                && v["app"].as_str().is_some_and(|a| {
                    !a.is_empty()
                        && a.len() <= 64
                        && a.bytes()
                            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
                })
        }
        "form-fields" => v.as_array().is_some_and(|rows| {
            rows.len() <= 32
                && rows.iter().all(|r| {
                    r["id"].is_string()
                        && r["label"].is_string()
                        && r["value"].as_str().is_some_and(|s| s.len() <= 65536)
                        && match r["type"].as_str() {
                            None | Some("text" | "label") => true,
                            Some("checkbox") => {
                                matches!(r["value"].as_str(), Some("true" | "false"))
                            }
                            Some("select") => r["options"].as_array().is_some_and(|options| {
                                options.len() <= 256
                                    && options
                                        .iter()
                                        .all(|o| o["id"].is_string() && o["label"].is_string())
                                    && options.iter().any(|o| o["id"] == r["value"])
                            }),
                            _ => false,
                        }
                })
        }),
        "document-tabs" => {
            v["items"].as_array().is_some_and(|items| {
                items.len() <= 1024
                    && items.iter().all(|i| {
                        i["id"].is_string()
                            && i["name"].is_string()
                            && i["meta"].is_string()
                            && i["dirty"].is_boolean()
                    })
            }) && v["selected"].is_string()
        }
        "search-bar" => {
            ["pattern", "replacement"]
                .iter()
                .all(|k| v[k].as_str().is_some_and(|s| s.len() <= 65536))
                && ["replace", "regex", "case"]
                    .iter()
                    .all(|k| v[k].is_boolean())
        }
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err("invalid editor controls".into())
    }
}
pub fn admit(kind: &str, current: &Value, action: &str, payload: &Value) -> Result<(), String> {
    if kind == "command-search" {
        return if current["open"] == true
            && ((action == "close" && payload.is_null())
                || (action == "command"
                    && current["items"].as_array().is_some_and(|items| {
                        items
                            .iter()
                            .any(|i| i["id"] == payload["command"] && i["enabled"] == true)
                    })))
        {
            Ok(())
        } else {
            Err("invalid search command".into())
        };
    }
    if kind == "notice-bar" {
        return if action == "command"
            && current.as_array().is_some_and(|rows| {
                rows.iter().any(|row| {
                    row["choices"].as_array().is_some_and(|choices| {
                        choices.iter().any(|choice| {
                            choice["id"] == payload["command"] && choice["enabled"] == true
                        })
                    })
                })
            }) {
            Ok(())
        } else {
            Err("invalid notice choice".into())
        };
    }
    if kind == "header-form" {
        return crate::header_controls::admit(current, action, payload);
    }
    if kind == "diagnostic-list" {
        return if action == "goto"
            && current
                .as_array()
                .is_some_and(|rows| rows.iter().any(|r| r == payload))
        {
            Ok(())
        } else {
            Err("invalid problem selection".into())
        };
    }
    if kind == "macro-menu" {
        return if action == "close" && payload.is_null() {
            Ok(())
        } else {
            Err("invalid macro menu intent".into())
        };
    }
    if kind == "form-fields" {
        return if action == "edit"
            && validate(kind, payload).is_ok()
            && current
                .as_array()
                .zip(payload.as_array())
                .is_some_and(|(a, b)| {
                    a.len() == b.len()
                        && a.iter().zip(b).all(|(a, b)| {
                            let mut expected = a.clone();
                            expected["value"] = b["value"].clone();
                            expected == *b && (a["read_only"] != true || a == b)
                        })
                })
        {
            Ok(())
        } else {
            Err("invalid field edit".into())
        };
    }
    let valid = if kind == "document-tabs" {
        ["select", "close"].contains(&action)
            && current["items"]
                .as_array()
                .is_some_and(|items| items.iter().any(|i| i["id"] == *payload))
    } else {
        (action == "edit" && validate(kind, payload).is_ok())
            || ["next", "prev", "replace", "replace-all", "close"].contains(&action)
    };
    if valid {
        Ok(())
    } else {
        Err("invalid editor control intent".into())
    }
}
pub fn draw(
    ui: &mut Ui,
    id: &str,
    kind: &str,
    value: &Value,
    events: &mut Vec<Intent>,
) -> egui::Response {
    if kind == "header-form" {
        return crate::header_controls::draw(ui, id, value, events);
    }
    let emit = |events: &mut Vec<Intent>, action: &str, value: Value| {
        events.push(Intent {
            id: id.into(),
            action: action.into(),
            value,
        })
    };
    if kind == "command-search" {
        let key = Id::unique((ui.ctx().viewport_id(), id, "query"));
        if value["open"] != true {
            ui.ctx().data_mut(|d| d.remove::<String>(key));
            return ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover());
        }
        let previous = ui.ctx().data(|d| d.get_temp::<String>(key));
        let first = previous.is_none();
        let mut query = previous.unwrap_or_default();
        let mut chosen = None;
        let response = toolkit::dialog::Dialog::new(key, "Search menus").show(ui.ctx(), |ui| {
            let field =
                toolkit::field::search(ui, &mut query, "Search menus", ui.available_width());
            if first {
                field.request_focus();
            }
            egui::ScrollArea::vertical().show(ui, |ui| {
                for item in value["items"].as_array().unwrap() {
                    let label = item["label"].as_str().unwrap();
                    if item["enabled"] == true
                        && item["id"] != "view.search"
                        && label.to_lowercase().contains(&query.to_lowercase())
                        && ui.button(label).clicked()
                    {
                        chosen = Some(item["id"].clone());
                    }
                }
            });
        });
        ui.ctx().data_mut(|d| d.insert_temp(key, query));
        if let Some(command) = chosen {
            emit(
                events,
                "command",
                json!({"command":command,"origin":"palette"}),
            );
        } else if response.dismissed {
            emit(events, "close", Value::Null);
        }
        return ui.interact(response.rect, key.with("receipt"), egui::Sense::hover());
    }
    if kind == "notice-bar" {
        return toolkit::bars::options(ui, |ui| {
            ui.vertical(|ui| {
                for row in value.as_array().unwrap() {
                    ui.horizontal(|ui| {
                        let icon = if row["icon"] == "unplug" {
                            crate::icons::Icon::Unplug
                        } else {
                            crate::icons::Icon::Info
                        };
                        ui.add(crate::icons::image_of(
                            ui.ctx(),
                            icon,
                            14.0,
                            ui.visuals().warn_fg_color,
                        ));
                        ui.label(row["label"].as_str().unwrap());
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            for choice in row["choices"].as_array().unwrap().iter().rev() {
                                if ui
                                    .add_enabled(
                                        choice["enabled"] == true,
                                        egui::Button::new(choice["label"].as_str().unwrap())
                                            .small(),
                                    )
                                    .clicked()
                                {
                                    emit(events, "command", json!({"command":choice["id"]}));
                                }
                            }
                        });
                    });
                }
            });
        })
        .response;
    }
    if kind == "diagnostic-list" {
        if value.as_array().unwrap().is_empty() {
            return ui.weak("No problems.");
        }
        return ui
            .scope(|ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for row in value.as_array().unwrap() {
                        let colour = match row["severity"].as_str() {
                            Some("error") => ui.visuals().error_fg_color,
                            Some("warning") => ui.visuals().warn_fg_color,
                            _ => ui.visuals().weak_text_color(),
                        };
                        let text = format!(
                            "{}:{}  {}  {}",
                            row["line"],
                            crate::widgets::whole(&row["col"]).unwrap_or(1),
                            row["code"].as_str().unwrap(),
                            row["message"].as_str().unwrap()
                        );
                        if ui
                            .add(
                                egui::Label::new(egui::RichText::new(text).color(colour))
                                    .sense(egui::Sense::click()),
                            )
                            .clicked()
                        {
                            emit(events, "goto", row.clone());
                        }
                    }
                });
            })
            .response;
    }
    if kind == "macro-menu" {
        if toolkit::macros::show_app(
            ui.ctx(),
            value["app"].as_str().unwrap(),
            value["open"] == true,
        ) {
            emit(events, "close", Value::Null);
        }
        return ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover());
    }
    if kind == "form-fields" {
        let mut rows = value.clone();
        let mut changed = false;
        if rows
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["type"].is_string())
        {
            let response = ui
                .vertical(|ui| {
                    for row in rows.as_array_mut().unwrap() {
                        let mut text = row["value"].as_str().unwrap().to_owned();
                        ui.push_id(row["id"].as_str().unwrap(), |ui| {
                            ui.add_enabled_ui(
                                row["read_only"] != true || row["type"] == "label",
                                |ui| match row["type"].as_str() {
                                    Some("label") => {
                                        ui.label(&text);
                                    }
                                    Some("checkbox") => {
                                        let mut checked = text == "true";
                                        changed |= ui
                                            .checkbox(&mut checked, row["label"].as_str().unwrap())
                                            .changed();
                                        text = checked.to_string();
                                    }
                                    Some("select") => {
                                        ui.horizontal(|ui| {
                                            if row["inline_label"] != false {
                                                ui.label(row["label"].as_str().unwrap());
                                            }
                                            let selected = row["options"]
                                                .as_array()
                                                .unwrap()
                                                .iter()
                                                .find(|o| o["id"] == text)
                                                .unwrap()["label"]
                                                .as_str()
                                                .unwrap();
                                            egui::ComboBox::from_id_salt("select")
                                                .selected_text(selected)
                                                .show_ui(ui, |ui| {
                                                    for option in row["options"].as_array().unwrap()
                                                    {
                                                        if toolkit::selectable(
                                                            ui,
                                                            option["id"] == text,
                                                            option["label"].as_str().unwrap(),
                                                        )
                                                        .clicked()
                                                        {
                                                            text = option["id"]
                                                                .as_str()
                                                                .unwrap()
                                                                .to_owned();
                                                            changed = true;
                                                        }
                                                    }
                                                })
                                                .response
                                                .accessible_name(row["label"].as_str().unwrap());
                                        });
                                    }
                                    _ => {
                                        ui.horizontal(|ui| {
                                            ui.label(row["label"].as_str().unwrap());
                                            changed |= ui
                                                .add(
                                                    egui::TextEdit::singleline(&mut text)
                                                        .desired_width(f32::INFINITY),
                                                )
                                                .changed();
                                        });
                                    }
                                },
                            );
                        });
                        row["value"] = json!(text);
                    }
                })
                .response;
            if changed {
                emit(events, "edit", rows);
            }
            return response;
        }
        return ui
            .scope(|ui| {
                egui::Grid::new(id).num_columns(2).show(ui, |ui| {
                    for row in rows.as_array_mut().unwrap() {
                        let labelled = ui.label(row["label"].as_str().unwrap()).id;
                        let mut text = row["value"].as_str().unwrap().to_owned();
                        let response = ui
                            .add_enabled(
                                row["read_only"] != true,
                                egui::TextEdit::singleline(&mut text)
                                    .desired_width(ui.available_width()),
                            )
                            .labelled_by(labelled);
                        let marker = Id::unique((id, row["id"].as_str().unwrap(), "focus"));
                        if row["focus"] == true
                            && ui.ctx().data(|d| d.get_temp::<Value>(marker)).as_ref()
                                != Some(&row["focus_instance"])
                        {
                            response.request_focus();
                            ui.ctx()
                                .data_mut(|d| d.insert_temp(marker, row["focus_instance"].clone()));
                        }
                        if response.has_focus() {
                            ui.ctx().data_mut(|d| {
                                d.insert_temp(egui::Id::unique("viu-focused-widget"), id.to_owned())
                            });
                        }
                        changed |= response.changed();
                        row["value"] = json!(text);
                        ui.end_row();
                    }
                });
                if changed {
                    emit(events, "edit", rows);
                }
            })
            .response;
    }
    if kind == "document-tabs" {
        let items = value["items"].as_array().expect("validated tabs");
        if items.is_empty() {
            return ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover());
        }
        let tabs = items
            .iter()
            .map(|i| {
                let name = i["name"].as_str().unwrap();
                let (marker, name) = if value["split_markers"] == true {
                    toolkit::tabs::split_marker(name)
                } else {
                    (None, name)
                };
                toolkit::tabs::DocTab {
                    name: name.into(),
                    title: i["title"]
                        .as_str()
                        .unwrap_or(i["name"].as_str().unwrap())
                        .into(),
                    meta: i["meta"].as_str().unwrap().into(),
                    dirty: i["dirty"] == true,
                    progress: (i["opening"] == true).then_some(0.0),
                    marker,
                }
            })
            .collect::<Vec<_>>();
        let selected = items
            .iter()
            .position(|i| i["id"] == value["selected"])
            .unwrap_or(0);
        return ui
            .scope(|ui| {
                let picked = toolkit::tabs::documents_on(
                    ui,
                    Id::unique(id),
                    &tabs,
                    selected,
                    None,
                    if value["body"] == "canvas" {
                        toolkit::chrome::Chrome::of(ui.ctx()).palette.canvas
                    } else if value["light"] == true {
                        editor::Palette::from_theme(toolkit::content::light_theme()).background
                    } else {
                        ui.ctx()
                            .data(|d| {
                                d.get_temp::<editor::Palette>(Id::unique("viu-gold-editor-palette"))
                            })
                            .map_or(ui.visuals().extreme_bg_color, |p| p.background)
                    },
                );
                if let Some(i) = picked.selected {
                    emit(events, "select", items[i]["id"].clone());
                }
                if let Some(i) = picked.closed {
                    emit(events, "close", items[i]["id"].clone());
                }
            })
            .response;
    }
    let mut v = value.clone();
    let mut pattern = v["pattern"].as_str().unwrap().to_owned();
    let mut replacement = v["replacement"].as_str().unwrap().to_owned();
    let mut regex = v["regex"] == true;
    let mut case = v["case"] == true;
    ui.spacing_mut().item_spacing.y = 0.0;
    let add = |ui: &mut Ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let field = toolkit::field::search(ui, &mut pattern, "Find", 220.0);
                let marker = Id::unique((id, "focus-instance"));
                if !ui.is_sizing_pass()
                    && ui.ctx().data(|d| d.get_temp::<Value>(marker)).as_ref()
                        != Some(&value["instance"])
                {
                    field.request_focus();
                    ui.ctx()
                        .data_mut(|d| d.insert_temp(marker, value["instance"].clone()));
                }
                if field.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    emit(events, "next", Value::Null);
                }
                for (action, label) in [("prev", "Previous"), ("next", "Next")] {
                    if ui.button(label).clicked() {
                        emit(events, action, Value::Null);
                    }
                }
                ui.checkbox(&mut regex, "Regex");
                ui.checkbox(&mut case, "Match case");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button("Close").clicked()
                        || ui.input(|i| i.key_pressed(egui::Key::Escape))
                    {
                        emit(events, "close", Value::Null);
                    }
                });
            });
            if value["replace"] == true {
                ui.horizontal(|ui| {
                    toolkit::field::text(ui, &mut replacement, "Replace with", 220.0);
                    for (action, label) in [("replace", "Replace"), ("replace-all", "All")] {
                        if ui.button(label).clicked() {
                            emit(events, action, Value::Null);
                        }
                    }
                });
            }
        });
        v["pattern"] = json!(pattern);
        v["replacement"] = json!(replacement);
        v["regex"] = json!(regex);
        v["case"] = json!(case);
        if &v != value {
            emit(events, "edit", v);
        }
    };
    if value["replace"] == true {
        // Fable C1 approved: the retained fixed-height gold options rail
        // clips its second row. Keep the same chrome and insets, but measure
        // both rows so Replace and All remain visible and reachable.
        let chrome = toolkit::chrome::Chrome::of(ui.ctx());
        egui::Frame::new()
            .fill(chrome.palette.chrome)
            .inner_margin(egui::Margin::symmetric(
                chrome.style.bars_bar_inset as i8,
                chrome.style.bar_padding_y as i8,
            ))
            .show(ui, add)
            .response
    } else {
        toolkit::bars::options(ui, add).response
    }
}
