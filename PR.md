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
`which("dopus")`, returning `UNAVAILABLE` if it is absent.

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
- Not tested: legacy Dopus Rust comparison executable (not a workspace
  member), interactive desktop, VT acceptance or production installation.
  Existing GPU/installed-icon tests remain ignored by their owning suites.

CBC receipts:

- `20261107-210610-69bf2c56-a1` at `2b7e776a`: 142 lint tests and both complete
  viu suites passed. The wrapper then returned 101 for an extra, invalid
  `viu-dopus` package invocation; that non-workspace step has been removed.
- `20261107-210932-d3432f56-a1` at `300a5459`: Workshop tests, fmt, env, lint
  ratchet and baseline generation passed, rc 0.
- `20261107-212208-6c41a2f5-a1` at `956a468b`: all 30 app Mix scripts passed
  with actual native bindings and filesystem operations, rc 0. CBC's
  rustfmt patch was applied locally.

## How to undo

Revert this branch's commits, including the regenerated baseline. No runtime
installation or external data migration is involved.

## Authorship and review

Author: OpenAI Codex, committed and pushed through the mixviu-agent bot.
Independent reviewer and human approval are pending. No PR has been opened,
as required by the pre-cut-over brief.
