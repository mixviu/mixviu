## Summary

Retire `mix build`, `mix clean`, `mix test` and `mix update`: each refuses
before touching source or installed binaries, including with `--help`, and
names `mix tools/ci.mix <stage>` and the CBC route. Update both help surfaces
and the manual.

Gate the four native-session delay/fault variables behind an opt-in
`test-hooks` feature. Ordinary builds ignore them; the build script refuses
the feature outside the debug profile, with a second compile guard when debug
assertions are off. The native PTY test target requires this feature, and the
CI test stage enables it explicitly. The env registry records the gate.

A linked hook also links an artefact marker. `mix tools/ci.mix mix-hygiene`
checks a hook-enabled debug `mix` carries it and an ordinary one does not;
the new `release-mix` stage builds the release `mix` and refuses it if the
marker is present, so the shipped artefact itself is checked, not only the
build profile. `tests/owned_editor_pty.rs`, which compiles the editor from
source by `#[path]`, now includes the same hook module (round 1 left it
unable to compile under the full test gate).

## Risk tier

2 — Mix CLI/runtime plus the CI recipe and a single historical gitleaks
false-positive fingerprint. Code-owner approval and the required review
remain necessary. No new dependencies or deployment.

## How tested

- Run: CBC job `20261107-204950-4537e2c4` on the light worker at source
  `805db4c61b62812999081e9bb7eb39ab677fd7ab`: `mix tools/ci.mix fmt`,
  `env` and `mix-hygiene`, all green (`BUILD_RC=0`). 29 tests passed
  (6 normal binary, 6 hook-enabled binary, 15 native PTY, 2 hook unit tests),
  plus expected release-feature rejection. Env: 176 names, zero problems.
  `mix lint tools/ci.mix`: zero errors and warnings.
- Observed: command refusals preserve an empty source/install fixture with
  no git or cargo on PATH. Tests cover both help surfaces, complete versus
  torn binary result frames, all four hook lookups with and without the
  feature, existing native PTY fault paths, and release-feature rejection.
- Round 2, source `53357d480966881475a04647498bdf8723b724ce`: full
  `mix tools/ci.mix test` on cbc3, CBC job `20261107-210937-ccb133a8-a1`,
  `BUILD_RC=0`: 10,560 passed, 0 failed, 90 ignored; `owned_editor_pty`
  79/79 with and without `test-hooks`; marker present in the hook build,
  absent in the ordinary build; release-feature rejection still fires.
  `release-mix` on cbc3, CBC job `20261107-214025-5234a7b5-a1`,
  `BUILD_RC=0`: release `target/release/mix` built, marker absent.
- Not tested: full `all` gate (fmt, clippy, mix-lint stages), desktop
  sessions or installation.

## How to undo

Revert this branch's commits. This restores the local Cargo shortcuts and
ungated fault variables; no stored data or wire formats change.

## Authorship and review

Authored by OpenAI Codex as `mixviu-agent[bot]` for Mark's Mix hygiene brief.
Independent reviewer findings and human approval are pending. This is PR #7;
main is updated only through the protected squash-merge process.

## Main synchronisation (2026-11-08)

Merged origin/main (60cb127) without rebasing or rewriting branch history.
Only last.md conflicted; both intent/evidence records were kept. CBC job
20261108-153105-ab2990de-a1 resolved Cargo.lock with no changes and
regenerated the baseline using the merged tree mix: no lint count increased
against main. Full test/release gate at 7cb24b0b is running on CBC.

Gitleaks found the historical help line `Keys: Ctrl+Shift+T/W new/close tab`
in commit 1bbcf96c, before the squash cut-over. The exact one-finding
fingerprint is ignored; the main gitleaks policy is unchanged. A full
redacted history scan with the pinned gitleaks 8.30.1 now passes.
