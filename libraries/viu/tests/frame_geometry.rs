// SPDX-License-Identifier: MIT OR Apache-2.0
#[test]
fn scene_header_has_full_band_and_no_extra_content_gap_on_either_caption_side() {
    for side in ["left", "right"] {
        let ctx = egui::Context::default();
        toolkit::install(&ctx, &toolkit::Theme::embedded());
        let mut frame = viu::window_frame::WindowFrame::default();
        ctx.run_ui(
            egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(
                    egui::Pos2::ZERO,
                    egui::vec2(1040.0, 680.0),
                )),
                ..Default::default()
            },
            |ui| {
                let outer = ui.available_rect_before_wrap();
                frame.show_with_chrome(
                    ui,
                    "Example",
                    &serde_json::json!({"window.decorations":"csd","window.caption_side":side}),
                    true,
                    true,
                );
                let header = frame.geometry["window.header"];
                assert_eq!(header.x_range(), outer.x_range());
                assert_eq!(ui.available_rect_before_wrap().top(), header.bottom());
                frame.finish_captions(ui, side == "left");
                let captions = frame.geometry["window.captions"];
                assert_eq!(frame.geometry["window.caption.close"].top(), header.top());
                if side == "left" {
                    assert_eq!(captions.left(), header.left());
                } else {
                    assert_eq!(captions.right(), header.right());
                }
            },
        )
        .drop_without_applying_deltas();
    }
}
