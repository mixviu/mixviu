# Runtime source closure

All imports are from the immutable MixOS source revision recorded in
`runtime-imports.json`. The original seed manifest is preserved unchanged.
Original versions and schema names are retained intentionally.

| Closure | Reason |
|---|---|
| Mix core, original tests and prelude | Embedded interpreter, async callbacks, existing BusHandler/send/reply seam |
| paths | Mandatory interpreter path primitive dependency; no daemon or transport |
| mimedb | Optional interpreter manifest dependency required for Cargo resolution; not enabled in the client |
| scene-host templates.rs | Native list template preflight, binding evaluation budgets, acceptance before display |
| patched eframe and egui-wgpu | Existing native Wayland/wgpu client lifecycle, renderer and capture fixes, upstream licences/assets |

Initial survey also copied calloop/dirs-sys patches. CBC showed neither was
used in this feature graph, so both closures and patch registrations were
removed. Their import hashes/removal reasons remain in the manifest.

Adaptations are explicit in the manifest: Mix's all-features self test edge is
removed, keeping the runtime feature selection bounded; eframe's patched
Offscreen enum is enabled for the backend's minimal feature rather than only
the broad default-feature alias. Original library source/tests are retained.
Optional-feature interpreter tests are not claimed by the runtime test gate.

The volume reference and seven original helpers remain unchanged. Separate
embedded entrypoint/context/fixture modules derive from them. The renderer's
route shape derives from scene-host view.rs but is a new client adapter,
covering the volume families through existing native egui controls. It does
not import compositor authority, session settings, toolkit services or world.

Cargo.lock is canonicalised by Cargo on CBC from the pinned source lock,
preserving selected external versions/checksums. Optional-feature dependency
resolution is broader than the compiled graph; report actual features/build
dependencies separately. No sibling checkout path or installed Viu binary is
used as the engine.

First-party sources retain original SPDX/copyright headers. Mix's package
licence is MIT; paths, mimedb, scene templates and the new runtime are dual
MIT/Apache-2.0. Vendored client crates retain their upstream licences and
PATCHES.md. Fonts/shaders/assets shipped by those crates retain their source
attribution. No production deployment/orchestration tree is imported.
