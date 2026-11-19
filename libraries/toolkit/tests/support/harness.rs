//! Behaviour/snapshot tests snap motion, including after theme reinstallation.
//! Production contexts never install this plugin. Motion tests opt in.
use egui_kittest::HarnessBuilder;

pub fn configure<State>(builder: HarnessBuilder<State>) -> HarnessBuilder<State> {
    builder.with_context_setup(reduced_motion)
}

#[allow(dead_code)] // Helpers may also install fonts/theme in context_setup.
pub fn reduced_motion(ctx: &egui::Context) {
    ctx.add_plugin(MotionPreference { animated: false });
}

#[allow(dead_code)] // Only motion-specific test binaries use this opt-in.
pub fn with_motion<State>(builder: HarnessBuilder<State>) -> HarnessBuilder<State> {
    builder.with_context_setup(|ctx| ctx.add_plugin(MotionPreference { animated: true }))
}

struct MotionPreference {
    animated: bool,
}

impl egui::plugin::Plugin for MotionPreference {
    fn debug_name(&self) -> &'static str {
        "toolkit test motion preference"
    }

    fn input_hook(&mut self, ctx: &egui::Context, _: &mut egui::RawInput) {
        // This hook precedes creation of the root Ui. A begin-pass callback
        // would update the context after the root had already copied its style.
        ctx.all_styles_mut(|style| {
            style.animation_time = if self.animated { 0.2 } else { 0.0 };
            if !self.animated {
                style.scroll_animation = egui::style::ScrollAnimation::none();
            }
        });
    }
}
