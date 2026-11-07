// SPDX-License-Identifier: MIT OR Apache-2.0
//! Scene layout, binding/template preparation and event adaptation. Control
//! leaves and legacy text/button/field/toggle use the shared widget dispatch.
use crate::templates::{PreparedLists, children, rows, text, validate_templates};
use crate::widgets::{Intent, Widget, Widgets};
use egui::{Color32, Sense, Vec2};
use scene::{Node, ResolvedScene};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// Data held by native virtual controls is independent of authored scene text
/// and of legacy template row expansion. Refuse before changing live state.
pub const MAX_MODEL_BYTES: usize = 8 * 1024 * 1024;

fn validate_model_budget(model: &Value) -> Result<(), String> {
    if !model.is_object() {
        return Err("scene model must be an object".into());
    }
    let bytes = serde_json::to_vec(model).map_err(|e| e.to_string())?;
    if bytes.len() > MAX_MODEL_BYTES {
        return Err("scene model exceeds 8 MiB data budget".into());
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct Event {
    pub command: String,
    pub body: Value,
}

pub struct SceneView {
    frame: Option<Box<SceneView>>,
    regions: BTreeMap<String, egui::Rect>,
    tree: ResolvedScene,
    lists: PreparedLists,
    pub revision: u64,
    pub geometry: BTreeMap<String, egui::Rect>,
    widgets: Widgets,
    focus_error: Option<String>,
    shown_revision: Option<u64>,
    slots: BTreeMap<String, egui::Rect>,
    overlays: BTreeSet<String>,
}

impl SceneView {
    pub fn set_native_menus(&mut self, enabled: bool) {
        self.widgets.set_native_menus(enabled);
        if let Some(frame) = self.frame.as_mut() {
            frame.set_native_menus(enabled);
        }
    }
    pub fn set_documents(&mut self, documents: crate::documents::Documents) {
        self.widgets.set_documents(documents.clone());
        if let Some(frame) = self.frame.as_mut() {
            frame.set_documents(documents);
        }
    }
    pub fn allocated_region(&self, id: &str) -> Option<egui::Rect> {
        self.frame
            .as_ref()
            .and_then(|frame| frame.allocated_region(id))
            .or_else(|| self.regions.get(id).copied())
    }
    pub fn new(source: &str) -> Result<Self, String> {
        Self::prepare(source, None)
    }
    pub fn with_model(source: &str, model: Value) -> Result<Self, String> {
        Self::prepare(source, Some(model))
    }
    fn prepare(source: &str, model: Option<Value>) -> Result<Self, String> {
        let mut document = scene::parse(source).map_err(|e| format!("{e:?}"))?;
        if let Some(model) = model {
            validate_model_budget(&model)?;
            document.model = Some(model);
        } else if let Some(model) = &document.model {
            validate_model_budget(model)?;
        }
        let tree = scene::resolve(&document).map_err(|e| format!("{e:?}"))?;
        let lists = validate_templates(&tree).map_err(|e| e.to_string())?;
        validate_controls(&tree)?;
        Ok(Self {
            frame: None,
            regions: BTreeMap::new(),
            tree,
            lists,
            revision: 0,
            geometry: BTreeMap::new(),
            widgets: Widgets::default(),
            focus_error: None,
            shown_revision: None,
            slots: BTreeMap::new(),
            overlays: BTreeSet::new(),
        })
    }

    pub fn model(&self) -> &Value {
        &self.tree.model
    }
    /// Mount independently resolved scenes, preserving every application ID.
    /// The frame authors the content rectangle; no app node is rewritten.
    pub fn with_frame(source: &str, model: Value, frame_source: &str) -> Result<Self, String> {
        let mut view = Self::with_model(source, model.clone())?;
        let frame = Self::with_model(frame_source, model)?;
        if !frame.tree.nodes.contains_key("gold-content") {
            return Err("frame must declare gold-content".into());
        }
        if frame
            .tree
            .nodes
            .keys()
            .any(|id| id != "root" && view.tree.nodes.contains_key(id))
        {
            return Err("frame and application node identities overlap".into());
        }
        view.frame = Some(Box::new(frame));
        Ok(view)
    }
    /// Hosted controls retain the same identity, model and intent dispatch.
    /// Their popup layers draw independently of the detached client layout.
    pub fn set_overlay_controls(
        &mut self,
        ids: impl IntoIterator<Item = String>,
    ) -> Result<(), String> {
        let ids = ids.into_iter().collect::<BTreeSet<_>>();
        if ids
            .iter()
            .any(|id| !matches!(self.control_kind(id), Some("menubar" | "command-bar")))
        {
            return Err("only menu/command controls support native hosted overlays".into());
        }
        self.overlays = ids;
        Ok(())
    }
    pub fn control_kind(&self, id: &str) -> Option<&str> {
        if let Some(kind) = self.frame.as_ref().and_then(|frame| frame.control_kind(id)) {
            return Some(kind);
        }
        let node = self.tree.nodes.get(id)?;
        if node.family != "control" {
            return None;
        }
        node.ports.get("definition")?.get("kind")?.as_str()
    }
    pub fn has_control_kind(&self, kind: &str) -> bool {
        if self
            .frame
            .as_ref()
            .is_some_and(|frame| frame.has_control_kind(kind))
        {
            return true;
        }
        self.tree
            .nodes
            .keys()
            .any(|id| self.control_kind(id) == Some(kind))
    }
    pub fn has_slot(&self, name: &str) -> bool {
        if self
            .frame
            .as_ref()
            .is_some_and(|frame| frame.has_slot(name))
        {
            return true;
        }
        self.tree
            .nodes
            .values()
            .any(|node| node.ports.get("window_slot").and_then(Value::as_str) == Some(name))
    }
    /// Host geometry for scene-declared slots. An attached dock child draws
    /// there instead of consuming its ordinary edge; no app layout is inferred.
    pub fn set_slots(&mut self, slots: BTreeMap<String, egui::Rect>) -> Result<(), String> {
        if slots
            .iter()
            .any(|(name, rect)| !self.has_slot(name) || !rect.is_finite() || !rect.is_positive())
        {
            return Err("invalid or undeclared scene slot".into());
        }
        let slots = if let Some(frame) = self.frame.as_mut() {
            let (outer, inner) = slots
                .into_iter()
                .partition(|(name, _)| frame.has_slot(name));
            frame.set_slots(outer)?;
            inner
        } else {
            slots
        };
        if self.slots != slots {
            self.slots = slots;
            self.shown_revision = None;
        }
        Ok(())
    }
    /// Shared control admission uses the currently rendered definition/model.
    /// Applications still own the meaning and execution of admitted intents.
    pub fn admit_intent(&self, id: &str, action: &str, value: &Value) -> Result<(), String> {
        if let Some(frame) = self
            .frame
            .as_ref()
            .filter(|frame| id != "root" && frame.tree.nodes.contains_key(id))
        {
            return frame.admit_intent(id, action, value);
        }
        let node = self.tree.nodes.get(id).ok_or("unknown scene node")?;
        match node.family.as_str() {
            "control" => {
                Widget::decode(&node.ports["definition"])?.admit(&self.tree.model, action, value)
            }
            "split" => {
                let min = num(node, "minimum").unwrap_or(0.1);
                let max = num(node, "maximum").unwrap_or(0.9);
                if action == "set"
                    && value
                        .as_f64()
                        .is_some_and(|v| v.is_finite() && (min..=max).contains(&(v as f32)))
                {
                    Ok(())
                } else {
                    Err("invalid split intent".into())
                }
            }
            "dock" => {
                let child = value["child"].as_str().ok_or("dock child required")?;
                let placement = &node.ports["placements"][child];
                let min = placement["min_fraction"].as_f64().unwrap_or(0.05) as f32;
                let max = placement["max_fraction"].as_f64().unwrap_or(0.8) as f32;
                if action == "set"
                    && placement["resizable"] == true
                    && value["fraction"]
                        .as_f64()
                        .is_some_and(|v| v.is_finite() && (min..=max).contains(&(v as f32)))
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
        validate_model_budget(&model)?;
        let set = scene::bindings::BindingSet::from_resolved(&self.tree);
        let evaluated = scene::bindings::reevaluate_with_model_limit(
            &self.tree,
            &set,
            "model",
            &model,
            MAX_MODEL_BYTES,
        )
        .map_err(|e| format!("{e:?}"))?;
        if !evaluated.diagnostics.is_empty() {
            return Err(format!("{:?}", evaluated.diagnostics));
        }
        let candidate = evaluated.tree;
        let lists = validate_templates(&candidate).map_err(|e| e.to_string())?;
        validate_controls(&candidate)?;
        for id in &self.overlays {
            if !candidate
                .nodes
                .get(id)
                .and_then(|n| n.ports.get("definition"))
                .is_some_and(|definition| {
                    matches!(definition["kind"].as_str(), Some("menubar" | "command-bar"))
                })
            {
                return Err("hosted overlay control changed to an incompatible kind".into());
            }
        }
        if let Some(frame) = self.frame.as_mut() {
            frame.apply_model(model)?;
        }
        self.tree = candidate;
        self.lists = lists;
        self.revision += 1;
        Ok(())
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<Event> {
        ui.ctx()
            .data_mut(|d| d.insert_temp(egui::Id::unique("viu-editor-focused"), false));
        if let Some(mut frame) = self.frame.take() {
            let mut events = frame.show(ui);
            if let Some(rect) = frame.regions.get("gold-content").copied() {
                events.extend(
                    ui.scope_builder(egui::UiBuilder::new().max_rect(rect), |ui| {
                        ui.set_clip_rect(ui.clip_rect().intersect(rect));
                        self.show_content(ui)
                    })
                    .inner,
                );
            } else {
                self.geometry.clear();
                self.shown_revision = None;
            }
            self.geometry.extend(
                frame
                    .geometry
                    .iter()
                    .filter(|(id, _)| id.as_str() != "root")
                    .map(|(id, rect)| (id.clone(), *rect)),
            );
            self.frame = Some(frame);
            return events;
        }
        self.show_content(ui)
    }
    fn show_content(&mut self, ui: &mut egui::Ui) -> Vec<Event> {
        let mut events = Vec::new();
        self.regions.clear();
        self.geometry.clear();
        self.widgets.begin_frame();
        let tree = self.tree.clone();
        let nodes = tree.nodes.into_iter().collect();
        self.node(ui, "root", &nodes, None, 0, &mut events);
        for id in &self.overlays {
            let node = &nodes[id];
            if let Ok(widget) = Widget::decode(&node.ports["definition"]) {
                egui::Area::new(ui.scope_id().with(("viu-hosted-overlay", id)))
                    .fixed_pos(ui.ctx().content_rect().left_top())
                    .order(egui::Order::Foreground)
                    .show(ui.ctx(), |ui| {
                        for intent in self.widgets.widget(ui, &widget, &self.tree.model) {
                            if let Some(command) =
                                node.ports.get("on_intent").and_then(Value::as_str)
                            {
                                events.push(Event::from_intent(command, &self.tree.name, intent));
                            }
                        }
                    });
            }
        }
        self.geometry
            .extend(self.widgets.geometry.iter().map(|(id, r)| (id.clone(), *r)));
        self.focus_error = self.widgets.finish_frame().err();
        self.shown_revision = Some(self.revision);
        events
    }
    /// A model-ready endpoint precedes the native control pass. Hosts can
    /// fence input replies on this without claiming GPU or physical display.
    pub fn current_revision_drawn(&self) -> bool {
        self.shown_revision == Some(self.revision)
            && self
                .frame
                .as_ref()
                .is_none_or(|frame| frame.current_revision_drawn())
    }
    pub fn request_focus(&mut self, id: &str) -> Result<(), String> {
        if let Some(frame) = self
            .frame
            .as_mut()
            .filter(|frame| frame.control_kind(id).is_some())
        {
            return frame.request_focus(id);
        }
        if !self.geometry.contains_key(id) {
            return Err("focus target not visible".into());
        }
        self.widgets.request_focus(id, None)?;
        Ok(())
    }
    /// Renderer-neutral character selection. The application chooses the
    /// range; it carries no filename, path or other application interpretation.
    pub fn request_focus_value(&mut self, value: &Value) -> Result<(), String> {
        let target = value.as_str().or_else(|| value["id"].as_str());
        if let Some(frame) = self
            .frame
            .as_mut()
            .filter(|frame| target.is_some_and(|id| frame.control_kind(id).is_some()))
        {
            return frame.request_focus_value(value);
        }
        if let Some(id) = value.as_str() {
            return self.request_focus(id);
        }
        let object = value.as_object().ok_or("focus requires an ID or object")?;
        if object
            .keys()
            .any(|key| !matches!(key.as_str(), "id" | "selection"))
        {
            return Err("unknown focus option".into());
        }
        let id = value["id"].as_str().ok_or("focus requires an ID")?;
        let selection = if let Some(selection) = value.get("selection") {
            let range = selection
                .as_array()
                .filter(|r| r.len() == 2)
                .ok_or("selection requires two character offsets")?;
            let start = range[0]
                .as_u64()
                .ok_or("selection start must be an integer")?;
            let end = range[1]
                .as_u64()
                .ok_or("selection end must be an integer")?;
            if start > end || end > 1_048_576 {
                return Err("invalid selection range".into());
            }
            let node = self.tree.nodes.get(id).ok_or("unknown focus target")?;
            let kind = node
                .ports
                .get("definition")
                .and_then(|v| v["kind"].as_str());
            if !matches!(kind, Some("field" | "multiline" | "editor")) {
                return Err("selection target must be a text field".into());
            }
            let widget = Widget::decode(
                node.ports
                    .get("definition")
                    .ok_or("text field definition missing")?,
            )?;
            let len = widget
                .value(&self.tree.model)
                .as_str()
                .map(|s| s.chars().count())
                .unwrap_or(0);
            Some([(start as usize).min(len), (end as usize).min(len)])
        } else {
            None
        };
        self.request_focus(id)?;
        self.widgets.request_focus(id, selection)?;
        Ok(())
    }
    pub fn take_focus_error(&mut self) -> Option<String> {
        self.focus_error.take().or_else(|| {
            self.frame
                .as_mut()
                .and_then(|frame| frame.take_focus_error())
        })
    }
    pub fn focus_pending(&self) -> bool {
        self.widgets.focus.is_some()
            || self
                .frame
                .as_ref()
                .is_some_and(|frame| frame.focus_pending())
    }
    pub fn text_input_active(&self) -> bool {
        self.widgets.text_input_active
            || self
                .frame
                .as_ref()
                .is_some_and(|frame| frame.text_input_active())
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
        if self.overlays.contains(id) {
            return;
        }
        if node.ports.get("hidden").and_then(Value::as_bool) == Some(true) {
            return;
        }
        if node
            .ports
            .get("slot_only")
            .and_then(Value::as_str)
            .is_some_and(|name| !self.slots.contains_key(name))
        {
            return;
        }
        let allocated = ui.available_rect_before_wrap();
        if node.family == "dock" {
            paint_outline(ui, node, allocated);
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
                    "band"=>{
                        let area=ui.available_rect_before_wrap();
                        let measure=|child:&str| {
                            let node=&nodes[child];
                            let widget=if node.family=="control" {Widget::decode(&node.ports["definition"]).expect("validated label")}else{Widget::new(child,"label","",json!(text(node,"text"))).with_properties(json!({"size":num(node,"size").unwrap_or(13.0),"bold":node.ports.get("bold"),"mono":node.ports.get("mono")}))};
                            widget.intrinsic_width(ui,&self.tree.model).expect("validated intrinsic label")
                        };
                        let groups=band_groups(node,crate::window_frame::Metrics::of(ui.ctx()).caption_width*3.0,ui.ctx().data(|d| d.get_temp::<design::title_band::Alignment>(egui::Id::unique("viu-caption-alignment"))).unwrap_or_else(|| toolkit::chrome::Chrome::of(ui.ctx()).captions.alignment(design::title_band::Alignment::Right)),Some(ui.ctx()),&measure).expect("validated band groups");
                        let active=children(node).zip(groups).filter(|(child,_)|nodes.get(*child).is_some_and(|child|child.ports.get("hidden").and_then(Value::as_bool)!=Some(true)&&child.ports.get("slot_only").and_then(Value::as_str).is_none_or(|name|self.slots.contains_key(name)))).collect::<Vec<_>>();
                        let groups=active.iter().map(|(_,group)|*group).collect::<Vec<_>>();
                        let rectangles=design::title_band::layout(area.width(),&groups);
                        for ((child,_),rect) in active.into_iter().zip(rectangles) {
                            let rect=egui::Rect::from_min_size(egui::pos2(area.left()+rect.x,area.top()),Vec2::new(rect.width,area.height()));
                            self.child_rect(ui,child,rect,nodes,route,depth,events);
                        }
                        ui.allocate_rect(area,Sense::hover());
                    },
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
                        let spacing=ui.ctx().global_style().spacing.item_spacing;
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
                            .inner_margin(egui::Margin { left: node_padding(node, "padding_left") as i8, right: node_padding(node, "padding_right") as i8, top: node_padding(node, "padding_top") as i8, bottom: node_padding(node, "padding_bottom") as i8 })
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
                                    if let Some(height) = num(node, "height") {
                                        ui.allocate_ui_with_layout(Vec2::new(ui.available_width(), height), egui::Layout::left_to_right(egui::Align::Center), |ui| draw(ui));
                                    } else { ui.horizontal(|ui| draw(ui)); }
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
        if node.family != "dock" {
            paint_outline(ui, node, response.rect);
        }
        if node.family != "control" {
            self.geometry.insert(
                key,
                if node.family == "dock" {
                    allocated
                } else {
                    response.rect
                },
            );
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
        self.regions
            .insert(id.into(), rect.intersect(ui.clip_rect()));
        if let Some(node) = nodes.get(id) {
            if let Some(background) = color(text(node, "background")).or_else(|| {
                text(node, "background")
                    .strip_prefix("design.")
                    .and_then(|role| crate::appearance::colour(ui.ctx(), role))
            }) {
                ui.painter().rect_filled(
                    rect.intersect(ui.clip_rect()),
                    num(node, "radius").unwrap_or(0.0),
                    background,
                );
            }
        }
        ui.scope_builder(egui::UiBuilder::new().max_rect(rect).id_salt(id), |ui| {
            if nodes
                .get(id)
                .is_none_or(|node| node.ports.get("clip") != Some(&Value::Bool(false)))
            {
                ui.set_clip_rect(ui.clip_rect().intersect(rect));
            }
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
        let active = child
            .iter()
            .filter(|id| {
                nodes
                    .get(**id)
                    .is_some_and(|node| node.ports.get("hidden") != Some(&Value::Bool(true)))
            })
            .copied()
            .collect::<Vec<_>>();
        if active.len() < 2 {
            if let Some(child) = active.first() {
                self.child_rect(ui, child, area, nodes, route, depth, events);
            }
            ui.advance_cursor_after_rect(area);
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
            if let Some(rect) = nodes
                .get(child)
                .and_then(|node| node.ports.get("window_slot"))
                .and_then(Value::as_str)
                .and_then(|slot| self.slots.get(slot))
                .copied()
            {
                self.child_rect(ui, child, rect, nodes, route, depth, events);
                continue;
            }
            let placement = &placements[child];
            let edge = placement["edge"].as_str().unwrap_or("fill");
            if edge == "fill" && placement["panel"] == true {
                let mut panel_ui = ui.new_child(egui::UiBuilder::new().max_rect(remaining));
                egui::CentralPanel::default()
                    .frame(toolkit::bars::dock_frame(ui.ctx()))
                    .show(&mut panel_ui, |ui| {
                        self.node(ui, child, nodes, route, depth + 1, events)
                    });
                continue;
            }
            if placement["minimum_window_width"]
                .as_f64()
                .is_some_and(|w| (area.width() as f64) < w)
            {
                continue;
            }
            if matches!(edge, "bottom" | "left" | "right") && placement["panel"] == true {
                let horizontal = edge != "bottom";
                let dimension = if horizontal {
                    area.width()
                } else {
                    area.height()
                };
                let default_size =
                    placement["size"]
                        .as_f64()
                        .map(|n| n as f32)
                        .unwrap_or_else(|| {
                            dimension * placement["fraction"].as_f64().unwrap_or(0.2) as f32
                        });
                let mut panel_ui = ui.new_child(egui::UiBuilder::new().max_rect(remaining));
                let id = egui::Id::unique((key, child));
                let panel = match edge {
                    "left" => egui::Panel::left(id),
                    "right" => egui::Panel::right(id),
                    _ => egui::Panel::bottom(id),
                };
                let shown = panel
                    .resizable(placement["resizable"] == true)
                    .default_size(default_size)
                    .size_range(
                        0.0..=dimension * placement["max_fraction"].as_f64().unwrap_or(0.3) as f32,
                    )
                    .frame(toolkit::bars::dock_frame(ui.ctx()))
                    .show(&mut panel_ui, |ui| {
                        self.node(ui, child, nodes, route, depth + 1, events)
                    });
                if horizontal && placement["resizable"] == true {
                    let fraction = shown.response.rect.width() / area.width().max(1.0);
                    if (fraction - placement["fraction"].as_f64().unwrap_or(0.2) as f32).abs()
                        > 1.0 / area.width().max(1.0)
                    {
                        self.layout_change(
                            node,
                            key,
                            json!({"child":child,"fraction":fraction}),
                            events,
                        );
                    }
                }
                remaining = panel_ui.available_rect_before_wrap();
                continue;
            }
            if placement["minimum_window_width"]
                .as_f64()
                .is_some_and(|w| (area.width() as f64) < w)
            {
                continue;
            }
            let horizontal = ["left", "right"].contains(&edge);
            // A top strip may follow its intrinsic content height. This keeps
            // forms and search rows correct when live typography changes.
            if edge == "top" && placement["size"] == "content" {
                if let Some(background) = nodes.get(child).and_then(|node| {
                    color(text(node, "background")).or_else(|| {
                        text(node, "background")
                            .strip_prefix("design.")
                            .and_then(|role| crate::appearance::colour(ui.ctx(), role))
                    })
                }) {
                    ui.painter().rect_filled(remaining, 0.0, background);
                }
                let mut child_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(remaining)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                );
                child_ui.set_clip_rect(remaining.intersect(ui.clip_rect()));
                self.node(&mut child_ui, child, nodes, route, depth + 1, events);
                remaining.min.y = child_ui
                    .min_rect()
                    .bottom()
                    .clamp(remaining.top(), remaining.bottom());
                continue;
            }
            let dimension = if horizontal {
                area.width()
            } else {
                area.height()
            };
            let requested = placement["size"]
                .as_f64()
                .map(|v| v as f32)
                .or_else(|| design_length(&placement["size"], Some(ui.ctx())))
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
                    if placement["size"] == "design.status_height" {
                        use egui::emath::GuiRounding;
                        rect = rect.round_ui();
                    }
                    remaining.max.y = rect.min.y;
                }
                _ => {}
            }
            self.child_rect(ui, child, rect, nodes, route, depth, events);
            if edge == "bottom" && placement["size"] == "design.status_height" {
                // The native panel rounds its frame and integer margins after
                // applying live scale. Reserve its painted bounds, rather than
                // leaving a fractional strip between it and the document.
                if let Some(top) = nodes
                    .values()
                    .filter_map(|node| node.ports.get("definition"))
                    .filter(|definition| definition["kind"] == "status-line")
                    .filter_map(|definition| definition["id"].as_str())
                    .filter_map(|id| self.widgets.geometry.get(id))
                    .filter(|bounds| bounds.top() >= rect.top() && bounds.bottom() <= rect.bottom())
                    .map(egui::Rect::top)
                    .min_by(f32::total_cmp)
                {
                    remaining.max.y = top;
                }
            }
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
    let mut slots = std::collections::BTreeSet::new();
    for node in tree.nodes.values() {
        if let Some(slot) = node.ports.get("window_slot") {
            let name = slot
                .as_str()
                .filter(|name| !name.is_empty() && name.len() <= 128)
                .ok_or("invalid scene slot name")?;
            if !slots.insert(name) {
                return Err("duplicate scene slot declaration".into());
            }
        }
    }
    for (id, node) in &tree.nodes {
        if node
            .ports
            .get("slot_only")
            .is_some_and(|slot| slot.as_str().is_none_or(|name| !slots.contains(name)))
        {
            return Err("slot-only node requires a declared scene slot".into());
        }
        if node.ports.contains_key("window_slot")
            && !tree
                .nodes
                .values()
                .any(|parent| parent.family == "dock" && children(parent).any(|child| child == id))
        {
            return Err("scene slots must attach a dock child".into());
        }
        match node.family.as_str() {
            "band" => {
                band_groups(
                    node,
                    0.0,
                    design::title_band::Alignment::Right,
                    None,
                    &|_| 0.0,
                )?;
                for child in children(node) {
                    let group = &node.ports["groups"][child];
                    if group["preferred"] == "content" || group["minimum"] == "content" {
                        let child = &tree.nodes[child];
                        if child.family != "text"
                            && !(child.family == "control"
                                && child.ports.get("definition").is_some_and(|definition| {
                                    matches!(
                                        definition["kind"].as_str(),
                                        Some("label" | "command-bar" | "menubar")
                                    )
                                }))
                        {
                            return Err(
                                "intrinsic band content requires a text or label child".into()
                            );
                        }
                        if child.family == "text" && text(child, "text").len() > 4096 {
                            return Err(
                                "intrinsic band text exceeds the native label budget".into()
                            );
                        }
                    }
                }
            }
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
                let placements = node
                    .ports
                    .get("placements")
                    .and_then(Value::as_object)
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
                            if key == "size" && v == "content" && edge == "top" {
                                continue;
                            }
                            if key == "size" && design_length(v, None).is_some() {
                                continue;
                            }
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
                let widget = Widget::decode(
                    node.ports
                        .get("definition")
                        .ok_or("control definition is required")?,
                )?;
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

fn design_length(value: &Value, ctx: Option<&egui::Context>) -> Option<f32> {
    match value.as_str()? {
        "design.mark_slot" => Some(ctx.map_or(44.0, |ctx| {
            let c = toolkit::chrome::Chrome::of(ctx);
            c.metrics.mark + c.metrics.mark_gap + c.metrics.title_bar_margin
        })),
        "design.title_bar_height" => {
            Some(ctx.map_or(38.0, |ctx| crate::window_frame::Metrics::of(ctx).height))
        }
        "design.status_height" => Some(ctx.map_or(30.0, |ctx| {
            toolkit::chrome::Chrome::of(ctx).metrics.bars.status
        })),
        "design.dock_margin" => Some(ctx.map_or(2.0, |ctx| {
            toolkit::chrome::Chrome::of(ctx).metrics.dock_margin
        })),
        "design.options_height" => Some(ctx.map_or(42.0, |ctx| {
            toolkit::chrome::Chrome::of(ctx).metrics.bars.options
        })),
        _ => None,
    }
}

fn band_groups(
    node: &Node,
    caption_width: f32,
    caption_side: design::title_band::Alignment,
    ctx: Option<&egui::Context>,
    content_width: &dyn Fn(&str) -> f32,
) -> Result<Vec<design::title_band::Group>, String> {
    use design::title_band::{Alignment, Group};
    let groups = node
        .ports
        .get("groups")
        .and_then(Value::as_object)
        .ok_or("band groups must be an object")?;
    let children = children(node).collect::<Vec<_>>();
    if groups.len() != children.len() {
        return Err("band requires one group per child".into());
    }
    children.into_iter().map(|child|{
        let group=groups.get(child).ok_or("band child missing group")?;
        let length=|name:&str|if group[name]=="window.captions"{Ok(caption_width)}else if group[name]=="content"{Ok(content_width(child))}else if let Some(length)=design_length(&group[name],ctx){Ok(length)}else{group[name].as_f64().filter(|n|n.is_finite()&&*n>=0.0&&(*n as f32).is_finite()).map(|n|n as f32).ok_or_else(||format!("band {name} must be a finite nonnegative length, content, or window.captions metric"))};
        let chrome=ctx.map(toolkit::chrome::Chrome::of);
        let align=match group["align"].as_str(){Some("window.captions")=>caption_side,Some("window.menu")=>chrome.as_ref().map_or(Alignment::Left,|c|c.menu_side.alignment()),Some("window.actions")=>chrome.as_ref().map_or(Alignment::Right,|c|c.band.actions.alignment()),Some("window.title")=>chrome.as_ref().map_or(Alignment::Center,|c|c.band.title.alignment(c.band.style.title_alignment())),Some("left")=>Alignment::Left,Some("center")=>Alignment::Center,Some("right")=>Alignment::Right,_=>return Err("band alignment must be left, center, right or a native window alignment".into())};
        let integer=|name:&str|group[name].as_f64().filter(|n|n.is_finite()&&n.fract()==0.0&&(0.0..=255.0).contains(n)).map(|n|n as u8).ok_or_else(||format!("band {name} must be an integer 0..255"));
        let shrink=if group["shrink"].is_null(){None}else{Some(integer("shrink")?)};
        let order=integer("order")?;
        Ok(Group{preferred:length("preferred")?,minimum:length("minimum")?,align,shrink,order})
    }).collect()
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

fn node_padding(node: &Node, side: &str) -> f32 {
    num(node, side).or(num(node, "padding")).unwrap_or(0.0)
}

fn paint_outline(ui: &egui::Ui, node: &Node, rect: egui::Rect) {
    if let Some(outline) = color(text(node, "outline")).or_else(|| {
        text(node, "outline")
            .strip_prefix("design.")
            .and_then(|name| crate::appearance::colour(ui.ctx(), name))
    }) {
        ui.painter().rect_stroke(
            rect.shrink(0.5),
            2.0,
            egui::Stroke::new(num(node, "outline_width").unwrap_or(1.0), outline),
            egui::StrokeKind::Inside,
        );
    }
}
