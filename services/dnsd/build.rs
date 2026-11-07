// SPDX-License-Identifier: MIT OR Apache-2.0
// Copyright 2026 Mark Constable <mc@mixos.dev>
// Emit MIX_GIT_SHA / MIX_GIT_DIRTY / MIX_BUILD_TIME for the
// build-provenance surface. Captures this workspace's committed HEAD.
fn main() {
    buildinfo::emit();
}
