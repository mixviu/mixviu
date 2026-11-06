// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
use decor::{ChromeStyle, ChromeTheme, Palette};
use design::TypographyRole;
use settings::{Diagnostic, Snapshot};
use std::sync::Arc;
use toolkit::theme::Theme;
const CONTEXT: &str = "desktop";
#[derive(Clone, Debug)]
pub struct Look {
    pub theme: Arc<Theme>,
    pub text_scale: f32,
    pub window_style: ChromeStyle,
    native: decor::appearance::Look,
}
pub fn build(snapshot: &Snapshot) -> Result<Look, Diagnostic> {
    let native = decor::appearance::build(snapshot)?;
    Ok(Look {
        theme: Arc::new((*native.theme).clone().into()),
        text_scale: native.text_scale,
        window_style: native.window_style,
        native,
    })
}
impl Look {
    pub fn chrome(&self, style: ChromeStyle) -> ChromeTheme {
        self.native.chrome(style)
    }
    pub fn palette(&self, dialog: bool) -> Palette {
        page(&self.theme, dialog)
    }
}
pub(crate) fn ui_size(theme: &Theme, text_scale: f32) -> f32 {
    let record = design::active_typography(Some(theme.typography()), TypographyRole::Ui);
    (if record.font_size > 0.0 {
        record.font_size as f32
    } else {
        crate::view::TEXT_SIZE
    }) * theme.interface_size_scale()
        * text_scale
}
pub(crate) fn page(theme: &Theme, dialog: bool) -> Palette {
    let mut palette = Palette::from_dictionary(theme.dictionary());
    if !dialog {
        palette.base = palette.secondary;
    }
    palette
}
pub(crate) fn install(ctx: &egui::Context, theme: &Theme, text_scale: f32, dialog: bool) {
    let mut ui = theme.ui().clone();
    ui.text_scale *= f64::from(text_scale);
    let scaled = theme.clone().with_ui(ui);
    toolkit::install(ctx, &scaled);
    let palette = page(theme, dialog);
    let fill = crate::view::colour(palette.base.surface);
    let ink = crate::view::colour(palette.base.foreground);
    ctx.global_style_mut(|style| {
        style.visuals.panel_fill = fill;
        style.visuals.window_fill = fill;
        style.visuals.override_text_color = Some(ink);
    });
}

#[cfg(test)]
pub(crate) fn fixture(mode: &str, scale: f64) -> Arc<Look> {
    let mut desktop = settings::Desktop::default();
    desktop.appearance.mode = mode.into();
    desktop.ui.text_scale = scale;
    Arc::new(build(&snapshot_of(desktop)).unwrap())
}

/// The snapshot settingsd serves for `desktop` once migrated: its v2
/// values (key space v2 §4.1), resolved and projected as schema 3 carries
/// them.
#[cfg(test)]
pub(crate) fn snapshot_of(desktop: settings::Desktop) -> Snapshot {
    let values = settings::migrate::migrate(&desktop).unwrap().values;
    let effective =
        settings::resolve::values::resolve(&values, settings::EMBEDDED_DEFAULT_SOURCE).unwrap();
    Snapshot {
        schema: settings::SCHEMA,
        binding: settings::Binding {
            instance: "desktop".into(),
            profile: "test".into(),
        },
        incarnation: "test".into(),
        revision: settings::Revision(1),
        design_revision: settings::Revision(1),
        source_digest: settings::source_digest(settings::EMBEDDED_DEFAULT_SOURCE),
        desktop: settings::migrate::reverse(&values).desktop,
        effective,
        values,
    }
}
