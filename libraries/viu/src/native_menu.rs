// SPDX-License-Identifier: MIT OR Apache-2.0
//! Mix-authored menus rendered by the gold toolkit, including its drive model.
use serde_json::Value;
use std::{collections::BTreeMap,sync::{Mutex,OnceLock}};
// The retained toolkit's command IDs are static. Intern only bounded command
// identifiers (never labels/data), once per app process, with a fixed ceiling.
fn command_id(id:&str)->Result<&'static str,String> {
    static IDS:OnceLock<Mutex<BTreeMap<String,&'static str>>>=OnceLock::new();
    let mut ids=IDS.get_or_init(Default::default).lock().map_err(|_|"menu ID owner failed")?;
    if let Some(id)=ids.get(id){return Ok(*id);}
    if id.is_empty()||id.len()>128||ids.len()>=4096{return Err("native menu ID budget exceeded".into());}
    let owned: &'static str=Box::leak(id.to_owned().into_boxed_str());
    ids.insert(id.into(),owned);Ok(owned)
}
fn entries(values:&Value)->Result<Vec<toolkit::menu::Entry>,String> {
    use toolkit::menu::{Entry,Row};
    values.as_array().ok_or("menu entries must be an array")?.iter().map(|item| {
        if item["separator"]==true{return Ok(Entry::Separator);}
        let label=item["label"].as_str().ok_or("menu label required")?;
        if item["children"].is_array(){return Ok(Entry::Row(Row::submenu(label,entries(&item["children"])?)));}
        let id=command_id(item["id"].as_str().ok_or("menu command ID required")?)?;
        let mut row=Row::command(id,label,item["shortcut"].as_str().map(str::to_owned),item["enabled"]==true);
        row.checked=item["checked"].as_bool();Ok(Entry::Row(row))
    }).collect()
}
pub fn draw(ui:&mut egui::Ui,values:&Value)->Result<Option<String>,String> {
    let menus=values.as_array().ok_or("menus must be an array")?.iter().map(|menu|Ok(toolkit::menu::Menu{title:menu["label"].as_str().ok_or("menu title required")?.into(),entries:entries(&menu["children"])?})).collect::<Result<Vec<_>,String>>()?;
    Ok(toolkit::menu::bar(ui,&menus).map(str::to_owned))
}
pub fn enable(ctx:&egui::Context,enabled:bool){ctx.data_mut(|d|d.insert_temp(egui::Id::unique("viu-native-menus"),enabled));}
pub fn enabled(ctx:&egui::Context)->bool{ctx.data(|d|d.get_temp(egui::Id::unique("viu-native-menus")).unwrap_or(false))}
