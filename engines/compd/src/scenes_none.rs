// SPDX-License-Identifier: MIT OR Apache-2.0
//! No in-process scene furniture in the compositor-only profile.
use smithay::reexports::calloop::LoopHandle;
use world::state::Loop;
pub struct Scenes;
impl Scenes {
    pub fn start(_lp: &Loop, _service: Option<&str>, _handle: &LoopHandle<'static, Loop>) -> Option<Self> { None }
    pub fn service(&mut self, _lp: &mut Loop) {}
    pub fn shutdown(self) {}
}
