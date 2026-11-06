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
    chooser: Option<(String, toolkit::choose::Handle)>,
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
        .and_then(|(row, handle)| handle.poll(ui.ctx()).map(|answer| (row.clone(), answer)));
    if let Some((row, answer)) = outcome {
        state.chooser = None;
        match answer {
            toolkit::choose::Outcome::Chosen {paths,..}=>events.push(Intent {id:id.into(),action:"edit".into(),value:json!({"row":row,"value":paths.iter().filter_map(|p|p.to_str()).collect::<Vec<_>>()})}),
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
                    Some((key.into(), toolkit::choose::choose(request)));
            }
        }
        geometry.insert(format!("{id}.row.{key}.control"), response.rect);
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
