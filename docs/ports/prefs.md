# Preferences gold parity inventory

Round-five status: implementation and native drawer repairs are complete;
overall acceptance remains OPEN pending the heavy exact-source release gate
and the release-byte evidence set. The historical tables below retain failed
captures and are superseded by the later receipts, not deleted.

Latest code is `876f50a85bcb3f299558425969596dc9f4a63d20`.
Private run `prefs-parity-20261107/pair.6Uttb8` repeats 42/42 baseline
pages with full/header/sidebar/content SSIM and RMSE distortion zero,
40/40 structured functional/refusal cases, identical HELP (55 commands),
describe, actions, page tree, menus and four menu paths. Only the sequential
accepted store revision is normalised in structured state replies.

Round-five `pair.7xWNxT` closes all ten open drawer states with zero SSIM
and RMSE distortion in every measured region. The native panel separator
reserves one point before drawer padding; the panel background supplies the
scroll fade. Originals and the paired contact sheet were inspected. All five
owned processes were confirmed stopped. `pair.29fCaL` exercises catalogue
selection through native input and confirms both font-family settings writes
against real private settingsd before restoration. Its catalogue comparisons
and the release rerun remain part of the final acceptance record.

The native image chooser is pixel-identical in `pair.96mOt2`. Email's nine
additional mutation/refusal cases and its inline empty-save errors match in
`pair.cZAFcS`; password set, forget, forget for a removed account and removal
through the real private Email process match in `pair.u74dge`. Rebound menu
queries, macro lifecycle and action/completion event payloads match in
`pair.dPAr7S`, excluding process-unique IDs, timestamps and broker sequence
offsets. Existing live-axis and authority-recovery evidence is retained in
`pair.UWEu2H`; release bytes must repeat it before overall acceptance.

Focused CBC `20261107-073405-f2ebf754` passed all 17 widget/Prefs tests and
the native build; its formatting patch is applied. Heavy release acceptance
job `20261107-073545-70c5676f` is queued for the exact code above, running
`mix apps/prefs/gate.mix --release`. The gate retains a separately hashed
release executable for the evidence suite. No production install is involved.

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

## Prefs round four checkpoint: admission and exact refusal replies

Effective settings readback is now reduced to its authored representation
before native registry transaction admission. Defaults, nulls and live
observations are excluded; domain/account and cross-setting validation remain
in registry::apply. The Email rejection regression and effective-default
regression both pass without weakening tests.

Native paired functional requests now match **40/40 exactly**, including
all twelve previous refusal/code/wording differences. Evidence:
`prefs-parity-20261107/pair.1jvzHj/functional-differences.json` (empty),
gold/candidate-functional.json and closure.json (all five processes stopped).
Tested code `6e17e2af265ad6bf22248150a49fbc22d129214f`; candidate binary
SHA256 `acc0ce1ec168624c55a508fd247efaf7c660e7c070eae8ac3f63c6ec995562f0`.

The same source completed all 42 baseline pairs and 17 live variants in
`pair.ZQshvb`. Thirty-six baseline pages have full/content/header/sidebar
SSIM and RMSE distortion zero; originals and side-by-side figures inspected.
All 42 headers and sidebars match. HELP 55, describe/actions/pages, full
menus and four menu paths are structurally identical. Service/theme/size
recipe remains the pinned gold, private native Bus/settingsd/compositor,
1040×680, scale 1, Studio Dark/comfortable/CSD. ImageMagick 7.1.2-35
reports distortion: zero identical. Comparison-index.json retains all metrics.

Three further pages, Wallpaper, Email and Ced, are pixel-identical at
`8f163b122acbdddfa251e65bff91f8cf3cb72350`, artifact SHA256
`0f90aaa13f409948267a3326d6e98f51dc99bd0d69c36a34b78570060893a40d`.
`pair.2ABWvc/diff-<page>` retains original/region/difference figures and
acceptance.json records inspection. Wallpaper uses the native thumbnail
pool/cache/rounded mesh; an extra background beneath ready textures caused
the last corner residue. Email/Ced now use gold's 14.667 text default.
These three PNG pairs have identical hashes. Baseline acceptance is **39/42**.

Appearance, Panel and Dock have only scrollbar differences at their initial
positions; bottom captures exposed a reset button incorrectly placed in a
labelled card. The standalone reset composition is corrected at `6d4b6c7`;
new complete captures are in progress. Live light-mode icon and fractional
density frame rounding fixes are also awaiting paired acceptance.

Authority loss preserves gold's accepted role/revision/look and a valid set
returns gold's queued-write reply. The first probe caught a lost write on a
missing-service reply; retry handling was repaired. The next probe correctly
kept the write active, exposing a harness assumption that Refresh is always
enabled. The harness now tolerates gold's DISABLED while writing. Recovery
must still be verified. Both failed authority runs retained all captures and
confirmed all seven processes stopped. No deviation is proposed or approved.

CBC `20261107-053534-6bd3db64-a1` passed the complete scoped app gate at
`55e404cd2032da1dde26f038c04006b6cf350c24` with no worker diff, including
all Viu/engine tests, check, clippy/build and 64 native menu tests. Complete
saved log read. Repair jobs `20261107-053304-929180a3-a1`,
`20261107-053725-ddcd0572-a1` and `20261107-054123-1ae3f7d6-a1` passed
15 tests/build and returned formatting, now applied. Job
`20261107-054452-bc7007ff-a1` caught an initial nil-look access, corrected
without test changes; `20261107-054605-8c5bf1b0-a1` passed 15 tests/build
with no diff at `6d4b6c7bebaf95255f1c3daeea8bd5d05760eeb4`.
Full gate `20261107-054646-be78c922` is in progress at code head
`2dd337a9719e7bc53a4e7c00b84afb91a14a2ded`. Overall **OPEN**: final
baseline/live variant acceptance, authority recovery and the outstanding
full mutation/topic/native-input/populated-service matrix.

## Prefs round four: complete baseline acceptance, overall still open

All **42/42 baseline pages** now match gold pixel-for-pixel in one complete
native paired run, `prefs-parity-20261107/pair.nIbtod`. Full window, header,
sidebar and content SSIM and RMSE distortions are **zero on every page**.
`comparison-index.json`, `acceptance.json` and `diff-<page>` preserve the
region rectangles, original PNG hashes, raw metrics and difference figures.
Originals/side-by-side images were inspected through the closure runs; the
final reset-page figures were inspected again in `pair.KghwYB`.

Tested source: `9353aaf74812d224eaea8dec5fb0071ba2131f05`.
Candidate binary SHA256:
`237b10a7b326769866426317e2f5f0f9d54257eed63ca663751ea810000e26ef`.
Gold revision remains `8779f33284b7d3f1178c3d57a699efb9b0867b22`, binary
SHA256 `18590ca486b4b417b9de5a762f2dcf31b6b13d2351bb3eddd46d66e783fe3c55`.
Both use the same private native Bus/settingsd/compositor, 1040×680 client,
scale 1, Studio Dark/comfortable/CSD. ImageMagick 7.1.2-35 reports distortion,
zero identical. Every HELP/describe/actions/pages/full-menu/four menu-path
comparison is exact. All seven owned processes, including both restarted
settingsd processes, are confirmed stopped in closure.json.

The functional matrix remains **40/40 exact**, repeated at
`49d9f6ae27bb0b1999c3ecac473d96960ff8faa5` in `pair.bISgS0` after fixing
toolbar accessible names to include native shortcut hints. The native search
click now focuses the same field and rectangle as gold. About and Keyboard
shortcuts PNGs are also exact (`pair.WUYkD6`). The original twelve refusal
differences are closed. Admission accepts effective defaults/nulls while
retaining the native registry's account/domain/cross-setting validation.

Fifteen live variants are exact in every region at the complete-run source;
their gold/candidate contact sheets and regional figures were inspected.
Native panel GUI rounding was then corrected from physical pixels to
egui's 1/32 logical-point rounding. Compact is exact in `pair.FGb10H`.
Spacious differs only at the Macros icon's 16×13-pixel anti-aliased edges
(whole SSIM distortion 0.0000928134); enlarged originals and native widget
bounds are retained. No geometry or other page region differs. Final live
acceptance and the retained-authority chrome capture are being rerun.

Both apps preserve the accepted role, stale flag and revision on local
settingsd loss, return the same valid queued-write reply, and settle it after
restart on the same PID. Pending controls now preview gold's queued values
and show its retained error notice. A shared-host defect reset window settings
on a lost session, hiding the client mark/captions; the host now retains its
accepted window settings. This repair awaits final paired verification.

Native input exposed a further search-result gap: gold presents matching
sections with live controls rather than a button list. Mix now authors grouped
results, rendered through reusable browser cards with page-scoped admission.
The native clear button is reused too. CBC's fifteen focused tests/build pass;
rendered-search and in-place edit verification are in progress.

Full exact-source scoped app gates passed with no worker diff at
`55e404c`, `2dd337a`, `9353aaf` and `49d9f6a` (jobs
`20261107-053534-6bd3db64-a1`, `20261107-054646-be78c922-a1`,
`20261107-055202-a9165bdb-a1`, `20261107-055711-a793ea55-a1`). Complete
saved logs were read. Subsequent repair jobs `20261107-055936-61cb9c95-a1`,
`20261107-060927-c44a0f83-a1`, `20261107-061124-012794a9-a1` and
`20261107-061354-a8769509-a1` passed fifteen tests/build and returned
formatting, now applied at code head
`ff82bf2165570dc896b08629aa1d5650a4bd23a4`.

Overall remains **OPEN**. Beyond final live/authority verification and a fresh
exact-source full gate, remaining acceptance includes native search edits,
drawers/menus/keyboard callbacks, populated Software/Email, macro mutations,
topics/properties and CLI/single-instance behaviour. Baseline page acceptance
does not claim those capabilities. No Fable deviation is proposed.

## Prefs round four: live response accepted, native interactions progressing

All **17/17 live Prefs variants** are accepted at tested source
`d23465bbee2436cb538f116f0e146f2601cc8dc3`, candidate SHA256
`fc09a098797c50a5d28c3c68f10419b500a8ee81f5b2b1f2c7d7155d2a88d8ec`.
Pinned gold revision/hash and the isolated 1040×680, scale-1 native recipe
remain as above. `prefs-parity-20261107/pair.UWEu2H/live-acceptance.json`
records both source/binary identities, accepted revisions, unchanged PIDs,
captures, regional figures, raw metrics, tool version and all seven stopped
processes. The invalid tune request returns gold's exact validation refusal.

Sixteen variants are pixel-identical in all regions. Spacious has only the
inspected 16×13 Macros-icon anti-aliasing residue: full SSIM distortion
0.0000928134, RMSE 0.00306816; header (0,0,1040,46) SSIM 0.00137202,
RMSE 0.0117965; sidebar/content both zero. Native bounds differ by 1/32
logical point. The parity method explicitly permits anti-aliasing/hinting;
this is accepted on inspection, without an arbitrary metric threshold.
Compact's correct header crop is (0,0,1040,33); all its metrics are zero.

**Settings authority absence and recovery are pixel-identical**, including
header, sidebar and content. Both preserve the accepted role/stale/revision,
return the same valid queued-write reply, retain the error notice and settle
the write after restart without restarting Prefs. Both original pairs and
the new `acceptance-diff-authority-*` figures were inspected. The lost-session
frame-retention repair is verified. No Fable deviation is proposed.

Native search clicks now focus gold's field and editing the live Compact
result settles in settingsd while the open page stays unchanged
(`pair.zhBDtX`). A candidate-only debug-build “Unaligned” annotation was
identified; normal native clients now suppress egui's development overlay,
with geometry preserved. A fresh native interaction run is pending.

All four native menu popups and the new-account form are now pixel-identical
in `pair.eVIWoy`, tested `82a1c159f26b570cfc0db4a234c3f9a3ffbc85fc`, binary
SHA256 `3fc976e5f76cf4d025e8f45993b1c685df6a7385d1e4940924180768aca892f7`.
The original/side-by-side pairs were inspected. Live shortcut bindings use
the native conflict resolver and menu formatter. Populated Software's five
columns/release notes and Email's account pane now render through reusable
widgets; their remaining viewport/separator differences are being closed.
Successful release arrays had exposed a Mix scalar-field assumption; it is
fixed. The private native release fixture is deterministic evidence, not a
claim that the real service was tested (no releases binary is installed).

A Mix launcher now provides page/row deep links, native Bus/service/compositor
options and reuse of the existing window. The generic host keeps the app's
command namespace separate from an overridden service identity. Native first
and second launch checks remain pending. Macro mutation probes follow gold's
per-app/global namespace rules; moving to global removes it from Prefs' own
catalogue and belongs to Workshop thereafter.

CBC `20261107-062527-a1aac21b-a1` passed 15 tests/build with formatting
applied. `20261107-063252-b15af459-a1` caught a missing menu-helper argument;
`20261107-063430-56c4de16-a1` caught an egui API call; both were fixed without
test changes. `20261107-063654-21e3e59d-a1` and
`20261107-064033-96555226-a1` passed all **17** focused tests/build, with
formatting applied. Complete saved logs read. Repair job
`20261107-064358-480a8017` tests code `5329cebc40d9a4eca47427dd4793e604b2ca025f`.

Overall **OPEN**: fresh final baseline/gate, populated-service visuals and
mutations, drawers/pickers, remaining native callbacks/topics and launcher
acceptance. Baseline acceptance stays 42/42; the direct functional/refusal
matrix stays 40/40 (only sequential accepted revisions are normalised after
mutation witnesses). No inventory disagreement or proposed deviation blocks
the work.

## Prefs round five: release witness and native search correction

Exact source `f015520ceca5449a60b35c77284cca750c53993a` passed heavy CBC
`20261107-074849-cf5ada53-a1` on cbc2, command `mix apps/prefs/gate.mix
--release`, with no worker diff. The complete saved log was read. Release
SHA256 is `877737ce2fcaf015206f04df9ea59b311abcc5ea3e5ea6e95db757a0d4e21d73`.
Its nine private release runs and 91 regional comparison sets are retained
under `prefs-parity-20261107/round5-f015520-receipt` and their original run
directories. All 42 baseline pages are pixel-identical, as are ten drawers,
catalogues, the native chooser, populated Software/Email and dialogs.
Sixty-seven structured reply comparisons pass; native settingsd, Email and
the installed Mix Releases citizen are exercised in private roots. The
Releases witness verifies a private package, install, rollback refusal and
removal. This complements the fully logged deterministic service fixture.

That release is not accepted: the search capture exposed a real caret reset,
not merely a blink. Native `den` then `sity` produced gold `density` and
candidate `sityden` in `pair.qqHr5f`. The shared browser now retains pending
text until owner acknowledgement, consumes repeated values in order, and
discards a draft on explicit navigation. The private reset token is absent
from the public Prefs state. Native repaired witness `pair.MTCmDB` produces
`density` on both sides. CBC `20261107-081753-981fcde5` passed 18 focused
checks/build; its formatting patch is applied. The regression includes
delayed/partial replies, edit/undo cycles and numeric reset tokens.

Code `249f9a3483b13a9b9ac0d26b878942d5d78191b1` is queued for heavy
release gate `20261107-082023-f8cb0b05`. Acceptance remains OPEN until that
gate and the ten-group release-byte rerun, including native continued
typing, finish. Spacious retains only the previously inspected Macros icon
anti-aliasing edge. No deviation from gold or Fable decision is proposed.