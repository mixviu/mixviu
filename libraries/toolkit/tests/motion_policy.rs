#[path = "support/harness.rs"]
mod test_harness;

#[test]
fn behaviour_harness_stays_reduced_after_theme_reinstallation() {
    let mut h = test_harness::configure(egui_kittest::Harness::builder()).build_ui(|ui| {
        assert_eq!(ui.style().animation_time, 0.);
        ui.label("Settled");
    });
    for _ in 0..2 {
        toolkit::install(&h.ctx, &toolkit::Theme::embedded());
        h.run();
        assert_eq!(h.ctx.global_style().animation_time, 0.);
    }
}

#[test]
fn production_defaults_and_explicit_motion_opt_in_still_animate() {
    let ctx = egui::Context::default();
    toolkit::install(&ctx, &toolkit::Theme::embedded());
    assert!(ctx.global_style().animation_time > 0.);
    let mut h = test_harness::with_motion(egui_kittest::Harness::builder()).build_ui(|ui| {
        assert!(ui.style().animation_time > 0.);
        ui.label("Motion enabled");
    });
    h.run();
    let overlay = toolkit::theme::Overlay {
        reduced_motion: true,
        ..Default::default()
    };
    toolkit::install(
        &ctx,
        &toolkit::Theme::embedded().with_overlay(overlay).unwrap(),
    );
    assert_eq!(ctx.global_style().animation_time, 0.);
}
