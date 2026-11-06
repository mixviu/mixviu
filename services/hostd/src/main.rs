// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
//! Standalone hostd binary; the program is the library's `main`.

fn main() -> anyhow::Result<()> {
    hostd::main()
}
