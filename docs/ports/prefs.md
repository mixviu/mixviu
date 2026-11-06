# Preferences gold parity inventory

Oracle: MixOS `8779f3328`, apps/prefs preferences navigation/catalog/store/live,
appearance/pages/email/text/terminal, shell/actions and docs/prefs/README.md.
Per-app Rust added: 0. Layout and behaviour belong to Mix; the shared
`settings-browser` widget renders application-supplied page/card data.
No overall parity acceptance is claimed yet.

| Original feature | Current parity status |
|---|---|
| settings.describe catalogue, settings.get values/defaults | Ported: Mix registry browser/editor over unchanged settingsd wire, placeholder paths excluded |
| Typed enum/bool/number/text values, reset and validation | Ported: enum/checkbox or JSON text editor, metadata-based type/range admission and reset; service remains authority |
| Fenced writes, incarnation/revision, wrong-target retarget | Ported: settings.apply, exact decimal revision text, native-admitted operation ID and one read retarget |
| Stale mirror/unreachable authority write inhibition and visible failures | Ported admission/status checks; failed writes inhibit further writes until explicit refresh, no silent replay |
| Appearance, fonts, wallpaper, windows, accessibility, terminal and editor settings | Ported prefix/category navigation with generic editors; wallpaper image/colour couples slideshow/reset changes; specialised layouts deferred |
| Canonical search/section/software selection | Implemented against pinned handlers; runtime comparison pending. Search returns `{query,hits}` with page/row targets without changing the window, including under a dialog; `prefs.select {app}` opens its Software group and requests notes; section replies include category and UI. Registry key selection moved to `prefs.registry.select`. |
| Topic/reconnect reads, per-page retained problems and receipt recovery | Ported bounded five-second reads while idle; polling pauses during draft edits. Topic subscriptions, exact queued/coalesced retry and receipt recovery deferred |
| Live output/input/network/power/time/workspace/panel backend pages | Deferred backend-specific controls; registry metadata may remain inspectable |
| Appearance packages/previews/drafts, pinned source and exact theme inheritance | Deferred specialised composition; generic keys remain subject to service admission |
| Wallpaper library/folder scan/colour picker/slideshow/per-output drawers | Deferred specialised chooser; direct typed key edits subject to coupled-setting checks |
| Email account migration/password/reset workflows | Deferred: require Email-owned migration/secret workflows, not a generic key editor |
| Software catalogues/install/remove/release progress | Deferred: releases service workflows outside settings specimen |
| Keyboard layout add/search, font discovery, custom assets | Deferred specialised native catalogues/choosers |
| Original action menus/dialogs/macros/theme/single-instance CLI/wire compatibility | Deferred; generic viu native automation and new Mix commands are explicit |
| Settings-driven appearance/keymaps of Prefs itself | Deferred with the generic host's session design/keymap follower; editing the authority does not yet restyle this registry browser |

## Corrections from pinned gold (P2 resolved)

Gold is authoritative under the updated standing brief. Earlier "Ported"
labels describe the registry specimen, not accepted gold parity. Every
"Deferred" entry above is outstanding work, with no approved deviation.
In particular, the former search mutated the registry filter and replied
`{}`, and select took `{key}`. Those were incompatible with gold.
The pinned command handler in `preferences/src/nav.rs` returns
`{query,hits}`; older prose and `navigation::search` describe a section
helper, not that canonical command. Native oracle readbacks confirm this.

Gold exposes 42 pages in COSMIC order, starting at Desktop. Its pending
pages and disabled backend controls are part of the target too; they must
retain their notices and refusals. Page/control descriptors, labels,
navigation and read-only search now have Mix representations. A generic
registry editor cannot substitute for those pages or their specialised
workflows. `provenance.md` in the app records the source adaptation.

The shared frame is adopted and has an exact-source CBC checkpoint at
`48b5b430f56fd7c2101f199ef15cd2cd89cabdde`; its scoped checks/test/clippy/build
gate passed. Live Prefs appearance follows the shared host session. Native
per-app response, full input matrix, gold Bus surface and matched visual
acceptance are still outstanding; the older deferred self-restyling entry
above is superseded by frame adoption, not by a full acceptance claim.

settingsd may be a mixd personality; no engine/interpreter migration is part of
this port. No settings registry or validation is copied into Rust.

Launch: `viu run apps/prefs [INSTANCE]` (default MIXOS_INSTANCE or desktop).
The original's transactional wire is settings.apply; unfenced settings.set is
deliberately unused. Read/write work uses the generic host's owned jobs;
`prefs.info` reports busy state, role, stale status, fence and visible errors.
Apply preserves the fence captured before draft editing. Explicit refresh
discards the draft and recovers after an uncertain outcome without replaying it.
Generic editors do not imply specialised-page parity or service acceptance of
every described key; settingsd's typed refusal is displayed.
