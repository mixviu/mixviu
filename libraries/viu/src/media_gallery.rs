// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Reusable data-authored media tiles, framed preview and native file picker.
//! Categories, scans, persistence and selection policy belong to the caller.
use crate::{settings_browser::Look, widgets::Intent};
use egui::{Color32, Rect, Response, Sense, Stroke, StrokeKind, Ui, vec2};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, mpsc, atomic::{AtomicU64, Ordering}},
};
struct State {
    pool: thumbs::Pool<String>,
    replies: mpsc::Receiver<thumbs::Reply<String>>,
    decoder: crate::thumbnail::Decoder<String>,
    live: Arc<AtomicU64>,
    textures: BTreeMap<String, Option<egui::TextureHandle>>,
    asked: BTreeMap<String,u64>,
    used: u64,
    active: bool,
    chooser: Option<(String, Option<String>, toolkit::choose::Handle)>,
}
impl State {
    fn new(ctx:&egui::Context)->Self {
        let (tx,replies)=mpsc::channel();let wake=ctx.clone();
        let pool=thumbs::Pool::new("viu",move |reply| {let sent=tx.send(reply).is_ok();wake.request_repaint();sent});
        Self {pool,replies,decoder:crate::thumbnail::Decoder::new("viu-media-thumbs"),live:Arc::new(AtomicU64::new(0)),textures:BTreeMap::new(),asked:BTreeMap::new(),used:0,active:false,chooser:None}
    }
    fn pump(&mut self,ctx:&egui::Context) {
        while let Ok(reply)=self.replies.try_recv() {
            if !self.asked.contains_key(&reply.lane) {continue;}
            match reply.outcome {
                thumbs::Outcome::Ready(cache)=>{if !self.decoder.ask(&reply.lane,&cache,ctx) {self.textures.insert(reply.lane,None);}},
                _=>{self.textures.insert(reply.lane,None);}
            }
        }
        for (key,image) in self.decoder.drain() {
            if self.asked.contains_key(&key) {self.textures.insert(key.clone(),image.map(|image|ctx.load_texture(&key,image,egui::TextureOptions::LINEAR)));}
        }
    }
    fn texture(&mut self,ctx:&egui::Context,path:&str,size:thumbs::Size)->Option<egui::TextureHandle> {
        self.pump(ctx);self.active=true;self.used+=1;
        let key=format!("{}:{path}",size.px());let first=self.asked.insert(key.clone(),self.used).is_none();
        while self.asked.len()>128 {
            let oldest=self.asked.iter().min_by_key(|(_,used)|*used).map(|(k,_)|k.clone()).unwrap();
            self.asked.remove(&oldest);self.textures.remove(&oldest);
        }
        if let Some(texture)=self.textures.get(&key) {return texture.clone();}
        if first {self.pool.submit(thumbs::Request{lane:key,ticket:thumbs::Ticket::new(self.live.clone(),self.live.load(Ordering::Relaxed)),path:path.into(),size});}
        None
    }
}
impl Drop for State {fn drop(&mut self) {self.live.fetch_add(1,Ordering::Relaxed);}}
fn state(ui: &Ui, id: &str) -> Arc<Mutex<State>> {
    let key = egui::Id::unique((ui.ctx().viewport_id(), id, "media"));
    ui.ctx().data_mut(|d| {
        d.get_temp_mut_or_insert_with(key, || {
            Arc::new(Mutex::new(State::new(ui.ctx())))
        })
        .clone()
    })
}
pub(crate) fn begin(ui: &Ui, id: &str) {
    state(ui, id).lock().unwrap().active=false;
}
pub(crate) fn finish(ui: &Ui, id: &str, events: &mut Vec<Intent>) {
    let shared = state(ui, id);
    let mut state = shared.lock().unwrap();
    if !state.active && !state.asked.is_empty() {
        state.live.fetch_add(1,Ordering::Relaxed);state.asked.clear();state.textures.clear();state.decoder.retain(&[]);
    }
    let outcome = state.chooser.as_mut().and_then(|(row, operation, handle)| {
        handle
            .poll(ui.ctx())
            .map(|answer| (row.clone(), operation.clone(), answer))
    });
    if let Some((row, operation, answer)) = outcome {
        state.chooser = None;
        match answer {
            toolkit::choose::Outcome::Chosen { paths, .. } => {
                let paths = json!(paths.iter().filter_map(|p| p.to_str()).collect::<Vec<_>>());
                let value = operation.map_or_else(
                    || paths.clone(),
                    |operation| json!({"action":operation,"value":paths}),
                );
                events.push(Intent {
                    id: id.into(),
                    action: "edit".into(),
                    value: json!({"row":row,"value":value}),
                });
            }
            toolkit::choose::Outcome::Failed(message) => events.push(Intent {
                id: id.into(),
                action: "problem".into(),
                value: json!({"row":row,"message":message}),
            }),
            toolkit::choose::Outcome::Cancelled => {}
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
        let response = ui
            .vertical(|ui| {
                let choices: Vec<_> = row["choices"].as_array().into_iter().flatten().collect();
                if row["all"] == true {
                    let galley = ui.painter().layout_no_wrap(
                        text(row, "all_label").into(),
                        toolkit::fonts::bound(ui.ctx(), toolkit::fonts::heading(14.0)),
                        look.text,
                    );
                    let (rect, r) =
                        ui.allocate_exact_size(vec2(ui.available_width(), 21.0), Sense::hover());
                    ui.painter()
                        .galley(rect.center() - galley.size() / 2.0, galley, look.text);
                    r.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::Role::Label, true, text(row, "all_label"))
                    });
                } else if choices.len() > 1 {
                    let widths: Vec<_> = choices
                        .iter()
                        .map(|c| {
                            ui.painter()
                                .layout_no_wrap(
                                    text(c, "label").into(),
                                    egui::FontId::proportional(14.0),
                                    look.text,
                                )
                                .size()
                                .x
                                + 2.0 * look.s
                        })
                        .collect();
                    let total = widths.iter().sum::<f32>() + look.xs * (choices.len() - 1) as f32;
                    let (area, _) =
                        ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::hover());
                    let mut x = area.center().x - total / 2.0;
                    for (index, (choice, width)) in choices.iter().zip(widths).enumerate() {
                        let rect =
                            Rect::from_min_size(egui::pos2(x, area.top()), vec2(width, 32.0));
                        x += width + look.xs;
                        let r =
                            ui.interact(rect, ui.scope_id().with((id, key, index)), Sense::click());
                        if r.hovered() {
                            ui.painter()
                                .rect_filled(rect, Look::radius(look.r_s), look.card_hover);
                        }
                        let selected = choice["value"] == row["value"];
                        ui.painter().text(
                            rect.center(),
                            egui::Align2::CENTER_CENTER,
                            text(choice, "label"),
                            egui::FontId::proportional(14.0),
                            look.text,
                        );
                        if selected {
                            ui.painter().hline(
                                rect.x_range(),
                                rect.bottom() - 1.0,
                                Stroke::new(2.0, look.accent),
                            );
                        }
                        r.widget_info(|| {
                            egui::WidgetInfo::selected(
                                egui::Role::RadioButton,
                                ui.is_enabled(),
                                selected,
                                text(choice, "label"),
                            )
                        });
                        geometry.insert(format!("{id}.row.{key}.choice.{index}"), rect);
                        if r.clicked() && !selected {
                            events.push(Intent {
                                id: id.into(),
                                action: "edit".into(),
                                value: json!({"row":key,"value":choice["value"]}),
                            });
                        }
                    }
                }
            })
            .response;
        return response;
    }
    if row["kind"] == "media-chooser" {
        let response = ui
            .vertical(|ui| {
                let send = |events: &mut Vec<Intent>, action: &str, value: Value| {
                    events.push(Intent {
                        id: id.into(),
                        action: "edit".into(),
                        value: json!({"row":key,"value":{"action":action,"value":value}}),
                    })
                };
                ui.horizontal(|ui| {
                    ui.set_width(ui.available_width());
                    let folder = row["value"]["kind"] == "folder";
                    let title = text(row, "display_value");
                    let width = ui
                        .painter()
                        .layout_no_wrap(title.into(), egui::FontId::proportional(14.0), look.text)
                        .size()
                        .x
                        + 32.0
                        + if folder { 24.0 } else { 0.0 };
                    let (rect, r) = ui.allocate_exact_size(vec2(width, 32.0), Sense::click());
                    let mut x = rect.left() + 4.0;
                    if folder {
                        toolkit::button::paint_icon(
                            ui,
                            toolkit::Icon::Folder,
                            egui::pos2(x + 8.0, rect.center().y),
                            16.0,
                            look.text,
                        );
                        x += 24.0;
                    }
                    ui.painter().text(
                        egui::pos2(x, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        title,
                        egui::FontId::proportional(14.0),
                        look.text,
                    );
                    toolkit::button::paint_icon(
                        ui,
                        toolkit::Icon::ChevronDown,
                        egui::pos2(rect.right() - 12.0, rect.center().y),
                        16.0,
                        look.text,
                    );
                    r.widget_info(|| {
                        egui::WidgetInfo::selected(
                            egui::Role::ComboBox,
                            ui.is_enabled(),
                            false,
                            text(row, "category_label"),
                        )
                    });
                    geometry.insert(format!("{id}.row.{key}.category"), rect);
                    egui::Popup::menu(&r).show(|ui| {
                        ui.set_min_width(width.max(200.0));
                        for (index, choice) in row["categories"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .enumerate()
                        {
                            if index == 2 {
                                ui.separator();
                                ui.label(
                                    egui::RichText::new(text(row, "recent_label"))
                                        .size(12.0)
                                        .color(look.text),
                                );
                            }
                            let on = choice["value"] == row["value"];
                            if toolkit::selectable(ui, on, text(choice, "label")).clicked() && !on {
                                send(events, "category", choice["value"].clone());
                                ui.close();
                            }
                        }
                        ui.separator();
                        if toolkit::selectable(ui, false, text(row, "folder_label")).clicked() {
                            if let Ok(request) =
                                toolkit::choose::request(&row["folder_request"].to_string())
                            {
                                state(ui, id).lock().unwrap().chooser = Some((
                                    key.into(),
                                    Some("folder".into()),
                                    toolkit::choose::choose(request),
                                ));
                            }
                            ui.close();
                        }
                    });
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if !text(row, "add_label").is_empty() {
                            let galley = ui.painter().layout_no_wrap(
                                text(row, "add_label").into(),
                                egui::FontId::proportional(14.0),
                                look.accent,
                            );
                            let (rect, r) = ui.allocate_exact_size(
                                vec2(galley.size().x + 20.0, 24.0),
                                Sense::click(),
                            );
                            ui.painter().galley(
                                egui::pos2(rect.left(), rect.center().y - galley.size().y / 2.0),
                                galley,
                                look.accent,
                            );
                            toolkit::button::paint_icon(
                                ui,
                                toolkit::Icon::MoveUpRight,
                                egui::pos2(rect.right() - 8.0, rect.center().y),
                                16.0,
                                look.accent,
                            );
                            r.widget_info(|| {
                                egui::WidgetInfo::labeled(
                                    egui::Role::Link,
                                    ui.is_enabled(),
                                    text(row, "add_label"),
                                )
                            });
                            geometry.insert(format!("{id}.row.{key}.add"), rect);
                            if r.clicked() {
                                if row["add_request"].is_object() {
                                    if let Ok(request) =
                                        toolkit::choose::request(&row["add_request"].to_string())
                                    {
                                        state(ui, id).lock().unwrap().chooser = Some((
                                            key.into(),
                                            Some("images".into()),
                                            toolkit::choose::choose(request),
                                        ));
                                    }
                                } else {
                                    send(events, "colour", Value::Null);
                                }
                            }
                        }
                    });
                });
                ui.add_space(look.m);
                if row["tiles"]["choices"]
                    .as_array()
                    .is_none_or(|a| a.is_empty())
                {
                    ui.label(
                        egui::RichText::new(text(row, "empty_label"))
                            .size(14.0)
                            .color(look.text),
                    );
                } else {
                    draw(ui, id, &row["tiles"], look, geometry, events);
                }
            })
            .response;
        geometry.insert(format!("{id}.row.{key}"), response.rect);
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
            match state.texture(ui.ctx(),text(row,"path"),thumbs::Size::at_least(300)) {
                Some(texture)=>{
                    if row["fit"]=="fit" {
                        ui.painter().rect_filled(screen,Look::radius(look.r_xs),Color32::BLACK);
                        let size=texture.size_vec2();let scale=(screen.width()/size.x.max(1.0)).min(screen.height()/size.y.max(1.0));
                        paint_texture(ui,Rect::from_center_size(screen.center(),size*scale),0.0,&texture,Rect::from_min_max(egui::Pos2::ZERO,egui::pos2(1.0,1.0)));
                    }else{paint_texture(ui,screen,look.r_xs,&texture,crop(texture.size_vec2(),screen.size()));}
                },
                None=>{ui.painter().rect_filled(screen,Look::radius(look.r_xs),look.card);}
            }
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
                    if let Some(texture)=state.texture(ui.ctx(),text(item,"path"),thumbs::Size::Normal) {paint_texture(ui,rect,look.r_s,&texture,crop(texture.size_vec2(),size));}
                    else {toolkit::button::paint_icon(ui,toolkit::Icon::Image,rect.center(),24.0,look.icon);}
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
fn crop(size:egui::Vec2,target:egui::Vec2)->Rect {
    let (image,wanted)=(size.x/size.y.max(1.0),target.x/target.y.max(1.0));
    if image>wanted {let w=wanted/image;Rect::from_min_max(egui::pos2((1.0-w)/2.0,0.0),egui::pos2((1.0+w)/2.0,1.0))}
    else {let h=image/wanted;Rect::from_min_max(egui::pos2(0.0,(1.0-h)/2.0),egui::pos2(1.0,(1.0+h)/2.0))}
}
fn paint_texture(ui:&Ui,rect:Rect,radius:f32,texture:&egui::TextureHandle,uv:Rect) {
    let shape=egui::epaint::RectShape::filled(rect,Look::radius(radius),Color32::WHITE);
    let mut tessellator=egui::epaint::Tessellator::new(ui.ctx().pixels_per_point(),Default::default(),[1,1],Vec::new());
    let mut mesh=egui::epaint::Mesh::default();tessellator.tessellate_rect(&shape,&mut mesh);
    for vertex in &mut mesh.vertices {
        let coverage=f32::from(vertex.color.a())/255.0;vertex.color=Color32::WHITE.gamma_multiply(coverage);
        let t=((vertex.pos-rect.min)/rect.size()).clamp(egui::Vec2::ZERO,egui::Vec2::splat(1.0));vertex.uv=uv.lerp_inside(t);
    }
    mesh.texture_id=texture.id();ui.painter().add(egui::Shape::mesh(mesh));
}
