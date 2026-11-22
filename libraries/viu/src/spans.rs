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
    family: Option<(&str, f32, Option<&str>)>,
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
        let fallback = if mono || run["mono"] == true {
            FontFamily::Monospace
        } else {
            FontFamily::Proportional
        };
        let requested = if let Some((name, weight, mono_name)) = family {
            FontId::new(
                size,
                if mono || run["mono"] == true {
                    mono_name.map_or(FontFamily::Monospace, |name| {
                        toolkit::fonts::optional::family(name, weight)
                    })
                } else {
                    toolkit::fonts::optional::family(name, weight)
                },
            )
        } else if mono || run["mono"] == true {
            FontId::new(size, FontFamily::Monospace)
        } else if bold {
            toolkit::fonts::heading(size)
        } else {
            FontId::new(size, FontFamily::Proportional)
        };
        job.append(
            run["text"].as_str().unwrap_or(""),
            0.,
            TextFormat {
                font_id: toolkit::fonts::bound_or(ui.ctx(), requested, fallback),
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn runtime_bound_families_fall_back_at_paint_without_optional_bundle() {
        let ctx = egui::Context::default();
        let absent = tempfile::tempdir().unwrap();
        crate::fonts::install_with_font_directory(&ctx, 400, 600, absent.path());
        let source = "---\nscene: 1\nname: fonts\ncitizen: test\n---\n```mix\n{root:{widget:\"text\",text:\"sans\",font:\"= $model.font\",mono_font:\"= $model.mono_font\"}}\n```\n";
        for (sans, mono) in [
            ("unknown", "unknown-mono"),
            ("Liberation Sans", "Noto Sans Mono"),
        ] {
            let mut document = scene::parse(source).unwrap();
            document.model = Some(json!({"font":sans,"mono_font":mono}));
            let resolved = scene::resolve(&document).unwrap();
            let ports = &resolved.nodes["root"].ports;
            let out = ctx.run_ui(egui::RawInput::default(), |ui| {
                for node_mono in [false, true] {
                    let job = job(
                        ui,
                        &json!([{"text":"sans"},{"text":"mono","mono":true}]),
                        24.,
                        node_mono,
                        false,
                        Color32::WHITE,
                        Some((
                            ports["font"].as_str().unwrap(),
                            400.,
                            ports["mono_font"].as_str(),
                        )),
                    );
                    assert_eq!(
                        job.sections[0].format.font_id.family,
                        if node_mono {
                            FontFamily::Monospace
                        } else {
                            FontFamily::Proportional
                        }
                    );
                    assert_eq!(
                        job.sections.last().unwrap().format.font_id.family,
                        FontFamily::Monospace
                    );
                    ui.label(job);
                }
            });
            assert!(out.shapes.iter().any(|shape| matches!(&shape.shape, egui::Shape::Text(t) if t.galley.job.text == "sansmono")));
            out.drop_without_applying_deltas();
        }
    }
}
