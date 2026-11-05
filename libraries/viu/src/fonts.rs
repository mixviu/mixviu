// SPDX-License-Identifier: MIT OR Apache-2.0
//! Theme-selected bundled faces. The renderer chooses the nearest retained
//! static face; no font discovery, settings daemon or application policy.
use egui::{FontData, FontDefinitions, FontFamily, FontId};
fn face(weight: u16) -> (&'static str, &'static [u8]) {
    match weight {
        0..=349 => (
            "Inter-Light",
            include_bytes!("../assets/fonts/Inter-Light.ttf"),
        ),
        350..=449 => (
            "Inter-Regular",
            include_bytes!("../assets/fonts/Inter-Regular.ttf"),
        ),
        450..=549 => (
            "Inter-Medium",
            include_bytes!("../assets/fonts/Inter-Medium.ttf"),
        ),
        550..=649 => (
            "Inter-SemiBold",
            include_bytes!("../assets/fonts/Inter-SemiBold.ttf"),
        ),
        _ => (
            "Inter-Bold",
            include_bytes!("../assets/fonts/Inter-Bold.ttf"),
        ),
    }
}
pub fn install(ctx: &egui::Context, body: u16, heading: u16) {
    let key = egui::Id::unique("viu-font-weights");
    if ctx.data(|d| d.get_temp::<(u16, u16)>(key)) == Some((body, heading)) {
        return;
    }
    let mut fonts = FontDefinitions::default();
    let mut add = |(name, bytes): (&'static str, &'static [u8])| {
        fonts
            .font_data
            .insert(name.into(), FontData::from_static(bytes).into());
        name.to_owned()
    };
    let body_face = add(face(body));
    let medium = add(face(body.saturating_add(100)));
    let heading_face = add(face(heading));
    let mono = add((
        "JetBrains-Mono",
        include_bytes!("../assets/fonts/JetBrainsMono-Regular.ttf"),
    ));
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, body_face);
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .push(mono.clone());
    fonts
        .families
        .entry(FontFamily::Monospace)
        .or_default()
        .insert(0, mono);
    for (family, first) in [("viu-medium", medium), ("viu-heading", heading_face)] {
        let mut stack = fonts.families[&FontFamily::Proportional].clone();
        stack.insert(0, first);
        fonts
            .families
            .insert(FontFamily::Name(family.into()), stack);
    }
    ctx.set_fonts(fonts);
    ctx.data_mut(|d| d.insert_temp::<(u16, u16)>(key, (body, heading)));
}
pub fn emphasis(ctx: &egui::Context, font: FontId) -> FontId {
    if ctx
        .data(|d| d.get_temp::<(u16, u16)>(egui::Id::unique("viu-font-weights")))
        .is_some()
    {
        FontId::new(font.size, FontFamily::Name("viu-medium".into()))
    } else {
        font
    }
}
