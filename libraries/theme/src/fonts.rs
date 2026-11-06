// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
use crate::Theme;
use design::TypographyRole;
/// The body weight of the chrome schemes: Inter Regular.
pub const CHROME_WEIGHT: u16 = 400;

/// Static Inter weights available to interface controls and title renderers.
pub const INTER_WEIGHTS: &[u16] = &[300, 400, 500, 600, 700];

const INTER_LIGHT: &[u8] = include_bytes!("../../toolkit/assets/fonts/Inter-Light.ttf");
const INTER_REGULAR: &[u8] = include_bytes!("../../toolkit/assets/fonts/Inter-Regular.ttf");
const INTER_MEDIUM: &[u8] = include_bytes!("../../toolkit/assets/fonts/Inter-Medium.ttf");
const INTER_SEMIBOLD: &[u8] = include_bytes!("../../toolkit/assets/fonts/Inter-SemiBold.ttf");
const INTER_BOLD: &[u8] = include_bytes!("../../toolkit/assets/fonts/Inter-Bold.ttf");

/// The embedded static Inter face nearest `weight` (CSS numbers): 300, 400,
/// 500, 600 or 700, with lighter requests taking Light and heavier Bold.
pub fn inter_face(weight: u16) -> (&'static str, &'static [u8]) {
    match weight {
        0..=349 => ("Inter-Light", INTER_LIGHT),
        350..=449 => ("Inter-Regular", INTER_REGULAR),
        450..=549 => ("Inter-Medium", INTER_MEDIUM),
        550..=649 => ("Inter-SemiBold", INTER_SEMIBOLD),
        _ => ("Inter-Bold", INTER_BOLD),
    }
}

/// The embedded monospace face (JetBrains Mono Regular), for consumers that
/// shape text themselves, such as the compositor's window titles.
pub fn mono_face() -> (&'static str, &'static [u8]) {
    assets::mixos::mono_face(400)
}

/// Body and display weights, with the session override preserving the
/// design's display offset and clamping it to the bundled range.
pub fn weights(theme: &Theme) -> (u16, u16) {
    let typography = theme.typography();
    let mut ui = design::active_typography(Some(typography), TypographyRole::Ui).weight;
    let mut display = design::active_typography(Some(typography), TypographyRole::UiDisplay).weight;
    // The chrome specification fixes its faces (§2.1): Inter Regular, with
    // Medium (window title, push buttons) and SemiBold (headings).
    if theme.style().faces == design::family::style::Faces::Fixed {
        (ui, display) = (CHROME_WEIGHT, CHROME_WEIGHT + 200);
    }
    // The session's weight replaces the body's, even where the chrome fixes
    // its faces; headings keep the design's offset above the body.
    if let Some(weight) = theme.overlay().weight {
        let offset = i32::from(display) - i32::from(ui);
        ui = weight;
        display = (i32::from(weight) + offset).clamp(300, 700) as u16;
    }
    (ui, display)
}
