# Decoration deadline tests — PR #232, round 2 green

PR: https://github.com/mixviu/mixviu/pull/232 — ready for review.
Tested head: `57155d0534b4bec292be1b299718d91c191b591d`.
Fetched `origin/main` was `e8e9da41729e`, already an ancestor of that head;
`git merge origin/main` reported "Already up to date". No conflicts or source
changes were needed. The subsequent commit updates this report only.

CBC job `20261122-085959-bec2fb03-a1`: `BUILD_RC=0`, 1808 seconds.
All Rust commands ran on CBC. No retries, ignores, serial-list changes or
stress harness were added.

## Fresh loaded soak

Twenty rounds from the start on the tested head, each using:

```text
mix tools/ci/test_pool.mix --jobs 8 -- --locked -p lib-viu -p toolkit --no-default-features --all-targets
```

Each round ran all 80 selected binaries. All ten decoration tests in each crate
passed every round: 400/400 test executions. In particular, all four original
timing-path tests passed 20/20 each. The owned macros flake was allowed only if
it was the sole failed test and sole failed binary; it did not occur.

All rows below belong to job `20261122-085959-bec2fb03-a1`. Pool seconds exclude
build/record (64 seconds in round 1, one second in subsequent rounds).

| Round | Decorations | Macros hits | Pool seconds |
|---|---|---|---|
| 1 | 20/20 | 0 | 79 |
| 2 | 20/20 | 0 | 78 |
| 3 | 20/20 | 0 | 79 |
| 4 | 20/20 | 0 | 80 |
| 5 | 20/20 | 0 | 81 |
| 6 | 20/20 | 0 | 79 |
| 7 | 20/20 | 0 | 78 |
| 8 | 20/20 | 0 | 79 |
| 9 | 20/20 | 0 | 79 |
| 10 | 20/20 | 0 | 78 |
| 11 | 20/20 | 0 | 78 |
| 12 | 20/20 | 0 | 78 |
| 13 | 20/20 | 0 | 80 |
| 14 | 20/20 | 0 | 79 |
| 15 | 20/20 | 0 | 79 |
| 16 | 20/20 | 0 | 79 |
| 17 | 20/20 | 0 | 78 |
| 18 | 20/20 | 0 | 80 |
| 19 | 20/20 | 0 | 78 |
| 20 | 20/20 | 0 | 79 |

## Source-mutation controls

The same CBC job ran both mutations separately in both crates, then restored
the exact original source before starting the soak.

| Mutation | lib-viu | toolkit |
|---|---|---|
| Production `now + BUDGET + 1 ms` | Equality test failed, rc101, 0.00 s | Equality test failed, rc101, 0.00 s |
| Production `recv()` instead of `recv_timeout(...)` | Watchdog rejected it, rc101, 5.00 s | Watchdog rejected it, rc101, 5.00 s |

Wrong-deadline mutations failed `probe_receives_exact_reply_deadline` with
`assertion left == right failed`. Unbounded receives failed
`worker_ignoring_deadline_cannot_hang_caller` with
`caller waited for a stuck worker: Timeout`. These intentional failures are
negative-control successes, not soak failures. The blocked fake worker is
released after the watchdog result.

## Formatting and clippy

The same job passed `cargo fmt --all -- --check` and CI-form scoped
`cargo clippy --locked -p lib-viu -p toolkit --all-targets`, without
`-D warnings`, in all four configurations:

- Default features.
- `--no-default-features`.
- `--features lib-viu/test-support`.
- `--no-default-features --features lib-viu/test-support`.

Round 1 also passed every target and explicit doc-tests in all four
configurations, including all nine feature-gated `workshop_bus` tests in both
test-support runs: job `20261122-073142-6352fc86-a1`.

## Source locations

| Property | lib-viu | toolkit |
|---|---|---|
| Production deadline seam | `libraries/viu/src/decorations.rs:75` | `libraries/toolkit/src/decorations.rs:92` |
| Exact deadline and unchanged 200 ms constant | `libraries/viu/src/decorations.rs:326` | `libraries/toolkit/src/decorations.rs:343` |
| Silent registry completion | `libraries/viu/src/decorations.rs:301` | `libraries/toolkit/src/decorations.rs:318` |
| Wrong-deadline rejection fixture | `libraries/viu/src/decorations.rs:352` | `libraries/toolkit/src/decorations.rs:369` |
| Worker ignoring deadline | `libraries/viu/src/decorations.rs:358` | `libraries/toolkit/src/decorations.rs:375` |
| Hung-operation watchdog control | `libraries/viu/src/decorations.rs:372` | `libraries/toolkit/src/decorations.rs:389` |

## Before/after and historical macros hit

Baseline job `20261122-073024-bd7c8334-a1` measured the unchanged 250 ms
assertions. Seventeen full eight-wide rounds passed; round 18 supplied all four
timing samples before cancellation under round 1's stop rule. Cancelled pool
rc124 results are cancellation artefacts, not findings.

| Original test | Alone: breaches / samples; maximum | Loaded: breaches / samples; maximum |
|---|---|---|
| lib-viu silent registry | 0/20; 200.278483 ms | 0/18; 200.932414 ms |
| toolkit silent registry | 0/20; 200.576783 ms | 0/18; 206.676539 ms |
| lib-viu non-Wayland | 0/20; 0.228476 ms | 0/18; 0.006227 ms |
| toolkit non-Wayland | 0/20; 0.096675 ms | 0/18; 0.010051 ms |

No baseline sample exceeded 250 ms; excess was zero. This sample did not
reproduce #231's original breach (`20261122-072127-c8e2cd79-a1`, old
`libraries/viu/src/decorations.rs:300`) and does not establish zero flake risk.
The fixed tests now assert exact deadline equality and five-second completion
independently: zero failures in 20/20 fresh loaded iterations per test.

Round 1 job `20261122-073142-6352fc86-a1`, round 8, hit unchanged
`macros::tests::closing_a_window_kills_its_group_and_reaps_the_leader` at
`libraries/toolkit/src/macros.rs:1447`: "timed out waiting for the macro worker".
The toolkit binary reported 415 passed, 1 failed, 2 ignored. viu-0c owns this
flake; round 2 explicitly permits recording it and continuing. Fresh round 2
hits: **none (0/20)**. No macros fix is claimed here.

Earlier job `20261122-072926-74499bdb-a1` returned rc75 without running;
`20261122-073055-1920ec06-a1` found our formatting, applied before the tested
code. No validation work remains for this brief.

Full raw logs and harnesses are retained privately under
`_var/evidence/2026-11-22-decorations-timing/`, with round 2 in `round2/`.
The flake ledger and TODO-viu record the completed evidence.
