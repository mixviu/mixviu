// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::Value;
use std::collections::BTreeMap;

pub fn errors(body: &Value) -> BTreeMap<String, String> {
    body["scenes"].as_array().into_iter().flatten().filter_map(|row| {
        let name = row["name"].as_str()?;
        let diagnostic = row.get("diagnostic").filter(|d| !d.is_null())?;
        let first = diagnostic.pointer("/context/upstream/diagnostics/0").unwrap_or(diagnostic);
        let message = first["message"].as_str().unwrap_or("Revision refused");
        Some((name.into(), format!("⚠ {}: {} — open Scene Editor", first["code"].as_str().or_else(|| first["error_code"].as_str()).unwrap_or("scene"), message)))
    }).collect()
}

/// Painted after content, with a separate hit target. Last-good layout stays intact.
pub fn error_strip(ui: &mut egui::Ui, message: &str) -> bool {
    let rect = egui::Rect::from_min_size(ui.max_rect().min, egui::vec2(ui.max_rect().width(), 22.0));
    ui.painter().rect_filled(rect, 0.0, egui::Color32::from_rgb(100, 30, 20));
    let text: String = message.chars().take(240).collect();
    ui.painter().text(rect.left_center() + egui::vec2(4.0, 0.0), egui::Align2::LEFT_CENTER, text, egui::FontId::proportional(12.0), egui::Color32::WHITE);
    ui.interact(rect, ui.id().with("revision-error"), egui::Sense::click()).on_hover_text(message).clicked()
}
