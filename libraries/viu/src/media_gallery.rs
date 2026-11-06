// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Reusable data-authored media tiles, framed preview and native file picker.
//! Categories, scans, persistence and selection policy belong to the caller.
use crate::{settings_browser::Look, widgets::Intent};
use egui::{Color32, Rect, Response, Sense, Stroke, StrokeKind, Ui, vec2};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
struct State {
    images: crate::image_view::Images,
    chooser: Option<(String, Option<String>, toolkit::choose::Handle)>,
}
fn state(ui: &Ui, id: &str) -> Arc<Mutex<State>> {
    let key = egui::Id::unique((ui.ctx().viewport_id(), id, "media"));
    ui.ctx().data_mut(|d| {
        d.get_temp_mut_or_insert_with(key, || {
            Arc::new(Mutex::new(State {
                images: crate::image_view::Images::surface_with_capacity(128),
                chooser: None,
            }))
        })
        .clone()
    })
}
pub(crate) fn begin(ui: &Ui, id: &str) {
    state(ui, id).lock().unwrap().images.begin();
}
pub(crate) fn finish(ui: &Ui, id: &str, events: &mut Vec<Intent>) {
    let shared = state(ui, id);
    let mut state = shared.lock().unwrap();
    state.images.finish();
    let outcome = state
        .chooser
        .as_mut()
        .and_then(|(row, operation, handle)| handle.poll(ui.ctx()).map(|answer| (row.clone(), operation.clone(), answer)));
    if let Some((row, operation, answer)) = outcome {
        state.chooser = None;
        match answer {
            toolkit::choose::Outcome::Chosen {paths,..}=>{
                let paths=json!(paths.iter().filter_map(|p|p.to_str()).collect::<Vec<_>>());
                let value=operation.map_or_else(||paths.clone(),|operation|json!({"action":operation,"value":paths}));
                events.push(Intent {id:id.into(),action:"edit".into(),value:json!({"row":row,"value":value})});
            },
            toolkit::choose::Outcome::Failed(message)=>events.push(Intent {id:id.into(),action:"problem".into(),value:json!({"row":row,"message":message})}),
            toolkit::choose::Outcome::Cancelled=>{}
        }
    }
}
fn text<'a>(v: &'a Value, k: &str) -> &'a str {
    v[k].as_str().unwrap_or("")
}
fn colour(value: &str) -> Option<Color32> {
    let hex = value.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let n = u32::from_str_radix(hex, 16).ok()?;
    Some(Color32::from_rgb((n >> 16) as u8, (n >> 8) as u8, n as u8))
}
fn fill(ui: &Ui, rect: Rect, value: &str, radius: f32) {
    if let Some(c) = colour(value) {
        ui.painter().rect_filled(rect, Look::radius(radius), c);
        return;
    }
    let Some(gradient) = value.strip_prefix("gradient:") else {
        return;
    };
    let Some((stops, angle)) = gradient.split_once('@') else {
        return;
    };
    let Some((a, b)) = stops.split_once(',') else {
        return;
    };
    let (Some(a), Some(b), Ok(angle)) = (colour(a), colour(b), angle.parse::<f32>()) else {
        return;
    };
    let (sin, cos) = angle.to_radians().sin_cos();
    let direction = vec2(sin, -cos);
    let length = (rect.width() * sin).abs() + (rect.height() * cos).abs();
    let shape = egui::epaint::RectShape::filled(rect, Look::radius(radius), Color32::WHITE);
    let mut tessellator = egui::epaint::Tessellator::new(
        ui.ctx().pixels_per_point(),
        Default::default(),
        [1, 1],
        Vec::new(),
    );
    let mut mesh = egui::epaint::Mesh::default();
    tessellator.tessellate_rect(&shape, &mut mesh);
    for vertex in &mut mesh.vertices {
        let t = (vertex.pos - rect.center()).dot(direction) / length.max(1.0) + 0.5;
        let coverage = f32::from(vertex.color.a()) / 255.0;
        vertex.color = a
            .lerp_to_gamma(b, t.clamp(0.0, 1.0))
            .gamma_multiply(coverage);
    }
    ui.painter().add(egui::Shape::mesh(mesh));
}
pub(crate) fn draw(
    ui: &mut Ui,
    id: &str,
    row: &Value,
    look: &Look,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) -> Response {
    let key = text(row, "id");
    if row["kind"] == "file-pick" {
        let response = ui.button(text(row, "title"));
        if response.clicked() {
            if let Ok(request) = toolkit::choose::request(&row["request"].to_string()) {
                state(ui, id).lock().unwrap().chooser =
                    Some((key.into(), None, toolkit::choose::choose(request)));
            }
        }
        geometry.insert(format!("{id}.row.{key}.control"), response.rect);
        return response;
    }
    if row["kind"] == "display-picker" {
        let response=ui.vertical(|ui| {
            let choices:Vec<_>=row["choices"].as_array().into_iter().flatten().collect();
            if row["all"]==true {
                let galley=ui.painter().layout_no_wrap(text(row,"all_label").into(),toolkit::fonts::bound(ui.ctx(),toolkit::fonts::heading(14.0)),look.text);
                let (rect,r)=ui.allocate_exact_size(vec2(ui.available_width(),21.0),Sense::hover());
                ui.painter().galley(rect.center()-galley.size()/2.0,galley,look.text);
                r.widget_info(||egui::WidgetInfo::labeled(egui::Role::Label,true,text(row,"all_label")));
            } else if choices.len()>1 {
                let widths:Vec<_>=choices.iter().map(|c|ui.painter().layout_no_wrap(text(c,"label").into(),egui::FontId::proportional(14.0),look.text).size().x+2.0*look.s).collect();
                let total=widths.iter().sum::<f32>()+look.xs*(choices.len()-1) as f32;
                let (area,_)=ui.allocate_exact_size(vec2(ui.available_width(),32.0),Sense::hover());
                let mut x=area.center().x-total/2.0;
                for (index,(choice,width)) in choices.iter().zip(widths).enumerate() {
                    let rect=Rect::from_min_size(egui::pos2(x,area.top()),vec2(width,32.0));x+=width+look.xs;
                    let r=ui.interact(rect,ui.scope_id().with((id,key,index)),Sense::click());
                    if r.hovered() {ui.painter().rect_filled(rect,Look::radius(look.r_s),look.card_hover);}
                    let selected=choice["value"]==row["value"];
                    ui.painter().text(rect.center(),egui::Align2::CENTER_CENTER,text(choice,"label"),egui::FontId::proportional(14.0),look.text);
                    if selected {ui.painter().hline(rect.x_range(),rect.bottom()-1.0,Stroke::new(2.0,look.accent));}
                    r.widget_info(||egui::WidgetInfo::selected(egui::Role::RadioButton,ui.is_enabled(),selected,text(choice,"label")));
                    geometry.insert(format!("{id}.row.{key}.choice.{index}"),rect);
                    if r.clicked()&&!selected {events.push(Intent{id:id.into(),action:"edit".into(),value:json!({"row":key,"value":choice["value"]})});}
                }
            }
        }).response;
        return response;
    }
    if row["kind"] == "media-chooser" {
        let response=ui.vertical(|ui| {
            let send=|events:&mut Vec<Intent>,action:&str,value:Value|events.push(Intent{id:id.into(),action:"edit".into(),value:json!({"row":key,"value":{"action":action,"value":value}})});
            ui.horizontal(|ui| {
                ui.set_width(ui.available_width());
                let folder=row["value"]["kind"]=="folder";
                let title=text(row,"display_value");
                let width=ui.painter().layout_no_wrap(title.into(),egui::FontId::proportional(14.0),look.text).size().x+32.0+if folder {24.0}else{0.0};
                let (rect,r)=ui.allocate_exact_size(vec2(width,32.0),Sense::click());
                let mut x=rect.left()+4.0;
                if folder {toolkit::button::paint_icon(ui,toolkit::Icon::Folder,egui::pos2(x+8.0,rect.center().y),16.0,look.text);x+=24.0;}
                ui.painter().text(egui::pos2(x,rect.center().y),egui::Align2::LEFT_CENTER,title,egui::FontId::proportional(14.0),look.text);
                toolkit::button::paint_icon(ui,toolkit::Icon::ChevronDown,egui::pos2(rect.right()-12.0,rect.center().y),16.0,look.text);
                r.widget_info(||egui::WidgetInfo::selected(egui::Role::ComboBox,ui.is_enabled(),false,text(row,"category_label")));
                geometry.insert(format!("{id}.row.{key}.category"),rect);
                egui::Popup::menu(&r).show(ui,|ui| {
                    ui.set_min_width(width.max(200.0));
                    for (index,choice) in row["categories"].as_array().into_iter().flatten().enumerate() {
                        if index==2 {ui.separator();ui.label(egui::RichText::new(text(row,"recent_label")).size(12.0).color(look.text));}
                        let on=choice["value"]==row["value"];
                        if toolkit::selectable(ui,on,text(choice,"label")).clicked()&&!on {send(events,"category",choice["value"].clone());ui.close();}
                    }
                    ui.separator();
                    if toolkit::selectable(ui,false,text(row,"folder_label")).clicked() {
                        if let Ok(request)=toolkit::choose::request(&row["folder_request"].to_string()) {state(ui,id).lock().unwrap().chooser=Some((key.into(),Some("folder".into()),toolkit::choose::choose(request)));}
                        ui.close();
                    }
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui| {
                    if !text(row,"add_label").is_empty() {
                        let galley=ui.painter().layout_no_wrap(text(row,"add_label").into(),egui::FontId::proportional(14.0),look.accent);
                        let (rect,r)=ui.allocate_exact_size(vec2(galley.size().x+20.0,24.0),Sense::click());
                        ui.painter().galley(egui::pos2(rect.left(),rect.center().y-galley.size().y/2.0),galley,look.accent);
                        toolkit::button::paint_icon(ui,toolkit::Icon::MoveUpRight,egui::pos2(rect.right()-8.0,rect.center().y),16.0,look.accent);
                        r.widget_info(||egui::WidgetInfo::labeled(egui::Role::Link,ui.is_enabled(),text(row,"add_label")));
                        geometry.insert(format!("{id}.row.{key}.add"),rect);
                        if r.clicked() {
                            if row["add_request"].is_object() {
                                if let Ok(request)=toolkit::choose::request(&row["add_request"].to_string()) {state(ui,id).lock().unwrap().chooser=Some((key.into(),Some("images".into()),toolkit::choose::choose(request)));}
                            } else {send(events,"colour",Value::Null);}
                        }
                    }
                });
            });
            ui.add_space(look.m);
            if row["tiles"]["choices"].as_array().is_none_or(|a|a.is_empty()) {ui.label(egui::RichText::new(text(row,"empty_label")).size(14.0).color(look.text));}
            else {draw(ui,id,&row["tiles"],look,geometry,events);}
        }).response;
        geometry.insert(format!("{id}.row.{key}"),response.rect);
        return response;
    }
    if row["kind"] == "media-preview" {
        let (slot, response) =
            ui.allocate_exact_size(vec2(ui.available_width(), 177.0), Sense::hover());
        let frame = Rect::from_center_size(slot.center(), vec2(308.0, 177.0));
        ui.painter()
            .rect_filled(frame, Look::radius(look.r_xs + 4.0), look.text);
        let screen = frame.shrink(4.0);
        ui.painter()
            .rect_filled(screen, Look::radius(look.r_xs), look.window);
        if !text(row, "colour").is_empty() {
            fill(ui, screen, text(row, "colour"), look.r_xs);
        }
        if row["path"].is_string() && ui.is_rect_visible(screen) {
            let shared = state(ui, id);
            let mut state = shared.lock().unwrap();
            let mut child = ui.new_child(egui::UiBuilder::new().max_rect(screen));
            state.images.draw_fill(&mut child,key,&json!({"phase":"ready","path":row["path"],"fit":row["fit"].as_str().unwrap_or("fill"),"colour":[look.window.r(),look.window.g(),look.window.b()]}));
        }
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::Role::Image, ui.is_enabled(), text(row, "title"))
        });
        geometry.insert(format!("{id}.row.{key}"), response.rect);
        return response;
    }
    let response=ui.horizontal_wrapped(|ui|{
        ui.spacing_mut().item_spacing=vec2(12.0,16.0);
        for (index,item) in row["choices"].as_array().into_iter().flatten().enumerate(){
            let is_colour=item["colour"].is_string();
            let size=if is_colour{vec2(70.0,70.0)}else{vec2(158.0,89.0)};
            let (rect,r)=ui.allocate_exact_size(size,Sense::click());
            if ui.is_rect_visible(rect){
                if is_colour{fill(ui,rect,text(item,"colour"),look.r_s);}else{
                    ui.painter().rect_filled(rect,Look::radius(look.r_s),look.card);
                    let shared=state(ui,id);let mut state=shared.lock().unwrap();
                    let mut child=ui.new_child(egui::UiBuilder::new().max_rect(rect));
                    state.images.draw_fill(&mut child,&format!("{key}:{index}"),&json!({"phase":"ready","path":item["path"],"fit":"fill","colour":[look.card.r(),look.card.g(),look.card.b()]}));
                }
                if item["value"]==row["value"]{ui.painter().rect_stroke(rect.expand(2.0),Look::radius(look.r_s+2.0),Stroke::new(2.0,look.accent),StrokeKind::Outside);}
                if item["removable"]==true {
                    let close=Rect::from_center_size(rect.right_top()+vec2(-12.0,12.0),vec2(20.0,20.0));
                    let remove=ui.interact(close,r.id.with("remove"),Sense::click());
                    ui.painter().circle_filled(close.center(),10.0,look.button);
                    toolkit::button::paint_icon(ui,toolkit::Icon::X,close.center(),12.0,look.text);
                    remove.widget_info(||egui::WidgetInfo::labeled(egui::Role::Button,ui.is_enabled(),format!("{} {}",text(row,"remove_label"),text(item,"label"))));
                    if remove.clicked(){events.push(Intent{id:id.into(),action:"remove".into(),value:json!({"row":key,"value":item["value"]})});}
                }
            }
            r.widget_info(||egui::WidgetInfo::selected(egui::Role::RadioButton,ui.is_enabled(),item["value"]==row["value"],text(item,"label")));
            geometry.insert(format!("{id}.row.{key}.choice.{index}"),r.rect);
            if r.clicked(){events.push(Intent{id:id.into(),action:"edit".into(),value:json!({"row":key,"value":item["value"]})});}
        }
    }).response;
    geometry.insert(format!("{id}.row.{key}"), response.rect);
    response
}
