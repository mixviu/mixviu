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
    if hex.len() != 6 || !hex.is_ascii() {
        return None;
    }
    Some(Color32::from_rgb(
        u8::from_str_radix(&hex[0..2], 16).ok()?,
        u8::from_str_radix(&hex[2..4], 16).ok()?,
        u8::from_str_radix(&hex[4..6], 16).ok()?,
    ))
}
fn array(v: &Value) -> impl Iterator<Item = &Value> {
    v.as_array().into_iter().flatten()
}
fn rows(value: &Value) -> impl Iterator<Item = &Value> {
    array(&value["sections"])
        .chain(array(&value["drawer"]["sections"]))
        .flat_map(|s| array(&s["rows"]))
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
                "text",
                "link",
                "button",
                "info",
                "image",
                "buttons",
                "records",
                "multitext",
                "secret",
                "tiles",
                "media-preview",
                "file-pick",
                "colour",
                "ordered",
                "pick",
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
                    "number" if !payload["value"].is_number() => {
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
}
pub(crate) fn draw(
    ui: &mut Ui,
    id: &str,
    value: &Value,
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) -> Response {
    let area = ui.available_rect_before_wrap();
    crate::media_gallery::begin(ui, id);
    let look = Look::of(
        &crate::appearance::session_theme(ui.ctx()),
        &value["settings"],
    );
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
                    if r.changed() {
                        emit(events, id, "query", json!(query));
                    }
                    if r.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                        emit(events, id, "query", json!(""));
                        ui.memory_mut(|m| m.surrender_focus(r.id));
                    }
                    if !query.is_empty() && ui.small_button("×").clicked() {
                        emit(events, id, "query", json!(""));
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
                    if r.clicked() {
                        emit(events, id, "navigate", json!({"page":p["id"]}));
                    }
                }
            });
        content.min.x = area.left() + 296.0;
    }
    if value["drawer"].is_object() {
        let width = (344.0 + 2.0 * look.l).min(content.width());
        let rect = Rect::from_min_max(pos2(content.right() - width, content.top()), content.max);
        let mut drawer = ui.new_child(
            egui::UiBuilder::new()
                .id_salt((id, "drawer"))
                .max_rect(rect.shrink(look.l))
                .layout(Layout::top_down(Align::Min)),
        );
        drawer.horizontal(|ui| {
            ui.heading(text(&value["drawer"], "title"));
            if ui.button("×").clicked() {
                emit(events, id, "drawer.close", Value::Null);
            }
        });
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
            .max_rect(column)
            .layout(Layout::top_down(Align::Min)),
    );
    body.set_width(width);
    if value["searching"] == true && !text(value, "query").trim().is_empty() {
        body.heading(text(value, "results_title"));
        egui::ScrollArea::vertical()
            .id_salt((id, "results"))
            .show(&mut body, |ui| {
                for hit in array(&value["hits"]) {
                    if ui.button(text(hit, "title")).clicked() {
                        emit(events, id, "navigate", hit.clone());
                    }
                }
            });
    } else {
        if !value["parent"].is_null() {
            let r = body
                .add(egui::Button::new(format!("‹  {}", text(value, "parent_title"))).frame(false));
            record(geometry, id, "back", &r);
            if r.clicked() {
                emit(events, id, "navigate", json!({"page":value["parent"]}));
            }
        }
        body.add_space(look.s);
        body.label(
            RichText::new(text(value, "title"))
                .size(24.0)
                .strong()
                .color(look.text),
        );
        body.add_space(look.l);
        egui::ScrollArea::vertical()
            .id_salt((id, "page", text(value, "page")))
            .auto_shrink([false, false])
            .show(&mut body, |ui| {
                ui.set_width(width);
                if let Some(notice) = value["notice"].as_str().filter(|s| !s.is_empty()) {
                    ui.label(RichText::new(notice).color(look.value));
                    ui.add_space(look.s);
                }
                for p in array(&value["children"]) {
                    let r = egui::Frame::NONE
                        .fill(look.card)
                        .corner_radius(Look::radius(look.r_s))
                        .inner_margin(egui::Margin::same(look.m as i8))
                        .show(ui, |ui| {
                            ui.set_min_width((width - 2.0 * look.m).max(0.0));
                            ui.horizontal(|ui| {
                                ui.add(crate::icons::image_of(
                                    ui.ctx(),
                                    crate::icons::Icon::from_name(text(p, "icon")).unwrap_or(
                                        crate::icons::Icon::from_name("settings").unwrap(),
                                    ),
                                    24.0,
                                    look.icon,
                                ));
                                ui.vertical(|ui| {
                                    ui.label(text(p, "title"));
                                    let desc = text(p, "description");
                                    if !desc.is_empty() {
                                        ui.small(desc);
                                    }
                                });
                                ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                    ui.label("›");
                                });
                            });
                        })
                        .response
                        .interact(Sense::click());
                    record(geometry, id, &format!("child.{}", text(p, "id")), &r);
                    if r.clicked() {
                        emit(events, id, "navigate", json!({"page":p["id"]}));
                    }
                    ui.add_space(look.xs);
                }
                cards(ui, id, &value["sections"], &look, geometry, events);
            });
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
    for section in array(sections).filter(|s| s["shown"] != false) {
        if section["plain"] == true {
            if let Some(title) = section["title"].as_str() {
                ui.separator();
                ui.label(RichText::new(title).strong());
                ui.add_space(6.0);
            }
            for row in array(&section["rows"]).filter(|r| r["shown"] != false) {
                if ["tiles", "media-preview", "file-pick"].contains(&text(row, "kind")) {
                    crate::media_gallery::draw(ui, id, row, look, geometry, events);
                } else {
                    draw_inline_row(ui, id, row, look, geometry, events);
                }
            }
            continue;
        }
        if let Some(title) = section["title"].as_str() {
            ui.label(RichText::new(title).strong().color(look.text));
            ui.add_space(look.s);
        }
        egui::Frame::NONE
            .fill(look.card)
            .corner_radius(Look::radius(look.r_s))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.spacing_mut().item_spacing.y = 0.0;
                for (index, row) in array(&section["rows"])
                    .filter(|r| r["shown"] != false)
                    .enumerate()
                {
                    if index > 0 {
                        let y = ui.cursor().top();
                        ui.painter().hline(
                            ui.min_rect().x_range(),
                            y,
                            Stroke::new(1.0, look.divider),
                        );
                    }
                    draw_row(ui, id, row, look, geometry, events);
                }
            });
        ui.add_space(look.l);
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
    if row["kind"] == "ordered" {
        draw_ordered(ui, id, row, geometry, events);
        return;
    }
    let key = text(row, "id");
    let mut edited = None;
    let response = egui::Frame::NONE
        .inner_margin(egui::Margin::symmetric(look.m as i8, look.xs as i8))
        .show(ui, |ui| {
            let width = ui.available_width();
            ui.set_min_width(width);
            let stacked = row["presentation"] != "dropdown"
                && matches!(
                    text(row, "kind"),
                    "choice" | "previews" | "swatches" | "image"
                );
            let titles = |ui: &mut Ui| {
                ui.label(RichText::new(text(row, "title")).color(look.text));
                if !text(row, "description").is_empty() {
                    ui.label(
                        RichText::new(text(row, "description"))
                            .size(12.0)
                            .color(look.text),
                    );
                }
                if let Some(why) = row["blocked"].as_str() {
                    ui.label(
                        RichText::new(format!("ⓘ {why}"))
                            .size(12.0)
                            .color(look.warning),
                    );
                }
            };
            if stacked {
                titles(ui);
                ui.add_space(look.s);
            }
            ui.add_enabled_ui(row["blocked"].is_null(), |ui| {
                if stacked {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = vec2(look.s, look.s);
                        for (index, choice) in array(&row["choices"]).enumerate() {
                            let selected = choice["value"] == row["value"];
                            let r = if text(row, "kind") == "swatches" {
                                let (rect, r) =
                                    ui.allocate_exact_size(vec2(48.0, 48.0), Sense::click());
                                let colour =
                                    parse_colour(text(choice, "colour")).unwrap_or(look.accent);
                                ui.painter()
                                    .rect_filled(rect, Look::radius(look.r_s), colour);
                                if selected {
                                    ui.painter().text(
                                        rect.center(),
                                        egui::Align2::CENTER_CENTER,
                                        "✓",
                                        egui::FontId::proportional(14.0),
                                        look.on_accent,
                                    );
                                }
                                r
                            } else if text(row, "kind") == "previews" {
                                let (rect, r) =
                                    ui.allocate_exact_size(vec2(192.0, 120.0), Sense::click());
                                let preview = Rect::from_min_size(rect.min, vec2(192.0, 100.0));
                                let colour =
                                    parse_colour(text(choice, "colour")).unwrap_or(look.window);
                                ui.painter()
                                    .rect_filled(preview, Look::radius(look.r_s), colour);
                                let inner = preview.shrink(20.0);
                                ui.painter().rect_filled(
                                    inner,
                                    Look::radius(look.r_s),
                                    colour.gamma_multiply(0.8),
                                );
                                ui.painter().rect_filled(
                                    Rect::from_min_size(
                                        inner.min + vec2(10.0, 10.0),
                                        vec2(36.0, 10.0),
                                    ),
                                    4.0,
                                    look.accent,
                                );
                                if selected {
                                    ui.painter().rect_stroke(
                                        preview,
                                        Look::radius(look.r_s),
                                        Stroke::new(2.0, look.accent),
                                        StrokeKind::Inside,
                                    );
                                }
                                ui.painter().text(
                                    pos2(rect.center().x, rect.bottom() - 6.0),
                                    egui::Align2::CENTER_CENTER,
                                    text(choice, "label"),
                                    egui::FontId::proportional(14.0),
                                    look.text,
                                );
                                r
                            } else {
                                ui.selectable_label(selected, text(choice, "label"))
                            };
                            record(geometry, id, &format!("row.{key}.choice.{index}"), &r);
                            if r.clicked() {
                                edited = Some(choice["value"].clone());
                            }
                        }
                    });
                } else {
                    ui.horizontal(|ui| {
                        ui.allocate_ui_with_layout(
                            vec2((width - 180.0).max(width * 0.5), 0.0),
                            Layout::top_down(Align::Min),
                            titles,
                        );
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            match text(row, "kind") {
                                "toggle" => {
                                    let mut v = row["value"].as_bool().unwrap_or(false);
                                    let r = cosmic_switch(ui, v, look, text(row, "title"));
                                    record(geometry, id, &format!("row.{key}.control"), &r);
                                    if r.clicked() {
                                        v = !v;
                                        edited = Some(json!(v));
                                    }
                                }
                                "number" => {
                                    let mut v = row["value"].as_f64().unwrap_or(0.0);
                                    let min = row["min"].as_f64().unwrap_or(0.0);
                                    let max = row["max"].as_f64().unwrap_or(100.0);
                                    let r = ui.add(
                                        egui::DragValue::new(&mut v)
                                            .range(min..=max)
                                            .speed(row["step"].as_f64().unwrap_or(1.0)),
                                    );
                                    record(geometry, id, &format!("row.{key}.control"), &r);
                                    if r.changed() {
                                        edited = Some(json!(v));
                                    }
                                }
                                "text" => {
                                    let mut v = row["value"].as_str().unwrap_or("").to_owned();
                                    let r = ui.add(
                                        egui::TextEdit::singleline(&mut v)
                                            .id_salt((id, key))
                                            .desired_width(160.0),
                                    );
                                    record(geometry, id, &format!("row.{key}.control"), &r);
                                    if r.changed() {
                                        edited = Some(json!(v));
                                    }
                                }
                                "choice" | "pick" => {
                                    let (r, value) = picker(ui, id, row);
                                    record(geometry, id, &format!("row.{key}.control"), &r);
                                    edited = value;
                                }
                                "colour" => {
                                    let (r, value) = colour_picker(ui, row, look);
                                    record(geometry, id, &format!("row.{key}.control"), &r);
                                    edited = value;
                                }
                                "link" | "button" => {
                                    let r = ui.button(if text(row, "kind") == "link" {
                                        "›"
                                    } else {
                                        text(row, "title")
                                    });
                                    record(geometry, id, &format!("row.{key}.control"), &r);
                                    if r.clicked() {
                                        emit(events, id, "activate", json!({"row":key}));
                                    }
                                }
                                _ => {
                                    ui.label(
                                        row["value"]
                                            .as_str()
                                            .map(str::to_owned)
                                            .unwrap_or_else(|| row["value"].to_string()),
                                    );
                                }
                            }
                        });
                    });
                }
            });
        })
        .response;
    record(geometry, id, &format!("row.{key}"), &response);
    if let Some(value) = edited {
        emit(events, id, "edit", json!({"row":key,"value":value}));
    }
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
                if text(row, "kind") == "info" {
                    ui.label(RichText::new(row["value"].as_str().unwrap_or("")).weak());
                } else if text(row, "kind") == "records" {
                    if array(&row["choices"]).next().is_none() {
                        ui.label(RichText::new(text(row, "empty")).weak());
                    }
                    for (index, item) in array(&row["choices"]).enumerate() {
                        let r = ui.selectable_label(
                            item["value"] == row["value"],
                            format!("{}    {}", text(item, "label"), text(item, "detail")),
                        );
                        record(geometry, id, &format!("row.{key}.choice.{index}"), &r);
                        if r.clicked() {
                            edit = Some(item["value"].clone());
                        }
                    }
                } else if text(row, "kind") == "buttons" {
                    ui.horizontal_wrapped(|ui| {
                        for (index, item) in array(&row["choices"]).enumerate() {
                            let r = ui.add_enabled(
                                item["enabled"] != false,
                                egui::Button::new(text(item, "label")),
                            );
                            record(geometry, id, &format!("row.{key}.choice.{index}"), &r);
                            if r.clicked() {
                                edit = Some(item["value"].clone());
                            }
                        }
                    });
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
                                        if r.changed() {
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
                                        ui.horizontal_wrapped(|ui| {
                                            for (index, item) in array(&row["choices"]).enumerate()
                                            {
                                                let r = ui.add_enabled(
                                                    item["enabled"] != false,
                                                    egui::Button::selectable(
                                                        item["value"] == row["value"],
                                                        text(item, "label"),
                                                    ),
                                                );
                                                record(
                                                    geometry,
                                                    id,
                                                    &format!("row.{key}.choice.{index}"),
                                                    &r,
                                                );
                                                if r.clicked() {
                                                    edit = Some(item["value"].clone());
                                                }
                                            }
                                        });
                                    }
                                    "number" => {
                                        let mut v = row["value"].as_f64().unwrap_or(0.0);
                                        let range = row["min"].as_f64().unwrap_or(0.0)
                                            ..=row["max"].as_f64().unwrap_or(100.0);
                                        let r = if row["slider"] == true {
                                            let mut slider = egui::Slider::new(&mut v, range)
                                                .suffix(text(row, "unit"));
                                            if let Some(places) = row["places"].as_u64() {
                                                slider = slider.fixed_decimals(places as usize);
                                            } else {
                                                slider = slider.integer();
                                            }
                                            ui.spacing_mut().slider_width =
                                                (ui.available_width() - 90.0).max(48.0);
                                            ui.add(slider)
                                        } else {
                                            ui.add(
                                                egui::DragValue::new(&mut v)
                                                    .range(range)
                                                    .speed(row["step"].as_f64().unwrap_or(1.0)),
                                            )
                                        };
                                        record(geometry, id, &format!("row.{key}.control"), &r);
                                        if r.changed() {
                                            edit = Some(json!(v));
                                        }
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
                                        let mut field = if text(row, "kind") == "multitext" {
                                            egui::TextEdit::multiline(&mut v).desired_rows(3)
                                        } else {
                                            egui::TextEdit::singleline(&mut v)
                                        };
                                        field = field
                                            .id(field_id)
                                            .desired_width(ui.available_width())
                                            .hint_text(text(row, "hint"))
                                            .password(secret);
                                        let r = ui.add(field);
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
            let mut changed = if r.changed() {
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
    geometry: &mut BTreeMap<String, Rect>,
    events: &mut Vec<Intent>,
) {
    let key = text(row, "id");
    let items: Vec<_> = array(&row["items"]).collect();
    ui.add_enabled_ui(row["blocked"].is_null(), |ui| {
        for (index, item) in items.iter().enumerate() {
            ui.horizontal(|ui| {
                ui.label(text(item, "label"));
                for (action, title, enabled) in [
                    ("up", "↑", index > 0),
                    ("down", "↓", index + 1 < items.len()),
                    ("remove", "×", true),
                ] {
                    let r = ui.add_enabled(enabled, egui::Button::new(title));
                    r.widget_info(|| {
                        egui::WidgetInfo::labeled(
                            egui::Role::Button,
                            ui.is_enabled(),
                            format!("{action} {}", text(item, "label")),
                        )
                    });
                    geometry.insert(format!("{id}.row.{key}.{index}.{action}"), r.rect);
                    if r.clicked() {
                        let mut values: Vec<Value> =
                            items.iter().map(|item| item["value"].clone()).collect();
                        match action {
                            "up" => values.swap(index, index - 1),
                            "down" => values.swap(index, index + 1),
                            _ => {
                                values.remove(index);
                            }
                        }
                        emit(events, id, "edit", json!({"row":key,"value":values}));
                    }
                }
            });
        }
        let r = ui.button(format!("+ {}", text(row, "title")));
        geometry.insert(format!("{id}.row.{key}.control"), r.rect);
        if r.clicked() {
            emit(events, id, "activate", json!({"row":key}));
        }
    });
}
