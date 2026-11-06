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

`menus.json` preserves the pinned native public menu readback (titles, rows,
separators, shortcuts and theme submenus). Mix supplies current enablement and
checks from its action registry; the reusable native toolkit draws it and
answers menu queries with the original command identities. The generic host
leaves explicitly declared HELP replies to the application.

Specialised Terminal, Ced, Email, Software and wallpaper composition and
reducers follow the same pinned engine's terminal, text, email, store,
appearance and wallpaper modules and native shell/view code. Pending controls
remain pending where gold has no backend or extra action.

Reusable Viu media controls use the existing toolkit native chooser and image
decoder. The font catalogue reuses gold's pure-Rust fontdb 0.23 discovery path
(including its default fontconfig parser); no new native C library or parallel
service is introduced. Keyboard layout labels and variants come from the
host's XKB rules data, parsed by Mix worker jobs. Runtime data, private paths,
captures and service credentials are not shipped in this bundle.
