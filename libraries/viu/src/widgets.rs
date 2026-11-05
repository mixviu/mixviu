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
    "icon",
    "toolbar",
];

#[derive(Clone, Debug)]
pub struct Widget {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub initial: Value,
    pub properties: Value,
}
impl Widget {
    pub fn new(id: &str, kind: &str, label: &str, initial: Value) -> Self {
        Self {
            id: id.into(),
            kind: kind.into(),
            label: label.into(),
            initial,
            properties: json!({}),
        }
    }
    pub fn decode(value: &Value) -> Result<Self, String> {
        let field = |name| {
            value[name]
                .as_str()
                .ok_or_else(|| format!("widget missing {name}"))
        };
        let mut widget = Self::new(
            field("id")?,
            field("kind")?,
            field("label")?,
            value["initial"].clone(),
        );
        widget.properties = value
            .get("properties")
            .cloned()
            .unwrap_or_else(|| json!({}));
        if !KINDS.contains(&widget.kind.as_str()) {
            return Err(format!("unsupported kind {}", widget.kind));
        }
        widget.validate(&widget.initial)?;
        Ok(widget)
    }
    pub fn encode(&self) -> Value {
        json!({"id":self.id,"kind":self.kind,"label":self.label,"initial":self.initial,"properties":self.properties})
    }
    pub fn with_properties(mut self, properties: Value) -> Self {
        self.properties = properties;
        self
    }
    fn choices(&self) -> Vec<&str> {
        self.properties["choices"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(Value::as_str)
            .collect()
    }
    fn text(&self, key: &str) -> &str {
        self.properties[key].as_str().unwrap_or("")
    }
    pub fn validate(&self, value: &Value) -> Result<(), String> {
        let valid = match self.kind.as_str() {
            "icon" => value["name"].as_str().and_then(crate::icons::Icon::from_name).is_some()
                && value["size"].as_u64().is_some_and(|s| (8..=64).contains(&s))
                && value["colour"].as_array().is_some_and(|c| c.len()==3 && c.iter().all(|v|v.as_u64().is_some_and(|n|n<=255)))
                && value["enabled"].is_boolean() && value["selected"].is_boolean(),
            "toolbar" => value.as_str().is_some_and(|v| self.properties["items"].as_array().is_some_and(|items|items.iter().any(|i|i["id"]==v && i["enabled"]==true))),
            "checkbox" | "toggle" | "collapse" | "dialog" => value.is_boolean(),
            "slider" | "number" | "progress" | "split" => value
                .as_f64()
                .is_some_and(|v| v.is_finite() && (0.0..=100.0).contains(&v)),
            "radio" | "combo" | "tabs" | "menu" => {
                value.as_str().is_some_and(|v| self.choices().contains(&v))
            }
            "list" | "table" | "tree" => value
                .as_u64()
                .is_some_and(|v| v < self.properties["row_count"].as_u64().unwrap_or(0)),
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
    /// Draw one shared widget within an application-owned layout. Both Rust
    /// and Mix adapters can use this entry point without the catalogue grid.
    pub fn widget(&mut self, ui: &mut egui::Ui, definition: &Widget, model: &Value) -> Vec<Intent> {
        let mut intents = Vec::new();
        let value = model["values"]
            .get(&definition.id)
            .unwrap_or(&definition.initial);
        ui.push_id(&definition.id, |ui| {
            self.draw(ui, definition, value, model, &mut intents)
        });
        intents
    }
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        definitions: &[Widget],
        model: &Value,
    ) -> Vec<Intent> {
        self.geometry.clear();
        let mut intents = Vec::new();
        let width = ((ui.available_width() - 12.0) / 2.0).max(200.0);
        egui::Grid::new("viu-widget-gallery")
            .num_columns(2)
            .min_col_width(width)
            .max_col_width(width)
            .spacing([12.0, 3.0])
            .show(ui, |ui| {
                for (index, widget) in definitions.iter().enumerate() {
                    let response = ui
                        .push_id(&widget.id, |ui| {
                            ui.set_min_width(width);
                            ui.set_max_width(width);
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
    /// Call once after drawing an application frame (including individual
    /// `widget` calls). A disappeared target must not latch future replies.
    pub fn finish_frame(&mut self) -> Result<(), String> {
        if let Some(id)=self.focus.take() { Err(format!("focus target not drawn: {id}")) } else { Ok(()) }
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
                let r = icon_button(ui,widget.text("icon"),widget.text("text"),false);
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
                    ui.checkbox(&mut checked, widget.text("text"))
                } else {
                    ui.toggle_value(&mut checked, widget.text("text"))
                };
                if r.changed() || r.clicked() {
                    changed = Some(json!(checked));
                }
                r
            }
            "radio" | "tabs" => {
                let response = ui
                    .horizontal(|ui| {
                        for option in widget.choices() {
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
                if widget.kind == "tabs" {
                    ui.small(
                        widget.properties["pages"][value.as_str().unwrap_or("")]
                            .as_str()
                            .unwrap_or(""),
                    );
                }
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
                    .selected_text(value.as_str().unwrap_or(""))
                    .show_ui(ui, |ui| {
                        for option in widget.choices() {
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
                let source = widget.text("dataset");
                let data = model["datasets"][source]
                    .as_array()
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                if widget.kind == "table" {
                    let reversed = model["sorted"][id].as_bool().unwrap_or(false);
                    let header = ui.button(format!(
                        "{} {}                         {}",
                        widget.properties["columns"][0].as_str().unwrap_or(""),
                        if reversed { "v" } else { "^" },
                        widget.properties["columns"][1].as_str().unwrap_or("")
                    ));
                    self.remember(ui, &format!("{id}.sort"), &header);
                    if header.clicked() {
                        events.push(Intent {
                            id: id.clone(),
                            action: "sort".into(),
                            value: json!(!reversed),
                        });
                    }
                }
                let expanded = model["expanded"][id].as_bool().unwrap_or(true);
                if widget.kind == "tree" {
                    let caption = format!(
                        "{} {}",
                        if expanded { "v" } else { ">" },
                        widget.text("parent")
                    );
                    let header = icon_button(ui,widget.text("parent_icon"),&caption,false);
                    self.remember(ui, &format!("{id}.expand"), &header);
                    if header.clicked() {
                        events.push(Intent {
                            id: id.clone(),
                            action: "expand".into(),
                            value: json!(!expanded),
                        });
                    }
                }
                let area = egui::ScrollArea::vertical()
                    .id_salt(id)
                    .max_height(54.0)
                    .auto_shrink([false, false])
                    .show_rows(
                        ui,
                        ui.spacing().interact_size.y,
                        if widget.kind == "tree" && !expanded {
                            0
                        } else {
                            data.len()
                        },
                        |ui, range| {
                            for row in range {
                                let row = if widget.kind == "table" && model["sorted"][id] == true {
                                    data.len() - 1 - row
                                } else {
                                    row
                                };
                                let cells = &data[row];
                                let name = cells[0].as_str().unwrap_or("");
                                let r = if widget.kind == "table" {
                                    let (rect, response) = ui.allocate_exact_size(
                                        Vec2::new(
                                            ui.available_width(),
                                            ui.spacing().interact_size.y,
                                        ),
                                        Sense::click(),
                                    );
                                    let selected = value.as_u64() == Some(row as u64);
                                    let visuals =
                                        ui.style().interact_selectable(&response, selected);
                                    ui.painter().rect_filled(rect, 2.0, visuals.bg_fill);
                                    let font = egui::FontId::proportional(13.0);
                                    if let Some(icon)=crate::icons::Icon::from_name(widget.text("row_icon")) {
                                        crate::icons::image(icon,16.0,visuals.text_color()).paint_at(ui,Rect::from_center_size(rect.left_center()+Vec2::new(12.0,0.0),Vec2::splat(16.0)));
                                    }
                                    ui.painter().text(
                                        rect.left_center() + Vec2::new(if widget.text("row_icon").is_empty() {6.0} else {24.0}, 0.0),
                                        egui::Align2::LEFT_CENTER,
                                        name,
                                        font.clone(),
                                        visuals.text_color(),
                                    );
                                    ui.painter().text(
                                        rect.right_center() - Vec2::new(6.0, 0.0),
                                        egui::Align2::RIGHT_CENTER,
                                        cells[1].as_str().unwrap_or(""),
                                        font,
                                        visuals.text_color(),
                                    );
                                    response
                                } else {
                                    icon_button(ui,widget.text("row_icon"),&if widget.kind == "tree" {
                                            format!("    · {name}")
                                        } else {
                                            name.to_owned()
                                        },value.as_u64() == Some(row as u64))
                                };
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
                                    let choice = icon_button(ui,widget.text("context_icon"),widget.text("context_action"),false);
                                    self.remember(ui, &format!("{id}.{row}.choose"), &choice);
                                    if choice.clicked() {
                                        changed = Some(json!(row));
                                        ui.close();
                                    }
                                });
                            }
                        },
                    );
                ui.interact(area.inner_rect, ui.scope_id().with(id), Sense::hover())
            }
            "collapse" => {
                let mut open = value.as_bool().unwrap_or(false);
                let r = ui.selectable_label(
                    open,
                    format!("{} {}", if open { "v" } else { ">" }, widget.text("text")),
                );
                if open {
                    ui.small(widget.text("body"));
                }
                if r.clicked() {
                    open = !open;
                    changed = Some(json!(open));
                }
                r
            }
            "menu" => {
                ui.menu_button(
                    format!("{} · {}", widget.text("text"), value.as_str().unwrap_or("")),
                    |ui| {
                        for option in widget.choices() {
                            let r = icon_button(ui,widget.properties["choice_icons"][option].as_str().unwrap_or(""),option,false);
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
                let r = ui.button(widget.text("text"));
                if r.clicked() {
                    changed = Some(json!(true));
                }
                if value.as_bool() == Some(true) {
                    egui::Window::new(widget.text("title"))
                        .id(Id::unique(id))
                        .collapsible(false)
                        .resizable(false)
                        .show(ui.ctx(), |ui| {
                            ui.label(widget.text("body"));
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
                for shape in widget.properties["shapes"].as_array().into_iter().flatten() {
                    let point = |v: &Value| {
                        rect.min
                            + Vec2::new(
                                v[0].as_f64().unwrap_or(0.0) as f32 * rect.width(),
                                v[1].as_f64().unwrap_or(0.0) as f32 * rect.height(),
                            )
                    };
                    if shape["kind"] == "line" {
                        ui.painter().line_segment(
                            [point(&shape["from"]), point(&shape["to"])],
                            (2.0, Color32::LIGHT_BLUE),
                        );
                    }
                    if shape["kind"] == "circle" {
                        ui.painter().circle_filled(
                            point(&shape["at"]),
                            shape["radius"].as_f64().unwrap_or(1.0) as f32,
                            Color32::GOLD,
                        );
                    }
                }
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
                let r = ui.link(widget.text("text"));
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
                .on_hover_text(widget.text("body")),
            "separator" => {
                let r = ui.separator();
                ui.small(value.as_str().unwrap_or("Section boundary"));
                r
            }
            "icon" => {
                let icon=crate::icons::Icon::from_name(value["name"].as_str().unwrap_or(""));
                if let Some(icon)=icon {
                    let rgb=value["colour"].as_array().unwrap();
                    let tint=Color32::from_rgb(rgb[0].as_u64().unwrap() as u8,rgb[1].as_u64().unwrap() as u8,rgb[2].as_u64().unwrap() as u8);
                    let r=crate::icons::button(ui,icon,widget.text("text"),value["size"].as_u64().unwrap() as f32,Some(tint),value["enabled"]==true,value["selected"]==true);
                    if r.clicked() { events.push(Intent{id:id.clone(),action:"activate".into(),value:value["name"].clone()}); }
                    r.on_hover_text(value["name"].as_str().unwrap_or(""))
                } else { ui.colored_label(Color32::RED,"Unknown icon") }
            }
            "toolbar" => ui.horizontal(|ui| {
                for item in widget.properties["items"].as_array().into_iter().flatten() {
                    if let Some(icon)=item["icon"].as_str().and_then(crate::icons::Icon::from_name) {
                        let r=crate::icons::button(ui,icon,item["text"].as_str().unwrap_or(""),16.0,None,item["enabled"]==true,value==&item["id"]);
                        self.remember(ui,&format!("{id}.{}",item["id"].as_str().unwrap_or("")),&r);
                        if r.clicked() { events.push(Intent{id:id.clone(),action:"activate".into(),value:item["id"].clone()}); }
                    }
                }
            }).response,
            _ => ui.colored_label(Color32::RED, "Unsupported widget"),
        };
        self.remember(ui, id, &response);
        if let Some(value) = changed {
            events.push(Intent::set(id, value));
        }
    }
}

fn icon_button(ui:&mut egui::Ui,name:&str,text:&str,selected:bool)->egui::Response {
    if let Some(icon)=crate::icons::Icon::from_name(name) {
        crate::icons::button(ui,icon,text,16.0,None,true,selected)
    } else { ui.add(egui::Button::new(text).selected(selected)) }
}
