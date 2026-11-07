// SPDX-License-Identifier: MIT OR Apache-2.0
//! In-process editor documents. File and editd authority belong to the Mix
//! owner; this adapter never opens, saves or sends a Bus message.
use std::collections::BTreeMap;

use egui::{Color32, Response, TextStyle, Ui};
use serde_json::Value;

use crate::widgets::{Intent, Widget};

struct Document {
    local: editor::local::Document,
    acknowledged: String,
    find: String,
    replacement: String,
    searching: bool,
    command_serial: u64,
}

#[derive(Default)]
pub struct Editors {
    documents: BTreeMap<(String, String), Document>,
}

struct Theme<'a>(&'a Ui);
impl editor::palette::Theme for Theme<'_> {
    fn editor_colour(&self, role: &str) -> Color32 {
        crate::appearance::colour(self.0.ctx(), role).unwrap_or_else(|| {
            let v = self.0.visuals();
            match role {
                "field" => v.extreme_bg_color,
                "text_dim" | "text_faint" | "palette.foreground.muted" => v.weak_text_color(),
                "accent" | "palette.accent.hover" => v.selection.bg_fill,
                "selection" => v.selection.bg_fill,
                "hover" => v.widgets.hovered.bg_fill,
                "separator" => v.widgets.noninteractive.bg_stroke.color,
                "warning" => v.warn_fg_color,
                "danger" => v.error_fg_color,
                _ => v.text_color(),
            }
        })
    }
}

impl Editors {
    pub fn draw(
        &mut self,
        ui: &mut Ui,
        widget: &Widget,
        value: &Value,
        selection: Option<[usize; 2]>,
        events: &mut Vec<Intent>,
    ) -> Response {
        let text = value.as_str().unwrap_or_default();
        let key = (
            widget.id.clone(),
            widget.properties["document"]
                .as_str()
                .unwrap_or(&widget.id)
                .to_owned(),
        );
        let language = widget.properties["language"].as_str().unwrap_or("text");
        if !self.documents.contains_key(&key) {
            match editor::local::Document::new(text, language) {
                Ok(local) => {
                    self.documents.insert(
                        key.clone(),
                        Document {
                            local,
                            acknowledged: text.to_owned(),
                            find: String::new(),
                            replacement: String::new(),
                            searching: false,
                            command_serial: 0,
                        },
                    );
                }
                Err(e) => return ui.colored_label(Color32::RED, e.to_string()),
            }
        }
        let doc = self.documents.get_mut(&key).unwrap();
        // A Mix echo acknowledges typing without reconstructing the model or
        // destroying undo, selection, scroll and composition. A different
        // authoritative value is an explicit reload, never a transport write.
        if text != doc.acknowledged {
            if text != doc.local.contents() {
                if let Err(e) = doc.local.set_contents(text) {
                    return ui.colored_label(Color32::RED, e.to_string());
                }
            }
            doc.acknowledged = text.to_owned();
        }
        if let Some([start, end]) = selection {
            // Native focus uses scalar offsets, the editor uses UTF-8 bytes.
            let byte = |n| text.char_indices().nth(n).map_or(text.len(), |(i, _)| i);
            doc.local.model_mut().sel = editor::Selection {
                anchor: byte(start),
                head: byte(end),
            };
        }
        let mut view = native_view(ui, widget);
        let before = doc.local.doc().revision;
        if let Some(serial) = crate::widgets::whole(&widget.properties["command"]["serial"]) {
            if serial > doc.command_serial && !view.read_only {
                doc.command_serial = serial;
                let name = widget.properties["command"]["name"].as_str().unwrap_or("");
                if name=="paste" {
                    ui.ctx().send_viewport_cmd(egui::ViewportCommand::RequestPaste);
                } else if matches!(name,"cut"|"copy") {
                    let selection=doc.local.selection();
                    let text=doc.local.contents();
                    let range=selection.anchor.min(selection.head)..selection.anchor.max(selection.head);
                    if let Some(text)=text.get(range).filter(|s|!s.is_empty()) {ui.ctx().copy_text(text.to_owned());}
                }
                if !view.read_only {
                    let command = match name {
                        "cut" => Some(editor::EditCommand::Delete),
                        "select-all" => Some(editor::EditCommand::SelectAll),
                        "duplicate-line" => Some(editor::EditCommand::DuplicateLine),
                        "delete-line" => Some(editor::EditCommand::DeleteLine),
                        "move-line-up" => Some(editor::EditCommand::MoveLineUp),
                        "move-line-down" => Some(editor::EditCommand::MoveLineDown),
                        "toggle-comment" => Some(editor::EditCommand::ToggleComment),
                        "indent" => Some(editor::EditCommand::Tab),
                        "outdent" => Some(editor::EditCommand::Outdent),
                        _ => None,
                    };
                    let result = if let Some(command) = command {
                        doc.local.command(command, &view)
                    } else if name == "undo" {
                        doc.local.undo()
                    } else if name == "redo" {
                        doc.local.redo()
                    } else {
                        Ok(())
                    };
                    if let Err(e) = result {
                        ui.colored_label(Color32::RED, e.to_string());
                    }
                }
            }
        }
        let editor_id = ui.make_persistent_id(&key);
        if ui.memory(|m| m.has_focus(editor_id))
            && ui.input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::F))
        {
            doc.searching = true;
        }
        if widget.properties["find_bar"] == true || doc.searching {
            ui.horizontal(|ui| {
                ui.label("Find");
                ui.text_edit_singleline(&mut doc.find);
                if ui.button("Next").clicked() && !doc.find.is_empty() {
                    let text = doc.local.contents();
                    let from = doc.local.selection().head;
                    let found = text[from..]
                        .find(&doc.find)
                        .map(|i| i + from)
                        .or_else(|| text[..from].find(&doc.find));
                    if let Some(start) = found {
                        doc.local.model_mut().sel = editor::Selection {
                            anchor: start,
                            head: start + doc.find.len(),
                        };
                    }
                }
                if ui.button("×").clicked() {
                    doc.searching = false;
                }
            });
            if !view.read_only {
                ui.horizontal(|ui| {
                    ui.label("Replace");
                    ui.text_edit_singleline(&mut doc.replacement);
                    if ui.button("Replace").clicked() && !doc.find.is_empty() {
                        let sel = doc.local.selection();
                        let text = doc.local.contents();
                        let range = sel.anchor.min(sel.head)..sel.anchor.max(sel.head);
                        if text.get(range) == Some(doc.find.as_str()) {
                            if let Err(e) = doc.local.command(
                                editor::EditCommand::Insert(doc.replacement.clone()),
                                &view,
                            ) {
                                ui.colored_label(Color32::RED, e.to_string());
                            }
                        }
                    }
                });
            }
            if !doc.find.is_empty() {
                view.matches = doc
                    .local
                    .contents()
                    .match_indices(&doc.find)
                    .map(|(i, s)| i..i + s.len())
                    .collect();
            }
        }
        let palette = native_palette(ui, widget);
        let padding = widget.properties["padding"].as_f64().unwrap_or(0.0).round() as i8;
        let out = egui::Frame::new()
            .fill(palette.background)
            .inner_margin(egui::Margin::same(padding))
            .show(ui, |ui| doc.local.show_as(ui, &key, &palette, &view))
            .inner;
        if let Some(error) = doc.local.take_refusal() {
            ui.colored_label(Color32::RED, error.to_string());
        }
        if before != doc.local.doc().revision {
            let contents = doc.local.contents();
            // Hold the draft locally until Mix acknowledges the new value.
            doc.acknowledged = text.to_owned();
            events.push(Intent::set(&widget.id, Value::String(contents)));
        }
        out.response
    }
}

pub(crate) fn native_view(ui: &Ui, widget: &Widget) -> editor::View {
    let view = editor::View {
        font_size: Some(
            widget.properties["font_size"]
                .as_f64()
                .map(|v| v as f32)
                .unwrap_or_else(|| {
                    if widget.properties["prose"] == true {
                        TextStyle::Body.resolve(ui.style()).size
                    } else {
                        TextStyle::Monospace.resolve(ui.style()).size
                    }
                }),
        ),
        wrap: if widget.properties["soft_wrap"] == true {
            editor::Wrap::Words
        } else {
            editor::Wrap::None
        },
        line_numbers: widget.properties["line_numbers"] != false,
        whitespace: widget.properties["whitespace"] == true,
        remote_carets: widget.properties["remote_carets"] != false,
        read_only: widget.properties["read_only"] == true,
        current_line: widget.properties["prose"] != true,
        label: Some(widget.label.clone()),
        ..Default::default()
    };
    view
}

pub(crate) fn native_palette(ui: &Ui, widget: &Widget) -> editor::Palette {
    if widget.properties["light"] == true {
        return editor::Palette::from_theme(toolkit::content::light_theme());
    }
    if let Some(palette) = ui
        .ctx()
        .data(|d| d.get_temp::<editor::Palette>(egui::Id::unique("viu-gold-editor-palette")))
    {
        return palette;
    }

    let palette = if widget.properties["light"] == true {
        static LIGHT: std::sync::OnceLock<editor::Palette> = std::sync::OnceLock::new();
        LIGHT
            .get_or_init(|| {
                let resolved = crate::appearance::resolve(
                    &serde_json::json!({"scheme":"mixos","mode":"light"}),
                )
                .expect("embedded light design");
                struct Light(std::collections::BTreeMap<String, Color32>);
                impl editor::palette::Theme for Light {
                    fn editor_colour(&self, role: &str) -> Color32 {
                        self.0.get(role).copied().unwrap_or(Color32::BLACK)
                    }
                }
                editor::Palette::from_theme(&Light(resolved.colours))
            })
            .clone()
    } else {
        editor::Palette::from_theme(&Theme(ui))
    };
    palette
}
