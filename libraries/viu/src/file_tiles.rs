// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native file tiles and transfer controls. Labels, eligibility, paths and
//! actions are supplied by the caller; this renderer never opens a file.
use crate::widgets::Intent;
use egui::{RichText, Sense, Ui};
use serde_json::Value;
use toolkit::{Icon, button::IconButton};

fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}
pub fn validate(v: &Value) -> Result<(), String> {
    if !v["expanded"].is_boolean()
        || !v["items"].as_array().is_some_and(|items| {
            items.len() <= 1024
                && items.iter().all(|p| {
                    ["key", "name", "size", "label", "icon"]
                        .iter()
                        .all(|key| p[key].as_str().is_some_and(|s| s.len() <= 4096))
                        && Icon::from_name(text(p, "icon")).is_some()
                        && p["enabled"].is_boolean()
                        && p["save_only"].is_boolean()
                        && (p["note"].is_null()
                            || p["note"].as_str().is_some_and(|s| s.len() <= 8192))
                })
        })
        || ![
            "title",
            "count",
            "size",
            "actions",
            "save_all",
            "open_all",
            "save",
            "open",
            "save_only",
            "undecodable",
        ]
        .iter()
        .all(|key| v[key].as_str().is_some_and(|s| s.len() <= 4096))
    {
        return Err("file tiles require bounded items, eligibility and labels".into());
    }
    Ok(())
}
fn emit(events: &mut Vec<Intent>, id: &str, action: &str, key: Value) {
    events.push(Intent {
        id: id.into(),
        action: action.into(),
        value: key,
    });
}
pub(crate) fn menu(
    response: &egui::Response,
    p: &Value,
    v: &Value,
    id: &str,
    events: &mut Vec<Intent>,
) {
    if p["enabled"] == true && response.clicked() {
        emit(
            events,
            id,
            if p["save_only"] == true {
                "save"
            } else {
                "open"
            },
            p["key"].clone(),
        );
    }
    response.context_menu(|ui| {
        let open = ui.add_enabled(
            p["enabled"] == true && p["save_only"] != true,
            egui::Button::new(text(v, "open")),
        );
        let open = if p["save_only"] == true {
            open.on_disabled_hover_text(text(v, "save_only"))
        } else {
            open
        };
        if open.clicked() {
            emit(events, id, "open", p["key"].clone());
            ui.close();
        }
        if ui
            .add_enabled(p["enabled"] == true, egui::Button::new(text(v, "save")))
            .clicked()
        {
            emit(events, id, "save", p["key"].clone());
            ui.close();
        }
    });
}
pub fn draw(ui: &mut Ui, id: &str, v: &Value, events: &mut Vec<Intent>) -> egui::Response {
    let limit = ui
        .ctx()
        .data(|d| {
            d.get_temp::<f32>(egui::Id::unique((
                ui.ctx().viewport_id(),
                "viu.dock.intrinsic-height",
            )))
        })
        .unwrap_or(ui.available_height());
    ui.scope(|ui| {
        let start_y = ui.cursor().top();
        toolkit::bars::hairline(ui);
        let items = v["items"].as_array().unwrap();
        let mut toggle = false;
        egui::Sides::new().show(
            ui,
            |ui| {
                ui.horizontal(|ui| {
                    if ui
                        .add(
                            IconButton::new(if v["expanded"] == true {
                                Icon::ChevronDown
                            } else {
                                Icon::ChevronRight
                            })
                            .tooltip(text(v, "title")),
                        )
                        .clicked()
                    {
                        toggle = true;
                    }
                    ui.add(toolkit::icons::image_of(
                        ui.ctx(),
                        Icon::Paperclip,
                        ui.text_style_height(&egui::TextStyle::Body),
                        ui.visuals().text_color(),
                    ));
                    ui.label(text(v, "count"));
                    ui.weak(text(v, "size"));
                });
            },
            |ui| {
                ui.horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    let more =
                        ui.add(IconButton::new(Icon::ChevronDown).tooltip(text(v, "actions")));
                    let any = items.iter().any(|p| p["enabled"] == true);
                    if ui
                        .add_enabled(any, egui::Button::new(text(v, "save_all")))
                        .clicked()
                    {
                        emit(events, id, "save-all", Value::Null);
                    }
                    egui::Popup::menu(&more).show(|ui| {
                        if ui
                            .add_enabled(any, egui::Button::new(text(v, "save_all")))
                            .clicked()
                        {
                            emit(events, id, "save-all", Value::Null);
                            ui.close();
                        }
                        if ui
                            .add_enabled(
                                items
                                    .iter()
                                    .any(|p| p["enabled"] == true && p["save_only"] != true),
                                egui::Button::new(text(v, "open_all")),
                            )
                            .clicked()
                        {
                            emit(events, id, "open-all", Value::Null);
                            ui.close();
                        }
                    });
                });
            },
        );
        if toggle {
            emit(events, id, "toggle", Value::Null);
        }
        if v["expanded"] == true {
            // The caller's limit covers the entire bar. Reserve its measured
            // controls and trailing space before sizing the tile scroll area.
            // Otherwise an exact bottom panel can clip those controls when
            // the tile list alone consumes the whole height budget.
            let trailing = ui.spacing().item_spacing.y * 2.0;
            let tiles_height = (limit - (ui.cursor().top() - start_y) - trailing).max(0.0);
            egui::ScrollArea::vertical()
                .id_salt("attachment-tiles")
                .max_height(tiles_height)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.with_layout(
                        egui::Layout::left_to_right(egui::Align::Min).with_main_wrap(true),
                        |ui| {
                            for p in items {
                                let body = ui.text_style_height(&egui::TextStyle::Body);
                                let frame = egui::Frame::group(ui.style());
                                let width = (egui::TextStyle::Body.resolve(ui.style()).size * 16.0)
                                    .min(ui.max_rect().width());
                                let shown = ui
                                    .allocate_ui_with_layout(
                                        egui::vec2(width, 0.0),
                                        egui::Layout::top_down(egui::Align::Min),
                                        |ui| {
                                            frame
                                                .show(ui, |ui| {
                                                    ui.set_width(
                                                        (width - frame.total_margin().sum().x)
                                                            .max(body),
                                                    );
                                                    ui.horizontal(|ui| {
                                                        let ink = if p["enabled"] == true {
                                                            ui.visuals().text_color()
                                                        } else {
                                                            ui.visuals().weak_text_color()
                                                        };
                                                        ui.add(toolkit::icons::image_of(
                                                            ui.ctx(),
                                                            Icon::from_name(text(p, "icon"))
                                                                .unwrap(),
                                                            body,
                                                            ink,
                                                        ));
                                                        ui.add(
                                                            egui::Label::new(
                                                                RichText::new(text(p, "name"))
                                                                    .color(ink),
                                                            )
                                                            .truncate(),
                                                        );
                                                    });
                                                    ui.weak(text(p, "size"));
                                                    if p["enabled"] != true {
                                                        ui.label(
                                                            RichText::new(text(v, "undecodable"))
                                                                .weak()
                                                                .small(),
                                                        );
                                                    }
                                                    if let Some(note) = p["note"].as_str() {
                                                        ui.add(
                                                            egui::Label::new(
                                                                RichText::new(note).weak().small(),
                                                            )
                                                            .wrap(),
                                                        );
                                                    }
                                                })
                                                .response
                                        },
                                    )
                                    .inner;
                                let response = ui
                                    .interact(shown.rect, shown.id.with("open"), Sense::click())
                                    .on_hover_text(text(p, "name"));
                                response.widget_info(|| {
                                    egui::WidgetInfo::labeled(
                                        egui::Role::Button,
                                        ui.is_enabled() && p["enabled"] == true,
                                        text(p, "label"),
                                    )
                                });
                                menu(&response, p, v, id, events);
                            }
                        },
                    );
                });
            ui.add_space(trailing);
        }
    })
    .response
}

pub fn intent(value: &Value, action: &str, payload: &Value) -> bool {
    match action {
        "toggle" | "save-all" | "open-all" => payload.is_null(),
        "open" | "save" => value["items"].as_array().is_some_and(|a| {
            a.iter().any(|p| {
                p["key"] == *payload
                    && p["enabled"] == true
                    && (action == "save" || p["save_only"] != true)
            })
        }),
        _ => false,
    }
}
