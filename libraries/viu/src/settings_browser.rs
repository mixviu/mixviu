// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Data-authored page browser, cards and controls. Policy and writes belong to Mix.
//! Visual tokens adapted from gold Prefs cosmic.rs at 8779f3328.
use crate::widgets::Intent;
use design::cosmic_tables::{Density, Roundness};
use egui::{
    Align, Color32, CornerRadius, Layout, Rect, Response, RichText, Sense, Stroke, StrokeKind, Ui,
    pos2, vec2,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

fn rgba(c: design::SrgbColour) -> Color32 {
    let [r, g, b, a] = c.to_srgba8();
    Color32::from_rgba_unmultiplied(r, g, b, a)
}

fn over(top: Color32, alpha: f32, base: Color32) -> Color32 {
    let mix = |t: u8, b: u8| (f32::from(t) * alpha + f32::from(b) * (1.0 - alpha)).round() as u8;
    Color32::from_rgb(
        mix(top.r(), base.r()),
        mix(top.g(), base.g()),
        mix(top.b(), base.b()),
    )
}

/// What the window draws with: the derived theme's roles, the density's
/// spacing and the roundness tokens.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Look {
    pub dark: bool,
    pub window: Color32,
    pub sidebar: Color32,
    pub sidebar_selected: Color32,
    pub sidebar_hover: Color32,
    pub card: Color32,
    pub card_hover: Color32,
    pub divider: Color32,
    pub text: Color32,
    pub icon: Color32,
    pub value: Color32,
    pub accent: Color32,
    pub on_accent: Color32,
    pub button: Color32,
    pub button_hover: Color32,
    pub track_off: Color32,
    pub knob: Color32,
    pub field: Color32,
    pub field_border: Color32,
    pub swatch_border: Color32,
    /// The control tint in effect (neutral 5, re-hued by the tint).
    pub control: Color32,
    pub warning: Color32,
    /// Spacing tokens xxs, xs, s, m, l, xl.
    pub xxs: f32,
    pub xs: f32,
    pub s: f32,
    pub m: f32,
    pub l: f32,
    pub xl: f32,
    /// Radius tokens xs, s, m, xl.
    pub r_xs: f32,
    pub r_s: f32,
    pub r_m: f32,
    pub r_xl: f32,
    pub density: Density,
}

impl Look {
    /// The look at the supplied density and roundness values.
    pub fn of(theme: &toolkit::Theme, values: &Value) -> Self {
        let derived = design::seeds::derived_theme(theme.context());
        let p = &derived.palette;
        let density = match values["appearance.density"].as_str() {
            Some("compact") => Density::Compact,
            Some("spacious") => Density::Spacious,
            _ => Density::Standard,
        };
        let space = density.spacing();
        let radii = match values["appearance.roundness"].as_str() {
            Some("slightly_round") => Roundness::SlightlyRound.corner_radii(),
            // MixOS's Square is 0 on every token (key space F2).
            Some("square") => {
                let mut r = Roundness::Square.corner_radii();
                for token in [
                    &mut r.radius_xs,
                    &mut r.radius_s,
                    &mut r.radius_m,
                    &mut r.radius_l,
                    &mut r.radius_xl,
                ] {
                    *token = [0.0; 4];
                }
                r
            }
            _ => Roundness::Round.corner_radii(),
        };
        // The sidebar is the primary container (five steps over the
        // window), a list card the background component (eight): spec 4
        // measures #272727 and #2E2E2E on #1B1B1B (Fable P10).
        let sidebar = rgba(derived.primary.base);
        let neutral5 = rgba(p.neutral[5]);
        let text = rgba(derived.primary.on);
        Self {
            dark: derived.is_dark,
            window: rgba(derived.background.base),
            sidebar,
            sidebar_selected: over(neutral5, 0.2, sidebar),
            sidebar_hover: over(neutral5, 0.1, sidebar),
            card: rgba(derived.background.component.base),
            card_hover: rgba(derived.background.component.hover),
            control: derived.control(5).map_or(rgba(p.neutral[5]), rgba),
            divider: rgba(derived.primary.divider),
            text,
            icon: text.gamma_multiply(0.95),
            value: rgba(derived.primary.component.on),
            accent: rgba(derived.accent.base),
            on_accent: rgba(derived.accent.on),
            button: rgba(derived.button.base),
            button_hover: rgba(derived.button.hover),
            track_off: rgba(if derived.is_dark {
                p.neutral[6]
            } else {
                p.neutral[5]
            }),
            knob: rgba(p.neutral[2]),
            field: rgba(derived.primary.small_widget),
            field_border: rgba(p.neutral[4]),
            swatch_border: rgba(p.neutral[8]),
            warning: rgba(derived.warning.base),
            xxs: f32::from(space.space_xxs),
            xs: f32::from(space.space_xs),
            s: f32::from(space.space_s),
            m: f32::from(space.space_m),
            l: f32::from(space.space_l),
            xl: f32::from(space.space_xl),
            r_xs: radii.radius_xs[0] as f32,
            r_s: radii.radius_s[0] as f32,
            r_m: radii.radius_m[0] as f32,
            r_xl: radii.radius_xl[0] as f32,
            density,
        }
    }

    /// A one-line row's height (40 compact, 48, 56).
    pub fn row_height(&self) -> f32 {
        match self.density {
            Density::Compact => 40.0,
            Density::Standard => 48.0,
            Density::Spacious => 56.0,
        }
    }

    /// A row's side padding (16, 24, 32).
    pub fn row_pad(&self) -> f32 {
        self.m
    }

    pub fn radius(r: f32) -> CornerRadius {
        CornerRadius::same(r.clamp(0.0, 255.0) as u8)
    }
}

fn text<'a>(v: &'a Value, key: &str) -> &'a str {
    v[key].as_str().unwrap_or("")
}
fn parse_colour(value: &str) -> Option<Color32> {
    let hex = value.strip_prefix('#')?;
    if ![6, 8].contains(&hex.len()) || !hex.is_ascii() {
        return None;
    }
    Some(Color32::from_rgba_unmultiplied(
        u8::from_str_radix(&hex[0..2], 16).ok()?,
        u8::from_str_radix(&hex[2..4], 16).ok()?,
        u8::from_str_radix(&hex[4..6], 16).ok()?,
        if hex.len() == 8 {
            u8::from_str_radix(&hex[6..8], 16).ok()?
        } else {
            255
        },
    ))
}
fn array(v: &Value) -> impl Iterator<Item = &Value> {
    v.as_array().into_iter().flatten()
}
fn rows(value: &Value) -> impl Iterator<Item = &Value> {
    array(&value["sections"])
        .chain(array(&value["drawer"]["sections"]))
        .flat_map(|s| array(&s["rows"]))
        .flat_map(|row| std::iter::once(row).chain(array(&row["detail_rows"])))
}
pub fn validate(value: &Value) -> Result<(), String> {
    if !value.is_object()
        || !value["title"].is_string()
        || !value["pages"].is_array()
        || array(&value["pages"]).count() > 128
        || !value["sections"].is_array()
    {
        return Err("page browser requires title, pages and sections".into());
    }
    let mut ids = BTreeSet::new();
    for row in rows(value) {
        if text(row, "id").is_empty()
            || !ids.insert(text(row, "id"))
            || !row["title"].is_string()
            || ![
                "toggle",
                "choice",
                "previews",
                "swatches",
                "number",
                "slider",
                "steps",
                "text",
                "link",
                "button",
                "info",
                "plain-row",
                "image",
                "buttons",
                "records",
                "table",
                "notes",
                "multitext",
                "secret",
                "tiles",
                "media-preview",
                "media-chooser",
                "display-picker",
                "file-pick",
                "colour",
                "ordered",
                "pick",
                "pick-list",
                "append-list",
                "colour-editor",
            ]
            .contains(&text(row, "kind"))
            || array(&row["choices"]).count() > 4096
        {
            return Err("invalid or duplicate browser row".into());
        }
    }
    if ids.len() > 512 {
        return Err("browser row budget exceeded".into());
    }
    Ok(())
}
pub fn admit(value: &Value, action: &str, payload: &Value) -> Result<(), String> {
    let mut selected;
    let value = if payload["page"].is_string() && ["edit", "activate", "remove"].contains(&action) {
        if value["searching"] != true {
            return Err("search results are not open".into());
        }
        let group = array(&value["search_groups"])
            .find(|group| group["page"] == payload["page"])
            .ok_or("search result is no longer present")?;
        selected = value.clone();
        selected["sections"] = group["sections"].clone();
        selected["drawer"] = Value::Null;
        &selected
    } else {
        value
    };
    validate(value)?;
    match action {
        "query" if payload.as_str().is_some_and(|q| q.len() <= 65536) => Ok(()),
        "navigate" if array(&value["pages"]).any(|p| p["id"] == payload["page"]) => Ok(()),
        "edit" | "activate" | "remove" => {
            let row = rows(value)
                .find(|r| r["id"] == payload["row"])
                .ok_or("unknown browser row")?;
            if row["shown"] == false || !row["blocked"].is_null() {
                return Err("browser row is disabled".into());
            }
            if action == "remove"
                && !array(&row["choices"])
                    .any(|c| c["value"] == payload["value"] && c["removable"] == true)
            {
                return Err("tile cannot be removed".into());
            }
            if row["kind"] == "file-pick"
                && action == "edit"
                && !payload["value"].as_array().is_some_and(|paths| {
                    paths.len() <= 64
                        && paths
                            .iter()
                            .all(|p| p.as_str().is_some_and(|p| p.starts_with('/')))
                })
            {
                return Err("picker requires absolute paths".into());
            }
            if action == "edit"
                && !row["choices"].is_null()
                && !array(&row["choices"]).any(|c| c["value"] == payload["value"])
            {
                return Err("unknown browser choice".into());
            }
            if action == "edit" {
                match text(row, "kind") {
                    "toggle" if !payload["value"].is_boolean() => {
                        return Err("toggle requires boolean".into());
                    }
                    "number" | "slider" | "steps" if !payload["value"].is_number() => {
                        return Err("number requires number".into());
                    }
                    "text" | "multitext" | "secret" if !payload["value"].is_string() => {
                        return Err("field requires text".into());
                    }
                    _ => {}
                }
                if array(&row["choices"])
                    .any(|c| c["value"] == payload["value"] && c["enabled"] == false)
                {
                    return Err("choice is disabled".into());
                }
            }
            Ok(())
        }
        "drawer.close" => Ok(()),
        "focus.done" if value["focus_row"].is_string() => Ok(()),
        "problem" if payload["message"].is_string() => Ok(()),
        _ => Err("unknown browser action".into()),
    }
}
fn emit(events: &mut Vec<Intent>, id: &str, action: &str, value: Value) {
    events.push(Intent {
        id: id.into(),
        action: action.into(),
        value,
    });
}
fn icon(ui: &Ui, name: &str, at: egui::Pos2, colour: Color32) {
    if let Some(icon) = toolkit::Icon::from_name(name) {
        toolkit::button::paint_icon(ui, icon, at, 16.0, colour);
    }
}
fn record(geometry: &mut BTreeMap<String, Rect>, id: &str, suffix: &str, response: &Response) {
    geometry.insert(format!("{id}.{suffix}"), response.rect);
    let key = egui::Id::unique((id, "browser-focus"));
    if response
        .ctx
        .data(|d| d.get_temp::<String>(key))
        .is_some_and(|row| suffix == format!("row.{row}"))
    {
        response.scroll_to_me(Some(Align::Center));
        response.ctx.data_mut(|d| {
            d.remove_temp::<String>(key);
            d.insert_temp(key.with("done"), true);
        });
    }
}
pub(crate) fn draw(
    ui: &mut Ui,
    id: &str,
    value: &Value,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) -> Response {
    let area = ui.available_rect_before_wrap();
    let focus_key = egui::Id::unique((id, "browser-focus"));
    ui.ctx().data_mut(|d| {
        if let Some(row) = value["focus_row"].as_str() {
            d.insert_temp(focus_key, row.to_owned());
        } else {
            d.remove_temp::<String>(focus_key);
        }
    });
    crate::media_gallery::begin(ui, id);
    let mut look = Look::of(
        &crate::appearance::session_theme(ui.ctx()),
        &value["settings"],
    );
    // The author decides when a palette snapshot changes. Other browsers
    // continue following the live theme on every frame.
    if let Some(refresh) = value.get("look_refresh_key") {
        let key = egui::Id::unique((ui.ctx().viewport_id(), id, "browser-look"));
        if let Some((previous, cached)) = ui.ctx().data(|d| d.get_temp::<(Value, Look)>(key)) {
            if previous == *refresh {
                look = cached;
            }
        }
        ui.ctx()
            .data_mut(|d| d.insert_temp(key, (refresh.clone(), look)));
    }
    ui.painter().rect_filled(area, 0.0, look.window);
    let condensed = area.width() < 648.0;
    let mut content = area;
    if !condensed || value["sidebar_open"] == true {
        let card = Rect::from_min_max(
            pos2(area.left() + 8.0, area.top()),
            pos2(area.left() + 288.0, area.bottom() - 8.0),
        );
        ui.painter()
            .rect_filled(card, Look::radius(look.r_s), look.sidebar);
        let inner = card.shrink(look.xxs.max(4.0));
        let mut side = ui.new_child(
            egui::UiBuilder::new()
                .id_salt((id, "sidebar"))
                .max_rect(inner)
                .layout(Layout::top_down(Align::Min)),
        );
        side.spacing_mut().item_spacing.y = look.xxs.max(4.0);
        let mut query = text(value, "query").to_owned();
        egui::Frame::NONE
            .fill(look.window)
            .corner_radius(Look::radius(look.r_xl))
            .inner_margin(egui::Margin::symmetric(8, 4))
            .show(&mut side, |ui| {
                ui.horizontal(|ui| {
                    ui.add(crate::icons::image_of(
                        ui.ctx(),
                        crate::icons::Icon::from_name("search").unwrap(),
                        16.0,
                        look.icon,
                    ));
                    let r = ui.add(
                        egui::TextEdit::singleline(&mut query)
                            .id_salt((id, "query"))
                            .frame(egui::Frame::NONE)
                            .hint_text(text(value, "search_hint"))
                            .desired_width(ui.available_width() - 24.0),
                    );
                    record(geometry, id, "search", &r);
                    if value["search_focus"] == true {
                        r.request_focus();
                    }
                    if r.changed() && ui.is_enabled() {
                        emit(events, id, "query", json!(query));
                    }
                    if r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        emit(events, id, "query", json!(""));
                        ui.memory_mut(|m| m.surrender_focus(r.id));
                    }
                    if !query.is_empty() {
                        if ui
                            .add(
                                toolkit::button::IconButton::new(toolkit::Icon::X)
                                    .size(16.0)
                                    .tooltip(text(value, "search_clear")),
                            )
                            .clicked()
                        {
                            emit(events, id, "query", json!(""));
                        }
                    }
                });
            });
        egui::ScrollArea::vertical()
            .id_salt((id, "pages-scroll"))
            .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
            .show(&mut side, |ui| {
                for p in array(&value["pages"]).filter(|p| p["parent"].is_null()) {
                    let (rect, r) =
                        ui.allocate_exact_size(vec2(inner.width(), 32.0), Sense::click());
                    let fill = if p["id"] == value["root"] {
                        look.sidebar_selected
                    } else if r.hovered() {
                        look.sidebar_hover
                    } else {
                        Color32::TRANSPARENT
                    };
                    ui.painter().rect_filled(rect, Look::radius(look.r_s), fill);
                    let x = rect.left() + look.s;
                    icon(
                        ui,
                        text(p, "icon"),
                        pos2(x + 8.0, rect.center().y),
                        look.icon,
                    );
                    ui.painter().text(
                        pos2(x + 28.0, rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        text(p, "title"),
                        egui::FontId::proportional(14.0),
                        look.text,
                    );
                    record(geometry, id, &format!("page.{}", text(p, "id")), &r);
                    r.widget_info(|| {
                        egui::WidgetInfo::selected(
                            egui::Role::Tab,
                            ui.is_enabled(),
                            p["id"] == value["root"],
                            text(p, "title"),
                        )
                    });
                    if r.clicked() {
                        emit(events, id, "navigate", json!({"page":p["id"]}));
                    }
                }
            });
        content.min.x = area.left() + 296.0;
        ui.painter().vline(
            content.left() - 0.5,
            area.y_range(),
            ui.visuals().widgets.noninteractive.bg_stroke,
        );
    }
    if value["drawer"].is_object() {
        let width = (344.0 + 2.0 * look.l).min(content.width());
        let rect = Rect::from_min_max(pos2(content.right() - width, content.top()), content.max);
        let mut drawer = ui.new_child(
            egui::UiBuilder::new()
                .id_salt((id, "drawer"))
                .max_rect(rect.shrink2(vec2(look.l, look.s)))
                .layout(Layout::top_down(Align::Min)),
        );
        drawer.horizontal(|ui| {
            let font = toolkit::fonts::bound(ui.ctx(), toolkit::fonts::heading(20.0));
            ui.label(
                RichText::new(text(&value["drawer"], "title"))
                    .font(font)
                    .color(look.text),
            );
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .add(
                        toolkit::button::IconButton::new(toolkit::Icon::X)
                            .size(16.0)
                            .tooltip(text(value, "drawer_close")),
                    )
                    .clicked()
                {
                    emit(events, id, "drawer.close", Value::Null);
                }
            });
        });
        drawer.add_space(look.m);
        egui::ScrollArea::vertical()
            .id_salt((id, "drawer-scroll"))
            .show(&mut drawer, |ui| {
                cards(
                    ui,
                    id,
                    &value["drawer"]["sections"],
                    &look,
                    geometry,
                    events,
                );
            });
        content.max.x = rect.left();
    }
    let side = if condensed { look.s } else { look.l };
    let width = (content.width() - 2.0 * side).clamp(0.0, 800.0);
    let column = Rect::from_min_max(
        pos2(content.center().x - width / 2.0, content.top()),
        pos2(content.center().x + width / 2.0, content.bottom()),
    );
    let mut body = ui.new_child(
        egui::UiBuilder::new()
            .id_salt((id, "body"))
            .ui_stack_info(
                egui::UiStackInfo::new(egui::UiKind::CentralPanel)
                    .with_frame(egui::Frame::NONE.fill(look.window)),
            )
            .max_rect(column)
            .layout(Layout::top_down(Align::Min)),
    );
    body.set_width(width);
    if value["searching"] == true && !text(value, "query").trim().is_empty() {
        egui::ScrollArea::vertical()
            .id_salt((id, "results"))
            .auto_shrink([false, false])
            .show(&mut body, |ui| {
                if array(&value["search_groups"]).next().is_none() {
                    ui.label(
                        RichText::new(text(value, "search_empty"))
                            .size(14.0)
                            .color(look.text),
                    );
                }
                for group in array(&value["search_groups"]) {
                    if let Some(parent) = group["parent_title"].as_str() {
                        ui.horizontal(|ui| {
                            ui.add_space(6.0);
                            ui.label(RichText::new(parent).size(14.0).color(look.text));
                        });
                    }
                    let link = ui.add(
                        egui::Label::new(
                            RichText::new(text(group, "title"))
                                .size(14.0)
                                .color(look.text),
                        )
                        .sense(Sense::click()),
                    );
                    link.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::Role::Link, true, text(group, "title"))
                    });
                    if link.clicked() {
                        emit(
                            events,
                            id,
                            "navigate",
                            json!({"page":group["page"],"row":group["row"]}),
                        );
                    }
                    ui.add_space(8.0);
                    ui.painter().hline(
                        ui.available_rect_before_wrap().x_range(),
                        ui.cursor().min.y,
                        Stroke::new(2.0, look.divider),
                    );
                    ui.add_space(2.0 + look.xs);
                    for section in array(&group["sections"]) {
                        let before = events.len();
                        ui.horizontal(|ui| {
                            ui.add_space(look.l);
                            ui.vertical(|ui| {
                                cards(ui, id, &json!([section]), &look, geometry, events);
                            });
                        });
                        for event in &mut events[before..] {
                            if event.value.is_object() {
                                event.value["page"] = group["page"].clone();
                            }
                        }
                        ui.add_space(look.m);
                    }
                    ui.add_space(look.m);
                }
            });
    } else {
        if !value["parent"].is_null() {
            let r = body
                .horizontal(|ui| {
                    ui.spacing_mut().item_spacing.x = 4.0;
                    ui.add(crate::icons::image_of(
                        ui.ctx(),
                        crate::icons::Icon::from_name("chevron-left").unwrap(),
                        16.0,
                        look.text,
                    ));
                    ui.add(
                        egui::Label::new(
                            RichText::new(text(value, "parent_title"))
                                .color(look.text)
                                .size(14.0),
                        )
                        .sense(Sense::click()),
                    )
                })
                .inner;
            r.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::Role::Link,
                    ui.is_enabled(),
                    text(value, "parent_title"),
                )
            });
            record(geometry, id, "back", &r);
            if r.clicked() {
                emit(events, id, "navigate", json!({"page":value["parent"]}));
            }
            body.add_space(6.0);
        }
        let title = body.painter().layout_no_wrap(
            text(value, "title").into(),
            toolkit::fonts::bound(body.ctx(), toolkit::fonts::heading(24.0)),
            look.text,
        );
        let title_width = title.size().x;
        let (rect, _) = body.allocate_exact_size(vec2(width, 36.0), Sense::hover());
        body.painter().galley(
            pos2(rect.left(), rect.center().y - title.size().y / 2.0),
            title,
            look.text,
        );
        let extras_width: f32 = array(&value["extras"])
            .map(|e| {
                body.painter()
                    .layout_no_wrap(
                        text(e, "label").into(),
                        egui::FontId::proportional(14.0),
                        look.text,
                    )
                    .size()
                    .x
                    + 32.0
            })
            .sum();
        if extras_width > 0.0 {
            let slot = if title_width + look.l + extras_width <= width {
                rect
            } else {
                body.allocate_exact_size(vec2(width, 28.0), Sense::hover())
                    .0
            };
            body.scope_builder(
                egui::UiBuilder::new()
                    .max_rect(slot)
                    .layout(Layout::right_to_left(Align::Center)),
                |ui| {
                    ui.spacing_mut().item_spacing.x = 8.0;
                    for extra in array(&value["extras"])
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                    {
                        if extra["link"] == true {
                            let ink = if extra["enabled"] == true {
                                look.accent
                            } else {
                                look.accent.gamma_multiply(0.5)
                            };
                            let galley = ui.painter().layout_no_wrap(
                                text(extra, "label").into(),
                                egui::FontId::proportional(14.0),
                                ink,
                            );
                            let (rect, response) = ui.allocate_exact_size(
                                vec2(galley.size().x + 20.0, 24.0),
                                Sense::hover(),
                            );
                            ui.painter().galley(
                                pos2(rect.left(), rect.center().y - galley.size().y / 2.0),
                                galley,
                                ink,
                            );
                            toolkit::button::paint_icon(
                                ui,
                                toolkit::Icon::MoveUpRight,
                                pos2(rect.right() - 8.0, rect.center().y),
                                16.0,
                                ink,
                            );
                            response.widget_info(|| {
                                egui::WidgetInfo::labeled(
                                    egui::Role::Link,
                                    extra["enabled"] == true,
                                    text(extra, "label"),
                                )
                            });
                        } else {
                            ui.add_enabled_ui(extra["enabled"] == true, |ui| {
                                standard_button(ui, text(extra, "label"), &look);
                            });
                        }
                    }
                },
            );
        }
        body.add_space(look.m);
        if value["scroll_content"] == false {
            cards(&mut body, id, &value["sections"], &look, geometry, events);
        } else {
            egui::ScrollArea::vertical()
                .id_salt((id, "page", text(value, "page")))
                .auto_shrink([false, false])
                .show(&mut body, |ui| {
                    ui.set_width(width);
                    for notice in array(&value["notices"]).filter_map(Value::as_str) {
                        browser_notice(ui, &look, notice);
                    }
                    if let Some(notice) = value["notice"].as_str().filter(|s| !s.is_empty()) {
                        browser_notice(ui, &look, notice);
                    }
                    if value["pending"].is_object() {
                        egui::Frame::NONE
                            .fill(look.card)
                            .corner_radius(Look::radius(look.r_s))
                            .inner_margin(egui::Margin::symmetric(look.row_pad() as i8, 4))
                            .show(ui, |ui| {
                                ui.set_width((width - 2.0 * look.row_pad()).max(0.0));
                                ui.set_min_height(32.0);
                                ui.spacing_mut().item_spacing.y = 0.0;
                                ui.vertical(|ui| {
                                    ui.label(
                                        RichText::new(text(&value["pending"], "title"))
                                            .size(14.0)
                                            .color(look.text),
                                    );
                                    ui.label(
                                        RichText::new(text(&value["pending"], "description"))
                                            .size(12.0)
                                            .color(look.text),
                                    );
                                });
                            });
                    }
                    for (index, p) in array(&value["children"]).enumerate() {
                        if index > 0 {
                            ui.add_space(look.s);
                        }
                        let (rect, r) = ui
                            .allocate_exact_size(vec2(width, 40.0 + 2.0 * look.s), Sense::click());
                        ui.painter().rect_filled(
                            rect,
                            Look::radius(look.r_s),
                            if r.hovered() {
                                look.card_hover
                            } else {
                                look.card
                            },
                        );
                        if let Some(glyph) = toolkit::Icon::from_name(text(p, "icon")) {
                            toolkit::button::paint_icon(
                                ui,
                                glyph,
                                pos2(rect.left() + look.m + 18.0, rect.center().y),
                                20.0,
                                look.icon,
                            );
                        }
                        let x = rect.left() + look.m + 40.0;
                        ui.painter().text(
                            pos2(x, rect.center().y - 1.0),
                            egui::Align2::LEFT_BOTTOM,
                            text(p, "title"),
                            egui::FontId::proportional(14.0),
                            look.text,
                        );
                        if !text(p, "description").is_empty() {
                            ui.painter().text(
                                pos2(x, rect.center().y + 1.0),
                                egui::Align2::LEFT_TOP,
                                text(p, "description"),
                                egui::FontId::proportional(12.0),
                                look.text,
                            );
                        }
                        toolkit::button::paint_icon(
                            ui,
                            toolkit::Icon::ChevronRight,
                            pos2(rect.right() - look.m - 18.0, rect.center().y),
                            20.0,
                            look.icon,
                        );
                        r.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::Role::Link,
                                ui.is_enabled(),
                                text(p, "title"),
                            )
                        });
                        focus_ring(ui, &r, rect, look.r_s, &look);
                        record(geometry, id, &format!("child.{}", text(p, "id")), &r);
                        if r.clicked() {
                            emit(events, id, "navigate", json!({"page":p["id"]}));
                        }
                    }
                    cards(ui, id, &value["sections"], &look, geometry, events);
                    if value["end_space"] == true {
                        ui.add_space(look.m);
                    }
                });
        }
    }
    if ui
        .ctx()
        .data_mut(|d| d.remove_temp::<bool>(focus_key.with("done")))
        .unwrap_or(false)
    {
        emit(events, id, "focus.done", Value::Null);
    }
    crate::media_gallery::finish(ui, id, events);
    ui.allocate_rect(area, Sense::hover())
}
fn cards(
    ui: &mut Ui,
    id: &str,
    sections: &Value,
    look: &Look,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) {
    for (section_index, section) in array(sections).enumerate() {
        if section["shown"] == false {
            continue;
        }
        if section["native_plain"] == true {
            if section_index > 0 {
                ui.add_space(look.l);
            }
            for (index, row) in array(&section["rows"])
                .filter(|r| r["shown"] != false && r["embedded"] != true)
                .enumerate()
            {
                if index > 0 {
                    ui.add_space(look.l);
                }
                ui.vertical(|ui| {
                    ui.set_width(ui.available_width());
                    ui.add_enabled_ui(row["blocked"].is_null(), |ui| {
                        crate::media_gallery::draw(ui, id, row, look, geometry, events);
                    });
                });
            }
            continue;
        }
        if section["panel_group"] == true {
            toolkit::panel::group(
                ui,
                text(section, "id"),
                &[text(section, "title")],
                |ui, _| {
                    for row in array(&section["rows"]).filter(|r| r["shown"] != false) {
                        draw_inline_row(ui, id, row, look, geometry, events);
                    }
                },
            );
            continue;
        }
        if section["plain"] == true {
            if let Some(title) = section["title"].as_str() {
                if section_index > 0 {
                    toolkit::bars::hairline(ui);
                }
                ui.add_space(ui.spacing().item_spacing.y);
                let size = egui::TextStyle::Body.resolve(ui.style()).size;
                let font = toolkit::fonts::bound(ui.ctx(), toolkit::fonts::heading(size * 1.1));
                ui.label(RichText::new(title).font(font).strong());
                ui.add_space(ui.spacing().item_spacing.y);
            }
            for row in array(&section["rows"]).filter(|r| r["shown"] != false) {
                if text(row, "kind") == "append-list" {
                    append_list(ui, id, row, look, geometry, events);
                } else if ["tiles", "media-preview", "file-pick"].contains(&text(row, "kind")) {
                    crate::media_gallery::draw(ui, id, row, look, geometry, events);
                } else {
                    draw_inline_row(ui, id, row, look, geometry, events);
                }
            }
            continue;
        }
        if section_index > 0 {
            ui.add_space(
                if array(sections)
                    .nth(section_index - 1)
                    .is_some_and(|s| s["native_plain"] == true)
                {
                    look.l
                } else {
                    look.m
                },
            );
        }
        if section["standalone"] == true {
            for row in array(&section["rows"]).filter(|r| r["shown"] != false) {
                let response = ui
                    .add_enabled_ui(row["blocked"].is_null(), |ui| {
                        standard_button(ui, text(row, "title"), look)
                    })
                    .inner;
                record(geometry, id, &format!("row.{}", text(row, "id")), &response);
                if response.clicked() {
                    emit(events, id, "activate", json!({"row":row["id"]}));
                }
            }
            continue;
        }
        if let Some(title) = section["title"].as_str() {
            let galley = ui.painter().layout_no_wrap(
                title.into(),
                toolkit::fonts::bound(ui.ctx(), toolkit::fonts::heading(14.0)),
                look.text,
            );
            let (rect, _) = ui.allocate_exact_size(vec2(galley.size().x, 21.0), Sense::hover());
            ui.painter().galley(
                pos2(rect.left(), rect.center().y - galley.size().y / 2.0),
                galley,
                look.text,
            );
            ui.add_space(look.xs);
        }
        if let Some(notice) = section["notice"].as_str() {
            browser_notice(ui, look, notice);
            if array(&section["rows"]).next().is_none() {
                continue;
            }
        }
        let visible: Vec<_> = array(&section["rows"])
            .filter(|r| r["shown"] != false)
            .collect();
        let quiet = visible.len() > 1
            && visible
                .iter()
                .all(|r| r.get("reason").unwrap_or(&r["blocked"]).is_string());
        if quiet {
            let mut said = BTreeSet::new();
            for row in &visible {
                if let Some(reason) = row.get("reason").unwrap_or(&row["blocked"]).as_str() {
                    if said.insert(reason) {
                        browser_notice(ui, look, reason);
                    }
                }
            }
        }
        egui::Frame::NONE
            .fill(look.card)
            .corner_radius(Look::radius(look.r_s))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.spacing_mut().item_spacing.y = 0.0;
                for (index, row) in array(&section["rows"])
                    .filter(|r| r["shown"] != false)
                    .filter(|r| {
                        r["kind"] != "ordered"
                            || array(&r["items"]).next().is_some()
                            || r.get("reason").unwrap_or(&r["blocked"]).is_string()
                    })
                    .enumerate()
                {
                    if index > 0 {
                        let y = ui.cursor().top();
                        ui.painter().hline(
                            ui.min_rect().x_range(),
                            y,
                            Stroke::new(1.0, look.divider),
                        );
                        ui.add_space(1.0);
                    }
                    if quiet {
                        let mut row = row.clone();
                        row["reason"] = Value::Null;
                        draw_row(ui, id, &row, look, geometry, events);
                    } else {
                        draw_row(ui, id, row, look, geometry, events);
                    }
                }
            });
        for row in &visible {
            if row["kind"] == "ordered" && row["add_label"].is_string() {
                ui.add_space(look.xs);
                ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
                    if ui
                        .add_enabled_ui(
                            row["blocked"].is_null()
                                && array(&row["items"]).count()
                                    < row["max_items"].as_u64().unwrap_or(64) as usize,
                            |ui| standard_button(ui, text(row, "add_label"), look),
                        )
                        .inner
                        .clicked()
                    {
                        emit(events, id, "activate", json!({"row":row["id"]}));
                    }
                });
            }
        }
    }
}
fn browser_notice(ui: &mut Ui, look: &Look, notice: &str) {
    ui.horizontal_wrapped(|ui| {
        ui.add(crate::icons::image_of(
            ui.ctx(),
            crate::icons::Icon::from_name("circle-alert").unwrap(),
            16.0,
            look.warning,
        ));
        ui.label(RichText::new(notice).size(12.0).color(look.text));
    });
    ui.add_space(look.xs);
}
struct Labels {
    title: std::sync::Arc<egui::Galley>,
    description: Option<std::sync::Arc<egui::Galley>>,
    reason: Option<std::sync::Arc<egui::Galley>>,
}
impl Labels {
    fn new(ui: &Ui, row: &Value, look: &Look, width: f32) -> Self {
        let lay = |s: &str, size, width: f32| {
            ui.painter().layout(
                s.into(),
                egui::FontId::proportional(size),
                look.text,
                width.max(1.0),
            )
        };
        Self {
            title: lay(text(row, "title"), 14.0, width),
            description: row["description"]
                .as_str()
                .filter(|s| !s.is_empty())
                .map(|s| lay(s, 12.0, width)),
            reason: row
                .get("reason")
                .unwrap_or(&row["blocked"])
                .as_str()
                .map(|s| lay(s, 12.0, width - 16.0)),
        }
    }
    fn height(&self) -> f32 {
        self.title.size().y
            + self.description.as_ref().map_or(0.0, |g| 2.0 + g.size().y)
            + self.reason.as_ref().map_or(0.0, |g| 2.0 + g.size().y)
    }
    fn paint(self, ui: &Ui, look: &Look, mut at: egui::Pos2) {
        let height = self.title.size().y;
        ui.painter().galley(at, self.title, look.text);
        at.y += height + 2.0;
        if let Some(g) = self.description {
            let h = g.size().y;
            ui.painter().galley(at, g, look.text);
            at.y += h + 2.0;
        }
        if let Some(g) = self.reason {
            toolkit::button::paint_icon(
                ui,
                toolkit::Icon::CircleAlert,
                at + vec2(6.0, 8.0),
                12.0,
                look.warning,
            );
            ui.painter().galley(at + vec2(16.0, 0.0), g, look.text);
        }
    }
}
fn focus_ring(ui: &Ui, response: &Response, rect: Rect, radius: f32, look: &Look) {
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.expand(1.0),
            Look::radius(radius),
            Stroke::new(1.0, look.accent),
            StrokeKind::Outside,
        );
    }
}
fn standard_button(ui: &mut Ui, text: &str, look: &Look) -> Response {
    let height = match look.density {
        Density::Compact => 24.0,
        Density::Standard => 32.0,
        Density::Spacious => 48.0,
    };
    let galley =
        ui.painter()
            .layout_no_wrap(text.into(), egui::FontId::proportional(14.0), look.text);
    let (rect, response) =
        ui.allocate_exact_size(vec2(galley.size().x + 24.0, height), Sense::click());
    let fill = if !ui.is_enabled() {
        look.button.gamma_multiply(0.5)
    } else if response.hovered() {
        look.button_hover
    } else {
        look.button
    };
    ui.painter()
        .rect_filled(rect, Look::radius(look.r_xl), fill);
    ui.painter().galley(
        rect.center() - galley.size() / 2.0,
        galley,
        if ui.is_enabled() {
            look.text
        } else {
            look.text.gamma_multiply(0.65)
        },
    );
    focus_ring(ui, &response, rect, look.r_xl, look);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), text));
    response
}
fn radio_rows(
    ui: &mut Ui,
    id: &str,
    row: &Value,
    look: &Look,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) {
    let key = text(row, "id");
    if let Some(reason) = row.get("reason").unwrap_or(&row["blocked"]).as_str() {
        egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(look.row_pad() as i8, 4))
            .show(ui, |ui| browser_notice(ui, look, reason));
        let y = ui.cursor().top();
        ui.painter()
            .hline(ui.min_rect().x_range(), y, Stroke::new(1.0, look.divider));
        ui.add_space(1.0);
    }
    for (index, choice) in array(&row["choices"]).enumerate() {
        if index > 0 {
            let y = ui.cursor().top();
            ui.painter()
                .hline(ui.min_rect().x_range(), y, Stroke::new(1.0, look.divider));
            ui.add_space(1.0);
        }
        let selected = choice["value"] == row["value"];
        let response = egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(look.row_pad() as i8, 4))
            .show(ui, |ui| {
                ui.set_min_height(32.0);
                ui.set_width(ui.available_width());
                ui.horizontal(|ui| {
                    ui.add_enabled_ui(row["blocked"].is_null(), |ui| {
                        let (rect, response) =
                            ui.allocate_exact_size(vec2(16.0, 16.0), Sense::click());
                        if selected {
                            ui.painter().circle_filled(rect.center(), 8.0, look.accent);
                            ui.painter()
                                .circle_filled(rect.center(), 3.0, look.on_accent);
                        } else {
                            ui.painter().circle_filled(rect.center(), 8.0, look.field);
                            ui.painter().circle_stroke(
                                rect.center(),
                                7.5,
                                Stroke::new(1.0, look.swatch_border),
                            );
                        }
                        response.widget_info(|| {
                            egui::WidgetInfo::selected(
                                egui::Role::RadioButton,
                                ui.is_enabled(),
                                selected,
                                text(choice, "label"),
                            )
                        });
                        if response.clicked() && !selected {
                            emit(
                                events,
                                id,
                                "edit",
                                json!({"row":key,"value":choice["value"]}),
                            );
                        }
                    });
                    ui.add_space(look.xs);
                    if text(choice, "description").is_empty() {
                        ui.label(
                            RichText::new(text(choice, "label"))
                                .size(14.0)
                                .color(look.text),
                        );
                    } else {
                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing.y = 2.0;
                            ui.label(
                                RichText::new(text(choice, "label"))
                                    .size(14.0)
                                    .color(look.text),
                            );
                            if let Some(description) =
                                choice["description"].as_str().filter(|s| !s.is_empty())
                            {
                                ui.label(RichText::new(description).size(12.0).color(look.text));
                            }
                        });
                    }
                });
            })
            .response;
        let click = ui.interact(
            response.rect,
            response.id.with((key, index)),
            Sense::click(),
        );
        record(geometry, id, &format!("row.{key}.choice.{index}"), &click);
        if selected {
            record(geometry, id, &format!("row.{key}"), &response);
        }
        if click.clicked() && row["blocked"].is_null() && !selected {
            emit(
                events,
                id,
                "edit",
                json!({"row":key,"value":choice["value"]}),
            );
        }
    }
}
fn draw_row(
    ui: &mut Ui,
    id: &str,
    row: &Value,
    look: &Look,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) {
    if row["kind"] == "plain-row" {
        egui::Frame::NONE
            .inner_margin(egui::Margin::symmetric(look.row_pad() as i8, 4))
            .show(ui, |ui| {
                ui.set_min_height(32.0);
                ui.vertical(|ui| {
                    ui.label(
                        RichText::new(text(row, "title"))
                            .size(14.0)
                            .color(look.text),
                    );
                });
            });
        return;
    }
    if row["kind"] == "choice" && row["presentation"] == "radio" {
        radio_rows(ui, id, row, look, geometry, events);
        return;
    }
    if row["kind"] == "ordered" {
        draw_ordered(ui, id, row, look, geometry, events);
        return;
    }
    let key = text(row, "id");
    let kind = text(row, "kind");
    let mut edited = None;
    let response = egui::Frame::NONE
        .inner_margin(egui::Margin::symmetric(look.row_pad() as i8, 4))
        .show(ui, |ui| {
            let width = ui.available_width();
            ui.set_width(width);
            let stacked = row["presentation"] != "dropdown"
                && matches!(kind, "choice" | "previews" | "swatches" | "image");
            let control_width = match kind {
                "toggle" => 48.0,
                "link" => 16.0,
                "colour" => {
                    if row["auto_link"] == true && row["value"] == "auto" {
                        text_width(ui, text(row, "auto_label"), 14.0) + 28.0
                    } else {
                        48.0
                    }
                }
                "number" => 96.0,
                "slider" | "steps" => 360.0,
                "choice" => {
                    array(&row["choices"])
                        .map(|c| text_width(ui, text(c, "label"), 14.0))
                        .fold(0.0_f32, f32::max)
                        + 33.0
                }
                "pick" => text_width(ui, text(row, "display_value"), 14.0) + 28.0,
                "info" => text_width(ui, text(row, "value"), 14.0),
                _ => 180.0,
            };
            let beside = !stacked && width - control_width - look.xs >= 200.0;
            if beside {
                let labels = Labels::new(ui, row, look, width - control_width - look.xs);
                let height = labels.height();
                let (rect, _) =
                    ui.allocate_exact_size(vec2(width, height.max(32.0)), Sense::hover());
                labels.paint(ui, look, pos2(rect.left(), rect.center().y - height / 2.0));
                let slot = Rect::from_min_max(
                    pos2(rect.right() - control_width - look.xs, rect.top()),
                    rect.max,
                );
                ui.scope_builder(
                    egui::UiBuilder::new()
                        .max_rect(slot)
                        .layout(Layout::right_to_left(Align::Center)),
                    |ui| {
                        ui.add_enabled_ui(row["blocked"].is_null(), |ui| {
                            edited = draw_control(ui, id, row, look, geometry, events);
                        });
                    },
                );
            } else {
                if kind != "previews" || !row.get("reason").unwrap_or(&row["blocked"]).is_null() {
                    let labels = Labels::new(ui, row, look, width);
                    let (rect, _) =
                        ui.allocate_exact_size(vec2(width, labels.height()), Sense::hover());
                    labels.paint(ui, look, rect.min);
                    ui.add_space(look.xs);
                }
                ui.horizontal(|ui| {
                    ui.add_enabled_ui(row["blocked"].is_null(), |ui| {
                        edited = draw_control(ui, id, row, look, geometry, events);
                    });
                });
            }
        })
        .response;
    record(geometry, id, &format!("row.{key}"), &response);
    if kind == "link" || kind == "pick" {
        let click = ui.interact(response.rect, response.id.with("go"), Sense::click());
        click.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::Role::Link,
                row["blocked"].is_null(),
                text(row, "title"),
            )
        });
        if click.clicked() && row["blocked"].is_null() {
            emit(events, id, "activate", json!({"row":key}));
        }
        focus_ring(ui, &click, response.rect, look.r_s, look);
    }
    if let Some(value) = edited {
        emit(events, id, "edit", json!({"row":key,"value":value}));
    }
}
fn draw_control(
    ui: &mut Ui,
    id: &str,
    row: &Value,
    look: &Look,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) -> Option<Value> {
    let key = text(row, "id");
    let mut edit = None;
    match text(row, "kind") {
        "previews" => {
            let gap = (look.xl * 3.0).min(ui.available_width() * 0.1);
            let width = 191.0_f32.min((ui.available_width() - gap) / 2.0).max(1.0);
            let (area, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 130.0), Sense::hover());
            let left = area.center().x - width - gap / 2.0;
            for (index, choice) in array(&row["choices"]).enumerate() {
                let dark = choice["value"] == "dark";
                let palette = if dark {
                    &design::cosmic_tables::DARK_PALETTE
                } else {
                    &design::cosmic_tables::LIGHT_PALETTE
                };
                let rect = Rect::from_min_size(
                    pos2(left + index as f32 * (width + gap), area.top()),
                    vec2(width, 100.0),
                );
                let r = ui.interact(rect, ui.scope_id().with((id, key, index)), Sense::click());
                ui.painter()
                    .rect_filled(rect, Look::radius(look.r_s), rgba(palette.gray_1));
                let inset = (rect.width() * 0.1).min(20.0);
                let inner = Rect::from_min_size(
                    rect.min + vec2(inset, 18.0),
                    vec2(rect.width() - 2.0 * inset, 64.0),
                );
                ui.painter().rect_filled(
                    inner,
                    Look::radius(look.r_s),
                    rgba(palette.neutral[if dark { 2 } else { 1 }]),
                );
                ui.painter().rect_filled(
                    Rect::from_min_size(inner.min + vec2(10.0, 10.0), vec2(36.0, 10.0)),
                    Look::radius(look.r_xs),
                    look.accent,
                );
                let selected = row["resolved_dark"]
                    .as_bool()
                    .map_or(choice["value"] == row["value"], |resolved| dark == resolved);
                if selected {
                    ui.painter().rect_stroke(
                        rect,
                        Look::radius(look.r_s),
                        Stroke::new(2.0, look.accent),
                        StrokeKind::Inside,
                    );
                }
                ui.painter().text(
                    pos2(rect.center().x, rect.bottom() + 8.0),
                    egui::Align2::CENTER_TOP,
                    text(choice, "label"),
                    egui::FontId::proportional(14.0),
                    look.text,
                );
                r.widget_info(|| {
                    egui::WidgetInfo::selected(
                        egui::Role::RadioButton,
                        ui.is_enabled(),
                        selected,
                        text(choice, "label"),
                    )
                });
                focus_ring(ui, &r, rect, look.r_s, look);
                record(geometry, id, &format!("row.{key}.choice.{index}"), &r);
                if r.clicked() && !selected {
                    edit = Some(choice["value"].clone());
                }
            }
        }
        "swatches" => {
            let palette = if row["resolved_dark"].as_bool().unwrap_or(look.dark) {
                &design::cosmic_tables::DARK_PALETTE
            } else {
                &design::cosmic_tables::LIGHT_PALETTE
            };
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(look.m, look.m);
                for (index, choice) in array(&row["choices"]).enumerate() {
                    let (rect, r) = ui.allocate_exact_size(vec2(48.0, 48.0), Sense::click());
                    let colour = palette
                        .swatch(text(choice, "palette"))
                        .map(rgba)
                        .or_else(|| parse_colour(text(choice, "colour")))
                        .unwrap_or(look.accent);
                    ui.painter()
                        .rect_filled(rect, Look::radius(look.r_s), colour);
                    let selected = choice["value"] == row["value"];
                    if selected {
                        toolkit::button::paint_icon(
                            ui,
                            toolkit::Icon::Check,
                            pos2(rect.left() + 12.0, rect.bottom() - 12.0),
                            16.0,
                            Color32::WHITE,
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
                    focus_ring(ui, &r, rect, look.r_s, look);
                    record(geometry, id, &format!("row.{key}.choice.{index}"), &r);
                    if r.clicked() && !selected {
                        edit = Some(choice["value"].clone());
                    }
                }
                if let Some(name) = row["custom_label"].as_str() {
                    let (rect, r) = ui.allocate_exact_size(vec2(48.0, 48.0), Sense::click());
                    if let Some(colour) = parse_colour(text(row, "value")) {
                        ui.painter()
                            .rect_filled(rect, Look::radius(look.r_s), colour);
                    } else {
                        ui.painter().rect_stroke(
                            rect,
                            Look::radius(look.r_s),
                            Stroke::new(1.0, look.swatch_border),
                            StrokeKind::Inside,
                        );
                        toolkit::button::paint_icon(
                            ui,
                            toolkit::Icon::Plus,
                            rect.center(),
                            16.0,
                            look.icon,
                        );
                    }
                    r.widget_info(|| {
                        egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), name)
                    });
                    record(geometry, id, &format!("row.{key}.custom"), &r);
                    if r.clicked() {
                        emit(events, id, "activate", json!({"row":key}));
                    }
                }
            });
        }
        "choice" if row["presentation"] == "schemes" => {
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = vec2(look.xs, look.xs);
                for (index, choice) in array(&row["choices"]).enumerate() {
                    let selected = choice["value"] == row["value"];
                    let galley = ui.painter().layout_no_wrap(
                        text(choice, "label").into(),
                        egui::FontId::proportional(13.0),
                        look.text,
                    );
                    let (rect, r) = ui.allocate_exact_size(
                        vec2((galley.size().x + 40.0).min(ui.max_rect().width()), 28.0),
                        Sense::click(),
                    );
                    ui.painter().rect_filled(
                        rect,
                        Look::radius(look.r_xl),
                        if selected {
                            look.sidebar_selected
                        } else {
                            look.field
                        },
                    );
                    if let Some((fill, ink)) = scheme_swatch(
                        text(choice, "value"),
                        row["resolved_dark"].as_bool().unwrap_or(look.dark),
                    ) {
                        let chip = Rect::from_center_size(
                            pos2(rect.left() + 16.0, rect.center().y),
                            vec2(14.0, 14.0),
                        );
                        ui.painter().rect_filled(chip, 3.0, fill);
                        if let Some(ink) = ink {
                            ui.painter().rect_filled(
                                Rect::from_min_max(pos2(chip.center().x, chip.top()), chip.max),
                                3.0,
                                ink,
                            );
                        }
                    }
                    ui.painter().galley(
                        pos2(rect.left() + 30.0, rect.center().y - galley.size().y / 2.0),
                        galley,
                        look.text,
                    );
                    if selected {
                        ui.painter().rect_stroke(
                            rect,
                            Look::radius(look.r_xl),
                            Stroke::new(1.0, look.accent),
                            StrokeKind::Inside,
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
                    focus_ring(ui, &r, rect, look.r_xl, look);
                    record(geometry, id, &format!("row.{key}.choice.{index}"), &r);
                    if r.clicked() && !selected {
                        edit = Some(choice["value"].clone());
                    }
                }
            });
        }
        "choice" if row["presentation"] == "segmented" => {
            let (rect, _) =
                ui.allocate_exact_size(vec2(ui.available_width(), 32.0), Sense::hover());
            ui.painter()
                .rect_filled(rect, Look::radius(look.r_xl), look.field);
            let each = rect.width() / array(&row["choices"]).count().max(1) as f32;
            for (index, choice) in array(&row["choices"]).enumerate() {
                let r = Rect::from_min_size(
                    pos2(rect.left() + each * index as f32, rect.top()),
                    vec2(each, 32.0),
                );
                let response = ui.interact(r, ui.scope_id().with((id, key, index)), Sense::click());
                let selected = choice["value"] == row["value"];
                if selected {
                    ui.painter().rect_filled(
                        r.shrink(1.0),
                        Look::radius(look.r_xl),
                        look.sidebar_selected,
                    );
                }
                let title = if selected {
                    format!("✓ {}", text(choice, "label"))
                } else {
                    text(choice, "label").to_owned()
                };
                ui.painter().text(
                    r.center(),
                    egui::Align2::CENTER_CENTER,
                    title,
                    egui::FontId::proportional(14.0),
                    look.text,
                );
                response.widget_info(|| {
                    egui::WidgetInfo::selected(
                        egui::Role::RadioButton,
                        ui.is_enabled(),
                        selected,
                        text(choice, "label"),
                    )
                });
                record(
                    geometry,
                    id,
                    &format!("row.{key}.choice.{index}"),
                    &response,
                );
                if response.clicked() && !selected {
                    edit = Some(choice["value"].clone());
                }
            }
        }
        "choice" if row["presentation"] == "images" => {
            images(
                ui,
                id,
                row,
                &array(&row["choices"]).collect::<Vec<_>>(),
                look,
                events,
                geometry,
            );
        }
        "choice" if row["presentation"] != "dropdown" => {
            ui.horizontal_wrapped(|ui| {
                for (index, choice) in array(&row["choices"]).enumerate() {
                    let r =
                        ui.selectable_label(choice["value"] == row["value"], text(choice, "label"));
                    record(geometry, id, &format!("row.{key}.choice.{index}"), &r);
                    if r.clicked() {
                        edit = Some(choice["value"].clone());
                    }
                }
            });
        }
        "toggle" => {
            let value = row["value"].as_bool().unwrap_or(false);
            let r = cosmic_switch(ui, value, look, text(row, "title"));
            record(geometry, id, &format!("row.{key}.control"), &r);
            if r.clicked() {
                edit = Some(json!(!value));
            }
        }
        "number" => {
            let min = row["min"].as_f64().unwrap_or(0.0);
            let max = row["max"].as_f64().unwrap_or(100.0);
            let value = row["value"]
                .as_f64()
                .filter(|v| v.is_finite())
                .unwrap_or(min)
                .round()
                .clamp(min, max);
            let step = row["step"].as_f64().unwrap_or(1.0);
            ui.spacing_mut().item_spacing.x = 4.0;
            let plus = cosmic_icon_button(
                ui,
                toolkit::Icon::Plus,
                value < max,
                &format!("{} +", text(row, "title")),
                look,
            );
            ui.label(
                RichText::new(value.to_string())
                    .size(14.0)
                    .color(look.value),
            );
            let minus = cosmic_icon_button(
                ui,
                toolkit::Icon::Minus,
                value > min,
                &format!("{} −", text(row, "title")),
                look,
            );
            record(geometry, id, &format!("row.{key}.plus"), &plus);
            record(geometry, id, &format!("row.{key}.minus"), &minus);
            if plus.clicked() {
                edit = Some(json!((value + step).min(max)));
            }
            if minus.clicked() {
                edit = Some(json!((value - step).max(min)));
            }
        }
        "steps" => {
            let steps: Vec<_> = array(&row["steps"]).collect();
            if !steps.is_empty() {
                let current = steps
                    .iter()
                    .position(|v| **v == row["value"])
                    .unwrap_or(steps.len() / 2) as f64;
                if let Some(next) = cosmic_slider(
                    ui,
                    (id, key),
                    current,
                    (0.0, (steps.len() - 1) as f64),
                    1.0,
                    false,
                    row["start"].as_str(),
                    row["end"].as_str(),
                    text(row, "title"),
                    look,
                ) {
                    edit =
                        Some(steps[(next.round().max(0.0) as usize).min(steps.len() - 1)].clone());
                }
            }
        }
        "slider" => {
            let scale = row["scale"].as_f64().unwrap_or(1.0);
            let value = row["value"]
                .as_f64()
                .unwrap_or(row["min"].as_f64().unwrap_or(0.0));
            let range = (
                row["min"].as_f64().unwrap_or(0.0),
                row["max"].as_f64().unwrap_or(100.0),
            );
            if let Some(next) = cosmic_slider(
                ui,
                (id, key),
                value * scale,
                (range.0 * scale, range.1 * scale),
                row["step"].as_f64().unwrap_or(0.0) * scale,
                row["show_value"] == true,
                row["start"].as_str(),
                row["end"].as_str(),
                text(row, "title"),
                look,
            ) {
                edit = Some(json!(
                    ((next / scale * 1e6).round() / 1e6).clamp(range.0, range.1)
                ));
            }
        }
        "text" => {
            let mut value = text(row, "value").to_owned();
            let r = ui.add(
                egui::TextEdit::singleline(&mut value)
                    .id_salt((id, key))
                    .desired_width(160.0),
            );
            record(geometry, id, &format!("row.{key}.control"), &r);
            if r.changed() && ui.is_enabled() {
                edit = Some(json!(value));
            }
        }
        "choice" => {
            let current = array(&row["choices"])
                .find(|c| c["value"] == row["value"])
                .map(|c| text(c, "label"))
                .unwrap_or(text(row, "value"));
            let width = array(&row["choices"])
                .map(|c| text_width(ui, text(c, "label"), 14.0))
                .fold(text_width(ui, current, 14.0), f32::max)
                + 33.0;
            let (rect, r) = ui.allocate_exact_size(vec2(width, 32.0), Sense::click());
            let ink = if ui.is_enabled() {
                look.value
            } else {
                look.value.gamma_multiply(0.65)
            };
            toolkit::button::paint_icon(
                ui,
                toolkit::Icon::ChevronDown,
                pos2(rect.right() - 17.0, rect.center().y),
                16.0,
                ink,
            );
            ui.painter().text(
                pos2(rect.right() - 33.0, rect.center().y),
                egui::Align2::RIGHT_CENTER,
                current,
                egui::FontId::proportional(14.0),
                ink,
            );
            focus_ring(ui, &r, rect, look.r_s, look);
            r.widget_info(|| {
                egui::WidgetInfo::selected(
                    egui::Role::ComboBox,
                    ui.is_enabled(),
                    false,
                    text(row, "title"),
                )
            });
            egui::Popup::menu(&r).show(|ui| {
                ui.set_min_width(width);
                for choice in array(&row["choices"]) {
                    let selected = choice["value"] == row["value"];
                    if toolkit::selectable(ui, selected, text(choice, "label")).clicked()
                        && !selected
                    {
                        edit = Some(choice["value"].clone());
                        ui.close();
                    }
                }
            });
            record(geometry, id, &format!("row.{key}.control"), &r);
        }
        "pick" => {
            ui.add(crate::icons::image_of(
                ui.ctx(),
                crate::icons::Icon::from_name("chevron-right").unwrap(),
                16.0,
                look.icon,
            ));
            ui.label(
                RichText::new(text(row, "display_value"))
                    .size(14.0)
                    .color(look.value),
            );
        }
        "colour" => {
            let auto = row["auto_link"] == true && row["value"] == "auto";
            let (rect, r) = ui.allocate_exact_size(
                vec2(
                    if auto {
                        text_width(ui, text(row, "auto_label"), 14.0) + 28.0
                    } else {
                        48.0
                    },
                    24.0,
                ),
                Sense::click(),
            );
            if auto {
                toolkit::button::paint_icon(
                    ui,
                    toolkit::Icon::ChevronRight,
                    pos2(rect.right() - 8.0, rect.center().y),
                    16.0,
                    look.icon,
                );
                ui.painter().text(
                    pos2(rect.right() - 20.0, rect.center().y),
                    egui::Align2::RIGHT_CENTER,
                    text(row, "auto_label"),
                    egui::FontId::proportional(14.0),
                    look.value,
                );
            } else {
                let colour =
                    parse_colour(text(row, "value")).unwrap_or(match text(row, "colour_role") {
                        "window" => look.window,
                        "card" => look.card,
                        "text" => look.text,
                        "control" => look.control,
                        _ => look.accent,
                    });
                ui.painter()
                    .rect_filled(rect, Look::radius(look.r_xs), colour);
                ui.painter().rect_stroke(
                    rect,
                    Look::radius(look.r_xs),
                    Stroke::new(1.0, look.swatch_border),
                    StrokeKind::Inside,
                );
            }
            focus_ring(ui, &r, rect, look.r_xs, look);
            r.widget_info(|| {
                egui::WidgetInfo::labeled(
                    if auto {
                        egui::Role::Link
                    } else {
                        egui::Role::Button
                    },
                    ui.is_enabled(),
                    text(row, "title"),
                )
            });
            record(geometry, id, &format!("row.{key}.control"), &r);
            if r.clicked() {
                emit(events, id, "activate", json!({"row":key}));
            }
        }
        "link" => {
            ui.add(crate::icons::image_of(
                ui.ctx(),
                crate::icons::Icon::from_name("chevron-right").unwrap(),
                16.0,
                look.icon,
            ));
        }
        "button" => {
            let r = ui.button(text(row, "title"));
            record(geometry, id, &format!("row.{key}.control"), &r);
            if r.clicked() {
                emit(events, id, "activate", json!({"row":key}));
            }
        }
        _ => {
            ui.label(row["value"].as_str().map(str::to_owned).unwrap_or_else(|| {
                if row["value"].is_null() {
                    String::new()
                } else {
                    row["value"].to_string()
                }
            }));
        }
    }
    edit
}
fn scheme_swatch(name: &str, dark: bool) -> Option<(Color32, Option<Color32>)> {
    type Cache = std::collections::HashMap<
        (design::Scheme, design::Mode),
        Option<(Color32, Option<Color32>)>,
    >;
    static CACHE: std::sync::OnceLock<std::sync::Mutex<Cache>> = std::sync::OnceLock::new();
    let scheme = design::Scheme::from_name(name)?;
    let mode = if dark {
        design::Mode::Dark
    } else {
        design::Mode::Light
    };
    let mut cache = CACHE
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    *cache.entry((scheme, mode)).or_insert_with(|| {
        let theme = toolkit::Theme::for_context(design::DesignContext {
            scheme,
            mode,
            ..Default::default()
        });
        theme.accent().map(|fill| {
            (
                rgba(fill),
                theme.dictionary().colours.pairs.get("accent").map(|p| {
                    let [r, g, b, a] = p.rendered_foreground.to_srgba8();
                    Color32::from_rgba_unmultiplied(r, g, b, a)
                }),
            )
        })
    })
}

fn cosmic_switch(ui: &mut Ui, on: bool, look: &Look, name: &str) -> Response {
    let (rect, response) = ui.allocate_exact_size(vec2(48.0, 24.0), Sense::click());
    let alpha = if ui.is_enabled() { 1.0 } else { 0.5 };
    ui.painter().rect_filled(
        rect,
        Look::radius(look.r_xl),
        if on { look.accent } else { look.track_off }.gamma_multiply(alpha),
    );
    let x = if on {
        rect.max.x - 12.0
    } else {
        rect.min.x + 12.0
    };
    ui.painter().circle_filled(
        pos2(x, rect.center().y),
        10.0,
        look.knob.gamma_multiply(alpha),
    );
    if response.hovered() && ui.is_enabled() {
        ui.painter().rect_filled(
            rect,
            Look::radius(look.r_xl),
            Color32::from_white_alpha(if look.dark { 26 } else { 0 }),
        );
    }
    if response.has_focus() {
        ui.painter().rect_stroke(
            rect.expand(1.0),
            Look::radius(look.r_xl),
            Stroke::new(1.0, look.accent),
            StrokeKind::Outside,
        );
    }
    response
        .widget_info(|| egui::WidgetInfo::selected(egui::Role::Switch, ui.is_enabled(), on, name));
    response
}

/// A compact two-column form, authored by the application. It shares native
/// input, accessibility and stable identities with the card controls above.
fn draw_inline_row(
    ui: &mut Ui,
    id: &str,
    row: &Value,
    look: &Look,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) {
    let key = text(row, "id");
    let mut edit = None;
    let response = ui
        .push_id((id, key), |ui| {
            ui.add_enabled_ui(row["blocked"].is_null(), |ui| {
                if text(row, "kind") == "colour-editor" {
                    colour_editor(ui, id, row, look, events);
                } else if text(row, "kind") == "pick-list" {
                    let filter_id = ui.make_persistent_id((id, key, "pick-filter"));
                    let mut query = ui
                        .data(|d| d.get_temp::<String>(filter_id))
                        .unwrap_or_default();
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut query)
                                .hint_text(text(row, "search_hint"))
                                .desired_width(ui.available_width()),
                        )
                        .changed()
                    {
                        ui.data_mut(|d| d.insert_temp(filter_id, query.clone()));
                    }
                    ui.add_space(look.s);
                    let choices: Vec<_> = array(&row["choices"])
                        .enumerate()
                        .filter(|(_, c)| {
                            text(c, "label")
                                .to_lowercase()
                                .contains(&query.to_lowercase())
                                && (!c["value"].is_null() || query.is_empty())
                        })
                        .collect();
                    egui::Frame::NONE
                        .fill(look.card)
                        .corner_radius(Look::radius(look.r_s))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            ui.spacing_mut().item_spacing.y = 0.0;
                            for (position, (index, choice)) in choices.iter().enumerate() {
                                if position > 0 {
                                    let y = ui.cursor().top();
                                    ui.painter().hline(
                                        ui.min_rect().x_range(),
                                        y,
                                        Stroke::new(1.0, look.divider),
                                    );
                                    ui.add_space(1.0);
                                }
                                let (rect, r) = ui.allocate_exact_size(
                                    vec2(ui.available_width(), 36.0),
                                    Sense::click(),
                                );
                                let selected = choice["value"] == row["value"];
                                ui.painter().text(
                                    rect.left_center() + vec2(look.m, 0.0),
                                    egui::Align2::LEFT_CENTER,
                                    text(choice, "label"),
                                    egui::FontId::proportional(14.0),
                                    look.text,
                                );
                                if selected {
                                    toolkit::button::paint_icon(
                                        ui,
                                        toolkit::Icon::Check,
                                        rect.right_center() - vec2(look.m + 8.0, 0.0),
                                        16.0,
                                        look.accent,
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
                                record(geometry, id, &format!("row.{key}.choice.{index}"), &r);
                                if r.clicked() && !selected {
                                    edit = Some(choice["value"].clone());
                                }
                            }
                        });
                } else if text(row, "kind") == "info" {
                    if row["separator_before"] == true {
                        toolkit::bars::hairline(ui);
                    }
                    let mut label = RichText::new(row["value"].as_str().unwrap_or(""));
                    label = if row["strong"] == true {
                        label.strong()
                    } else {
                        label.weak()
                    };
                    ui.add(egui::Label::new(label).wrap());
                } else if text(row, "kind") == "table" {
                    egui::Grid::new((id, key, "table"))
                        .num_columns(array(&row["columns"]).count())
                        .striped(true)
                        .spacing([18.0, 6.0])
                        .show(ui, |ui| {
                            for column in array(&row["columns"]) {
                                toolkit::panel::section_label(ui, column.as_str().unwrap_or(""));
                            }
                            ui.end_row();
                            for (index, item) in array(&row["choices"]).enumerate() {
                                let r = toolkit::selectable(
                                    ui,
                                    item["value"] == row["value"],
                                    text(item, "label"),
                                );
                                record(geometry, id, &format!("row.{key}.choice.{index}"), &r);
                                if r.clicked() {
                                    edit = Some(item["value"].clone());
                                }
                                for cell in array(&item["cells"]) {
                                    let mut label = RichText::new(text(cell, "text"));
                                    if cell["mono"] == true {
                                        label = label.monospace();
                                    }
                                    if cell["weak"] == true {
                                        label = label.weak();
                                    }
                                    if cell["tone"] == "warning" {
                                        label = label.color(ui.visuals().warn_fg_color);
                                    }
                                    if cell["tone"] == "error" {
                                        label = label.color(ui.visuals().error_fg_color);
                                    }
                                    ui.label(label).on_hover_text(text(cell, "hint"));
                                }
                                ui.end_row();
                            }
                        });
                } else if text(row, "kind") == "notes" {
                    if !text(row, "tag").is_empty() {
                        ui.horizontal(|ui| {
                            ui.strong(text(row, "tag"));
                            if let Some(date) = row["published"].as_str() {
                                ui.label(RichText::new(date).weak());
                            }
                            if let Some(url) = row["url"].as_str() {
                                ui.hyperlink_to(RichText::new(url).small(), url);
                            }
                        });
                    }
                    for block in array(&row["blocks"]) {
                        if block["kind"] == "space" {
                            ui.add_space(ui.spacing().item_spacing.y);
                            continue;
                        }
                        let mut label = RichText::new(text(block, "text"));
                        if block["kind"] == "heading" {
                            label = label.strong();
                        }
                        if block["weak"] == true {
                            label = label.weak();
                        }
                        if block["error"] == true {
                            label = label.color(ui.visuals().error_fg_color);
                        }
                        ui.add(egui::Label::new(label).wrap());
                    }
                } else if text(row, "kind") == "choice" && row["standalone"] == true {
                    edit = inline_choices(ui, id, row, geometry);
                } else if text(row, "kind") == "records" {
                    if row["split"] == true {
                        toolkit::bars::hairline(ui);
                    }
                    let shown = ui.horizontal_top(|ui| {
                        ui.vertical(|ui| {
                            if row["split"] == true {
                                ui.set_width(220.0);
                                toolkit::panel::section_label(ui, text(row, "title"));
                            }
                            if array(&row["choices"]).next().is_none() {
                                ui.label(RichText::new(text(row, "empty")).weak());
                            }
                            for (index, item) in array(&row["choices"]).enumerate() {
                                let r = toolkit::selectable(
                                    ui,
                                    item["value"] == row["value"],
                                    if item["weak"] == true {
                                        RichText::new(text(item, "label")).weak()
                                    } else {
                                        RichText::new(text(item, "label"))
                                    },
                                );
                                record(geometry, id, &format!("row.{key}.choice.{index}"), &r);
                                if r.clicked() {
                                    edit = Some(item["value"].clone());
                                }
                                ui.label(RichText::new(text(item, "detail")).small().weak());
                                ui.add_space(ui.spacing().item_spacing.y);
                            }
                        });
                        if row["split"] == true {
                            ui.add_space(ui.spacing().item_spacing.x * 2.0);
                            ui.vertical(|ui| {
                                ui.set_width(ui.available_width());
                                if array(&row["detail_rows"]).next().is_some() {
                                    for detail in
                                        array(&row["detail_rows"]).filter(|r| r["shown"] != false)
                                    {
                                        draw_inline_row(ui, id, detail, look, geometry, events);
                                    }
                                } else {
                                    ui.add(
                                        egui::Label::new(
                                            RichText::new(text(row, "empty_detail")).weak(),
                                        )
                                        .wrap(),
                                    );
                                }
                            });
                        }
                    });
                    if row["split"] == true {
                        let rect = shown.response.rect;
                        ui.painter().vline(
                            rect.left() + 220.0 + ui.spacing().item_spacing.x * 2.0,
                            rect.y_range(),
                            ui.visuals().widgets.noninteractive.bg_stroke,
                        );
                    }
                } else if text(row, "kind") == "buttons" {
                    ui.horizontal_wrapped(|ui| {
                        for (index, item) in array(&row["choices"]).enumerate() {
                            let mut button = |ui: &mut Ui| {
                                let r = ui.add_enabled(
                                    item["enabled"] != false,
                                    if item["primary"] == true {
                                        toolkit::button::PushButton::primary(text(item, "label"))
                                    } else {
                                        toolkit::button::PushButton::secondary(text(item, "label"))
                                    },
                                );
                                record(geometry, id, &format!("row.{key}.choice.{index}"), &r);
                                if r.clicked() {
                                    edit = Some(item["value"].clone());
                                }
                            };
                            if item["align"] == "right" {
                                ui.with_layout(Layout::right_to_left(Align::Center), button);
                            } else {
                                button(ui);
                            }
                        }
                    });
                    if row["separator_after"] == true {
                        toolkit::bars::hairline(ui);
                    }
                } else {
                    egui::Grid::new("form-row")
                        .num_columns(2)
                        .spacing([24.0, 10.0])
                        .show(ui, |ui| {
                            let title = ui
                                .allocate_ui_with_layout(
                                    vec2(
                                        if text(row, "title").is_empty() {
                                            0.0
                                        } else {
                                            144.0
                                        },
                                        ui.spacing().interact_size.y,
                                    ),
                                    Layout::left_to_right(Align::Center),
                                    |ui| {
                                        if !text(row, "title").is_empty() {
                                            ui.set_min_width(144.0);
                                            ui.style_mut().wrap_mode =
                                                Some(egui::TextWrapMode::Wrap);
                                            ui.label(text(row, "title"));
                                        }
                                    },
                                )
                                .response;
                            ui.vertical(|ui| {
                                ui.set_width(ui.available_width());
                                ui.label_inputs_by(title.id, |ui| match text(row, "kind") {
                                    "colour" => {
                                        let (r, value) = colour_picker(ui, row, look);
                                        record(geometry, id, &format!("row.{key}.control"), &r);
                                        edit = value;
                                    }
                                    "pick" => {
                                        let (r, value) = picker(ui, id, row);
                                        record(geometry, id, &format!("row.{key}.control"), &r);
                                        edit = value;
                                    }
                                    "toggle" => {
                                        let mut v = row["value"].as_bool().unwrap_or(false);
                                        let r = ui.add(toolkit::toggle::Toggle::new(&mut v, ""));
                                        r.widget_info(|| {
                                            egui::WidgetInfo::selected(
                                                egui::Role::CheckBox,
                                                ui.is_enabled(),
                                                v,
                                                text(row, "title"),
                                            )
                                        });
                                        record(geometry, id, &format!("row.{key}.control"), &r);
                                        if r.changed() && ui.is_enabled() {
                                            edit = Some(json!(v));
                                        }
                                    }
                                    "choice" => {
                                        if row["presentation"] == "dropdown" {
                                            let (r, value) = picker(ui, id, row);
                                            record(geometry, id, &format!("row.{key}.control"), &r);
                                            edit = value;
                                            return;
                                        }
                                        edit = inline_choices(ui, id, row, geometry);
                                    }
                                    "number" => {
                                        let (r, value) = numeric(ui, id, row);
                                        record(geometry, id, &format!("row.{key}.control"), &r);
                                        edit = value;
                                    }
                                    "text" | "multitext" | "secret" => {
                                        let field_id = ui.make_persistent_id("field");
                                        let secret = text(row, "kind") == "secret";
                                        let keep = secret || row["commit"] == true;
                                        if secret {
                                            let epoch = ui.make_persistent_id("secret-epoch");
                                            let current = row["epoch"].as_u64().unwrap_or(0);
                                            ui.ctx().data_mut(|d| {
                                                if d.get_temp::<u64>(epoch) != Some(current) {
                                                    d.insert_temp(epoch, current);
                                                    d.insert_temp(field_id, String::new());
                                                }
                                            });
                                        }
                                        let mut v = if keep {
                                            ui.ctx()
                                                .data_mut(|d| d.get_temp::<String>(field_id))
                                                .unwrap_or_else(|| text(row, "value").into())
                                        } else {
                                            text(row, "value").into()
                                        };
                                        let r = if text(row, "kind") == "text" && !secret {
                                            ui.push_id(field_id, |ui| {
                                                toolkit::field::text(
                                                    ui,
                                                    &mut v,
                                                    text(row, "hint"),
                                                    ui.available_width(),
                                                )
                                            })
                                            .inner
                                        } else {
                                            let mut field = if text(row, "kind") == "multitext" {
                                                egui::TextEdit::multiline(&mut v).desired_rows(
                                                    row["lines"].as_u64().unwrap_or(3).min(32)
                                                        as usize,
                                                )
                                            } else {
                                                egui::TextEdit::singleline(&mut v)
                                            };
                                            field = field
                                                .id(field_id)
                                                .desired_width(ui.available_width())
                                                .hint_text(text(row, "hint"))
                                                .password(secret);
                                            ui.add(field)
                                        };
                                        record(geometry, id, &format!("row.{key}.control"), &r);
                                        let commit = r.lost_focus()
                                            || (r.has_focus()
                                                && ui.input(|i| i.key_pressed(egui::Key::Enter)));
                                        if r.changed() || (keep && commit) {
                                            if keep {
                                                ui.ctx().data_mut(|d| {
                                                    d.insert_temp(field_id, v.clone())
                                                });
                                            }
                                            if row["commit"] != true || commit {
                                                edit = Some(json!(v));
                                            }
                                        }
                                    }
                                    _ => {
                                        ui.label(text(row, "value"));
                                    }
                                });
                                if !text(row, "description").is_empty() {
                                    ui.label(RichText::new(text(row, "description")).weak());
                                }
                            });
                            ui.end_row();
                        });
                }
            });
        })
        .response;
    record(geometry, id, &format!("row.{key}"), &response);
    if let Some(value) = edit {
        emit(events, id, "edit", json!({"row":key,"value":value}));
    }
    if let Some(gap) = row["space_after"].as_f64() {
        ui.add_space(gap.clamp(0.0, 64.0) as f32);
    }
}

fn inline_choices(
    ui: &mut Ui,
    id: &str,
    row: &Value,
    geometry: &mut BTreeMap<String, Rect>,
) -> Option<Value> {
    let mut edited = None;
    ui.horizontal_wrapped(|ui| {
        for (index, item) in array(&row["choices"]).enumerate() {
            let selected = item["value"] == row["value"];
            let response = ui
                .add_enabled_ui(item["enabled"] != false, |ui| {
                    if row["presentation"] == "radio" {
                        ui.radio(selected, text(item, "label"))
                    } else {
                        ui.add(egui::Button::selectable(selected, text(item, "label")))
                    }
                })
                .inner;
            record(
                geometry,
                id,
                &format!("row.{}.choice.{index}", text(row, "id")),
                &response,
            );
            if response.clicked() {
                edited = Some(item["value"].clone());
            }
        }
    });
    edited
}
fn append_list(
    ui: &mut Ui,
    id: &str,
    row: &Value,
    look: &Look,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) {
    let key = text(row, "id");
    let filter_id = ui.make_persistent_id((id, key, "append-filter"));
    let more_id = filter_id.with("more");
    let mut query = ui
        .data(|d| d.get_temp::<String>(filter_id))
        .unwrap_or_default();
    let search = ui.add(
        egui::TextEdit::singleline(&mut query)
            .hint_text(text(row, "search_hint"))
            .desired_width(ui.available_width()),
    );
    record(geometry, id, &format!("row.{key}.search"), &search);
    if search.changed() {
        ui.data_mut(|d| d.insert_temp(filter_id, query.clone()));
    }
    ui.add_space(look.s);
    let mut extended = ui.data(|d| d.get_temp::<bool>(more_id)).unwrap_or(false);
    if row["extended"] == true {
        egui::Frame::NONE
            .fill(look.card)
            .corner_radius(Look::radius(look.r_s))
            .show(ui, |ui| {
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(look.row_pad() as i8, 4))
                    .show(ui, |ui| {
                        ui.set_min_height(32.0);
                        ui.set_width(ui.available_width());
                        ui.horizontal(|ui| {
                            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                let r = cosmic_switch(ui, extended, look, text(row, "more_label"));
                                record(geometry, id, &format!("row.{key}.more"), &r);
                                if r.clicked() {
                                    extended = !extended;
                                    ui.data_mut(|d| d.insert_temp(more_id, extended));
                                }
                                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                    ui.label(
                                        RichText::new(text(row, "more_label"))
                                            .size(14.0)
                                            .color(look.text),
                                    );
                                });
                            });
                        });
                    });
            });
        ui.add_space(look.s);
    }
    let query = query.trim().to_lowercase();
    let choices: Vec<_> = array(&row["choices"])
        .enumerate()
        .filter(|(i, c)| {
            (extended || *i < row["primary_count"].as_u64().unwrap_or(0) as usize)
                && text(c, "label").to_lowercase().contains(&query)
        })
        .collect();
    egui::Frame::NONE
        .fill(look.card)
        .corner_radius(Look::radius(look.r_s))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.spacing_mut().item_spacing.y = 0.0;
            if choices.is_empty() {
                egui::Frame::NONE
                    .inner_margin(egui::Margin::symmetric(look.row_pad() as i8, 4))
                    .show(ui, |ui| {
                        ui.set_min_height(32.0);
                        ui.label(
                            RichText::new(text(row, "empty_label"))
                                .size(14.0)
                                .color(look.text),
                        );
                    });
            }
            for (position, (index, choice)) in choices.iter().enumerate() {
                if position > 0 {
                    let y = ui.cursor().top();
                    ui.painter()
                        .hline(ui.min_rect().x_range(), y, Stroke::new(1.0, look.divider));
                    ui.add_space(1.0);
                }
                let (rect, _) =
                    ui.allocate_exact_size(vec2(ui.available_width(), 40.0), Sense::hover());
                if !ui.is_rect_visible(rect) {
                    continue;
                }
                let indent = if choice["value"]
                    .as_array()
                    .and_then(|a| a.last())
                    .and_then(Value::as_str)
                    .is_some_and(|s| !s.is_empty())
                {
                    look.l
                } else {
                    0.0
                };
                ui.painter().text(
                    rect.left_center() + vec2(look.m + indent, 0.0),
                    egui::Align2::LEFT_CENTER,
                    text(choice, "label"),
                    egui::FontId::proportional(14.0),
                    look.text,
                );
                let have = array(&row["value"]).any(|v| v == &choice["value"]);
                let full =
                    array(&row["value"]).count() >= row["max_items"].as_u64().unwrap_or(8) as usize;
                let word = text(row, if have { "added_label" } else { "add_label" });
                let slot = Rect::from_min_max(
                    pos2(rect.right() - 160.0, rect.top()),
                    rect.max - vec2(look.m, 0.0),
                );
                ui.scope_builder(
                    egui::UiBuilder::new()
                        .max_rect(slot)
                        .layout(Layout::right_to_left(Align::Center)),
                    |ui| {
                        let r = ui
                            .add_enabled_ui(row["blocked"].is_null() && !have && !full, |ui| {
                                standard_button(ui, word, look)
                            })
                            .inner;
                        r.widget_info(|| {
                            egui::WidgetInfo::labeled(
                                egui::Role::Button,
                                ui.is_enabled() && !have && !full,
                                format!("{word} {}", text(choice, "label")),
                            )
                        });
                        record(geometry, id, &format!("row.{key}.choice.{index}"), &r);
                        if r.clicked() {
                            emit(
                                events,
                                id,
                                "edit",
                                json!({"row":key,"value":choice["value"]}),
                            );
                        }
                    },
                );
            }
        });
}
fn numeric(ui: &mut Ui, id: &str, row: &Value) -> (Response, Option<Value>) {
    let current = row["value"].as_f64().unwrap_or(0.0);
    let key = ui.make_persistent_id((id, text(row, "id"), "numeric"));
    let mut value = ui.data(|d| d.get_temp::<f64>(key)).unwrap_or(current);
    let range = row["min"].as_f64().unwrap_or(0.0)..=row["max"].as_f64().unwrap_or(100.0);
    let mut commit = false;
    let response = ui
        .horizontal(|ui| {
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let escaped = ui.input(|i| i.key_pressed(egui::Key::Escape));
                let field = ui.add(
                    toolkit::field::ValueField::new(&mut value, range.clone())
                        .unit(text(row, "unit")),
                );
                field.widget_info(|| {
                    egui::WidgetInfo::labeled(
                        egui::Role::TextInput,
                        ui.is_enabled(),
                        text(row, "title"),
                    )
                });
                let cancelled = field.lost_focus() && escaped;
                if cancelled {
                    ui.data_mut(|d| d.remove::<f64>(key));
                    value = current;
                } else if field.has_focus() || field.dragged() {
                    ui.data_mut(|d| d.insert_temp(key, value));
                }
                if !cancelled && (field.lost_focus() || field.drag_stopped()) {
                    commit = true;
                }
                if row["slider"] == true {
                    let slider = ui.add(
                        toolkit::slider::Slider::new(&mut value, range).label(text(row, "title")),
                    );
                    if slider.dragged() {
                        ui.data_mut(|d| d.insert_temp(key, value));
                    } else if slider.drag_stopped() || slider.changed() {
                        commit = true;
                    }
                }
            });
        })
        .response;
    if !commit || !ui.is_enabled() {
        return (response, None);
    }
    ui.data_mut(|d| d.remove::<f64>(key));
    let scale = 10f64.powi(row["places"].as_i64().unwrap_or(0).clamp(0, 6) as i32);
    let rounded = (value * scale).round() / scale;
    (
        response,
        (rounded != (current * scale).round() / scale).then(|| json!(rounded)),
    )
}

fn picker(ui: &mut Ui, id: &str, row: &Value) -> (Response, Option<Value>) {
    let key = text(row, "id");
    let mut picked = None;
    let current = array(&row["choices"])
        .find(|c| c["value"] == row["value"])
        .map(|c| text(c, "label").to_owned())
        .unwrap_or_else(|| row["value"].as_str().unwrap_or(text(row, "title")).into());
    let response = egui::ComboBox::from_id_salt((id, key, "pick"))
        .selected_text(current)
        .show_ui(ui, |ui| {
            let query_id = ui.make_persistent_id("filter");
            let mut query = ui
                .ctx()
                .data_mut(|d| d.get_temp::<String>(query_id))
                .unwrap_or_default();
            if array(&row["choices"]).count() > 20 {
                let r = ui.text_edit_singleline(&mut query);
                r.widget_info(|| {
                    egui::WidgetInfo::labeled(egui::Role::TextInput, ui.is_enabled(), "Search")
                });
                ui.ctx()
                    .data_mut(|d| d.insert_temp(query_id, query.clone()));
            }
            let choices: Vec<_> = array(&row["choices"])
                .filter(|c| {
                    text(c, "label")
                        .to_lowercase()
                        .contains(&query.to_lowercase())
                })
                .collect();
            egui::ScrollArea::vertical().max_height(360.0).show_rows(
                ui,
                24.0,
                choices.len(),
                |ui, range| {
                    for index in range {
                        let item = choices[index];
                        let r =
                            ui.selectable_label(item["value"] == row["value"], text(item, "label"));
                        if r.clicked() {
                            picked = Some(item["value"].clone());
                            ui.close();
                        }
                    }
                },
            );
        })
        .response;
    response.widget_info(|| {
        egui::WidgetInfo::labeled(egui::Role::ComboBox, ui.is_enabled(), text(row, "title"))
    });
    (response, picked)
}
fn colour_picker(ui: &mut Ui, row: &Value, look: &Look) -> (Response, Option<Value>) {
    let mut value = parse_colour(text(row, "value")).unwrap_or(look.accent);
    let mut rgb = [value.r(), value.g(), value.b()];
    let response = ui
        .horizontal(|ui| {
            let r = ui.color_edit_button_srgb(&mut rgb);
            let mut changed = if r.changed() && ui.is_enabled() {
                value = Color32::from_rgb(rgb[0], rgb[1], rgb[2]);
                Some(json!(format!(
                    "#{:02x}{:02x}{:02x}",
                    value.r(),
                    value.g(),
                    value.b()
                )))
            } else {
                None
            };
            if row["auto_link"] == true && ui.small_button("Auto").clicked() {
                changed = Some(Value::Null);
            }
            (r, changed)
        })
        .inner;
    response.0.widget_info(|| {
        egui::WidgetInfo::labeled(egui::Role::Button, ui.is_enabled(), text(row, "title"))
    });
    response
}
fn draw_ordered(
    ui: &mut Ui,
    id: &str,
    row: &Value,
    look: &Look,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) {
    let key = text(row, "id");
    let items: Vec<_> = array(&row["items"]).collect();
    ui.add_enabled_ui(row["blocked"].is_null(), |ui| {
        for (index, item) in items.iter().enumerate() {
            if index > 0 {
                let y = ui.cursor().top();
                ui.painter()
                    .hline(ui.min_rect().x_range(), y, Stroke::new(1.0, look.divider));
                ui.add_space(1.0);
            }
            egui::Frame::NONE
                .inner_margin(egui::Margin::symmetric(look.row_pad() as i8, 4))
                .show(ui, |ui| {
                    ui.set_min_height(32.0);
                    ui.set_width(ui.available_width());
                    ui.horizontal(|ui| {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let more = cosmic_icon_button(
                                ui,
                                toolkit::Icon::EllipsisVertical,
                                true,
                                text(item, "label"),
                                look,
                            );
                            record(geometry, id, &format!("row.{key}.{index}.more"), &more);
                            egui::Popup::menu(&more).show(|ui| {
                                ui.set_min_width(160.0);
                                for (action, title, enabled) in [
                                    ("up", text(row, "move_up"), index > 0),
                                    ("down", text(row, "move_down"), index + 1 < items.len()),
                                    (
                                        "remove",
                                        text(row, "remove_label"),
                                        items.len()
                                            > row["min_items"].as_u64().unwrap_or(0) as usize,
                                    ),
                                ] {
                                    if action == "remove" {
                                        ui.separator();
                                    }
                                    let r = ui
                                        .add_enabled_ui(enabled, |ui| {
                                            toolkit::selectable(ui, false, title)
                                        })
                                        .inner;
                                    record(
                                        geometry,
                                        id,
                                        &format!("row.{key}.{index}.{action}"),
                                        &r,
                                    );
                                    if r.clicked() {
                                        let mut values: Vec<Value> = items
                                            .iter()
                                            .map(|item| item["value"].clone())
                                            .collect();
                                        match action {
                                            "up" => values.swap(index, index - 1),
                                            "down" => values.swap(index, index + 1),
                                            _ => {
                                                values.remove(index);
                                            }
                                        }
                                        emit(events, id, "edit", json!({"row":key,"value":values}));
                                        ui.close();
                                    }
                                }
                            });
                            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                ui.label(
                                    RichText::new(text(item, "label"))
                                        .size(14.0)
                                        .color(look.text),
                                );
                            });
                        });
                    });
                });
        }
    });
}

#[allow(clippy::too_many_arguments)]
fn cosmic_slider(
    ui: &mut Ui,
    salt: (&str, &str),
    value: f64,
    range: (f64, f64),
    step: f64,
    show_value: bool,
    start: Option<&str>,
    end: Option<&str>,
    name: &str,
    look: &Look,
) -> Option<f64> {
    let id = ui.scope_id().with(("slider", salt));
    let held: Option<f64> = ui.data(|d| d.get_temp(id));
    let shown = held.unwrap_or(value);
    let mut out = None;
    ui.spacing_mut().item_spacing.x = look.xs;
    // Read left to right: the value, the start label, the rail, the end
    // label, whichever way the row lays it out.
    let rtl = ui.layout().prefer_right_to_left();
    let text = |ui: &mut Ui, s: String| {
        ui.label(RichText::new(s).size(14.0).color(look.text));
    };
    let value_text = show_value.then(|| format!("{shown:.0}"));
    if rtl {
        if let Some(end) = end {
            text(ui, end.to_owned());
        }
    } else {
        if let Some(v) = &value_text {
            text(ui, v.clone());
        }
        if let Some(start) = start {
            text(ui, start.to_owned());
        }
    }
    // Beside its labels the rail is COSMIC's 250; stacked under them it
    // spans the row, its end label at the card padding.
    let width = if rtl {
        250.0_f32.min(ui.available_width() * 0.5).max(120.0)
    } else {
        let end_width = end.map_or(0.0, |e| text_width(ui, &e.to_owned(), 14.0) + look.xs);
        (ui.available_width() - end_width).max(60.0)
    };

    let (rect, response) = ui.allocate_exact_size(vec2(width, 24.0), Sense::click_and_drag());
    let span = (range.1 - range.0).max(f64::EPSILON);
    let rail = Rect::from_center_size(rect.center(), vec2(rect.width() - 20.0, 4.0));
    let t = ((shown - range.0) / span).clamp(0.0, 1.0) as f32;
    let x = rail.min.x + t * rail.width();
    let alpha = if ui.is_enabled() { 1.0 } else { 0.5 };
    ui.painter().rect_filled(
        rail,
        Look::radius(look.r_xs),
        look.track_off.gamma_multiply(alpha),
    );
    ui.painter().rect_filled(
        Rect::from_min_max(rail.min, pos2(x, rail.max.y)),
        Look::radius(look.r_xs),
        look.accent.gamma_multiply(alpha),
    );
    ui.painter().circle_filled(
        pos2(x, rect.center().y),
        10.0,
        look.accent.gamma_multiply(alpha),
    );
    focus_ring(ui, &response, rect, look.r_m, look);
    let at = |pos: egui::Pos2| {
        let t = ((pos.x - rail.min.x) / rail.width()).clamp(0.0, 1.0) as f64;
        let v = range.0 + t * span;
        if step > 0.0 {
            range.0 + ((v - range.0) / step).round() * step
        } else {
            v
        }
    };
    if (response.dragged() || response.drag_started())
        && let Some(pos) = response.interact_pointer_pos()
    {
        ui.data_mut(|d| d.insert_temp(id, at(pos)));
    }
    // Arrow keys step a focused slider (the Role::Slider promise).
    if response.has_focus() {
        let unit = if step > 0.0 { step } else { span / 20.0 };
        let delta = ui.input(|i| {
            if i.key_pressed(egui::Key::ArrowRight) || i.key_pressed(egui::Key::ArrowUp) {
                unit
            } else if i.key_pressed(egui::Key::ArrowLeft) || i.key_pressed(egui::Key::ArrowDown) {
                -unit
            } else {
                0.0
            }
        });
        if delta != 0.0 {
            out = Some((value + delta).clamp(range.0, range.1));
        }
    }
    if response.drag_stopped() {
        out = ui.data_mut(|d| d.remove_temp::<f64>(id));
    } else if response.clicked()
        && let Some(pos) = response.interact_pointer_pos()
    {
        out = Some(at(pos));
    }
    response.widget_info(|| egui::WidgetInfo::slider(ui.is_enabled(), shown, name));
    if rtl {
        if let Some(start) = start {
            text(ui, start.to_owned());
        }
        if let Some(v) = value_text {
            text(ui, v);
        }
    } else if let Some(end) = end {
        text(ui, end.to_owned());
    }
    out.filter(|v| (v - value).abs() > f64::EPSILON)
}

fn text_width(ui: &Ui, text: &str, size: f32) -> f32 {
    ui.painter()
        .layout_no_wrap(
            text.to_owned(),
            egui::FontId::proportional(size),
            Color32::WHITE,
        )
        .size()
        .x
}

fn images(
    ui: &mut Ui,
    id: &str,
    row: &Value,
    options: &[&Value],
    look: &Look,
    events: &mut Vec<Intent>,
    geometry: &mut BTreeMap<String, Rect>,
) {
    let key = text(row, "id");
    let count = options.len().max(1) as f32;
    let gap = look.xs;
    // Below three cells' width the images shrink to fit, never clip.
    let cell = 191.0_f32
        .min((ui.available_width() - gap * (count - 1.0)) / count)
        .max(1.0);
    let total = cell * count + gap * (count - 1.0);
    let (area, _) =
        ui.allocate_exact_size(vec2(ui.available_width(), 100.0 + 30.0), Sense::hover());
    let left = area.center().x - total / 2.0;
    for (i, option) in options.iter().enumerate() {
        let selected = option["value"] == row["value"];
        let text = text(option, "label").to_owned();
        let radius = match option["value"].as_str() {
            Some("round") => 16.0,
            Some("slightly_round") => 6.0,
            _ => 0.0,
        };
        let rect = Rect::from_min_size(
            pos2(left + i as f32 * (cell + gap), area.min.y),
            vec2(cell, 100.0),
        );
        let response = ui.interact(rect, ui.scope_id().with(("image", key, i)), Sense::click());
        ui.painter()
            .rect_filled(rect, Look::radius(look.r_s), look.window);
        let window =
            Rect::from_min_size(rect.min + vec2(cell * 0.17, 24.0), vec2(cell * 0.66, 60.0));
        ui.painter()
            .rect_filled(window, Look::radius(radius), look.card);
        ui.painter().rect_filled(
            Rect::from_min_size(window.min + vec2(12.0, 14.0), vec2(48.0, 18.0)),
            Look::radius(radius.min(9.0)),
            look.accent,
        );
        if selected {
            ui.painter().rect_stroke(
                rect,
                Look::radius(look.r_s),
                Stroke::new(2.0, look.accent),
                StrokeKind::Inside,
            );
        }
        focus_ring(ui, &response, rect, look.r_s, look);
        response.widget_info(|| {
            egui::WidgetInfo::selected(egui::Role::RadioButton, ui.is_enabled(), selected, &text)
        });
        record(geometry, id, &format!("row.{key}.choice.{i}"), &response);
        if response.clicked() && !selected && ui.is_enabled() {
            emit(
                events,
                id,
                "edit",
                json!({"row":key,"value":option["value"]}),
            );
        }
        ui.painter().text(
            pos2(rect.center().x, rect.max.y + 8.0),
            egui::Align2::CENTER_TOP,
            &text,
            egui::FontId::proportional(14.0),
            look.text,
        );
    }
}

fn colour_editor(ui: &mut Ui, id: &str, row: &Value, look: &Look, events: &mut Vec<Intent>) {
    if row["submit_label"].is_string() {
        colour_adder(ui, id, row, look, events);
        return;
    }
    let key = text(row, "id");
    let alpha = row["alpha"] == true;
    let stored = parse_colour(text(row, "value"));
    let held_id = ui.make_persistent_id((id, key, "picking"));
    let mut picking = ui
        .data(|d| d.get_temp::<Color32>(held_id))
        .or(stored)
        .unwrap_or(look.accent);
    if egui::widgets::color_picker::color_picker_color32(
        ui,
        &mut picking,
        if alpha {
            egui::widgets::color_picker::Alpha::OnlyBlend
        } else {
            egui::widgets::color_picker::Alpha::Opaque
        },
    ) {
        ui.data_mut(|d| d.insert_temp(held_id, picking));
    }
    if ui.input(|i| i.pointer.any_released())
        && let Some(held) = ui.data(|d| d.get_temp::<Color32>(held_id))
        && Some(held) != stored
    {
        ui.data_mut(|d| d.remove_temp::<Color32>(held_id));
        emit(
            events,
            id,
            "edit",
            json!({"row":key,"value":colour_hex(held,alpha)}),
        );
    }
    ui.add_space(look.s);
    let hex_id = held_id.with("hex");
    let mut value = ui
        .data(|d| d.get_temp::<String>(hex_id))
        .unwrap_or_else(|| text(row, "value").to_owned());
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(text(row, "hex_label"))
                .size(12.0)
                .color(look.text),
        );
        let edit = ui.add(
            egui::TextEdit::singleline(&mut value)
                .desired_width(120.0)
                .id_salt(hex_id),
        );
        if edit.changed() {
            ui.data_mut(|d| d.insert_temp(hex_id, value.clone()));
        }
        if edit.lost_focus() && ui.data(|d| d.get_temp::<String>(hex_id)).is_some() {
            ui.data_mut(|d| d.remove_temp::<String>(hex_id));
            if let Some(colour) = parse_colour(value.trim()) {
                emit(
                    events,
                    id,
                    "edit",
                    json!({"row":key,"value":colour_hex(colour,alpha)}),
                );
            } else {
                emit(
                    events,
                    id,
                    "problem",
                    json!({"message":row["invalid_label"]}),
                );
            }
        }
    });
    ui.add_space(look.m);
    if standard_button(ui, text(row, "reset_label"), look).clicked() {
        ui.data_mut(|d| {
            d.remove_temp::<Color32>(held_id);
            d.remove_temp::<String>(hex_id);
        });
        emit(
            events,
            id,
            "edit",
            json!({"row":key,"value":row["auto_value"]}),
        );
    }
}
fn colour_adder(ui: &mut Ui, id: &str, row: &Value, look: &Look, events: &mut Vec<Intent>) {
    let held_id = ui.make_persistent_id((id, text(row, "id"), "adding"));
    let mut picking = ui
        .data(|d| d.get_temp::<Color32>(held_id))
        .unwrap_or(look.accent);
    if egui::widgets::color_picker::color_picker_color32(
        ui,
        &mut picking,
        egui::widgets::color_picker::Alpha::Opaque,
    ) {
        ui.data_mut(|d| d.insert_temp(held_id, picking));
    }
    ui.add_space(look.s);
    let hex_id = held_id.with("hex");
    if ui.data(|d| d.get_temp::<Color32>(held_id))
        != ui.data(|d| d.get_temp::<Color32>(hex_id.with("at")))
    {
        ui.data_mut(|d| {
            d.remove_temp::<String>(hex_id);
            d.insert_temp(hex_id.with("at"), picking);
        });
    }
    let mut value = ui
        .data(|d| d.get_temp::<String>(hex_id))
        .unwrap_or_else(|| colour_hex(picking, false));
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(text(row, "hex_label"))
                .size(12.0)
                .color(look.text),
        );
        let r = ui.add(
            egui::TextEdit::singleline(&mut value)
                .desired_width(120.0)
                .id_salt(hex_id),
        );
        if r.changed() {
            ui.data_mut(|d| {
                d.insert_temp(hex_id, value.clone());
                if let Some(colour) = parse_colour(value.trim()) {
                    d.insert_temp(held_id, colour);
                }
            });
        }
    });
    ui.add_space(look.m);
    if standard_button(ui, text(row, "submit_label"), look).clicked() {
        ui.data_mut(|d| {
            d.remove_temp::<Color32>(held_id);
            d.remove_temp::<String>(hex_id);
        });
        emit(
            events,
            id,
            "edit",
            json!({"row":row["id"],"value":colour_hex(picking,false)}),
        );
    }
}
fn colour_hex(colour: Color32, alpha: bool) -> String {
    let [r, g, b, a] = colour.to_srgba_unmultiplied();
    if alpha && a < 255 {
        format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
    } else {
        format!("#{r:02x}{g:02x}{b:02x}")
    }
}
fn cosmic_icon_button(
    ui: &mut Ui,
    icon: toolkit::Icon,
    enabled: bool,
    name: &str,
    look: &Look,
) -> Response {
    let enabled = enabled && ui.is_enabled();
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::hover()
    };
    let (rect, response) = ui.allocate_exact_size(egui::Vec2::splat(24.0), sense);
    if response.hovered() && enabled {
        ui.painter()
            .circle_filled(rect.center(), 12.0, look.button_hover);
    }
    let ink = if enabled {
        look.value
    } else {
        look.value.gamma_multiply(0.5)
    };
    toolkit::button::paint_icon(ui, icon, rect.center(), 16.0, ink);
    focus_ring(ui, &response, rect, look.r_xl, look);
    response.widget_info(|| egui::WidgetInfo::labeled(egui::Role::Button, enabled, name));
    response
}
