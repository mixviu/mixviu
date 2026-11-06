// SPDX-License-Identifier: MIT OR Apache-2.0
//! Plain compositor UI surface and future capture Bus contracts.
pub mod element;
pub mod error;
pub mod handle;
pub mod input;
pub mod interface;
pub mod space;
pub mod view;
pub use element::UiRenderElement;
pub use handle::HandleId;
pub use interface::{UiSurfaces, PaneView};
pub use space::{Transform, UiSpace};
pub use view::UiItemView;
pub mod producer;

pub mod factory;
