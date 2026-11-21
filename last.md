# Decoration deadline tests — PR #232

PR: https://github.com/mixviu/mixviu/pull/232 (draft).
Tested code head: `02eb96f56e4b49e3d4fa230df34eaa63a30458d9`. Base: `e8e9da41729e`.

Stopped under the requested rule on an unchanged test in the normal CBC pool.
Do not mark ready: the requested 20 green loaded rounds are incomplete.

## Change

Both detectors retain `BUDGET = 200 ms`. A private seam takes the start instant
and probe implementation; tests inspect the actual worker argument and assert
exactly `now + BUDGET`. Separate five-second channel watchdogs cover the real
silent registry and non-Wayland detector. No retry, ignore or serial-list change.

Source locations (viu / toolkit):

| Property | Source |
|---|---|
| Production seam | `libraries/viu/src/decorations.rs:75`, `libraries/toolkit/src/decorations.rs:92` |
| Exact deadline and 200 ms constant | `libraries/viu/src/decorations.rs:326`, `libraries/toolkit/src/decorations.rs:343` |
| Real silent registry completion | `libraries/viu/src/decorations.rs:301`, `libraries/toolkit/src/decorations.rs:318` |
| Wrong-deadline rejection control | `libraries/viu/src/decorations.rs:352`, `libraries/toolkit/src/decorations.rs:369` |
| Worker ignores deadline; caller still returns None | `libraries/viu/src/decorations.rs:358`, `libraries/toolkit/src/decorations.rs:375` |
| Hung operation rejected by same five-second watchdog | `libraries/viu/src/decorations.rs:372`, `libraries/toolkit/src/decorations.rs:389` |

The blocked fixtures are released after the watchdog result, including failure
paths. Their intentional indefinite waits do not leave permanent test workers.

## Measurements

CBC baseline job `20261122-073024-bd7c8334-a1`, no-default features.
Instrumentation only records elapsed time immediately before the unchanged
250 ms assertions. Twenty standalone runs per test completed. Seventeen complete
normal eight-wide rounds passed. Round 18 recorded all four timing samples before
cancellation at the stop below. Each round selected all 80 lib-viu/toolkit targets
through the existing test pool; cancelled-round rc124 results are cancellation
artefacts, not test findings.

| Original test | Alone: breaches / samples; maximum | Loaded: breaches / samples; maximum |
|---|---|---|
| lib-viu silent registry | 0/20; 200.278483 ms | 0/18; 200.932414 ms |
| toolkit silent registry | 0/20; 200.576783 ms | 0/18; 206.676539 ms |
| lib-viu non-Wayland | 0/20; 0.228476 ms | 0/18; 0.006227 ms |
| toolkit non-Wayland | 0/20; 0.096675 ms | 0/18; 0.010051 ms |

No sample exceeded 250 ms, so measured excess is zero. This sample does not
reproduce the original #231 failure (`20261122-072127-c8e2cd79-a1`, old
`libraries/viu/src/decorations.rs:300`); it does not establish a flake frequency
of zero under other scheduler loads. No artificial stress harness was used.

Fixed job `20261122-073142-6352fc86-a1`: all four real timing-path tests passed
in all eight loaded rounds (0/8 failures per test). Seven full rounds were green;
round eight failed only the unchanged toolkit macros test below. Fixed tests
assert deadline equality and completion, not a 250 ms wall-clock measurement.

## Validation and stop

CBC `20261122-073142-6352fc86-a1` passed:

- `cargo fmt --all -- --check`.
- CI-form scoped `cargo clippy --locked -p lib-viu -p toolkit --all-targets`
  without `-D warnings`, for default, no-default, default + lib-viu/test-support,
  and no-default + lib-viu/test-support.
- All 80 targets plus explicit doc-tests in each of those four configurations.
  Both test-support runs included all nine `workshop_bus` tests under the
  existing serial policy.
- All new deadline/watchdog controls in both crates in those runs and the soak.

Round eight of the subsequent normal no-default eight-wide soak failed
`macros::tests::closing_a_window_kills_its_group_and_reaps_the_leader`:
`libraries/toolkit/src/macros.rs:1447`, "timed out waiting for the macro worker".
The test starts at `libraries/toolkit/src/macros.rs:1658`; that file has no diff
from the base. The toolkit binary reported 415 passed, 1 failed, 2 ignored.
Job returned `BUILD_RC=1`. No rerun or fix of that pre-existing failure was made.

The baseline job was cancelled via CBC; cancellation receipt
`20261122-073024-bd7c8334-a1` reports worker rc143. The 20-round goal and additional
source-mutation runs (wrong production deadline; unbounded production receive)
were not reached. Committed negative fixtures did pass; these further mutation
checks remain pending. Earlier jobs: `20261122-072926-74499bdb-a1` rc75 did not run
(worker interpreter path); `20261122-073055-1920ec06-a1` found our formatting,
applied before the tested head.

Full raw logs and harnesses are retained in the private evidence directory
`_var/evidence/2026-11-22-decorations-timing/`. The private flake ledger,
TODO-viu and TODO-toolkit record the evidence and stop.
