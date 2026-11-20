// SPDX-License-Identifier: MIT OR Apache-2.0
//! Static fields rasterised once per node/size/resolved colour inputs.
use egui::{Color32, Rect, Shape, TextureHandle};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Default)]
pub(crate) struct Backdrops(BTreeMap<String, Entry>);
struct Entry {
    size: egui::Vec2,
    base: Color32,
    spec: Value,
    texture: TextureHandle,
    seen: bool,
}
impl Backdrops {
    pub fn begin(&mut self) {
        for e in self.0.values_mut() {
            e.seen = false;
        }
    }
    pub fn end(&mut self) {
        self.0.retain(|_, e| e.seen);
    }
    pub fn shape(
        &mut self,
        ctx: &egui::Context,
        key: &str,
        rect: Rect,
        base: Color32,
        spec: &Value,
    ) -> Shape {
        let e = self
            .0
            .entry(key.into())
            .or_insert_with(|| entry(ctx, rect.size(), base, spec));
        if e.size != rect.size() || e.base != base || e.spec != *spec {
            *e = entry(ctx, rect.size(), base, spec);
        }
        e.seen = true;
        Shape::image(
            e.texture.id(),
            rect,
            Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1., 1.)),
            Color32::WHITE,
        )
    }
}
fn entry(ctx: &egui::Context, size: egui::Vec2, base: Color32, spec: &Value) -> Entry {
    // Low-frequency field, bilinearly sampled; fixed bounded texture storage.
    let (w, h) = (512, 288);
    let mut pixels = Vec::with_capacity(w * h);
    for y in 0..h {
        for x in 0..w {
            let mut rgb = [base.r() as f64, base.g() as f64, base.b() as f64];
            for r in spec.as_array().into_iter().flatten().rev() {
                let dx = (x as f64 / (w - 1) as f64 - r["x"].as_f64().unwrap()) * size.x as f64
                    / r["rx"].as_f64().unwrap();
                let dy = (y as f64 / (h - 1) as f64 - r["y"].as_f64().unwrap()) * size.y as f64
                    / r["ry"].as_f64().unwrap();
                let a = (1. - dx.hypot(dy) / r["stop"].as_f64().unwrap()).clamp(0., 1.);
                let c = u32::from_str_radix(&r["color"].as_str().unwrap()[1..], 16).unwrap();
                for (i, ch) in rgb.iter_mut().enumerate() {
                    *ch = *ch * (1. - a) + ((c >> (16 - i * 8)) & 255) as f64 * a;
                }
            }
            pixels.push(Color32::from_rgb(
                rgb[0].round() as u8,
                rgb[1].round() as u8,
                rgb[2].round() as u8,
            ));
        }
    }
    ctx.data_mut(|d| {
        let id = egui::Id::new("viu-backdrop-builds");
        let n = d.get_temp::<usize>(id).unwrap_or(0);
        d.insert_temp(id, n + 1);
    });
    let texture = ctx.load_texture(
        "scene backdrop",
        egui::ColorImage::new([w, h], pixels),
        egui::TextureOptions::LINEAR,
    );
    Entry {
        size,
        base,
        spec: spec.clone(),
        texture,
        seen: true,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn static_fields_are_reused_and_only_inputs_invalidate() {
        let ctx = egui::Context::default();
        let mut cache = Backdrops::default();
        let mut rect = Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1600., 900.));
        let mut spec = serde_json::json!([{ "x":0.85,"y":-0.1,"rx":1200,"ry":700,"stop":0.6,"color":"#173048"}]);
        for _ in 0..100 {
            cache.begin();
            let _ = cache.shape(&ctx, "root", rect, Color32::BLACK, &spec);
            cache.end();
        }
        let builds = || {
            ctx.data(|d| {
                d.get_temp::<usize>(egui::Id::new("viu-backdrop-builds"))
                    .unwrap()
            })
        };
        assert_eq!(builds(), 1);
        rect = rect.translate(egui::vec2(20., 30.));
        let _ = cache.shape(&ctx, "root", rect, Color32::BLACK, &spec);
        assert_eq!(builds(), 1);
        rect.max.x += 1.;
        let _ = cache.shape(&ctx, "root", rect, Color32::BLACK, &spec);
        assert_eq!(builds(), 2);
        let _ = cache.shape(&ctx, "root", rect, Color32::BLUE, &spec);
        assert_eq!(builds(), 3);
        spec[0]["color"] = serde_json::json!("#123456");
        let _ = cache.shape(&ctx, "root", rect, Color32::BLUE, &spec);
        assert_eq!(builds(), 4);
        cache.begin();
        cache.end();
        assert!(cache.0.is_empty());
    }
}
