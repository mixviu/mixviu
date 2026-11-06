// SPDX-License-Identifier: MIT OR Apache-2.0
#[macro_use]
extern crate model;
pub use world::*;
pub mod capture;
pub mod fps;
pub mod handle;
pub mod notify;
pub mod owner;
pub fn register() {
    ui::register();
    ui_surface::capture::register(owner::open);
    world::surface::draw::fps::register(fps::per_frame);
    world::notify::present::registry::register(|| Box::new(notify::present::registry::Presenter::default()));
}