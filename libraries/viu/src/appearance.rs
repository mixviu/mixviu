// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared application typography and palette adaptation. The application
//! supplies its chosen mode, accent and metrics; this module owns egui wiring.
use egui::{Color32, FontFamily, FontId, TextStyle};
use serde_json::Value;
#[derive(Clone)]
pub struct ResolvedAppearance {
    pub style: egui::Style,
    pub colours: std::collections::BTreeMap<String, Color32>,
    pub body_weight:u16,
    pub heading_weight:u16,
    pub icon_stroke:f32,
    pub icon_mask:[u8;4],
}
pub fn colour(ctx: &egui::Context, name: &str) -> Option<Color32> {
    ctx.data(|d| {
        d.get_temp::<std::sync::Arc<std::collections::BTreeMap<String, Color32>>>(egui::Id::unique(
            "viu-design-colours",
        ))
    })?
    .get(name)
    .copied()
}
pub fn install(ctx: &egui::Context) {
    crate::fonts::install(ctx,400,600);
    crate::icons::install(ctx);
}
pub fn apply(ctx: &egui::Context, description: &Value, accent: [u8; 3]) {
    let mut style = egui::Style::default();
    style.visuals = if description["mode"] == "light" {
        egui::Visuals::light()
    } else {
        egui::Visuals::dark()
    };
    let body = description["body"]
        .as_f64()
        .unwrap_or(14.666667)
        .clamp(8.0, 36.0) as f32;
    for (text, size, family) in [
        (TextStyle::Body, body, FontFamily::Proportional),
        (TextStyle::Button, body, FontFamily::Proportional),
        (TextStyle::Small, 12.0, FontFamily::Proportional),
        (TextStyle::Heading, body * 1.35, FontFamily::Proportional),
        (TextStyle::Monospace, body, FontFamily::Monospace),
    ] {
        style.text_styles.insert(text, FontId::new(size, family));
    }
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.interact_size.y = 24.0;
    style.visuals.selection.bg_fill = Color32::from_rgb(accent[0], accent[1], accent[2]);
    style.visuals.selection.stroke = egui::Stroke::new(1.0, Color32::WHITE);
    let theme = if description["mode"] == "light" {
        egui::Theme::Light
    } else {
        egui::Theme::Dark
    };
    ctx.set_theme(theme);
    ctx.set_style_of(theme, style);
}

/// Resolve before committing a scene update so a bad design leaves the live
/// model, geometry and style intact.
pub fn resolve(description: &Value) -> Result<ResolvedAppearance, String> {
    crate::design_style::resolve(description)
}

pub fn install_style(ctx: &egui::Context, description: &Value, appearance: ResolvedAppearance) {
    crate::fonts::install(ctx,appearance.body_weight,appearance.heading_weight);
    crate::icons::install_style(ctx,appearance.icon_stroke,appearance.icon_mask);
    let theme = if description["mode"] == "light" {
        egui::Theme::Light
    } else {
        egui::Theme::Dark
    };
    let style = appearance.style.clone();
    ctx.data_mut(|d| {
        d.insert_temp(
            egui::Id::unique("viu-design-colours"),
            std::sync::Arc::new(appearance.colours),
        )
    });
    ctx.set_theme(theme);
    ctx.set_style_of(egui::Theme::Light, style.clone());
    ctx.set_style_of(egui::Theme::Dark, style);
}
