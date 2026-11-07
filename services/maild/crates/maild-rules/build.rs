// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>

// Emit MIX_GIT_SHA / MIX_GIT_DIRTY / MIX_BUILD_TIME so `--version`
// reports this crate's build provenance (buildinfo::exit_on_version!).
fn main() {
    buildinfo::emit();
}
