# Preferences port inventory (round 2)

Oracle: MixOS `8779f3328`, apps/prefs preferences navigation/catalog/store/live,
appearance/pages/email/text/terminal, shell/actions and docs/prefs/README.md.
Per-app Rust added: 0. Inventory precedes implementation.

| Original feature | Round 2 disposition |
|---|---|
| settings.describe catalogue, settings.get values/defaults | Planned Mix registry browser/editor over unchanged settingsd wire |
| Typed enum/bool/number/text values, reset and validation | Planned Mix admission from described key metadata; service remains authority |
| Fenced writes, incarnation/revision, wrong-target retarget | Planned Mix settings.apply path (original uses apply, not unfenced set) |
| Stale mirror/unreachable authority write inhibition and visible failures | Planned status checks and explicit failed outcomes; no silent retry of uncertain writes |
| Appearance, fonts, wallpaper, windows, accessibility, terminal and editor settings | Planned prefix/category navigation with generic typed editors; specialised layouts deferred |
| Search/section/key selection, refresh and Bus commands | Planned Mix UI and semantic handlers |
| Topic/reconnect reads, per-page retained problems and receipt recovery | Bounded refresh port planned; exact queued/coalesced retry/receipt semantics deferred |
| Live output/input/network/power/time/workspace/panel backend pages | Deferred backend-specific controls; registry metadata may remain inspectable |
| Appearance packages/previews/drafts, pinned source and exact theme inheritance | Deferred specialised composition; generic keys remain subject to service admission |
| Wallpaper library/folder scan/colour picker/slideshow/per-output drawers | Deferred specialised chooser; direct typed key edits subject to coupled-setting checks |
| Email account migration/password/reset workflows | Deferred: require Email-owned migration/secret workflows, not a generic key editor |
| Software catalogues/install/remove/release progress | Deferred: releases service workflows outside settings specimen |
| Keyboard layout add/search, font discovery, custom assets | Deferred specialised native catalogues/choosers |
| Original action menus/dialogs/macros/theme/single-instance CLI/wire compatibility | Deferred; generic viu native automation and new Mix commands are explicit |

settingsd may be a mixd personality; no engine/interpreter migration is part of
this port. No settings registry or validation is copied into Rust.
