# viud output identity — 2026-11-09

Production fix derived from origin/main on `fix/viud-output-identity`.
PR: https://github.com/mixviu/mixviu/pull/47 (opened and pushed as mixviu-agent).
Implementation head: `c6d7aba223d4bb3ee4951533cb2d15ef1e49f684`.
The final report commit changes only this receipt; the PR records its head SHA.

## Identity contract

`world::state::output_key()` now returns the backend connector/protocol name.
EDID fields are never runtime equality. `output_preference_key()` and
`MonitorIdentity::preference_key()` return `make model serial @connector:NAME`.
The connector qualifier is always present, avoiding identity changes when the
connected set changes. Two identical serial-42 panels remain separate, and
unplugging/replugging one cannot rename or consume the survivor's view state.
This deliberately binds new preferences to ports; physically indistinguishable
units cannot follow swapped cables. `MonitorIdentity::key()` retains the legacy
EDID format for compatibility and physical properties retain the actual EDID.

## Consumer audit

All paths below are relative to `engines/viud/crates`. Direct consumers and
transitive `current_output_key()` / `active_output_key()` consumers were checked.

| Consumers | Identity and reason |
| --- | --- |
| `world/src/state.rs`, `state/wire.rs`, `camera/pin.rs`, `canvas/draw/scene.rs`, `surface/interface/hit.rs`, `surface/interface/core/hit.rs` | Runtime: current/render/cursor output, independent view trees, camera, geometry and hit testing. |
| `world/src/comp/corners.rs`, `comp/session_lock.rs`, `comp/region.rs`, `comp/presentation.rs` | Runtime: select the exact output for corners, lock, region and frame source. |
| `frames/src/draw/present/{parked,callbacks,cursor}.rs`, `frames/src/scene.rs`, `scene/{hooks,layershell}.rs`, `hook/window/interface.rs` | Runtime: callbacks, cursor, fractional scale, render pass, layers and per-output surfaces. |
| `native/src/render/execute.rs`, `native/src/wire/{frame,session}.rs`, `native/src/context/display/apply.rs` | Runtime: redraw registration/release, render scheduling, capture, frame liveness and teardown. |
| `native/src/context/display/reconcile.rs` | Runtime for redraw, retire, live-set and cursor routing; preference for preferred/default/active/mode selection. Compare full preference identity when deciding whether immutable Output properties must be rebuilt on fail-over. |
| `native/src/context/display/mode.rs` | Preference for picker request/watchdog pipe resolution and snapshot updates; runtime for redraw release. |
| `native/src/wire/input/map.rs` | Preference for saved touch claims, converted to runtime before routing. Automatic matching and returned keys stay runtime. |
| `nested/src/scene/compose.rs`, `headless/src/frame.rs` | Runtime: render/capture/redraw routes. |
| `seat/src/pointer/{restore.rs,input/motion.rs}` | Runtime: pointer target, saved live cursor output and teleport destination. |
| `policy-host/src/{project,control,region,input,capture,chrome}.rs` | World identity calls stay runtime: select output for props, geometry, input and capture. |
| `recorder/src/interface.rs`, `graphics/src/capture/registry/source.rs`, `effects/src/host.rs`, `model/src/stats/registry/counter.rs` | Runtime: capture sources, effect state and per-output presentation counters. |
| `comp-model/src/{snapshot,diff,snapshot_tests}.rs`, `policy-host/src/{truth,edges}.rs`, `world/src/comp.rs`, `world/src/comp/geometry.rs` | Separate public `comp_model::snapshot::output_key(name)` slug, not the EDID function. Retained; public comp keys already derive from connector names. Mixed policy-host files retain these public-slug calls too. |
| `kms/src/connector/select.rs`, `native/src/assemble/display.rs`, `native/src/context/display/enumerate.rs`, `native/src/wire/entry.rs` | Preference: startup selection, per-panel modes, settings picker and snapshots. |
| `drivers/src/output/base.rs::build_teleport` | Preference for placement lookup/active flag; converts to connector runtime keys in the live layout. |

Current main has no persisted per-output scale, workspace assignment or scene
profile in OutputProfile (mode/active/touch only); KMS scale is explicitly global.
Viewport trees/cameras and scene/render worker buffers are live runtime state.
No new persistence subsystem was added for those future settings.

## Compatible migration

The preference identity helper tries exact qualified keys first, then the legacy
EDID-only key, independent of profile order. Existing mode/active/touch fields
are copied in full before a qualified profile is edited. The legacy entry stays
available for other ports and old versions; reads do not rewrite the document.
Mode/default/active startup and reconcile paths use compatible lookup.

Legacy teleport placements resolve to one deterministic matching connector,
preserving zone ids rather than copying a zone onto two runtime outputs.
Explicit qualified placements supersede legacy placements for that connector.
Persisted layouts keep preference identities; live placements use runtime keys.

Main currently creates one headless output. Its constructor now accepts an
index and assigns matching name/serial `headless-N`, rather than `Unknown`.
The real constructor is tested with indices 0 and 1; no multi-output CLI or
runtime topology feature is introduced.

## Tests and CBC receipts

- `world/tests/output_identity.rs`: duplicate nonzero serial 42, distinct runtime
  and preference keys, independent view state, real Smithay Space unmap/remap,
  survivor identity and reconnect stability; serialless/virtual identity cases.
- `kms/tests/module_paths.rs`: actual EDID serial-42 parse, unchanged stable
  EDID identity and distinct connector-qualified preference keys; retains zero
  serial and unreadable EDID coverage.
- `model/tests/output_identity.rs`: legacy JSON profile read, whole-profile
  inheritance, exact-over-legacy priority, mode/touch preservation and JSON
  round-trip with independent active flags for the pair.
- `drivers/tests/output_identity.rs`: preference-to-runtime layout conversion,
  unplug survivor, reconnect restoration, legacy layout and explicit override.
- `headless/src/wire.rs`: real synthetic constructor yields stable distinct
  `headless-0` / `headless-1` serials and both identity kinds.

CBC jobs (saved logs read; no local Rust execution):

1. `20261109-174328-ab93993b-a1`, cbc2, RC 0: initial formatting capture at
   `0993cbb0`. Superseded by the next complete specimen.
2. `20261109-174530-e7aba0b6-a1`, cbc2, RC 0 at `bf3825c4`:
   `cargo fmt --all`, print diff, restore this job's changes; then
   `cargo test --locked -p model -p world -p drivers -p kms -p headless --lib --tests`
   and `cargo check --locked -p viud --all-targets`. All ran successfully.
   Formatting diff applied locally; final source is c6d7aba2 (format/comments).
3. `20261109-174718-0ef39d70-a1`, cbc2, RC 0 at
   `c6d7aba223d4bb3ee4951533cb2d15ef1e49f684`: `cargo fmt --all --check`,
   `cargo test --locked -p world -p drivers -p model --test output_identity`
   (5 passed), and `cargo check --locked -p viud --all-targets`.

All new CPU regressions passed. Existing GPU/headless acceptance tests retained
their explicit ignores. No real-glass/GPU rendering acceptance, VT, desk or
installation changes were performed. This is a PR for production main, not a
lab promotion or deployment.