// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared native window controls. The compositor retains movement, resize,
//! focus and presentation authority; this sends the usual viewport commands.
use egui::{Rect,Ui,vec2,pos2,ViewportCommand as Command};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Clone,Copy)]
pub struct Metrics {
    pub height:f32,
    pub caption_width:f32,
    pub caption_glyph:f32,
    pub title_size:f32,
    pub resize_edge:f32,
    pub resize_corner:f32,
}
impl Metrics {
    pub fn of(ctx:&egui::Context)->Self {
        ctx.data(|d|d.get_temp::<Self>(egui::Id::unique("viu-window-metrics"))).unwrap_or(Self{height:38.0,caption_width:36.0,caption_glyph:14.0,title_size:14.0,resize_edge:4.0,resize_corner:12.0})
    }
}

#[derive(Default)]
pub struct WindowFrame {
    server: Option<bool>,
    client: bool,
    pub geometry: BTreeMap<String,Rect>,
    /// The owner persists/cancels work and acknowledges automation before
    /// closing its native viewport.
    pub close_requested: bool,
}
impl WindowFrame {
    pub fn show(&mut self, ui:&mut Ui,title:&str,settings:&Value) {
        self.geometry.clear();
        self.close_requested=false;
        let requested=settings["window.decorations"].as_str().unwrap_or("auto");
        self.client=requested=="csd";
        let server=!self.client;
        if self.server!=Some(server){ui.ctx().send_viewport_cmd(Command::Decorations(server));self.server=Some(server);}
        if !self.client{return;}
        let metrics=Metrics::of(ui.ctx());
        let height=metrics.height;
        let (band,_)=ui.allocate_exact_size(vec2(ui.available_width(),height),egui::Sense::hover());
        ui.painter().rect_filled(band,0.0,crate::appearance::colour(ui.ctx(),"chrome").unwrap_or(ui.visuals().panel_fill));
        let left=settings["window.caption_side"]=="left";
        let caption_width=metrics.caption_width;
        let controls=Rect::from_min_size(pos2(if left{band.left()}else{band.right()-3.0*caption_width},band.top()),vec2(3.0*caption_width,height));
        let drag=Rect::from_min_max(pos2(if left{controls.right()}else{band.left()},band.top()),pos2(if left{band.right()}else{controls.left()},band.bottom()));
        self.geometry.insert("window.title".into(),drag);
        let response=ui.interact(drag,ui.id().with("viu-window-drag"),egui::Sense::click_and_drag());
        let maximized=ui.input(|i|i.viewport().maximized.unwrap_or(false));
        if response.double_clicked(){ui.ctx().send_viewport_cmd(Command::Maximized(!maximized));}
        else if response.drag_started(){ui.ctx().send_viewport_cmd(Command::StartDrag);}
        let font=crate::fonts::emphasis(ui.ctx(),egui::FontId::new(metrics.title_size,egui::FontFamily::Proportional));
        let title=crate::elide::fit(ui,title,&font,(drag.width()-24.0).max(0.0));
        ui.painter_at(drag).text(drag.center(),egui::Align2::CENTER_CENTER,title,font,ui.visuals().text_color());
        let names=if left{["close","minimize","maximize"]}else{["minimize","maximize","close"]};
        for (i,name) in names.into_iter().enumerate(){
            let rect=Rect::from_min_size(pos2(controls.left()+i as f32*caption_width,band.top()),vec2(caption_width,height));
            self.geometry.insert(format!("window.caption.{name}"),rect);
            let r=ui.interact(rect,ui.id().with(("viu-caption",name)),egui::Sense::click());
            r.widget_info(||egui::WidgetInfo::labeled(egui::Role::Button,true,name));
            if r.hovered(){ui.painter().rect_filled(rect,0.0,if name=="close"{ui.visuals().error_fg_color}else{ui.visuals().widgets.hovered.weak_bg_fill});}
            let glyph=if name=="close"{"x"}else if name=="minimize"{"minus"}else if maximized{"copy"}else{"square"};
            if let Some(icon)=crate::icons::Icon::from_name(glyph){crate::icons::image_of(ui.ctx(),icon,metrics.caption_glyph,ui.visuals().text_color()).paint_at(ui,Rect::from_center_size(rect.center(),vec2(metrics.caption_glyph,metrics.caption_glyph)));}
            if r.on_hover_text(name).clicked(){
                if name=="close"{self.close_requested=true;}
                else{ui.ctx().send_viewport_cmd(if name=="minimize"{Command::Minimized(true)}else{Command::Maximized(!maximized)});}
            }
        }
    }
    pub fn edges(&mut self,ui:&mut Ui){
        if !self.client || ui.input(|i|i.viewport().maximized.unwrap_or(false)||i.viewport().fullscreen.unwrap_or(false)){return;}
        let metrics=Metrics::of(ui.ctx());
        let r=ui.ctx().content_rect();let e=metrics.resize_edge;let c=metrics.resize_corner;
        use egui::ResizeDirection as D;
        for (name,direction,rect) in [
            ("nw",D::NorthWest,Rect::from_min_size(r.min,vec2(c,c))),
            ("ne",D::NorthEast,Rect::from_min_size(pos2(r.right()-c,r.top()),vec2(c,c))),
            ("sw",D::SouthWest,Rect::from_min_size(pos2(r.left(),r.bottom()-c),vec2(c,c))),
            ("se",D::SouthEast,Rect::from_min_size(r.max-vec2(c,c),vec2(c,c))),
            ("n",D::North,Rect::from_min_max(pos2(r.left()+c,r.top()),pos2(r.right()-c,r.top()+e))),
            ("s",D::South,Rect::from_min_max(pos2(r.left()+c,r.bottom()-e),pos2(r.right()-c,r.bottom()))),
            ("w",D::West,Rect::from_min_max(pos2(r.left(),r.top()+c),pos2(r.left()+e,r.bottom()-c))),
            ("e",D::East,Rect::from_min_max(pos2(r.right()-e,r.top()+c),pos2(r.right(),r.bottom()-c))),
        ]{
            self.geometry.insert(format!("window.resize.{name}"),rect);
            let response=ui.interact(rect,ui.id().with(("viu-resize",name)),egui::Sense::drag());
            if response.drag_started(){ui.ctx().send_viewport_cmd(Command::BeginResize(direction));}
        }
    }
}
