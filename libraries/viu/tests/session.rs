// SPDX-License-Identifier: MIT OR Apache-2.0
use serde_json::json;
use std::{
    sync::{Arc, mpsc},
    time::{Duration, Instant},
};
use viu::runtime::{Program, Runtime};

#[test]
fn mix_session_revisions_retarget_refusals_and_appearance_are_not_f64_state() {
    let source = r#"
$s=require("session.mix")
$state=$s.session_new("example")
$state=$s.session_answer($state,10,{status:"wrong_target",binding:{instance:"real",profile:"default"}},nil)
if $state.binding.instance!="real" or $state.retry!=0 then raise("TEST","retarget") end
$state=$s.session_answer($state,0,{incarnation:"one",revision:"18446744073709551614",values:{"appearance.mode":"auto","state.appearance.daylight":true,"appearance.contrast":"high","apps.dopus.contrast":"normal","keymaps.dopus.nav.parent":["Ctrl+U"]}},nil)
$before=$state
$state=$s.session_answer($state,0,{incarnation:"one",revision:"18446744073709551613",values:{}},nil)
if $state.changed or not deep_eq($before.values,$state.values) then raise("TEST","stale revision") end
$state=$s.session_answer($state,10,{status:"denied"},"Denied")
if $state.lost or not deep_eq($before.values,$state.values) then raise("TEST","refusal is not absence") end
$appearance=$s.session_appearance($state.values,"dopus")
if $appearance.mode!="light" or $appearance.contrast!="high" then raise("TEST","appearance overlay") end
$keys=$s.session_keys($state.values,"dopus")
if not deep_eq($keys["nav.parent"],["Ctrl+U"]) then raise("TEST","key binding") end
$state=$s.session_answer($state,10,nil,"Service not found")
if not $state.lost then raise("TEST","absence") end
$state=$s.session_answer($state,0,{incarnation:"one",revision:"18446744073709551614",values:$before.values},nil)
if not $state.changed or $state.lost then raise("TEST","same revision after reconnect") end
on verify
  reply(0,json_encode({verified:true}))
end
"#;
    let (tx, rx) = mpsc::channel();
    let mut runtime = Runtime::new(
        2,
        Arc::new(move || {
            let _ = tx.send(());
        }),
        None,
    );
    runtime
        .load(Program {
            path: std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("mix/session-probe.mix"),
            source: source.into(),
            scene: "session-test".into(),
            citizen: "session-test".into(),
        })
        .unwrap();
    let seq = runtime.event("verify".into(), json!({})).unwrap();
    let until = Instant::now() + Duration::from_secs(5);
    loop {
        for done in runtime.drain() {
            assert!(done.error.is_none(), "{:?}", done.error);
            if done.sequence == seq {
                assert_eq!(done.reply.unwrap().body, json!({"verified":true}));
                return;
            }
        }
        assert!(Instant::now() < until);
        let _ = rx.recv_timeout(Duration::from_millis(20));
    }
}

#[test]
fn shared_design_applies_session_density_roundness_and_text_scale_once() {
    let ordinary = viu::appearance::resolve(&json!({})).unwrap().style;
    let changed = viu::appearance::resolve(
        &json!({"roundness":"square","density":"compact","text_scale":1.5,"reduced_motion":true}),
    )
    .unwrap()
    .style;
    assert_eq!(
        changed.visuals.widgets.inactive.corner_radius,
        egui::CornerRadius::ZERO
    );
    assert!(
        (changed.spacing.interact_size.y - ordinary.spacing.interact_size.y * 0.85).abs() < 0.01
    );
    assert!(
        (changed.text_styles[&egui::TextStyle::Body].size
            - ordinary.text_styles[&egui::TextStyle::Body].size * 1.5)
            .abs()
            < 0.01
    );
    assert_eq!(changed.animation_time, 0.0);
}

#[test]
fn client_captions_have_native_geometry_and_close_is_an_owner_intent() {
    let ctx = egui::Context::default();
    let mut frame = viu::window_frame::WindowFrame::default();
    let settings = json!({"window.decorations":"csd","window.caption_side":"left"});
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        frame.show(ui, "Reusable window", &settings);
        frame.edges(ui);
    });
    output.textures_delta.clear();
    assert!(
        frame.geometry["window.caption.close"].left()
            < frame.geometry["window.caption.maximize"].left()
    );
    assert_eq!(
        frame
            .geometry
            .keys()
            .filter(|key| key.starts_with("window.resize."))
            .count(),
        8
    );
    let point = frame.geometry["window.caption.close"].center();
    let mut output = ctx.run_ui(
        egui::RawInput {
            events: vec![
                egui::Event::PointerMoved(point),
                egui::Event::PointerButton {
                    pos: point,
                    button: egui::PointerButton::Primary,
                    pressed: true,
                    modifiers: egui::Modifiers::NONE,
                },
                egui::Event::PointerButton {
                    pos: point,
                    button: egui::PointerButton::Primary,
                    pressed: false,
                    modifiers: egui::Modifiers::NONE,
                },
            ],
            ..Default::default()
        },
        |ui| {
            frame.show(ui, "Reusable window", &settings);
            frame.edges(ui);
        },
    );
    output.textures_delta.clear();
    assert!(frame.close_requested);
    assert!(
        !output
            .viewport_output
            .values()
            .flat_map(|v| &v.commands)
            .any(|c| matches!(c, egui::ViewportCommand::Close))
    );
    let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
        frame.show(ui, "Reusable window", &json!({"window.decorations":"ssd"}));
        frame.edges(ui);
    });
    output.textures_delta.clear();
    assert!(frame.geometry.is_empty());
    assert!(!frame.close_requested);
}

#[test]
fn scene_authored_caption_groups_share_native_controls_and_retire_when_detached() {
    use viu::{view::SceneView, widgets::Widget};
    let ctx = egui::Context::default();
    viu::appearance::install(&ctx);
    let mut frame = viu::window_frame::WindowFrame::default();
    for side in ["left", "center", "right"] {
        let source = format!(
            "---\nscene: 1\nname: window\ncitizen: window\n---\n```mix\n{}\n```\n",
            json!({"root":{"widget":"dock","children":["header","body"],"placements":{"header":{"edge":"top","size":30},"body":{"edge":"fill"}}},"header":{"widget":"band","window_slot":"window.header","children":["caption","title"],"groups":{"caption":{"preferred":"window.captions","minimum":"window.captions","align":side,"order":3},"title":{"preferred":200,"minimum":0,"align":"center","shrink":0,"order":1}}},"caption":{"widget":"control","slot_only":"window.header","on_intent":"viu.window","definition":Widget::new("caption","window-controls","",json!({"side":side,"enabled":true})).encode()},"title":{"widget":"text","text":"Title"},"body":{"widget":"text","text":"Body"}})
        );
        let mut view = SceneView::new(&source).unwrap();
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 400.0),
                )),
                ..Default::default()
            },
            |ui| {
                frame.show_with_chrome(
                    ui,
                    "Native window",
                    &json!({"window.decorations":"csd"}),
                    true,
                    true,
                );
                assert!(
                    !frame.geometry.contains_key("window.caption.close"),
                    "Scene owns exactly one caption group"
                );
                view.set_slots(std::collections::BTreeMap::from([(
                    "window.header".into(),
                    frame.geometry["window.header"],
                )]))
                .unwrap();
                assert!(view.show(ui).is_empty());
            },
        );
        output.textures_delta.clear();
        let close = view.geometry["caption.caption.close"];
        assert!(!close.intersects(view.geometry["title"]));
        assert_eq!(view.control_kind("caption"), Some("window-controls"));
        let mut intents = Vec::new();
        let point = close.center();
        let mut output = ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(800.0, 400.0),
                )),
                events: vec![
                    egui::Event::PointerMoved(point),
                    egui::Event::PointerButton {
                        pos: point,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: egui::Modifiers::NONE,
                    },
                    egui::Event::PointerButton {
                        pos: point,
                        button: egui::PointerButton::Primary,
                        pressed: false,
                        modifiers: egui::Modifiers::NONE,
                    },
                ],
                ..Default::default()
            },
            |ui| {
                frame.show_with_chrome(ui,"Native window",&json!({"window.decorations":"csd"}),true,true);
                view.set_slots(std::collections::BTreeMap::from([("window.header".into(),frame.geometry["window.header"])] )).unwrap();
                intents = view.show(ui);
            },
        );
        output.textures_delta.clear();
        assert!(
            intents
                .iter()
                .any(|event| event.command == "viu.window" && event.body["action"] == "close")
        );
        assert!(
            !output
                .viewport_output
                .values()
                .flat_map(|v| &v.commands)
                .any(|command| matches!(command, egui::ViewportCommand::Close))
        );
        view.set_slots(Default::default()).unwrap();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            view.show(ui);
        });
        output.textures_delta.clear();
        assert!(!view.geometry.contains_key("caption.caption.close"));
        assert!(viu::window_frame::apply_control(&ctx, "unknown").is_err());
        assert_eq!(
            viu::window_frame::apply_control(&ctx, "close").unwrap(),
            true
        );
    }
    let disabled = Widget::new(
        "caption",
        "window-controls",
        "",
        json!({"side":"right","enabled":false}),
    );
    for action in disabled.actions() {
        assert!(
            disabled
                .admit(&json!({}), action, &serde_json::Value::Null)
                .is_err()
        );
    }
}
