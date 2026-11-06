// SPDX-License-Identifier: MIT OR Apache-2.0
pub(crate) use theme::ui_scale::tokens;
pub use theme::ui_scale::{CommonUi, DENSITY_STEPS, density_step};

/// The installed desktop values, also used by independently sized content.
pub fn of(ctx: &egui::Context) -> CommonUi {
    ctx.data(|data| data.get_temp(egui::Id::unique("toolkit.common-ui")))
        .unwrap_or_default()
}

pub(crate) fn install(ctx: &egui::Context, ui: &CommonUi) {
    ctx.data_mut(|data| data.insert_temp(egui::Id::unique("toolkit.common-ui"), ui.clone()));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `design::geometry` takes over the density half of [`tokens`] (key
    /// space v2, S8a); until then both must agree exactly at every named
    /// step, and `comfortable` must change nothing.
    #[test]
    fn design_geometry_reproduces_the_density_steps_exactly() {
        for scheme in design::Scheme::ALL {
            for style in design::Style::ALL {
                let theme = crate::Theme::for_context(design::DesignContext {
                    scheme,
                    style: Some(style),
                    ..Default::default()
                });
                let raw = theme.raw_style();
                for (multiplier, step) in DENSITY_STEPS
                    .into_iter()
                    .zip(design::geometry::Density::ALL)
                {
                    let ui = CommonUi {
                        density: multiplier,
                        ..CommonUi::default()
                    };
                    assert_eq!(
                        tokens(raw, &ui),
                        design::geometry::legacy_lengths(raw, step),
                        "{scheme:?} {style:?} {multiplier}"
                    );
                }
                assert_eq!(
                    design::geometry::legacy_lengths(raw, design::geometry::Density::Standard),
                    tokens(raw, &CommonUi::default())
                );
            }
        }
    }
}
