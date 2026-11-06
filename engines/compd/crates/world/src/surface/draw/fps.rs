// SPDX-License-Identifier: MIT OR Apache-2.0
use smithay::{backend::renderer::gles::GlesRenderer, utils::{Physical, Size}};
use crate::state::Loop;
type Hook = fn(&mut Loop, &mut GlesRenderer, Size<i32, Physical>);
static HOOK: std::sync::OnceLock<Hook> = std::sync::OnceLock::new();
pub fn register(hook: Hook) { HOOK.set(hook).unwrap_or_else(|_| panic!("FPS owner already registered")); }
pub fn per_frame(state: &mut Loop, renderer: &mut GlesRenderer, size: Size<i32, Physical>) {
    if let Some(hook) = HOOK.get() { hook(state, renderer, size); }
}