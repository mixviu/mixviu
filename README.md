# mixviu

A Mix-first desktop and application project, using native Rust foundations
where required and benchmark evidence to justify specific performance fallbacks.

Application workflows and compound-widget composition start as Mix scenes and
behaviours. Complexity alone does not decide their implementation language.
The aim is near-native median performance with a much shorter feature-edit loop.

## Source layout

| Directory | Holds |
|---|---|
| engines/ | The four engines: mix, mixd, viu and viud |
| libraries/ | Shared native facilities |
| apps/ | Mix-first shell/application bundles |
| tools/ | Small reproducible Mix checks |
| docs/ | Public architecture and development notes |
| provenance/ | Source revision, hashes and import decisions |

The repository owns the eventual four-engine stack and its Mix applications.
The Mix shell/server-host split is later work. The existing compositor can host
early Viu experiments; a full system rewrite is not a prerequisite.

## Current state

The first client-runtime lab is active. `libraries/viu` hosts embedded Mix
behaviour on a worker, with bounded ordered input, completion wakes and scene
generation fences. `engines/viu` presents the volume scene as a native Wayland
client using an explicitly labelled safe audio fixture. Its original Mix
callbacks and model helpers remain the application policy.

This specimen is not a replacement desktop or production audio integration.
The compositor retains all global input/window authority. Geometry and theme
coverage are reduced; no full-product or physical-presentation parity follows.
See [runtime contract](docs/runtime-lab.md) for the exact boundary and checks.

Read [architecture](docs/architecture.md), [import scope](provenance/README.md),
[checks](docs/development.md) and [repository rules](AGENTS.md).

## Licence

MIT OR Apache-2.0. See [LICENSE-MIT](LICENSE-MIT),
[LICENSE-APACHE](LICENSE-APACHE), [NOTICE](NOTICE) and [REUSE.toml](REUSE.toml).
Imported source retains its attribution.
