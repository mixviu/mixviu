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
| Shared frame, menus, title, actions, captions and close | Shared gold frame adopted; exact-source frame checkpoint `48b5b430f56fd7c2101f199ef15cd2cd89cabdde`. Shared branch merged through `15956a6`; further paired frame acceptance is open. Native menu model and four open-path replies match gold. |
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
| Full canonical Bus surface | 55 gold command descriptors preserved; canonical handlers, native drive, macros and debug registered. HELP, `app.describe`, `prefs.actions`, `prefs.pages`, full menus and File/View/View → Theme/Help open-path replies match native gold. Host commands are hidden from the explicit HELP surface. Structured mutation/refusal coverage remains open. |
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

Gold's COSMIC page palette is retained across mode/scheme/accent changes until
density or roundness changes, while header, typography and control swatches
follow the live effective theme. The pinned shell and native light-mode pair
confirm this behaviour; the browser follows the same authored refresh key.

Round-three native run `pair.TD1EvM` completed all 42 page pairs, including
Terminal, and 11 accepted live variants per process with unchanged PIDs.
Every accepted variant waited for its settings receipt revision. All owned
processes stopped. The 42 `diff-<page>` figures and regional metrics are in
that private run's `comparison-index.json`. No page has visual acceptance.
Appearance SSIM distortion is full `0.0734909`, header `0.0883552`, sidebar
`0.00161027`, content `0.100063`; Terminal full is `0.0577293`. Later source
repairs page composition, native controls and shared framing; recapture is
required. The earlier figures below remain historical failed evidence.

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

## Prefs round three: complete native page pair and first visual acceptance

Tested source `8297a5c474679b4e4049dde11d2d4e4ed9a6f388`, candidate
SHA256 `dd34591e59f1c6b72bbe57b434020db9ae82c764618324f78c4cfe81a79f9827`.
Pinned gold is `8779f33284b7d3f1178c3d57a699efb9b0867b22`, installed
Prefs 0.1.2, binary SHA256
`18590ca486b4b417b9de5a762f2dcf31b6b13d2351bb3eddd46d66e783fe3c55`.
Private evidence `prefs-parity-20261107/pair.QUy9dU` contains all 42 paired
1040×680 client captures, 17 accepted live variants on each unchanged PID,
the invalid tune refusal and verified cleanup of all five owned processes.
Gold ran first against the same private settingsd/Bus/headless compositor;
scale 1, Studio Dark, comfortable, CSD/right captions/bar/row actions.

Native HELP (55 commands), app.describe, actions, pages, full menu and all
four tested open paths are structurally identical. The preceding successful
`pair.PC3WBd` also confirms all 42 page replies after normalising only the
sequential accepted store revision. Full mutation/topic/native-input and
authority restart acceptance are still open.

ImageMagick 7.1.2-35 uses SSIM distortion (zero identical). Every header and
sidebar region is pixel-identical. Original pairs, side-by-side figures and
the zero-difference contact sheet were inspected for the 20 accepted pages.
Ced's remaining 16×14 slider-edge residue is retained as open, with full
distortion 0.00000000184564; it is not silently passed by a threshold.
All remaining page figures are retained under `diff-<page>`.
| Page | Full SSIM distortion | Content SSIM distortion | Verdict |
| --- | ---: | ---: | --- |
| network | 0.000507405 | 0.000500839 | open |
| vpn | 0 | 0 | accepted |
| wifi | 0.00538869 | 0.00797843 | open |
| wired | 0 | 0 | accepted |
| bluetooth | 0 | 0 | accepted |
| accessibility | 0.000317218 | 0.000469669 | open |
| magnifier | 0.00523187 | 0.00774624 | open |
| osk | 0 | 0 | accepted |
| desktop | 0.00101478 | 0.00100165 | open |
| wallpaper | 0.0380345 | 0.0563134 | open |
| appearance | 0.000260795 | 0.00038613 | open |
| panel | 0.000124565 | 0.00018443 | open |
| panel_applets | 0 | 0 | accepted |
| dock | 0.000136277 | 0.00020177 | open |
| dock_applets | 0 | 0 | accepted |
| window_management | 0.000353627 | 0.000523576 | open |
| workspaces | 0.00880702 | 0.0130396 | open |
| displays | 0 | 0 | accepted |
| sound | 0 | 0 | accepted |
| sound_applications | 0 | 0 | accepted |
| sound_profiles | 0 | 0 | accepted |
| power | 0.0488153 | 0.0722754 | open |
| input | 0.000676554 | 0.000667799 | open |
| keyboard | 0.000498594 | 0.000738212 | open |
| keyboard_shortcuts | 0 | 0 | accepted |
| mouse | 0 | 0 | accepted |
| tablet | 0 | 0 | accepted |
| touchpad | 0 | 0 | accepted |
| applications | 0.00101478 | 0.00100165 | open |
| default_apps | 0 | 0 | accepted |
| startup_apps | 0 | 0 | accepted |
| x11 | 0 | 0 | accepted |
| term | 0.000197217 | 0.000291998 | open |
| email | 0.0155522 | 0.0230263 | open |
| ced | 0.00000000184564 | 0.00000000273264 | open |
| time | 0.000338261 | 0.000333884 | open |
| date_time | 0 | 0 | accepted |
| region | 0.00634753 | 0.00939807 | open |
| system | 0.000507405 | 0.000500839 | open |
| users | 0 | 0 | accepted |
| about | 0 | 0 | accepted |
| software | 0.031198 | 0.0461913 | open |

The next repair source `e476df87b84fc90c3a5233e84dc2c1af2c98b39a`
matches gold's authored theme-install trigger, live/empty Wi-Fi controls,
radio alignment, empty ordered-list separators, searchable append drawers,
child-link alignment and native Software/Email composition. CBC jobs
`20261107-044205-93245db2-a1`, `20261107-045001-4c23a9e7-a1` and
`20261107-045438-a29b5928-a1` passed all 15 scoped widget/Prefs tests and
the native build, returning formatting now applied. Complete saved logs
were read. The typo checkout queue job `20261107-044157-09b52b69` did not
run (rc75); it was requeued correctly, not counted as a failed test.
The full exact-source app gate remains required. No deviation is proposed.

## Prefs round three: thirty accepted pages, shared frame through 9a6cfd8

Merged the advanced shared frame at `8c52cd1`, including logic-only native
Bus polling, hosted chrome, single-instance activation and native key bindings.
Prefs retains owned menu IDs, its request queue, explicit HELP and optional
status. The merge's private toolkit-string call was corrected at `4d736a8`.
CBC `20261107-050018-d95872b5-a1` caught that compile issue;
`20261107-050310-4e784986-a1` then passed the complete scoped checks,
clippy, tests, native build and 64 menu tests. Its sole formatting patch is
applied. Complete saved logs were read. This is preparation, not the final
exact-source gate after the subsequent repairs.

The complete pair `prefs-parity-20261107/pair.nnxBpc` tests
`39c9a762f6dc5658958494dc5fa0ffdee1b202e7`, candidate binary SHA256
`dacad78fd9b9001f14175a5057521ff17132218ef4de70a340f7118963690460`.
Gold revision/binary, native 1040×680 dimensions, scale and services follow
the preceding record. All 42 baseline pairs and 17 live variants per app
completed; both PIDs stayed unchanged. All seven owned process IDs, including
the restarted private settingsd processes, are confirmed stopped.

Thirty baseline pages now have full/header/sidebar/content SSIM **and RMSE
distortion zero**. The newly accepted pages are Network & wireless, Wi-Fi,
Magnifier, Desktop, Input devices, Applications, Time & language, Region &
language, System & accounts and Software. Their originals, side-by-side
figures and zero-difference contact sheet were inspected; the earlier twenty
remain pixel-identical. `acceptance.json` names every accepted and open page.
All 42 header and sidebar regions are pixel-identical. HELP (55), describe,
actions, page tree, full menus and the four menu paths still match exactly.

The twelve remaining baseline pages are Accessibility, Wallpaper, Appearance,
Panel, Dock, Window management, Workspaces, Power, Keyboard, Term, Email and
Ced. Inspection locates most differences in the bottom scroll fade, caused
by the candidate's transparent UI stack background; the native panel frame
is now supplied. Wallpaper layout matches apart from thumbnail rendering:
the candidate now uses the native thumbnail pool, cache sizes and rounded
texture mesh. Ced retains the tiny slider-edge residue. These fixes require
new captures; none is silently accepted by a threshold.

The paired native authority probe reveals gold's precise behaviour: local
settingsd absence retains the accepted role/revision/stale state and look,
reports the missing service, and can queue a valid control write. A transport
failure is distinct from a mirror reporting authority_unreachable. The
candidate's stale/error state and validation of effective defaults differed;
repairs now check only proposed values through a reusable native registry
binding. A regression test includes effective defaults and nulls. Recovery
and queued-write parity still need native verification.

The independent native functional pair `pair.BI8UOc` matched 28/40 direct
requests exactly, including macro queries/refusals and debug. Twelve typed
refusal/code/wording differences were recorded and corrected from gold;
About and shortcut dialogs now use the existing native toolkit dialog and
gold strings. Full mutation, topic, native-input, populated Software/Email,
drawer and variant visual acceptance remain open. No deviation is proposed.

CBC `20261107-045438-a29b5928-a1`, `20261107-051345-3457782c-a1` and
`20261107-052000-ee775a89-a1` passed all 15 scoped widget/Prefs tests and
build, with formatting applied. `20261107-050905-9b06833f-a1` caught a
gallery popup call error; `20261107-051013-204dc482-a1` passed at the tested
capture source with no worker diff. New exact-source repair job
`20261107-052621-6dd8b9e9` is pending. Overall acceptance remains **OPEN**.
