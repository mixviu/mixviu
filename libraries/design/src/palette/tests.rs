// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Palette goldens: `tests/fixtures/palette/<scheme>.json`, both modes. On
//! a mismatch the test prints the new JSON between `GOLDEN <file>` and
//! `END GOLDEN`; `DESIGN_UPDATE_GOLDEN=1` writes it instead. Read the
//! diff before committing one.

use super::*;
use crate::geometry::{Density, Roundness};
use crate::{AccentChoice, Contrast, EMBEDDED_DEFAULT_SOURCE, Scheme, Swatch, parse_design_source};

fn document() -> DesignSourceDocument {
    parse_design_source(
        crate::SourceIdentity::new("embedded"),
        EMBEDDED_DEFAULT_SOURCE,
    )
    .unwrap()
}

fn palette_of(context: DesignContext) -> Palette {
    let document = document();
    palette(
        &document,
        &context,
        &Geometry::default(),
        &FrostedGlass::default(),
    )
    .unwrap()
}

fn golden(name: &str, palette: &Palette) {
    let path = format!(
        "{}/tests/fixtures/palette/{name}.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let json = serde_json::to_string_pretty(palette).unwrap() + "\n";
    if std::env::var_os("DESIGN_UPDATE_GOLDEN").is_some() {
        std::fs::create_dir_all(std::path::Path::new(&path).parent().unwrap()).unwrap();
        std::fs::write(&path, &json).unwrap();
        return;
    }
    let Ok(stored) = std::fs::read_to_string(&path) else {
        println!("GOLDEN {name}.json\n{json}END GOLDEN");
        panic!("palette golden {path} is missing; write it with DESIGN_UPDATE_GOLDEN=1");
    };
    if stored != json {
        println!("GOLDEN {name}.json\n{json}END GOLDEN");
        panic!("palette golden {name}.json differs");
    }
}

#[test]
fn derived_palette_golden() {
    golden(
        "mixos",
        &palette_of(DesignContext {
            scheme: Scheme::Mixos,
            ..DesignContext::default()
        }),
    );
}

#[test]
fn authored_palette_golden() {
    golden(
        "studio",
        &palette_of(DesignContext {
            scheme: Scheme::Studio,
            ..DesignContext::default()
        }),
    );
}

#[test]
fn derived_palette_is_the_derivation_itself() {
    let palette = palette_of(DesignContext {
        scheme: Scheme::Mixos,
        ..DesignContext::default()
    });
    for mode in [&palette.dark, &palette.light] {
        assert_eq!(mode.parity, "exact");
        // Each role is the role map read off the reported theme.
        for (role, source) in crate::seeds::ROLE_MAP {
            assert_eq!(
                mode.roles[role].to_srgba8(),
                source.pick(&mode.cosmic.theme).to_srgba8(),
                "{} {role}",
                mode.mode
            );
        }
        assert_eq!(mode.accent, mode.cosmic.theme.accent.base);
        assert_eq!(mode.swatches.len(), 9);
        assert!(mode.flat.values().all(|colour| colour.alpha >= 1.0 - 1e-9));
    }
    assert_eq!(palette.dark.accent.to_hex(), "#63D0DFFF");
    assert_eq!(palette.light.accent.to_hex(), "#00525AFF");
}

#[test]
fn authored_palette_is_closest_and_reports_the_axis_accent() {
    let palette = palette_of(DesignContext {
        scheme: Scheme::Studio,
        accent: AccentChoice::Swatch(Swatch::Green),
        ..DesignContext::default()
    });
    assert_eq!(palette.dark.parity, "closest");
    assert_eq!(palette.dark.accent.to_hex(), "#92CF9CFF");
    assert_eq!(palette.light.accent.to_hex(), "#185529FF");
    assert_eq!(palette.dark.swatches["green"].to_hex(), "#92CF9CFF");
    // The COSMIC builder takes the authored roles as seeds.
    let chrome = palette.dark.roles["chrome"];
    assert_eq!(palette.dark.cosmic.builder.bg_color, Some(chrome));
    assert_eq!(palette.dark.cosmic.theme.background.base, chrome);
}

#[test]
fn theme_palette_carries_all_32_colours_by_name() {
    let palette = palette_of(DesignContext {
        scheme: Scheme::Mixos,
        ..DesignContext::default()
    });
    let json = serde_json::to_value(&palette).unwrap();
    for mode in ["dark", "light"] {
        let colours = json[mode]["cosmic"]["theme"]["palette"]
            .as_object()
            .unwrap();
        assert_eq!(colours.len(), 33, "{mode}: kind plus 32 colours");
        assert!(!colours.contains_key("neutral"));
        for k in 0..=10 {
            assert!(colours[&format!("neutral_{k}")].is_string(), "{mode} {k}");
        }
    }
    let dark = &json["dark"]["cosmic"]["theme"]["palette"];
    assert_eq!(dark["neutral_0"], "#000000FF");
    assert_eq!(dark["neutral_5"], "#636363FF");
    assert_eq!(
        json["light"]["cosmic"]["theme"]["palette"]["neutral_0"],
        "#FFFFFFFF"
    );
}

#[test]
fn flat_composites_translucent_roles_over_their_surface() {
    let palette = palette_of(DesignContext {
        scheme: Scheme::Mixos,
        ..DesignContext::default()
    });
    let dark = &palette.dark;
    let soft = dark.roles["accent_soft"];
    assert!(soft.alpha < 1.0);
    assert_eq!(dark.flat["accent_soft"], over(soft, dark.roles["card"]));
    assert_eq!(dark.flat["chrome"], dark.roles["chrome"]);
    let faint = dark.roles["text_faint"];
    assert_eq!(dark.flat["text_faint"], over(faint, dark.roles["chrome"]));
}

#[test]
fn geometry_reaches_the_cosmic_values() {
    let document = document();
    let square = mode_palette(
        &document,
        &DesignContext {
            scheme: Scheme::Mixos,
            contrast: Contrast::High,
            ..DesignContext::default()
        },
        &Geometry {
            roundness: Roundness::Square,
            density: Density::Compact,
        },
        &FrostedGlass::default(),
    )
    .unwrap();
    assert_eq!(square.cosmic.corner_radii.radius_xl, [0.0; 4]);
    assert_eq!(square.cosmic.spacing.space_m, 16);
    assert_eq!(square.contrast, "high");
    assert!(square.cosmic.theme.is_high_contrast);
}

#[test]
fn high_contrast_strengthens_every_scheme_in_both_modes() {
    let ratio = |a: SrgbColour, b: SrgbColour| crate::contrast_ratio(a.to_linear(), b.to_linear());
    for scheme in Scheme::ALL {
        let normal = palette_of(DesignContext { scheme, ..Default::default() });
        let high = palette_of(DesignContext { scheme, contrast: Contrast::High, ..Default::default() });
        for (normal, high) in [(&normal.dark, &high.dark), (&normal.light, &high.light)] {
            for surface in ["chrome", "canvas", "dock", "card", "field", "hover", "pressed", "accent_soft"] {
                for text in ["text", "text_dim", "text_faint", "icon", "accent_text"] {
                    let got = ratio(high.flat[text], high.flat[surface]);
                    assert!(got >= 7.0, "{scheme:?}/{} {text}/{surface}: {got}", high.mode);
                    assert!(got >= ratio(normal.flat[text], normal.flat[surface]), "high must not reduce {text}/{surface}");
                }
            }
            for (text, bg) in [("primary_text", "primary_bg"), ("menu_highlight_text", "menu_highlight"), ("caption_close_text", "caption_close")] {
                assert!(ratio(high.flat[text], high.flat[bg]) >= 7.0);
            }
            for (edge, bg) in [("field_border", "field"), ("card_border", "card"), ("separator", "chrome"), ("accent_border", "card")] {
                let got = ratio(high.flat[edge], high.flat[bg]);
                assert!(got >= 3.0, "{scheme:?}/{} {edge}: {got}", high.mode);
                assert!(got > ratio(normal.flat[edge], normal.flat[bg]), "stronger {scheme:?} {edge}");
            }
            for (fg, bg) in [("window_fg_color", "window_bg_color"), ("view_fg_color", "view_bg_color"), ("card_fg_color", "card_bg_color")] {
                assert!(ratio(high.toolkit.gtk[fg], high.toolkit.gtk[bg]) >= 7.0);
            }
            for group in [high.toolkit.qt.active, high.toolkit.qt.inactive, high.toolkit.qt.disabled] {
                for (fg, bg) in [(0,10), (6,9), (8,1), (13,12), (19,18), (20,9)] {
                    assert!(ratio(group[fg], group[bg]) >= 7.0);
                }
            }
        }
    }
}
