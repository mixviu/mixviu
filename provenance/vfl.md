# VFL writer dependencies

The VFL container and XOR policy are original MixViu code. The initial design
is a lossless BGRA frame log with key, delta and repeat records. No code was
copied from another container or codec implementation.

`lz4_flex` 0.14.0, MIT, upstream https://github.com/pseitz/lz4_flex,
registry SHA256 `ecbdfe44b1bd960b68170b417450a628c43f7cf56bb3c5317e61cb230ee7f226`.
Inspected packaged Cargo.toml and block API. `build=false`; no dependencies
with the selected `std,safe-encode,safe-decode` features. Default features
are disabled, so no frame wrapper or twox-hash dependency is introduced.

`crc32fast` 1.5.4, MIT OR Apache-2.0, upstream
https://github.com/srijs/rust-crc32fast, registry SHA256
`5f97d3416d5337d15c4b0d78c8c91df922ca143f57ec20e2d1c188333047ef04`.
Already in the workspace lock. Its build script probes rustc version and
sets Rust cfg flags; it does not compile C or assembly. Intrinsics are Rust.
The only dependency is the existing `cfg-if` crate.

Both are tier-2 pure Rust dependencies under `tools/foreign_gate.mix`:
neither compiles or binds foreign code, so neither belongs in the exclusive
foreign allow/debt/admitted lists. Only graphics and headless tests gain
edges; the media constants crate has no dependencies. No desktop dependency
edge is added to Mix or mixd.
