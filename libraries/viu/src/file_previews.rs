// SPDX-License-Identifier: MIT OR Apache-2.0
//! Caller-owned local image previews with the same file actions as file tiles.
use egui::{Sense,Ui};
use serde_json::{Value,json};
use crate::widgets::Intent;
pub fn validate(v:&Value)->Result<(),String>{
    crate::file_tiles::validate(v)?;
    for p in v["items"].as_array().unwrap(){
        crate::image_view::validate(&p["preview"])?;
        if !p["alt"].as_str().is_some_and(|s|s.len()<=4096) || !p["status"].as_str().is_some_and(|s|s.len()<=8192){return Err("preview labels exceed budget".into());}
    }
    Ok(())
}
pub(crate) fn draw(ui:&mut Ui,id:&str,v:&Value,images:&mut crate::image_view::Images,events:&mut Vec<Intent>)->egui::Response{
    ui.scope(|ui|{
        for p in v["items"].as_array().unwrap(){
            let name=p["name"].as_str().unwrap();
            ui.horizontal(|ui|{
                let height=ui.text_style_height(&egui::TextStyle::Body);
                let (short,_)=ui.allocate_exact_size(egui::vec2(height,height),Sense::hover());
                let stroke=ui.visuals().widgets.noninteractive.bg_stroke;
                ui.painter().hline(short.x_range(),short.center().y,stroke);
                ui.add(egui::Label::new(name).truncate()).on_hover_text(name);
                let (line,_)=ui.allocate_exact_size(egui::vec2(ui.available_width(),height),Sense::hover());
                ui.painter().hline(line.x_range(),line.center().y,stroke);
            });
            if p["preview"]["phase"]=="ready"{
                let mut preview=p["preview"].clone();preview["natural"]=json!(true);preview["alt"]=p["alt"].clone();
                let response=images.draw(ui,&format!("{id}:{}",p["key"].as_str().unwrap()),&preview);
                crate::file_tiles::menu(&response,p,v,id,events);
            }else{ui.weak(p["status"].as_str().unwrap());}
        }
    }).response
}
