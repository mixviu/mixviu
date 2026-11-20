// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;

fn patch(operations:Value)->crate::model_patch::Patch {
    crate::model_patch::Patch::decode(json!({"generation":1,"base":1,"revision":2,"operations":operations})).unwrap()
}

#[test]
fn incremental_model_matches_full_ports_and_rolls_back_invalid_controls() {
    let source="---\nscene: 1\nname: patch\ncitizen: test\n---\n```mix\nroot: {widget: \"column\", children: [\"a\",\"b\"]}\na: {widget: \"text\", text: \"= $model.rows[0].label\"}\nb: {widget: \"control\", definition: {id: \"b\", kind: \"field\", label: \"Field\", initial: \"ok\"}}\n```\n";
    let model=json!({"rows":[{"label":"before"}],"values":{"b":"ok"},"unrelated":"untouched"});
    let mut incremental=SceneView::with_model(source,model.clone()).unwrap();
    let update=patch(json!([{"op":"splice","path":["rows"],"index":0,"remove":1,"items":[{"label":"after"}]}]));
    incremental.apply_model_patch(&update).unwrap();
    let mut full=SceneView::with_model(source,model).unwrap();
    full.apply_model(incremental.model().clone()).unwrap();
    assert_eq!(incremental.tree,full.tree);assert_eq!(incremental.lists,full.lists);
    assert_eq!(incremental.model_bytes,crate::model_patch::model_bytes(incremental.model()));
    let revision=incremental.revision;
    let bad=patch(json!([{"op":"set","path":["rows",0,"label"],"value":"must roll back"},{"op":"set","path":["values","b"],"value":42}]));
    assert!(incremental.apply_model_patch(&bad).is_err());
    assert_eq!(incremental.tree,full.tree);assert_eq!(incremental.lists,full.lists);assert_eq!(incremental.revision,revision);
}

#[test]
fn frame_refusal_rolls_back_root_model_ports_and_revision() {
    let content="---\nscene: 1\nname: root\ncitizen: test\n---\n```mix\nroot: {widget: \"text\", text: \"= $model.label\"}\n```\n";
    let frame="---\nscene: 1\nname: frame\ncitizen: test\n---\n```mix\nroot: {widget: \"column\", children: [\"gold-content\",\"field\"]}\n\"gold-content\": {widget: \"spacer\"}\nfield: {widget: \"control\", definition: {id: \"field\", kind: \"field\", label: \"Field\", initial: \"ok\"}}\n```\n";
    let model=json!({"label":"before","values":{"field":"ok"}});
    let mut view=SceneView::with_frame(content,model.clone(),frame).unwrap();
    let nodes=view.tree.nodes.clone();let frame_nodes=view.frame.as_ref().unwrap().tree.nodes.clone();
    let bad=patch(json!([{"op":"set","path":["label"],"value":"after"},{"op":"set","path":["values","field"],"value":42}]));
    assert!(view.apply_model_patch(&bad).is_err());
    assert_eq!(view.model(),&model);assert_eq!(view.tree.nodes,nodes);assert_eq!(view.revision,0);
    let frame=view.frame.as_ref().unwrap();assert_eq!(frame.model(),&model);assert_eq!(frame.tree.nodes,frame_nodes);assert_eq!(frame.revision,0);
}

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
