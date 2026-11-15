// SPDX-License-Identifier: MIT OR Apache-2.0
//! Headless media format contracts. No codecs or desktop dependencies.
pub mod vfl {
    pub const VERSION: u32 = 1;
    pub const RECORD_HEADER_BYTES: usize = 17;
    pub const KEY: u8 = 0;
    pub const DELTA: u8 = 1;
    pub const REPEAT: u8 = 2;
    pub const MAX_FRAME_BYTES: usize = 256 * 1024 * 1024;
    pub const CHUNK_BYTES: u64 = 64 * 1024 * 1024;
}
