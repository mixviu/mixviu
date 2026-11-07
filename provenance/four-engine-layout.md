# Four-engine layout: moves and renames

Branch `cutover/four-engines`, from `lab/gold-parity` at
`4755e9e10bd2071cded1d18c7865c29d754a1009`. No source was imported; files
moved within this tree, so every earlier import manifest's `destination`
reads through the mapping below. Those manifests stay as recorded. Licences
and SPDX headers moved with their files unchanged.

## Paths (git mv, rename history kept)

| Old path | New path |
|---|---|
| `engines/compd/src` | `engines/viud/src` |
| `engines/compd/build.rs` | `engines/viud/build.rs` |
| `engines/compd/tests` | `engines/viud/tests` |
| `engines/compd/crates/*` | `engines/viud/crates/*` |
| `engines/compd/crates/ui` | `libraries/ui` |
| `engines/compd/i18n` | `engines/viud/i18n` |
| `engines/compd/layers.conf.mix` | `engines/viud/layers.conf.mix` |
| `engines/compd/scripts` | `engines/viud/scripts` |
| `engines/compd/units` | `engines/viud/units` |
| `engines/compd/README.md` | `engines/viud/README.md` |
| `engines/compd/CHANGELOG.md` | `engines/viud/CHANGELOG.md` |
| `engines/mix-shell` | `libraries/mix-shell` |

## Packages

| Old package | New package | Directory | Rust crate name |
|---|---|---|---|
| `system` (bins `mix`, `mix-standalone`) | `mix` (bin `mix`) | `engines/mix` | n/a |
| `mix` | `lib-mix` | `libraries/mix` | `mix` (lib name) |
| `viu` | `lib-viu` | `libraries/viu` | `viu` (lib name) |
| `viu-engine` (bin `viu`, lib `furniture`) | `viu` | `engines/viu` | unchanged |
| `compd` (bin `compd`) | removed; `viud` builds its sources | `engines/viud` | n/a |
| `viud` | `viud` | `engines/viud` | unchanged |

Dependents name the renamed libraries through `package =` aliases
(`mix = { package = "lib-mix", ... }`, `viu = { package = "lib-viu", ... }`),
so `use mix::` and `use viu::` are unchanged.

## Removed

- `engines/mix/src/standalone.rs` (the `mix-standalone` measurement target)
  and the argv[0] personality table in `engines/mix/src/main.rs`.
- `engines/compd/Cargo.toml` (the compd package) and the `inproc-ui` feature.
- `engines/compd/crates/scene-host` and `engines/compd/crates/overlays`
  (in-process egui scene host and capture/fps/notify overlays; nothing else
  used them), `engines/compd/src/scenes.rs` (the in-process scene host
  wiring) and the tests that only covered them (`tests/bus_wake.rs`,
  `tests/settings_test.mix`, `engines/mix/tests/personalities.rs`).

Their last source is at `4755e9e10bd2071cded1d18c7865c29d754a1009`; their
origin is recorded in `engine-split-comp-imports.json` and
`engine-split-mix-imports.json`.
