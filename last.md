# Four-engine layout for the cut-over: gate green

Branch `cutover/four-engines`, from `lab/gold-parity` 4755e9e, with
origin/lab/gold-parity merged in twice (through 6f77a75; no rebase, no force
push). Moves and edits are separate commits (7404a61 moves only).
MixViu now has exactly four engines, each its own package and binary: `mix`,
`mixd`, `viu`, `viud`. viud owns compd's sources and is egui-free by
construction; the compd package, `inproc-ui`, `mix-standalone` and the mix
personality table are gone. Full mapping: `provenance/four-engine-layout.md`.

## Old -> new

Packages and binaries:

| Old | New |
|---|---|
| package `system` (bins `mix`, `mix-standalone`), `engines/mix` | package `mix`, bin `mix` (shell profile only) |
| package `mix` (interpreter lib), `libraries/mix` | package `lib-mix`, lib name `mix` |
| package `viu` (Viu runtime lib), `libraries/viu` | package `lib-viu`, lib name `viu` |
| package `viu-engine` (called `viu-client` in older scripts), bin `viu`, lib `furniture`, `engines/viu` | package `viu`, bin `viu`, lib `furniture` |
| package `compd`, bin `compd`, `engines/compd` | removed; package/bin `viud`, `engines/viud` |
| package `viud` (built from `../compd/src`) | package `viud`, own `src/`, `build.rs`, `tests/` |
| packages `scene-host`, `overlays` | removed (in-process egui UI only) |
| `mix-shell`, `ui` | unchanged names |

Dependents use `mix = { package = "lib-mix" }` and `viu = { package = "lib-viu" }`,
so no `use` lines changed.

Paths: `engines/compd/{src,build.rs,tests,crates,i18n,layers.conf.mix,scripts,units,README.md,CHANGELOG.md}`
-> `engines/viud/...`; `engines/compd/crates/ui` -> `libraries/ui`;
`engines/mix-shell` -> `libraries/mix-shell`.

Scripts: `-p system` -> `-p mix`, `-p mix` -> `-p lib-mix`, `-p viu` -> `-p lib-viu`,
`-p viu-engine`/`viu-client` -> `-p viu` in every `tools/*.mix` and `apps/**/*.mix`.
New `tools/cutover_gate.mix` is the four-engine gate. Lab-only (note at the top:
needs gold compd from mixos 8779f33 for any compd/oracle step): p2_gate,
p2_rehearsal, p2_prepare, p2_furniture_debug, gate_furniture, gate_comp_split,
engine_split_smoke, engine_split_size_ablation, check_comp_profiles,
check_native_chrome, gold_parity_gate, integrated_gate, integrated_prepare,
app_ports_gate/prepare, email_list_gate, email_wire_prepare, bulk_prepare, and
docs/ports/parity-method.md.

## Gate (CBC, tools/cutover_gate.mix gate at 5d1ec61)

fmt, workspace clippy (excluding mix), mix clippy, `build -p mixd`,
`test -p mix`, `test --workspace --exclude mix`: all pass.
`cargo tree --locked -p viud -e normal`: 0 egui/egui-wgpu/egui-winit/toolkit lines.

Tests: 10,484 passed, 0 failed, 90 ignored. The base (4755e9e, integrated_gate
on CBC 20261107-111516-437fd7b1) was 10,735 passed, 0 failed, 100 ignored. The
difference of 251 passed and 10 ignored is exactly the removed code (counted at
base, CBC cutover-removed-4755e9e):

- scene-host unit tests: 232 passed, 10 ignored. The in-process egui scene host.
- scene-host tests/registry.rs: 3. Same crate.
- overlays unit test: 1. In-process capture/fps/notify overlays.
- compd bin unit tests: 13. The same `src/main.rs` tests still run once, as viud's.
- compd tests/bus_wake.rs: 1. Gated on `inproc-ui`.
- mix `src/main.rs` personality_name test: 1. argv[0] dispatch removed.
- Also removed, never compiled before: tests/personalities.rs (cfg on features
  that did not exist), mix-standalone (0 tests), tests/settings_test.mix
  (scene-host's settings fixture, not part of any gate).

Release sizes (bytes, base from /build/es-artifacts/4755e9e on cbc2):
mix 20,521,752 -> 20,518,872; mixd 65,896,480 -> 65,896,416;
viu 55,726,024 -> 55,722,568; viud 32,489,704 -> 32,489,144.

CBC: base 20261107-111516-437fd7b1 (cbc2); first run cutover-gate-b916abc
(cbc3, one failure: logging's own-target list lacked lib_mix/lib_viu, fixed in
5d1ec61); final cutover-gate-5d1ec61 (cbc2) PASS; removed-test count
cutover-removed-4755e9e (cbc3); logging check 20261107-124729-ab35c975.

## Open items for Mark

- `engines/viud/units/*.service` still start `@{prefix}/bin/compd` and are named
  `compd@`/`compd-headless@`. They are deployment units; renaming them changes
  how desks start, so they are left for an explicit decision.
- Runtime names keep compd: Bus service `comp`, config dir `compd`, `COMPD_*`
  variables, user-facing `compd:` messages, wlcs shim `libcompd_wlcs.so`.
- The furniture comparison test and p2 furniture gate need a compd-oracle
  binary; on this branch that comes only from gold compd (mixos 8779f33).
- docs/mix still describes MixOS paths (`libs/mix`, `cli/mix-shell`) outside the
  `-p system`/`cli/system` lines changed here.

Branch cutover/four-engines; head is this last.md commit (gated source 5d1ec61; later commits are docs-only lab merges and this file); gate PASS 10,484/0/90 vs base 10,735/0/100 less 251/10 removed; CBC cutover-gate-5d1ec61, cutover-gate-b916abc, 20261107-111516-437fd7b1, cutover-removed-4755e9e.
