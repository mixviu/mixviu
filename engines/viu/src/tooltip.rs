// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Client card renderer and placement, adapted from scene-host/tooltip.rs.
use crate::view::tip::{self, Tip, TipAt};
use std::sync::Arc;
/// Transparent margin around the card for its shadow (logical px).
pub const MARGIN: f32 = 12.0;
/// Space between the scene's edge and the card.
pub const GAP: f32 = 6.0;
/// The card never comes closer than this to the output's edges.
pub const EDGE: f32 = 4.0;
/// Card corner radius.
const RADIUS: f32 = 6.0;

/// Where a card goes, in output logical px: centred on its anchor along the
/// scene's long side, just outside the scene on the side facing the middle
/// of the output, clamped to the output. `scene` and the result are
/// `(x, y, w, h)`; the result is the CARD (the surface adds [`MARGIN`]).
pub fn place(scene: (f32, f32, f32, f32), at: &TipAt, output: (f32, f32)) -> (f32, f32, f32, f32) {
    let (sx, sy, sw, sh) = scene;
    let (ax, ay, aw, ah) = at.anchor;
    let (ow, oh) = output;
    let (cw, ch) = (
        at.size.0.min((ow - 2.0 * EDGE).max(0.0)),
        at.size.1.min((oh - 2.0 * EDGE).max(0.0)),
    );
    let (cx, cy) = (sx + ax + aw / 2.0, sy + ay + ah / 2.0);
    let clamp = |v: f32, size: f32, extent: f32| v.min(extent - size - EDGE).max(EDGE);
    if sw >= sh {
        // A horizontal scene (a top or bottom panel): above or below it.
        let y = if cy > oh / 2.0 {
            sy - GAP - ch
        } else {
            sy + sh + GAP
        };
        (clamp(cx - cw / 2.0, cw, ow), clamp(y, ch, oh), cw, ch)
    } else {
        // A vertical scene (a side panel): beside it.
        let x = if cx < ow / 2.0 {
            sx + sw + GAP
        } else {
            sx - GAP - cw
        };
        (clamp(x, cw, ow), clamp(cy - ch / 2.0, ch, oh), cw, ch)
    }
}

/// The text size a card's title and lines derive from: the look's UI size,
/// as the scene measured it.
fn ui_size(look: Option<&crate::appearance::Look>, theme: &toolkit::theme::Theme) -> f32 {
    let scale = look.map_or(1.0, |look| look.text_scale);
    crate::appearance::ui_size(theme, scale)
}

#[derive(Clone, Debug)]
pub enum Message {
    Replace(Tip),
    Appearance(Arc<crate::appearance::Look>),
}

pub(crate) struct TooltipUi {
    pub(crate) tip: Tip,
    pub(crate) look: Option<Arc<crate::appearance::Look>>,
    pub(crate) installed: Option<usize>,
}

impl TooltipUi {
    fn identity(&self) -> usize {
        self.look
            .as_ref()
            .map_or(0, |look| Arc::as_ptr(look) as usize)
    }

    /// Install the look's fonts and style (an edge page's, so the card
    /// measures exactly as the panel measured it).
    fn install(&mut self, ctx: &egui::Context) {
        let scale = self.look.as_ref().map_or(1.0, |look| look.text_scale);
        crate::appearance::install(ctx, &self.theme(), scale, false);
        self.installed = Some(self.identity());
    }

    fn theme(&self) -> Arc<toolkit::theme::Theme> {
        match &self.look {
            Some(look) => look.theme.clone(),
            // The panel's own embedded look: an edge page in dark mode.
            None => Arc::new(toolkit::theme::Theme::for_context(design::DesignContext {
                mode: design::Mode::Dark,
                ..design::DesignContext::default()
            })),
        }
    }
}

#[cfg(test)]
impl ui::UiProgram for TooltipUi {
    ui::program_owner!(TooltipUi);
    type Message = Message;
    fn setup(&mut self, ctx: &egui::Context) {
        TooltipUi::setup(self, ctx);
    }
    fn show(&mut self, ui: &mut egui::Ui, emit: &mut Vec<Message>) {
        TooltipUi::show(self, ui, emit);
    }
    fn update(&mut self, message: Message) {
        TooltipUi::update(self, message);
    }
}

impl TooltipUi {
    pub(crate) fn setup(&mut self, ctx: &egui::Context) {
        // Before the first pass: egui binds new fonts from the NEXT pass on,
        // so a first pass that drew the bold (heading) family before this
        // install would panic ("not bound to any fonts").
        self.install(ctx);
    }

    pub(crate) fn show(&mut self, ui: &mut egui::Ui, _emit: &mut Vec<Message>) {
        let ctx = ui.ctx().clone();
        let theme = self.theme();
        if self.installed != Some(self.identity()) {
            // A new look: its fonts bind from the next pass. Draw nothing
            // this pass rather than measure with fonts not yet bound.
            self.install(&ctx);
            ctx.request_repaint();
            return;
        }
        let palette = crate::appearance::page(&theme, false);
        let surface = crate::view::colour(palette.base.surface);
        let ink = crate::view::colour(palette.base.foreground);
        let border = crate::view::colour(palette.border);
        let full = ui.max_rect();
        let card = full.shrink(MARGIN);
        let painter = ui.painter();
        painter.add(
            egui::epaint::Shadow {
                offset: [0, 2],
                blur: 10,
                spread: 0,
                color: egui::Color32::from_black_alpha(110),
            }
            .as_shape(card, RADIUS),
        );
        painter.rect_filled(card, RADIUS, surface);
        painter.rect_stroke(
            card,
            RADIUS,
            egui::Stroke::new(1.0, border),
            egui::StrokeKind::Inside,
        );
        let measure = |text: &str, font: &egui::FontId| {
            let size = ctx.fonts_mut(|fonts| {
                fonts
                    .layout_no_wrap(text.to_owned(), font.clone(), egui::Color32::WHITE)
                    .size()
            });
            (size.x, size.y)
        };
        let laid = tip::card(&self.tip, ui_size(self.look.as_deref(), &theme), &measure);
        // The thumbnail's box: a quiet well the live window is drawn over (by
        // the scene, `world::window::draw::thumbnail`); a window that cannot
        // be shown leaves just the well.
        if let Some((tx, ty, tw, th)) = laid.thumb {
            let well = egui::Rect::from_min_size(
                egui::pos2(card.left() + tx, card.top() + ty),
                egui::vec2(tw, th),
            );
            // The theme's secondary surface, rounded like the card's inside.
            painter.rect_filled(
                well,
                RADIUS / 2.0,
                crate::view::colour(palette.secondary.surface),
            );
        }
        for line in &laid.lines {
            painter.text(
                egui::pos2(
                    card.left() + tip::PAD_X,
                    card.top() + line.y + line.height / 2.0,
                ),
                egui::Align2::LEFT_CENTER,
                &line.text,
                crate::view::tip_font(line.size, line.bold),
                tip::line_colour(ink, line.dim),
            );
        }
    }

    pub(crate) fn update(&mut self, message: Message) {
        match message {
            Message::Replace(tip) => self.tip = tip,
            Message::Appearance(look) => {
                self.look = Some(look);
                self.installed = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interface_font_size_matches_the_scene_measurement_for_cards() {
        let mut desktop = settings::Desktop::default();
        desktop.appearance.font_size = Some(18.0);
        desktop.ui.text_scale = 1.5;
        let look = crate::appearance::build(&crate::appearance::snapshot_of(desktop)).unwrap();
        assert!((ui_size(Some(&look), &look.theme) - 27.0).abs() < 0.001);
        let plain = crate::appearance::fixture("dark", 1.0);
        let design =
            design::active_typography(Some(plain.theme.typography()), design::TypographyRole::Ui);
        assert_eq!(ui_size(None, &plain.theme), design.font_size as f32);
    }

    fn at(anchor: (f32, f32, f32, f32), size: (f32, f32)) -> TipAt {
        TipAt {
            tip: Tip {
                title: "Mail".into(),
                ..Tip::default()
            },
            anchor,
            size,
            thumb: None,
        }
    }

    /// The card's first passes draw a bold (heading-family) title. egui binds
    /// fonts from the pass AFTER an install, so the card must install before
    /// its first pass; drawing first panicked compd on VT4 (2026-10-12).
    #[test]
    fn the_card_draws_its_first_passes_and_a_new_look_without_panicking() {
        let tip = Tip {
            title: "Workspace 2".into(),
            subtitle: "Mail · 2 windows".into(),
            lines: vec!["Inbox".into(), "Drafts".into()],
            thumbnail: Some(crate::view::tip::Thumbnail {
                id: 3,
                generation: 1,
                w: 1600.0,
                h: 900.0,
            }),
        };
        for look in [None, Some(crate::appearance::fixture("dark", 1.25))] {
            let mut rt = ui::UiRuntime::headless(
                TooltipUi {
                    tip: tip.clone(),
                    look: look.clone(),
                    installed: None,
                },
                (240, 120),
                1.0,
            );
            rt.tick();
            rt.tick();
            // A look change mid-life: reinstall, skip one pass, then draw.
            rt.queue_message(Message::Appearance(crate::appearance::fixture(
                "light", 1.0,
            )));
            rt.tick();
            rt.tick();
            rt.queue_message(Message::Replace(Tip {
                title: "Thunderbird".into(),
                ..Tip::default()
            }));
            rt.tick();
        }
    }

    #[test]
    fn a_bottom_panel_tip_sits_above_it_centred_on_its_anchor() {
        // A 1536x864 output, a 52-px bottom panel; a button at x 100.
        let (x, y, w, h) = place(
            (0.0, 812.0, 1536.0, 52.0),
            &at((100.0, 6.0, 40.0, 40.0), (120.0, 40.0)),
            (1536.0, 864.0),
        );
        assert_eq!((w, h), (120.0, 40.0));
        assert_eq!(x, 120.0 - 60.0);
        assert_eq!(y, 812.0 - GAP - 40.0);
    }

    #[test]
    fn a_top_panel_tip_sits_below_it() {
        let (_, y, _, _) = place(
            (0.0, 0.0, 1536.0, 40.0),
            &at((300.0, 0.0, 40.0, 40.0), (120.0, 40.0)),
            (1536.0, 864.0),
        );
        assert_eq!(y, 40.0 + GAP);
    }

    #[test]
    fn action_tips_follow_the_icon_and_stay_inside_the_output() {
        for scene in [
            (0.0, 0.0, 28.0, 28.0),
            (1520.0, 0.0, 28.0, 28.0),
            (1520.0, 840.0, 28.0, 28.0),
        ] {
            let (x, y, w, h) = place(
                scene,
                &at((0.0, 0.0, 28.0, 28.0), (120.0, 40.0)),
                (1536.0, 864.0),
            );
            assert!(x >= EDGE && y >= EDGE && x + w <= 1536.0 - EDGE && y + h <= 864.0 - EDGE);
            if scene.1 == 0.0 {
                assert_eq!(y, 28.0 + GAP);
            } else {
                assert!(y + h < scene.1);
            }
        }
        let (x, y, w, h) = place(
            (90.0, 0.0, 28.0, 28.0),
            &at((0.0, 0.0, 28.0, 28.0), (400.0, 200.0)),
            (100.0, 60.0),
        );
        assert!(x >= EDGE && y >= EDGE && x + w <= 100.0 - EDGE && y + h <= 60.0 - EDGE);
    }

    #[test]
    fn side_panel_tips_go_beside_the_panel() {
        let left = place(
            (0.0, 0.0, 52.0, 864.0),
            &at((6.0, 400.0, 40.0, 40.0), (120.0, 40.0)),
            (1536.0, 864.0),
        );
        assert_eq!(left.0, 52.0 + GAP);
        assert_eq!(left.1, 420.0 - 20.0);
        let right = place(
            (1484.0, 0.0, 52.0, 864.0),
            &at((6.0, 400.0, 40.0, 40.0), (120.0, 40.0)),
            (1536.0, 864.0),
        );
        assert_eq!(right.0, 1484.0 - GAP - 120.0);
    }

    #[test]
    fn cards_clamp_inside_the_output() {
        // A button at the far left and one at the far right.
        let (x, _, _, _) = place(
            (0.0, 812.0, 1536.0, 52.0),
            &at((0.0, 6.0, 40.0, 40.0), (300.0, 40.0)),
            (1536.0, 864.0),
        );
        assert_eq!(x, EDGE);
        let (x, _, _, _) = place(
            (0.0, 812.0, 1536.0, 52.0),
            &at((1500.0, 6.0, 36.0, 40.0), (300.0, 40.0)),
            (1536.0, 864.0),
        );
        assert_eq!(x, 1536.0 - 300.0 - EDGE);
    }
}
