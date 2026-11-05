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
fn scene_container_gaps_do_not_change_the_shared_controls_internal_spacing(){
    let ctx=egui::Context::default();
    let mut style=(*ctx.style()).clone();style.spacing.item_spacing.y=9.0;
    ctx.set_theme(egui::Theme::Dark);ctx.set_style_of(egui::Theme::Dark,style);
    let widget=Widget::new("items","tree-table","",table(2)).with_properties(json!({"height":200,"row_height":20}));
    let mut direct=Widgets::default();frame(&ctx,&mut direct,&widget,vec![]);
    let direct_step=direct.geometry["items.row.1"].min.y-direct.geometry["items.row.0"].min.y;
    let source=format!("---\nscene: 1\nname: spacing\ncitizen: test\n---\n```mix\n{}\n```\n",json!({"root":{"widget":"column","gap":0,"children":["items"]},"items":{"widget":"control","definition":widget.encode()}}));
    let mut view=SceneView::new(&source).unwrap();
    let mut output=ctx.run_ui(egui::RawInput::default(),|ui|{view.show(ui);});output.textures_delta.clear();
    let scene_step=view.geometry["items.row.1"].min.y-view.geometry["items.row.0"].min.y;
    assert_eq!(scene_step,direct_step);
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
}

#[test]
fn native_drag_to_a_folder_in_the_same_table_reaches_the_folder_before_background() {
    let ctx = egui::Context::default();
    let mut widgets = Widgets::default();
    let w = Widget::new("items", "tree-table", "", table(3)).with_properties(json!({"height":320}));
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
            && e.value["keys"] == json!(["item-1"])),
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
