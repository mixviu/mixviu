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
    parent: Option<String>,
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
        let t = self.spec.progress(self.start, now);
        self.from
            .iter()
            .zip(&self.to)
            .map(|(a, b)| a * (1.0 - t) + b * t)
            .collect()
    }
}
pub struct Capture {
    key: String,
    layer: LayerId,
    start: ShapeIdx,
}
impl SceneMotion {
    pub fn begin(&mut self, ui: &Ui) {
        self.now = ui.input(|i| i.time);
        self.reduced = ui.style().animation_time == 0.0;
        for n in self.nodes.values_mut() {
            n.seen = false;
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
        if enter.is_none() && exit.is_none() {
            return None;
        }
        let now = self.now;
        let n = self
            .nodes
            .entry(key.to_owned())
            .or_insert_with(|| NodeMotion {
                seen: true,
                entrance: enter
                    .and_then(|v| serde_json::from_value::<Entrance>(v.clone()).ok())
                    .map(|e| e.playback(now, 0, false)),
                exit: None,
                leaving: None,
                shapes: Vec::new(),
                rect: Rect::NOTHING,
                parent: parent.map(str::to_owned),
            });
        n.seen = true;
        n.leaving = None;
        n.exit = exit.and_then(|v| serde_json::from_value(v.clone()).ok());
        let layer = ui.layer_id();
        let start = ui.ctx().graphics_mut(|g| g.entry(layer).next_idx());
        Some(Capture {
            key: key.into(),
            layer,
            start,
        })
    }
    pub fn painted(&mut self, ui: &Ui, capture: Option<Capture>, rect: Rect) {
        let Some(c) = capture else {
            return;
        };
        let n = self.nodes.get_mut(&c.key).unwrap();
        n.rect = rect;
        let now = self.now;
        let values = motion::scene::sample(n.entrance.as_ref(), now, self.reduced);
        let active = n
            .entrance
            .as_ref()
            .is_some_and(|p| motion::scene::active(p, now));
        ui.ctx().graphics_mut(|g| {
            let list = g.entry(c.layer);
            let end = list.next_idx();
            if n.exit.is_some() {
                n.shapes = list
                    .all_entries()
                    .skip(c.start.0)
                    .take(end.0 - c.start.0)
                    .cloned()
                    .collect();
            }
            for i in c.start.0..end.0 {
                list.mutate_shape(ShapeIdx(i), |s| transform(s, rect, &values));
            }
        });
        if active {
            ui.ctx().request_repaint();
        }
    }
    pub fn end(&mut self, ui: &Ui) {
        if self.reduced {
            self.nodes.clear();
            return;
        }
        let missing = self
            .nodes
            .iter()
            .filter(|(_, n)| !n.seen)
            .map(|(id, _)| id.clone())
            .collect::<std::collections::BTreeSet<_>>();
        let now = self.now;
        self.nodes.retain(|_, n| {
            if n.seen {
                return true;
            }
            if n.parent.as_ref().is_some_and(|p| missing.contains(p)) {
                return false;
            }
            let Some(exit) = &n.exit else {
                return false;
            };
            let p = n.leaving.get_or_insert_with(|| exit.playback(now, 0, true));
            if !motion::scene::active(p, now) {
                return false;
            }
            let values = p.sample(now);
            for shape in &n.shapes {
                let mut shape = shape.clone();
                transform(&mut shape, n.rect, &values);
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
        let current =
            s.from * (1.0 - curve.progress(s.start, now)) + s.to * curve.progress(s.start, now);
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
            s.from * (1.0 - t) + s.to * t
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
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
