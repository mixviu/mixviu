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
  The full-history CI checkout fetches that branch too. These are confirmed
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
| MIX-E1102 | 1051 | 646 |
| MIX-E1201 | 4 | 4 |
| MIX-E1401 | 2 | 2 |
| MIX-W2201 | 2 | 2 |
| MIX-W2302 | 5 | 5 |
| MIX-W2307 | 2 | 2 |
| MIX-W2401 | 53 | 49 |
| **Total** | **1265** | **855** |

410 diagnostics removed; no per-file/code regression.


The pre-cut-over reduction was 828 to 527. Main subsequently added app and
service files. Remaining diagnostics inherited from main, including the newer
Email sibling modules, webd examples/widgets and intentional negative tests,
stay in the ratchet; this refresh does not broaden that migration.

## Risk tier

Tier 1 for lint and viu runtime/app integration. The exact historical scanner
exceptions are security-policy changes requiring Tier 2 review. Module
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

CBC refresh receipts so far:

- `20261108-153502-b6ad5325-a1` at `adffa80b`: focused Dopus startup and
  migrated contract tests, plus the 30-script native harness; rc 0. Submitted
  with `--light`, scheduled on cbc2.
- `20261108-154554-5a9cc400` at `adffa80b`: CBC light override for metadata,
  formatting and one-crate Mix build; env 195 names / zero problems, baseline
  regenerated; rc 0. `cargo update --workspace --offline` changed no packages
  and produced no lockfile diff, preserving all external pins. The generated
  formatting/baseline patch was printed, worker changes cleared in that job,
  and the patch applied locally.

Full focused and workspace gates are pending on the refreshed commit.

## How to undo

Revert the squash commit, including the regenerated baseline and exact scanner
exceptions. No runtime installation or external data migration is involved.

## Authorship and review

Author: OpenAI Codex, committed and pushed through the mixviu-agent bot.
GLM-5.3 and DeepSeek approved the earlier implementation with nits; all seven
follow-up items were addressed. This main refresh needs review of its new
diff, including security-policy review, and human approval. Open PR: #6.
