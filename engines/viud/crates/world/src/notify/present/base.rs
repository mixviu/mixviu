// SPDX-License-Identifier: MIT OR Apache-2.0
use super::registry::Presenter;
use crate::smithay_glue::data::ScreenContext;
use slots::storage::slot::base::Storage;
use smithay::backend::renderer::gles::GlesRenderer;
pub struct NotifyFrame {
    pub elements: Vec<ui_surface::UiRenderElement>,
    pub animating: bool,
}
pub fn tick(
    p: &mut Presenter,
    next: Option<String>,
    kernel: &Storage,
    renderer: &mut GlesRenderer,
    node: &str,
    screen: &ScreenContext,
) -> (Option<NotifyFrame>, bool) {
    match p.owner.as_mut() {
        Some(owner) => owner.tick(next, kernel, renderer, node, screen),
        None => (None, false),
    }
}
