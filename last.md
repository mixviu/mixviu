# CI flakes 2 — 2026-11-10

Branch: `fix/ci-flakes-2`, based on `origin/main` at
`aaedc99d03a3c6c286fda4c3e292988c688c52f6`.
Tested code head: `e055f84615df0bf63c17f24980d4714440768db0`.
The receipt commit changes only last.md; the PR exposes its current head SHA.

## Audio: queued change mistaken for terminal exit

The source publishes per read, not per printf. The fixture repeats sink #5.
If the reader publishes one sink change, the test consumes it, and another
read publishes the duplicate, the queue is ready after Drop for an ordinary
change. The old assertion required an empty queue and did not inspect
`closed`. Drop sets cancelled before killing the group, then joins the reader;
it is not an early check for worker completion.

The replacement drains the now-stable queue after Drop, checks each remaining
batch contains the expected sink change and rejects any closed marker.
The original filtering, coalescing and child-reaping assertions remain.

Controlled proof: a FIFO-fed real cat child emits one sink line, the test
consumes its batch, then emits the duplicate and waits until that second read
has queued a batch before dropping the source. Restoring the old empty-queue
assertion fails (cargo rc 101); the corrected assertion passes (rc 0).

A natural EOF can pass the worker's cancelled check before a concurrent Drop
records cancellation; that is an already-observed stream exit. This is not
the cause with the still-live fixture. The public source_unwatch path removes
the pending slot BEFORE dropping the source, so even such a late terminal
callback is discarded. No product change is required.

## Lifecycle: boot trace corrupted by formatted fixture writes

The failing wait was for one boot record, before RELOAD, not a heartbeat or
commit. CI run [42581336097](https://github.com/mixviu/mixviu/actions/runs/42581336097)
contains:

```text
legacy-ready|{"pid":boot|{"legacy":11230,"managed":11231}|t=407.542488794
11230}|t=0
```

The helper's writeln!(File, format, pid) performs several writes. The citizen's
append landed between them, so parse_trace could never find a valid boot
record. Waiting longer cannot repair those bytes. The helper now formats the
whole readiness record before write_all, giving the regular append file one
complete short record per write.

Controlled proof: a Write adapter inserts a valid boot record after each
write. Restoring the old writeln! makes the parser regression fail (rc 101);
the complete-record helper passes (rc 0). This forces the CI interleaving.

The grace assertion still requires at least 1.5 seconds between candidate-done
and commit and verifies the old child is reaped. The heartbeat lives until the
citizen is stopped, rather than expiring after five wall-clock seconds.
At least one stamp must lie strictly inside the grace window: that proves
reactor progress; the previous minimum of five also imposed scheduler
throughput. A runner starved for the entire window can still fail this
wall-clock responsiveness test; deadlines were not lengthened.

## CBC evidence

All Rust commands ran on CBC. Saved logs were read. Two yes processes and
each cargo test process (including descendants) were pinned to the same
allowed CPU with taskset. Each original named test was invoked separately
50 times before and 50 times after; counters include every invocation.
Load processes were stopped in finally.

| Run | Audio pass/fail | Lifecycle pass/fail |
| --- | --- | --- |
| Before | 50 / 0 | 50 / 0 |
| After | 50 / 0 | 50 / 0 |

Both baseline loops passed: repetition alone did not expose the races.
Each controlled regression failed once with the old code and passed once
with the fix. The final module/binary run repeats both passing regressions.

| CBC job | Revision | BUILD_RC | Evidence |
| --- | --- | --- | --- |
| 20261110-002554-b937890f-a1 | aaedc99d | 1 | Load harness setup refused a stdout path string; no test ran. Corrected to native stdout null and requeued. |
| 20261110-002703-51923577-a1 | aaedc99d | 0 | Before: 50/50 each under load. |
| 20261110-003109-73e78118 | 78417d67 | 0 | CBC rustfmt diff, fixed regressions rc 0 each, old-code mutations rc 101 each. Own worker edits restored in finally. |
| 20261110-003329-79ce459c | e055f846 | 0 | Final checks and after loops: 50/50 each. |
| 20261110-002809-00858da7 | 3d39ac37 | 143 | Superseded queued probe, cancelled before running. |
| 20261110-003024-e567df98 | 78417d67 | 143 | Superseded queued probe, cancelled before running. |

Final checks: cargo fmt --all -- --check; cargo clippy --locked -p lib-mix
--all-targets; separately cargo clippy --locked -p mix --all-targets;
cargo test --locked -p lib-mix --lib desktop_events:: (17 passed);
cargo test --locked -p mix --test serve_reload_lifecycle (8 passed,
1 ignored subprocess helper); tools/ci.mix env (196 names, zero problems).
Existing unrelated deprecation/dead-code/clippy warnings remain.

Saved logs (under the maintainer's CBC log directory):

- `20261110-002703-cbc3-aaedc99d03a3-20261110-002703-51923577-a1-59246340.log`
- `20261110-003109-cbc3-78417d6752f8-20261110-003109-73e78118-aa20931d.log`
- `20261110-003329-cbc3-e055f84615df-20261110-003329-79ce459c-6326d611.log`

PR: https://github.com/mixviu/mixviu/pull/50
Pushed and opened through gh_agent as mixviu-agent.
No product changes, timeout increases, new environment variables, deployments,
desktop/VT interaction or installed-prefix writes.

---

# Quoin P1 hand-off — 2026-11-09

Implemented P1 items 1, 2, 4 and 5. Item 3 remains the later lab.
Author: Codex (Sol).

- Branch: `feat/quoin-p1-dev-loop`
- Base: `0780b2ea` from `origin/docs/quoin-p0` at job start. P0 has since advanced; no other session's branch was merged.
- P1 PR: https://github.com/mixviu/mixviu/pull/38 (draft, base main)
- P0 dependency: https://github.com/mixviu/mixviu/pull/36
- P1 implementation head before this receipt: `90dad687e2c40dfc794573f7065e4eb1b62f5f02`
- Final code tested: `3835467abdcf6556f850bf37e833e54b6cdaa6af`; subsequent commit changes only the Bus registry documentation.

## Files

- `engines/viud/scripts/scenes.mix`: shared/custom library watches and quoin.reload handler. User lib is covered by the installed-root watch.
- `engines/viud/scripts/lib/scenes.mix`: conservative dependency fan-out, cached helper checks, library fingerprints/overflow recovery, group preflight and atomic host submission. Both discovery paths exclude the reserved lib directory.
- `engines/viud/scripts/scene_preview.mix`: passive use of real loader candidate, behaviour checks and recorded JSON model.
- `engines/viu/src/store.rs`: clone/stage/commit load envelopes with sources; refuses the entire batch on a bad candidate or seat collision.
- `engines/viu/src/store_tests.rs`: batch failure retains revisions/seats; successful fan-out publishes both notices.
- `engines/viu/src/dev_loop.rs`: first-diagnostic extraction and foreground error strip.
- `engines/viu/src/preview.rs`: plain watched specimen window, authored/recorded model, passive actions, last-good retention; discovers validator from current checkout or VIU_PREVIEW_VALIDATOR.
- `engines/viu/src/main.rs`, `lib.rs`: preview CLI/module wiring.
- `engines/viu/src/runtime/toast.rs`: toast refusal strip.
- `engines/viud/tests/scenes_test.mix`: helper failure/recovery, fan-out, quoin scopes and native inotify timing.
- `docs/scene-documents.md`, `docs/docs/spec/bus/commands.conf.mix`: P1 contracts and registered command/batch shapes.

## runtime.rs hunks for P2 coordination

Only State.errors and its initialisation; Program::show's diagnostic strip; worker's scenes.changed subscription/initial scenes.list; and its authenticated scenes topic receipt branch. No bindings, sources, comp subscriptions, output polling or structural rendering changes.

## Validation

- Mix loader: 400 assertions pass. A real fs_watch/fs_wait helper edit plus two-dependent headless loader fan-out is asserted below one second. Host/process adapters are fake in this transition test; real host atomicity is covered by the Rust store test.
- Mix lint: scenes.mix, lib/scenes.mix and scene_preview.mix pass with zero errors/warnings.
- Registry strict-data load and unique quoin.reload entry verified. Full source registry gate cannot start: pre-existing missing tools/lib/command_registry_source.mix.
- Existing scenes_io integration test: default invocation hits obsolete mix_path builtin; explicit MIX_TEST_NODED_EXE=/opt/mixos/bin/noded bypasses it but installed broker cannot establish owned private-bus readiness. No production configuration changed.
- git diff --check passes.
- No desk4, VT, /opt writes, production installs, behaviour state hand-off or visual acceptance performed. Opus's eye checks remain pending. Security is expected red until #21.

## CBC receipts

All Rust work was on CBC. Saved final log read in full.

| Job | Commit | Result |
|---|---|---|
| 20261109-153955-8005b781-a1 | 64588834 | BUILD_RC=101, initial egui App API mismatch, fixed before later passes |
| 20261109-154304-d021212d-a1 | 18be0c32 | BUILD_RC=0; 99 passed, 5 ignored |
| 20261109-154543-fa958fbe-a1 | d7442ba2 | BUILD_RC=0; 101 passed, 5 ignored |
| 20261109-154901-329f6c89 | d7442ba2 | BUILD_RC=0; CBC formatting diff applied locally; own worker changes restored in same job |
| 20261109-154948-87cffd24-a1 | 8438f7b0 | BUILD_RC=0; 101 passed, 5 ignored |
| 20261109-155106-f61e0c72 | 369309d4 | BUILD_RC=75; worker lock busy, command did not run; requeued |
| 20261109-155215-02577fd0 | 369309d4 | BUILD_RC=75; other worker lock busy, command did not run; requeued |
| 20261109-155336-3c23428c-a1 | 3835467a | BUILD_RC=0 on cbc3; fmt --check, 101 host tests (5 ignored), cargo check --locked -p viu --bin viu |

Final log: `/home/user/.cache/mixos/cbc-build/20261109-155421-cbc3-3835467abdcf-20261109-155336-3c23428c-a1-93002c98.log`.
Queued formatter 20261109-154541-e5d7aa8a was superseded/cancelled; it was not a validation pass.

## Trade-offs

Computed require paths mean library dependency selection conservatively covers all enabled behaviour scenes, including transitive imports. Helpers are checked without executing them. Document acceptance is atomic; behaviour runtime faults after acceptance retain the existing restart/diagnostic semantics. Reload still kills/spawns behaviours; no state hand-off is claimed.

Preview has headless load/recorded-model/refusal/repair coverage and binary compilation coverage. Its live window and overlay pointer behaviour still need Opus's visual check. This receipt is prepended to the existing tracked hand-off history. The current PR head includes the receipt commit.

---

# viud output identity — 2026-11-09

---

# SQLite security fix — 2026-11-09

PR: https://github.com/mixviu/mixviu/pull/49 (`fix/security-sqlite` -> `main`),
opened and pushed through mixviu-agent. Validated head SHA:
`23809a9d26fa97553ef9460dd614417baf4d4cc4`; this receipt-only commit follows it.
The PR exposes the current head SHA. Based on origin/main `95382ad1`.

- rusqlite: 0.38.0 -> 0.41.1; crates.io confirms its libsqlite3-sys ^0.39.0 requirement.
- libsqlite3-sys: 0.36.0 -> 0.39.1; linked bundled SQLite: **3.54.1**.
  The new files/sqlite_version test prints rusqlite::version(), asserts at least
  3.54.0, and checks SELECT sqlite_version() agrees.
- Targeted cargo update reused hashlink 0.12.2 and removed 0.11.1; no other
  registry versions changed. SQLite remains bundled; no new C crate or deny ignore.
- The old spamlite 0.9.6 pin also required rusqlite ^0.38. New pin
  `e4cda94713f696414550f414feaea4e8d6c4355e` is exactly one manifest change
  from `98f91a9d8142f92ae89ba4f070a165ce86fa5e8d`, on spamlite's retained
  fix/security-sqlite branch. No unrelated spamlite changes were imported.
- API adapters: consume column_names() directly as an iterator in shell stats;
  webd AuthGuard borrows the locked connection mutably for authorizer install
  and Drop cleanup. Queries use that guard's connection; policy, binding and
  schema isolation remain unchanged. Existing webd authorizer tests pass.
- Open PR overlap checked: #8 and #25 both touch Cargo.toml/Cargo.lock but
  neither upgrades SQLite. #25 references old hashlink 0.11.1; lockfiles may
  need regeneration after merge. Neither PR was edited.

CBC jobs (all worker any, CARGO_BUILD_JOBS=2; saved logs inspected):

- `20261109-203931-86119cbe-a1`: rc=101; found the pinned spamlite constraint.
- `20261109-210051-72e1ea03-a1`: rc=0; targeted dependency resolution;
  exact generated lock diff applied locally.
- `20261109-210123-3cce10fa-a1`: rc=1; fmt output applied, version test and
  cargo-deny passed; temporary foreign-gate binary-name adaptation corrected.
- `20261109-210242-a4c52dd8-a1`: rc=1; found webd's mutable authorizer API;
  version test, cargo-deny and adapted foreign gate passed.
- `20261109-210420-33f8ce8f-a1`: **BUILD_RC=0**, validated source above.
  Full `mix tools/ci.mix all`, explicit SQLite version test, `cargo deny check`
  and foreign gate each returned 0. Across 605 Rust test summaries: 10,614
  passed, zero failed, 100 ignored (includes the explicit version test).
  Mix policy checks and lint ratchet passed, zero regressions.

MixViu main lacks tools/foreign_gate.mix and tools/foreign.conf.mix. CBC used
temporary copies of MixOS's existing gate/policy with repository root adapted,
strict crates mix/mix-shell/lib-mix, and the inherited system allocator exception
mapped to the renamed mix binary. Result: 15 foreign crates, all listed;
strict closure and ELF checks clean. No committed policy change or new exception.

GitHub security-ok passed on the validated source. Code-owner approval and
cross-family review remain merge requirements. No deployment, VT interaction
or /opt writes.

---

# Quoin P0 — 2026-11-09

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

## PR 47 review round 2 — 2026-11-09

All four requested items addressed on `fix/viud-output-identity`.
PR: https://github.com/mixviu/mixviu/pull/47.
New implementation head: `ce38baa18c38fbc5265735f98d8d7ddd497de90c`.
The receipt-only commit appending this section follows that implementation.

1. **Qualified touch override:** replaced the first-claim lookup and identity-only
   `touch_index` with `resolve_touch_claim`, which inspects the effective profile
   for each driven output. Any exact qualified profile decides its own claim,
   including None or a different device. Only outputs without an exact profile
   can inherit the legacy claim; association chooses among those candidates.
   Qualified positive claims take priority over inherited ones. When a saved
   claim exists but all matching live outputs override it, the native caller
   returns unmapped instead of reintroducing it through automatic association,
   size or EDID heuristics. With no saved claim, normal automatic routing remains.
   The Bus spec documents this boundary.

   Tests in `model/tests/output_identity.rs`:
   - `cleared_qualified_touch_does_not_inherit_legacy_on_duplicate_panels` uses
     the real setter to clear DP-2 with both panels connected, asserts legacy L
     still claims touch and DP-2's exact profile is None, then verifies association
     with DP-2 routes to eligible DP-1. It also covers association absent, only
     the overridden panel driven, all candidates overridden and another exact
     device claim.
   - `legacy_touch_single_panel_and_unqualified_duplicates_keep_their_routes`
     covers a single panel, duplicate panels without qualified profiles, associated
     and unassociated selection, an unclaimed device, and clearing the sole panel
     (which refreshes legacy and restores automatic mapping).
2. **Teleport cleanup:** removed the redundant `!output_active(...)` disjunct
   after the earlier active filter. The legacy placement, active-connector and
   unplug/replug tests continue to pass unchanged in behaviour.
3. **Complete topology API:** all four profile mutation APIs and
   `sync_legacy_output` now take the private-field `ConnectedOutputs` wrapper.
   Construction is explicitly named `from_complete_snapshot`; its API requires
   all connected preference keys, including inactive panels, never just the
   driven set or saved profiles. Debug assertions reject unqualified keys and
   repeated runtime connector names. The API explicitly states that shape checks
   cannot discover hardware omitted by the caller. Empty/unknown topology still
   cannot authorise shared legacy writes. Existing duplicate/unknown-topology
   mutation tests use the wrapper; two new debug-build panic tests prove the
   malformed-snapshot guards.
4. **Display flags verified:** `DisplayInfo.active` marks the primary connector,
   so its handle comparison is intentional and independent of saved preference
   identity. The saved active flag is exposed as `DisplayInfo.enabled`, already
   resolved through `profile::active` and `identity::find`. Kept those meanings
   intact and clarified the enumeration comment. Extracted `profile::active_in`
   as the exact lookup called by `active`; the new KMS regression
   `display_enabled_lookup_accepts_migrated_legacy_profile_and_exact_override`
   verifies a migrated qualified key inherits a disabled legacy profile and that
   a later explicit qualified enabled profile wins over it.

CBC receipts (BUILD_RC checked and saved logs read):

- `20261109-183151-fca1937b-a1`, cbc2, **RC 0**, source `8315126e`:
  `cargo fmt --all`, then the same round 1 suites:
  `cargo test --locked -p model -p drivers -p kms -p world -p headless --lib --tests`
  and `cargo check --locked -p viud --all-targets`. Worker formatting diff was
  printed and restored in that same job, then applied locally.
- `20261109-183303-d0250a55-a1`, cbc2, **RC 0**, exact implementation
  `ce38baa18c38fbc5265735f98d8d7ddd497de90c`: `cargo fmt --all --check`;
  `cargo test --locked -p model -p drivers -p kms -p world --test output_identity`
  (**18 passed**); `cargo check --locked -p viud --all-targets`.

Variables (gold column: read by gold term / portald / inputd at mixos 8779f33,
directly or through libs bus, paths, config, settings, design, toolkit,
citizen, logging, buildinfo):

| Old | New | Gold reads |
|---|---|---|
| `MIXOS` (root) | `MIX_ROOT` | term, portald, inputd (paths) |
| `MIXOS_PREFIX`, `MIXOS_SHARE`, `MIXOS_ETC`, `MIXOS_VAR`, `MIXOS_RUN`, `MIXOS_CACHE` | `MIX_PREFIX`, `MIX_SHARE`, `MIX_ETC`, `MIX_VAR`, `MIX_RUN`, `MIX_CACHE` | term, portald, inputd |
| `MIXOS_NODED_URL`, `MIXOS_NODE_CONFIG`, `MIXOS_BUS_SOCKET`, `MIXOS_SESSION_FD`, `MIXOS_BROKER_ACCOUNT` | `MIX_NODED_URL`, `MIX_NODE_CONFIG`, `MIX_BUS_SOCKET`, `MIX_SESSION_FD`, `MIX_BROKER_ACCOUNT` | term, portald, inputd (bus) |
| `MIXOS_APP_HOME`, `MIXOS_APPS_HOME` | `MIX_APP_HOME`, `MIX_APPS_HOME` | term, portald, inputd (config) |
| `MIXOS_SETTINGS_INSTANCE`, `MIXOS_SETTINGS_PROFILE` | `MIX_SETTINGS_INSTANCE`, `MIX_SETTINGS_PROFILE` | term, portald |
| `MIXOS_DECORATIONS` | `MIX_DECORATIONS` | term, portald |
| `MIXOS_COMP_SERVICE` | `MIX_COMP_SERVICE` | term |
| `MIXOS_COMPD_CONFIG` | `MIX_VIUD_CONFIG` | no |
| `MIXOS_OUTPUT_SCALE`, `MIXOS_HEADLESS_SIZE`, `MIXOS_CONTAIN_CHILDREN`, `MIXOS_USER_KEYMAP` | `MIX_VIUD_OUTPUT_SCALE`, `MIX_VIUD_HEADLESS_SIZE`, `MIX_VIUD_CONTAIN_CHILDREN`, `MIX_VIUD_USER_KEYMAP` | no |
| `MIXOS_VT`, `MIXOS_INSTANCE`, `MIXOS_SCENE_SERVICE`, `MIXOS_SCREENSHOTS_DIR` | `MIX_VT`, `MIX_INSTANCE`, `MIX_SCENE_SERVICE`, `MIX_SCREENSHOTS_DIR` | no |
| `MIXOS_EDIT_RECOVERY(_DIR)`, `MIXOS_INDEXD_SEARCH`, `MIXOS_WEBD_SESSION_KEY_PATH`, `MIXOS_SPEC_DIR`, `MIXOS_SPEC_RELEASE_DIR`, `MIXOS_SPEC_RELEASE_SHA256`, `MIXOS_NODED_EXPECTED_SERVER_PID` | `MIX_EDITD_RECOVERY(_DIR)`, `MIX_INDEXD_SEARCH`, `MIX_WEBD_SESSION_KEY_PATH`, `MIX_NODED_SPEC_DIR`, `MIX_NODED_SPEC_RELEASE_DIR`, `MIX_NODED_SPEC_RELEASE_SHA256`, `MIX_NODED_EXPECTED_SERVER_PID` | no |
| `MIXOS_MACRO_{APP,ID,PATH,TARGET,TIER}`, `MIXOS_MIX` | `MIX_MACRO_*`, `MIX_EXE` | no |
| `MIXOS_BIN`, `MIXOS_SRC`, `MIXOS_TERM_BIN`, `MIXOS_AGENT_CLI_MIX`, `MIXOS_MAN_SOURCE`, `MIXOS_MAN_URL` | `MIX_BIN`, `MIX_SRC`, `MIX_TERM_EXE`, `MIX_AGENT_CLI_EXE`, `MIX_MAN_SOURCE`, `MIX_MAN_URL` | no |
| `COMPD_*` runtime (`COMPD_XWAYLAND`, `COMPD_FRAME_TRACE*`, `COMPD_KEYMAPS`, `COMPD_CAPTURE_*`, `COMPD_TEARING`, ...) | `MIX_VIUD_XWAYLAND`, `MIX_VIUD_FRAME_TRACE`, ... (viud engine word) | no |
| `COMPD_*` test-only (`COMPD_TEST_RENDER_NODE`, `COMPD_TEST_BINARY`, `COMPD_WLCS_*`, `COMPD_NATIVE_TEST_BINARY`, ...) | `MIX_TEST_VIUD_RENDER_NODE`, `MIX_TEST_VIUD_EXE`, ... (EXE for binaries) | no |
| `VIU_SERVICE`, `VIU_BIN`, `VIU_COMP_SERVICE`, `NODED_SERVICE`, `DLNAD_CONF`, `DLNAD_STATE`, `HOSTD_SOCKET` | `MIX_VIU_SERVICE`, `MIX_VIU_EXE`, `MIX_VIU_COMP_SERVICE`, `MIX_NODED_SERVICE`, `MIX_DLNAD_CONF`, `MIX_DLNAD_STATE`, `MIX_HOSTD_SOCKET` | no |
| build env `MIXOS_GIT_SHA(_FULL)`, `MIXOS_GIT_DIRTY`, `MIXOS_BUILD_TIME`, `MIXOS_BUILDINFO_MARKER` | `MIX_*` | compile time only |

Data paths by owner (the list desk4 moves data by; `<etc>`, `<var>`, `<run>`
are the tier roots: `/etc/mix`, `/var/lib/mix`, `/run/mix` for the system
tier, `~/.config/mix`, `~/.local/share/mix`, `$XDG_RUNTIME_DIR/mix` for a
user):

| Owner | Old | New | Gold reads |
|---|---|---|---|
| maild | `<var>/jmap/{mail.db,blobs,mds,spamlite}`, unit `StateDirectory=... mix/jmap` | `<var>/maild/jmap/...`, `StateDirectory=mix/maild` | no |
| maild | `<etc>/jmap.conf.mix` | `<etc>/maild/jmap.conf.mix` | no |
| webd | `<var>/web.db` (default without --db-path) | `<var>/webd/web.db` | no |
| editd | `<var>/edit/recovery`, `~/.local/state/mix/edit/recovery` | `.../editd/recovery` | no |
| settingsd | `<etc>/settings/<profile>` | `<etc>/settingsd/<profile>` | no (term/portald use the Bus) |
| viud | `<etc>/comp/xwayland-enabled.<socket>` | `<etc>/viud/xwayland-enabled.<socket>` | no |
| viud | `~/.config/compd/*.json`, `~/.local/state/compd`, `~/.local/share/compd`, `$XDG_RUNTIME_DIR/compd/<socket>.xwayland.env` | `~/.config/mix/viud`, `~/.local/state/mix/viud`, `~/.local/share/mix/viud`, `$XDG_RUNTIME_DIR/mix/viud` | no |
| every config::store user | `<etc>/<service>.conf.mix` | `<etc>/<service>/<service>.conf.mix` | lib only at gold (no caller in term/portald/inputd) |
| logging, noded | `<var>/log/<identity>.log`, `<var>/log/bus.log` | `~/.local/log/mix/<owner>/` or `/var/log/mix/<owner>/` (noded: `noded/bus.log`) | term, portald, inputd write their own logs under `/var/lib/mixos/log` (gold libs/logging) |
| apps | `~/.local/state/mix/apps/<app>`, `<var>/apps/<app>` | `~/.local/state/mix/<app>`, `<var>/<app>` | term keeps its gold home (`~/.local/state/mixos/apps/term`) |
| indexd | `~/.config/mix/indexd.conf.mix` | `~/.config/mix/indexd/config.conf.mix` | no |
| hostd | `/run/hostd/hostd.sock` | `/run/mix/hostd/hostd.sock` (unit, hostd, mixtrayd, Quoin/scenes clients) | no |
| viu | `$XDG_RUNTIME_DIR/viu-captures` | `$XDG_RUNTIME_DIR/mix/viu/captures` | no |
| inputd keymap | gold inputd writes `~/.config/mixos/inputd/keymap.json` | viud reads `~/.config/mix/inputd/keymap.json` | yes: gold inputd is the writer; copy or point it at the new file |

Already per owner, only the prefix changed: noded (`<etc>/noded`, `<var>/noded`,
`<run>/noded/bus.sock`), maild (`<var>/maild/{dkim,tls,rules}`,
`<etc>/maild/config.conf.mix`), webd (`<var>/webd/{session.key,acme}`), dnsd,
blobd, dlnad and indexd system paths, scenes (`~/.config/mix/scenes`,
`~/.local/share/mix/scenes`), macros (`~/.config/mix/macros`).

Directories (gold term, portald and inputd all resolve these through
libs/paths and libs/config; portald's unit reads the session env file):

| Old | New | Move data? |
|---|---|---|
| `/etc/mixos` (incl. `session/<vt>.env`, `node.conf.mix`) | `/etc/mix` | yes: copy config; gold keeps reading `/etc/mixos` |
| `/var/lib/mixos/<desk>` and `/var/lib/mixos/<svc>` (settings stores, noded, editd recovery, maild/jmap, webd, indexd, dnsd, dlnad, blobd) | `/var/lib/mix/...` | yes: move the stores |
| `/run/mixos`, `/tmp/mix-run`, `$XDG_RUNTIME_DIR/mixos` | `/run/mix`, `/tmp/mix-run`, `$XDG_RUNTIME_DIR/mix` | no (runtime) |
| `/var/cache/mixos`, `~/.cache/mixos` | `/var/cache/mix`, `~/.cache/mix` | no |
| `~/.config/mixos` (node.conf.mix, `<service>.conf.mix`, scenes, macros) | `~/.config/mix` | yes |
| `~/.local/share/mixos` (apps data, scenes lib, assets) | `~/.local/share/mix` | yes |
| `~/.config/compd` (settings.json, preferences.json, keybinding.json, experimental.json) | `~/.config/mix/viud` | yes |
| `~/.local/state/compd` (world document store), `~/.local/share/compd` (shaders) | `~/.local/state/mix/viud`, `~/.local/share/mix/viud` | yes |
| `$XDG_RUNTIME_DIR/compd/<socket>.xwayland.env` (launcher read `mixos-comp/`) | `$XDG_RUNTIME_DIR/mix/viud/<socket>.xwayland.env` | no |
| `/usr/local/lib/mixos` | `/usr/local/lib/mix` | if present |

## What keeps an old name (rg -i mixos, compd)

1. Host interpreter paths: `/opt/mixos/bin/mix` in tools/*.mix, apps/*/gate.mix,
   capture-build.mix, apps/gallery/README.md and Mix-script test defaults
   (MIX_TEST_MIX_EXE falls back to the host's gold mix on CBC).
2. Wire, data and protocol identifiers (Mark's call whether to break):
   - Bus service `comp` and `comp.*` commands.
   - App ids `dev.mixos.*` (Wayland app_id, desktop ids, icon names; gold term uses `dev.mixos.term`).
   - Desktop name `MixOS` (XDG_CURRENT_DESKTOP, CLIENT_SIDE_DESKTOPS) and `MIXOS` (OnlyShowIn/NotShowIn).
   - Theme, scheme and window-style value `mixos` (`Scheme::Mixos`, `ChromeStyle::Mixos`, `WindowStyle::Mixos`, `.mixos()`/`mixos_only` registry flag, `view.theme.mixos` action ids, label "MixOS"), icon theme `icons/mixos`: stored settings values shared with gold term.
   - HTTP headers `X-Mixos-*`/`x-mixos-*` (blobd lanes, maild), cookies `mixos_session`, `mixos_csrf`, `mixos_flash`, `mixos_login_pending`, URL path `/_mixos/assets/`.
   - Prometheus metrics `mixos_process_*`, `mixos_stats_*`.
   - Native-session transcript labels `mixos.native-session.proof/allocate`; mesh admission domain `mixos-d2-admit-v1`; binary marker `MIXOS-BUILDINFO:1`.
   - Data formats `mixos.static-assets.v1`, `mixos-mds-set-export`; DAV properties `mixos:rawicalendar`/`mixos:rawvcard`; service binary name `mixos-jmap` in service_info.
   - System group `mixos-tls`; CA names `MixOS Internal Root/Issuing CA`.
   - Software groups `mixos-desktop`/`mixos-server` (app manifests); webd vhost variable `$mixos_delegation`.
3. Provenance and gold: provenance/, vendor/ patch markers (`compd:` notes
   attribute MixOS patches), docs/ports/ evidence, CHANGELOGs and
   docs/mix/changelog, tools/ lab and oracle scripts, the gold oracle names
   `compd-oracle`/`COMPD_INPROC_TEST_BINARY`, references to MixOS 8779f33 and
   github.com/markc/mixos, `~/Projects/mixos`, gold crate and daemon names in
   comments and the Mix manual (`mixos-lib-*`, `mixos-mds`, `mixos-editd`,
   `mixos-webd`, `mixos-dnsd`, `mixos-claude`, `mixos-mcp`, `mixos_maild` log
   targets, `mixos-comp` as the manual's hyphenated-command example, generated
   site `mixos-build` meta tags), the brand logo file name, domains
   mixos.nexus and mixos.au, and every mixos.dev URL and the mc@mixos.dev
   address (copyright lines; the manual's links: mixos.dev/, /mix/, /maild/,
   /dlnad/, /indexd/, /webd/, /compd/ and the per-page manual URLs).
4. Not renamed, outside the three categories (about 280 hits): lowercase
   mentions of the `mixos` theme in comments and test names (the theme value
   itself is category 2), a few local identifiers (`mixos` field in
   mix-shell, `mixos_dbkey_test`). Cosmetic; a follow-up can take them.
   `rg -i compd` outside provenance, vendor, tools and docs/ports: only the
   gold oracle (`COMPD_INPROC_TEST_BINARY`), the `provenance/compd-oracle`
   include paths and the viud README naming gold compd.

Other decisions for Mark:
- User-visible app names say MixViu ("MixViu Editor"): a Mark-mandated
  difference from the gold apps.
- Category 2 wire names stay until one deliberate PR after desk4's gold
  binaries are gone (Mark).
- Mix-script tests and parity gates that run under the host's gold mix
  (`/opt/mixos/bin/mix`) now see MIX_* names and the `mix_path` builtin, which
  gold mix does not know; they need this tree's mix (indexd's Bus test now
  prefers `target/debug/mix`; others, such as dlnad's limits_test, still
  default to gold).
- Lab scripts (tools/bulk_native, p2_rehearsal, p2 furniture) still set the
  old VIU_COMP_SERVICE / COMPD_NATIVE_TEST_BINARY names.
- The furniture comparison (`COMPD_INPROC_TEST_BINARY`) still names the gold
  oracle; lab scripts pass the old `COMPD_NATIVE_TEST_BINARY`, now
  `VIUD_NATIVE_TEST_BINARY` in the test.

## Release set and apps.tar

The full cut-over gate (`tools/cutover_gate.mix gate`) ends by writing
`/build/cutover-artifacts/<sha>/{mix,mixd,viu,viud,apps.tar,SHA256SUMS}`:
the four release engines, then `tools/apps_bundle.mix` run with the release
mix, then SHA256SUMS over all five. `cutover_gate.mix bundle` is the light
check of the same step.

apps.tar is laid out as installed under `/opt/mix` (root 0:0, commit mtime,
sorted), with `share/mixviu/BUNDLE.conf.mix` listing every file's size and
SHA-256:

| Path | What |
|---|---|
| `share/mixviu/apps/<app>/` | the Mix apps backdrop, ced, dopus, email, gallery, monitor, prefs, quoin, screenshot, workshop (runtime files: no Rust, tests or gate drivers; gallery-reference is a Rust build and is left out) |
| `share/mixviu/libraries/viu/mix/` | the Viu Mix library the apps require (`../../libraries/...`) |
| `share/mixviu/units/` | unit templates with `@{prefix}`/`@{etc}`/`@{share}` placeholders: viud@, viud-headless@, apps@, scenes@, mix-headless.slice, clipd, clipd@, notifyd@, exportd-*, dlnad |
| `share/mixviu/{LICENSE-APACHE,LICENSE-MIT,NOTICE}` | licences |
| `share/scenes/`, `share/viud/scripts/` | scenes and the viud Mix scripts (apps@, scenes@) |
| `lib/{clipd,notifyd,exportd,dlnad}/` | the Mix services, where their units start them |
| `share/icons/`, `share/assets/`, `share/wallpapers/`, `share/layout/`, `share/daylight/` | icon theme (built by its offline installer), brand marks, the pinned font set `2026-10-14-core-5` with `current`, 12 wallpapers, layout and daylight data |

Light run (cachyos, fa950b2, debug mix): 360 entries, 45,785,739 bytes of
files, tar 46,161,920 bytes; top: share/mixviu 194, share/icons 49,
share/scenes 45, share/assets 22, share/wallpapers 15, lib/exportd 10,
share/viud 7, lib/dlnad 5, lib/clipd 4, share/layout 4, lib/notifyd 2,
share/daylight 2. Retired lab hosts (viu-client, viu-dopus,
viu-dopus-client) are not in it.

The migration's `extract_bundle` still expects the old repo-relative layout
(`apps/`, `libraries/viu/mix`, `share/scenes`, `engines/viud/scripts` at the
tar root): it needs to unpack this tar at `/opt/mix` instead, and it can drop
its `copy_tree` of gold `icons`, `wallpapers`, `brand` and `assets`.

Asset store: `tools/apps_bundle.mix` rebuilds it from the pins. Fonts are
downloaded by `share/assets/install.mix` from each pinned upstream URL at its
revision, checked by BLAKE3 and SHA-256. Wallpapers download from the pinned
GitHub release `assets-wallpapers-2026-10-14` (recorded in
`share/wallpapers/manifest.conf.mix`; asset name = path with `/` as `--`),
following at most three redirects and stored only when size and SHA-256 match
the pin; the URL is never trusted. The gold install, read only, is the last
resort for both. Proof at aa1ee08, empty scratch store (cachyos
20261107-213836-eb7e5600): 17 fonts downloaded, 12 wallpapers from the
release, none from gold.

Current apps.tar at aa1ee08: 387 entries, 45,809,153 bytes of files, tar
46,213,120 bytes; share/mixviu 221 (apps, Viu Mix library, 39 unit templates
and snippets, licences, manifest), share/icons 49, share/scenes 45,
share/assets 22, share/wallpapers 15, lib/exportd 10, share/viud 7,
lib/dlnad 5, lib/clipd 4, share/layout 4, lib/notifyd 2, share/daylight 2.

Unit templates: apps.tar carries every unit under engines/ and services/
(viud@, viud-headless@, viu-host@, apps@, scenes@, mix-headless.slice, noded,
noded@, settingsd@, editd@, monitord, dnsd, blobd, hostd + socket, mixtrayd,
maild + sockets, webd, indexd, dlnad, clipd, clipd@, notifyd@, exportd-*) and
the sysusers/tmpfiles snippets, owner-prefixed. Desk-specific drop-ins and
session@/seatd@/session-dbus@ (written by the desk image) are not in it,
nor pipewire@/wireplumber@ (desk audio). Added since: monitord@ (per-session
history in mix/monitord/%i), backdrop@ and viu-gallery@ (the gallery app
ships in apps.tar); app unit templates come from apps/*/units. At 37e7f3b:
390 entries, share/mixviu 224, 45,810,831 bytes (cachyos
20261107-221523-0bbe5b3a; fonts upstream, wallpapers from the release).

Directories per owner (37e7f3b): maild@ is one instance per session broker,
so each keeps its own state: StateDirectory/RuntimeDirectory mix/maild/%i,
MIX_VAR/HOME/WorkingDirectory %S/mix/maild/%i. maild.service RuntimeDirectory
mix/maild; maild sysusers home @{var}/maild. hostd StateDirectory mix/hostd and
code default /var/lib/mix/hostd. monitord StateDirectory mix/monitord, code
default /var/lib/mix/monitord, sysusers home @{var}/monitord. dnsd tmpfiles
creates @{etc}/dnsd (was @{prefix}/etc/dnsd). All other Directory= and
sysusers homes were already mix/<owner>; noded@ and viud keep their
session-scoped mix/session/%i/... runtime dirs; mixos-tls stays.
viud@ and viud-headless@ no longer pass `--scene-service shell`: viud parses
the flag (name check only) and its scene stub ignores it, because the shell
furniture now runs in viu-host@ (`viu --host`), a new template in
engines/viu/units.

quoin: `apps/volume-lab` is not an app bundle: the runnable app is
`apps/volume-lab/volume` (contract.json with behaviour viu.mix, scene.mix). It
needs no launch arguments or host initialise. The migration's generated
launcher `viu run /opt/mix/share/mixviu/apps/volume-lab` must become
`viu run /opt/mix/share/mixviu/apps/volume-lab/volume`.

exportd and real homes: GTK/Qt already refused to write when XDG_CONFIG_HOME
is (or defaults to) the home's own .config unless MIX_EXPORTD_GTKQT_HOME=1.
COSMIC and KDE wrote there by default; now they run dry and report
`home_guard` unless MIX_EXPORTD_COSMIC_HOME=1 / MIX_EXPORTD_KDE_HOME=1 (an
explicit MIX_EXPORTD_COSMIC_ROOT or XDG_CONFIG_HOME elsewhere is allowed). KDE
state moved to ~/.local/state/mix/exportd/kde. Lint with this tree's mix: 0
errors (cachyos 20261107-213810-b6eb81ec); unit tests pass
(20261107-213742-29994673). Registry: 200 names.

## Ported Mix services

clipd, notifyd and exportd come from mixos 8779f33 (verbatim import, then
renames; provenance `provenance/gold-mix-services.md`). New variables are
registered: MIX_CLIPD_CONFIG, MIX_CLIPD_MODE, MIX_CLIPD_SESSION,
MIX_CLIPD_TARGET, MIX_CLIPD_WATCH_TOKEN, MIX_EXPORTD_STATE,
MIX_EXPORTD_COSMIC_ROOT, MIX_EXPORTD_GTKQT_HOME, MIX_EXPORTD_DRY_RUN,
MIX_EXPORTD_KDE_STATE, MIX_EXPORTD_KDE_DRY, MIX_EXPORTD_KDE_RECONFIGURE,
MIX_EXPORTD_KDE_KWRITECONFIG, MIX_EXPORTD_KDE_BUSCTL, MIX_NOTIFYD_STATE, MIX_SETTINGS_SERVICE, MIX_TEST_BLOBD_EXE,
MIX_TEST_EXPORTD_REAL_KDE. clipd's config moves to
`<etc>/clipd/clipboard.conf.mix`; units run as `User=mix`. Exported KDE/Qt
names (MixOS-Dark/Light, MixOSDark/Light, the `[MixOS]` kdeglobals stamp) stay:
they name files gold exportd already wrote. notifyd's default app name is
MixViu. tests/cbc_checks.mix (a MixOS crate check) is not imported.

Lint with this tree's mix (`cutover_gate.mix lint`, cachyos
20261107-211822-f6329f7d): 34 files, 0 errors, 0 warnings (kde_test 2 notes).
Unit tests with this tree's mix (`cutover_gate.mix mixtest`, cachyos
20261107-212515-4a70d7c5): clipboard_test, notify_test, gtkqt_test,
cosmic_test, busctl_test pass; kde_test passes its unit checks and then
needs a built noded for its Bus part (not run). The gold mix cannot lint or
run them (no `mix_path`).

## Shared assets

`provenance/shared-assets.md`: the 87 share files imported from mixos 8779f33
(icons: Lucide ISC plus MIT OR Apache-2.0 caption glyphs; brand: MIT OR
Apache-2.0; assets installer and pin manifest; layout; daylight from tzdb,
public domain), the 17 pinned files of font set 2026-10-14-core-5 (all OFL-1.1:
Inter, JetBrains Mono, Noto Sans, Noto Serif, Quicksand, Noto Color Emoji,
with their licence texts; each with upstream URL, revision and hashes) and the 12 wallpapers: 6 Wikimedia Commons photos
(CC0, CC BY 4.0 x2, CC BY-SA 4.0 x3) and 6 generated (MIT OR Apache-2.0).
The JPEGs and fonts are not committed. The photos' CREDITS still asks for the
Nunivak author to be named before a public release.

## Gate status

No full gate on cbc2/cbc3 for this round (Mark: light worker only until the
tree is final; parity has priority). Last full run, cutover-gate-2d7e1c7:
10,514 passed, 3 failed, 90 ignored; one failure since fixed (indexd), two
inherited from lab 9bcfeee (workshop_frame; the lab round is fixing them).

Light checks on cachyos at 34a8d52 (one or two crates per job):
- config 61 passed, 1 failed; editd + daemon 144/0/2; settingsd + hostd 165/0/6;
  maild 1,651/0/4; webd 490/0/0; noded + mixtrayd 277/0/2;
  policy-host + comp-model 106/0/0; lib-viu 105 passed, 3 failed;
  viu + viud 118/0/6; clippy lib-viu + viud clean.
- The config and lib-viu failures (non-UTF-8 names, links) are the cachyos
  filesystem: the same config and viu file_data tests fail at base 4755e9e
  on cachyos (20261107-185148-df07c276, 20261107-185233-ce550d01) and pass on
  cbc2 in every full gate. raster_decode's folder test (merged lab code) is
  the same kind.
- At 7fddb05: rustfmt clean (20261107-185355-f8c3d8cd); logging + indexd
  green (20261107-185418-c5fe3256).

Branch cutover/four-engines; head is this last.md commit (source 7fddb05); gate: full run held for parity, light checks green apart from cachyos filesystem cases; CBC last full cutover-gate-2d7e1c7, light 20261107-185355-f8c3d8cd, 20261107-185418-c5fe3256, earlier cutover-gate-5d1ec61, cutover-gate-43de632, cutover-removed-4755e9e, 20261107-111516-437fd7b1.


## Mix hygiene pre-PR change (2026-11-07)

The four local Cargo shortcuts now refuse before touching source or installed
binaries. CLI help and the manual point to the CI recipe and CBC. All four
native-session fault variables use the explicit debug-only test-hooks feature;
release builds reject it. The native PTY target requires that feature and CI
enables it. No dependencies or deployment changed.

Source 805db4c61b62812999081e9bb7eb39ab677fd7ab passed fmt, env and
mix-hygiene on CBC job 20261107-204950-4537e2c4: 29 tests passed, expected
release-feature rejection, 176 env names and zero registry problems. Full
worker log was read. An earlier base implementation also passed on cbc3, job
20261107-204601-09d3a4fd-a1. Intermediate rustfmt and help-test findings were
fixed. Full workspace all gate and independent review remain unrun.

Round 2 (source 53357d4): the full test gate failed only because
tests/owned_editor_pty.rs compiles editor/runtime.rs by #[path] and lacked the
new crate::test_hooks module (E0433). The release-profile panic in that log was
the intended negative check passing. Fix: the test includes test_hooks.rs. The
hooks now link a marker; mix-hygiene checks the hook build carries it and the
ordinary build does not, and the new release-mix stage refuses a release mix
that carries it. Full test gate 20261107-210937-ccb133a8-a1 (cbc3) BUILD_RC=0,
10,560 passed / 0 failed / 90 ignored; release-mix 20261107-214025-5234a7b5-a1
(cbc3) BUILD_RC=0, marker absent.

PR.md contains the review draft; no PR opened under the binding brief.

Branch fix/mix-hygiene; head SHA: this evidence commit (tested source 53357d480966881475a04647498bdf8723b724ce); gate: full test GREEN, release-mix GREEN (earlier fmt/env/mix-hygiene GREEN at 805db4c); CBC jobs 20261107-210937-ccb133a8-a1, 20261107-214025-5234a7b5-a1, 20261107-204950-4537e2c4; Mark: wire release-mix into the gold/release recipe, independent review and human approval before merge.

## Main cut-over record
Final lab merge (2026-11-08): origin/lab/gold-parity at 01912877 (eight accepted apps; Email 70ea474, Dopus c66fe3c; runtime e1bbea9). Conflicts took the lab side of libraries/viu (launch, app_host, chooser_host, Cargo.toml) with lib-viu/lib-mix put back, the Dopus Rust crates deleted as the lab retired them, and the lab Cargo.lock with the four-engine renames reapplied. Renames applied to the merged files: package names in the new Dopus/Email gate scripts, MIXOS_INSTANCE -> MIX_INSTANCE, MIXOS_COMP_SERVICE -> MIX_COMP_SERVICE, VIU_SERVICE -> MIX_VIU_SERVICE, /opt/mix, Dopus home ~/.local/state/mix/dopus, MixViu prose in the Dopus and Backdrop contracts. No new registry names. Kept as wire/data: theme value mixos and its MixOS label, the email token label mixos-email. apps.tar at 7bde4c0: 428 entries (share/mixviu 262), 46,138,946 bytes; fonts upstream, wallpapers from the release, none from gold (cachyos 20261108-105441-54343276). Light checks: fmt clean; cargo tree --locked -p viud ok; lib-viu --tests 263 passed, 3 failed (the known cachyos filesystem cases); dopus_contract, dopus_bundle, backdrop, app_ports pass at db16141.

Branch cutover/four-engines; head is this last.md commit (source db16141); full gate: the coordinator runs it on infra/pr-ci; CBC light 20261108-105611-7534e7c2, 20261108-104904-e4317f0f, 20261108-104841-c85e2532, 20261108-105441-54343276.

Branch fix/email-desk4-tree-table; tested commit 1e38766b9a8dfcc260a8f377162aaaa3c37ec19d; Email legacy account-tree startup FIXED; hot-apply apps/email/mail.mix + apps/email/surface.mix to /opt/mix/share/mixviu/apps/email/; CBC email-desk4-scope-r4-a1 rc0 (17/17), email-desk4-native-r5 rc0 (real maild before/after); no desk4/main/infra/pr-ci changes.


## PR #7 main synchronisation (2026-11-08)

Merged main 60cb127 into fix/mix-hygiene without rebasing or force-pushing.
The only conflict was last.md; both evidence records were preserved. Main
cut-over rules and the gated hooks coexist; the env registry has 194 names
and zero problems.

CBC 20261108-153105-ab2990de-a1 resolved Cargo.lock without changes and
regenerated the lint baseline using the merged tree mix. Counts only shrink
against main: 182 to 152 files and 1265 to 1211 diagnostics.

CBC 20261108-153201-d97092aa-a1 at source
7cb24b0b596befc40f05c55893e39124a5b463f7 passed fmt, env, mix-lint, full test
and release-mix (BUILD_RC=0). All 611 test result summaries have zero failures.
Both debug artefact checks and the release artefact check passed; the release
feature guard refused fault injection. The full 904311-byte saved log was read.

The historical scanner false positive and two notes quoting it have exact
fingerprint exceptions. Main scanner rules are unchanged. Pinned gitleaks
8.30.1 passes a full-history redacted scan. Only scanner exceptions and docs
changed after the tested Rust tree. PR.md and PR #7 contain current evidence.
Required GitHub checks are ci-ok/security-ok, verified separately on the final pushed head;
independent review and human approval are separate requirements.

Branch fix/mix-hygiene; head: this evidence commit (tested Rust tree 7cb24b0b596befc40f05c55893e39124a5b463f7); gate: fmt/env/mix-lint/test/release-mix GREEN; CBC jobs 20261108-153105-ab2990de-a1, 20261108-153201-d97092aa-a1; PR #7.


## Final CI fixture repair (2026-11-08)

GitHub run 42453072066 exposed a readiness/boot trace interleaving in the
existing serve_reload_lifecycle grace fixture. Formatting directly into a
File wrote fragments; the child now formats the whole readiness record
before write_all. All assertions and timeouts remain unchanged.

Source 2c80c0bf0ff115ce5d0bfa1985cdf062d3a8424b passed formatting and env
checks. CBC 20261108-161926-f3889961 built a fresh sibling mixd and passed
both lifecycle configurations (7 tests each; existing subprocess helper
ignored), plus ten further runs of the affected grace case. The full saved
log was read. The first focused CBC job 20261108-161651-31de42e8 did not
preserve Cargo failure details; the corrected job retained full output.
Production code is unchanged from the green full gate at 7cb24b0b.

Branch fix/mix-hygiene; head: this evidence commit (full tested tree 7cb24b0b596befc40f05c55893e39124a5b463f7, tested fixture 2c80c0bf0ff115ce5d0bfa1985cdf062d3a8424b); gate: fmt/env/mix-lint/test/release-mix GREEN plus lifecycle repeat GREEN; CBC jobs 20261108-153105-ab2990de-a1, 20261108-153201-d97092aa-a1, 20261108-161926-f3889961; PR #7.
## P1 rebase onto merged P0 — 2026-11-09

This receipt supersedes the original P0 dependency/base above.

- New base: `cd85e2a3024f27bd997f8787e50d59d349b3ab67` (`origin/main`, P0 squash PR #41).
- Old P1 head: `5cb266763fd5b4ea811c10b9284e0b494dfcfbca`; retained locally as `backup/quoin-p1-before-main-20261109`.
- Replayed only `0780b2ea..feat/quoin-p1-dev-loop` onto main. All eight commits rebased automatically, with no conflict stops. Reviewed the resulting main diff: only P1 files and this session's harness/receipt changes remain.
- P0's name-limit/header-only panel text, registry draft statuses, backdrop.view.list contract and viu registry test owner filter are preserved. Neither document required manual conflict resolution; the auto-merge retained P0's reviewed text plus the P1 additions.
- New rebased source head tested and pushed: `75255585405b5ab511f59032dbda47b48f88e45b`. The subsequent receipt commit changes only last.md; its hash is the final branch head returned in the session hand-off.
- Bot push used `gh_agent.mix git -C WORKTREE push --force-with-lease origin feat/quoin-p1-dev-loop`, on this branch only. No PR opened or reopened; Opus handles the bot PR.

Harness work:

- Separate commit `75255585`: scenes_io_test.mix now uses `mixos_path("prefix")` instead of obsolete `mix_path("prefix")`. Local syntax/lint pass. The default integration invocation now gets past that builtin and reaches the already-known private broker readiness failure; it still does not pass end to end with the installed noded.
- Registry source helper deferred as larger than a small fix. The original MixOS checker is 390 lines and requires another omitted 173-line command_registry_embed.mix. It traverses the old apps/services/cli/libs layout; MixViu needs engines/libraries ownership coverage, embedding-context/provenance assessment and fixture tests. Merely copying the missing file would leave another missing require and incomplete source coverage. The gate remains blocked at its missing require, with no bypass or stub added.

CBC, exact source `75255585405b5ab511f59032dbda47b48f88e45b`, cbc2:

| Job | BUILD_RC | Outcome |
|---|---|---|
| 20261109-174228-83fb5387-a1 | 1 | 400 loader assertions and three P1 script lint checks passed; stopped at the worker's older linter not knowing mixos_path in the harness |
| 20261109-174321-7bf300df-a1 | 0 | 400 loader assertions; P1 script lint; harness lint with --allow-function mixos_path for the older worker binary; cargo fmt -p viu -- --check; cargo test -p viu --lib (101 passed, 5 ignored); cargo check -p viu --bin viu --locked |

Final saved log was read in full: `20261109-174321-cbc2-75255585405b-20261109-174321-7bf300df-a1-ef6cb226.log` under the maintainer's CBC cache. Harness lint has one existing literal-$tmp note, zero errors/warnings. Other P1 lint checks have zero errors/warnings/notes. Existing dependency warnings remain in the Rust log.

No desk4, VT or /opt changes. No additional runtime.rs hunks beyond those listed in the original P1 hand-off.

## PR #46 review round 1 — 2026-11-09

Addressed GLM-5.3/DeepSeek v4-pro items on the existing branch. Source head tested: `44a81cfbe5a6d1f2f13c8386634b17500a84bce2`. The following receipt/development-staging clarification commit changes only documentation; the bot push's final head is reported in the session hand-off.

1. Preview trust: removed cwd ancestor search and production CARGO_MANIFEST_DIR fallback. Only the explicit VIU_PREVIEW_VALIDATOR override or `<running-binary-prefix>/share/viud/scripts/scene_preview.mix` is selected. Mix comes from that prefix's bin/mix, with its share directory supplied for catalogue discovery. Confirmed packaging in tools/apps_bundle.mix. The planted-ancestor regression executes from a downloaded-scene cwd and proves only the trusted validator runs; the planted marker is absent. Build-machine paths/native test interpreter references remain in cfg(test) fixtures only.
2. In-flight edits: retain the signature captured before preflight, never a new post-commit signature. Compare after commit and repeat on a difference, for filesystem and explicit library/all reloads. Four immediate passes bound continuous writes; a coalesced local task defers further retries. Tests inject a helper write in the batch host call after preflight, proving a second full commit on both entry paths. Singleton reload does not acknowledge a library signature.
3. Mixed batches/overflow: group refusal retains a fence for its dependants while filesystem processing continues for unrelated scenes and overflow rescans. The refusal response survives reconciliation. Post-commit spawn errors also leave the rest of the batch eligible. Tests cover all three; a singleton cannot bypass the standing shared-library refusal. Disable/removal still reaches normal reconciliation.
4. Strip diagnostic: inventory now explicitly publishes load_diagnostic (diagnostic already uses failure(), which prefers that field). errors() prefers that refusal and its nested upstream/helper diagnostic; an unrelated lint problem cannot override it. The integration regression drives the real Mix loader transitions through its notify adapter, reads the refused-lib publication, draws last-good plus the strip with egui and asserts the refusal text is painted without advancing the view revision. Host/process RPCs are fixture adapters, not a live desk.
5. Mount rollback: SceneMount is an immutable input receipt inside acceptance, not a mutable external mount object. Page/dialog/toast effects reside in SceneStore, including in a batch's cloned store. Internal APIs now take shared references, preventing receipt mutation. A two-scene batch with a newly mounted page first and a refused second scene proves pages, caller receipt, removals and notices stay unchanged.
6. Watch lifecycle: templates and custom libraries are watched recursively from an existing parent; custom lib watching is established even when the lib directory does not yet exist. The root parent watch is recursive too. Native tests replace the catalogue, edit its new inode again, create a late lib, then edit it again; each remains observable.
7. Cost/coalescing: path-prefix/ancestor/overflow checks precede library hashing. Pure scene edits do not hash helpers unless retrying a standing library refusal. Native filesystem batching plus unchanged-signature suppression coalesce duplicate saves.
8. Digests: accepted helpers use the same complete bounded SHA-256 read for signatures and check-cache entries. bounded_read already refuses cap+1 bytes; no oversized prefix is accepted. Fingerprints mark oversize for refusal, and the test includes a >256 KiB tail.
9. Symlinks: both passes use no-follow checks, including library roots. Fingerprinting marks a refused link and preflight rejects it; neither silently follows or skips it as valid. Regression covers link refusal.
10. Replies: docs/scene-documents.md and the registry document all three results, including rc10 with new revisions for post-commit behaviour faults. A spawn-failure regression asserts both revisions and the runtime diagnostic.
11. Strip routing/text: edge and toast clicks use the configured SCENES_SERVICE. The passive preview/error parser has no editor-action hint. Strip text and tooltip shorten absolute path tokens to basenames; covered by the surface regression.
12. Preview validation/transport: preview and install staging call the real check_library. Checker/lint output remains captured inside scenes_io.check; only the final scene source is printed by the validator. Preview regression uses a helper containing print (never executed or leaked), then a broken helper and asserts last-good remains.
13. Reload arguments/authority: scene scope requires a string name; lib/all reject name even if nil. Tests cover missing, numeric and inapplicable names; the request error path also guards its name lookup against non-string input. PR and scene docs explicitly defer caller authority to the comp-identity work's later steps.
14. Fan-out scope: scene documents use strict-data fence parsing and cannot call require (libraries/strict/src/lib.rs and libraries/scene/src/envelope.rs). Behaviour-only dependency selection is therefore valid; computed/transitive behaviour imports still use the conservative all-enabled-behaviour set. Documented in the scene spec.

CBC receipts (Rust only on CBC; formatting diff restored in its own worker job and applied locally):

| Job | Source | BUILD_RC | Result |
|---|---|---|---|
| 20261109-180656-91382df9-a1 | e6d913e6 | 101 | New surface test dropped egui texture deltas; test harness cleanup corrected. Other Rust tests passed. |
| 20261109-180846-e9c19d39-a1 | 8003928c | 0 | 419 loader assertions, lint, 104 host tests (5 ignored), binary check; CBC formatting diff applied locally. |
| 20261109-181135-b1281ec5-a1 | 44a81cfb | 0 | 420 loader assertions; scenes.mix/lib/scenes.mix/scene_preview.mix lint: zero errors/warnings/notes; cargo fmt -p viu -- --check; cargo test -p viu --lib: 104 passed, 5 ignored; cargo check -p viu --bin viu --locked. |

Final saved log read in full: `20261109-181135-cbc2-44a81cfbe5a6-20261109-181135-b1281ec5-a1-b546e3ff.log`. Existing dependency warnings remain; no new source warning was reported. git diff --check passes. No desk4, VT, /opt or production changes. No added runtime.rs work beyond the diagnostic service routing/hint hunks; P2 comp subscription/binding work remains untouched. Bot pushes use the existing feat/quoin-p1-dev-loop branch; no new PR.

---

Existing GPU acceptance targets remain explicitly ignored. No local Rust,
deployment, desk/VT or installed-prefix changes. All pushes used the bot.


## PR #7 current-main settlement (2026-11-10)

Validated head/source SHA: `9ad45200302f610533b701fd8ac9762e932cc88f`.
Merged `origin/main` at `d17f7caab647954da3d2b05a7aa552e4e02fc461`
into the shared `fix/mix-hygiene` branch; no rebase or force push.
The final evidence commit changes only PR.md and last.md after that source.

Conflicts and resolutions:

- `PR.md`: retained the hygiene description, then refreshed its current evidence;
  the main version described a different security PR.
- `last.md`: retained both historical evidence records.
- `engines/mix/tests/serve_reload_lifecycle.rs`: retained the main helper and
  regression test. They already implement the complete-record append fix.
- `tools/ci/mix-lint-baseline.json`: retained the main baseline unchanged.
  The merged gate reports zero regressions; no baseline regeneration needed.

Cargo.lock matches main exactly, including rusqlite 0.41.1 and
libsqlite3-sys 0.39.1. CBC `cargo update -w` and `cargo fmt --all` produced
no patch; the worker printed the diff and restored its checkout in the same job.

CBC receipts at the validated source:

- `20261110-065002-54b96c63-a1`, cbc2, **BUILD_RC=0**: lockfile resolution
  and rustfmt. Saved log read; no diff to apply.
- `20261110-065037-808eaae5-a1`, cbc2, **BUILD_RC=0**:
  `/opt/mixos/bin/mix tools/ci.mix all`. The complete 1,099,182-byte saved
  log was read and audited: 619 summaries, 10,681 passed, zero failed,
  100 ignored; 197 environment names, zero registry problems; 153 lint
  files, zero regressions. Clippy and the viud dependency check passed.
  The release-feature build failed at the intended profile guard, as required.
  The hook marker was present in the opt-in debug binary and absent in the
  ordinary debug binary. Screencopy client errors were passing refusal tests.
- `20261110-065038-039c7d15`, **BUILD_RC=75**: release check did not run;
  the queue wait expired. Saved log read. Requeued as a check with a longer wait.

- `20261110-072134-e547727e-a1`, cbc3, **BUILD_RC=1**: the wrapper
  hit the Mix default 30-second process timeout during Cargo compilation.
  No scan ran. Saved log read; corrected the wrapper with an explicit
  14,400-second timeout and inherited compiler output.

Release artefact verification: pending corrected CBC job
`20261110-073834-988e97f8`.

No desktop access, installation, production deployment or PR merge performed.
Opus retains independent review and the protected merge gate.
