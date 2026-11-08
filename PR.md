## Summary

Retire `mix build`, `mix clean`, `mix test` and `mix update`: each refuses
before touching source or installed binaries, including with `--help`, and
names the CI recipe and CBC route. Update both help surfaces and the manual.

Gate the four native-session delay/fault variables behind opt-in debug-only
`test-hooks`. Ordinary builds ignore them; release profiles reject the feature.
Process tests enable it explicitly. Debug and release artefact scans prove
that ordinary binaries do not link the hooks.

Merge main 60cb127 without rebasing or rewriting history. Keep both evidence
records from the only conflict, in last.md. Resolve Cargo.lock on CBC with
no changes. Regenerate the lint baseline with the merged tree mix: 54 fewer
diagnostics, across 30 fewer files, and no per-file count increased.

Three exact gitleaks fingerprints cover historical terminal shortcut text
and notes quoting it. Main detection rules are unchanged; no credential is
exempted and no test or CI job is disabled.

## Risk tier

2 — Mix CLI/runtime, CI recipe and a historical scanner exception. Requires
code-owner approval and the repository review process. No new dependencies
or deployment.

## How tested

- Run: CBC job `20261108-153105-ab2990de-a1` at merge 6d4dd39e:
  offline Cargo metadata resolution, build the merged tree mix and run
  `mix-lint-baseline` with that binary. Cargo.lock unchanged; baseline only shrank.
- Run: CBC job `20261108-153201-d97092aa-a1` on cbc3 at source
  `7cb24b0b596befc40f05c55893e39124a5b463f7`: `mix tools/ci.mix fmt`,
  `env`, `mix-lint`, `test` and `release-mix`, all green (`BUILD_RC=0`).
  All 611 test result summaries report zero failures. The full saved log was read.
- Observed: 194 env names, zero registry problems; zero lint regressions;
  hook marker present in the opt-in debug binary and absent from ordinary
  debug and release binaries; release-feature rejection fired as intended.
- Run: pinned gitleaks 8.30.1 full-history redacted scan passed after the
  scanner exception commit.
- GitHub exposed a lifecycle fixture trace race: formatted writes from its
  readiness child interleaved with the citizen boot record. Assemble that
  small record before appending it. Assertions and timeouts are unchanged;
  CBC job `20261108-161926-f3889961` at `2c80c0bf` passed both lifecycle
  configurations (7 tests each, one existing subprocess helper ignored) and
  ten further runs of the affected grace case. A fresh sibling mixd was
  built first. Formatting and env checks also passed at this source.
  Production code is unchanged from the full CBC gate.
- GitHub required checks: `ci-ok` and `security-ok`; live results are on PR #7.
- Not tested: desktop sessions, installation or production deployment.

## How to undo

Revert the squash commit after merge. No stored data or wire formats change.

## Authorship and review

Authored by OpenAI Codex as `mixviu-agent[bot]` for Mark. Independent review
and human approval remain required. PR #7 is updated; main changes only
through the protected squash-merge process.
