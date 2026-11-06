# Preferences port inventory (round 2)

Oracle: MixOS `8779f3328`, apps/prefs preferences navigation/catalog/store/live,
appearance/pages/email/text/terminal, shell/actions and docs/prefs/README.md.
Per-app Rust added: 0. The inventory was committed before implementation.

| Original feature | Round 2 disposition |
|---|---|
| settings.describe catalogue, settings.get values/defaults | Ported: Mix registry browser/editor over unchanged settingsd wire, placeholder paths excluded |
| Typed enum/bool/number/text values, reset and validation | Ported: enum/checkbox or JSON text editor, metadata-based type/range admission and reset; service remains authority |
| Fenced writes, incarnation/revision, wrong-target retarget | Ported: settings.apply, exact decimal revision text, native-admitted operation ID and one read retarget |
| Stale mirror/unreachable authority write inhibition and visible failures | Ported admission/status checks; failed writes inhibit further writes until explicit refresh, no silent replay |
| Appearance, fonts, wallpaper, windows, accessibility, terminal and editor settings | Ported prefix/category navigation with generic editors; wallpaper image/colour couples slideshow/reset changes; specialised layouts deferred |
| Search/section/key selection, refresh and Bus commands | Ported Mix controls/handlers; described lists and Email-owned workflows remain read-only |
| Topic/reconnect reads, per-page retained problems and receipt recovery | Ported bounded five-second reads while idle; polling pauses during draft edits. Topic subscriptions, exact queued/coalesced retry and receipt recovery deferred |
| Live output/input/network/power/time/workspace/panel backend pages | Deferred backend-specific controls; registry metadata may remain inspectable |
| Appearance packages/previews/drafts, pinned source and exact theme inheritance | Deferred specialised composition; generic keys remain subject to service admission |
| Wallpaper library/folder scan/colour picker/slideshow/per-output drawers | Deferred specialised chooser; direct typed key edits subject to coupled-setting checks |
| Email account migration/password/reset workflows | Deferred: require Email-owned migration/secret workflows, not a generic key editor |
| Software catalogues/install/remove/release progress | Deferred: releases service workflows outside settings specimen |
| Keyboard layout add/search, font discovery, custom assets | Deferred specialised native catalogues/choosers |
| Original action menus/dialogs/macros/theme/single-instance CLI/wire compatibility | Deferred; generic viu native automation and new Mix commands are explicit |
| Settings-driven appearance/keymaps of Prefs itself | Deferred with the generic host's session design/keymap follower; editing the authority does not yet restyle this registry browser |

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
