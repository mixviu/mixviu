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

## PR 47 review round 1 — 2026-11-09

All ten requested items addressed on the same branch. PR remains
https://github.com/mixviu/mixviu/pull/47.
New tested implementation head: `5549936b1585d11e403566a4b19c2ac39b8e9f43`.
The following receipt-only commit appends this section; its SHA is the PR head.

1. **Headless migration:** `identity::find` and `matches` accept legacy
   `MixOS Headless Unknown` only for qualified headless-0. Exact new keys win;
   headless-1 cannot inherit it. Test:
   `model/tests/output_identity.rs::headless_legacy_maps_to_zero_only_and_exact_wins`.
2. **Inactive teleport target:** active filtering now precedes minimum-key
   selection. Test: `drivers/tests/output_identity.rs::legacy_placement_uses_smallest_active_matching_connector`
   disables DP-1 and proves the legacy zone survives on DP-2.
3. **Startup shadowing:** shared `identity::preferred_index` tries exact matches
   across every profile before any legacy match, retaining priority within each
   pass. KMS startup, hotplug target selection and all-inactive recovery share
   this rule. Test: `qualified_selection_beats_earlier_legacy_but_keeps_priority_within_pass`.
4. **Legacy touch claims:** `touch_index` retains an exact qualified claim first;
   for legacy duplicates it prefers the associated libinput output when it is a
   matching driven output. Otherwise the existing first-match order is preserved.
   Test: `legacy_touch_uses_association_only_within_matching_outputs` covers a
   matching association, absent/unmatched association, and explicit priority.
   This fallback is documented in code and the Bus contract.
5. **Port moves:** profile mutation APIs (`upsert_output`, `set_active`,
   `set_touch_device`, `set_default`) now require the complete connected
   preference-key list, including inactive panels, and call `sync_legacy_output`.
   A sole matching panel refreshes/creates the legacy fallback from the complete
   qualified profile; duplicates or unknown topology cannot refresh it. This
   allows a move to a new port to inherit the latest mode/active/touch fields.
   Test: `single_panel_writes_refresh_legacy_for_moves_but_duplicates_do_not`
   exercises all four APIs, checks a move, then duplicate and unknown-topology
   writes. No in-tree production Rust caller currently invokes these legacy
   profile mutation APIs (the repository search found only definitions/tests);
   future callers must supply live topology rather than infer it from saved
   profiles. Raw JSON file edits remain the editor's responsibility.
6. **Public format:** corrected the snapshot comment in `native/src/wire/entry.rs`.
   `docs/docs/spec/bus/commands.conf.mix` now describes the qualified `edid_key`,
   opaque echo-back requirement, legacy/headless migration and touch fallback.
   `rg` audit of `apps/` and `share/` found no `edid_key`, `@connector:` or
   `MixOS Headless` references; no app parser/constructor needs migration.
7. **Last separator:** `identity::stable` explains why it splits on the last
   qualifier. Test: `stable_splits_only_the_last_connector_suffix` uses an EDID
   model containing the separator text.
8. **Teleport caller proof:** `kms::edid::identity::connector_key` is now the
   shared function used by reconcile's `identity_key` and enumeration. Test
   `kms/tests/output_identity.rs::reconcile_connector_key_preserves_edid_placement`
   supplies a serial-42 EDID, constructs the Output, compares preference keys,
   and feeds that connector key through `build_teleport`, retaining the DP-2 zone.
9. **Connector name agreement:** `physical::create` now calls `connector_name`
   directly. A narrow `ConnectorInfo` trait exposes the same three getters on
   real DRM Info and CPU fixtures, allowing the actual constructor to be tested
   without a DRM device or unsafe fabricated struct. Test:
   `physical_output_and_connector_selection_share_names` checks DP, HDMI-A and
   eDP fixtures with `physical::create(...).name() == connector_name(...)`.
10. **EDID-less spelling:** shared `for_connector` deliberately preserves the
    historical Debug spelling in fallback serials (`DisplayPort-1`), while the
    qualifier and runtime name use canonical `DP-1`. Test:
    `enumeration_key_matches_output_even_with_debug_spelled_fallback_serial`
    proves enumeration's saved key equals the real Output preference key.

Round 1 CBC receipts (BUILD_RC and saved logs read):

- `20261109-181155-54b6c08d-a1`, cbc3, **RC 0**, source `e81180f4`:
  `cargo fmt --all`; affected model/drivers/kms/world/headless `--lib --tests`;
  `cargo check --locked -p viud --all-targets`. Captured formatting and the two
  test-only workspace lock dependencies, printed diff and restored this job's
  worker changes in the same job. Applied the captured diff locally.
- `20261109-181328-b0746f10-a1`, cbc2, **RC 0**, exact implementation head
  `5549936b1585d11e403566a4b19c2ac39b8e9f43`: `cargo fmt --all --check`;
  `cargo test --locked -p model -p drivers -p kms -p world --test output_identity`
  (**14 passed**); `cargo check --locked -p viud --all-targets`.

No local Rust commands, deployment, desk, VT or installed-prefix changes.
Existing GPU acceptance tests retain their explicit ignores.