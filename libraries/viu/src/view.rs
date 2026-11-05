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
            if !model.is_object() {
                return Err("scene model must be an object".into());
            }
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
    /// Shared control admission uses the currently rendered definition/model.
    /// Applications still own the meaning and execution of admitted intents.
    pub fn admit_intent(&self, id: &str, action: &str, value: &Value) -> Result<(), String> {
        let node = self.tree.nodes.get(id).ok_or("unknown scene node")?;
        match node.family.as_str() {
            "control" => {
                Widget::decode(&node.ports["definition"])?.admit(&self.tree.model, action, value)
            }
            "split" => {
                let min = num(node, "minimum").unwrap_or(0.1) as f64;
                let max = num(node, "maximum").unwrap_or(0.9) as f64;
                if action == "set"
                    && value
                        .as_f64()
                        .is_some_and(|v| v.is_finite() && (min..=max).contains(&v))
                {
                    Ok(())
                } else {
                    Err("invalid split intent".into())
                }
            }
            "dock" => {
                let child = value["child"].as_str().ok_or("dock child required")?;
                let placement = &node.ports["placements"][child];
                let min = placement["minimum"].as_f64().unwrap_or(0.05);
                let max = placement["maximum"].as_f64().unwrap_or(0.5);
                if action == "set"
                    && placement["resizable"] == true
                    && value["fraction"]
                        .as_f64()
                        .is_some_and(|v| v.is_finite() && (min..=max).contains(&v))
                {
                    Ok(())
                } else {
                    Err("invalid dock intent".into())
                }
            }
            _ => Err("scene node does not accept intents".into()),
        }
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
    pub fn text_input_active(&self) -> bool {
        self.widgets.text_input_active
    }
    pub fn image_states(&self) -> Value {
        self.widgets.image_states()
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
                    "dock"=>self.dock(ui,&key,node,nodes,route,depth,events),
                    "split"=>self.split(ui,&key,node,nodes,route,depth,events),
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
                        // Container gaps arrange children; they must not
                        // rewrite the native control's internal theme metrics.
                        let theme=if ui.visuals().dark_mode{egui::Theme::Dark}else{egui::Theme::Light};
                        let spacing=ui.ctx().style_of(theme).spacing.item_spacing;
                        ui.spacing_mut().item_spacing=spacing;
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
                            ui.add(crate::icons::image_of(ui.ctx(),icon,w,ui.visuals().text_color()).fit_to_exact_size(Vec2::new(w,h)));
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
                        let background = color(text(node,"background")).or_else(||text(node,"background").strip_prefix("design.").and_then(|role|crate::appearance::colour(ui.ctx(),role))).unwrap_or(Color32::TRANSPARENT);
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

    fn child_rect(
        &mut self,
        ui: &mut egui::Ui,
        id: &str,
        rect: egui::Rect,
        nodes: &BTreeMap<String, Node>,
        route: Option<(&str, &Node, &Value)>,
        depth: usize,
        events: &mut Vec<Event>,
    ) {
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect).id_salt(id), |ui| {
            ui.set_clip_rect(ui.clip_rect().intersect(rect));
            self.node(ui, id, nodes, route, depth + 1, events);
        });
    }

    fn layout_change(&self, node: &Node, id: &str, value: Value, events: &mut Vec<Event>) {
        if let Some(command) = node.ports.get("on_change").and_then(Value::as_str) {
            events.push(Event::from_intent(
                command,
                &self.tree.name,
                Intent::set(id, value),
            ));
        }
    }

    fn split(
        &mut self,
        ui: &mut egui::Ui,
        key: &str,
        node: &Node,
        nodes: &BTreeMap<String, Node>,
        route: Option<(&str, &Node, &Value)>,
        depth: usize,
        events: &mut Vec<Event>,
    ) {
        let area = ui.available_rect_before_wrap();
        let child = children(node).collect::<Vec<_>>();
        if child.len() != 2 {
            return;
        }
        let horizontal = text(node, "axis") != "vertical";
        let divider = num(node, "divider").unwrap_or(6.0).min(
            if horizontal {
                area.width()
            } else {
                area.height()
            }
            .max(0.0),
        );
        let min = num(node, "minimum").unwrap_or(0.1);
        let max = num(node, "maximum").unwrap_or(0.9);
        let ratio = num(node, "ratio").unwrap_or(0.5).clamp(min, max);
        let length = if horizontal {
            area.width()
        } else {
            area.height()
        };
        let first = ((length - divider).max(0.0) * ratio).round();
        let (a, grip, b) = if horizontal {
            let a = egui::Rect::from_min_size(area.min, Vec2::new(first, area.height()));
            let grip = egui::Rect::from_min_size(
                egui::pos2(a.right(), area.top()),
                Vec2::new(divider, area.height()),
            );
            (
                a,
                grip,
                egui::Rect::from_min_max(egui::pos2(grip.right(), area.top()), area.max),
            )
        } else {
            let a = egui::Rect::from_min_size(area.min, Vec2::new(area.width(), first));
            let grip = egui::Rect::from_min_size(
                egui::pos2(area.left(), a.bottom()),
                Vec2::new(area.width(), divider),
            );
            (
                a,
                grip,
                egui::Rect::from_min_max(egui::pos2(area.left(), grip.bottom()), area.max),
            )
        };
        let response = ui.interact(
            grip,
            egui::Id::unique((ui.ctx().viewport_id(), key, "divider")),
            Sense::click_and_drag(),
        );
        self.geometry.insert(format!("{key}.divider"), grip);
        if response.hovered() || response.dragged() {
            ui.ctx().set_cursor_icon(if horizontal {
                egui::CursorIcon::ResizeHorizontal
            } else {
                egui::CursorIcon::ResizeVertical
            });
        }
        if response.double_clicked() {
            self.layout_change(
                node,
                key,
                json!(0.5f64.clamp(min as f64, max as f64)),
                events,
            );
        } else if response.dragged() {
            if let Some(pointer) = response.interact_pointer_pos() {
                let offset = if horizontal {
                    pointer.x - area.left()
                } else {
                    pointer.y - area.top()
                };
                self.layout_change(
                    node,
                    key,
                    json!((offset / (length - divider).max(1.0)).clamp(min, max)),
                    events,
                );
            }
        }
        let stroke = ui.visuals().widgets.noninteractive.bg_stroke;
        if horizontal {
            ui.painter().vline(grip.center().x, grip.y_range(), stroke);
        } else {
            ui.painter().hline(grip.x_range(), grip.center().y, stroke);
        }
        self.child_rect(ui, child[0], a, nodes, route, depth, events);
        self.child_rect(ui, child[1], b, nodes, route, depth, events);
        ui.allocate_rect(area, Sense::hover());
    }

    fn dock(
        &mut self,
        ui: &mut egui::Ui,
        key: &str,
        node: &Node,
        nodes: &BTreeMap<String, Node>,
        route: Option<(&str, &Node, &Value)>,
        depth: usize,
        events: &mut Vec<Event>,
    ) {
        let area = ui.available_rect_before_wrap();
        let mut remaining = area;
        let placements = &node.ports["placements"];
        for child in children(node) {
            if nodes
                .get(child)
                .is_some_and(|n| n.ports.get("hidden") == Some(&Value::Bool(true)))
            {
                continue;
            }
            let placement = &placements[child];
            let edge = placement["edge"].as_str().unwrap_or("fill");
            if placement["minimum_window_width"]
                .as_f64()
                .is_some_and(|w| (area.width() as f64) < w)
            {
                continue;
            }
            let horizontal = ["left", "right"].contains(&edge);
            let dimension = if horizontal {
                area.width()
            } else {
                area.height()
            };
            let requested = placement["size"]
                .as_f64()
                .map(|v| v as f32)
                .unwrap_or_else(|| {
                    placement["fraction"].as_f64().unwrap_or(0.2) as f32 * dimension
                });
            let maximum = (if horizontal {
                remaining.width()
            } else {
                remaining.height()
            })
            .max(0.0);
            let size = requested.clamp(0.0, maximum);
            let mut rect = remaining;
            match edge {
                "left" => {
                    rect.max.x = rect.min.x + size;
                    remaining.min.x = rect.max.x;
                }
                "right" => {
                    rect.min.x = rect.max.x - size;
                    remaining.max.x = rect.min.x;
                }
                "top" => {
                    rect.max.y = rect.min.y + size;
                    remaining.min.y = rect.max.y;
                }
                "bottom" => {
                    rect.min.y = rect.max.y - size;
                    remaining.max.y = rect.min.y;
                }
                _ => {}
            }
            self.child_rect(ui, child, rect, nodes, route, depth, events);
            if placement["resizable"] == true && edge != "fill" {
                let line = match edge {
                    "left" => egui::Rect::from_min_max(
                        egui::pos2(rect.right() - 3.0, rect.top()),
                        egui::pos2(rect.right() + 3.0, rect.bottom()),
                    ),
                    "right" => egui::Rect::from_min_max(
                        egui::pos2(rect.left() - 3.0, rect.top()),
                        egui::pos2(rect.left() + 3.0, rect.bottom()),
                    ),
                    "top" => egui::Rect::from_min_max(
                        egui::pos2(rect.left(), rect.bottom() - 3.0),
                        egui::pos2(rect.right(), rect.bottom() + 3.0),
                    ),
                    _ => egui::Rect::from_min_max(
                        egui::pos2(rect.left(), rect.top() - 3.0),
                        egui::pos2(rect.right(), rect.top() + 3.0),
                    ),
                };
                let response = ui.interact(
                    line,
                    egui::Id::unique((ui.ctx().viewport_id(), key, child, "resize")),
                    Sense::drag(),
                );
                self.geometry.insert(format!("{child}.divider"), line);
                if response.hovered() || response.dragged() {
                    ui.ctx().set_cursor_icon(if horizontal {
                        egui::CursorIcon::ResizeHorizontal
                    } else {
                        egui::CursorIcon::ResizeVertical
                    });
                }
                if response.dragged() {
                    if let Some(pointer) = response.interact_pointer_pos() {
                        let pixels = match edge {
                            "left" => pointer.x - rect.left(),
                            "right" => rect.right() - pointer.x,
                            "top" => pointer.y - rect.top(),
                            _ => rect.bottom() - pointer.y,
                        };
                        let min = placement["min_fraction"].as_f64().unwrap_or(0.05) as f32;
                        let max = placement["max_fraction"].as_f64().unwrap_or(0.8) as f32;
                        let fraction = (pixels / dimension.max(1.0)).clamp(min, max);
                        self.layout_change(
                            node,
                            key,
                            json!({"child":child,"fraction":fraction}),
                            events,
                        );
                    }
                }
                if horizontal {
                    ui.painter().vline(
                        line.center().x,
                        line.y_range(),
                        ui.visuals().widgets.noninteractive.bg_stroke,
                    );
                } else {
                    ui.painter().hline(
                        line.x_range(),
                        line.center().y,
                        ui.visuals().widgets.noninteractive.bg_stroke,
                    );
                }
            }
        }
        ui.allocate_rect(area, Sense::hover());
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
            "split" => {
                if children(node).count() != 2
                    || num(node, "minimum").unwrap_or(0.1) > num(node, "maximum").unwrap_or(0.9)
                {
                    return Err(format!(
                        "split {id}: two children and ordered bounds required"
                    ));
                }
            }
            "dock" => {
                let children = children(node).collect::<Vec<_>>();
                let placements = node.ports["placements"]
                    .as_object()
                    .ok_or("dock placements must be an object")?;
                if placements.len() != children.len() {
                    return Err(format!("dock {id}: each child requires one placement"));
                }
                let mut fill = 0;
                for (index, child) in children.iter().enumerate() {
                    let p = placements
                        .get(*child)
                        .ok_or("dock child missing placement")?;
                    let edge = p["edge"].as_str().ok_or("dock edge required")?;
                    if !["top", "bottom", "left", "right", "fill"].contains(&edge) {
                        return Err("invalid dock edge".into());
                    }
                    if edge == "fill" {
                        fill += 1;
                        if index + 1 != children.len() {
                            return Err("dock fill must be last".into());
                        }
                    }
                    for key in [
                        "size",
                        "fraction",
                        "minimum_window_width",
                        "min_fraction",
                        "max_fraction",
                    ] {
                        if let Some(v) = p.get(key) {
                            if v.as_f64().is_none_or(|n| {
                                !n.is_finite() || n < 0.0 || (key.contains("fraction") && n > 1.0)
                            }) {
                                return Err(format!("invalid dock {key}"));
                            }
                        }
                    }
                    if p["min_fraction"].as_f64().unwrap_or(0.05)
                        > p["max_fraction"].as_f64().unwrap_or(0.8)
                    {
                        return Err("dock bounds are reversed".into());
                    }
                    if p.get("resizable").is_some_and(|v| !v.is_boolean()) {
                        return Err("dock resizable must be bool".into());
                    }
                }
                if fill != 1 {
                    return Err("dock requires one fill child".into());
                }
            }
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
