# mixviu

The second build of [MixOS](https://github.com/markc/mixos): four engines,
everything else Mix.

MixOS grew a Rust crate for nearly every app and service. mixviu rebuilds it
around four native binaries and moves the rest (apps, scenes, services,
glue) into Mix, the MixOS shell and scripting language. A change to an app's
behaviour should be a Mix edit, not a Rust rebuild.

## The four engines

| Binary | Role | Repository |
|---|---|---|
| `mix` | the shell: login shell, CLI, scripts, gates, Bus clients | [markc/mixos](https://github.com/markc/mixos) |
| `mixd` | the server host: `mix` plus inbound servers and the native service personalities | [markc/mixos](https://github.com/markc/mixos) |
| `viud` | the Wayland compositor only: KMS, seat, input, rendering, window policy | here |
| `viu` | the app and shell runtime: scene evaluator, widgets, Mix behaviour host | here |

`mix` never accepts inbound connections; that is what `mixd` is for. `viud`
does not link the Mix interpreter; app behaviour runs in `viu`. All four
speak the MixOS Bus, whose wire format is frozen. This repository pins the
Bus from `markc/mixos` by revision.

## Status

Early. The foundation work (baselines, captures of current behaviour to test
against) is under way and no engine code has landed yet. `viud` arrives
first, as the MixOS compositor (compd) moved and trimmed; `viu` follows,
built and proven on the panel and monitor apps before the rest are ported.

## Building

Rust, with gates and scripts in Mix, never Python or Bash. Repository rules
for contributors and agents are in [AGENTS.md](AGENTS.md).

## Licence

mixviu is dual-licensed under the **MIT licence** or the **Apache License,
Version 2.0**, at your option (`MIT OR Apache-2.0`):

- [LICENSE-MIT](LICENSE-MIT)
- [LICENSE-APACHE](LICENSE-APACHE)

Unless you state otherwise, any contribution you submit for inclusion is
dual-licensed the same way, without additional terms. [NOTICE](NOTICE) lists
incorporated and third-party code, and per-file licensing is recorded in
[REUSE.toml](REUSE.toml).
