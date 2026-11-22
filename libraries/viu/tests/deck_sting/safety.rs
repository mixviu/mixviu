use super::*;

#[test]
fn bad_and_nil_bound_primitives_paint_without_panicking() {
    let source="---\nscene: 1\nname: drawing-safety\ncitizen: test\n---\n```mix\n{root:{widget:\"drawing\",w:100,h:100,primitives:\"= $model.x\",shadow:\"= $model.shadow\",sample_time:\"= $model.time\"}}\n```\n";
    let mut view=SceneView::with_model(source,json!({"x":"bad","shadow":false,"time":1e308})).unwrap();
    let ctx=egui::Context::default();
    for model in [json!({"x":"bad","shadow":false,"time":1e308}),json!({"x":null,"shadow":null,"time":null}),json!({"x":[{"kind":"arc","centre":[0,0],"radius":[1e-40,1e-40],"colour":"#ffffff","sweep":1e-40}],"time":0})] {
        view.apply_model(model).unwrap();
        ctx.run_ui(egui::RawInput::default(),|ui| {view.show(ui);}).drop_without_applying_deltas();
    }
}
