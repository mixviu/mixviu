// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;
use std::collections::BTreeSet;

pub(super) fn spec_list(marker: &str) -> Vec<String> {
    let doc = include_str!("../../../../docs/ui-drive-contract.md");
    let list = doc.split(marker).nth(1).unwrap().split("```json\n").nth(1).unwrap();
    serde_json::from_str(list.split("\n```").next().unwrap()).unwrap()
}

#[test]
fn fixture_window_pins_tree_schema_and_roles() {
    let ctx = Context::default();
    install(&ctx, "fixture");
    let mut text = String::from("hello");
    let mut checked = true;
    let mut number = 0.5;
    for _ in 0..3 {
        ctx.run_ui(RawInput {
            screen_rect: Some(egui::Rect::from_min_size(Pos2::ZERO, vec2(640.0, 480.0))),
            ..Default::default()
        }, |ui| {
            ui.label("Greeting");
            let _ = ui.button("Go");
            ui.checkbox(&mut checked, "Check");
            ui.radio(true, "Radio");
            ui.text_edit_singleline(&mut text);
            ui.add(egui::Slider::new(&mut number, 0.0..=1.0));
        }).drop_without_applying_deltas();
    }
    let answer = request(&ctx, 1, "ui.tree", &json!({})).unwrap();
    assert_eq!(answer.rc, 0);
    let fields = |v: &Value| v.as_object().unwrap().keys().map(String::as_str).collect::<BTreeSet<_>>();
    assert_eq!(fields(&answer.body), BTreeSet::from(["root", "focus", "nodes"]));
    let nodes = answer.body["nodes"].as_array().unwrap();
    let mut roles = BTreeSet::new();
    for node in nodes {
        assert_eq!(fields(node), BTreeSet::from([
            "id", "role", "label", "value", "placeholder", "rect", "enabled",
            "focused", "selected", "checked", "children", "depth",
        ]));
        assert!(node["id"].as_str().unwrap().parse::<u64>().is_ok());
        assert!(node["enabled"].is_boolean() && node["focused"].is_boolean());
        assert!(node["selected"].is_null() || node["selected"].is_boolean());
        for field in ["label", "value", "placeholder"] {
            assert!(node[field].is_null() || node[field].is_string());
        }
        assert!(node["checked"].is_null() || ["true", "false", "mixed"].contains(&node["checked"].as_str().unwrap()));
        assert!(node["rect"].is_null() || (node["rect"].as_array().unwrap().len() == 4 && node["rect"].as_array().unwrap().iter().all(Value::is_number)));
        assert!(node["children"].as_array().unwrap().iter().all(Value::is_string));
        assert!(node["depth"].is_u64());
        roles.insert(node["role"].as_str().unwrap());
    }
    assert_eq!(roles, BTreeSet::from([
        "Window", "Label", "Button", "CheckBox", "RadioButton", "TextInput", "Slider",
    ]));
}
