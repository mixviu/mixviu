// SPDX-License-Identifier: MIT OR Apache-2.0
//! Disposable captured-oracle replay; not an acceptance test.
use serde_json::Value;
use viu::{bundle::Bundle,view::SceneView};

#[test]
fn captured_model_patch_replay() {
    let root=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../");
    let bundle=Bundle::load(&root.join("apps/workshop"), &["--headless".into()]).unwrap();
    let mut paths=std::fs::read_dir(root.join("target/workshop-model-captures")).unwrap().map(|e|e.unwrap().path()).filter(|p|p.extension().is_some_and(|e|e=="json")).collect::<Vec<_>>();
    paths.sort();
    assert!(!paths.is_empty());
    let mut view: Option<SceneView>=None;
    for path in paths {
        let model:Value=serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        let patch:Value=serde_json::from_slice(&std::fs::read(format!("{}.patch",path.display())).unwrap()).unwrap();
        if view.is_none() { view=Some(SceneView::with_model(&bundle.scene,model).unwrap());continue; }
        let view=view.as_mut().unwrap();
        let start=std::time::Instant::now();
        view.apply_patch_lab(&patch).unwrap();
        let apply_us=start.elapsed().as_micros();
        let start=std::time::Instant::now();
        let full=SceneView::with_model(&bundle.scene,model).unwrap();
        let full_us=start.elapsed().as_micros();
        assert!(view.lab_equal(&full),"{}",path.display());
        eprintln!("LAB_NATIVE {} apply_us={apply_us} full_resolve_us={full_us}",path.display());
    }
}
