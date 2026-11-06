# Four-engine integration specimen

Lab only, on `lab/mixviu-poc`; no promotion or installation.

Inputs:

- Track A `25622ff14016ae9f6fd0891700446d90dbfa3b12` is the base.
- Track B `be6bf5e2d33d706fb76046ad8676e16aefac770d` supplies the compositor,
  furniture, supporting libraries, vendor patches and scene bundles.
- App ports `c8132ddf2bde63a9e81025964f549fc2f1a30937` supplies `libraries/viu`
  and the application specimens, including their reference implementations.

The earlier import manifests remain immutable records of those inputs.

## Duplicate-library disposition

Track A's interpreter is unchanged. All engines resolve `mix` to that one
`libraries/mix`. Viu registers `viu_context` and its other host functions with
the existing extension API; the trimmed interpreter is excluded entirely.

Track A's bus, config, design, props, settings, strict, expr, logging, paths,
assets, mesh, mesh-trust, edit and mimedb remain. The app-ports Bus changes
(including its older TCP connection behaviour) are not imported. Track B's
new libraries supply the desktop closure. Its icons catalogue is retained
whole, including the additional furniture icons and existing enum API.
Its macros and thumbs libraries retain their complete original APIs.

Two app-port additions are required beside Track A's full libraries:

- Scene schema adds the existing Viu control/grid/dock/band/split vocabulary,
  optional window slots and column surface ports. The separate host model
  budget API preserves the original document-limited entry point. Their
  imported tests remain; original scene vocabulary names/defaults are retained.
- Files adds the lossless Unix path-token codec used by the chooser and app
  file data. The full files library remains; it is not replaced by the trimmed
  chooser-only app-ports version.

## Engine adaptations

`viu-engine` owns one `viu` executable. Its `furniture` library is Track B's
furniture implementation; `viu --host` selects it. The executable otherwise
delegates to the shared app runtime's `viu run APP_DIR`. `libraries/viu`
continues to own reusable app widgets and adapters. Quoin and Dopus have
bundle metadata; Dopus initialisation and event routing stay in Mix.
Reference Rust app sources are retained as provenance/oracles, outside the
workspace; they are not shipped app hosts.

`viud` builds compd's source in a separate package with `backend-all` and no
`inproc-ui` default. The original compd package/default features remain the
oracle. Per-package builds preserve Track A's feature-unification guard.

The compositor publication reservation is in `services/noded`, with its
foreign-publisher refusal and broker-attestation test. Test-broker embeds the
real service modules directly; the copied broker implementation is removed.

The integrated B6 gate exposed layer lifecycle defects: a furniture layer
hidden before its first buffer discarded a configure the server had not
invalidated, while a genuinely unmapped layer needed a new initial configure
on its remap commit. The client now tracks whether it attached a buffer and
keeps hidden layers out of draw deadlines. Presentation control has an
explicit event-loop wake, independent of whether egui has run its first pass.
The compositor sends pending configures after arrangement. The original
launcher mapping/focus assertion remains and now also checks close/reopen.

## Reproduction

All Rust commands run through CBC. `tools/p2_gate.mix` builds each engine
separately, copies it into a source-addressed artifact directory before other
builds, prints hashes/ldd, and runs the integration and existing native/oracle
gates. `tools/p2_rehearsal.mix` uses the existing native Bus test harness and
owned headless/nested compositors, without a live display. Measurements and
captures belong to the private operational receipt, not this public tree.
