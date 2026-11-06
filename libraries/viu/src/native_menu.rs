// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded adapter from Mix command data to the native menu renderer.
//! Static native slots are allocated once; application IDs remain owned data.
use egui::emath::GuiRounding;
use serde_json::Value;
use std::sync::OnceLock;
use toolkit::menu::{Entry, Menu, Row};

#[derive(Default)]
pub struct NativeMenu;
impl NativeMenu {
    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        widget: &crate::widgets::Widget,
        value: &Value,
        events: &mut Vec<crate::widgets::Intent>,
    ) -> egui::Response {
        let response = draw(ui, value, false, widget.properties["header_icon"].as_str());
        if let Some(command) = picked(ui.ctx()) {
            events.push(crate::widgets::Intent {
                id: widget.id.clone(),
                action: "command".into(),
                value: serde_json::json!({"command":command}),
            });
        }
        response
    }
}

const CAPACITY: usize = 256;
fn slots() -> &'static [&'static str] {
    static SLOTS: OnceLock<Vec<&'static str>> = OnceLock::new();
    SLOTS.get_or_init(|| {
        (0..CAPACITY)
            .map(|n| &*Box::leak(format!("viu.menu.slot.{n:03}").into_boxed_str()))
            .collect()
    })
}
pub fn command_slot(index: usize) -> Option<&'static str> {
    slots().get(index).copied()
}
#[derive(Clone, Default)]
struct Model {
    ids: Vec<String>,
    menu_len: usize,
}
fn model_id(ctx: &egui::Context) -> egui::Id {
    egui::Id::unique((ctx.viewport_id(), "viu-native-menu-model"))
}
pub fn application_id(ctx: &egui::Context, slot: &str) -> Option<String> {
    let index = slots().iter().position(|id| *id == slot)?;
    let key = model_id(ctx);
    ctx.data(|data| data.get_temp::<Model>(key))?
        .ids
        .get(index)
        .cloned()
}
pub fn native_id(ctx: &egui::Context, command: &str) -> Option<&'static str> {
    let key = model_id(ctx);
    let index = ctx.data_mut(|d| {
        let model = d.get_temp_mut_or_default::<Model>(key);
        if let Some(index) = model.ids.iter().position(|id| id == command) {
            return Some(index);
        }
        if model.ids.len() == CAPACITY {
            return None;
        }
        model.ids.push(command.into());
        Some(model.ids.len() - 1)
    })?;
    Some(slots()[index])
}
pub(crate) fn shortcut(ctx: Option<&egui::Context>, item: &Value) -> Option<String> {
    if item.get("shortcut_binding").is_some() {
        item["shortcut_binding"]
            .as_str()
            .and_then(toolkit::keymap::parse_chord)
            .and_then(|shortcut| ctx.map(|ctx| ctx.format_shortcut(&shortcut)))
    } else {
        item["shortcut"].as_str().map(|s| ctx.and_then(|ctx| toolkit::keymap::parse_chord(s).map(|key| ctx.format_shortcut(&key))).unwrap_or_else(||s.into()))
    }
}
fn entries(
    value: &Value,
    ids: &mut Vec<String>,
    ctx: Option<&egui::Context>,
) -> Result<Vec<Entry>, String> {
    value
        .as_array()
        .ok_or("menu entries must be a list")?
        .iter()
        .map(|item| {
            if item["separator"] == true {
                return Ok(Entry::Separator);
            }
            let children = if item["children"].is_array() {
                entries(&item["children"], ids, ctx)?
            } else {
                vec![]
            };
            let id = if item["children"].is_array() {
                None
            } else {
                if ids.len() == CAPACITY {
                    return Err("native menu exceeds 256 commands".into());
                }
                let slot = slots()[ids.len()];
                ids.push(item["id"].as_str().unwrap_or("").into());
                Some(slot.into())
            };
            Ok(Entry::Row(Row {
                id,
                label: item["label"].as_str().unwrap_or("").into(),
                shortcut: shortcut(ctx, item),
                enabled: item["enabled"] == true,
                checked: item["checked"].as_bool(),
                children,
            }))
        })
        .collect()
}
pub fn validate(value: &Value) -> Result<(), String> {
    let mut ids = vec![];
    for menu in value.as_array().ok_or("menus must be a list")? {
        entries(&menu["children"], &mut ids, None)?;
    }
    Ok(())
}
pub fn draw(ui: &mut egui::Ui, value: &Value, compact: bool, mark: Option<&str>) -> egui::Response {
    let mut ids = vec![];
    let menus: Vec<Menu> = value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| Menu {
            title: item["label"].as_str().unwrap_or("").into(),
            entries: entries(&item["children"], &mut ids, Some(ui.ctx()))
                .expect("validated native menu"),
        })
        .collect();
    let key = model_id(ui.ctx());
    let menu_len = ids.len();
    if let Some(old) = ui.ctx().data(|d| d.get_temp::<Model>(key)) {
        if old.ids[..old.menu_len] != ids {
            toolkit::menu::close(ui.ctx());
        }
        for id in old.ids.into_iter().skip(old.menu_len) {
            if !ids.contains(&id) && ids.len() < CAPACITY {
                ids.push(id);
            }
        }
    }
    ui.ctx()
        .data_mut(|d| d.insert_temp(key, Model { ids, menu_len }));
    let search = menus
        .iter()
        .position(|m| m.title == "Help")
        .map(|menu| toolkit::menu::Search {
            menu,
            hint: "Search menus".into(),
            empty: "No matching commands".into(),
        });
    let shown = ui.allocate_ui_with_layout(
        ui.available_size_before_wrap(),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            let hosted = toolkit::menu::form(ui.ctx())["hosted"] == true;
            header_mark(ui, mark, hosted);
            if hosted {
                let rect = ui.ctx().content_rect();
                let side = toolkit::chrome::Chrome::of(ui.ctx()).menu_side;
                let x = match side {
                    design::MenuSide::Left => rect.left(),
                    design::MenuSide::Center => rect.center().x,
                    design::MenuSide::Right => rect.right(),
                };
                toolkit::menu::compact_picked(
                    ui,
                    &menus,
                    search.as_ref(),
                    egui::Rect::from_min_size(egui::pos2(x, rect.top()), egui::Vec2::ZERO),
                )
            } else if compact {
                let response = ui.add(toolkit::button::IconButton::new(
                    crate::icons::Icon::from_name("menu").unwrap(),
                ));
                if (response.hovered() && ui.input(|i| i.pointer.primary_pressed()))
                    || (response.clicked() && !response.clicked_by(egui::PointerButton::Primary))
                {
                    toolkit::menu::toggle(ui.ctx());
                }
                toolkit::menu::compact_picked(ui, &menus, search.as_ref(), response.rect)
            } else {
                toolkit::menu::bar_picked(ui, &menus, search.as_ref())
            }
        },
    );
    if let Some(pick) = shown.inner {
        if let Some(command) = application_id(ui.ctx(), pick.0) {
            ui.ctx()
                .data_mut(|d| d.insert_temp(key.with("picked"), serde_json::json!({"command":command,"origin":if pick.1 {"palette"}else{"menu"}})));
        }
    }
    shown.response
}
pub fn picked(ctx: &egui::Context) -> Option<Value> {
    let key = model_id(ctx).with("picked");
    ctx.data_mut(|d| d.remove_temp::<Value>(key))
}
#[cfg(test)]
mod tests {
    #[test]
    fn hosted_tail_commands_keep_their_owned_identity_across_menu_frames() {
        let ctx = egui::Context::default();
        crate::appearance::install(&ctx);
        let menus = serde_json::json!([{"label":"File","children":[{"id":"file.refresh","label":"Refresh","enabled":true}]}]);
        let mut tail = None;
        for _ in 0..3 {
            ctx.run_ui(egui::RawInput::default(), |ui| {
                super::draw(ui, &menus, false, Some("monitor"));
                let current = super::native_id(ui.ctx(), "view.search").unwrap();
                if let Some(previous) = tail {
                    assert_eq!(current, previous);
                }
                tail = Some(current);
                assert_eq!(
                    super::application_id(ui.ctx(), current).as_deref(),
                    Some("view.search")
                );
                assert_eq!(
                    super::application_id(
                        ui.ctx(),
                        super::native_id(ui.ctx(), "file.refresh").unwrap()
                    )
                    .as_deref(),
                    Some("file.refresh")
                );
            })
            .drop_without_applying_deltas();
        }
    }
}

pub(crate) fn header_mark(ui: &mut egui::Ui, mark: Option<&str>, hosted: bool) {
    let chrome = toolkit::chrome::Chrome::of(ui.ctx());
    if !hosted && chrome.menu_side == design::MenuSide::Left {
        ui.add_space(
            toolkit::chrome::Chrome::of(ui.ctx())
                .metrics
                .title_bar_margin,
        );
    }
    if crate::window_frame::client_of(ui.ctx()) {
        if let Some(icon) = mark.and_then(crate::icons::Icon::from_name) {
            let (area, _) = ui.allocate_exact_size(
                egui::vec2(
                    chrome.metrics.mark + chrome.metrics.mark_gap,
                    ui.available_height(),
                ),
                egui::Sense::hover(),
            );
            let rect = egui::Rect::from_min_size(
                egui::pos2(area.left(), area.center().y - chrome.metrics.mark / 2.0),
                egui::Vec2::splat(chrome.metrics.mark),
            )
            .round_to_pixels(ui.pixels_per_point());
            toolkit::icons::image_of(ui.ctx(), icon, chrome.metrics.mark, chrome.palette.icon)
                .paint_at(ui, rect);
            let response = ui.interact(
                rect,
                ui.scope_id().with("window-menu-mark"),
                egui::Sense::click(),
            );
            response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::Role::Button,
                    true,
                    toolkit::strings::own("window-system-menu"),
                )
            });
            if response.clicked_by(egui::PointerButton::Primary) {
                crate::window_frame::request_menu(ui.ctx(), response.interact_pointer_pos());
            }
        }
    }
}
