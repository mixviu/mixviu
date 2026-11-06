// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::{Value, json};
use viu::{
    view::SceneView,
    widgets::{Widget, Widgets},
};

fn table(n: usize) -> Value {
    json!({"key":"container","revision":1,"active":true,"columns":[{"id":"name","label":"Name"},{"id":"size","label":"Size","align":"right"},{"id":"modified","label":"Modified","align":"right"}],"sort":"name","ascending":true,"rows":(0..n).map(|i|json!({"key":format!("item-{i}"),"cells":[format!("Item {i}"),format!("{i} KiB"),"2026-10-11 09:30"],"depth":0,"branch":i==0,"expanded":false,"selected":false,"focused":false,"icon":if i==0{"folder"}else{"file"}})).collect::<Vec<_>>(),"context":[{"id":"inspect","label":"Inspect","enabled":true},{"id":"disabled","label":"Disabled","enabled":false}],"background_context":[]})
}
fn frame(
    ctx: &egui::Context,
    widgets: &mut Widgets,
    w: &Widget,
    mut events: Vec<egui::Event>,
) -> Vec<viu::widgets::Intent> {
    let modifiers = events
        .iter()
        .find_map(|e| {
            if let egui::Event::PointerButton { modifiers, .. } = e {
                Some(*modifiers)
            } else {
                None
            }
        })
        .unwrap_or_default();
    events.insert(0, egui::Event::ModifiersChanged(modifiers));
    let input = egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(700.0, 400.0),
        )),
        events,
        ..Default::default()
    };
    let mut intents = Vec::new();
    let mut out = ctx.run_ui(input, |ui| {
        widgets.begin_frame();
        intents = widgets.widget(ui, w, &json!({}));
        widgets.finish_frame().unwrap();
    });
    out.textures_delta.clear();
    intents
}

#[test]
fn data_authored_menus_share_native_model_and_keyboard_activation() {
    let ctx = egui::Context::default();
    viu::appearance::install(&ctx);
    let widget = Widget::new("commands", "menubar", "", json!([
        {"id":"file","label":"File","enabled":true,"children":[
            {"id":"runtime.command-42","label":"Run owned command","enabled":true,"shortcut":"Ctrl+R","shortcut_binding":"F2"},
            {"separator":true},
            {"id":"disabled","label":"Refused command","enabled":false}
        ]}
    ])).with_properties(json!({"native_menu":true}));
    let mut widgets = Widgets::default();
    for _ in 0..2 {
        frame(&ctx, &mut widgets, &widget, vec![]);
    }
    let model = toolkit::menu::model(&ctx);
    assert_eq!(model[0].title, "File");
    assert_eq!(model[0].entries[0].row().unwrap().shortcut.as_deref(), Some("F2"));
    assert_eq!(
        model[0].entries[0].row().unwrap().id.as_deref(),
        Some("runtime.command-42")
    );
    assert!(!model[0].entries[2].row().unwrap().enabled);
    toolkit::menu::open_path(&ctx, &["File".into()]).unwrap();
    let key = |key| egui::Event::Key {
        key,
        physical_key: Some(key),
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    };
    frame(&ctx, &mut widgets, &widget, vec![key(egui::Key::ArrowDown)]);
    let intents = frame(&ctx, &mut widgets, &widget, vec![key(egui::Key::Enter)]);
    assert!(
        intents.iter().any(|i| i.id == "commands"
            && i.action == "command"
            && i.value["command"] == "runtime.command-42"),
        "{intents:?}"
    );
    assert!(!toolkit::menu::is_open(&ctx));
}

#[test]
fn browser_actions_are_admitted_through_the_public_widget_boundary() {
    let widget = Widget::new(
        "browser",
        "settings-browser",
        "",
        json!({
            "title":"Settings","pages":[{"id":"page","title":"Page"}],
            "sections":[{"rows":[{"id":"toggle","title":"Enabled","kind":"toggle","value":false}]}]
        }),
    );
    assert!(
        widget
            .admit(&json!({}), "edit", &json!({"row":"toggle","value":true}))
            .is_ok()
    );
    assert!(
        widget
            .admit(&json!({}), "navigate", &json!({"page":"page"}))
            .is_ok()
    );
    assert!(
        widget
            .admit(&json!({}), "edit", &json!({"row":"missing","value":true}))
            .is_err()
    );
    assert!(
        widget
            .admit(&json!({}), "edit", &json!({"row":"toggle","value":"true"}))
            .is_err()
    );
}

#[test]
fn passive_numeric_redraws_never_publish_rounded_or_clamped_settings() {
    let ctx = egui::Context::default();
    viu::appearance::install(&ctx);
    let widget = Widget::new(
        "browser",
        "settings-browser",
        "",
        json!({
            "title":"Text","pages":[],"settings":{},"sections":[{"plain":true,"rows":[
                {"id":"size","title":"Font size","kind":"number","inline":true,"value":14.6666666667,"min":4,"max":72,"places":1,"slider":true},
                {"id":"scrollback","title":"Scrollback","kind":"number","inline":true,"value":1000,"min":0,"max":100000}
            ]}]
        }),
    );
    let mut widgets = Widgets::default();
    for _ in 0..6 {
        let events = frame(&ctx, &mut widgets, &widget, vec![]);
        assert!(
            !events.iter().any(|e| e.action == "edit"),
            "passive redraw wrote settings: {events:?}"
        );
    }
}

#[test]
fn scene_container_gaps_do_not_change_the_shared_controls_internal_spacing() {
    let ctx = egui::Context::default();
    let mut style = (*ctx.style_of(egui::Theme::Dark)).clone();
    style.spacing.item_spacing.y = 9.0;
    ctx.set_theme(egui::Theme::Dark);
    ctx.set_style_of(egui::Theme::Dark, style);
    let widget = Widget::new("items", "tree-table", "", table(2))
        .with_properties(json!({"height":200,"row_height":20}));
    let mut direct = Widgets::default();
    frame(&ctx, &mut direct, &widget, vec![]);
    let direct_step = direct.geometry["items.row.1"].min.y - direct.geometry["items.row.0"].min.y;
    let source = format!(
        "---\nscene: 1\nname: spacing\ncitizen: test\n---\n```mix\n{}\n```\n",
        json!({"root":{"widget":"column","gap":0,"children":["items"]},"items":{"widget":"control","definition":widget.encode()}})
    );
    let mut view = SceneView::new(&source).unwrap();
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        view.show(ui);
    });
    output.textures_delta.clear();
    let scene_step = view.geometry["items.row.1"].min.y - view.geometry["items.row.0"].min.y;
    assert_eq!(scene_step, direct_step);
}

#[test]
fn tree_table_virtualises_and_preserves_native_selection_modifiers() {
    let ctx = egui::Context::default();
    let mut widgets = Widgets::default();
    let w =
        Widget::new("items", "tree-table", "", table(3000)).with_properties(json!({"height":320}));
    frame(&ctx, &mut widgets, &w, vec![]);
    assert!(
        widgets
            .geometry
            .keys()
            .filter(|k| k.starts_with("items.row."))
            .count()
            < 30
    );
    assert!(!widgets.geometry.contains_key("items.row.2999"));
    let p = widgets.geometry["items.row.1"].center();
    frame(&ctx, &mut widgets, &w, vec![egui::Event::PointerMoved(p)]);
    let modifiers = egui::Modifiers {
        ctrl: true,
        shift: true,
        ..Default::default()
    };
    let intents = frame(
        &ctx,
        &mut widgets,
        &w,
        [true, false]
            .into_iter()
            .map(|pressed| egui::Event::PointerButton {
                pos: p,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers,
            })
            .collect(),
    );
    assert!(
        intents.iter().any(|i| i.action == "select"
            && i.value["key"] == "item-1"
            && i.value["ctrl"] == true
            && i.value["shift"] == true),
        "{intents:?}"
    );
}

#[test]
fn tree_table_admission_is_based_on_current_row_and_column_identity() {
    let w = Widget::new("renamed", "tree-table", "", table(3));
    assert!(
        w.admit(&json!({}), "activate", &json!({"key":"item-2"}))
            .is_ok()
    );
    assert!(
        w.admit(&json!({}), "activate", &json!({"key":"item-99"}))
            .is_err()
    );
    assert!(
        w.admit(&json!({}), "expand", &json!({"key":"item-2"}))
            .is_err()
    );
    assert!(
        w.admit(&json!({}), "sort", &json!({"column":"modified"}))
            .is_ok()
    );
    assert!(
        w.admit(&json!({}), "sort", &json!({"column":"unknown"}))
            .is_err()
    );
    assert!(
        w.admit(&json!({}), "command", &json!({"command":"disabled"}))
            .is_err()
    );
    let mut bad = table(3);
    bad["rows"][1]["key"] = json!("item-0");
    assert!(w.validate(&bad).is_err());
    for selection in [json!(["absent"]), json!(["item-0", "item-0"]), json!(false)] {
        let mut bad = table(3);
        bad["selected_keys"] = selection;
        assert!(w.validate(&bad).is_err());
    }
    let mut bad = table(3);
    bad["focused_key"] = json!("absent");
    assert!(w.validate(&bad).is_err());
}

#[test]
fn native_drag_to_a_folder_in_the_same_table_reaches_the_folder_before_background() {
    let ctx = egui::Context::default();
    let mut widgets = Widgets::default();
    let mut value = table(3);
    value["selected_keys"] = json!(["item-1", "item-2"]);
    value["focused_key"] = json!("item-1");
    let w = Widget::new("items", "tree-table", "", value).with_properties(json!({"height":320}));
    frame(&ctx, &mut widgets, &w, vec![]);
    let start = widgets.geometry["items.row.1"].center();
    let end = widgets.geometry["items.row.0"].center();
    frame(
        &ctx,
        &mut widgets,
        &w,
        vec![egui::Event::PointerMoved(start)],
    );
    frame(
        &ctx,
        &mut widgets,
        &w,
        vec![egui::Event::PointerButton {
            pos: start,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    frame(
        &ctx,
        &mut widgets,
        &w,
        vec![egui::Event::PointerMoved(start + egui::vec2(12.0, 0.0))],
    );
    frame(&ctx, &mut widgets, &w, vec![egui::Event::PointerMoved(end)]);
    let events = frame(
        &ctx,
        &mut widgets,
        &w,
        vec![egui::Event::PointerButton {
            pos: end,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: egui::Modifiers::NONE,
        }],
    );
    assert!(
        events.iter().any(|e| e.action == "drop"
            && e.value["destination"] == "item-0"
            && e.value["keys"] == json!(["item-1", "item-2"])),
        "{events:?}"
    );
    assert_eq!(events.iter().filter(|e| e.action == "drop").count(), 1);
}

#[test]
fn menus_and_forms_refuse_disabled_and_stale_choices() {
    let bar = Widget::new(
        "commands",
        "command-bar",
        "",
        json!([{"id":"go","label":"Go","enabled":false,"icon":"play"}]),
    );
    assert!(
        bar.admit(&json!({}), "command", &json!({"command":"go"}))
            .is_err()
    );
    let form = Widget::new(
        "prompt",
        "dialog-form",
        "",
        json!({"open":false,"title":"Name","body":"","field":"name","choices":[{"id":"ok","label":"OK","enabled":true}]}),
    );
    assert!(
        form.admit(&json!({}), "answer", &json!({"choice":"ok"}))
            .is_err()
    );
    let mut invalid = bar.initial.clone();
    invalid[0]["icon"] = json!("absent");
    assert!(bar.validate(&invalid).is_err());
    let open = Widget::new(
        "prompt",
        "dialog-form",
        "",
        json!({"open":true,"instance":2,"title":"Delete","body":"Pinned selection","field":null,"choices":[{"id":"ok","label":"Delete","enabled":true}]}),
    );
    for payload in [json!({"choice":"ok"}), json!({"choice":"ok","instance":1})] {
        assert!(open.admit(&json!({}), "answer", &payload).is_err());
    }
    assert!(
        open.admit(&json!({}), "answer", &json!({"choice":"ok","instance":2}))
            .is_ok()
    );
    let nested = Widget::new(
        "commands",
        "command-bar",
        "",
        json!([{"id":"parent","label":"Parent","enabled":false,"children":[{"id":"child","label":"Child","enabled":true}]}]),
    );
    assert!(
        nested
            .admit(&json!({}), "command", &json!({"command":"child"}))
            .is_err()
    );
}

fn scene(nodes: Value) -> String {
    format!("---\nscene: 1\nname: layout\ncitizen: test\n---\n```mix\n{nodes}\n```\n")
}
#[test]
fn dock_and_split_validate_bounds_and_expose_live_divider_geometry() {
    let nodes = json!({"root":{"widget":"dock","children":["heading","body"],"placements":{"heading":{"edge":"top","size":30},"body":{"edge":"fill"}}},"heading":{"widget":"text","text":"Header"},"body":{"widget":"split","children":["a","b"],"ratio":0.5,"on_change":"layout.change"},"a":{"widget":"text","text":"Left"},"b":{"widget":"text","text":"Right"}});
    let mut view = SceneView::new(&scene(nodes.clone())).unwrap();
    let ctx = egui::Context::default();
    let mut out = ctx.run_ui(
        egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 500.0),
            )),
            ..Default::default()
        },
        |ui| {
            view.show(ui);
        },
    );
    out.textures_delta.clear();
    assert!(view.geometry["a"].center().x < view.geometry["body.divider"].center().x);
    assert!(view.geometry["b"].center().x > view.geometry["body.divider"].center().x);
    assert!(view.admit_intent("body", "set", &json!(0.5)).is_ok());
    assert!(view.admit_intent("body", "set", &json!(1.5)).is_err());
    assert!(
        view.admit_intent("heading", "set", &json!("policy"))
            .is_err()
    );
    let mut bad = nodes.clone();
    bad["body"]["minimum"] = json!(0.9);
    bad["body"]["maximum"] = json!(0.1);
    assert!(SceneView::new(&scene(bad)).is_err());
    let mut bad = nodes;
    bad["root"]["placements"]["heading"]["edge"] = json!("fill");
    assert!(SceneView::new(&scene(bad)).is_err());
}

#[test]
fn dock_resize_native_events_and_admission_share_the_declared_fraction_bounds() {
    let source = scene(
        json!({"root":{"widget":"dock","on_change":"layout.change","children":["side","body"],"placements":{"side":{"edge":"left","fraction":0.2,"resizable":true,"min_fraction":0.1,"max_fraction":0.3},"body":{"edge":"fill"}}},"side":{"widget":"text","text":"Sidebar"},"body":{"widget":"text","text":"Body"}}),
    );
    let mut view = SceneView::new(&source).unwrap();
    for value in [0.1, 0.2, 0.3] {
        assert!(
            view.admit_intent("root", "set", &json!({"child":"side","fraction":value}))
                .is_ok()
        );
    }
    for value in [0.09, 0.31, 0.7] {
        assert!(
            view.admit_intent("root", "set", &json!({"child":"side","fraction":value}))
                .is_err()
        );
    }
    let ctx = egui::Context::default();
    let raw = |events| egui::RawInput {
        screen_rect: Some(egui::Rect::from_min_size(
            egui::Pos2::ZERO,
            egui::vec2(800.0, 400.0),
        )),
        events,
        ..Default::default()
    };
    let mut out = ctx.run_ui(raw(Vec::new()), |ui| {
        view.show(ui);
    });
    out.textures_delta.clear();
    let start = view.geometry["side.divider"].center();
    let mut out = ctx.run_ui(
        raw(vec![
            egui::Event::PointerMoved(start),
            egui::Event::PointerButton {
                pos: start,
                button: egui::PointerButton::Primary,
                pressed: true,
                modifiers: egui::Modifiers::NONE,
            },
        ]),
        |ui| {
            view.show(ui);
        },
    );
    out.textures_delta.clear();
    let mut observed = Vec::new();
    let mut out = ctx.run_ui(
        raw(vec![egui::Event::PointerMoved(egui::pos2(790.0, start.y))]),
        |ui| {
            observed = view.show(ui);
        },
    );
    out.textures_delta.clear();
    assert_eq!(observed.len(), 1);
    let value = &observed[0].body["value"];
    assert!((value["fraction"].as_f64().unwrap() - 0.3).abs() < 1e-6);
    assert!(view.admit_intent("root", "set", value).is_ok());
}
