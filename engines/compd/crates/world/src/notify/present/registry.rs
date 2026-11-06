// SPDX-License-Identifier: MIT OR Apache-2.0
use super::base::NotifyFrame;
use slots::storage::slot::base::Storage;
use smithay::backend::renderer::gles::GlesRenderer;
use crate::smithay_glue::data::ScreenContext;
pub trait NotifyPresenter {
    fn tick(&mut self, next: Option<String>, kernel: &Storage, renderer: &mut GlesRenderer,
        node: &str, screen: &ScreenContext) -> (Option<NotifyFrame>, bool);
}
type Factory = fn() -> Box<dyn NotifyPresenter>;
static FACTORY: std::sync::OnceLock<Factory> = std::sync::OnceLock::new();
pub fn register(factory: Factory) { FACTORY.set(factory).unwrap_or_else(|_| panic!("notify presenter already registered")); }
pub struct Presenter { pub(super) owner: Option<Box<dyn NotifyPresenter>> }
impl Default for Presenter { fn default() -> Self { Self { owner: FACTORY.get().map(|factory| factory()) } } }