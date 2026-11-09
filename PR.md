## Summary

`mix build`, `mix clean`, `mix test` and `mix update` now refuse before
touching source or installed binaries, including with `--help`. Both help
surfaces and the manual direct maintainers to the CI recipe and CBC.

The four native-session delay/fault variables require the opt-in debug-only
`test-hooks` feature. Ordinary builds ignore them; release builds reject the
feature. Process tests enable it explicitly. Artefact scans check that the
hook marker is present in the opt-in binary and absent from ordinary builds.

Merged current main d17f7caa without rebasing. Kept the main lifecycle trace
writer and regression test, lint baseline, and SQLite security versions.
Preserved both handoff records and retained the description for this PR.
No new dependencies or installation.

## Risk tier

Tier 2: Mix CLI/runtime and the CI recipe. Independent review and approval
remain part of the protected merge process.

## Validation

Current CBC evidence is recorded in last.md against source
9ad45200302f610533b701fd8ac9762e932cc88f. Lockfile re-resolution and rustfmt
produced no changes. Full tools/ci.mix all passed on CBC job
20261110-065037-808eaae5-a1: 10,681 passed, zero failed, 100 ignored;
197 environment names, zero registry problems and zero lint regressions.
The release artefact check is pending.

## How to undo

Revert the squash commit. No stored data or wire formats change.

## Authorship

Authored and updated by OpenAI Codex through mixviu-agent[bot] for Mark.
