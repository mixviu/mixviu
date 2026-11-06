// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! The shell's look, prepared on the Bus worker from an accepted settings
//! snapshot and activated as one value after the session fence: the MixOS
//! theme the desktop profile describes (`toolkit::theme::Theme`, the one the
//! apps draw with), its text scale, the panel preferences, and the
//! server-side decoration themes `decor` draws window frames with.
use settings::view::{Accessibility, Appearance, View, Window};
struct Profile {
    window: Window,
    appearance: Appearance,
    accessibility: Accessibility,
}
impl Profile {
    fn read(snapshot: &Snapshot) -> Result<Self, Diagnostic> {
        settings::registry::validate(&snapshot.values)
            .map_err(|errors| errors.into_iter().next().expect("validation error"))?;
        Ok(Self {
            window: Window::of(&snapshot.values),
            appearance: Appearance::of(&snapshot.values),
            accessibility: Accessibility::of(&snapshot.values),
        })
    }
}
use crate::{
    ChromeTheme, Palette,
    layout::{ChromeStyle, DecoFontFamily, DecoFontWeight},
};
use design::{
    CaptionSide, Contrast, DesignContext, Mode, RequestedDecorations, Scheme, Style,
    TypographyGeneric, TypographyRole,
};
use settings::{Diagnostic, Snapshot};
use std::sync::Arc;
use theme::theme::{Overlay, Theme};

/// The effective context the shell draws in.
pub(crate) const CONTEXT: &str = "desktop";

/// The proportional size used to measure and paint shell menus and cards.
pub(crate) fn ui_size(theme: &Theme, text_scale: f32) -> f32 {
    let record = design::active_typography(Some(theme.typography()), TypographyRole::Ui);
    let size = if record.font_size > 0.0 {
        record.font_size as f32
    } else {
        14.0
    };
    size * theme.interface_size_scale() * text_scale
}

#[derive(Clone, Debug)]
pub struct Look {
    pub theme: Arc<Theme>,
    /// The profile's text scale (`ui.text_scale`), applied to every text style.
    pub text_scale: f32,
    pub window_style: ChromeStyle,
    /// `window.gaps`: the inner gap between tiles, in pixels.
    pub gaps: u32,
    chrome: Vec<ChromeTheme>,
}

fn unsupported(path: &str, message: &str) -> Diagnostic {
    Diagnostic::new("unsupported_presentation", path, message)
}

/// The theme an accepted snapshot describes: its own design source when the
/// profile carries one, otherwise the embedded design, either way at the
/// effective scheme, style, mode and contrast settingsd accepted, framed as
/// the profile asks (`window.*`). The style and the source are the authored
/// `appearance.*` values; the font overlay still reads the format-4
/// appearance, the one toolkit constructor scene-host can reach.
fn theme_of(snapshot: &Snapshot, profile: &Profile) -> Result<Theme, Diagnostic> {
    let framing = &profile.window.framing;
    let effective = snapshot.effective.get(CONTEXT).ok_or_else(|| {
        Diagnostic::new(
            "missing_context",
            "effective",
            "Snapshot has no desktop context",
        )
    })?;
    let decorations = RequestedDecorations::from_name(&framing.decorations)
        .ok_or_else(|| unsupported("window.decorations", "Unknown decorations"))?;
    let captions = CaptionSide::resolve(&framing.caption_side);
    let style = match &profile.appearance.style {
        None => None,
        Some(name) => Some(
            Style::from_name(name)
                .ok_or_else(|| unsupported("appearance.style", "Unknown style"))?,
        ),
    };
    let context = DesignContext {
        scheme: Scheme::from_name(&effective.scheme)
            .ok_or_else(|| unsupported("appearance.scheme", "Unknown scheme"))?,
        mode: Mode::from_name(&effective.mode)
            .ok_or_else(|| unsupported("appearance.mode", "Unknown mode"))?,
        contrast: Contrast::from_name(&effective.contrast)
            .ok_or_else(|| unsupported("appearance.contrast", "Unknown contrast"))?,
        style,
        ..DesignContext::default()
    };
    let theme = match &profile.appearance.source {
        Some(source) => {
            Theme::from_source_in("settings:desktop", source, context).map_err(|error| {
                unsupported(
                    "appearance.source",
                    &format!("Design does not compile: {error:?}"),
                )
            })?
        }
        None => Theme::for_context(context),
    };
    let theme = overlay_or_design(
        theme,
        Overlay::from_appearance(&snapshot.desktop.appearance),
    );
    Ok(theme
        .framed(decorations, captions)
        .with_menus(
            design::MenuStyle::resolve(&framing.menu_style),
            design::MenuSide::resolve(&framing.menu_side),
        )
        .with_band(design::title_band::Settings::from_names(
            &framing.title_align,
            &framing.actions_align,
            &framing.actions_place,
            &framing.style,
        )))
}

/// An invalid optional overlay must never discard the accepted design.
fn overlay_or_design(theme: Theme, overlay: Overlay) -> Theme {
    match theme.clone().with_overlay(overlay) {
        Ok(overlaid) => overlaid,
        Err(error) => {
            static WARN: std::sync::Once = std::sync::Once::new();
            WARN.call_once(
                || tracing::warn!(%error, "scene host: ignoring invalid appearance overlay"),
            );
            theme
        }
    }
}

/// Build the whole look from an accepted snapshot (on the worker).
pub fn build(snapshot: &Snapshot) -> Result<Look, Diagnostic> {
    let profile = Profile::read(snapshot)?;
    let theme = theme_of(snapshot, &profile)?;
    let window_style = ChromeStyle::resolve(&profile.window.framing.style);
    let text_scale = (profile.accessibility.text_scale as f32).clamp(0.5, 3.0);
    let title = design::active_typography(Some(theme.typography()), TypographyRole::UiDisplay);
    let title_weight = if theme.overlay().weight.is_some() {
        theme::fonts::weights(&theme).1
    } else {
        title.weight
    };
    let size_ratio = theme.interface_size_scale();
    let family = match title.generic {
        TypographyGeneric::Monospace => DecoFontFamily::Monospace,
        _ if title.family.is_empty() || title.family == "Inter" => DecoFontFamily::SystemUi,
        _ => DecoFontFamily::Named(title.family.clone()),
    };
    let chrome = ChromeStyle::ALL
        .into_iter()
        .map(|style| {
            let mut chrome = ChromeTheme::from_read(
                style,
                theme.dictionary(),
                family.clone(),
                DecoFontWeight(title_weight),
                title.font_size as f32 * size_ratio * text_scale,
            );
            chrome.menu_style = theme.menu_style();
            chrome.menu_side = theme.menu_side();
            chrome.band = theme.band();
            chrome.captions = theme.captions();
            chrome.action_style = theme.style();
            chrome.action_stroke = snapshot
                .effective
                .get(CONTEXT)
                .and_then(|e| e.icon_stroke)
                .map(|stroke| stroke as f32)
                .unwrap_or_else(|| stroke_width(&theme));
            chrome
        })
        .collect();
    Ok(Look {
        theme: Arc::new(theme),
        text_scale,
        window_style,
        gaps: profile.window.gaps,
        chrome,
    })
}

impl Look {
    pub fn chrome(&self, style: ChromeStyle) -> ChromeTheme {
        self.chrome
            .iter()
            .find(|theme| theme.deco.style == style)
            .expect("every chrome style was prepared")
            .clone()
    }

    /// The scene palette: the theme's, with an edge page on the secondary
    /// pair (Quoin's PANEL token); a dialog keeps the base pair.
    pub fn palette(&self, dialog: bool) -> Palette {
        page(&self.theme, dialog)
    }
}

/// [`Look::palette`] for any theme.
pub(crate) fn page(theme: &Theme, dialog: bool) -> Palette {
    let mut palette = Palette::from_dictionary(theme.dictionary());
    if !dialog {
        palette.base = palette.secondary;
    }
    palette
}

fn stroke_width(theme: &Theme) -> f32 {
    theme.overlay().stroke.unwrap_or_else(|| {
        theme
            .dictionary()
            .metrics
            .get("icon.stroke_width")
            .filter(|metric| metric.kind == design::ResolvedMetricKind::Px && metric.value > 0.0)
            .map_or(theme::icons::DEFAULT_STROKE, |metric| metric.value as f32)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn accepted_settings_reach_native_chrome_without_toolkit() {
        let mut desktop = settings::Desktop::default();
        desktop.appearance.mode = "light".into();
        desktop.ui.text_scale = 1.5;
        let values = settings::migrate::migrate(&desktop).unwrap().values;
        let snapshot = Snapshot {
            schema: settings::SCHEMA,
            binding: settings::Binding {
                instance: "test".into(),
                profile: "test".into(),
            },
            incarnation: "test".into(),
            revision: settings::Revision(1),
            design_revision: settings::Revision(1),
            source_digest: settings::source_digest(settings::EMBEDDED_DEFAULT_SOURCE),
            effective: settings::resolve::values::resolve(
                &values,
                settings::EMBEDDED_DEFAULT_SOURCE,
            )
            .unwrap(),
            desktop: settings::migrate::reverse(&values).desktop,
            values,
        };
        let look = build(&snapshot).unwrap();
        assert_eq!(look.theme.mode(), Mode::Light);
        assert_eq!(look.text_scale, 1.5);
        for style in ChromeStyle::ALL {
            let chrome = look.chrome(style);
            assert_eq!(chrome.deco.style, style);
            assert!(chrome.deco.metrics.title_size_px > 0.0);
        }
    }
}
