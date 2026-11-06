// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared gold title-band geometry over Mix-authored command data.
use serde_json::Value;
use std::collections::BTreeSet;
use toolkit::{action::{Action, Registry}, titlebar::Control};

fn add(registry: &mut Registry<()>, seen:&mut BTreeSet<String>, row:&Value, menu:Option<&'static str>, submenu:Option<&'static str>, group:u8) -> Result<(),String> {
    let id=row["id"].as_str().ok_or("title band action ID required")?;
    if !seen.insert(id.into()) {return Ok(());}
    let checked=row["checked"].as_bool();
    let action=Action {
        id:crate::native_menu::command_id(id)?,
        label:crate::native_menu::retained_text(row["label"].as_str().ok_or("action label required")?)?,
        menu,submenu,group,
        shortcut:row["shortcut"].as_str().and_then(toolkit::keymap::parse_chord),
        icon:row["icon"].as_str().and_then(crate::icons::Icon::from_name),
        enabled:if row["enabled"]==true { |_|true }else{ |_|false },
        run:|_|{},
    };
    if let Some(checked)=checked {registry.add_toggle(action,move |_|checked);}else{registry.add(action);}
    Ok(())
}
fn rows(registry:&mut Registry<()>,seen:&mut BTreeSet<String>, values:&Value,menu:&'static str,submenu:Option<&'static str>,start:u8)->Result<(),String> {
    let mut group=start;
    for row in values.as_array().ok_or("menu rows required")? {
        if row["separator"]==true {group=group.checked_add(1).ok_or("too many menu groups")?;}
        else if row["children"].is_array() {
            if submenu.is_some(){return Err("native title band supports one submenu level".into());}
            let label=crate::native_menu::retained_text(row["label"].as_str().ok_or("submenu label required")?)?;
            rows(registry,seen,&row["children"],menu,Some(label),group)?;
        } else {add(registry,seen,row,Some(menu),submenu,group)?;}
    }
    Ok(())
}
pub fn draw(ui:&mut egui::Ui, values:&Value, search_menu:Option<&str>)->Result<Vec<(String,String)>,String> {
    let mut registry=Registry::new();let mut seen=BTreeSet::new();
    for menu in values["gold-menu"].as_array().ok_or("title band menus required")? {
        let title=crate::native_menu::retained_text(menu["label"].as_str().ok_or("menu title required")?)?;
        rows(&mut registry,&mut seen,&menu["children"],title,None,0)?;
    }
    let toolbar=values["gold-toolbar"].as_array().ok_or("title band actions required")?;
    for row in toolbar {add(&mut registry,&mut seen,row,None,None,0)?;}
    registry.no_search();
    if let Some(menu)=search_menu {registry.search_menu(crate::native_menu::retained_text(menu)?);}
    let mut controls=Vec::new();
    for row in toolbar.iter().rev() {controls.push(Control::Icon {
        command:crate::native_menu::command_id(row["id"].as_str().ok_or("bar command required")?)?,
        icon:row["icon"].as_str().and_then(crate::icons::Icon::from_name).ok_or("bar icon required")?,
        selected:row["checked"]==true,
    });}
    crate::native_menu::publish_bar(ui.ctx(),&values["gold-toolbar"])?;
    if toolkit::titlebar::hosting::hosted(ui.ctx()){controls.clear();}
    let fired=toolkit::titlebar::show_with(ui,values["gold-title"].as_str().unwrap_or(""),values["gold-icon"].as_str().and_then(crate::icons::Icon::from_name),&registry,&(),&toolkit::strings::Strings::new(""),&mut controls);
    Ok(fired.into_iter().map(|(id,origin)|(id.into(),match origin {toolkit::action::Origin::Menu=>"menu",toolkit::action::Origin::Palette=>"palette",_=>"ui"}.into())).collect())
}
