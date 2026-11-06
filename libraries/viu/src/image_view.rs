// SPDX-License-Identifier: MIT OR Apache-2.0
//! Capped PNG preview display. The application chooses a cache entry; decoding
//! and GPU texture ownership belong to this shared renderer primitive.
use egui::{TextureHandle, Ui};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
type Key = (String, String);
pub fn validate(value: &Value) -> Result<(), String> {
    if !["none", "pending", "ready", "skipped", "failed"]
        .contains(&value["phase"].as_str().unwrap_or(""))
    {
        return Err("unknown image phase".into());
    }
    if value["phase"] == "ready"
        && value["path"]
            .as_str()
            .is_none_or(|s| s.len() > 4096 || !std::path::Path::new(s).is_absolute())
    {
        return Err("ready image requires an absolute cache path".into());
    }
    Ok(())
}
struct Slot {
    path: String,
    texture: Option<TextureHandle>,
    answered: bool,
}
pub(crate) struct Images {
    decoder: crate::thumbnail::Decoder<Key>,
    slots: BTreeMap<String, Slot>,
    visible: BTreeSet<String>,
}
impl Default for Images {
    fn default() -> Self {
        Self {
            decoder: crate::thumbnail::Decoder::new("viu-preview-decode"),
            slots: BTreeMap::new(),
            visible: BTreeSet::new(),
        }
    }
}
impl Images {
    pub fn document() -> Self {
        Self {
            decoder: crate::thumbnail::Decoder::with_decoder("viu-document-decode", |path| {
                let document = crate::annotation::document::Document::open(path).ok()?;
                let (w, h) = document.dimensions();
                Some(egui::ColorImage::from_rgba_unmultiplied(
                    [w as usize, h as usize],
                    document.original().as_raw(),
                ))
            }),
            ..Self::default()
        }
    }
    pub fn surface() -> Self {
        Self {
            decoder: crate::thumbnail::Decoder::with_decoder("viu-surface-decode", decode_surface),
            ..Self::default()
        }
    }
    pub fn states(&self) -> Value {
        serde_json::json!(self.slots.iter().map(|(id,s)|(id.clone(),serde_json::json!({"path":s.path,"phase":if s.texture.is_some(){"ready"}else if s.answered{"failed"}else{"pending"}}))).collect::<BTreeMap<_,_>>())
    }
    pub fn begin(&mut self) {
        self.visible.clear();
    }
    pub fn finish(&mut self) {
        self.slots.retain(|id, _| self.visible.contains(id));
        self.decoder.retain(
            &self
                .slots
                .iter()
                .map(|(id, s)| (id.clone(), s.path.clone()))
                .collect::<Vec<_>>(),
        );
    }
    pub fn draw(&mut self, ui: &mut Ui, id: &str, value: &Value) -> egui::Response {
        self.draw_internal(ui, id, value, false)
    }
    pub fn draw_fill(&mut self, ui: &mut Ui, id: &str, value: &Value) -> egui::Response {
        self.draw_internal(ui, id, value, true)
    }
    pub fn draw_canvas(
        &mut self,
        ui: &mut Ui,
        id: &str,
        value: &Value,
        events: &mut Vec<crate::widgets::Intent>,
    ) -> egui::Response {
        let response = self.draw_internal(ui, id, value, true);
        if value["phase"] != "ready" {
            return response;
        }
        use crate::annotation::{document::Point, viewport::Viewport};
        let view = Viewport::fit(
            (
                value["width"].as_u64().unwrap() as u32,
                value["height"].as_u64().unwrap() as u32,
            ),
            (response.rect.width(), response.rect.height()),
            value["zoom"].as_f64().unwrap() as f32,
            Point {
                x: value["pan"]["x"].as_f64().unwrap() as f32,
                y: value["pan"]["y"].as_f64().unwrap() as f32,
            },
        );
        let point = |p: egui::Pos2| {
            let mut p = view.image(Point {
                x: p.x - response.rect.min.x,
                y: p.y - response.rect.min.y,
            });
            p.x += value["crop"]["x"].as_f64().unwrap_or(0.0) as f32;
            p.y += value["crop"]["y"].as_f64().unwrap_or(0.0) as f32;
            p
        };
        let image_rect = egui::Rect::from_min_size(
            response.rect.min + egui::vec2(view.origin.x, view.origin.y),
            egui::vec2(view.width, view.height),
        );
        let to_view = |p: Point| {
            response.rect.min
                + egui::vec2(
                    view.origin.x
                        + (p.x - value["crop"]["x"].as_f64().unwrap_or(0.0) as f32) * view.scale,
                    view.origin.y
                        + (p.y - value["crop"]["y"].as_f64().unwrap_or(0.0) as f32) * view.scale,
                )
        };
        let selection = serde_json::from_value::<crate::annotation::document::Shape>(
            value["selection"].clone(),
        )
        .ok();
        let painter = ui.painter_at(response.rect);
        if let Some(shape) = &selection {
            let (l, t, r, b) = shape.bounds();
            outline(
                ui,
                &painter,
                egui::Rect::from_two_pos(
                    to_view(Point { x: l, y: t }),
                    to_view(Point { x: r, y: b }),
                )
                .expand((shape.width * view.scale / 2.0).max(2.0)),
            );
        }
        let gesture_id = egui::Id::unique(("canvas-points", id));
        let mut points = ui.data_mut(|d| d.get_temp::<Vec<Point>>(gesture_id).unwrap_or_default());
        let panning = ui.input(|i| {
            i.pointer.button_down(egui::PointerButton::Middle) || i.key_down(egui::Key::Space)
        });
        let mut emit = |action: &str, value: Value| {
            events.push(crate::widgets::Intent {
                id: id.into(),
                action: action.into(),
                value,
            })
        };
        let scale_id = egui::Id::unique(("canvas-scale", id));
        if ui.data_mut(|d| d.get_temp::<f32>(scale_id)) != Some(view.scale) {
            ui.data_mut(|d| d.insert_temp(scale_id, view.scale));
            emit("viewport", serde_json::json!({"scale":view.scale}));
        }
        if value["enabled"] != true {
            ui.data_mut(|d| d.remove::<Vec<Point>>(gesture_id));
            return response;
        }
        if response.hovered() {
            ui.ctx()
                .set_cursor_icon(if value["guide_mode"] == "select" {
                    egui::CursorIcon::Default
                } else {
                    egui::CursorIcon::Crosshair
                });
        }
        if response.hovered() {
            let zoom = ui.input(|i| i.zoom_delta());
            if (zoom - 1.0).abs() > 0.001 {
                emit("zoom", serde_json::json!({"factor":zoom}));
            }
            let wheel = ui.input(|i| i.smooth_scroll_delta);
            if (zoom - 1.0).abs() <= 0.001 && wheel != egui::Vec2::ZERO {
                emit("pan", serde_json::json!({"x":wheel.x,"y":wheel.y}));
            }
        }
        if panning && response.dragged() {
            let delta = ui.input(|i| i.pointer.delta());
            if delta != egui::Vec2::ZERO {
                emit("pan", serde_json::json!({"x":delta.x,"y":delta.y}));
            }
            points.clear();
        } else {
            if response.drag_started_by(egui::PointerButton::Primary) {
                points.clear();
                if let Some(p) = ui
                    .input(|i| i.pointer.press_origin())
                    .filter(|p| image_rect.contains(*p))
                {
                    points.push(point(p));
                }
            }
            if response.dragged_by(egui::PointerButton::Primary)
                && !points.is_empty()
                && points.len() < document_point_limit()
            {
                if let Some(p) = response.interact_pointer_pos() {
                    points.push(point(p));
                }
            }
            if response.drag_stopped_by(egui::PointerButton::Primary) && !points.is_empty() {
                if let Some(p) = response.interact_pointer_pos() {
                    points.push(point(p));
                }
                emit(
                    "gesture",
                    serde_json::json!({"points":points,"scale":view.scale}),
                );
                points.clear();
            }
            if response.clicked_by(egui::PointerButton::Primary) {
                if let Some(p) = response
                    .interact_pointer_pos()
                    .filter(|p| image_rect.contains(*p))
                {
                    emit(
                        "pick",
                        serde_json::json!({"point":point(p),"scale":view.scale}),
                    );
                }
            }
        }
        if points.len() > 1 {
            let a = to_view(points[0]);
            let b = to_view(*points.last().unwrap());
            let boxed = egui::Rect::from_two_pos(a, b);
            match value["guide_mode"].as_str() {
                Some("select") => {
                    if let Some(shape) = &selection {
                        let (l, t, r, bottom) = shape.bounds();
                        outline(
                            ui,
                            &painter,
                            egui::Rect::from_two_pos(
                                to_view(Point { x: l, y: t }),
                                to_view(Point { x: r, y: bottom }),
                            )
                            .translate(b - a),
                        );
                    }
                }
                Some("crop") => outline(ui, &painter, boxed),
                _ => {
                    if let Ok(mut shape) = serde_json::from_value::<
                        crate::annotation::document::Shape,
                    >(value["guide"].clone())
                    {
                        shape.points = points.clone();
                        draw_guide(&painter, &shape, a, b, view.scale, &to_view);
                    }
                }
            }
        }
        ui.data_mut(|d| d.insert_temp(gesture_id, points));
        response
    }
    fn draw_internal(
        &mut self,
        ui: &mut Ui,
        id: &str,
        value: &Value,
        fill: bool,
    ) -> egui::Response {
        self.visible.insert(id.into());
        let rect = ui.available_rect_before_wrap();
        if fill {
            if value["canvas"] == true {
                toolkit::canvas::surround(ui, rect);
            } else {
                let colour = if let Some(c) = value["colour"].as_array() {
                    egui::Color32::from_rgb(
                        c[0].as_u64().unwrap() as u8,
                        c[1].as_u64().unwrap() as u8,
                        c[2].as_u64().unwrap() as u8,
                    )
                } else {
                    crate::appearance::colour(
                        ui.ctx(),
                        value["colour"]
                            .as_str()
                            .unwrap()
                            .strip_prefix("design.")
                            .unwrap(),
                    )
                    .unwrap_or(ui.visuals().panel_fill)
                };
                ui.painter().rect_filled(rect, 0.0, colour);
            }
        }
        if value["phase"] != "ready" {
            self.slots.remove(id);
            if value["canvas"] == true {
                ui.painter().text(
                    rect.center(),
                    egui::Align2::CENTER_CENTER,
                    value["empty_text"].as_str().unwrap_or(""),
                    egui::TextStyle::Body.resolve(ui.style()),
                    ui.visuals().weak_text_color(),
                );
                return ui.allocate_rect(rect, egui::Sense::hover());
            }
            return ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover());
        }
        let path = value["path"].as_str().unwrap_or("");
        if self.slots.get(id).is_some_and(|s| s.path != path) {
            self.slots.remove(id);
        }
        if !self.slots.contains_key(id) {
            if self.slots.len() >= 16 {
                return ui.label("Preview capacity reached");
            }
            let key = (id.to_owned(), path.to_owned());
            let accepted = self.decoder.ask(&key, std::path::Path::new(path), ui.ctx());
            self.slots.insert(
                id.into(),
                Slot {
                    path: path.into(),
                    texture: None,
                    answered: !accepted,
                },
            );
        }
        for ((id, path), image) in self.decoder.drain() {
            if let Some(slot) = self.slots.get_mut(&id).filter(|s| s.path == path) {
                slot.answered = true;
                slot.texture = image.map(|image| {
                    ui.ctx().load_texture(
                        format!("preview:{id}:{path}"),
                        image,
                        egui::TextureOptions::LINEAR,
                    )
                });
            }
        }
        if let Some(texture) = self.slots.get(id).and_then(|s| s.texture.as_ref()) {
            let natural = texture.size_vec2() / ui.ctx().pixels_per_point();
            if fill {
                if value["canvas"] == true {
                    use crate::annotation::{document::Point, viewport::Viewport};
                    let vp = Viewport::fit(
                        (
                            value["width"].as_u64().unwrap() as u32,
                            value["height"].as_u64().unwrap() as u32,
                        ),
                        (rect.width(), rect.height()),
                        value["zoom"].as_f64().unwrap() as f32,
                        Point {
                            x: value["pan"]["x"].as_f64().unwrap() as f32,
                            y: value["pan"]["y"].as_f64().unwrap() as f32,
                        },
                    );
                    let image = egui::Rect::from_min_size(
                        rect.min + egui::vec2(vp.origin.x, vp.origin.y),
                        egui::vec2(vp.width, vp.height),
                    );
                    let painter = ui.painter_at(rect);
                    painter.rect_filled(image, 0.0, egui::Color32::WHITE);
                    painter.image(
                        texture.id(),
                        image,
                        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                    return ui.allocate_rect(rect, egui::Sense::click_and_drag());
                }
                let fit = value["fit"].as_str().unwrap();
                let scale = match fit {
                    "fill" => (rect.width() / natural.x).max(rect.height() / natural.y),
                    "fit" => (rect.width() / natural.x).min(rect.height() / natural.y),
                    _ => 1.0,
                };
                let size = if fit == "stretch" {
                    rect.size()
                } else {
                    natural * scale
                };
                let painter = ui.painter().with_clip_rect(rect);
                if fit == "tile" {
                    let cols = (rect.width() / size.x).ceil() as usize;
                    let rows = (rect.height() / size.y).ceil() as usize;
                    if cols.saturating_mul(rows) <= 4096 {
                        for y in 0..rows {
                            for x in 0..cols {
                                painter.image(
                                    texture.id(),
                                    egui::Rect::from_min_size(
                                        rect.min + egui::vec2(x as f32 * size.x, y as f32 * size.y),
                                        size,
                                    ),
                                    egui::Rect::from_min_max(
                                        egui::Pos2::ZERO,
                                        egui::pos2(1.0, 1.0),
                                    ),
                                    egui::Color32::WHITE,
                                );
                            }
                        }
                    }
                } else {
                    painter.image(
                        texture.id(),
                        egui::Rect::from_center_size(rect.center(), size),
                        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                        egui::Color32::WHITE,
                    );
                }
                return ui.allocate_rect(rect, egui::Sense::hover());
            }
            let scale = (ui.available_width() / natural.x)
                .min(240.0 / natural.y)
                .min(1.0);
            ui.vertical_centered(|ui| {
                ui.add(egui::Image::new(texture).fit_to_exact_size(natural * scale));
            })
            .response
        } else {
            if value["canvas"] == true {
                return ui.allocate_rect(rect, egui::Sense::hover());
            }
            ui.allocate_response(egui::Vec2::ZERO, egui::Sense::hover())
        }
    }
}
fn document_point_limit() -> usize {
    crate::annotation::document::MAX_POINTS - 1
}
fn outline(ui: &Ui, painter: &egui::Painter, rect: egui::Rect) {
    painter.rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(3.0, ui.visuals().panel_fill),
        egui::StrokeKind::Outside,
    );
    painter.rect_stroke(
        rect,
        0.0,
        egui::Stroke::new(1.0, ui.visuals().text_color()),
        egui::StrokeKind::Outside,
    );
}
fn draw_guide(
    painter: &egui::Painter,
    shape: &crate::annotation::document::Shape,
    a: egui::Pos2,
    b: egui::Pos2,
    scale: f32,
    view: &impl Fn(crate::annotation::document::Point) -> egui::Pos2,
) {
    use crate::annotation::document::Kind;
    let [r, g, bl, al] = shape.colour;
    let ink = egui::Color32::from_rgba_unmultiplied(r, g, bl, al);
    let stroke = egui::Stroke::new((shape.width * scale).max(1.0), ink);
    let boxed = egui::Rect::from_two_pos(a, b);
    match shape.kind {
        Kind::Redact => {
            painter.rect_filled(boxed, 0.0, ink.to_opaque());
        }
        Kind::Rectangle | Kind::Text => {
            painter.rect_stroke(boxed, 0.0, stroke, egui::StrokeKind::Middle);
        }
        Kind::Ellipse | Kind::Number => {
            painter.add(egui::Shape::ellipse_stroke(
                boxed.center(),
                boxed.size() / 2.0,
                stroke,
            ));
        }
        Kind::Pen | Kind::Highlighter => {
            let stroke = if shape.kind == Kind::Highlighter {
                egui::Stroke::new(stroke.width, ink.gamma_multiply(80.0 / 255.0))
            } else {
                stroke
            };
            painter.add(egui::Shape::line(
                shape.points.iter().copied().map(view).collect(),
                stroke,
            ));
        }
        Kind::Line | Kind::Arrow => {
            painter.line_segment([a, b], stroke);
            if shape.kind == Kind::Arrow {
                let angle = (b.y - a.y).atan2(b.x - a.x);
                let length = (shape.width * 4.0).max(12.0) * scale;
                for offset in [-0.5_f32, 0.5] {
                    let tip = egui::pos2(
                        b.x - length * (angle + offset).cos(),
                        b.y - length * (angle + offset).sin(),
                    );
                    painter.line_segment([tip, b], stroke);
                }
            }
        }
    }
}
pub fn validate_canvas(value: &Value) -> Result<(), String> {
    validate_fill(value)?;
    for key in ["selection", "guide"] {
        if !value[key].is_null() {
            let shape: crate::annotation::document::Shape =
                serde_json::from_value(value[key].clone()).map_err(|e| e.to_string())?;
            shape.validate()?;
        }
    }
    if value["canvas"] != true
        || !value["enabled"].is_boolean()
        || !value["zoom"]
            .as_f64()
            .is_some_and(|n| n.is_finite() && (0.1..=8.0).contains(&n))
        || ["x", "y"].iter().any(|k| {
            !value["pan"][k]
                .as_f64()
                .is_some_and(|n| n.is_finite() && n.abs() < 1e9)
        })
        || (value["phase"] == "ready"
            && ["width", "height"]
                .iter()
                .any(|k| !value[k].as_u64().is_some_and(|n| (1..=16384).contains(&n))))
    {
        return Err("invalid image canvas geometry".into());
    }
    Ok(())
}
pub fn admit_canvas(current: &Value, action: &str, payload: &Value) -> Result<(), String> {
    if (current["enabled"] != true && action != "viewport") || current["phase"] != "ready" {
        return Err("DISABLED".into());
    }
    let point = |p: &Value| {
        ["x", "y"].iter().all(|k| {
            p[k].as_f64()
                .is_some_and(|n| n.is_finite() && n.abs() < 1e9)
        })
    };
    let valid = match action {
        "viewport" => payload["scale"]
            .as_f64()
            .is_some_and(|n| n.is_finite() && n > 0.0),
        "pan" => point(payload),
        "zoom" => payload["factor"]
            .as_f64()
            .is_some_and(|n| n.is_finite() && (0.01..=100.0).contains(&n)),
        "pick" => {
            point(&payload["point"])
                && payload["scale"]
                    .as_f64()
                    .is_some_and(|n| n.is_finite() && n > 0.0)
        }
        "gesture" => {
            payload["points"].as_array().is_some_and(|p| {
                (2..=crate::annotation::document::MAX_POINTS).contains(&p.len())
                    && p.iter().all(point)
            }) && payload["scale"]
                .as_f64()
                .is_some_and(|n| n.is_finite() && n > 0.0)
        }
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err("invalid canvas intent".into())
    }
}

pub fn validate_fill(value: &Value) -> Result<(), String> {
    let design = matches!(
        value["colour"].as_str(),
        Some("design.canvas" | "design.dock" | "design.base")
    );
    if !(design
        || value["colour"].as_array().is_some_and(|c| {
            c.len() == 3
                && c.iter().all(|v| {
                    v.as_f64().is_some_and(|n| {
                        n.is_finite() && (0.0..=255.0).contains(&n) && n.fract() == 0.0
                    })
                })
        }))
        || !["fill", "fit", "centre", "tile", "stretch"]
            .contains(&value["fit"].as_str().unwrap_or(""))
    {
        return Err("image-fill requires RGB colour and fill/fit/centre/tile/stretch".into());
    }
    validate(value)
}
pub(crate) fn decode_surface(path: &std::path::Path) -> Option<egui::ColorImage> {
    let bytes = thumbs::cache::read_regular(path, 32 * 1024 * 1024).ok()?;
    let mut reader = image::ImageReader::new(std::io::Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    if !matches!(
        reader.format(),
        Some(image::ImageFormat::Png | image::ImageFormat::Jpeg | image::ImageFormat::WebP)
    ) {
        return None;
    }
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    let image = reader.decode().ok()?;
    if u64::from(image.width()) * u64::from(image.height()) > 16_777_216 {
        return None;
    }
    let image = image.to_rgba8();
    Some(egui::ColorImage::from_rgba_unmultiplied(
        [image.width() as usize, image.height() as usize],
        image.as_raw(),
    ))
}

pub(crate) fn decode_document(path: &std::path::Path) -> Option<egui::ColorImage> {
    let doc=crate::annotation::document::Document::open(path).ok()?;
    let (w,h)=doc.dimensions();
    Some(egui::ColorImage::from_rgba_unmultiplied([w as usize,h as usize],doc.original().as_raw()))
}
