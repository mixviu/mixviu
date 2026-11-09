// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::Value;
use std::collections::BTreeMap;

pub fn errors(body: &Value) -> BTreeMap<String, String> {
    body["scenes"].as_array().into_iter().flatten().filter_map(|row| {
        let name = row["name"].as_str()?;
        let diagnostic = row.get("diagnostic").filter(|d| !d.is_null())?;
        let first = row.pointer("/problems/0").or_else(|| diagnostic.pointer("/context/upstream/diagnostics/0")).unwrap_or(diagnostic);
        let message = first["message"].as_str().unwrap_or("Revision refused");
        Some((name.into(), format!("⚠ {}: {} — open Scene Editor", first["code"].as_str().or_else(|| first["error_code"].as_str()).unwrap_or("scene"), message)))
    }).collect()
}

/// Painted after content, with a separate hit target. Last-good layout stays intact.
pub fn error_strip(ui: &mut egui::Ui, message: &str) -> bool {
    let rect = egui::Rect::from_min_size(ui.max_rect().min, egui::vec2(ui.max_rect().width(), 22.0));
    let text: String = message.chars().take(240).collect();
    egui::Area::new(ui.scope_id().with("revision-error"))
        .order(egui::Order::Foreground).fixed_pos(rect.min).show(ui.ctx(), |ui| {
            ui.add_sized(rect.size(), egui::Button::new(egui::RichText::new(text).size(12.0).color(egui::Color32::WHITE))
                .fill(egui::Color32::from_rgb(100, 30, 20))).on_hover_text(message).clicked()
        }).inner
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn refusal_uses_first_problem_and_a_successful_snapshot_clears_it() {
        let errors = errors(&json!({"scenes":[{"name":"panel","diagnostic":{"message":"host refused"},"problems":[{"code":"mix-parse","message":"expected map"}]}]}));
        assert!(errors["panel"].contains("mix-parse: expected map"));
        assert!(super::errors(&json!({"scenes":[{"name":"panel","diagnostic":null}]})).is_empty());
    }
}
