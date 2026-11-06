# Preferences gold parity inventory

Oracle: MixOS `8779f33284b7d3f1178c3d57a699efb9b0867b22`, including
`apps/prefs`, its preferences engine and the shared toolkit. Gold wins any
inventory disagreement. No overall feature, Bus or visual acceptance is
claimed. No deviation from gold has approval.

Application layout, state transitions and service policy live in Mix. Rust
additions are reusable Viu widgets and host facilities. Intra-app events and
jobs stay in process; native Bus calls cross process boundaries.

| Gold feature | Implementation and outstanding acceptance |
| --- | --- |
| Shared frame, menus, title, actions, captions and close | Shared gold frame adopted; exact-source frame checkpoint `48b5b430f56fd7c2101f199ef15cd2cd89cabdde`. Further generic caption/title geometry repairs are under test. Native menu model/input compatibility remains open. |
| 42 pages, 11 sidebar roots, children, drawers, search and legacy sections | Mix page descriptors, navigation and read-only page/row search implemented. All 42 pages mount in the embedded runtime test. Native default pages/actions metadata matches gold. Full navigation/input acceptance remains open. |
| Registry, defaults, typed validation, fenced writes | Native settings registry supplies defaults and validation. Mix queues/coalesces absolute values, retains failed writes and refreshes/replays after conflicts or uncertain results. Topic/reconnect reads replace idle polling. Recovery acceptance remains open. |
| Appearance drafts, conflict Keep/Revert and uncertain apply receipts | Mix appearance state machine and operation receipt lookup implemented; embedded conflict/receipt tests pass. Native mutation/refusal/restart comparison remains open. |
| Appearance mode previews, palettes, accent, typography, window composition and accessibility | Controls implemented through data-authored shared widgets. Current matched Appearance capture fails visual acceptance; geometry, bold fonts, previews, palettes and disabled extras remain open. |
| Wallpaper gallery, bounded folder scan, custom folders/images, native chooser, colours, slideshow and per-output drawer | Mix gallery state/scan jobs and coupled writes implemented; reusable native media widgets added. Native picker, gallery limits, output and visual acceptance remain open. |
| Terminal and Ced settings | Specialised Mix pages and typed native validation implemented. Passive numeric redraw regression repaired. Full native input/layout comparison remains open. |
| Email accounts/default, identities, migration, password, connect, reset and mail options | Mix workflows use fenced full-account writes and actual Email replies. Password stays out of public models/settingsd; deferred reply and validation/redaction tests pass. Native Email lifecycle, removal cleanup and UI acceptance remain open. |
| Software groups, listing/check, notes, install/update/remove/rollback and progress | Mix releases jobs, strict reply admission, stale-note rejection and lost-mutation reconciliation implemented. Native deterministic service cases and real-daemon acceptance remain open. |
| Keyboard layout/variant picker, font/icon/locale catalogues and asset selection | Worker jobs and searchable reusable pickers implemented; native fontdb catalogue reused. Catalogue parsing, native chooser and representative dataset acceptance remain open. |
| Backend pages and disabled controls | Gold's pending notices, hidden battery controls and missing-backend refusals are the target. These are not proposed future backend implementations. Native visibility/refusal coverage remains open. |
| Actions, dialogs, themes, macros and debug | Canonical descriptors and Mix actions/forms implemented; generic native macro editor/session and debug surface reused. Complete action/input/dialog/macro comparison remains open. |
| Full canonical Bus surface | 55 gold command descriptors preserved; canonical handlers, native drive, macros and debug registered. `app.describe`, `prefs.actions`, `prefs.pages`, default `prefs.appearance` and missing-releases `prefs.apps` match native gold. HELP currently exposes extra host commands and menu queries differ; structured mutation/refusal coverage remains open. |
| Single-instance CLI, service options and page/row deep links | Outstanding generic launch/host integration. Current lab launch is `viu run apps/prefs [INSTANCE]`. |
| Live Prefs response of Prefs itself | Shared host follows native session appearance without restart. Per-app complete axes/tune/app-override matrix and authority loss/recovery comparison remain open. |

## Corrections from pinned gold

The old registry specimen and its “ported” labels did not establish gold
parity. `prefs.search` returns `{query,hits}` with page/row targets and leaves
the window unchanged, including beneath a dialog. It does not return the
older section helper's `matches`. `prefs.select {app}` opens that software
group, selects the catalogue app and requests notes. Registry key selection
uses the separate `prefs.registry.select` extension.

Gold's typed `prefs.page.open`, `prefs.set` and `prefs.item` replies wrap the
result as `{executed,result}`. Gold's Appearance Import/Export extras and
several backend controls are disabled pending work; their disabled state,
notice and refusal belong in parity. Implementing future backends would
change the target.

## Evidence and remaining gate

The native oracle binary reports `prefs 0.1.2 (8779f33284b7, built
2026-11-04T19:10:04Z)`. Private capture run `pair.WrTxPr` uses isolated native
Bus/settingsd/headless compd, gold first, equal 1040×680 clients, scale 1,
Studio Dark, CSD, right captions, bar menus and row actions. All owned
processes were confirmed stopped. The candidate artifact comes from tested
source `f6370e6ce2260b87ec244177048823115b2f056d`.

ImageMagick 7.1.2-35 reports SSIM **distortion**, zero identical: Appearance
full `0.113842`, header `0.120927`, sidebar `0.0670151`, content `0.131337`.
Originals, montage and regional differences were inspected. The frame has a
content gap and displaced title/actions; page headings, preview/card sizes,
palette chips and disabled controls differ. These are unresolved differences,
not accepted anti-aliasing differences. Whole-app verdict remains **FAIL**.

Recent CBC evidence: `20261107-024331-ac8cf226-a1` passed the scoped
fmt/check/test/clippy/build sequence with a returned formatting diff;
`20261107-030733-1696f3b7-a1` passed six focused Prefs/deferred-request tests.
Worker formatting and lock patches were applied locally. The final
exact-source gate, native functional/Bus/Prefs matrix and further matched
captures remain required under `parity-method.md`.
