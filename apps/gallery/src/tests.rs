// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;
use viu::widgets::Widgets;
use std::time::Duration;
fn completion(runtime: &Runtime) -> viu::runtime::Completion {
    let start = std::time::Instant::now();
    loop {
        if let Some(done) = runtime.try_completion() {
            assert!(done.error.is_none(), "{:?}", done.error);
            return done;
        }
        assert!(start.elapsed().as_secs() < 10);
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
}
fn runtime() -> Runtime {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("gallery.mix");
    let mut runtime = Runtime::new(32, Arc::new(|| {}), None);
    runtime
        .load(Program {
            source: std::fs::read_to_string(&path).unwrap(),
            path,
            scene: "gallery".into(),
            citizen: "gallery-mix".into(),
        })
        .unwrap();
    runtime
}
#[test]
fn independent_mix_composition_and_every_action_match_rust() {
    let mut runtime = runtime();
    let mut expected = controller::initial();
    assert_eq!(completion(&runtime).model.unwrap(), expected);
    let widgets = controller::composition();
    assert_eq!(widgets.len(), viu::widgets::KINDS.len());
    for widget in &widgets {
        for action in controller::actions(widget) {
            let value = match widget.kind.as_str() {
                "icon" if action == "activate" => json!("copy"),
                "icon" => {
                    json!({"name":"star","size":32,"colour":[220,180,40],"enabled":true,"selected":true})
                }
                "toolbar" => json!("save"),
                "checkbox" | "toggle" | "collapse" | "dialog" => json!(true),
                "slider" | "number" | "progress" | "split" => json!(82),
                "radio" | "combo" | "tabs" | "menu" => json!("Gamma"),
                "list" | "table" | "tree" => json!(17),
                "colour" => json!([200, 80, 20]),
                _ => json!("Changed"),
            };
            let value = if action == "expand" || action == "sort" {
                json!(true)
            } else {
                value
            };
            let intent = json!({"id":widget.id,"action":action,"value":value});
            controller::validate(&widgets, &intent).unwrap();
            controller::apply(&mut expected, &intent);
            runtime
                .event("gallery.execute".into(), intent.clone())
                .unwrap();
            assert_eq!(
                completion(&runtime).model.unwrap(),
                expected,
                "intent {intent}"
            );
        }
    }
    let reset = json!({"action":"reset"});
    controller::apply(&mut expected, &reset);
    runtime.event("gallery.execute".into(), reset).unwrap();
    assert_eq!(completion(&runtime).model.unwrap(), expected);
}
#[test]
fn invalid_actions_and_values_are_refused() {
    let widgets = controller::composition();
    for body in [
        json!({"id":"missing","action":"set","value":0}),
        json!({"id":"slider","action":"set","value":101}),
        json!({"id":"checkbox","action":"set","value":"true"}),
        json!({"id":"label","action":"accept"}),
        json!({"id":"colour","action":"set","value":[0,-1,256]}),
    ] {
        assert!(controller::validate(&widgets, &body).is_err(), "{body}");
    }
}
#[test]
fn automated_double_click_activates_native_row_across_frames() {
    let ctx = egui::Context::default();
    let mut widgets = Widgets::default();
    let definition = Widget::new("items", "list", "Items", json!(0))
        .with_properties(json!({"dataset":"items","row_count":2,"context_action":"Select"}));
    let model = json!({"values":{"items":0},"datasets":{"items":[["One"],["Two"]]}});
    let mut draw = |events: Vec<egui::Event>, time: f64| {
        let mut intents = Vec::new();
        let mut output = ctx.run_ui(
            egui::RawInput {
                time: Some(time),
                events,
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 300.0),
                )),
                ..Default::default()
            },
            |ui| {
                intents = widgets.widget(ui, &definition, &model);
            },
        );
        output.textures_delta.clear();
        (intents, widgets.geometry.get("items.1").copied())
    };
    let (_, rect) = draw(Vec::new(), 0.0);
    let pos = rect.unwrap().center();
    let mut activations = Vec::new();
    for (i, events) in click_frames(
        pos,
        egui::PointerButton::Primary,
        egui::Modifiers::NONE,
        true,
    )
    .into_iter()
    .enumerate()
    {
        let (intents, _) = draw(events, 0.04 + i as f64 * 0.02);
        activations.extend(intents.into_iter().filter(|e| e.action == "activate"));
    }
    assert_eq!(activations.len(), 1);
    assert_eq!(activations[0].value, json!(1));
}
#[test]
fn shared_widgets_accept_non_gallery_content() {
    let choice = Widget::new("destination", "combo", "Destination", json!("Archive"))
        .with_properties(json!({"choices":["Inbox","Archive"]}));
    assert!(choice.validate(&json!("Inbox")).is_ok());
    assert!(choice.validate(&json!("Gamma")).is_err());
    assert_eq!(
        Widget::decode(&choice.encode()).unwrap().encode(),
        choice.encode()
    );
    let list = Widget::new("contacts", "list", "Contacts", json!(0)).with_properties(
        json!({"dataset":"people","row_count":2,"context_action":"Select contact"}),
    );
    let model = json!({"values":{"destination":"Archive","contacts":0},"datasets":{"people":[["Ada"],["Grace"]]}});
    let ctx = egui::Context::default();
    viu::icons::install(&ctx);
    let mut widgets = Widgets::default();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        widgets.widget(ui, &choice, &model);
        widgets.widget(ui, &list, &model);
    });
    output.textures_delta.clear();
    assert!(widgets.geometry.contains_key("contacts.0"));
    assert!(!widgets.geometry.contains_key("contacts.2"));
}
#[test]
fn every_widget_renders_in_shared_native_path() {
    let ctx = egui::Context::default();
    viu::icons::install(&ctx);
    let mut view=SceneView::with_model(include_str!("../scene.mix"),controller::initial()).unwrap();
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(740.0, 1000.0),
            )),
            ..Default::default()
        },
        |ui| {
            view.show(ui);
        },
    );
    assert!(!output.shapes.is_empty());
    output.textures_delta.clear();
    for widget in controller::composition() {
        assert!(
            view.geometry.contains_key(&widget.id),
            "{} geometry",
            widget.id
        );
    }
}

#[test]
fn publications_refuse_unknown_icons_and_duplicate_ids() {
    assert!(!request_ready(false, "gallery.describe"));
    assert!(!request_ready(false, "gallery.execute"));
    assert!(request_ready(false, "gallery.state"));
    assert!(request_ready(false, "gallery.quit"));
    assert!(request_ready(true, "gallery.describe"));
    let mut model = controller::initial();
    assert!(controller::validate_model(&model).is_ok());
    model["values"]["icon"]["name"] = json!("unknown");
    assert!(controller::validate_model(&model).is_err());
    let mut model = controller::initial();
    model["widgets"][1]["id"] = json!("label");
    assert!(controller::validate_model(&model).is_err());
    assert!(controller::validate_model(&json!({})).is_err());
}

#[test]
fn activation_retains_row_payload_and_disabled_toolbar_is_refused() {
    let mut model = controller::initial();
    let event = json!({"id":"list","action":"activate","value":42});
    controller::apply(&mut model, &event);
    assert_eq!(model["last_activation"], json!({"id":"list","value":42}));
    assert!(
        controller::validate(
            &controller::composition(),
            &json!({"id":"toolbar","action":"activate","value":"trash"})
        )
        .is_err()
    );
}

#[test]
fn disappeared_focus_is_refused_once_without_latching_next_frame() {
    let mut widgets = Widgets::default();
    widgets.focus = Some("closed-popup.choice".into());
    assert!(widgets.finish_frame().is_err());
    assert!(widgets.focus.is_none());
    assert!(widgets.finish_frame().is_ok());
}

#[test]
fn reload_retires_inflight_request_and_replies_superseded() {
    let path=PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("gallery.mix");
    let mut runtime=Runtime::new(8,Arc::new(||{}),None);
    runtime.load(Program{source:"on gallery.execute async; sleep(0.1); reply(0,\"{}\"); end".into(),path:path.clone(),scene:"gallery".into(),citizen:"gallery-mix".into()}).unwrap();
    completion(&runtime);
    let model=controller::initial();
    let mut gallery=Gallery{view:Some(SceneView::with_model(include_str!("../scene.mix"),model.clone()).unwrap()),model,definitions:controller::composition(),runtime:Some(runtime),bus:NativeBus::offline(),pending:BTreeMap::new(),inflight:BTreeSet::new(),input:Vec::new(),later_input:Default::default(),input_replies:Vec::new(),witness:None,error:String::new(),mode:"mix".into()};
    let (reply,mut received)=tokio::sync::oneshot::channel();
    gallery.execute(json!({"id":"label","action":"set","value":"old"}),Some(reply));
    assert_eq!(gallery.inflight.len(),1);
    std::thread::sleep(Duration::from_millis(20));
    gallery.runtime.as_mut().unwrap().load(Program{source:include_str!("../gallery.mix").into(),path,scene:"gallery".into(),citizen:"gallery-mix".into()}).unwrap();
    let start=std::time::Instant::now();
    while !gallery.inflight.is_empty() && start.elapsed()<Duration::from_secs(5) { gallery.completions(); std::thread::sleep(Duration::from_millis(1)); }
    assert!(gallery.inflight.is_empty());
    assert!(gallery.pending.is_empty());
    let (rc,body)=received.try_recv().unwrap();
    assert_eq!(rc,10);
    assert_eq!(body["error_code"],"SUPERSEDED");
}
