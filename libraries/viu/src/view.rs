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

fn model_dependency_changed(deps: &BTreeSet<String>, before: &Value, after: &Value) -> bool {
    deps.iter().any(|dep| {
        dep.strip_prefix("model.")
            .and_then(|path| path.split('.').next())
            .is_none_or(|key| before.get(key) != after.get(key))
    })
}

#[cfg(feature = "test-support")]
thread_local! { static PROFILE_APPLY: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
#[cfg(feature = "test-support")]
pub(crate) fn profile_apply(enabled: bool) {
    PROFILE_APPLY.set(enabled);
}

fn validate_model_budget(model: &Value) -> Result<usize, String> {
    if !model.is_object() {
        return Err("scene model must be an object".into());
    }
    let bytes = serde_json::to_vec(model).map_err(|e| e.to_string())?;
    if bytes.len() > MAX_MODEL_BYTES {
        return Err("scene model exceeds 8 MiB data budget".into());
    }
    Ok(bytes.len())
}

#[derive(Clone, Debug)]
pub struct Event {
    pub command: String,
    pub body: Value,
}

pub struct SceneView {
    motion: toolkit::scene_motion::SceneMotion,
    numbers: crate::numbers::Numbers,
    backdrops: crate::backdrop::Backdrops,
    scene_images: crate::image_view::Images,
    motion_parent: Option<String>,
    collect_widget_rects: bool,
    widget_rects: Vec<crate::widget_rects::WidgetRect>,
    frame: Option<Box<SceneView>>,
    regions: BTreeMap<String, egui::Rect>,
    stretch_heights: BTreeMap<String, f32>,
    tree: ResolvedScene,
    bindings: scene::bindings::BindingSet,
    model_bytes: usize,
    lists: PreparedLists,
    pub revision: u64,
    pub geometry: BTreeMap<String, egui::Rect>,
    widgets: Widgets,
    focus_error: Option<String>,
    shown_revision: Option<u64>,
    font_families: Option<BTreeSet<String>>,
    slots: BTreeMap<String, egui::Rect>,
    overlays: BTreeSet<String>,
    #[cfg(test)]
    evaluated_last_apply: usize,
}

impl SceneView {
    /// Capture geometry during the next pass only. Ordinary frames do no
    /// rectangle-table construction or readback.
    pub fn request_widget_rectangles(&mut self) {
        self.collect_widget_rects = true;
        if let Some(frame) = self.frame.as_mut() {
            frame.request_widget_rectangles();
        }
    }
    /// The most recent requested capture; the host requests a new pass before
    /// answering each batch of Bus queries.
    pub fn widget_rectangles(&self, args: &Value) -> Result<Value, String> {
        let mut rows = self.widget_rects.clone();
        if let Some(frame) = &self.frame {
            rows.extend(
                frame
                    .widget_rects
                    .iter()
                    .filter(|r| r.id != "root")
                    .cloned(),
            );
        }
        crate::widget_rects::query(&rows, args)
    }
    pub fn set_terminals(&mut self, terminals: crate::terminal::sessions::Sessions) {
        self.widgets.set_terminals(terminals.clone());
        if let Some(frame) = self.frame.as_mut() {
            frame.set_terminals(terminals);
        }
    }
    pub fn raster_states(&self) -> Value {
        self.widgets.raster_states()
    }
    pub fn set_rasters(&mut self, rasters: crate::raster::Rasters) {
        self.scene_images.set_rasters(Some(rasters.clone()));
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
        validate_controls(&tree, None)?;
        let bindings = scene::bindings::BindingSet::from_resolved(&tree);
        let model_bytes = crate::model_patch::model_bytes(&tree.model);
        Ok(Self {
            motion: toolkit::scene_motion::SceneMotion::default(),
            numbers: crate::numbers::Numbers::default(),
            backdrops: crate::backdrop::Backdrops::default(),
            scene_images: crate::image_view::Images::scene(),
            motion_parent: None,
            frame: None,
            regions: BTreeMap::new(),
            stretch_heights: BTreeMap::new(),
            tree,
            bindings,
            model_bytes,
            lists,
            revision: 0,
            geometry: BTreeMap::new(),
            widget_rects: Vec::new(),
            collect_widget_rects: false,
            widgets: Widgets::default(),
            focus_error: None,
            shown_revision: None,
            font_families: None,
            slots: BTreeMap::new(),
            overlays: BTreeSet::new(),
            #[cfg(test)]
            evaluated_last_apply: 0,
        })
    }

    pub fn model(&self) -> &Value {
        &self.tree.model
    }
    #[cfg(feature = "test-support")]
    pub(crate) fn publication_equivalent(&self, other: &Self) -> bool {
        self.tree == other.tree
            && self.lists == other.lists
            && match (&self.frame, &other.frame) {
                (Some(a), Some(b)) => a.publication_equivalent(b),
                (None, None) => true,
                _ => false,
            }
    }
    /// Stage data edits and binding ports, validate, then commit the frame
    /// before the root. No fallible work follows the frame commit.
    pub(crate) fn apply_model_patch(
        &mut self,
        patch: &crate::model_patch::Patch,
    ) -> Result<(), String> {
        #[cfg(feature = "test-support")]
        let started = PROFILE_APPLY
            .get()
            .then(crate::app_host::test_support::thread_cpu);
        let edit = crate::model_patch::Edit::apply(&mut self.tree.model, self.model_bytes, patch)?;
        #[cfg(feature = "test-support")]
        let edited = started.map(|_| crate::app_host::test_support::thread_cpu());
        let mut displaced = BTreeMap::new();
        let result = (|| {
            let nodes =
                scene::bindings::prepare_model_roots(&self.tree, &self.bindings, &patch.roots())
                    .map_err(|e| format!("{e:?}"))?;
            for (id, node) in nodes {
                displaced.insert(id.clone(), self.tree.nodes.insert(id, node).unwrap());
            }
            #[cfg(feature = "test-support")]
            let rebound = started.map(|_| crate::app_host::test_support::thread_cpu());
            let lists = validate_templates(&self.tree).map_err(|e| e.to_string())?;
            validate_controls(&self.tree, self.font_families.as_ref())?;
            self.validate_hosted_overlays(&self.tree)?;
            #[cfg(feature = "test-support")]
            let validated = started.map(|_| crate::app_host::test_support::thread_cpu());
            if let Some(frame) = self.frame.as_mut() {
                frame.apply_model_patch(patch)?;
            }
            self.lists = lists;
            #[cfg(feature = "test-support")]
            if let (Some(started), Some(edited), Some(rebound), Some(validated)) =
                (started, edited, rebound, validated)
            {
                let end = crate::app_host::test_support::thread_cpu();
                eprintln!(
                    "WORKSHOP_PATCH {}",
                    json!({"nodes":self.tree.nodes.len(),"model_bytes":edit.bytes,"edit_budget_cpu_us":(edited-started).as_micros(),"binding_copy_cpu_us":(rebound-edited).as_micros(),"templates_controls_cpu_us":(validated-rebound).as_micros(),"frame_commit_cpu_us":(end-validated).as_micros(),"total_cpu_us":(end-started).as_micros()})
                );
            }
            Ok(())
        })();
        match result {
            Ok(()) => {
                self.model_bytes = edit.bytes;
                self.revision += 1;
                Ok(())
            }
            Err(error) => {
                edit.rollback(&mut self.tree.model);
                for (id, node) in displaced {
                    self.tree.nodes.insert(id, node);
                }
                Err(error)
            }
        }
    }
    fn validate_hosted_overlays(&self, candidate: &ResolvedScene) -> Result<(), String> {
        for id in &self.overlays {
            if !candidate
                .nodes
                .get(id)
                .and_then(|n| n.ports.get("definition"))
                .is_some_and(|d| matches!(d["kind"].as_str(), Some("menubar" | "command-bar")))
            {
                return Err("hosted overlay control changed to an incompatible kind".into());
            }
        }
        Ok(())
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
        #[cfg(feature = "test-support")]
        let probe = PROFILE_APPLY.get().then(|| {
            (
                std::time::Instant::now(),
                crate::app_host::test_support::thread_cpu(),
            )
        });
        let model_bytes = validate_model_budget(&model)?;
        #[cfg(feature = "test-support")]
        let validated = probe.map(|_| {
            (
                std::time::Instant::now(),
                crate::app_host::test_support::thread_cpu(),
            )
        });
        let mut set = scene::bindings::BindingSet::from_resolved(&self.tree);
        // A full model replacement need not evaluate every binding. Compare
        // the top-level value containing each recorded dependency; whole-model
        // and dynamic reads remain conservative. Validation still runs below.
        set.order.retain(|path| {
            let binding = &set.bindings[path];
            model_dependency_changed(&binding.deps, &self.tree.model, &model)
        });
        let evaluated = if set.order.is_empty() {
            let mut tree = self.tree.clone();
            tree.model = model.clone();
            scene::bindings::ReEval {
                tree,
                diagnostics: vec![],
                changed: vec![],
                evaluated: vec![],
            }
        } else {
            scene::bindings::reevaluate_with_model_limit(
                &self.tree,
                &set,
                "model",
                &model,
                MAX_MODEL_BYTES,
            )
            .map_err(|e| format!("{e:?}"))?
        };
        #[cfg(test)]
        {
            self.evaluated_last_apply = evaluated.evaluated.len();
        }
        if !evaluated.diagnostics.is_empty() {
            return Err(format!("{:?}", evaluated.diagnostics));
        }
        let candidate = evaluated.tree;
        #[cfg(feature = "test-support")]
        let diffed = probe.map(|_| {
            (
                std::time::Instant::now(),
                crate::app_host::test_support::thread_cpu(),
            )
        });
        let lists = validate_templates(&candidate).map_err(|e| e.to_string())?;
        validate_controls(&candidate, self.font_families.as_ref())?;
        #[cfg(feature = "test-support")]
        let rebuilt = probe.map(|_| {
            (
                std::time::Instant::now(),
                crate::app_host::test_support::thread_cpu(),
            )
        });
        self.validate_hosted_overlays(&candidate)?;
        if let Some(frame) = self.frame.as_mut() {
            frame.apply_model(model)?;
        }
        self.model_bytes = model_bytes;
        self.tree = candidate;
        self.lists = lists;
        self.revision += 1;
        #[cfg(feature = "test-support")]
        if let (Some(start), Some(validated), Some(diffed), Some(rebuilt)) =
            (probe, validated, diffed, rebuilt)
        {
            let end = (
                std::time::Instant::now(),
                crate::app_host::test_support::thread_cpu(),
            );
            let span = |a: (std::time::Instant, std::time::Duration),
                        b: (std::time::Instant, std::time::Duration)| json!({"wall_us":b.0.duration_since(a.0).as_micros() as u64,"cpu_us":b.1.saturating_sub(a.1).as_micros() as u64});
            eprintln!(
                "WORKSHOP_APPLY {}",
                json!({"nodes":self.tree.nodes.len(),"model_bytes":serde_json::to_vec(&self.tree.model).unwrap().len(),"evaluated":evaluated.evaluated.len(),"changed":evaluated.changed.len(),"validate":span(start,validated),"bindings_diff":span(validated,diffed),"templates_controls":span(diffed,rebuilt),"frame_commit":span(rebuilt,end),"total":span(start,end)})
            );
        }
        Ok(())
    }

    /// Admit named text families against the renderer's actual bound font set.
    /// Retain that set for transactional model/patch validation without file IO.
    pub fn bind_fonts(&mut self, ctx: &egui::Context) -> Result<(), String> {
        if let Some(frame) = self.frame.as_mut() {
            frame.bind_fonts(ctx)?;
        }
        if !self
            .tree
            .nodes
            .values()
            .any(|node| node.ports.contains_key("font") || node.ports.contains_key("mono_font"))
        {
            return Ok(());
        }
        let families = ctx.fonts(|fonts| {
            fonts
                .definitions()
                .families
                .keys()
                .filter_map(|family| {
                    if let egui::FontFamily::Name(name) = family {
                        Some(name.to_string())
                    } else {
                        None
                    }
                })
                .collect::<BTreeSet<_>>()
        });
        if self.font_families.as_ref() != Some(&families) {
            validate_controls(&self.tree, Some(&families))?;
            self.font_families = Some(families);
        }
        Ok(())
    }

    pub fn show(&mut self, ui: &mut egui::Ui) -> Vec<Event> {
        if let Err(error) = self.bind_fonts(ui.ctx()) {
            ui.colored_label(Color32::RED, error);
            return Vec::new();
        }
        toolkit::scene_motion::install(ui.ctx());
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
                self.widget_rects.clear();
                self.collect_widget_rects = false;
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
        self.motion.begin(ui);
        self.numbers.begin();
        self.backdrops.begin();
        self.scene_images.begin();
        if self.collect_widget_rects {
            self.widget_rects.clear();
            ui.ctx()
                .data_mut(|d| d.remove::<String>(crate::widgets::focus_key()));
        }
        let mut events = Vec::new();
        self.regions.clear();
        self.geometry.clear();
        self.widgets.begin_frame();
        let tree = self.tree.clone();
        let nodes = tree.nodes.into_iter().collect();
        self.node(ui, "root", &nodes, None, 0, &mut events);
        self.motion.end(ui);
        self.numbers.end();
        self.backdrops.end();
        self.scene_images.finish();
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
        if self.collect_widget_rects {
            let focused = ui
                .ctx()
                .data(|d| d.get_temp::<String>(crate::widgets::focus_key()));
            let zoom = ui.ctx().zoom_factor();
            let logical = |rect: egui::Rect| {
                egui::Rect::from_min_max(
                    (rect.min.to_vec2() * zoom).to_pos2(),
                    (rect.max.to_vec2() * zoom).to_pos2(),
                )
            };
            for row in &mut self.widget_rects {
                row.focused = focused.as_deref() == Some(row.id.as_str());
                if let Some(rect) = self.geometry.get(&row.id) {
                    row.rect = *rect;
                }
                row.rect = logical(row.rect);
                row.clip = logical(row.clip);
            }
            self.collect_widget_rects = false;
        }
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
        let Some(node) = nodes.get(id) else {
            return;
        };
        let list_row = route.is_some_and(|(_, list, _)| text(list, "row") == id);
        if !list_row
            && self.motion_parent.is_none()
            && ![
                "enter",
                "exit",
                "opacity",
                "offset",
                "transition",
                "paint_effect",
            ]
            .iter()
            .any(|p| node.ports.contains_key(*p))
        {
            self.node_contents(ui, id, nodes, route, depth, events);
            return;
        }
        if node.ports.get("hidden") == Some(&Value::Bool(true)) {
            return;
        }
        let key = route.map_or_else(
            || id.to_owned(),
            |(_, _, item)| format!("{}#{id}", item["id"].as_str().unwrap_or("")),
        );
        let start = ui.ctx().graphics_mut(|g| g.entry(ui.layer_id()).next_idx());
        let enabled = node.ports.get("motion") != Some(&Value::Bool(false))
            && ui.style().animation_time != 0.0;
        let opacity = toolkit::scene_motion::value(
            ui.ctx(),
            &key,
            "opacity",
            node.ports.get("opacity").unwrap_or(&json!(1.0)),
            node.ports.get("transition"),
            enabled,
        )
        .as_f64()
        .unwrap_or(1.0);
        let offset = toolkit::scene_motion::value(
            ui.ctx(),
            &key,
            "offset",
            node.ports.get("offset").unwrap_or(&json!([0.0, 0.0])),
            node.ports.get("transition"),
            enabled,
        );
        let default_fade = json!("fade");
        let mut capture = self.motion.capture(
            ui,
            &key,
            self.motion_parent.as_deref(),
            node.ports
                .get("enter")
                .or(if list_row { Some(&default_fade) } else { None }),
            node.ports.get("exit").or(if list_row {
                Some(&default_fade)
            } else {
                Some(&Value::Null)
            }),
            node.ports.get("motion") != Some(&Value::Bool(false)),
        );
        let offset = [
            offset[0].as_f64().unwrap_or(0.),
            offset[1].as_f64().unwrap_or(0.),
        ];
        let layer = self
            .motion
            .layer(ui, &mut capture, ui.available_rect_before_wrap(), offset)
            .or_else(|| toolkit::scene_motion::offset_layer(ui, &key, offset));
        let parent = self.motion_parent.clone();
        if capture.is_some() {
            self.motion_parent = Some(key.clone());
        }
        let paint_start = ui
            .ctx()
            .graphics_mut(|g| g.entry(layer.unwrap_or(ui.layer_id())).next_idx());
        ui.scope_builder(
            egui::UiBuilder::new().layer_id(layer.unwrap_or(ui.layer_id())),
            |ui| {
                ui.multiply_opacity(opacity.clamp(0., 1.) as f32);
                self.node_contents(ui, id, nodes, route, depth, events);
            },
        );
        self.motion_parent = parent;
        let painted_rect = self.geometry.get(&key).copied().unwrap_or(ui.min_rect());
        if let Some(spec) = node.ports.get("paint_effect") {
            toolkit::scene_motion::paint_effect(
                ui,
                &key,
                layer.unwrap_or(ui.layer_id()),
                paint_start,
                painted_rect,
                spec,
            );
        }
        toolkit::scene_motion::highlight(
            ui,
            layer.unwrap_or(ui.layer_id()),
            if layer.is_some() {
                egui::layers::ShapeIdx(0)
            } else {
                start
            },
            &key,
            node.ports.get("transition"),
            enabled,
        );
        self.motion.painted(ui, capture, painted_rect);
        if let Some(layer) = layer
            && let Some(rect) = self.geometry.get_mut(&key)
        {
            *rect = ui
                .ctx()
                .layer_transform_to_global(layer)
                .unwrap_or_default()
                .mul_rect(*rect);
        }
    }

    fn node_contents(
        &mut self,
        ui: &mut egui::Ui,
        id: &str,
        nodes: &BTreeMap<String, Node>,
        route: Option<(&str, &Node, &Value)>,
        depth: usize,
        events: &mut Vec<Event>,
    ) {
        let Some(node) = nodes.get(id) else {
            return;
        };
        if node.ports.get("hidden") == Some(&Value::Bool(true)) {
            return;
        }
        let width = matches!(node.family.as_str(), "column" | "row")
            .then(|| {
                num(node, "width").or_else(|| {
                    (node.ports.get("fill") == Some(&Value::Bool(true))
                        || num(node, "grow").unwrap_or(0.) > 0.
                        || num(node, "shrink").unwrap_or(0.) > 0.)
                        .then(|| ui.available_width())
                })
            })
            .flatten();
        if let Some(width) = width {
            let width = bounded_layout(width, 1e6)
                .max(num(node, "min_width").unwrap_or(0.))
                .min(num(node, "max_width").unwrap_or(1e6));
            let available = ui.available_rect_before_wrap();
            let x = match ui.layout().horizontal_placement() {
                egui::Align::Center => available.center().x - width / 2.,
                egui::Align::Max => available.right() - width,
                _ => available.left(),
            };
            let rect = egui::Rect::from_min_size(
                egui::pos2(x, available.top()),
                egui::vec2(width, available.height()),
            );
            let mut cell = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            cell.set_clip_rect(ui.clip_rect().intersect(egui::Rect::from_min_max(
                egui::pos2(x, ui.clip_rect().top()),
                egui::pos2(x + width, ui.clip_rect().bottom()),
            )));
            self.node_surface(&mut cell, id, nodes, route, depth, events);
            let height = cell.min_rect().height();
            ui.allocate_rect(
                egui::Rect::from_min_size(rect.min, egui::vec2(width, height)),
                Sense::hover(),
            );
        } else {
            self.node_surface(ui, id, nodes, route, depth, events);
        }
    }

    fn node_surface(
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
        let mut painted_node = std::borrow::Cow::Borrowed(node);
        for prop in ["color", "background", "hover"]
            .into_iter()
            .filter(|_| node.ports.contains_key("transition"))
        {
            if let Some(target) = node.ports.get(prop) {
                painted_node.to_mut().ports.insert(
                    prop.into(),
                    toolkit::scene_motion::value(
                        ui.ctx(),
                        id,
                        prop,
                        target,
                        node.ports.get("transition"),
                        node.ports.get("motion") != Some(&Value::Bool(false))
                            && ui.style().animation_time != 0.0,
                    ),
                );
            }
        }
        let node = &painted_node;
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
        let mut response = ui
            .push_id(&key, |ui| {
                if node.ports.get("motion") == Some(&Value::Bool(false)) { ui.style_mut().animation_time = 0.0; }
                let _surface_policy=toolkit::scene_motion::policy(ui);
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
                    "column" if node.ports.get("scroll").and_then(Value::as_bool)==Some(true)
                        && (node.ports.contains_key("scroll_key") || node.ports.get("native_frame")==Some(&Value::Bool(true))) => {
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
                        let frame = if node.ports.get("native_frame")==Some(&Value::Bool(true)) {
                            toolkit::bars::dock_frame(ui.ctx())
                        } else {
                            let background = color(text(node,"background")).or_else(||text(node,"background").strip_prefix("design.").and_then(|role|crate::appearance::colour(ui.ctx(),role))).unwrap_or(Color32::TRANSPARENT);
                            egui::Frame::new().fill(background).inner_margin(egui::Margin {
                                left: node_padding(node,"padding_left") as i8,
                                right: node_padding(node,"padding_right") as i8,
                                top: node_padding(node,"padding_top") as i8,
                                bottom: node_padding(node,"padding_bottom") as i8,
                            }).corner_radius(num(node,"radius").unwrap_or(0.0))
                        };
                        let rect=frame.show(ui,|ui|draw_scroll(ui)).inner;
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
                        let columns=(num(node,"columns").unwrap_or(2.0) as usize).clamp(1,16);
                        let gap=num(node,"column_gap").unwrap_or(12.0);
                        let width=((ui.available_width()-gap*(columns-1) as f32)/columns as f32).max(num(node,"min_column_width").unwrap_or(0.0));
                        let origin=ui.available_rect_before_wrap().min;
                        let mut top=origin.y;
                        let active:Vec<_>=children(node).collect();
                        for (row,group) in active.chunks(columns).enumerate() {
                            let cache=ui.scope_id().with((&key,"grid-height",row));
                            let previous=ui.ctx().data(|d|d.get_temp::<f32>(cache)).unwrap_or(0.);
                            let mut height=0.0_f32;
                            for (column,child) in group.iter().enumerate() {
                                let rect=egui::Rect::from_min_size(egui::pos2(origin.x+column as f32*(width+gap),top),egui::vec2(width,ui.available_height().max(previous)));
                                let mut cell=ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(egui::Layout::top_down(egui::Align::Min)));
                                cell.set_width(width);
                                cell.set_clip_rect(ui.clip_rect());
                                self.node(&mut cell,child,nodes,route,depth+1,events);
                                height=height.max(cell.min_rect().height());
                            }
                            if (height-previous).abs()>0.01 {ui.ctx().data_mut(|d|d.insert_temp(cache,height));ui.ctx().request_discard("scene layout measurement");}
                            top+=height+num(node,"row_gap").unwrap_or(3.);
                        }
                        let height=if active.is_empty(){0.}else{top-origin.y-num(node,"row_gap").unwrap_or(3.)};
                        ui.allocate_space(egui::vec2(columns as f32*width+gap*(columns-1) as f32,height));
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
                        if let Some(width)=num(node,"max_width") {ui.set_max_width(width.min(ui.available_width()));}
                        let colour=color(text(node,"color")).or_else(||text(node,"color").strip_prefix("design.").and_then(|role|crate::appearance::colour(ui.ctx(),role))).unwrap_or(ui.visuals().text_color()).to_array();
                        let number = node.ports.get("number").map(|spec| self.numbers.text(ui, &key, spec, node.ports.get("motion") != Some(&Value::Bool(false))));
                        let plain = json!([{"text":number.as_deref().unwrap_or_else(||text(node,"text"))}]);
                        {
                            let spans = node.ports.get("spans").unwrap_or(&plain);
                            let family = (node.ports.get("font").and_then(Value::as_str), num(node,"weight").unwrap_or(if node.ports.get("bold") == Some(&Value::Bool(true)) {700.} else {400.}), node.ports.get("mono_font").and_then(Value::as_str));
                            let mut job = crate::spans::job(ui, spans, num(node,"size").unwrap_or(13.0), node.ports.get("mono") == Some(&Value::Bool(true)), node.ports.get("bold") == Some(&Value::Bool(true)), Color32::from_rgba_unmultiplied(colour[0], colour[1], colour[2], colour[3]), family);
                            job.halign = match text(node,"align") {"center"=>egui::Align::Center,"right"=>egui::Align::Max,_=>egui::Align::Min};
                            for section in &mut job.sections { section.format.extra_letter_spacing = num(node,"letter_spacing").unwrap_or(0.); section.format.line_height=num(node,"line_height").map(|v|v.clamp(0.1,4096.)); }
                            job.justify=false;job.wrap.max_width=if node.ports.get("wrap")==Some(&Value::Bool(true)){ui.available_width()}else{f32::INFINITY};
                            let galley=ui.fonts_mut(|f|f.layout_job(job));
                            let label = egui::Label::new(galley);
                            let label=if node.ports.get("wrap") == Some(&Value::Bool(true)) {label.wrap()} else {label.extend()};
                            if node.ports.get("fill")==Some(&Value::Bool(true)) {ui.add_sized(egui::vec2(ui.available_width(),0.),label);} else {ui.add(label);}
                        }
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
                    "drawing" => {
                        crate::drawing::show(ui, &key, node);
                    }
                    "image" => {
                        if text(node,"src")=="brand:cyclops" {
                            let w=num(node,"w").unwrap_or(200.0);
                            let h=num(node,"h").unwrap_or(w);
                            ui.add(egui::Image::from_bytes("bytes://viu/cyclops.svg",include_bytes!("../../../share/brand/logos/alien-cyclops.svg").as_slice()).fit_to_exact_size(Vec2::new(w,h)));
                        } else if let Some(icon)=text(node,"src").strip_prefix("icon:").and_then(crate::icons::Icon::from_name) {
                            let w=num(node,"w").or(num(node,"h")).unwrap_or(24.0);
                            let h=num(node,"h").unwrap_or(w);
                            ui.add(crate::icons::image_of(ui.ctx(),icon,w,ui.visuals().text_color()).fit_to_exact_size(Vec2::new(w,h)));
                        } else if text(node,"src").starts_with("raster:") {
                            let w=num(node,"w").unwrap_or(320.0);
                            let h=num(node,"h").unwrap_or(180.0);
                            ui.allocate_ui(Vec2::new(w,h), |ui| {
                                self.scene_images.draw(ui,id,&json!({"phase":"ready","handle":text(node,"src"),"scene_size":[w,h]}));
                            });
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
                        let radial_shape = ui.painter().add(egui::Shape::Noop);
                        let frame_width = num(node,"width").or(num(node,"min_width")).or_else(|| (node.ports.get("fill")==Some(&Value::Bool(true)) || num(node,"grow").unwrap_or(0.)>0. || num(node,"shrink").unwrap_or(0.)>0.).then(||ui.available_width()));
                        let bound_width=num(node,"width").or_else(||(node.ports.get("fill")==Some(&Value::Bool(true)) || num(node,"grow").unwrap_or(0.)>0. || num(node,"shrink").unwrap_or(0.)>0.).then(||ui.available_width().max(num(node,"min_width").unwrap_or(0.))));
                        let frame = egui::Frame::new()
                            .fill(Color32::TRANSPARENT)
                            .inner_margin(egui::Margin { left: node_padding(node, "padding_left") as i8, right: node_padding(node, "padding_right") as i8, top: node_padding(node, "padding_top") as i8, bottom: node_padding(node, "padding_bottom") as i8 })
                            .corner_radius(num(node, "radius").unwrap_or(0.0));
                        let mut response = frame
                            .show(ui, |ui| {
                                if let Some(width)=frame_width {
                                    let inner=(width-node_padding(node,"padding_left")-node_padding(node,"padding_right")).max(0.);
                                    ui.set_min_width(inner);
                                }
                                if let Some(width)=bound_width {
                                    let inner=(width-node_padding(node,"padding_left")-node_padding(node,"padding_right")).max(0.);
                                    ui.set_max_width(inner);

                                }
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
                                let assigned_height = self.stretch_heights.get(id).copied();
                                let mut draw = |ui: &mut egui::Ui| {
                                    for child in children(node) {
                                        self.node(ui, child, nodes, route, depth + 1, events);
                                    }
                                };
                                if node.family == "row" {
                                    if node.ports.get("wrap").and_then(Value::as_bool)==Some(true) && node.ports.get("native_spacing")!=Some(&Value::Bool(true)) {
                                        let origin=ui.available_rect_before_wrap().min;let available=ui.available_width();let gap=ui.spacing().item_spacing;
                                        let active:Vec<_>=children(node).filter(|c|nodes.get(*c).is_some_and(|n|n.ports.get("hidden")!=Some(&Value::Bool(true)))).collect();
                                        let sizes:Vec<egui::Vec2>=active.iter().map(|c|{let old=ui.ctx().data(|d|d.get_temp::<egui::Vec2>(ui.scope_id().with((&key,"wrap-size",c)))).unwrap_or_default();egui::vec2(if nodes[*c].ports.get("fill")==Some(&Value::Bool(true)){available}else{intrinsic_text_width(ui,&nodes[*c],nodes,&self.tree.model,false,0)},old.y)}).collect();
                                        let mut rows:Vec<Vec<usize>>=vec![];let mut current=vec![];let mut used=0.;
                                        for (i,size) in sizes.iter().enumerate(){if !current.is_empty() && used+size.x>available+0.01{rows.push(std::mem::take(&mut current));used=0.;}current.push(i);used+=size.x+gap.x;}
                                        if !current.is_empty(){rows.push(current);}
                                        let mut y=origin.y;let mut total_width=0.0_f32;
                                        for row in rows {
                                            let cross=row.iter().map(|i|sizes[*i].y).fold(0.0_f32,f32::max);let mut x=origin.x;let mut actual_height=0.0_f32;
                                            for i in row {
                                                let child=active[i];let size=sizes[i];let top=y+if text(node,"align")=="center"{(cross-size.y).max(0.)/2.}else{0.};
                                                let rect=egui::Rect::from_min_size(egui::pos2(x,top),egui::vec2(size.x,ui.available_height().max(size.y)));
                                                let mut cell=ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(egui::Layout::top_down(egui::Align::Min)));cell.set_clip_rect(ui.clip_rect());
                                                self.node(&mut cell,child,nodes,route,depth+1,events);let actual=cell.min_rect().size();
                                                ui.ctx().data_mut(|d|{d.insert_temp(natural_size_key(&nodes[child]),actual);d.insert_temp(ui.scope_id().with((&key,"wrap-size",child)),actual);});
                                                if (actual-size).length()>0.01{ui.ctx().request_discard("wrapped compound size");}
                                                actual_height=actual_height.max(actual.y);x+=actual.x+gap.x;
                                            }
                                            total_width=total_width.max(x-origin.x-gap.x);y+=actual_height+gap.y;
                                        }
                                        let height=(y-origin.y-gap.y).max(0.);
                                        ui.allocate_space(egui::vec2(if node.ports.get("fill")==Some(&Value::Bool(true)){available}else{total_width},height));
                                    } else if node.ports.get("wrap").and_then(Value::as_bool)==Some(true) {
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
                                    } else if text(node,"justify")=="between" && node.ports.get("native_spacing")!=Some(&Value::Bool(true)) {
                                        let active:Vec<_>=children(node).filter(|child|nodes.get(*child).is_some_and(|n|n.ports.get("hidden").and_then(Value::as_bool)!=Some(true))).collect();
                                        let fill_count=active.iter().filter(|c|nodes[**c].ports.get("fill")==Some(&Value::Bool(true))).count();
                                        let fixed:f32=active.iter().filter(|c|nodes[**c].ports.get("fill")!=Some(&Value::Bool(true))).map(|c|intrinsic_text_width(ui,&nodes[*c],nodes,&self.tree.model,false,0)).sum();
                                        let sizes:Vec<egui::Vec2>=active.iter().map(|child|{let old=ui.ctx().data(|d|d.get_temp::<egui::Vec2>(ui.scope_id().with((&key,"between-size",child)))).unwrap_or_default();egui::vec2(if nodes[*child].ports.get("fill")==Some(&Value::Bool(true)){(ui.available_width()-fixed).max(0.)/fill_count.max(1) as f32}else{intrinsic_text_width(ui,&nodes[*child],nodes,&self.tree.model,false,0)},old.y)}).collect();
                                        let available=ui.available_width();
                                        let gap=((available-sizes.iter().map(|s|s.x).sum::<f32>())/active.len().saturating_sub(1).max(1) as f32).max(0.);
                                        let cross=sizes.iter().map(|s|s.y).fold(0.0_f32,f32::max);
                                        let origin=ui.available_rect_before_wrap().min;
                                        let mut x=origin.x;
                                        let mut height=0.0_f32;
                                        for (child,size) in active.iter().zip(&sizes) {
                                            let rect=egui::Rect::from_min_size(egui::pos2(x,origin.y+(cross-size.y)/2.),*size);
                                            let measure=egui::Rect::from_min_size(rect.min,egui::vec2(size.x,ui.available_height().max(size.y)));
                                            let mut cell=ui.new_child(egui::UiBuilder::new().max_rect(measure).layout(egui::Layout::top_down(egui::Align::Min)));
                                            cell.set_clip_rect(ui.clip_rect());
                                            self.node(&mut cell,child,nodes,route,depth+1,events);
                                            let actual=cell.min_rect().size();
                                            ui.ctx().data_mut(|d|d.insert_temp(natural_size_key(&nodes[*child]),actual));
                                            if (actual-*size).length()>0.01 {ui.ctx().data_mut(|d|d.insert_temp(ui.scope_id().with((&key,"between-size",child)),actual));ui.ctx().request_discard("scene layout measurement");}
                                            height=height.max(actual.y);
                                            x+=size.x+gap;
                                        }
                                        ui.allocate_space(egui::vec2(available,height));
                                    } else if text(node,"justify")=="between" {
                                        ui.horizontal(|ui|{let active:Vec<_>=children(node).collect();if let Some(first)=active.first(){self.node(ui,first,nodes,route,depth+1,events);ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|for child in active.iter().skip(1).rev(){self.node(ui,child,nodes,route,depth+1,events);});}});
                                    } else if children(node).any(|c|nodes.get(c).is_some_and(|n|n.ports.get("hidden")!=Some(&Value::Bool(true)) && ["grow","shrink","basis"].iter().any(|p|num(n,p).unwrap_or(0.)>0.))) {
                                        let active:Vec<_>=children(node).filter(|c|nodes.get(*c).is_some_and(|n|n.ports.get("hidden")!=Some(&Value::Bool(true)))).collect();
                                        let available=(ui.available_width()-ui.spacing().item_spacing.x*active.len().saturating_sub(1) as f32).max(0.);
                                        let natural:Vec<f32>=active.iter().map(|c|intrinsic_text_width(ui,&nodes[*c],nodes,&self.tree.model,false,0)).collect();
                                        let minimum:Vec<f32>=active.iter().map(|c|intrinsic_text_width(ui,&nodes[*c],nodes,&self.tree.model,true,0)).collect();
                                        let grow:Vec<f32>=active.iter().map(|c|bounded_layout(num(&nodes[*c],"grow").unwrap_or(0.),1000.)).collect();
                                        let mut widths:Vec<f32>=active.iter().enumerate().map(|(i,c)|num(&nodes[*c],"basis").map(|n|bounded_layout(n,available)).unwrap_or(if grow[i]>0. {0.}else{natural[i]})).collect();
                                        let extra=(available-widths.iter().sum::<f32>()).max(0.);
                                        let total_grow=grow.iter().sum::<f32>();
                                        if total_grow>0. {for i in 0..widths.len() {widths[i]+=extra*grow[i]/total_grow;}}
                                        for _ in 0..active.len().min(32) {
                                            let excess=(widths.iter().sum::<f32>()-available).max(0.);if excess<0.01 {break;}
                                            let weights:Vec<f32>=active.iter().enumerate().map(|(i,c)|if widths[i]>minimum[i]+0.01 {bounded_layout(num(&nodes[*c],"shrink").unwrap_or(0.),1000.)*natural[i].max(1.)}else{0.}).collect();
                                            let total=weights.iter().sum::<f32>();if total<=0. {break;}
                                            for i in 0..widths.len() {widths[i]=(widths[i]-excess*weights[i]/total).max(minimum[i].min(widths[i]));}
                                        }
                                        // Explicit bases reserve a bounded budget, even without grow.
                                        let fixed: f32=active.iter().enumerate().filter(|(_,c)|!nodes[**c].ports.contains_key("basis")).map(|(i,_)|widths[i]).sum();
                                        let based: f32=active.iter().enumerate().filter(|(_,c)|nodes[**c].ports.contains_key("basis")).map(|(i,_)|widths[i]).sum();
                                        let budget=(available-fixed).max(0.);
                                        if based>budget && based>0. {for (i,c) in active.iter().enumerate() {if nodes[*c].ports.contains_key("basis") {widths[i]*=budget/based;}}}
                                        for (i,c) in active.iter().enumerate() {widths[i]=widths[i].max(num(&nodes[*c],"min_width").unwrap_or(0.)).min(num(&nodes[*c],"max_width").unwrap_or(1e6));}
                                        let origin=ui.available_rect_before_wrap().min;let row_id=ui.scope_id();
                                        let cross=active.iter().map(|c|ui.ctx().data(|d|d.get_temp::<f32>(row_id.with(("flex-height",c)))).unwrap_or(0.)).fold(0.0_f32,f32::max);
                                        let mut x=origin.x+if text(node,"align")=="end" {(available-widths.iter().sum::<f32>()).max(0.)}else{0.};let mut height=0.0_f32;
                                        for (child,width) in active.iter().zip(&widths) {
                                            let cache=row_id.with(("flex-height",child));let previous=ui.ctx().data(|d|d.get_temp::<f32>(cache)).unwrap_or(0.);
                                            let top=origin.y+if text(node,"align")=="center" {(cross-previous).max(0.)/2.}else{0.};
                                            let rect=egui::Rect::from_min_size(egui::pos2(x,top),egui::vec2(*width,ui.available_height().max(previous)));
                                            let mut cell=ui.new_child(egui::UiBuilder::new().max_rect(rect).layout(egui::Layout::top_down(egui::Align::Min)));
                                            cell.set_max_width(*width);
                                            cell.set_clip_rect(ui.clip_rect().intersect(egui::Rect::from_min_max(egui::pos2(x,ui.clip_rect().top()),egui::pos2(x+width,ui.clip_rect().bottom()))));
                                            let stretch=text(node,"align")=="stretch" && num(&nodes[*child],"height").is_none();
                                            if stretch {self.stretch_heights.insert((*child).into(),cross);}
                                            if nodes[*child].family=="spacer"{cell.allocate_space(egui::vec2(*width,0.));self.geometry.insert((*child).into(),egui::Rect::from_min_size(rect.min,egui::vec2(*width,0.)));}else{self.node(&mut cell,child,nodes,route,depth+1,events);}
                                            if stretch {self.stretch_heights.remove(*child);}
                                            let actual=cell.min_rect().height();
                                            if (actual-previous).abs()>0.01 {ui.ctx().data_mut(|d|d.insert_temp(cache,actual));ui.ctx().request_discard("flex cross size");}
                                            if (cell.min_rect().width()-width).abs()>0.01 {ui.ctx().request_discard("natural leaf width");}ui.ctx().data_mut(|d|d.insert_temp(natural_size_key(&nodes[*child]),cell.min_rect().size()));
                                            height=height.max(actual);x+=width+ui.spacing().item_spacing.x;
                                        }
                                        ui.allocate_space(egui::vec2(ui.available_width(),height));
                                    } else if text(node,"align")=="baseline" {
                                        let active:Vec<_>=children(node).filter(|c|nodes.get(*c).is_some_and(|n|n.ports.get("hidden")!=Some(&Value::Bool(true)))).collect();
                                        let bases:Vec<f32>=active.iter().map(|c|text_baseline(ui,&nodes[*c])).collect();
                                        let baseline=bases.iter().copied().fold(0.0_f32,f32::max);let origin=ui.available_rect_before_wrap().min;let mut x=origin.x;let mut height=0.0_f32;
                                        for (child,base) in active.iter().zip(bases) {let width=intrinsic_text_width(ui,&nodes[*child],nodes,&self.tree.model,false,0);let offset=baseline-base;let pos=egui::pos2(x,origin.y+offset);let mut cell=ui.new_child(egui::UiBuilder::new().max_rect(egui::Rect::from_min_size(pos,egui::vec2(width,ui.available_height()))).layout(egui::Layout::top_down(egui::Align::Min)));self.node(&mut cell,child,nodes,route,depth+1,events);height=height.max(offset+cell.min_rect().height());x+=width+ui.spacing().item_spacing.x;}
                                        ui.allocate_space(egui::vec2((x-origin.x-ui.spacing().item_spacing.x).max(0.),height));
                                    } else if text(node,"align")=="end" {
                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|for child in children(node).collect::<Vec<_>>().into_iter().rev(){self.node(ui,child,nodes,route,depth+1,events);});
                                    } else if text(node,"align")=="start" {
                                        if let Some(h)=num(node,"height") {ui.allocate_ui_with_layout(egui::vec2(ui.available_width(),h),egui::Layout::left_to_right(egui::Align::Min),|ui|draw(ui));}else{ui.horizontal_top(|ui|draw(ui));}
                                    } else if let Some(height) = num(node, "height") {
 ui.allocate_ui_with_layout(Vec2::new(ui.available_width(),height),egui::Layout::left_to_right(egui::Align::Center),|ui|draw(ui));
 } else { ui.horizontal(|ui| draw(ui)); }
                                } else if node.ports.get("scroll")==Some(&Value::Bool(true)) {
                                    egui::ScrollArea::vertical().id_salt(&key).min_scrolled_height(0.0).max_height(ui.available_height()).show(ui,|ui| { ui.vertical(|ui| draw(ui)); });


                                } else {
                                    let mode=text(node,"justify");
                                    let centre=matches!(mode,"center"|"end"|"between"|"around"|"evenly");
                                    let count=children(node).filter(|c|nodes.get(*c).is_some_and(|n|n.ports.get("hidden")!=Some(&Value::Bool(true)))).count();
                                    let old_gap=ui.spacing().item_spacing.y;let mut extra=0.;
                                    let cache = ui.scope_id().with((&key,"column-content-height"));
                                    if centre {
                                        let content = ui.ctx().data_mut(|d| d.get_temp::<f32>(cache)).unwrap_or(0.);
                                        if content>0. {let height=num(node,"height").or(assigned_height).unwrap_or(ui.available_height());let free=(height-content).max(0.);let leading=match mode{"end"=>free,"between"=>{extra=if count>1{free/(count-1) as f32}else{0.};0.},"around"=>{extra=free/count.max(1) as f32;extra/2.},"evenly"=>{extra=free/(count+1) as f32;extra},_=>free/2.};if leading>0.{ui.add_space(leading);}ui.spacing_mut().item_spacing.y=old_gap+extra;}
                                    }
                                    let content = if text(node,"align")=="center" { ui.vertical_centered(|ui| draw(ui)) } else { ui.vertical(|ui| draw(ui)) }.response.rect.height()-extra*count.saturating_sub(1) as f32;
                                    ui.spacing_mut().item_spacing.y=old_gap;
                                    if centre {
                                        let changed=ui.ctx().data_mut(|d| {
                                            let old=d.get_temp::<f32>(cache);
                                            d.insert_temp(cache,content);
                                            old.is_none_or(|old|(old-content).abs()>0.01)
                                        });
                                        if changed {ui.ctx().request_discard("scene layout measurement");}
                                    }
                                }
                            })
                            .response;
                        if let Some(width)=bound_width {response.rect.max.x=response.rect.min.x+width;}
                        if let Some(height)=self.stretch_heights.get(id).copied() {
                            response.rect.max.y=response.rect.max.y.max(response.rect.min.y+height);
                        }
                        if !node.ports.contains_key("background_radials") {let mut background_rect=response.rect;if let Some(a)=node.ports.get("background_inset").and_then(Value::as_array).filter(|a|a.len()==4){let v:Vec<f32>=a.iter().map(|v|bounded_layout(v.as_f64().unwrap_or(0.) as f32,1e6)).collect();background_rect.min+=egui::vec2(v[3],v[0]);background_rect.max-=egui::vec2(v[1],v[2]);background_rect.max=background_rect.max.max(background_rect.min);}ui.painter().set(radial_shape,egui::Shape::rect_filled(background_rect,num(node,"radius").unwrap_or(0.),background));}
                        if let Some(radials)=node.ports.get("background_radials") { ui.painter().set(radial_shape,self.backdrops.shape(ui.ctx(),&key,response.rect,background,radials)); }
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
        if let Some(height) = self.stretch_heights.get(id).copied() {
            response.rect.max.y = response.rect.max.y.max(response.rect.min.y + height);
        }
        if matches!(node.family.as_str(), "column" | "row") {
            if let Some(width) = num(node, "width").or_else(|| {
                (node.ports.get("fill") == Some(&Value::Bool(true))
                    || num(node, "grow").unwrap_or(0.) > 0.
                    || num(node, "shrink").unwrap_or(0.) > 0.)
                    .then(|| allocated.width().max(num(node, "min_width").unwrap_or(0.)))
            }) {
                response.rect.max.x = response.rect.min.x + width;
            }
        }
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
        // Template instances do not have a unique scene id. Internal control
        // children likewise have no scene node and are deliberately omitted.
        if self.collect_widget_rects && route.is_none() {
            let rect = if node.family == "control" {
                node.ports
                    .get("definition")
                    .and_then(|v| v["id"].as_str())
                    .and_then(|id| self.widgets.geometry.get(id))
                    .copied()
            } else {
                self.geometry.get(id).copied()
            };
            if let Some(rect) = rect.filter(|r| r.is_finite()) {
                let definition = node.ports.get("definition");
                let enabled = ui.is_enabled()
                    && node.ports.get("enabled") != Some(&Value::Bool(false))
                    && definition.is_none_or(|v| v["properties"]["enabled"] != false);
                let row = crate::widget_rects::WidgetRect {
                    id: id.into(),
                    kind: definition
                        .and_then(|v| v["kind"].as_str())
                        .unwrap_or(&node.family)
                        .into(),
                    label: node
                        .ports
                        .get("label")
                        .or_else(|| node.ports.get("text"))
                        .or_else(|| definition.and_then(|v| v.get("label")))
                        .and_then(Value::as_str)
                        .map(str::to_owned),
                    rect,
                    clip: ui.clip_rect(),
                    enabled,
                    focused: false,
                };
                self.widget_rects.push(row);
            }
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
            if edge == "fill" && placement["panel"] == true {
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

fn validate_controls(
    tree: &ResolvedScene,
    families: Option<&BTreeSet<String>>,
) -> Result<(), String> {
    if let Some(families) = families {
        for node in tree.nodes.values() {
            let weight = num(node, "weight").unwrap_or(
                if node.ports.get("bold") == Some(&Value::Bool(true)) {
                    700.
                } else {
                    400.
                },
            );
            for port in ["font", "mono_font"] {
                if let Some(name) = node.ports.get(port).and_then(Value::as_str) {
                    let family = toolkit::fonts::optional::family(name, weight);
                    if let egui::FontFamily::Name(name) = family
                        && !families.contains(name.as_ref())
                    {
                        return Err(format!("required font family not bound: {name}"));
                    }
                }
            }
        }
    }
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
                for key in ["w", "h"] {
                    if num(node, key).is_some_and(|n| !(1.0..=16384.0).contains(&n)) {
                        return Err(format!("image {id}: {key} must be 1..16384"));
                    }
                }
                if text(node, "src")
                    .strip_prefix("icon:")
                    .and_then(crate::icons::Icon::from_name)
                    .is_none()
                    && !text(node, "src").starts_with("raster:")
                    && text(node, "src") != "brand:cyclops"
                {
                    return Err(format!(
                        "image {id}: expected known icon: or retained raster: source"
                    ));
                }
            }
            "text" | "button" | "field" | "toggle" | "row" | "column" | "list" | "spacer"
            | "grid" | "drawing" => {}
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

fn text_baseline(ui: &egui::Ui, node: &Node) -> f32 {
    let plain = json!([{"text":text(node,"text")}]);
    let bold = node.ports.get("bold") == Some(&Value::Bool(true));
    let job = crate::spans::job(
        ui,
        node.ports.get("spans").unwrap_or(&plain),
        num(node, "size").unwrap_or(13.),
        node.ports.get("mono") == Some(&Value::Bool(true)),
        bold,
        Color32::WHITE,
        (
            node.ports.get("font").and_then(Value::as_str),
            num(node, "weight").unwrap_or(if bold { 700. } else { 400. }),
            node.ports.get("mono_font").and_then(Value::as_str),
        ),
    );
    let g = ui.fonts_mut(|f| f.layout_job(job));
    g.rows
        .first()
        .and_then(|r| r.glyphs.first().map(|glyph| r.pos.y + glyph.pos.y))
        .unwrap_or(0.)
}

fn bounded_layout(value: f32, maximum: f32) -> f32 {
    if value.is_finite() {
        value.clamp(0., maximum.max(0.))
    } else {
        maximum.max(0.)
    }
}

fn natural_size_key(node: &Node) -> egui::Id {
    egui::Id::new((
        "scene-natural-leaf",
        &node.family,
        format!("{:?}", node.ports),
    ))
}

// Shape the original spans once. Word bounds are read from the natural glyph
// positions (bounded scan), rather than shaping every word on every frame.
fn intrinsic_text_width(
    ui: &egui::Ui,
    node: &Node,
    nodes: &BTreeMap<String, Node>,
    model: &Value,
    words: bool,
    depth: usize,
) -> f32 {
    if depth > 32 || node.ports.get("hidden") == Some(&Value::Bool(true)) {
        return 0.;
    }
    if node.family == "text" {
        let cache = natural_size_key(node).with(("text-widths", ui.pixels_per_point().to_bits()));
        let sizes = ui
            .ctx()
            .data(|d| d.get_temp::<(f32, f32)>(cache))
            .unwrap_or_else(|| {
                let plain = json!([{"text":text(node,"text")}]);
                let bold = node.ports.get("bold") == Some(&Value::Bool(true));
                let mut job = crate::spans::job(
                    ui,
                    node.ports.get("spans").unwrap_or(&plain),
                    num(node, "size").unwrap_or(13.),
                    node.ports.get("mono") == Some(&Value::Bool(true)),
                    bold,
                    Color32::WHITE,
                    (
                        node.ports.get("font").and_then(Value::as_str),
                        num(node, "weight").unwrap_or(if bold { 700. } else { 400. }),
                        node.ports.get("mono_font").and_then(Value::as_str),
                    ),
                );
                for s in &mut job.sections {
                    s.format.extra_letter_spacing = num(node, "letter_spacing").unwrap_or(0.);
                }
                let galley = ui.fonts_mut(|f| f.layout_job(job));
                let natural = galley.size().x;
                let mut minimum = 0.0_f32;
                let mut start = None;
                let mut seen = 0;
                for row in &galley.rows {
                    for glyph in &row.glyphs {
                        seen += 1;
                        if seen > 4096 {
                            minimum = natural;
                            break;
                        }
                        if glyph.chr.is_whitespace() {
                            start = None;
                        } else {
                            let begin = *start.get_or_insert(glyph.pos.x);
                            minimum = minimum.max(glyph.max_x() - begin);
                        }
                    }
                    start = None;
                }
                let result = (natural, minimum);
                ui.ctx().data_mut(|d| d.insert_temp(cache, result));
                result
            });
        return (if words { sizes.1 } else { sizes.0 })
            .min(num(node, "max_width").unwrap_or(f32::INFINITY));
    }
    let cached = ui
        .ctx()
        .data(|d| d.get_temp::<egui::Vec2>(natural_size_key(node)))
        .map(|v| v.x);
    let explicit = num(node, "width")
        .or(num(node, "w"))
        .or(num(node, "min_width"));
    if let Some(width) = explicit {
        return bounded_layout(width, 1e6);
    }
    if node.family == "control" {
        return Widget::decode(&node.ports["definition"])
            .ok()
            .and_then(|w| w.intrinsic_width(ui, model).ok())
            .or(cached)
            .unwrap_or(80.);
    }
    if !matches!(node.family.as_str(), "column" | "row" | "grid") {
        return cached.unwrap_or_else(|| match node.family.as_str() {
            "image" => {
                if text(node, "src") == "brand:cyclops" {
                    200.
                } else {
                    24.
                }
            }
            "drawing" => 24.,
            "spacer" => num(node, "size").unwrap_or(8.),
            "field" => ui.spacing().text_edit_width,
            _ => {
                ui.fonts_mut(|f| {
                    f.layout_no_wrap(
                        text(node, "label").to_owned(),
                        egui::FontId::proportional(14.),
                        Color32::WHITE,
                    )
                })
                .size()
                .x + 2. * ui.spacing().button_padding.x
            }
        });
    }
    let widths: Vec<f32> = children(node)
        .filter_map(|c| nodes.get(c))
        .map(|n| intrinsic_text_width(ui, n, nodes, model, words, depth + 1))
        .collect();
    let content = if node.family == "row" {
        widths.iter().sum::<f32>()
            + num(node, "gap").unwrap_or(0.) * widths.len().saturating_sub(1) as f32
    } else {
        widths.into_iter().fold(0.0_f32, f32::max)
    };
    content + node_padding(node, "padding_left") + node_padding(node, "padding_right")
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
            num(node, "radius").unwrap_or(2.0),
            egui::Stroke::new(num(node, "outline_width").unwrap_or(1.0), outline),
            egui::StrokeKind::Inside,
        );
    }
}

#[cfg(test)]
#[path = "view_motion_identity_tests.rs"]
mod motion_identity_tests;

#[cfg(test)]
#[path = "view_binding_tests.rs"]
mod binding_tests;
