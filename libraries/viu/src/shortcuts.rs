// SPDX-License-Identifier: MIT OR Apache-2.0
//! Exact chord admission shared by data-authored application frames.
pub fn take(ctx: &egui::Context, chord: &egui::KeyboardShortcut) -> bool {
    let clipboard = !ctx.egui_wants_keyboard_input();
    ctx.input_mut(|input| {
        let found = input.events.iter().position(|event| {
            let (key, modifiers) = match event {
                egui::Event::Key {
                    key,
                    pressed: true,
                    modifiers,
                    ..
                } => (*key, *modifiers),
                egui::Event::Copy if clipboard => (egui::Key::C, input.modifiers),
                egui::Event::Cut if clipboard => (egui::Key::X, input.modifiers),
                egui::Event::Paste(_) | egui::Event::PasteImage(_) if clipboard => {
                    (egui::Key::V, input.modifiers)
                }
                _ => return false,
            };
            key == chord.logical_key && modifiers.matches_exact(chord.modifiers)
        });
        if let Some(index) = found {
            input.events.remove(index);
            true
        } else {
            false
        }
    })
}
#[cfg(test)]
mod tests {
    #[test]
    fn shift_delete_never_falls_through_to_delete() {
        let ctx = egui::Context::default();
        ctx.run_ui(
            egui::RawInput {
                events: vec![egui::Event::Key {
                    key: egui::Key::Delete,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers: egui::Modifiers::SHIFT,
                }],
                ..Default::default()
            },
            |ui| {
                assert!(!super::take(
                    ui.ctx(),
                    &egui::KeyboardShortcut::new(egui::Modifiers::NONE, egui::Key::Delete)
                ));
                assert!(super::take(
                    ui.ctx(),
                    &egui::KeyboardShortcut::new(egui::Modifiers::SHIFT, egui::Key::Delete)
                ));
            },
        )
        .drop_without_applying_deltas();
    }
}
