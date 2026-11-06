// SPDX-License-Identifier: MIT OR Apache-2.0
//! Data-authored resource chooser and blocks. Mix supplies every item, block,
//! label, range, size and selection; this widget only renders and emits intents.
use crate::widgets::Intent;
use egui::{Align, Layout, RichText, ScrollArea, Sense, Ui, Vec2};
use serde_json::{Value, json};
use toolkit::graph::{self, Hue};
fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}
fn series_values(v: &Value) -> Vec<f64> {
    v.as_array()
        .into_iter()
        .flatten()
        .map(|n| n.as_f64().unwrap_or(f64::NAN))
        .collect()
}
fn hue(v: &Value) -> Hue {
    match v.as_str() {
        Some("accent-plus") => Hue::AccentPlus,
        Some("accent-minus") => Hue::AccentMinus,
        Some("warning") => Hue::Warning,
        _ => v
            .as_u64()
            .map_or(Hue::Accent, |n| Hue::Series(n.min(255) as usize)),
    }
}
fn blocks_valid(blocks: &Value, depth: usize, budget: &mut usize) -> Result<(), String> {
    if depth > 8 {
        return Err("dashboard nesting exceeds eight".into());
    }
    for block in blocks.as_array().ok_or("dashboard blocks must be a list")? {
        *budget += 1;
        if *budget > 512 {
            return Err("dashboard block budget exceeded".into());
        }
        match text(block, "kind") {
            "chart" => crate::time_series::validate(&block["chart"])?,
            "columns" => blocks_valid(&block["blocks"], depth + 1, budget)?,
            "heading" | "small" | "gap" | "meter" => {}
            "figures" | "sparklines" | "meters" | "chips" => {
                if block["items"].as_array().is_none_or(|a| a.len() > 256) {
                    return Err("dashboard items exceed budget".into());
                }
            }
            "table" => crate::tree_table::validate(&block["table"])?,
            _ => return Err("unknown dashboard block".into()),
        }
    }
    Ok(())
}
pub fn validate(value: &Value) -> Result<(), String> {
    let items = value["items"]
        .as_array()
        .filter(|a| a.len() <= 256)
        .ok_or("dashboard requires at most 256 chooser items")?;
    let mut seen = std::collections::BTreeSet::new();
    for item in items {
        if text(item, "id").is_empty()
            || !seen.insert(text(item, "id"))
            || !item["title"].is_string()
        {
            return Err("dashboard item identities must be unique".into());
        }
    }
    if !items.is_empty() && !seen.contains(text(value, "selected")) {
        return Err("unknown dashboard selection".into());
    }
    blocks_valid(&value["blocks"], 0, &mut 0)
}
pub fn admit(value: &Value, action: &str, payload: &Value) -> Result<(), String> {
    if action != "choose"
        || !value["items"]
            .as_array()
            .into_iter()
            .flatten()
            .any(|i| i["id"] == *payload)
    {
        return Err("unknown dashboard choice".into());
    }
    Ok(())
}
fn choose(events: &mut Vec<Intent>, id: &str, key: &Value) {
    events.push(Intent {
        id: id.into(),
        action: "choose".into(),
        value: key.clone(),
    });
}
fn readout(unit: &str, name: &str, value: f64, age: u64) -> String {
    format!(
        "{name}: {} · {age} s ago",
        crate::time_series::format_value(unit, value)
    )
}
fn blocks(ui: &mut Ui, id: &str, items: &Value, interval: f64, events: &mut Vec<Intent>) {
    for (index, block) in items.as_array().unwrap().iter().enumerate() {
        let key = format!(
            "{id}.{}",
            block["id"]
                .as_str()
                .map(str::to_owned)
                .unwrap_or_else(|| index.to_string())
        );
        match text(block, "kind") {
            "heading" => {
                ui.horizontal(|ui| {
                    ui.heading(text(block, "title"));
                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                        ui.weak(text(block, "detail"));
                    });
                });
            }
            "small" => {
                ui.small(text(block, "text"));
            }
            "chips" => {
                ui.horizontal_wrapped(|ui| {
                    for item in block["items"].as_array().unwrap() {
                        ui.small(item.as_str().unwrap_or(""));
                    }
                });
            }
            "gap" => ui.add_space(block["size"].as_f64().unwrap_or(8.0).clamp(0.0, 100.0) as f32),
            "chart" => {
                crate::time_series::draw(ui, &key, &block["chart"]);
            }
            "columns" => {
                let children = block["blocks"].as_array().unwrap();
                if !children.is_empty() {
                    ui.columns(children.len(), |columns| {
                        for (i, child) in children.iter().enumerate() {
                            blocks(
                                &mut columns[i],
                                &format!("{key}.{i}"),
                                &json!([child]),
                                interval,
                                events,
                            );
                        }
                    });
                }
            }
            "figures" => {
                let columns = block["columns"].as_u64().unwrap_or(2).clamp(1, 16) as usize;
                egui::Grid::new(&key)
                    .num_columns(columns * 2)
                    .spacing([18.0, 6.0])
                    .show(ui, |ui| {
                        for (i, pair) in block["items"].as_array().unwrap().iter().enumerate() {
                            ui.weak(text(pair, "label"));
                            ui.label(RichText::new(text(pair, "value")).strong());
                            if (i + 1) % columns == 0 {
                                ui.end_row();
                            }
                        }
                    });
            }
            "sparklines" => {
                let items = block["items"].as_array().unwrap();
                let n = items.len();
                if n > 0 {
                    let width = ui.available_width();
                    let columns = ((n as f32).sqrt().ceil() as usize)
                        .clamp(1, 16)
                        .max((width / 160.0) as usize)
                        .min(n);
                    let cell = Vec2::new(
                        width / columns as f32 - 6.0,
                        block["height"].as_f64().unwrap_or(44.0) as f32,
                    );
                    egui::Grid::new(&key).spacing([6.0, 6.0]).show(ui, |ui| {
                        for (i, item) in items.iter().enumerate() {
                            let values = series_values(&item["values"]);
                            let format = |_, v, age| {
                                readout(text(item, "format"), text(item, "name"), v, age)
                            };
                            graph::line(
                                ui,
                                cell,
                                &values,
                                item["peak"].as_f64().unwrap_or(100.0),
                                hue(&item["hue"]),
                                Some(graph::Readout {
                                    samples: 60,
                                    interval_s: interval,
                                    format: &format,
                                }),
                            );
                            if (i + 1) % columns == 0 {
                                ui.end_row();
                            }
                        }
                    });
                }
            }
            "meter" => {
                graph::meter(
                    ui,
                    ui.available_width(),
                    block["share"].as_f64().unwrap_or(0.0),
                    hue(&block["hue"]),
                );
            }
            "meters" => {
                for item in block["items"].as_array().unwrap() {
                    ui.horizontal(|ui| {
                        ui.label(text(item, "label"));
                        ui.weak(text(item, "detail"));
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(text(item, "caption"));
                            let width = item["width"].as_f64().unwrap_or(160.0) as f32;
                            graph::meter(
                                ui,
                                width.min(ui.available_width()),
                                item["share"].as_f64().unwrap_or(0.0),
                                hue(&item["hue"]),
                            );
                        });
                    });
                }
            }
            "table" => {
                let widget =
                    crate::widgets::Widget::new(&key, "tree-table", "", block["table"].clone())
                        .with_properties(
                            json!({"style":"grid","vscroll":false,"auto_shrink":true}),
                        );
                crate::tree_table::draw(
                    ui,
                    &widget,
                    &block["table"],
                    &mut std::collections::BTreeMap::new(),
                    events,
                );
            }
            _ => unreachable!("validated dashboard block"),
        }
    }
}
pub(crate) fn draw(
    ui: &mut Ui,
    id: &str,
    value: &Value,
    events: &mut Vec<Intent>,
) -> egui::Response {
    let items = value["items"].as_array().unwrap();
    let interval = value["interval_s"].as_f64().unwrap_or(1.0);
    ui.scope(|ui| {
        if ui.available_width() < value["compact_width"].as_f64().unwrap_or(640.0) as f32 {
            egui::Panel::top(egui::Id::unique((id, "compact"))).show(ui, |ui| {
                let selected = items.iter().find(|i| i["id"] == value["selected"]);
                egui::ComboBox::from_id_salt(id)
                    .width(ui.available_width())
                    .selected_text(selected.map_or("", |i| text(i, "title")))
                    .show_ui(ui, |ui| {
                        for item in items {
                            if ui
                                .selectable_label(
                                    item["id"] == value["selected"],
                                    text(item, "title"),
                                )
                                .clicked()
                            {
                                choose(events, id, &item["id"]);
                            }
                        }
                    });
            });
        } else {
            egui::Panel::left(egui::Id::unique((id, "chooser")))
                .resizable(false)
                .default_size(value["sidebar_width"].as_f64().unwrap_or(250.0) as f32)
                .frame(toolkit::bars::dock_frame(ui.ctx()))
                .show(ui, |ui| {
                    ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                        for item in items {
                            ui.horizontal(|ui| {
                                let values = series_values(&item["values"]);
                                let second = series_values(&item["second_values"]);
                                let series = if second.is_empty() {
                                    vec![(values.as_slice(), hue(&item["hue"]))]
                                } else {
                                    vec![
                                        (values.as_slice(), hue(&item["hue"])),
                                        (second.as_slice(), Hue::Series(2)),
                                    ]
                                };
                                let format = |_, v, age| readout(text(item, "format"), "", v, age);
                                graph::lines(
                                    ui,
                                    Vec2::new(64.0, 36.0),
                                    &series,
                                    item["peak"].as_f64().unwrap_or(100.0),
                                    Some(graph::Readout {
                                        samples: 60,
                                        interval_s: interval,
                                        format: &format,
                                    }),
                                );
                                ui.vertical(|ui| {
                                    let title = toolkit::selectable(
                                        ui,
                                        item["id"] == value["selected"],
                                        RichText::new(text(item, "title")).strong(),
                                    )
                                    .clicked();
                                    let detail = ui
                                        .add(
                                            egui::Label::new(
                                                RichText::new(text(item, "subtitle")).small(),
                                            )
                                            .sense(Sense::click()),
                                        )
                                        .clicked();
                                    if title || detail {
                                        choose(events, id, &item["id"]);
                                    }
                                });
                            });
                            ui.add_space(6.0);
                        }
                    });
                });
        }
        egui::CentralPanel::default()
            .frame(toolkit::bars::dock_frame(ui.ctx()))
            .show(ui, |ui| {
                ScrollArea::vertical()
                    .auto_shrink(false)
                    .show(ui, |ui| blocks(ui, id, &value["blocks"], interval, events));
            });
    })
    .response
}
