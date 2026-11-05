// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared native widgets. Applications supply their own definitions and model;
//! this module draws and reports intents, never executes application policy.
use egui::{Color32, Id, Rect, Sense, Vec2};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub const KINDS: &[&str] = &[
    "label",
    "button",
    "field",
    "multiline",
    "checkbox",
    "toggle",
    "radio",
    "slider",
    "number",
    "progress",
    "combo",
    "list",
    "table",
    "tree",
    "collapse",
    "tabs",
    "menu",
    "dialog",
    "split",
    "colour",
    "canvas",
    "link",
    "tooltip",
    "separator",
];

#[derive(Clone, Debug)]
pub struct Widget {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub initial: Value,
}
impl Widget {
    pub fn new(id: &str, kind: &str, label: &str, initial: Value) -> Self {
        Self {
            id: id.into(),
            kind: kind.into(),
            label: label.into(),
            initial,
        }
    }
    pub fn decode(value: &Value) -> Result<Self, String> {
        let field = |name| {
            value[name]
                .as_str()
                .ok_or_else(|| format!("widget missing {name}"))
        };
        let widget = Self::new(
            field("id")?,
            field("kind")?,
            field("label")?,
            value["initial"].clone(),
        );
        if !KINDS.contains(&widget.kind.as_str()) {
            return Err(format!("unsupported kind {}", widget.kind));
        }
        widget.validate(&widget.initial)?;
        Ok(widget)
    }
    pub fn encode(&self) -> Value {
        json!({"id":self.id,"kind":self.kind,"label":self.label,"initial":self.initial})
    }
    pub fn validate(&self, value: &Value) -> Result<(), String> {
        let valid = match self.kind.as_str() {
            "checkbox" | "toggle" | "collapse" | "dialog" => value.is_boolean(),
            "slider" | "number" | "progress" | "split" => value
                .as_f64()
                .is_some_and(|v| v.is_finite() && (0.0..=100.0).contains(&v)),
            "radio" | "combo" | "tabs" | "menu" => value
                .as_str()
                .is_some_and(|v| ["Alpha", "Beta", "Gamma"].contains(&v)),
            "list" | "table" | "tree" => value.as_u64().is_some_and(|v| v < 1000),
            "colour" => value.as_array().is_some_and(|v| {
                v.len() == 3 && v.iter().all(|c| c.as_u64().is_some_and(|c| c <= 255))
            }),
            _ => value.as_str().is_some_and(|v| v.len() <= 4096),
        };
        if valid {
            Ok(())
        } else {
            Err(format!("invalid value for {} ({})", self.id, self.kind))
        }
    }
}

#[derive(Clone, Debug)]
pub struct Intent {
    pub id: String,
    pub action: String,
    pub value: Value,
}
impl Intent {
    pub fn set(id: &str, value: Value) -> Self {
        Self {
            id: id.into(),
            action: "set".into(),
            value,
        }
    }
    pub fn encode(&self) -> Value {
        json!({"id":self.id,"action":self.action,"value":self.value})
    }
}

#[derive(Default)]
pub struct Widgets {
    pub geometry: BTreeMap<String, Rect>,
    pub focus: Option<String>,
}
impl Widgets {
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        definitions: &[Widget],
        model: &Value,
    ) -> Vec<Intent> {
        self.geometry.clear();
        let mut intents = Vec::new();
        egui::Grid::new("viu-widget-gallery")
            .num_columns(2)
            .spacing([12.0, 5.0])
            .show(ui, |ui| {
                let width = ((ui.available_width() - 12.0) / 2.0).max(200.0);
                for (index, widget) in definitions.iter().enumerate() {
                    let response = ui
                        .push_id(&widget.id, |ui| {
                            ui.set_width(width);
                            ui.vertical(|ui| {
                                ui.label(
                                    egui::RichText::new(format!(
                                        "{:02}  {}",
                                        index + 1,
                                        widget.label
                                    ))
                                    .strong()
                                    .size(12.0),
                                );
                                let value =
                                    model["values"].get(&widget.id).unwrap_or(&widget.initial);
                                self.draw(ui, widget, value, model, &mut intents);
                            });
                        })
                        .response;
                    self.geometry
                        .insert(format!("{}.tile", widget.id), response.rect);
                    if index % 2 == 1 {
                        ui.end_row();
                    }
                }
            });
        intents
    }
    fn remember(&mut self, ui: &egui::Ui, id: &str, response: &egui::Response) {
        self.geometry.insert(id.into(), response.rect);
        if self.focus.as_deref() == Some(id) {
            response.request_focus();
            self.focus = None;
        }
        if response.has_focus() {
            ui.ctx()
                .data_mut(|d| d.insert_temp(Id::unique("viu-focused-widget"), id.to_owned()));
        }
    }
    fn draw(
        &mut self,
        ui: &mut egui::Ui,
        widget: &Widget,
        value: &Value,
        model: &Value,
        events: &mut Vec<Intent>,
    ) {
        let id = &widget.id;
        let mut changed = None;
        let response = match widget.kind.as_str() {
            "label" => ui.label(value.as_str().unwrap_or_default()),
            "button" => {
                let r = ui.button("Press me");
                if r.clicked() {
                    events.push(Intent {
                        id: id.clone(),
                        action: "activate".into(),
                        value: Value::Null,
                    });
                }
                r
            }
            "field" | "multiline" => {
                let mut text = value.as_str().unwrap_or_default().to_owned();
                let edit = if widget.kind == "field" {
                    egui::TextEdit::singleline(&mut text)
                } else {
                    egui::TextEdit::multiline(&mut text).desired_rows(1)
                };
                let r = ui.add(edit.desired_width(ui.available_width()).id_salt(id));
                if r.changed() {
                    changed = Some(json!(text));
                }
                if r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    events.push(Intent {
                        id: id.clone(),
                        action: "submit".into(),
                        value: json!(text),
                    });
                }
                r
            }
            "checkbox" | "toggle" => {
                let mut checked = value.as_bool().unwrap_or(false);
                let r = if widget.kind == "checkbox" {
                    ui.checkbox(&mut checked, "Enabled")
                } else {
                    ui.toggle_value(&mut checked, "Power")
                };
                if r.changed() || r.clicked() {
                    changed = Some(json!(checked));
                }
                r
            }
            "radio" | "tabs" => {
                let response=ui.horizontal(|ui| {
                    for option in ["Alpha", "Beta", "Gamma"] {
                        let r = if widget.kind == "radio" {
                            ui.radio(value == option, option)
                        } else {
                            ui.selectable_label(value == option, option)
                        };
                        self.remember(ui, &format!("{id}.{option}"), &r);
                        if r.clicked() {
                            changed = Some(json!(option));
                        }
                    }
                })
                .response;
                if widget.kind=="tabs" {ui.small(format!("{} page content",value.as_str().unwrap_or("Alpha")));}
                response
            }
            "slider" | "number" | "split" => {
                let mut number = value.as_f64().unwrap_or(50.0);
                let r = if widget.kind == "number" {
                    ui.add(
                        egui::DragValue::new(&mut number)
                            .range(0.0..=100.0)
                            .speed(1.0),
                    )
                } else if widget.kind == "split" {
                    let (rect, r) = ui
                        .allocate_exact_size(Vec2::new(ui.available_width(), 24.0), Sense::drag());
                    if r.dragged() {
                        if let Some(pos) = r.interact_pointer_pos() {
                            number = ((pos.x - rect.left()) / rect.width() * 100.0)
                                .clamp(0.0, 100.0) as f64;
                        }
                    }
                    let x = rect.left() + rect.width() * number as f32 / 100.0;
                    ui.painter()
                        .rect_filled(rect, 2.0, Color32::from_rgb(35, 56, 78));
                    ui.painter().rect_filled(
                        Rect::from_min_max(egui::pos2(x, rect.top()), rect.max),
                        2.0,
                        Color32::from_rgb(66, 46, 70),
                    );
                    ui.painter()
                        .vline(x, rect.y_range(), (3.0, Color32::LIGHT_BLUE));
                    r
                } else {
                    ui.add(egui::Slider::new(&mut number, 0.0..=100.0).suffix(" %"))
                };
                if r.changed() || r.dragged() {
                    changed = Some(json!(number));
                }
                r
            }
            "progress" => ui.add(
                egui::ProgressBar::new(value.as_f64().unwrap_or(50.0) as f32 / 100.0)
                    .show_percentage()
                    .desired_width(ui.available_width()),
            ),
            "combo" => {
                egui::ComboBox::from_id_salt(id)
                    .selected_text(value.as_str().unwrap_or("Alpha"))
                    .show_ui(ui, |ui| {
                        for option in ["Alpha", "Beta", "Gamma"] {
                            let r = ui.selectable_label(value == option, option);
                            self.remember(ui, &format!("{id}.{option}"), &r);
                            if r.clicked() {
                                changed = Some(json!(option));
                            }
                        }
                    })
                    .response
            }
            "list" | "table" | "tree" => {
                if widget.kind=="table" {
                    let reversed=model["sorted"][id].as_bool().unwrap_or(false);
                    let header=ui.button(if reversed {"Name ↓                         Size"}else{"Name ↑                         Size"});
                    self.remember(ui,&format!("{id}.sort"),&header);
                    if header.clicked(){events.push(Intent{id:id.clone(),action:"sort".into(),value:json!(!reversed)});}
                }
                let expanded=model["expanded"][id].as_bool().unwrap_or(true);
                if widget.kind=="tree" {
                    let header=ui.button(if expanded {"▾ Example folder"}else{"▸ Example folder"});
                    self.remember(ui,&format!("{id}.expand"),&header);
                    if header.clicked(){events.push(Intent{id:id.clone(),action:"expand".into(),value:json!(!expanded)});}
                }
                let area = egui::ScrollArea::vertical()
                    .id_salt(id)
                    .max_height(54.0)
                    .auto_shrink([false, false])
                    .show_rows(ui, 16.0, if widget.kind=="tree"&&!expanded {0}else{1000}, |ui, range| {
                        for row in range {
                            let row=if widget.kind=="table"&&model["sorted"][id]==true {999-row}else{row};
                            let text = match widget.kind.as_str() {
                                "table" => format!("Item {row:04}                 {} KiB", row + 1),
                                "tree" => format!("    · Item {row:04}"),
                                _ => format!("Item {row:04}"),
                            };
                            let r = ui.selectable_label(value.as_u64() == Some(row as u64), text);
                            self.remember(ui, &format!("{id}.{row}"), &r);
                            if r.clicked() {
                                changed = Some(json!(row));
                            }
                            if r.double_clicked() {
                                events.push(Intent {
                                    id: id.clone(),
                                    action: "activate".into(),
                                    value: json!(row),
                                });
                            }
                            r.context_menu(|ui| {
                                let choice=ui.button("Choose item");
                                self.remember(ui,&format!("{id}.{row}.choose"),&choice);
                                if choice.clicked() {
                                    changed = Some(json!(row));
                                    ui.close();
                                }
                            });
                        }
                    });
                ui.interact(area.inner_rect, ui.scope_id().with(id), Sense::hover())
            }
            "collapse" => {
                let mut open = value.as_bool().unwrap_or(false);
                let r = ui.selectable_label(
                    open,
                    if open {
                        "▾ Details: reusable Viu content"
                    } else {
                        "▸ Details"
                    },
                );
                if r.clicked() {
                    open = !open;
                    changed = Some(json!(open));
                }
                r
            }
            "menu" => {
                ui.menu_button(
                    format!("Actions · {}", value.as_str().unwrap_or("Alpha")),
                    |ui| {
                        for option in ["Alpha", "Beta", "Gamma"] {
                            let r = ui.button(option);
                            self.remember(ui, &format!("{id}.{option}"), &r);
                            if r.clicked() {
                                changed = Some(json!(option));
                                ui.close();
                            }
                        }
                    },
                )
                .response
            }
            "dialog" => {
                let r = ui.button("Open confirmation…");
                if r.clicked() {
                    changed = Some(json!(true));
                }
                if value.as_bool() == Some(true) {
                    egui::Window::new("Gallery confirmation")
                        .id(Id::unique(id))
                        .collapsible(false)
                        .resizable(false)
                        .show(ui.ctx(), |ui| {
                            ui.label("The same dialog in Rust and Mix.");
                            for choice in ["Accept", "Cancel"] {
                                let r = ui.button(choice);
                                self.remember(ui, &format!("{id}.{choice}"), &r);
                                if r.clicked() {
                                    events.push(Intent {
                                        id: id.clone(),
                                        action: choice.to_lowercase(),
                                        value: Value::Null,
                                    });
                                }
                            }
                        });
                }
                r
            }
            "colour" => {
                let mut rgb = [0u8; 3];
                for (i, c) in rgb.iter_mut().enumerate() {
                    *c = value[i].as_u64().unwrap_or(0) as u8;
                }
                let r = ui.color_edit_button_srgb(&mut rgb);
                if r.changed() {
                    changed = Some(json!(rgb));
                }
                r
            }
            "canvas" => {
                let (rect, r) = ui.allocate_exact_size(
                    Vec2::new(ui.available_width(), 24.0),
                    Sense::click_and_drag(),
                );
                ui.painter()
                    .rect_filled(rect, 3.0, Color32::from_rgb(32, 48, 64));
                ui.painter().line_segment(
                    [rect.left_bottom(), rect.right_top()],
                    (2.0, Color32::LIGHT_BLUE),
                );
                ui.painter()
                    .circle_filled(rect.center(), 7.0, Color32::GOLD);
                if r.clicked() {
                    events.push(Intent {
                        id: id.clone(),
                        action: "activate".into(),
                        value: Value::Null,
                    });
                }
                r
            }
            "link" => {
                let r = ui.link("Activate a safe local link");
                if r.clicked() {
                    events.push(Intent {
                        id: id.clone(),
                        action: "activate".into(),
                        value: Value::Null,
                    });
                }
                r
            }
            "tooltip" => ui
                .label(value.as_str().unwrap_or("Hover for help"))
                .on_hover_text("Shared Viu tooltip; identical in both languages."),
            "separator" => {
                let r = ui.separator();
                ui.small(value.as_str().unwrap_or("Section boundary"));
                r
            }
            _ => ui.colored_label(Color32::RED, "Unsupported widget"),
        };
        self.remember(ui, id, &response);
        if let Some(value) = changed {
            events.push(Intent::set(id, value));
        }
    }
}
