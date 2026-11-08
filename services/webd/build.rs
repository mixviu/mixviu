// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
// Emit MIX_GIT_SHA / MIX_GIT_DIRTY / MIX_BUILD_TIME for the
// build-provenance surface (version-discovery contract). Captures this
// repo's (cos) HEAD because this build.rs runs in cos.
fn main() {
    buildinfo::emit();
}
