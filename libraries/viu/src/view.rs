// SPDX-License-Identifier: MIT OR Apache-2.0
//! Client adaptation of scene-host routes and template preparation, using
//! native egui primitives. Covers the Quoin and small Dopus specimen families,
//! including editable text fields with change/submit callbacks.
use crate::templates::{PreparedLists, children, rows, text, validate_templates};
use egui::{Color32, RichText, Sense, Vec2};
use scene::{Node, ResolvedScene};
use serde_json::{Value, json};
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub struct Event {
    pub command: String,
    pub body: Value,
}

pub struct SceneView {
    tree: ResolvedScene,
    lists: PreparedLists,
    pub revision: u64,
    pub geometry: BTreeMap<String, egui::Rect>,
    fields: BTreeMap<String, (String, String)>,
}

impl SceneView {
    pub fn new(source: &str) -> Result<Self, String> {
        let document = scene::parse(source).map_err(|e| format!("{e:?}"))?;
        let tree = scene::resolve(&document).map_err(|e| format!("{e:?}"))?;
        let lists = validate_templates(&tree).map_err(|e| e.to_string())?;
        Ok(Self {
            tree,
            lists,
            revision: 0,
            geometry: BTreeMap::new(),
            fields: BTreeMap::new(),
        })
    }

    pub fn model(&self) -> &Value {
        &self.tree.model
    }

    pub fn apply_model(&mut self, model: Value) -> Result<(), String> {
        let set = scene::bindings::BindingSet::from_resolved(&self.tree);
        let evaluated = scene::bindings::reevaluate(&self.tree, &set, "model", &model)
            .map_err(|e| format!("{e:?}"))?;
        if !evaluated.diagnostics.is_empty() {
            return Err(format!("{:?}", evaluated.diagnostics));
        }
        let candidate = evaluated.tree;
        let lists = validate_templates(&candidate).map_err(|e| e.to_string())?;
        self.tree = candidate;
        self.lists = lists;
        self.revision += 1;
        Ok(())
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<Event> {
        let mut events = Vec::new();
        self.geometry.clear();
        let tree = self.tree.clone();
        let nodes = tree.nodes.into_iter().collect();
        self.node(ui, "root", &nodes, None, 0, &mut events);
        events
    }

    fn node(
        &mut self,
        ui: &mut egui::Ui,
        id: &str,
        nodes: &BTreeMap<String, Node>,
        route: Option<(&str, &Node, &Value)>,
        depth: usize,
        events: &mut Vec<Event>,
    ) {
        if depth > 128 {
            return;
        }
        let Some(node) = nodes.get(id) else {
            return;
        };
        if node.ports.get("hidden").and_then(Value::as_bool) == Some(true) {
            return;
        }
        let key = route.map_or_else(
            || id.to_owned(),
            |(_, _, item)| format!("{}#{id}", item["id"].as_str().unwrap_or("")),
        );
        let response = ui
            .push_id(&key, |ui| {
                if let Some(width) = num(node, "width").or(num(node, "min_width")) {
                    ui.set_width(width);
                }
                ui.spacing_mut().item_spacing = Vec2::splat(num(node, "gap").unwrap_or(4.0));
                match node.family.as_str() {
                    "text" => {
                        let mut rich = RichText::new(text(node, "text"))
                            .size(num(node, "size").unwrap_or(13.0))
                            .color(color(text(node, "color")).unwrap_or(Color32::WHITE));
                        if node.ports.get("bold").and_then(Value::as_bool) == Some(true) {
                            rich = rich.strong();
                        }
                        ui.label(rich);
                    }
                    "button" => {
                        if ui.button(text(node, "label")).clicked() {
                            route_event(&self.tree.name, id, node, route, events);
                        }
                    }
                    "field" => {
                        let bound = text(node, "value").to_owned();
                        let field = self.fields.entry(key.clone())
                            .or_insert_with(|| (bound.clone(), bound.clone()));
                        // Preserve an in-progress edit across unrelated model updates.
                        // A changed authoritative binding replaces the edit buffer.
                        if field.0 != bound {
                            *field = (bound.clone(), bound);
                        }
                        let response = ui.add(egui::TextEdit::singleline(&mut field.1)
                            .hint_text(text(node, "placeholder"))
                            .desired_width(num(node, "width").unwrap_or(200.0)));
                        for (triggered, port, kind) in [
                            (response.changed(), "on_change", "change"),
                            (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)), "on_submit", "submit"),
                        ] {
                            if triggered {
                                if let Some(command) = node.ports.get(port).and_then(Value::as_str) {
                                    events.push(Event { command: command.into(), body: json!({
                                        "scene":self.tree.name, "node":id, "kind":kind, "value":field.1
                                    }) });
                                }
                            }
                        }
                    }
                    "spacer" => {
                        ui.add_space(8.0);
                    }
                    "list" => {
                        let horizontal = text(node, "flow") == "horizontal";
                        let data = self.lists.get(id).cloned().unwrap_or_default();
                        let list_rows = rows(node).to_vec();
                        let mut draw = |ui: &mut egui::Ui| {
                            for item in &list_rows {
                                if let Some(instance) =
                                    item["id"].as_str().and_then(|id| data.instances.get(id))
                                {
                                    self.node(
                                        ui,
                                        text(node, "row"),
                                        instance,
                                        Some((id, node, item)),
                                        depth + 1,
                                        events,
                                    );
                                }
                            }
                        };
                        if horizontal {
                            ui.horizontal(|ui| draw(ui));
                        } else {
                            egui::ScrollArea::vertical()
                                .min_scrolled_height(num(node, "min_height").unwrap_or(64.0))
                                .max_height(num(node, "max_height").unwrap_or(280.0))
                                .id_salt(id)
                                .show(ui, |ui| draw(ui));
                        }
                    }
                    "row" | "column" => {
                        let background =
                            color(text(node, "background")).unwrap_or(Color32::TRANSPARENT);
                        let frame = egui::Frame::new()
                            .fill(background)
                            .inner_margin(num(node, "padding").unwrap_or(0.0))
                            .corner_radius(num(node, "radius").unwrap_or(0.0));
                        let response = frame
                            .show(ui, |ui| {
                                if let Some(height) = num(node, "height") {
                                    ui.set_min_height(height);
                                }
                                if children(node).next().is_none() {
                                    ui.allocate_space(Vec2::new(
                                        num(node, "min_width")
                                            .or(num(node, "width"))
                                            .unwrap_or(0.0),
                                        num(node, "height").unwrap_or(0.0),
                                    ));
                                }
                                let mut draw = |ui: &mut egui::Ui| {
                                    for child in children(node) {
                                        self.node(ui, child, nodes, route, depth + 1, events);
                                    }
                                };
                                if node.family == "row" {
                                    if text(node, "align") == "stretch" {
                                        ui.horizontal_top(|ui| draw(ui));
                                    } else {
                                        ui.horizontal(|ui| draw(ui));
                                    }
                                } else {
                                    ui.vertical(|ui| draw(ui));
                                }
                            })
                            .response;
                        if route.is_some_and(|(_, list, _)| text(list, "row") == id)
                            || node.ports.contains_key("on_click")
                        {
                            if ui
                                .interact(response.rect, response.id.with("click"), Sense::click())
                                .clicked()
                            {
                                route_event(&self.tree.name, id, node, route, events);
                            }
                        }
                    }
                    _ => {
                        ui.colored_label(
                            Color32::RED,
                            format!("Unsupported widget: {}", node.family),
                        );
                    }
                }
            })
            .response;
        self.geometry.insert(key, response.rect);
    }
}

fn route_event(
    scene: &str,
    id: &str,
    node: &Node,
    route: Option<(&str, &Node, &Value)>,
    events: &mut Vec<Event>,
) {
    let (route_id, node, item) =
        route.map_or((id, node, None), |(id, node, item)| (id, node, Some(item)));
    if let Some(command) = node.ports.get("on_click").and_then(Value::as_str) {
        let mut body = json!({"scene":scene,"node":route_id,"kind":"click"});
        if let Some(item) = item {
            body["item"] = item.clone();
        }
        events.push(Event {
            command: command.into(),
            body,
        });
    }
}
fn num(node: &Node, port: &str) -> Option<f32> {
    node.ports.get(port)?.as_f64().map(|n| n as f32)
}
fn color(s: &str) -> Option<Color32> {
    let s = s.strip_prefix('#')?;
    if !s.is_ascii() || (s.len() != 6 && s.len() != 8) {
        return None;
    }
    let byte = |i| u8::from_str_radix(&s[i..i + 2], 16).ok();
    Some(Color32::from_rgba_unmultiplied(
        byte(0)?,
        byte(2)?,
        byte(4)?,
        if s.len() == 8 { byte(6)? } else { 255 },
    ))
}
