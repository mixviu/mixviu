// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;
fn completion(runtime:&Runtime)->viu::runtime::Completion {
    let start=std::time::Instant::now();
    loop {if let Some(done)=runtime.try_completion(){assert!(done.error.is_none(),"{:?}",done.error);return done;}assert!(start.elapsed().as_secs()<10);std::thread::sleep(std::time::Duration::from_millis(1));}
}
fn runtime()->Runtime {
    let path=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("gallery.mix");
    let mut runtime=Runtime::new(32,Arc::new(||{}),None);
    runtime.load(Program{source:std::fs::read_to_string(&path).unwrap(),path,scene:"gallery".into(),citizen:"gallery-mix".into()}).unwrap();runtime
}
#[test]
fn independent_mix_composition_and_every_action_match_rust() {
    let mut runtime=runtime();let mut expected=controller::initial();
    assert_eq!(completion(&runtime).model.unwrap(),expected);
    let widgets=controller::composition();
    assert_eq!(widgets.len(),viu::widgets::KINDS.len());
    for widget in &widgets {
        for action in controller::actions(widget) {
            let value=match widget.kind.as_str(){"checkbox"|"toggle"|"collapse"|"dialog"=>json!(true),"slider"|"number"|"progress"|"split"=>json!(82),"radio"|"combo"|"tabs"|"menu"=>json!("Gamma"),"list"|"table"|"tree"=>json!(17),"colour"=>json!([200,80,20]),_=>json!("Changed")};
            let intent=json!({"id":widget.id,"action":action,"value":value});
            controller::validate(&widgets,&intent).unwrap();
            controller::apply(&mut expected,&intent);
            runtime.event("gallery.execute".into(),intent.clone()).unwrap();
            assert_eq!(completion(&runtime).model.unwrap(),expected,"intent {intent}");
        }
    }
    let reset=json!({"action":"reset"});controller::apply(&mut expected,&reset);runtime.event("gallery.execute".into(),reset).unwrap();assert_eq!(completion(&runtime).model.unwrap(),expected);
}
#[test]
fn invalid_actions_and_values_are_refused() {
    let widgets=controller::composition();
    for body in [json!({"id":"missing","action":"set","value":0}),json!({"id":"slider","action":"set","value":101}),json!({"id":"checkbox","action":"set","value":"true"}),json!({"id":"label","action":"accept"}),json!({"id":"colour","action":"set","value":[0,-1,256]})]{assert!(controller::validate(&widgets,&body).is_err(),"{body}");}
}
#[test]
fn every_widget_renders_in_shared_native_path() {
    let ctx=egui::Context::default();let mut widgets=Widgets::default();
    let output=ctx.run(egui::RawInput{screen_rect:Some(egui::Rect::from_min_size(egui::Pos2::ZERO,egui::vec2(740.0,1000.0))),..Default::default()},|ctx|{egui::CentralPanel::default().show(ctx,|ui|{widgets.show(ui,&controller::composition(),&controller::initial());});});
    assert!(!output.shapes.is_empty());
    for widget in controller::composition(){assert!(widgets.geometry.contains_key(&widget.id),"{} geometry",widget.id);}
}
