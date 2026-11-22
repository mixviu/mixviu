# Decorations watchdog follow-up — PR #237

PR: https://github.com/mixviu/mixviu/pull/237

Branch: test/decorations-watchdog-2s. Tested code head: ac7271f5c4bd3edcff5d6d23478b1f08e15f97e5. The subsequent report commit changes only this receipt.

Both TEST_DEADLINE constants are now two seconds. BUDGET remains 200 ms, and the exact now + BUDGET assertion is unchanged. No new app tests, environment variables, dependencies, retries or serial registrations.

## CBC checks

- 20261122-120843-ce440545-a1: BUILD_RC=0. Workspace fmt check; CI-form scoped all-target clippy (no -D warnings); normal pools and doc tests in all four configurations; mutation controls; twenty loaded rounds.
- 20261122-121516-35d3e27d-a1: BUILD_RC=0. Explicit --all-targets pools, all 80 binaries in each of the four configurations.
- Configurations: default, --no-default-features, --features lib-viu/test-support, and both latter flags together. Feature-enabled workshop_bus ran all nine tests in its existing serial lane. Existing clippy warnings were non-fatal. Rust ran only on CBC.

## Mutation controls

Each control was applied to the production bounded_probe_with in one crate at a time, tested with cargo test --locked -p PACKAGE --no-default-features --lib TEST -- --nocapture, then restored.

- Late return: preserve let deadline = now + BUDGET and the production recv_timeout, save its result, sleep three seconds, then return it. probe_receives_exact_reply_deadline fails with injected probe hung: Timeout in both crates (cargo rc101, 2.00 s). This mutation honours the production deadline before stalling, so deadline equality alone cannot reject it.
- Unbounded receive: replace production recv_timeout with recv. worker_ignoring_deadline_cannot_hang_caller fails with caller waited for a stuck worker: Timeout in both crates (cargo rc101, 2.00 s).
- The existing permanent never-returning operation and different-deadline controls remain green throughout the normal runs. All mutations and timing instrumentation were restored; final worker git diff --exit-code passed.

## Twenty loaded rounds

Command: mix tools/ci/test_pool.mix --jobs 8 -- --locked -p lib-viu -p toolkit --no-default-features -- --show-output

All twenty full 78-binary pools and doc tests passed; decoration tests 400/400 green. No macro-close flake hits, retries or exclusions.

Temporary test-helper instrumentation measures successful watchdog receive completion, including wait scheduling, from immediately after spawning the operation. Each row is the maximum of four successful completion samples per crate (silent registry, stuck worker with bounded caller, injected exact-deadline probe, and non-Wayland detection). Negative watchdog timeouts are excluded. These are observed wall times, not a replacement timing assertion. Initial thread-spawn overhead is outside this measurement.

| Round | lib-viu maximum ms | toolkit maximum ms |
|---|---:|---:|
| 1 | 200.904 | 202.858 |
| 2 | 202.902 | 202.125 |
| 3 | 200.765 | 201.897 |
| 4 | 201.87 | 204.702 |
| 5 | 201.155 | 202.031 |
| 6 | 202.829 | 200.877 |
| 7 | 202.046 | 201.854 |
| 8 | 204.89 | 204.89 |
| 9 | 200.184 | 204.571 |
| 10 | 207.84 | 205.05 |
| 11 | 202.973 | 205.898 |
| 12 | 209.927 | 201.149 |
| 13 | 205.023 | 205.926 |
| 14 | 202.866 | 206.712 |
| 15 | 200.148 | 203.135 |
| 16 | 205.599 | 201.682 |
| 17 | 202.966 | 201.062 |
| 18 | 200.227 | 203.471 |
| 19 | 200.898 | 205.863 |
| 20 | 201.179 | 206.021 |

No pre-existing failure triggered the stop rule. The known toolkit macro-close flake remains owned by viu-0c; this PR claims no fix for it. The private TODO-viu follow-up is resolved by #237, with validation noted in the flake ledger. No merge or deployment performed.
