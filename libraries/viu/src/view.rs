// SPDX-License-Identifier: MIT OR Apache-2.0
//! Scene layout, binding/template preparation and event adaptation. Control
//! leaves and legacy text/button/field/toggle use the shared widget dispatch.
use crate::templates::{PreparedLists, children, rows, text, validate_templates};
use crate::widgets::{Intent, Widget, Widgets};
use egui::{Color32, Sense, Vec2};
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
    widgets: Widgets,
    focus_error: Option<String>,
}

impl SceneView {
    pub fn new(source: &str) -> Result<Self, String> {
        Self::prepare(source, None)
    }
    pub fn with_model(source: &str, model: Value) -> Result<Self, String> {
        Self::prepare(source, Some(model))
    }
    fn prepare(source: &str, model: Option<Value>) -> Result<Self, String> {
        let mut document = scene::parse(source).map_err(|e| format!("{e:?}"))?;
        if let Some(model) = model {
            document.model = Some(model);
        }
        let tree = scene::resolve(&document).map_err(|e| format!("{e:?}"))?;
        let lists = validate_templates(&tree).map_err(|e| e.to_string())?;
        validate_controls(&tree)?;
        Ok(Self {
            tree,
            lists,
            revision: 0,
            geometry: BTreeMap::new(),
            widgets: Widgets::default(),
            focus_error: None,
        })
    }

    pub fn model(&self) -> &Value {
        &self.tree.model
    }

    pub fn apply_model(&mut self, model: Value) -> Result<(), String> {
        if !model.is_object() {
            return Err("scene model must be an object".into());
        }
        let set = scene::bindings::BindingSet::from_resolved(&self.tree);
        let evaluated = scene::bindings::reevaluate(&self.tree, &set, "model", &model)
            .map_err(|e| format!("{e:?}"))?;
        if !evaluated.diagnostics.is_empty() {
            return Err(format!("{:?}", evaluated.diagnostics));
        }
        let candidate = evaluated.tree;
        let lists = validate_templates(&candidate).map_err(|e| e.to_string())?;
        validate_controls(&candidate)?;
        self.tree = candidate;
        self.lists = lists;
        self.revision += 1;
        Ok(())
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<Event> {
        let mut events = Vec::new();
        self.geometry.clear();
        self.widgets.begin_frame();
        let tree = self.tree.clone();
        let nodes = tree.nodes.into_iter().collect();
        self.node(ui, "root", &nodes, None, 0, &mut events);
        self.geometry
            .extend(self.widgets.geometry.iter().map(|(id, r)| (id.clone(), *r)));
        self.focus_error = self.widgets.finish_frame().err();
        events
    }
    pub fn request_focus(&mut self, id: &str) -> Result<(), String> {
        if !self.geometry.contains_key(id) {
            return Err("focus target not visible".into());
        }
        self.widgets.focus = Some(id.into());
        Ok(())
    }
    pub fn take_focus_error(&mut self) -> Option<String> {
        self.focus_error.take()
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
                if node.family!="control" {
                    ui.spacing_mut().item_spacing=Vec2::new(num(node,"column_gap").or(num(node,"gap")).unwrap_or(4.0),num(node,"row_gap").or(num(node,"gap")).unwrap_or(4.0));
                }
                match node.family.as_str() {
                    "grid" => {
                        let columns=num(node,"columns").unwrap_or(2.0) as usize;
                        let gap=num(node,"column_gap").unwrap_or(12.0);
                        let width=((ui.available_width()-gap*(columns-1) as f32)/columns as f32).max(num(node,"min_column_width").unwrap_or(0.0));
                        egui::Grid::new(text(node,"identity")).num_columns(columns).min_col_width(width).max_col_width(width)
                            .spacing([gap,num(node,"row_gap").unwrap_or(3.0)]).show(ui,|ui| {
                                for (index,child) in children(node).enumerate() {
                                    let cell=ui.scope(|ui| { ui.set_min_width(width); ui.set_max_width(width);
                                        self.node(ui,child,nodes,route,depth+1,events);
                                    });
                                    self.geometry.insert(child.into(),cell.response.rect);
                                    if index%columns==columns-1 { ui.end_row(); }
                                }
                            });
                    }
                    "control" => {
                        match Widget::decode(&node.ports["definition"]) {
                            Ok(widget)=> {
                                for intent in self.widgets.widget(ui,&widget,&self.tree.model) {
                                    if let Some(command)=node.ports.get("on_intent").and_then(Value::as_str) {
                                        events.push(Event::from_intent(command,&self.tree.name,intent));
                                    }
                                }
                            }
                            Err(error)=>{ ui.colored_label(Color32::RED,error); }
                        }
                    }
                    "text" => {
                        let colour=color(text(node,"color")).unwrap_or(Color32::WHITE).to_array();
                        let widget=Widget::new(&key,"label","",json!(text(node,"text"))).with_properties(json!({"size":num(node,"size").unwrap_or(13.0),"bold":node.ports.get("bold"),"mono":node.ports.get("mono"),"colour":colour}));
                        self.widgets.widget(ui,&widget,&json!({}));
                    }
                    "button" => {
                        let widget=Widget::new(&key,"button","",json!("Ready")).with_properties(json!({"text":text(node,"label")}));
                        for intent in self.widgets.widget(ui,&widget,&json!({})) {
                            if intent.action=="activate" { route_event(&self.tree.name,id,node,route,events); }
                        }
                    }
                    "field" | "toggle" => {
                        let widget=Widget::new(&key,&node.family,"",node.ports["value"].clone()).with_properties(json!({"text":text(node,"label"),"placeholder":text(node,"placeholder"),"password":node.ports.get("password")}));
                        for intent in self.widgets.widget(ui,&widget,&json!({})) {
                            let port=if intent.action=="submit" {"on_submit"} else {"on_change"};
                            if let Some(command)=node.ports.get(port).and_then(Value::as_str) {
                                events.push(Event{command:command.into(),body:json!({"scene":self.tree.name,"node":id,"kind":if intent.action=="submit" {"submit"} else {"change"},"value":intent.value})});
                            }
                        }
                    }
                    "image" => {
                        if let Some(icon)=text(node,"src").strip_prefix("icon:").and_then(crate::icons::Icon::from_name) {
                            let w=num(node,"w").or(num(node,"h")).unwrap_or(24.0);
                            let h=num(node,"h").unwrap_or(w);
                            ui.add(crate::icons::image(icon,w,ui.visuals().text_color()).fit_to_exact_size(Vec2::new(w,h)));
                        }
                    }
                    "spacer" => {
                        ui.add_space(num(node,"size").unwrap_or(8.0));
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
                                .max_height(280.0)
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
                                    ui.horizontal(|ui| draw(ui));
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
        if node.family != "control" {
            self.geometry.insert(key, response.rect);
        }
    }
}

impl Event {
    pub fn from_intent(command: &str, scene: &str, intent: Intent) -> Self {
        let mut body = intent.encode();
        body["scene"] = json!(scene);
        body["node"] = json!(intent.id);
        Self {
            command: command.into(),
            body,
        }
    }
}

fn validate_controls(tree: &ResolvedScene) -> Result<(), String> {
    for (id, node) in &tree.nodes {
        match node.family.as_str() {
            "control" => {
                let widget = Widget::decode(&node.ports["definition"])?;
                if &widget.id != id {
                    return Err(format!("control ID {} differs from node {id}", widget.id));
                }
                widget.validate_model(&tree.model)?;
            }
            "image" => {
                if text(node, "src")
                    .strip_prefix("icon:")
                    .and_then(crate::icons::Icon::from_name)
                    .is_none()
                {
                    return Err(format!("image {id}: only known icon: sources supported"));
                }
            }
            "text" | "button" | "field" | "toggle" | "row" | "column" | "list" | "spacer"
            | "grid" => {}
            family => return Err(format!("unsupported scene content family {family}")),
        }
    }
    Ok(())
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
