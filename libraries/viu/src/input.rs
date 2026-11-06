// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native egui input automation; all targets come from the current SceneView
//! geometry, and modifiers travel as native input events as well as payloads.
use crate::view::SceneView;
use serde_json::Value;
use std::collections::VecDeque;
#[derive(Default)]
pub struct Input {
    pub now: Vec<egui::Event>,
    pub later: VecDeque<Vec<egui::Event>>,
    menu_path: VecDeque<String>,
    menu_wait_frames: u8,
    menu_depth: usize,
}
fn finite_point(x: f64, y: f64) -> Result<egui::Pos2, String> {
    let point = egui::pos2(x as f32, y as f32);
    if !point.is_finite() {
        return Err("Coordinates must fit finite native points".into());
    }
    Ok(point)
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
            "menu.open" => {
                let targets =
                    crate::rich::menu_targets(&view.model()["values"]["menu"], "menu", a)?;
                let mut targets: VecDeque<_> = targets.into();
                let depth = targets.len();
                if view.geometry.contains_key("menu.toggle") {
                    targets.push_front("menu.toggle".into());
                }
                let first = targets.pop_front().ok_or("menu path is empty")?;
                if view.geometry.contains_key(&first) {self.queue("click", &serde_json::json!({"id":first}), view)?;}
                else {targets.push_front(first);}
                self.menu_path = targets;
                self.menu_wait_frames = 0;
                self.menu_depth = depth;
                return Ok(());
            }
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
                    finite_point(x, y)?
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
                        finite_point(
                            a["to_x"].as_f64().ok_or("to or to_x/to_y required")?,
                            a["to_y"].as_f64().ok_or("to_y required")?,
                        )?
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
    /// Resolve each submenu against the geometry drawn after the preceding
    /// native click. No command intent is synthesised by this driver.
    pub fn advance_menu(&mut self, view: &mut SceneView) -> Option<Result<(), String>> {
        if !self.now.is_empty() || !self.later.is_empty() {
            return None;
        }
        let Some(target) = self.menu_path.front().cloned() else {
            if self.menu_depth == 0 {
                return None;
            }
            let snapshot =
                crate::rich::menu_snapshot(&view.model()["values"]["menu"], "menu", &view.geometry);
            if snapshot["path"]
                .as_array()
                .is_some_and(|path| path.len() == self.menu_depth)
            {
                self.menu_depth = 0;
                return None;
            }
            self.menu_wait_frames += 1;
            if self.menu_wait_frames <= 3 {
                return Some(Ok(()));
            }
            self.menu_depth = 0;
            return Some(Err(
                "Native menu did not produce visible popup geometry".into()
            ));
        };
        if !view.geometry.contains_key(&target) && self.menu_wait_frames < 3 {
            self.menu_wait_frames += 1;
            return Some(Ok(()));
        }
        self.menu_wait_frames = 0;
        self.menu_path.pop_front();
        let result = self.queue("click", &serde_json::json!({"id":target}), view);
        if result.is_err() {
            self.menu_path.clear();
            self.menu_depth = 0;
        }
        Some(result)
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn malformed_gestures_do_not_queue_partial_native_input() {
        let mut view=SceneView::new("---\nscene: 1\nname: input\ncitizen: input\n---\n```mix\n{root:{widget:\"text\",text:\"Input\"}}\n```\n").unwrap();
        let mut input = Input::default();
        for (command, args) in [
            ("click", json!({"x":1e100,"y":0})),
            ("drag", json!({"x":1,"y":2,"to_x":1e100,"to_y":0})),
            ("drag", json!({"x":1,"y":2,"to_x":0})),
            ("click", json!({"id":"missing"})),
        ] {
            assert!(input.queue(command, &args, &mut view).is_err());
            assert!(input.now.is_empty() && input.later.is_empty());
        }
        input
            .queue("drag", &json!({"x":1,"y":2,"to_x":3,"to_y":4}), &mut view)
            .unwrap();
        assert_eq!(input.later.len(), 3);
        for key in egui::Key::ALL {
            assert_eq!(egui::Key::from_name(key_name(*key)), Some(*key));
        }
    }
}
