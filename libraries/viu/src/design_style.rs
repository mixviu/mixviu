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
    let tokens = dictionary
        .style
        .ok_or("compiled design omitted style tokens")?;
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
    if value["reduced_motion"] == true {
        style.animation_time = 0.0;
        style.scroll_animation = egui::style::ScrollAnimation::none();
    }
    let colours=dictionary.chrome.as_ref().map(|c|c.iter().map(|(name,value)|(name.to_owned(),colour(value))).collect()).unwrap_or_default();
    Ok(crate::appearance::ResolvedAppearance{style,colours})
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
                        .unwrap().style;
                    assert!(style.text_styles[&TextStyle::Body].size > 0.0);
                    assert!(style.spacing.interact_size.is_finite());
                }
            }
        }
    }

    #[test]
    fn bad_context_refuses_and_studio_projects_the_authored_chrome() {
        for value in [
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
}
