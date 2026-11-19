//! Scene painter adapter. Retained frames contain paint only, never widgets,
//! routes or model state, so departing nodes cannot consume input.
use egui::{Context, LayerId, Rect, Ui, epaint::ClippedShape, layers::ShapeIdx};
use motion::scene::{Entrance, Transition};
use motion::{Preset, Property};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct SceneMotion {
    nodes: BTreeMap<String, NodeMotion>,
    now: f64,
    reduced: bool,
}
struct NodeMotion {
    seen: bool,
    entrance: Option<Preset>,
    exit: Option<Entrance>,
    leaving: Option<Preset>,
    shapes: Vec<ClippedShape>,
    rect: Rect,
    paint_rect: Rect,
    child_enter: Option<Entrance>,
    child_exit: Option<Entrance>,
    exit_index: u32,
    child_index: u32,
    textures: Vec<egui::TextureHandle>,
    opacity: f64,
}
#[derive(Clone)]
struct Tween {
    from: Vec<f64>,
    to: Vec<f64>,
    start: f64,
    spec: Transition,
}
impl Tween {
    fn sample(&self, now: f64) -> Vec<f64> {
        self.from
            .iter()
            .zip(&self.to)
            .map(|(a, b)| self.spec.interpolate(self.start, now, *a, *b))
            .collect()
    }
}
pub struct Capture {
    key: String,
    layer: LayerId,
    start: ShapeIdx,
    transformed: bool,
    outer: egui::emath::TSTransform,
    offset: [f64; 2],
}
impl SceneMotion {
    /// A laid-out node outside its paint viewport is still present. Do not
    /// turn scrolling into an exit, or retain a stale visible image for it.
    pub fn keep_unpainted(&mut self, key: &str) {
        if let Some(n) = self.nodes.get_mut(key) {
            n.seen = true;
            n.shapes.clear();
            n.textures.clear();
        }
    }
    pub fn keep_unseen(&mut self, present: impl Fn(&str) -> bool) {
        for (key, n) in &mut self.nodes {
            if !n.seen && present(key) {
                n.seen = true;
                n.shapes.clear();
                n.textures.clear();
            }
        }
    }
    pub fn begin(&mut self, ui: &Ui) {
        self.now = ui.input(|i| i.time);
        self.reduced = ui.style().animation_time == 0.0;
        for n in self.nodes.values_mut() {
            n.seen = false;
            n.child_index = 0;
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub fn capture(
        &mut self,
        ui: &Ui,
        key: &str,
        parent: Option<&str>,
        enter: Option<&Value>,
        exit: Option<&Value>,
        enabled: bool,
    ) -> Option<Capture> {
        if !enabled || self.reduced || ui.style().animation_time == 0.0 {
            self.nodes.remove(key);
            return None;
        }
        let inherited = parent.and_then(|p| self.nodes.get_mut(p)).map(|n| {
            let index = n.child_index;
            n.child_index += 1;
            (
                n.child_enter.clone(),
                n.child_exit.clone().or_else(|| n.exit.clone()),
                index,
                if n.child_exit.is_some() {
                    index
                } else {
                    n.exit_index
                },
            )
        });
        let own = enter.and_then(|v| serde_json::from_value::<Entrance>(v.clone()).ok());
        let (entrance, index) = own
            .clone()
            .map(|e| (e, 0))
            .or_else(|| {
                inherited
                    .as_ref()
                    .and_then(|(e, _, i, _)| e.clone().map(|e| (e, *i)))
            })
            .map_or((None, 0), |(e, i)| (Some(e), i));
        let child_enter = own.filter(|e| e.options().stagger_ms > 0.0);
        let own_exit = exit.and_then(|v| serde_json::from_value::<Entrance>(v.clone()).ok());
        let child_exit = own_exit.clone().filter(|e| e.options().stagger_ms > 0.0);
        let exit_index = if own_exit.is_some() {
            0
        } else {
            inherited.as_ref().map_or(0, |(_, _, _, i)| *i)
        };
        let inherited_exit = inherited.and_then(|(_, e, _, _)| e);
        if entrance.is_none() && exit.is_none() && inherited_exit.is_none() {
            return None;
        }
        let now = self.now;
        let parent_opacity = parent
            .and_then(|p| self.nodes.get(p))
            .map_or(1.0, |n| n.opacity);
        let n = self
            .nodes
            .entry(key.to_owned())
            .or_insert_with(|| NodeMotion {
                seen: true,
                entrance: if child_enter.is_some() {
                    None
                } else {
                    entrance.map(|e| e.playback(now, index, false))
                },
                exit: None,
                leaving: None,
                shapes: Vec::new(),
                rect: Rect::NOTHING,
                paint_rect: Rect::NOTHING,
                child_enter: child_enter.clone(),
                child_exit: None,
                exit_index,
                child_index: 0,
                textures: Vec::new(),
                opacity: 1.0,
            });
        n.seen = true;
        n.child_enter = child_enter;
        n.leaving = None;
        n.exit = if child_exit.is_some() {
            None
        } else {
            own_exit.or(inherited_exit)
        };
        n.child_exit = child_exit;
        n.exit_index = exit_index;
        n.opacity = motion::scene::sample(n.entrance.as_ref(), now, false).get(Property::Opacity)
            * parent_opacity;
        let layer = ui.layer_id();
        let start = ui.ctx().graphics_mut(|g| g.entry(layer).next_idx());
        Some(Capture {
            key: key.into(),
            layer,
            start,
            transformed: false,
            outer: egui::emath::TSTransform::IDENTITY,
            offset: [0., 0.],
        })
    }
    /// Install the same transform for painting and input before widgets run.
    pub fn layer(
        &self,
        ui: &Ui,
        capture: &mut Option<Capture>,
        hint: Rect,
        offset: [f64; 2],
    ) -> Option<LayerId> {
        let c = capture.as_mut()?;
        let n = self.nodes.get(&c.key)?;
        let rect = if n.rect.is_finite() { n.rect } else { hint };
        let values = motion::scene::sample(n.entrance.as_ref(), self.now, self.reduced);
        let scale = values.get(Property::Scale) as f32;
        let translate = egui::vec2(
            values.get(Property::Tx) as f32 + offset[0] as f32,
            values.get(Property::Ty) as f32 + offset[1] as f32,
        );
        let local = egui::emath::TSTransform {
            scaling: scale.max(0.0001),
            translation: rect.center().to_vec2() * (1.0 - scale) + translate,
        };
        let outer = ui
            .ctx()
            .layer_transform_to_global(ui.layer_id())
            .unwrap_or_default();
        let layer = LayerId::new(
            ui.layer_id().order,
            ui.scope_id().with(("scene-motion", &c.key)),
        );
        ui.ctx().set_sublayer(ui.layer_id(), layer);
        ui.ctx().set_transform_layer(layer, outer * local);
        c.layer = layer;
        c.start = ui.ctx().graphics_mut(|g| g.entry(layer).next_idx());
        c.transformed = true;
        c.outer = outer;
        c.offset = offset;
        Some(layer)
    }
    pub fn painted(&mut self, ui: &Ui, capture: Option<Capture>, rect: Rect) {
        let Some(c) = capture else {
            return;
        };
        let n = self.nodes.get_mut(&c.key).unwrap();
        let first_bounds = !n.rect.is_finite();
        n.rect = rect;
        let now = self.now;
        let mut values = motion::scene::sample(n.entrance.as_ref(), now, self.reduced);
        values.set(Property::Opacity, n.opacity);
        if c.transformed {
            let scale = values.get(Property::Scale) as f32;
            let translation = rect.center().to_vec2() * (1.0 - scale)
                + egui::vec2(
                    values.get(Property::Tx) as f32 + c.offset[0] as f32,
                    values.get(Property::Ty) as f32 + c.offset[1] as f32,
                );
            ui.ctx().set_transform_layer(
                c.layer,
                c.outer
                    * egui::emath::TSTransform {
                        scaling: scale.max(0.0001),
                        translation,
                    },
            );
            if first_bounds && scale != 1.0 {
                ui.ctx().request_discard("motion layout bounds");
            }
        }
        let active = n
            .entrance
            .as_ref()
            .is_some_and(|p| motion::scene::active(p, now));
        let global = if c.transformed {
            ui.ctx()
                .layer_transform_to_global(c.layer)
                .unwrap_or_default()
        } else {
            egui::emath::TSTransform::IDENTITY
        };
        n.paint_rect = global.mul_rect(rect);
        ui.ctx().graphics_mut(|g| {
            let list = g.entry(c.layer);
            let end = list.next_idx();
            for i in c.start.0..end.0 {
                list.mutate_shape(ShapeIdx(i), |s| {
                    let mut values = values;
                    if c.transformed {
                        values.set(Property::Tx, 0.0);
                        values.set(Property::Ty, 0.0);
                        values.set(Property::Scale, 1.0);
                    }
                    transform(s, rect, &values);
                });
            }
            if n.exit.is_some() {
                n.shapes = list
                    .all_entries()
                    .skip(c.start.0)
                    .take(end.0 - c.start.0)
                    .cloned()
                    .collect();
                if c.transformed {
                    for s in &mut n.shapes {
                        s.transform(global);
                    }
                }
            }
        });
        if n.exit.is_some() {
            let mut ids = std::collections::HashSet::new();
            fn collect(shape: &egui::Shape, ids: &mut std::collections::HashSet<egui::TextureId>) {
                if let egui::Shape::Vec(shapes) = shape {
                    for s in shapes {
                        collect(s, ids);
                    }
                } else {
                    ids.insert(shape.texture_id());
                }
            }
            for s in &n.shapes {
                collect(&s.shape, &mut ids);
            }
            let manager = ui.ctx().tex_manager();
            let mut textures = Vec::new();
            for id in ids {
                if matches!(id, egui::TextureId::Managed(_)) && manager.read().meta(id).is_some() {
                    manager.write().retain(id);
                    textures.push(egui::TextureHandle::new(manager.clone(), id));
                }
            }
            n.textures = textures;
        }
        if active {
            if let Some(p) = n.entrance.as_ref().filter(|p| now < p.timing.start()) {
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_secs_f64(
                        p.timing.start() - now,
                    ));
            } else {
                ui.ctx().request_repaint();
            }
        }
    }
    pub fn end(&mut self, ui: &Ui) {
        if self.reduced {
            self.nodes.clear();
            return;
        }
        let now = self.now;
        self.nodes.retain(|_, n| {
            if n.seen {
                return true;
            }
            let Some(exit) = &n.exit else {
                return false;
            };
            if n.shapes.is_empty() {
                return false;
            }
            let p = n
                .leaving
                .get_or_insert_with(|| exit.playback(now, n.exit_index, true));
            if !motion::scene::active(p, now) {
                return false;
            }
            let values = p.sample(now);
            for shape in &n.shapes {
                let mut shape = shape.clone();
                transform(&mut shape, n.paint_rect, &values);
                ui.painter()
                    .with_clip_rect(shape.clip_rect)
                    .add(shape.shape);
            }
            ui.ctx().request_repaint();
            true
        });
    }
}

/// Animate an authored visual port; identity is separate from node lifecycle.
pub fn value(
    ctx: &Context,
    key: &str,
    prop: &str,
    target: &Value,
    declaration: Option<&Value>,
    enabled: bool,
) -> Value {
    let Some(spec) = declaration
        .and_then(|v| motion::scene::transitions(v).ok())
        .and_then(|ts| ts.into_iter().find(|t| t.prop == prop))
    else {
        return target.clone();
    };
    let colour = matches!(prop, "color" | "background" | "hover" | "highlight");
    let decode = |v: &Value| -> Option<Vec<f64>> {
        if colour {
            let s = v.as_str()?.strip_prefix('#')?;
            if !s.is_ascii() || ![6, 8].contains(&s.len()) {
                return None;
            }
            let mut cs = (0..s.len())
                .step_by(2)
                .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok().map(f64::from))
                .collect::<Option<Vec<_>>>()?;
            if cs.len() == 3 {
                cs.push(255.0);
            }
            Some(cs)
        } else if let Some(a) = v.as_array() {
            a.iter().map(Value::as_f64).collect()
        } else {
            Some(vec![v.as_f64()?])
        }
    };
    let Some(to) = decode(target) else {
        return target.clone();
    };
    let now = ctx.input(|i| i.time);
    let id = egui::Id::unique(("scene-transition", key, prop));
    let reduced = ctx.global_style().animation_time == 0.0 || !enabled;
    let (sample, active) = ctx.data_mut(|d| {
        let mut t = d.get_temp::<Tween>(id).unwrap_or(Tween {
            from: to.clone(),
            to: to.clone(),
            start: now,
            spec: spec.clone(),
        });
        if t.to != to {
            t = Tween {
                from: t.sample(now),
                to: to.clone(),
                start: now,
                spec,
            };
        }
        if reduced {
            t.from = to.clone();
            t.to = to;
        }
        let sample = t.sample(now);
        let active = !reduced && t.from != t.to && now < t.start + t.spec.ms / 1000.0;
        d.insert_temp(id, t);
        (sample, active)
    });
    if active {
        ctx.request_repaint();
    }
    if colour {
        Value::String(format!(
            "#{:02x}{:02x}{:02x}{:02x}",
            sample[0].clamp(0., 255.).round() as u8,
            sample[1].clamp(0., 255.).round() as u8,
            sample[2].clamp(0., 255.).round() as u8,
            sample[3].clamp(0., 255.).round() as u8
        ))
    } else if target.is_array() {
        serde_json::json!(sample)
    } else {
        serde_json::json!(sample[0])
    }
}

pub fn visual(ui: &Ui, start: ShapeIdx, rect: Rect, opacity: f64, offset: [f64; 2]) {
    if opacity == 1.0 && offset == [0.0, 0.0] {
        return;
    }
    let mut values = motion::Values::default();
    values.set(Property::Opacity, opacity);
    values.set(Property::Tx, offset[0]);
    values.set(Property::Ty, offset[1]);
    ui.ctx().graphics_mut(|g| {
        let list = g.entry(ui.layer_id());
        let end = list.next_idx();
        for i in start.0..end.0 {
            list.mutate_shape(ShapeIdx(i), |s| transform(s, rect, &values));
        }
    });
}
/// Interpolate the actual colours a widget painted (hover, pressed, selected,
/// toggle and foreground), rather than a second set of interaction styles.
pub fn highlight(
    ui: &Ui,
    layer: LayerId,
    start: ShapeIdx,
    key: &str,
    declaration: Option<&Value>,
    enabled: bool,
) {
    let Some(declaration) = declaration.filter(|v| {
        motion::scene::transitions(v).is_ok_and(|ts| ts.iter().any(|t| t.prop == "highlight"))
    }) else {
        return;
    };
    let shapes = ui.ctx().graphics_mut(|g| {
        g.entry(layer)
            .all_entries()
            .skip(start.0)
            .cloned()
            .collect::<Vec<_>>()
    });
    for (index, shape) in shapes.into_iter().enumerate() {
        let clip = shape.clip_rect;
        let mut painted = Vec::new();
        for primitive in ui.ctx().tessellate(vec![shape], ui.pixels_per_point()) {
            match primitive.primitive {
                egui::epaint::Primitive::Mesh(mut mesh) => {
                    let mut colours = Vec::<(egui::Color32, egui::Color32)>::new();
                    for vertex in &mut mesh.vertices {
                        if let Some((_, sampled)) =
                            colours.iter().find(|(target, _)| *target == vertex.color)
                        {
                            vertex.color = *sampled;
                            continue;
                        }
                        let target = vertex.color;
                        let [r, g, b, a] = target.to_srgba_unmultiplied();
                        let k = format!("{key}/{index}/{}", colours.len());
                        let sampled = value(
                            ui.ctx(),
                            &k,
                            "highlight",
                            &serde_json::json!(format!("#{r:02x}{g:02x}{b:02x}{a:02x}")),
                            Some(declaration),
                            enabled,
                        );
                        let s = sampled.as_str().unwrap().strip_prefix('#').unwrap();
                        let byte = |i| u8::from_str_radix(&s[i..i + 2], 16).unwrap();
                        vertex.color = egui::Color32::from_rgba_unmultiplied(
                            byte(0),
                            byte(2),
                            byte(4),
                            byte(6),
                        );
                        colours.push((target, vertex.color));
                    }
                    painted.push(egui::Shape::mesh(mesh));
                }
                egui::epaint::Primitive::Callback(callback) => {
                    painted.push(egui::Shape::Callback(callback))
                }
            }
        }
        ui.ctx().graphics_mut(|g| {
            g.entry(layer)
                .set(ShapeIdx(start.0 + index), clip, egui::Shape::Vec(painted))
        });
    }
}
fn transform(shape: &mut ClippedShape, rect: Rect, values: &motion::Values) {
    let scale = values.get(Property::Scale) as f32;
    let offset = egui::vec2(
        values.get(Property::Tx) as f32,
        values.get(Property::Ty) as f32,
    );
    shape.shape.transform(egui::emath::TSTransform {
        scaling: scale,
        translation: rect.center().to_vec2() * (1.0 - scale) + offset,
    });
    let opacity = values.get(Property::Opacity).clamp(0.0, 1.0) as f32;
    if opacity != 1.0 {
        egui::epaint::shape_transform::adjust_colors(&mut shape.shape, move |c| {
            if *c != egui::Color32::PLACEHOLDER {
                *c = c.gamma_multiply(opacity);
            }
        });
    }
}

/// Shared deterministic state transition for built-in widgets. Retargets from
/// the value sampled at this frame; settled widgets request no repaint.
pub fn scalar(ctx: &Context, id: egui::Id, target: f64, ms: f64, enabled: bool) -> f32 {
    #[derive(Clone)]
    struct State {
        from: f64,
        to: f64,
        start: f64,
    }
    let now = ctx.input(|i| i.time);
    let reduced = ctx.global_style().animation_time == 0.0 || !enabled;
    let curve = Transition {
        prop: "opacity".into(),
        ms,
        ease: motion::Curve::EaseOut,
    };
    let (value, active) = ctx.data_mut(|d| {
        let mut s = d.get_temp::<State>(id).unwrap_or(State {
            from: target,
            to: target,
            start: now,
        });
        let current = curve.interpolate(s.start, now, s.from, s.to);
        if s.to != target {
            s = State {
                from: current,
                to: target,
                start: now,
            };
        }
        let t = curve.progress(s.start, now);
        let value = if reduced {
            target
        } else {
            curve.interpolate(s.start, now, s.from, s.to)
        };
        let active = !reduced && s.from != s.to && t < 1.0;
        if reduced {
            s.from = target;
            s.to = target;
        }
        d.insert_temp(id, s);
        (value as f32, active)
    });
    if active {
        ctx.request_repaint();
    }
    value
}

/// Observe native floating surfaces at the end of a pass. Their default is a
/// fade, which leaves layout and input geometry unchanged. Closed surfaces
/// replay paint only, after the model has already closed them.
pub fn install(ctx: &Context) {
    let key = egui::Id::unique("scene-motion-installed");
    if ctx.data(|d| d.get_temp::<bool>(key)) == Some(true) {
        return;
    }
    ctx.data_mut(|d| d.insert_temp(key, true));
    ctx.on_begin_pass(
        "scene surface motion policy",
        std::sync::Arc::new(|ui| {
            let key = egui::Id::unique(("motion-disabled-surfaces", ui.ctx().viewport_id()));
            ui.ctx()
                .data_mut(|d| d.insert_temp(key, std::collections::HashSet::<LayerId>::new()));
        }),
    );
    ctx.on_end_pass(
        "scene surface motion",
        std::sync::Arc::new(|ui| {
            let ctx = ui.ctx().clone();
            let key = egui::Id::unique(("scene-surfaces", ctx.viewport_id()));
            let shared = ctx.data_mut(|d| {
                let state = d
                    .get_temp::<std::sync::Arc<std::sync::Mutex<SceneMotion>>>(key)
                    .unwrap_or_default();
                d.insert_temp(key, state.clone());
                state
            });
            let mut state = shared.lock().unwrap();
            state.begin(ui);
            let layers = ctx.memory(|m| m.layer_ids().collect::<Vec<_>>());
            for layer in layers {
                if layer == ui.layer_id()
                    || !matches!(layer.order, egui::Order::Foreground | egui::Order::Middle)
                {
                    continue;
                }
                // Scene transform sublayers have no Area; only floating surfaces
                // own area state. Tooltips are excluded by their order.
                if ctx.memory(|m| m.area_rect(layer.id)).is_none() {
                    continue;
                }
                let rect = ctx.memory(|m| m.area_rect(layer.id)).unwrap();
                let has_paint = ctx.graphics_mut(|g| g.get(layer).is_some_and(|p| !p.is_empty()));
                if !has_paint {
                    continue;
                }
                let disabled = ctx
                    .data(|d| {
                        d.get_temp::<std::collections::HashSet<LayerId>>(egui::Id::unique((
                            "motion-disabled-surfaces",
                            ctx.viewport_id(),
                        )))
                    })
                    .is_some_and(|ids| ids.contains(&layer));
                let mut capture = state.capture(
                    ui,
                    &format!("{layer:?}"),
                    None,
                    Some(&serde_json::json!("fade")),
                    Some(&serde_json::json!("fade")),
                    !disabled,
                );
                if let Some(c) = capture.as_mut() {
                    c.layer=layer;
                    c.start = ShapeIdx(0);
                }
                // Creating a Ui here would register widgets while the plugin
                // callback lock is held, re-entering on_widget_under_pointer.
                state.painted(ui, capture, rect);
            }
            state.end(ui);
        }),
    );
}

pub struct SurfacePolicy {
    ctx: Context,
    before: Vec<(LayerId, usize)>,
}
fn surface_counts(ctx: &Context) -> Vec<(LayerId, usize)> {
    let layers = ctx.memory(|m| m.layer_ids().collect::<Vec<_>>());
    ctx.graphics_mut(|g| {
        layers
            .into_iter()
            .map(|l| (l, g.get(l).map_or(0, |p| p.next_idx().0)))
            .collect()
    })
}
pub fn policy(ui: &Ui) -> Option<SurfacePolicy> {
    (ui.style().animation_time == 0.0).then(|| SurfacePolicy {
        ctx: ui.ctx().clone(),
        before: surface_counts(ui.ctx()),
    })
}
impl Drop for SurfacePolicy {
    fn drop(&mut self) {
        let disabled = surface_counts(&self.ctx)
            .into_iter()
            .filter(|(l, n)| {
                self.before
                    .iter()
                    .find(|(before, _)| before == l)
                    .map_or(0, |(_, n)| *n)
                    < *n
            })
            .map(|(l, _)| l)
            .collect::<Vec<_>>();
        let key = egui::Id::unique(("motion-disabled-surfaces", self.ctx.viewport_id()));
        self.ctx.data_mut(|d| {
            let mut ids = d
                .get_temp::<std::collections::HashSet<LayerId>>(key)
                .unwrap_or_default();
            ids.extend(disabled);
            d.insert_temp(key, ids);
        });
    }
}

/// A built-in compound widget's changing content (for example a tab page).
/// The closure runs only for current content, never for a departing snapshot.
pub fn content<R>(ui: &mut Ui, id: egui::Id, key: &str, draw: impl FnOnce(&mut Ui) -> R) -> R {
    let state = state(ui.ctx(), id);
    let mut state = state.lock().unwrap();
    state.begin(ui);
    let capture = state.capture(
        ui,
        key,
        None,
        Some(&serde_json::json!("fade")),
        Some(&serde_json::json!("fade")),
        ui.style().animation_time != 0.0,
    );
    let result = draw(ui);
    state.painted(ui, capture, ui.min_rect());
    state.end(ui);
    result
}
pub fn state(ctx: &Context, id: egui::Id) -> std::sync::Arc<std::sync::Mutex<SceneMotion>> {
    ctx.data_mut(|d| {
        let state = d
            .get_temp::<std::sync::Arc<std::sync::Mutex<SceneMotion>>>(id)
            .unwrap_or_default();
        d.insert_temp(id, state.clone());
        state
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    fn text_present(shapes: &[ClippedShape], label: &str) -> bool {
        fn has(shape: &egui::Shape, label: &str) -> bool {
            match shape {
                egui::Shape::Text(t) => t.galley.job.text.contains(label),
                egui::Shape::Vec(shapes) => shapes.iter().any(|s| has(s, label)),
                _ => false,
            }
        }
        shapes.iter().any(|s| has(&s.shape, label))
    }
    #[test]
    fn painted_highlight_colours_follow_the_declared_curve() {
        let ctx = Context::default();
        let declaration = json!({"prop":"highlight","ms":100,"ease":{"kind":"linear"}});
        let mut sampled = egui::Color32::TRANSPARENT;
        for (t, colour) in [
            (0., egui::Color32::RED),
            (1., egui::Color32::BLUE),
            (1.05, egui::Color32::BLUE),
        ] {
            ctx.run_ui(raw(t), |ui| {
                let start = ui.ctx().graphics_mut(|g| g.entry(ui.layer_id()).next_idx());
                ui.painter().rect_filled(
                    Rect::from_min_size(egui::pos2(20., 20.), egui::vec2(30., 20.)),
                    0.,
                    colour,
                );
                highlight(
                    ui,
                    ui.layer_id(),
                    start,
                    "highlight",
                    Some(&declaration),
                    true,
                );
                sampled = ui.ctx().graphics_mut(|g| {
                    let shape = &g
                        .entry(ui.layer_id())
                        .all_entries()
                        .nth(start.0)
                        .unwrap()
                        .shape;
                    let egui::Shape::Vec(shapes) = shape else {
                        panic!("highlight mesh")
                    };
                    shapes
                        .iter()
                        .find_map(|s| {
                            if let egui::Shape::Mesh(m) = s {
                                m.vertices
                                    .iter()
                                    .find(|v| v.color.a() == 255)
                                    .map(|v| v.color)
                            } else {
                                None
                            }
                        })
                        .unwrap()
                });
            })
            .drop_without_applying_deltas();
        }
        assert!((i32::from(sampled.r()) - 128).abs() <= 1);
        assert!((i32::from(sampled.b()) - 128).abs() <= 1);
    }
    #[test]
    fn native_menu_and_popup_close_with_paint_only_and_reduced_motion_snaps() {
        for popup in [false, true] {
            for reduced in [false, true] {
                let ctx = Context::default();
                install(&ctx);
                if reduced {
                    ctx.global_style_mut(|s| s.animation_time = 0.);
                }
                let mut open = true;
                for t in [0., 0.1, 0.2] {
                    let mut input=raw(t);
                    input.events.push(egui::Event::PointerMoved(egui::pos2(25.,45.)));
                    ctx.run_ui(input, |ui| {
                        if popup {
                            let button = ui.button("Open");
                            egui::Popup::from_response(&button)
                                .open_bool(&mut open)
                                .show(|ui| {
                                    ui.label("Motion popup item");
                                });
                        } else {
                            crate::menu::anchored(
                                ui,
                                egui::Id::unique("motion-menu"),
                                Rect::from_min_size(egui::pos2(20., 20.), egui::vec2(30., 20.)),
                                &mut open,
                                crate::menu::Menu {
                                    title: "Menu".into(),
                                    entries: vec![crate::menu::Entry::Row(
                                        crate::menu::Row::command(
                                            "motion-item",
                                            "Motion menu item",
                                            None,
                                            true,
                                        ),
                                    )],
                                },
                                &[],
                            );
                        }
                    })
                    .drop_without_applying_deltas();
                }
                open = false;
                let mut closed = ctx.run_ui(raw(0.3), |ui| {
                    if popup {
                        let button = ui.button("Open");
                        egui::Popup::from_response(&button)
                            .open_bool(&mut open)
                            .show(|ui| {
                                ui.label("Motion popup item");
                            });
                    } else {
                        crate::menu::anchored(
                            ui,
                            egui::Id::unique("motion-menu"),
                            Rect::NOTHING,
                            &mut open,
                            crate::menu::Menu {
                                title: "Menu".into(),
                                entries: vec![],
                            },
                            &[],
                        );
                    }
                });
                assert_eq!(
                    text_present(
                        &closed.shapes,
                        if popup {
                            "Motion popup item"
                        } else {
                            "Motion menu item"
                        }
                    ),
                    !reduced,
                    "popup={popup}, reduced={reduced}"
                );
                closed.textures_delta.clear();
                for t in [3., 4., 5.] {
                    ctx.run_ui(raw(t), |_| {}).drop_without_applying_deltas();
                }
                assert!(!ctx.has_requested_repaint());
            }
        }
    }
    #[test]
    fn tab_switch_retains_old_page_and_animates_new_page() {
        for reduced in [false, true] {
            let ctx = Context::default();
            if reduced {
                ctx.global_style_mut(|s| s.animation_time = 0.);
            }
            let id = egui::Id::unique("tab");
            ctx.run_ui(raw(0.), |ui| {
                content(ui, id, "one", |ui| {
                    ui.label("old tab page");
                });
            })
            .drop_without_applying_deltas();
            let mut switched = ctx.run_ui(raw(1.), |ui| {
                content(ui, id, "two", |ui| {
                    ui.label("new tab page");
                });
            });
            assert_eq!(text_present(&switched.shapes, "old tab page"), !reduced);
            assert!(text_present(&switched.shapes, "new tab page"));
            switched.textures_delta.clear();
            assert_eq!(
                state(&ctx, id).lock().unwrap().nodes.len(),
                if reduced { 0 } else { 2 }
            );
        }
    }
    #[test]
    fn stagger_samples_two_children_on_library_timing() {
        let ctx = Context::default();
        let mut state = SceneMotion::default();
        let enter = json!({"preset":"up","stagger_ms":100});
        for t in [0., 0.2] {
            let mut out = ctx.run_ui(raw(t), |ui| {
                state.begin(ui);
                let c = state.capture(ui, "container", None, Some(&enter), Some(&enter), true);
                for key in ["a", "b"] {
                    let c = state.capture(ui, key, Some("container"), None, None, true);
                    ui.painter().rect_filled(
                        Rect::from_min_size(egui::pos2(20., 20.), egui::vec2(30., 20.)),
                        0.,
                        egui::Color32::RED,
                    );
                    state.painted(ui, c, ui.min_rect());
                }
                state.painted(ui, c, ui.min_rect());
                state.end(ui);
            });
            let rectangles = out
                .shapes
                .iter()
                .filter_map(|s| {
                    if let egui::Shape::Rect(r) = &s.shape {
                        Some(r)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();
            for (index, r) in rectangles.iter().enumerate() {
                let expected = serde_json::from_value::<Entrance>(enter.clone())
                    .unwrap()
                    .playback(0., index as u32, false)
                    .sample(t);
                assert!((r.rect.top() - 20. - expected.get(Property::Ty) as f32).abs() < 0.001);
            }
            assert_eq!(rectangles.len(), 2);
            out.textures_delta.clear();
        }
        for (key, index) in [("a", 0), ("b", 1)] {
            let expected = serde_json::from_value::<Entrance>(enter.clone())
                .unwrap()
                .playback(0., index, false)
                .sample(0.2);
            assert_eq!(
                state.nodes[key].entrance.as_ref().unwrap().sample(0.2),
                expected
            );
        }
        assert_ne!(
            state.nodes["a"].entrance.as_ref().unwrap().sample(0.2),
            state.nodes["b"].entrance.as_ref().unwrap().sample(0.2)
        );
        ctx.run_ui(raw(1.), |ui| {
            state.begin(ui);
            state.end(ui);
        })
        .drop_without_applying_deltas();
        for (key, index) in [("a", 0), ("b", 1)] {
            let expected = serde_json::from_value::<Entrance>(enter.clone())
                .unwrap()
                .playback(1., index, true)
                .sample(1.2);
            assert_eq!(
                state.nodes[key].leaving.as_ref().unwrap().sample(1.2),
                expected
            );
        }
    }
    #[test]
    fn exit_retains_and_then_frees_image_texture() {
        let ctx = Context::default();
        let mut state = SceneMotion::default();
        let image = ctx.load_texture(
            "exit-image",
            egui::ColorImage::filled([2, 2], egui::Color32::RED),
            Default::default(),
        );
        let id = image.id();
        ctx.run_ui(raw(0.), |ui| {
            state.begin(ui);
            let c = state.capture(ui, "image", None, None, Some(&json!("fade")), true);
            let rect = Rect::from_min_size(egui::pos2(20., 20.), egui::vec2(20., 20.));
            ui.painter().image(
                id,
                rect,
                Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1., 1.)),
                egui::Color32::WHITE,
            );
            state.painted(ui, c, rect);
            state.end(ui);
        })
        .drop_without_applying_deltas();
        drop(image);
        assert!(ctx.tex_manager().read().meta(id).is_some());
        ctx.run_ui(raw(0.1), |ui| {
            state.begin(ui);
            state.end(ui);
        })
        .drop_without_applying_deltas();
        assert!(ctx.tex_manager().read().meta(id).is_some());
        ctx.run_ui(raw(3.), |ui| {
            state.begin(ui);
            state.end(ui);
        })
        .drop_without_applying_deltas();
        assert!(ctx.tex_manager().read().meta(id).is_none());
    }
    #[test]
    fn settled_scene_arms_no_repaint() {
        let ctx = Context::default();
        install(&ctx);
        let mut state = SceneMotion::default();
        for t in [0., 1., 2., 3., 4.] {
            frame(&ctx, &mut state, t, true);
        }
        assert!(!ctx.has_requested_repaint());
        for t in [5., 7., 8., 9.] {
            frame(&ctx, &mut state, t, false);
        }
        assert!(!ctx.has_requested_repaint());
    }
    fn raw(t: f64) -> egui::RawInput {
        egui::RawInput {
            time: Some(t),
            screen_rect: Some(Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(400., 400.),
            )),
            ..Default::default()
        }
    }
    fn frame(ctx: &Context, state: &mut SceneMotion, t: f64, shown: bool) -> Vec<ClippedShape> {
        let mut out = ctx.run_ui(raw(t), |ui| {
            state.begin(ui);
            if shown {
                let c = state.capture(
                    ui,
                    "node",
                    None,
                    Some(&json!("up")),
                    Some(&json!("fade")),
                    true,
                );
                let rect = Rect::from_min_size(egui::pos2(20., 20.), egui::vec2(100., 30.));
                ui.painter().rect_filled(rect, 0., egui::Color32::RED);
                state.painted(ui, c, rect);
            }
            state.end(ui);
        });
        out.textures_delta.clear();
        out.shapes
    }
    #[test]
    fn entrance_samples_library_and_retains_exit_paint_only() {
        let ctx = Context::default();
        let mut state = SceneMotion::default();
        frame(&ctx, &mut state, 0., true);
        for t in [0.175, 0.35, 0.7] {
            let shapes = frame(&ctx, &mut state, t, true);
            let rect = shapes
                .iter()
                .find_map(|s| {
                    if let egui::Shape::Rect(r) = &s.shape {
                        Some(r)
                    } else {
                        None
                    }
                })
                .unwrap();
            let p = motion::presets()
                .into_iter()
                .find(|p| p.name == "up")
                .unwrap();
            assert!((rect.rect.top() - 20. - p.sample(t).get(Property::Ty) as f32).abs() < 0.001);
            assert!(
                (rect.fill.a() as f64 / 255. - p.sample(t).get(Property::Opacity)).abs() < 0.005
            );
        }
        let shapes = frame(&ctx, &mut state, 1., false);
        assert!(
            shapes
                .iter()
                .any(|s| matches!(s.shape, egui::Shape::Rect(_)))
        );
        assert!(state.nodes["node"].leaving.is_some());
        frame(&ctx, &mut state, 3., false);
        assert!(state.nodes.is_empty());
    }
    #[test]
    fn reduced_motion_clears_lifecycle_and_snaps_widget_curves() {
        let ctx = Context::default();
        let mut state = SceneMotion::default();
        ctx.global_style_mut(|s| s.animation_time = 0.);
        let shapes = frame(&ctx, &mut state, 0., true);
        assert!(state.nodes.is_empty());
        let rect = shapes
            .iter()
            .find_map(|s| {
                if let egui::Shape::Rect(r) = &s.shape {
                    Some(r)
                } else {
                    None
                }
            })
            .unwrap();
        assert_eq!(rect.rect.top(), 20.);
        assert_eq!(rect.fill.a(), 255);
        let mut sample = 0.;
        ctx.run_ui(raw(1.), |ui| {
            sample = scalar(ui.ctx(), egui::Id::unique("toggle"), 1., 160., true)
        })
        .drop_without_applying_deltas();
        assert_eq!(sample, 1.);
    }
    #[test]
    fn widget_curves_retarget_from_presented_value() {
        let ctx = Context::default();
        let id = egui::Id::unique("widget");
        let mut sample = 0.;
        for (t, target, expected) in [
            (0., 0., 0.),
            (1., 1., 0.),
            (1.08, 1., motion::Curve::EaseOut.sample(0.5)),
            (1.16, 1., 1.),
        ] {
            ctx.run_ui(raw(t), |ui| {
                sample = scalar(ui.ctx(), id, target, 160., true)
            })
            .drop_without_applying_deltas();
            assert!(
                (sample as f64 - expected).abs() < 1e-5,
                "{t}: {sample} vs {expected}"
            );
        }
    }
    #[test]
    fn bound_transition_retargets_without_a_jump() {
        let ctx = Context::default();
        let spec = json!({"prop":"opacity","ms":100,"ease":{"kind":"linear"}});
        let mut v = Value::Null;
        for (t, target, expected) in [
            (0., 0., 0.),
            (1., 1., 0.),
            (1.05, 1., 0.5),
            (1.05, 0., 0.5),
            (1.1, 0., 0.25),
            (1.15, 0., 0.),
        ] {
            ctx.run_ui(raw(t), |ui| {
                v = value(
                    ui.ctx(),
                    "bound",
                    "opacity",
                    &json!(target),
                    Some(&spec),
                    true,
                )
            })
            .drop_without_applying_deltas();
            assert!((v.as_f64().unwrap() - expected).abs() < 1e-6);
        }
    }
}
