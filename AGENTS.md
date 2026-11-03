# mixviu: repository guide

mixviu is the second build of MixOS: four engines, everything else Mix.
This repository holds the `viu` and `viud` engines (decision D1-a). `mix`,
`mixd`, the Bus and all Mix live in `markc/mixos`, with `bus` pinned by rev.

## What belongs here

Code, public docs and settled decisions only. Plans, briefs, journals, lab
captures and anything about the maintainer's machines live in a private
hub outside this repo; agents with access should read its `AGENTS.md` and
the plan there first, as the plan wins over every doc.

## Licence

`MIT OR Apache-2.0`, as in `markc/mixos`. Code moved from mixos brings its
attribution with it: the commit that adds it also updates `NOTICE`,
`LICENSE-MIT` and `REUSE.toml` (viud carries the y5 / Nourish notice).

Write everything here as public, because it will be:

- No keys, tokens, passwords, captures of real traffic or mail.
- No mesh host names, addresses or operator home paths. Use RFC 5737
  addresses, `example.*` domains and `/home/user`.

## Layout

Engine code goes at the top level as the plan's §2 layout describes (from
stage 3a: `viu`, `viud`). Docs filenames are `YYYY-MM-DD-lower-case-title.md`.

## Names (D6)

The binaries are `mix` + `mixd` and `viu` + `viud`. The clashes with the Arch
`viu` image viewer and Elixir's `mix` are accepted.

## Builds

Rust builds never run on a developer desktop; they go through the
maintainer's build workers. Gates and scripts are Mix, never Python or Bash.
