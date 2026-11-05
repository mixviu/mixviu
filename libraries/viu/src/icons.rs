// SPDX-License-Identifier: MIT OR Apache-2.0
//! Shared Lucide rendering. Asset identity is renderer independent; size is in
//! logical points, colours follow egui visuals unless explicitly supplied.
use egui::{Color32, Image, ImageSource, Response, Vec2};
pub use icons::Icon;
use std::borrow::Cow;

pub fn install(ctx: &egui::Context) {
    egui_extras::install_image_loaders(ctx);
}

pub fn source(icon: Icon) -> String {
    icon.svg().replace("currentColor", "#ffffff")
}

/// Cache key depends only on asset identity. Size/DPI are handled by the SVG
/// loader and tint is applied by the GPU; no per-colour texture cache growth.
pub fn image(icon: Icon, size: f32, colour: Color32) -> Image<'static> {
    Image::new(ImageSource::Bytes {
        uri: Cow::Owned(format!("bytes://viu-lucide/{}.svg", icon.name())),
        bytes: source(icon).into_bytes().into(),
    })
    .fit_to_exact_size(Vec2::splat(size.clamp(8.0, 64.0)))
    .tint(colour)
}

pub fn button(
    ui: &mut egui::Ui,
    icon: Icon,
    text: &str,
    size: f32,
    colour: Option<Color32>,
    enabled: bool,
    selected: bool,
) -> Response {
    ui.add_enabled_ui(enabled, |ui| {
        let tint = if !enabled {
            ui.visuals().weak_text_color()
        } else {
            colour.unwrap_or_else(|| ui.visuals().text_color())
        };
        ui.add(egui::Button::image_and_text(image(icon, size, tint), text).selected(selected))
    })
    .inner
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_imported_svg_rasterises_at_two_scales() {
        for icon in Icon::ALL {
            for scale in [1.0, 2.5] {
                let image = egui_extras::image::load_svg_bytes_with_size(
                    source(*icon).as_bytes(),
                    egui::SizeHint::Width((24.0 * scale) as u32),
                    &Default::default(),
                )
                .unwrap_or_else(|e| panic!("{}: {e}", icon.name()));
                assert!(image.pixels.iter().any(|p| p.a() > 0), "{}", icon.name());
            }
        }
    }
}
