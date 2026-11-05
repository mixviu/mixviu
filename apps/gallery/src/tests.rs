// SPDX-License-Identifier: MIT OR Apache-2.0
use super::*;
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
    let mut widgets = Widgets::default();
    let mut output = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(740.0, 1000.0),
            )),
            ..Default::default()
        },
        |ui| {
            widgets.show(ui, &controller::composition(), &controller::initial());
        },
    );
    assert!(!output.shapes.is_empty());
    output.textures_delta.clear();
    for widget in controller::composition() {
        assert!(
            widgets.geometry.contains_key(&widget.id),
            "{} geometry",
            widget.id
        );
    }
}
