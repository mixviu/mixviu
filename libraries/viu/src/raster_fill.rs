// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Shared retained image-fill with original-pixel placement and crossfade.
//! Composition/placement follow the pinned native toolkit raster contract.
use crate::{
    raster::{Pixels, Rasters, TextureLease},
    raster_fit::{self, Fit},
};
use egui::{
    Color32, ColorImage, Rect, TextureHandle, TextureOptions, TextureWrapMode, Ui, pos2, vec2,
};
use serde_json::{Value, json};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
struct Slot {
    handle: String,
    pixels: Arc<Pixels>,
    texture: TextureHandle,
    base: Color32,
    fit: Fit,
    _lease: TextureLease,
}
#[derive(Default)]
pub struct Fill {
    observer: Option<crate::raster::RepaintLease>,
    current: Option<Slot>,
    previous: Option<Slot>,
    transition: Value,
    started: Option<Instant>,
    pub status: Value,
}
fn base(ui: &Ui, value: &Value) -> Color32 {
    if value["theme"] == true {
        return ui.visuals().panel_fill;
    }
    let c = value["colour"].as_array().expect("validated colour");
    Color32::from_rgb(
        c[0].as_f64().unwrap() as u8,
        c[1].as_f64().unwrap() as u8,
        c[2].as_f64().unwrap() as u8,
    )
}
fn upload(image: &crate::raster_decode::Decoded, fallback: Color32) -> ColorImage {
    let [w, h] = image.size;
    let base: egui::Rgba = fallback.into();
    let pixels = image
        .rgba
        .as_raw()
        .chunks_exact(4)
        .map(|p| match p[3] {
            255 => Color32::from_rgb(p[0], p[1], p[2]),
            0 => fallback,
            _ => {
                let source: egui::Rgba =
                    Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]).into();
                Color32::from(source + base * (1.0 - source.a()))
            }
        })
        .collect();
    ColorImage::new([w as usize, h as usize], pixels)
}
fn options(fit: Fit) -> TextureOptions {
    TextureOptions {
        wrap_mode: if fit == Fit::Tile {
            TextureWrapMode::Repeat
        } else {
            TextureWrapMode::ClampToEdge
        },
        ..TextureOptions::LINEAR
    }
}
fn placement(ui: &Ui, slot: &Slot, rect: Rect) -> (Rect, Rect) {
    let p = raster_fit::placement(
        slot.pixels.decoded.original_size,
        [rect.width(), rect.height()],
        ui.ctx().pixels_per_point(),
        slot.fit,
    );
    (
        Rect::from_min_max(
            rect.min + vec2(p.min[0], p.min[1]),
            rect.min + vec2(p.max[0], p.max[1]),
        ),
        Rect::from_min_max(
            pos2(p.uv.min[0], p.uv.min[1]),
            pos2(p.uv.max[0], p.uv.max[1]),
        ),
    )
}
fn paint(ui: &Ui, slot: &Slot, rect: Rect, alpha: f32) {
    let (dest, uv) = placement(ui, slot, rect);
    ui.painter().image(
        slot.texture.id(),
        dest,
        uv,
        Color32::from_white_alpha((alpha.clamp(0.0, 1.0) * 255.0).round() as u8),
    );
}
impl Fill {
    pub fn draw(&mut self, ui: &mut Ui, value: &Value, rasters: &Rasters) -> egui::Response {
        if self.observer.is_none(){self.observer=Some(rasters.observe(ui.ctx()));}
        let rect = ui.available_rect_before_wrap();
        let colour = base(ui, value);
        let handle = value["handle"].as_str();
        let fit: Fit = serde_json::from_value(value["fit"].clone()).expect("validated fit");
        let mut error = None;
        let transition = &value["transition"];
        if &self.transition != transition {
            self.transition = transition.clone();
            self.started = if transition.is_object() {
                Some(
                    Instant::now()
                        - Duration::from_millis(
                            transition["elapsed_ms"].as_u64().unwrap_or(0).min(500),
                        ),
                )
            } else {
                None
            };
        }
        let progress = self
            .started
            .map_or(1.0, |at| (at.elapsed().as_secs_f32() / 0.5).clamp(0.0, 1.0));
        if progress >= 1.0 {
            self.previous = None;
        }
        if let Some(handle) = handle {
            if self.current.as_ref().is_none_or(|s| s.handle != handle) {
                self.previous = None;
                match rasters.get(handle) {
                    None => error = Some("decoded raster handle is no longer available".into()),
                    Some(pixels) => {
                        let maximum = ui.input(|i| i.max_texture_side);
                        if pixels.decoded.size.iter().any(|s| *s as usize > maximum) {
                            error = Some(format!(
                                "image texture {} × {} exceeds the device's {maximum}-pixel side limit",
                                pixels.decoded.size[0], pixels.decoded.size[1]
                            ));
                        } else {
                            match rasters.texture(pixels.decoded.bytes()) {
                                Err(e) => {
                                    self.current = None;
                                    error = Some(e);
                                }
                                Ok(lease) => {
                                    let texture = ui.ctx().load_texture(
                                        "retained-image-fill",
                                        upload(&pixels.decoded, colour),
                                        options(fit),
                                    );
                                    let outgoing = self.current.replace(Slot {
                                        handle: handle.into(),
                                        pixels,
                                        texture,
                                        base: colour,
                                        fit,
                                        _lease: lease,
                                    });
                                    if progress < 1.0 {
                                        self.previous = outgoing;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        } else {
            self.current = None;
            self.previous = None;
        }
        if let Some(slot) = &mut self.current {
            if slot.base != colour || slot.fit != fit {
                slot.texture
                    .set(upload(&slot.pixels.decoded, colour), options(fit));
                slot.base = colour;
                slot.fit = fit;
            }
        }
        let shows = if let Some(slot) = &self.current {
            if let Some(previous) = self.previous.as_ref().filter(|_| progress < 1.0) {
                ui.painter().rect_filled(rect, 0.0, previous.base);
                paint(ui, previous, rect, 1.0);
                let (dest, _) = placement(ui, slot, rect);
                for bar in [
                    Rect::from_min_max(rect.min, pos2(rect.max.x, dest.min.y)),
                    Rect::from_min_max(pos2(rect.min.x, dest.max.y), rect.max),
                    Rect::from_min_max(pos2(rect.min.x, dest.min.y), pos2(dest.min.x, dest.max.y)),
                    Rect::from_min_max(pos2(dest.max.x, dest.min.y), pos2(rect.max.x, dest.max.y)),
                ] {
                    if bar.width() > 0.0 && bar.height() > 0.0 {
                        ui.painter()
                            .rect_filled(bar, 0.0, colour.gamma_multiply(progress));
                    }
                }
                paint(ui, slot, rect, progress);
                ui.ctx().request_repaint();
            } else {
                ui.painter().rect_filled(rect, 0.0, colour);
                paint(ui, slot, rect, 1.0);
            }
            "image"
        } else {
            ui.painter().rect_filled(rect, 0.0, colour);
            if handle.is_some() {
                "fallback"
            } else if value["theme"] == true {
                "theme"
            } else {
                "colour"
            }
        };
        self.status =
            json!({"shows":shows,"error":error,"fit":if handle.is_some(){Some(fit)}else{None}});
        ui.allocate_rect(rect, egui::Sense::hover())
    }
}
