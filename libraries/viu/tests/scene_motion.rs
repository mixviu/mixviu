//! Scene motion through the application headless painter, no window or GPU.
use serde_json::json;
use viu::view::SceneView;
fn source(animate:bool)->String {
    let mut nodes=json!({"root":{"widget":"column","children":(0..100).map(|i|format!("n{i}")).collect::<Vec<_>>()}});
    for i in 0..100 {
        let mut node=json!({"widget":"text","text":format!("Motion {i}"),"size":13});
        if animate {node["enter"]=json!("up");}
        nodes[format!("n{i}")]=node;
    }
    format!("---\nscene: 1\nname: motion\ncitizen: test\n---\n```mix\n{nodes}\n```\n")
}
fn frame(ctx:&egui::Context,view:&mut SceneView,t:f64)->egui::FullOutput {
    ctx.run_ui(egui::RawInput {time:Some(t),screen_rect:Some(egui::Rect::from_min_size(egui::Pos2::ZERO,egui::vec2(1000.,4000.))),..Default::default()},|ui|{view.show(ui);})
}
#[test]
fn application_painter_samples_up_at_presentation_time() {
    let ctx=egui::Context::default();let mut view=SceneView::new(&source(true)).unwrap();
    frame(&ctx,&mut view,0.).drop_without_applying_deltas();
    let mut moving=frame(&ctx,&mut view,0.35);
    let at=moving.shapes.iter().find_map(|s|if let egui::Shape::Text(t)=&s.shape {if t.galley.job.text=="Motion 0" {Some(t.pos.y)} else {None}}else{None}).unwrap();
    moving.textures_delta.clear();
    let mut settled=frame(&ctx,&mut view,1.);
    let end=settled.shapes.iter().find_map(|s|if let egui::Shape::Text(t)=&s.shape {if t.galley.job.text=="Motion 0" {Some(t.pos.y)} else {None}}else{None}).unwrap();
    settled.textures_delta.clear();
    let preset=motion::presets().into_iter().find(|p|p.name=="up").unwrap();
    let expected=preset.sample(0.35).get(motion::Property::Ty) as f32;
    assert!((at-end-expected).abs()<0.001,"offset: {} vs {expected}",at-end);
}
#[test]
#[ignore="release measurement; run explicitly with --release --ignored --nocapture"]
fn hundred_node_cost() {
    for animate in [false,true] {
        let ctx=egui::Context::default();let mut view=SceneView::new(&source(animate)).unwrap();
        frame(&ctx,&mut view,0.).drop_without_applying_deltas();
        let mut us=Vec::new();
        for i in 1..1001 {
            let start=std::time::Instant::now();
            let mut out=frame(&ctx,&mut view,i as f64/2000.);
            let _=ctx.tessellate(std::mem::take(&mut out.shapes),out.pixels_per_point);
            out.textures_delta.clear();
            us.push(start.elapsed().as_secs_f64()*1e6);
        }
        us.sort_by(f64::total_cmp);
        println!("100 nodes animated={animate}: median_us={:.3} p95_us={:.3} frames=1000 (headless layout, paint, tessellation; no GPU)",us[500],us[950]);
    }
}
