// SPDX-License-Identifier: MIT OR Apache-2.0
//! Renderer-neutral virtual tree/table. Rows, selection, commands and sorting
//! belong to the application; this control only draws and emits intents.
use crate::widgets::{Intent, Widget};
use egui::{Align2, FontId, Id, Rect, Sense, TextStyle, Ui, pos2, vec2};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}
fn number(v: &Value, key: &str, default: f32) -> f32 {
    v[key].as_f64().unwrap_or(default as f64) as f32
}
fn emit(events: &mut Vec<Intent>, id: &str, action: &str, value: Value) {
    events.push(Intent {
        id: id.into(),
        action: action.into(),
        value,
    });
}

pub fn validate(value: &Value) -> Result<(), String> {
    if text(value, "key").is_empty() {
        return Err("tree-table requires a container key".into());
    }
    let rows = value["rows"].as_array().ok_or("tree-table requires rows")?;
    let columns = value["columns"]
        .as_array()
        .filter(|c| !c.is_empty() && c.len() <= 16)
        .ok_or("tree-table requires 1..16 columns")?;
    let mut keys = BTreeSet::new();
    let mut column_keys = BTreeSet::new();
    for column in columns {
        if !column["fraction"].is_null()
            && column["fraction"]
                .as_f64()
                .is_none_or(|n| !n.is_finite() || n <= 0.0 || n > 1.0)
        {
            return Err("invalid column width fraction".into());
        }
        if text(column, "id").is_empty()
            || !column_keys.insert(text(column, "id"))
            || !column["label"].is_string()
        {
            return Err("invalid tree-table column".into());
        }
    }
    for row in rows {
        if text(row, "key").is_empty() || !keys.insert(text(row, "key")) {
            return Err("tree-table requires unique non-empty row keys".into());
        }
        let cells = row["cells"]
            .as_array()
            .ok_or("tree-table row requires cells")?;
        if cells.len() != columns.len()
            || cells
                .iter()
                .any(|c| c.as_str().is_none_or(|s| s.len() > 4096))
        {
            return Err("tree-table cell/column mismatch".into());
        }
        if row["depth"]
            .as_f64()
            .is_none_or(|d| !d.is_finite() || d < 0.0 || d > 256.0 || d.fract() != 0.0)
        {
            return Err("invalid tree-table depth".into());
        }
        if !row["branch"].is_boolean()
            || !row["selected"].is_boolean()
            || !row["focused"].is_boolean()
            || !row["expanded"].is_boolean()
        {
            return Err("tree-table flags must be bool".into());
        }
        if let Some(icon) = row["icon"].as_str() {
            if crate::icons::Icon::from_name(icon).is_none() {
                return Err("unknown tree-table icon".into());
            }
        }
    }
    if let Some(selected) = value.get("selected_keys") {
        let selected = selected
            .as_array()
            .ok_or("selected_keys must be an array")?;
        let mut seen = BTreeSet::new();
        for key in selected {
            let key = key.as_str().ok_or("selection key must be text")?;
            if !keys.contains(key) || !seen.insert(key) {
                return Err("unknown or duplicate selection key".into());
            }
        }
    }
    if let Some(focused) = value.get("focused_key") {
        if !focused.is_null() && focused.as_str().is_none_or(|key| !keys.contains(key)) {
            return Err("unknown focus key".into());
        }
    }
    Ok(())
}

fn selected(value: &Value, row: &Value) -> bool {
    value
        .get("selected_keys")
        .map_or(row["selected"] == true, |keys| {
            keys.as_array()
                .is_some_and(|keys| keys.contains(&row["key"]))
        })
}

fn focused(value: &Value, row: &Value) -> bool {
    value
        .get("focused_key")
        .map_or(row["focused"] == true, |key| key == &row["key"])
}

pub fn validate_properties(properties: &Value) -> Result<(), String> {
    for key in ["height", "row_height", "name_min", "metadata_scale"] {
        if let Some(value) = properties.get(key) {
            if value.as_f64().is_none_or(|v| {
                !v.is_finite()
                    || v < 0.0
                    || (["row_height", "metadata_scale"].contains(&key) && v == 0.0)
            }) {
                return Err(format!("invalid tree-table {key}"));
            }
        }
    }
    Ok(())
}

pub fn admit(value: &Value, action: &str, payload: &Value) -> Result<(), String> {
    validate(value)?;
    if ["ctrl", "shift"]
        .iter()
        .any(|key| payload.get(*key).is_some_and(|v| !v.is_boolean()))
    {
        return Err("selection modifiers must be bool".into());
    }
    let row = || {
        value["rows"]
            .as_array()
            .and_then(|rows| rows.iter().find(|r| r["key"] == payload["key"]))
            .ok_or("unknown row")
    };
    match action {
        "select" | "activate" | "context" => {
            row()?;
        }
        "expand" => {
            if row()?["branch"] != true {
                return Err("row is not expandable".into());
            }
        }
        "sort" => {
            if !value["columns"]
                .as_array()
                .is_some_and(|c| c.iter().any(|c| c["id"] == payload["column"]))
            {
                return Err("unknown column".into());
            }
        }
        "navigate" => {
            if ![
                "ArrowUp",
                "ArrowDown",
                "Home",
                "End",
                "ArrowLeft",
                "ArrowRight",
            ]
            .contains(&text(payload, "key"))
            {
                return Err("unknown navigation key".into());
            }
        }
        "drop" => {
            if !payload["source"].is_string()
                || !payload["destination"].is_string()
                || payload["keys"]
                    .as_array()
                    .is_none_or(|a| a.is_empty() || a.iter().any(|v| !v.is_string()))
            {
                return Err("invalid drop payload".into());
            }
        }
        "command" => {
            if payload["key"].is_string() {
                row()?;
            }
            if !value["context"]
                .as_array()
                .into_iter()
                .flatten()
                .chain(value["background_context"].as_array().into_iter().flatten())
                .any(|c| c["id"] == payload["command"] && c["enabled"] == true)
            {
                return Err("disabled or unknown context command".into());
            }
        }
        "activate-pane" => {}
        _ => return Err("unsupported tree-table action".into()),
    }
    Ok(())
}

#[derive(Clone, PartialEq)]
struct WidthKey {
    revision: Value,
    columns: Value,
    rows: usize,
    body: FontId,
    small: FontId,
}
#[derive(Clone)]
struct WidthCache {
    key: WidthKey,
    widths: Vec<f32>,
}
fn widths(ui: &Ui, id: &str, value: &Value, metadata_scale: f32) -> Vec<f32> {
    let columns = value["columns"].as_array().expect("validated columns");
    let rows = value["rows"].as_array().expect("validated rows");
    let key = WidthKey {
        revision: value["revision"].clone(),
        columns: value["columns"].clone(),
        rows: rows.len(),
        body: {
            let mut font = TextStyle::Body.resolve(ui.style());
            font.size *= metadata_scale;
            font
        },
        small: TextStyle::Small.resolve(ui.style()),
    };
    let cache_id = Id::unique((ui.ctx().viewport_id(), id, "column-widths"));
    if let Some(cache) = ui.data(|d| d.get_temp::<WidthCache>(cache_id)) {
        if cache.key == key {
            return cache.widths;
        }
    }
    let mut widths = Vec::with_capacity(columns.len());
    for (i, column) in columns.iter().enumerate() {
        let mut width = ui
            .painter()
            .layout_no_wrap(
                format!("{} ▼", text(column, "label")),
                key.small.clone(),
                ui.visuals().text_color(),
            )
            .size()
            .x
            + 12.0;
        if i != 0 {
            for row in rows {
                width = width.max(
                    ui.painter()
                        .layout_no_wrap(
                            row["cells"][i].as_str().unwrap_or("").into(),
                            key.body.clone(),
                            ui.visuals().text_color(),
                        )
                        .size()
                        .x
                        + 12.0,
                );
            }
        }
        widths.push(width.max(number(column, "min_width", 40.0)));
    }
    ui.data_mut(|d| {
        d.insert_temp(
            cache_id,
            WidthCache {
                key,
                widths: widths.clone(),
            },
        )
    });
    widths
}

#[derive(Clone)]
struct DragRows {
    source: String,
    keys: Vec<String>,
    label: String,
}
fn drop_target(
    ui: &Ui,
    response: &egui::Response,
    destination: &str,
    id: &str,
    events: &mut Vec<Intent>,
) {
    if let Some(payload) = response.dnd_release_payload::<DragRows>() {
        {
            emit(
                events,
                id,
                "drop",
                json!({"source":payload.source,"keys":payload.keys,"destination":destination,"ctrl":ui.input(|i|i.modifiers.ctrl||i.modifiers.command),"shift":ui.input(|i|i.modifiers.shift)}),
            );
        }
    }
}
fn icon(ui: &Ui, name: &str, rect: Rect, colour: egui::Color32) {
    if let Some(icon) = crate::icons::Icon::from_name(name) {
        crate::icons::image_of(ui.ctx(), icon, rect.height(), colour).paint_at(ui, rect);
    }
}
fn context(
    ui: &mut Ui,
    items: &Value,
    key: &Value,
    id: &str,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) {
    for item in items.as_array().into_iter().flatten() {
        if item["separator"] == true {
            ui.separator();
            continue;
        }
        let response = ui.add_enabled(
            item["enabled"] == true,
            egui::Button::new(text(item, "label")),
        );
        geometry.insert(format!("{id}.context.{}", text(item, "id")), response.rect);
        if response.clicked() {
            emit(
                events,
                id,
                "command",
                json!({"command":item["id"],"key":key}),
            );
            ui.close();
        }
    }
}

pub(crate) fn draw(
    ui: &mut Ui,
    widget: &Widget,
    value: &Value,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) -> egui::Response {
    if widget.properties["style"] == "grid" {
        return draw_grid(ui, widget, value, geometry, events);
    }
    let id = &widget.id;
    let rows = value["rows"].as_array().expect("validated rows");
    let columns = value["columns"].as_array().expect("validated columns");
    let metadata_scale = number(&widget.properties, "metadata_scale", 1.0);
    let mut measured = widths(ui, id, value, metadata_scale);
    let available = ui.available_width();
    let name_min = number(&widget.properties, "name_min", 140.0);
    let mut visible = columns.len();
    while visible > 1 && measured[1..visible].iter().sum::<f32>() + name_min > available {
        visible -= 1;
    }
    measured[0] = (available - measured[1..visible].iter().sum::<f32>()).max(20.0);
    let height = number(&widget.properties, "height", ui.available_height()).max(32.0);
    let row_h = number(
        &widget.properties,
        "row_height",
        (ui.text_style_height(&TextStyle::Body) + 6.0).round(),
    );
    let result=ui.allocate_ui(vec2(available,height),|ui| {
        ui.set_min_size(vec2(available,height));
        let header_h=ui.text_style_height(&TextStyle::Small)+8.0;
        let (header,_)=ui.allocate_exact_size(vec2(available,header_h),Sense::hover());
        let mut x=header.left();
        for i in 0..visible {
            let column=&columns[i];
            let cell=Rect::from_min_size(pos2(x,header.top()),vec2(measured[i],header_h));x+=measured[i];
            let response=ui.interact(cell,Id::unique((ui.ctx().viewport_id(),id,"column",i)),Sense::click());
            response.widget_info(||egui::WidgetInfo::labeled(egui::Role::Button,true,text(column,"label")));
            geometry.insert(format!("{id}.column.{}",text(column,"id")),cell);
            if response.hovered(){ui.painter().rect_filled(cell,2.0,ui.visuals().widgets.hovered.weak_bg_fill);}
            let right=text(column,"align")=="right";
            let arrow=if value["sort"]==column["id"] {if value["ascending"]==true {" ▲"}else{" ▼"}}else{""};
            ui.painter().text(pos2(if right{cell.right()-6.0}else{cell.left()+6.0},cell.center().y),if right{Align2::RIGHT_CENTER}else{Align2::LEFT_CENTER},format!("{}{}",text(column,"label"),arrow),TextStyle::Small.resolve(ui.style()),ui.visuals().weak_text_color());
            if response.clicked(){emit(events,id,"sort",json!({"column":column["id"]}));}
        }
        ui.painter().hline(header.x_range(),header.bottom(),ui.visuals().widgets.noninteractive.bg_stroke);
        let list_h=(height-header_h-ui.spacing().item_spacing.y).max(row_h);
        let body=Rect::from_min_size(ui.cursor().min,vec2(available,list_h));
        let background=ui.interact(body,Id::unique((ui.ctx().viewport_id(),id,"background")),Sense::click());
        geometry.insert(format!("{id}.background"),body);
        if background.clicked(){background.request_focus();emit(events,id,"activate-pane",Value::Null);}
        background.context_menu(|ui|context(ui,&value["background_context"],&Value::Null,id,geometry,events));
        egui::ScrollArea::vertical().id_salt((id,"rows")).auto_shrink([false,false]).max_height(list_h).show_rows(ui,row_h,rows.len(),|ui,range|{
            for index in range {
                let row=&rows[index];
                let key=text(row,"key");
                let (rect,response)=ui.allocate_exact_size(vec2(available,row_h),Sense::click_and_drag());
                geometry.insert(format!("{id}.row.{index}"),rect);
                geometry.insert(format!("{id}.key.{key}"),rect);
                response.widget_info(||egui::WidgetInfo::selected(egui::Role::Button,true,selected(value,row),row["cells"][0].as_str().unwrap_or("")));
                let is_selected=selected(value,row);
                let colour=if is_selected{ui.visuals().selection.stroke.color}else{ui.visuals().text_color()};
                let painter=ui.painter_at(rect);
                if is_selected||response.hovered(){painter.rect_filled(rect,2.0,if is_selected{ui.visuals().selection.bg_fill}else{ui.visuals().widgets.hovered.weak_bg_fill});}
                if focused(value,row)&&value["active"]==true{painter.rect_stroke(rect.shrink(0.5),2.0,egui::Stroke::new(1.0,ui.visuals().selection.stroke.color.gamma_multiply(0.6)),egui::StrokeKind::Inside);}
                let mut tx=rect.left()+4.0+number(row,"depth",0.0)*14.0;
                let chevron=Rect::from_center_size(pos2(tx+8.0,rect.center().y),vec2(16.0,16.0));
                let mut toggled=false;
                if row["branch"]==true {
                    let toggle=ui.interact(chevron,Id::unique((ui.ctx().viewport_id(),id,key,"expand")),Sense::click());
                    geometry.insert(format!("{id}.expand.{index}"),chevron);
                    icon(ui,if row["expanded"]==true{"chevron-down"}else{"chevron-right"},chevron.shrink(2.0),ui.visuals().weak_text_color());
                    if toggle.clicked(){emit(events,id,"expand",json!({"key":key}));toggled=true;}
                }
                tx+=18.0;
                let icon_rect=Rect::from_center_size(pos2(tx+8.0,rect.center().y),vec2(16.0,16.0));
                icon(ui,text(row,"icon"),icon_rect,if is_selected{colour}else if row["branch"]==true{ui.visuals().hyperlink_color}else{ui.visuals().weak_text_color()});
                tx+=22.0;
                let mut cx=rect.left();
                for i in 0..visible {
                    let right=cx+measured[i];
                    let cell=Rect::from_min_max(pos2(cx,rect.top()),pos2(right,rect.bottom()));
                    let mut font=TextStyle::Body.resolve(ui.style());
                    if i>0 {font.size*=metadata_scale;}
                    let text=row["cells"][i].as_str().unwrap_or("");
                    if i==0 {
                        let width=(right-tx-4.0).max(0.0);
                        let text=crate::elide::fit(ui,text,&font,width);
                        painter.with_clip_rect(cell.intersect(ui.clip_rect())).text(pos2(tx,rect.center().y),Align2::LEFT_CENTER,text,font,colour);
                    } else {
                        let right_align=text_value(&columns[i],"align")=="right";
                        painter.with_clip_rect(cell.intersect(ui.clip_rect())).text(pos2(if right_align{right-6.0}else{cx+6.0},rect.center().y),if right_align{Align2::RIGHT_CENTER}else{Align2::LEFT_CENTER},text,font,if is_selected{colour}else{ui.visuals().weak_text_color()});
                    }
                    cx=right;
                }
                let modifiers=ui.input(|i|i.modifiers);
                if !toggled&&response.clicked(){response.request_focus();emit(events,id,"select",json!({"key":key,"ctrl":modifiers.ctrl||modifiers.command,"shift":modifiers.shift}));}
                if !toggled&&response.double_clicked(){emit(events,id,"activate",json!({"key":key}));}
                if response.secondary_clicked(){emit(events,id,"context",json!({"key":key}));}
                if response.drag_started(){
                    let keys=if is_selected{rows.iter().filter(|r|selected(value,r)).map(|r|text_value(r,"key").to_owned()).collect()}else{vec![key.to_owned()]};
                    response.dnd_set_drag_payload(DragRows{source:id.clone(),keys,label:row["cells"][0].as_str().unwrap_or("").into()});
                }
                if row["branch"]==true {drop_target(ui,&response,key,id,events);}
                response.context_menu(|ui|context(ui,&value["context"],&row["key"],id,geometry,events));
            }
        });
        drop_target(ui,&background,text(value,"key"),id,events);
    });
    if let Some(payload) = egui::DragAndDrop::payload::<DragRows>(ui.ctx()) {
        if let Some(pos) = ui.ctx().pointer_hover_pos() {
            egui::Area::new(Id::unique((ui.ctx().viewport_id(), id, "drag-label")))
                .order(egui::Order::Tooltip)
                .fixed_pos(pos + vec2(12.0, 12.0))
                .interactable(false)
                .show(ui.ctx(), |ui| {
                    egui::Frame::popup(ui.style()).show(ui, |ui| {
                        ui.label(&payload.label);
                    });
                });
        }
    }
    result.response
}

/// Resizable, striped native table. Column sizing and cell semantics are
/// supplied by Mix; this renderer has no resource or application knowledge.
fn draw_grid(
    ui: &mut Ui,
    widget: &Widget,
    value: &Value,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) -> egui::Response {
    use egui_extras::{Column, TableBuilder};
    let id = &widget.id;
    let rows = value["rows"].as_array().expect("validated rows");
    let columns = value["columns"].as_array().expect("validated columns");
    let height = ui.text_style_height(&TextStyle::Body)
        + number(&widget.properties, "row_padding", 6.0).clamp(0.0, 32.0);
    ui.scope(|ui| {
        let read_only = widget.properties["read_only"] == true;
        let room = (ui.available_width()
            - ui.spacing().item_spacing.x * (columns.len().saturating_sub(1) as f32))
            .max(1.0);
        let mut table = TableBuilder::new(ui)
            .id_salt(id)
            .auto_shrink([widget.properties["auto_shrink"] == true; 2])
            .vscroll(widget.properties["vscroll"] != false)
            .striped(true)
            .resizable(widget.properties["resizable"] != false)
            .sense(if read_only {
                Sense::hover()
            } else {
                Sense::click()
            })
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center));
        for column in columns {
            let width = if column["remainder"] == true {
                Column::remainder()
            } else if let Some(fraction) = column["fraction"].as_f64() {
                Column::exact(room * fraction as f32)
            } else {
                Column::initial(number(column, "width", 90.0))
            };
            table = table.column(width.at_least(number(column, "min_width", 50.0)).clip(true));
        }
        table
            .header(height, |mut header| {
                for column in columns {
                    header.col(|ui| {
                        let arrow = if value["sort"] == column["id"] {
                            if value["reverse"] == true {
                                " ↑"
                            } else {
                                " ↓"
                            }
                        } else {
                            ""
                        };
                        let label =
                            egui::RichText::new(format!("{}{arrow}", text(column, "label")))
                                .strong();
                        let response = if read_only || column["sortable"] == false {
                            ui.label(label)
                        } else {
                            ui.add(egui::Button::new(label).frame(false))
                        };
                        let response = if let Some(tip) = column["tooltip"].as_str() {
                            response.on_hover_text(tip)
                        } else {
                            response
                        };
                        geometry
                            .insert(format!("{id}.column.{}", text(column, "id")), response.rect);
                        if !read_only && response.clicked() && column["sortable"] != false {
                            emit(events, id, "sort", json!({"column":column["id"]}));
                        }
                    });
                }
            })
            .body(|body| {
                body.rows(height, rows.len(), |mut row| {
                    let data = &rows[row.index()];
                    let key = text(data, "key");
                    row.set_selected(selected(value, data));
                    for (i, column) in columns.iter().enumerate() {
                        row.col(|ui| {
                            if i == 0 {
                                ui.add_space(number(data, "depth", 0.0) * 14.0);
                            }
                            let label =
                                egui::RichText::new(data["cells"][i].as_str().unwrap_or(""));
                            let label = if column["weak"] == true {
                                label.color(ui.visuals().weak_text_color())
                            } else {
                                label
                            };
                            if column["align"] == "right" {
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| ui.label(label),
                                );
                            } else {
                                if column["leading_gap"] == true {
                                    ui.add_space(ui.spacing().item_spacing.x);
                                }
                                let response = ui.label(label);
                                if let Some(tip) = data["tooltips"][i].as_str() {
                                    response.on_hover_text(tip);
                                }
                            }
                        });
                    }
                    let response = row.response();
                    geometry.insert(format!("{id}.key.{key}"), response.rect);
                    if !read_only && response.clicked() {
                        emit(events, id, "select", json!({"key":key}));
                    }
                });
            });
    })
    .response
}
fn text_value<'a>(v: &'a Value, key: &str) -> &'a str {
    text(v, key)
}
