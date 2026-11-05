// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native egui input automation; all targets come from the current SceneView
//! geometry, and modifiers travel as native input events as well as payloads.
use serde_json::Value;
use std::collections::VecDeque;
use crate::view::SceneView;
#[derive(Default)]
pub struct Input {
    pub now: Vec<egui::Event>,
    pub later: VecDeque<Vec<egui::Event>>,
}
impl Input {
    pub fn queue(&mut self, command: &str, a: &Value, view: &mut SceneView) -> Result<(), String> {
        if !self.now.is_empty() || !self.later.is_empty() {
            return Err("BUSY: native input gesture in progress".into());
        }
        let modifiers = egui::Modifiers {
            ctrl: a["ctrl"] == true,
            command: a["ctrl"] == true,
            shift: a["shift"] == true,
            alt: a["alt"] == true,
            ..Default::default()
        };
        let mut now = vec![egui::Event::ModifiersChanged(modifiers)];
        match command {
            "focus" => {
                if let Some(selection) = a.get("selection") {
                    view.request_focus_value(
                        &serde_json::json!({"id":a["id"],"selection":selection}),
                    )?;
                } else {
                    view.request_focus(a["id"].as_str().ok_or("id required")?)?;
                }
            }
            "key" => {
                let key = a["key"]
                    .as_str()
                    .and_then(egui::Key::from_name)
                    .ok_or("Unknown native key")?;
                for pressed in [true, false] {
                    now.push(egui::Event::Key {
                        key,
                        physical_key: None,
                        pressed,
                        repeat: false,
                        modifiers,
                    });
                }
            }
            "type" | "text" => {
                let text = a["text"]
                    .as_str()
                    .filter(|s| s.len() <= 65_536)
                    .ok_or("Text limit is 65536 bytes")?;
                now.push(egui::Event::Text(text.into()));
            }
            "click" | "hover" | "scroll" | "drag" | "pointer" => {
                let position = |id: &str| {
                    view.geometry
                        .get(id)
                        .filter(|r| r.is_positive())
                        .map(|r| r.center())
                        .ok_or_else(|| format!("Target is not visible: {id}"))
                };
                let start = if let Some(id) = a["id"].as_str() {
                    position(id)?
                } else {
                    let x = a["x"].as_f64().ok_or("id or x/y required")?;
                    let y = a["y"].as_f64().ok_or("id or x/y required")?;
                    if !x.is_finite() || !y.is_finite() {
                        return Err("Coordinates must be finite".into());
                    }
                    egui::pos2(x as f32, y as f32)
                };
                now.push(egui::Event::PointerMoved(start));
                let button = match a["button"].as_str().unwrap_or("left") {
                    "left" => egui::PointerButton::Primary,
                    "right" => egui::PointerButton::Secondary,
                    _ => return Err("Unknown pointer button".into()),
                };
                if command == "scroll" {
                    let delta = a["delta"]
                        .as_f64()
                        .filter(|v| v.is_finite() && v.abs() <= 100_000.0)
                        .ok_or("finite delta required")?;
                    now.push(egui::Event::MouseWheel {
                        unit: egui::MouseWheelUnit::Point,
                        delta: egui::vec2(0.0, delta as f32),
                        modifiers,
                        phase: egui::TouchPhase::Move,
                        source: egui::MouseWheelSource::Unknown,
                    });
                } else if command == "drag" {
                    let end = if let Some(id) = a["to"].as_str() {
                        position(id)?
                    } else {
                        egui::pos2(
                            a["to_x"].as_f64().ok_or("to or to_x/to_y required")? as f32,
                            a["to_y"].as_f64().ok_or("to_y required")? as f32,
                        )
                    };
                    now.push(egui::Event::PointerButton {
                        pos: start,
                        button,
                        pressed: true,
                        modifiers,
                    });
                    self.later
                        .push_back(vec![egui::Event::PointerMoved(start.lerp(end, 0.5))]);
                    self.later.push_back(vec![egui::Event::PointerMoved(end)]);
                    self.later.push_back(vec![
                        egui::Event::PointerMoved(end),
                        egui::Event::PointerButton {
                            pos: end,
                            button,
                            pressed: false,
                            modifiers,
                        },
                    ]);
                } else if command == "click" {
                    now.push(egui::Event::PointerButton {
                        pos: start,
                        button,
                        pressed: true,
                        modifiers,
                    });
                    self.later.push_back(vec![egui::Event::PointerButton {
                        pos: start,
                        button,
                        pressed: false,
                        modifiers,
                    }]);
                    if a["double"] == true {
                        self.later.push_back(vec![egui::Event::PointerButton {
                            pos: start,
                            button,
                            pressed: true,
                            modifiers,
                        }]);
                        self.later.push_back(vec![egui::Event::PointerButton {
                            pos: start,
                            button,
                            pressed: false,
                            modifiers,
                        }]);
                    }
                } else if command == "pointer" {
                    let pressed = a["pressed"].as_bool().ok_or("pressed flag required")?;
                    now.push(egui::Event::PointerButton {
                        pos: start,
                        button,
                        pressed,
                        modifiers,
                    });
                }
            }
            _ => return Err("Unknown native input command".into()),
        }
        self.now = now;
        Ok(())
    }
}

/// Stable key spelling at the native-to-Mix event boundary.
pub fn key_name(key: egui::Key) -> &'static str {
    match key {
        egui::Key::ArrowDown => "ArrowDown",
        egui::Key::ArrowUp => "ArrowUp",
        egui::Key::ArrowLeft => "ArrowLeft",
        egui::Key::ArrowRight => "ArrowRight",
        _ => key.name(),
    }
}
