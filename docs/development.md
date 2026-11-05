# Source-seed checks

The seed is deliberately small and does not provide an engine binary or an
installer. It has no dependency on a sibling source checkout.

Run the static seed-integrity check with the installed Mix interpreter:

    mix tools/verify_seed.mix

The check reads files and provenance; it does not execute the volume behaviour,
change audio, launch a desktop, install code or compile Rust.

Rust checks run on the configured native build workers, not the workstation:

    cargo fmt --all -- --check
    cargo test --workspace --locked

Submit those commands through the maintainer's build service for the exact
pushed source revision. Do not run them locally. The small seed workspace
contains only expr, strict and scene. The retained original tests check their
existing contracts; no app/runtime acceptance is implied.

The imported lockfile is the reachable dependency closure of the recorded
source lock. External versions/checksums remain pinned. The three internal
packages use the seed workspace's 0.1.0 version; that is not a product release.

Record new source imports and any adaptations in provenance. Frozen import
hashes identify this seed; when files intentionally evolve, preserve the import
record and explicitly update the integrity policy rather than disguising edits
as an unchanged copy.
