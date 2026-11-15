// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Capture encoding: GPU→CPU readback, PNG image save, lossless VFL video,
//! default save paths, and the XDG portal "Save As"
//! file dialog.
//!
//! Saving was deferred in the first capture cut; this is where it lands. All
//! of it is best-effort and isolated from the render path (VFL and portal
//! calls run on background threads). Legacy re-encoding is a separate helper.

// Developer logging: bring error!/warn!/info!/trace!/abort! into scope.

pub mod frame;
#[cfg(feature = "desktop-dbus")]
pub mod portal;
#[cfg(not(feature = "desktop-dbus"))]
#[path = "encode/dialog_unavailable.rs"]
pub mod portal;
pub mod readback;
pub mod reencode;
pub mod save;
pub mod vfl;

pub use frame::Frame;
pub use readback::{AsyncReadback, readback};
pub use reencode::{
    OptimizedCodec, ReencodeJob, ReencodeStatus, ffmpeg_format, partial_path, reencode_detached,
    save_fallback,
};
pub use save::{default_path, save_png};
pub use vfl::VideoEncoder;
