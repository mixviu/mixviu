// SPDX-License-Identifier: MIT OR Apache-2.0
//! Application-authored menu data adapted to the retained gold menu engine.
//! Its legacy API takes static command IDs: a bounded process-lifetime pool
//! interns at most 1024 IDs of at most 256 bytes; model updates reuse them.
use serde_json::{Value,json};
use std::collections::BTreeMap;
#[derive(Default)]
pub struct NativeMenu { ids:BTreeMap<String,&'static str> }
impl NativeMenu {
    fn id(&mut self,id:&str)->Result<&'static str,String>{
        if let Some(id)=self.ids.get(id){return Ok(*id);}
        if self.ids.len()>=1024 || id.len()>256{return Err("native menu command budget exceeded".into());}
        let owned: &'static str=Box::leak(id.to_owned().into_boxed_str());self.ids.insert(id.into(),owned);Ok(owned)
    }
    fn entries(&mut self,value:&Value)->Result<Vec<toolkit::menu::Entry>,String>{
        use toolkit::menu::{Entry,Row};let mut rows=Vec::new();
        for item in value.as_array().into_iter().flatten(){
            if item["separator"]==true{rows.push(Entry::Separator);continue;}
            let label=item["label"].as_str().unwrap_or("");
            if item["children"].is_array(){rows.push(Entry::Row(Row::submenu(label,self.entries(&item["children"])?)));}
            else{let id=self.id(item["id"].as_str().ok_or("menu id required")?)?;let shortcut=item["shortcut"].as_str().map(str::to_owned);let enabled=item["enabled"]==true;
                rows.push(Entry::Row(if let Some(checked)=item["checked"].as_bool(){Row::choice(id,label,shortcut,enabled,checked)}else{Row::command(id,label,shortcut,enabled)}));}
        }Ok(rows)
    }
    pub fn draw(&mut self,ui:&mut egui::Ui,widget:&crate::widgets::Widget,value:&Value,events:&mut Vec<crate::widgets::Intent>)->egui::Response{
        let menus=value.as_array().into_iter().flatten().map(|item|Ok(toolkit::menu::Menu{title:item["label"].as_str().unwrap_or("").into(),entries:self.entries(&item["children"])?})).collect::<Result<Vec<_>,String>>();
        let menus=match menus{Ok(m)=>m,Err(e)=>return ui.colored_label(egui::Color32::RED,e)};
        ui.scope(|ui|{if let Some(command)=toolkit::menu::bar(ui,&menus){events.push(crate::widgets::Intent{id:widget.id.clone(),action:"command".into(),value:json!({"command":command})});}}).response
    }
}
