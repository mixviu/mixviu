// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native persistence envelope. Mix owns its versioned application value;
//! toolkit owns portable frames, secondary windows and named widget state.
use serde_json::{Value,json};
pub struct Saved {
    pub store: toolkit::saved::Store<Value>,
    pub announced: bool,
    accepted: bool,
}
impl Saved {
    pub fn load(app:&str,version:u32,headless:bool)->Self {
        Self{store:toolkit::saved::Store::load(if headless {None}else{config::AppDirs::resolve(app).map(|d|d.state())},version),announced:false,accepted:false}
    }
    pub fn notice(&self)->Value {json!({"state":self.store.describe(),"app_loaded":self.store.app_loaded()})}
    pub fn apply(&mut self,model:&Value) {
        self.accepted|=model.get("saved_app").is_some() || model["saved_panels"].is_object();
        if let Some(app)=model.get("saved_app"){self.store.app=app.clone();}
        if let Some(panels)=model["saved_panels"].as_object(){for(name,value) in panels{if let Some(share)=value.as_f64().filter(|v|v.is_finite() && (0.0..=1.0).contains(v)){self.store.widgets.panels.insert(name.clone(),share as f32);}}}
        if let Some(appearance)=model["appearance"].as_object(){
            let mut choice=Value::Object(appearance.clone());choice["captions"]=choice["caption_side"].clone();
            if let Ok(theme)=serde_json::from_value(choice){self.store.window.theme=theme;}
        }
    }
    pub fn tick(&mut self,ctx:&egui::Context,app:&str,flush:bool) {
        if !self.accepted {return;}
        self.store.window.capture(ctx);self.store.widgets.collect(ctx);self.store.collect_windows(ctx,app);
        let result=if flush{self.store.flush()}else{self.store.tick(std::time::Instant::now(),ctx.input(|i|i.pointer.any_down()))};
        if let Err(e)=result{eprintln!("saved window state: {e}");}
    }
}
