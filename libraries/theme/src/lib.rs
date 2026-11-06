// SPDX-License-Identifier: MIT OR Apache-2.0
//! Egui-free theme values shared by the compositor and UI adapters.
pub mod fonts;
pub mod theme;
pub mod ui_scale;
pub use theme::*;
pub mod strings {
    pub use action::Strings;
}
pub mod icons {
    pub const DEFAULT_STROKE: f32 = 1.5;
}

pub mod region;
