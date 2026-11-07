// SPDX-License-Identifier: MIT OR Apache-2.0
//! Reusable terminal surface. Mix supplies session handles, focus requests,
//! appearance and shortcut bindings; native output stays in native storage.
use super::{
    config::Cursor,
    gpu::Pane,
    ime::Composition,
    input,
    model::{MouseModifiers, SelectionSide, SelectionType},
    raster::Raster,
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
    raster: Arc<Mutex<Raster>>,
    pane: Pane,
    look: (f32, f32, String, u16),
    focus: u64,
    wheel: f32,
    preedit: String,
    grid_size: (u16, u16, u16, u16),
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
    use super::*;

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
        for size in [14.667, 14.667, 18.0, 18.0] {
            widget.properties = json!({"font_size":size});
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
            let t = owner.lock().unwrap();
            assert!(t.snapshot().contains("native fixture"));
            assert!(t.grid_snapshot().dirty_rows.iter().all(|dirty| !dirty));
        }
    }
}

#[derive(Default)]
pub struct Views {
    surfaces: BTreeMap<String, Surface>,
    composition: Composition,
    focused: Option<String>,
}

impl Views {
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
            return self.draw(ui, widget, value, sessions, events);
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

    pub fn draw(
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
        let size = p["font_size"].as_f64().unwrap_or(14.667) as f32;
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
                    raster: Arc::new(Mutex::new(raster)),
                    pane: Pane::new(),
                    look,
                    focus: 0,
                    wheel: 0.0,
                    preedit: String::new(),
                    grid_size: (0, 0, 0, 0),
                },
            );
        }
        let surface = self.surfaces.get_mut(handle).unwrap();
        let cell = {
            let mut r = surface.raster.lock().unwrap();
            r.set_cursor(cursor);
            r.cell()
        };
        let padding = p["padding"].as_f64().unwrap_or(4.0).max(0.0) as f32;
        let available = ui.available_size().max(egui::vec2(1.0, 1.0));
        let (area, _) = ui.allocate_exact_size(available, Sense::hover());
        let id = ui.make_persistent_id((&widget.id, handle));
        let response = ui.interact(area, id, Sense::click_and_drag());
        let inner = area.shrink(padding);
        let origin = egui::pos2(
            (inner.min.x * scale).ceil() / scale,
            (inner.min.y * scale).ceil() / scale,
        );
        let cols = ((inner.max.x - origin.x) * scale / cell.0 as f32)
            .floor()
            .clamp(1.0, 240.0) as u16;
        let rows = ((inner.max.y - origin.y) * scale / cell.1 as f32)
            .floor()
            .clamp(1.0, 100.0) as u16;
        let grid = Rect::from_min_size(
            origin,
            egui::vec2(
                cols as f32 * cell.0 as f32 / scale,
                rows as f32 * cell.1 as f32 / scale,
            ),
        );
        let grid_size = (
            cols,
            rows,
            cols.saturating_mul(cell.0 as u16),
            rows.saturating_mul(cell.1 as u16),
        );
        sessions.placed(handle, area);
        if grid_size != surface.grid_size {
            terminal.resize(grid_size.0, grid_size.1, grid_size.2, grid_size.3);
            surface.grid_size = grid_size;
        }
        let serial = crate::widgets::whole(&p["focus_serial"]).unwrap_or(0);
        let focus_target = p["focused_handle"]
            .as_str()
            .is_none_or(|wanted| wanted == handle);
        if response.clicked() || (focus_target && serial > surface.focus) {
            surface.focus = serial;
            response.request_focus();
            events.push(Intent {
                id: widget.id.clone(),
                action: "focus".into(),
                value: json!(handle),
            });
        }
        let pane_id = handle
            .strip_prefix("vt-")
            .and_then(|s| s.parse::<u64>().ok());
        if !response.has_focus() && self.focused.as_deref() == Some(handle) {
            self.composition.focus(None);
            self.focused = None;
            surface.preedit.clear();
        }
        if response.has_focus() {
            self.focused = Some(handle.into());
            self.composition.focus(pane_id);
            let input_events = ui.input(|i| i.events.clone());
            for event in input_events {
                let result = match event {
                    egui::Event::Key {
                        key,
                        pressed: true,
                        modifiers,
                        ..
                    } => {
                        let command = p["shortcuts"].as_array().and_then(|a| {
                            a.iter().find(|s| {
                                s["key"].as_str() == Some(key.name())
                                    && (s["ctrl"] == true) == modifiers.ctrl
                                    && (s["shift"] == true) == modifiers.shift
                                    && (s["alt"] == true) == modifiers.alt
                            })
                        });
                        if let Some(command) = command {
                            events.push(Intent {
                                id: widget.id.clone(),
                                action: "command".into(),
                                value: command["command"].clone(),
                            });
                            Ok(())
                        } else {
                            terminal.keys(&input::keys_for(key, modifiers), Instant::now())
                        }
                    }
                    egui::Event::Text(text)
                        if !self.composition.has_owner() && !self.composition.stale() =>
                    {
                        terminal.keys(
                            &input::typed(&text, ui.input(|i| i.modifiers)),
                            Instant::now(),
                        )
                    }
                    egui::Event::Paste(text) => terminal.paste(&text),
                    egui::Event::Copy => {
                        if let Some(text) = terminal.selection_text() {
                            ui.ctx().copy_text(text);
                        }
                        Ok(())
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
        if response.hovered() || response.dragged() {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                let relative = pos - grid.min;
                let (col, row) = input::pointer_cell(
                    egui::pos2(relative.x, relative.y),
                    0.0,
                    (cell.0 as f32 / scale, cell.1 as f32 / scale),
                    (cols, rows),
                );
                let modifiers = ui.input(|i| i.modifiers);
                let mods = MouseModifiers {
                    shift: modifiers.shift,
                    alt: modifiers.alt,
                    ctrl: modifiers.ctrl,
                };
                let local = modifiers.shift || !terminal.mouse_reporting();
                if response.drag_started() || response.clicked() {
                    if local {
                        let ty = if response.triple_clicked() {
                            SelectionType::Lines
                        } else if response.double_clicked() {
                            SelectionType::Semantic
                        } else if modifiers.ctrl {
                            SelectionType::Block
                        } else {
                            SelectionType::Simple
                        };
                        terminal.selection_start(col, row, SelectionSide::Left, ty);
                    } else {
                        terminal.mouse_button(col, row, 0, true, mods);
                    }
                }
                if response.dragged() {
                    if local {
                        terminal.selection_update(col, row, SelectionSide::Right);
                    } else {
                        terminal.mouse_motion(col, row, 0, mods);
                    }
                }
                if response.drag_stopped() || response.clicked() {
                    if local {
                        terminal.selection_finish();
                    } else {
                        terminal.mouse_button(col, row, 0, false, mods);
                    }
                }
                for event in ui.input(|i| i.events.clone()) {
                    if let egui::Event::MouseWheel { unit, delta, .. } = event {
                        let lines = input::scroll_steps(
                            &mut surface.wheel,
                            input::Wheel::of(unit, delta),
                            cell.1 as f32 / scale,
                            p["scroll_speed"].as_f64().unwrap_or(40.0) as f32,
                        );
                        if !terminal.mouse_scroll(col, row, lines, mods) {
                            terminal.scroll_wheel(lines, mods);
                        }
                    }
                }
            }
        }
        let mut snapshot = terminal.grid_snapshot();
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
        ui.painter().rect_filled(area, 0.0, Color32::BLACK);
        surface.pane.update(snapshot.screen, snapshot.dirty_rows);
        ui.painter()
            .with_clip_rect(inner)
            .add(surface.pane.callback(grid, surface.raster.clone()));
        if !surface.preedit.is_empty() && response.has_focus() {
            ui.painter().text(
                caret.min,
                egui::Align2::LEFT_TOP,
                &surface.preedit,
                egui::FontId::monospace(size),
                Color32::WHITE,
            );
        }
        response
    }
}
