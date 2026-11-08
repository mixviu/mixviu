## Summary

`mix lint` now recognises viu's native vocabulary only for files declaring
`-- host: viu`. A build-time table comes from the real runtime registration
sites, without adding GUI dependencies to Mix. Unknown names still fail;
host extensions also participate in collision checks and do not leak into
remote `ssh_mix` bodies.

Mix apps declare their host. Ced's sibling policy functions use explicit,
literal `require` bindings and namespaced calls; lazy imports preserve its
cyclic policy dependencies. Reusable macro policy is separate from the
standalone macro handler. Backdrop and Dopus handlers stay in their app
entries because `require` deliberately refuses handler-owning modules.
Dopus's comparison harness loads the same entry with automatic startup
disabled, then sends its existing initialise event.

A native test harness runs every app `*-test.mix` with real viu bindings,
including a disposable filesystem root for Dopus's argument-driven tests.

Newly exposed defect fixed: Workshop's open-in-Dopus job used the removed
`mixos_path()` builtin. It now resolves the installed executable with
`mix_path("prefix", "bin/dopus")`, returning `UNAVAILABLE` if it is absent.

Existing diagnostics outside
the app migration, including the webd examples/widgets and intentional
negative interpreter tests, remain in the ratchet.

The CBC-generated baseline shrinks from 828 to 527 diagnostics (301 removed).

| Code | Before | After |
| --- | ---: | ---: |
| MIX-E1102 | 728 | 433 |
| MIX-E1101 | 36 | 33 |
| MIX-W2401 | 50 | 47 |
| Other errors/warnings | 14 | 14 |

`apps/` has no error or dynamic-include diagnostic remaining. Its three
existing warnings (two discarded validation results and one unchecked send)
are unchanged. No baseline entry increases.

## Per-area changes and round-two review

- Lint vocabulary: generate names from viu's registration sites; an actual
  fully provisioned evaluator registry now round-trips against that list.
  A standalone lib-mix build warns clearly and omits the unavailable viu
  vocabulary instead of failing when sibling sources are absent.
- Macro-policy split: reusable functions live in `macro-policy.mix` while
  the standalone handler remains in `macros.mix`. A Ced worker test now
  exercises catalogue open, New and Close through `job.run`/`macro-panel`.
- Ced migration: literal, lazy `require` imports preserve cyclic policy
  calls; distinct `$ced_*` aliases avoid local-state collisions. SPDX is
  restored to the first line of its entry script.
- Backdrop inline: handler-owning runtime stays in the entry's original
  startup error boundary. The duplicated inner metadata comments are gone.
- Dopus entry: the bundle's injected argument list still auto-starts it;
  the legacy comparison host explicitly sends `dopus.initialise` when it
  deliberately injects nil. Tests require a model before the bundle's first
  command and readiness after calling the production comparison startup
  helper (`mix_start::initialise`), not a copied test sender. Only one version
  header remains.
- Workshop executable lookup: replace the removed `mixos_path()` call with
  the existing `mix_path("prefix", "bin/dopus")` contract. The reviewed
  `which()` lookup is gone; launch does not depend on session PATH.
- Baseline regeneration: CBC's own Mix generated 527 diagnostics versus 828
  originally; no entry grows. The 30 app scripts now each run in their own
  native test process, so write-once script argv cannot leak to another test.

### Dopus launcher audit

| Launcher | Startup contract |
| --- | --- |
| Generic `viu run` / `app_host::run` | `launch::load` calls `Bundle::load`; the latter always injects a list as `$app_args`, even with no arguments. The entry auto-starts. The preview load only reads metadata. |
| Generic launcher/test tools (`tools/bulk_native.mix`, `tools/p2_rehearsal.mix`) | Invoke `viu run` and inherit the same bundle contract. |
| Legacy comparison host, Mix mode (`apps/dopus/src/main.rs`, `src/mix_start.rs`) | Calls the shared, tested startup helper: inject nil, load `app.mix`, then immediately send `dopus.initialise` with `app_dir` and `profile` before creating the UI. Rust mode owns its own native initialisation. |
| Installed desktop entries and Workshop | Launch the installed prefix Dopus binary directly; they do not load `app.mix` with nil arguments. Their native entry initialises itself. Workshop now resolves that binary by install prefix. |
| systemd units | No Dopus launcher unit exists in the tree or the inspected system/user unit roots. No additional deferred Mix startup path was found. |

## Risk tier

Tier 1: Mix lint and viu runtime/app integration. Module isolation is the
main behavioural risk; the native Ced, Backdrop and Dopus tests exercise it.

## How tested

- Run: CBC Mix build, all 142 lint regression tests, complete `lib-viu` and
  `viu` test suites, and a fresh Workshop suite after its launcher fix.
  `tools/ci.mix fmt`, `env`, `mix-lint` and `mix-lint-baseline` pass with the
  tree's own interpreter. All 30 app Mix scripts pass in the native harness.
- Observed: native Ced editing/Bus/dialog/session tests, Backdrop transitions,
  Dopus row projection and the 2,000-row runtime specimen all pass. The lint
  ratchet reports zero regressions; the env registry reports zero problems.
- Not tested: the complete legacy Dopus Rust UI executable (not a workspace
  member); its actual startup helper is compiled and executed by the startup
  test. No interactive desktop, VT acceptance or production installation.
  Existing GPU/installed-icon tests remain ignored by their owning suites.

CBC receipts for the review follow-up (all rc 0):

- `20261107-214035-60ebc86a-a1` at `33c90a50`: standalone build-script probe
  with absent viu sources, live registry round-trip, Ced macro-panel jobs,
  startup contracts and 30 isolated app scripts.
- `20261107-214421-7710d9a8-a1` at `765938a3`: full focused gate — formatting,
  env (177 names, zero problems), committed lint ratchet, 142 lint tests,
  complete `lib-viu` and `viu` suites, including all 30 app scripts.
- `20261107-215253-e40d2bb3-a1` at `8072059a`: the final startup test calls
  the production comparison-host sender directly; Mix build, env and lint
  pass. CBC's formatting patch was applied locally.

The baseline was originally generated by
`20261107-210932-d3432f56-a1` at `300a5459`. This review follow-up leaves that
527-diagnostic file unchanged: zero lint regressions and zero changes.
## How to undo

Revert this branch's commits, including the regenerated baseline. No runtime
installation or external data migration is involved.

## Authorship and review

Author: OpenAI Codex, committed and pushed through the mixviu-agent bot.
GLM-5.3 and DeepSeek approved with nits. This follow-up addresses their seven
items; re-review of the new diff and human approval remain pending. No PR has
been opened, as required by the pre-cut-over brief.

## Main cut-over refresh (2026-11-08)

Merged origin/main without rewriting the open PR branch. Main retires the
Dopus Rust host; its updated Mix controller remains inline in app.mix.
Production launchers, including Workshop, now use the generic viu bundle
and native readiness/Open contract. The startup regression covers that real
launch plan plus explicit embedded initialisation. Retired Rust sources and
their helper stay removed; main's migrated raw-name/chooser tests remain.
The lockfile and lint baseline will be re-resolved on CBC before final gates.

PR: #6. Current merge validation is in progress.
