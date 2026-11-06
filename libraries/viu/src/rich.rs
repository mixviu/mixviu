// SPDX-License-Identifier: MIT OR Apache-2.0
//! Application-supplied menus, command bars, property fields and modal forms.
//! Command identity, availability and form outcomes remain application data.
use crate::widgets::{Intent, Widget};
use egui::{Rect, Ui};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

pub const KINDS: &[&str] = &[
    "menubar",
    "command-bar",
    "property-grid",
    "dialog-form",
    "settings-browser",
];
#[derive(Clone, Default)]
struct MenuPresentation {
    settings: Value,
    service: Option<String>,
    popup: Option<(f32, bool)>,
    hosted_menu: bool,
    hosted_actions: bool,
    command_popup: Option<String>,
}
fn presentation_id(ctx: &egui::Context) -> egui::Id {
    egui::Id::unique((ctx.viewport_id(), "viu-menu-presentation"))
}
fn presentation(ctx: &egui::Context) -> MenuPresentation {
    let key = presentation_id(ctx);
    ctx.data(|data| data.get_temp(key)).unwrap_or_default()
}
/// Native window/menu policy is shared by scene and direct Rust callers.
/// A compositor-hosted menu is never claimed before a host adapter exists.
pub fn configure_menu(ctx: &egui::Context, settings: &Value, service: Option<&str>) {
    let key = presentation_id(ctx);
    ctx.data_mut(|data| {
        let state = data.get_temp_mut_or_default::<MenuPresentation>(key);
        state.settings = settings.clone();
        state.service = service.map(str::to_owned);
    });
}
/// Host acknowledgement is supplied by the owning native client, never
/// inferred from desktop branding alone. A foreign/unavailable host retains
/// the ordinary controls in the client.
pub fn configure_hosting(ctx: &egui::Context, menu: bool, actions: bool) {
    let key = presentation_id(ctx);
    ctx.data_mut(|d| {
        let p = d.get_temp_mut_or_default::<MenuPresentation>(key);
        p.hosted_menu = menu;
        p.hosted_actions = actions;
    });
}
pub fn command_popup(ctx: &egui::Context, id: &str) {
    let key = presentation_id(ctx);
    ctx.data_mut(|d| {
        d.get_temp_mut_or_default::<MenuPresentation>(key)
            .command_popup = Some(id.into())
    });
    egui::Popup::open_id(
        ctx,
        egui::Id::unique((ctx.viewport_id(), "viu-hosted-command-popup")),
    );
    ctx.request_repaint();
}
pub fn menu_form(ctx: &egui::Context) -> Value {
    let state = presentation(ctx);
    let style = design::MenuStyle::resolve(
        state.settings["window.menu_style"]
            .as_str()
            .unwrap_or("bar"),
    );
    let side = design::MenuSide::resolve(
        state.settings["window.menu_side"]
            .as_str()
            .unwrap_or("left"),
    );
    let effective = if state.hosted_menu {
        "titlebar"
    } else if style == design::MenuStyle::Bar && ctx.content_rect().width() >= 640.0 {
        "bar"
    } else {
        "button"
    };
    json!({"style":style.name(),"side":side.name(),"effective":effective,"hosted":state.hosted_menu,"service":state.service})
}
/// Open the complete vertical menu at the top of this viewport. The normal
/// popup memory owns dismissal and Escape; no alternate input dispatcher.
pub fn menu_popup(ctx: &egui::Context, id: &str, args: &Value) -> Result<(), String> {
    if !args.is_object() {
        return Err("arguments must be an object".into());
    }
    let state = presentation(ctx);
    let side = match args.get("side") {
        None => design::MenuSide::resolve(
            state.settings["window.menu_side"]
                .as_str()
                .unwrap_or("left"),
        ),
        Some(value) => value
            .as_str()
            .and_then(design::MenuSide::from_name)
            .ok_or("side must be left, center or right")?,
    };
    let bounds = ctx.content_rect();
    let x = match args.get("x") {
        None => match side {
            design::MenuSide::Left => bounds.left(),
            design::MenuSide::Center => bounds.center().x,
            design::MenuSide::Right => bounds.right(),
        },
        Some(value) => value
            .as_f64()
            .filter(|x| x.is_finite() && (*x as f32).is_finite())
            .ok_or("x must be finite")? as f32,
    }
    .clamp(bounds.left(), bounds.right());
    let key = presentation_id(ctx);
    ctx.data_mut(|data| {
        data.get_temp_mut_or_default::<MenuPresentation>(key).popup =
            Some((x, side == design::MenuSide::Right))
    });
    egui::Popup::open_id(
        ctx,
        egui::Id::unique((ctx.viewport_id(), id, "vertical-menu")),
    );
    ctx.request_repaint();
    Ok(())
}
pub fn actions(kind: &str) -> Vec<&'static str> {
    match kind {
        "menubar" | "command-bar" => vec!["set", "command"],
        "dialog-form" => vec!["set", "edit", "answer"],
        "settings-browser" => vec![
            "set",
            "navigate",
            "query",
            "edit",
            "activate",
            "remove",
            "problem",
            "drawer.close",
        ],
        _ => vec!["set"],
    }
}
fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}
fn entries(value: &Value, depth: usize, ids: &mut BTreeSet<String>) -> Result<(), String> {
    if depth > 8 {
        return Err("menu nesting exceeds eight levels".into());
    }
    let values = value.as_array().ok_or("command items must be an array")?;
    for entry in values {
        if entry["separator"] == true {
            continue;
        }
        if text(entry, "id").is_empty()
            || !ids.insert(text(entry, "id").into())
            || !entry["label"].is_string()
            || !entry["enabled"].is_boolean()
        {
            return Err("command items require unique ID, label and enabled flag".into());
        }
        if let Some(icon) = entry["icon"].as_str() {
            if crate::icons::Icon::from_name(icon).is_none() {
                return Err("unknown command icon".into());
            }
        }
        if entry.get("children").is_some() {
            entries(&entry["children"], depth + 1, ids)?;
        }
    }
    Ok(())
}
pub fn validate(kind: &str, value: &Value) -> Result<(), String> {
    match kind {
        "menubar" | "command-bar" => entries(value, 0, &mut BTreeSet::new()),
        "property-grid" => {
            let fields = value.as_array().ok_or("property fields must be an array")?;
            if fields
                .iter()
                .any(|f| !f["label"].is_string() || !f["value"].is_string())
            {
                Err("property fields require label and value strings".into())
            } else {
                Ok(())
            }
        }
        "dialog-form" => {
            if !value["open"].is_boolean()
                || !value["title"].is_string()
                || !value["body"].is_string()
            {
                return Err("modal form requires open, title and body".into());
            }
            if value
                .get("field")
                .is_some_and(|f| !f.is_null() && !f.is_string())
            {
                return Err("modal form field must be string or null".into());
            }
            if value
                .get("max_field_bytes")
                .is_some_and(|v| v.as_u64().is_none_or(|n| n == 0 || n > 1_048_576))
                || value.get("multiline").is_some_and(|v| !v.is_boolean())
                || value
                    .get("rows")
                    .is_some_and(|v| v.as_u64().is_none_or(|n| !(1..=40).contains(&n)))
                || value.get("width").is_some_and(|v| {
                    v.as_f64()
                        .is_none_or(|n| !n.is_finite() || !(320.0..=1200.0).contains(&n))
                })
            {
                return Err("invalid bounded form editor options".into());
            }
            entries(&value["choices"], 0, &mut BTreeSet::new())
        }
        "settings-browser" => crate::settings_browser::validate(value),
        _ => Err("unknown rich control".into()),
    }
}
fn find<'a>(value: &'a Value, id: &str) -> Option<&'a Value> {
    for item in value.as_array().into_iter().flatten() {
        if item["enabled"] != true {
            continue;
        }
        if text(item, "id") == id {
            return Some(item);
        }
        if let Some(found) = find(&item["children"], id) {
            return Some(found);
        }
    }
    None
}
/// Resolve an application-owned menu path to native geometry identities.
/// Both labels and stable IDs are accepted; disabled/leaf/ambiguous paths
/// refuse before any input is queued.
pub fn menu_targets(value: &Value, id: &str, args: &Value) -> Result<Vec<String>, String> {
    validate("menubar", value)?;
    let mut names = Vec::new();
    if let Some(path) = args.get("path") {
        for entry in path.as_array().ok_or("menu path must be an array")? {
            names.push(entry.as_str().ok_or("menu path entries must be strings")?);
        }
    } else {
        names.push(args["menu"].as_str().ok_or("menu label or ID required")?);
    }
    if names.is_empty() || names.len() > 8 {
        return Err("menu path exceeds eight levels".into());
    }
    let mut level = value;
    let mut targets = Vec::new();
    for (depth, name) in names.into_iter().enumerate() {
        let matches: Vec<_> = level
            .as_array()
            .into_iter()
            .flatten()
            .filter(|item| text(item, "id") == name || text(item, "label") == name)
            .collect();
        if matches.len() != 1 || matches[0]["enabled"] != true || !matches[0]["children"].is_array()
        {
            return Err("unknown, disabled, ambiguous or leaf menu path".into());
        }
        let item = matches[0];
        targets.push(format!(
            "{id}.{}.{}",
            if depth == 0 { "menu" } else { "command" },
            text(item, "id")
        ));
        level = &item["children"];
    }
    Ok(targets)
}

pub fn menu_snapshot(value: &Value, id: &str, geometry: &BTreeMap<String, Rect>) -> Value {
    fn descend(value: &Value, id: &str, geometry: &BTreeMap<String, Rect>, path: &mut Vec<String>) {
        for item in value.as_array().into_iter().flatten() {
            if let Some(children) = item["children"].as_array() {
                if children.iter().any(|child| {
                    geometry.contains_key(&format!("{id}.command.{}", text(child, "id")))
                }) {
                    path.push(text(item, "label").into());
                    descend(&item["children"], id, geometry, path);
                    break;
                }
            }
        }
    }
    let mut path = Vec::new();
    descend(value, id, geometry, &mut path);
    json!({"open":!path.is_empty() || geometry.contains_key(&format!("{id}.popup")),"path":path})
}
pub fn admit(kind: &str, current: &Value, action: &str, payload: &Value) -> Result<(), String> {
    if kind == "settings-browser" {
        return crate::settings_browser::admit(current, action, payload);
    }
    validate(kind, current)?;
    match action {
        "command" => {
            if find(current, text(payload, "command"))
                .is_none_or(|item| item["enabled"] != true || item.get("children").is_some())
            {
                return Err("disabled or unknown command".into());
            }
        }
        "answer" => {
            if current["open"] != true
                || current
                    .get("instance")
                    .is_some_and(|instance| payload.get("instance") != Some(instance))
                || find(&current["choices"], text(payload, "choice"))
                    .is_none_or(|item| item["enabled"] != true)
            {
                return Err("disabled or unknown form choice".into());
            }
        }
        "edit" => {
            if current["open"] != true
                || !current["field"].is_string()
                || payload.as_str().is_none_or(|s| {
                    s.len() > current["max_field_bytes"].as_u64().unwrap_or(4096) as usize
                })
            {
                return Err("form is not editable".into());
            }
        }
        _ => {}
    }
    Ok(())
}
fn emit(events: &mut Vec<Intent>, id: &str, action: &str, value: Value) {
    events.push(Intent {
        id: id.into(),
        action: action.into(),
        value,
    });
}
fn label(item: &Value) -> egui::RichText {
    egui::RichText::new(format!(
        "{}{}",
        if item["checked"] == true { "✓  " } else { "" },
        text(item, "label")
    ))
}
fn menu(
    ui: &mut Ui,
    value: &Value,
    id: &str,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) {
    for item in value.as_array().into_iter().flatten() {
        if item["separator"] == true {
            ui.separator();
            continue;
        }
        if item.get("children").is_some() {
            let shown = ui.add_enabled_ui(item["enabled"] == true, |ui| {
                ui.menu_button(label(item), |ui| {
                    menu(ui, &item["children"], id, geometry, events)
                })
            });
            if ui.is_visible() && !ui.is_sizing_pass() {
                geometry.insert(
                    format!("{id}.command.{}", text(item, "id")),
                    shown.inner.response.rect,
                );
            }
        } else {
            let mut button = if let Some(icon) = item["icon"]
                .as_str()
                .and_then(crate::icons::Icon::from_name)
            {
                egui::Button::image_and_text(
                    crate::icons::image_of(ui.ctx(), icon, 16.0, ui.visuals().text_color()),
                    label(item),
                )
            } else {
                egui::Button::new(label(item))
            };
            if let Some(shortcut) = item["shortcut"].as_str() {
                button = button.shortcut_text(shortcut);
            }
            let response = ui.add_enabled(item["enabled"] == true, button);
            if ui.is_visible() && !ui.is_sizing_pass() {
                geometry.insert(format!("{id}.command.{}", text(item, "id")), response.rect);
            }
            if response.clicked() {
                emit(events, id, "command", json!({"command":item["id"]}));
                ui.close();
            }
        }
    }
}
fn menu_roots(
    ui: &mut Ui,
    value: &Value,
    id: &str,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) {
    for item in value.as_array().into_iter().flatten() {
        let shown = ui.add_enabled_ui(item["enabled"] == true, |ui| {
            ui.menu_button(text(item, "label"), |ui| {
                menu(ui, &item["children"], id, geometry, events)
            })
        });
        if ui.is_visible() && !ui.is_sizing_pass() {
            geometry.insert(
                format!("{id}.menu.{}", text(item, "id")),
                shown.inner.response.rect,
            );
        }
    }
}
fn native_entries(value: &Value) -> Vec<toolkit::menu::Entry> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .map(|item| {
            if item["separator"] == true {
                return toolkit::menu::Entry::Separator;
            }
            let children = native_entries(&item["children"]);
            toolkit::menu::Entry::Row(toolkit::menu::Row {
                id: if item["children"].is_array() {
                    None
                } else {
                    Some(text(item, "id").to_owned().into())
                },
                label: text(item, "label").into(),
                shortcut: item["shortcut"].as_str().map(str::to_owned),
                enabled: item["enabled"] == true,
                checked: item["checked"].as_bool(),
                children,
            })
        })
        .collect()
}

fn native_menu(ui: &mut Ui, value: &Value, id: &str, events: &mut Vec<Intent>) -> egui::Response {
    let menus: Vec<_> = value
        .as_array()
        .into_iter()
        .flatten()
        .map(|item| toolkit::menu::Menu {
            title: text(item, "label").into(),
            entries: native_entries(&item["children"]),
        })
        .collect();
    let search = menus
        .iter()
        .position(|m| m.title == "Help")
        .map(toolkit::menu::Search::new);
    ui.allocate_ui_with_layout(
        ui.available_size_before_wrap(),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            let form = menu_form(ui.ctx());
            let picked = if form["effective"] == "bar" {
                toolkit::menu::data_picked_with_origin(ui, &menus, search.as_ref(), None)
            } else {
                let response = ui.add(
                    toolkit::button::IconButton::new(toolkit::Icon::Menu).tooltip("Window menu"),
                );
                if (response.hovered() && ui.input(|i| i.pointer.primary_pressed()))
                    || (response.clicked() && !response.clicked_by(egui::PointerButton::Primary))
                {
                    toolkit::menu::toggle(ui.ctx());
                }
                toolkit::menu::data_picked_with_origin(
                    ui,
                    &menus,
                    search.as_ref(),
                    Some(response.rect),
                )
            };
            if let Some((command, palette)) = picked {
                emit(
                    events,
                    id,
                    "command",
                    json!({"command":command,"origin":if palette {"palette"}else{"menu"}}),
                );
            }
        },
    )
    .response
}

pub(crate) fn draw(
    ui: &mut Ui,
    widget: &Widget,
    value: &Value,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) -> egui::Response {
    let id = &widget.id;
    match widget.kind.as_str() {
        "settings-browser" => crate::settings_browser::draw(ui, id, value, geometry, events),
        "menubar" => {
            if widget.properties["native_menu"] == true {
                return native_menu(ui, value, id, events);
            }

            let form = menu_form(ui.ctx());
            let response = if form["hosted"] == true {
                ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
            } else if widget.properties["style"] == "gold" {
                let response = crate::native_menu::draw(ui, value, form["effective"] != "bar");
                if let Some(command) = crate::native_menu::picked(ui.ctx()) {
                    emit(events, id, "command", command);
                }
                response
            } else if form["effective"] == "bar" {
                egui::MenuBar::new()
                    .ui(ui, |ui| menu_roots(ui, value, id, geometry, events))
                    .response
            } else {
                let response = crate::icons::button(
                    ui,
                    crate::icons::Icon::from_name("menu").unwrap(),
                    "",
                    16.0,
                    None,
                    true,
                    false,
                )
                .on_hover_text("Window menu");
                geometry.insert(format!("{id}.toggle"), response.rect);
                if let Some(shown) = egui::Popup::menu(&response).show(|ui| {
                    menu_roots(ui, value, id, geometry, events);
                    ui.is_visible() && !ui.is_sizing_pass()
                }) {
                    if shown.inner {
                        geometry.insert(format!("{id}.popup"), shown.response.rect);
                    }
                }
                response
            };
            let popup = presentation(ui.ctx()).popup;
            if let Some((x, right)) = popup {
                let popup_id = egui::Id::unique((ui.ctx().viewport_id(), id, "vertical-menu"));
                if let Some(shown) = egui::Popup::new(
                    popup_id,
                    ui.ctx().clone(),
                    egui::Rect::from_min_max(
                        egui::pos2(x, ui.ctx().content_rect().top()),
                        egui::pos2(x, ui.ctx().content_rect().top()),
                    ),
                    ui.layer_id(),
                )
                .kind(egui::PopupKind::Menu)
                .layout(egui::Layout::top_down_justified(egui::Align::Min))
                .align(if right {
                    egui::RectAlign::BOTTOM_END
                } else {
                    egui::RectAlign::BOTTOM_START
                })
                .open_memory(None)
                .show(|ui| {
                    menu_roots(ui, value, id, geometry, events);
                    ui.is_visible() && !ui.is_sizing_pass()
                }) {
                    if shown.inner {
                        geometry.insert(format!("{id}.popup"), shown.response.rect);
                    }
                }
            }
            response
        }
        "command-bar" => {
            let state = presentation(ui.ctx());
            let response = if state.hosted_actions {
                ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
            } else {
                ui.allocate_ui_with_layout(
                    ui.available_size_before_wrap(),
                    egui::Layout::left_to_right(egui::Align::Center),
                    |ui| {
                        if widget.properties["style"] == "gold" {
                            ui.spacing_mut().item_spacing.x = toolkit::chrome::Chrome::of(ui.ctx())
                                .style
                                .title_control_spacing
                                as f32;
                            ui.add_space(
                                toolkit::chrome::Chrome::of(ui.ctx())
                                    .style
                                    .title_control_spacing as f32,
                            );
                        }
                        for item in value.as_array().into_iter().flatten() {
                            if item["separator"] == true {
                                ui.separator();
                                continue;
                            }
                            let response = if widget.properties["style"] == "gold"
                                && item["icon"].as_str().is_some()
                            {
                                let icon =
                                    crate::icons::Icon::from_name(item["icon"].as_str().unwrap())
                                        .expect("validated icon");
                                ui.add_enabled(
                                    item["enabled"] == true,
                                    toolkit::button::IconButton::new(icon)
                                        .selected(item["checked"] == true)
                                        .tooltip(
                                            item["shortcut"]
                                                .as_str()
                                                .and_then(toolkit::keymap::parse_chord)
                                                .map_or_else(
                                                    || text(item, "label").to_owned(),
                                                    |s| {
                                                        format!(
                                                            "{} ({})",
                                                            text(item, "label"),
                                                            ui.ctx().format_shortcut(&s)
                                                        )
                                                    },
                                                ),
                                        ),
                                )
                            } else if let Some(icon) = item["icon"]
                                .as_str()
                                .and_then(crate::icons::Icon::from_name)
                            {
                                crate::icons::button(
                                    ui,
                                    icon,
                                    if item["icon_only"] == true {
                                        ""
                                    } else {
                                        text(item, "label")
                                    },
                                    16.0,
                                    None,
                                    item["enabled"] == true,
                                    item["checked"] == true,
                                )
                            } else {
                                ui.add_enabled(
                                    item["enabled"] == true,
                                    egui::Button::new(label(item)),
                                )
                            };
                            let response = response.on_hover_text(text(item, "label"));
                            geometry.insert(
                                format!("{id}.command.{}", text(item, "id")),
                                response.rect,
                            );
                            if item.get("children").is_some() {
                                egui::Popup::menu(&response)
                                    .show(|ui| menu(ui, &item["children"], id, geometry, events));
                            } else if response.clicked() {
                                emit(events, id, "command", json!({"command":item["id"]}));
                            }
                        }
                    },
                )
                .response
            };
            if let Some(target) = state.command_popup.as_deref().and_then(|id| {
                value.as_array()?.iter().find(|item| {
                    text(item, "id") == id && item["enabled"] == true && item["children"].is_array()
                })
            }) {
                let bounds = ui.ctx().content_rect();
                egui::Popup::new(
                    egui::Id::unique((ui.ctx().viewport_id(), "viu-hosted-command-popup")),
                    ui.ctx().clone(),
                    egui::Rect::from_min_size(bounds.right_top(), egui::Vec2::ZERO),
                    ui.layer_id(),
                )
                .kind(egui::PopupKind::Menu)
                .align(egui::RectAlign::BOTTOM_END)
                .open_memory(None)
                .show(|ui| menu(ui, &target["children"], id, geometry, events));
            }
            response
        }
        "property-grid" => {
            ui.vertical(|ui| {
                egui::ScrollArea::vertical()
                    .id_salt((id, "fields"))
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        for field in value.as_array().into_iter().flatten() {
                            ui.label(egui::RichText::new(text(field, "label")).small().weak());
                            ui.add(egui::Label::new(text(field, "value")).wrap());
                            ui.add_space(4.0);
                        }
                    });
            })
            .response
        }
        "dialog-form" => {
            if widget.properties["style"] == "gold" {
                let dialog_id = egui::Id::unique((ui.ctx().viewport_id(), id, "gold-dialog"));
                let choices: Vec<_> = value["choices"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|c| {
                        toolkit::dialog::Choice::new(
                            text(c, "label"),
                            if c["default"] == true {
                                toolkit::dialog::Role::Default
                            } else if c["cancel"] == true {
                                toolkit::dialog::Role::Cancel
                            } else {
                                toolkit::dialog::Role::Other
                            },
                        )
                        .enabled(c["enabled"] == true)
                    })
                    .collect();
                let shown = toolkit::dialog::Dialog::new(dialog_id, text(value, "title"))
                    .buttons(choices)
                    .show(ui.ctx(), |ui| {
                        if let Some(rows) = value["grid"].as_array() {
                            egui::ScrollArea::vertical()
                                .max_height(360.0)
                                .show(ui, |ui| {
                                    egui::Grid::new((id, "shortcuts")).show(ui, |ui| {
                                        for row in rows {
                                            ui.label(text(row, "label"));
                                            ui.weak(text(row, "shortcut"));
                                            ui.end_row();
                                        }
                                    });
                                });
                        } else {
                            ui.add(egui::Label::new(text(value, "body")).wrap());
                        }
                    });
                if let Some(index) = shown.chosen {
                    let choice = &value["choices"][index];
                    emit(
                        events,
                        id,
                        "answer",
                        json!({"choice":choice["id"],"instance":value["instance"]}),
                    );
                    toolkit::dialog::Dialog::reset(ui.ctx(), dialog_id);
                } else if shown.dismissed {
                    emit(
                        events,
                        id,
                        "answer",
                        json!({"choice":"close","instance":value["instance"]}),
                    );
                    toolkit::dialog::Dialog::reset(ui.ctx(), dialog_id);
                }
                return ui.interact(shown.rect, dialog_id.with("receipt"), egui::Sense::hover());
            }
            let mut field = value["field"].as_str().map(str::to_owned);
            let modal = egui::Modal::new(egui::Id::unique((ui.ctx().viewport_id(), id, "modal")))
                .show(ui.ctx(), |ui| {
                    let width=value["width"].as_f64().unwrap_or(520.0) as f32;
                    ui.set_min_width(width.min(ui.ctx().content_rect().width()-32.0).max(280.0));
                    ui.set_max_width(width);
                    ui.heading(text(value, "title"));
                    if !text(value, "body").is_empty() {
                        egui::ScrollArea::vertical()
                            .id_salt((id, "body"))
                            .max_height(350.0)
                            .show(ui, |ui| {
                                ui.add(egui::Label::new(text(value, "body")).wrap());
                            });
                    }
                    if let Some(text) = field.as_mut() {
                        let edit=if value["multiline"]==true{egui::TextEdit::multiline(text).desired_rows(value["rows"].as_u64().unwrap_or(16) as usize).code_editor()}else{egui::TextEdit::singleline(text)};
                        let response = ui.add(
                            edit
                                .id(egui::Id::unique((ui.ctx().viewport_id(), id, "field")))
                                .desired_width(ui.available_width()),
                        );
                        geometry.insert(format!("{id}.field"), response.rect);
                        let marker = egui::Id::unique((
                            ui.ctx().viewport_id(),
                            id,
                            "modal-focused",
                        ));
                        if ui.ctx().data(|d|d.get_temp::<Value>(marker)).as_ref()!=Some(&value["instance"])
                            || (!response.has_focus() && ui.memory(|m|m.focused().is_none())) {
                            response.request_focus();
                            ui.ctx().data_mut(|d| d.insert_temp(marker,value["instance"].clone()));
                        }
                        if response.changed() {
                            emit(events, id, "edit", json!(text));
                        }
                    }
                    if !text(value, "error").is_empty() {
                        ui.colored_label(ui.visuals().error_fg_color, text(value, "error"));
                    }
                    let mut choices = |ui: &mut Ui| {
                        for choice in value["choices"].as_array().into_iter().flatten() {
                            let response = ui.add_enabled(
                                choice["enabled"] == true,
                                egui::Button::new(text(choice, "label")),
                            );
                            geometry.insert(
                                format!("{id}.choice.{}", text(choice, "id")),
                                response.rect,
                            );
                            if response.clicked() {
                                emit(
                                    events,
                                    id,
                                    "answer",
                                    json!({"choice":choice["id"],"field":field,"instance":value["instance"]}),
                                );
                            }
                        }
                    };
                    if value["choice_layout"] == "list" {
                        egui::ScrollArea::vertical()
                            .id_salt((id, "choices"))
                            .max_height(350.0)
                            .show(ui, |ui| {
                                ui.vertical(&mut choices);
                            });
                    } else {
                        ui.horizontal(choices);
                    }
                });
            if modal.should_close() {
                if let Some(cancel) = value["choices"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .find(|c| c["cancel"] == true)
                {
                    emit(
                        events,
                        id,
                        "answer",
                        json!({"choice":cancel["id"],"field":field,"instance":value["instance"]}),
                    );
                }
            } else if ui.input(|i| {
                (value["multiline"] != true && i.key_pressed(egui::Key::Enter))
                    || (value["multiline"] == true
                        && i.modifiers.ctrl
                        && (i.key_pressed(egui::Key::Enter) || i.key_pressed(egui::Key::S)))
            }) {
                if let Some(default) = value["choices"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .find(|c| c["default"] == true && c["enabled"] == true)
                {
                    emit(
                        events,
                        id,
                        "answer",
                        json!({"choice":default["id"],"field":field,"instance":value["instance"]}),
                    );
                }
            }
            modal.response
        }
        _ => ui.colored_label(egui::Color32::RED, "Unsupported rich widget"),
    }
}
