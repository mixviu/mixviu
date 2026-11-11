// SPDX-License-Identifier: MIT OR Apache-2.0
//! Reusable terminal surface. Mix supplies session handles, focus requests,
//! appearance and shortcut bindings; native output stays in native storage.
use super::{
    config::Cursor, gpu::Pane, ime::Composition, input, model::MouseModifiers, raster::Raster,
    sessions::Sessions,
};
use crate::widgets::{Intent, Widget};
use egui::{Color32, Rect, Response, Sense, Ui};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Instant,
};

struct Surface {
    blink: super::blink::Blink,
    raster: Arc<Mutex<Raster>>,
    pane: Pane,
    look: (f32, f32, String, u16),
    focus: u64,
    preedit: String,
    grid_size: (u16, u16, u16, u16),
    search: Option<super::view_search::Search>,
}

// Gold's pane arithmetic: fit in physical pixels, retain a legal minimum
// grid even for tiny panes, and respect the common GPU texture ceiling.
fn grid_geometry(
    area: Rect,
    cell: (u32, u32),
    scale: f32,
    border: f32,
    padding: f32,
) -> ((u16, u16), Rect, f32) {
    let border = (border * scale).round().max(1.0) / scale;
    let inset = ((padding * scale).round() / scale).max(border);
    let fit = |logical: f32, cell: u32, minimum: u32, maximum: u32| {
        ((((logical - 2.0 * inset).max(0.0) * scale + 0.01) / cell.max(1) as f32) as u32)
            .clamp(minimum, maximum.min(4096 / cell.max(1)).max(minimum)) as u16
    };
    let size = (
        fit(area.width(), cell.0, 2, 240),
        fit(area.height(), cell.1, 1, 100),
    );
    let grid = Rect::from_min_size(
        area.min + egui::vec2(inset, inset),
        egui::vec2(
            size.0 as f32 * cell.0 as f32 / scale,
            size.1 as f32 * cell.1 as f32 / scale,
        ),
    );
    (size, grid, border)
}

fn shortcut(
    p: &Value,
    key: egui::Key,
    modifiers: egui::Modifiers,
    alternate: bool,
) -> Option<&Value> {
    if toolkit::keymap::reserved(key, modifiers) {
        return None;
    }
    p["shortcuts"].as_array()?.iter().find(|s| {
        s["key"].as_str() == Some(key.name())
            && (s["ctrl"] == true) == modifiers.ctrl
            && (s["shift"] == true) == modifiers.shift
            && (s["alt"] == true) == modifiers.alt
            && !(s["normal_screen_only"] == true && alternate)
    })
}

pub fn validate(value: &Value) -> Result<(), String> {
    fn node(value: &Value, depth: usize, handles: &mut std::collections::BTreeSet<String>) -> bool {
        if depth >= 64 || handles.len() >= 256 {
            return false;
        }
        if let Some(handle) = value.as_str() {
            return !handle.is_empty() && handle.len() <= 128 && handles.insert(handle.into());
        }
        value.is_object()
            && matches!(value["dir"].as_str(), Some("h" | "v"))
            && value["ratio"]
                .as_f64()
                .is_some_and(|r| r.is_finite() && (0.0..=1.0).contains(&r))
            && node(&value["first"], depth + 1, handles)
            && node(&value["second"], depth + 1, handles)
    }
    if node(value, 0, &mut Default::default()) {
        Ok(())
    } else {
        Err("terminal layout requires bounded distinct session handles and h/v splits".into())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn desk_keys_never_encode_or_dispatch() {
        for code in b"ABCD" {
            for modifier in [3, 5, 7] {
                assert!(
                    super::super::model::encode(super::super::model::Key::Modified {
                        code: *code,
                        modifier,
                    })
                    .is_empty()
                );
            }
        }
        for &key in toolkit::keymap::USER_KEYS.iter().chain(
            [
                egui::Key::ArrowLeft,
                egui::Key::ArrowRight,
                egui::Key::ArrowUp,
                egui::Key::ArrowDown,
            ]
            .iter(),
        ) {
            for modifiers in [
                egui::Modifiers::CTRL,
                egui::Modifiers::ALT,
                egui::Modifiers::CTRL | egui::Modifiers::ALT,
            ] {
                let forged = serde_json::json!({"shortcuts":[{"key":key.name(),"ctrl":modifiers.ctrl,"alt":modifiers.alt,"command":"forged"}]});
                assert!(super::shortcut(&forged, key, modifiers, false).is_none());
                let bytes: Vec<_> = super::input::keys_for(key, modifiers)
                    .into_iter()
                    .flat_map(super::super::model::encode)
                    .collect();
                assert!(bytes.is_empty());
            }
        }
    }
    use super::*;

    #[test]
    fn find_bar_submit_keeps_focus_and_never_reaches_pty() {
        for opt_in in [false, true] {
            for (shift, released) in [(false, false), (true, false), (true, true)] {
                let sessions = Sessions::fixture();
                let receiver = sessions.get("vt-1").unwrap().lock().unwrap().listener.test_input_receiver();
                let ctx = egui::Context::default();
                crate::appearance::install(&ctx);
                crate::fonts::install(&ctx, 400, 600);
                let mut views = Views::default();
                let mut widget = Widget::new("vt", "terminal", "Terminal", json!("vt-1"));
                let mut bar = json!({"pattern":"needle","replacement":"","replace":false,"regex":false,"case":true,"instance":1});
                if opt_in { bar["submit_modifiers"] = json!(true); }
                let key = |pressed, modifiers| egui::Event::Key {
                    key: egui::Key::Enter, physical_key: None, pressed, repeat: false, modifiers,
                };
                let mut frame = |open: bool, serial: u64, input_events: Vec<egui::Event>, modifiers: egui::Modifiers| {
                    let mut events = Vec::new();
                    let mut input_events = input_events;
                    input_events.insert(0, egui::Event::ModifiersChanged(modifiers));
                    widget.properties = json!({"focus_serial":serial});
                    ctx.run_ui(egui::RawInput {
                        screen_rect: Some(Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(900.0,560.0))),
                        events: input_events, ..Default::default()
                    }, |ui| {
                        if open {
                            crate::editor_controls::draw(ui,"find","search-bar",&bar,&mut events);
                        }
                        views.draw(ui,&widget,&json!("vt-1"),&sessions,&mut events);
                    }).drop_without_applying_deltas();
                    events
                };
                for _ in 0..2 { frame(false,1,vec![],egui::Modifiers::NONE); }
                for _ in 0..2 { frame(true,1,vec![],egui::Modifiers::NONE); }
                let field = ctx.memory(|m| m.focused()).expect("find field focused");
                let modifiers = egui::Modifiers { shift, ..egui::Modifiers::NONE };
                let mut input = vec![key(true,modifiers)];
                if released { input.push(egui::Event::ModifiersChanged(egui::Modifiers::NONE)); }
                let events = frame(true,1,input,modifiers);
                let actions: Vec<_> = events.iter().filter(|e| e.id=="find").collect();
                assert_eq!(actions.len(),1,"{events:?}");
                assert_eq!(actions[0].action,if opt_in {"submit"} else {"next"});
                if opt_in {
                    assert_eq!(actions[0].value,json!({"shift":shift}));
                    assert_eq!(ctx.memory(|m| m.focused()),Some(field));
                    let control = Widget::new("find", "search-bar", "Find", bar.clone());
                    assert!(control.actions().contains(&"submit"));
                    assert!(control.validate_action("submit", &actions[0].value).is_ok());
                    assert!(control.admit(&json!({}), "submit", &actions[0].value).is_ok());
                } else {
                    let control = Widget::new("find", "search-bar", "Find", bar.clone());
                    assert!(!control.actions().contains(&"submit"));
                    assert!(control.validate_action("submit", &json!({"shift":shift})).is_err());
                }
                assert!(super::pointer_order_tests::read(&receiver).is_empty(),"find submission reached the PTY input queue");
                frame(false,2,vec![key(false,egui::Modifiers::NONE)],egui::Modifiers::NONE);
                frame(false,2,vec![],egui::Modifiers::NONE);
                frame(false,2,vec![key(true,egui::Modifiers::NONE)],egui::Modifiers::NONE);
                assert_eq!(super::pointer_order_tests::read(&receiver),b"\r","closed find bar must leave Enter with the shell");
            }
        }
    }

    #[test]
    fn authored_search_reports_from_the_drawn_owner_and_clears_on_removal() {
        let sessions = Sessions::fixture();
        sessions.fixture_pane("vt-2", b"sibling");
        let owner = sessions.get("vt-1").unwrap();
        let mut views = Views::default();
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx, 400, 600);
        let mut widget = Widget::new("vt", "terminal", "Terminal", json!("vt-1"));
        widget.properties = json!({"search":{"handle":"vt-1","query":"native","request":1}});
        let mut events = Vec::new();
        let mut draw = |widget: &Widget, value: &Value, events: &mut Vec<Intent>| {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(900.0, 560.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    views.draw(ui, widget, value, &sessions, events);
                },
            )
            .drop_without_applying_deltas();
        };
        draw(&widget, &json!("vt-1"), &mut events);
        assert_eq!(events.iter().filter(|e| e.action == "search").count(), 1);
        let found = events.iter().find(|e| e.action == "search").unwrap();
        assert_eq!(found.id, "vt");
        assert_eq!(found.value["handle"], "vt-1");
        assert_eq!(found.value["matched"]["start"]["col"], 0);
        assert_eq!(found.value["matched"]["end"]["col"], 5);
        events.clear();
        draw(&widget, &json!("vt-1"), &mut events);
        assert!(!events.iter().any(|e| e.action == "search"));
        draw(&widget, &json!("vt-2"), &mut events);
        assert!(!events.iter().any(|e| e.action == "search"));
        assert!(
            sessions
                .get("vt-2")
                .unwrap()
                .lock()
                .unwrap()
                .selection_text()
                .is_none()
        );
        widget.properties["search"] = Value::Null;
        draw(&widget, &json!("vt-1"), &mut events);
        assert!(views.surfaces["vt-1"].search.is_none());
        assert!(Arc::ptr_eq(&owner, &sessions.get("vt-1").unwrap()));
        assert!(owner.lock().unwrap().selection_text().is_none());
        sessions.call("close", "vt-1", json!({})).unwrap();
        let mut widget = Widget::new("vt", "terminal", "Terminal", json!("vt-2"));
        widget.properties = json!({"search":{"handle":"vt-1","query":"sibling","request":2}});
        ctx.run_ui(Default::default(), |ui| {
            views.draw(ui, &widget, &json!("vt-2"), &sessions, &mut events);
        })
        .drop_without_applying_deltas();
        assert!(!views.surfaces.contains_key("vt-1"));
        assert!(!events.iter().any(|e| e.action == "search"));
    }

    #[test]
    fn alternate_screen_keeps_scroll_keys_but_not_other_commands() {
        let shift = egui::Modifiers {
            shift: true,
            ..egui::Modifiers::NONE
        };
        let properties = json!({"shortcuts":[
            {"key":"PageUp","shift":true,"command":"history","normal_screen_only":true},
            {"key":"T","shift":true,"command":"open"}
        ]});
        assert!(shortcut(&properties, egui::Key::PageUp, shift, false).is_some());
        assert!(shortcut(&properties, egui::Key::PageUp, shift, true).is_none());
        assert_eq!(
            shortcut(&properties, egui::Key::T, shift, true).unwrap()["command"],
            "open"
        );
        assert_eq!(
            input::keys_for(egui::Key::PageUp, shift)
                .into_iter()
                .flat_map(super::super::model::encode)
                .collect::<Vec<_>>(),
            vec![27, b'[', b'5', b'~']
        );
    }

    #[test]
    fn held_commands_are_consumed_and_only_authored_repeats_run() {
        let sessions = Sessions::fixture();
        let mut views = Views::default();
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx, 400, 600);
        let mut widget = Widget::new("vt", "terminal", "Terminal", json!("vt-1"));
        for (repeat, allowed, expected) in [(false, false, 1), (true, false, 0), (true, true, 1)] {
            widget.properties = json!({"focus_serial":1,"shortcuts":[{"key":"T","ctrl":true,"command":"open","repeat":allowed}]});
            let mut events = Vec::new();
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(900.0, 560.0),
                    )),
                    events: vec![egui::Event::Key {
                        key: egui::Key::T,
                        physical_key: None,
                        pressed: true,
                        repeat,
                        modifiers: egui::Modifiers {
                            ctrl: true,
                            ..egui::Modifiers::NONE
                        },
                    }],
                    ..Default::default()
                },
                |ui| {
                    views.draw(ui, &widget, &json!("vt-1"), &sessions, &mut events);
                },
            )
            .drop_without_applying_deltas();
            assert_eq!(
                events.iter().filter(|e| e.action == "command").count(),
                expected
            );
            assert!(!events.iter().any(|e| e.action == "error"));
        }
    }

    #[test]
    fn tab_stays_in_terminal_with_another_focusable_control() {
        let sessions = Sessions::fixture();
        let mut views = Views::default();
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx, 400, 600);
        let mut widget = Widget::new("vt", "terminal", "Terminal", json!("vt-1"));
        widget.properties = json!({"focus_serial":1,"shortcuts":[{"key":"Tab","command":"next"}]});
        // egui's filter requires had_focus_last_frame as well as has_focus.
        // Establish focus, install the filter, then deliver the real Tab.
        for pressed in [false, false, true] {
            let mut events = Vec::new();
            let input = if pressed {
                vec![egui::Event::Key {
                    key: egui::Key::Tab,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::NONE,
                }]
            } else {
                vec![]
            };
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(900.0, 560.0),
                    )),
                    events: input,
                    ..Default::default()
                },
                |ui| {
                    views.draw(ui, &widget, &json!("vt-1"), &sessions, &mut events);
                    let _ = ui.button("Other focus target");
                },
            )
            .drop_without_applying_deltas();
            if pressed {
                assert!(
                    events
                        .iter()
                        .any(|e| e.action == "command" && e.value == "next")
                );
            }
        }
    }

    #[test]
    fn control_wheel_reaches_authored_commands_instead_of_scrollback() {
        let sessions = Sessions::fixture();
        let mut views = Views::default();
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx, 400, 600);
        let mut widget = Widget::new("vt", "terminal", "Terminal", json!("vt-1"));
        widget.properties =
            json!({"focus_serial":1,"wheel_zoom":{"increase":"larger","decrease":"smaller"}});
        for wheel in [false, true] {
            let mut events = Vec::new();
            let mut input = vec![egui::Event::PointerMoved(egui::pos2(100.0, 100.0))];
            if wheel {
                input.push(egui::Event::MouseWheel {
                    unit: egui::MouseWheelUnit::Line,
                    delta: egui::vec2(0.0, 2.0),
                    modifiers: egui::Modifiers {
                        ctrl: true,
                        ..egui::Modifiers::NONE
                    },
                    phase: egui::TouchPhase::Move,
                    source: egui::MouseWheelSource::Unknown,
                });
            }
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(900.0, 560.0),
                    )),
                    events: input,
                    ..Default::default()
                },
                |ui| {
                    views.draw(ui, &widget, &json!("vt-1"), &sessions, &mut events);
                },
            )
            .drop_without_applying_deltas();
            if wheel {
                assert_eq!(
                    events
                        .iter()
                        .filter(|e| e.action == "command" && e.value == "larger")
                        .count(),
                    2
                );
            }
        }
    }

    #[test]
    fn pane_geometry_matches_gold_at_fractional_scales_and_texture_limits() {
        let area = |w, h| Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(w, h));
        for (w, h, cell, scale, expected) in [
            (816.0, 496.0, (8, 16), 1.0, (100, 30)),
            (815.0, 495.0, (8, 16), 1.0, (99, 29)),
            (400.0, 200.0, (20, 40), 2.5, (48, 11)),
            (3.0, 3.0, (8, 16), 1.0, (2, 1)),
            (9000.0, 9000.0, (40, 80), 1.0, (102, 51)),
        ] {
            assert_eq!(grid_geometry(area(w, h), cell, scale, 1.0, 8.0).0, expected);
        }
        for scale in [1.0, 1.25, 1.5, 2.5] {
            for width in 1..=4 {
                for padding in [0, 1, 4, 8, 16] {
                    let (_, rect, border) = grid_geometry(
                        area(832.0, 512.0),
                        (19, 40),
                        scale,
                        width as f32,
                        padding as f32,
                    );
                    assert!((rect.min.x * scale).fract().abs() < 0.001);
                    assert!(rect.min.x >= border);
                    assert!(border * scale >= 1.0);
                }
            }
        }
    }

    #[test]
    fn pane_readbacks_are_widget_local_while_pointer_areas_stay_in_the_window() {
        let sessions = Sessions::fixture();
        for handle in ["vt-2", "vt-3", "vt-4"] {
            sessions.fixture_pane(handle, b"pane geometry");
        }
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx, 400, 600);
        let mut views = Views::default();
        let widget = Widget::new("split", "terminal", "Terminal", json!(null));
        let sibling = Widget::new("sibling", "terminal", "Terminal", json!(null));
        let layout = json!({"dir":"v","ratio":0.5,"first":"vt-1",
            "second":{"dir":"h","ratio":0.5,"first":"vt-2","second":"vt-3"}});
        let expected = [
            ("vt-1", [0.0, 0.0, 300.0, 400.0]),
            ("vt-2", [300.0, 0.0, 300.0, 200.0]),
            ("vt-3", [300.0, 200.0, 300.0, 200.0]),
        ];
        for origin in [egui::pos2(32.0, 70.0), egui::pos2(45.0, 110.0)] {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(1000.0, 700.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    let mut child = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(Rect::from_min_size(origin, egui::vec2(600.0, 400.0)))
                            .layout(egui::Layout::top_down(egui::Align::Min)),
                    );
                    views.draw(&mut child, &widget, &layout, &sessions, &mut Vec::new());
                    for (placement, (handle, rect)) in views.placements.iter().zip(expected) {
                        assert_eq!(placement.handle, handle);
                        assert_eq!(placement.area.min, origin + egui::vec2(rect[0], rect[1]));
                        assert_eq!(placement.area.size(), egui::vec2(rect[2], rect[3]));
                    }
                    let mut other = ui.new_child(
                        egui::UiBuilder::new()
                            .max_rect(Rect::from_min_size(
                                egui::pos2(740.0, 65.0),
                                egui::vec2(120.0, 100.0),
                            ))
                            .layout(egui::Layout::top_down(egui::Align::Min)),
                    );
                    views.draw(
                        &mut other,
                        &sibling,
                        &json!("vt-4"),
                        &sessions,
                        &mut Vec::new(),
                    );
                    assert_eq!(views.placements[0].area.min, egui::pos2(740.0, 65.0));
                },
            )
            .drop_without_applying_deltas();
            for (handle, rect) in expected
                .into_iter()
                .chain([("vt-4", [0.0, 0.0, 120.0, 100.0])])
            {
                let state = sessions.call("state", handle, json!({})).unwrap();
                assert_eq!(
                    state["geometry"],
                    json!({"x":rect[0],"y":rect[1],"w":rect[2],"h":rect[3]})
                );
                let window_origin = if handle == "vt-4" {
                    egui::pos2(740.0, 65.0)
                } else {
                    origin
                };
                assert_eq!(
                    state["window_geometry"],
                    json!({"x":window_origin.x+rect[0],"y":window_origin.y+rect[1],"w":rect[2],"h":rect[3]})
                );
            }
        }
    }

    #[test]
    fn malformed_and_aliased_layouts_are_refused() {
        assert!(validate(&json!({"dir":"v","ratio":0.5,"first":"vt-1","second":"vt-2"})).is_ok());
        for value in [
            json!({"dir":"v","ratio":0.5,"first":"vt-1","second":"vt-1"}),
            json!({"dir":"x","ratio":0.5,"first":"vt-1","second":"vt-2"}),
            json!({"dir":"h","ratio":2,"first":"vt-1","second":"vt-2"}),
        ] {
            assert!(validate(&value).is_err());
        }
    }

    #[test]
    fn live_font_change_keeps_session_and_idle_draw_does_not_resize_again() {
        let sessions = Sessions::fixture();
        let owner = sessions.get("vt-1").unwrap();
        let mut views = Views::default();
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx, 400, 600);
        let mut widget = Widget::new("vt", "terminal", "Terminal", json!("vt-1"));
        let mut previous_height = None;
        for (size, text_scale) in [
            (14.667, 1.0),
            (14.667, 1.0),
            (18.0, 1.0),
            (18.0, 1.5),
            (18.0, 1.5),
        ] {
            widget.properties = json!({"font_size":size,"text_scale":text_scale});
            let mut output = ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(900.0, 560.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    views.draw(ui, &widget, &json!("vt-1"), &sessions, &mut Vec::new());
                },
            );
            output.textures_delta.clear();
            assert!(Arc::ptr_eq(&owner, &sessions.get("vt-1").unwrap()));
            let observed = sessions.call("state", "vt-1", json!({})).unwrap();
            assert_eq!(observed["font_px"], json!(size as f32));
            assert_eq!(observed["scale"], json!(1.0));
            let cell_height = observed["cell"][1].as_f64().unwrap();
            assert!(cell_height > 0.0);
            if let Some(previous) = previous_height {
                assert!(cell_height >= previous);
            }
            previous_height = Some(cell_height);
            let t = owner.lock().unwrap();
            assert!(t.snapshot().contains("native fixture"));
            let diagnostic = t.snapshot();
            assert!(
                diagnostic.contains("VT_notification_to_GPU_snapshot:"),
                "{diagnostic}"
            );
            assert!(
                diagnostic.contains("gpu_prepare_cpu_side:n=0"),
                "unrendered egui fixture must not invent prepare samples: {diagnostic}"
            );
            assert!(
                t.stats.lock().unwrap().frames > 0,
                "live widget records GPU handoffs"
            );
            assert!(t.grid_snapshot().dirty_rows.iter().all(|dirty| !dirty));
        }
    }

    #[test]
    fn platform_clipboard_events_reach_mix_once_and_keep_the_pane_owner() {
        let sessions = Sessions::fixture();
        let mut views = Views::default();
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx, 400, 600);
        let mut widget = Widget::new("vt", "terminal", "Terminal", json!("vt-1"));
        widget.properties = json!({"focus_serial":1});
        let shifted = egui::Modifiers {
            ctrl: true,
            shift: true,
            ..egui::Modifiers::NONE
        };
        for (mut input_events, modifiers, expected) in [
            (vec![], shifted, 0),
            (vec![egui::Event::Copy, egui::Event::Copy], shifted, 1),
            (
                vec![
                    egui::Event::Key {
                        key: egui::Key::C,
                        physical_key: None,
                        pressed: false,
                        repeat: false,
                        modifiers: shifted,
                    },
                    egui::Event::Copy,
                ],
                shifted,
                1,
            ),
            (vec![egui::Event::Paste("wide: 中".into())], shifted, 1),
            (
                vec![egui::Event::Copy],
                egui::Modifiers {
                    ctrl: true,
                    ..egui::Modifiers::NONE
                },
                0,
            ),
            (
                vec![
                    egui::Event::Copy,
                    egui::Event::ModifiersChanged(egui::Modifiers::NONE),
                ],
                shifted,
                1,
            ),
        ] {
            let mut events = Vec::new();
            input_events.insert(0, egui::Event::ModifiersChanged(modifiers));
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(900.0, 560.0),
                    )),
                    events: input_events,
                    ..Default::default()
                },
                |ui| {
                    views.draw(ui, &widget, &json!("vt-1"), &sessions, &mut events);
                },
            )
            .drop_without_applying_deltas();
            let clipboard = events
                .iter()
                .filter(|event| event.action == "clipboard")
                .collect::<Vec<_>>();
            assert_eq!(clipboard.len(), expected);
            for event in clipboard {
                assert_eq!(event.value["handle"], "vt-1");
            }
        }
    }
}

#[cfg(test)]
mod pointer_order_tests {
    use super::*;
    use rio_vt::{corcovado::channel, event::Msg};
    pub(super) fn read(receiver: &channel::Receiver<Msg>) -> Vec<u8> {
        let mut out = Vec::new();
        while let Ok(message) = receiver.try_recv() {
            if let Msg::Input(bytes) = message {
                out.extend_from_slice(&bytes);
            }
        }
        out
    }
    fn ordered_input(click_second: bool) {
        let sessions = Sessions::fixture();
        sessions.fixture_pane("vt-2", b"second");
        let first = sessions
            .get("vt-1")
            .unwrap()
            .lock()
            .unwrap()
            .listener
            .test_input_receiver();
        let second = sessions
            .get("vt-2")
            .unwrap()
            .lock()
            .unwrap()
            .listener
            .test_input_receiver();
        let mut views = Views::default();
        let ctx = egui::Context::default();
        crate::fonts::install(&ctx, 400, 600);
        let tree = json!({"dir":"v","ratio":0.5,"first":"vt-1","second":"vt-2"});
        let mut widget = Widget::new("vt", "terminal", "Terminal", tree.clone());
        widget.properties = json!({"focus_serial":1,"focused_handle":"vt-1"});
        for _ in 0..2 {
            ctx.run_ui(
                egui::RawInput {
                    screen_rect: Some(Rect::from_min_size(
                        egui::Pos2::ZERO,
                        egui::vec2(900.0, 560.0),
                    )),
                    ..Default::default()
                },
                |ui| {
                    views.draw(ui, &widget, &tree, &sessions, &mut Vec::new());
                },
            )
            .drop_without_applying_deltas();
        }
        assert_eq!(views.focused.as_deref(), Some("vt-1"));
        let pos = views
            .placements
            .iter()
            .find(|p| p.handle == "vt-2")
            .unwrap()
            .grid
            .center();
        let pos = if click_second {
            pos
        } else {
            egui::pos2(-20.0, -20.0)
        };
        let mut intents = Vec::new();
        ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(900.0, 560.0),
                )),
                events: vec![
                    egui::Event::Text("before".into()),
                    egui::Event::PointerMoved(pos),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: true,
                        modifiers: Default::default(),
                    },
                    egui::Event::Text("after".into()),
                    egui::Event::PointerButton {
                        pos,
                        button: egui::PointerButton::Primary,
                        pressed: false,
                        modifiers: Default::default(),
                    },
                ],
                ..Default::default()
            },
            |ui| {
                views.draw(ui, &widget, &tree, &sessions, &mut intents);
            },
        )
        .drop_without_applying_deltas();
        assert_eq!(read(&first), b"before");
        assert_eq!(
            read(&second),
            if click_second {
                b"after".as_slice()
            } else {
                b""
            }
        );
        assert_eq!(
            views.focused.as_deref(),
            if click_second { Some("vt-2") } else { None }
        );
        assert_eq!(
            intents
                .iter()
                .any(|intent| intent.action == "focus" && intent.value == "vt-2"),
            click_second
        );
    }
    #[test]
    fn typing_before_and_after_a_same_frame_click_reaches_the_correct_pane() {
        ordered_input(true);
    }
    #[test]
    fn outside_click_preserves_preceding_text_then_surrenders_terminal_focus() {
        ordered_input(false);
    }
}

#[derive(Default)]
pub struct Views {
    surfaces: BTreeMap<String, Surface>,
    composition: Composition,
    focused: Option<String>,
    focused_widget: Option<String>,
    clipboard: input::ClipboardInput,
    modifiers: egui::Modifiers,
    wheels: input::Wheels,
    mice: BTreeMap<String, super::pointer::Mouse>,
    mouse_seen: std::collections::BTreeSet<String>,
    placements: Vec<super::pointer::Placement>,
    layout_origin: egui::Pos2,
}

impl Views {
    fn sync_keyboard_focus(&mut self, widget: &Widget, active: Option<&str>) -> bool {
        if active.is_none() && self.focused_widget.as_deref() != Some(widget.id.as_str()) {
            return false;
        }
        if self.focused.as_deref() == active {
            return false;
        }
        self.composition.focus(
            active
                .and_then(|handle| handle.strip_prefix("vt-"))
                .and_then(|s| s.parse().ok()),
        );
        if let Some(old) = self
            .focused
            .as_ref()
            .and_then(|handle| self.surfaces.get_mut(handle))
        {
            old.preedit.clear();
        }
        self.focused = active.map(str::to_owned);
        self.focused_widget = active.map(|_| widget.id.clone());
        self.clipboard = input::ClipboardInput::default();
        self.modifiers = egui::Modifiers::NONE;
        true
    }
    fn keyboard(
        &mut self,
        widget: &Widget,
        handle: &str,
        event: &egui::Event,
        sessions: &Sessions,
        events: &mut Vec<Intent>,
    ) {
        let Ok(owner) = sessions.get(handle) else {
            return;
        };
        let terminal = owner.lock().unwrap();
        let p = &widget.properties;
        let Some(surface) = self.surfaces.get_mut(handle) else {
            return;
        };
        let pane_id = handle
            .strip_prefix("vt-")
            .and_then(|s| s.parse::<u64>().ok());
        self.focused = Some(handle.into());
        self.composition.focus(pane_id);
        let input_events = std::iter::once(event.clone());
        for event in input_events {
            if matches!(
                &event,
                egui::Event::Key { pressed: true, .. }
                    | egui::Event::Text(_)
                    | egui::Event::Ime(egui::ImeEvent::Commit(_))
            ) {
                sessions.activity();
            }
            if let egui::Event::ModifiersChanged(modifiers) = &event {
                self.modifiers = *modifiers;
                self.wheels.modifiers_changed(*modifiers);
            }
            if let Some(clipboard) = self.clipboard.event(&event, self.modifiers) {
                let result = match clipboard {
                    input::Clipboard::Keys(keys) => {
                        surface.blink.activity(Instant::now());
                        terminal.keys(&keys, Instant::now())
                    }
                    input::Clipboard::Ignore => Ok(()),
                    clipboard => {
                        let value = match clipboard {
                            input::Clipboard::Copy => json!({"kind":"copy","handle":handle}),
                            input::Clipboard::Cut => json!({"kind":"cut","handle":handle}),
                            input::Clipboard::Paste(text) => {
                                json!({"kind":"paste","handle":handle,"text":text})
                            }
                            _ => unreachable!(),
                        };
                        events.push(Intent {
                            id: widget.id.clone(),
                            action: "clipboard".into(),
                            value,
                        });
                        Ok(())
                    }
                };
                if let Err(error) = result {
                    events.push(Intent {
                        id: widget.id.clone(),
                        action: "error".into(),
                        value: json!(error),
                    });
                }
                continue;
            }
            let result = match event {
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    repeat,
                    ..
                } => {
                    let command = shortcut(p, key, modifiers, terminal.alternate_screen());
                    if let Some(command) = command {
                        if !repeat || command["repeat"] == true {
                            events.push(Intent {
                                id: widget.id.clone(),
                                action: "command".into(),
                                value: command["command"].clone(),
                            });
                        }
                        Ok(())
                    } else {
                        let keys = input::keys_for(key, modifiers);
                        if !keys.is_empty() {
                            surface.blink.activity(Instant::now());
                        }
                        terminal.keys(&keys, Instant::now())
                    }
                }
                egui::Event::Text(text)
                    if !self.composition.has_owner() && !self.composition.stale() =>
                {
                    let keys = input::typed(&text, self.modifiers);
                    if !keys.is_empty() {
                        surface.blink.activity(Instant::now());
                    }
                    terminal.keys(&keys, Instant::now())
                }
                egui::Event::Ime(egui::ImeEvent::Preedit { text, .. }) => {
                    if self.composition.preedit(&text, pane_id) {
                        surface.preedit = text;
                    } else {
                        surface.preedit.clear();
                    }
                    Ok(())
                }
                egui::Event::Ime(egui::ImeEvent::Commit(text)) => {
                    surface.preedit.clear();
                    if self.composition.commit(pane_id) {
                        surface.blink.activity(Instant::now());
                        terminal.keys(&input::text_keys(&text), Instant::now())
                    } else {
                        Ok(())
                    }
                }
                _ => Ok(()),
            };
            if let Err(error) = result {
                events.push(Intent {
                    id: widget.id.clone(),
                    action: "error".into(),
                    value: json!(error),
                });
            }
        }
    }
    fn tree(
        &mut self,
        ui: &mut Ui,
        widget: &Widget,
        value: &Value,
        sessions: &Sessions,
        events: &mut Vec<Intent>,
        depth: usize,
    ) -> Response {
        if depth >= 64 {
            return ui.colored_label(Color32::RED, "terminal layout nesting exceeds 64");
        }
        if value.is_string() {
            return self.draw_leaf(ui, widget, value, sessions, events);
        }
        let (area, mut response) = ui.allocate_exact_size(ui.available_size(), Sense::hover());
        let ratio = value["ratio"].as_f64().unwrap_or(0.5).clamp(0.0, 1.0) as f32;
        let scale = ui.ctx().pixels_per_point();
        let (mut first, mut second) = (area, area);
        if value["dir"] == "v" {
            let edge = ((area.min.x + area.width() * ratio) * scale).round() / scale;
            first.max.x = edge;
            second.min.x = edge;
        } else if value["dir"] == "h" {
            let edge = ((area.min.y + area.height() * ratio) * scale).round() / scale;
            first.max.y = edge;
            second.min.y = edge;
        } else {
            return ui.colored_label(Color32::RED, "terminal layout requires h/v direction");
        }
        for (part, rect) in [("first", first), ("second", second)] {
            let mut child = ui.new_child(
                egui::UiBuilder::new()
                    .max_rect(rect)
                    .layout(egui::Layout::top_down(egui::Align::Min)),
            );
            child.set_clip_rect(ui.clip_rect().intersect(rect));
            response = response.union(self.tree(
                &mut child,
                widget,
                &value[part],
                sessions,
                events,
                depth + 1,
            ));
        }
        response
    }

    pub fn begin_frame(&mut self) {
        self.mouse_seen.clear();
    }
    pub fn finish_frame(&mut self) {
        self.mice.retain(|id, _| self.mouse_seen.contains(id));
    }
    pub fn draw(
        &mut self,
        ui: &mut Ui,
        widget: &Widget,
        value: &Value,
        sessions: &Sessions,
        events: &mut Vec<Intent>,
    ) -> Response {
        self.mouse_seen.insert(widget.id.clone());
        self.placements.clear();
        self.layout_origin = ui.available_rect_before_wrap().min;
        let response = self.draw_leaf(ui, widget, value, sessions, events);
        let another_grab = self
            .mice
            .iter()
            .any(|(id, mouse)| id != &widget.id && mouse.grabbed());
        let mut mouse = self.mice.remove(&widget.id).unwrap_or_default();
        let raw = ui.input(|i| i.events.clone());
        let accepting = ui.is_enabled() && ui.input(|i| i.focused);
        let now = Instant::now();
        let copy = widget.properties["copy_on_select"] == true;
        mouse.update(&self.placements, sessions, &[], accepting, copy, now);
        let mut position = mouse.position();
        for event in std::iter::once(None).chain(raw.iter().map(Some)) {
            if let Some(event) = event {
                if let egui::Event::ModifiersChanged(value) = event {
                    self.wheels.modifiers_changed(*value);
                }
                let pointer = matches!(
                    event,
                    egui::Event::PointerMoved(_)
                        | egui::Event::PointerButton { .. }
                        | egui::Event::MouseWheel { .. }
                );
                let mut delivered = event.clone();
                if let egui::Event::PointerMoved(pos) | egui::Event::PointerButton { pos, .. } =
                    event
                {
                    position = Some(*pos);
                }
                let blocked = pointer
                    && position.is_some_and(|pos| {
                        ui.ctx()
                            .layer_id_at(pos)
                            .is_some_and(|layer| layer != ui.layer_id())
                    });
                if blocked {
                    mouse.cancel();
                    delivered = egui::Event::PointerGone;
                }
                if let egui::Event::PointerButton {
                    pos, pressed: true, ..
                } = event
                {
                    let inside = self.placements.iter().any(|p| {
                        pos.x >= p.area.min.x
                            && pos.x < p.area.max.x
                            && pos.y >= p.area.min.y
                            && pos.y < p.area.max.y
                    });
                    if blocked || !inside {
                        for placed in &self.placements {
                            ui.memory_mut(|m| m.surrender_focus(placed.id));
                        }
                    }
                }
                if !pointer || !another_grab {
                    let effects = mouse.update(
                        &self.placements,
                        sessions,
                        std::slice::from_ref(&delivered),
                        accepting,
                        copy,
                        now,
                    );
                    for effect in effects {
                        self.pointer_effect(ui, widget, sessions, events, effect);
                    }
                }
            }
            let active = if accepting {
                self.placements
                    .iter()
                    .find(|p| ui.memory(|m| m.has_focus(p.id)))
                    .map(|p| p.handle.clone())
            } else {
                None
            };
            if self.sync_keyboard_focus(widget, active.as_deref()) {
                ui.ctx().request_repaint();
            }
            if let (Some(handle), Some(event)) = (active, event) {
                self.keyboard(widget, &handle, event, sessions, events);
            }
        }
        self.composition.finish_batch();
        if !raw.is_empty() {
            ui.ctx().request_repaint();
        }
        self.mice.insert(widget.id.clone(), mouse);
        response
    }
    fn pointer_effect(
        &mut self,
        ui: &mut Ui,
        widget: &Widget,
        sessions: &Sessions,
        events: &mut Vec<Intent>,
        effect: super::pointer::Effect,
    ) {
        use super::pointer::Effect;
        match effect {
            Effect::Focus(handle) => {
                if let Some(placed) = self.placements.iter().find(|p| p.handle == handle) {
                    let focused = ui.memory(|m| m.has_focus(placed.id));
                    ui.memory_mut(|m| m.request_focus(placed.id));
                    if !focused {
                        events.push(Intent {
                            id: widget.id.clone(),
                            action: "focus".into(),
                            value: json!(handle),
                        });
                    }
                }
            }
            Effect::Read { handle, primary } => events.push(Intent {
                id: widget.id.clone(),
                action: "clipboard".into(),
                value: json!({"kind":"read","handle":handle,"primary":primary}),
            }),
            Effect::Write { text, primary } => events.push(Intent {
                id: widget.id.clone(),
                action: "selection".into(),
                value: json!({"text":text,"primary":primary}),
            }),
            Effect::Wheel {
                handle,
                col,
                row,
                unit,
                delta,
                modifiers,
            } => {
                let Some(placed) = self.placements.iter().find(|p| p.handle == handle) else {
                    return;
                };
                let (zoom, lines) = self.wheels.event(
                    &handle,
                    input::Wheel::of(unit, delta),
                    placed.cell.1,
                    widget.properties["scroll_speed"].as_f64().unwrap_or(40.0) as f32,
                    modifiers,
                );
                if zoom {
                    let command = &widget.properties["wheel_zoom"]
                        [if lines > 0 { "increase" } else { "decrease" }];
                    if command.is_string() {
                        for _ in 0..lines.unsigned_abs() {
                            events.push(Intent {
                                id: widget.id.clone(),
                                action: "command".into(),
                                value: command.clone(),
                            });
                        }
                    }
                } else if let Ok(terminal) = sessions.get(&handle) {
                    let terminal = terminal.lock().unwrap();
                    let mods = MouseModifiers {
                        shift: modifiers.shift,
                        alt: modifiers.alt,
                        ctrl: modifiers.ctrl,
                    };
                    if !terminal.mouse_scroll(col, row, lines, mods) {
                        terminal.scroll_wheel(lines, mods);
                    }
                }
            }
        }
    }
    fn draw_leaf(
        &mut self,
        ui: &mut Ui,
        widget: &Widget,
        value: &Value,
        sessions: &Sessions,
        events: &mut Vec<Intent>,
    ) -> Response {
        if value.is_object() {
            return self.tree(ui, widget, value, sessions, events, 0);
        }
        let handle = value.as_str().unwrap_or_default();
        self.surfaces
            .retain(|handle, _| sessions.get(handle).is_ok());
        let owner = match sessions.get(handle) {
            Ok(t) => t,
            Err(e) => return ui.colored_label(Color32::RED, e),
        };
        let terminal = owner.lock().unwrap();
        let p = &widget.properties;
        let scale = ui.ctx().pixels_per_point();
        let font_px = p["font_size"].as_f64().unwrap_or(14.667) as f32;
        let text_scale = p["text_scale"]
            .as_f64()
            .filter(|n| n.is_finite())
            .unwrap_or(1.0)
            .clamp(0.5, 3.0) as f32;
        let size = font_px * text_scale;
        let family = p["font_family"].as_str().unwrap_or("");
        let weight = crate::widgets::whole(&p["font_weight"]).unwrap_or(400) as u16;
        let look = (scale, size, family.to_owned(), weight);
        let cursor = match p["cursor"].as_str() {
            Some("block") => Cursor::Block,
            Some("bar") => Cursor::Bar,
            _ => Cursor::Underline,
        };
        if !self.surfaces.contains_key(handle) || self.surfaces[handle].look != look {
            let raster = match Raster::with_face(
                scale,
                size,
                cursor,
                if family.is_empty() {
                    None
                } else {
                    Some(family)
                },
                weight,
            ) {
                Ok(r) => r,
                Err(e) => return ui.colored_label(Color32::RED, e),
            };
            self.surfaces.insert(
                handle.into(),
                Surface {
                    blink: super::blink::Blink::default(),
                    raster: Arc::new(Mutex::new(raster)),
                    pane: Pane::new(),
                    look,
                    focus: 0,
                    preedit: String::new(),
                    grid_size: (0, 0, 0, 0),
                    search: None,
                },
            );
        }
        let surface = self.surfaces.get_mut(handle).unwrap();
        let cell = {
            let mut r = surface.raster.lock().unwrap();
            r.set_cursor(cursor);
            r.cell()
        };
        let padding = p["padding"].as_f64().unwrap_or(8.0).max(0.0) as f32;
        let available = ui.available_size().max(egui::vec2(1.0, 1.0));
        let (area, _) = ui.allocate_exact_size(available, Sense::hover());
        let area = Rect::from_min_max(
            egui::pos2(
                (area.min.x * scale).ceil() / scale,
                (area.min.y * scale).ceil() / scale,
            ),
            egui::pos2(
                (area.max.x * scale).floor() / scale,
                (area.max.y * scale).floor() / scale,
            ),
        );
        let id = ui.make_persistent_id((&widget.id, handle));
        // egui normally surrenders focus using the whole frame's pointer state.
        // This compound surface dispatches that transition at the actual press,
        // after any preceding keyboard events have reached their original pane.
        let surrender = ui.memory_mut(|m| {
            let previous = m.options.input_options.surrender_focus_on;
            m.options.input_options.surrender_focus_on = egui::SurrenderFocusOn::Never;
            previous
        });
        let response = ui.interact(area, id, Sense::focusable_noninteractive());
        ui.memory_mut(|m| m.options.input_options.surrender_focus_on = surrender);
        let ((cols, rows), grid, border) = grid_geometry(
            area,
            cell,
            scale,
            p["border_width"].as_f64().unwrap_or(1.0).max(0.0) as f32,
            padding,
        );
        let inner = area.shrink(border);
        let grid_size = (
            cols,
            rows,
            cols.saturating_mul(cell.0 as u16),
            rows.saturating_mul(cell.1 as u16),
        );
        // Session geometry is local to this compound widget, as Gold's pane
        // readbacks and directional focus expect. Pointer/IME rectangles stay
        // in window coordinates below.
        sessions.placed(
            handle,
            area.translate(-self.layout_origin.to_vec2()),
            area,
            cell,
            scale,
            font_px,
        );
        self.placements.push(super::pointer::Placement {
            handle: handle.into(),
            id,
            area,
            grid,
            cell: (cell.0 as f32 / scale, cell.1 as f32 / scale),
            dimensions: (cols, rows),
        });
        if grid_size != surface.grid_size {
            terminal.resize(grid_size.0, grid_size.1, grid_size.2, grid_size.3);
            surface.grid_size = grid_size;
        }
        let serial = crate::widgets::whole(&p["focus_serial"]).unwrap_or(0);
        let focus_target = p["focused_handle"]
            .as_str()
            .is_none_or(|wanted| wanted == handle);
        if focus_target && serial > surface.focus {
            surface.focus = serial;
            response.request_focus();
            events.push(Intent {
                id: widget.id.clone(),
                action: "focus".into(),
                value: json!(handle),
            });
        }
        ui.memory_mut(|m| {
            m.set_focus_lock_filter(
                id,
                egui::EventFilter {
                    tab: true,
                    horizontal_arrows: true,
                    vertical_arrows: true,
                    escape: true,
                },
            )
        });
        let mut snapshot = super::view_search::snapshot(
            &mut surface.search,
            widget,
            handle,
            &terminal,
            super::view_search::colors(ui.ctx()),
            events,
        );
        terminal
            .stats
            .lock()
            .expect("metrics")
            .gpu_frame(surface.pane.take_prepare());
        let (shown, next) = surface.blink.frame(
            Instant::now(),
            response.has_focus() && ui.input(|i| i.focused),
            p["cursor_blink"] == true,
            snapshot.screen.cursor_visible,
        );
        if let Some(next) = next {
            ui.ctx().request_repaint_after(next);
        }
        if !shown {
            snapshot.screen.cursor_visible = false;
        }
        if p["cursor_visible"] == false {
            snapshot.screen.cursor_visible = false;
        }
        let caret = Rect::from_min_size(
            grid.min
                + egui::vec2(
                    snapshot.screen.cursor.0 as f32 * cell.0 as f32 / scale,
                    snapshot.screen.cursor.1 as f32 * cell.1 as f32 / scale,
                ),
            egui::vec2(cell.0 as f32 / scale, cell.1 as f32 / scale),
        );
        let interrupt = self.composition.take_interrupt();
        if response.has_focus() || interrupt {
            ui.ctx().output_mut(|o| {
                o.ime = Some(egui::output::IMEOutput {
                    purpose: egui::IMEPurpose::Terminal,
                    rect: area,
                    cursor_rect: caret,
                    should_interrupt_composition: interrupt,
                })
            });
        }
        let palette = toolkit::chrome::Chrome::of(ui.ctx()).palette;
        let ring = if focus_target && p["show_focus"] == true {
            if p["focus_ring"] == "accent" {
                palette.card_border.lerp_to_gamma(
                    palette.accent,
                    p["focus_strength"].as_f64().unwrap_or(0.25).clamp(0.0, 1.0) as f32,
                )
            } else {
                palette.card_border.lerp_to_gamma(palette.text, 0.25)
            }
        } else {
            palette.card_border
        };
        ui.painter().rect_filled(area, 0.0, ring);
        ui.painter().rect_filled(inner, 0.0, palette.canvas);
        surface.pane.update(snapshot.screen, snapshot.dirty_rows);
        ui.painter()
            .with_clip_rect(inner)
            .add(surface.pane.callback(grid, surface.raster.clone()));
        if !surface.preedit.is_empty() && response.has_focus() {
            let painter = ui.painter();
            let galley = painter.layout_no_wrap(
                surface.preedit.clone(),
                egui::FontId::monospace(caret.height() * 0.8),
                palette.text,
            );
            let rect = Rect::from_min_size(
                caret.min,
                egui::vec2(galley.size().x.max(caret.width()), caret.height()),
            );
            painter.rect_filled(rect, 0.0, palette.field);
            painter.galley(
                egui::pos2(rect.min.x, rect.center().y - galley.size().y / 2.0),
                galley,
                Color32::PLACEHOLDER,
            );
            painter.hline(
                rect.x_range(),
                rect.bottom() - 1.0,
                egui::Stroke::new(1.0, palette.accent),
            );
        }
        response
    }
}
