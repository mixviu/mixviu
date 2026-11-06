// SPDX-License-Identifier: MIT OR Apache-2.0
use std::sync::{Arc, OnceLock};
use graphics::surface::WgpuGlContext;
use crate::UiSurfaces;
type Factory = fn(Arc<WgpuGlContext>) -> Box<dyn UiSurfaces>;
static FACTORY: OnceLock<Factory> = OnceLock::new();
pub fn register(factory: Factory) { FACTORY.set(factory).unwrap_or_else(|_| panic!("UI factory already registered")); }
pub fn create(gpu: Arc<WgpuGlContext>) -> Option<Box<dyn UiSurfaces>> { FACTORY.get().map(|factory| factory(gpu)) }
pub fn is_registered() -> bool { FACTORY.get().is_some() }
