// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use viu::view::SceneView;

#[test]
fn over_budget_binding_keeps_whole_previous_model_and_revision() {
    let mut view = SceneView::new(include_str!("../../../apps/volume-lab/volume/scene.mix")).unwrap();
    let before = view.model().clone();
    let mut rejected = before.clone();
    rejected["out"]["bar"] = json!(
        (0..501)
            .map(|i| json!({"id":format!("v{i}"),"cells":[],"pct":i,"w":15,"background":"#ffffff"}))
            .collect::<Vec<_>>()
    );
    assert!(view.apply_model(rejected).is_err());
    assert_eq!(view.model(), &before);
    assert_eq!(view.revision, 0);
}
