//! Behaviour/snapshot tests snap motion, including after theme reinstallation.
//! Production contexts never install this callback. Motion tests opt in.
use egui_kittest::HarnessBuilder;

pub fn configure<State>(builder: HarnessBuilder<State>) -> HarnessBuilder<State> {
    builder.with_context_setup(reduced_motion)
}

#[allow(dead_code)] // For helpers that also install fonts/theme in context_setup.
pub fn reduced_motion(ctx: &egui::Context) {
    install_policy(ctx, false);
}

#[allow(dead_code)] // Only motion-specific test binaries use this opt-in.
pub fn with_motion<State>(builder: HarnessBuilder<State>) -> HarnessBuilder<State> {
    policy(builder, true)
}

fn policy<State>(builder: HarnessBuilder<State>, animated: bool) -> HarnessBuilder<State> {
    builder.with_context_setup(move |ctx| install_policy(ctx, animated))
}

fn install_policy(ctx: &egui::Context, animated: bool) {
        ctx.on_begin_pass("toolkit test motion preference", std::sync::Arc::new(move |ctx| {
            ctx.global_style_mut(|style| {
                style.animation_time = if animated { 0.2 } else { 0.0 };
                if !animated {
                    style.scroll_animation = egui::style::ScrollAnimation::none();
                }
            });
        }));
}
