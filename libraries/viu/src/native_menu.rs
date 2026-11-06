// SPDX-License-Identifier: MIT OR Apache-2.0
//! Bounded adapter from Mix command data to the native menu renderer.
//! Static native slots are allocated once; application IDs remain owned data.
use serde_json::Value;
use std::sync::OnceLock;
use toolkit::menu::{Entry, Menu, Row};

const CAPACITY: usize = 256;
fn slots() -> &'static [&'static str] {
    static SLOTS: OnceLock<Vec<&'static str>> = OnceLock::new();
    SLOTS.get_or_init(|| {
        (0..CAPACITY)
            .map(|n| &*Box::leak(format!("viu.menu.slot.{n}").into_boxed_str()))
            .collect()
    })
}
#[derive(Clone, Default)]
struct Model {
    ids: Vec<String>,
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
fn entries(value: &Value, ids: &mut Vec<String>) -> Result<Vec<Entry>, String> {
    value
        .as_array()
        .ok_or("menu entries must be a list")?
        .iter()
        .map(|item| {
            if item["separator"] == true {
                return Ok(Entry::Separator);
            }
            let children = if item["children"].is_array() {
                entries(&item["children"], ids)?
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
                Some(slot)
            };
            Ok(Entry::Row(Row {
                id,
                label: item["label"].as_str().unwrap_or("").into(),
                shortcut: item["shortcut"].as_str().map(str::to_owned),
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
        entries(&menu["children"], &mut ids)?;
    }
    Ok(())
}
pub fn draw(ui: &mut egui::Ui, value: &Value, compact: bool) -> egui::Response {
    let mut ids = vec![];
    let menus: Vec<Menu> = value
        .as_array()
        .unwrap()
        .iter()
        .map(|item| Menu {
            title: item["label"].as_str().unwrap_or("").into(),
            entries: entries(&item["children"], &mut ids).expect("validated native menu"),
        })
        .collect();
    let key = model_id(ui.ctx());
    if ui
        .ctx()
        .data(|d| d.get_temp::<Model>(key))
        .is_some_and(|old| old.ids != ids)
    {
        toolkit::menu::close(ui.ctx());
    }
    ui.ctx().data_mut(|d| d.insert_temp(key, Model { ids }));
    let search = menus
        .iter()
        .position(|m| m.title == "Help")
        .map(|menu| toolkit::menu::Search {
            menu,
            hint: "Search commands".into(),
            empty: "No matching commands".into(),
        });
    let shown = ui.allocate_ui_with_layout(
        ui.available_size_before_wrap(),
        egui::Layout::left_to_right(egui::Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = 0.0;
            if !crate::window_frame::client_of(ui.ctx()) {
                ui.add_space(toolkit::chrome::Chrome::of(ui.ctx()).metrics.title_bar_margin);
            }
            if compact {
                let response = ui.add(toolkit::button::IconButton::new(
                    crate::icons::Icon::from_name("menu").unwrap(),
                ));
                if response.clicked() {
                    if toolkit::menu::is_open(ui.ctx()) {
                        toolkit::menu::close(ui.ctx());
                    } else if let Some(first) = menus.first() {
                        let _ = toolkit::menu::open_path(ui.ctx(), &[first.title.clone()]);
                    }
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
