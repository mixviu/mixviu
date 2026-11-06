// SPDX-License-Identifier: MIT OR Apache-2.0
pub mod appearance;
pub mod command;
pub mod mount;
pub mod seat;
pub mod store;
pub mod templates;
pub mod images;
pub mod icons;
pub mod freedesktop;
pub mod view;
pub mod panels;
pub mod runtime;
pub use command::SceneCommand;
pub const DEFAULT_SERVICE: &str = "shell";
