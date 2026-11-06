// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared application typography and palette adaptation. The application
//! supplies its chosen mode, accent and metrics; this module owns egui wiring.
use egui::{Color32, FontFamily, FontId, TextStyle};
use serde_json::Value;
pub fn session_theme(ctx: &egui::Context) -> toolkit::Theme {
    ctx.data(|d| d.get_temp::<toolkit::Theme>(egui::Id::unique("viu-session-theme")))
        .unwrap_or_else(toolkit::Theme::embedded)
}
#[derive(Clone)]
pub struct ResolvedAppearance {
    pub style: egui::Style,
    pub colours: std::collections::BTreeMap<String, Color32>,
    pub body_weight: u16,
    pub heading_weight: u16,
    pub icon_stroke: f32,
    pub icon_mask: [u8; 4],
    pub frame: crate::window_frame::Metrics,
}
pub fn colour(ctx: &egui::Context, name: &str) -> Option<Color32> {
    ctx.data(|d| {
        d.get_temp::<std::sync::Arc<std::collections::BTreeMap<String, Color32>>>(egui::Id::unique(
            "viu-design-colours",
        ))
    })?
    .get(name)
    .copied()
}
pub fn install(ctx: &egui::Context) {
    crate::fonts::install(ctx, 400, 600);
    crate::icons::install(ctx);
}
/// Follow the same complete look contract as the gold applications, including
/// app overrides, auto mode, geometry, font choices and tunables.
pub fn install_session(
    ctx: &egui::Context,
    snapshot: &Value,
    app: &str,
    file: &toolkit::Theme,
    last_ui: &settings::model::CommonUi,
) -> Result<settings::model::CommonUi, String> {
    let (theme, ui) = session_base(snapshot, app, file, last_ui)?;
    install_theme(ctx, &theme)?;
    ctx.data_mut(|d| {
        d.insert_temp(egui::Id::unique("viu-session-theme"), theme);
        d.remove::<Value>(egui::Id::unique("viu-local-choice"));
    });
    Ok(ui)
}
pub fn session_base(
    snapshot: &Value,
    app: &str,
    file: &toolkit::Theme,
    last_ui: &settings::model::CommonUi,
) -> Result<(toolkit::Theme, settings::model::CommonUi), String> {
    use settings::view::View;
    let values: settings::registry::Values =
        serde_json::from_value(snapshot["state"]["values"].clone()).map_err(|e| e.to_string())?;
    let look = settings::view::Look::of(&values);
    let lost = snapshot["state"]["lost"] == true;
    let theme = toolkit::session_look::session_theme(
        if lost { None } else { Some(&look) },
        Some(app),
        look.appearance.source.is_some(),
        file,
        last_ui,
    );
    let ui = if lost {
        last_ui.clone()
    } else {
        toolkit::session_look::ui(&look, Some(app))
    };
    Ok((theme, ui))
}
pub fn install_session_choice(
    ctx: &egui::Context,
    snapshot: &Value,
    app: &str,
    file: &toolkit::Theme,
    last_ui: &settings::model::CommonUi,
    choice: &Value,
) -> Result<settings::model::CommonUi, String> {
    let (base, ui) = session_base(snapshot, app, file, last_ui)?;
    let empty = serde_json::json!({});
    let choice = if choice.is_null() { &empty } else { choice };
    let theme = base
        .with_choice(&crate::native_theme::choice(choice)?)
        .map_err(|e| e.to_string())?;
    install_theme(ctx, &theme)?;
    Ok(ui)
}
pub fn install_theme(ctx: &egui::Context, theme: &toolkit::Theme) -> Result<(), String> {
    toolkit::install(ctx, theme);
    let editor_palette = editor::Palette::from_theme(theme);
    // Normal clients have the same presentation in debug and release builds.
    // Fractional spacing is valid; egui's development overlay is opt-in.
    #[cfg(debug_assertions)]
    for theme in [egui::Theme::Light, egui::Theme::Dark] {
        ctx.style_mut_of(theme, |style| style.debug.show_unaligned = false);
    }
    ctx.data_mut(|d| d.insert_temp(egui::Id::unique("viu-session-theme"), theme.clone()));
    let chrome = toolkit::chrome::Chrome::of(ctx);
    let mut colours = std::collections::BTreeMap::new();
    for role in resolve(&serde_json::json!({}))?.colours.keys() {
        if let Some(colour) = chrome.palette.get(role) {
            colours.insert(role.clone(), colour);
        }
    }
    colours.insert("editor.background".into(), editor_palette.background);
    colours.insert(
        "editor.light-background".into(),
        editor::Palette::from_theme(toolkit::content::light_theme()).background,
    );
    let m = chrome.metrics;
    ctx.data_mut(|d| {
        d.insert_temp(egui::Id::unique("viu-resolved-theme"), theme.clone());
        d.insert_temp(egui::Id::unique("viu-gold-editor-palette"), editor_palette);
        d.insert_temp(egui::Id::unique("viu-gold-fonts"), true);
        d.insert_temp(
            egui::Id::unique("viu-design-colours"),
            std::sync::Arc::new(colours),
        );
        d.insert_temp(
            egui::Id::unique("viu-window-metrics"),
            crate::window_frame::Metrics {
                height: m.title_bar_height,
                caption_width: m.caption_width,
                caption_glyph: m.caption_glyph,
                title_size: m.title_size,
                resize_edge: m.resize_edge,
                resize_corner: m.resize_corner,
            },
        );
    });
    crate::icons::install_style(
        ctx,
        toolkit::icons::stroke_width(theme),
        chrome.style.icon_mask.0,
    );
    Ok(())
}

/// A window choice overlays the live session theme, as in gold clients.
pub fn local_mode(ctx: &egui::Context, name: &str) -> Result<(), String> {
    let mode = match name {
        "dark" => design::Mode::Dark,
        "light" => design::Mode::Light,
        _ => return Err("invalid local mode".into()),
    };
    local_choice(ctx, &serde_json::json!({"mode":mode.name()}))
}
pub fn local_choice(ctx: &egui::Context, value: &Value) -> Result<(), String> {
    let key = egui::Id::unique("viu-local-choice");
    if ctx.data(|d| d.get_temp::<Value>(key)).as_ref() == Some(value) {
        return Ok(());
    }
    let choice = toolkit::theme::Choice {
        scheme: value["theme"].as_str().and_then(design::Scheme::from_name),
        mode: value["mode"].as_str().and_then(design::Mode::from_name),
        style: value["style"].as_str().map(|s| design::Style::from_name(s)),
        decorations: value["decorations"]
            .as_str()
            .and_then(design::RequestedDecorations::from_name),
        captions: value["captions"]
            .as_str()
            .and_then(design::CaptionSide::from_name),
    };
    let theme = ctx
        .data(|d| d.get_temp::<toolkit::Theme>(egui::Id::unique("viu-session-theme")))
        .unwrap_or_else(toolkit::Theme::load);
    let theme = theme.with_choice(&choice).map_err(|e| e.to_string())?;
    toolkit::install(ctx, &theme);
    let palette = editor::Palette::from_theme(&theme);
    let chrome = toolkit::chrome::Chrome::of(ctx);
    let mut colours = resolve(&serde_json::json!({}))?
        .colours
        .keys()
        .filter_map(|role| chrome.palette.get(role).map(|c| (role.clone(), c)))
        .collect::<std::collections::BTreeMap<_, _>>();
    colours.insert("editor.background".into(), palette.background);
    colours.insert(
        "editor.light-background".into(),
        editor::Palette::from_theme(toolkit::content::light_theme()).background,
    );
    ctx.data_mut(|d| {
        d.insert_temp(key, value.clone());
        d.insert_temp(egui::Id::unique("viu-resolved-theme"), theme);
        d.insert_temp(egui::Id::unique("viu-gold-editor-palette"), palette);
        d.insert_temp(
            egui::Id::unique("viu-design-colours"),
            std::sync::Arc::new(colours),
        );
    });
    Ok(())
}
pub fn apply(ctx: &egui::Context, description: &Value, accent: [u8; 3]) {
    let mut style = egui::Style::default();
    style.visuals = if description["mode"] == "light" {
        egui::Visuals::light()
    } else {
        egui::Visuals::dark()
    };
    let body = description["body"]
        .as_f64()
        .unwrap_or(14.666667)
        .clamp(8.0, 36.0) as f32;
    for (text, size, family) in [
        (TextStyle::Body, body, FontFamily::Proportional),
        (TextStyle::Button, body, FontFamily::Proportional),
        (TextStyle::Small, 12.0, FontFamily::Proportional),
        (TextStyle::Heading, body * 1.35, FontFamily::Proportional),
        (TextStyle::Monospace, body, FontFamily::Monospace),
    ] {
        style.text_styles.insert(text, FontId::new(size, family));
    }
    style.spacing.item_spacing = egui::vec2(6.0, 4.0);
    style.spacing.interact_size.y = 24.0;
    style.visuals.selection.bg_fill = Color32::from_rgb(accent[0], accent[1], accent[2]);
    style.visuals.selection.stroke = egui::Stroke::new(1.0, Color32::WHITE);
    let theme = if description["mode"] == "light" {
        egui::Theme::Light
    } else {
        egui::Theme::Dark
    };
    ctx.set_theme(theme);
    ctx.set_style_of(theme, style);
}

/// Resolve before committing a scene update so a bad design leaves the live
/// model, geometry and style intact.
pub fn resolve(description: &Value) -> Result<ResolvedAppearance, String> {
    crate::design_style::resolve(description)
}

pub fn install_style(ctx: &egui::Context, description: &Value, appearance: ResolvedAppearance) {
    ctx.data_mut(|d| d.insert_temp(egui::Id::unique("viu-gold-fonts"), false));
    crate::fonts::install(ctx, appearance.body_weight, appearance.heading_weight);
    crate::icons::install_style(ctx, appearance.icon_stroke, appearance.icon_mask);
    let theme = if description["mode"] == "light" {
        egui::Theme::Light
    } else {
        egui::Theme::Dark
    };
    let style = appearance.style.clone();
    ctx.data_mut(|d| {
        d.insert_temp(egui::Id::unique("viu-window-metrics"), appearance.frame);
        d.insert_temp(
            egui::Id::unique("viu-design-colours"),
            std::sync::Arc::new(appearance.colours),
        )
    });
    ctx.set_theme(theme);
    ctx.set_style_of(egui::Theme::Light, style.clone());
    ctx.set_style_of(egui::Theme::Dark, style);
}

pub fn install_session_choice(
    ctx: &egui::Context,
    snapshot: &Value,
    app: &str,
    file: &toolkit::Theme,
    last_ui: &settings::model::CommonUi,
    choice: &Value,
) -> Result<settings::model::CommonUi, String> {
    let (base, ui) = session_base(snapshot, app, file, last_ui)?;
    let theme = base
        .with_choice(&theme_choice(choice)?)
        .map_err(|e| e.to_string())?;
    install_theme(ctx, &theme)?;
    Ok(ui)
}
