// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared native window controls. The compositor retains movement, resize,
//! focus and presentation authority; this sends the usual viewport commands.
use egui::{Rect, Ui, ViewportCommand as Command, pos2, vec2};
use serde_json::{Value, json};
use std::collections::BTreeMap;

pub fn is_client(settings: &Value) -> bool {
    design::decorations::resolve_with_actions(
        settings["window.decorations"]
            .as_str()
            .and_then(design::RequestedDecorations::from_name),
        design::RequestedDecorations::Auto,
        design::ActionsPlace::resolve(settings["window.actions_place"].as_str().unwrap_or("row")),
    )
    .mode
        == design::Decorations::Client
}
pub fn client_of(ctx: &egui::Context) -> bool {
    ctx.data(|d| d.get_temp::<bool>(egui::Id::unique("viu-window-client")))
        .unwrap_or(true)
}

#[derive(Clone, Copy)]
pub struct Metrics {
    pub height: f32,
    pub caption_width: f32,
    pub caption_glyph: f32,
    pub title_size: f32,
    pub resize_edge: f32,
    pub resize_corner: f32,
}
impl Metrics {
    pub fn of(ctx: &egui::Context) -> Self {
        ctx.data(|d| d.get_temp::<Self>(egui::Id::unique("viu-window-metrics")))
            .unwrap_or(Self {
                height: 38.0,
                caption_width: 36.0,
                caption_glyph: 14.0,
                title_size: 14.0,
                resize_edge: 4.0,
                resize_corner: 12.0,
            })
    }
}

#[derive(Default)]
pub struct WindowFrame {
    caption_alignment: design::title_band::Alignment,
    server: Option<bool>,
    client: bool,
    pub geometry: BTreeMap<String, Rect>,
    /// The owner persists/cancels work and acknowledges automation before
    /// closing its native viewport.
    pub close_requested: bool,
}
impl WindowFrame {
    /// Describe the controls actually drawn by this adapter. Requested hosted
    /// actions remain in the application row until native hosting is wired.
    pub fn form(&self, settings: &Value, service: Option<&str>) -> Value {
        json!({
            "place":settings["window.actions_place"].as_str().unwrap_or("row"),
            "effective":"row","hosted":false,"service":service,
            "title_align":design::TitleAlign::resolve(settings["window.title_align"].as_str().unwrap_or("auto")).alignment(design::WindowStyle::resolve(settings["window.style"].as_str().unwrap_or("mac")).title_alignment()).name(),
            "actions_align":design::ActionsAlign::resolve(settings["window.actions_align"].as_str().unwrap_or("right")).name(),
            "menu_side":design::MenuSide::resolve(settings["window.menu_side"].as_str().unwrap_or("left")).name(),
            "caption_side":settings["window.caption_side"].as_str().filter(|v|["left","center","right"].contains(v)).unwrap_or("right"),
            "requested":{"title_align":settings["window.title_align"].as_str().unwrap_or("auto"),"caption_side":settings["window.caption_side"].as_str().unwrap_or("auto")},
            "decorations":if self.client {"csd"}else{"ssd"},
            "bar":{"revision":0,"items":[]}
        })
    }
    pub fn show(&mut self, ui: &mut Ui, title: &str, settings: &Value) {
        self.show_with_header(ui, title, settings, false);
    }
    pub fn show_with_header(&mut self, ui: &mut Ui, title: &str, settings: &Value, header: bool) {
        self.show_with_chrome(ui, title, settings, header, false);
    }
    pub fn show_with_chrome(
        &mut self,
        ui: &mut Ui,
        title: &str,
        settings: &Value,
        header: bool,
        scene_captions: bool,
    ) {
        self.geometry.clear();
        self.close_requested = false;
        let requested = settings["window.decorations"]
            .as_str()
            .and_then(design::RequestedDecorations::from_name);
        let actions = design::ActionsPlace::resolve(
            settings["window.actions_place"].as_str().unwrap_or("row"),
        );
        let resolved = design::decorations::resolve_with_actions(
            requested,
            design::RequestedDecorations::Auto,
            actions,
        );
        self.client = resolved.mode == design::Decorations::Client;
        let server = !self.client;
        if self.server != Some(server) {
            ui.ctx().send_viewport_cmd(Command::Decorations(server));
            self.server = Some(server);
        }
        ui.ctx()
            .data_mut(|d| d.insert_temp(egui::Id::unique("viu-window-client"), self.client));
        if !self.client && !header {
            return;
        }
        if !self.client
            && header
            && toolkit::menu::form(ui.ctx())["hosted"] == true
            && toolkit::titlebar::hosting::hosted(ui.ctx())
        {
            return;
        }
        let metrics = Metrics::of(ui.ctx());
        let height = metrics.height;
        let gap = ui.spacing().item_spacing.y;
        ui.spacing_mut().item_spacing.y = 0.0;
        let (band, _) =
            ui.allocate_exact_size(vec2(ui.available_width(), height), egui::Sense::hover());
        ui.spacing_mut().item_spacing.y = gap;
        ui.painter().rect_filled(
            band,
            0.0,
            crate::appearance::colour(ui.ctx(), "chrome").unwrap_or(ui.visuals().panel_fill),
        );
        ui.painter().hline(
            band.x_range(),
            band.bottom() - 0.5,
            ui.visuals().widgets.noninteractive.bg_stroke,
        );
        if !self.client {
            self.geometry.insert("window.header".into(), band);
            let mut content = band;
            content.max.y -= ui.visuals().widgets.noninteractive.bg_stroke.width;
            self.geometry
                .insert("window.header.content".into(), content);
            return;
        }
        let alignment = design::CaptionSide::resolve(
            settings["window.caption_side"].as_str().unwrap_or("auto"),
        )
        .alignment(
            toolkit::chrome::Chrome::of(ui.ctx())
                .captions
                .alignment(design::title_band::Alignment::Right),
        );
        self.caption_alignment = alignment;
        ui.ctx()
            .data_mut(|d| d.insert_temp(egui::Id::unique("viu-caption-alignment"), alignment));
        let left = alignment == design::title_band::Alignment::Left;
        let caption_width = metrics.caption_width;
        let controls = Rect::from_min_size(
            pos2(
                match alignment {
                    design::title_band::Alignment::Left => band.left(),
                    design::title_band::Alignment::Center => band.center().x - 1.5 * caption_width,
                    design::title_band::Alignment::Right => band.right() - 3.0 * caption_width,
                },
                band.top(),
            ),
            vec2(
                3.0 * caption_width,
                (height - ui.visuals().widgets.noninteractive.bg_stroke.width).max(0.0),
            ),
        );
        self.geometry.insert("window.captions".into(), controls);
        let drag = if scene_captions {
            band
        } else {
            Rect::from_min_max(
                pos2(
                    if left { controls.right() } else { band.left() },
                    band.top(),
                ),
                pos2(
                    if left { band.right() } else { controls.left() },
                    band.bottom(),
                ),
            )
        };
        self.geometry.insert("window.title".into(), drag);
        if header {
            self.geometry.insert("window.header".into(), drag);
            let mut content = drag;
            content.max.y -= ui.visuals().widgets.noninteractive.bg_stroke.width;
            self.geometry
                .insert("window.header.content".into(), content);
        }
        let response = ui.interact(
            drag,
            ui.scope_id().with("viu-window-drag"),
            egui::Sense::click_and_drag(),
        );
        let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
        if response.double_clicked() {
            ui.ctx().send_viewport_cmd(Command::Maximized(!maximized));
        } else if response.drag_started() {
            ui.ctx().send_viewport_cmd(Command::StartDrag);
        }
        let font = crate::fonts::emphasis(
            ui.ctx(),
            egui::FontId::new(metrics.title_size, egui::FontFamily::Proportional),
        );
        let title = crate::elide::fit(ui, title, &font, (drag.width() - 24.0).max(0.0));
        if !header {
            ui.painter_at(drag).text(
                drag.center(),
                egui::Align2::CENTER_CENTER,
                title,
                font,
                ui.visuals().text_color(),
            );
        }
        if !scene_captions {
            let (close, geometry) = caption_controls(ui, controls, left, "window");
            self.close_requested = close;
            self.geometry.extend(geometry);
        }
    }
    /// Paint captions after a scene-authored header, preserving the full band
    /// for the shared layout solver while keeping native controls on top.
    pub fn finish_captions(&mut self, ui: &mut Ui, reserved: Option<Rect>) {
        if let Some(mut area) = self.geometry.get("window.captions").copied() {
            if self.caption_alignment == design::title_band::Alignment::Center {
                if let Some(rect) = reserved {
                    area.min.x = rect.min.x;
                    area.max.x = rect.max.x;
                }
            }
            self.geometry.insert("window.captions".into(), area);
            let (close, geometry) = caption_controls(
                ui,
                area,
                self.caption_alignment == design::title_band::Alignment::Left,
                "window",
            );
            self.close_requested |= close;
            self.geometry.extend(geometry);
        }
    }
    pub fn edges(&mut self, ui: &mut Ui) {
        if !self.client
            || ui.input(|i| {
                i.viewport().maximized.unwrap_or(false) || i.viewport().fullscreen.unwrap_or(false)
            })
        {
            return;
        }
        let metrics = Metrics::of(ui.ctx());
        let r = ui.ctx().content_rect();
        let e = metrics.resize_edge;
        let c = metrics.resize_corner;
        use egui::ResizeDirection as D;
        for (name, direction, rect) in [
            ("nw", D::NorthWest, Rect::from_min_size(r.min, vec2(c, c))),
            (
                "ne",
                D::NorthEast,
                Rect::from_min_size(pos2(r.right() - c, r.top()), vec2(c, c)),
            ),
            (
                "sw",
                D::SouthWest,
                Rect::from_min_size(pos2(r.left(), r.bottom() - c), vec2(c, c)),
            ),
            (
                "se",
                D::SouthEast,
                Rect::from_min_size(r.max - vec2(c, c), vec2(c, c)),
            ),
            (
                "n",
                D::North,
                Rect::from_min_max(
                    pos2(r.left() + c, r.top()),
                    pos2(r.right() - c, r.top() + e),
                ),
            ),
            (
                "s",
                D::South,
                Rect::from_min_max(
                    pos2(r.left() + c, r.bottom() - e),
                    pos2(r.right() - c, r.bottom()),
                ),
            ),
            (
                "w",
                D::West,
                Rect::from_min_max(
                    pos2(r.left(), r.top() + c),
                    pos2(r.left() + e, r.bottom() - c),
                ),
            ),
            (
                "e",
                D::East,
                Rect::from_min_max(
                    pos2(r.right() - e, r.top() + c),
                    pos2(r.right(), r.bottom() - c),
                ),
            ),
        ] {
            self.geometry.insert(format!("window.resize.{name}"), rect);
            let response = ui.interact(
                rect,
                ui.scope_id().with(("viu-resize", name)),
                egui::Sense::drag(),
            );
            if response.drag_started() {
                ui.ctx().send_viewport_cmd(Command::BeginResize(direction));
            }
        }
    }
}

/// Shared native caption control. Closing is an owner intent; minimise and
/// maximise use the existing native viewport commands.
pub fn caption_controls(
    ui: &mut Ui,
    area: Rect,
    left: bool,
    id: &str,
) -> (bool, BTreeMap<String, Rect>) {
    let metrics = Metrics::of(ui.ctx());
    let height = area.height();
    let band = area;
    let controls = area;
    let caption_width = area.width() / 3.0;
    let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
    let mut close_requested = false;
    let mut geometry = BTreeMap::new();
    let names = if left {
        ["close", "minimize", "maximize"]
    } else {
        ["minimize", "maximize", "close"]
    };
    for (i, name) in names.into_iter().enumerate() {
        let rect = Rect::from_min_size(
            pos2(controls.left() + i as f32 * caption_width, band.top()),
            vec2(caption_width, height),
        );
        geometry.insert(format!("{id}.caption.{name}"), rect);
        let r = ui.interact(
            rect,
            ui.scope_id().with(("viu-caption", id, name)),
            egui::Sense::click(),
        );
        r.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, true, name));
        let chrome = toolkit::chrome::Chrome::of(ui.ctx());
        let pressed = r.is_pointer_button_down_on();
        let hovered = r.hovered() || pressed;
        let palette = &chrome.palette;
        let (fill, glyph) = match name {
            "close" if pressed => (
                Some(
                    palette
                        .caption_close
                        .gamma_multiply(chrome.style.close_pressed_alpha as f32),
                ),
                palette.caption_close_text,
            ),
            "close" if hovered => (Some(palette.caption_close), palette.caption_close_text),
            _ if pressed => (Some(palette.pressed), palette.icon),
            _ if hovered => (Some(palette.hover), palette.icon),
            _ => (None, palette.icon),
        };
        let painter = ui.painter();
        if let Some(fill) = fill {
            painter.rect_filled(rect, chrome.style.square_radius, fill);
        }
        let b = Rect::from_center_size(
            rect.center(),
            vec2(metrics.caption_glyph, metrics.caption_glyph),
        );
        let line = egui::Stroke::new(chrome.style.hairline_width as f32, glyph);
        match name {
            "minimize" => {
                painter.line_segment([b.left_center(), b.right_center()], line);
            }
            "maximize" if maximized => {
                let front = Rect::from_min_max(
                    pos2(b.left(), b.top() + 2.0),
                    pos2(b.right() - 2.0, b.bottom()),
                );
                painter.rect_stroke(
                    front,
                    chrome.style.square_radius,
                    line,
                    egui::StrokeKind::Middle,
                );
                painter.add(egui::Shape::line(
                    vec![
                        pos2(front.left() + 2.0, front.top()),
                        pos2(front.left() + 2.0, b.top()),
                        b.right_top(),
                        pos2(b.right(), b.bottom() - 2.0),
                        pos2(front.right(), b.bottom() - 2.0),
                    ],
                    line,
                ));
            }
            "maximize" => {
                painter.rect_stroke(
                    b,
                    chrome.style.square_radius,
                    line,
                    egui::StrokeKind::Middle,
                );
            }
            _ => {
                painter.line_segment([b.left_top(), b.right_bottom()], line);
                painter.line_segment([b.right_top(), b.left_bottom()], line);
            }
        }
        if r.on_hover_text(name).clicked() {
            if name == "close" {
                close_requested = true;
            } else {
                ui.ctx().send_viewport_cmd(if name == "minimize" {
                    Command::Minimized(true)
                } else {
                    Command::Maximized(!maximized)
                });
            }
        }
    }
    (close_requested, geometry)
}

/// Apply an admitted native window intent. The owner handles close after its
/// own persistence, cancellation and reply-delivery rules.
pub fn apply_control(ctx: &egui::Context, action: &str) -> Result<bool, String> {
    match action {
        "close" => return Ok(true),
        "minimize" => ctx.send_viewport_cmd(Command::Minimized(true)),
        "toggle-maximize" => ctx.send_viewport_cmd(Command::Maximized(
            !ctx.input(|i| i.viewport().maximized.unwrap_or(false)),
        )),
        _ => return Err("unknown native window control".into()),
    }
    Ok(false)
}
