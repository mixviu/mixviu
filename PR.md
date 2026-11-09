## Summary

`mix lint` recognises viu's native builtins for files declaring `-- host: viu`.
The vocabulary is generated from the runtime registrations without linking GUI
dependencies into Mix. Unknown names and collisions remain visible; local host
extensions do not leak into remote `ssh_mix` bodies. A live, fully provisioned
viu evaluator round-trips against the generated list. Standalone lib-mix builds
without sibling viu sources warn clearly and omit that unavailable vocabulary.

App policy uses literal `require()` imports and namespaced calls. Ced's lazy
imports preserve cyclic dependencies. Handler-owning code remains in the
Backdrop and Dopus entries, since `require` rejects handler-owning modules.
The native harness runs all 30 app Mix scripts in separate processes so
write-once script argv cannot leak between tests.

Merged main's cut-over (`60cb127`) into the open branch with a merge commit;
no rebase or force push. The four engines, native environment registry and
per-owner directories are retained. Main retires the Dopus Rust host and moves
production launchers onto generic viu. Its updated controller is now inline
in `app.mix`; the migrated chooser, raw-name and filesystem tests are retained.

## Per-area changes

- Lint vocabulary: generated registration table, host opt-in, collision/unknown
  name/remote-boundary regression tests and live registry equality.
- Macro-policy split: reusable policy in `macro-policy.mix`; standalone
  handlers in `macros.mix`. Ced's real worker test exercises panel Open, New
  and Close through `job.run` kind `macro-panel`.
- Ced migration: literal, lazy requires and distinct `$ced_*` aliases avoid
  sibling-name and local-state collisions. SPDX is first in its entry.
- Backdrop inline: retains the original startup error boundary and removes
  duplicated metadata comments.
- Dopus entry: main's complete generic-host controller and event surface stay
  intact. Bundle arguments auto-start it; explicit embedded loads can defer to
  `dopus.initialise`. The startup test requires an initial model and a working
  first command for both paths. The retired Rust sender stays removed.
- Workshop lookup: the original fix replaced the removed `mixos_path()` call
  with prefix resolution. After cut-over, both Ced and Dopus launch through
  the existing viu executable, wait for native readiness and receive exactly
  one Open command. No Dopus binary/PATH lookup remains.
- Baseline regeneration: the merged tree's own Mix regenerates it on CBC.
  Main has 1,265 diagnostics; the regenerated total and per-code comparison
  are recorded below. No per-file/code entry may grow.
- Security scan: three exact historical findings in `.gitleaksignore` cover
  terminal shortcut help and two notes quoting it in another open branch.
  These fingerprints already exist on main; this PR changes only their comment. These are confirmed
  text false positives; secret-detection rules and workflow coverage remain.

### Dopus launcher audit after cut-over

| Launcher | Startup contract |
| --- | --- |
| Generic `viu run` / `app_host::run` | `launch::load` runs the Mix launch policy, then `Bundle::load` injects the argument list and the entry auto-starts. The regression uses this actual production launch path. |
| Generic `viu-host@.service`, bundle launch entries and launcher tools | Invoke the generic host and inherit the same bundle startup contract. |
| Workshop | Starts the selected bundle using `viu_context().executable`, waits for `<app>.ping`, then sends `<app>.open` once. |
| Explicit embedded runtime | Loads the entry with nil arguments and sends `dopus.initialise` before the first command; the retained test requires readiness and a model. No production launcher relies on this deferred path after the Rust host's retirement. |

### Baseline against current main

| Code | Main | Merged branch |
| --- | ---: | ---: |
| MIX-E1101 | 146 | 145 |
| MIX-E1102 | 1051 | 647 |
| MIX-E1201 | 4 | 4 |
| MIX-E1401 | 2 | 2 |
| MIX-W2201 | 2 | 2 |
| MIX-W2302 | 5 | 5 |
| MIX-W2307 | 2 | 2 |
| MIX-W2401 | 53 | 49 |
| **Total** | **1265** | **856** |

409 diagnostics removed; no per-file/code regression. The 2026-11-10 refresh
merged main at d17f7caa. Its newer P1, place-attribute and icon-cache scripts
already use require imports; the volume-lab rename retains its host declaration.
CBC regenerated the baseline using this tree’s Mix: errors 1203 -> 798,
warnings 62 -> 58. Lock resolution and rustfmt produced no changes.


The pre-cut-over reduction was 828 to 527. Main subsequently added app and
service files. Remaining diagnostics inherited from main, including the newer
Email sibling modules, webd examples/widgets and intentional negative tests,
stay in the ratchet; this refresh does not broaden that migration.

### Per-file baseline provenance (review round)

Compared with main at d17f7caa. Counts below are per code, before → after.

| File | Entries reduced or removed | Cause |
| --- | --- | --- |
| `apps/backdrop/app.mix` | MIX-W2401: 1 → 0 | Host declaration (viu vocabulary); require resolution; source replaced by require/inlining |
| `apps/backdrop/decode-job.mix` | MIX-E1102: 3 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/backdrop/probe.mix` | MIX-E1102: 2 → 0 | Host declaration (viu vocabulary) |
| `apps/backdrop/runtime.mix` | MIX-E1102: 23 → 0 | File inlined into app.mix by this PR; require resolution and host declaration at entry |
| `apps/ced/actions.mix` | MIX-E1102: 35 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/ced/app.mix` | MIX-W2401: 1 → 0 | Host declaration (viu vocabulary); require resolution; source replaced by require/inlining |
| `apps/ced/bus.mix` | MIX-E1102: 42 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/ced/core.mix` | MIX-E1102: 25 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/ced/dialogs.mix` | MIX-E1102: 8 → 0 | Require resolution: sibling calls are namespaced |
| `apps/ced/job.mix` | MIX-W2401: 1 → 0 | Host declaration (viu vocabulary); require resolution; source replaced by require/inlining |
| `apps/ced/mail.mix` | MIX-E1102: 20 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/ced/session.mix` | MIX-E1102: 10 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/ced/ui.mix` | MIX-E1101: 1 → 0; MIX-E1102: 38 → 0 | Host declaration (viu vocabulary); require resolution; module-local viu_context binding replaces ambient ctx |
| `apps/dopus/app.mix` | MIX-W2401: 1 → 0 | Host declaration (viu vocabulary); require resolution; source replaced by require/inlining |
| `apps/dopus/chooser-engine.mix` | MIX-E1102: 1 → 0 | Host declaration (viu vocabulary) |
| `apps/dopus/chooser.mix` | MIX-E1102: 21 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/dopus/config.mix` | MIX-E1102: 1 → 0 | Host declaration (viu vocabulary) |
| `apps/dopus/core.mix` | MIX-E1102: 11 → 0 | Host declaration (viu vocabulary) |
| `apps/dopus/dopus.mix` | MIX-E1102: 16 → 0 | File inlined into app.mix by this PR; require resolution and host declaration at entry |
| `apps/dopus/effects.mix` | MIX-E1102: 1 → 0 | Host declaration (viu vocabulary) |
| `apps/dopus/macro-job.mix` | MIX-E1102: 1 → 0 | Host declaration (viu vocabulary) |
| `apps/dopus/operations.mix` | MIX-E1102: 8 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/dopus/presentation.mix` | MIX-E1102: 1 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/dopus/preview-job.mix` | MIX-E1102: 1 → 0 | Host declaration (viu vocabulary) |
| `apps/dopus/reducer.mix` | MIX-E1102: 1 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/email/mail.mix` | MIX-E1102: 4 → 0 | Host declaration (viu vocabulary) |
| `apps/gallery/gallery.mix` | MIX-E1102: 2 → 0 | Host declaration (viu vocabulary) |
| `apps/monitor/app.mix` | MIX-E1102: 3 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/monitor/fetch.mix` | MIX-E1102: 1 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/prefs/app.mix` | MIX-E1102: 9 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/prefs/catalogue-job.mix` | MIX-E1102: 2 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/prefs/core.mix` | MIX-E1102: 2 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/prefs/custom.mix` | MIX-E1102: 3 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/prefs/email.mix` | MIX-E1102: 4 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/prefs/gallery-job.mix` | MIX-E1102: 5 → 0 | Host declaration (viu vocabulary) |
| `apps/prefs/gallery.mix` | MIX-E1102: 3 → 0 | Host declaration (viu vocabulary) |
| `apps/prefs/look.mix` | MIX-E1102: 1 → 0 | Host declaration (viu vocabulary) |
| `apps/prefs/pages.mix` | MIX-E1102: 2 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/screenshot/app.mix` | MIX-E1102: 1 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/screenshot/events.mix` | MIX-E1102: 4 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/screenshot/job.mix` | MIX-E1102: 10 → 0 | Host declaration (viu vocabulary) |
| `apps/screenshot/macro-job.mix` | MIX-E1102: 1 → 0 | Host declaration (viu vocabulary) |
| `apps/screenshot/macros.mix` | MIX-E1102: 2 → 0 | Host declaration (viu vocabulary) |
| `apps/screenshot/operations.mix` | MIX-E1102: 3 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/screenshot/state.mix` | MIX-E1102: 1 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/screenshot/storage.mix` | MIX-E1102: 3 → 0 | Host declaration (viu vocabulary) |
| `apps/volume-lab/lib/embedded-runtime.mix` | MIX-E1102: 1 → 0 | Host declaration (viu vocabulary) |
| `apps/workshop/app.mix` | MIX-E1102: 1 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/workshop/controller.mix` | MIX-E1102: 5 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/workshop/discovery.mix` | MIX-E1102: 1 → 0 | Host declaration (viu vocabulary); require resolution |
| `apps/workshop/macro-job.mix` | MIX-E1102: 4 → 0 | Host declaration (viu vocabulary) |
| `apps/workshop/macro.mix` | MIX-E1102: 1 → 0 | Host declaration (viu vocabulary); require resolution |
| `engines/viud/crates/testkit/scripts/app_reference.mix` | MIX-E1102: 1 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `engines/viud/scripts/lib/apps.mix` | MIX-E1102: 1 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `engines/viud/tests/editor_app_bus_test.mix` | MIX-E1102: 2 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `engines/viud/tests/network_behaviour_test.mix` | MIX-E1102: 2 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `engines/viud/tests/scene_editor_e2e.mix` | MIX-E1102: 2 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `engines/viud/tests/scene_editor_test.mix` | MIX-E1102: 3 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `engines/viud/tests/scene_reconnect_test.mix` | MIX-E1102: 2 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `engines/viud/tests/toast_summary_test.mix` | MIX-E1102: 1 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `engines/viud/tests/volume_behaviour_test.mix` | MIX-E1102: 2 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `libraries/toolkit/scripts/install_licenses.mix` | MIX-E1102: 1 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `libraries/toolkit/tests/asset_licenses_test.mix` | MIX-E1102: 7 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `libraries/viu/mix/macros.mix` | MIX-E1102: 2 → 0 | Host declaration (viu vocabulary); require resolution |
| `libraries/viu/mix/session-runtime.mix` | MIX-E1102: 1 → 0 | Host declaration (viu vocabulary); require resolution |
| `services/clipd/scripts/clipd.mix` | MIX-E1102: 4 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `services/clipd/tests/clipd_test.mix` | MIX-E1102: 2 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `services/clipd/tests/wayland_test.mix` | MIX-E1102: 3 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `services/dlnad/scripts/dlnad.mix` | MIX-E1102: 2 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `services/editd/tests/edit_bus_test.mix` | MIX-E1102: 2 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `services/exportd/scripts/gtkqt.mix` | MIX-E1102: 1 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `services/exportd/scripts/kde.mix` | MIX-E1102: 1 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `services/exportd/tests/cosmic_export_test.mix` | MIX-E1102: 1 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `services/exportd/tests/gtkqt_export_test.mix` | MIX-E1102: 1 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `services/exportd/tests/gtkqt_recovery_test.mix` | MIX-E1102: 1 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `services/exportd/tests/kde_test.mix` | MIX-E1102: 1 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `services/notifyd/tests/notifyd_test.mix` | MIX-E1102: 3 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `share/assets/install.mix` | MIX-E1102: 1 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `share/brand/install.mix` | MIX-E1102: 1 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `share/icons/install.mix` | MIX-E1102: 1 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `share/layout/apply.mix` | MIX-E1102: 1 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `share/scenes/sync.mix` | MIX-E1102: 2 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |
| `tests/lib/private_bus.mix` | MIX-E1102: 1 → 0 | Stale entry cleared: core mix_path is already registered in libraries/mix/src/builtins.rs:277 on main |

Host audit: `git grep -n -E '^-- host: viu[[:space:]]*$' -- services tools share tests engines` returns no matches (rc 1). The complete `git grep -l -E '^-- host: viu[[:space:]]*$' -- '*.mix'` list contains only viu app/worker modules and libraries/viu/mix helpers. No plain-mix or mixd service entry opts into viu. In particular clipd, dlnad, editd, exportd and notifyd have no such declaration. Their removed E1102 counts match core mix_path calls; require imports remain namespaced. The stale main baseline predates that core registration. No service diagnostic is hidden by the new vocabulary.

## Risk tier

Tier 2 overall, including the exact historical scanner exceptions. Lint and
viu runtime/app integration are Tier 1 areas within that review. Module
isolation and preserving Dopus's cut-over behaviour are the main runtime risks.

## How tested

- Run: `mix tools/lint-viu-check.mix` on CBC: tree Mix build, formatting, env,
  committed lint ratchet, 142 lint tests and complete lib-viu/viu suites.
  `mix tools/ci.mix test` additionally checks the merged workspace. Focused
  Dopus startup/raw-name/chooser checks and all 30 app scripts run with real
  host bindings. Lockfile resolution and baseline generation run only on CBC.
- Observed: no lint entry grows against main; native Ced editing, Bus and
  macro-panel paths, Backdrop transitions and Dopus startup/row tests pass.
  GitHub's required checks must pass on the final pushed head.
- Not tested: interactive desktop/VT acceptance, production installation or
  app windows. Existing GPU/installed-icon test ignores remain unchanged.

CBC refresh receipts:

- `20261108-153502-b6ad5325-a1` at `adffa80b`: focused Dopus startup and
  migrated contract tests, plus the 30-script native harness; rc 0. Submitted
  with `--light`, scheduled on cbc2.
- `20261108-154554-5a9cc400` at `adffa80b`: CBC light override for metadata,
  formatting and one-crate Mix build; env 195 names / zero problems, baseline
  regenerated; rc 0. `cargo update --workspace --offline` changed no packages
  and produced no lockfile diff, preserving all external pins. The generated
  formatting/baseline patch was printed, worker changes cleared in that job,
  and the patch applied locally.

- `20261108-153408-bb620337-a1` at `adffa80b`: the original heavy-worker
  regeneration also completed, rc 0, and produced the identical patch.
- Complete merge gate `20261108-154812-28b71f73-a1` at `d2d046e1` on cbc3:
  `mix tools/lint-viu-check.mix`, then `mix tools/ci.mix test`, submitted with
  `--worker any`. The subsequent receipt commit changes documentation only.

The final gate result and final head are recorded in the local `last.md`
receipt; GitHub runs both required checks on the final pushed head.

## How to undo

Revert the squash commit, including the regenerated baseline and exact scanner
exceptions. No runtime installation or external data migration is involved.

## Authorship and review

Author: Sol (gpt-6.1-sol), committed and pushed through the mixviu-agent bot.
Earlier review nits were addressed; the 2026-11-10 GLM and DeepSeek round
requested changes. Its findings and dispositions are recorded in last.md. This main refresh needs review of its new
diff, including security-policy review, and human approval. Open PR: #6.
