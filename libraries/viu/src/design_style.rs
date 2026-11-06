// SPDX-License-Identifier: MIT OR Apache-2.0
//! Native design-token projection, adapted from the retained toolkit style and
//! chrome adapters. It renders an application-supplied design context; no app
//! command, settings service, filesystem operation or compositor is owned here.
use design::family::style::{ResolvedStyle, Selection, WidgetStroke, Widgets};
use design::{DesignCompileResult, DesignContext, Mode, Scheme, SourceIdentity};
use egui::{Color32, CornerRadius, FontFamily, FontId, Margin, Stroke, TextStyle, Visuals};
use serde_json::Value;
fn colour(c: design::LinearRgba) -> Color32 {
    let [r, g, b, a] = c.to_srgba8();
    Color32::from_rgba_unmultiplied(r, g, b, a)
}
fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let blend = |x: u8, y: u8| (f32::from(x) + (f32::from(y) - f32::from(x)) * t).round() as u8;
    Color32::from_rgba_unmultiplied(
        blend(a.r(), b.r()),
        blend(a.g(), b.g()),
        blend(a.b(), b.b()),
        blend(a.a(), b.a()),
    )
}
pub fn resolve(value: &Value) -> Result<crate::appearance::ResolvedAppearance, String> {
    if !value.is_object() {
        return Err("design context must be an object".into());
    }
    for key in ["scheme", "mode", "style", "contrast", "accent"] {
        if value.get(key).is_some_and(|v| !v.is_string()) {
            return Err(format!("design {key} must be text"));
        }
    }
    if value.get("body").is_some_and(|v| !v.is_number())
        || value.get("reduced_motion").is_some_and(|v| !v.is_boolean())
    {
        return Err("invalid typography or motion choice".into());
    }
    let mut context = DesignContext::default();
    context.scheme = Scheme::from_name(value["scheme"].as_str().unwrap_or("studio"))
        .ok_or("unknown design scheme")?;
    context.mode =
        Mode::from_name(value["mode"].as_str().unwrap_or("dark")).ok_or("unknown design mode")?;
    context.contrast = design::Contrast::from_name(value["contrast"].as_str().unwrap_or("normal"))
        .ok_or("unknown design contrast")?;
    if let Some(style) = value["style"].as_str().filter(|s| *s != "default") {
        context.style = Some(design::Style::from_name(style).ok_or("unknown design style")?);
    }
    if let Some(accent) = value["accent"].as_str() {
        context.accent = design::AccentChoice::from_name(accent).ok_or("unknown accent")?;
    }
    let document = design::parse_design_source(
        SourceIdentity::new("viu-embedded-design"),
        design::EMBEDDED_DEFAULT_SOURCE,
    )
    .map_err(|e| format!("{e:?}"))?;
    let compiled = design::compile_design(&document, context.clone());
    let DesignCompileResult::Success(compiled) = compiled else {
        return Err("embedded native design did not compile".into());
    };
    let dictionary = compiled.candidate.dictionary();
    let mut tokens = dictionary
        .style
        .ok_or("compiled design omitted style tokens")?;
    if let Some(roundness) = value.get("roundness") {
        tokens = design::geometry::round(
            tokens,
            design::geometry::Roundness::from_name(
                roundness.as_str().ok_or("roundness must be text")?,
            )
            .ok_or("unknown roundness")?,
        );
    }
    if let Some(density) = value.get("density") {
        let density =
            design::geometry::Density::from_name(density.as_str().ok_or("density must be text")?)
                .ok_or("unknown density")?;
        tokens =
            design::geometry::legacy_lengths(design::geometry::spacing(tokens, density), density);
    }
    let mut style = egui::Style::default();
    style.visuals = pair_visuals(dictionary, &tokens, context.mode);
    if tokens.widgets == Widgets::Chrome {
        style.visuals = chrome_visuals(dictionary, &tokens, context.mode)?;
    }
    let spacing = &mut style.spacing;
    spacing.item_spacing = egui::vec2(tokens.item_spacing_x as f32, tokens.item_spacing_y as f32);
    spacing.button_padding = egui::vec2(
        tokens.button_padding_x as f32,
        tokens.button_padding_y as f32,
    );
    spacing.interact_size = egui::Vec2::splat(tokens.interact as f32);
    spacing.window_margin = Margin::same(tokens.window_margin as i8);
    spacing.menu_margin = Margin::same(tokens.menu_margin as i8);
    spacing.slider_width = tokens.slider_width as f32;
    spacing.combo_width = tokens.combo_width as f32;
    spacing.icon_width = tokens.icon_width as f32;
    spacing.tooltip_width = tokens.tooltip_width as f32;
    spacing.menu_width = tokens.menu_min_width as f32;
    spacing.scroll = scroll(&tokens);
    style.interaction.tooltip_delay = tokens.tooltip_delay as f32;
    style.interaction.tooltip_grace_time = tokens.tooltip_grace as f32;
    style.interaction.show_tooltips_only_when_still = true;
    style.animation_time = tokens.animation_time as f32;
    let typography = compiled.candidate.typography();
    let sizes = if tokens.widgets == Widgets::Chrome {
        [
            tokens.body_size,
            tokens.small_size,
            tokens.heading_size,
            tokens.mono_size,
        ]
    } else {
        [
            design::active_typography(Some(typography), design::TypographyRole::Ui).font_size,
            design::active_typography(Some(typography), design::TypographyRole::Small).font_size,
            design::active_typography(Some(typography), design::TypographyRole::UiDisplay)
                .font_size,
            design::active_typography(Some(typography), design::TypographyRole::Mono).font_size,
        ]
    };
    for (key, size, family) in [
        (TextStyle::Body, sizes[0], FontFamily::Proportional),
        (TextStyle::Button, sizes[0], FontFamily::Proportional),
        (TextStyle::Small, sizes[1], FontFamily::Proportional),
        (TextStyle::Heading, sizes[2], FontFamily::Proportional),
        (TextStyle::Monospace, sizes[3], FontFamily::Monospace),
    ] {
        style
            .text_styles
            .insert(key, FontId::new(size as f32, family));
    }
    if let Some(size) = value["body"].as_f64() {
        if !size.is_finite() || !(8.0..=36.0).contains(&size) {
            return Err("body size outside supported range".into());
        }
        let ratio = size as f32 / style.text_styles[&TextStyle::Body].size;
        for (role, font) in &mut style.text_styles {
            if *role != TextStyle::Monospace {
                font.size *= ratio;
            }
        }
    }
    if let Some(scale) = value.get("text_scale") {
        let scale = scale
            .as_f64()
            .filter(|s| s.is_finite() && (0.5..=3.0).contains(s))
            .ok_or("invalid text scale")? as f32;
        for font in style.text_styles.values_mut() {
            font.size *= scale;
        }
    }
    if value["reduced_motion"] == true {
        style.animation_time = 0.0;
        style.scroll_animation = egui::style::ScrollAnimation::none();
    }
    let mut colours = palette_from_visuals(&style.visuals);
    // Editor highlighting resolves semantic primitives (success, warning,
    // accent hover) as well as chrome roles, just like the gold toolkit.
    for (name, value) in &dictionary.colours.primitives {
        colours
            .entry(name.clone())
            .or_insert_with(|| colour(*value));
    }
    if tokens.widgets == Widgets::Chrome {
        if let Some(chrome) = &dictionary.chrome {
            colours.extend(
                chrome
                    .iter()
                    .map(|(name, value)| (name.to_owned(), colour(value))),
            );
        }
    }
    let (mut body_weight, mut heading_weight) = if tokens.faces
        == design::family::style::Faces::Fixed
    {
        (400u16, 600u16)
    } else {
        (
            design::active_typography(Some(typography), design::TypographyRole::Ui).weight,
            design::active_typography(Some(typography), design::TypographyRole::UiDisplay).weight,
        )
    };
    if let Some(weight) = value.get("weight") {
        let weight = weight
            .as_f64().filter(|w| w.is_finite() && w.fract() == 0.0).map(|w| w as u64)
            .filter(|w| (100..=900).contains(w) && w % 100 == 0)
            .ok_or("font weight must be 100..900 in steps of 100")? as u16;
        let offset = i32::from(heading_weight) - i32::from(body_weight);
        body_weight = weight;
        heading_weight = (i32::from(weight) + offset).clamp(300, 700) as u16;
    }
    if let Some(font) = style.text_styles.get_mut(&TextStyle::Heading) {
        font.family = FontFamily::Name("viu-heading".into());
    }
    let icon_stroke = value
        .get("icon_stroke")
        .map(|v| {
            v.as_f64()
                .filter(|v| v.is_finite() && (1.0..=3.0).contains(v) && (*v * 4.0).fract() == 0.0)
                .ok_or("icon stroke must be 1..3 in quarter steps")
        })
        .transpose()?
        .unwrap_or_else(|| {
            dictionary
                .metrics
                .get("icon.stroke_width")
                .filter(|m| m.kind == design::ResolvedMetricKind::Px && m.value > 0.0)
                .map_or(1.5, |m| m.value)
        }) as f32;
    Ok(crate::appearance::ResolvedAppearance {
        style,
        colours,
        body_weight,
        heading_weight,
        icon_stroke,
        icon_mask: tokens.icon_mask.0,
        frame: crate::window_frame::Metrics {
            height: tokens.title_bar_height as f32,
            caption_width: tokens.caption_width as f32,
            caption_glyph: tokens.caption_glyph as f32,
            title_size: tokens.title_size as f32,
            resize_edge: tokens.resize_edge as f32,
            resize_corner: tokens.resize_corner as f32,
        },
    })
}
// The retained native chrome adapter reads roles back from pair visuals for
// pair widgets. Using the dictionary's derived chrome palette here would give
// hue schemes a different dock surface from the reference application.
fn palette_from_visuals(v: &Visuals) -> std::collections::BTreeMap<String, Color32> {
    let w = &v.widgets;
    let text = v.text_color();
    [
        ("chrome", v.panel_fill),
        ("canvas", v.extreme_bg_color),
        ("canvas_dot", v.extreme_bg_color),
        ("dock", v.faint_bg_color),
        ("card", v.window_fill),
        ("card_border", v.window_stroke.color),
        ("tab_strip", v.faint_bg_color),
        ("field", v.extreme_bg_color),
        ("field_border", w.inactive.bg_stroke.color),
        ("hover", w.hovered.weak_bg_fill),
        ("pressed", w.active.weak_bg_fill),
        ("row_selected", v.selection.bg_fill),
        ("text", text),
        ("text_dim", text),
        ("text_faint", v.weak_text_color()),
        ("icon", text),
        ("accent", v.selection.bg_fill),
        ("accent_soft", v.selection.bg_fill),
        ("accent_border", v.selection.stroke.color),
        ("accent_text", v.selection.stroke.color),
        ("menu_highlight", w.hovered.weak_bg_fill),
        ("menu_highlight_text", text),
        ("separator", w.noninteractive.bg_stroke.color),
        ("shadow", v.popup_shadow.color),
        ("scrim", v.popup_shadow.color),
        ("primary_bg", v.hyperlink_color),
        ("primary_text", v.panel_fill),
        ("danger", v.error_fg_color),
        ("warning", v.warn_fg_color),
        ("caption_close", v.error_fg_color),
        ("caption_close_text", v.panel_fill),
    ]
    .into_iter()
    .map(|(name, c)| (name.to_owned(), c))
    .collect()
}
fn pair_visuals(dictionary: &design::ResolvedDictionary, t: &ResolvedStyle, mode: Mode) -> Visuals {
    let mut v = if mode == Mode::Dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    let c = &dictionary.colours;
    let pair = |key: &str| {
        c.pairs
            .get(key)
            .map(|p| (colour(p.rendered_surface), colour(p.rendered_foreground)))
    };
    if let Some((surface, text)) = pair("base") {
        v.panel_fill = surface;
        v.override_text_color = Some(text);
        v.widgets.noninteractive.bg_fill = surface;
        v.widgets.noninteractive.weak_bg_fill = surface;
        v.widgets.noninteractive.fg_stroke.color = text;
        v.text_edit_bg_color = Some(surface);
    }
    if let Some((surface, _)) = pair("popover") {
        v.window_fill = surface;
    }
    if let Some((surface, _)) = pair("card") {
        v.faint_bg_color = surface;
    }
    if let Some((surface, _)) = pair("muted") {
        v.extreme_bg_color = surface;
        v.code_bg_color = surface;
    }
    if let Some((surface, text)) = pair("secondary") {
        v.widgets.inactive.bg_fill = surface;
        v.widgets.inactive.weak_bg_fill = surface;
        v.widgets.inactive.fg_stroke.color = text;
        for (state, fill) in [
            (
                &mut v.widgets.hovered,
                mix(surface, text, t.pair_hover_mix as f32),
            ),
            (
                &mut v.widgets.open,
                mix(surface, text, t.pair_hover_mix as f32),
            ),
            (
                &mut v.widgets.active,
                mix(surface, text, t.pair_active_mix as f32),
            ),
        ] {
            state.bg_fill = fill;
            state.weak_bg_fill = fill;
            state.fg_stroke.color = text;
        }
    }
    if let Some((_, text)) = pair("accent") {
        v.selection.stroke.color = text;
    }
    if let Some(accent) = design::accent_for(dictionary, Widgets::Pairs) {
        let [r, g, b, a] = accent.to_srgba8();
        v.selection.bg_fill = Color32::from_rgba_unmultiplied(r, g, b, a);
    }
    if let Some((surface, _)) = pair("primary") {
        v.hyperlink_color = surface;
    }
    if let Some((surface, _)) = pair("destructive") {
        v.error_fg_color = surface;
        v.warn_fg_color = surface;
    }
    if let Some(warning) = c.primitives.get("status.warning") {
        v.warn_fg_color = colour(*warning);
    }
    if let Some(border) = c.non_text.get("border") {
        let border = colour(border.value);
        for widget in [
            &mut v.widgets.noninteractive,
            &mut v.widgets.inactive,
            &mut v.widgets.hovered,
            &mut v.widgets.active,
            &mut v.widgets.open,
        ] {
            widget.bg_stroke.color = border;
        }
        v.window_stroke.color = border;
    }
    for widget in [
        &mut v.widgets.noninteractive,
        &mut v.widgets.inactive,
        &mut v.widgets.hovered,
        &mut v.widgets.active,
        &mut v.widgets.open,
    ] {
        widget.corner_radius = CornerRadius::same(t.radius_sm);
        widget.bg_stroke.width = t.hairline_width as f32;
    }
    v.window_stroke.width = t.hairline_width as f32;
    v.menu_corner_radius = CornerRadius::same(t.radius);
    v.window_corner_radius = CornerRadius::same(t.radius_lg);
    v.popup_shadow = egui::Shadow {
        offset: [t.popup_shadow_offset_x as i8, t.popup_shadow_offset_y as i8],
        blur: t.popup_shadow_blur as u8,
        spread: t.popup_shadow_spread as u8,
        color: v
            .popup_shadow
            .color
            .gamma_multiply(t.popup_shadow_alpha as f32),
    };
    v
}
fn chrome_visuals(
    dictionary: &design::ResolvedDictionary,
    t: &ResolvedStyle,
    mode: Mode,
) -> Result<Visuals, String> {
    let roles = dictionary
        .chrome
        .as_ref()
        .ok_or("native design omitted chrome roles")?;
    let get = |name: &str| {
        roles
            .get(name)
            .map(colour)
            .ok_or_else(|| format!("missing chrome role {name}"))
    };
    let chrome = get("chrome")?;
    let rgba = egui::Rgba::from(chrome);
    let dark = 0.2126 * rgba.r() + 0.7152 * rgba.g() + 0.0722 * rgba.b() <= t.dark_luminance as f32;
    let mut v = if dark {
        Visuals::dark()
    } else {
        Visuals::light()
    };
    let _ = mode;
    v.panel_fill = chrome;
    v.window_fill = get("card")?;
    v.window_stroke = Stroke::new(t.hairline_width as f32, get("card_border")?);
    v.extreme_bg_color = get("field")?;
    v.code_bg_color = get("field")?;
    v.text_edit_bg_color = Some(get("field")?);
    v.faint_bg_color = get("card")?;
    v.hyperlink_color = get("accent")?;
    if let Some(accent) = design::accent_for(dictionary, Widgets::Chrome) {
        let [r, g, b, a] = accent.to_srgba8();
        v.hyperlink_color = Color32::from_rgba_unmultiplied(r, g, b, a);
    }
    v.warn_fg_color = get("warning")?;
    v.error_fg_color = get("danger")?;
    v.override_text_color = Some(get("text")?);
    v.weak_text_alpha = t.weak_alpha as f32;
    v.disabled_alpha = t.disabled_alpha as f32;
    v.selection.bg_fill = match t.selection {
        Selection::Accent => v.hyperlink_color,
        Selection::AccentSoft => get("accent_soft")?,
    };
    v.selection.stroke = Stroke::new(t.hairline_width as f32, get("accent_text")?);
    let stroke = |role: &str| -> Result<Stroke, String> {
        Ok(Stroke::new(
            t.hairline_width as f32,
            get(if t.widget_stroke == WidgetStroke::Bevel {
                "text_dim"
            } else {
                role
            })?,
        ))
    };
    for (state, fill, border, text) in [
        (
            &mut v.widgets.noninteractive,
            get("card")?,
            Stroke::new(t.hairline_width as f32, get("separator")?),
            get("text_dim")?,
        ),
        (
            &mut v.widgets.inactive,
            get("field")?,
            stroke("field_border")?,
            get("text")?,
        ),
        (
            &mut v.widgets.hovered,
            get("hover")?,
            stroke("field_border")?,
            get("text")?,
        ),
        (
            &mut v.widgets.active,
            get("pressed")?,
            stroke("accent_border")?,
            get("text")?,
        ),
        (
            &mut v.widgets.open,
            get("hover")?,
            stroke("field_border")?,
            get("text")?,
        ),
    ] {
        state.bg_fill = fill;
        state.weak_bg_fill = fill;
        state.bg_stroke = border;
        state.fg_stroke.color = text;
        state.corner_radius = CornerRadius::same(t.radius_sm);
        state.expansion = 0.0;
    }
    v.window_corner_radius = CornerRadius::same(t.radius_lg);
    v.menu_corner_radius = CornerRadius::same(t.radius);
    v.window_shadow = egui::Shadow {
        offset: [
            t.window_shadow_offset_x as i8,
            t.window_shadow_offset_y as i8,
        ],
        blur: t.window_shadow_blur as u8,
        spread: t.window_shadow_spread as u8,
        color: get("shadow")?,
    };
    v.popup_shadow = egui::Shadow {
        offset: [t.popup_shadow_offset_x as i8, t.popup_shadow_offset_y as i8],
        blur: t.popup_shadow_blur as u8,
        spread: t.popup_shadow_spread as u8,
        color: get("shadow")?,
    };
    v.slider_trailing_fill = true;
    v.handle_shape = egui::style::HandleShape::Circle;
    v.striped = false;
    v.indent_has_left_vline = false;
    Ok(v)
}
fn scroll(t: &ResolvedStyle) -> egui::style::ScrollStyle {
    let solid = t.scroll_bars == design::family::style::ScrollBars::Solid;
    egui::style::ScrollStyle {
        floating: !solid,
        bar_width: if solid {
            t.scroll_width
        } else {
            t.scroll_width_hot
        } as f32,
        floating_width: t.scroll_width as f32,
        floating_allocated_width: if solid {
            0.0
        } else {
            t.scroll_allocated as f32
        },
        handle_min_length: t.scroll_handle_min as f32,
        bar_inner_margin: t.scroll_margin as f32,
        bar_outer_margin: t.scroll_outer_margin as f32,
        foreground_color: true,
        dormant_background_opacity: t.scroll_track_alpha_rest as f32,
        active_background_opacity: t.scroll_track_alpha_active as f32,
        interact_background_opacity: t.scroll_track_alpha_hot as f32,
        dormant_handle_opacity: t.scroll_handle_alpha_rest as f32,
        active_handle_opacity: t.scroll_handle_alpha_active as f32,
        interact_handle_opacity: t.scroll_handle_alpha_hot as f32,
        ..egui::style::ScrollStyle::solid()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn every_native_scheme_mode_and_style_resolves_without_fallback() {
        for scheme in Scheme::ALL {
            for mode in ["dark", "light"] {
                for style in [
                    "default", "plain", "pro", "studio", "classic", "breeze", "adwaita",
                ] {
                    let style = resolve(&json!({"scheme":scheme.name(),"mode":mode,"style":style}))
                        .unwrap()
                        .style;
                    assert!(style.text_styles[&TextStyle::Body].size > 0.0);
                    assert!(style.spacing.interact_size.is_finite());
                }
            }
        }
    }

    #[test]
    fn bad_context_refuses_and_studio_projects_the_authored_chrome() {
        for value in [
            json!(null),
            json!([]),
            json!({"scheme":1}),
            json!({"body":"large"}),
            json!({"weight":550}),
            json!({"icon_stroke":1.1}),
            json!({"scheme":"missing"}),
            json!({"mode":"missing"}),
            json!({"style":"missing"}),
            json!({"contrast":"missing"}),
            json!({"accent":"missing"}),
            json!({"body":0}),
        ] {
            assert!(resolve(&value).is_err(), "{value}");
        }
        let document = design::parse_design_source(
            SourceIdentity::new("test"),
            design::EMBEDDED_DEFAULT_SOURCE,
        )
        .unwrap();
        let DesignCompileResult::Success(compiled) =
            design::compile_design(&document, DesignContext::default())
        else {
            panic!("default design");
        };
        let style = resolve(&json!({})).unwrap().style;
        let roles = compiled.candidate.dictionary().chrome.as_ref().unwrap();
        assert_eq!(
            style.visuals.panel_fill,
            colour(roles.get("chrome").unwrap())
        );
        assert_eq!(
            style.visuals.text_edit_bg_color,
            Some(colour(roles.get("field").unwrap()))
        );
        assert_eq!(
            style.visuals.override_text_color,
            Some(colour(roles.get("text").unwrap()))
        );
    }

    #[test]
    fn hue_dock_roles_follow_pair_visuals_and_fonts_follow_the_design() {
        for mode in ["dark", "light"] {
            let appearance = resolve(&json!({"scheme":"ocean","mode":mode})).unwrap();
            assert_eq!(
                appearance.colours["dock"],
                appearance.style.visuals.faint_bg_color
            );
            assert_eq!(
                appearance.colours["chrome"],
                appearance.style.visuals.panel_fill
            );
            assert_eq!(appearance.icon_stroke, 1.5);
            let ctx = egui::Context::default();
            crate::appearance::install_style(&ctx, &json!({"mode":mode}), appearance);
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                ui.heading("Heading");
                ui.label("A sanitised name �");
                ui.monospace("012345");
            });
            output.textures_delta.clear();
        }
    }
}
