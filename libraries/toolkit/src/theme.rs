// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Toolkit adapter over egui-free theme values.
use design::{CaptionSide, DesignContext, RequestedDecorations};
use std::path::Path;
pub use theme::theme::{Choice, Error, Overlay, THEME_FILE, theme_path};
#[derive(Clone, Debug, PartialEq)]
pub struct Theme(theme::Theme);
impl From<theme::Theme> for Theme {
    fn from(theme: theme::Theme) -> Self {
        Self(theme)
    }
}
impl std::ops::Deref for Theme {
    type Target = theme::Theme;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Theme {
    pub fn embedded() -> Self {
        Self(theme::Theme::embedded())
    }
    pub fn for_context(context: DesignContext) -> Self {
        Self(theme::Theme::for_context(context))
    }
    pub fn from_source(identity: &str, source: &str) -> Result<Self, Error> {
        theme::Theme::from_source(identity, source).map(Self)
    }
    pub fn from_source_in(
        identity: &str,
        source: &str,
        context: DesignContext,
    ) -> Result<Self, Error> {
        theme::Theme::from_source_in(identity, source, context).map(Self)
    }
    pub fn read(path: &Path) -> Result<Option<Self>, Error> {
        theme::Theme::read(path).map(|t| t.map(Self))
    }
    pub fn load() -> Self {
        Self(theme::Theme::load())
    }
    pub fn with_choice(&self, choice: &Choice) -> Result<Self, Error> {
        self.0.with_choice(choice).map(Self)
    }
    pub fn framed(
        self,
        decorations: impl Into<RequestedDecorations>,
        captions: CaptionSide,
    ) -> Self {
        Self(self.0.framed(decorations, captions))
    }
    pub fn with_menus(self, style: design::MenuStyle, side: design::MenuSide) -> Self {
        Self(self.0.with_menus(style, side))
    }
    pub fn with_band(self, band: design::title_band::Settings) -> Self {
        Self(self.0.with_band(band))
    }
    pub fn with_overlay(self, overlay: Overlay) -> Result<Self, Error> {
        self.0.with_overlay(overlay).map(Self)
    }
    pub fn with_ui(self, ui: settings::model::CommonUi) -> Self {
        Self(self.0.with_ui(ui))
    }
}
impl editor::palette::Theme for Theme {
    fn editor_colour(&self, role: &str) -> egui::Color32 {
        let chrome = crate::chrome::Chrome::for_theme(self);
        if role == "selection" {
            return chrome.palette.selection(&chrome.style);
        }
        if let Some(colour) = chrome.palette.get(role) {
            return colour;
        }
        self.dictionary()
            .colours
            .primitives
            .get(role)
            .map(|value| crate::style::colour(*value))
            .unwrap_or(if role == "palette.accent.hover" {
                chrome.palette.accent
            } else {
                chrome.palette.text
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use design::{DesignContext, DesignSourceDocument, SourceIdentity};
    fn embedded_document() -> DesignSourceDocument {
        design::parse_design_source(
            SourceIdentity::new("embedded"),
            design::EMBEDDED_DEFAULT_SOURCE,
        )
        .unwrap()
    }
    impl Theme {
        fn compile(doc: &DesignSourceDocument, ctx: DesignContext) -> Result<Self, Error> {
            theme::Theme::compile(doc, ctx).map(Self)
        }
    }
    /// A custom design with no style family keeps deriving the hue
    /// schemes' title bar and menus from its own spacing and radius: the
    /// embedded style's fixed lengths would shrink its hit targets.
    #[test]
    fn a_custom_design_without_a_style_family_derives_its_chrome_geometry() {
        let mut document = embedded_document();
        document.v1.families.style = None;
        document.v1.primitives.scales.get_mut("spacing").unwrap()[9] =
            design::MetricSource::px(48.0);
        document
            .v1
            .primitives
            .metrics
            .insert("radius".into(), design::MetricSource::px(10.0));
        let theme = Theme::compile(&document, DesignContext::revision_one()).unwrap();
        let m = crate::chrome::Chrome::for_theme(&theme).metrics;
        // One 48 pt control, and a 3 pt item gap above and below it.
        assert_eq!(
            (m.menu_row_height, m.menu_title_height, m.title_bar_height),
            (48.0, 48.0, 54.0)
        );
        assert_eq!((m.menu_row_padding.x, m.menu_min_width), (32.0, 384.0));
        assert_eq!(
            (m.radius_sm, m.radius, m.radius_lg, m.menu_highlight_radius),
            (10, 7, 20, 10)
        );
        // Explicit style tokens stay authoritative.
        let mut styled = embedded_document();
        styled.v1.primitives.scales.get_mut("spacing").unwrap()[9] = design::MetricSource::px(48.0);
        let theme = Theme::compile(&styled, DesignContext::revision_one()).unwrap();
        assert_eq!(
            crate::chrome::Chrome::for_theme(&theme)
                .metrics
                .title_bar_height,
            30.0
        );
    }
}
