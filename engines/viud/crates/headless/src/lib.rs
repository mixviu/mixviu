// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! headless: viud's render-node backend with a virtual output and no seat.
//!
//! GBM/EGL owns only a render node. Frames are full repaints into two
//! offscreen textures, paced by an on-demand calloop timer at output refresh.

#[macro_use]
extern crate model;

mod frame;
#[cfg(feature = "test-support")]
mod record_fixture;
pub mod wire;

/// OSD paths injected by native integration fixtures. Keeping the feature in
/// the backend lets workspace test builds enable it through headless's existing
/// self dev-dependency, without enabling hooks in normal compositor builds.
pub fn test_osd_paths() -> Option<(std::path::PathBuf, std::path::PathBuf)> {
    #[cfg(feature = "test-support")]
    {
        Some((
            std::env::var_os(paths::env::test::TEST_VIUD_OSD_BUNDLE)?.into(),
            std::env::var_os(paths::env::test::TEST_VIU_EXE)?.into(),
        ))
    }
    #[cfg(not(feature = "test-support"))]
    None
}
