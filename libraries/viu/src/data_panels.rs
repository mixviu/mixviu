// SPDX-License-Identifier: MIT OR Apache-2.0
//! Data-authored text blocks and two-column forms. Owners supply labels,
//! choices, availability and outcomes; these controls only draw and emit intent.
use crate::widgets::{Intent, Widget};
use egui::{Rect, Ui, Vec2};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub const KINDS: &[&str] = &[
    "block-list",
    "form-grid",
    "cell-grid",
    "history-lines",
    "label-run",
];
fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}
pub fn validate(kind: &str, value: &Value) -> Result<(), String> {
    if kind == "history-lines" {
        if value["series"]
            .as_array()
            .is_none_or(|rows| rows.is_empty() || rows.iter().any(|r| !r["values"].is_array()))
        {
            return Err("history lines require series".into());
        }
        return Ok(());
    }
    let rows = value.as_array().ok_or("panel rows must be an array")?;
    if kind == "cell-grid" && rows.iter().any(|r| !r.is_array()) {
        return Err("cell grid requires row arrays".into());
    }
    if kind == "form-grid"
        && rows
            .iter()
            .any(|r| !r["id"].is_string() || !r["label"].is_string() || !r["value"].is_string())
    {
        return Err("form rows require id, label and value".into());
    }
    if kind == "block-list"
        && rows
            .iter()
            .any(|r| !r["id"].is_string() || !r["kind"].is_string())
    {
        return Err("blocks require id and kind".into());
    }
    Ok(())
}
pub fn admit(kind: &str, current: &Value, action: &str, value: &Value) -> Result<(), String> {
    validate(kind, current)?;
    if action == "set" {
        return validate(kind, value);
    }
    let rows = current.as_array().unwrap();
    if kind == "form-grid" && action == "edit" {
        let row = rows
            .iter()
            .find(|r| r["id"] == value["field"] && r["enabled"] != false)
            .ok_or("unknown or disabled form field")?;
        let val = value["value"].as_str().ok_or("field value must be text")?;
        if row["editable"] == true
            || row["choices"]
                .as_array()
                .is_some_and(|cs| cs.iter().any(|c| c["id"] == val))
        {
            return Ok(());
        }
    }
    if kind == "block-list"
        && action == "choose"
        && rows.iter().any(|r| {
            r["id"] == value["item"]
                && r["choices"].as_array().is_some_and(|cs| {
                    cs.iter().any(|c| {
                        c["enabled"] != false
                            && c["id"] == value["choice"]
                            && c["value"] == value["value"]
                    })
                })
        })
    {
        return Ok(());
    }
    Err("unknown or unavailable panel action".into())
}
fn styled(v: &Value, value: &str) -> egui::RichText {
    let mut r = egui::RichText::new(value);
    if v["weak"] == true {
        r = r.weak();
    }
    if v["small"] == true {
        r = r.small();
    }
    if v["strong"] == true {
        r = r.strong();
    }
    if v["italic"] == true {
        r = r.italics();
    }
    if v["mono"] == true {
        r = r.monospace();
    }
    r
}
fn block(
    ui: &mut Ui,
    parent: &str,
    v: &Value,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) {
    let key = (parent, text(v, "id"));
    ui.push_id(key,|ui| {
        if let Some(gap)=v["space"].as_f64() { ui.add_space(gap.max(0.) as f32); }
        match text(v,"kind") {
            "separator"=>{ui.separator();},
            "collapse"=>{
                let shown=egui::CollapsingHeader::new(styled(v,text(v,"title"))).id_salt(key).show(ui,|ui| {
                    if let Some(width) = v["body_max_width"].as_f64() { ui.set_max_width(ui.available_width().min(width as f32)); }
                    if !text(v,"subtitle").is_empty(){ui.label(egui::RichText::new(text(v,"subtitle")).small().weak());}
                    ui.add(egui::Label::new(styled(&v["body_style"],text(v,"text"))).wrap());
                });
                if shown.body_response.is_none() && !text(v,"subtitle").is_empty(){ui.label(egui::RichText::new(text(v,"subtitle")).small().weak());}
                geometry.insert(format!("{parent}.{}",text(v,"id")),shown.header_response.rect);
            },
            "choices"=>{
                ui.horizontal(|ui| {
                    for choice in v["choices"].as_array().into_iter().flatten() {
                        let button=if choice["primary"]==true {toolkit::button::PushButton::primary(text(choice,"label"))} else {toolkit::button::PushButton::secondary(text(choice,"label"))};
                        let r=ui.add_enabled(choice["enabled"]!=false,button);
                        geometry.insert(format!("{parent}.{}.{}",text(v,"id"),text(choice,"id")),r.rect);
                        if r.clicked() { events.push(Intent{id:parent.into(),action:"choose".into(),value:json!({"item":v["id"],"choice":choice["id"],"value":choice["value"]})}); }
                    }
                });
            },
            _=>{
                let content=styled(v,text(v,"text"));
                let r=if v["error"]==true {ui.colored_label(ui.visuals().error_fg_color,content)}else{ui.add(egui::Label::new(content).wrap())};
                geometry.insert(format!("{parent}.{}",text(v,"id")),r.rect);
            }
        }
    });
}
pub fn draw(
    ui: &mut Ui,
    widget: &Widget,
    value: &Value,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) -> egui::Response {
    let id = &widget.id;
    if widget.kind == "label-run" {
        return ui
            .horizontal_wrapped(|ui| {
                let palette = toolkit::chrome::Chrome::of(ui.ctx()).palette;
                for (index, row) in value.as_array().into_iter().flatten().enumerate() {
                    let label = styled(row, text(row, "text"));
                    let response = match text(row, "hue") {
                        "accent" => ui.colored_label(palette.accent, label),
                        "warning" => ui.colored_label(palette.warning, label),
                        "danger" => ui.colored_label(palette.danger, label),
                        "dim" => ui.colored_label(palette.text_dim, label),
                        _ => ui.label(label),
                    };
                    geometry.insert(format!("{id}.{index}"), response.rect);
                }
            })
            .response;
    }
    if widget.kind == "history-lines" {
        use toolkit::graph::{self, Hue, Readout};
        let raw = value["series"].as_array().unwrap();
        let data: Vec<Vec<f64>> = raw
            .iter()
            .map(|s| {
                s["values"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_f64().unwrap_or(0.))
                    .collect()
            })
            .collect();
        let series: Vec<_> = raw
            .iter()
            .zip(&data)
            .map(|(s, values)| {
                (
                    values.as_slice(),
                    if s["hue"] == "warning" {
                        Hue::Warning
                    } else {
                        Hue::Accent
                    },
                )
            })
            .collect();
        let format = |i: usize, n: f64, age: u64| {
            text(value, "readout")
                .replace("{series}", text(&raw[i], "name"))
                .replace("{value}", &format!("{n:.0}"))
                .replace("{age}", &age.to_string())
        };
        return graph::lines(
            ui,
            Vec2::new(
                ui.available_width(),
                ui.text_style_height(&egui::TextStyle::Body)
                    * value["height_lines"].as_f64().unwrap_or(4.) as f32,
            ),
            &series,
            value["max"].as_f64().unwrap_or(0.),
            Some(Readout {
                samples: value["samples"].as_u64().unwrap_or(300) as usize,
                interval_s: value["interval_s"].as_f64().unwrap_or(1.),
                format: &format,
            }),
        );
    }
    if widget.kind == "cell-grid" {
        let mut render = |ui: &mut Ui| {
            egui::Grid::new(id)
                .striped(widget.properties["striped"] == true)
                .show(ui, |ui| {
                    for (ri, row) in value.as_array().unwrap().iter().enumerate() {
                        for (ci, cell) in row.as_array().unwrap().iter().enumerate() {
                            if cell["collapse"].is_string() {
                                let r =
                                    egui::CollapsingHeader::new(styled(cell, text(cell, "text")))
                                        .id_salt((id, ri, ci))
                                        .show(ui, |ui| {
                                            ui.set_max_width(
                                                ui.available_width().min(
                                                    cell["max_width"].as_f64().unwrap_or(420.)
                                                        as f32,
                                                ),
                                            );
                                            ui.add(
                                                egui::Label::new(
                                                    egui::RichText::new(text(cell, "collapse"))
                                                        .monospace(),
                                                )
                                                .wrap(),
                                            );
                                        });
                                geometry.insert(format!("{id}.{ri}.{ci}"), r.header_response.rect);
                            } else {
                                let r = ui.label(styled(
                                    cell,
                                    cell.as_str().unwrap_or_else(|| text(cell, "text")),
                                ));
                                geometry.insert(format!("{id}.{ri}.{ci}"), r.rect);
                            }
                        }
                        ui.end_row();
                    }
                });
        };
        if widget.properties["scroll"] == true {
            let r = egui::ScrollArea::both()
                .id_salt(id)
                .auto_shrink(false)
                .stick_to_bottom(widget.properties["stick_to_bottom"] == true)
                .show(ui, &mut render)
                .inner_rect;
            return ui.interact(r, ui.make_persistent_id(id), egui::Sense::hover());
        }
        return ui.vertical(&mut render).response;
    }
    if widget.kind == "form-grid" {
        return egui::Grid::new((id, "form"))
            .num_columns(2)
            .show(ui, |ui| {
                for row in value.as_array().into_iter().flatten() {
                    ui.weak(text(row, "label"));
                    let mut val = text(row, "value").to_owned();
                    let r = if let Some(choices) = row["choices"].as_array() {
                        let label = choices
                            .iter()
                            .find(|c| c["id"] == val)
                            .map_or_else(
                                || {
                                    if val.is_empty() {
                                        text(row, "placeholder")
                                    } else {
                                        val.as_str()
                                    }
                                },
                                |c| text(c, "label"),
                            )
                            .to_owned();
                        ui.add_enabled_ui(row["enabled"] != false, |ui| {
                            egui::ComboBox::from_id_salt((id, text(row, "id")))
                                .selected_text(label)
                                .show_ui(ui, |ui| {
                                    for choice in choices {
                                        if toolkit::button::selectable(
                                            ui,
                                            val == text(choice, "id"),
                                            text(choice, "label"),
                                        )
                                        .clicked()
                                        {
                                            val = text(choice, "id").into();
                                        }
                                    }
                                })
                                .response
                        })
                        .inner
                    } else if row["editable"] == true {
                        ui.add_enabled(
                            row["enabled"] != false,
                            egui::TextEdit::singleline(&mut val)
                                .desired_width(f32::INFINITY)
                                .hint_text(text(row, "placeholder")),
                        )
                    } else {
                        ui.label(styled(row, &val))
                    };
                    geometry.insert(format!("{id}.{}", text(row, "id")), r.rect);
                    if val != text(row, "value") {
                        events.push(Intent {
                            id: id.clone(),
                            action: "edit".into(),
                            value: json!({"field":row["id"],"value":val}),
                        });
                    }
                    ui.end_row();
                }
            })
            .response;
    }
    let mut render = |ui: &mut Ui| {
        for row in value.as_array().into_iter().flatten() {
            block(ui, id, row, geometry, events);
        }
    };
    if widget.properties["scroll"] == false {
        return ui.vertical(&mut render).response;
    }
    if let Some(space) = widget.properties["space_before"].as_f64() {
        ui.add_space(space as f32);
    }
    let height = widget.properties["max_height"]
        .as_f64()
        .map(|x| x as f32)
        .unwrap_or_else(|| {
            (ui.available_height()
                - widget.properties["reserve_height"].as_f64().unwrap_or(0.) as f32
                - widget.properties["reserve_lines"].as_f64().unwrap_or(0.) as f32
                    * ui.text_style_height(&egui::TextStyle::Body))
            .max(60.)
        });
    let scroll = if widget.properties["scroll_both"] == true {
        egui::ScrollArea::both()
    } else {
        egui::ScrollArea::vertical()
    };
    let rect = scroll
        .id_salt((id, widget.properties["document"].as_str().unwrap_or("")))
        .auto_shrink([false, false])
        .max_height(height)
        .stick_to_bottom(widget.properties["stick_to_bottom"] == true)
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            render(ui);
        })
        .inner_rect;
    ui.interact(rect, ui.make_persistent_id(id), egui::Sense::hover())
}
