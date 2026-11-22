// SPDX-License-Identifier: MIT OR Apache-2.0
//! Inline text uses the same layout engine as plain text. No timers or state.
use egui::{Color32, FontFamily, FontId, Stroke, TextFormat, text::LayoutJob};
use serde_json::Value;

pub(crate) fn job(
    ui: &egui::Ui,
    spans: &Value,
    size: f32,
    mono: bool,
    bold: bool,
    base: Color32,
) -> LayoutJob {
    let mut job = LayoutJob::default();
    for run in spans.as_array().into_iter().flatten() {
        let role = match run["role"].as_str().unwrap_or("text") {
            "keyword" | "bad" => "danger",
            "function" | "acc" => "accent",
            "string" | "ok" => "status.success",
            "value" | "warn" => "warning",
            "comment" | "mut" => "text_faint",
            _ => "text",
        };
        let color = run["color"]
            .as_str()
            .and_then(|s| {
                let rgb = u32::from_str_radix(s.strip_prefix('#')?, 16).ok()?;
                Some(Color32::from_rgb(
                    (rgb >> 16) as u8,
                    (rgb >> 8) as u8,
                    rgb as u8,
                ))
            })
            .or_else(|| crate::appearance::colour(ui.ctx(), role))
            .unwrap_or(base);
        job.append(
            run["text"].as_str().unwrap_or(""),
            0.,
            TextFormat {
                font_id: if mono || run["mono"] == true {
                    FontId::new(size, FontFamily::Monospace)
                } else if run["bold"].as_bool().unwrap_or(bold) {
                    toolkit::fonts::bound(ui.ctx(), toolkit::fonts::heading(size))
                } else {
                    FontId::new(size, FontFamily::Proportional)
                },
                color,
                background: if run["highlight"] == true {
                    color.gamma_multiply(0.14)
                } else {
                    Color32::TRANSPARENT
                },
                strikethrough: if run["strike"] == true {
                    Stroke::new(1., color)
                } else {
                    Stroke::NONE
                },
                ..Default::default()
            },
        );
    }
    job
}
