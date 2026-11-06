// SPDX-License-Identifier: MIT OR Apache-2.0
//! Capped PNG preview display. The application chooses a cache entry; decoding
//! and GPU texture ownership belong to this shared renderer primitive.
use egui::{TextureHandle, Ui};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
type Key = (String, String);
pub fn validate(value: &Value) -> Result<(), String> {
    if !["none", "pending", "ready", "skipped", "failed"]
        .contains(&value["phase"].as_str().unwrap_or(""))
    {
        return Err("unknown image phase".into());
    }
    if value["phase"] == "ready" && value["handle"].is_null()
        && value["path"]
            .as_str()
            .is_none_or(|s| s.len() > 4096 || !std::path::Path::new(s).is_absolute())
    {
        return Err("ready image requires an absolute cache path".into());
    }
    if !value["handle"].is_null() && value["handle"].as_str().is_none_or(|h| !h.starts_with("raster:") || h.len() > 64) {
        return Err("invalid retained raster handle".into());
    }
    Ok(())
}
struct Slot {
    path: String,
    texture: Option<TextureHandle>,
    answered: bool,
    pixels: Option<std::sync::Arc<crate::raster::Pixels>>,
    _lease: Option<crate::raster::TextureLease>,
}
pub(crate) struct Images {
    rasters: Option<crate::raster::Rasters>,
    decoder: crate::thumbnail::Decoder<Key>,
    slots: BTreeMap<String, Slot>,
    visible: BTreeSet<String>,
}
impl Default for Images {
    fn default() -> Self {
        Self {
            rasters: None,
            decoder: crate::thumbnail::Decoder::new("viu-preview-decode"),
            slots: BTreeMap::new(),
            visible: BTreeSet::new(),
        }
    }
}
impl Images {
    pub fn set_rasters(&mut self, rasters: Option<crate::raster::Rasters>) {
        self.rasters = rasters;
    }
    pub fn surface() -> Self {
        Self {
            decoder: crate::thumbnail::Decoder::with_decoder("viu-surface-decode", decode_surface),
            ..Self::default()
        }
    }
    pub fn states(&self) -> Value {
        serde_json::json!(self.slots.iter().map(|(id,s)|(id.clone(),serde_json::json!({"path":s.path,"phase":if s.texture.is_some(){"ready"}else if s.answered{"failed"}else{"pending"}}))).collect::<BTreeMap<_,_>>())
    }
    pub fn begin(&mut self) {
        self.visible.clear();
    }
    pub fn finish(&mut self) {
        self.slots.retain(|id, _| self.visible.contains(id));
        self.decoder.retain(
            &self
                .slots
                .iter()
                .map(|(id, s)| (id.clone(), s.path.clone()))
                .collect::<Vec<_>>(),
        );
    }
    pub fn draw(&mut self, ui: &mut Ui, id: &str, value: &Value) -> egui::Response {
        self.draw_internal(ui, id, value, false)
    }
    pub fn draw_fill(&mut self, ui: &mut Ui, id: &str, value: &Value) -> egui::Response {
        self.draw_internal(ui, id, value, true)
    }
    fn draw_internal(
        &mut self,
        ui: &mut Ui,
        id: &str,
        value: &Value,
        fill: bool,
    ) -> egui::Response {
        self.visible.insert(id.into());
        let rect = ui.available_rect_before_wrap();
        if fill {
            let c = value["colour"].as_array().unwrap();
            ui.painter().rect_filled(
                rect,
                0.0,
                egui::Color32::from_rgb(
                    c[0].as_f64().unwrap() as u8,
                    c[1].as_f64().unwrap() as u8,
                    c[2].as_f64().unwrap() as u8,
                ),
            );
        }
        if value["phase"] != "ready" {
            self.slots.remove(id);
            return ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover());
        }
        let handle = value["handle"].as_str();
        let path = handle.or_else(|| value["path"].as_str()).unwrap_or("");
        if handle.is_none() && self.slots.get(id).is_some_and(|s| s.path != path) {
            self.slots.remove(id);
        }
        if !self.slots.contains_key(id) || self.slots.get(id).is_some_and(|s| s.path != path) {
            if self.slots.len() >= 16 {
                return ui.label("Preview capacity reached");
            }
            if let Some(handle) = handle {
                if let Some(pixels) = self.rasters.as_ref().and_then(|r| r.get(handle)) {
                    let decoded = &pixels.decoded;
                    let limit = ui.input(|i| i.max_texture_side);
                    let lease = self.rasters.as_ref().unwrap().texture(decoded.bytes());
                    if decoded.size.iter().all(|s| *s as usize <= limit) && let Ok(lease) = lease {
                    let image = egui::ColorImage::from_rgba_unmultiplied([decoded.size[0] as usize, decoded.size[1] as usize], decoded.rgba.as_raw());
                    let texture = ui.ctx().load_texture(handle, image, egui::TextureOptions::LINEAR);
                    self.slots.insert(id.into(), Slot { path: path.into(), texture: Some(texture), answered: true, pixels: Some(pixels), _lease: Some(lease) });
                    }
                }
                // A released or unknown handle never falls back to opening a path.
                if !self.slots.contains_key(id) {
                    return ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover());
                }
            } else {
            let key = (id.to_owned(), path.to_owned());
            let accepted = self.decoder.ask(&key, std::path::Path::new(path), ui.ctx());
            self.slots.insert(
                id.into(),
                Slot {
                    path: path.into(),
                    texture: None,
                    answered: !accepted,
                    pixels: None,
                    _lease: None,
                },
            );
            }
        }
        for ((id, path), image) in self.decoder.drain() {
            if let Some(slot) = self.slots.get_mut(&id).filter(|s| s.path == path) {
                slot.answered = true;
                slot.texture = image.map(|image| {
                    ui.ctx().load_texture(
                        format!("preview:{id}:{path}"),
                        image,
                        egui::TextureOptions::LINEAR,
                    )
                });
            }
        }
        if let Some(texture) = self.slots.get(id).and_then(|s| s.texture.as_ref()) {
            let natural = self.slots.get(id).and_then(|s| s.pixels.as_ref()).map_or_else(
                || texture.size_vec2(),
                |p| egui::vec2(p.decoded.original_size[0] as f32, p.decoded.original_size[1] as f32),
            ) / ui.ctx().pixels_per_point();
            if fill {
                let fit = value["fit"].as_str().unwrap();
                let scale = match fit {
                    "fill" => (rect.width() / natural.x).max(rect.height() / natural.y),
                    "fit" => (rect.width() / natural.x).min(rect.height() / natural.y),
                    _ => 1.0,
                };
                let size = if fit == "stretch" {
                    rect.size()
                } else {
                    natural * scale
                };
                let painter = ui.painter().with_clip_rect(rect);
                if fit == "tile" {
                    let cols = (rect.width() / size.x).ceil() as usize;
                    let rows = (rect.height() / size.y).ceil() as usize;
                    if cols.saturating_mul(rows) <= 4096 {
                        for y in 0..rows {
                            for x in 0..cols {
                                painter.image(
                                    texture.id(),
                                    egui::Rect::from_min_size(
                                        rect.min + egui::vec2(x as f32 * size.x, y as f32 * size.y),
                                        size,
                                    ),
                                    egui::Rect::from_min_max(
                                        egui::Pos2::ZERO,
                                        egui::pos2(1.0, 1.0),
                                    ),
                                    egui::Color32::WHITE,
                                );
                            }
                        }
                    }
                } else {
                    painter.image(
                        texture.id(),
                        egui::Rect::from_center_size(rect.center(), size),
                        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
                return ui.allocate_rect(rect, egui::Sense::hover());
            }
            let scale = (ui.available_width() / natural.x)
                .min(240.0 / natural.y)
                .min(1.0);
            ui.vertical_centered(|ui| {
                ui.add(egui::Image::new(texture).fit_to_exact_size(natural * scale));
            })
            .response
        } else {
            ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
        }
    }
}

pub fn validate_fill(value: &Value) -> Result<(), String> {
    if !value["colour"].as_array().is_some_and(|c| {
        c.len() == 3
            && c.iter().all(|v| {
                v.as_f64().is_some_and(|n| {
                    n.is_finite() && (0.0..=255.0).contains(&n) && n.fract() == 0.0
                })
            })
    }) || !["fill", "fit", "centre", "tile", "stretch"]
        .contains(&value["fit"].as_str().unwrap_or(""))
    {
        return Err("image-fill requires RGB colour and fill/fit/centre/tile/stretch".into());
    }
    validate(value)
}
pub(crate) fn decode_surface(path: &std::path::Path) -> Option<egui::ColorImage> {
    let bytes = thumbs::cache::read_regular(path, 32 * 1024 * 1024).ok()?;
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    if !matches!(
        reader.format(),
        Some(image::ImageFormat::Png | image::ImageFormat::Jpeg | image::ImageFormat::WebP)
    ) {
        return None;
    }
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let image = reader.decode().ok()?;
    if u64::from(image.width()) * u64::from(image.height()) > 16_777_216 {
        return None;
    }
    let image = image.to_rgba8();
    Some(egui::ColorImage::from_rgba_unmultiplied(
        [image.width() as usize, image.height() as usize],
        image.as_raw(),
    ))
}
