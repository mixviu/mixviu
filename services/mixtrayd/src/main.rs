// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Standalone mixtrayd binary; the program is the library's `main`.

fn main() -> anyhow::Result<()> {
    // --version/-V first, before the tokio runtime the library's main builds.
    // This also registers the build stamp before any setting read.
    buildinfo::exit_on_version!();
    mixtrayd::main()
}
