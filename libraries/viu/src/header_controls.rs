// SPDX-License-Identifier: MIT OR Apache-2.0
//! Labelled header rows, optional row actions and attachment chips. The Mix
//! owner supplies visibility, values and commands; this control owns no mail.
use crate::widgets::Intent;
use egui::{Id, Ui};
use serde_json::{Value, json};
use std::sync::{Arc, Mutex};

pub fn validate(v: &Value) -> bool {
    v["enabled"].is_boolean()
        && v["fields"].as_array().is_some_and(|rows| {
            rows.len() <= 32
                && rows.iter().all(|r| {
                    r["id"].is_string()
                        && r["label"].is_string()
                        && r["value"].as_str().is_some_and(|s| s.len() <= 65536)
                        && r["visible"].is_boolean()
                        && r["actions"].as_array().is_some_and(|a| {
                            a.len() <= 8
                                && a.iter().all(|a| {
                                    a["id"].is_string()
                                        && a["label"].is_string()
                                        && (a["icon"].is_null()
                                            || a["icon"]
                                                .as_str()
                                                .and_then(crate::icons::Icon::from_name)
                                                .is_some())
                                })
                        })
                })
        })
        && v["attachments"].as_array().is_some_and(|a| {
            a.len() <= 1024
                && a.iter().all(|p| {
                    p.as_str().is_some_and(|s| s.len() <= 65536)
                        || (p["path"].is_string() && p["text"].is_string())
                })
        })
        && v["attachment_label"].is_string()
        && v["attach_label"].is_string()
        && v["remove_label"].is_string()
        && v["chooser"]["open"].is_boolean()
        && v["chooser"]["title"].is_string()
        && v["chooser"]["directory"].is_string()
}
pub fn admit(current: &Value, action: &str, payload: &Value) -> Result<(), String> {
    let valid = current["enabled"] == true
        && match action {
            "edit" => payload.as_object().is_some_and(|m| {
                !m.is_empty()
                    && m.iter().all(|(key, v)| {
                        v.as_str().is_some_and(|s| s.len() <= 65536)
                            && current["fields"].as_array().unwrap().iter().any(|r| {
                                r["id"] == *key && r["read_only"] != true && r["visible"] == true
                            })
                    })
            }),
            "commit" => {
                payload.is_null()
                    || payload.as_str().is_some_and(|key| {
                        current["fields"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .any(|r| r["id"] == key)
                    })
            }
            "complete" => payload["field"].as_str().is_some_and(|key| {
                current["fields"].as_array().unwrap().iter().any(|r| {
                    r["id"] == key
                        && crate::widgets::whole(&payload["index"]).is_some_and(|i| {
                            r["suggestions"]
                                .as_array()
                                .is_some_and(|a| i < (a.len() as u64))
                        })
                })
            }),
            "completion_move" | "completion_close" => {
                payload["field"].as_str().is_some_and(|key| {
                    current["fields"].as_array().unwrap().iter().any(|r| {
                        r["id"] == key && r["suggestions"].as_array().is_some_and(|a| !a.is_empty())
                    })
                })
            }
            "command" => {
                payload == "attach"
                    || payload.as_str().is_some_and(|key| {
                        current["fields"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .flat_map(|r| r["actions"].as_array().unwrap())
                            .any(|a| a["id"] == key)
                    })
            }
            "remove" => payload
                .as_u64()
                .is_some_and(|i| i < current["attachments"].as_array().unwrap().len() as u64),
            "picked" => {
                current["chooser"]["open"] == true
                    && payload.as_array().is_some_and(|a| {
                        a.len() <= 1024
                            && a.iter()
                                .all(|p| p.as_str().is_some_and(|s| s.len() <= 65536))
                    })
            }
            "saved" => {
                current["chooser"]["open"] == true
                    && current["chooser"]["mode"] == "save"
                    && payload["path"].as_str().is_some_and(|p| p.starts_with('/'))
                    && payload["replace"].is_boolean()
            }
            "cancel" => current["chooser"]["open"] == true && payload.is_null(),
            _ => false,
        };
    if valid {
        Ok(())
    } else {
        Err("invalid header form intent".into())
    }
}
pub fn draw(ui: &mut Ui, id: &str, v: &Value, events: &mut Vec<Intent>) -> egui::Response {
    use toolkit::header_form as form;
    let emit = |events: &mut Vec<Intent>, action: &str, value| {
        events.push(Intent {
            id: id.into(),
            action: action.into(),
            value,
        })
    };
    let response = ui
        .scope(|ui| {
            if v["native_spacing"] != true {
                ui.spacing_mut().item_spacing.y = 0.0;
            }
            let column = form::column(
                ui,
                v["fields"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|r| r["label"].as_str().unwrap().to_owned())
                    .chain([v["attachment_label"].as_str().unwrap().to_owned()]),
            );
            let mut edits = serde_json::Map::new();
            let mut commits = Vec::new();
            ui.add_enabled_ui(v["enabled"] == true, |ui| {
                for r in v["fields"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|r| r["visible"] == true)
                {
                    let key = r["id"].as_str().unwrap();
                    let mut text = r["value"].as_str().unwrap().to_owned();
                    let mut field_events = Vec::new();
                    let height = if r["kind"] == "label" || r["choices"].is_array() {
                        toolkit::chrome::Chrome::of(ui.ctx())
                            .metrics
                            .push_height
                            .max(toolkit::field::height_of(ui.ctx()))
                    } else {
                        toolkit::field::height_of(ui.ctx())
                    };
                    form::row_of(
                        ui,
                        r["label"].as_str().unwrap(),
                        column,
                        height,
                        |ui, labelled| {
                            let mut field = |ui: &mut Ui| {
                                if r["kind"] == "label" {
                                    ui.add(egui::Label::new(&text).truncate())
                                        .labelled_by(labelled);
                                    return;
                                }
                                if let Some(choices) = r["choices"].as_array() {
                                    let names = choices
                                        .iter()
                                        .filter_map(Value::as_str)
                                        .map(str::to_owned)
                                        .collect::<Vec<_>>();
                                    let mut at = names.iter().position(|n| *n == text).unwrap_or(0);
                                    if toolkit::combo::show(
                                        ui,
                                        (id, key),
                                        &mut at,
                                        &names,
                                        Some(ui.available_width()),
                                    )
                                    .labelled_by(labelled)
                                    .changed()
                                    {
                                        edits.insert(key.to_owned(), json!(names[at]));
                                    }
                                    return;
                                }
                                let field_id = Id::unique((id, &v["identity"], key));
                                let suggestions =
                                    r["suggestions"].as_array().filter(|s| !s.is_empty());
                                if suggestions.is_some() && ui.memory(|m| m.has_focus(field_id)) {
                                    ui.input_mut(|i| {
                                        if i.consume_key(
                                            egui::Modifiers::NONE,
                                            egui::Key::ArrowDown,
                                        ) {
                                            emit(
                                                &mut field_events,
                                                "completion_move",
                                                json!({"field":key,"delta":1}),
                                            );
                                        }
                                        if i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp)
                                        {
                                            emit(
                                                &mut field_events,
                                                "completion_move",
                                                json!({"field":key,"delta":-1}),
                                            );
                                        }
                                        if i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                                            || i.consume_key(egui::Modifiers::NONE, egui::Key::Tab)
                                        {
                                            emit(
                                                &mut field_events,
                                                "complete",
                                                json!({"field":key,"index":r["cursor"]}),
                                            );
                                        }
                                        if i.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                                            emit(
                                                &mut field_events,
                                                "completion_close",
                                                json!({"field":key}),
                                            );
                                        }
                                    });
                                }
                                let response = ui
                                    .add_enabled_ui(r["read_only"] != true, |ui| {
                                        form::text(
                                            ui,
                                            field_id,
                                            labelled,
                                            &mut text,
                                            ui.available_width(),
                                        )
                                    })
                                    .inner;
                                if response.has_focus() {
                                    if let Some(suggestions) = suggestions {
                                        egui::Area::new(response.id.with("suggestions"))
                                            .order(egui::Order::Foreground)
                                            .fixed_pos(response.rect.left_bottom())
                                            .show(ui.ctx(), |ui| {
                                                egui::Frame::popup(ui.style()).show(ui, |ui| {
                                                    ui.set_width(response.rect.width());
                                                    let body =
                                                        egui::TextStyle::Body.resolve(ui.style());
                                                    let bold = toolkit::fonts::bound(
                                                        ui.ctx(),
                                                        toolkit::fonts::heading(body.size),
                                                    );
                                                    for (index, item) in
                                                        suggestions.iter().enumerate()
                                                    {
                                                        let mut job =
                                                            egui::text::LayoutJob::default();
                                                        if let Some(name) = item["name"]
                                                            .as_str()
                                                            .filter(|s| !s.is_empty())
                                                        {
                                                            append_match(
                                                                &mut job,
                                                                name,
                                                                &bold,
                                                                ui.visuals().strong_text_color(),
                                                                ui.visuals()
                                                                    .selection
                                                                    .bg_fill
                                                                    .gamma_multiply(0.5),
                                                                r["query"].as_str().unwrap_or(""),
                                                            );
                                                            job.append(
                                                                "  ",
                                                                0.0,
                                                                egui::TextFormat::simple(
                                                                    body.clone(),
                                                                    ui.visuals().weak_text_color(),
                                                                ),
                                                            );
                                                        }
                                                        append_match(
                                                            &mut job,
                                                            item["email"].as_str().unwrap_or(""),
                                                            &body,
                                                            ui.visuals().weak_text_color(),
                                                            ui.visuals()
                                                                .selection
                                                                .bg_fill
                                                                .gamma_multiply(0.5),
                                                            r["query"].as_str().unwrap_or(""),
                                                        );
                                                        if toolkit::selectable(
                                                            ui,
                                                            crate::widgets::whole(&r["cursor"])
                                                                == Some(index as u64),
                                                            job,
                                                        )
                                                        .clicked()
                                                        {
                                                            emit(
                                                                &mut field_events,
                                                                "complete",
                                                                json!({"field":key,"index":index}),
                                                            );
                                                        }
                                                    }
                                                });
                                            });
                                    }
                                }
                                let marker = Id::unique((id, key, "focus"));
                                if r["focus"] == true
                                    && ui.ctx().data(|d| d.get_temp::<Value>(marker)).as_ref()
                                        != Some(&v["focus_instance"])
                                {
                                    response.request_focus();
                                    ui.ctx().data_mut(|d| {
                                        d.insert_temp(marker, v["focus_instance"].clone())
                                    });
                                }
                                if response.has_focus() {
                                    ui.ctx().data_mut(|d| {
                                        d.insert_temp(
                                            Id::unique("viu-focused-widget"),
                                            id.to_owned(),
                                        )
                                    });
                                }
                                if response.changed() {
                                    edits.insert(key.to_owned(), json!(text));
                                }
                                if response.lost_focus() {
                                    commits.push(json!(key));
                                }
                            };
                            if r["actions"].as_array().unwrap().is_empty() {
                                field(ui);
                            } else {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.spacing_mut().item_spacing.x = 2.0;
                                        for a in r["actions"].as_array().unwrap().iter().rev() {
                                            let response = if let Some(icon) = a["icon"]
                                                .as_str()
                                                .and_then(crate::icons::Icon::from_name)
                                            {
                                                ui.add(
                                                    toolkit::button::IconButton::new(icon)
                                                        .size(toolkit::field::height_of(ui.ctx()))
                                                        .selected(a["selected"] == true)
                                                        .tooltip(a["label"].as_str().unwrap()),
                                                )
                                            } else {
                                                ui.add(toolkit::button::PushButton::secondary(
                                                    a["label"].as_str().unwrap(),
                                                ))
                                            };
                                            if response.clicked() {
                                                emit(events, "command", a["id"].clone());
                                            }
                                        }
                                        ui.add_space(form::GAP - 2.0);
                                        ui.with_layout(
                                            egui::Layout::left_to_right(egui::Align::Center),
                                            |ui| {
                                                field(ui);
                                            },
                                        );
                                    },
                                );
                            }
                        },
                    );
                    events.extend(field_events);
                }
                form::row(
                    ui,
                    v["attachment_label"].as_str().unwrap(),
                    column,
                    |ui, _| {
                        ui.horizontal_wrapped(|ui| {
                            let attach = if let Some(icon) = v["attach_icon"]
                                .as_str()
                                .and_then(crate::icons::Icon::from_name)
                            {
                                ui.add(
                                    toolkit::button::IconButton::new(icon)
                                        .size(toolkit::field::height_of(ui.ctx()))
                                        .tooltip(v["attach_label"].as_str().unwrap()),
                                )
                            } else {
                                ui.button(v["attach_label"].as_str().unwrap())
                            };
                            if attach.clicked() {
                                emit(events, "command", json!("attach"));
                            }
                            for (i, p) in v["attachments"].as_array().unwrap().iter().enumerate() {
                                let path = p.as_str().or_else(|| p["path"].as_str()).unwrap();
                                let text = p.as_str().or_else(|| p["text"].as_str()).unwrap();
                                if form::attachment_chip(
                                    ui,
                                    i,
                                    std::path::Path::new(path),
                                    text,
                                    p["bad"] == true,
                                    toolkit::field::height_of(ui.ctx()),
                                    v["remove_label"].as_str().unwrap(),
                                ) {
                                    emit(events, "remove", json!(i));
                                }
                            }
                        });
                    },
                );
            });
            // Preview values always precede commands collected in this frame.
            if !edits.is_empty() {
                events.insert(
                    0,
                    Intent {
                        id: id.into(),
                        action: "edit".into(),
                        value: Value::Object(edits),
                    },
                );
            }
            for key in commits {
                emit(events, "commit", key);
            }
            if let Some(notice) = v["notice"].as_str().filter(|s| !s.is_empty()) {
                ui.weak(notice);
            }
        })
        .response;
    let chooser_id = Id::unique((id, "chooser", &v["identity"], &v["chooser"]["instance"]));
    if v["chooser"]["open"] == true {
        let chooser = ui.ctx().data_mut(|d| {
            d.get_temp_mut_or_insert_with(chooser_id, || {
                let start = v["chooser"]["directory"].as_str().unwrap();
                let title = v["chooser"]["title"].as_str().unwrap();
                let confirm = v["chooser"]["confirm"]
                    .as_str()
                    .unwrap_or(v["attach_label"].as_str().unwrap());
                Arc::new(Mutex::new(match v["chooser"]["mode"].as_str() {
                    Some("save") => toolkit::file_chooser::FileChooser::save(start, title, confirm),
                    Some("open") => {
                        toolkit::file_chooser::FileChooser::single(start, title, confirm)
                    }
                    _ => toolkit::file_chooser::FileChooser::new(start, title, confirm),
                }))
            })
            .clone()
        });
        let outcome = chooser.lock().unwrap().show(ui.ctx());
        if let Some(outcome) = outcome {
            match outcome {
                toolkit::file_chooser::Outcome::Picked(paths) => {
                    emit(events, "picked", json!(paths))
                }
                toolkit::file_chooser::Outcome::Save { path, replace } => {
                    emit(events, "saved", json!({"path":path,"replace":replace}))
                }
                _ => emit(events, "cancel", Value::Null),
            }
            ui.ctx().data_mut(|d| {
                d.remove::<Arc<Mutex<toolkit::file_chooser::FileChooser>>>(chooser_id)
            });
        }
    } else {
        ui.ctx()
            .data_mut(|d| d.remove::<Arc<Mutex<toolkit::file_chooser::FileChooser>>>(chooser_id));
    }
    response
}
fn append_match(
    job: &mut egui::text::LayoutJob,
    text: &str,
    font: &egui::FontId,
    colour: egui::Color32,
    mark: egui::Color32,
    query: &str,
) {
    let base = egui::TextFormat::simple(font.clone(), colour);
    let mut folded = String::new();
    let mut source = Vec::new();
    for (at, ch) in text.char_indices() {
        for lower in ch.to_lowercase() {
            folded.push(lower);
        }
        source.resize(folded.len(), at..at + ch.len_utf8());
    }
    let needle = query.to_lowercase();
    if let Some(at) = (!needle.is_empty()).then(|| folded.find(&needle)).flatten() {
        let range = source[at].start..source[at + needle.len() - 1].end;
        job.append(&text[..range.start], 0.0, base.clone());
        job.append(
            &text[range.clone()],
            0.0,
            egui::TextFormat {
                background: mark,
                ..base.clone()
            },
        );
        job.append(&text[range.end..], 0.0, base);
    } else {
        job.append(text, 0.0, base);
    }
}
