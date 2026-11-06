// SPDX-License-Identifier: MIT OR Apache-2.0
//! Data-driven document tabs and search controls. Mix owns document policy.
use crate::widgets::Intent;
use egui::{Id, Ui};
use serde_json::{Value, json};

pub fn validate(kind: &str, v: &Value) -> Result<(), String> {
    let valid = match kind {
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
                            a["id"] == b["id"]
                                && a["label"] == b["label"]
                                && (a["read_only"] != true || a == b)
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
    let emit = |events: &mut Vec<Intent>, action: &str, value: Value| {
        events.push(Intent {
            id: id.into(),
            action: action.into(),
            value,
        })
    };
    if kind == "diagnostic-list" {
        if value.as_array().unwrap().is_empty() {
            return ui.weak("No problems");
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
        return toolkit::bars::options(ui, |ui| {
            egui::Grid::new(id).num_columns(2).show(ui, |ui| {
                for row in rows.as_array_mut().unwrap() {
                    ui.label(row["label"].as_str().unwrap());
                    let mut text = row["value"].as_str().unwrap().to_owned();
                    let response = ui.add_enabled(
                        row["read_only"] != true,
                        egui::TextEdit::singleline(&mut text).desired_width(ui.available_width()),
                    );
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
        let tabs = items
            .iter()
            .map(|i| toolkit::tabs::DocTab {
                name: i["name"].as_str().unwrap().into(),
                title: i["title"]
                    .as_str()
                    .unwrap_or(i["name"].as_str().unwrap())
                    .into(),
                meta: i["meta"].as_str().unwrap().into(),
                dirty: i["dirty"] == true,
                progress: (i["opening"] == true).then_some(0.0),
                marker: None,
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
                    ui.visuals().extreme_bg_color,
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
    toolkit::bars::options(ui, |ui| {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let field = toolkit::field::search(ui, &mut pattern, "Find", 220.0);
                let marker = Id::unique((id, "focus-instance"));
                if ui.ctx().data(|d| d.get_temp::<Value>(marker)).as_ref()
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
                    toolkit::field::text(ui, &mut replacement, "Replacement", 220.0);
                    for (action, label) in [("replace", "Replace"), ("replace-all", "Replace All")]
                    {
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
    })
    .response
}
