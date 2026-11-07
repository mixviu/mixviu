// SPDX-License-Identifier: MIT OR Apache-2.0
//! Scene layout, binding/template preparation and event adaptation. Control
//! leaves and legacy text/button/field/toggle use the shared widget dispatch.
use crate::templates::{PreparedLists, children, rows, text, validate_templates};
use crate::widgets::{Intent, Widget, Widgets};
use egui::emath::GuiRounding;
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
    pub fn raster_states(&self) -> Value {
        self.widgets.raster_states()
    }
    pub fn set_rasters(&mut self, rasters: crate::raster::Rasters) {
        self.widgets.set_rasters(rasters.clone());
        if let Some(frame) = self.frame.as_mut() {
            frame.set_rasters(rasters);
        }
    }
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
                        // Native panels apply their own paint margins. Let a
                        // dock opt into that contract at the frame boundary.
                        if !self.tree.nodes.get("root").is_some_and(|node| {
                            node.family == "dock"
                                && node.ports.get("clip") == Some(&Value::Bool(false))
                        }) {
                            ui.set_clip_rect(ui.clip_rect().intersect(rect));
                        }
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
        if node.family == "dock" && node.ports.get("enabled") != Some(&Value::Bool(false)) {
            paint_outline(ui, node, allocated);
        }
        let key = route.map_or_else(
            || id.to_owned(),
            |(_, _, item)| format!("{}#{id}", item["id"].as_str().unwrap_or("")),
        );
        let response = ui
            .push_id(&key, |ui| {
                if node.ports.get("enabled") == Some(&Value::Bool(false)) { ui.disable(); if node.family == "dock" { paint_outline(ui,node,allocated); } }
                if let Some(width) = num(node, "width").or(num(node, "min_width")) {
                    ui.set_width(width);
                }
                if let Some(height) = num(node, "max_height") { ui.set_max_height(height); }
                if let Some(height) = num(node, "min_height") { ui.set_min_height(height); }
                if let Some(reserve) = num(node, "reserve_height") {
                    ui.set_height((ui.available_height()-reserve).max(num(node,"min_height").unwrap_or(0.)));
                }
                if matches!(node.family.as_str(),"panes"|"content"|"section"|"sides") || node.ports.get("native_spacing")==Some(&Value::Bool(true)) {
                    ui.spacing_mut().item_spacing=ui.ctx().global_style().spacing.item_spacing;
                } else if node.family!="control" {
                    ui.spacing_mut().item_spacing=Vec2::new(num(node,"column_gap").or(num(node,"gap")).unwrap_or(4.0),num(node,"row_gap").or(num(node,"gap")).unwrap_or(4.0));
                }
                match node.family.as_str() {
                    "sides"=>{
                        let children=children(node).collect::<Vec<_>>();
                        let draw=std::cell::RefCell::new((&mut *self,&mut *events));
                        let sides=if node.ports.get("wrap")==Some(&Value::Bool(true)){egui::Sides::new().wrap()}else{egui::Sides::new()};
                        sides.show(ui,|ui|{let mut draw=draw.borrow_mut();let (view,events)=&mut *draw;view.node(ui,children[0],nodes,route,depth+1,events);},|ui|{let mut draw=draw.borrow_mut();let (view,events)=&mut *draw;view.node(ui,children[1],nodes,route,depth+1,events);});
                    },
                    "panes"=>self.panes(ui,&key,node,nodes,route,depth,events),
                    "content"=>{
                        let content:settings::content::Content=serde_json::from_value(node.ports.get("settings").cloned().unwrap_or_else(|| json!({}))).expect("validated prose content");
                        let fill=if node.ports.get("surface").and_then(Value::as_str)==Some("editor") {ui.ctx().data(|d|d.get_temp::<editor::Palette>(egui::Id::unique("viu-gold-editor-palette"))).map(|p|p.background).unwrap_or(ui.visuals().panel_fill)} else if content.light {toolkit::chrome::Chrome::for_theme(toolkit::content::light_theme()).palette.canvas}else{ui.visuals().panel_fill};
                        toolkit::content::surface_filled(ui,&content,fill,|ui|{
                            let mut draw=|ui:&mut egui::Ui|for child in children(node){self.node(ui,child,nodes,route,depth+1,events);};
                            if node.ports.get("scroll")!=Some(&Value::Bool(false)){
                                let marker=egui::Id::unique((&key,"keyboard-scroll"));
                                let mut scroll=egui::ScrollArea::vertical().id_salt((&key,"body")).auto_shrink(false);
                                if let Some(target)=node.ports.get("keyboard_scroll").and_then(|v|v["focus"].as_str()) {
                                    if ui.memory(|m|m.has_focus(egui::Id::unique(target))) {
                                        let line=node.ports["keyboard_scroll"]["line"].as_f64().unwrap_or(48.0) as f32;
                                        let page=ui.available_height()*node.ports["keyboard_scroll"]["page_fraction"].as_f64().unwrap_or(0.9) as f32;
                                        let step=ui.input_mut(|i|{
                                            let mut step=0.0;
                                            for (key,delta) in [(egui::Key::ArrowDown,line),(egui::Key::ArrowUp,-line),(egui::Key::PageDown,page),(egui::Key::PageUp,-page)] {
                                                if i.consume_key(egui::Modifiers::NONE,key){step+=delta;}
                                            }
                                            step
                                        });
                                        if step!=0.0 {
                                            if let Some(id)=ui.ctx().data(|d|d.get_temp::<egui::Id>(marker)) {
                                                if let Some(state)=egui::scroll_area::State::load(ui.ctx(),id){scroll=scroll.vertical_scroll_offset((state.clamped_offset().y+step).max(0.0));}
                                            }
                                        }
                                    }
                                }
                                let shown=scroll.show(ui,|ui|draw(ui));
                                ui.ctx().data_mut(|d|d.insert_temp(marker,shown.id));
                            }else{draw(ui);}
                        });
                    },
                    "section"=>{
                        ui.spacing_mut().item_spacing=ui.ctx().global_style().spacing.item_spacing;
                        let titles=node.ports["titles"].as_array().expect("validated section titles").iter().map(|v|v.as_str().expect("validated section title")).collect::<Vec<_>>();
                        let mut draw=|ui:&mut egui::Ui| {
                            let mut actions=Vec::new();
                            let shown=if let Some(items)=node.ports.get("actions").and_then(Value::as_array) {
                                toolkit::panel::group_with_actions(ui,&key,|ui|{
                                    ui.horizontal(|ui|for item in items {
                                        if item["separator"]==true {toolkit::bars::divider(ui,16.0);continue;}
                                        if let Some(icon)=item["icon"].as_str().and_then(crate::icons::Icon::from_name) {
                                            if let Some(children)=item["children"].as_array() {
                                                let button=ui.add_enabled(item["enabled"]!=false,toolkit::button::IconButton::new(icon).size(ui.spacing().interact_size.y*0.8));
                                                egui::Popup::menu(&button).show(|ui|for child in children {
                                                    if ui.add_enabled(child["enabled"]!=false,egui::Button::new(child["label"].as_str().unwrap_or(""))).clicked(){actions.push(child["id"].clone());}
                                                });
                                                continue;
                                            }
                                            let r=ui.add_enabled(item["enabled"]!=false,toolkit::button::Link::new(icon,item["label"].as_str().unwrap_or("")));
                                            if r.clicked(){actions.push(item["id"].clone());}
                                        }
                                    });
                                },|ui,_|for child in children(node){self.node(ui,child,nodes,route,depth+1,events);})
                            }else{
                                toolkit::panel::group(ui,&key,&titles,|ui,_|{
                                    for child in children(node){self.node(ui,child,nodes,route,depth+1,events);}
                                })
                            };
                            if let Some(command)=node.ports.get("on_action").and_then(Value::as_str) {
                                for action in actions {events.push(Event{command:command.into(),body:json!({"id":action,"origin":"panel"})});}
                            }
                            if shown.handle.drag_stopped() {
                                if let Some(command)=node.ports.get("on_change").and_then(Value::as_str) {
                                    events.push(Event{command:command.into(),body:json!({"id":key,"selected":shown.state.selected,"collapsed":shown.state.collapsed})});
                                }
                            }
                        };
                        if text(node,"frame")=="dock" {
                            let focused=node.ports.get("focused")==Some(&Value::Bool(true));
                            let ring=if focused{let palette=&toolkit::chrome::Chrome::of(ui.ctx()).palette;palette.card_border.lerp_to_gamma(palette.accent,0.25)}else{Color32::TRANSPARENT};
                            let frame=toolkit::bars::dock_frame(ui.ctx());
                            let frame=if node.ports.contains_key("focused"){frame.stroke(egui::Stroke::new(1.0,ring))}else{frame};
                            frame.show(ui,|ui|draw(ui));
                        }else{draw(ui);}
                    },
                    "column" if !text(node,"group_title").is_empty() => {
                        let title=text(node,"group_title");
                        let group_key=if text(node,"group_key").is_empty(){&key}else{text(node,"group_key")};
                        let draw=|ui:&mut egui::Ui| {
                            ui.spacing_mut().item_spacing=ui.ctx().global_style().spacing.item_spacing;
                            let mut group = |ui: &mut egui::Ui| toolkit::saved::widgets::group(ui, group_key, &[title], |ui,_| {
                                for child in children(node) {
                                    self.node(ui, child, nodes, route, depth+1, events);
                                }
                            });
                            if node.ports.get("group_frame")==Some(&Value::Bool(false)){ui.scope(|ui|group(ui)).response.rect}else{toolkit::bars::dock_frame(ui.ctx()).show(ui, |ui| group(ui)).response.rect}
                        };
                        let rect=if node.ports.get("scroll").and_then(Value::as_bool)==Some(true) {
                            let mut draw=draw;
                            egui::ScrollArea::vertical().id_salt((&key,"group-scroll")).auto_shrink([false,false]).show(ui,|ui| draw(ui)).inner
                        } else { let mut draw=draw; draw(ui) };
                        ui.interact(rect,ui.scope_id().with((&key,"group")),Sense::hover());
                    }
                    "column" if node.ports.get("scroll").and_then(Value::as_bool)==Some(true) => {
                        let mut draw_scroll=|ui: &mut egui::Ui| {
                        let height=ui.available_height();
                        let context=node.ports.get("scroll_key").and_then(Value::as_str).filter(|s|!s.is_empty());
                        let ready=node.ports.get("scroll_ready").and_then(Value::as_bool)!=Some(false);
                        let after=if node.ports.get("reserve_panel_after").and_then(Value::as_bool)==Some(true){toolkit::chrome::Chrome::of(ui.ctx()).style.panel_card_after as f32}else{0.};
                        let rect=toolkit::saved::widgets::scroll(ui,&key,context,ready,egui::ScrollArea::vertical().auto_shrink(false).max_height(height),|ui|{
                            ui.set_max_height((height-after).max(0.));
                            for child in children(node){self.node(ui,child,nodes,route,depth+1,events);}
                        }).inner_rect;
                        rect
                        };
                        let rect=if node.ports.get("native_frame")==Some(&Value::Bool(true)){toolkit::bars::dock_frame(ui.ctx()).show(ui,|ui|draw_scroll(ui)).inner}else{draw_scroll(ui)};
                        ui.interact(rect,ui.scope_id().with((&key,"scroll")),Sense::hover());
                    }

                    "dock"=>self.dock(ui,&key,node,nodes,route,depth,events),
                    "band"=>{
                        let area=ui.available_rect_before_wrap();
                        let measure=|child:&str,minimum:bool| {
                            let node=&nodes[child];
                            let widget=if node.family=="control" {Widget::decode(&node.ports["definition"]).expect("validated label")}else{Widget::new(child,"label","",json!(text(node,"text"))).with_properties(json!({"size":num(node,"size").unwrap_or(13.0),"bold":node.ports.get("bold"),"mono":node.ports.get("mono")}))};
                            if minimum {widget.minimum_width(ui,&self.tree.model)}else{widget.intrinsic_width(ui,&self.tree.model)}.expect("validated intrinsic label")
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
                        let colour=color(text(node,"color")).or_else(||text(node,"color").strip_prefix("design.").and_then(|role|crate::appearance::colour(ui.ctx(),role))).unwrap_or(ui.visuals().text_color()).to_array();
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
                        if text(node,"packing")=="toolbar" {
                            ui.spacing_mut().item_spacing=ui.ctx().global_style().spacing.item_spacing;
                            ui.allocate_ui_with_layout(ui.available_size_before_wrap(),egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let end=text(node,"end");
                                if nodes.get(end).is_some_and(|n|n.ports.get("hidden")!=Some(&json!(true))) {
                                    self.node(ui,end,nodes,route,depth+1,events);
                                    ui.add_space(8.0);
                                }
                                ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                                    ui.set_clip_rect(ui.max_rect().intersect(ui.clip_rect()));
                                    let flex=text(node,"flex");
                                    let reserve=nodes.get(flex).filter(|n|n.ports.get("hidden")!=Some(&json!(true))).map_or(0.0, |_|80.0+ui.spacing().item_spacing.x);
                                    for child in children(node).filter(|c|*c!=end) {
                                        let n=&nodes[child];
                                        if n.ports.get("hidden")==Some(&json!(true)) {continue;}
                                        if node.ports.get("fit").and_then(Value::as_array).is_some_and(|items|items.iter().any(|v|v==child)) {
                                            let widget=Widget::decode(&n.ports["definition"]).expect("validated toolbar control");
                                            let width=widget.intrinsic_width(ui,&self.tree.model).expect("validated toolbar width");
                                            if ui.available_width()<width+ui.spacing().item_spacing.x+reserve {continue;}
                                        }
                                        if child==flex {
                                            let width=(ui.available_width()-2.0).clamp(80.0,220.0);
                                            let area=egui::Rect::from_min_size(ui.cursor().min,egui::vec2(width,ui.available_height()));
                                            self.child_rect(ui,child,area,nodes,route,depth,events);
                                            ui.allocate_rect(area,egui::Sense::hover());
                                        } else {self.node(ui,child,nodes,route,depth+1,events);}
                                    }
                                });
                            });
                            return;
                        }
                        if ["options","tools"].contains(&text(node,"bar")) {
                            let mut draw=|ui:&mut egui::Ui| {
                                for child in children(node) {self.node(ui,child,nodes,route,depth+1,events);}
                            };
                            if text(node,"bar")=="options" {toolkit::bars::options(ui,&mut draw);}else{toolkit::bars::tools(ui,&mut draw);}
                            return;
                        }
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
                                    if node.ports.get("wrap").and_then(Value::as_bool)==Some(true) {
                                        ui.horizontal_wrapped(|ui| {
                                            for child in children(node) {
                                                let Some(control)=nodes.get(child) else {continue};
                                                if control.ports.get("hidden")==Some(&Value::Bool(true)){continue;}
                                                if node.ports.get("native_spacing")==Some(&Value::Bool(true)) && control.family=="control" {
                                                    match Widget::decode(&control.ports["definition"]) {
                                                        Ok(widget)=>for intent in self.widgets.widget_inline(ui,&widget,&self.tree.model) {
                                                            if let Some(command)=control.ports.get("on_intent").and_then(Value::as_str){events.push(Event::from_intent(command,&self.tree.name,intent));}
                                                        },
                                                        Err(error)=>{ui.colored_label(Color32::RED,error);}
                                                    }
                                                } else {self.node(ui,child,nodes,route,depth+1,events);}
                                            }
                                        });
                                    } else if text(node,"justify")=="between" {
                                        let active:Vec<_>=children(node).filter(|child|nodes.get(*child).is_some_and(|n|n.ports.get("hidden").and_then(Value::as_bool)!=Some(true))).collect();
                                        ui.horizontal(|ui| {if let Some(first)=active.first(){self.node(ui,first,nodes,route,depth+1,events);ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{for child in active.iter().skip(1).rev(){self.node(ui,child,nodes,route,depth+1,events);}});}});
                                    } else if children(node).any(|child|nodes.get(child).is_some_and(|n|num(n,"grow").unwrap_or(0.)>0.)) {
                                        let active:Vec<_>=children(node).filter(|child|nodes.get(*child).is_some_and(|n|n.ports.get("hidden").and_then(Value::as_bool)!=Some(true))).collect();
                                        let fixed:Vec<_>=active.iter().map(|child|{let n=&nodes[*child];if num(n,"grow").unwrap_or(0.)>0.{0.}else{num(n,"width").or(num(n,"min_width")).unwrap_or_else(||if n.family=="control"{Widget::decode(&n.ports["definition"]).and_then(|w|w.intrinsic_width(ui,&self.tree.model)).unwrap_or(0.)}else{0.})}}).collect();
                                        let weight:f32=active.iter().map(|child|num(&nodes[*child],"grow").unwrap_or(0.).max(0.)).sum();
                                        let remaining=(ui.available_width()-fixed.iter().sum::<f32>()-ui.spacing().item_spacing.x*active.len().saturating_sub(1) as f32).max(0.);
                                        ui.horizontal(|ui|for (i,child) in active.iter().enumerate(){let grow=num(&nodes[*child],"grow").unwrap_or(0.);let width=if grow>0.{remaining*grow/weight}else{fixed[i]};ui.allocate_ui(egui::vec2(width,0.),|ui|{ui.set_min_width(width);self.node(ui,child,nodes,route,depth+1,events);});});
                                    } else if text(node,"align")=="end" {
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|for child in children(node).collect::<Vec<_>>().into_iter().rev(){self.node(ui,child,nodes,route,depth+1,events);});
                                    } else if let Some(height) = num(node, "height") {
 ui.allocate_ui_with_layout(Vec2::new(ui.available_width(),height),egui::Layout::left_to_right(egui::Align::Center),|ui|draw(ui));
 } else { ui.horizontal(|ui| draw(ui)); }
                                } else if node.ports.get("scroll")==Some(&Value::Bool(true)) {
                                    egui::ScrollArea::vertical().id_salt((&key,"scroll")).auto_shrink(false).show(ui,|ui|draw(ui));


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
            if nodes.get(id).is_none_or(|node| {
                node.ports.get("clip") != Some(&Value::Bool(false))
                    || num(node, "clip_below")
                        .is_some_and(|width| ui.ctx().content_rect().width() < width)
            }) {
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

    /// Native resizable navigation/detail panes. Applications provide every
    /// child and persist the returned fractions; this owns only layout.
    fn panes(
        &mut self,
        ui: &mut egui::Ui,
        key: &str,
        node: &Node,
        nodes: &BTreeMap<String, Node>,
        route: Option<(&str, &Node, &Value)>,
        depth: usize,
        events: &mut Vec<Event>,
    ) {
        if node.ports.get("clip") == Some(&Value::Bool(false)) {
            ui.set_clip_rect(ui.ctx().content_rect());
        }
        let children = children(node).collect::<Vec<_>>();
        let width = ui.available_width();
        if node.ports.get("keyboard") == Some(&Value::Bool(true)) && !ui.ctx().text_edit_focused() {
            let targets = node.ports.get("focus_targets").cloned().unwrap_or_default();
            // A compact pane is mounted on the frame after its intent is
            // applied. Retake its focus then: the target was absent in the
            // frame that consumed Tab, so egui may have retired that request.
            let marker = egui::Id::unique((ui.ctx().viewport_id(), key, "pane-model-focus"));
            let model_focus = text(node, "focus");
            let previous_model = ui.ctx().data_mut(|d| {
                let previous = d.get_temp::<String>(marker);
                d.insert_temp(marker, model_focus.to_owned());
                previous
            });
            if previous_model.is_some_and(|previous| previous != model_focus) {
                if let Some(target) = targets[model_focus].as_str() {
                    if let Some(id) = target.strip_prefix("table:") {
                        toolkit::datatable::focus(ui.ctx(), id);
                    } else {
                        ui.memory_mut(|m| m.request_focus(egui::Id::unique(target)));
                    }
                }
            }
            let names = ["left", "center", "right"];
            let count = if node.ports.get("right_visible") == Some(&Value::Bool(false)) {
                2
            } else {
                3
            };
            let mut focus = text(node, "focus");
            let previous = ui.input_mut(|i| i.consume_key(egui::Modifiers::SHIFT, egui::Key::Tab));
            let next =
                !previous && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Tab));
            if previous || next {
                let at = names[..count].iter().position(|n| *n == focus).unwrap_or(0);
                focus = names[(at + count + if next { 1 } else { count - 1 }) % count];
                if let Some(target) = targets[focus].as_str() {
                    if let Some(id) = target.strip_prefix("table:") {
                        toolkit::datatable::focus(ui.ctx(), id);
                    } else {
                        ui.memory_mut(|m| m.request_focus(egui::Id::unique(target)));
                    }
                }
            } else {
                for name in &names[..count] {
                    if let Some(target) = targets[*name].as_str() {
                        let focused = if let Some(id) = target.strip_prefix("table:") {
                            toolkit::datatable::is_focused(ui.ctx(), id)
                        } else {
                            ui.memory(|m| m.has_focus(egui::Id::unique(target)))
                        };
                        if focused {
                            focus = name;
                            break;
                        }
                    }
                }
            }
            if focus != text(node, "focus") {
                self.layout_change(node, key, json!({"focus":focus}), events);
            }
        }
        let pane_ctx = ui.ctx().clone();
        let frame = |which: &str| {
            let ring = if text(node, "focus") == which {
                let palette = &toolkit::chrome::Chrome::of(&pane_ctx).palette;
                palette.card_border.lerp_to_gamma(palette.accent, 0.25)
            } else {
                Color32::TRANSPARENT
            };
            toolkit::bars::dock_frame(&pane_ctx).stroke(egui::Stroke::new(1.0, ring))
        };
        let left_frame = frame("left");
        let right_frame = frame("right");
        let center_frame = frame("center");
        if width < num(node, "compact_below").unwrap_or(0.0) {
            let focus = if text(node, "focus") == "right"
                && node.ports.get("right_visible") == Some(&Value::Bool(false))
            {
                "center"
            } else {
                text(node, "focus")
            };
            egui::Panel::top(egui::Id::unique((key, "compact-panes"))).show(ui, |ui| {
                ui.horizontal(|ui| {
                    for (name, label) in
                        [("left", "Left"), ("center", "Centre"), ("right", "Right")]
                    {
                        if name == "right"
                            && node.ports.get("right_visible") == Some(&Value::Bool(false))
                        {
                            continue;
                        }
                        let label = node
                            .ports
                            .get("pane_labels")
                            .and_then(|v| v[name].as_str())
                            .unwrap_or(label);
                        if toolkit::selectable(ui, focus == name, label).clicked() {
                            self.layout_change(node, key, json!({"focus":name}), events);
                        }
                    }
                });
            });
            egui::CentralPanel::default()
                .frame(frame(focus))
                .show(ui, |ui| {
                    self.node(
                        ui,
                        children[if focus == "left" {
                            0
                        } else if focus == "right" {
                            2
                        } else {
                            1
                        }],
                        nodes,
                        route,
                        depth + 1,
                        events,
                    );
                });
            return;
        }
        let narrow = width < num(node, "narrow_below").unwrap_or(0.0);
        let mut left_panel = egui::Panel::left(egui::Id::unique((
            ui.ctx().viewport_id(),
            key,
            if narrow { "left-narrow" } else { "left" },
        )))
        .resizable(true)
        .default_size(width * num(node, "left_fraction").unwrap_or(0.18))
        .frame(left_frame);
        if narrow {
            let range = node
                .ports
                .get("narrow_left_range")
                .cloned()
                .unwrap_or_default();
            let low = range[0].as_f64().unwrap_or(0.0).clamp(0.0, 1.0) as f32;
            let high = range[1].as_f64().unwrap_or(1.0).clamp(low as f64, 1.0) as f32;
            left_panel = left_panel.size_range(width * low..=width * high);
        }
        let left = left_panel.show(ui, |ui| {
            self.node(ui, children[0], nodes, route, depth + 1, events)
        });
        let left_fraction = left.response.rect.width() / width.max(1.0);
        let mut right_fraction = num(node, "right_fraction").unwrap_or(0.42);
        if node.ports.get("right_visible") != Some(&Value::Bool(false)) {
            let remaining = ui.available_width();
            let mut right_panel = egui::Panel::right(egui::Id::unique((
                ui.ctx().viewport_id(),
                key,
                if narrow { "right-narrow" } else { "right" },
            )))
            .resizable(true)
            .default_size(remaining * right_fraction)
            .frame(right_frame);
            if narrow {
                let range = node
                    .ports
                    .get("narrow_right_range")
                    .cloned()
                    .unwrap_or_default();
                let low = range[0].as_f64().unwrap_or(0.0).clamp(0.0, 1.0) as f32;
                let high = range[1].as_f64().unwrap_or(1.0).clamp(low as f64, 1.0) as f32;
                right_panel = right_panel.size_range(remaining * low..=remaining * high);
            }
            let right = right_panel.show(ui, |ui| {
                self.node(ui, children[2], nodes, route, depth + 1, events)
            });
            right_fraction = right.response.rect.width() / remaining.max(1.0);
        }
        egui::CentralPanel::default()
            .frame(center_frame)
            .show(ui, |ui| {
                self.node(ui, children[1], nodes, route, depth + 1, events)
            });
        if !narrow
            && ((left_fraction - num(node, "left_fraction").unwrap_or(0.18)).abs() > 0.005
                || (right_fraction - num(node, "right_fraction").unwrap_or(0.42)).abs() > 0.005)
        {
            self.layout_change(
                node,
                key,
                json!({"left":left_fraction,"right":right_fraction}),
                events,
            );
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
        if let Some(target) = node.ports.get("focus_target").and_then(Value::as_str) {
            let area = ui.max_rect();
            let id = egui::Id::unique(target);
            let back = ui.interact(area, id, Sense::focusable_noninteractive());
            if ui.input(|i| i.pointer.primary_pressed()) && ui.rect_contains_pointer(area) {
                back.request_focus();
            }
            if ui.memory(|m| m.has_focus(id)) {
                ui.memory_mut(|m| {
                    m.set_focus_lock_filter(
                        id,
                        egui::EventFilter {
                            tab: true,
                            horizontal_arrows: true,
                            vertical_arrows: true,
                            ..Default::default()
                        },
                    )
                });
            }
        }
        let area = ui.available_rect_before_wrap();
        let mut remaining = area;
        let placements = &node.ports["placements"];
        if node.ports.get("clip") == Some(&Value::Bool(true)) {
            ui.set_clip_rect(ui.clip_rect().intersect(area));
        }
        if node.ports.get("passthrough_single") == Some(&Value::Bool(true)) {
            let visible = children(node)
                .filter(|id| {
                    nodes
                        .get(*id)
                        .is_some_and(|n| n.ports.get("hidden") != Some(&Value::Bool(true)))
                })
                .collect::<Vec<_>>();
            if visible.len() == 1 && placements[visible[0]]["edge"] == "fill" {
                self.node(ui, visible[0], nodes, route, depth + 1, events);
                return;
            }
        }
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
            if edge == "fill" && placement["passthrough"] == true {
                self.node(ui, child, nodes, route, depth + 1, events);
                continue;
            }
            if matches!(edge, "bottom" | "fill") && placement["panel"] == true {
                let mut panel_ui = ui.new_child(egui::UiBuilder::new().max_rect(remaining));
                let panel = egui::CentralPanel::default();
                let panel = if placement["frame"] == "native" {
                    panel
                } else {
                    panel.frame(toolkit::bars::dock_frame(ui.ctx()))
                };
                panel.show(&mut panel_ui, |ui| {
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
            if matches!(edge, "top" | "bottom" | "left" | "right") && placement["panel"] == true {
                let horizontal = matches!(edge, "left" | "right");
                let dimension = if horizontal {
                    area.width()
                } else {
                    area.height()
                };
                let measured = placement["measure"]
                    .as_str()
                    .and_then(|id| nodes.get(id))
                    .and_then(|node| Widget::decode(&node.ports["definition"]).ok())
                    .and_then(|widget| widget.intrinsic_width(ui, &self.tree.model).ok());
                let default_size = measured.unwrap_or_else(|| {
                    placement["size"]
                        .as_f64()
                        .map(|n| n as f32)
                        .unwrap_or_else(|| {
                            dimension * placement["fraction"].as_f64().unwrap_or(0.2) as f32
                        })
                });
                let mut panel_ui = ui.new_child(egui::UiBuilder::new().max_rect(remaining));
                let id = egui::Id::unique((key, child));
                let panel = match edge {
                    "left" => egui::Panel::left(id),
                    "right" => egui::Panel::right(id),
                    "top" => egui::Panel::top(id),
                    _ => egui::Panel::bottom(id),
                };
                let panel = panel.resizable(placement["resizable"] == true);
                let panel = if placement["size"] == "content" && !horizontal {
                    panel
                } else {
                    panel.default_size(default_size)
                };
                let panel = if placement["exact"] == true {
                    panel.exact_size(default_size)
                } else {
                    panel.size_range(
                        0.0..=dimension * placement["max_fraction"].as_f64().unwrap_or(0.3) as f32,
                    )
                };
                let panel = if placement["frame"] == "native" {
                    panel
                } else {
                    panel.frame(toolkit::bars::dock_frame(ui.ctx()))
                };
                let shown = panel.show(&mut panel_ui, |ui| {
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
                            json!({"child":child,"fraction":serde_json::from_str::<Value>(&fraction.to_string()).expect("finite panel fraction")}),
                            events,
                        );
                    }
                }
                remaining = panel_ui.available_rect_before_wrap();
                continue;
            }
            if edge == "fill" && placement["native_panel"] == true {
                egui::CentralPanel::default()
                    .frame(toolkit::bars::dock_frame(ui.ctx()))
                    .show(ui, |ui| {
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
            let horizontal = ["left", "right"].contains(&edge);
            if edge == "bottom" && placement["size"] == "content" && placement["measure"] == true {
                let limit =
                    remaining.height() * placement["max_fraction"].as_f64().unwrap_or(1.0) as f32;
                let marker =
                    egui::Id::unique((ui.ctx().viewport_id(), "viu.dock.intrinsic-height"));
                ui.ctx().data_mut(|d| d.insert_temp(marker, limit));
                let mut measure = ui.new_child(
                    egui::UiBuilder::new()
                        .id_salt((child, "measure"))
                        .max_rect(remaining)
                        .layout(egui::Layout::top_down(egui::Align::Min))
                        .invisible()
                        .sizing_pass(),
                );
                self.node(
                    &mut measure,
                    child,
                    nodes,
                    route,
                    depth + 1,
                    &mut Vec::new(),
                );
                let height = measure.min_rect().height().ceil().min(limit);
                let mut panel_ui = ui.new_child(egui::UiBuilder::new().max_rect(remaining));
                let panel = egui::Panel::bottom(egui::Id::unique((key, child, "intrinsic")))
                    .frame(egui::Frame::new())
                    .exact_size(height)
                    .show(&mut panel_ui, |ui| {
                        self.node(ui, child, nodes, route, depth + 1, events);
                    });
                ui.ctx().data_mut(|d| {
                    d.remove::<f32>(marker);
                });
                remaining = panel_ui.available_rect_before_wrap();
                remaining.max.y = remaining.max.y.min(panel.response.rect.top());
                continue;
            }
            if edge == "bottom" && placement["size"] == "content" {
                // Reserve the last measured height, then repaint at the new
                // height when live text or typography changes the footer.
                let marker = egui::Id::unique((ui.ctx().viewport_id(), key, child, "dock-height"));
                let height = ui.ctx().data(|d| d.get_temp::<f32>(marker)).unwrap_or(0.0);
                let mut rect = remaining;
                rect.min.y = (rect.max.y - height).max(rect.min.y);
                let mut child_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(rect)
                        .layout(egui::Layout::top_down(egui::Align::Min)),
                );
                child_ui.set_clip_rect(remaining.intersect(ui.clip_rect()));
                self.node(&mut child_ui, child, nodes, route, depth + 1, events);
                let measured = child_ui.min_rect().height().min(remaining.height());
                if (measured - height).abs() > 0.1 {
                    ui.ctx().data_mut(|d| d.insert_temp(marker, measured));
                    ui.ctx().request_repaint();
                }
                remaining.max.y = rect.min.y;
                continue;
            }
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
                if placement["native_spacing"] == true {
                    remaining.min.y = (remaining.min.y
                        + ui.ctx().global_style().spacing.item_spacing.y)
                        .min(remaining.max.y);
                }
                continue;
            }
            let dimension = placement["reference_extent"]
                .as_f64()
                .map(|n| n as f32)
                .unwrap_or_else(|| {
                    if horizontal {
                        area.width()
                    } else {
                        area.height()
                    }
                });
            let requested = placement["size"]
                .as_f64()
                .map(|v| v as f32)
                .or_else(|| design_length(&placement["size"], Some(ui.ctx())))
                .unwrap_or_else(|| {
                    placement["fraction"].as_f64().unwrap_or(0.2) as f32 * dimension
                });
            if placement["native_panel"] == true && edge != "fill" {
                let name = placement["name"].as_str().unwrap_or(child);
                let panel = match edge {
                    "left" => egui::Panel::left(name),
                    "right" => egui::Panel::right(name),
                    "top" => egui::Panel::top(name),
                    _ => egui::Panel::bottom(name),
                }
                .frame(toolkit::bars::dock_frame(ui.ctx()));
                if placement["resizable"] == true {
                    let (_, changed) = toolkit::saved::widgets::panel(
                        ui,
                        name,
                        panel,
                        dimension,
                        placement["fraction"].as_f64().unwrap_or(0.2) as f32,
                        placement["minimum"].as_f64().unwrap_or(0.05) as f32
                            ..=placement["maximum"].as_f64().unwrap_or(0.8) as f32,
                        horizontal,
                        |ui| self.node(ui, child, nodes, route, depth + 1, events),
                    );
                    if let (Some(fraction), Some(command)) =
                        (changed, node.ports["on_change"].as_str())
                    {
                        events.push(Event { command: command.into(), body: json!({"node":key,"kind":"dock","value":{"child":child,"fraction":fraction}}) });
                    }
                } else {
                    panel.resizable(false).exact_size(requested).show(ui, |ui| {
                        self.node(ui, child, nodes, route, depth + 1, events)
                    });
                }
                remaining = ui.available_rect_before_wrap();
                continue;
            }
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
            // Native panels round their outer bounds before placing contents.
            // Keep the neighbouring scene regions on the same layout grid.
            rect = rect.round_ui();
            match edge {
                "left" => remaining.min.x = rect.max.x,
                "right" => remaining.max.x = rect.min.x,
                "top" => remaining.min.y = rect.max.y,
                "bottom" => remaining.max.y = rect.min.y,
                _ => {}
            }
            if placement["clip"] == false || placement["clip"] == "vertical" {
                ui.scope_builder(egui::UiBuilder::new().max_rect(rect).id_salt(child), |ui| {
                    if placement["clip"] == "vertical" {
                        let mut clip = ui.clip_rect();
                        clip.min.y = clip.min.y.max(rect.min.y);
                        clip.max.y = clip.max.y.min(rect.max.y);
                        ui.set_clip_rect(clip);
                    }
                    self.node(ui, child, nodes, route, depth + 1, events);
                });
            } else {
                self.child_rect(ui, child, rect, nodes, route, depth, events);
            }

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
            "row" if text(node, "packing") == "toolbar" => {
                let kids = children(node).collect::<Vec<_>>();
                for name in ["end", "flex"] {
                    if node
                        .ports
                        .get(name)
                        .is_some_and(|v| v.as_str().is_none_or(|s| !kids.contains(&s)))
                    {
                        return Err("toolbar role must name a child".into());
                    }
                }
                if node
                    .ports
                    .get("fit")
                    .and_then(Value::as_array)
                    .is_some_and(|items| {
                        items.iter().any(|v| {
                            v.as_str().is_none_or(|s| {
                                !kids.contains(&s)
                                    || ![Some("combo"), Some("swatches")].contains(
                                        &tree.nodes[s].ports["definition"]["kind"].as_str(),
                                    )
                            })
                        })
                    })
                {
                    return Err("toolbar fit requires combo or swatches children".into());
                }
            }
            "row" | "column"
                if node
                    .ports
                    .get("bar")
                    .is_some_and(|v| ![Some("options"), Some("tools")].contains(&v.as_str())) =>
            {
                return Err("unknown native bar role".into());
            }
            "band" => {
                band_groups(
                    node,
                    0.0,
                    design::title_band::Alignment::Right,
                    None,
                    &|_, _| 0.0,
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
                    if !p["reference_extent"].is_null()
                        && p["reference_extent"]
                            .as_f64()
                            .is_none_or(|n| !n.is_finite() || n <= 0.0)
                    {
                        return Err("dock reference_extent must be a positive finite number".into());
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
                            if key == "size" && v == "content" && matches!(edge, "top" | "bottom") {
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
            "panes" => {
                if children(node).count() != 3 {
                    return Err("panes requires left, center, right children".into());
                }
                for key in ["left_fraction", "right_fraction"] {
                    if num(node, key).is_none_or(|v| !(0.05..=0.95).contains(&v)) {
                        return Err("pane fractions must be 0.05..0.95".into());
                    }
                }
            }
            "sides" => {
                if children(node).count() != 2 {
                    return Err("sides requires left and right children".into());
                }
            }
            "content" => {
                let content: settings::content::Content = serde_json::from_value(
                    node.ports
                        .get("settings")
                        .cloned()
                        .unwrap_or_else(|| json!({})),
                )
                .map_err(|e| e.to_string())?;
                if !content.font.size.is_finite()
                    || !(6.0..=48.0).contains(&content.font.size)
                    || ![1, 2, 4, 8, 16].contains(&content.padding)
                    || content
                        .font
                        .family
                        .as_deref()
                        .is_some_and(|f| !["Inter", "JetBrains Mono"].contains(&f))
                {
                    return Err("invalid prose content settings".into());
                }
            }
            "section" => {
                let titles = node
                    .ports
                    .get("titles")
                    .and_then(Value::as_array)
                    .filter(|t| !t.is_empty() && t.len() <= 32)
                    .ok_or("section requires 1..32 titles")?;
                if titles
                    .iter()
                    .any(|t| t.as_str().is_none_or(|s| s.len() > 4096))
                {
                    return Err("invalid section title".into());
                }
            }
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
            use egui::emath::GuiRounding;
            Some(ctx.map_or(38.0, |ctx| {
                crate::window_frame::Metrics::of(ctx).height.round_ui()
            }))
        }
        "design.status_height" => Some(ctx.map_or(30.0, |ctx| {
            toolkit::chrome::Chrome::of(ctx).metrics.bars.status
        })),
        "design.dock_margin" => Some(ctx.map_or(2.0, |ctx| {
            toolkit::chrome::Chrome::of(ctx).metrics.dock_margin
        })),
        "design.collapsed_dock" => Some(ctx.map_or(42.0, |ctx| {
            let chrome = toolkit::chrome::Chrome::of(ctx);
            chrome.style.icon_button as f32 + 2.0 * chrome.metrics.dock_margin
        })),
        "design.options_height" => Some(ctx.map_or(42.0, |ctx| {
            toolkit::chrome::Chrome::of(ctx).metrics.bars.options
        })),
        "design.tools_width" => Some(ctx.map_or(50.0, |ctx| {
            toolkit::chrome::Chrome::of(ctx).metrics.bars.tool
        })),
        "design.tool_margin" => Some(ctx.map_or(8.0, |ctx| {
            toolkit::chrome::Chrome::of(ctx).metrics.bars.tool_margin
        })),
        "design.bar_inset" => Some(ctx.map_or(8.0, |ctx| {
            toolkit::chrome::Chrome::of(ctx).style.bars_bar_inset as f32
        })),
        "design.bar_padding_y" => Some(ctx.map_or(6.0, |ctx| {
            toolkit::chrome::Chrome::of(ctx).style.bar_padding_y as f32
        })),
        _ => None,
    }
}

fn band_groups(
    node: &Node,
    caption_width: f32,
    caption_side: design::title_band::Alignment,
    ctx: Option<&egui::Context>,
    content_width: &dyn Fn(&str, bool) -> f32,
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
        let length=|name:&str|if group[name]=="window.captions"{Ok(caption_width)}else if group[name]=="content"{Ok(content_width(child,false))}else if group[name]=="content.minimum"{Ok(content_width(child,true))}else if let Some(length)=design_length(&group[name],ctx){Ok(length)}else{group[name].as_f64().filter(|n|n.is_finite()&&*n>=0.0&&(*n as f32).is_finite()).map(|n|n as f32).ok_or_else(||format!("band {name} must be a finite nonnegative length, content, or window.captions metric"))};
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
