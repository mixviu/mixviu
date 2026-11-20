// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;

#[test]
fn model_replacement_evaluates_only_affected_bindings() {
    let source = "---\nscene: 1\nname: bindings\ncitizen: test\n---\n```mix\nroot: {widget: \"column\", children: [\"a\",\"b\"]}\na: {widget: \"text\", text: \"= $model.a\"}\nb: {widget: \"text\", text: \"= $model.b ?? \\\"missing\\\"\"}\n```\n";
    let mut model = json!({"a":"one","b":"two","unrelated":"before"});
    let mut view = SceneView::with_model(source, model.clone()).unwrap();
    model["unrelated"] = json!("after");
    view.apply_model(model.clone()).unwrap();
    assert_eq!(
        view.evaluated_last_apply, 0,
        "unrelated replacement evaluated bindings"
    );
    assert_eq!(view.model(), &model);
    model["a"] = json!("changed");
    view.apply_model(model.clone()).unwrap();
    assert_eq!(view.evaluated_last_apply, 1);
    let oracle = SceneView::with_model(source, model.clone()).unwrap();
    assert_eq!(view.tree.nodes, oracle.tree.nodes);
    model.as_object_mut().unwrap().remove("b");
    view.apply_model(model.clone()).unwrap();
    assert_eq!(view.evaluated_last_apply, 1);
    let oracle = SceneView::with_model(source, model).unwrap();
    assert_eq!(view.tree.nodes, oracle.tree.nodes);
}

#[test]
fn dependency_filter_is_conservative_for_whole_models_and_nested_changes() {
    let before = json!({"a":{"b":1},"other":0});
    let after = json!({"a":{"b":2},"other":0});
    assert!(model_dependency_changed(
        &["model".into()].into(),
        &before,
        &after
    ));
    assert!(model_dependency_changed(
        &["model.a.b".into()].into(),
        &before,
        &after
    ));
    assert!(!model_dependency_changed(
        &["model.other".into()].into(),
        &before,
        &after
    ));
}

#[test]
fn no_binding_change_still_validates_controls_atomically() {
    let source = "---\nscene: 1\nname: control\ncitizen: test\n---\n```mix\nroot: {widget: \"control\", definition: {id: \"root\", kind: \"field\", label: \"Field\", initial: \"ok\"}}\n```\n";
    let model = json!({"values":{"root":"ok"}});
    let mut view = SceneView::with_model(source, model.clone()).unwrap();
    assert!(view.apply_model(json!({"values":{"root":42}})).is_err());
    assert_eq!(view.model(), &model);
}
