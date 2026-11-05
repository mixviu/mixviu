# Lab source checks

The isolated runtime lab extends the preserved source seed with a client
specimen. It has no dependency on a sibling source checkout and provides no
installer or production acceptance. See [runtime boundaries](runtime-lab.md).

Run the static seed-integrity check with the installed Mix interpreter:

    mix tools/verify_seed.mix
    mix tools/verify_runtime.mix
    mix tools/verify_seed_pins.mix

These checks read files and provenance; they do not execute the volume behaviour,
change audio, launch a desktop, install code or compile Rust.

Rust checks run on the configured native build workers, not the workstation:

    cargo fmt --all -- --check
    cargo test --workspace --locked

Submit those commands through the maintainer's build service for the exact
pushed source revision. Do not run them locally. Use scoped package gates for
the runtime lab; imported optional-feature tests are not automatically part of
that gate. The retained original seed tests check their existing contracts;
runtime and native-session acceptance require their own receipts.

The imported lockfile is the reachable dependency closure of the recorded
source lock. External versions/checksums remain pinned. The three internal
packages use the seed workspace's 0.1.0 version; that is not a product release.
The runtime adds dependencies. `seed-external-pins.json` records all 14 registry
identities from the immutable seed revision; its verifier checks those names,
versions, sources and checksums remain in the expanded lock. It does not claim
that the expanded graph has only 17 packages or that every locked feature is
compiled. Runtime provenance checks cover retained imports and adaptations.

Record new source imports and any adaptations in provenance. Frozen import
hashes identify this seed; when files intentionally evolve, preserve the import
record and explicitly update the integrity policy rather than disguising edits
as an unchanged copy.
