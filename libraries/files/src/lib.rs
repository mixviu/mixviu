// SPDX-License-Identifier: MIT OR Apache-2.0
//! Minimal coherent native source closure; corpus/database modules not imported.
pub mod atomic;
pub mod chooser;
pub mod error;
pub use error::{FilesError, Result};
