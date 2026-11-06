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
fn native_document_preserves_unicode_selection_and_undo_across_model_echo() {
    let source = format!(
        "---\nscene: 1\nname: input\ncitizen: input\n---\n```mix\n{}\n```\n",
        json!({"root":{"widget":"column","children":["editor"]},"editor":{"widget":"control","on_intent":"input","definition":{"id":"editor","kind":"editor","label":"Buffer","initial":"café.txt","properties":{"native_document":true,"document":"one","language":"rust"}}}})
    );
    let mut view = SceneView::with_model(&source, json!({"values":{"editor":"café.txt"}})).unwrap();
    let ctx = Context::default();
    viu::appearance::install(&ctx);
    draw(&ctx, &mut view, vec![]);
    view.request_focus_value(&json!({"id":"editor","selection":[0,4]}))
        .unwrap();
    draw(&ctx, &mut view, vec![]);
    let events = draw(&ctx, &mut view, vec![Event::Text("λ".into())]);
    assert!(
        events.iter().any(|e| e.body["value"] == "λ.txt"),
        "{events:?}"
    );
    view.apply_model(json!({"values":{"editor":"λ.txt"}}))
        .unwrap();
    draw(&ctx, &mut view, vec![]);
    let modifiers = Modifiers {
        ctrl: true,
        command: true,
        ..Modifiers::NONE
    };
    let mut undo = key(Key::Z);
    for event in &mut undo {
        if let Event::Key { modifiers: m, .. } = event {
            *m = modifiers;
        }
    }
    undo.insert(0, Event::ModifiersChanged(modifiers));
    let events = draw(&ctx, &mut view, undo);
    assert!(
        events.iter().any(|e| e.body["value"] == "café.txt"),
        "{events:?}"
    );
}

#[test]
fn native_document_admits_large_files_without_relaxing_legacy_controls() {
    let text = "λ\n".repeat(100_000);
    let widget = viu::widgets::Widget::new("editor", "editor", "", json!(""));
    assert!(widget.validate(&json!(text)).is_err());
    assert!(
        widget
            .with_properties(json!({"native_document":true}))
            .validate(&json!(text))
            .is_ok()
    );
}

#[test]
fn native_document_commands_are_local_and_a_serial_is_consumed_once() {
    let ctx = Context::default();
    viu::appearance::install(&ctx);
    let mut widgets = viu::widgets::Widgets::default();
    for (serial, name, expected) in [(1, "duplicate-line", Some("alpha\nalpha")), (2, "undo", Some("alpha")), (2, "undo", None)] {
        let widget = viu::widgets::Widget::new("editor", "editor", "", json!("alpha"))
            .with_properties(json!({"native_document":true,"command":{"serial":serial,"name":name}}));
        let mut events = Vec::new();
        ctx.run_ui(RawInput::default(), |ui| {
            widgets.begin_frame();
            events = widgets.widget(ui, &widget, &json!({}));
            widgets.finish_frame().unwrap();
        }).drop_without_applying_deltas();
        match expected {
            Some(text) => assert!(events.iter().any(|e| e.value == text), "{events:?}"),
            None => assert!(events.is_empty(), "{events:?}"),
        }
    }
}
#[test]
fn read_only_editor_keeps_full_text_scrollable_and_rejects_native_edits() {
    let body = "An immutable Unicode line λ\n".repeat(1000);
    let definition = json!({"id":"body","kind":"editor","label":"","initial":"","properties":{"rows":4,"read_only":true}});
    let source = format!(
        "---\nscene: 1\nname: reader\ncitizen: reader\n---\n```mix\n{}\n```\n",
        json!({"root":{"widget":"column","children":["body"]},"body":{"widget":"control","on_intent":"input","definition":definition}})
    );
    let model = json!({"values":{"body":body}});
    let mut view = SceneView::with_model(&source, model.clone()).unwrap();
    let ctx = Context::default();
    draw(&ctx, &mut view, vec![]);
    assert!(view.geometry["body"].height() < 120.0);
    assert!(
        view.admit_intent("body", "set", &json!("replacement"))
            .is_err()
    );
    view.request_focus("body").unwrap();
    draw(&ctx, &mut view, vec![]);
    assert!(
        draw(
            &ctx,
            &mut view,
            vec![
                Event::Text("replacement".into()),
                Event::Paste("paste".into())
            ]
        )
        .is_empty()
    );
    assert_eq!(view.model(), &model);
    let invalid = viu::widgets::Widget::new("bad", "editor", "", json!(""))
        .with_properties(json!({"read_only":"yes"}));
    assert!(invalid.validate(&json!("")).is_err());
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
    assert!(view.text_input_active());
    draw(&ctx, &mut view, vec![]);
    assert!(!view.text_input_active());
    assert!(view.admit_intent("name", "cancel", &Value::Null).is_ok());
    view.request_focus("name").unwrap();
    draw(&ctx, &mut view, vec![]);
    draw(&ctx, &mut view, vec![]);
    let outside = egui::pos2(500.0, 350.0);
    let blur = draw(
        &ctx,
        &mut view,
        vec![
            Event::PointerMoved(outside),
            Event::PointerButton {
                pos: outside,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: Modifiers::NONE,
            },
            Event::PointerButton {
                pos: outside,
                button: egui::PointerButton::Primary,
                pressed: false,
                modifiers: Modifiers::NONE,
            },
        ],
    );
    assert!(blur.iter().any(|event| event.body["action"] == "blur"));
    assert!(
        view.request_focus_value(&json!({"id":"missing","selection":[0,1]}))
            .is_err()
    );
}

#[test]
fn rejected_modal_submission_keeps_the_field_editable_without_an_extra_click() {
    let source = format!(
        "---\nscene: 1\nname: modal\ncitizen: modal\n---\n```mix\n{}\n```\n",
        json!({"root":{"widget":"control","on_intent":"input","definition":{"id":"root","kind":"dialog-form","label":"","initial":{"open":true,"title":"Name","body":"","field":"invalid/name","instance":1,"choices":[{"id":"ok","label":"OK","enabled":true,"default":true}]}}}})
    );
    let mut view = SceneView::new(&source).unwrap();
    let ctx = Context::default();
    viu::appearance::install(&ctx);
    draw(&ctx, &mut view, vec![]);
    draw(&ctx, &mut view, vec![]);
    let submitted = draw(&ctx, &mut view, key(Key::Enter));
    assert!(submitted.iter().any(|e| e.body["action"] == "answer"));
    // The application refuses the name and retains this instance/model.
    draw(&ctx, &mut view, vec![]);
    draw(&ctx, &mut view, vec![]);
    let mut select = key(Key::A);
    let modifiers = Modifiers {
        ctrl: true,
        command: true,
        ..Modifiers::NONE
    };
    for event in &mut select {
        if let Event::Key {
            modifiers: current, ..
        } = event
        {
            *current = modifiers;
        }
    }
    select.insert(0, Event::ModifiersChanged(modifiers));
    draw(&ctx, &mut view, select);
    let edited = draw(&ctx, &mut view, vec![Event::Text("repaired".into())]);
    assert!(
        edited
            .iter()
            .any(|e| e.body["action"] == "edit" && e.body["value"] == "repaired"),
        "{edited:?}"
    );
}
