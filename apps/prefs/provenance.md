# Preferences parity catalogue

Reference: MixOS commit `8779f33284b7d3f1178c3d57a699efb9b0867b22`.
Original copyright: 2026 Mark Constable, MIT OR Apache-2.0.

`strings.ftl` preserves `apps/prefs/i18n/en/prefs.ftl` at that revision,
with its copyright and licence identified in the file header.
`pages.json` adapts the public page/control descriptors from
`apps/prefs/crates/preferences/src/pages.rs` and `pages/*.rs`, checked
against native `prefs.pages` and `prefs.page` readbacks. It contains static
labels, control metadata and missing-backend notices only; settings values,
readouts, gallery contents, process state and capture files are excluded.
Navigation and search behaviour in `navigation.mix` follows `nav.rs`,
`navigation.rs`, and the canonical dispatch in `engine.rs`.

The pinned canonical `prefs.search` returns `{query, hits}` with page/row
targets. The older section search helper and prose describing `matches`
are superseded by this handler. Search does not alter window state.
