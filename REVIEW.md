# Reviewing mixviu changes

Every change to `main` is a pull request. Review asks two questions: **what
could go wrong with this change**, and **which check, automated or human, is
the right one to catch it**. Human attention is the scarce resource; nobody
should look at anything a check or an agent could have settled first.

## Risk tiers (by path, never by author)

| Tier | Paths | Who approves |
|---|---|---|
| 0 | Mix apps (`apps/`), docs, tests-only changes | one human reads the evidence |
| 1 | viu widgets/runtime (`libraries/viu`, `engines/viu`), `mix` builtins, viud paths other than input/focus, CI for Tier 0 paths | one human reads the evidence and the reviewer findings |
| 2 | `.github/` and `CODEOWNERS` paths: mixd and its network personalities, Bus, mesh and trust, viud input/focus, release and signing, `deny.toml` | the code owner reads the code; two agent families review; security pass |

A classifier or reviewer may raise a tier, never lower one. An agent
approval never fills the human slot, and any push after an approval needs a
fresh one.

## How each engine fails

- **mix**: a script that ran yesterday fails today (a builtin's contract
  changed); a shell-profile build picks up service builtins; the mix → mixd
  handoff loops or resolves to itself.
- **mixd**: a listener exposed beyond its intended address; one tenant's data
  reaching another (webd vhosts, maild mailboxes); dnsd misroutes or
  amplifies; maild backscatters or accepts unauthenticated relay.
- **viu**: a widget ignores a live Prefs or Bus setting; a Mix app's Bus
  replies change shape and break callers; row-heavy views lose their
  2,000-row budget.
- **viud**: pointer focus or hit-testing goes to the wrong surface; reserved
  space or layer ordering breaks; egui or the toolkit creeps back into its
  dependency tree (CI checks this).

## Evidence before review

The author (human or agent) checks what changed first: formatting, clippy
and the tests of the touched crates, plus a smoke run. The full
`mix tools/ci.mix all` runs once, in the merge gate (`gate-ok`); a PR that
changes `tools/ci.mix` also runs it once before review. The PR body says
what was run, what was observed, and what was not tested. "Done" without
evidence is sent back.

## Agent review rules

- Reviewers come from model families other than the author's.
- Each finding is tagged `important`, `nit` or `pre-existing`, and says
  whether it is blocking (yes/no). Every blocking finding is fixed or closed
  with a reason. Nits never force a new commit; skip more than five.
- At most three review rounds; after that the reviewer asks the author to
  rethink the approach instead of patching again.
- After the first full pass, re-review only the new diff. Never repost a
  finding. Flag a class of bug once so every instance is fixed in one commit.
- If the diff grows past three times its first-round size, stop and ask Mark.
- Run a simplify-the-diff pass before asking for review: each layer can add
  code as well as remove bugs.

## Fix the code before writing prose

When review keeps catching the same mistake, fix the code so the mistake is
hard to make, then add a check (lint, test, type), and only then a sentence
in a doc.

## Skip

`Cargo.lock`-only churn from dependabot (CI covers it), generated
`provenance/` records, and vendored upstream sources under `vendor/` unless
the PR changes them on purpose.
