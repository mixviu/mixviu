//! Scene painter adapter. Retained frames contain paint only, never widgets,
//! routes or model state, so departing nodes cannot consume input.
use std::collections::BTreeMap;
use egui::{epaint::ClippedShape, layers::ShapeIdx, Context, LayerId, Rect, Ui};
use motion::{Property, Preset};
use motion::scene::{Entrance, Transition};
use serde_json::Value;

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
    values: BTreeMap<String, Tween>,
}
struct Tween { from: Vec<f64>, to: Vec<f64>, start: f64, spec: Transition }
impl Tween {
    fn sample(&self, now: f64) -> Vec<f64> {
        let t = self.spec.progress(self.start, now);
        self.from.iter().zip(&self.to).map(|(a,b)| a*(1.0-t)+b*t).collect()
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
        for n in self.nodes.values_mut() { n.seen = false; }
    }
    pub fn capture(&mut self, ui: &Ui, key: &str, parent: Option<&str>, enter: Option<&Value>, exit: Option<&Value>, enabled: bool) -> Option<Capture> {
        if !enabled || self.reduced { self.nodes.remove(key); return None; }
        if enter.is_none() && exit.is_none() { return None; }
        let now = self.now;
        let n = self.nodes.entry(key.to_owned()).or_insert_with(|| NodeMotion {
            seen: true,
            entrance: enter.and_then(|v| serde_json::from_value::<Entrance>(v.clone()).ok()).map(|e|e.playback(now,0,false)),
            exit: None, leaving: None, shapes: Vec::new(), rect: Rect::NOTHING,
            parent: parent.map(str::to_owned), values: BTreeMap::new(),
        });
        n.seen = true;
        n.leaving = None;
        n.exit = exit.and_then(|v|serde_json::from_value(v.clone()).ok());
        let layer = ui.layer_id();
        let start = ui.ctx().graphics_mut(|g|g.entry(layer).next_idx());
        Some(Capture { key:key.into(), layer, start })
    }
    pub fn painted(&mut self, ui: &Ui, capture: Option<Capture>, rect: Rect) {
        let Some(c) = capture else { return; };
        let n = self.nodes.get_mut(&c.key).unwrap();
        n.rect = rect;
        let now = self.now;
        let values = motion::scene::sample(n.entrance.as_ref(), now, self.reduced);
        let active = n.entrance.as_ref().is_some_and(|p|motion::scene::active(p,now));
        ui.ctx().graphics_mut(|g| {
            let list = g.entry(c.layer);
            let end = list.next_idx();
            if n.exit.is_some() { n.shapes = list.all_entries().skip(c.start.0).take(end.0-c.start.0).cloned().collect(); }
            for i in c.start.0..end.0 {
                list.mutate_shape(ShapeIdx(i), |s|transform(s, rect, &values));
            }
        });
        if active { ui.ctx().request_repaint(); }
    }
    pub fn end(&mut self, ui: &Ui) {
        if self.reduced { self.nodes.clear(); return; }
        let missing = self.nodes.iter().filter(|(_,n)|!n.seen).map(|(id,_)|id.clone()).collect::<std::collections::BTreeSet<_>>();
        let now = self.now;
        self.nodes.retain(|_,n| {
            if n.seen { return true; }
            if n.parent.as_ref().is_some_and(|p|missing.contains(p)) { return false; }
            let Some(exit) = &n.exit else { return false; };
            let p = n.leaving.get_or_insert_with(|| exit.playback(now,0,true));
            if !motion::scene::active(p,now) { return false; }
            let values = p.sample(now);
            for shape in &n.shapes {
                let mut shape = shape.clone();
                transform(&mut shape,n.rect,&values);
                ui.painter().with_clip_rect(shape.clip_rect).add(shape.shape);
            }
            ui.ctx().request_repaint();
            true
        });
    }
}
fn transform(shape: &mut ClippedShape, rect: Rect, values: &motion::Values) {
    let scale = values.get(Property::Scale) as f32;
    let offset = egui::vec2(values.get(Property::Tx) as f32,values.get(Property::Ty) as f32);
    shape.shape.transform(egui::emath::TSTransform { scaling: scale, translation: rect.center().to_vec2()*(1.0-scale)+offset });
    let opacity = values.get(Property::Opacity).clamp(0.0,1.0) as f32;
    if opacity != 1.0 {
        egui::epaint::shape_transform::adjust_colors(&mut shape.shape,|c| {
            if *c != egui::Color32::PLACEHOLDER { *c = c.gamma_multiply(opacity); }
        });
    }
}

/// Shared deterministic state transition for built-in widgets. Retargets from
/// the value sampled at this frame; settled widgets request no repaint.
pub fn scalar(ctx: &Context, id: egui::Id, target: f64, ms: f64, enabled: bool) -> f32 {
    #[derive(Clone)]
    struct State { from:f64, to:f64, start:f64 }
    let now = ctx.input(|i|i.time);
    let reduced = ctx.style().animation_time == 0.0 || !enabled;
    let curve = Transition { prop:"opacity".into(),ms,ease:motion::Curve::EaseOut };
    let (value,active) = ctx.data_mut(|d| {
        let mut s = d.get_temp::<State>(id).unwrap_or(State { from:target,to:target,start:now });
        let current = s.from*(1.0-curve.progress(s.start,now))+s.to*curve.progress(s.start,now);
        if s.to != target { s = State { from:current,to:target,start:now }; }
        let t = curve.progress(s.start,now);
        let value = if reduced { target } else { s.from*(1.0-t)+s.to*t };
        let active = !reduced && s.from != s.to && t < 1.0;
        if reduced { s.from=target; s.to=target; }
        d.insert_temp(id,s);
        (value as f32,active)
    });
    if active { ctx.request_repaint(); }
    value
}
