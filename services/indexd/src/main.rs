// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Standalone indexd binary; the program is the library's `main`.

fn main() -> anyhow::Result<()> {
    indexd::main()
}
