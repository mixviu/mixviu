# Scenes parent-watch flake — 2026-11-10

Branch: `fix/scenes-watch-flake`, based on `origin/main` at `ad326896`.
Source change: `364bd37958e1` (pushed as the agent bot). No PR opened.

## Cause

The test assumed every fs_wait batch contained changes. The actual contract
is changes OR overflow. This is a userspace publication race, with a valid
overflow-only batch, rather than an inotify loss or a spurious timeout:

- `libraries/mix/src/fs_watch.rs`, State::event: structural reconciliation
  installs new directory watches; if the set grows, it calls overflow()
  BEFORE record() publishes the creation event. The evaluator may consume
  the first publication between these two calls.
- This intentionally covers files written before their directory watch
  was installed. It does not promise an individual event for every file.
- `libraries/mix/src/native_events.rs`, Queue::take_watch: returns None when
  both changes and overflow are empty. Queue::next_selected loops on
  readiness. fs_wait has no timeout or empty-spurious-wakeup return path.
- `engines/viud/scripts/lib/scenes.mix`, watch_anchor: picks the closest
  existing ancestor of the requested parent. filesystem() treats overflow
  as a library invalidation, recomputes signatures, and reloads dependants;
  filesystem_scenes() also rescans. Ancestor creation events likewise
  invalidate a library. No loader or builtin change is needed.

## Change

All four parent-watch assertions now accept overflow or a relevant change.
The sequence lives in `engines/viud/tests/scenes_parent_watch.mix` and loops
past irrelevant batches, with a five-second elapsed-time check. Since
fs_wait itself has no timeout argument, scenes_test.mix runs this child
with a hard 25-second run_argv deadline that also bounds a blocked wait.
Subsequent edit checks require the intended file and modified kind (or
overflow), preventing an unrelated creation record from satisfying them.
The child uses the exact running Mix executable.

The real late-library creation batch is passed to filesystem(). The test
requires the helper signature to change and be acknowledged, and an enabled
dependent scene revision to advance. The library is absent at watch startup
and its helper is written immediately after mkdir, without sleeps or retries
of the write. The template library path is also the default SCENES_LIB path.

## Validation

Local Mix lint: zero diagnostics in both changed scripts; git diff --check clean.
Original selected Rust test reproduced the reported line-1580 failure at
iteration 4 on CBC while the other worker was busy. Six of the first seven
completed iterations passed; one failed (1/7). Each took about 33 seconds,
so the 200-full-test loop was cancelled in favour of 200 isolated watch-block
iterations. Cancellation/interruption records from iteration 8 onwards are
excluded. Cancelled baseline attempt: `20261110-213109-b0c147f9-a1`, rc 143.

Full gate: `mix tools/ci.mix all`, through cbc-build at pushed source SHA
`364bd37958e15cadc6d155bf5c4a91cfdee6a8f5`.
`BUILD_RC=0 job=20261110-213256-259e8dce-a1 worker=cbc2`.
The previously failing selected Rust test passed, including the added real
loader/signature/revision assertion. All stages passed, with 625 test-result
summaries and zero failed results; Mix lint reports zero regressions.

Isolated comparison on CBC, with the other worker running a full gate:

| Sequence | Failures | Rate | CBC job (rc 0) |
| --- | --- | --- | --- |
| Original four assertions | 16/200 | 8% | `20261110-213612-83d2dadc-a1` |
| Fixed four checks | 0/200 | 0% | `20261110-213459-ea23fab3-a1` |

The original block was extracted unchanged from `ad326896`, with fs_wait
wrapped only to print its returned batch to stderr. It ran using this source
build of Mix (the builtin implementation is unchanged). Nine failures were
the missing-library creation assertion, seven were catalogue continuation.
Every failing batch was `{changes:[], overflow:true}`. These are finite-sample
stress results; logging and isolation change scheduling from a complete gate.

The fixed 200-run probe captured overflow-only batches for creation once and
catalogue continuation nine times, and accepted them. A negative control
replaced the catalogue watch with a quiet sibling watch while still doing
the same mkdir/write operations. It timed out at the probe’s two-second
run_argv deadline, proving missing notification coverage still fails.

Saved logs read in full (basenames under the local cbc-build log directory):

- `20261110-213109-cbc2-ad326896ac08-20261110-213109-b0c147f9-a1-21c0d2cd.log`
- `20261110-213832-cbc2-364bd37958e1-20261110-213256-259e8dce-a1-990ebf55.log`
- `20261110-220459-cbc2-364bd37958e1-20261110-213459-ea23fab3-a1-95fe1044.log`
- `20261110-220536-cbc2-364bd37958e1-20261110-213612-83d2dadc-a1-4520a3d7.log`

Nothing remains for this fix. This final receipt commit changes only last.md;
the source tested by the full gate remains the source at the SHA above.

No desktop, VT, installed prefix, main branch or production changes.
