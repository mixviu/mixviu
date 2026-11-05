// SPDX-License-Identifier: MIT OR Apache-2.0
//! Capped PNG preview display. The application chooses a cache entry; decoding
//! and GPU texture ownership belong to this shared renderer primitive.
use egui::{TextureHandle,Ui};
use serde_json::Value;
use std::collections::{BTreeMap,BTreeSet};
type Key=(String,String);
pub fn validate(value:&Value)->Result<(),String>{
    if !["none","pending","ready","skipped","failed"].contains(&value["phase"].as_str().unwrap_or("")){return Err("unknown image phase".into());}
    if value["phase"]=="ready"&&value["path"].as_str().is_none_or(|s|s.len()>4096||!std::path::Path::new(s).is_absolute()){return Err("ready image requires an absolute cache path".into());}
    Ok(())
}
struct Slot{path:String,texture:Option<TextureHandle>,answered:bool}
pub(crate) struct Images{decoder:crate::thumbnail::Decoder<Key>,slots:BTreeMap<String,Slot>,visible:BTreeSet<String>}
impl Default for Images{fn default()->Self{Self{decoder:crate::thumbnail::Decoder::new("viu-preview-decode"),slots:BTreeMap::new(),visible:BTreeSet::new()}}}
impl Images{
    pub fn states(&self)->Value{serde_json::json!(self.slots.iter().map(|(id,s)|(id.clone(),serde_json::json!({"path":s.path,"phase":if s.texture.is_some(){"ready"}else if s.answered{"failed"}else{"pending"}}))).collect::<BTreeMap<_,_>>())}
    pub fn begin(&mut self){self.visible.clear();}
    pub fn finish(&mut self){
        self.slots.retain(|id,_|self.visible.contains(id));
        self.decoder.retain(&self.slots.iter().map(|(id,s)|(id.clone(),s.path.clone())).collect::<Vec<_>>());
    }
    pub fn draw(&mut self,ui:&mut Ui,id:&str,value:&Value)->egui::Response{
        self.visible.insert(id.into());
        if value["phase"]!="ready"{self.slots.remove(id);return ui.allocate_response(egui::Vec2::ZERO,egui::Sense::hover());}
        let path=value["path"].as_str().unwrap_or("");
        if self.slots.get(id).is_some_and(|s|s.path!=path){self.slots.remove(id);}
        if !self.slots.contains_key(id){
            if self.slots.len()>=16{return ui.label("Preview capacity reached");}
            let key=(id.to_owned(),path.to_owned());
            let accepted=self.decoder.ask(&key,std::path::Path::new(path),ui.ctx());
            self.slots.insert(id.into(),Slot{path:path.into(),texture:None,answered:!accepted});
        }
        for ((id,path),image) in self.decoder.drain(){
            if let Some(slot)=self.slots.get_mut(&id).filter(|s|s.path==path){slot.answered=true;slot.texture=image.map(|image|ui.ctx().load_texture(format!("preview:{id}:{path}"),image,egui::TextureOptions::LINEAR));}
        }
        if let Some(texture)=self.slots.get(id).and_then(|s|s.texture.as_ref()){
            let natural=texture.size_vec2()/ui.ctx().pixels_per_point();
            let scale=(ui.available_width()/natural.x).min(240.0/natural.y).min(1.0);
            ui.vertical_centered(|ui|{ui.add(egui::Image::new(texture).fit_to_exact_size(natural*scale));}).response
        }else{ui.allocate_response(egui::Vec2::ZERO,egui::Sense::hover())}
    }
}
