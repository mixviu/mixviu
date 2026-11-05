// SPDX-License-Identifier: MIT OR Apache-2.0
use egui::{Context, Event, Key, Modifiers, RawInput};
use serde_json::{Value, json};
use viu::view::SceneView;
fn draw(ctx: &Context, view: &mut SceneView, events: Vec<Event>) -> Vec<viu::view::Event> {
    let mut found = Vec::new();
    let mut output = ctx.run_ui(
        RawInput {
            events,
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(600.0, 400.0),
            )),
            ..Default::default()
        },
        |ui| {
            found = view.show(ui);
        },
    );
    output.textures_delta.clear();
    drop(output);
    found
}
fn key(key: Key) -> Vec<Event> {
    [true, false]
        .into_iter()
        .map(|pressed| Event::Key {
            key,
            physical_key: None,
            pressed,
            repeat: false,
            modifiers: Modifiers::NONE,
        })
        .collect()
}
#[test]
fn native_text_selection_edits_unicode_stems_and_escape_releases_focus() {
    let source = format!(
        "---\nscene: 1\nname: input\ncitizen: input\n---\n```mix\n{}\n```\n",
        json!({"root":{"widget":"column","children":["name"]},"name":{"widget":"control","on_intent":"input","definition":{"id":"name","kind":"field","label":"","initial":"café.txt","properties":{"cancel_on_escape":true,"notify_blur":true}}}})
    );
    let mut view = SceneView::with_model(&source, json!({"values":{"name":"café.txt"}})).unwrap();
    let ctx = Context::default();
    viu::appearance::install(&ctx);
    draw(&ctx, &mut view, vec![]);
    for selection in [
        json!([5, 2]),
        json!([-1, 3]),
        json!([0, 1.5]),
        json!([0, 1048577]),
    ] {
        assert!(
            view.request_focus_value(&json!({"id":"name","selection":selection}))
                .is_err()
        );
    }
    view.request_focus_value(&json!({"id":"name","selection":[0,4]}))
        .unwrap();
    draw(&ctx, &mut view, vec![]);
    let edits = draw(&ctx, &mut view, vec![Event::Text("new".into())]);
    let value = edits
        .iter()
        .find(|e| e.body["action"] == "set")
        .unwrap()
        .body["value"]
        .clone();
    assert_eq!(value, "new.txt");
    view.apply_model(json!({"values":{"name":value}})).unwrap();
    let escaped = draw(&ctx, &mut view, key(Key::Escape));
    assert!(escaped.iter().any(|e| e.body["action"] == "cancel"));
    draw(&ctx, &mut view, vec![]);
    assert!(!view.text_input_active());
    assert!(view.admit_intent("name", "cancel", &Value::Null).is_ok());
    view.request_focus("name").unwrap();draw(&ctx,&mut view,vec![]);draw(&ctx,&mut view,vec![]);
    let outside=egui::pos2(500.0,350.0);
    let blur=draw(&ctx,&mut view,vec![Event::PointerMoved(outside),Event::PointerButton{pos:outside,button:egui::PointerButton::Primary,pressed:true,modifiers:Modifiers::NONE},Event::PointerButton{pos:outside,button:egui::PointerButton::Primary,pressed:false,modifiers:Modifiers::NONE}]);
    assert!(blur.iter().any(|event|event.body["action"]=="blur"));
    assert!(
        view.request_focus_value(&json!({"id":"missing","selection":[0,1]}))
            .is_err()
    );
}
