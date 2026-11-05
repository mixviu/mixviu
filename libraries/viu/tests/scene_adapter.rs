// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use viu::{
    view::SceneView,
    widgets::{Widget, Widgets},
};

fn scene(nodes: serde_json::Value) -> String {
    format!("---\nscene: 1\nname: test\ncitizen: test\n---\n```mix\n{nodes}\n```\n")
}

#[test]
fn renamed_disabled_icon_and_invalid_row_payloads_refuse() {
    let disabled = json!({"name":"copy","size":24.0,"colour":[80.0,180.0,230.0],"enabled":false,"selected":false});
    let w = Widget::new("another-icon", "icon", "", disabled.clone());
    assert!(w.validate(&disabled).is_ok());
    assert!(w.admit(&json!({}), "activate", &json!("copy")).is_err());
    let list = Widget::new("files", "list", "", json!(0)).with_properties(json!({"row_count":2}));
    for value in [json!(2), json!(-1), json!(1.5), json!("row"), json!(null)] {
        assert!(list.validate_action("activate", &value).is_err());
    }
    assert!(list.validate_action("activate", &json!(1.0)).is_ok());
}

#[test]
fn malformed_public_widget_is_visible_refusal_without_panic_or_geometry() {
    let ctx = egui::Context::default();
    let mut widgets = Widgets::default();
    let w = Widget::new("broken", "icon", "", json!({"name":"copy"}));
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        widgets.begin_frame();
        assert!(widgets.widget(ui, &w, &json!({})).is_empty());
        assert!(widgets.finish_frame().is_ok());
    });
    output.textures_delta.clear();
    assert!(!widgets.geometry.contains_key("broken"));
}

#[test]
fn frame_boundary_retires_disappearing_widget_geometry() {
    let ctx = egui::Context::default();
    let mut widgets = Widgets::default();
    let w =
        Widget::new("item", "button", "", json!("ready")).with_properties(json!({"text":"Item"}));
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        widgets.begin_frame();
        widgets.widget(ui, &w, &json!({}));
        widgets.finish_frame().unwrap();
    });
    output.textures_delta.clear();
    assert!(widgets.geometry.contains_key("item"));
    widgets.begin_frame();
    widgets.finish_frame().unwrap();
    assert!(widgets.geometry.is_empty());
}

#[test]
fn scene_control_binding_rejects_bad_model_transactionally() {
    let source = scene(
        json!({"root":{"widget":"control","definition":"= $model.control","on_intent":"act"}}),
    );
    let widget = Widget::new(
        "root",
        "icon",
        "",
        json!({"name":"copy","size":24,"colour":[80,180,230],"enabled":true,"selected":false}),
    );
    let good = json!({"control":widget.encode()});
    let mut view = SceneView::with_model(&source, good.clone()).unwrap();
    assert!(view.apply_model(json!([])).is_err());
    let bad = json!({"control":{"id":"root","kind":"icon","label":"","initial":{"name":"copy"}}});
    assert!(view.apply_model(bad).is_err());
    assert_eq!(view.revision, 0);
    assert_eq!(view.model(), &good);
    let mut wrong = good.clone();
    wrong["control"]["id"] = json!("different");
    assert!(view.apply_model(wrong).is_err());
}

#[test]
fn unsupported_scene_content_refuses_at_load() {
    let source = scene(json!({"root":{"widget":"image","src":"file:///tmp/private.png"}}));
    assert!(
        SceneView::new(&source)
            .err()
            .unwrap()
            .contains("only known icon:")
    );
}
