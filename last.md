# Monitor completion timeout — PR #225 round 2

PR: https://github.com/mixviu/mixviu/pull/225, branch
`fix/monitor-completion-timeout`, author mixviu-agent. Scoped validation is
green; ready for review. Tested source `eff7e42600488796e1a502995cd1d38e42b0f92a`;
the receipt commit changes only this file.

Merged current main through `f36d3948` (#226) without conflicts. The Monitor
implementation diff remains two files. #226 replaces the pre-existing
Workshop byte guard that stopped round 1 with its publication-kind assertion.

## Cause and repair

Plain main `b36a00d6`, without any #220 source or private artifact-cache
prototype, reproduced **1/20 Monitor failures (5%)**, CBC
`20261121-224904-4432b3fd-a1`. The surrounding ordinary width-eight pool
passed all 100 binaries. The cache prototype is not required for the failure.

Private timestamp trace `20261121-225952-335e7d65-a1` identifies the first
`apps/monitor/core-test.mix` load:

| Event | Unix timestamp, seconds |
|---|---:|
| Receive begins | 1795266304.379864250 |
| Runtime work begins | 1795266304.382523212 |
| Unchanged ten-second receive expires | 1795266314.379956395 |

No completion or wake was recorded for that load before expiry. A passing
ordinary-pool trace (`20261121-225055-c03c4174-a1`) shows the same load taking
7.117067179 s wall / 4.085684696 s CPU, followed promptly by completion
(1795265628.752633762), wake (1795265628.752865153) and receive
(1795265628.755407864). This locates slow fixture work; it does not establish
a lost wake or a post-timeout completion time.

The fixture repeatedly appended through a map field, copying its growing
list. `apps/monitor/core-test.mix:13` and `:23` now construct the identical
2,000- and 3,000-row lists directly with `map(range(...))`.
`libraries/viu/tests/monitor_policy.rs:19` adds `#[track_caller]` so future
completion failures report their calling assertion.

Both full-size projections, malformed-row refusal, unlimited query, fake
procfs, disabled/protected signal assertions and fake successful execution
remain intact. No real signals, production changes, widened waits/deadlines,
retries, serial-list changes or new environment variables. No new app test.

## Validation on merged source

| Check | Result | CBC job |
|---|---|---|
| Twenty explicit loaded `monitor_policy` invocations | **20/20 green**, zero failures | `20261122-025238-d48719fa-a1` |
| Ordinary six-package test-support all-target pool, width eight | **100/100 binaries green**, including every lib-viu target and `app_mix`; existing serial Workshop green | same job |
| lib-viu default all-target pool, width eight | **55/55 binaries green**, including `app_mix` | `20261122-025239-245d6edd-a1` |
| Fmt, default and test-support all-target clippy | pass | same job |
| Default and test-support doctests | pass | same job |
| Explicit standalone test-support `workshop_bus` | **8/8 green**, 19.99 s | same job |
| Env registry | **277 names, zero problems** | same job |
| Workspace Mix lint ratchet | **zero regressions** | same job |

The six selected packages are design, theme, toolkit, lib-viu, viu and
settingsd. Twenty explicit `cargo test --locked -p lib-viu --features
test-support --test monitor_policy` invocations ran alongside that ordinary
pool, using the same experiment as round 1. Invocation wall time includes
Cargo lock/build waits; pool replay follows its build/record pass. Retries
were disabled. Existing ignored tests remain unchanged.

`app_mix` passed in the eight-wide pools: **46.25 s default / 75.93 s support**.
The existing Monitor core script also passed standalone; its CBC step took
about 0.20 s, below five seconds. The known toolkit macros/scene_motion
nested-read flake did not fire. No pre-existing failure was bypassed.

Before/after: **1/20 failures on plain main**, **0/20 with the round-1 fix**
(`20261121-230938-e7e02839-a1`, stopped on the old Workshop guard), and now
**0/20 after merging #226**, with the surrounding pool green.

Both CBC jobs ended `BUILD_RC=0`; saved logs were read. No full-workspace CI,
merge, install or deployment was run. The private flake ledger and
TODO-monitor record the repair and round-2 validation. The merge gate owns
the full-workspace run.
