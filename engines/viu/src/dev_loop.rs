// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::Value;
use std::collections::BTreeMap;

pub fn errors(body: &Value) -> BTreeMap<String, String> {
    body["scenes"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|row| {
            let name = row["name"].as_str()?;
            let diagnostic = row.get("load_diagnostic").filter(|d| !d.is_null())
                .or_else(|| row.get("diagnostic").filter(|d| !d.is_null()))?;
            let first = diagnostic.pointer("/context/upstream/diagnostics/0")
                .or_else(|| diagnostic.pointer("/context/problems/0"))
                .unwrap_or(diagnostic);
            let message = first["message"].as_str().unwrap_or("Revision refused");
            Some((
                name.into(),
                format!(
                    "⚠ {}: {}",
                    first["code"]
                        .as_str()
                        .or_else(|| first["error_code"].as_str())
                        .unwrap_or("scene"),
                    compact_text(message)
                ),
            ))
        })
        .collect()
}

pub fn scenes_service() -> String {
    std::env::var("SCENES_SERVICE").unwrap_or_else(|_| "scenes".into())
}

fn compact_text(message: &str) -> String {
    message.split_whitespace().map(|word| {
        if let Some(start) = word.find('/') {
            format!("{}…/{}", &word[..start], word.rsplit('/').next().unwrap_or_default())
        } else { word.into() }
    }).collect::<Vec<_>>().join(" ")
}

/// Painted after content, with a separate hit target. Last-good layout stays intact.
pub fn error_strip(ui: &mut egui::Ui, message: &str) -> bool {
    let rect =
        egui::Rect::from_min_size(ui.max_rect().min, egui::vec2(ui.max_rect().width(), 22.0));
    let message = compact_text(message);
    let text: String = message.chars().take(240).collect();
    egui::Area::new(ui.scope_id().with("revision-error"))
        .order(egui::Order::Foreground)
        .fixed_pos(rect.min)
        .show(ui.ctx(), |ui| {
            ui.add_sized(
                rect.size(),
                egui::Button::new(
                    egui::RichText::new(text)
                        .size(12.0)
                        .color(egui::Color32::WHITE),
                )
                .fill(egui::Color32::from_rgb(100, 30, 20)),
            )
            .on_hover_text(&message)
            .clicked()
        })
        .inner
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn refusal_uses_first_problem_and_a_successful_snapshot_clears_it() {
        let errors = errors(
            &json!({"scenes":[{"name":"panel","diagnostic":{"message":"host refused"},"problems":[{"code":"mix-parse","message":"expected map"}]}]}),
        );
        assert!(errors["panel"].contains("host refused"));
        assert!(super::errors(&json!({"scenes":[{"name":"panel","diagnostic":null}]})).is_empty());
    }

    #[test]
    fn refused_lib_publication_draws_strip_over_last_good_surface() {
        let temp = tempfile::tempdir().unwrap();
        let notice = temp.path().join("notice.json");
        let output = std::process::Command::new("/opt/mixos/bin/mix")
            .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/../viud/tests/scenes_test.mix"))
            .env("P1_SURFACE_NOTICE", &notice).output().unwrap();
        assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let body: Value = serde_json::from_slice(&std::fs::read(notice).unwrap()).unwrap();
        let mut body = body;
        let row = body["scenes"].as_array_mut().unwrap().iter_mut().find(|r| r["name"]=="alpha").unwrap();
        assert_eq!(row["mounted"], true);
        row["problems"] = json!([{"message":"unrelated lint warning"}]);
        let errors = errors(&body);
        assert!(errors["alpha"].contains("broken"));
        assert!(!errors["alpha"].contains("unrelated lint"));
        let ctx = egui::Context::default();
        let mut store = crate::store::SceneStore::default();
        let source = "---\nscene: 1\nname: alpha\ncitizen: alpha\n---\n```mix\nroot: {widget: text, text: 'last good'}\n```\n";
        store.request(crate::SceneCommand::Load, source, &Value::Null).unwrap();
        let entry = store.scene("alpha").unwrap();
        let revision = entry.revision();
        let mut view = crate::view::SceneUi::new(std::sync::Arc::new(crate::view::Content {
            tree:entry.tree().clone(), lists:entry.prepared().clone(), revision, frame:None, dialog:false,
        }));
        view.setup(&ctx);
        for pass in 0..3 {
            let output = ctx.run_ui(egui::RawInput {screen_rect:Some(egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(512.0,128.0))), ..Default::default()}, |ui| {
                view.show(ui, &mut Vec::new());
                error_strip(ui, &errors["alpha"]);
            });
            assert!(!output.shapes.is_empty());
            if pass == 2 { assert!(format!("{:?}", output.shapes).contains("broken")); }
        }
        assert_eq!(view.revision(), revision);
        assert_eq!(compact_text("refused /home/private/projects/scene.mix:7"), "refused …/scene.mix:7");
        assert!(!errors["alpha"].contains("open Scene Editor"));
    }
}
