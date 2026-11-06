// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! `StopHud` — the top-right Stop button shown while a capture is active. Its
//! own small screen-space instance (positioned by the interface) so it only
//! intercepts clicks where the button is.

use crate::strings;
use ui::UiProgram;
use world::capture::session::message::CaptureMessage;

use crate::capture::style;

pub struct StopHud;

impl ui::UiProgram for StopHud {
    ui::program_owner!(StopHud);
    type Message = CaptureMessage;
    fn capture_message(message: CaptureMessage) -> Option<Self::Message> {
        Some(message)
    }

    fn update(&mut self, _message: Self::Message) {}

    fn setup(&mut self, ctx: &egui::Context) {
        style::install(ctx);
    }

    fn show(&mut self, ui: &mut egui::Ui, emit: &mut Vec<CaptureMessage>) {
        style::keep_up(ui.ctx());
        egui::Frame::NONE.inner_margin(10.0).show(ui, |ui| {
            let danger = style::danger(ui);
            if style::button(ui, strings::text("capture-stop"), Some(danger)).clicked() {
                emit.push(CaptureMessage::Stop);
            }
        });
    }
}
