# mixviu

A Mix-first desktop and application project, using native Rust foundations
where required and benchmark evidence to justify specific performance fallbacks.

Application workflows and compound-widget composition start as Mix scenes and
behaviours. Complexity alone does not decide their implementation language.
The aim is near-native median performance with a much shorter feature-edit loop.

## Source layout

| Directory | Holds |
|---|---|
| engines/ | Eventual mix, mixd, viu and viud hosts |
| libraries/ | Shared native facilities |
| apps/ | Mix-first shell/application bundles |
| tools/ | Small reproducible Mix checks |
| docs/ | Public architecture and development notes |
| provenance/ | Source revision, hashes and import decisions |

The repository owns the eventual four-engine stack and its Mix applications.
The Mix shell/server-host split is later work. The existing compositor can host
early Viu experiments; a full system rewrite is not a prerequisite.

## Current state

Workspace preparation only. The seed contains the existing scene parser,
binding expressions and strict-data dependency closure, original library tests,
and one Quoin volume specimen with its module-level helpers. No new widget
fallback, interpreter split, Viu runtime or replacement desktop exists yet.
Feature implementation is on hold.

The copied volume bundle requires a scene host and service facilities. It is
an input specimen, not a standalone runnable application or installation.

Read [architecture](docs/architecture.md), [import scope](provenance/README.md),
[checks](docs/development.md) and [repository rules](AGENTS.md).

## Licence

MIT OR Apache-2.0. See [LICENSE-MIT](LICENSE-MIT),
[LICENSE-APACHE](LICENSE-APACHE), [NOTICE](NOTICE) and [REUSE.toml](REUSE.toml).
Imported source retains its attribution.
