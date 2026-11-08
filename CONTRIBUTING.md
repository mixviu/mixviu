# Contributing to mixviu

mixviu is four engines (`mix`, `mixd`, `viu`, `viud`) and applications
written in Mix. Most new work is Mix; Rust is for engine primitives.

## How a change lands

1. Open an issue first for anything bigger than a fix, so direction is
   settled before code is written. This applies to AI agents too.
2. Branch from `main`: `fix/…`, `feat/…`, `docs/…`, `infra/…`. One
   checkpoint per PR (one app, widget or engine change). Small PRs.
3. Run the gate: `mix tools/ci.mix all` (or the stages you touched), plus a
   smoke run of what you changed.
4. Open a PR with the template filled in: risk tier, how tested (run /
   observed / not tested), how to undo, author and reviewers.
5. CI (`ci-ok`, `security-ok`) must pass. A maintainer who is not the
   author approves; Tier 2 paths also need the code owner (`REVIEW.md`).
6. Squash-merge, with a PR title that reads as a commit message.

`main` is protected: no direct pushes, no force pushes, no bypass for anyone.

## Agents

Agent work is contribution work. Agents push and open PRs under the
`mixviu-agent` GitHub App, never as a maintainer. Commit messages carry a
`Co-Authored-By:` trailer naming the model. Agents read review findings and
decide which are valid before a human sees the PR.

## Mix and Rust

- Scripts, tools and automation are Mix (`.mix`). No Python or shell
  scripts in the repository.
- `mix lint` diagnostics may only go down: `tools/ci/mix-lint-baseline.json`
  is a ratchet. A PR that fixes lint regenerates it with
  `mix tools/ci.mix mix-lint-baseline`.
- Build each engine with its own `-p` (`cargo build -p mix`): a
  workspace-wide build unifies features into the shell and is refused.
- New dependencies must pass `cargo deny` (permissive licences only, no
  unknown sources). Known advisories in `deny.toml` are debt that only
  shrinks.

## Licence

Contributions are dual-licensed MIT OR Apache-2.0, like the rest of the
repository.
