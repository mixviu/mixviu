// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use viu::{
    view::SceneView,
    widgets::{Widget, Widgets},
};

fn scene(nodes: serde_json::Value) -> String {
    format!("---\nscene: 1\nname: test\ncitizen: test\n---\n```mix\n{nodes}\n```\n")
}

#[test]
fn renamed_disabled_icon_and_invalid_row_payloads_refuse() {
    let disabled = json!({"name":"copy","size":24.0,"colour":[80.0,180.0,230.0],"enabled":false,"selected":false});
    let w = Widget::new("another-icon", "icon", "", disabled.clone());
    assert!(w.validate(&disabled).is_ok());
    assert!(w.admit(&json!({}), "activate", &json!("copy")).is_err());
    let list = Widget::new("files", "list", "", json!(0)).with_properties(json!({"row_count":2}));
    for value in [json!(2), json!(-1), json!(1.5), json!("row"), json!(null)] {
        assert!(list.validate_action("activate", &value).is_err());
    }
    assert!(list.validate_action("activate", &json!(1.0)).is_ok());
}

#[test]
fn malformed_public_widget_is_visible_refusal_without_panic_or_geometry() {
    let ctx = egui::Context::default();
    let mut widgets = Widgets::default();
    let w = Widget::new("broken", "icon", "", json!({"name":"copy"}));
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        widgets.begin_frame();
        assert!(widgets.widget(ui, &w, &json!({})).is_empty());
        assert!(widgets.finish_frame().is_ok());
    });
    output.textures_delta.clear();
    assert!(!widgets.geometry.contains_key("broken"));
}

#[test]
fn frame_boundary_retires_disappearing_widget_geometry() {
    let ctx = egui::Context::default();
    let mut widgets = Widgets::default();
    let w =
        Widget::new("item", "button", "", json!("ready")).with_properties(json!({"text":"Item"}));
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        widgets.begin_frame();
        widgets.widget(ui, &w, &json!({}));
        widgets.finish_frame().unwrap();
    });
    output.textures_delta.clear();
    assert!(widgets.geometry.contains_key("item"));
    widgets.begin_frame();
    widgets.finish_frame().unwrap();
    assert!(widgets.geometry.is_empty());
}

#[test]
fn scene_control_binding_rejects_bad_model_transactionally() {
    let source = scene(
        json!({"root":{"widget":"control","definition":"= $model.control","on_intent":"act"}}),
    );
    let widget = Widget::new(
        "root",
        "icon",
        "",
        json!({"name":"copy","size":24,"colour":[80,180,230],"enabled":true,"selected":false}),
    );
    let good = json!({"control":widget.encode()});
    let mut view = SceneView::with_model(&source, good.clone()).unwrap();
    assert!(view.apply_model(json!([])).is_err());
    let bad = json!({"control":{"id":"root","kind":"icon","label":"","initial":{"name":"copy"}}});
    assert!(view.apply_model(bad).is_err());
    assert!(view.apply_model(json!({"control":null})).is_err());
    assert_eq!(view.revision, 0);
    assert_eq!(view.model(), &good);
    let mut wrong = good.clone();
    wrong["control"]["id"] = json!("different");
    assert!(view.apply_model(wrong).is_err());
}

#[test]
fn unsupported_scene_content_refuses_at_load() {
    let source = scene(json!({"root":{"widget":"image","src":"file:///tmp/private.png"}}));
    assert!(
        SceneView::new(&source)
            .err()
            .unwrap()
            .contains("only known icon:")
    );
}

#[test]
fn applying_a_model_does_not_claim_that_native_controls_have_drawn_it() {
    let source = scene(json!({"root":{"widget":"text","text":"= $model.label"}}));
    let mut view = SceneView::with_model(&source, json!({"label":"Before"})).unwrap();
    let ctx = egui::Context::default();
    assert!(!view.current_revision_drawn());
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        view.show(ui);
    });
    output.textures_delta.clear();
    assert!(view.current_revision_drawn());
    view.apply_model(json!({"label":"After"})).unwrap();
    assert!(!view.current_revision_drawn());
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        view.show(ui);
    });
    output.textures_delta.clear();
    assert!(view.current_revision_drawn());
    assert!(view.apply_model(json!([])).is_err());
    assert!(view.current_revision_drawn());
}

#[test]
fn menu_paths_drive_native_popups_and_refuse_disabled_or_leaf_targets() {
    let menus = json!([{"id":"View","label":"View","enabled":true,"children":[
        {"id":"Theme","label":"Theme","enabled":true,"children":[
            {"id":"Dark","label":"Dark","enabled":true}]},
        {"id":"Off","label":"Off","enabled":false,"children":[]}]}]);
    for args in [
        json!({"menu":"missing"}),
        json!({"path":["View","Off"]}),
        json!({"path":["View","Theme","Dark"]}),
    ] {
        assert!(viu::rich::menu_targets(&menus, "menu", &args).is_err());
    }
    let source = scene(
        json!({"root":{"widget":"column","children":["menu"]},"menu":{"widget":"control","definition":{
            "id":"menu","kind":"menubar","label":"","initial":menus
        }}}),
    );
    let mut view = SceneView::with_model(&source, json!({"values":{"menu":menus}})).unwrap();
    let ctx = egui::Context::default();
    let mut input = viu::input::Input::default();
    for frame in 0..12 {
        let mut raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 400.0),
            )),
            time: Some(frame as f64 * 0.05),
            ..Default::default()
        };
        raw.events.append(&mut input.now);
        let mut out = ctx.run_ui(raw, |ui| {
            view.show(ui);
        });
        out.textures_delta.clear();
        if frame == 0 {
            input
                .queue("menu.open", &json!({"path":["View","Theme"]}), &mut view)
                .unwrap();
        } else if let Some(next) = input.later.pop_front() {
            input.now = next;
        } else if let Some(result) = input.advance_menu(&mut view) {
            result.unwrap();
        } else {
            assert_eq!(
                viu::rich::menu_snapshot(&menus, "menu", &view.geometry)["path"],
                json!(["View", "Theme"]),
                "Native input must not finish on a popup sizing pass"
            );
        }
    }
    let snapshot = viu::rich::menu_snapshot(&menus, "menu", &view.geometry);
    assert_eq!(snapshot["path"], json!(["View", "Theme"]));
    assert!(view.geometry.contains_key("menu.command.Dark"));
    egui::Popup::close_all(&ctx);
    let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
        view.show(ui);
    });
    out.textures_delta.clear();
    assert_eq!(
        viu::rich::menu_snapshot(&menus, "menu", &view.geometry)["open"],
        false
    );
}

#[test]
fn host_slots_reuse_scene_composition_without_consuming_the_fallback_dock_edge() {
    let source = scene(
        json!({"root":{"widget":"dock","children":["header","body"],"placements":{"header":{"edge":"top","size":20},"body":{"edge":"fill"}}},"header":{"widget":"column","window_slot":"host.header","children":["heading","only"]},"heading":{"widget":"text","text":"Header"},"only":{"widget":"text","text":"Attached","slot_only":"host.header"},"body":{"widget":"text","text":"Body"}}),
    );
    let mut view = SceneView::new(&source).unwrap();
    let ctx = egui::Context::default();
    let pass = |view: &mut SceneView, reserve: f32| {
        let mut content_top = 0.0;
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(400.0, 300.0),
                )),
                ..Default::default()
            },
            |ui| {
                if reserve > 0.0 {
                    ui.allocate_space(egui::vec2(400.0, reserve));
                }
                content_top = ui.available_rect_before_wrap().min.y;
                view.show(ui);
            },
        );
        out.textures_delta.clear();
        content_top
    };
    pass(&mut view, 0.0);
    assert!(!view.geometry.contains_key("only"));
    assert!((view.geometry["body"].min.y - 20.0).abs() < 0.1);
    assert!(
        view.set_slots(std::collections::BTreeMap::from([(
            "unknown".into(),
            egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(20.0, 20.0))
        )]))
        .is_err()
    );
    view.set_slots(std::collections::BTreeMap::from([(
        "host.header".into(),
        egui::Rect::from_min_size(egui::pos2(100.0, 0.0), egui::vec2(250.0, 40.0)),
    )]))
    .unwrap();
    assert!(!view.current_revision_drawn());
    let content_top = pass(&mut view, 40.0);
    assert!(view.geometry.contains_key("only"));
    assert!((view.geometry["heading"].min.x - 100.0).abs() < 0.1);
    assert!(view.geometry["body"].min.y >= 40.0);
    assert!((view.geometry["body"].min.y - content_top).abs() < 0.1);
    view.set_slots(std::collections::BTreeMap::new()).unwrap();
    pass(&mut view, 0.0);
    assert!(!view.geometry.contains_key("only"));
    assert!((view.geometry["body"].min.y - 20.0).abs() < 0.1);
}

#[test]
fn compact_and_requested_vertical_menus_keep_native_paths_and_report_the_effective_form() {
    let menus = json!([{"id":"View","label":"View","enabled":true,"children":[{"id":"Dark","label":"Dark","enabled":true}]}]);
    let source = scene(
        json!({"root":{"widget":"column","children":["menu"]},"menu":{"widget":"control","definition":{"id":"menu","kind":"menubar","label":"","initial":menus}}}),
    );
    let mut view = SceneView::with_model(&source, json!({"values":{"menu":menus}})).unwrap();
    let ctx = egui::Context::default();
    viu::appearance::install(&ctx);
    viu::rich::configure_menu(
        &ctx,
        &json!({"window.menu_style":"titlebar","window.menu_side":"right"}),
        Some("example"),
    );
    assert_eq!(viu::rich::menu_form(&ctx)["effective"], "button");
    assert_eq!(viu::rich::menu_form(&ctx)["hosted"], false);
    let mut input = viu::input::Input::default();
    for frame in 0..24 {
        let mut raw = egui::RawInput {
            screen_rect: Some(egui::Rect::from_min_size(
                egui::Pos2::ZERO,
                egui::vec2(800.0, 400.0),
            )),
            time: Some(frame as f64 * 0.05),
            ..Default::default()
        };
        raw.events.append(&mut input.now);
        let mut out = ctx.run_ui(raw, |ui| {
            view.show(ui);
        });
        out.textures_delta.clear();
        if frame == 0 {
            input
                .queue("menu.open", &json!({"menu":"View"}), &mut view)
                .unwrap();
        } else if let Some(next) = input.later.pop_front() {
            input.now = next;
        } else if let Some(result) = input.advance_menu(&mut view) {
            result.unwrap();
        }
    }
    assert!(view.geometry.contains_key("menu.command.Dark"));
    egui::Popup::close_all(&ctx);
    for args in [
        json!({"side":"invalid"}),
        json!({"x":"bad"}),
        json!({"x":1e100}),
        json!([]),
    ] {
        assert!(viu::rich::menu_popup(&ctx, "menu", &args).is_err());
    }
    viu::rich::menu_popup(&ctx, "menu", &json!({"side":"right","x":1e6})).unwrap();
    for _ in 0..3 {
        let mut out = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 400.0),
                )),
                ..Default::default()
            },
            |ui| {
                view.show(ui);
            },
        );
        out.textures_delta.clear();
    }
    assert!(view.geometry.contains_key("menu.menu.View"));
    assert_eq!(
        viu::rich::menu_snapshot(&menus, "menu", &view.geometry)["open"],
        true
    );
    egui::Popup::close_all(&ctx);
    let mut out = ctx.run_ui(egui::RawInput::default(), |ui| {
        view.show(ui);
    });
    out.textures_delta.clear();
    assert!(!view.geometry.contains_key("menu.menu.View"));
    assert_eq!(
        viu::rich::menu_snapshot(&menus, "menu", &view.geometry)["open"],
        false
    );
}

#[test]
fn virtual_table_data_has_a_separate_bounded_transactional_model_budget() {
    let source = scene(json!({"root":{"widget":"control","definition":{
        "id":"root","kind":"tree-table","label":"","initial":{
            "key":"files","columns":[{"id":"name","label":"Name"}],"rows":[]
        }
    }}}));
    let rows: Vec<_> = (0..2000)
        .map(|index| {
            json!({
                "key":format!("/example/{}/{index}", "long-directory/".repeat(12)),
                "cells":[format!("item-{index}.txt")],"depth":0,"branch":false,
                "expanded":false,"selected":false,"focused":false
            })
        })
        .collect();
    let model = json!({"values":{"root":{"key":"files","columns":[{"id":"name","label":"Name"}],"rows":rows}}});
    assert!(serde_json::to_vec(&model).unwrap().len() > scene::MAX_DOCUMENT_BYTES);
    let mut view = SceneView::new(&source).unwrap();
    view.apply_model(model.clone()).unwrap();
    assert_eq!(view.model(), &model);
    SceneView::with_model(&source, model.clone()).unwrap();
    let mut oversized = model.clone();
    oversized["padding"] = json!("x".repeat(viu::view::MAX_MODEL_BYTES));
    assert!(
        view.apply_model(oversized.clone())
            .unwrap_err()
            .contains("data budget")
    );
    assert!(SceneView::with_model(&source, oversized).is_err());
    assert_eq!(view.revision, 1);
    assert_eq!(view.model(), &model);
    let document = scene::parse(&source).unwrap();
    let tree = scene::resolve(&document).unwrap();
    let bindings = scene::bindings::BindingSet::from_resolved(&tree);
    assert!(scene::bindings::reevaluate(&tree, &bindings, "model", &model).is_err());
}

#[test]
fn scene_band_reuses_the_native_solver_with_app_owned_groups_and_transactional_validation() {
    let source=scene(json!({"root":{"widget":"band","children":["a","b","c"],"groups":"= $model.groups"},"a":{"widget":"text","text":"Menu"},"b":{"widget":"text","text":"Title"},"c":{"widget":"text","text":"Actions"}}));
    let ctx=egui::Context::default();
    for alignment in ["left","center","right"] {
        let model=json!({"groups":{"a":{"preferred":100,"minimum":40,"align":alignment,"shrink":1,"order":0},"b":{"preferred":200,"minimum":0,"align":"center","shrink":0,"order":1},"c":{"preferred":150,"minimum":80,"align":"right","shrink":1,"order":2}}});
        let mut view=SceneView::with_model(&source,model.clone()).unwrap();
        let mut out=ctx.run_ui(egui::RawInput{screen_rect:Some(egui::Rect::from_min_size(egui::Pos2::ZERO,egui::vec2(600.0,300.0))),..Default::default()},|ui|{view.show(ui);});out.textures_delta.clear();
        for (a,b) in [("a","b"),("a","c"),("b","c")] {assert!(!view.geometry[a].intersects(view.geometry[b]),"{alignment}: {:?} {:?}",view.geometry[a],view.geometry[b]);}
        for bad in [json!(null),json!({}),json!({"a":{"preferred":-1,"minimum":0,"align":"left","order":0}})] {
            assert!(view.apply_model(json!({"groups":bad})).is_err());
            assert_eq!(view.model(),&model);
        }
    }
}
