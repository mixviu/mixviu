// SPDX-License-Identifier: MIT OR Apache-2.0
pub mod appearance;
pub mod command;
pub mod freedesktop;
pub mod icons;
pub mod images;
pub mod mount;
pub mod panels;
pub mod runtime;
pub mod seat;
pub mod store;
pub mod templates;
pub mod view;
pub use command::SceneCommand;
pub const DEFAULT_SERVICE: &str = "shell";

pub(crate) mod tooltip;

pub(crate) mod preferences;
